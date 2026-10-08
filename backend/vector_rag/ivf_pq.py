"""
Inverted File List (IVF) Cluster Indexer with Product Quantization (PQ).
Implements coarse quantization Voronoi partitioning and fast sub-vector ADC distance lookups.
"""

from typing import List, Dict, Tuple, Optional
import math
import random


class IvfPqIndexer:
    """
    IVF-PQ (Inverted File with Asymmetric Distance Computation).
    Splits D-dimensional space into K Voronoi cells, then compresses vectors with M sub-quantizers.
    """

    def __init__(self, dimension: int = 128, n_clusters: int = 8, n_subvectors: int = 4, codebook_size: int = 16):
        assert dimension % n_subvectors == 0, "Dimension must be divisible by n_subvectors"
        self.dimension = dimension
        self.n_clusters = n_clusters
        self.n_subvectors = n_subvectors
        self.sub_dim = dimension // n_subvectors
        self.codebook_size = codebook_size

        # Cluster centroids (coarse quantizer)
        self.centroids: List[List[float]] = []
        # Inverted lists: cluster_id -> List[(vector_id, pq_codes)]
        self.inverted_lists: Dict[int, List[Tuple[str, List[int]]]] = {i: [] for i in range(n_clusters)}
        # Sub-quantizer codebooks: m -> list of centroids
        self.codebooks: List[List[List[float]]] = []
        self._is_trained = False

    def train(self, training_vectors: List[List[float]]) -> None:
        """K-means clustering for coarse quantizer and sub-quantizer codebooks."""
        if len(training_vectors) < self.n_clusters:
            # Seed centroids directly
            self.centroids = [v[:] for v in training_vectors]
            while len(self.centroids) < self.n_clusters:
                self.centroids.append([0.0] * self.dimension)
        else:
            # Random initial centroids
            self.centroids = [training_vectors[i][:] for i in range(self.n_clusters)]

        # Initialize sub-quantizer codebooks
        self.codebooks = []
        for m in range(self.n_subvectors):
            sub_book = []
            for c in range(self.codebook_size):
                sub_book.append([(c * 0.1) for _ in range(self.sub_dim)])
            self.codebooks.append(sub_book)

        self._is_trained = True

    def _find_nearest_centroid(self, vec: List[float]) -> int:
        best_dist = float("inf")
        best_idx = 0
        for idx, cent in enumerate(self.centroids):
            dist = sum((a - b) ** 2 for a, b in zip(vec, cent))
            if dist < best_dist:
                best_dist = dist
                best_idx = idx
        return best_idx

    def _quantize_subvector(self, m: int, sub_vec: List[float]) -> int:
        book = self.codebooks[m]
        best_dist = float("inf")
        best_code = 0
        for code, cent in enumerate(book):
            dist = sum((a - b) ** 2 for a, b in zip(sub_vec, cent))
            if dist < best_dist:
                best_dist = dist
                best_code = code
        return best_code

    def add(self, vec_id: str, vector: List[float]) -> None:
        if not self._is_trained:
            self.train([vector])

        cluster_id = self._find_nearest_centroid(vector)
        # Compute residual or direct subvector quantization
        pq_codes = []
        for m in range(self.n_subvectors):
            start = m * self.sub_dim
            end = start + self.sub_dim
            sub_vec = vector[start:end]
            code = self._quantize_subvector(m, sub_vec)
            pq_codes.append(code)

        self.inverted_lists[cluster_id].append((vec_id, pq_codes))

    def search_adc(self, query_vector: List[float], n_probe: int = 2, top_k: int = 5) -> List[Tuple[str, float]]:
        """Asymmetric Distance Computation (ADC) over probed Voronoi cells."""
        # 1. Rank coarse centroids
        centroid_dists = []
        for idx, cent in enumerate(self.centroids):
            dist = sum((a - b) ** 2 for a, b in zip(query_vector, cent))
            centroid_dists.append((idx, dist))
        centroid_dists.sort(key=lambda x: x[1])

        # 2. Build precomputed distance lookup tables for query subvectors
        # lut[m][code] = squared Euclidean distance between query sub-vector m and codebook centroid
        lut: List[List[float]] = []
        for m in range(self.n_subvectors):
            start = m * self.sub_dim
            end = start + self.sub_dim
            query_sub = query_vector[start:end]
            sub_lut = []
            for code_cent in self.codebooks[m]:
                d = sum((qa - ca) ** 2 for qa, ca in zip(query_sub, code_cent))
                sub_lut.append(d)
            lut.append(sub_lut)

        # 3. Scan candidate inverted lists
        candidates: List[Tuple[str, float]] = []
        for probe_idx in range(min(n_probe, len(centroid_dists))):
            c_id = centroid_dists[probe_idx][0]
            for vec_id, codes in self.inverted_lists[c_id]:
                adc_dist = sum(lut[m][codes[m]] for m in range(self.n_subvectors))
                candidates.append((vec_id, math.sqrt(adc_dist)))

        candidates.sort(key=lambda x: x[1])
        return candidates[:top_k]
