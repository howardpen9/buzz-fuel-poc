# 08 — Peer-review convergence

**Date:** 2026-07-22  
**Status:** final planning disposition  
**Input:** [07-GROK-ALTERNATE-VIEW.md](./07-GROK-ALTERNATE-VIEW.md)

This file records how the Grok alternate view was resolved. The executable requirements are folded into `00`–`06`; implementers and reviewers follow those files, not the alternate view.

## Frame

- **Question:** Which Grok divergences improve the 24-hour POC without weakening payment safety, evidence quality, or role independence?
- **Done means:** every material divergence is LOCK, PARK, or REJECT, with no parallel implementation plan left behind.

## Challenge disposition

| # | Verdict | Final disposition |
|---|---|---|
| 1 | REJECT | C2 still requires an independent reviewer. Review happens after implementer handoff, so it does not block implementation velocity. |
| 2 | PARTIAL | Tests are sequenced in tiers, not deleted. Money-critical tests block the first real Stars charge; the complete matrix blocks final public C2 acceptance. |
| 3 | PARTIAL | A C1-only demo may be published only with dry/live labeling and no x402 or completed-delivery claim. Any real Stars charge must reach C2 delivery or the refund closure. |
| 4 | ACCEPT | MakeReel core is the only authority that maps canonical job state to terminal fuel-intent state. TG and Buzz are observers. |
| 5 | ACCEPT | P0 uses a process-local `dict + lock + TTL` store patterned after miniapp orders. No database migration. |
| 6 | ACCEPT | `fuel_delivery.py` is optional extraction after a bounded working loop, not a day-one architecture gate. |
| 7 | ACCEPT | The proposed fixed prompt is the development default; Howard may replace it once before canonical quote and C2. |
| 8 | PARTIAL | Keep the 8–13 focused-hour estimate and add a 24-hour calendar cut. Historical six-hour estimates are marked superseded, not erased. |
| 9 | ACCEPT | Every internal-token run is labeled `NON-C2` in logs, evidence, and media. |
| 10 | ACCEPT | If standalone Buzz auth fails, make one owner-attested smoke attempt and stop if it also fails. Do not redesign Buzz auth in P0. |

## LOCK

1. `makereel-core` owns payment truth and the only terminal intent-state transition function.
2. `C1-dry` and `C1-live` are distinct; neither proves x402 settlement.
3. The independent reviewer remains mandatory for final C2 acceptance.
4. Claims honesty is an acceptance gate, not release-copy polish.
5. Module/file layout may simplify during implementation as long as contracts, tests, and scope boundaries remain intact.

## OPEN — Day0 values only

| # | Question | Blocks | Owner | Next action |
|---|---|---|---|---|
| O1 | Exact Buzz relay URL and channel UUID | Buzz adapter smoke | Howard | Supply one existing/local demo target at assignment |
| O2 | Does standalone write succeed there? | Buzz implementation | Implementer | Run countdown-bot-derived no-code/key smoke; try owner-attested once on failure |
| O3 | Is production `@MakeReel_xyz_bot` healthy today? | TG deployment work | Operator | Capture timestamped `/start` and health evidence |
| O4 | Can the signer cover one canonical quote plus 20% buffer? | Real C2 | Operator | Capture redacted balance/quote evidence before approval |

O5 is provisionally locked to the prompt in `00-DECISIONS.md`. O6 is provisionally X primary plus at most one already-open Buzz community surface.

## PARK / REJECT

- **PARK:** extracting `fuel_delivery.py`, splitting the Rust crate into multiple modules, and a `Running` message; do them only when the working code clearly benefits.
- **REJECT:** self-approved C2, Postgres/schema work, internal-token C2 claims, P2 policy fields, and scope expansion into ACP/MCP/runtime work.

## Handoff

| Owner | Action | Artifact | Due |
|---|---|---|---|
| Howard | Assign the implementer using the finalized prompt | `05-IMPLEMENTER-PROMPT.md` | now |
| Implementer | Execute Day0, implement, and produce evidence without self-approval | implementer handoff + evidence bundle | within the 24-hour cut |
| Independent reviewer | Verify the handoff without fixing code | report using `06-REVIEWER-PROMPT.md` | after handoff |

