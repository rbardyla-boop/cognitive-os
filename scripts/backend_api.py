#!/usr/bin/env python3
"""Local stdlib HTTP API for Cognitive OS v0.1."""

from __future__ import annotations

import json
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlparse

from backend_storage import BackendStore, seed_static_memory
from bridge_world_demo import WORLD, available_scenarios, load_scenario, run
from world_encoder import encode_world_state


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_DB = ROOT / "data" / "cognitive_os.sqlite3"


class ApiHandler(BaseHTTPRequestHandler):
    store: BackendStore

    def do_GET(self) -> None:
        path = urlparse(self.path).path
        if path == "/health":
            self._json({"ok": True, "service": "cognitive-os", "schema_migrations": self.store.migrations()})
        elif path == "/packets":
            self._json({"packets": self.store.list_packets()})
        elif path.startswith("/traces/"):
            trace_id = path.removeprefix("/traces/")
            trace = self.store.get_trace(trace_id)
            self._json({"trace": trace} if trace else {"error": "trace_not_found"}, status=200 if trace else 404)
        elif path.startswith("/memory/"):
            memory_id = path.removeprefix("/memory/")
            memory = self.store.get_memory(memory_id)
            self._json({"memory": memory} if memory else {"error": "memory_not_found"}, status=200 if memory else 404)
        elif path == "/system-state":
            state = self.store.latest_system_state()
            if state is None:
                with (WORLD / "world_state.json").open("r", encoding="utf-8") as handle:
                    state = {"payload": encode_world_state(json.load(handle))}
            self._json({"system_state": state})
        else:
            self._json({"error": "not_found"}, status=404)

    def do_POST(self) -> None:
        path = urlparse(self.path).path
        body = self._body()
        if path == "/input":
            command = body.get("input") or body.get("command") or "Get to the far side safely."
            trace = run(command)
            self.store.insert_trace(trace)
            self._json({"trace_id": trace[0]["header"]["trace_id"], "packets": trace})
        elif path == "/simulate/scenario":
            scenario_name = body.get("scenario", "normal_crossing")
            if scenario_name not in available_scenarios():
                self._json({"error": "unknown_scenario", "scenario": scenario_name}, status=404)
                return
            try:
                scenario = load_scenario(scenario_name, allow_test_trusted=False)
            except PermissionError as exc:
                self._json({"error": "scenario_not_allowed", "reason": str(exc)}, status=403)
                return
            trace = run(scenario["command"], scenario)
            self.store.insert_trace(trace)
            self._json({"scenario": scenario_name, "trace_id": trace[0]["header"]["trace_id"], "packets": trace})
        else:
            self._json({"error": "not_found"}, status=404)

    def log_message(self, _format: str, *_args) -> None:
        return

    def _body(self) -> dict:
        length = int(self.headers.get("content-length", "0"))
        if length == 0:
            return {}
        return json.loads(self.rfile.read(length).decode("utf-8"))

    def _json(self, payload: dict, status: int = 200) -> None:
        data = json.dumps(payload, sort_keys=True).encode("utf-8")
        self.send_response(status)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)


def build_server(db_path: Path, port: int) -> ThreadingHTTPServer:
    store = BackendStore(db_path)
    seed_static_memory(store, WORLD)
    ApiHandler.store = store
    return ThreadingHTTPServer(("127.0.0.1", port), ApiHandler)


def main() -> int:
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8765
    db_path = Path(sys.argv[2]) if len(sys.argv) > 2 else DEFAULT_DB
    server = build_server(db_path, port)
    print(f"Listening on http://127.0.0.1:{port}")
    server.serve_forever()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
