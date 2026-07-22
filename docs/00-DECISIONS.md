# 00 — Decisions and scope

**Decision date:** 2026-07-22  
**Status:** peer-reviewed and ready for assignment; implementation not started

## Frame

- **Question:** Can one existing MakeReel Telegram Stars payment fund one real x402 video job and return a signed receipt/result to Buzz?
- **Done means:** required C1-dry/C1-live/C2 gates pass, evidence exists, and an independent reviewer signs the acceptance sheet.
- **Not the question:** whether a general Agent Fuel Runtime, DAO, or shared treasury should be built.

## LOCK

1. **Telegram merchant:** use [@MakeReel_xyz_bot](https://t.me/MakeReel_xyz_bot); do not create a new bot.
2. **Telegram entry:** Buzz publishes `https://t.me/MakeReel_xyz_bot?start=fuel_<opaque-token>`; the bot learns the TG user from `/start` before creating an invoice.
3. **Invoice method:** the bot uses `send_invoice` after claiming the intent. A generic invoice link must not be generated before the payer is bound.
4. **Price:** fixed SKU, canonical price. `makereel-core` obtains the x402 quote and converts it with the existing Stars pricing function. No hard-coded 10 Stars.
5. **SKU:** `POST /generate/seedance-fast/5s-720p`, fixed server-side prompt, `16:9`, no free-form user prompt.
6. **Settlement:** existing `makereel-core/api/payer.py` signs the x402 v2 EIP-3009 authorization. Gateway internal-token bypass does not count as C2.
7. **Buzz adapter:** derive from `buzz/examples/countdown-bot`; use direct WebSocket, NIP-42, kind 0 profile, kind 9 channel messages, and standalone bot identity.
8. **Delivery:** Telegram receives the generated video using the existing polling/delivery pattern; Buzz receives a stable MakeReel share URL, not an expiring gateway URL.
9. **Storage:** reuse existing MakeReel account, ledger, manifest, and idempotency. P0 adds only an intent/order store following the existing short-lived order pattern; no new database.
10. **Deployment safety:** `BUZZ_FUEL_ENABLED=false` by default. Stopping the Buzz adapter and turning the flag off is the rollback.
11. **Terminal status authority:** `makereel-core` alone maps canonical gateway/job state into fuel-intent `delivered` or `failed`; Telegram and Buzz only observe core state.
12. **Development prompt:** use `Buzz launch reel, bold kinetic typography "BUZZ", dark neon workspace, 5s, cinematic` until Howard replaces it once before the canonical quote/C2 run.
13. **Claims gate:** mock, real Stars, and real x402 evidence are labeled separately. No internal-token or mocked run may be described as x402 settlement.

## Acceptance checkpoints

### C1-dry — simulated payment event → signed Buzz receipt

```text
approved fake/staging successful_payment update
→ MakeReel records one paid fuel intent
→ Buzz bot publishes a signed “Fueled” kind 9 message
```

C1-dry proves event correlation and signed messaging only. Public material must label it as a test/simulated payment path and must not claim real Stars or x402 settlement.

### C1-live — real Stars → signed Buzz receipt

```text
real Telegram successful_payment
→ MakeReel records the charge and one fuel intent
→ Buzz bot publishes a signed “Fueled” kind 9 message
```

C1-live alone does not prove x402. A real accepted Stars charge must continue to C2 delivery or enter the defined refund closure.

### C2 — x402 → delivered result

```text
the same Stars charge
→ PLATFORM_SIGNER_KEY creates one real x402 settlement
→ one gateway job succeeds
→ Telegram gets the video
→ Buzz gets a stable result link and receipt identifiers
```

## OPEN — resolve only when its blocking phase begins

| # | Question | Default/next action | Owner | Evidence required | Blocks |
|---|---|---|---|---|---|
| O1 | Which Buzz relay and channel host the demo? | Use one existing/local demo relay and a dedicated channel UUID supplied at assignment | Howard | URL + channel UUID | Buzz smoke only |
| O2 | Can a standalone bot key authenticate and write there? | Test standalone first; if it fails, try owner-attested once, then stop | Implementer | accepted event ID or two bounded failure records | Buzz implementation only |
| O3 | Is production `@MakeReel_xyz_bot` healthy today? | Run `/start` and the existing health check | Operator | timestamped screenshot + health response | TG deployment work |
| O4 | Can the platform signer cover the canonical quote? | Require quote amount plus 20% buffer on the configured network | Operator | redacted balance/quote evidence | real C2 only |

Release default: X primary plus at most one already-open Buzz community surface. Do not create a multi-channel launch plan for P0.

## PARK

- Agent Fuel SDK/runtime and generalized funding policy.
- More models or a SKU catalog.
- Multiple contributors or threshold funding.
- Capability tokens, user API keys, subscriptions, or balances.
- Owner-attested Buzz auth; revisit only if standalone admission is impossible.
- QR visual polish; the t.me link is sufficient for acceptance.

## REJECT

- Forking Buzz Desktop.
- Modifying x402 gateway for this POC.
- Calling an internal-token bypass “x402 settlement.”
- Letting TG-facing copy mention USDC, crypto wallets, or an external crypto checkout.
- Counting Howard's own payment as market demand.
- Shipping without refund, duplicate-payment, and legacy-payment regression evidence.
- Adding database tables, mission-policy fields, contributor lists, or capability tokens to P0.

## Convergence handoff

| Role | Owns | Does not own |
|---|---|---|
| Howard | O1/O5/O6, real-money approval, final publish | self-acceptance of code |
| Core implementer | MakeReel intent, quote, fulfillment, ledger contract | Buzz protocol changes |
| Bot implementer | deep link, invoice relay, TG delivery/refund UX | pricing or settlement logic |
| Buzz implementer | NIP auth/messages and status mirroring | payment authority |
| Independent reviewer | test evidence and acceptance verdict | implementation fixes |

Paste-ready role assignments:

- Implementer: [05-IMPLEMENTER-PROMPT.md](./05-IMPLEMENTER-PROMPT.md)
- Independent reviewer: [06-REVIEWER-PROMPT.md](./06-REVIEWER-PROMPT.md)

Peer-review disposition: [08-PEER-REVIEW-CONVERGENCE.md](./08-PEER-REVIEW-CONVERGENCE.md)
