# 07 — Grok alternate view (peer-review input)

**Author:** Grok (xAI)  
**Date:** 2026-07-22  
**Role of this file:** one independent implementation stance for Howard.  
**Not authority:** the working plan is `00`–`06`; accepted/rejected divergences are recorded in `08-PEER-REVIEW-CONVERGENCE.md`.  
**Do not treat this as an implementation order.** It is preserved as the adversarial input to the final convergence.

---

## 0. How to use this document

Howard’s workflow:

1. Codex reads this file as an adversarial peer, not as a junior implementer.
2. Codex answers every item under **§6 Challenge list** with `ACCEPT` / `REJECT` / `PARTIAL` + one-sentence reason.
3. Howard locks only the divergences that survive that fight.
4. Implementation still follows the surviving `00`–`04` tree, not this file.

Grok’s frame for the whole POC:

> **24-hour glue experiment.** Reuse MakeReel payment authority and gateway buyer path. Derive a tiny Buzz poster. Prove Stars → real x402 → result, then stop. Do not build the product that the README will one day describe.

---

## 1. What I agree with Codex (LOCK from my side too)

These are correct. I am **not** challenging them.

| # | Codex lock | Why I agree |
|---|---|---|
| L1 | Reuse `@MakeReel_xyz_bot`; no new merchant bot | Merchant credentials, Stars policy, refund path already exist |
| L2 | Deep link `?start=fuel_<opaque-token>` then bind TG user before invoice | Correct Stars UX; invoice without payer binding is a footgun |
| L3 | No hard-coded Stars price; quote via existing `tg_quote` + conversion | My earlier “10 Stars fixed” was worse. Codex wins. |
| L4 | SKU `seedance-fast/5s-720p`, fixed server-side prompt | Known smoke path; no free-form prompt surface |
| L5 | C2 requires real `PLATFORM_SIGNER_KEY` / payer path; internal token ≠ C2 | Protects public “x402 out” claim |
| L6 | Do not modify Buzz upstream or x402 gateway in P0 | Scope control |
| L7 | Derive Buzz adapter from `countdown-bot`; exact `/fuel` only | No ACP, no LLM agent, no Desktop fork |
| L8 | `BUZZ_FUEL_ENABLED` default off + fail-closed | Safe deploy/rollback |
| L9 | TG copy crypto-free; stable MakeReel share URL to Buzz | Policy + URL expiry correctness |
| L10 | One Stars charge → at most one x402 job; refund on unfulfillable charge | Existing MakeReel money rules |

If Codex’s plan only needed validation of the above, this file would be short: **ship it**.

The rest is about **velocity, over-structure, and solo-founder realism**.

---

## 2. Where I diverge (and why)

### D1 — Optimise for 24h ship, not for multi-role process perfection

**Codex:** implementer ≠ independent reviewer; C2 cannot be self-approved; multi-role handoff table.  
**Grok:** for a solo 24h probe, process that assumes a second human becomes dead time.

**Proposal:**

- Keep the **evidence bundle and checklist** (excellent).
- Allow **self-review with a forced cooling rule**: implementer fills evidence; waits ≥30 minutes; re-runs the checklist as “reviewer mode”; fails any missing artifact.
- Only require a second human if Howard explicitly assigns one.

**Challenge to Codex:** Prove the independent-reviewer gate improves safety more than it burns the Buzz attention window. If yes, keep it. If it is cargo-cult process, demote to “nice when available.”

---

### D2 — Cut the automated test surface to money-critical cases first

**Codex:** A01–A15, B01–B11, C01–C08 before real money.  
**Grok:** the matrix is high quality; it is also roughly **half a day** if written seriously from zero.

**Proposal — two tiers:**

**Tier-1 (blocks any real Stars spend):**

- A02 create idempotency (same Buzz event → one intent)
- A05/A06 claim binding + foreign claimant reject
- A08/A09 wrong payer / wrong amount precheck
- A10/A11 fulfill once + duplicate charge replay
- A13 gateway reject → non-2xx → refund path required
- B01 legacy `/start` unchanged
- B04/B05 payload namespace isolation (`bf:` vs legacy)
- B08 duplicate successful_payment
- B10 refund-on-failure once
- C01/C02/C05/C06 `/fuel` only + dedupe + one message per transition

**Tier-2 (can land after C1, before public post):**

- A01 flag off
- A03 entropy cosmetics
- A12 mid-flight price change
- A15 serialization redaction audit
- B11 copy lint
- C03 historical event ignore
- C07 relay reject logging polish
- full `uv run pytest` suite of makereel-core as regression, not as first gate

**Challenge to Codex:** Which Tier-2 tests are actually load-bearing for a single fixed-SKU, feature-flagged POC? List any you refuse to demote and why money breaks without them.

---

### D3 — Timebox is 24h calendar, not 8–13 focused ideal hours

**Codex estimate:** 8–13h after Day0 green (honest vs the original 6h fantasy).  
**Grok:** agree on engineering hours; disagree on how to schedule them.

**Proposal schedule (calendar, after Day0):**

| Block | Focus | Hard stop if unfinished |
|---|---|---|
| H0–H2 | Core create/claim/precheck only | No fulfill yet; no Buzz |
| H2–H4 | Core fulfill + status via existing `tg_generate` | No TG delivery polish |
| H4–H6 | TG `fuel_` + `bf:` dispatch + invoice | No native video if poll hard; DM “generating…” ok |
| H6–H8 | Buzz `/fuel` + poll `Fueled` only | **Ship C1 demo even if C2 not ready** |
| H8–H12 | C2 real money + TG video + Buzz URL | Cut polish, not money safety |
| H12–H24 | Evidence, edit, post, sleep buffer | No new features |

**Critical product of this schedule:** **C1 is a shippable intermediate.**  
Codex already defines C1; Grok wants it treated as a **publicable half-story** if C2 slips:

> “Agent asked for fuel → human paid Stars → room got a signed Fueled receipt.”

x402 out can be the second cut. That matches the research plan’s failure cut more aggressively.

**Challenge to Codex:** Is publishing C1-only acceptable, or does public copy require C2? If C2-only, the schedule must front-load payer smoke harder than Buzz UX.

---

### D4 — Thin the intent API surface if any endpoint is only ceremony

**Codex contracts:** create → claim → precheck → fulfill → status (5).  
**Grok:** claim + precheck + fulfill are money-correct. Create + status are orchestration. I keep all five **unless** implementer finds create+claim can merge without losing binding safety.

I do **not** propose a parallel “simple dict in the Buzz process” architecture. Codex is right that payment authority stays in MakeReel.

**Soft challenge only:**

- Keep endpoints.
- Avoid inventing a new long-lived schema; mirror miniapp short-lived orders (Codex already says this — reaffirm).
- Prefer in-memory / existing process-local store with TTL over any migration.

**Challenge to Codex:** Confirm the intended store is process-local/ephemeral like miniapp orders. If someone “helpfully” adds Postgres tables in P0, that is REJECT.

---

### D5 — Polling ownership: one writer for job terminal state

**Codex:** TG bot background-polls job and delivers video; Buzz adapter polls intent status.  
**Grok:** agree on dual observers, but **one authority must mark `delivered` / `failed`.**

**Proposal:**

- **MakeReel core** is the only status authority.
- TG bot may push terminal updates into core when it learns job success/failure (if not already done by `tg_generate` path).
- Buzz adapter is **read-only** against core status; it never invents `delivered`.

If core already transitions on generate/poll internally, good. If not, do not let Buzz and TG race two writers.

**Challenge to Codex:** Name the single function/path that flips intent → `delivered`. If it is “whoever polls last,” redesign before code.

---

### D6 — Day0 OPEN items need defaults, not blank cells

Codex OPEN table is right as a gate list. Blank cells kill 24h starts.

**Grok defaults (Howard can override in 60 seconds):**

| OPEN | Proposed default |
|---|---|
| O1 relay/channel | Local or already-used demo relay; one dedicated channel UUID written into adapter env only |
| O2 standalone write | Must pass before any code; if fail → try owner-attested **once**, then stop (do not redesign) |
| O3 bot health | Production `@MakeReel_xyz_bot` `/start` screenshot today |
| O4 signer funds | One canonical quote + 20% buffer USDC on the network payer.py already uses |
| O5 prompt | Exactly one English line, e.g. `Buzz launch reel, bold kinetic typography "BUZZ", dark neon workspace, 5s, cinematic` — Howard edits once |
| O6 publish | X primary; one secondary only if already open. Do not invent a multi-channel launch plan |

**Challenge to Codex:** Accept these as temporary LOCK-until-Day0, or supply better defaults now. Empty OPEN is not a plan.

---

### D7 — Do not port more of `makereel-tg-chat` than the minimum

**Codex source map:** selectively port poll/delivery from `makereel-tg-chat`.  
**Grok:** that cross-repo port is a stealth third product surface.

**Proposal:**

1. First try: call existing core job status/share endpoints with a **20–40 line** poll loop inside miniapp bot.
2. Only if that loop is clearly a bad copy of chat bot behavior, port the chat pattern.
3. Cap ported code review at “strong task ref + timeout + single refund.”

**Challenge to Codex:** Is `fuel_delivery.py` necessary on day one, or is it premature modularisation?

---

### D8 — Buzz adapter language/deps: countdown-bot fidelity beats cleanliness

**Codex:** standalone Cargo package, path `buzz-sdk`, NOTICE attribution.  
**Grok:** agree. Push further: **prefer mechanical derive over elegant rewrite.**

- Copy countdown-bot structure first.
- Replace command parser with `/fuel`.
- Add HTTP client for MakeReel.
- Defer clippy-perfect module splits if they cost hours.

**Challenge to Codex:** `src/{main,buzz,makereel,model}.rs` split is fine, but not a gate. One `main.rs` under ~400 lines is acceptable for P0.

---

### D9 — Narrative honesty gates are more important than architecture diagrams

Public failure modes I care about more than module layout:

1. Saying “x402 out” after A1/internal bypass → **lie**. Codex already blocks this. Keep.
2. Saying “autonomous agent” when the bot only mirrors status → **overclaim**. Copy must say reference experiment / payment bridge.
3. Saying “market validated” after Howard pays → **lie**. Codex already blocks this. Keep.
4. Publishing a polished multi-contributor fuel story from a single-payer POC → **scope lie**.

**Add to acceptance (Grok):** a 5-line **claims checklist** in `evidence/acceptance.md`:

```text
[ ] Did not claim partnership / official Buzz support
[ ] Did not claim decentralized custody
[ ] “x402 out” only if C2 PASS with real settlement evidence
[ ] Howard payment labeled plumbing, not demand
[ ] TG user-facing copy has zero crypto checkout language
```

**Challenge to Codex:** Merge this into `03` or reject as redundant.

---

### D10 — Research plan P2 material must stay out of this folder’s critical path

Codex already parks SDK/runtime. Good.

**Grok extra:** do not let implementers “just add” mission policy fields, contributor lists, or capability tokens because the long research doc mentioned them.  
If a PR adds fields not required by create/claim/precheck/fulfill/status above → automatic REJECT for P0.

---

## 3. Grok’s compressed architecture (same systems, less ceremony)

```text
Buzz channel
  user: /fuel
  bot:  signed kind9 + t.me/MakeReel_xyz_bot?start=fuel_<token>
           │
           ▼
MakeReel core  (only money brain)
  create intent → claim(tg_user) → tg_quote → send_invoice path
  precheck → fulfill → existing tg_generate(stars) → payer.py → gateway
  status: awaiting_claim → awaiting_payment → paid|running → delivered|failed|refunded|expired
           │
     ┌─────┴─────┐
     ▼           ▼
  TG bot      Buzz bot
  (pay+video) (read status, post Fueled/Running/Delivered)
```

Same as Codex’s topology. Differences are **process and cut lines**, not a new system.

---

## 4. Grok’s must / nice / not-now (execution cut)

### Must (C2 complete)

1. Real Stars `successful_payment` on production merchant bot path.
2. Real platform-signer x402 settlement (one job).
3. Signed Buzz `Fueled` + later `Delivered` with stable share URL.
4. Refund or explicit failed path without double charge.
5. Legacy miniapp payment path still works.
6. Evidence bundle on disk (even if self-reviewed).

### Nice (only after Must)

- Native TG video vs link-only
- `Running` intermediate Buzz message
- Pretty receipt formatting
- Public multi-platform launch

### Not now

- Mini App UI, multi SKU, multi payer, SDK, ACP/MCP, gateway changes, hard-coded Stars, internal-token C2, Desktop fork

---

## 5. Explicit disagreements summary (for Codex to score)

| ID | Topic | Codex stance | Grok stance | Severity |
|---|---|---|---|---|
| D1 | Independent reviewer required | Hard gate | Checklist self-review OK for solo 24h | Medium |
| D2 | Full A/B/C test matrix before money | Broad | Tier-1 money tests first | High |
| D3 | C1 publicability | Implied internal | C1 may ship as partial public story | Medium |
| D4 | Five endpoints | Keep | Keep; no new DB | Low (aligned) |
| D5 | Status writer | Implicit | Name single authority | High |
| D6 | OPEN defaults | Empty until Day0 | Fill temporary defaults now | Medium |
| D7 | Port tg-chat delivery module | Planned file | Prefer 20–40 line loop first | Low–Medium |
| D8 | Buzz code structure | Multi-file crate | Mechanical derive OK | Low |
| D9 | Claims checklist | Scattered | One explicit acceptance block | Low |
| D10 | P2 field creep | Parked | Automatic REJECT if appears in PR | Low (aligned) |

---

## 6. Challenge list — Codex must answer these

Answer each with `ACCEPT` / `REJECT` / `PARTIAL` and ≤2 sentences. No essay.

1. **Self-review vs second human:** Is C2 blocked without a second person when Howard is solo?
2. **Tiered tests:** Which of A01–A15 are non-negotiable before first real Stars charge?
3. **C1 publish:** May Howard post a C1-only demo without claiming x402 settlement?
4. **Delivered authority:** Which exact core path sets terminal success, and can TG/Buzz only observe it?
5. **Store:** Confirm no Postgres migration in P0; name the store pattern (miniapp-like).
6. **`fuel_delivery.py`:** Required day-one module, or optional extraction after a working loop?
7. **O5 prompt:** Accept Grok’s placeholder string until Howard edits, or hard-block all coding without final prompt?
8. **Estimate honesty:** Keep 8–13h post-Day0 as the planning number (Grok agrees) and delete any residual “6h” fantasy from sibling research docs?
9. **A1 internal token:** Even for private dry runs, must logs label it non-C2 so it cannot leak into demo voiceover?
10. **If Day0 O2 standalone auth fails:** One owner-attested attempt, then stop — agree?

---

## 7. What would make me change my mind

I will drop D1–D3 if Codex shows:

- a named second reviewer already available same day, and
- Tier-2 tests are mostly fixtures that take &lt;60 minutes total, and
- Buzz attention value of a C1-only post is negative (e.g. confuses the only narrative that matters).

I will escalate harder against Codex if implementation PRs:

- add schema migrations,
- touch gateway,
- introduce free-form prompts,
- or skip refund/idempotency tests “to save time.”

Those are not velocity; those are how you corrupt the existing MakeReel money path.

---

## 8. Bottom line for Howard

**Codex plan quality:** high. Correct money boundaries, correct reuse map, correct non-goals.  
**Grok risk read:** process + test breadth may push a 24h probe into a multi-day “proper project.”

**Recommended merge direction (my bias):**

1. Keep Codex architecture and LOCK table almost entirely.
2. Absorb Grok D2 tiered tests, D5 single status authority, D6 OPEN defaults, D9 claims checklist.
3. Soften D1 to checklist self-review unless a second reviewer is actually assigned.
4. Treat C1 as a hard intermediate ship gate (D3).

Then let Codex strike back on §6. Winner = whatever survives with fewer moving parts **without** weakening Stars/x402 safety.

---

## 9. Non-action reminder

This file does **not** authorize:

- editing `00`–`04`,
- writing production code,
- spending Stars/USDC,
- or changing MakeReel/Buzz/gateway checkouts.

It is a review artifact only.
