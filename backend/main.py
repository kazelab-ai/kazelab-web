"""
KazeLab AI - SynapseFlow Core Engine (Startup Enterprise v3.0.0)
Autonomous Multi-Agent Cognitive Architecture Platform
Native Model Context Protocol (MCP) & Anthropic Claude 3.5 Sonnet Integration
"""

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from middleware.rate_limiter import RateLimiterMiddleware
from core.config import settings
from api.routes import router
from api.v2.routes import router_v2

app = FastAPI(
    title=settings.app_name,
    description=(
        "Next-Generation Autonomous Multi-Agent Cognitive Architecture & Enterprise Intelligence Platform. "
        "Native Model Context Protocol (MCP 1.1) toolchain orchestration powered by Anthropic Claude 3.5 Sonnet."
    ),
    version=settings.app_version,
    docs_url="/docs",
    redoc_url="/redoc",
    contact={
        "name": "KazeLab AI Engineering",
        "url": settings.official_website,
        "email": settings.founder_email,
    },
    license_info={
        "name": "Apache 2.0",
        "url": "https://www.apache.org/licenses/LICENSE-2.0.html",
    }
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.allowed_origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.add_middleware(RateLimiterMiddleware, max_requests_per_minute=240)

# Mount modular router
app.include_router(router)
app.include_router(router_v2)

if __name__ == "__main__":
    import uvicorn
    uvicorn.run("main:app", host=settings.host, port=settings.port, reload=True)
