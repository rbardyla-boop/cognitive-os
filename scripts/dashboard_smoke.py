#!/usr/bin/env python3
"""Smoke-check dashboard audit surface scaffolds."""

from __future__ import annotations

from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DASHBOARD = ROOT / "ui" / "dashboard"
REQUIRED = {
    "packet_stream.tsx": "PacketStream",
    "memory_graph.tsx": "MemoryGraph",
    "attention_view.tsx": "AttentionView",
    "trace_view.tsx": "TraceView",
    "verifier_view.tsx": "VerifierView",
}


def main() -> int:
    for filename, export_name in REQUIRED.items():
        path = DASHBOARD / filename
        if not path.exists():
            raise SystemExit(f"missing dashboard file: {path}")
        text = path.read_text(encoding="utf-8")
        if f"export function {export_name}" not in text:
            raise SystemExit(f"missing export {export_name} in {path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

