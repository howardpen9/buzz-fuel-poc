# Hosted OG Community：Agent、權限與對外開放模型

**調查日期：** 2026-07-23  
**範圍：** `wss://og.communities.buzz.xyz` 與 Howard 目前的 Buzz Desktop managed-agent records。  
**原則：** Community hosting 與 agent execution 是兩件不同的事。

## 0. 2026-07-23 remediation

- 所有 9 筆 local records（3 persona definitions、3 localhost instances、3 OG instances）的 `parallelism` 從 24 降為 1。
- 三個仍用舊值運行的 localhost harness 已 graceful stop；72 個 Claude ACP children 已退出。
- 變更後驗證：所有 records 都是 `parallelism=1`，Buzz agent process count 為 0。
- 下次從 Buzz UI Start 任一 instance 時，它會以一個 ACP worker 啟動。
- 原設定備份：`~/Library/Application Support/xyz.block.buzz.app.dev.main/agents/managed-agents.json.parallelism-24-backup-20260723T1419`。

## 1. 最短答案

`og.communities.buzz.xyz` 是 Block-operated Buzz hosted Service。Block 負責 relay、資料庫、Redis/object storage、TLS 與服務營運；你的 Desktop 負責 UI。

但是你目前建立的 OG agents 都是：

```text
backend = local
runtime = claude-agent-acp
start_on_app_launch = false
parallelism = 1
```

所以：

- OG relay 不會自動替你執行 Claude。
- 你從 Buzz Desktop 按 Start 時，agent process 會在你的 Mac 啟動。
- 你的 Mac 必須在線、不能睡眠，agent 才能持續回應 OG。
- 模型 credential、模型費用與本機工具權限仍由你承擔。

## 2. 四層心理模型

```text
Block-hosted OG community
  ├─ 儲存成員、channel、message、agent identity、presence
  ├─ 驗證 NIP-42 / NIP-43 / channel membership
  └─ 把符合條件的 event 經 WebSocket 送出
                    │
                    ▼
Howard 的 Buzz Desktop / buzz-acp（Mac）
  ├─ 檢查 respond-to author gate
  ├─ 啟動 N 個 claude-agent-acp subprocess
  ├─ 把 event 變成 agent prompt
  └─ 提供 Buzz CLI / MCP / workspace tools
                    │
                    ▼
Connected model provider
  └─ Claude / Codex / Goose 所使用的模型 API 或帳號
```

「Hosted community」只把第一層交給 Block，不等於第二、三層也在 Block 雲端。

## 3. 你現在實際有什麼

### `parallelism` 是什麼

它是**每一個 managed-agent identity 背後，同時預備多少個 ACP worker subprocess**。

本次 Buzz source 在 `buzz/desktop/src-tauri/src/managed_agents/types.rs` 明確硬編碼：

```rust
pub const DEFAULT_AGENT_PARALLELISM: u32 = 24;
```

所以原本的 24 是該版本的產品預設，不是 Howard 手動設定，也不是依 Mac CPU core 數自動計算。建立 Fizz／Honey／Bumble instances 時，它們繼承了這個預設。

```text
parallelism = 1
  一個 Buzz agent identity
  └─ 一個 ACP/model worker

parallelism = 24
  一個 Buzz agent identity
  └─ 24 個 ACP/model workers
```

這 24 個 worker 對使用者看起來仍是同一個 Fizz／Honey／Bumble，使用同一個 Nostr identity。它不是 24 個不同角色，也不會讓單一回答品質變成 24 倍。

Pool 的用途是讓同一 identity 同時處理不同 channel 的 work。Buzz queue 仍保證同一 channel 不會由兩個 worker 同時處理；跨 channel 才可能並行。每個 worker 可能再帶自己的 MCP subprocess，因此 RAM、process 數與潛在模型併發成本近似隨 N 成長。

對目前單一 Buzz Fuel Demo：

```text
parallelism = 1
```

已足夠。只有在真實使用量證明多個 channel 長期排隊時，才應逐步增加到 2；不應從 24 起步。

### OG records

| Agent | Trigger gate | Pool | Running |
|---|---|---:|---|
| Fizz | owner-only | 1 | no |
| Honey | owner + 1 allowlisted pubkey | 1 | no |
| Bumble | owner + 1 allowlisted pubkey | 1 | no |

三者都是 local backend、Claude ACP、`start_on_app_launch=false`。目前 OG agent 不消耗本機 agent runtime；只保留設定與 relay-side identity/presence records。

### localhost records

Fizz、Honey、Bumble 的 localhost instance 已停止，pool 已設為 1。下次三者全部啟動時，預期是三個 `buzz-acp` harness 與每個一個 ACP worker，而不是 72 個 workers。

這也更正一個先前誤判：Fizz／Honey／Bumble 的 persona 名稱確實來自內建 catalog，但你已經把同名 persona 部署成真正 managed-agent instances。它們不只是 mock cards。

## 4. 別人能不能直接叫你的 agent？

要讓一個人的訊息真正進入 agent turn，至少要連過這些 gates：

1. **Community membership**  
   對方必須先成為 OG relay member。
2. **Channel membership**  
   Agent 與對方都要能進入同一個 channel；private channel 必須明確加入。
3. **Subscription rule**  
   預設是 @mention agent；普通聊天不會直接啟動 agent。
4. **Inbound author gate**  
   `owner-only` 只接受 owner；`allowlist` 只接受 owner 與列出的 pubkeys；`anyone` 才是任何成員都能觸發。

所以你目前的 OG 設定下，陌生 member 即使看到並 @mention Fizz，也不應被轉成 agent prompt；Honey／Bumble 只多開放給各自的一個 allowlisted pubkey。

但這不是完整的 prompt-injection 防線：

- Agent 身為 channel member，使用 Buzz CLI 查歷史時仍可能讀到其他成員寫的內容。
- 一個被允許的使用者可能引用、轉貼或要求 agent 處理不可信內容。
- Agent 若有 shell、檔案、MCP、wallet 或 production secret，誤執行的 blast radius 來自工具權限，不只來自「誰能 @mention」。

因此 public channel 的 agent 應使用獨立、低權限 runtime；不要只依賴 `respond-to`。

## 5. Hosted 與 self-hosted 差別

| 面向 | Block-hosted OG | Self-hosted |
|---|---|---|
| Relay／DB／Redis／storage | Block 營運 | 你營運 |
| TLS、升級、availability | Block 負責主要運維 | 你負責 |
| 資料與 metadata | 儲存在 Block-operated Service | 儲存在你的 infra |
| Server-side 可見性 | 不是 E2E；operator/storage layer 可處理內容 | 由你的 operator/storage policy 決定 |
| Agent execution | 依 backend；你目前是 Mac local | 仍可選 laptop、server 或 remote provider |
| Model credential／費用 | 你提供 connected model | 你提供 connected model |
| 備份／restore／監控 | 依 Block hosted policy | 全部是你的責任 |
| Community member管理 | UI invite／npub、受 hosted policy 約束 | UI/CLI/DB，完全由你控制 |
| 成本 | 少掉 relay ops；仍有 model 與本機 agent 成本 | infra + ops + model + agent compute |
| 可遷移性 | Nostr identity portable；資料 export/migration 要實測 | DB/domain/backup 都由你掌握 |

本機 self-host 的 idle infra 抽樣約 0.7–0.8 GiB RAM；真正的 agent 成本近似：

```text
agent count × parallelism × (ACP runtime + MCP + model/client overhead)
```

原本的 `3 × 24` 比 relay 本身更值得先處理；2026-07-23 已降為 `3 × 1` 並停止舊 pools。

## 6. Email 通過代表什麼

OG 的 Terms 與 Privacy 文件表明它是 Block-operated hosted Service；要 Block 代為 host workspace，需要建立 account 並成為 hosted workspace administrator。

但僅從「收到 email 批准」不能確定你目前是：

- OG 的 owner；
- admin；
- 或只是 member。

兩分鐘內可在 Buzz Desktop 核對：

```text
Settings → Community access
```

- 看得到完整 member list、Create invite link、角色旁有 Crown：owner。
- 能建立 invite、管理一般 members：admin。
- 看不到 Community access 管理卡：通常只是 member。

這個角色比 email 文案更可信，因為它直接來自 relay 的 NIP-43 membership state。

## 7. 對外開放 OG：安全的三階段

### Phase A — 2–5 位可信測試者

1. 優先用 **Add member by npub**；需要 link 時選 1-day expiry。
2. 只開一個測試 channel；不要把測試者加入含 secrets／內部紀錄的 channel。
3. Agent 設 `parallelism=1`、`respond-to=allowlist`，只加入測試者 pubkeys。
4. Agent 使用乾淨 workspace、read-only/approval-first tools、無 production wallet/key。
5. 設每日 turn／模型預算與人工 kill switch；先觀察一週。

### Phase B — 邀請制 beta

1. Public discussion agent 與 privileged operator agent 分成不同 identity。
2. 邀請 link 保持短效；注意它在到期前是 multi-use bearer link。
3. 新 member 預設只進 public/onboarding channels，不自動進 agent execution channel。
4. 為每個 agent 設 allowlist、rate limit、最大 turn duration 與審批政策。
5. 保存 agent audit、tool calls、cost、cancel/timeout 與 abuse reports。

### Phase C — 公開社群

不要把有本機 shell／repo write／wallet 能力的 agent 設成 `respond-to=anyone`。

公開版至少要拆成：

```text
Public concierge agent
  └─ 無本機敏感 workspace、低預算、無 secrets、可被任何 member 使用

Privileged build/payment agent
  └─ private channel、owner/allowlist only、人工 approval、獨立 key
```

如果 MacBook 不是 24/7 server，公開 agent 應搬到隔離的 VM/container/remote backend；不要為了 availability 讓個人電腦長期暴露高權限 runtime。

## 8. Invite link 的重要限制

Buzz 現行 invite code：

- 預設有效 72 小時；UI 可選 1、3、7、30 天。
- 到期前可以被多人使用。
- 只能授予 `member`，不能直接授予 admin/owner。
- 單一 invite code 目前不能個別撤銷。
- 洩漏後可移除已加入 member；要使所有未到期 codes 失效，需要 relay operator 輪替 relay key。

因此公開貼一個 30-day invite link 等同開一張可轉傳的 multi-use 入場券，不適合作為第一階段。

## 9. LOCK / OPEN / PARK

### LOCK

1. OG 是 Block-hosted community；agent compute 是獨立層。
2. 你目前的 OG agents 是 local backend、停止中、parallelism 1，不由 Block 代跑模型。
3. 公開測試先採 dedicated channel + allowlist + parallelism 1。
4. Privileged agent 不對 public members 開 `anyone`。

### OPEN

| 問題 | 為什麼重要 | Owner | 下一步 |
|---|---|---|---|
| Howard 在 OG 是 owner/admin/member？ | 決定能否邀請與移除 members | Howard | 在 Settings → Community access 看角色 |
| OG agent 實際 workspace/tool permission？ | 決定 prompt injection blast radius | Agent operator | 在 Edit Agent 匯出非秘密設定與 Doctor 結果 |
| 哪一個 agent 要給社群使用？ | 決定 identity 與安全配置 | Howard | LOCK public concierge vs privileged fuel bot |
| Block hosted 的資料 retention/export/刪除流程？ | 決定是否適合正式資料 | Howard + Block support | 依 hosted Privacy/Terms 與產品 export 實測 |

### PARK

- Public `respond-to=anyone`：等隔離 runtime、quota、abuse handling 完成。
- 把 privileged x402/payment agent 搬到 remote VM：先完成 canonical POC demo。
- Self-host migration：等 hosted beta 產生明確 ops／data-control 痛點再評估。

## 10. 現在不要做的事

- 不要公開 30-day invite link。
- 不要在沒有 queue-depth 證據時把 agent parallelism 從 1 調高。
- 不要讓 public member 觸發帶 wallet、production key 或 unrestricted shell 的 agent。
- 不要把 hosted relay 誤認為 hosted model/agent compute。
- 不要用 `pkill node` 清理；從 Buzz UI 正常 Stop agent，再確認 PID。
