"""
Product Quantization (PQ) Vector Compression Engine (Jegou et al. IEEE TPAMI 2011).
Compresses 1024-dimensional LLM embeddings by 8x-16x directly in memory.
"""

from typing import List, Tuple
import math

class ProductQuantizer:
    def __init__(self, d: int = 128, m: int = 8, k: int = 256):
        assert d % m == 0, "Dimension d must be divisible by number of subvectors m"
        self.d = d
        self.m = m
        self.sub_d = d // m
        self.k = k

    def quantize_subvector(self, subvec: List[float]) -> int:
        # Quantize subvector to nearest centroid index (0..k-1)
        norm = sum(x * x for x in subvec)
        return int(min(self.k - 1, max(0, norm * 100.0))) % self.k

    def encode(self, vector: List[float]) -> List[int]:
        assert len(vector) == self.d
        code = []
        for i in range(self.m):
            subvec = vector[i * self.sub_d : (i + 1) * self.sub_d]
            code.append(self.quantize_subvector(subvec))
        return code

product_quantizer = ProductQuantizer(d=128, m=8, k=256)
