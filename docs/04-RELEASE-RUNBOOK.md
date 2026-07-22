# 04 — Release, demo, and rollback runbook

This runbook is executed only after implementation and automated tests. It does not authorize deployment or spending.

## Phase 0 — assign and freeze

1. Assign one implementer and a different reviewer.
2. Record `git status --short` and commit SHA for all three checkouts.
3. Preserve the existing untracked `makereel-tg-miniapp/docs/` directory.
4. Resolve each OPEN item before the phase it blocks; O1/O2 do not block core work and O4 does not block C1-dry.
5. Agree on a maximum real Stars/USDC test budget.

Recommended branches:

```text
buzz402:             poc/buzz-fuel-adapter
makereel-core:       poc/buzz-fuel-core
makereel-tg-miniapp: poc/buzz-fuel-bot
```

Do not create branches or commits in Buzz upstream or x402 gateway.

## Phase 1 — merge/deploy order

1. **Core first:** deploy disabled code with `BUZZ_FUEL_ENABLED=false`.
2. **Telegram bot second:** deploy payload routing; with flag off, fuel links fail closed while legacy flows remain live.
3. **Buzz adapter last:** configure it only after core and bot health checks pass.
4. Turn on `BUZZ_FUEL_ENABLED` for the controlled test window.

This order prevents a public Buzz link from pointing to an unavailable payment path.

## Configuration inventory

### Existing MakeReel secrets — reuse, never copy into this repo

- `BOT_TOKEN`
- `INTERNAL_API_KEY`
- `PLATFORM_SIGNER_KEY`
- existing MakeReel/gateway URLs and session secrets

### Planned core configuration

- `BUZZ_FUEL_ENABLED=false`
- `BUZZ_FUEL_PROMPT=<Howard-approved fixed prompt>`
- `BUZZ_FUEL_BOT_USERNAME=MakeReel_xyz_bot`
- `BUZZ_FUEL_INTENT_TTL_SECONDS=1800`

### Planned Buzz adapter configuration

- `BUZZ_RELAY_URL`
- `BUZZ_CHANNEL_ID`
- `BUZZ_BOT_PRIVATE_KEY`
- `BUZZ_BOT_AUTH_MODE=standalone`
- `MAKEREEL_API_URL`
- `INTERNAL_API_KEY`
- bounded polling interval/timeout

Never put example secret values in docs, shell history, screenshots, or evidence.

## Phase 2 — safe smoke

1. Start the Buzz adapter with core feature flag off; `/fuel` must fail closed with a maintenance message or no link.
2. Enable the flag in a controlled environment.
3. Run C1-dry using Telegram's approved test environment or mocked payment update; label every artifact TEST/SIMULATED.
4. Run regression checks for normal `/start` and existing invoice kinds.
5. Assemble the pre-money test results. Howard—not the implementer—approves moving to C1-live/C2.

## Phase 3 — controlled real run

1. Confirm platform payer balance and maximum spend.
2. Start screen recording before posting `/fuel`.
3. Complete C1-live by paying the Stars invoice as Howard.
4. Keep the recording continuous until Buzz shows signed `Fueled`.
5. Capture redacted x402 settlement and job evidence.
6. Wait for the model. The public edit may cut waiting time.
7. Capture Telegram video delivery and Buzz stable share link.
8. Stop accepting new intents by turning the flag off unless Howard explicitly opens the POC publicly.

Howard's payment proves plumbing only, not demand.

If C2 cannot finish, refund the accepted Stars charge exactly once. A paid C1-live receipt without delivery or refund is not a publishable success.

## Demo edit

The public 30–45 second edit contains:

1. Buzz `/fuel` request.
2. Signed funding link from the Buzz bot.
3. Telegram Stars invoice and payment confirmation.
4. Buzz `Fueled` receipt.
5. A visible time cut for model waiting.
6. Telegram result and Buzz stable result link.

Do not show:

- bot token, internal bearer, private keys
- wallet signature/payment header
- full Stars charge ID or TG user ID
- admin dashboards or unrelated balances

Copy rules:

- Telegram copy: Stars and generation only.
- Public/Buzz copy may say x402 only after C2 evidence passes.
- Do not claim partnership, official Buzz integration, decentralization, or market validation.
- A C1-dry-only edit must say test/simulated and must not imply real Stars or x402.
- A C1-live-only edit may say a Stars payment produced a signed receipt, but must not imply x402 or delivery; it may publish only after the charge is refunded.
- Any internal-token footage/log is marked `NON-C2` and excluded from an “x402 out” voiceover.

## Rollback

Rollback order:

1. Set `BUZZ_FUEL_ENABLED=false`.
2. Stop the standalone Buzz adapter.
3. Verify new `fuel_` links fail closed and create no invoice.
4. Verify legacy bot `/start`, Mini App invoice, and unlock/generation payments still work.
5. If required, roll back the TG bot commit, then the core commit.

No gateway or Buzz upstream rollback should exist because P0 does not modify them.

## Incident rules

| Incident | Immediate action |
|---|---|
| Duplicate x402 job/settlement | Disable flag, stop adapter, preserve evidence, do not retry |
| Stars charged but no job | Disable flag, run existing Stars refund, preserve charge/job correlation |
| TG copy exposes crypto checkout | Disable flag and revert copy before further tests |
| Secret appears in logs/video | stop release, rotate affected secret, delete unsafe artifact |
| Buzz bot cannot authenticate/write | stop; fix membership/auth, do not switch to ACP or modify relay |
| Model job exceeds demo timeout | keep truthful pending state; do not fake Delivered |

## Post-run decision

Within 24–72 hours record only:

- external Stars payment
- external builder asking to connect an Agent
- Buzz maintainer/community adoption or sharing

Likes and impressions do not unlock more build. Without external pull, archive the POC as a reference experiment and leave the long-term Agent Fuel plan parked.
