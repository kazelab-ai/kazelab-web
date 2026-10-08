"""
eBPF-Inspired Dynamic Kernel & Process Observability Tracer.
Monitors syscall invocations, network socket writes, and process fork events in agent sandbox.
"""

from typing import List, Dict, Any
from pydantic import BaseModel
import time

class SyscallTraceEvent(BaseModel):
    timestamp_ns: int
    pid: int
    syscall_name: str
    args: List[str]
    return_code: int
    latency_ns: int

class EbpfObservabilityEngine:
    def __init__(self):
        self._events_ring: List[SyscallTraceEvent] = []

    def capture_event(self, pid: int, syscall: str, args: List[str], ret: int, lat_ns: int):
        event = SyscallTraceEvent(
            timestamp_ns=time.time_ns(),
            pid=pid,
            syscall_name=syscall,
            args=args,
            return_code=ret,
            latency_ns=lat_ns
        )
        self._events_ring.append(event)
        if len(self._events_ring) > 1000:
            self._events_ring.pop(0)

    def get_anomalies(self) -> List[Dict[str, Any]]:
        # Detect suspicious syscalls like ptrace or execve with shell
        anomalies = []
        for e in self._events_ring:
            if e.syscall_name in ["ptrace", "sys_ptrace"] or ("sh" in e.args and e.syscall_name == "execve"):
                anomalies.append({
                    "severity": "CRITICAL",
                    "event": e.dict(),
                    "reason": "Unauthorized process introspection or shell breakout attempt"
                })
        return anomalies

ebpf_tracer = EbpfObservabilityEngine()
