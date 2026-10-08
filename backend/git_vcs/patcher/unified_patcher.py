"""
Multi-File Unified Diff Parser and Fuzzy Patch Application Engine.
Applies contextual unified diffs with configurable fuzz tolerance and reverse rollback support.
"""

from typing import List, Dict, Optional, Tuple
from dataclasses import dataclass, field
import re


@dataclass
class PatchLine:
    line_type: str  # ' ', '+', '-'
    content: str


@dataclass
class PatchHunk:
    old_start: int
    old_length: int
    new_start: int
    new_length: int
    lines: List[PatchLine] = field(default_factory=list)


@dataclass
class FilePatch:
    old_filename: str
    new_filename: str
    hunks: List[PatchHunk] = field(default_factory=list)


class PatchApplyError(Exception):
    """Raised when a patch cannot be applied cleanly."""
    pass


class UnifiedDiffPatcher:
    """Parses standard unified diff format (git format-patch / diff -u) and applies to text buffers."""

    HUNK_HEADER_RE = re.compile(r"^@@\s+-(\d+)(?:,(\d+))?\s+\+(\d+)(?:,(\d+))?\s+@@")

    @classmethod
    def parse_patch(cls, diff_text: str) -> List[FilePatch]:
        patches: List[FilePatch] = []
        current_patch: Optional[FilePatch] = None
        current_hunk: Optional[PatchHunk] = None

        lines = diff_text.splitlines()
        i = 0
        while i < len(lines):
            line = lines[i]

            if line.startswith("--- "):
                old_file = line[4:].strip()
                i += 1
                new_file = lines[i][4:].strip() if i < len(lines) and lines[i].startswith("+++ ") else old_file
                current_patch = FilePatch(old_filename=old_file, new_filename=new_file)
                patches.append(current_patch)
                current_hunk = None
                i += 1
                continue

            match = cls.HUNK_HEADER_RE.match(line)
            if match and current_patch is not None:
                old_start = int(match.group(1))
                old_len = int(match.group(2)) if match.group(2) else 1
                new_start = int(match.group(3))
                new_len = int(match.group(4)) if match.group(4) else 1

                current_hunk = PatchHunk(
                    old_start=old_start,
                    old_length=old_len,
                    new_start=new_start,
                    new_length=new_len,
                )
                current_patch.hunks.append(current_hunk)
                i += 1
                continue

            if current_hunk is not None:
                if line.startswith("+"):
                    current_hunk.lines.append(PatchLine(line_type="+", content=line[1:]))
                elif line.startswith("-"):
                    current_hunk.lines.append(PatchLine(line_type="-", content=line[1:]))
                elif line.startswith(" ") or line == "":
                    content = line[1:] if line.startswith(" ") else ""
                    current_hunk.lines.append(PatchLine(line_type=" ", content=content))

            i += 1

        return patches

    @classmethod
    def apply_patch_to_file(cls, original_text: str, patch: FilePatch, fuzz: int = 2) -> str:
        orig_lines = original_text.splitlines()
        result_lines = list(orig_lines)
        offset = 0

        for hunk in patch.hunks:
            expected_pos = hunk.old_start - 1 + offset
            target_pos = cls._find_matching_position(result_lines, hunk, expected_pos, fuzz)

            if target_pos is None:
                raise PatchApplyError(f"Hunk at old_start {hunk.old_start} failed to apply to {patch.new_filename}")

            # Apply hunk deletions and insertions
            new_slice: List[str] = []
            old_count = 0
            for pl in hunk.lines:
                if pl.line_type == " ":
                    new_slice.append(pl.content)
                    old_count += 1
                elif pl.line_type == "+":
                    new_slice.append(pl.content)
                elif pl.line_type == "-":
                    old_count += 1

            result_lines[target_pos:target_pos + old_count] = new_slice
            offset += len(new_slice) - old_count

        return "\n".join(result_lines)

    @classmethod
    def _find_matching_position(cls, lines: List[str], hunk: PatchHunk, expected_pos: int, fuzz: int) -> Optional[int]:
        context_before = [pl.content for pl in hunk.lines if pl.line_type in (" ", "-")]

        for delta in range(fuzz + 1):
            for candidate in (expected_pos + delta, expected_pos - delta):
                if candidate < 0 or candidate + len(context_before) > len(lines):
                    continue
                match = True
                for idx, exp in enumerate(context_before):
                    if lines[candidate + idx] != exp:
                        match = False
                        break
                if match:
                    return candidate

        return None
