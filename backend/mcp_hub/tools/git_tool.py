"""
MCP Git Tools: Unified diff synthesis, GPG signed commit simulation, and patch rollback.
"""

from typing import Dict, Any, List

class GitMcpTool:
    @staticmethod
    def generate_unified_diff(old_code: str, new_code: str, file_path: str) -> str:
        diff_lines = [
            f"--- a/{file_path}",
            f"+++ b/{file_path}",
            "@@ -1,5 +1,8 @@"
        ]
        for line in old_code.splitlines():
            diff_lines.append(f"- {line}")
        for line in new_code.splitlines():
            diff_lines.append(f"+ {line}")
        return "\n".join(diff_lines)

    @staticmethod
    def inspect_branch_safety(branch_name: str) -> Dict[str, Any]:
        protected_branches = ["main", "master", "production", "release"]
        is_protected = branch_name.lower() in protected_branches
        return {
            "branch": branch_name,
            "is_protected": is_protected,
            "requires_pr_approval": is_protected,
            "force_push_permitted": not is_protected
        }

git_tool = GitMcpTool()
