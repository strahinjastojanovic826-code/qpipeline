use crate::error::PipelineError;

/// Predstavlja jedno kvartarno stanje (2 bita = 4 stanja: Q0, Q1, Q2, Q3).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuatState {
    Q0 = 0b00,
    Q1 = 0b01,
    Q2 = 0b10,
    Q3 = 0b11,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct QuatChunk(pub u64);

impl From<u8> for QuatState {
    #[inline(always)]
    fn from(val: u8) -> Self {
        match val & 0b11 {
            0b00 => QuatState::Q0,
            0b01 => QuatState::Q1,
            0b10 => QuatState::Q2,
            _ => QuatState::Q3,
        }
    }
}

impl QuatState {
    /// Modular addition of two quaternary states (mod 4).
    #[inline(always)]
    pub const fn add(self, other: Self) -> Self {
        let res = ((self as u8) + (other as u8)) & 0b11;
        match res {
            0 => QuatState::Q0,
            1 => QuatState::Q1,
            2 => QuatState::Q2,
            _ => QuatState::Q3,
        }
    }

    /// Modular subtraction of two quaternary states (mod 4).
    #[inline(always)]
    pub const fn sub(self, other: Self) -> Self {
        let res = (((self as u8) + 4) - (other as u8)) & 0b11;
        match res {
            0 => QuatState::Q0,
            1 => QuatState::Q1,
            2 => QuatState::Q2,
            _ => QuatState::Q3,
        }
    }
}

impl QuatChunk {
    #[inline(always)]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// Bezbedno dohvata stanje na datom indeksu (0-31).
    ///
    /// # Examples
    /// ```
    /// use qpipeline::chunk::{QuatChunk, QuatState};
    /// let chunk = QuatChunk::new(0b1100);
    /// assert_eq!(chunk.get_state(0).unwrap(), QuatState::Q0);
    /// assert_eq!(chunk.get_state(1).unwrap(), QuatState::Q3);
    /// ```
    #[inline]
    pub fn get_state(&self, index: usize) -> Result<QuatState, PipelineError> {
        if index >= 32 {
            return Err(PipelineError::IndexOutOfBounds { index, max: 31 });
        }
        let shift = index * 2;
        let bits = ((self.0 >> shift) & 0b11) as u8;
        Ok(QuatState::from(bits))
    }

    /// Bezbedno postavlja stanje na datom indeksu (0-31).
    #[inline]
    pub fn set_state(&mut self, index: usize, state: QuatState) -> Result<(), PipelineError> {
        if index >= 32 {
            return Err(PipelineError::IndexOutOfBounds { index, max: 31 });
        }
        let shift = index * 2;
        let mask = !(0b11u64 << shift);
        let val = (state as u64) << shift;
        self.0 = (self.0 & mask) | val;
        Ok(())
    }

    /// Invertuje sva stanja u bloku unutar jedne CPU operacije.
    #[inline(always)]
    pub fn invert(self) -> Self {
        Self(!self.0)
    }

    /// Prebrojava koliko puta se određeno stanje pojavljuje (SWAR / Parallel Bit-Count).
    #[inline]
    pub fn count_state(&self, state: QuatState) -> usize {
        let s = state as u64;
        let low_bit = s & 1;
        let high_bit = (s >> 1) & 1;

        let even = self.0;
        let odd = self.0 >> 1;

        let match_low = if low_bit == 1 { even } else { !even };
        let match_high = if high_bit == 1 { odd } else { !odd };

        let mask = match_low & match_high & 0x5555_5555_5555_5555;
        mask.count_ones() as usize
    }

    /// Branchless zamena svih pojavljivanja jednog stanja drugim.
    #[inline(always)]
    pub fn replace_state_branchless(&mut self, from: QuatState, to: QuatState) {
        let f = from as u64;
        let t = to as u64;

        for i in 0..32 {
            let shift = i * 2;
            let current = (self.0 >> shift) & 0b11;
            let is_match = ((current ^ f) == 0) as u64;
            let mask = !(0b11u64 << shift);
            let new_val = (t * is_match + current * (1 - is_match)) << shift;
            self.0 = (self.0 & mask) | new_val;
        }
    }

    #[inline(always)]
    pub const fn raw(&self) -> u64 {
        self.0
    }
}

pub struct QuatChunkIter {
    chunk: QuatChunk,
    index: usize,
}

impl Iterator for QuatChunkIter {
    type Item = QuatState;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < 32 {
            let state = self.chunk.get_state(self.index).ok()?;
            self.index += 1;
            Some(state)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = 32 - self.index;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for QuatChunkIter {}

impl IntoIterator for QuatChunk {
    type Item = QuatState;
    type IntoIter = QuatChunkIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        QuatChunkIter { chunk: self, index: 0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct QuatDoubleChunk(pub u128);

impl QuatDoubleChunk {
    #[inline(always)]
    pub const fn new(raw: u128) -> Self {
        Self(raw)
    }

    /// Inverts all states inside the 128-bit block using a single CPU instruction.
    #[inline(always)]
    pub fn invert(self) -> Self {
        Self(!self.0)
    }

    #[inline(always)]
    pub const fn raw(&self) -> u128 {
        self.0
    }
}