use cap_rand::RngCore;

pub struct ConstantRng {
    value: u8,
}

impl ConstantRng {
    pub fn new(value: u8) -> Self {
        Self { value }
    }
}

impl RngCore for ConstantRng {
    fn next_u32(&mut self) -> u32 {
        let b = self.value as u32;
        b | (b << 8) | (b << 16) | (b << 24)
    }

    fn next_u64(&mut self) -> u64 {
        let b = self.value as u64;
        b | (b << 8) | (b << 16) | (b << 24) |
        (b << 32) | (b << 40) | (b << 48) | (b << 56)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        dest.fill(self.value);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), cap_rand::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}
