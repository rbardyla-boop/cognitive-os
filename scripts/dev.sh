#!/usr/bin/env sh
set -eu
cd "$(dirname "$0")/.."
python3 scripts/bridge_world_demo.py "Use Bridge A if it is still the fastest route."

