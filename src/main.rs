mod convert;
mod date;
mod rand;
mod ntp4;

use core::time::Duration;
use std::time::Instant;

#[derive(Debug, Default)]
pub struct Device {
    data: [u8; 17],
}

impl Device {
    #[cfg(target_arch = "x86_64")]
    pub fn sync(&mut self) {
        use std::io::Write;
        let mut stdout = std::io::stdout();

        stdout.write_all(b"\r").unwrap();
        stdout.write_all(&self.data).unwrap();
        stdout.flush().unwrap();
    }

    #[cfg(target_arch = "riscv")]
    pub fn sync(&mut self) {
    }
}

fn main() {
    let mut device = Device::default();
    let mut time = date::Time::new("es.pool.ntp.org:123");
    time.set_offset(2);

    let mut last_reset = Instant::now();
    let mut now = Instant::now();

    #[cfg(target_arch = "x86_64")]
    ctrlc::set_handler(|| std::process::exit(0)).unwrap();

    time.sync(&mut device);
    
    // core::arch::x86_64::_rdtsc()

    loop {
        if now.elapsed() < Duration::from_millis(1) {
            core::hint::spin_loop();
            continue;
        }

        if last_reset.elapsed() > Duration::from_secs(60 * 60) {
            time.sync_simple();
            last_reset = Instant::now();
        }

        time.update(&mut device, now.elapsed());

        now = Instant::now();
    } 
}
