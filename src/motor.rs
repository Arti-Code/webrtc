use rppal::gpio::{Gpio, OutputPin};

// const PWMA: u8 = 16;
// const AIN1: u8 = 21;
// const AIN2: u8 = 20;
// const PWMB: u8 = 26;
// const BIN1: u8 = 13;
// const BIN2: u8 = 19;
// const STBY: u8 = 6;

// const PWMA: u8 = 6;
// const AIN1: u8 = 19;
// const AIN2: u8 = 13;
// const PWMB: u8 = 16;
// const BIN1: u8 = 21;
// const BIN2: u8 = 20;
// const STBY: u8 = 26;

const PWMA: u8 = 6;
const AIN1: u8 = 13;
const AIN2: u8 = 19;
const PWMB: u8 = 12;
const BIN1: u8 = 16;
const BIN2: u8 = 20;
const STBY: u8 = 26;

pub struct Motors {
    // gpio: Gpio,
    pwma: OutputPin,
    ain1: OutputPin,
    ain2: OutputPin,
    pwmb: OutputPin,
    bin1: OutputPin,
    bin2: OutputPin,
    stby: OutputPin,
}

impl Motors {

    pub fn new() -> Motors {
        let gpio = Gpio::new().unwrap();
        let ain1 = gpio.get(AIN1).unwrap().into_output();
        let ain2 = gpio.get(AIN2).unwrap().into_output();
        let pwma = gpio.get(PWMA).unwrap().into_output();
        let bin1 = gpio.get(BIN1).unwrap().into_output();
        let bin2 = gpio.get(BIN2).unwrap().into_output();
        let stby = gpio.get(STBY).unwrap().into_output();
        let pwmb = gpio.get(PWMB).unwrap().into_output();
        Motors { pwma: pwma, ain1: ain1, ain2: ain2, pwmb: pwmb, bin1: bin1, bin2: bin2, stby: stby }
    }

    pub fn prepare(&mut self) {
        _ = self.pwma.set_pwm_frequency(50.0, 1.0);
        _ = self.pwmb.set_pwm_frequency(50.0, 1.0);
        self.ain1.set_low();
        self.ain2.set_low();
        self.bin1.set_low();
        self.bin2.set_low();
        self.stby.set_high();
    }

    pub fn stop(&mut self) {
        self.ain1.set_low();
        self.ain2.set_low();
        self.bin1.set_low();
        self.bin2.set_low();
    }    

    pub fn back(&mut self) {
        self.ain1.set_high();
        self.ain2.set_low();
        self.bin1.set_high();
        self.bin2.set_low();
    }

    pub fn front(&mut self) {
        self.ain1.set_low();
        self.ain2.set_high();
        self.bin1.set_low();
        self.bin2.set_high();
    }

    pub fn right(&mut self) {
        self.ain1.set_high();
        self.ain2.set_low();
        self.bin1.set_low();
        self.bin2.set_high();
    }

    pub fn left(&mut self) {
        self.ain1.set_low();
        self.ain2.set_high();
        self.bin1.set_high();
        self.bin2.set_low();
    }

    pub fn finish(&mut self) {
        _ = self.pwma.clear_pwm();
        _ = self.pwmb.clear_pwm();
        self.stby.set_low();
    }

}