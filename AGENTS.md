# AGENTS.md — Buzz Fuel POC guardrails

These instructions apply to everything under `buzz-fuel-poc/`.

## Before changing code

Read, in order:

1. `docs/00-DECISIONS.md`
2. `docs/01-SOURCE-MAP.md`
3. `docs/02-IMPLEMENTATION-PLAN.md`
4. `docs/03-TEST-ACCEPTANCE.md`
5. `docs/04-RELEASE-RUNBOOK.md`
6. `docs/08-PEER-REVIEW-CONVERGENCE.md`
7. Your role prompt: `docs/05-IMPLEMENTER-PROMPT.md` or `docs/06-REVIEWER-PROMPT.md`
8. The applicable instructions in each source repo before editing it.

`docs/07-GROK-ALTERNATE-VIEW.md` is peer-review history, not a second plan. If it differs from `00`–`06`, follow the folded-back requirements in `00`–`06` and the disposition in `08`.

Stop and report any conflict between the real source and these documents. Do not silently redesign the plan.

## Scope

P0 is glue among three existing systems. It is not a runtime, marketplace, SDK, ACP integration, or new payment rail.

Allowed implementation surfaces:

- This folder: a derived standalone Buzz bot and POC-only support files.
- `makereel-core`: internal fuel-intent contract behind `BUZZ_FUEL_ENABLED`.
- `makereel-tg-miniapp`: `fuel_` deep-link routing, Stars invoice relay, result delivery.

Do not modify Buzz upstream or x402 gateway in P0. If the plan appears to require that, stop and request a decision.

## Money and secrets

- Never run a real-money test without Howard's explicit approval for that run.
- Never print, copy, commit, or expose bot tokens, Nostr private keys, internal bearer tokens, wallet keys, or payment headers.
- Telegram-facing copy stays Stars-only and crypto-free.
- Upstream API keys never leave the gateway.
- A Telegram Stars charge must produce at most one x402 job.
- Any accepted Stars payment that cannot be fulfilled must follow the existing refund path.
- Never hard-code a Stars amount. Use the existing canonical x402 quote and MakeReel Stars conversion.
- Only `makereel-core` may map canonical job state to terminal fuel-intent state; TG and Buzz adapters are observers.
- Mark mock/internal-token artifacts TEST or `NON-C2`; never reuse them as real settlement evidence.

## Drift prevention

Do not add before C1-dry, C1-live, and C2 pass:

- Mini App UI
- ACP, MCP, or an LLM agent
- free-form prompts
- multiple SKUs
- multi-contributor funding
- QR/fuel-meter polish
- Buzz media uploads
- API keys or capability tokens
- a new database, queue, treasury, or account system

## Existing worktrees

At planning time, `makereel-tg-miniapp` contains an untracked `docs/` directory. Treat it as user-owned and do not delete, move, overwrite, or include it in a commit unless explicitly authorized.

Before edits, record `git status --short` for every checkout. Preserve unrelated changes.

## Required delivery workflow

1. Pass Day0 gates without code changes.
2. Implement the MakeReel core contract and tests.
3. Implement the Telegram adapter and tests.
4. Implement the Buzz adapter and tests.
5. Pass C1-dry with fake/staging dependencies and label the evidence accordingly.
6. Obtain explicit approval for a real Stars/x402 run.
7. Pass C1-live and C2, or refund the real Stars charge exactly once; collect the evidence bundle.
8. Hand the bundle to an independent reviewer; the implementer does not self-approve.

If a gate fails, stop at that gate. Do not widen scope to work around it.

## Role separation

- The implementer may write code but may not mark C1-dry, C1-live, or C2 accepted.
- The reviewer may issue the acceptance verdict but may not repair the implementation in the same review assignment.
- A FAIL returns a bounded defect list to the implementer. The reviewer starts a fresh verification pass after fixes.
- Howard alone approves credentials, deployment, real Stars/USDC spending, and public release.
