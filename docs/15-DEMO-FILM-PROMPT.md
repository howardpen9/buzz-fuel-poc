# Demo film — `/fuel` with visible prompt

**Status:** live for filming (2026-07-23)  
**Channel:** Buzz Fuel C1 Dry (`72ef2918-e8db-44de-9e28-10f504c44ac9`)  
**Adapter log:** `/tmp/buzz-fuel-bot-prompt.log` (pid file `/tmp/buzz-fuel-bot-prompt.pid`)

## Why

Bare `/fuel` always used the same server prompt → videos looked almost identical.  
Now each take can carry its own generation prompt, and Buzz shows that prompt on payment + status messages.

## Commands (type exactly)

```text
/fuel
```

→ default: `Buzz launch reel, bold kinetic typography "BUZZ", dark neon workspace, 5s, cinematic`

```text
/fuel Golden hour rooftop, bold kinetic typography "BUZZ", rain reflections, 5s cinematic
```

```text
/fuel Cyberpunk night market, neon BUZZ logo on fog, handheld energy, 5s
```

```text
/fuel Minimal white studio, matte black "BUZZ" type, slow orbit camera, 5s
```

Max **500** characters after `/fuel `.

## What you should see on Buzz (camera)

1. **Pay with Telegram Stars**  
   - `Prompt: "…"` block  
   - deep link + QR  
   - tip line about `/fuel <prompt>`
2. After Stars: **Fueled** (same Prompt block)
3. **Delivered** (same Prompt block + share URL)

## Film sequence (~3–4 min)

1. Screen record Desktop on **#Buzz Fuel C1 Dry**
2. Take A: bare `/fuel` → pay 1★ (whitelist) → wait Delivered
3. Take B: `/fuel` + different prompt → pay → Delivered  
   Side-by-side share links so reels clearly differ
4. Optional Take C: third prompt

## Stack notes

| Layer | Role |
|-------|------|
| Local adapter | parses `/fuel` / `/fuel <prompt>`, posts Prompt on Buzz |
| Railway makereel-core | stores prompt on intent; uses it on generate |
| TG Stars | still whitelist → 1★; gateway x402 still settles job |

Redeploy wiped process-local intents (expected). Old unpaid intents are gone after deploy.
