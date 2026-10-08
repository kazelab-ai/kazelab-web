"""
KazeLab Python Tooling Registry.
"""

from typing import Dict, Any, Callable

class PythonToolRegistry:
    def __init__(self):
        self._tools: Dict[str, Callable[[Any], Any]] = {}

    def register(self, name: str, func: Callable[[Any], Any]):
        self._tools[name] = func

    def call(self, name: str, *args, **kwargs) -> Any:
        if name not in self._tools:
            raise ValueError(f"Tool {name} not found")
        return self._tools[name](*args, **kwargs)

python_tools = PythonToolRegistry()
