"""Local in-process CIP broker with priority lanes and trace events."""

from __future__ import annotations

from collections import defaultdict, deque
from datetime import datetime, timezone


PRIORITY_LANES = {
    "P0": "safety interrupt",
    "P1": "active action correction",
    "P2": "active goal relevance",
    "P3": "contradiction/anomaly",
    "P4": "memory maintenance",
    "P5": "curiosity/background learning",
    "P6": "archival/compression",
}


def _now() -> str:
    return datetime.now(timezone.utc).isoformat()


class InProcessBroker:
    def __init__(self) -> None:
        self.subscriptions: dict[str, set[str]] = defaultdict(set)
        self.queues: dict[str, dict[str, deque[dict]]] = defaultdict(
            lambda: {lane: deque() for lane in PRIORITY_LANES}
        )
        self.in_flight: dict[str, tuple[str, dict]] = {}
        self.deferred: dict[str, tuple[dict, str]] = {}
        self.dead_letters: dict[str, tuple[dict, str]] = {}
        self.acked: set[str] = set()
        self.published: list[dict] = []
        self.events: list[dict] = []

    def subscribe(self, engine: str, packet_type: str) -> None:
        self.subscriptions[engine].add(packet_type)
        self._event("subscribe", engine=engine, packet_type=packet_type)

    def publish(self, packet: dict) -> dict:
        header = packet["header"]
        packet_type = header["packet_type"]
        priority = header["priority"]
        if priority not in PRIORITY_LANES:
            raise ValueError(f"Unknown priority lane: {priority}")

        self.published.append(packet)
        delivered = 0
        for engine, packet_types in self.subscriptions.items():
            if packet_type in packet_types or "*" in packet_types:
                self.queues[engine][priority].append(packet)
                delivered += 1

        self._event(
            "publish",
            packet_id=header["packet_id"],
            packet_type=packet_type,
            trace_id=header["trace_id"],
            priority=priority,
            subscribers=delivered,
        )
        return packet

    def poll(self, engine: str) -> dict | None:
        for priority in PRIORITY_LANES:
            queue = self.queues[engine][priority]
            if queue:
                packet = queue.popleft()
                packet_id = packet["header"]["packet_id"]
                self.in_flight[packet_id] = (engine, packet)
                self._event("poll", engine=engine, packet_id=packet_id, priority=priority)
                return packet
        self._event("poll_empty", engine=engine)
        return None

    def ack(self, packet_id: str) -> None:
        engine, _packet = self._take_in_flight(packet_id)
        self.acked.add(packet_id)
        self._event("ack", engine=engine, packet_id=packet_id)

    def defer(self, packet_id: str, reason: str) -> None:
        engine, packet = self._take_in_flight(packet_id)
        self.deferred[packet_id] = (packet, reason)
        self._event("defer", engine=engine, packet_id=packet_id, reason=reason)

    def dead_letter(self, packet_id: str, reason: str) -> None:
        engine, packet = self._take_in_flight(packet_id)
        self.dead_letters[packet_id] = (packet, reason)
        self._event("dead_letter", engine=engine, packet_id=packet_id, reason=reason)

    def trace_for(self, trace_id: str) -> list[dict]:
        return [packet for packet in self.published if packet["header"]["trace_id"] == trace_id]

    def _take_in_flight(self, packet_id: str) -> tuple[str, dict]:
        if packet_id not in self.in_flight:
            raise KeyError(f"Packet is not in flight: {packet_id}")
        return self.in_flight.pop(packet_id)

    def _event(self, event_type: str, **fields) -> None:
        self.events.append({"event_type": event_type, "created_at": _now(), **fields})

