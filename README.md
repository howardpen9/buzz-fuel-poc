# Buzz Fuel POC

**Status:** implementation in progress on POC branches; automated + simulated C1-dry ready for independent review. Not accepted / not shipped.  
**Goal:** ship one auditable reference experiment connecting Buzz, Telegram Stars, and the existing x402 video rail without creating a fourth payment system.

**Implementer handoff:** [IMPLEMENTER-HANDOFF.md](./IMPLEMENTER-HANDOFF.md)

## One-line product contract

```text
Buzz /fuel
→ t.me/MakeReel_xyz_bot?start=fuel_<opaque-token>
→ canonical Telegram Stars invoice
→ existing MakeReel platform payer settles x402
→ the same result is delivered to Telegram and Buzz
```

The complete demo may say:

> Stars in. x402 out. Proof back to the room.

Only when the acceptance evidence proves a real x402 settlement.

## Read in this order

1. [00-DECISIONS.md](./docs/00-DECISIONS.md) — locked choices, open gates, non-goals.
2. [01-SOURCE-MAP.md](./docs/01-SOURCE-MAP.md) — exact local code to reuse and what not to copy.
3. [02-IMPLEMENTATION-PLAN.md](./docs/02-IMPLEMENTATION-PLAN.md) — cross-repo change sets and work order.
4. [03-TEST-ACCEPTANCE.md](./docs/03-TEST-ACCEPTANCE.md) — test matrix, evidence, independent sign-off.
5. [04-RELEASE-RUNBOOK.md](./docs/04-RELEASE-RUNBOOK.md) — staged release, rollback, demo recording.
6. [08-PEER-REVIEW-CONVERGENCE.md](./docs/08-PEER-REVIEW-CONVERGENCE.md) — final disposition of the peer-review challenges.
7. Choose the role entry: [05-IMPLEMENTER-PROMPT.md](./docs/05-IMPLEMENTER-PROMPT.md) or [06-REVIEWER-PROMPT.md](./docs/06-REVIEWER-PROMPT.md).

Archived peer input: [07-GROK-ALTERNATE-VIEW.md](./docs/07-GROK-ALTERNATE-VIEW.md). It is not implementation authority.

Folder-level implementation guardrails live in [AGENTS.md](./AGENTS.md).

## Role assignment

| Role | Start here | Authority | Forbidden |
|---|---|---|---|
| Implementer | [05-IMPLEMENTER-PROMPT.md](./docs/05-IMPLEMENTER-PROMPT.md) | change approved code surfaces, run non-destructive tests, assemble evidence | self-approve, deploy/spend without approval, widen scope |
| Independent reviewer | [06-REVIEWER-PROMPT.md](./docs/06-REVIEWER-PROMPT.md) | inspect diffs, rerun safe tests, declare PASS/PARTIAL/FAIL | implement fixes, reinterpret product decisions, authorize spending |

## Source-of-truth ownership

| Concern | Authority | P0 treatment |
|---|---|---|
| Telegram merchant and payment events | `MakeReel/makereel-tg-miniapp` + `makereel-core` | Extend behind a feature flag |
| Account, quote, Stars conversion, ledger, refund | `makereel-core` | Reuse; never duplicate here |
| x402 quote, settlement, job lifecycle | `x402video-gateway` via `makereel-core/api/payer.py` | Call unchanged |
| Buzz identity, NIP-42, kind 9 messages | `buzz/examples/countdown-bot` | Derive a small standalone adapter here |
| POC coordination and docs | This folder | New |

## Planned checkout topology

Implementation will touch three Git roots:

```text
/Users/howard/orca/projects/buzz402
  └── buzz-fuel-poc/                    # new Buzz adapter + integration docs

/Users/howard/Projects/x402/MakeReel/makereel-core
  └── api/                              # fuel intent/payment orchestration

/Users/howard/Projects/x402/MakeReel/makereel-tg-miniapp
  └── bot/                              # /start fuel_*, invoice relay, TG delivery
```

P0 must not modify:

- `/Users/howard/orca/projects/buzz402/buzz`
- `/Users/howard/Projects/x402/x402Video/x402video-gateway`
- MakeReel Mini App web UI

Those projects are reference/dependency surfaces, not POC worktrees.

## Current state

- [x] Planning folder created.
- [x] Existing payment, payer, gateway, polling, refund, and Buzz bot examples mapped.
- [x] Acceptance and rollback defined.
- [x] Change set A — makereel-core fuel contract + 18 tests (`poc/buzz-fuel-core`).
- [x] Change set B — TG `fuel_` / `bf:` dispatch + 15 tests (`poc/buzz-fuel-bot`).
- [x] Change set C — standalone Buzz `/fuel` adapter + unit tests (`poc/buzz-fuel-adapter`).
- [x] Simulated C1-dry core path (labeled NON-C2) — see `scripts/c1_dry_core_path.py`.
- [ ] Day0 live smokes (O1–O4) — need Howard/operator inputs.
- [ ] C1-live / C2 real money — blocked on explicit approval.
- [ ] Independent reviewer sign-off.

## Local commands (safe)

```bash
# Core
cd /Users/howard/Projects/x402/MakeReel/makereel-core
uv run pytest tests/test_buzz_fuel.py -v

# Telegram bot
cd /Users/howard/Projects/x402/MakeReel/makereel-tg-miniapp
uv run pytest tests/ -v

# Buzz adapter
cd /Users/howard/orca/projects/buzz402/buzz-fuel-poc
cargo test

# Simulated C1-dry (no money)
cd /Users/howard/Projects/x402/MakeReel/makereel-core
uv run python /Users/howard/orca/projects/buzz402/buzz-fuel-poc/scripts/c1_dry_core_path.py
```
- [x] Grok adversarial review resolved and folded into the authoritative plan.
- [ ] Day0 environment gates run.
- [ ] Implementation authorized and assigned.
- [ ] Code written.
- [ ] Independent acceptance completed.
