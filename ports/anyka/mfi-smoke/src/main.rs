//! Exercise CatPlay's real I2C implementation without printing chip identity data.
use catplay_mfi::{MfiDevice, MfiDeviceI2C};

#[cfg(all(target_arch = "arm", target_env = "musl"))]
#[path = "../../stock-clock.rs"]
mod stock_clock;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
    let device = MfiDeviceI2C::new(0, 0x10)?;
    let certificate = device.read_certificate()?;
    if certificate.first() != Some(&0x30) {
        return Err("Certificate does not begin with an ASN.1 sequence".into());
    }
    println!("MFi certificate read: {} bytes", certificate.len());
    if std::env::args().any(|arg| arg == "--challenge") {
        // A fixed diagnostic challenge, containing no user or device identity.
        let response = device.generate_challenge_response(&[0x43; 32])?;
        if response.is_empty() {
            return Err("Empty authentication response".into());
        }
        println!("MFi challenge response: {} bytes", response.len());
    }
    println!("MFi diagnostic elapsed: {} ms", start.elapsed().as_millis());
    Ok(())
}
