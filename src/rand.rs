// NOTE: The state word must be initialized to non-zero
pub struct XorShift {
    state: u32,
}

impl XorShift {
    pub fn new() -> Self {
        unsafe {
            let mut state = 0;
            core::arch::x86_64::_rdrand32_step(&mut state);
            Self { state }
        }
    }

    pub fn rand(&mut self) -> u32 {
        let mut x = self.state;

        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;

        self.state = x;
        x
    }

    pub fn rand_range(&mut self, range: std::ops::Range<u32>) -> u32 {
        let rand = self.rand() % (range.end - range.start);
        range.start + rand
    }
}
