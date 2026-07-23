# 01 — Local source and reuse map

This document maps semantics, not just filenames. Copy the behavior indicated; do not transplant unrelated product logic.

## A. Buzz signed bot adapter

Source root:

```text
../buzz
```

| Source | Reuse | Planned target |
|---|---|---|
| `examples/countdown-bot/README.md` | standalone vs owner-attested auth contract; relay/channel membership distinction | deployment notes |
| `examples/countdown-bot/Cargo.toml` | minimal dependency shape | standalone POC Cargo manifest; replace workspace deps with explicit versions/path dependency |
| `examples/countdown-bot/src/main.rs::Config` | `BUZZ_RELAY_URL`, channel, bot key, auth mode | POC config |
| `connect_and_authenticate` / `build_auth_event` | NIP-42 challenge and signed AUTH | unchanged semantics |
| `publish_profile` | kind 0 agent profile | rename to MakeReel Fuel Bot |
| `announce_channel_membership` | best-effort kind 9000 role=bot | same |
| `subscribe_to_channel` | kind 9 + `h` tag channel subscription | same |
| `maybe_reply` / `wait_for_ok` | ignore self, publish signed kind 9, confirm relay acceptance | adapt to `/fuel` only |

Do not import:

- countdown/fibonacci commands
- ACP, MCP, managed-agent lifecycle
- Desktop code
- new relay endpoints or event kinds

License note: Buzz is Apache-2.0. Preserve attribution/license notices when deriving code.

## B. Telegram Stars merchant

Source root:

```text
<makereel-tg-miniapp-checkout>
```

| Source | Reuse | Planned change |
|---|---|---|
| `bot/handlers.py::start` | existing `/start` fallback and crypto-free copy | parse `fuel_<token>` first; default behavior unchanged |
| `bot/handlers.py::pre_checkout` | pre-check before Telegram charges | dispatch `bf:` payloads to fuel precheck; legacy orders unchanged |
| `bot/handlers.py::on_paid` | successful-payment authority and refund closure | dispatch `bf:` payloads to fuel fulfill; legacy orders unchanged |
| `bot/backend.py` | single authenticated path from bot to MakeReel | add internal fuel API client functions |
| `bot/main.py` | production webhook/polling and allowed updates | no new bot token, webhook, or service |

Additional owned source to port selectively:

```text
<makereel-tg-chat-checkout>/bot/generation.py
<makereel-tg-chat-checkout>/bot/api_client.py
```

Reuse only:

- strong task references for background polling
- 8-second bounded polling
- failed/cancelled/timed-out refund behavior
- native Telegram video delivery
- job status/video internal API calls

Do not import prompt unlock, referrals, balance hints, keyboards, or remix UI.

## C. MakeReel payment and orchestration authority

Source root:

```text
<makereel-core-checkout>
```

| Source | Reuse | Planned change |
|---|---|---|
| `api/internal.py::TgQuoteRequest` / `tg_quote` | fixed spec validation and canonical x402 quote | call, do not duplicate |
| `api/internal.py::TgGenerateRequest` / `tg_generate` | account lock, Stars idempotency, payer signing, gateway submit, manifest | call with `client="buzz_fuel"` |
| `api/payer.py` | x402 v2 EIP-3009 platform signer | unchanged |
| `api/gateway.py` | 402 quote, paid retry, job status | unchanged |
| `api/ledger.py` | `stars-gen:<charge_id>` idempotency and events | reuse; add only a `buzz_fuel` event if useful |
| `api/miniapp.py::Order` / `_orders` pattern | short-lived payer-bound order and safe precheck/fulfill | mirror in a focused `api/buzz_fuel.py`, not another payment ledger |
| `api/internal.py::tg_job_status` / `tg_job_video` / `tg_share_job` | poll, archive, stable share URL | reuse for TG/Buzz delivery |
| `api/main.py` router registration | one new internal router | register behind feature flag |

Planned fixed generation spec:

```json
{
  "model_family": "seedance-fast",
  "duration": 5,
  "resolution": "720p",
  "ratio": "16:9",
  "generate_audio": false,
  "prompt": "<Howard-approved server-side launch prompt>"
}
```

The quote decides the Stars amount. The POC must not supply its own price table.

## D. x402 gateway contract

Source root:

```text
<x402video-gateway-checkout>
```

| Source | Use | Modification |
|---|---|---|
| `scripts/buyer-test.ts` | Day0 settlement smoke and evidence shape | none |
| `src/config.ts` SKU `seedance-fast-5s-720p` | canonical route contract | none |
| `src/index.ts` | strict request and x402 middleware | none |
| `src/jobs/store.ts` | job/idempotency authority | none |

Important rules inherited from gateway:

- filter before payment
- canonical price only
- upstream keys stay inside gateway
- exact request body must be reused after the 402 quote
- one idempotency key produces at most one live upstream job
- gateway output URLs expire; MakeReel must mint/return a stable share URL

## E. Copy/call matrix

| Asset | Strategy |
|---|---|
| Countdown bot | derive into this folder with attribution |
| MakeReel quote/generate/ledger | call in place; never copy |
| Mini App payment handlers | extend in place; preserve legacy branches |
| TG chat poll/delivery | port the minimal background delivery pattern into Mini App bot |
| Gateway buyer test | execute only as approved smoke; never embed in product code |

