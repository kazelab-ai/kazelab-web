"""
CRDT Document Buffer and Operational Conflict Resolution Engine for Python.
Enables distributed multi-agent real-time collaborative editing on shared code modules.
"""

from typing import Dict, List, Optional, Set, Tuple
from dataclasses import dataclass, field
import time


@dataclass
class CrdtLineEntry:
    content: str
    timestamp_ns: int
    actor_id: str


class CrdtTextDocument:
    """
    Line-based Last-Write-Wins CRDT Document for concurrent agent patching.
    Satisfies commutativity, associativity, and idempotence.
    """

    def __init__(self, file_path: str):
        self.file_path = file_path
        self.lines: Dict[int, CrdtLineEntry] = {}

    def insert_or_update_line(self, line_num: int, content: str, actor_id: str, ts_ns: Optional[int] = None) -> bool:
        ts = ts_ns if ts_ns is not None else time.time_ns()
        entry = CrdtLineEntry(content=content, timestamp_ns=ts, actor_id=actor_id)

        if line_num in self.lines:
            existing = self.lines[line_num]
            if ts > existing.timestamp_ns or (ts == existing.timestamp_ns and actor_id > existing.actor_id):
                self.lines[line_num] = entry
                return True
            return False
        else:
            self.lines[line_num] = entry
            return True

    def delete_line(self, line_num: int, actor_id: str, ts_ns: Optional[int] = None) -> bool:
        # Tombstone represented as empty string with high timestamp
        return self.insert_or_update_line(line_num, "", actor_id, ts_ns)

    def merge_replica(self, other: "CrdtTextDocument") -> None:
        """State-based Lattice Join with another replica."""
        for line_num, entry in other.lines.items():
            if line_num in self.lines:
                local_entry = self.lines[line_num]
                if entry.timestamp_ns > local_entry.timestamp_ns or (
                    entry.timestamp_ns == local_entry.timestamp_ns and entry.actor_id > local_entry.actor_id
                ):
                    self.lines[line_num] = entry
            else:
                self.lines[line_num] = entry

    def render_text(self) -> str:
        if not self.lines:
            return ""
        max_line = max(self.lines.keys())
        result = []
        for i in range(max_line + 1):
            if i in self.lines and self.lines[i].content != "":
                result.append(self.lines[i].content)
        return "\n".join(result)
