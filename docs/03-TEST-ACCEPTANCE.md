# 03 — Test and independent acceptance plan

The implementer must produce evidence. A separate reviewer decides PASS/FAIL.

## Acceptance roles

| Role | Responsibility |
|---|---|
| Implementer | code, automated tests, runbook execution, evidence bundle |
| Operator/Howard | credentials, real-money approval, fixed prompt, publish decision |
| Independent reviewer | reproduce safe tests, inspect evidence, sign final verdict |

The implementer cannot mark C2 accepted.

## Test sequencing — criteria are not deleted

The matrix below is the final acceptance contract. For velocity it runs in two sequences:

- **Before the first real Stars charge:** A01–A13 and A15; B01–B08, B10, and B11; C01, C02, C05, and C06; plus the targeted regression tests for every touched path.
- **Before final public C2 acceptance:** the complete A/B/C matrix, complete relevant repo suites, C1-live/C2 evidence, refund evidence, claims review, and rollback.

A14 may be demonstrated during the first controlled C2 run, but C2 cannot pass or be publicly claimed until the stable result URL is proven. Tests may be written and run earlier; this sequencing only defines stop gates.

## Gate 0 — preflight, no code

| ID | Test | Pass condition | Evidence |
|---|---|---|---|
| D0.1 | Buzz standalone auth | Bot AUTH accepted | redacted log + bot npub |
| D0.2 | Buzz channel write | Signed kind 9 appears in selected channel | event ID + screenshot |
| D0.3 | Existing bot health | `@MakeReel_xyz_bot` default `/start` works | timestamped screenshot |
| D0.4 | Existing Stars path | Operator confirms invoice/refund path health | redacted test record |
| D0.5 | Platform payer liquidity | Wallet can cover one canonical quote | redacted balance or approved smoke |
| D0.6 | Gateway SKU | `seedance-fast/5s-720p` quote/smoke succeeds | status and job/quote evidence |

A FAIL blocks the phase named by its dependency: D0.1/D0.2 block Buzz work, D0.3/D0.4 block live Telegram work, and D0.5/D0.6 block real C2. Safe core work may continue when an unrelated external gate is still OPEN. Do not compensate by adding systems.

## Automated unit and contract tests

### MakeReel core

| ID | Case | Expected |
|---|---|---|
| A01 | feature flag off | every fuel endpoint unavailable |
| A02 | create intent twice with same Buzz event | same live intent, not two |
| A03 | opaque token entropy/format | no sequential/guessable identifier |
| A04 | expired intent claim | rejected, no order |
| A05 | first claimant | account created/found; canonical quote converted to Stars |
| A06 | second claimant | conflict; original binding unchanged |
| A07 | same claimant retries | same pending order returned |
| A08 | wrong payer precheck | `{ok:false}` |
| A09 | wrong Stars amount | precheck/fulfill rejected |
| A10 | fulfill succeeds | `tg_generate(payment=stars, client=buzz_fuel)` called once |
| A11 | duplicate charge delivery | same job ID; no second generate/settlement |
| A12 | price changed after invoice | rejected/refund path, never silently charge changed price |
| A13 | gateway reject before settlement | non-2xx; no job; refund required |
| A14 | job succeeds | stable share URL becomes available |
| A15 | public status serialization | no TG identity, secret, payment header, or raw charge ID |
| A16 | concurrent TG/Buzz status reads | core maps each terminal intent transition once; adapters cannot write it |

### Telegram bot

| ID | Case | Expected |
|---|---|---|
| B01 | `/start` without fuel token | existing Watch behavior unchanged |
| B02 | malformed `fuel_` token | friendly expired/invalid response; no invoice |
| B03 | valid fuel token | one invoice with backend-returned Stars amount |
| B04 | legacy invoice payload | existing precheck/fulfill path unchanged |
| B05 | `bf:` payload | only fuel backend functions called |
| B06 | precheck failure | Telegram payment rejected before charge |
| B07 | successful payment | immediate receipt + one background poll task |
| B08 | duplicate successful update | no second task/job |
| B09 | job success | native video delivered once |
| B10 | failed/cancelled/timed-out job | Stars refund attempted once; failure copy shown |
| B11 | copy compliance | no USDC/x402/wallet/external crypto checkout in TG messages |

### Buzz adapter

| ID | Case | Expected |
|---|---|---|
| C01 | exact `/fuel` | creates one intent |
| C02 | unknown text or implicit NLP | ignored |
| C03 | pre-start historical event | ignored |
| C04 | bot's own event | ignored |
| C05 | duplicate Buzz event | same intent/no second payment link |
| C06 | state repeats while polling | one message per transition |
| C07 | relay rejects event | explicit failure, no false success log |
| C08 | delivered state | stable URL + safe IDs only |

## Required commands

Exact commands may be adjusted to the implemented test layout, but the reviewer must run equivalent gates.

```bash
# New standalone adapter
cargo fmt --manifest-path buzz-fuel-poc/Cargo.toml -- --check
cargo clippy --manifest-path buzz-fuel-poc/Cargo.toml -- -D warnings
cargo test --manifest-path buzz-fuel-poc/Cargo.toml

# MakeReel core
cd <makereel-core-checkout>
uv run pytest tests/test_buzz_fuel.py
uv run pytest

# Telegram Mini App bot
cd <makereel-tg-miniapp-checkout>
uv run pytest
```

No gateway or Buzz upstream test suite is required because P0 must not modify them. Their targeted smoke tests are Day0/integration dependencies.

## Integration checkpoint C1-dry

Use fake/staging core dependencies first; do not spend real money.

Steps:

1. In Buzz, post exact `/fuel`.
2. Verify a signed bot reply contains the correct `t.me/...start=fuel_...` link.
3. Open it as the intended TG user.
4. Verify the invoice price equals the canonical mocked quote conversion.
5. Feed an approved payment update in the test environment.
6. Verify core intent becomes paid/running.
7. Verify exactly one signed Buzz `Fueled` event.

C1-dry PASS requires correlation among the original Buzz event ID, intent ID, TG order, and signed `Fueled` event without exposing secrets. It proves plumbing only; evidence and public copy must say simulated/test payment and must not claim real Stars or x402.

## Integration checkpoint C1-live — explicit real Stars approval required

1. Start from a fresh `/fuel` event and bind the intended TG payer.
2. Confirm the invoice amount equals the approved canonical quote conversion.
3. Pay using the approved real Stars account and capture the `successful_payment` evidence.
4. Verify one core intent and one signed Buzz `Fueled` event.
5. Continue immediately into C2, or demonstrate the defined one-time refund closure.

C1-live never proves x402 by itself. A real charge left without C2 delivery or refund is FAIL.

## Integration checkpoint C2 — explicit real-money approval required

Before running, Howard approves:

- target network and gateway URL
- maximum USDC spend
- Stars amount shown
- platform payer address
- fixed prompt

Steps:

1. Continue from the approved C1-live event, or start a fresh `/fuel` Buzz event.
2. Pay the real Stars invoice with the approved TG account if not already paid.
3. Preserve the uncut recording from the Telegram payment confirmation through Buzz `Fueled`.
4. Confirm x402 payment response/transaction evidence and exactly one job ID.
5. Wait for success; model waiting may be cut from the public edit.
6. Verify Telegram receives the video.
7. Verify Buzz receives a stable MakeReel share URL and safe receipt IDs.
8. Re-deliver the same `successful_payment` update in a controlled test and verify the same job ID/no second settlement.

If the internal-token bypass is used, C2 is FAIL/PARTIAL and the public copy must not say “x402 out.”

Every internal-token dry run must be labeled `NON-C2` in logs, evidence metadata, filenames where practical, and any retained recording notes.

## Failure and refund acceptance

At least one controlled failure path must be demonstrated without a paid upstream generation, for example a mocked gateway rejection.

PASS requires:

- precheck failure prevents charge when possible
- post-charge pre-submit failure invokes `refund_star_payment`
- terminal job failure invokes the chosen refund policy exactly once
- Buzz publishes `Failed/Refunded`, never `Delivered`
- retry does not create a second paid job

## Regression acceptance

The reviewer must confirm:

- normal `/start` still opens the existing Watch experience
- an existing Mini App generation invoice still prechecks and fulfills
- an existing unlock invoice still follows its legacy branch
- no current payload can be mistaken for the `bf:` namespace
- disabling `BUZZ_FUEL_ENABLED` removes the new path without affecting legacy payments

## Evidence bundle

Store locally under a gitignored `evidence/<timestamp>/` directory; do not commit secrets or raw payment headers.

Required:

```text
evidence/<timestamp>/
  acceptance.md             # reviewer checklist and verdict
  versions.txt              # commit SHA of all three repos
  test-results.txt          # commands and summaries
  buzz-events.json          # safe event IDs/content/signatures
  payment-redacted.json     # Stars amount + safe charge suffix only
  x402-redacted.json        # network, amount, tx/payment response, no signature header
  job-redacted.json         # job ID, state transitions, stable share URL
  c1-recording.*
  c2-public-edit.*
  rollback-result.txt
```

`acceptance.md` must contain this claims checklist:

```text
[ ] Did not claim partnership or official Buzz support
[ ] Did not claim decentralized custody
[ ] Said “x402 out” only when C2 passed with real settlement evidence
[ ] Labeled Howard's payment as plumbing, not demand
[ ] Kept Telegram user-facing copy free of crypto checkout language
[ ] Labeled every mock/internal-token artifact TEST or NON-C2
```

## Independent reviewer sign-off

```markdown
### Buzz Fuel POC acceptance

- Day0 gates: PASS / FAIL
- Automated tests: PASS / FAIL
- C1-dry simulated payment → Buzz: PASS / FAIL
- C1-live real Stars → Buzz: PASS / NOT RUN / FAIL
- C2 real x402 → result: PASS / PARTIAL / FAIL
- Refund/idempotency: PASS / FAIL
- Legacy regression: PASS / FAIL
- Secret/copy review: PASS / FAIL
- Claims-honesty checklist: PASS / FAIL
- Rollback drill: PASS / FAIL

Verdict: ACCEPT / REJECT
Reviewer:
Date:
Blocking evidence gaps:
```
