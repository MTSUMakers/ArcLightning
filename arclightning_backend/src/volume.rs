extern crate rodio;
use std::io::BufReader;
use std::thread;
use std::time::Duration;

fn main() {
    let device = rodio::default_output_device().unwrap();

    let file = std::fs::File::open("src/ChillingMusic.wav").unwrap();
    let mut beep1 = rodio::play_once(&device, BufReader::new(file)).unwrap();
    beep1.set_volume(0.6);
    println!("Started beep1");
    thread::sleep(Duration::from_millis(5500));
    drop(beep1);
}