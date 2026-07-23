# Buzz Community：線上架構、安全性與本機資源 Guidance

**檢查時間：** 2026-07-23，Asia/Ho_Chi_Minh  
**範圍：** `wss://og.communities.buzz.xyz`、本機 `ws://localhost:3000`、Buzz Desktop 所顯示的 agent。  
**方式：** 公開端點與本機唯讀檢查；沒有修改部署、設定、資料或程序。

## 1. 先講結論

1. `og.communities.buzz.xyz` 的公開表面是 **Cloudflare → Envoy → Buzz relay**。本地 Buzz 文件描述正式部署可使用 Docker/ECR、Terraform、Helm、ArgoCD 與 Kubernetes，但只靠公開端點不能證明 OG production 的底層雲供應商、region 或 pod 數量。
2. OG relay 公開宣告 `auth_required: true`、`restricted_writes: true`，並支援 NIP-42／NIP-43；這是正確方向，但不等於多租戶、proxy headers、media 與 secrets 已全部安全。
3. Fizz、Honey、Bumble 同時存在「內建 persona」與「真正 managed-agent instance」。只看名字或綠點不能判斷；必須檢查 instance 的 backend、relay URL、PID 與 `respond-to`。
4. 2026-07-23 14:16 的唯讀檢查發現：三個 OG instance 是 local backend 但已停止；三個 localhost instance 當時每個 `parallelism=24`，合計 72 個 Claude ACP children。當日已把所有 records 降為 1，並 graceful stop 舊 pools。
5. Mac 當時已有明顯累積負載：24 GiB RAM、約 6.9 GiB swap 已使用；14:16 的 load average 達 80／51／37。不要用廣泛 `pkill node` 清理；本次只停止三個已核對的 `buzz-acp` parent，沒有碰其他 coding sessions。

## 2. 哪些是觀察、哪些是推論

| 項目 | 結論 | 信心／來源 |
|---|---|---|
| DNS／edge | Cloudflare CDN | 公開 DNS 與 response headers，已觀察 |
| TLS | `communities.buzz.xyz` wildcard／SAN 有效憑證 | 公開 TLS handshake，已觀察 |
| reverse proxy | response 有 Envoy upstream header | 公開 response，已觀察 |
| Relay policy | auth required、restricted writes、no payment required | 公開 NIP-11 document，已觀察 |
| Relay implementation | Buzz Relay v0.2.0；支援 NIP-42／43 等 | 公開 NIP-11 document，已觀察 |
| Production orchestration | 可能是 Kubernetes + Helm + ArgoCD | 與本地官方 repo deployment docs 一致，但 OG production 尚未由 cloud/deployment repo 證實 |
| Storage | Buzz hosted mode 可共用 Postgres、Redis、object storage，以 host-derived tenant context 隔離 | source/docs 能力；OG 實際 topology 未證實 |

公開 relay document：

- `https://og.communities.buzz.xyz/`
- `wss://og.communities.buzz.xyz`

## 3. 本機到底有哪些東西會吃效能

### 3.1 三種不同概念

| 類型 | 是什麼 | 是否必然吃本機 LLM 資源 |
|---|---|---|
| Desktop agent card／綠點 | UI persona 與 presence 狀態 | 否 |
| Buzz relay | WebSocket、event、subscription 與 presence transport | 少量 CPU／RAM |
| ACP agent process | 實際啟動 Codex、Claude、Goose 等 subprocess，可能帶工具與 workspace | 是；依 pool 與任務而變 |

Fizz／Honey／Bumble 的名稱與 system prompt 定義在：

```text
buzz/desktop/src/testing/e2eBridge.ts
```

這段 bridge 只應在 `window.__BUZZ_E2E__` 測試模式載入。但同名 persona 可以被部署成真正的 managed-agent instance，因此「名字是內建的」不代表「目前沒有 process」。

真正的 ACP pool 由 `BUZZ_ACP_AGENTS` 控制，允許 1–32，sample config 預設為 1。Presence heartbeat 只代表 process 最近報到，不代表正在進行 inference。

### 3.2 目前 managed-agent inventory

唯讀檢查 `managed-agents.json` 的非秘密欄位：

| Relay | Agent | Backend／runtime | `respond-to` | Parallelism | 現況 |
|---|---|---|---|---:|---|
| OG | Fizz | local／Claude ACP | owner-only | 1 | stopped |
| OG | Honey | local／Claude ACP | allowlist（1 人） | 1 | stopped |
| OG | Bumble | local／Claude ACP | allowlist（1 人） | 1 | stopped |
| localhost | Fizz | local／Claude ACP | owner-only | 1 | stopped |
| localhost | Honey | local／Claude ACP | allowlist（1 人） | 1 | stopped |
| localhost | Bumble | local／Claude ACP | allowlist（1 人） | 1 | stopped |

另外還有三筆沒有 relay URL 的內建 persona definitions；它們不是獨立 runtime。

這代表：

- OG 官方主機保存 community 資料，但目前不替這三個 instance 執行模型。
- 一旦從 Desktop 啟動 OG agent，`buzz-acp`／Claude ACP 仍會在這台 Mac 上執行並連向 OG。
- OG agent 的 `start_on_app_launch=false`；目前不會因為只開 Desktop 就自動啟動。
- localhost 的三個 instance 是 72 個 children 的原始來源；降為 1 並停止後，Buzz agent process count 已回到 0。

### 3.3 2026-07-23 抽樣

以下是單次快照，不是長期 benchmark：

| 類別 | 數量 | 約 CPU | 約 RSS |
|---|---:|---:|---:|
| `buzz-relay` | 1 | 0% | 39 MiB |
| Buzz Docker supporting stack | 5 containers | 2% | 724 MiB |
| Grok processes | 13 | 7.3% | 603 MiB |
| Codex processes | 2 | 9.7% | 159 MiB |
| Claude CLI | 1 | 2.1% | 117 MiB |
| Node processes | 101 | 接近 0%（快照） | 3.16 GiB |

Buzz Docker stack 主要記憶體：

- Keycloak：約 483 MiB。
- MinIO：約 91 MiB。
- Postgres：約 84 MiB。
- Prometheus：約 57 MiB。
- Redis：約 9 MiB。

因此這次機器的主要問題不是 `localhost:3000` relay，而是大量累積的 coding-engine／Node processes，加上 Docker supporting services。CPU 是瞬時值；RSS 類別可能有 parent/child wrapper 重疊，應視為容量線索，不作帳務級加總。

14:16 的追加快照：

- `buzz-acp` + `claude-agent-acp`：75 processes、約 351 MiB RSS、當下 CPU 接近 0%。
- 其中三個 harness 各有 24 個直接 child。
- Load average 80／51／37，表示機器當時另有大量 runnable／blocked work；不能只用 agent RSS 解釋全部負載。

## 4. 建議的本機運行檔位

### Demo／開發時

```text
Buzz relay:             1
BUZZ_ACP_AGENTS:        1
Active coding engines:  1 implementer + 1 reviewer（不要同時執行）
Docker supporting stack:只啟動這次測試真的需要的服務
Heartbeat:              不需要 presence 時設 0
```

2026-07-23 已把三個 localhost、三個 OG 與三個同名 persona records 全部降成 1。舊 localhost pools 已 graceful stop；下次由 operator 在 Buzz UI 按 Start 即會用新值。

### 開始錄影前的 5 分鐘檢查

```bash
sysctl vm.swapusage
uptime
docker stats --no-stream
lsof -nP -iTCP:3000 -sTCP:LISTEN
ps -axo pid,ppid,%cpu,rss,etime,command | sort -k3 -nr | head -25
```

判讀：

- load average 持續高於 CPU logical core 數，先停止背景 build／test。
- swap 很高但 memory pressure 正常，仍可能只是先前壓力留下；觀察 UI 是否卡頓與 page-out 是否持續。
- 同一 coding engine 有大量陳舊 process，回到對應 terminal 正常退出。
- 不要把所有 `node` 都殺掉；Orca、桌面 UI、MCP、language server 都可能使用 Node。

### 安全收尾

1. 在啟動它的 terminal 用 `Ctrl-C` 停止 dev server／agent。
2. 確認 feature flag 已回復為 false。
3. 若本次不再需要 Buzz supporting services，在正確 compose 目錄執行 `docker compose stop`；先確認 project name 與 containers。
4. 關閉已完成的 Grok／Codex／Claude sessions。
5. 再跑一次上面的唯讀檢查，留下 before／after snapshot。

## 5. OG community 安全檢查

### 已有的正向控制

- TLS 與 Cloudflare edge。
- Relay 宣告 NIP-42 authentication required。
- Relay 宣告 restricted writes，並支援 relay membership NIP-43。
- 公開 limits 包含 message size、subscription、filter 與 result limit。
- Buzz source 對 host-derived tenant context 與 cross-tenant isolation 有明確設計與測試。

### 現在值得處理的風險

#### P0 — 本機 relay 暴露範圍

抽樣時 `buzz-relay` 監聽 `*:3000`，不是只監聽 `127.0.0.1:3000`。如果 macOS firewall 或網路規則允許，同一 LAN 的其他裝置可能連入。

建議：

- 個人開發預設 bind loopback。
- 若手機 demo 需要 LAN 存取，限時開放、使用可信 Wi-Fi，並在結束後恢復。
- 不要把 dev relay 直接 port-forward 到 public internet。

#### P0 — Agent subprocess 的工具與 secrets

Relay 本身通常不是最大權限面；能讀 workspace、執行 shell、呼叫 MCP 的 ACP agent 才是。

建議：

- `BUZZ_ACP_AGENTS=1` 起步。
- 限制 workspace、respond-to allowlist、permission mode 與可用 MCP tools。
- production bot key、wallet key、Telegram token 不進聊天、event content、screen recording 或 repo。
- localhost 與 OG 使用不同 identity／credential；不要重用測試私鑰。

#### P1 — Wildcard CORS

公開 HTTP response 觀察到 `Access-Control-Allow-Origin: *`。WebSocket 安全不能只靠 CORS；NIP-42 才是 relay auth 核心。但若同一 host 還有 cookie/token-bearing HTTP API，wildcard origin 必須被刻意評估。

建議核對 production 的 `BUZZ_CORS_ORIGINS`，不要依賴 dev-mode permissive default。

#### P1 — Host／proxy／tenant binding

多租戶隔離依賴 Host／SNI 與 trusted proxy headers。Cloudflare → Envoy 的鏈路若錯信任 `X-Forwarded-Host`，可能把請求解析到錯誤 community。

必驗：

- 只有可信 edge 能覆寫 forwarded headers。
- auth challenge、membership 與 event query 都綁定同一 tenant。
- 以另一個 community host 的 credential 做 cross-tenant negative tests。

#### P1 — Media、logs 與 evidence

確認 production media GET/WRITE auth，不採用不安全的 dev default；object keys、signed URLs、logs 與 evidence export 都必須保留 tenant boundary。完整 Telegram charge ID、payment signature、private event 或使用者身份不可進公開 logs。

#### P2 — Fingerprinting 與維護

NIP-11 合理地公開 software version、supported NIPs 與 relay pubkey，但也方便攻擊者 fingerprint。需有規律 patching、rate-limit/WAF、dependency scan 與 incident-response owner。

## 6. 還不知道的事：交給 OG operator 回答

| OPEN | 需要的證據 | Owner |
|---|---|---|
| 實際 cloud provider、region、replicas | deployment inventory／ArgoCD app | Buzz operator |
| Postgres／Redis／object storage encryption、backup、restore test | infra config + 最近 restore report | Buzz operator |
| Cloudflare WAF／rate rules | zone rules export／dashboard evidence | Buzz operator |
| Trusted proxy 與 host normalization | Envoy/Istio config + negative test | Platform engineer |
| Secrets rotation | secret manager policy + last rotation | Security／operator |
| Log retention 與 PII redaction | logging policy + sample redacted event | Security／operator |
| Media cross-tenant isolation | integration test evidence | Reviewer |
| Agent sandbox／tool policy | ACP production config | Agent operator |

在這些證據補齊前，可以說「公開安全表面合理」，不能說「production 已完成安全驗證」。

## 7. Demo 的推薦連線方式

- **要展示真實社群體驗：** Buzz Desktop 連 `wss://og.communities.buzz.xyz`；不需要為了 UI 啟動本機 relay。
- **要重現／debug relay：** 使用 `ws://localhost:3000`，但把畫面標成 local development，不要與 OG production 混稱。
- **要拍 canonical payment：** Telegram 用第二個非白名單帳號；Buzz UI 可以使用 hosted OG，只要 bot identity、membership 與 channel 已驗證。
- **要降低 Mac 負擔：** 只留一個 active coding engine、`BUZZ_ACP_AGENTS=1`，並停掉與 demo 無關的 Docker/Node sessions。

## 8. 文件維護規則

- 效能數據附時間與測量指令，不把單次快照寫成固定成本。
- 公開端點觀察與內部部署事實分欄，不把推論升格成事實。
- 每次 production config 改動後重跑 auth、cross-tenant、rate-limit、media 與 proxy-header tests。
- 每次 demo 後更新 lessons learned，但不修改原始 evidence 或 reviewer verdict。
