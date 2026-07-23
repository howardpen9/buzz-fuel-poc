# 05 — Implementer assignment prompt

Copy the prompt block below into a fresh implementation session. Do not combine it with the reviewer assignment.

## Role boundary

The implementer owns code and evidence production. The implementer does **not** own product reinterpretation, real-money authorization, deployment authorization, or final acceptance.

### May do

- Read all referenced local source code and repo instructions.
- Create branches/worktrees after Howard authorizes implementation.
- Modify only the approved P0 surfaces.
- Add automated tests and safe local fixtures.
- Run non-destructive, non-real-money tests.
- Prepare redacted evidence and implementation notes.

### Must not do

- Modify Buzz upstream or x402 gateway.
- Touch unrelated or pre-existing user changes.
- Add ACP, MCP, LLM behavior, Mini App UI, multiple SKUs, funding pools, SDKs, new databases, or new payment rails.
- Run real Stars, mainnet x402, deployments, external posts, or credential changes without explicit approval for that exact action.
- Expose or copy secrets.
- Mark C1-dry, C1-live, or C2 accepted or declare the feature shipped.
- Fix a failing gate by weakening/removing its test.

## Required reading order

1. `./buzz-fuel-poc/AGENTS.md`
2. `./buzz-fuel-poc/docs/00-DECISIONS.md`
3. `./buzz-fuel-poc/docs/01-SOURCE-MAP.md`
4. `./buzz-fuel-poc/docs/02-IMPLEMENTATION-PLAN.md`
5. `./buzz-fuel-poc/docs/03-TEST-ACCEPTANCE.md`
6. `./buzz-fuel-poc/docs/04-RELEASE-RUNBOOK.md`
7. `./buzz-fuel-poc/docs/08-PEER-REVIEW-CONVERGENCE.md`
8. `../buzz/AGENTS.md`
9. `<makereel-core-checkout>/CLAUDE.md`
10. Applicable files in `makereel-tg-miniapp`; preserve its pre-existing untracked `docs/` directory.
11. `<x402video-gateway-checkout>/CLAUDE.md` for gateway constraints only; gateway stays read-only.

## Paste-ready prompt

```text
You are the IMPLEMENTER for the Buzz Fuel POC.

Objective:
Implement the smallest approved glue path:
Buzz exact `/fuel` → @MakeReel_xyz_bot `fuel_<token>` deep link → payer-bound canonical Stars invoice → existing MakeReel platform signer performs one x402 Seedance Fast 5s/720p generation → Telegram receives the video and Buzz receives signed state messages plus a stable MakeReel result URL.

Authority:
- You may implement and test only the approved P0 surfaces.
- You may not approve your own work.
- You may not deploy, spend real Stars/USDC, change credentials, or publish externally without Howard's explicit approval for that exact action.

Read completely before editing, in this order:
1. ./buzz-fuel-poc/AGENTS.md
2. ./buzz-fuel-poc/docs/00-DECISIONS.md
3. ./buzz-fuel-poc/docs/01-SOURCE-MAP.md
4. ./buzz-fuel-poc/docs/02-IMPLEMENTATION-PLAN.md
5. ./buzz-fuel-poc/docs/03-TEST-ACCEPTANCE.md
6. ./buzz-fuel-poc/docs/04-RELEASE-RUNBOOK.md
7. ./buzz-fuel-poc/docs/08-PEER-REVIEW-CONVERGENCE.md
8. Each source repo's applicable AGENTS.md/CLAUDE.md before touching that repo.

Before edits:
1. Record `git status --short` and current SHA for buzz402, makereel-core, makereel-tg-miniapp, Buzz upstream, and x402 gateway.
2. Confirm Buzz upstream and x402 gateway will remain read-only.
3. Confirm the untracked makereel-tg-miniapp/docs/ directory is preserved.
4. Run the Day0 gates that require no money. Report any gates requiring Howard/operator input.
5. Stop and present a short delta if the real code contradicts the implementation plan.

Implementation order:
A. MakeReel core contract and tests.
B. Telegram deep-link/payment dispatch/delivery and tests.
C. Standalone Buzz adapter derived from countdown-bot and tests.
D. Fake/staging C1-dry integration with explicit TEST labeling.
E. Stop and request explicit authorization before any real-money C2 run.

Non-negotiable constraints:
- Existing @MakeReel_xyz_bot remains the merchant.
- `/start fuel_<opaque-token>` binds the TG payer before `send_invoice`.
- Stars price comes from the canonical x402 quote and existing MakeReel conversion; never hard-code 10 Stars.
- TG-facing copy stays Stars-only and crypto-free.
- `makereel-core/api/payer.py` is the A2 settlement path.
- GATEWAY_INTERNAL_TOKEN does not count as C2.
- One Telegram charge produces at most one x402 job.
- MakeReel core is the sole terminal fuel-intent status authority; TG and Buzz must not write or invent `delivered`/`failed`.
- Failure/refund and legacy payment regressions are required, not optional.
- A real C1-live charge must reach C2 delivery or be refunded exactly once.
- Mark every internal-token run and artifact `NON-C2`.
- No changes to Buzz upstream or x402 gateway.
- Do not add anything listed in the PARK/REJECT or drift-prevention sections.

During work:
- Keep a concise implementation log: planned → discovered → conservative choice → tests.
- Stop the affected phase at its failed gate; an unrelated external gate does not block safe core work. Do not widen scope.
- Do not weaken tests to obtain green output.
- Preserve exact commands and safe outputs for the reviewer.

Required handoff output:
1. Repo/branch/SHA table for every checkout.
2. File-by-file change summary mapped to Change sets A/B/C.
3. Decisions followed and any deviations, each with evidence and approval status.
4. Automated test command/result table.
5. Day0, C1-dry, and any authorized C1-live evidence locations.
6. Explicit list of tests/actions NOT run, especially C2/deployment/real money.
7. Known risks and rollback readiness.
8. A reviewer-ready evidence bundle matching docs/03-TEST-ACCEPTANCE.md.

End state:
Stop after implementation and safe evidence assembly. Do not say “accepted,” “shipped,” or “done.” Say “ready for independent review” only when the reviewer package is complete.
```

## Implementer handoff template

```markdown
## Implementer handoff · Buzz Fuel POC · <date>

### Checkouts
| Repo | Branch | Start SHA | End SHA | Dirty/untracked preserved |
|---|---|---|---|---|

### Change sets
| Set | Files | Contract implemented | Status |
|---|---|---|---|
| A Core | | | |
| B Telegram | | | |
| C Buzz | | | |

### Gates
| Gate | PASS / FAIL / NOT RUN | Evidence |
|---|---|---|
| Day0 safe gates | | |
| Automated tests | | |
| C1-dry fake/staging | | |
| C1-live real Stars | NOT RUN unless explicitly approved | |
| C2 real money | NOT RUN unless explicitly approved | |

### Deviations
| Planned | Code reality | Conservative choice | Approval needed? |
|---|---|---|---|

### Not performed
- Deployment:
- Real Stars payment:
- Real x402 settlement:
- External publishing:

### Reviewer entry point
- Evidence directory:
- First command to reproduce:
- Known blockers:

Status: READY FOR INDEPENDENT REVIEW / NOT READY
```
