# 02 — Implementation plan

No implementation has been performed. This is the ordered work package for the assigned builder.

## Architecture

```text
Buzz relay/channel
  ↑ signed kind 9                              GET intent status
  │                                                     ↑
[buzz-fuel-poc adapter] ── create intent ──> [makereel-core /internal/buzz-fuel]
  │                                                     ↑
  └─ t.me/MakeReel_xyz_bot?start=fuel_<token>           │
                                                        │
Telegram user → [existing Mini App bot] ─ claim/sendInvoice/precheck/fulfill
                                                        │
                                                        ↓
                                 existing tg_quote → platform payer → x402 gateway
                                                        │
                                 existing job poll/share ←┘
```

There is no direct Buzz → gateway call and no payment authority in the Buzz adapter.

## Planned contracts

All MakeReel endpoints below are internal Bearer-authenticated and disabled unless `BUZZ_FUEL_ENABLED=true`.

### 1. Create intent — Buzz adapter → core

```http
POST /internal/buzz-fuel/intents
```

```json
{
  "buzz_channel_id": "<uuid>",
  "buzz_request_event_id": "<hex>"
}
```

Response:

```json
{
  "intent_id": "fi_<opaque>",
  "start_token": "<single-use opaque token>",
  "telegram_url": "https://t.me/MakeReel_xyz_bot?start=fuel_<token>",
  "status": "awaiting_claim"
}
```

Rules:

- fixed mission and prompt are server-side
- one Buzz request event creates at most one live intent
- intent TTL is 30 minutes
- no price is promised before TG user claim and canonical quote

### 2. Claim intent — Telegram bot → core

```http
POST /internal/buzz-fuel/intents/{start_token}/claim
```

```json
{
  "tg_user_id": 123,
  "tg_username": "optional"
}
```

Response:

```json
{
  "order_id": "<opaque>",
  "invoice_payload": "bf:<order-id>",
  "title": "Fuel a Buzz launch reel",
  "description": "5s · 720p · one generation",
  "amount_stars": 42,
  "intent_id": "fi_<opaque>"
}
```

Rules:

- create/get the TG account using existing accounts code
- first valid claimant binds the intent; repeat `/start` by the same TG user returns the same pending order
- another TG user receives a conflict/expired response
- call `tg_quote` for the fixed spec; use existing Stars conversion

### 3. Precheck — Telegram bot → core

```http
POST /internal/buzz-fuel/orders/{order_id}/precheck
```

Returns `{ "ok": true }` only for the bound user, exact amount, pending state, and unexpired order.

### 4. Fulfill — Telegram bot → core

```http
POST /internal/buzz-fuel/orders/{order_id}/fulfill
```

```json
{
  "tg_user_id": 123,
  "stars_charge_id": "<telegram-charge>",
  "stars_amount": 42
}
```

Behavior:

1. Verify user, amount, state, and charge replay.
2. Call existing `tg_generate` with payment=`stars`, expected quote amount, fixed spec, and client=`buzz_fuel`.
3. Record `job_id`, transaction reference, and intent status.
4. Return the same job on duplicate Telegram delivery.
5. Throw non-2xx before job acceptance so the bot follows its existing Stars refund closure.

### 5. Intent status — Buzz adapter → core

```http
GET /internal/buzz-fuel/intents/{intent_id}
```

Response states:

```text
awaiting_claim → awaiting_payment → paid/running → delivered
                                      └→ failed/refunded/expired
```

Delivered response includes a stable MakeReel share URL, not the raw gateway URL. Public response/logs must not expose TG user ID, username, charge ID, private keys, or payment headers.

#### Single terminal-state authority

Implement one core-owned path, provisionally named `_refresh_intent_from_job(intent)`, in `api/buzz_fuel.py`:

1. Read/refresh the canonical job through the existing `internal.poll_and_download`/manifest path.
2. Map accepted/submitted work to `paid` or `running`.
3. Map canonical `failed`/`cancelled` to the fuel failure/refund state.
4. Map to `delivered` only after the canonical job is `succeeded`, media is playable, and a stable MakeReel share URL exists.
5. Persist/dedupe the mapped transition under the intent lock.

Both the Telegram bot and Buzz adapter are observers of this core endpoint. They may trigger a refresh by reading status, but neither may submit a terminal intent state or invent `Delivered`. Concurrent reads must be idempotent.

## Change set A — MakeReel core

Repository:

```text
/Users/howard/Projects/x402/MakeReel/makereel-core
```

Planned files:

- **New:** `api/buzz_fuel.py`
- **Modify:** `api/main.py` to register internal router
- **Modify only if required:** `api/config.py` for `BUZZ_FUEL_ENABLED`, fixed prompt, bot username
- **Tests:** new `tests/test_buzz_fuel.py`

Implementation rules:

- keep the module thin; delegate quote, generate, poll, share, accounts, ledger to existing functions
- default feature flag off
- use an opaque random start token, never the sequential intent ID
- reuse `stars-gen:<charge_id>` idempotency
- no schema migration and no new payment ledger
- legacy miniapp order and TG generation behavior must remain byte-for-byte compatible at the API boundary

Exit criteria for change set A:

- unit/contract tests in `03-TEST-ACCEPTANCE.md` pass
- no network or real money required
- reviewer can exercise create → claim → precheck → fulfill with fake quote/generate dependencies

## Change set B — Telegram Mini App bot

Repository:

```text
/Users/howard/Projects/x402/MakeReel/makereel-tg-miniapp
```

Planned files:

- **Modify:** `bot/handlers.py`
- **Modify:** `bot/backend.py`
- **Optional extraction:** `bot/fuel_delivery.py` only when the first bounded working loop makes `handlers.py` materially harder to review
- **New:** tests under `tests/` and test dependency configuration if absent

Behavior:

1. Parse `fuel_<token>` using aiogram's command object; default `/start` remains unchanged.
2. Claim intent through core and call `send_invoice` with the returned canonical Stars amount and payload `bf:<order_id>`.
3. Route only `bf:` payloads to fuel precheck/fulfill; all existing payloads stay on current handlers.
4. On payment, acknowledge fuel immediately, retain a strong background task reference, and poll the existing internal job status.
5. Deliver video natively in Telegram; when failed/cancelled/timed out, call `refund_star_payment` exactly once.
6. Telegram copy says Stars, generation, and result only. It must not mention USDC, wallet, x402, or a crypto checkout.

Exit criteria for change set B:

- all legacy start/payment handler tests pass
- fuel payload dispatch cannot intercept existing invoice payloads
- unknown/expired/foreign intent never reaches an accepted pre-checkout
- duplicate successful-payment update never creates a second job
- TG observes terminal intent/job state from core; it does not own or push `delivered`

## Change set C — standalone Buzz adapter

Repository/folder:

```text
/Users/howard/orca/projects/buzz402/buzz-fuel-poc
```

Suggested files after implementation is authorized:

```text
Cargo.toml
src/main.rs       # lifecycle and channel subscription
src/buzz.rs       # derived NIP-42/kind 0/kind 9 code
src/makereel.rs   # internal Bearer client
src/model.rs      # intent/status response types
tests/            # command/state formatting tests
NOTICE            # Buzz countdown-bot attribution
```

This module split is not an acceptance gate. A mechanically derived `src/main.rs` under roughly 400 lines is acceptable for P0 when it is clearer and faster to review.

Dependency choice:

- use explicit crate versions matching Buzz's workspace
- for the 24-hour local POC, `buzz-sdk` may be a path dependency to `../buzz/crates/buzz-sdk`
- do not copy the relay or add this crate to Buzz's workspace
- a later standalone release must pin a Buzz git revision or replace the path dependency; that is not P0

Behavior:

1. Authenticate as a standalone bot; publish kind 0 profile; announce kind 9000 membership; subscribe to kind 9 scoped by `h` tag.
2. Accept only exact `/fuel` from events created after process start.
3. Ignore own events and dedupe by Buzz request event ID.
4. Create one intent through core and reply with the Telegram link.
5. Poll the core-owned intent state and publish each transition at most once: `Fueled`, optional `Running`, `Delivered`, or `Failed/Refunded`.
6. Include safe receipt identifiers: Buzz event ID, intent ID, job ID, stable share URL. Do not include payment headers, secrets, raw charge IDs, or TG identity.

Exit criteria for change set C:

- derived command parser and transition dedupe tests pass
- rejected/unauthorized relay events are surfaced clearly
- local Buzz channel displays accepted signed messages

## Ordered work and stop gates

| Step | Work | Stop gate |
|---|---|---|
| D0 | Record OPEN owners and run currently available no-code smokes | O1/O2 block Buzz work; O3 blocks TG deployment; O4 blocks only real C2 |
| A1 | Core data model + create/claim/precheck contracts | Unit tests green |
| A2 | Core fulfill via existing `tg_generate` + status/share | Idempotency/refund contract green |
| B1 | Bot deep-link and `send_invoice` dispatch | Legacy payment tests green |
| B2 | TG bounded poll/delivery/refund; extract a module only if useful | Failure tests green |
| C-1 | Buzz adapter `/fuel` → TG link | Local signed-message smoke green |
| I1 | Fake/staging integration: payment event → Buzz `Fueled` | **Checkpoint C1-dry** |
| I2 | Approved real run: Stars → A2 x402 → result | **Checkpoint C2** |
| R1 | Independent review and evidence sign-off | No publish before approval |

## Estimate after Day0 is green

| Change set | Estimate |
|---|---:|
| Core contract + tests | 2–3h |
| TG adapter + tests | 2–3h |
| Buzz adapter + tests | 2–3h |
| Integration, real run, recording | 2–4h plus model latency |

This is approximately 8–13 focused hours, not six, once independent tests and handoff quality are included. A rough demo may be faster; this plan optimizes for another person being able to verify it without trusting the implementer.

## 24-hour calendar cut after Day0

| Window | Primary outcome | Cut rule |
|---|---|---|
| H0–H2 | Core create/claim/precheck | Do not start Buzz polish |
| H2–H4 | Core fulfill + core-owned status refresh | Stop if money/idempotency contracts are not green |
| H4–H6 | TG `fuel_`/`bf:` dispatch + invoice | Link-only/generating acknowledgement is acceptable initially |
| H6–H8 | Buzz `/fuel` + signed `Fueled`; C1-dry evidence | Preserve a publishable, clearly labeled dry demo |
| H8–H12 | Approved C1-live/C2, delivery, refund evidence | Cut formatting and `Running`, never payment safety |
| H12–H24 | Complete tests, evidence, review handoff, edit buffer | No new features |

Any sibling six-hour estimate is superseded by this 8–13 focused-hour/24-hour calendar plan.
