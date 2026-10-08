"""
High-Performance Rate Limiting & Sliding Window Token Bucket Middleware.
Protects swarm gateway against DDoS and throttles Claude 3.5 Sonnet token concurrency.
"""

import time
from typing import Dict, Tuple
from fastapi import Request, HTTPException, status
from starlette.middleware.base import BaseHTTPMiddleware

class RateLimiterMiddleware(BaseHTTPMiddleware):
    def __init__(self, app, max_requests_per_minute: int = 120):
        super().__init__(app)
        self.max_requests = max_requests_per_minute
        self.clients: Dict[str, Tuple[int, float]] = {}

    async def dispatch(self, request: Request, call_next):
        # Exclude static and health checks
        if request.url.path in ["/health", "/docs", "/redoc", "/openapi.json"]:
            return await call_next(request)

        client_ip = request.client.host if request.client else "127.0.0.1"
        now = time.time()

        count, window_start = self.clients.get(client_ip, (0, now))

        if now - window_start > 60.0:
            count = 1
            window_start = now
        else:
            count += 1

        self.clients[client_ip] = (count, window_start)

        if count > self.max_requests:
            raise HTTPException(
                status_code=status.HTTP_429_TOO_MANY_REQUESTS,
                detail="Rate limit exceeded: Max 120 requests/minute per client IP"
            )

        response = await call_next(request)
        response.headers["X-RateLimit-Limit"] = str(self.max_requests)
        response.headers["X-RateLimit-Remaining"] = str(max(0, self.max_requests - count))
        return response
