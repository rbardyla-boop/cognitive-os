# Environment / Runtime Lock (v0.1)

This document is the runtime lock for `cognitive-os-v0.1.0`. It is the source of truth for how to
run the checks reproducibly.

## Python version

- **Minimum:** Python 3.10 (the code uses `X | Y` union type syntax, which is 3.10+).
- **Tested / supported:** Python 3.12.3.

Running under Python 3.9 or earlier fails with cryptic `X | Y` type-syntax errors. Do not run below
3.10.

## The single third-party dependency

The only third-party dependency across `scripts/` is `cryptography` (Ed25519 asymmetric signing and
verification). Everything else is the standard library or local modules.

```
cryptography==41.0.7
```

`cryptography` is imported directly by `scripts/replay_asymmetric_key.py` and
`scripts/design_signing.py` (and used transitively by `scripts/recovery_replay.py`) for Ed25519 key
generation, private/public key serialization, and signature creation/verification (Sprints 21, 30-31).

Install:

```sh
/usr/bin/python3 -m pip install -r requirements.txt
```

## Rust toolchain (engine, P1+)

The deterministic engine ([ADR-002](ADR-002-runtime-engine-replay-contract.md), `a.md` Prototype-First
Track) is a Rust workspace at the repo root (`Cargo.toml` → `crates/vibe-core`). `release_check.sh`
builds and tests it, so a Rust toolchain is **required** from P1 onward:

```text
cargo / rustc 1.94.0 (system, at /usr/bin/cargo — on the documented PATH=/usr/bin)
rustfmt + clippy components (release_check runs `cargo fmt --check` and `cargo clippy -D warnings`)
```

`vibe-core` (the L0 kernel) declares **zero dependencies**, so `cargo test --offline` needs no network
and no registry access — only the toolchain and std. `target/` is git-ignored; `Cargo.lock` is tracked
for reproducibility. The kernel holds no wall-clock, entropy, filesystem, network, signing, async, or
backend code; release_check enforces this by source scan + an empty dependency tree.

## CRITICAL: interpreter pitfall

The default `python3` on the shell PATH in this environment is an **unrelated virtualenv**
(`foundry/.venv`) that does **not** have `cryptography` installed. Running the scripts naively as
`python3 scripts/...` fails with `ModuleNotFoundError: No module named 'cryptography'`.

The **correct interpreter** is the system Python at `/usr/bin/python3` (Python 3.12.3, cryptography
41.0.7). Prefix every Python invocation so the system interpreter wins on PATH:

```sh
PATH=/usr/bin:$PATH python3 scripts/<script>.py
```

Verify your interpreter before running anything:

```sh
PATH=/usr/bin:$PATH python3 --version            # expect Python 3.12.3 (>= 3.10)
PATH=/usr/bin:$PATH python3 -c "import cryptography; print(cryptography.__version__)"  # expect 41.0.7
```

## The correct run command

Run the full release gate (it must exit 0 and produce zero bytes on stdout and stderr):

```sh
PATH=/usr/bin:$PATH bash scripts/release_check.sh
```

Run the test suite the gate calls:

```sh
PATH=/usr/bin:$PATH bash scripts/test.sh
```

If you need to run an individual audit with the local scripts on the import path:

```sh
PATH=/usr/bin:$PATH PYTHONPATH=scripts python3 scripts/design_audit.py --scenario <name>
```

Note: `scripts/release_check.sh` invokes bare `python3` internally; it succeeds only because the
leading `PATH=/usr/bin:$PATH` puts the system interpreter first. Always invoke the gate with that
prefix.

## Determinism guarantees

The system is fully deterministic and reproducible:

- All decision logic uses **logical ticks** passed as integers (`evaluation_tick`,
  `valid_from_tick`, etc.), never wall-clock time. A release gate asserts zero
  `datetime`/`time.time`/`time.monotonic` symbols in `design_signing.py`.
- No randomness: no `random.*`, `randint`, or `choice` in `scripts/`.
- Ed25519 signing is deterministic given the same private key and message payload.
- All hashing (SHA-256, HMAC-SHA256) is deterministic.
- All canonical JSON serialization uses `sort_keys=True` and fixed separators.

The same inputs produce byte-identical outputs across runs.

## No-network guarantee

- No network access is required or performed. There is no live internet access, no remote key
  fetch, and no external service call anywhere in `scripts/`.
- The only cryptographic material in the repo is **public keys** (in
  `authorized_design_signers.json`). No private signing key is committed; a release gate asserts
  this (`BEGIN PRIVATE KEY` / `BEGIN OPENSSH PRIVATE KEY` appear in zero files under `simulations/`
  and `scripts/`, excluding the gate scripts that name the pattern as a grep argument). Signing
  private keys are generated at authoring time and discarded.
