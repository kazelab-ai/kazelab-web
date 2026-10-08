import pytest
from httpx import AsyncClient, ASGITransport
from main import app

@pytest.mark.asyncio
async def test_health_check():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        res = await client.get("/health")
        assert res.status_code == 200
        data = res.json()
        assert data["status"] == "healthy"
        assert data["version"] == "3.0.0"
        assert "Claude 3.5 Sonnet" in data["foundation_engine"]

@pytest.mark.asyncio
async def test_root_index_serving():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        res = await client.get("/")
        assert res.status_code == 200
        assert "KazeLab AI" in res.text

@pytest.mark.asyncio
async def test_telemetry():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        res = await client.get("/api/v1/telemetry")
        assert res.status_code == 200
        data = res.json()
        assert data["swe_bench_verified"] == 94.8
        assert data["mcp_cluster_nodes_online"] == 8

@pytest.mark.asyncio
async def test_mcp_hub_registry():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        res = await client.get("/api/v1/mcp/servers")
        assert res.status_code == 200
        servers = res.json()
        assert len(servers) >= 4
        assert any(s["server_id"] == "mcp-ast-analyzer" for s in servers)

@pytest.mark.asyncio
async def test_swarm_dispatch_cycle():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        payload = {
            "repository_url": "https://github.com/kazelab/distributed-runtime",
            "instruction": "Refactor memory allocation to jemalloc with zero false sharing",
            "target_language": "C++20",
            "enable_prompt_caching": True
        }
        res = await client.post("/api/v1/swarm/dispatch", json=payload)
        assert res.status_code == 202
        data = res.json()
        assert data["status"] == "COMPLETED"
        assert data["tokens_saved_via_cache"] > 0
        assert len(data["steps_executed"]) == 4

        # Retrieve task
        task_id = data["task_id"]
        res_get = await client.get(f"/api/v1/swarm/task/{task_id}")
        assert res_get.status_code == 200
        assert res_get.json()["task_id"] == task_id

@pytest.mark.asyncio
async def test_lead_and_waitlist_pipeline():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        res = await client.post("/api/v1/waitlist/join", json={"email": "architect@silicon-valley-ai.org"})
        assert res.status_code == 201
        data = res.json()
        assert data["success"] is True
        assert data["queue_position"] >= 142

@pytest.mark.asyncio
async def test_mcp_tools_catalog():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        res = await client.get("/api/v1/mcp/tools")
        assert res.status_code == 200
        tools = res.json()
        assert len(tools) >= 10
        assert any(t["tool_name"] == "parse_ast" for t in tools)

@pytest.mark.asyncio
async def test_swe_bench_evaluation_registry():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        res = await client.get("/api/v1/benchmarks/swe-bench")
        assert res.status_code == 200
        tasks = res.json()
        assert len(tasks) == 3
        assert any(t["task_id"] == "SWE-DJANGO-1429" for t in tasks)
        assert any(t["task_id"] == "SWE-TOKIO-3821" for t in tasks)

@pytest.mark.asyncio
async def test_token_economics_calculator():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        res = await client.get("/api/v1/economics/calculator?loc=100000&runs=150")
        assert res.status_code == 200
        data = res.json()
        assert data["repository_loc"] == 100000
        assert data["savings_percentage"] > 80.0
        assert data["ttft_synapseflow_ms"] < data["ttft_standard_ms"]
