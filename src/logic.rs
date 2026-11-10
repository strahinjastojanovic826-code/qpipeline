use crate::chunk::QuatState;

/// Extended quaternary logical operators (Łukasiewicz & Post logic)
pub trait QuatLogicOps {
    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
    fn negate(self) -> Self;
}

impl QuatLogicOps for QuatState {
    #[inline(always)]
    fn min(self, other: Self) -> Self {
        if (self as u8) < (other as u8) { self } else { other }
    }

    #[inline(always)]
    fn max(self, other: Self) -> Self {
        if (self as u8) > (other as u8) { self } else { other }
    }

    #[inline(always)]
    fn negate(self) -> Self {
        match (3 - (self as u8)) & 0b11 {
            0 => QuatState::Q0,
            1 => QuatState::Q1,
            2 => QuatState::Q2,
            _ => QuatState::Q3,
        }
    }
}

/// 4x4 Truth Table for arbitrary 2-input quaternary operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuatTruthTable {
    pub table: [[QuatState; 4]; 4],
}

impl QuatTruthTable {
    pub const fn new(table: [[QuatState; 4]; 4]) -> Self {
        Self { table }
    }

    #[inline(always)]
    pub fn evaluate(&self, a: QuatState, b: QuatState) -> QuatState {
        self.table[a as usize][b as usize]
    }
}

/// 2D Matrix of Quaternary States
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuatMatrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<QuatState>,
}

impl QuatMatrix {
    pub fn new(rows: usize, cols: usize, default_value: QuatState) -> Self {
        Self {
            rows,
            cols,
            data: vec![default_value; rows * cols],
        }
    }

    #[inline(always)]
    pub fn get(&self, row: usize, col: usize) -> Option<QuatState> {
        if row < self.rows && col < self.cols {
            Some(self.data[row * self.cols + col])
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn set(&mut self, row: usize, col: usize, val: QuatState) -> bool {
        if row < self.rows && col < self.cols {
            self.data[row * self.cols + col] = val;
            true
        } else {
            false
        }
    }

    /// Transpose the quaternary matrix
    pub fn transpose(&self) -> Self {
        let mut transposed = Self::new(self.cols, self.rows, QuatState::Q0);
        for r in 0..self.rows {
            for c in 0..self.cols {
                transposed.set(c, r, self.get(r, c).unwrap());
            }
        }
        transposed
    }
}