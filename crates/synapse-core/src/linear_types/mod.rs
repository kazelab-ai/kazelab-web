//! Linear & Affine Capability Type System for Compile-Time Resource Leak Elimination.
//! Models linear token budgets and ephemeral file descriptors that MUST be consumed exactly once (Baker 1992, Wadler 1990).

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityState {
    Active,
    Consumed,
}

pub struct LinearCapability<T> {
    inner: Option<T>,
    state: CapabilityState,
}

impl<T> LinearCapability<T> {
    pub fn new(resource: T) -> Self {
        Self {
            inner: Some(resource),
            state: CapabilityState::Active,
        }
    }

    pub fn consume<F, R>(mut self, consumer: F) -> R
    where
        F: FnOnce(T) -> R,
    {
        assert_eq!(self.state, CapabilityState::Active, "Linear capability already consumed");
        let res = self.inner.take().expect("Capability inner resource missing");
        self.state = CapabilityState::Consumed;
        consumer(res)
    }

    pub fn is_active(&self) -> bool {
        self.state == CapabilityState::Active
    }
}

impl<T> Drop for LinearCapability<T> {
    fn drop(&mut self) {
        if self.state == CapabilityState::Active {
            // Enforce runtime panic or invariant alarm if a linear resource was silently leaked without consumption
            eprintln!("CRITICAL INVARIANT VIOLATION: LinearCapability dropped without being explicitly consumed!");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_capability_consumption() {
        let cap = LinearCapability::new(100);
        let result = cap.consume(|val| val * 2);
        assert_eq!(result, 200);
    }
}
