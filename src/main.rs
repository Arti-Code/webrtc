//mod motor;
mod emulator;
mod utils;

use clap::Parser;
use anyhow::Result;
use core::panic;
use webrtc::ice_transport::ice_credential_type::RTCIceCredentialType;
use std::sync::Arc;
use std::time::Duration;
use std::env;
use std::thread;
use tokio::time::sleep;
use tokio::net::UdpSocket;
use serde::{Serialize, Deserialize};
use firebase_rs::*;
use base64::{Engine as _, engine::general_purpose};
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_connection_state::RTCIceConnectionState;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_local::{TrackLocal, TrackLocalWriter};
use webrtc::Error;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::media_engine::MIME_TYPE_VP8;
//use webrtc::api::media_engine::MIME_TYPE_H264;
//use crate::motor::*;
use crate::emulator::*;
use crate::utils::*;



const DB_REF: &str = "https://my-insta-chat-default-rtdb.europe-west1.firebasedatabase.app/";
//const DB_REF: &str = "https://my-killer-bot-default-rtdb.europe-west1.firebasedatabase.app";
//const DEVICE: &str = "-NrQ8zF-IXyRCityY0R2";


#[derive(Serialize, Deserialize, Debug)]
struct Answer {
  answer: String
}

#[derive(Serialize, Deserialize, Debug)]
struct Offer {
    offer: String
}

#[derive(Serialize, Deserialize, Debug)]
struct Negotiation {
  offer: String,
  answer: String,
}

impl Negotiation {

    pub fn empty() -> Self {
        Self {
            offer: String::from(""),
            answer: String::from(""),
        }
    }

}

#[derive(Parser, Debug)]
struct Arguments {
    #[arg(short, long, default_value_t=String::from("-NrQ8zF-IXyRCityY0R2"))]
    device: String,
    #[arg(short, long, default_value_t=5008)]
    port: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Device {
    name: String
}

impl Device {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string()
        }
    }
    fn get_name(&self) -> &str {
        &self.name
    }
}

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let args = Arguments::parse();
    loop {
        match run(args.device.clone(), args.port) {
            Ok(restart) if restart => {
                log_this("[MAIN PROCESS]: RESTART");
                thread::sleep(Duration::from_secs(1));
                continue;
            },
            Ok(restart) if !restart => {break;},
            Ok(_) => {break;},
            Err(_) => {break;}
        }
    }
}

#[tokio::main]
async fn run(device: String, port: i32) -> Result<bool> {
    prog_intro().await;
    let device: Device = create_device(&device).await;
    let mut m = MediaEngine::default();
    m.register_default_codecs()?;
    let mut registry = Registry::new();
    registry = register_default_interceptors(registry, &mut m)?;
    let api = APIBuilder::new()
    .with_media_engine(m)
    .with_interceptor_registry(registry)
    .build();
    restart_info(&device, port).await; 
    let config = create_rtcconfig().await;
    let peer_connection = Arc::new(api.new_peer_connection(config).await?);
    
    let listener = create_video_listener(port).await;
    let video_track = create_video().await;
    let rtp_sender = peer_connection
        .add_track(Arc::clone(&video_track) as Arc<dyn TrackLocal + Send + Sync>).await?;
    
    let rtp_sender2 = rtp_sender.clone();
    tokio::spawn(async move {
        let mut rtcp_buf = vec![0u8; 4096];
        while let Ok((_, _)) = rtp_sender.read(&mut rtcp_buf).await {}
        Result::<()>::Ok(())
    });

    let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<()>(5);
    let done_tx1 = done_tx.clone();
    let (_data_tx, mut _data_rx) = tokio::sync::mpsc::channel::<()>(5);
    let (drive_tx, mut drive_rx) = tokio::sync::mpsc::channel::<&str>(10);
    
    // DataChannel events closures
    peer_connection.on_data_channel(Box::new(move |d: Arc<RTCDataChannel>| {
        let drive_tx1 = drive_tx.clone();
        let d_label = d.label().to_owned();
        Box::pin(async move {
            let _d2 = Arc::clone(&d);
            let d_label2 = d_label.clone();
            d.on_open(Box::new(move || {
                log_this(&format!("[DATACHANNEL] OPEN: {}", d_label2));
                Box::pin(async move {})
            }));
            
            d.on_message(Box::new(move |msg: DataChannelMessage| {
                let cmd = String::from_utf8(msg.data.to_vec()).unwrap();
                let drive_tx2 = drive_tx1.clone();
                //log_this("[↓]: {}", cmd);
                if cmd=="front" {
                    _ = drive_tx2.try_send("front");
                }
                else if cmd=="back" {
                    _ = drive_tx2.try_send("back");
                }
                else if cmd=="left" {
                    _ = drive_tx2.try_send("left");
                }
                else if cmd=="right" {
                    _ = drive_tx2.try_send("right");
                }
                else if cmd=="stop" {
                    _ = drive_tx2.try_send("stop");
                }
                Box::pin(async {})
            }));
        })
    }));

    // Receive and consume remote commands
    tokio::spawn(async move {
        let mut motors = Motors::new();
        motors.prepare();
        while let Some(cmd) = drive_rx.recv().await {
            if cmd=="front" {
                motors.front();
            } else if cmd=="stop" {
                motors.stop();
            } else if cmd=="back" {
                motors.back();
            } else if cmd=="left" {
                motors.left();
            } else if cmd=="right" {
                motors.right();
            }
        }
        motors.finish();
    });

    // ICE events closures
    peer_connection.on_ice_connection_state_change(Box::new(move |connection_state: RTCIceConnectionState| {
        log_this(&format!("[ICE CONNECTION]: {}", connection_state));
        if connection_state == RTCIceConnectionState::Disconnected {
            let _ = done_tx1.try_send(());
        }
        if connection_state == RTCIceConnectionState::Failed {
            let _ = done_tx1.try_send(());
        }
        Box::pin(async {})
    }));
    
    // PeerConnection state change event closure
    let done_tx2 = done_tx.clone();
    peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
        log_this(&format!("[PEER CONNECTION]: {}", s));
        if s == RTCPeerConnectionState::Failed {
            let _ = done_tx2.try_send(());
        }
        Box::pin(async {})
    }));
    
    //Negotiations 
    clean_negotiation(device.get_name()).await;
    let offer_encoded = wait_offer(device.get_name()).await;
    let offer_str = decode(&offer_encoded);
    let offer: RTCSessionDescription;
    match serde_json::from_str::<RTCSessionDescription>(&offer_str) {
        Ok(offer_json) => {
            offer = offer_json.to_owned();
            log_this("[OFFER]: RECEIVED");
            //log_this("{:?}", offer_json);
        },
        Err(_) => {
            log_this("[ERROR]: error during decoding and deserializing offer!");
            panic!();
        }
    }
    match peer_connection.set_remote_description(offer).await {
        Ok(_) => {
           log_this("[OFFER]: SET");
        },
        Err(_) => {
            log_this("error during setting remote description");
            panic!();
        },
    }
    let answer: RTCSessionDescription;
    match peer_connection.create_answer(None).await{
        Ok(rtcsd) => {
            log_this("[ANSWER]: CREATED");
            answer = rtcsd.to_owned();
        },
        Err(_) => {
            log_this("error during creating answer");
            panic!();
        }
    }

    let mut gather_complete = peer_connection.gathering_complete_promise().await;
    
    match peer_connection.set_local_description(answer).await {
        Ok(_) => {
            log_this("[ANSWER]: SET");
        },
        Err(_) => {
            log_this("error during setting answer as local description");
            panic!();
        },
    }
    
    let _ = gather_complete.recv().await;

    if let Some(local_desc) = peer_connection.local_description().await {
        send_answer(&local_desc, device.get_name()).await;
    } else {
        log_this("[ANSWER]: FAILED");
    }
    //lazy_static; 
    let done_tx3 = done_tx.clone();
    
    let l = Arc::clone(&listener);
    tokio::spawn(async move {
        let mut inbound_rtp_packet = vec![0u8; 4096];
        while let Ok((n, _)) = l.recv_from(&mut inbound_rtp_packet).await {
            if let Err(err) = video_track.write(&inbound_rtp_packet[..n]).await {
                if Error::ErrClosedPipe == err {
                } else {
                    log_this(&format!("[VIDEO]: error: {}", err));
                }
                let _ = done_tx3.try_send(());
                return;
            }
        }
    });
    let mut restart: bool=true;
    tokio::select! {
        _ = done_rx.recv() => {
            log_this("[PROCESS]: DONE");
        }
        _ = tokio::signal::ctrl_c() => {
            log_this("[PROCESS]: CTRL+C by user");
            restart=false;
            clean_negotiation(device.get_name()).await;
        }
    };

    peer_connection.remove_track(&rtp_sender2).await.unwrap();
    peer_connection.close().await?;
    log_this("[CONNECTION] CLOSE");
    Ok(restart)
}

async fn wait(seconds: u32) {
    sleep(Duration::from_secs(seconds as u64)).await;
}

async fn prog_intro() {
    let app_name = env!("CARGO_PKG_NAME").to_uppercase();
    let author = env!("CARGO_PKG_AUTHORS");
    let version = env!("CARGO_PKG_VERSION");
    println!("*** {} ***", app_name);
    println!("version {}", version);
    println!("made by {} ®2023-2024", author);
    wait(1).await;
}

async fn create_device(name: &str) -> Device {
    let device: Device = Device::new(name);
    device
}

async fn create_video_listener(port: i32) -> Arc<UdpSocket> {
    let sock = UdpSocket::bind(format!("0.0.0.0:{}", port))
        .await.unwrap();
    //let sock = UdpSocket::bind("0.0.0.0:5008").await.unwrap();
    return Arc::new(sock);
}

async fn create_rtcconfig() -> RTCConfiguration {
    RTCConfiguration {
        /* ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }], */

        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:fr-turn1.xirsys.com".to_owned()],
            username: "23Xgr3XVCOk2GqoZW5eWhbdXM1EfA8VcC6OVVacJSpFdoljTUOsTcgAoFUvfN4vcAAAAAGNFN29nd296ZHlrMg==".to_owned(),
            credential: "2ed490ce-4947-11ed-bd3d-0242ac120004".to_owned(),
            credential_type: RTCIceCredentialType::default(),
        }],
        ..Default::default()
    }
}

async fn create_video() -> Arc<TrackLocalStaticRTP> {
    Arc::new(TrackLocalStaticRTP::new(
        RTCRtpCodecCapability {
            mime_type: MIME_TYPE_VP8.to_owned(),
            ..Default::default()}, 
        "robot_cam".to_owned(), 
        "robot_stream".to_owned(), 
    ))
}

async fn restart_info(device: &Device, port: i32) {
    log_this(&format!("[DEVICE]: {}", device.get_name().to_uppercase()));
    log_this(&format!("[VIDEO PORT]: {}", port));
    wait(1).await;
}

async fn wait_offer(device: &str) -> String {
    let firebase = Firebase::new(DB_REF)
        .unwrap().at("devices").at(&device).at("offer");
    let mut offer_founded: bool=false;
    let mut offer_b64: String=String::new();
    log_this("[LISTENING...]");
    sleep(Duration::from_secs(1)).await;
    let mut t: f32 = 1.0;
    //let mut counter0: u32 = 0;
    //let mut counter1: u32 = 0;
    while !offer_founded {
        let encod = firebase.get::<String>().await;
        match encod  {
            Ok(v) if v != "" => {
                offer_b64 = v;
                offer_founded = true;
                let firebase2 = Firebase::new(DB_REF)
                    .unwrap().at("devices").at(&device);
                let clear_offer: Offer=Offer { offer: "".to_string() };
                firebase2.update(&clear_offer).await.unwrap();
            },
            Ok(_) => {
                sleep(Duration::from_secs(t.round() as u64)).await;
                if t < 15.0 {
                    t += 0.25;
                }
                //counter0 += 1;
                //if counter0 == u32::MAX {
                //    counter1 += 1;
                //}
            },
            Err(e) => {
                log_this(&format!("wait: Err: {}", e));
                sleep(Duration::from_secs(3)).await;
            }
        }
    }
    offer_b64
}

async fn clean_negotiation(device: &str) {
    let firebase = Firebase::new(DB_REF)
        .unwrap().at("devices").at(device);
    let clear_negotiation: Negotiation = Negotiation::empty();
    firebase.update(&clear_negotiation).await.unwrap(); 
    log_this("[NEGOTIATION]: CLEANED"); 
}

async fn send_answer(answer: &RTCSessionDescription, device: &str) {
    let json_str = serde_json::to_string(answer).unwrap();
    let b64 = encode(&json_str);
    let firebase = Firebase::new(DB_REF)
        .unwrap().at("devices").at(device);
    let ans: Answer=Answer { answer: b64 };
    firebase.update(&ans).await.unwrap();
    log_this("[ANSWER]: sended");
}

fn encode(b: &str) -> String {
    let encoded = general_purpose::STANDARD.encode(b);
    encoded
}

fn decode(s: &str) -> String {
    let b64 = general_purpose::STANDARD.decode(s).unwrap();
    let decoded = String::from_utf8(b64).unwrap();
    decoded
}