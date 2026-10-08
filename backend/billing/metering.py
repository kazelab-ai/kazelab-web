"""
Token Metering, Prompt Cache Savings Calculator & Quota Enforcement Engine
Based on official Anthropic Claude 3.5 Sonnet API Pricing Matrix.
"""

from typing import Dict, Any
from pydantic import BaseModel

class TokenUsageBreakdown(BaseModel):
    base_input_tokens: int
    cache_read_tokens: int
    cache_creation_tokens: int
    output_tokens: int
    cost_without_cache_usd: float
    actual_cost_usd: float
    dollars_saved_usd: float
    percentage_saved: float

class BillingService:
    # Anthropic Claude 3.5 Sonnet Standard Pricing ($ per million tokens)
    PRICE_INPUT_PER_MTOK: float = 3.00
    PRICE_CACHE_WRITE_PER_MTOK: float = 3.75
    PRICE_CACHE_READ_PER_MTOK: float = 0.30
    PRICE_OUTPUT_PER_MTOK: float = 15.00

    @classmethod
    def calculate_cost(
        cls,
        base_input: int,
        cached_read: int,
        cached_creation: int,
        output: int
    ) -> TokenUsageBreakdown:
        # Standard cost if all were fresh input
        total_input_tokens = base_input + cached_read + cached_creation
        cost_no_cache = (
            (total_input_tokens / 1_000_000.0) * cls.PRICE_INPUT_PER_MTOK
            + (output / 1_000_000.0) * cls.PRICE_OUTPUT_PER_MTOK
        )

        # Actual cost with Anthropic Prompt Caching discount
        actual_input_cost = (base_input / 1_000_000.0) * cls.PRICE_INPUT_PER_MTOK
        actual_cache_read_cost = (cached_read / 1_000_000.0) * cls.PRICE_CACHE_READ_PER_MTOK
        actual_cache_write_cost = (cached_creation / 1_000_000.0) * cls.PRICE_CACHE_WRITE_PER_MTOK
        actual_output_cost = (output / 1_000_000.0) * cls.PRICE_OUTPUT_PER_MTOK

        actual_total = actual_input_cost + actual_cache_read_cost + actual_cache_write_cost + actual_output_cost
        saved = max(0.0, cost_no_cache - actual_total)
        pct_saved = (saved / cost_no_cache * 100.0) if cost_no_cache > 0 else 0.0

        return TokenUsageBreakdown(
            base_input_tokens=base_input,
            cache_read_tokens=cached_read,
            cache_creation_tokens=cached_creation,
            output_tokens=output,
            cost_without_cache_usd=round(cost_no_cache, 4),
            actual_cost_usd=round(actual_total, 4),
            dollars_saved_usd=round(saved, 4),
            percentage_saved=round(pct_saved, 1)
        )

billing_engine = BillingService()
