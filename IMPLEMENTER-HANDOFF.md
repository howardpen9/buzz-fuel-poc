# Implementer handoff · Buzz Fuel POC · 2026-07-22

**Role:** implementer (not reviewer). Does **not** mark C1-live/C2 accepted or “shipped.”

**Status:** `PARTIAL / READY FOR NEXT GATE`  
Reviewer preliminary: automated tests acceptable; full Buzz C1-dry, real Stars, real x402, and delivery remain unverified.

---

### Checkouts (frozen local commits)

| Repo | Branch | Start SHA | **Commit SHA** | Dirty/untracked preserved |
|---|---|---|---|---|
| makereel-core | `poc/buzz-fuel-core` | `ba8bd89282e6938457323cbe07d5411a89648106` | **`8213b4b29cbbac0a036bd2d0a7f70268259ff00a`** | clean (allowlist committed) |
| makereel-tg-miniapp | `poc/buzz-fuel-bot` | `98536a5c374643a892db5b523217f6387add72e4` | **`263276977470c0ebccb217f21ab8e869257ea48c`** | **`docs/` still untracked, untouched** |
| buzz402 / buzz-fuel-poc | `poc/buzz-fuel-adapter` | `77cbfdae27366a4503ab451032e21002da424d03` | **`078e0a3b3abbe106474b6f4068aa542a8de89a29`** (code) · handoff docs tip = `git rev-parse HEAD` on branch | `buzz/`, `research/` untracked, **not** in commits |
| Buzz upstream | `main` (read-only) | `7e34bee62cacaa9d8a96c14d5892a471b59a1983` | unchanged | no edits |
| x402 gateway | `feat/seedance-2-0-mini` (read-only) | `fb4211b4be0a6be4c2f1b5a89460ad5513e3bb20` | unchanged | no edits |

**Not pushed. Not merged. Not deployed.**

Allowlist-only commits (`git add .` was not used):

- Core: `api/buzz_fuel.py`, `api/config.py`, `api/main.py`, `tests/test_buzz_fuel.py`
- Bot: `bot/backend.py`, `bot/handlers.py`, `pyproject.toml`, `uv.lock`, `tests/*`
- POC: `buzz-fuel-poc/{src,docs,scripts,Cargo.toml,NOTICE,README,AGENTS,IMPLEMENTER-HANDOFF,.gitignore}` only

---

### Change sets

| Set | Files | Contract implemented | Status |
|---|---|---|---|
| A Core | `api/buzz_fuel.py` (new), `api/config.py`, `api/main.py`, `tests/test_buzz_fuel.py` | create/claim/precheck/fulfill + core-owned `_refresh_intent_from_job` | automated **18 PASS** · committed |
| B Telegram | `bot/handlers.py`, `bot/backend.py`, `tests/test_fuel_handlers.py`, `pyproject.toml` | `fuel_` deep link, `bf:` dispatch, invoice, refund/poll observer | automated **15 PASS** · committed |
| C Buzz | `buzz-fuel-poc/` Cargo crate (`main.rs`, `model.rs`, `makereel.rs`, NOTICE) | exact `/fuel` → intent → signed status poller | **fmt/clippy/test PASS** · committed |

---

### Gates

| Gate | PASS / FAIL / NOT RUN | Evidence |
|---|---|---|
| Day0 D0.1/D0.2 Buzz relay write | **NOT RUN** | Blocked on O1 choice + Howard publish approval |
| Day0 D0.3/D0.4 TG bot health | **NOT RUN** | Operator |
| Day0 D0.5/D0.6 payer/SKU | **NOT RUN** | Blocks real C2 only |
| Automated tests | **PASS** | `evidence/*/test-results.txt` |
| C1-dry core path (simulated payment) | **PASS (SIMULATED / NON-C2)** | `evidence/*/c1-dry-core-path.json` |
| C1-dry full Buzz WS + TG | **NOT RUN** | Needs O1 + O2 approval |
| C1-live real Stars | **NOT RUN** | Needs explicit Howard approval |
| C2 real money | **NOT RUN** | Needs explicit Howard approval |

---

### O1 — read-only resolution (2026-07-22)

Searched local countdown-bot docs, Buzz `.env.example`, compose examples, research notes, and presence of local `.env` files (secret values never printed).

| Candidate | Relay URL | Channel UUID | Notes |
|---|---|---|---|
| **A · local demo (default)** | `ws://localhost:3000` | **not found locally** | Documented default in countdown-bot + `buzz/.env.example` `RELAY_URL`. Requires a running local Buzz relay + a channel UUID you create/copy in the desktop app. |
| B · compose production-style example | `wss://buzz.example.com` | n/a | Placeholder only in `deploy/compose/.env.example` — **not a real target**. |
| C · remote/prod relay | (not present in local non-secret config) | (none) | No committed real channel UUID or production relay found. |

**No single clearly intended test channel UUID exists in the tree.**  
**Howard must choose:**

1. Relay: recommend **`ws://localhost:3000`** if a local `just relay` (or equivalent) is up; otherwise supply the real demo `wss://…` URL.  
2. Channel UUID: create/copy a dedicated demo channel and paste the UUID.

---

### O2 / full C1-dry — prepared commands (DO NOT RUN until Howard approves)

**Stop line:** implementer will not connect a write path or publish until Howard replies with exact relay URL, channel UUID, and permission to publish one labeled test message.

#### Prerequisites (non-secret checklist)

- [ ] Local or demo Buzz relay reachable at the approved URL  
- [ ] Channel UUID approved  
- [ ] Standalone bot key available in env (never commit): `BUZZ_BOT_PRIVATE_KEY`  
- [ ] Bot pubkey allowlisted / channel membership for standalone auth  
- [ ] MakeReel core running with `BUZZ_FUEL_ENABLED=true` and matching `INTERNAL_API_KEY`  
- [ ] TG bot **not** required for simulated C1-dry payment (use core mock fulfill path)

#### Step 0 — generate a throwaway bot key (local only; do not commit)

```bash
# Example only — use any nsec/hex you control; never paste into chat/commits
# openssl rand -hex 32   # or `nak key generate` if available
```

#### Step 1 — O2 smoke: standalone AUTH + one labeled test message

After Howard approves `RELAY` + `CHANNEL`:

```bash
export BUZZ_RELAY_URL='<APPROVED_RELAY_URL>'          # e.g. ws://localhost:3000
export BUZZ_CHANNEL_ID='<APPROVED_CHANNEL_UUID>'
export BUZZ_BOT_PRIVATE_KEY='<local-only-bot-nsec-or-hex>'
export BUZZ_BOT_AUTH_MODE=standalone

# Optional: countdown-bot first (smaller surface) for AUTH+write proof
cd /Users/howard/orca/projects/buzz402/buzz
cargo run --manifest-path examples/countdown-bot/Cargo.toml
# In the channel UI, send: !countdown 3
# PASS: AUTH accepted + signed kind 9 reply event ID visible
```

#### Step 2 — full C1-dry Buzz WebSocket + **simulated** Telegram payment

```bash
# Terminal A — MakeReel core with fuel enabled (no real gateway required if you only
# exercise create/link; for fulfill use the simulated script or mock).
cd /Users/howard/Projects/x402/MakeReel/makereel-core
export BUZZ_FUEL_ENABLED=true
export INTERNAL_API_KEY='<local-dev-key>'
export BUZZ_FUEL_BOT_USERNAME=MakeReel_xyz_bot
# start your usual uvicorn/dev entry for makereel-core

# Terminal B — Buzz fuel adapter (writes only after Howard approval)
cd /Users/howard/orca/projects/buzz402/buzz-fuel-poc
export BUZZ_RELAY_URL='<APPROVED_RELAY_URL>'
export BUZZ_CHANNEL_ID='<APPROVED_CHANNEL_UUID>'
export BUZZ_BOT_PRIVATE_KEY='<local-only-bot-nsec-or-hex>'
export BUZZ_BOT_AUTH_MODE=standalone
export MAKEREEL_API_URL='http://127.0.0.1:8001'   # or your core URL
export INTERNAL_API_KEY='<same-as-core>'
cargo run --manifest-path Cargo.toml

# Terminal C — in Buzz channel UI, post exactly:
#   /fuel
# Expect: signed bot reply containing t.me/MakeReel_xyz_bot?start=fuel_<token>
# Label: C1-DRY / TEST — not real Stars

# Simulated payment (no Telegram charge) — core path only:
cd /Users/howard/Projects/x402/MakeReel/makereel-core
uv run python /Users/howard/orca/projects/buzz402/buzz-fuel-poc/scripts/c1_dry_core_path.py --write-evidence
# Or: claim+fulfill via TestClient / internal API with mocked tg_generate
# Mark all artifacts: C1-DRY · SIMULATED · NON-C2
```

#### Exact one-liner template Howard can approve

```text
APPROVE O2/C1-dry write:
  relay = <url>
  channel = <uuid>
  allow one signed TEST kind-9 message from standalone fuel bot
  no real Stars / no x402 / no deploy
```

---

### Deviations

| Planned | Code reality | Conservative choice | Approval needed? |
|---|---|---|---|
| Optional `fuel_delivery.py` extraction | delivery kept in `handlers.py` | Plan allows optional extract | No |
| Optional multi-module Rust split | `main` + `model` + `makereel` | Clearer than one file | No |
| `Running` Buzz message | off by default | PARK | No |
| `mark-refunded` helper on core | TG reports refund once | adapters cannot write `delivered` | No |

---

### Not performed

- Push / merge / deploy  
- Real Stars payment  
- Real x402 settlement  
- External publishing  
- Writing any event to a Buzz relay in this tranche  
- Printing or committing private keys  

---

### Reviewer / next-gate entry

1. Reproduce automated suites from commit SHAs above.  
2. Howard answers O1: exact relay URL + channel UUID.  
3. Howard grants O2: one labeled test message.  
4. Implementer runs O2 then full C1-dry WS path; still no real money.

---

### Rollback readiness

1. `BUZZ_FUEL_ENABLED=false` (default)  
2. Stop Buzz fuel adapter  
3. Branches are local-only — drop branches or revert commits if needed  
4. No gateway / Buzz upstream rollback  

---

**Next (Howard, under 2 minutes):** reply with  
`relay=<…>` · `channel=<uuid>` · `OK to publish one TEST kind-9`  
or reject and keep PARTIAL frozen.
