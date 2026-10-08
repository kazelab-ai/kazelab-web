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

# Static Landing Page Mount
import os
from fastapi.responses import FileResponse

INDEX_PATH = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "index.html"))
LOGO_PATH = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "kazelab-logo.jpg"))

@app.get("/", include_in_schema=False)
async def serve_index():
    if os.path.exists(INDEX_PATH):
        return FileResponse(INDEX_PATH)
    return {"status": "ok", "service": "KazeLab AI SynapseFlow Engine"}

@app.get("/kazelab-logo.jpg", include_in_schema=False)
async def serve_logo():
    if os.path.exists(LOGO_PATH):
        return FileResponse(LOGO_PATH)
    return {"error": "Logo asset not found"}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run("main:app", host=settings.host, port=settings.port, reload=True)
