# Buzz Agent Integration：未來開發者提醒與 Upstream Feedback

**來源：** Buzz Fuel POC 的 hosted OG、localhost、Telegram Stars 與 x402 整合經驗。  
**用途：** 開新 Buzz integration 前的 10 分鐘檢查，以及回報 Buzz upstream 時的事實底稿。  
**範圍：** 只記錄可重現的產品／開發摩擦；不把 MakeReel 自己的 evidence 或 pricing 問題誤歸給 Buzz。

## 1. 開工前最重要的五件事

### 1.1 Hosted community 不等於 hosted agent

先畫清楚：

```text
Community relay 跑在哪裡？
Agent harness 跑在哪裡？
Connected model 跑在哪裡？
Tools／workspace 在哪裡？
誰付模型費用？
```

Buzz hosted relay 可以由 Block 營運，但 `backend=local` 的 managed agent 仍在開發者電腦執行。Mac 睡眠、離線或 Desktop 關閉時，agent 不再可用。

### 1.2 Persona、instance、worker 是三種東西

- Persona：名字、prompt、預設行為的 definition。
- Managed-agent instance：部署到某個 community 的 identity/config。
- Worker：instance 背後實際執行的 ACP subprocess。

同名 Fizz 可以同時有 builtin persona、localhost instance 與 OG instance。看到一張 agent card 或綠色 presence，不能推斷 process 數或執行位置。

### 1.3 啟動前先算 worker 數

```text
總 workers ≈ 啟動中的 instances × parallelism
```

本次三個 instances 繼承 `parallelism=24`，所以產生 72 個 Claude ACP children。單一路徑 POC 應從 1 起步；只有 queue depth 證明需要時才增加到 2。

### 1.4 三層 access gate 不要混在一起

```text
Relay membership
  → 能否進 community

Channel membership
  → 能否看／寫特定 room

respond-to author gate
  → 訊息是否被送進 agent turn
```

`respond-to=owner-only/allowlist` 只控制誰能直接觸發，不會讓 channel history 變成可信內容。Agent 主動查歷史時仍可能讀到其他人的 prompt injection。

### 1.5 Saved config 不等於 running config

修改 parallelism 等 runtime config 後，現有 subprocess 不會立刻變少。Buzz 會標記 config drift，並在符合 observer、idle、quiescence 條件後才安全 auto-restart；否則需要明確 Stop → Start。

驗收必須同時看：

- record 中的新值；
- UI 的 restart-needed 狀態；
- 實際 PID／children 數。

## 2. 安全提醒

### Public agent 與 privileged agent 要分開

```text
Public concierge
  無 production secrets
  無 wallet
  無 unrestricted shell
  低預算／低 timeout

Privileged build/payment agent
  private channel
  owner-only／allowlist
  人工 approval
  獨立 identity
```

不要因為 `respond-to` 是 allowlist，就把 production wallet、Telegram token、repo write 與 unrestricted shell 全放進同一個 agent。

### Invite link 是 bearer credential

目前 Buzz invite link：

- 到期前可多人使用；
- 可以被轉傳；
- 無法個別撤銷；
- 只能授予 member，但 member 仍可能看到所有他被加入的內容。

小規模測試優先用 Add member by npub；使用 link 時選 1 day，避免公開 30-day link。

### localhost 暴露範圍

確認 relay 是 bind `127.0.0.1` 還是 `*:3000`。若是後者，同一 LAN 可能有機會連入；手機 demo 需要 LAN 時才限時開放。

### Hosted data 不是自動 E2E

Hosted relay 需要儲存、索引、備份 messages/events。不要把 wallet key、API token、raw payment header 或完整 Telegram charge ID 寫進 channel content。

## 3. 開發與驗收提醒

### Agent identity 也需要 membership

人的 identity 能進 OG，不代表新 agent identity 自動成為 relay member。建立 agent 後必須確認：

1. Agent 是 relay member。
2. Agent 被加入指定 channel，role 為 bot。
3. Agent 使用正確 owner attestation／identity。
4. Agent 能完成 NIP-42 auth。

否則容易把 membership 問題誤診成 runtime crash。

### 不要用 presence 當健康證明

綠點只能表示最近 presence/heartbeat。正式健康檢查至少包含：

- process alive；
- relay WebSocket connected；
- author gate 正確；
- test mention 被接收；
- reply event 有完整 Nostr signature。

### 啟動會 replay 舊 mentions

ACP harness 重新啟動後可能 replay 尚未處理的 mentions。公開測試前先清楚定義：

- replay cursor；
- idempotency；
- 舊 payment/funding intent 不可再次執行；
- message event ID 去重。

對付款 bot 而言，一次 Stars payment 最多只能建立一個 x402 job。

### UI 成功不是 evidence

保留跨系統 correlation：

```text
Telegram charge
→ funding intent
→ x402 job
→ Base transaction
→ signed Buzz event
→ result URL
```

Whitelist 價格與 canonical 價格要分開判定；不可用 1-Star run 證明 75-Star pricing path。

### 清理要有正確層級

不要使用：

```bash
pkill node
```

正確順序：

1. Buzz UI Stop managed agent。
2. 確認該 instance PID 與 children 消失。
3. 停止不需要的 local relay／Docker services。
4. 關閉獨立的 coding-engine sessions。
5. 保留 feature flag rollback evidence。

## 4. 值得回報 Buzz upstream 的產品問題

### P0 — Parallelism default 與 eager resource use

問題：

- Source 預設 `DEFAULT_AGENT_PARALLELISM = 24`。
- 建立 instance 後會繼承 24。
- 三個 instance 可產生 72 個 ACP workers。
- UI 建立流程沒有把「將啟動 24 個 local subprocess」當成高成本決策提醒。

建議：

- 預設改成 1；需要吞吐時讓使用者主動增加。
- Lazy pool 成為預設，只在 work 到達時啟動 worker。
- Start 前顯示 `instances × parallelism = total workers`。
- 顯示預估 process／memory／model concurrency。
- 在筆電或低記憶體環境對高值加 warning。

### P0 — Hosted vs local execution 不夠明顯

建議在 agent card 長期顯示：

```text
Runs on: This Mac
Relay: og.communities.buzz.xyz
Model: Claude
Workers: 1
Access: Owner only
```

不要只在建立 dialog 顯示 `This computer`。Hosted community 很容易讓使用者以為 agent 也由 hosted operator 執行。

### P1 — Saved config 與 running config 缺乏差異提示

建議：

- 同時顯示 `Saved: 1 / Running: 24`。
- 提供 `Restart now when idle`。
- 顯示 auto-restart 被哪個 gate 阻擋：working、observer disconnected、quiescence timer。
- Stop/Start 後驗證實際 worker count。

### P1 — Persona 與 deployed instance 容易混淆

建議：

- Persona card 標 `Definition`。
- 真 instance 標 community、backend、PID/provider、running/stopped。
- 同名 instance 使用 community badge。
- UI 不用 presence 綠點代替 process/runtime 狀態。

### P1 — Access model 需要合併說明

建議在 `respond-to` UI 明確寫：

> This controls who can trigger a turn. It does not prevent the agent from reading other channel content through tools.

並在公開選擇 `anyone` 時提醒 workspace/tool blast radius。

## 5. 可直接貼到 GitHub 的 Issue Draft

### Title

```text
Managed agents default to parallelism 24 and eagerly spawn 24 ACP workers per instance
```

### Body

````markdown
## Summary

Creating a locally managed agent without overriding parallelism inherits:

```rust
pub const DEFAULT_AGENT_PARALLELISM: u32 = 24;
```

In a small demo workspace, starting three managed agents created three
`buzz-acp` harnesses and 72 direct `claude-agent-acp` child processes.

The agents were idle, but the process count and local resource footprint were
surprising. The hosted community UI also made it easy to miss that the agents
were running on the local Mac rather than on hosted infrastructure.

## Environment

- macOS, 24 GiB RAM
- Buzz Desktop development build
- Three local-backend managed agents
- Runtime: `claude-agent-acp`
- Each instance inherited `parallelism=24`
- Relay targets tested: localhost and a hosted community

## Actual behavior

```text
3 instances × 24 workers = 72 ACP child processes
```

Changing the saved value did not change the running pool until the instances
were restarted.

## Expected behavior

For a newly created local agent, default to one worker unless the user
explicitly requests concurrency. The UI should show the total process impact
before Start.

## Suggestions

1. Default `parallelism` to 1.
2. Make lazy worker startup the default.
3. Show `instances × parallelism = total workers` before launch.
4. Show saved vs currently-running parallelism when restart is required.
5. Keep “Runs on this computer” visible on the agent card.

## Why this matters

Most first-time users are testing one agent in one channel. A default pool of
24 adds laptop resource pressure and model concurrency risk before there is
evidence that cross-channel queue depth requires it.
````

## 6. 哪些不是 Buzz upstream 的責任

- MakeReel whitelist 1-Star 與 canonical 75-Star pricing 的驗收差異。
- x402 transaction/payment-response evidence 保存。
- Telegram charge correlation 與 privacy redaction。
- Buzz Fuel feature flag rollback。
- Implementer 與 reviewer 的角色分離。

這些要保留在本專案的 acceptance/runbook，不能用 upstream issue 取代。
