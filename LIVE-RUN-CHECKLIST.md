# Live-run preflight checklist · Buzz Fuel POC

**Date:** 2026-07-22  
**Mode:** SAFE Railway deploy preflight complete · `BUZZ_FUEL_ENABLED=false` throughout  
**C1-dry:** independently ACCEPTED  
**This status:** **READY FOR HOWARD LIVE MONEY APPROVAL / BLOCKED** (money not authorized)

No Stars charge · no x402 settle · no Buzz adapter on prod · no git push/merge · flag remains **false**.

---

## Railway identity (exact)

| Resource | Value |
|---|---|
| Workspace | Howard Peng's Projects |
| Project | 👽 x402video.com / MakeReel.xyz |
| **Project ID** | `81ae07f8-a586-4ef0-95f8-ad291838bae0` |
| Environment | production |
| **Environment ID** | `d4068cb8-6f54-434f-bf70-76d3a1ce6b3b` |
| **makereel-core service ID** | `b4cd414d-4141-4f7a-b9e5-ea00f69faf51` |
| **makereel-tg-miniapp service ID** | `79c10a0a-cdc4-44d6-986a-1291d6d376e6` |
| Core public URL | https://makereel.xyz |
| Bot public URL | https://makereel-miniapp-tg-production.up.railway.app |
| Merchant | @MakeReel_xyz_bot |

---

## Deployments (terminal SUCCESS only)

| Service | Deployed commit | Deployment ID | Status |
|---|---|---|---|
| makereel-core | `8213b4b29cbbac0a036bd2d0a7f70268259ff00a` | **`c5ad6337-a1f0-4009-8557-6295de17f580`** | **SUCCESS** |
| makereel-tg-miniapp | `263276977470c0ebccb217f21ab8e869257ea48c` | **`cd29d805-107f-4d11-9222-ce12148a385b`** | **SUCCESS** |

`BUZZ_FUEL_ENABLED=false` set on core before/during deploy (verified in container).

### Rollback

| Service | Prior deployment to restore | Command sketch |
|---|---|---|
| core | `aebc3616-a92c-48d0-9429-3b6c4e6bd7b6` (pre-fuel production) | Railway dashboard → service → Deployments → Redeploy prior, **or** `railway deployment redeploy <id>` if available |
| bot | `b898bba2-5a2b-4ed6-a945-ba67515ed1b9` (prior SUCCESS) | same |
| flag | keep / set `BUZZ_FUEL_ENABLED=false` | `railway variable set BUZZ_FUEL_ENABLED=false --service makereel-core` |

---

## Post-deploy verification

| Check | Result |
|---|---|
| Core health `GET https://makereel.xyz/api/health` | `{"status":"ok","gateway":"https://api.x402video.com"}` |
| Bot health `GET …/health` | `{"status":"ok"}` |
| `t.me/MakeReel_xyz_bot` | HTTP 200 |
| In-container `BUZZ_FUEL_ENABLED` | `'false'` |
| Fuel create (internal, loopback) | **404** `{"detail":"Buzz Fuel is not enabled."}` · **fail-closed** |
| Legacy miniapp precheck route | **200** `{"ok":false}` for unknown order (route alive) |
| Buzz adapter on production | **not started** |

Legacy `/start` (Watch) and existing Stars payment paths remain the live bot surface; fuel invoice path cannot create intents while the flag is false.

---

## Production money preflight (read-only)

| Field | Value |
|---|---|
| Platform signer **public** address | **`0x294B4e2e543af7bD6291Bed5db277AD069061f3b`** |
| Signer Base USDC balance | **`$23.1719`** (`23171907` atomic) |
| Network | Base mainnet `eip155:8453` |
| Partner quote path | `POST /generate/seedance-fast/custom` + `X-Partner-Key` (no payment) |
| Partner quote amount | **`750000` atomic = `$0.75` USDC** |
| Stars conversion | `MINIAPP_STARS_PER_USDC=100`, `MINIAPP_STARS_MIN=10` |
| Exact Stars invoice (expected) | **`75★`** |
| Proposed max USDC spend | **`$0.90`** (= quote × 1.20) |
| Balance vs max | **OK** ($23.17 ≫ $0.90) |
| Fixed prompt | `Buzz launch reel, bold kinetic typography "BUZZ", dark neon workspace, 5s, cinematic` |

Public (non-partner) quote earlier was $1.05 / 105★ — production claim will use **partner** pricing above.

---

## Remaining Howard manual steps (before any live money)

1. Confirm signer address `0x294B…f3b` is the intended production payer.  
2. Approve **one** live run with max USDC **$0.90** and expect **75★** invoice.  
3. When ready for money path only: set `BUZZ_FUEL_ENABLED=true` (still no auto-charge).  
4. Optionally start Buzz fuel adapter against a chosen relay/channel (not done here).  
5. Open fuel deep link on intended TG account and pay Stars only after amount matches claim.  
6. If fulfillment fails: refund exactly once or continue to C2 under separate approval.  
7. After test window: set `BUZZ_FUEL_ENABLED=false` and stop adapter.

---

## Approval block (fill and return for live money)

```text
I authorize a SINGLE controlled LIVE MONEY run of Buzz Fuel P0 under these exact parameters:

- Railway project/environment/service IDs:
    project=81ae07f8-a586-4ef0-95f8-ad291838bae0
    environment=d4068cb8-6f54-434f-bf70-76d3a1ce6b3b (production)
    makereel-core=b4cd414d-4141-4f7a-b9e5-ea00f69faf51
    makereel-tg-miniapp=79c10a0a-cdc4-44d6-986a-1291d6d376e6

- successful deployment IDs (already live, flag OFF):
    core=c5ad6337-a1f0-4009-8557-6295de17f580 @ 8213b4b29cbbac0a036bd2d0a7f70268259ff00a
    bot=cd29d805-107f-4d11-9222-ce12148a385b @ 263276977470c0ebccb217f21ab8e869257ea48c

- deployed commit SHAs:
    core=8213b4b29cbbac0a036bd2d0a7f70268259ff00a
    bot=263276977470c0ebccb217f21ab8e869257ea48c

- signer public address: 0x294B4e2e543af7bD6291Bed5db277AD069061f3b
- signer Base USDC balance: $23.1719 (as of preflight)
- exact partner quote: $0.75 USDC (750000 atomic) on eip155:8453
- exact Stars amount: 75★
- proposed maximum USDC spend: $0.90 (quote × 1.20)

- rollback deployment IDs/commands:
    core → redeploy aebc3616-a92c-48d0-9429-3b6c4e6bd7b6 (or prior known-good)
    bot  → redeploy b898bba2-5a2b-4ed6-a945-ba67515ed1b9
    always: railway variable set BUZZ_FUEL_ENABLED=false --service makereel-core

- remaining Howard manual steps:
    [ ] set BUZZ_FUEL_ENABLED=true for the test window only
    [ ] start Buzz adapter only if live Buzz messaging is required
    [ ] pay one Stars invoice at the claim amount (expect 75★)
    [ ] if fail: refund once OR continue to C2 under separate approval
    [ ] disable flag + stop adapter after the window

I do NOT authorize multi-run spend, production marketing claims, or C2 without a separate explicit approval.

Signed: _______________  Date: _______________
```

---

**Status: READY FOR HOWARD LIVE MONEY APPROVAL / BLOCKED**

Do **not** enable `BUZZ_FUEL_ENABLED` or run C1-live/C2 until the approval block is signed.
