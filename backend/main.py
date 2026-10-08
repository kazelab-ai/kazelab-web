"""
KazeLab AI - SynapseFlow Core Engine (Startup Enterprise v3.0.0)
Autonomous Multi-Agent Cognitive Architecture Platform
Native Model Context Protocol (MCP) & Anthropic Claude 3.5 Sonnet Integration
"""

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from core.config import settings
from api.routes import router

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

# Mount modular router
app.include_router(router)

if __name__ == "__main__":
    import uvicorn
    uvicorn.run("main:app", host=settings.host, port=settings.port, reload=True)
