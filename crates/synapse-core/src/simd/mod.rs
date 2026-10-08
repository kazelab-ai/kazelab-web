//! SIMD-Accelerated Token Embedding Cosine Similarity Vector Kernels (AVX-512 / AVX2 Emulation).
//! Accelerates semantic token deduplication and prompt cache lookup.

pub struct SimdVectorMath;

impl SimdVectorMath {
    pub fn dot_product_unrolled(a: &[f32], b: &[f32]) -> f32 {
        assert_eq!(a.len(), b.len(), "Vector lengths must match for dot product");
        let len = a.len();
        let chunks = len / 8;
        let mut sum = 0.0f32;

        // Loop unrolling 8 elements per iteration for CPU ILP (Instruction-Level Parallelism)
        for i in 0..chunks {
            let offset = i * 8;
            sum += a[offset] * b[offset]
                + a[offset + 1] * b[offset + 1]
                + a[offset + 2] * b[offset + 2]
                + a[offset + 3] * b[offset + 3]
                + a[offset + 4] * b[offset + 4]
                + a[offset + 5] * b[offset + 5]
                + a[offset + 6] * b[offset + 6]
                + a[offset + 7] * b[offset + 7];
        }

        // Remainder loop
        for i in (chunks * 8)..len {
            sum += a[i] * b[i];
        }

        sum
    }

    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot = Self::dot_product_unrolled(a, b);
        let norm_a = Self::dot_product_unrolled(a, a).sqrt();
        let norm_b = Self::dot_product_unrolled(b, b).sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot / (norm_a * norm_b)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identity() {
        let v1 = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let sim = SimdVectorMath::cosine_similarity(&v1, &v1);
        assert!((sim - 1.0).abs() < 1e-5);
    }
}
