# Live-run preflight checklist · Buzz Fuel POC

**Date:** 2026-07-22  
**Mode:** READ-ONLY preflight only — no deploy, no Stars charge, no x402 settle, no push/merge.  
**Prior gate:** Full local C1-dry independently **ACCEPTED**.  
**This document status:** READY FOR HOWARD LIVE-RUN APPROVAL / **BLOCKED** (see blockers).

---

## A. Soft defects closed (this tranche)

| Item | Action |
|---|---|
| Terminal-state regression | `keys_for_phase` / `plan_publications` + test: first poll `delivered` → Fueled then Delivered once |
| Evidence exporter | `scripts/export_buzz_evidence.py` preserves `sig`/`signed_event` when present; never private keys; records `sig_status` instead of `sig:null` |
| Pre-fix evidence | `evidence/20260722T101134Z/PRE_FIX_ATTEMPT.md` + old core-path bundle labeled **PRE-FIX ATTEMPT · NOT ACCEPTANCE EVIDENCE** |
| Version notes | `versions.txt` distinguishes `code_freeze_sha` vs `docs_tip_sha` (no DIRTY) |

---

## B. Production inventory (read-only)

### 1. Telegram merchant `@MakeReel_xyz_bot`

| Check | Result |
|---|---|
| Public page `https://t.me/MakeReel_xyz_bot` | HTTP 200 |
| Railway service | `makereel-tg-miniapp` · **Online** |
| Public URL | `https://makereel-miniapp-tg-production.up.railway.app` |
| Health | `GET /health` → `{"status":"ok"}` |
| Webhook | `WEBHOOK_URL` set (production) |
| `BOT_TOKEN` | set (not printed) |
| `MAKEREEL_API_URL` | `https://makereel.xyz` |
| `MINIAPP_BOT_USERNAME` (core) | `MakeReel_xyz_bot` |
| Deployed code | Production service is **not** on `poc/buzz-fuel-bot` until Howard deploys that branch |

**Manual Telegram check still required from Howard:** open `@MakeReel_xyz_bot` → `/start` (legacy Watch) works.

### 2. MakeReel core

| Check | Result |
|---|---|
| Railway service | `makereel-core` · **Online** |
| Public URL | `https://makereel.xyz` |
| Health | `GET /api/health` → `{"status":"ok","gateway":"https://api.x402video.com"}` |
| Gateway | `GATEWAY_BASE=https://api.x402video.com` |
| `PLATFORM_SIGNER_KEY` | **set** in Railway (not printed) |
| `INTERNAL_API_KEY` | set |
| `MINIAPP_BOT_TOKEN` | set |
| **`BUZZ_FUEL_ENABLED`** | **NOT present in production env** |
| **`BUZZ_FUEL_PROMPT`** | **NOT present** |
| **`BUZZ_FUEL_BOT_USERNAME`** | **NOT present** (defaults in code to MakeReel_xyz_bot if deployed) |
| **`BUZZ_FUEL_INTENT_TTL_SECONDS`** | **NOT present** (defaults to 1800) |
| Deployed code | Production is **not** on `poc/buzz-fuel-core` until Howard deploys |

### 3. Platform signer (preflight)

| Check | Result |
|---|---|
| Network for settlement | **Base mainnet** · `eip155:8453` (from live gateway quote) |
| USDC asset | `0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913` |
| Gateway payTo | `0xC81a36fCdf531b6eD7ae8694220d59F9D3D0128B` |
| Local `.env` signer address (dev machine only) | `0x597a57263ab465F8c4272256a9C92891DaeB7e1a` |
| Production signer address | **Not independently confirmed** (Railway key set; may or may not match local) |
| USDC balance | **NOT VERIFIED** — public Base RPCs returned 403/unauthorized from this environment |
| Enabled | Production has `PLATFORM_SIGNER_KEY` set → payer path can enable when code uses it |

**Howard/operator must confirm:** production signer address + USDC balance ≥ one quote + 20% buffer before approving spend.

### 4. Canonical SKU quote (no payment submitted)

Probe (unpaid POST, expect 402):

```http
POST https://api.x402video.com/generate/seedance-fast/custom
Content-Type: application/json
X-Client: makereel/1.0

{
  "prompt": "Buzz launch reel, bold kinetic typography \"BUZZ\", dark neon workspace, 5s, cinematic",
  "duration": 5,
  "resolution": "720p",
  "ratio": "16:9",
  "generate_audio": false
}
```

| Field | Value |
|---|---|
| HTTP | **402 Payment Required** (quote only — no settlement) |
| Amount atomic | `1050000` |
| Price USDC | **$1.05** |
| Network | `eip155:8453` |
| Fixed SKU path `/5s-720p` | Rejected body fields (use `/custom` with duration/resolution — matches makereel-core) |

**Pricing caveat:** production core has `GATEWAY_PARTNER_KEY` set. This probe did **not** send the partner key, so Stars may be **lower** at live claim time. Invoice amount must be taken from claim response, never assumed.

### 5. Expected Stars and max USDC

| Metric | Value |
|---|---|
| Stars conversion config (prod) | `MINIAPP_STARS_PER_USDC=100`, `MINIAPP_STARS_MIN=10` |
| Expected Stars (from public quote) | **105★** · formula `max(10, ceil(1.05 * 100))` |
| Max USDC spend (quote + 20%) | **$1.26** |
| Partner-priced Stars | **unknown until claim** after deploy |

### 6. Rollback (exact)

```bash
# 1) Feature flag off (Railway makereel-core)
BUZZ_FUEL_ENABLED=false
# or unset the variable

# 2) Stop standalone Buzz fuel adapter process (wherever it runs)
#    kill the buzz-fuel-bot process / disable its service

# 3) Verify fail-closed
#    POST /internal/buzz-fuel/intents → 404 when flag off
#    /start without fuel_ still opens Watch

# 4) If needed, redeploy previous production commits
#    (not the poc/* branches)
```

No gateway or Buzz upstream rollback (P0 does not modify them).

### 7. Actions that require Howard on Telegram (manual)

1. Approve and perform deploy of **core** then **miniapp bot** POC branches (or cherry-picks) to production.  
2. Set production env: `BUZZ_FUEL_ENABLED=true`, optional `BUZZ_FUEL_PROMPT`, `BUZZ_FUEL_BOT_USERNAME=MakeReel_xyz_bot`.  
3. Confirm `@MakeReel_xyz_bot` `/start` still works after deploy.  
4. Confirm platform USDC balance ≥ **$1.26** on Base for the production signer.  
5. Open the live `t.me/MakeReel_xyz_bot?start=fuel_…` link on the intended TG account.  
6. Pay the Stars invoice when amount matches claim (expect ~105★ unless partner pricing differs).  
7. Approve C1-live → C2 continuation or one-time refund if fulfillment fails.  
8. Choose Buzz relay/channel for any live Buzz-side demo (local C1-dry channel is not production).

---

## Blockers before C1-live / C2

| # | Blocker | Owner |
|---|---|---|
| B1 | Production **does not** have `BUZZ_FUEL_ENABLED` or fuel code deploy | Howard deploy |
| B2 | Production bot/core not on `poc/buzz-fuel-*` commits | Howard deploy |
| B3 | Platform USDC balance **not verified** from this agent environment | Howard/operator |
| B4 | Production signer address **not confirmed** vs local | Howard |
| B5 | Live Buzz relay/channel for public demo **not chosen** (O1 for prod surface) | Howard |
| B6 | Explicit spend approval (Stars + max USDC) | Howard |

---

## Approval block (fill and return)

```text
I authorize a SINGLE controlled live run of Buzz Fuel P0 under these exact parameters:

- production services to deploy:
    [ ] makereel-core @ 8213b4b29cbbac0a036bd2d0a7f70268259ff00a (or approved equivalent)
    [ ] makereel-tg-miniapp @ 263276977470c0ebccb217f21ab8e869257ea48c (or approved equivalent)
    [ ] buzz-fuel adapter (where?): _______________________________
- target network: Base mainnet (eip155:8453)
- platform payer address: 0x________________________________________
- Stars amount: _____ ★ (must match claim; public probe suggested 105)
- maximum USDC spend: $_____ (probe +20% buffer suggested $1.26)
- Buzz relay/channel: relay=________________ channel=________________
- fixed prompt: Buzz launch reel, bold kinetic typography "BUZZ", dark neon workspace, 5s, cinematic
    [ ] keep as-is   [ ] replace with: _______________________________
- rollback: BUZZ_FUEL_ENABLED=false + stop adapter + verify legacy /start
- manual Howard steps:
    [ ] deploy core then bot with flag on
    [ ] confirm balance + signer address
    [ ] pay one Stars invoice on @MakeReel_xyz_bot
    [ ] if fail: refund exactly once or continue to C2 delivery

Signed: _______________  Date: _______________
```

---

**Status: READY FOR HOWARD LIVE-RUN APPROVAL / BLOCKED**

Do not run C1-live or C2 until the approval block is completed and blockers B1–B6 are cleared.
