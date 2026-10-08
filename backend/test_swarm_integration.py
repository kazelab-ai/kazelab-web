"""
SynapseFlow Swarm Orchestration End-to-End Integration Test Suite.
Validates end-to-end multi-agent execution, AST ingestion, Wasm sandboxing,
eBPF bytecode verification, IVF-PQ indexing, and Git unified patching.
"""

import pytest
import math
from wasm_engine import WasmSandboxedEngine, WasmTrapException, WasmFuelExhaustedException
from ebpf.verifier import EbpfBytecodeVerifier, BpfInsn, BpfVerificationException
from vector_rag.ivf_pq import IvfPqIndexer
from git_vcs.patcher import UnifiedDiffPatcher, PatchApplyError
from core.leads import lead_service
from core.swarm import swarm_engine, SwarmTaskRequest
from billing.metering import billing_engine


class TestWasmSandboxedEngine:
    def test_basic_arithmetic_ops(self):
        engine = WasmSandboxedEngine(initial_fuel=1000)
        # 10 + 20 * 2 = 50
        program = [
            ("i32.const", [10]),
            ("i32.const", [20]),
            ("i32.const", [2]),
            ("i32.mul", []),
            ("i32.add", []),
        ]
        result = engine.run_bytecode_program(program)
        assert result == 50
        assert engine.fuel < 1000

    def test_memory_load_store(self):
        engine = WasmSandboxedEngine(initial_fuel=1000)
        # Store 1337 at offset 64, load back
        program = [
            ("i32.const", [64]),
            ("i32.const", [1337]),
            ("i32.store", []),
            ("i32.const", [64]),
            ("i32.load", []),
        ]
        result = engine.run_bytecode_program(program)
        assert result == 1337

    def test_out_of_bounds_trap(self):
        engine = WasmSandboxedEngine(initial_fuel=1000)
        # Try writing past 2 * 65536 bytes
        program = [
            ("i32.const", [200000]),
            ("i32.const", [42]),
            ("i32.store", []),
        ]
        with pytest.raises(WasmTrapException):
            engine.run_bytecode_program(program)

    def test_fuel_exhaustion(self):
        engine = WasmSandboxedEngine(initial_fuel=3)
        program = [
            ("i32.const", [1]),
            ("i32.const", [2]),
            ("i32.const", [3]),
            ("i32.const", [4]),
        ]
        with pytest.raises(WasmFuelExhaustedException):
            engine.run_bytecode_program(program)


class TestEbpfVerifier:
    def test_safe_program_verification(self):
        verifier = EbpfBytecodeVerifier()
        program = [
            BpfInsn(opcode="MOV_IMM", dst_reg=0, src_reg=0, offset=0, imm=42),
            BpfInsn(opcode="EXIT", dst_reg=0, src_reg=0, offset=0, imm=0),
        ]
        assert verifier.verify_program(program) is True

    def test_uninitialized_return_register_fails(self):
        verifier = EbpfBytecodeVerifier()
        # Exit without setting R0
        program = [
            BpfInsn(opcode="MOV_IMM", dst_reg=2, src_reg=0, offset=0, imm=10),
            BpfInsn(opcode="EXIT", dst_reg=0, src_reg=0, offset=0, imm=0),
        ]
        with pytest.raises(BpfVerificationException, match="R0 must be initialized"):
            verifier.verify_program(program)

    def test_invalid_stack_write_fails(self):
        verifier = EbpfBytecodeVerifier()
        # Move R10 to R2, add out of bounds offset, write
        program = [
            BpfInsn(opcode="MOV_REG", dst_reg=2, src_reg=10, offset=0, imm=0),
            BpfInsn(opcode="ALU_ADD_IMM", dst_reg=2, src_reg=0, offset=0, imm=1000),
        ]
        with pytest.raises(BpfVerificationException, match="out of bounds"):
            verifier.verify_program(program)


class TestIvfPqIndexer:
    def test_clustering_and_adc_search(self):
        dim = 16
        indexer = IvfPqIndexer(dimension=dim, n_clusters=4, n_subvectors=2, codebook_size=8)
        
        # Insert vectors
        v1 = [1.0] * dim
        v2 = [-1.0] * dim
        v3 = [0.5] * dim
        indexer.add("vec_pos", v1)
        indexer.add("vec_neg", v2)
        indexer.add("vec_half", v3)

        results = indexer.search_adc(query_vector=[0.9] * dim, n_probe=4, top_k=2)
        assert len(results) > 0
        assert results[0][0] in ["vec_pos", "vec_half"]


class TestUnifiedDiffPatcher:
    def test_clean_hunk_application(self):
        original = "def foo():\n    return 42\n"
        diff_text = """--- a/foo.py
+++ b/foo.py
@@ -1,2 +1,2 @@
 def foo():
-    return 42
+    return 100
"""
        patches = UnifiedDiffPatcher.parse_patch(diff_text)
        assert len(patches) == 1
        patched = UnifiedDiffPatcher.apply_patch_to_file(original, patches[0])
        assert "return 100" in patched
        assert "return 42" not in patched

    def test_fuzz_tolerance_matching(self):
        original = "\n\ndef foo():\n    return 42\n"
        diff_text = """--- a/foo.py
+++ b/foo.py
@@ -1,2 +1,2 @@
 def foo():
-    return 42
+    return 1337
"""
        patches = UnifiedDiffPatcher.parse_patch(diff_text)
        patched = UnifiedDiffPatcher.apply_patch_to_file(original, patches[0], fuzz=3)
        assert "return 1337" in patched


class TestBillingAndOrchestration:
    def test_prompt_cache_discount_calculation(self):
        # 1,000 fresh input tokens, 9,000 cached read tokens, 0 creation, 2,000 output tokens
        breakdown = billing_engine.calculate_cost(
            base_input=1000,
            cached_read=9000,
            cached_creation=0,
            output=2000,
        )
        assert breakdown.actual_cost_usd > 0
        assert breakdown.dollars_saved_usd > 0
        assert breakdown.percentage_saved > 20.0

    @pytest.mark.asyncio
    async def test_swarm_engine_task_execution(self):
        req = SwarmTaskRequest(
            repository_url="https://github.com/kazelab-ai/synapse-core",
            instruction="Refactor lock-free ring buffer and prove memory invariants",
            target_language="Rust",
        )
        result = await swarm_engine.execute_task(req)
        assert result.status == "COMPLETED"
        assert len(result.steps_executed) >= 4
        roles = [step.agent_role.value for step in result.steps_executed]
        assert "Cognitive Architect" in roles
        assert "Systems Coder" in roles
        assert "Verification Engine" in roles
        assert result.verification_passed is True
