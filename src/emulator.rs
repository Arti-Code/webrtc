#![allow(unused)]

const AIN1: u8 = 0;
const AIN2: u8 = 1;
const PWMA: u8 = 2;
const BIN1: u8 = 3;
const BIN2: u8 = 4;
const STBY: u8 = 5;
const PWMB: u8 = 6;

pub struct Motors {
    // gpio: Gpio,
    pwma: u8,
    ain1: u8,
    ain2: u8,
    pwmb: u8,
    bin1: u8,
    bin2: u8,
    stby: u8,
}

impl Motors {

    pub fn new() -> Motors {
        //let gpio = Gpio::new().unwrap();
        let ain1 = AIN1;
        let ain2 = AIN2;
        let pwma = PWMA;
        let bin1 = BIN1;
        let bin2 = BIN2;
        let stby = STBY;
        let pwmb = PWMB;
        Motors { pwma: pwma, ain1: ain1, ain2: ain2, pwmb: pwmb, bin1: bin1, bin2: bin2, stby: stby }
    }

    pub fn prepare(&self) {
        println!("prepare");
    }

    pub fn stop(&self) {
        println!("[↓]: stop");
    }    

    pub fn back(&self) {
        println!("[↓]: back");
    }

    pub fn front(&self) {
        println!("[↓]: front");
    }

    pub fn right(&self) {
        println!("[↓]: right");
    }

    pub fn left(&mut self) {
        println!("[↓]: left");
    }

    pub fn finish(&mut self) {
        println!("finish");
    }


}