# Implementer handoff · Buzz Fuel POC · 2026-07-22

**Role:** implementer (not reviewer). Does **not** mark C1-live/C2 accepted or “shipped.”

**Status:** `READY FOR INDEPENDENT REVIEW` for automated suites + simulated C1-dry core path only.

---

### Checkouts

| Repo | Branch | Start SHA | End SHA | Dirty/untracked preserved |
|---|---|---|---|---|
| makereel-core | `poc/buzz-fuel-core` | `ba8bd89282e6938457323cbe07d5411a89648106` | uncommitted WIP on branch | implementation files only |
| makereel-tg-miniapp | `poc/buzz-fuel-bot` | `98536a5c374643a892db5b523217f6387add72e4` | uncommitted WIP on branch | **`docs/` still untracked, untouched** |
| buzz402 / buzz-fuel-poc | `poc/buzz-fuel-adapter` | `77cbfdae27366a4503ab451032e21002da424d03` | uncommitted POC tree | new adapter files |
| Buzz upstream | `main` (read-only) | `7e34bee62cacaa9d8a96c14d5892a471b59a1983` | unchanged | no edits |
| x402 gateway | `feat/seedance-2-0-mini` (read-only) | `fb4211b4be0a6be4c2f1b5a89460ad5513e3bb20` | unchanged | no edits |

---

### Change sets

| Set | Files | Contract implemented | Status |
|---|---|---|---|
| A Core | `api/buzz_fuel.py` (new), `api/config.py`, `api/main.py`, `tests/test_buzz_fuel.py` | create/claim/precheck/fulfill + core-owned `_refresh_intent_from_job` | automated **18 PASS** |
| B Telegram | `bot/handlers.py`, `bot/backend.py`, `tests/test_fuel_handlers.py`, `pyproject.toml` | `fuel_` deep link, `bf:` dispatch, invoice, refund/poll observer | automated **15 PASS** |
| C Buzz | `buzz-fuel-poc/` Cargo crate (`main.rs`, `model.rs`, `makereel.rs`, NOTICE) | exact `/fuel` → intent → signed status poller | **fmt/clippy/test PASS** (5 unit) |

---

### Gates

| Gate | PASS / FAIL / NOT RUN | Evidence |
|---|---|---|
| Day0 safe gates (D0.1–D0.6) | **NOT RUN** | Need Howard/operator: O1 relay+channel, O3 bot health, O4 liquidity |
| Automated tests | **PASS** | `evidence/*/test-results.txt` |
| C1-dry fake/staging (core path) | **PASS (SIMULATED / NON-C2)** | `evidence/*/c1-dry-core-path.json` |
| C1-dry full Buzz WS + TG | **NOT RUN** | Blocked on O1/O2 |
| C1-live real Stars | **NOT RUN** | Needs explicit Howard approval |
| C2 real money | **NOT RUN** | Needs explicit Howard approval |

---

### Deviations

| Planned | Code reality | Conservative choice | Approval needed? |
|---|---|---|---|
| Optional `fuel_delivery.py` extraction | delivery kept in `handlers.py` (~poll loop) | Plan allows optional extract | No |
| Optional multi-module Rust split | `main` + `model` + `makereel` | Clearer than one 400-line file | No |
| `Running` Buzz message | off by default (`BUZZ_FUEL_PUBLISH_RUNNING`) | PARK until useful | No |
| `mark-refunded` helper on core | added so TG can report refund once | adapters still cannot write `delivered` | No |
| Full matrix A14 live share | unit-tested with fake poll/share | real share needs C2 | No |

---

### Not performed

- Deployment of core/bot/adapter
- Real Stars payment
- Real x402 settlement / platform signer spend
- External publishing / X posts
- Day0 production bot screenshot
- Buzz relay smoke (needs O1 channel + bot key)

---

### Reviewer entry point

1. Evidence directory: `buzz-fuel-poc/evidence/20260722T085710Z/` (gitignored)
2. First commands to reproduce:

```bash
cd /Users/howard/Projects/x402/MakeReel/makereel-core
uv run pytest tests/test_buzz_fuel.py -v

cd /Users/howard/Projects/x402/MakeReel/makereel-tg-miniapp
uv run pytest tests/ -v

cd /Users/howard/orca/projects/buzz402/buzz-fuel-poc
cargo fmt -- --check && cargo clippy -- -D warnings && cargo test

cd /Users/howard/Projects/x402/MakeReel/makereel-core
uv run python /Users/howard/orca/projects/buzz402/buzz-fuel-poc/scripts/c1_dry_core_path.py
```

3. Known blockers before live demo:
   - **O1:** `BUZZ_RELAY_URL` + `BUZZ_CHANNEL_ID`
   - **O2:** standalone bot write smoke
   - **Howard approval** for C1-live/C2 real money
   - Commits not yet made (working tree dirty by design until you want commits)

---

### Rollback readiness

1. `BUZZ_FUEL_ENABLED=false` (default)
2. Stop Buzz fuel adapter process
3. Deploy without fuel branches → legacy miniapp paths unchanged
4. No gateway / Buzz upstream rollback needed (untouched)

---

### Implementation log (concise)

| Planned | Discovered | Choice | Tests |
|---|---|---|---|
| Core intent store like miniapp orders | miniapp `Order` + lock/TTL pattern | process-local dict + lock | A01–A16 |
| Quote via existing path | `gateway.request_quote` + `stars_for_atomic` | no hard-coded Stars | A05 |
| Fulfill via `tg_generate` | client=`buzz_fuel`, expected atomic | non-2xx → failed + refund duty | A10–A13 |
| Terminal authority | poll_and_download + set_share | only core writes delivered | A14/A16 |
| TG `fuel_` + `bf:` | CommandObject args; payload prefix | legacy path untouched | B01–B11 |
| Buzz `/fuel` | countdown-bot AUTH/profile/sub | exact match only; poller reconnects to publish | C01–C06 unit |

---

**Next for Howard (under 2 min):** supply O1 (`BUZZ_RELAY_URL` + channel UUID) if you want full C1-dry with signed Buzz messages; or authorize commits on the three POC branches.
