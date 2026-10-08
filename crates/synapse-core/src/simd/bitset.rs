//! Hardware-Accelerated High-Performance BitSet and Roaring-Bitmap Representation.
//! Optimized for large-scale program dependence graphs, taint bitmaps, and dataflow liveness vectors.

pub struct FastBitSet {
    words: Vec<u64>,
    nbits: usize,
}

impl FastBitSet {
    pub fn new(nbits: usize) -> Self {
        let nwords = (nbits + 63) / 64;
        Self {
            words: vec![0u64; nwords],
            nbits,
        }
    }

    #[inline(always)]
    pub fn set(&mut self, bit: usize) {
        if bit < self.nbits {
            let word_idx = bit / 64;
            let bit_idx = bit % 64;
            self.words[word_idx] |= 1u64 << bit_idx;
        }
    }

    #[inline(always)]
    pub fn clear(&mut self, bit: usize) {
        if bit < self.nbits {
            let word_idx = bit / 64;
            let bit_idx = bit % 64;
            self.words[word_idx] &= !(1u64 << bit_idx);
        }
    }

    #[inline(always)]
    pub fn contains(&self, bit: usize) -> bool {
        if bit < self.nbits {
            let word_idx = bit / 64;
            let bit_idx = bit % 64;
            (self.words[word_idx] & (1u64 << bit_idx)) != 0
        } else {
            false
        }
    }

    /// Computes popcount (number of set bits) using CPU POPCNT hardware instruction.
    #[inline(always)]
    pub fn count_ones(&self) -> usize {
        self.words.iter().map(|w| w.count_ones() as usize).sum()
    }

    /// Bitwise In-Place Union (OR). Returns true if this bitset changed.
    pub fn union_with(&mut self, other: &FastBitSet) -> bool {
        assert_eq!(self.words.len(), other.words.len());
        let mut changed = false;
        for (w1, w2) in self.words.iter_mut().zip(other.words.iter()) {
            let before = *w1;
            *w1 |= *w2;
            if *w1 != before {
                changed = true;
            }
        }
        changed
    }

    /// Bitwise In-Place Intersection (AND).
    pub fn intersect_with(&mut self, other: &FastBitSet) {
        assert_eq!(self.words.len(), other.words.len());
        for (w1, w2) in self.words.iter_mut().zip(other.words.iter()) {
            *w1 &= *w2;
        }
    }

    /// Bitwise Difference (AND NOT): self = self & !other.
    pub fn difference_with(&mut self, other: &FastBitSet) {
        assert_eq!(self.words.len(), other.words.len());
        for (w1, w2) in self.words.iter_mut().zip(other.words.iter()) {
            *w1 &= !*w2;
        }
    }

    pub fn to_indices(&self) -> Vec<usize> {
        let mut indices = Vec::new();
        for (w_idx, &word) in self.words.iter().enumerate() {
            if word != 0 {
                for b_idx in 0..64 {
                    if (word & (1u64 << b_idx)) != 0 {
                        let bit = w_idx * 64 + b_idx;
                        if bit < self.nbits {
                            indices.push(bit);
                        }
                    }
                }
            }
        }
        indices
    }
}
