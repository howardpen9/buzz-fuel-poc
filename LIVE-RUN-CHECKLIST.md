# Live-run preflight checklist · Buzz Fuel POC

**Purpose:** Operator template for a controlled money-window preflight.  
**Public tree policy:** No production Railway IDs, balances, or live share URLs.  
Fill a **private** copy (outside this repo, or under gitignored `evidence/`) before any live run.

**Defaults:** `BUZZ_FUEL_ENABLED=false` until an explicit money authorization exists.

No Stars charge · no x402 settle · no public ship · flag remains **false** until authorized.

---

## Identity (private fill-in)

| Resource | Value (private) |
|---|---|
| Cloud project / environment | `<redacted — private ops notes>` |
| makereel-core service | `<redacted>` |
| makereel-tg-miniapp service | `<redacted>` |
| Core public URL | `https://makereel.xyz` (or staging host) |
| Merchant | `@MakeReel_xyz_bot` |

---

## Deployments

| Service | Deployed commit | Deployment ID | Status |
|---|---|---|---|
| makereel-core | `<git sha>` | `<private>` | SUCCESS required |
| makereel-tg-miniapp | `<git sha>` | `<private>` | SUCCESS required |

Confirm `BUZZ_FUEL_ENABLED=false` in the running core container before and after preflight.

### Rollback sketch

| Service | Action |
|---|---|
| core / bot | Redeploy last known-good deployment from the cloud dashboard |
| flag | `BUZZ_FUEL_ENABLED=false` on makereel-core |

---

## Post-deploy verification

| Check | Expected |
|---|---|
| Core health | HTTP 200, healthy payload |
| Bot health | HTTP 200 |
| `t.me/MakeReel_xyz_bot` | Reachable |
| In-container `BUZZ_FUEL_ENABLED` | `false` |
| Fuel create (internal) | **404** fail-closed while flag off |
| Buzz adapter on production | not started unless authorized |

---

## Money preflight (read-only)

| Field | Value (private) |
|---|---|
| Platform signer **public** address | `<confirm intended payer — do not paste private keys>` |
| Signer USDC balance | `<private snapshot>` |
| Network | Base mainnet `eip155:8453` (or documented staging) |
| Expected Stars invoice | from live quote + Stars conversion (**never hard-code in adapter**) |
| Proposed max USDC spend | quote × safety factor (e.g. 1.20) |
| Fixed prompt / SKU | P0 fixed Seedance Fast · 5s · 720p |

---

## Remaining operator steps (before any live money)

1. Confirm the platform signer public address is the intended payer.  
2. Approve **one** live run with an explicit max USDC and expected Stars amount.  
3. When ready: set `BUZZ_FUEL_ENABLED=true` for the test window only.  
4. Optionally start the Buzz fuel adapter against a chosen relay/channel.  
5. Open the fuel deep link and pay only if the invoice amount matches claim.  
6. If fulfillment fails: refund exactly once, or continue under separate approval.  
7. After the window: set `BUZZ_FUEL_ENABLED=false` and stop the adapter.

---

## Approval block (template)

```text
I authorize a SINGLE controlled LIVE MONEY run of Buzz Fuel P0 under these parameters:

- environment / service identifiers: <private>
- deployed commit SHAs: core=<sha> bot=<sha>
- signer public address: <address>
- expected quote / Stars: <from live quote>
- max USDC spend: <cap>
- rollback: redeploy prior known-good + BUZZ_FUEL_ENABLED=false
- I do NOT authorize multi-run spend, production marketing claims, or C2 without a separate explicit approval.

Signed: _______________  Date: _______________
```

---

## Evidence

Store run artifacts under gitignored `evidence/<timestamp>/` only.  
Never commit screenshots, charge IDs, or raw payment payloads to the public tree.
