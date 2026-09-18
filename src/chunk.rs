#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuatState {
    Q0 = 0b00,
    Q1 = 0b01,
    Q2 = 0b10,
    Q3 = 0b11,
}

/// 64-bitni blok koji drži 32 2-bitna kvat stanja
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct QuatChunk(pub u64);

impl QuatChunk {
    #[inline(always)]
    pub fn new(raw: u64) -> Self {
        Self(raw)
    }

    #[inline(always)]
    pub fn get_state(&self, index: usize) -> QuatState {
        debug_assert!(index < 32, "Index out of range (0-31)");
        let shift = index * 2;
        let bits = ((self.0 >> shift) & 0b11) as u8;
        match bits {
            0b00 => QuatState::Q0,
            0b01 => QuatState::Q1,
            0b10 => QuatState::Q2,
            _ => QuatState::Q3,
        }
    }

    #[inline(always)]
    pub fn set_state(&mut self, index: usize, state: QuatState) {
        debug_assert!(index < 32, "Index out of range (0-31)");
        let shift = index * 2;
        let mask = !(0b11u64 << shift);
        let val = (state as u64) << shift;
        self.0 = (self.0 & mask) | val;
    }
}