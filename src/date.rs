use core::time::Duration;

use crate::convert::Serialize;
use crate::rand::XorShift;

macro_rules! try_again {
    ($expr:expr, $tries:expr) => {{
        let mut idx = 0;
        loop {
            if idx == $tries {
                panic!(stringify!($expr));
            }

            match $expr {
                Ok(v) => break v,
                _ => std::thread::sleep(std::time::Duration::from_secs(5)),
            }

            idx += 1;
        }
    }};
}

/// Convert from ntp timestamp to `time` crate compatible nanosecond unix timestamp.
fn ntp_to_unix(mut secs: u32, mut frac: u32) -> i128 {
    secs -= 2208988800;
    frac = ((frac as u64 * 1000000) / (1u64 << 32)) as u32;
    (secs as u64 * 1000000 + frac as u64) as i128 * 1000
}

#[derive(Debug)]
pub struct Time {
    pub server: &'static str,
    date: time::OffsetDateTime,
}

impl Time {
    pub fn new(server: &'static str) -> Self {
        let ntp = try_again!(ntp::request(server), 4);
        let date = Self::date(ntp.transmit_time);

        Self { server, date }
    }

    pub fn set_offset(&mut self, hours: i8) {
        self.date = self
            .date
            .to_offset(time::UtcOffset::from_hms(hours, 0, 0).unwrap())
    }

    fn date(stamp: ntp::formats::timestamp::TimestampFormat) -> time::OffsetDateTime {
        let stamp = ntp_to_unix(stamp.sec, stamp.frac);
        time::OffsetDateTime::from_unix_timestamp_nanos(stamp).unwrap()
    }

    /// Periodically called to get the current time
    pub fn update(&mut self, device: &mut crate::Device, elapsed: Duration) {
        self.date += elapsed;

        let buf = &mut device.data;
        self.date.year().serialize(&mut buf[0..4]);
        self.date.month().serialize(&mut buf[4..6]);
        self.date.day().serialize(&mut buf[6..8]);
        self.date.hour().serialize(&mut buf[8..10]);
        self.date.minute().serialize(&mut buf[10..12]);
        self.date.second().serialize(&mut buf[12..14]);
        self.date.millisecond().serialize(&mut buf[14..17]);

        device.sync();
    }

    /// Sync with ntp server and show animation
    pub fn sync(&mut self, device: &mut crate::Device) {
        let ntp = try_again!(ntp::request(self.server), 4);
        let now = std::time::Instant::now();

        let indices = [0..4, 4..6, 6..8, 8..10, 10..12, 12..14, 14..17];
        let rng_range = [1..9999, 1..12, 1..30, 1..24, 1..60, 1..60, 1..999];
        let values = [
            self.date.year() as u64,
            self.date.month() as u64,
            self.date.day() as u64,
            self.date.hour() as u64,
            self.date.minute() as u64,
            self.date.millisecond() as u64,
        ];

        let mut rng = XorShift::new();
        let mut buf = [0u8; 17];

        for interval in [100, 90, 80, 70, 70, 60, 60, 50, 50, 40, 40, 30, 30, 20, 20] {
            let buf = &mut device.data;

            for (dev, num) in indices.iter().cloned().zip(rng_range.clone()) {
                rng.rand_range(num).serialize(&mut buf[dev]);
            }

            device.sync();

            std::thread::sleep(Duration::from_millis(interval));
        }

        for (idx, value) in (0..indices.len()).zip(values) {
            value.serialize(&mut buf[indices[idx].clone()]);

            for jdx in indices[idx].clone() {
                for (dev, num) in indices[idx + 1..].iter().cloned().zip(rng_range.clone()) {
                    rng.rand_range(num).serialize(&mut device.data[dev]);
                }

                device.data[jdx] = buf[jdx];
                device.sync();

                std::thread::sleep(Duration::from_millis(50));
            }
        }

        self.date = Self::date(ntp.transmit_time).to_offset(self.date.offset());
        self.update(device, now.elapsed());
    }

    /// Sync with ntp server
    pub fn sync_simple(&mut self) {
        let ntp = try_again!(ntp::request(self.server), 4);
        self.date = Self::date(ntp.transmit_time).to_offset(self.date.offset());
    }
}
