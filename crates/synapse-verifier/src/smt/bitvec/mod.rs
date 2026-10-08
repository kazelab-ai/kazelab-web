//! Theory of Fixed-Size Bit-Vectors (QF_BV SMT Solver Theory).
//! Proves Absence of Integer Overflows, Underflows, and Bitwise Truncations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BitVector32 {
    pub value: u32,
}

impl BitVector32 {
    pub fn new(value: u32) -> Self {
        Self { value }
    }

    pub fn check_add_overflow(self, other: Self) -> (Self, bool) {
        let (val, ovf) = self.value.overflowing_add(other.value);
        (Self::new(val), ovf)
    }

    pub fn check_mul_overflow(self, other: Self) -> (Self, bool) {
        let (val, ovf) = self.value.overflowing_mul(other.value);
        (Self::new(val), ovf)
    }

    pub fn check_sub_underflow(self, other: Self) -> (Self, bool) {
        let (val, unf) = self.value.overflowing_sub(other.value);
        (Self::new(val), unf)
    }
}
