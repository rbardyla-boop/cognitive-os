"""SQLite storage and migrations for the local backend."""

from __future__ import annotations

import json
import sqlite3
from pathlib import Path


MIGRATIONS: list[tuple[str, str]] = [
    (
        "001_initial_backend",
        """
        CREATE TABLE IF NOT EXISTS packets (
            packet_id TEXT PRIMARY KEY,
            trace_id TEXT NOT NULL,
            packet_type TEXT NOT NULL,
            schema_version TEXT NOT NULL,
            source_engine TEXT NOT NULL,
            target_engine TEXT NOT NULL,
            created_at TEXT NOT NULL,
            priority TEXT NOT NULL,
            raw_json TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS episodes (
            episode_id TEXT PRIMARY KEY,
            timestamp TEXT,
            source TEXT,
            confidence REAL,
            trace_id TEXT,
            raw_json TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS memory_nodes (
            memory_id TEXT PRIMARY KEY,
            claim TEXT,
            confidence REAL,
            status TEXT,
            schema_version TEXT,
            raw_json TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS rules (
            rule_id TEXT PRIMARY KEY,
            base_id TEXT,
            version INTEGER,
            raw_json TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS procedures (
            procedure_id TEXT PRIMARY KEY,
            status TEXT,
            confidence REAL,
            raw_json TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS contradictions (
            contradiction_id TEXT PRIMARY KEY,
            subject_id TEXT,
            contradicts_id TEXT,
            raw_json TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS traces (
            trace_id TEXT PRIMARY KEY,
            created_at TEXT,
            raw_json TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS deferred_jobs (
            job_id TEXT PRIMARY KEY,
            job_type TEXT NOT NULL,
            trace_id TEXT,
            status TEXT NOT NULL,
            raw_json TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS system_events (
            event_id INTEGER PRIMARY KEY AUTOINCREMENT,
            event_type TEXT NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            raw_json TEXT NOT NULL
        );
        """,
    ),
    (
        "002_packet_read_compat",
        """
        CREATE INDEX IF NOT EXISTS idx_packets_trace_id ON packets(trace_id);
        CREATE INDEX IF NOT EXISTS idx_packets_type ON packets(packet_type);
        CREATE INDEX IF NOT EXISTS idx_deferred_jobs_trace ON deferred_jobs(trace_id);
        """,
    ),
]


class BackendStore:
    def __init__(self, db_path: Path | str) -> None:
        self.db_path = Path(db_path)
        self.db_path.parent.mkdir(parents=True, exist_ok=True)
        self.conn = sqlite3.connect(self.db_path, check_same_thread=False)
        self.conn.row_factory = sqlite3.Row
        self.migrate()

    def migrate(self) -> None:
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations (version TEXT PRIMARY KEY, applied_at TEXT DEFAULT CURRENT_TIMESTAMP)"
        )
        applied = {
            row["version"]
            for row in self.conn.execute("SELECT version FROM schema_migrations").fetchall()
        }
        for version, sql in MIGRATIONS:
            if version in applied:
                continue
            self.conn.executescript(sql)
            self.conn.execute("INSERT INTO schema_migrations(version) VALUES (?)", (version,))
        self.conn.commit()

    def insert_packet(self, packet: dict) -> None:
        header = packet["header"]
        self.conn.execute(
            """
            INSERT OR REPLACE INTO packets
            (packet_id, trace_id, packet_type, schema_version, source_engine, target_engine, created_at, priority, raw_json)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                header["packet_id"],
                header["trace_id"],
                header["packet_type"],
                header["schema_version"],
                header["source_engine"],
                header["target_engine"],
                header["created_at"],
                header["priority"],
                json.dumps(packet, sort_keys=True),
            ),
        )

    def insert_trace(self, trace: list[dict]) -> None:
        if not trace:
            return
        trace_id = trace[0]["header"]["trace_id"]
        self.conn.execute(
            "INSERT OR REPLACE INTO traces(trace_id, created_at, raw_json) VALUES (?, ?, ?)",
            (trace_id, trace[0]["header"]["created_at"], json.dumps(trace, sort_keys=True)),
        )
        for packet in trace:
            self.insert_packet(packet)
            self._index_packet_payload(packet)
        self.conn.commit()

    def list_packets(self) -> list[dict]:
        rows = self.conn.execute("SELECT raw_json FROM packets ORDER BY created_at, packet_id").fetchall()
        return [json.loads(row["raw_json"]) for row in rows]

    def get_trace(self, trace_id: str) -> list[dict] | None:
        row = self.conn.execute("SELECT raw_json FROM traces WHERE trace_id = ?", (trace_id,)).fetchone()
        return json.loads(row["raw_json"]) if row else None

    def get_memory(self, memory_id: str) -> dict | None:
        for table, key in (
            ("memory_nodes", "memory_id"),
            ("episodes", "episode_id"),
            ("rules", "rule_id"),
            ("procedures", "procedure_id"),
        ):
            row = self.conn.execute(f"SELECT raw_json FROM {table} WHERE {key} = ?", (memory_id,)).fetchone()
            if row:
                return json.loads(row["raw_json"])
        return None

    def latest_system_state(self) -> dict | None:
        row = self.conn.execute(
            "SELECT raw_json FROM packets WHERE packet_type = 'SystemStatePacket' ORDER BY created_at DESC LIMIT 1"
        ).fetchone()
        return json.loads(row["raw_json"]) if row else None

    def insert_deferred_job(self, job: dict) -> None:
        self.conn.execute(
            "INSERT OR REPLACE INTO deferred_jobs(job_id, job_type, trace_id, status, raw_json) VALUES (?, ?, ?, ?, ?)",
            (
                job["job_id"],
                job["job_type"],
                job.get("trace_id"),
                job.get("status", "queued"),
                json.dumps(job, sort_keys=True),
            ),
        )
        self.conn.commit()

    def migrations(self) -> list[str]:
        rows = self.conn.execute("SELECT version FROM schema_migrations ORDER BY version").fetchall()
        return [row["version"] for row in rows]

    def _index_packet_payload(self, packet: dict) -> None:
        payload = packet["payload"]
        packet_type = packet["header"]["packet_type"]
        if packet_type == "EpisodePacket":
            self.conn.execute(
                "INSERT OR REPLACE INTO episodes(episode_id, timestamp, source, confidence, trace_id, raw_json) VALUES (?, ?, ?, ?, ?, ?)",
                (
                    payload["episode_id"],
                    payload["timestamp"],
                    payload["source"],
                    payload["confidence"],
                    payload["trace_id"],
                    json.dumps(payload, sort_keys=True),
                ),
            )
        elif packet_type == "MemoryMutation" and payload.get("operation") == "append_episode":
            episode = payload["episode"]
            self.conn.execute(
                "INSERT OR REPLACE INTO episodes(episode_id, timestamp, source, confidence, trace_id, raw_json) VALUES (?, ?, ?, ?, ?, ?)",
                (
                    episode["episode_id"],
                    episode["timestamp"],
                    episode["source"],
                    episode["confidence"],
                    episode["trace_id"],
                    json.dumps(episode, sort_keys=True),
                ),
            )
        elif packet_type == "BackpressureCommand" and payload.get("type") == "post_action_revalidation":
            self.insert_deferred_job(
                {
                    "job_id": f"JOB_{payload['after_action']}",
                    "job_type": "post_action_revalidation",
                    "trace_id": payload["trace_id"],
                    "status": "queued",
                    "payload": payload,
                }
            )


def seed_static_memory(store: BackendStore, world_dir: Path) -> None:
    for path, table, key in (
        (world_dir / "semantic_memory.json", "memory_nodes", "memory_id"),
        (world_dir / "rules.json", "rules", "id"),
        (world_dir / "procedures.json", "procedures", "procedure_id"),
    ):
        with path.open("r", encoding="utf-8") as handle:
            items = json.load(handle)
        for item in items:
            if table == "memory_nodes":
                store.conn.execute(
                    "INSERT OR REPLACE INTO memory_nodes(memory_id, claim, confidence, status, schema_version, raw_json) VALUES (?, ?, ?, ?, ?, ?)",
                    (
                        item["memory_id"],
                        item["claim"],
                        item["confidence"],
                        item["status"],
                        item["schema_version"],
                        json.dumps(item, sort_keys=True),
                    ),
                )
                for contradiction_id in item.get("contradictions", []):
                    store.conn.execute(
                        "INSERT OR REPLACE INTO contradictions(contradiction_id, subject_id, contradicts_id, raw_json) VALUES (?, ?, ?, ?)",
                        (
                            f"{item['memory_id']}->{contradiction_id}",
                            item["memory_id"],
                            contradiction_id,
                            json.dumps(
                                {"subject_id": item["memory_id"], "contradicts_id": contradiction_id},
                                sort_keys=True,
                            ),
                        ),
                    )
            elif table == "rules":
                store.conn.execute(
                    "INSERT OR REPLACE INTO rules(rule_id, base_id, version, raw_json) VALUES (?, ?, ?, ?)",
                    (item["id"], item["base_id"], item["version"], json.dumps(item, sort_keys=True)),
                )
            elif table == "procedures":
                store.conn.execute(
                    "INSERT OR REPLACE INTO procedures(procedure_id, status, confidence, raw_json) VALUES (?, ?, ?, ?)",
                    (item["procedure_id"], item["status"], item["confidence"], json.dumps(item, sort_keys=True)),
                )
    store.conn.commit()
