"""
FastAPI Router Definitions
Modular API Layer: Swarm, MCP Hub, Telemetry, and Leads.
"""

from fastapi import APIRouter, HTTPException, status, WebSocket, WebSocketDisconnect
from typing import List, Dict, Any
import asyncio
import json

from core.swarm import swarm_engine, SwarmTaskRequest, SwarmTaskResult
from mcp_hub.registry import mcp_hub, MCPServerStatus
from telemetry.metrics import collect_telemetry, SystemTelemetry
from core.leads import lead_service, WaitlistSubmission, ContactInquiry

router = APIRouter()

# --- Health & Telemetry ---
@router.get("/health", tags=["System Telemetry"])
async def get_health() -> Dict[str, Any]:
    return {
        "status": "healthy",
        "service": "KazeLab SynapseFlow Engine",
        "version": "3.0.0",
        "foundation_engine": "Anthropic Claude 3.5 Sonnet",
        "mcp_protocol": "MCP 1.1.0",
        "prompt_caching_active": True
    }

@router.get("/api/v1/telemetry", response_model=SystemTelemetry, tags=["System Telemetry"])
async def get_system_telemetry() -> SystemTelemetry:
    return collect_telemetry(additional_tasks=swarm_engine.total_tasks())

# --- Swarm Execution ---
@router.post("/api/v1/swarm/dispatch", response_model=SwarmTaskResult, status_code=status.HTTP_202_ACCEPTED, tags=["Swarm Orchestration"])
async def dispatch_swarm_task(request: SwarmTaskRequest) -> SwarmTaskResult:
    if not request.instruction.strip():
        raise HTTPException(status_code=400, detail="Instruction cannot be empty")
    return await swarm_engine.execute_task(request)

@router.get("/api/v1/swarm/task/{task_id}", response_model=SwarmTaskResult, tags=["Swarm Orchestration"])
async def get_swarm_task(task_id: str) -> SwarmTaskResult:
    res = swarm_engine.get_task(task_id)
    if not res:
        raise HTTPException(status_code=404, detail=f"Task {task_id} not found")
    return res

# --- MCP Toolchain Hub ---
@router.get("/api/v1/mcp/servers", response_model=List[MCPServerStatus], tags=["MCP Toolchain Hub"])
async def list_mcp_servers() -> List[MCPServerStatus]:
    return mcp_hub.list_servers()

@router.get("/api/v1/mcp/tools", tags=["MCP Toolchain Hub"])
async def list_mcp_tools() -> List[Dict[str, Any]]:
    return mcp_hub.list_all_tools()

@router.get("/api/v1/mcp/server/{server_id}", response_model=MCPServerStatus, tags=["MCP Toolchain Hub"])
async def get_mcp_server(server_id: str) -> MCPServerStatus:
    srv = mcp_hub.get_server(server_id)
    if not srv:
        raise HTTPException(status_code=404, detail=f"MCP Server {server_id} not found")
    return srv

# --- Benchmarks & Token Economics ---
from core.benchmarks import get_benchmark_tasks, calculate_token_economics, BenchmarkTask

@router.get("/api/v1/benchmarks/swe-bench", response_model=List[BenchmarkTask], tags=["Benchmarks & Evaluation"])
async def get_swe_benchmarks() -> List[BenchmarkTask]:
    return get_benchmark_tasks()

@router.get("/api/v1/economics/calculator", tags=["Economics & Token Caching"])
async def get_economics_calculation(loc: int = 50000, runs: int = 100) -> Dict[str, Any]:
    return calculate_token_economics(loc=loc, runs_per_month=runs)

# --- Leads & Waitlist ---
@router.post("/api/v1/waitlist/join", status_code=status.HTTP_201_CREATED, tags=["Growth & Inquiries"])
async def join_waitlist(submission: WaitlistSubmission) -> Dict[str, Any]:
    return lead_service.enroll_waitlist(submission)

@router.post("/api/v1/contact", status_code=status.HTTP_200_OK, tags=["Growth & Inquiries"])
async def submit_contact(contact: ContactInquiry) -> Dict[str, Any]:
    return lead_service.record_contact(contact)

# --- WebSocket Streaming Telemetry ---
@router.websocket("/ws/telemetry")
async def websocket_telemetry(websocket: WebSocket):
    await websocket.accept()
    try:
        while True:
            telemetry = collect_telemetry(additional_tasks=swarm_engine.total_tasks()).dict()
            await websocket.send_text(json.dumps(telemetry))
            await asyncio.sleep(2.0)
    except WebSocketDisconnect:
        pass
