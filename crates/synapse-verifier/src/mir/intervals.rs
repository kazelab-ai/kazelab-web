//! Static Abstract Interpretation Framework for Constant Range and Value Interval Propagation.
//! Performs lattice-based interval analysis ([min, max]) over MIR variables to detect integer overflows.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValueInterval {
    pub min: i64,
    pub max: i64,
}

impl ValueInterval {
    pub fn new(min: i64, max: i64) -> Self {
        Self { min, max }
    }

    pub fn point(val: i64) -> Self {
        Self { min: val, max: val }
    }

    pub fn top() -> Self {
        Self {
            min: i64::MIN,
            max: i64::MAX,
        }
    }

    pub fn join(&self, other: &ValueInterval) -> ValueInterval {
        ValueInterval {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    pub fn meet(&self, other: &ValueInterval) -> Option<ValueInterval> {
        let new_min = self.min.max(other.min);
        let new_max = self.max.min(other.max);
        if new_min <= new_max {
            Some(ValueInterval {
                min: new_min,
                max: new_max,
            })
        } else {
            None
        }
    }

    pub fn add(&self, other: &ValueInterval) -> ValueInterval {
        ValueInterval {
            min: self.min.saturating_add(other.min),
            max: self.max.saturating_add(other.max),
        }
    }

    pub fn sub(&self, other: &ValueInterval) -> ValueInterval {
        ValueInterval {
            min: self.min.saturating_sub(other.max),
            max: self.max.saturating_sub(other.min),
        }
    }

    pub fn mul(&self, other: &ValueInterval) -> ValueInterval {
        let p1 = self.min.saturating_mul(other.min);
        let p2 = self.min.saturating_mul(other.max);
        let p3 = self.max.saturating_mul(other.min);
        let p4 = self.max.saturating_mul(other.max);

        let min_p = p1.min(p2).min(p3).min(p4);
        let max_p = p1.max(p2).max(p3).max(p4);

        ValueInterval {
            min: min_p,
            max: max_p,
        }
    }

    pub fn contains(&self, val: i64) -> bool {
        val >= self.min && val <= self.max
    }
}

pub struct IntervalAnalysisState {
    pub intervals: HashMap<u32, ValueInterval>,
}

impl IntervalAnalysisState {
    pub fn new() -> Self {
        Self {
            intervals: HashMap::new(),
        }
    }

    pub fn set_interval(&mut self, reg_id: u32, interval: ValueInterval) {
        self.intervals.insert(reg_id, interval);
    }

    pub fn get_interval(&self, reg_id: u32) -> ValueInterval {
        self.intervals
            .get(&reg_id)
            .copied()
            .unwrap_or_else(ValueInterval::top)
    }

    pub fn check_overflow(&self, reg_id: u32, min_bound: i64, max_bound: i64) -> bool {
        let iv = self.get_interval(reg_id);
        iv.min < min_bound || iv.max > max_bound
    }
}
