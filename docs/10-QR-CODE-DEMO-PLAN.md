# 10 — Buzz Fuel: QR code in channel (demo plan)

**Status:** PLAN ONLY · not required for C1/C2 acceptance  
**Audience:** implementer agent building the feature  
**Language:** EN (implementation) · product notes may be bilingual  

---

## Goal

After a user posts exact `/fuel` in Buzz, **MakeReel Fuel Bot** (the fuel adapter) should reply with:

1. Existing demo-friendly **text** (link + steps + ref footer)
2. An **inline QR image** of the public Telegram deep link so a phone can open the invoice without typing

```text
Buzz /fuel
  → adapter creates intent
  → reply: copy + QR of https://t.me/MakeReel_xyz_bot?start=fuel_<token>
  → human scans / opens TG → Stars invoice
```

**Out of scope for this plan:** Desktop fork, TG bot QR photo (optional later), x402/USDC in QR or copy.

---

## Policy / gates

From `AGENTS.md` and `00-DECISIONS.md`:

| Item | Rule |
|---|---|
| C1-dry / C1-live / C2 acceptance | **Does not require** QR; text `t.me` link is enough |
| Early P0 list | “QR/fuel-meter polish” and “Buzz media uploads” were **parked** until gates pass |
| This plan | **Post-gate demo polish** — implement only when Howard explicitly wants demo UX |

Do **not** claim shipping is blocked without QR.

---

## Research verdict (read-only, 2026-07-23)

| Question | Answer |
|---|---|
| Can kind 9 show an image in Desktop? | **Yes** — NIP-92 `imeta` **and** markdown body line `![image](<url>)` |
| imeta alone without markdown? | **No** — Desktop only renders `<img>` from `![image](url)` in content |
| Data URL in imeta? | **No** — relay requires local `/media/{64hex}.{ext}` (or tenant media base) |
| External host URL in imeta? | **No** — foreign imeta URLs rejected; must upload to **this** community’s Blossom first |
| Desktop QR-specific message type? | **None** — use normal image attachment path |
| Desktop fork needed? | **No** |
| QR payload | **Only** public deep link: `https://t.me/MakeReel_xyz_bot?start=fuel_<opaque>` — no keys, charge IDs, USDC, x402 |

Reference Desktop path:

- Composer / CLI: upload blob → `BlobDescriptor` → append `![image](url)` → attach `imeta` tags
- Render: `MessageRow` → `parseImetaTags` → Markdown `ImageBlock`
- Invite UI QR exists but is **chrome only**, not a message protocol

Fuel adapter today: text-only `format_payment_link` + `build_message(..., &[])` (empty media tags).

---

## Recommended approach (Option A)

**Local QR PNG → Blossom `PUT /upload` on the same Buzz relay → kind 9 with imeta + `![image](url)`.**

Fallback if upload fails: **post text-only** (never block `/fuel` path).

### Rejected / deferred options

| Option | Why not first |
|---|---|
| B — External temp host + markdown only | Extra infra, link rot, not first-class imeta |
| C — ASCII QR in text | Low demo quality; OK as emergency fallback only |
| D — Telegram `send_photo` QR only | Helps TG, **zero** in Buzz channel (optional follow-up) |
| E — Status quo text link | Keep as **always-on** baseline + failure fallback |

---

## Architecture

```text
maybe_handle_fuel
  → makereel.create_intent(...)
  → format_payment_link(url, intent_id, buzz_event_id)   # existing text
  → optional: qr_png = encode(url)
  → optional: blob = blossom_upload(relay, bot_keys, png)
  → body = text + "\n\n![image](" + blob.url + ")"
  → media_tags = [imeta: url, m, x, size, dim?]
  → publish_reply(..., body, media_tags)
```

**Colocation:** Adapter already connects to `BUZZ_RELAY_URL`. Upload must target **that same** relay’s media base (local demo: `http://localhost:3000` / `ws://localhost:3000` → HTTP origin for Blossom).

---

## Implementation tasks (for builder agent)

### 1. Dependencies (`buzz-fuel-poc`)

- Add crates for QR + PNG encode (e.g. `qrcode`, `image` — pin versions consistent with workspace style).
- Prefer pure Rust; no new network deps beyond existing `reqwest` if possible.
- Reuse Buzz patterns for Blossom upload from `buzz` / `buzz-cli` (search: `sign_blossom_upload`, kind **24242**, BUD-02 `PUT {relay}/upload`).

### 2. Blossom upload helper

New module e.g. `src/blossom.rs` or under `makereel.rs` sibling:

- Input: PNG bytes, bot `Keys`, relay HTTP base URL
- Compute SHA-256 of body
- Auth as required by Buzz relay (NIP-98 / Blossom 24242 — **match existing CLI exactly**)
- `PUT` to `{http_base}/upload` (confirm path from Buzz relay source)
- Output: `url` (must be local `/media/...` form), `sha256` hex, `mime=image/png`, `size`, optional `dim`

Map `ws://` / `wss://` → `http://` / `https://` for upload (adapter already has this pattern for other HTTP).

### 3. QR encode helper

- Input: `telegram_url: &str` (exact string from `CreateIntentResponse.telegram_url`)
- Output: PNG bytes (reasonable size for phone scan on a laptop screen, e.g. 256–512 px module scale)
- **Do not** put intent_id, buzz event id, or secrets in the QR — only the deep link

### 4. Wire into publish path (`src/main.rs` + `src/model.rs`)

- Extend `publish_reply` (or equivalent) to accept `media_tags: Vec<Tag>` (or SDK equivalent) instead of hardcoding empty.
- After successful `create_intent`:
  1. Build text via `format_payment_link` (keep demo copy + ref footer)
  2. Try QR + upload
  3. On success: append `\n\n![image]({url})` to content; set imeta tags  
     Example imeta shape (confirm against relay validator):
     `["imeta", "url <url>", "m image/png", "x <sha256>", "size <n>", "dim <W>x<H>"]`
  4. On failure: log `eprintln!`, post **text only**
- Status messages (`Fueled` / `Delivered` / …): **no QR required** (optional later only on payment link)

### 5. Profile / display name

Unrelated to QR; leave as-is unless already changed for demo copy.

### 6. Tests

| Test | Expect |
|---|---|
| Unit: QR payload | Encoded string == public `t.me` URL only |
| Unit: message body | Contains `![image](` when media present; still contains steps + `intent:` footer |
| Unit: failure path | Upload Err → body has no imeta requirement; still has `t.me` link |
| Unit: secrets | No `USDC`, `x402`, `tg_user`, `charge` in payment message |
| Manual smoke | Local relay + Desktop: `/fuel` → inline QR → phone scans → Telegram start |

### 7. Manual demo checklist

1. `buzz-relay` on `:3000`, Desktop on `ws://localhost:3000`
2. Channel = adapter `BUZZ_CHANNEL_ID` (e.g. Buzz Fuel C1 Dry)
3. Adapter running with new binary
4. Core `BUZZ_FUEL_ENABLED=true` for live link; or flag off to test offline copy only
5. Post exact `/fuel`
6. Confirm Desktop shows image **and** text steps
7. Scan QR with phone → Telegram opens `@MakeReel_xyz_bot` with `fuel_` start param

### 8. Docs / evidence

- Do **not** treat QR as acceptance gate in handoff
- If shipping the feature: one short note in README or handoff under “Demo polish”
- Screenshots for demo: redact nothing required (link is public); still avoid secrets in frame

---

## Non-goals

- Forking or patching Buzz Desktop / relay protocol for QR
- Changing Telegram invoice UX / Stars amounts
- Encoding amount, job id, or internal API URLs in the QR
- Multi-SKU or free-form prompts
- Blocking fuel if media auth fails
- Committing binary media fixtures of production links

---

## Security

| Rule | Detail |
|---|---|
| QR content | Public `https://t.me/MakeReel_xyz_bot?start=fuel_<token>` only |
| Token in URL | Opaque claim token is **by design** in the deep link; same as text link today |
| Never in QR / body | `INTERNAL_API_KEY`, bot nsec, PLATFORM_SIGNER, charge IDs, TG user IDs |
| Claims language | Channel copy remains **Stars-only** (no USDC/x402 marketing in payment message) |

---

## Effort estimate

| Work | Estimate |
|---|---|
| Blossom upload client + auth match | 0.5–1 d |
| QR encode + wire + fallback | 0.25–0.5 d |
| Tests + local Desktop smoke | 0.25–0.5 d |
| **Total** | **~1–2 days** |

ASCII-only fallback: ~2–4 hours if A blocked by media auth.

---

## File map (likely touch)

```text
buzz-fuel-poc/
  Cargo.toml                 # qrcode + image (+ any auth helpers)
  src/main.rs                # publish_reply media tags; call QR after create_intent
  src/model.rs               # optional: format_payment_link_with_image(url, ...)
  src/blossom.rs             # NEW: upload helper (name flexible)
  src/qr.rs                  # NEW: encode helper (name flexible)
  docs/10-QR-CODE-DEMO-PLAN.md  # this file
```

Reference (read, do not fork product into adapter):

```text
buzz/examples/countdown-bot/     # bot auth / publish patterns
buzz desktop imeta + markdown    # display contract
buzz-cli media upload            # Blossom PUT + sign pattern
buzz relay imeta validation      # url must be local /media/
```

---

## Acceptance criteria (for this feature only)

1. Happy path: `/fuel` → signed kind 9 with scannable QR **and** text deep link.
2. Upload failure: text deep link still posts; no crash; no duplicate intents.
3. Desktop on same relay shows image without Desktop code changes.
4. Unit tests pass; no secrets in formatted messages.
5. QR encodes exactly `telegram_url` from create_intent response.

---

## Builder agent prompt (copy-paste)

```text
Implement Buzz Fuel demo QR per buzz-fuel-poc/docs/10-QR-CODE-DEMO-PLAN.md Option A.

Constraints:
- Only touch buzz-fuel-poc (not buzz Desktop/relay product code unless reading reference)
- QR payload = public telegram_url only
- Fallback to text-only if Blossom upload fails
- Keep Stars-only channel copy (no USDC/x402 in payment message)
- Exact /fuel still the only trigger
- Add unit tests; run cargo test
- Do not enable Railway flags or pay Stars unless Howard asks

Verify: local relay + adapter; optional Desktop smoke with ![image] visible.
```

---

## Open decisions (Howard)

1. Ship QR now as demo polish, or wait until after a clean 75★ pricing run?
2. Optional follow-up: Telegram `send_photo` of same QR (Option D)?
3. QR size preference (screen share vs phone on desk)?

---

## Changelog

| Date | Note |
|---|---|
| 2026-07-23 | Initial plan from explore research (kind 9 + imeta + Blossom; P0 park noted) |
