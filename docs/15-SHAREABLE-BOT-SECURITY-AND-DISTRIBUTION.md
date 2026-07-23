# MakeReel Fuel：可分享 Bot 的安全與發佈規格

**狀態：** Draft for implementation review  
**目的：** 把已跑通的 Buzz → Telegram Stars → x402 → delivery 流程，收斂成其他 community 可以安全部署的 acquisition kit。  
**不在本文件範圍：** Fuel prompt 文案、QR 視覺、任意模型／SKU、ACP runtime、分潤。

## 0. 先講結論

這套流程可以變成 MakeReel 的 distribution loop：

```text
Community 安裝一個薄 Buzz adapter
  → member 在 Buzz 按 /fuel
  → adapter 顯示 MakeReel 官方 Telegram deep link／QR
  → @MakeReel_xyz_bot 收 Stars
  → MakeReel core 代付 x402、執行 job、退款或交付
  → adapter 把 signed status／result 貼回原 Buzz channel
```

對外分享的是 **低權限 adapter**，不是付款核心、wallet 或通用 agent。

如果 Stars 要進 MakeReel，merchant 必須固定為：

```text
https://t.me/MakeReel_xyz_bot
```

不能讓 community deployer 任意換 bot username。否則 Stars 會進另一個 bot owner，付款責任、退款、客服與詐騙風險也會一起分裂。White-label merchant 是另一個產品，不能混在第一版。

## 1. 安全邊界

```text
Untrusted / community-controlled
┌──────────────────────────────────────┐
│ Buzz member / channel / hosted relay │
│ Community-operated thin adapter      │
└───────────────────┬──────────────────┘
                    │ opaque intent/status capability
                    ▼
MakeReel-controlled
┌──────────────────────────────────────┐
│ Intent API + canonical quote         │
│ @MakeReel_xyz_bot + Stars payment    │
│ payer binding + idempotency/refund   │
│ platform signer + x402 settlement    │
│ result delivery + audit ledger       │
└──────────────────────────────────────┘
```

### 1.1 Community adapter 可以擁有

- 自己的 Buzz／Nostr bot private key。
- `installation_id`。
- 每個 installation 獨立且可撤銷的 MakeReel credential；若 P0 採公開 signed-intent API，則不需要長期 credential。
- Relay URL、允許的 community／channel、固定 SKU ID。
- 只能讀取自己建立的 intent status。

### 1.2 Community adapter 永遠不應拿到

- `@MakeReel_xyz_bot` 的 Telegram bot token。
- Platform signer private key。
- Gateway internal token。
- x402 payment signature。
- 完整 Telegram payment charge ID。
- MakeReel database/admin credential。
- 其他 installation 的 intent、payer 或 result。

### 1.3 MakeReel 必須保持 server-authoritative

- SKU、route、network、asset、`payTo`、x402 amount。
- Stars 換算、invoice amount、quote TTL。
- Telegram payer binding 與 successful payment。
- 一筆付款最多建立一個 job。
- x402 settle、poll、delivery 與 refund。
- Installation attribution 與使用量配額。

Community 傳來的 price、route、callback URL 或 execution target 都視為不可信輸入。

## 2. 最重要的威脅與控制

| 優先級 | 風險 | 必要控制 |
|---|---|---|
| P0 | 假 deep link／QR 把人帶去冒牌 bot | Official kit 固定 `t.me/MakeReel_xyz_bot`；QR payload 驗證；畫面標示 seller；不允許 env 覆寫 merchant username |
| P0 | Intent 被猜中、轉貼或重播 | 高 entropy、短效、single-purpose、single-use token；綁 installation/community/channel；開 invoice 前綁 TG user |
| P0 | Telegram webhook retry 造成重複 job | `telegram_payment_charge_id` DB unique；atomic state transition；一 charge 一 job；所有 handler idempotent |
| P0 | 重播 `/fuel` 或舊 Buzz event | 以 signed Nostr event ID 去重；限制未付款 intent 數；intent 自動到期 |
| P0 | Platform signer 被任意扣款 | Signer 只在 MakeReel；固定 SKU allowlist；驗證 network/asset/payTo/amount；per-job、per-install、daily cap；circuit breaker |
| P0 | 付款成功但 job 失敗／交付中斷 | Durable state machine；retry-safe fulfillment；exactly-once refund；不要在 `successful_payment` 前交付 |
| P0 | Public agent 取得本機 shell／secrets | Distribution kit 不需要 LLM/ACP；deterministic adapter；container 無 workspace、wallet、shell tool 或 production secrets |
| P0 | 多租戶資料串線 | 所有查詢以 `installation_id` + intent scope；cross-tenant negative tests；result URL 不可猜或需授權 |
| P1 | Callback 偽造、重播或 SSRF | P0 優先用 adapter polling；若用 webhook，HMAC/Ed25519 + timestamp + nonce + replay cache；callback host allowlist |
| P1 | Logs/evidence 洩漏付款或身份 | 公開 log 只留 correlation ID、hash/suffix；TG identity、raw charge、signer/payment signature 不得輸出 |
| P1 | 大量 unpaid intents／spam | Pubkey、installation、community、IP rate limit；最大 outstanding intents；TTL cleanup；concurrency 預設 1 |
| P1 | 惡意 prompt／內容造成濫用 | P0 固定 prompt/SKU；未來開放輸入時做長度、類型、content policy 與 moderation |
| P1 | 惡意 fork 假冒 official kit | Signed release/container、checksum、pinned digest、SBOM、安全公告與 revoke 機制 |
| P1 | MakeReel 中央服務故障仍持續收款 | Pre-checkout fail closed；payer/gateway/capacity health gate；不能履約時不要接受新付款 |

## 3. Payment 與 x402 的正確證據鏈

```text
signed Buzz event
  → installation_id + intent_id
  → Telegram user binding
  → successful_payment + durable charge correlation
  → job_id
  → verified x402 PAYMENT-RESPONSE / Base tx
  → result_url
  → signed Buzz delivery event
```

每一步共用一個不含秘密的 correlation ID。Operator 必須能從任何一段往前後追查，但 community adapter 不必看到完整 payment record。

### 3.1 Telegram Stars

- 數位商品／服務使用 `XTR`。
- 只在收到 `successful_payment` 後履約。
- 保存完整 `telegram_payment_charge_id` 在 MakeReel 的受控 storage，供退款使用。
- `pre_checkout_query` 必須及時回答；容量、quote、payer 或 gateway 不健康時 fail closed。
- Invoice payload 只放 opaque intent reference，不放 TG identity、price authority 或 secret。
- 重複 webhook、重複 pre-checkout 與重複 successful payment 都必須安全。

### 3.2 x402

- Payment requirements 只能由 canonical MakeReel/gateway quote 產生。
- Settle 前重新驗證 network、asset、amount、recipient、expiry 與 SKU。
- 保存 redacted `PAYMENT-RESPONSE`、tx hash、payer、amount 與 job correlation。
- Gateway internal bypass 可以作 recovery fallback，但不能被宣稱為真 x402 settlement。

## 4. Public deployment contract

### 4.1 可公開設定

```text
MAKEREEL_BOT_USERNAME=MakeReel_xyz_bot
MAKEREEL_PUBLIC_BASE_URL=https://<official-domain>
BUZZ_RELAY_URL=wss://<community-relay>
BUZZ_COMMUNITY_ID=<community>
BUZZ_CHANNEL_ID=<allowed-channel>
MAKEREEL_INSTALLATION_ID=<issued-installation>
MAKEREEL_SKU_ID=<fixed-supported-sku>
```

`MAKEREEL_BOT_USERNAME` 與 public API origin 應由 signed manifest 或 build-time allowlist 驗證，不應只是任意字串。

### 4.2 每個 deployment 的 secrets

```text
BUZZ_NOSTR_PRIVATE_KEY=<unique-per-bot>
MAKEREEL_INSTALLATION_TOKEN=<unique-revocable-token-if-required>
```

不得共用全域 deployment token。需要能單獨 rotate/revoke 某一個 installation，而不影響其他 community。

### 4.3 Kit 應包含

- Deterministic adapter binary/container。
- `.env.example`，逐項標示 public、secret、forbidden。
- `doctor`／dry-run：檢查 relay auth、channel membership、official MakeReel endpoint、固定 SKU，不花 Stars。
- Healthcheck、structured redacted logs、version command。
- 安裝、升級、撤銷、移除與 incident 回報說明。
- Signed checksum/container digest、dependency lock、SBOM。
- 預設 non-root、read-only filesystem、最小 egress allowlist。

## 5. Merchant、客服與退款責任

第一版的 seller/merchant 是 MakeReel，不是 community operator。

因此 MakeReel 必須提供：

- `/terms`。
- `/support` 與 `/paysupport`。
- 明確的價格、交付內容與退款條件。
- 可由 `telegram_payment_charge_id` 執行的退款流程。
- 退款 retry 不造成重複退款。
- Bot owner account 啟用 2-step verification。
- Payment、intent、job ledger 的備份與 recovery test。

Community 可以帶來流量，但不應在文案上假裝自己處理付款或保證 MakeReel 的退款。若未來要分潤，需另建 attribution ledger、結算、稅務與爭議規則。

## 6. Acquisition attribution

這個 kit 的商業價值不是「多一個 bot」，而是每個 community 都變成 MakeReel 的入口。

建議 funnel：

```text
installation
→ /fuel shown
→ deep link / QR opened
→ Telegram /start
→ invoice shown
→ paid
→ x402 settled
→ delivered
→ repeat purchase
```

安全規則：

- Attribution token 必須 opaque、signed、短效且不可竄改。
- `installation_id` 只能影響 attribution，不可改 price、SKU、payTo 或權限。
- 最好把 attribution reference 與 payment authorization capability 分離。
- P0 只做 attribution，不做 commission。
- QR scan、bot start 與 unpaid invoice 都不是收入。

## 7. Rollout

### Phase A — Curated pilot

對象：2–5 個可信 community。

- 固定 MakeReel bot、固定 SKU、固定 prompt。
- `parallelism=1`，精確 `/fuel` trigger，限制單一 channel。
- Adapter polling status，不開 public callback。
- 每個 installation 有 quota、revoke 與 kill switch。
- 人工審核安裝，不做 self-service。

成功條件：canonical 非白名單付款、x402 settlement、delivery、退款演練、跨 tenant negative tests 全部通過。

### Phase B — Invite-only self-service

- Installation registration service。
- Per-install credential、quota、rotation、revoke。
- Signed deployment manifest。
- Operator dashboard：intent、payment、job、refund、spend、abuse。
- Automated secret scan、image scan、dependency update。

### Phase C — Public distribution

- Stable versioned API 與 compatibility policy。
- Abuse detection、WAF/rate limit、incident response owner。
- Public security contact與漏洞通報流程。
- One-click deploy recipes，但仍維持 merchant/signing core centralized。

## 8. Release gates

公開給第三方部署前，下列全部必須有 evidence：

- [ ] 第二個非白名單 Telegram 帳號完成 canonical Stars price。
- [ ] 相同 `successful_payment` 重送不建立第二個 job。
- [ ] 相同 Buzz event 重播不建立第二個有效 intent。
- [ ] Expired、tampered、已使用 intent 均 fail closed。
- [ ] 錯誤 TG user 無法使用別人的 bound intent。
- [ ] Community 無法改 Stars price、x402 amount、route、network、asset 或 payTo。
- [ ] x402 failure 能退款，retry 不會重複退款。
- [ ] Signer per-job/daily cap 與 emergency circuit breaker 有測試。
- [ ] Cross-installation status/result access 被拒絕。
- [ ] Logs、screenshots、evidence pack 通過 secret/PII scan。
- [ ] Adapter image 不含 MakeReel bot token、signer 或 gateway internal token。
- [ ] Stop adapter + feature flag off 能阻止新付款，既有 paid job 仍可安全收尾。
- [ ] `/terms`、`/support`、`/paysupport` 與 refund owner 已上線。
- [ ] 官方 deep link、QR 與 seller identity 有 anti-phishing 檢查。

任何 P0 gate 未通過，只能繼續 curated demo，不能宣稱為 public self-service deployment。

## 9. LOCK / OPEN / PARK

### LOCK

- 一個 canonical MakeReel Telegram merchant bot。
- Community 部署 deterministic thin adapter，不部署付款核心。
- MakeReel 控制 quote、Stars invoice、payer binding、x402 signer、fulfillment、refund。
- P0 固定 SKU／prompt；intent opaque、短效、single-use。
- Community 永遠拿不到 Telegram merchant token、platform signer 或 internal gateway token。
- P0 用 polling status，避免 public webhook/SSRF 面。

### OPEN

| 決策 | 建議預設 | Owner | 何時要鎖 |
|---|---|---|---|
| Intent creation auth | Curated pilot 用 per-install token | Product + Security | Phase A 前 |
| Result URL privacy | 短效 signed URL 或 authenticated fetch | Product + Security | Public beta 前 |
| Packaging | 先 Docker，再評估 binary/one-click | Eng/Ops | Pilot 後 |
| Merchant/legal wording | 「Sold and fulfilled by MakeReel」 | Howard/legal | 收第一筆公開付款前 |
| Installation attribution retention | 最小化並設 retention | Product/Privacy | Phase B 前 |
| Public webhook | 先不做；有明確需求才加入 | Eng/Security | Phase B 評估 |

### PARK

- Community 自己的 Telegram merchant bot。
- White-label payment。
- Referral commission／revenue share。
- 任意 prompt、任意 SKU、任意 model。
- ACP/MCP/Agent Fuel Runtime SDK。
- Public `respond-to=anyone` privileged agent。
- 多人共同出資同一個 intent。

## 10. 下一個可執行決策

先不要立刻做「任何人一鍵部署」。

下一輪只交付一個 **Curated Community Kit v0**：

1. 固定 `@MakeReel_xyz_bot`。
2. 固定一個 SKU。
3. Community 只設定 Buzz relay/channel、bot key、installation token。
4. Status 使用 polling。
5. 先讓 2–5 個 community 完成上面的 release gates。

這樣能驗證 distribution 與 conversion，同時避免把 merchant rail、signer 與本機高權限 agent 公開出去。

## References

- Telegram, [Bot Payments API for Digital Goods and Services](https://core.telegram.org/bots/payments-stars)
- Telegram, [Bot API](https://core.telegram.org/bots/api)
- Coinbase Developer Platform, [How x402 Works](https://docs.cdp.coinbase.com/x402/core-concepts/how-it-works)

