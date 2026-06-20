# Operator Controls Milestone: OPS-0 → OPS-2 (FROZEN for operator-controls-v0.1)

> Status: **FROZEN** as of `operator-controls-v0.1`. This document freezes the OPS-0 through OPS-2
> operator-controls arc as a named, auditable milestone before any further behavior is added. It is the
> single milestone-freeze record for the operator-controls layer; the per-sprint decisions live in
> `docs/PROJECT_CHARTER.md` (`DD-2026-06-20-D`, `DD-2026-06-20-E`, `DD-2026-06-20-F`). This file freezes the
> arc, the commit lineage, the frozen base, the demonstrated operator-control capability, the explain-and-
> verify boundary, the verification discipline, the training-gate verdict, the honest residuals, and the
> frozen-status declaration. It does not restate the per-sprint detail — it pins it.

## 0. Snapshot (recoverable freeze point)

```text
tag            operator-controls-v0.1
points at      the OPS-3 freeze commit (this document + its gate lock)
freezes        the OPS-0..OPS-2 operator-controls arc (head 0876ba0)
release_check  green + silent (exit 0, 0 bytes stdout, 0 bytes stderr)
recover        git checkout operator-controls-v0.1
training gate  training_not_justified (P12 training_justified=false) — weights forbidden
frozen base    multi-trace-validation-v0.1 @ 460be0c (frozen)
deeper base    integration-demo-v0.1 @ 95b586d, hypothesis-track-v0.1 @ bb20acf,
               reading-track-v0.1 @ f6fa55a, cognitive-os-governance-v0.1 @ bbd1113 (all frozen)
```

The operator-controls arc is a documentation and tooling layer over the frozen prototype: no model is in
the loop and no crate behavior was added. It is the operator's view of the system — a plain manual, an
executable smoke guard that keeps the manual honest, and a local archive snapshot of the frozen state.
Nothing it adds can execute a probe, mutate reading memory, alter a verifier receipt, promote an
observation, change the P12 training verdict, or release anything remotely. It is a CONTROL layer: it may
explain, reproduce, re-derive, verify, and record, and nothing else.

## 1. What is frozen — the commit lineage

Three commits form the arc. Each is docs/tooling-only — no crate behavior, no `Cargo.toml` change, and
(OPS-0/OPS-2) no code-crate edit at all. The hashes are auditable against `git log`.

| Step | What it added (invariant) | Commit |
| --- | --- | --- |
| OPS-0 | Operator Manual / Prototype Capability Guide: `OPERATOR_MANUAL.md`, a plain operator guide to the frozen prototype — what it is and is not, the five frozen milestone tags with recovery (`git checkout <tag>`) and verify (`./scripts/release_check.sh`) commands, every `cognitive-demo` command to reproduce each demo (with real flags and the eight audit-question slugs), the authority boundaries, and the P12 verdict. Every command was verified by running the binary. No code crate touched | `7aa17ec` |
| OPS-1 | Operator Smoke Script / Manual Drift Guard: `scripts/operator_smoke.sh`, a deterministic smoke that runs the WHOLE documented operator path end-to-end against the built binary in a throwaway temp dir and fails closed if any documented command, boundary line, or verify step has drifted from the manual. It re-derives every generated artifact through the binary's own verify subcommands (never trusts the bytes) and proves a tampered artifact is still refused. `release_check.sh` runs it (requiring a completion sentinel, so a vacuous smoke is caught). No code crate touched | `c33dea7` |
| OPS-2 | Operator Release Snapshot / Local Archive Manifest: `OPERATOR_RELEASE_SNAPSHOT.md`, a docs-only LOCAL snapshot of the post-OPS-1 state — the HEAD commit, all five frozen tags with commits, the recovery commands, the two verification commands, what the prototype can and cannot do, and the P12 verdict. It records the state honestly: it is NOT a remote release, and its own commit is a docs-only child of the state it records. No code crate touched | `0876ba0` |

The release snapshot frozen here is `OPERATOR_RELEASE_SNAPSHOT.md` at `0876ba0`.

## 2. The frozen base

The operator-controls arc is built ON TOP OF the five frozen milestones it documents, and it edits NONE of
them. The honest, precise statement: every prior tag still points where it did, and the `cognitive-demo`
crate (and `a.md`) are byte-for-byte identical to `multi-trace-validation-v0.1 @ 460be0c`.

```text
multi-trace-validation-v0.1   @ 460be0c   (scenario pack / coverage matrix / failure injection)
integration-demo-v0.1         @ 95b586d   (the cognitive-demo crate: trace / report / questions / bundle)
hypothesis-track-v0.1         @ bb20acf   (propose → probe → review → intent → observation → promotion-refusal)
reading-track-v0.1            @ f6fa55a   (the read0 verifier + the verified reading receipt)
cognitive-os-governance-v0.1  @ bbd1113   (the v0.1 governance / evidence-contract lineage)
```

OPS-0 and OPS-2 added documentation only; OPS-1 added a shell script and a gate lock. None of the three
edited a code crate: `git diff 460be0c..0876ba0 -- crates/ a.md Cargo.toml` is empty. Every prior milestone
tag is unmoved, and the frozen canonical `demo()` trace and bundle are byte-for-byte identical.

## 3. What the operator can now do (the demonstrated capability)

The prototype could already produce, report, interrogate, package, validate, and forge-then-reject one
canonical trace (through `multi-trace-validation-v0.1`). The operator-controls arc adds the operator's
control surface over that frozen system:

1. **Read what it is and how to run it (OPS-0).** A single plain-language manual documents every demo, the
   recovery and verification commands, the audit-question surface, and the authority boundaries — each
   command verified against the real binary.
2. **Verify the manual has not drifted (OPS-1).** An executable smoke runs the whole documented operator
   path against the built binary, re-derives every artifact through the binary's own verify subcommands,
   proves tamper is refused, and fails closed on any drift — and `release_check.sh` runs it, so drift
   breaks the gate.
3. **Record the frozen state locally (OPS-2).** A docs-only snapshot records the HEAD commit, the frozen
   tags and their commits, the recovery and verification commands, and the can/cannot-do boundaries —
   without pretending a remote release, training, execution, or authority expansion happened.

Every one of these is an operator CONTROL over the frozen prototype. None of them is authority, and none
releases anything.

## 4. The boundary that holds across the arc

These are the load-bearing invariants the whole arc preserves. None was weakened by a later sprint; each is
enforced by the release gate from the artifacts' own bytes.

1. **The controls explain and verify; they do not act (OPS-0/1/2).** The manual explains, the smoke
   verifies, the snapshot records. No control executes a probe, promotes an observation, mutates memory, or
   moves the training verdict.
2. **The smoke re-derives, never trusts (OPS-1).** Every artifact the smoke checks is re-derived through the
   binary's own verify subcommands and byte-compared; a tampered artifact is refused. The smoke writes only
   under a temp dir (no repo debris), uses `--out` (never a shell redirect, which the re-derive correctly
   refuses), and is fail-closed.
3. **The snapshot records, it does not release (OPS-2).** The snapshot is explicitly local: no push, no
   publication, no upload. It records the capability state and is honest that its own commit is a docs-only
   child of that state, changing no capability.
4. **No false authority across the arc.** The arc executes no probe, promotes nothing, mutates no memory,
   moves no training verdict, and releases nothing remotely. The strongest honest case is preserved from the
   layers below: governance may APPROVE a probe, yet execution stays `requires_operator` (never `executed`),
   the observation stays `requires_review`/`observation_only` (never `recorded`), and the promotion request
   is `rejected`. Approval is not execution; an observation is not evidence.
5. **No model in the loop.** The arc is documentation and tooling over a fully deterministic prototype. Any
   future model could only PROPOSE through the frozen hypothesis layer; it can never ground a claim, mutate
   memory, execute a probe, promote evidence, self-authorize, or release.

## 5. The training-gate verdict (P12)

**`training_not_justified`** (the P12 `TrainingDecision.training_justified` bit is
`training_justified=false`). The operator-controls arc is orthogonal to P12 and does not move it: every OPS
sprint reads the training decision before and after building its artifacts and proves it identical, and the
manual, smoke, and snapshot each record the verdict as closed. Weight training stays forbidden until the P11
eval proves a stable, recurring model failure that survives fixes to task spec, schema, prompt, examples,
tooling, context, and verifier design. P13–P15 (LoRA candidate, shadow mode, promotion gate) stay closed
under this freeze. This milestone makes no claim that training has opened.

## 6. The release gate (verification discipline)

The gate is `scripts/release_check.sh`. DONE means it **exits 0 AND is byte-silent** (0 bytes stdout, 0
bytes stderr). The operator-controls locks pin, per sprint: the manual's existence, its five frozen tag
names, every documented command invocation, a real audit-question slug, the training verdict, and the
six-line manual boundary (OPS-0); the smoke's existence and executability, an actual RUN of the smoke
requiring its completion sentinel (so a vacuous early-exit smoke is caught), and source pins proving it uses
`--out`, keeps its temp-dir cleanup, runs every documented command, re-derives through the verify
subcommands, proves tamper is refused, and records its five-line boundary (OPS-1); and the snapshot's
existence, the HEAD commit it records, the five frozen tag names and their commits, the recovery and verify
commands, the training verdict, the no-remote-release disclaimer, and its six-line boundary (OPS-2). Each
lock additionally guards against any artifact that falsely claims training has opened. This milestone block
additionally pins the freeze record itself (this document's OPS-0..OPS-2 commit lineage, the frozen-base tag
and commit references, the six boundary lines, and the `training_not_justified` verdict). The pinned commit
hashes are auditable against `git log`; this lock stays git-free and does NOT require the tag to exist — the
tag is created only after a clean tree and a green gate. The acceptance discipline for every sprint in this
arc was: rubric → green byte-silent `release_check` → live sabotage proving the gate catches a regression
(restored byte-identical by `cp`+`md5`, never `git checkout`) → an independent read-only adversarial verifier
panel with a fresh context → any residual folded before close.

## 7. Independent verification

Every sprint OPS-0 through OPS-2 was closed against read-only adversarial panels (Explore agents,
refute-by-default, scratch confined to a temp dir, each driving the compiled binary or inspecting the
artifacts), run until a fully-dry round with zero real findings. OPS-1 drove two genuine folds before its
panel went dry (the gate suppressed the smoke's failure stderr → surfaced on failure only; and a
`scenario-matrix-report --out` wrote an unvalidated file → content-validated without `--out`). OPS-0 and
OPS-2 each reached a dry round; OPS-2's boundary lens inverted the finding semantics (it labelled its
satisfied-criterion confirmations as findings) but none described a violation, and the other three lenses
reported zero. Each gate lock was proven load-bearing by live sabotage that failed the gate and was restored
byte-identical. Every claim in this document is checkable by running `scripts/release_check.sh` and reading
the named commits.

## 8. Honest residuals (NOT closed in operator-controls-v0.1)

Accepted limitations of the frozen milestone, published as caveats. They are the known edge of the
operator-controls layer, not bugs.

1. **The controls describe one fixed prototype.** The manual, smoke, and snapshot cover the single frozen
   canonical demo and its scenario/matrix/failure surfaces. They are an operator's control surface over that
   fixed system, not a general operator console; parameterizing the demo corpus is future work.
2. **Drift detection assumes one binary build.** The smoke verifies by re-deriving artifacts within the same
   deterministic build; cross-version reproduction is not claimed. The load-bearing integrity check is the
   byte-for-byte re-derivation, not a cryptographic digest.
3. **The snapshot is local, not a release.** OPS-2 records the state in the repository; it is explicitly not
   a remote release, publication, or upload, and claims none.
4. **Multi-file insider forgery is out of scope.** The re-derive-not-trust discipline and the gate locks
   defend against off-wire tampering and accidental regression, both of which the gate provably catches.
   They do not defend against an insider with commit access who authors malicious code AND rewrites the gate
   in the same change — that is the domain of code review and the governance/signing layer.
5. **No model in the loop.** The operator-controls arc is documentation and tooling over a fully
   deterministic prototype. Any future model may only PROPOSE through the frozen hypothesis layer; it can
   never ground a claim, mutate memory, execute a probe, promote evidence, self-authorize, or release. The
   P10 adapter stays gated shut by P12.
6. **Prototype, not production.** This is a deterministic Rust prototype and testbed, not a production
   reasoning system, and the operator controls describe it as such.
7. **Process caveat (verification method).** The read-only adversarial panels have on prior tracks left
   stray debris in the working tree despite their read-only instruction, and have occasionally inverted the
   finding-label semantics; each was caught and reconciled before close. It remains a known operational
   caveat of the panel method.

## 9. Frozen-status declaration

The OPS-0 → OPS-2 operator-controls arc is **FROZEN at `operator-controls-v0.1`**. The explain-and-verify
boundary is the frozen surface:

```text
The operator controls explain and verify the prototype.
They do not release remotely.
They do not create authority.
They do not execute.
They do not promote.
They do not train.
```

Any change that lets a control become authority; that lets the snapshot pretend a remote release happened;
that executes a probe, promotes an observation, creates evidence, mutates memory, or reopens training — must
pass through the same machinery: a rubric, a green byte-silent `release_check.sh`, a live sabotage, and an
independent adversarial panel, and must leave `training_justified=false` unless a clean recurring model
failure is proven. Relaxing any criterion requires explicit operator sign-off; it must not be edited
mid-stream to make a failing check pass. P13–P15 do not start under this freeze.
