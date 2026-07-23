# Buzz Fuel POC — Open Source / Pilot / Self-Service Audit

**Date:** 2026-07-23 (audit) · open-source remediation same day  
**Scope:** `buzz-fuel-poc` (adapter) + cross-repo `makereel-core` / `makereel-tg-miniapp` fuel paths  
**Method:** static review, secret/history scan, clean-clone build, unit/integration test runs (mock only; no real Stars/x402 spend)

**Open-source remediation landed:** `LICENSE`, tracked `Cargo.lock`, `bootstrap-deps.sh`, path/ID scrub, hygiene script, CI template. Clean-clone: bootstrap + `cargo test --locked` → 15 pass.

---

## Gate summary

| Gate | Verdict | Blocking findings |
|---|---|---|
| Public GitHub source | **CONDITIONAL** | Buildable after bootstrap; hygiene scrub done. Still: publish intentional tree; never force-add `evidence/`; do not claim pilot-safe credentials |
| Curated 2–5 community pilot | **FAIL** | Global `INTERNAL_API_KEY`; no installation scope; in-memory stores; no official-origin / `telegram_url` allowlist; kill-switch abandons paid fulfill |
| Public self-service deployment | **FAIL** | No install credentials, multi-tenant isolation, rate limits, container/ops kit, etc. |

**Do not treat “POC ran once” as pilot or self-service.**

### Sub-checklist (section 1 required outputs)

| Check | Verdict |
|---|---|
| Open-source repository | **CONDITIONAL** |
| Clean-clone build | **PASS** with `scripts/bootstrap-deps.sh` then `cargo test --locked` |
| Secret/history scan | **PASS** for live keys in git; evidence stays gitignored |
| License review | **PASS** (`LICENSE` Apache-2.0 + NOTICE) |
| Files to exclude or rewrite | `evidence/` forever; private ops notes outside repo |

---

## Evidence runs performed

1. **Adapter unit tests (working tree):** `cargo test` → 15 passed, 1 ignored (`blossom_live_upload`).
2. **Clean clone (git archive of tracked `buzz-fuel-poc` only):** `cargo test` → **FAIL** — missing `../buzz/crates/buzz-sdk`.
3. **Monorepo path-dep clone simulation** (working-tree `buzz-fuel-poc` + rsync `buzz/` excluding targets): `cargo test --locked` → **PASS** (15/1).
4. **Git history secret scan** (`git log -p -- buzz-fuel-poc`): no live `nsec1…` / PEM private keys; only placeholders and test keys.
5. **`evidence/`:** gitignored; never in git history. Local disk contains redacted payment packs, screenshots, signer address, Base tx hashes — **must not be force-added**.
6. **Core:** `uv run pytest tests/test_buzz_fuel.py -v` → **19 passed**.
7. **TG bot:** `uv run pytest tests/test_fuel_handlers.py -v` → **15 passed**.
8. **Parent git:** `git ls-files buzz` → **0** (Buzz checkout is untracked local dependency).

---

## Authority answers (section 4)

| Question | Current answer (as implemented) |
|---|---|
| 誰有權觸發 `/fuel`？ | Any Nostr pubkey whose kind:9 is delivered on the adapter’s `BUZZ_CHANNEL_ID` subscription. Adapter does **not** call `event.verify()`, does **not** check membership, does **not** rate-limit. Relies on relay filter + process start time. |
| 誰負責付款？ | Telegram user who opens `t.me/…?start=fuel_*` and pays Stars to **MakeReel** merchant bot (core builds URL from `BUZZ_FUEL_BOT_USERNAME` / miniapp bot username). Community is not merchant. |
| 誰能看到 prompt 與影片？ | Prompt is posted in the Buzz channel (public to channel members). Share URL is public `PUBLIC_BASE_URL/s/{token}` when delivered. Same result also delivered in TG to the payer. |
| 誰能消耗 MakeReel 預算？ | Any holder of `INTERNAL_API_KEY` who can call create-intent (then any TG user who claims+pays). Platform signer spends x402 after Stars; no per-installation spend cap in fuel module. |
| 誰可以停用或撤銷一個 installation？ | **No installation object exists.** Only global `BUZZ_FUEL_ENABLED` flag and ops redeploy. Cannot revoke one community without affecting all. |

---

# Findings

## [P0] Clean clone of published tree does not build

- **Verdict:** FAIL  
- **Surface:** repo  
- **Evidence:**  
  - `buzz-fuel-poc/Cargo.toml:15` — `buzz-sdk = { path = "../buzz/crates/buzz-sdk" }`  
  - `git ls-files buzz` → 0 tracked files  
  - Clean archive log: `Unable to update .../buzz/crates/buzz-sdk`  
  - HEAD `src/` only: `main.rs`, `makereel.rs`, `model.rs` (working tree also has untracked `blossom.rs`, `qr.rs`)  
- **Current behavior:** Operator machine builds only because local untracked `buzz/` + WIP sources exist beside the POC.  
- **Attack or failure scenario:** Third party clones GitHub → cannot build/test. Reviewers cannot reproduce.  
- **Impact:** Open-source and pilot install docs are false.  
- **Required fix:** Publish a buildable unit: git dependency or versioned crate of `buzz-sdk`/`buzz-core`, or monorepo that vendors minimal crates; track `Cargo.lock`; commit or drop WIP modules consistently.  
- **Required regression test:** CI job: `git archive` / clean checkout → `cargo test --locked` with zero local path hacks.  
- **Blocks:**  
  - [x] Public source  
  - [x] Curated pilot  
  - [x] Public self-service  

## [P0] Global `INTERNAL_API_KEY` is the community credential

- **Verdict:** FAIL  
- **Surface:** adapter / core  
- **Evidence:**  
  - `buzz-fuel-poc/src/main.rs:168-169` — loads `MAKEREEL_API_URL` + `INTERNAL_API_KEY`  
  - `buzz-fuel-poc/src/makereel.rs:39-41` — `Authorization: Bearer {api_key}` on every call  
  - `makereel-core/api/internal.py:47-52` — single global key  
  - `makereel-core/api/buzz_fuel.py:321-324` — entire fuel router uses `require_internal` only  
  - No `installation_id` / per-install token in create or get  
- **Current behavior:** One shared Bearer unlocks create intent, claim helpers (via bot), fulfill, get status, mark-refunded for **all** fuel intents.  
- **Attack or failure scenario:** Community A’s misconfigured or leaked key reads/creates intents for community B; or attacker sets `MAKEREEL_API_URL` to evil host and exfiltrates the same global key.  
- **Impact:** Cross-tenant data access; credential blast radius = entire product.  
- **Required fix:** Per-installation revocable credential scoped to create + poll **own** intents only; never ship platform `INTERNAL_API_KEY` to communities.  
- **Required regression test:** `installation A` cannot `GET` B’s `intent_id`; revoked token → 401; rotate A does not break B.  
- **Blocks:**  
  - [ ] Public source (doc-only warning insufficient)  
  - [x] Curated pilot  
  - [x] Public self-service  

## [P0] `MAKEREEL_API_URL` and `telegram_url` are not allowlisted

- **Verdict:** FAIL  
- **Surface:** adapter  
- **Evidence:**  
  - `main.rs:168` — any string accepted as API base  
  - `makereel.rs:30-51` — POSTs Bearer to `{base}/internal/buzz-fuel/intents`  
  - `main.rs:324-337` — posts `intent.telegram_url` and QR of that URL with **no** host check  
  - `model.rs:243-262` / `qr.rs:12-14` — pass-through  
  - Core **does** construct official URL (`buzz_fuel.py:338,363`) when talking to real core — but adapter never enforces  
- **Current behavior:** Misconfig or MITM/wrong env sends Bearer to attacker; malicious API response can return phishing Telegram link + QR.  
- **Attack or failure scenario:**  
  1. `MAKEREEL_API_URL=http://attacker.example` → key theft + fake `telegram_url`.  
  2. Compromised/malicious response body `telegram_url=https://t.me/NotMakeReel_bot?...` → channel members pay attacker.  
- **Impact:** Credential theft; Stars phishing; brand abuse.  
- **Required fix:** Allowlist HTTPS origins (e.g. `https://makereel.xyz`); reject non-TLS in production; require `telegram_url` host path `t.me/MakeReel_xyz_bot` (exact bot username); refuse to post/QR otherwise.  
- **Required regression test:** unit tests for attacker HTTP origin, wrong `t.me` host, missing HTTPS.  
- **Blocks:**  
  - [x] Curated pilot  
  - [x] Public self-service  

## [P0] Adapter replay protection is process-local only

- **Verdict:** FAIL  
- **Surface:** adapter  
- **Evidence:**  
  - `main.rs:177-182` — `seen_events` / `published` in `BotState`  
  - `main.rs:302-308` — dedupe only in-memory  
  - `main.rs:63-64` — `started_at = now` ignores historical events but **restart re-accepts live rebroadcasts**  
  - Core: `buzz_fuel.py:100,333-341` — idempotent on `buzz_request_event_id` **while process memory lives**  
- **Current behavior:** Same `/fuel` event handled once per adapter process. Restart + relay replay can call create again; core may return same intent if still live, or create new if expired/pruned.  
- **Attack or failure scenario:** `restart adapter → replay same event`; unpaid intent spam after TTL; duplicate payment links.  
- **Impact:** Intent spam, confusing multi-links, weak audit trail under pilot load.  
- **Required fix:** Durable adapter or core-side event dedupe store (DB) with TTL beyond intent TTL; adapter should also verify signatures.  
- **Required regression test:** restart adapter → replay same event ID → single live intent.  
- **Blocks:**  
  - [x] Curated pilot  
  - [x] Public self-service  

## [P0] Core fuel store is in-process memory (except charge ledger hit)

- **Verdict:** FAIL for multi-instance / restart durability  
- **Surface:** core  
- **Evidence:**  
  - `buzz_fuel.py:98-102` — `_intents`, `_by_token`, `_by_buzz_event`, `_orders` dicts  
  - `buzz_fuel.py:490-516` — fulfill charge idempotency via `ledger.get_ref(f"stars-gen:{cid}")` only  
  - No installation, no DB table for fuel intents  
- **Current behavior:** Core restart loses unpaid/paid-but-not-ledger-written intent state. Multi-replica = split brain.  
- **Attack or failure scenario:** Deploy restart during money window; webhook retry may still hit ledger for charge, but claim/status/order maps vanish.  
- **Impact:** Broken status polling; support dead-ends; unsafe for multi-community ops.  
- **Required fix:** Persist intents/orders with unique constraints on `buzz_request_event_id`, `start_token`, `stars_charge_id`.  
- **Required regression test:** restart core mid-flight; same charge; same event; status recoverable.  
- **Blocks:**  
  - [x] Curated pilot (if multi-community or multi-instance)  
  - [x] Public self-service  

## [P0] No installation / tenant scope on status queries

- **Verdict:** FAIL  
- **Surface:** core  
- **Evidence:**  
  - `buzz_fuel.py:595-604` — `GET /intents/{intent_id}` any authenticated internal caller  
  - Create request has only `buzz_channel_id` + `buzz_request_event_id` (`buzz_fuel.py:296-300`)  
  - No cross-tenant negative tests in `tests/test_buzz_fuel.py`  
- **Current behavior:** Knowing/guessing `fi_*` + holding global key returns status/prompt/share_url.  
- **Attack or failure scenario:** `installation A reads B's intent/job`.  
- **Impact:** Cross-community privacy breach.  
- **Required fix:** Bind intent to `installation_id`; authorize get by installation; cross-tenant tests.  
- **Required regression test:** A token + B intent_id → 404/403.  
- **Blocks:**  
  - [x] Curated pilot  
  - [x] Public self-service  

## [P0] Feature flag OFF blocks paid fulfill / status (abandon risk)

- **Verdict:** FAIL  
- **Surface:** core  
- **Evidence:**  
  - `buzz_fuel.py:118-120`, `488`, `597`, `613` — `_require_enabled()` on fulfill, get status, mark-refunded  
- **Current behavior:** Kill switch returns 404 for new **and** in-flight paid operations that still need fulfill/status/refund marking.  
- **Attack or failure scenario:** Ops sets `BUZZ_FUEL_ENABLED=false` during paid window → fulfill 404 → bot refunds (good if refund path works) **or** already-running jobs become unobservable via fuel status API.  
- **Impact:** Either forced refunds or orphaned observability; not “safe drain.”  
- **Required fix:** Flag must fail-closed on **new** unpaid creates only; paid/running must continue; document drain mode.  
- **Required regression test:** flag OFF during unpaid create → 404; flag OFF after paid → fulfill/status still work until terminal.  
- **Blocks:**  
  - [x] Curated pilot  
  - [x] Public self-service  

## [P0] Nostr events are not signature-verified by the adapter

- **Verdict:** FAIL  
- **Surface:** adapter  
- **Evidence:**  
  - `main.rs:276` — `Event::from_json` only  
  - No `event.verify()` (nostr 0.44 requires explicit `verify()`; see registry `event/mod.rs:160`)  
  - No membership check of `event.pubkey` against channel members  
- **Current behavior:** Trusts relay-delivered JSON shape. Honest Buzz relay verifies; **community-controlled or malicious relay** can inject unsigned/forged kind:9 `/fuel`.  
- **Attack or failure scenario:** Malicious relay injects `/fuel` as victim pubkey → spam intents / social engineering links in channel replies.  
- **Impact:** Intent spam and reputation abuse under community-hosted relays (the pilot deployment model).  
- **Required fix:** `event.verify()`; optional membership allowlist; reject bad `h` tags if present.  
- **Required regression test:** forged event without valid sig ignored.  
- **Blocks:**  
  - [x] Curated pilot (community relay)  
  - [x] Public self-service  

## [P0] Production infra / personal paths / live share URL in tracked docs

- **Verdict:** FAIL for open source as-is  
- **Surface:** repo  
- **Evidence (tracked):**  
  - `README.md:74-122` — `/Users/howard/...` paths  
  - `LIVE-RUN-CHECKLIST.md:20,42,68,102` — Railway environment ID `d4068cb8-…`, deploy ID `aebc3616-…`, signer `0x294B…f3b`  
  - `IMPLEMENTER-HANDOFF.md:53-54` — live share `https://makereel.xyz/s/<redacted>`, signer  
  - Many docs under `docs/0x-*.md` with absolute personal paths  
- **Current behavior:** Publishing branch leaks operator filesystem layout and production operational identifiers.  
- **Attack or failure scenario:** Recon for Railway project; targeted social engineering; unnecessary correlation of demo runs.  
- **Impact:** Privacy/ops security hygiene failure; not a wallet key leak but still publish-blocking.  
- **Required fix:** Rewrite docs to relative/placeholder paths; move live runbooks to private ops repo; scrub share URLs and deploy IDs from public tree.  
- **Required regression test:** CI `rg` banlist: `/Users/`, Railway UUIDs, raw share tokens.  
- **Blocks:**  
  - [x] Public source  
  - [ ] Curated pilot (can keep private runbooks)  
  - [x] Public self-service  

## [P1] Blossom media URL taken from upload response without origin check

- **Verdict:** FAIL  
- **Surface:** adapter  
- **Evidence:**  
  - `blossom.rs:149-151,186-198` — uses `desc.url` from JSON  
  - `main.rs:324-337` / `model.rs:276-290` — embeds `![image](url)` into kind:9  
- **Current behavior:** Malicious/compromised Blossom can return `https://evil/phishing.png` or `javascript:`-like schemes (URL scheme depends on client render).  
- **Attack or failure scenario:** Channel members load attacker-controlled image host (tracking / fake QR overlay if client renders aggressively).  
- **Impact:** Secondary phishing/tracking vector.  
- **Required fix:** Require returned URL host/path under same HTTP base as relay + `/media/{64hex}.png` shape.  
- **Required regression test:** evil URL in BlobDescriptor → upload treated as failure → text-only fallback.  
- **Blocks:**  
  - [ ] Public source  
  - [x] Curated pilot (if QR enabled)  
  - [x] Public self-service  

## [P1] HTTP error bodies may log internal details

- **Verdict:** FAIL  
- **Surface:** adapter  
- **Evidence:**  
  - `makereel.rs:48-49,66-67` — `create_intent HTTP {status}: {body}` into `anyhow`  
  - `main.rs:349` — `eprintln!("create_intent failed … {err}")`  
- **Current behavior:** Upstream error text (possibly stack traces, tokens, internal hosts) printed to operator logs.  
- **Attack or failure scenario:** Log aggregation leakage; support paste of full logs.  
- **Impact:** Secondary secret/PII exposure.  
- **Required fix:** Log status code + correlation ID only; never raw body.  
- **Required regression test:** mock 500 body with fake secret → assert logs redacted.  
- **Blocks:**  
  - [ ] Public source  
  - [x] Curated pilot  
  - [x] Public self-service  

## [P1] Prompt abuse surface open; no rate limits

- **Verdict:** FAIL for public; CONDITIONAL for tiny curated room  
- **Surface:** adapter / core  
- **Evidence:**  
  - `model.rs:144-164` — `/fuel <prompt>` up to 500 chars; no moderation  
  - `model.rs:185-190,248-262` — prompt echoed into channel Markdown-ish text  
  - No per-pubkey / channel / IP limits in adapter or `buzz_fuel.py`  
  - Core truncates only (`buzz_fuel.py:134-141`)  
- **Current behavior:** Any member (or forged event) can open intents + post long Unicode / Markdown-ish prompts to channel.  
- **Attack or failure scenario:** Spam unpaid intents; prompt injection into channel UI; budget drain via many paid generations.  
- **Impact:** Abuse, cost, UX harm.  
- **Required fix:** Rate limits; max outstanding unpaid intents; content policy; optional fixed-prompt mode for pilot.  
- **Required regression test:** N+1 `/fuel` from same pubkey rejected; oversize prompt truncated/rejected consistently.  
- **Blocks:**  
  - [ ] Public source  
  - [ ] Curated pilot if fixed prompt + human ops  
  - [x] Public self-service  

## [P1] Share URLs are guess-resistant but public forever (by design)

- **Verdict:** NEEDS-EVIDENCE for expiry policy  
- **Surface:** core  
- **Evidence:**  
  - `buzz_fuel.py:279` — `share_url = f"{PUBLIC_BASE_URL}/s/{share_token}"`  
  - `storage.set_share` uses `token_urlsafe(16)` (~96 bits)  
  - Status exposes share only when `delivered` (`buzz_fuel.py:191-192`)  
- **Current behavior:** High entropy token; once known, treated as public demo link. No fuel-specific ACL.  
- **Attack or failure scenario:** Link forwarded outside community; content not scoped to members.  
- **Impact:** Acceptable for demo if disclosed; not private delivery.  
- **Required fix:** Document public nature; optional expiry/auth for self-service.  
- **Required regression test:** unguessable token entropy test; optional TTL.  
- **Blocks:**  
  - [ ] Public source  
  - [ ] Curated pilot (disclose)  
  - [x] Public self-service if privacy promised  

## [P1] Missing open-source license file and lockfile policy

- **Verdict:** FAIL  
- **Surface:** repo  
- **Evidence:**  
  - `Cargo.toml:6` — `license = "Apache-2.0"`  
  - `NOTICE` present  
  - No `LICENSE` file in `buzz-fuel-poc/`  
  - `.gitignore:8` — `Cargo.lock` ignored (binary app should commit lock)  
- **Current behavior:** License intent unclear to GitHub; builds non-reproducible across machines without lock.  
- **Required fix:** Add Apache-2.0 `LICENSE`; stop ignoring `Cargo.lock`; document third-party fonts if shipped.  
- **Required regression test:** CI checks LICENSE + lock present.  
- **Blocks:**  
  - [x] Public source  
  - [ ] Curated pilot  
  - [x] Public self-service  

## [P1] No container / doctor / health / CI / SBOM for distribution

- **Verdict:** FAIL  
- **Surface:** deployment  
- **Evidence:** no `Dockerfile`, `.env.example`, `.github/workflows`, `doctor`, version CLI in tree  
  - Spec only in `docs/15-SHAREABLE-BOT-SECURITY-AND-DISTRIBUTION.md` (untracked draft)  
- **Current behavior:** Manual operator run with env vars; no non-root image, secret scan CI, or clean-clone gate.  
- **Required fix:** Match section 5 of the task: container, env example, doctor, health, CI (test + secret scan + audit + SBOM).  
- **Required regression test:** pipeline green on clean clone.  
- **Blocks:**  
  - [ ] Public source (nice-to-have)  
  - [x] Curated pilot (minimum doctor + env example)  
  - [x] Public self-service  

## [P2] Markdown / control characters in prompts not sanitized for channel clients

- **Verdict:** FAIL  
- **Surface:** adapter  
- **Evidence:** `model.rs:186-190` — raw prompt interpolation into message body  
- **Current behavior:** User prompt embedded in quotes; no strip of control chars / spoofing of footer lines.  
- **Attack or failure scenario:** Prompt contains `intent: fi_fake` or markdown image lines to confuse operators.  
- **Impact:** Social engineering / log confusion.  
- **Required fix:** Escape/sanitize display; separate structured tags from free text.  
- **Required regression test:** prompt with newlines + `intent:` spoof still safe.  
- **Blocks:**  
  - [ ] Public source  
  - [ ] Curated pilot  
  - [x] Public self-service  

## [P0] Required multi-tenant payment scenarios — results matrix

| Required scenario | Result | Notes |
|---|---|---|
| same Buzz event × 2 | **PASS (core, in-memory)** | `test_a02_create_twice_same_event_same_intent` |
| same successful_payment × 2 | **PASS (core/TG unit)** | `test_a11_duplicate_fulfill_same_job`; TG `test_b08_…` |
| restart adapter → replay same event | **FAIL / untested** | adapter memory only; no regression |
| installation A reads B’s intent/job | **FAIL** | no installation model; no negative test |
| wrong TG user claims intent | **PASS (core unit)** | `test_a06_second_claimant_conflict` |
| expired / used token | **PASS (core unit)** | `test_a04_expired_intent_claim_rejected` |
| tampered token | **PASS (practical)** | unknown token → 404 |
| x402 failure → refund retry × 2 | **PARTIAL** | TG refunds on fulfill fail / job fail (`test_b10_*`); process-local `refunded` flag — not durable across bot restart; no second-refund ledger test in fuel module |
| feature flag OFF unpaid | **PASS** | `test_a01_flag_off_returns_404` |
| feature flag OFF paid/in-flight | **FAIL** | fulfill/status also gated |

Core payment happy-path unit suite is strong for **single-process demo**. It is **not** multi-tenant production.

---

## Section 1 — Files to exclude or rewrite before any public tree

**Must exclude from public Git (or never add):**

- `buzz-fuel-poc/evidence/**` (already gitignored — keep it that way; do not force-add)
- `buzz-fuel-poc/target/**`
- `buzz-fuel-poc/scripts/__pycache__/**`
- `**/.env`, keys, `*.nsec`, `*.pem`
- Local `buzz/.env` (not in this POC tree but adjacent)
- Untracked talk-pack media if it embeds live demo correlation you do not want public

**Must rewrite before public source:**

- `README.md` (absolute paths, operator-only commands)
- `IMPLEMENTER-HANDOFF.md` (live share URL, signer, deploy narrative)
- `LIVE-RUN-CHECKLIST.md` (Railway IDs, production URLs, signer)
- `docs/01-SOURCE-MAP.md`, `02`, `03`, `05`, `06` (absolute `/Users/howard/...` paths)
- Any untracked `docs/1x-*` that copy production fingerprints if those are to be published

**Track or explicitly out-of-scope:**

- `src/blossom.rs`, `src/qr.rs` (present in working tree, absent from HEAD archive)
- `Cargo.lock` (currently ignored)
- `LICENSE`
- `.env.example`, `Dockerfile`, CI

---

## Trust-boundary threat analysis (credential / official domain)

```text
Community adapter env today
  BUZZ_BOT_PRIVATE_KEY     → community bot identity (OK if unique)
  BUZZ_CHANNEL_ID          → subscription filter (OK)
  MAKEREEL_API_URL         → UNTRUSTED string, Bearer sent here (BAD)
  INTERNAL_API_KEY         → GLOBAL platform key (BAD for community)

Core constructs telegram_url from server config (GOOD if adapter is honest)
Adapter displays any telegram_url from JSON (BAD if API is not honest)

Blossom returns media URL (unchecked) → channel markdown (BAD if Blossom evil)
```

Official domain enforcement is a **documentation wish** (`docs/15-…`) not code.

---

## Gate rationale (why three independent FAILs)

### Public GitHub source — FAIL

Even without security hardening, the repository is not a clean, licensed, reproducible public artifact: path dependency on untracked Buzz, no LICENSE, lockfile ignored, operator PII/paths and production IDs in tracked docs, HEAD ≠ working tree (QR modules).

### Curated 2–5 community pilot — FAIL

Payment state machine unit tests are real progress, but pilot communities **must not** receive `INTERNAL_API_KEY`, and the code has no other credential. Without installation scope, allowlisted API/Telegram host, durable dedupe, and safe kill-switch drain, a second community is a shared-blast-radius demo, not a safe pilot.

**Path to CONDITIONAL pilot (ops-only, not self-serve):** MakeReel runs all adapters; communities never get internal key; single channel allowlist; fixed prompt; human watch; flag procedures written. That is an **ops exception**, not a code PASS.

### Public self-service — FAIL

Missing almost every item in sections 3–5: install tokens, multi-tenant isolation, rate limits, container, CI, doctor, SBOM, revoke, moderation, durable refund/restart tests.

---

# MUST FIX BEFORE OPEN SOURCE

1. Make clean-clone `cargo test --locked` work (publish/vend `buzz-sdk` story; commit lockfile).  
2. Add `LICENSE` (Apache-2.0) consistent with NOTICE.  
3. Scrub `/Users/howard`, Railway deploy/env IDs, live share URLs, and production-only runbooks from the public tree.  
4. Align git HEAD with intended source (`blossom`/`qr` commit or remove).  
5. Keep `evidence/` and secrets out of git forever; add CI secret path banlist.  
6. Document clearly: this is a low-privilege adapter sample — not a payment core.

# MUST FIX BEFORE CURATED PILOT

1. **Do not give communities `INTERNAL_API_KEY`.** Issue per-install scoped credentials or host adapters centrally.  
2. Allowlist official `MAKEREEL_API_URL` (HTTPS) and `t.me/MakeReel_xyz_bot` for any posted/QR link.  
3. Durable idempotency for Buzz event IDs (core DB) + adapter `event.verify()`.  
4. Installation-scoped status reads + cross-tenant negative tests.  
5. Kill switch: block new unpaid only; drain paid work safely.  
6. Blossom URL origin validation if QR enabled.  
7. `.env.example` + doctor/dry-run + redacted logging.  
8. Written refund owner, support path, and incident contact for pilot ops.  
9. Re-run matrix including restart replay and flag-during-paid.

# MUST FIX BEFORE PUBLIC SELF-SERVICE

1. Everything in curated pilot list, plus:  
2. Rate limits (pubkey/channel/install/IP), unpaid intent caps, concurrency + daily spend caps.  
3. Prompt moderation / fixed SKU policy for open prompt.  
4. Persistent multi-replica-safe intent/order store.  
5. Container (non-root, read-only FS), SBOM, dependency + image scan CI.  
6. Installation register/rotate/revoke UI or API; signed release digests.  
7. Full release-gate checklist in `docs/15-…` section 8 with evidence packs.  
8. Terms/support/paysupport and durable refund retry ledger.

# PARK FOR LATER

- White-label merchant bots / community-owned Stars sellers.  
- Commission/attribution payouts.  
- Private (non-public) result delivery ACLs.  
- Webhook callbacks to community adapters (prefer polling until signed).  
- Arbitrary SKU/model marketplace.  
- ACP/LLM agent packaging inside the fuel adapter.  
- Talk-pack fonts/media branding polish (JetBrains Mono OFL if published).

---

## Verdict recap

```text
 Gate                              Verdict     Blocking class
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━
 Public GitHub source              FAIL        build + hygiene + license
 Curated 2–5 community pilot       FAIL        credential + tenant + drain
 Public self-service deployment    FAIL        all of above + abuse/ops
```

**Next action after this report:** choose one gate and implement only its MUST FIX list; re-audit with evidence. Do not open the repo or invite communities until that gate flips.
