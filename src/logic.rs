use crate::chunk::QuatState;
use crate::PipelineError;

pub trait QuatLogicOps {
    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
    fn negate(self) -> Self;

    /// Lukasiewicz implication in 4-valued logic (0..=3): min(3, 3 - a + b)
    fn imply_lukasiewicz(self, other: Self) -> Self;

    /// Godel implication: Q3 if self <= other, otherwise other
    fn imply_godel(self, other: Self) -> Self;

    /// Consensus operator: max of pairwise min across all states (generalized consensus)
    fn consensus(self, other: Self) -> Self;
}

impl QuatLogicOps for QuatState {
    #[inline(always)]
    fn min(self, other: Self) -> Self {
        if (self as u8) < (other as u8) {
            self
        } else {
            other
        }
    }

    #[inline(always)]
    fn max(self, other: Self) -> Self {
        if (self as u8) > (other as u8) {
            self
        } else {
            other
        }
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

    #[inline(always)]
    fn imply_lukasiewicz(self, other: Self) -> Self {
        let a = (self as u8) as i16;
        let b = (other as u8) as i16;
        let val = (3 - a + b).clamp(0, 3) as u8;
        QuatState::from(val)
    }

    #[inline(always)]
    fn imply_godel(self, other: Self) -> Self {
        if (self as u8) <= (other as u8) {
            QuatState::Q3
        } else {
            other
        }
    }

    #[inline(always)]
    fn consensus(self, other: Self) -> Self {
        self.min(other).max(self.negate().min(other.negate()))
    }
}

impl QuatTruthTable {
    pub const fn new(table: [[QuatState; 4]; 4]) -> Self {
        Self { table }
    }

    /// Builds a 4x4 quaternary truth table from a pure mapping function
    pub fn from_fn<F>(mut f: F) -> Self
    where
        F: FnMut(QuatState, QuatState) -> QuatState,
    {
        let states = [
            QuatState::Q0,
            QuatState::Q1,
            QuatState::Q2,
            QuatState::Q3,
        ];
        let mut table = [[QuatState::Q0; 4]; 4];

        for (r, &a) in states.iter().enumerate() {
            for (c, &b) in states.iter().enumerate() {
                table[r][c] = f(a, b);
            }
        }

        Self { table }
    }

    #[inline(always)]
    pub fn evaluate(&self, a: QuatState, b: QuatState) -> QuatState {
        self.table[a as usize][b as usize]
    }
}

/// 4x4 Truth Table for arbitrary 2-input quaternary operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuatTruthTable {
    pub table: [[QuatState; 4]; 4],
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

    /// Quaternary matrix multiplication using (max, min) semiring:
    /// C[i, k] = max_j( min(A[i, j], B[j, k]) )
    pub fn mul_min_max(&self, rhs: &Self) -> Result<Self, PipelineError> {
        if self.cols != rhs.rows {
            return Err(PipelineError::Custom(format!(
                "Matrix dimension mismatch for multiplication: {}x{} and {}x{}",
                self.rows, self.cols, rhs.rows, rhs.cols
            )));
        }

        let mut out = Self::new(self.rows, rhs.cols, QuatState::Q0);
        for r in 0..self.rows {
            for c in 0..rhs.cols {
                let mut acc = QuatState::Q0;
                for k in 0..self.cols {
                    let a = self.get(r, k).unwrap_or(QuatState::Q0);
                    let b = rhs.get(k, c).unwrap_or(QuatState::Q0);
                    let term = a.min(b);
                    acc = acc.max(term);
                }
                out.set(r, c, acc);
            }
        }
        Ok(out)
    }

    /// Quaternary matrix multiplication using modular arithmetic (mod 4):
    /// C[i, k] = sum_mod4_j( add_mod4(A[i, j], B[j, k]) )
    pub fn mul_modular(&self, rhs: &Self) -> Result<Self, PipelineError> {
        if self.cols != rhs.rows {
            return Err(PipelineError::Custom(format!(
                "Matrix dimension mismatch for modular multiplication: {}x{} and {}x{}",
                self.rows, self.cols, rhs.rows, rhs.cols
            )));
        }

        let mut out = Self::new(self.rows, rhs.cols, QuatState::Q0);
        for r in 0..self.rows {
            for c in 0..rhs.cols {
                let mut acc = QuatState::Q0;
                for k in 0..self.cols {
                    let a = self.get(r, k).unwrap_or(QuatState::Q0);
                    let b = rhs.get(k, c).unwrap_or(QuatState::Q0);
                    let prod = QuatState::add(a, b);
                    acc = QuatState::add(acc, prod);
                }
                out.set(r, c, acc);
            }
        }
        Ok(out)
    }

}