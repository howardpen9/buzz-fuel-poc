# Buzz 上做 Bot：文檔彙整 + MakeReel Fuel Bot 案例（講稿／PDF 素材）

**用途：** 拍講稿影片、出 PDF 時的單一入口。  
**狀態：** 技術 reference experiment 已完成（C1-live + C2 PASS；75★ pricing NOT VALIDATED）。  
**對應 live channel：** `72ef2918-e8db-44de-9e28-10f504c44ac9`  
**對應 live relay：** `ws://localhost:3000`（本機自架 relay，不是 OG hosted）

---

## 0. 一句話先對齊

| 問題 | 答案 |
|---|---|
| 我們那次 demo 是 localhost 嗎？ | **是。** Relay = `ws://localhost:3000`；channel UUID 就是上面那串。 |
| Buzz 上 bot 一定要 LLM 嗎？ | **不必。** 官方 `countdown-bot` 就是純演算法 bot。 |
| 我們做的是哪一類？ | **Thin signed bot**（WebSocket + NIP-42 + kind 9），外掛 MakeReel 支付／x402，不是 ACP 智能體。 |
| 可以對外怎麼說？ | 「Stars → platform-settled x402 → 簽名回執回 Buzz」的 **reference experiment**。不可說已驗證 75★ 定價或 PMF。 |

---

## 1. Buzz 官方：三種「在房間裡做事的程式」

講稿建議先畫這張分層，再講我們落在哪一層。

```text
┌─────────────────────────────────────────────────────────────┐
│  A. Thin bot（非 AI）                                        │
│     自己握 nsec → NIP-42 AUTH → 訂閱 channel → 簽 kind 9 回覆 │
│     範例：examples/countdown-bot                             │
│     我們的 MakeReel Fuel Bot 落在這一層 + 外部 HTTP            │
├─────────────────────────────────────────────────────────────┤
│  B. Managed AI agent（ACP harness）                           │
│     buzz-acp 聽 @mention → stdio ACP → Goose/Codex/Claude    │
│     回覆靠 buzz-cli；identity 仍是自己的 Nostr key            │
├─────────────────────────────────────────────────────────────┤
│  C. buzz-agent（最小 ACP agent 本體）                         │
│     只做 LLM loop + MCP tools；不知道 Buzz；由 harness 接上   │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
                   buzz-relay（真相源）
              NIP-42 · kind 0 profile · kind 9 messages
              kind 9000 channel membership · audit log
```

**講一句：**  
> Buzz 裡「bot／agent」不是另一套 API 帳號，而是 **一把 Nostr 金鑰 + 簽名事件**。人、agent、thin bot 共用同一套成員與訊息模型。

---

## 2. 官方文檔地圖（給 PDF 附錄／講者備忘）

### 2.1 必讀（做 thin bot）

| 文件 | 路徑 | 講什麼 |
|---|---|---|
| Countdown Bot README | `buzz/examples/countdown-bot/README.md` | 非 AI bot 最小契約；standalone vs owner-attested 兩種 AUTH |
| Examples 總覽 | `buzz/examples/README.md` | bot 與 persona-pack 入口 |
| 原始實作 | `buzz/examples/countdown-bot/src/main.rs` | NIP-42、kind 0、kind 9000、kind 9 全流程可讀 |

**Thin bot 生命週期（講 30 秒）：**

1. 連 `BUZZ_RELAY_URL`（本機預設 `ws://localhost:3000`）
2. NIP-42 challenge → 用 bot key 簽 AUTH（可選 owner-attested `auth` tag）
3. 發 kind `0` profile（名稱／圖示／about）
4. best-effort kind `9000` self-add，`role=bot`（成員列表與 @ 自動完成）
5. `REQ` 訂閱該 channel 的 kind `9`
6. 收到指令 → 忽略自己 → 簽 kind `9` 回覆 → 等 `OK`

**兩種身份路徑：**

| 模式 | 用途 | 本機 demo 常用 |
|---|---|---|
| `standalone` | bot key 本身被 relay 允許／是 member | **我們用這個** |
| `owner-attested` | bot 自簽訊息，但 AUTH 帶 owner 的 NIP-OA 背書 | closed relay 備援 |

### 2.2 必讀（做 AI agent）

| 文件 | 路徑 | 講什麼 |
|---|---|---|
| Vision：agent 哲學 | `buzz/VISION_AGENT.md` | 為何拆 buzz-agent + buzz-dev-mcp；可審計、可並行 |
| buzz-acp harness | `buzz/crates/buzz-acp/README.md` | @mention → agent → buzz-cli 回訊息；key mint |
| buzz-agent | `buzz/crates/buzz-agent/README.md` | ACP stdio loop、MCP tools、provider env |
| MCP hooks | `buzz/docs/MCP_DRIVEN_HOOKS.md` | agent 生命週期 hook（進階） |
| 產品總覽 | `buzz/README.md` | humans + agents 同一房間、同一 audit trail |

**AI agent 最小啟動（講稿可放 code slide）：**

```bash
# mint identity
cargo run -p buzz-admin -- mint-token --name "my-agent" \
  --scopes "messages:read,messages:write,channels:read"

# harness（goose 例）
export BUZZ_PRIVATE_KEY="nsec1..."
export BUZZ_RELAY_URL="ws://localhost:3000"
export GOOSE_MODE=auto
buzz-acp
```

### 2.3 架構／本機開發

| 文件 | 路徑 |
|---|---|
| 架構全景 | `buzz/ARCHITECTURE.md` |
| 本專案 ASCII 接線 | `research/buzz-architecture-ascii.md` |
| 本機 relay | `just setup` → `just relay`（見 countdown-bot README） |
| Hosted vs local 執行邊界 | `buzz-fuel-poc/docs/12-HOSTED-OG-AGENT-ACCESS-MODEL.md` |

**Hosted 陷阱（OG 社群講 10 秒）：**  
`og.communities.buzz.xyz` 只 host **community／relay**；Desktop 上的 managed agent 多半是 **local backend**——Mac 離線／睡眠就沒人回。Hosted community ≠ hosted agent runtime。

### 2.4 協議關鍵字（PDF 術語表）

| 詞 | 意義 |
|---|---|
| Nostr key / nsec / npub | bot／agent 身分；每則事件用私鑰簽 |
| NIP-42 | WebSocket AUTH（challenge-response） |
| NIP-OA | owner attestation：owner 背書 agent／bot 入場 |
| kind 0 | profile metadata |
| kind 9 | channel message（我們 fuel 狀態回報都是這個） |
| kind 9000 | channel membership（`role=bot`） |
| ACP | Agent Client Protocol（JSON-RPC 2.0 over stdio） |
| MCP | Model Context Protocol（tools） |
| buzz-cli | agent-first CLI，JSON in／out |

---

## 3. 案例記錄：MakeReel Fuel Bot（我們做好的那個）

### 3.1 定位（對聽眾講清楚）

| 是 | 不是 |
|---|---|
| Thin signed Buzz bot | ACP／LLM agent |
| 固定指令 `/fuel` 一次任務 | 自由對話 mission board |
| Telegram Stars 收款 + 既有 MakeReel platform payer 代結 x402 | 使用者自備 wallet／API key |
| 本機 relay 上的 reference experiment | 公開 ship 的產品 |
| 簽名 kind 9 狀態回寫 | 支付真相源（真相在 makereel-core） |

**產品契約（投影片大字）：**

```text
Buzz /fuel
  → t.me/MakeReel_xyz_bot?start=fuel_<opaque-token>
  → Telegram Stars invoice
  → MakeReel platform payer 結算 x402
  → 同一結果送到 Telegram + Buzz（簽過名的進度）
```

短句（僅在真實 x402 evidence 存在時用）：

> Stars in. x402 out. Proof back to the room.

### 3.2 程式落點

| 層 | Repo / 路徑 | 職責 |
|---|---|---|
| Buzz adapter | `buzz-fuel-poc/`（本資料夾） | 聽 `/fuel`、建 intent、poll core、發 kind 9 |
| 衍生來源 | `buzz/examples/countdown-bot` | AUTH／profile／membership／subscribe／publish |
| 編排與支付 | `makereel-core`（`poc/buzz-fuel-core`） | intent、quote、ledger、x402 payer、terminal status |
| Telegram 入口 | `makereel-tg-miniapp`（`poc/buzz-fuel-bot`） | `fuel_*` deep link、invoice、付款 webhook、TG 交付 |

**Adapter 本體：**

- `src/main.rs` — WS loop、NIP-42、profile、membership、`/fuel` 處理、poller
- `src/model.rs` — intent phase、transition keys（`fueled` → `delivered`）、訊息文案
- `src/makereel.rs` — 對 core 的 internal HTTP

**指令契約（刻意極窄）：**

- 只認 **exact** `/fuel`（不是自然語言）
- 建立 funding intent → 貼 Telegram deep link
- 之後只 **鏡像** core 的狀態，不自己發明 paid／delivered

### 3.3 本機 live run 指紋（講「我們真的跑過」）

| 欄位 | 值 |
|---|---|
| Relay | `ws://localhost:3000` |
| Channel | `72ef2918-e8db-44de-9e28-10f504c44ac9` |
| Label | C1-LIVE · C2 · REAL STARS（1★ whitelist）· REAL x402 · SINGLE RUN |
| Intent | `fi_w33B7aFXJhihSwAW` |
| Job | `58809074-aef4-4cfb-b0dc-8e46a3110795` |
| Share | https://makereel.xyz/s/<redacted> |
| x402 | $0.75 USDC on Base |
| Tx | `0x1e8fde97801232d6ea874f04277776afecb1bb018e33d5f143baa9df5071c7c0` |
| Explorer | https://basescan.org/tx/0x1e8fde97801232d6ea874f04277776afecb1bb018e33d5f143baa9df5071c7c0 |
| `/fuel` event | `4b6033db9cb43e89ef67dbc01ac443894b962e0229697a83ab38343351d24a1f` |
| Link reply | `91feeeb77345c53ea8eac80b53da6b29d62ecf37902e3c77725d58724ec5a976` |
| Fueled | `01e84ad8910e4891aeb4cb3586d4bc6314db2307ba6528daee38bbc79bc493fd` |
| Delivered | `f69016388f932e208ffbf49eb877defca9edb2db6478932d9eef40c51a45445b` |
| Evidence | `evidence/20260722T112900Z-C1LIVE-C2/` + `-RECOVERY/` |

**誠實標籤（講稿必講，不可省略）：**

| 軸 | 結果 |
|---|---|
| C1-live plumbing | PASS |
| C2 real x402 → delivered | PASS |
| Canonical 75★ pricing | **FAIL / NOT VALIDATED**（實際 1★ whitelist） |
| Overall product acceptance | **PARTIAL** |
| Public ship | **NOT**（flag 已關） |

### 3.4 狀態機（一張圖講完 bot 行為）

```text
User in Buzz channel
        │
        │  exact "/fuel"
        ▼
  Fuel Bot (kind 9 listen)
        │  POST create intent (makereel-core)
        ▼
  kind 9: payment link
  t.me/MakeReel_xyz_bot?start=fuel_<token>
        │
        │  user pays Stars on Telegram
        ▼
  makereel-core: paid → running → delivered|failed
        │  bot polls every ~5s
        ▼
  kind 9: Fueled → (optional Running) → Delivered
        + Telegram also gets the video / share URL
```

**Transition keys（`model.rs`）：**

| Core status | Buzz 最多發一次 |
|---|---|
| `paid` | `fueled` |
| `running` | `running`（可關） |
| `delivered` | 若還沒發過 fueled → 先 `fueled` 再 `delivered` |
| `failed` / `refunded` / `expired` | `fueled`（若需要）+ `failed` |

### 3.5 本機怎麼再跑一次 thin bot（操作備忘）

```bash
# 1) Buzz relay（buzz repo）
. ./bin/activate-hermit
just setup
just relay

# 2) Desktop 連 localhost，建／選 channel，複製 UUID
#    我們那次：72ef2918-e8db-44de-9e28-10f504c44ac9

# 3) Fuel bot
cd buzz-fuel-poc
export BUZZ_RELAY_URL=ws://localhost:3000
export BUZZ_CHANNEL_ID=<channel-uuid>
export BUZZ_BOT_PRIVATE_KEY=<nsec>
export BUZZ_BOT_AUTH_MODE=standalone
export MAKEREEL_API_URL=...
export MAKEREEL_INTERNAL_API_KEY=...
# 並確認 makereel-core / TG bot 已起、BUZZ_FUEL_ENABLED 依 runbook

cargo run --release
```

官方對照範例（不連 MakeReel）：

```bash
BUZZ_RELAY_URL=ws://localhost:3000 \
BUZZ_CHANNEL_ID=<channel-uuid> \
BUZZ_BOT_PRIVATE_KEY=<nsec> \
BUZZ_BOT_AUTH_MODE=standalone \
cargo run --manifest-path buzz/examples/countdown-bot/Cargo.toml
```

---

## 4. 講稿骨架（可直接剪進影片／PDF 章節）

完整 60–90 秒鏡頭腳本見：`10-DEMO-TUTORIAL-AND-LESSONS.md`。  
這裡給「技術分享版」約 5–8 分鐘大綱。

| 分鐘 | 章節 | 講點 | 素材 |
|---:|---|---|---|
| 0:00 | 問題 | 人在 agent 房間裡 fuel 一個具體任務；不要自己管 wallet／API key | room 截圖 |
| 0:40 | Buzz bot 分層 | thin bot vs ACP agent；「bot = key + signed events」 | §1 圖 |
| 1:30 | 官方最小例 | countdown-bot：NIP-42 → kind 0 → 9000 → 9 | 官方 README |
| 2:20 | 我們的 bot | 從 countdown 衍生；只做 `/fuel` + 狀態鏡像 | architecture |
| 3:00 | 支付邊界 | TG Stars 入、platform payer 出 x402；Buzz 不做 merchant | 流程圖 |
| 4:00 | Live 證據 | localhost channel、event id、Base tx、share URL | §3.3 表 |
| 5:00 | 誠實邊界 | 1★ whitelist ≠ 75★；PARTIAL；非 PMF | 紅字 slide |
| 5:45 | 可帶走 | 開源路徑 + 三層選擇：thin / acp / full agent | 文檔地圖 |
| 6:30 | Q&A | evidence bundle、rollback flag、為何不 fork Desktop | runbook |

**可說／不可說** 完整表：見 `10-DEMO-TUTORIAL-AND-LESSONS.md` §7。

---

## 5. PDF 組卷建議（之後用 make-pdf）

建議章節順序（約 8–12 頁）：

1. **封面** — Stars in · x402 out · Proof back to the room（副標：reference experiment）
2. **Buzz 上三種 bot／agent** — §1 圖
3. **Thin bot 協議最小集** — NIP-42 / kind 0 / 9 / 9000
4. **案例：MakeReel Fuel Bot** — 產品契約 + 架構
5. **端到端時序** — §3.4 狀態機
6. **Live run 證據表** — §3.3（遮罩後）
7. **誠實限制** — 1★ vs 75★、PARTIAL、non-goals
8. **附錄 A** — 官方文檔路徑索引（§2）
9. **附錄 B** — 本專案決策／驗收文件索引（§6）
10. **附錄 C** — event IDs / explorer links（可抽成一頁 QR）

已有視覺素材可併入：

- `docs/thread-buzz-jack/` — 四張論述圖（agent teammates、parallel lab、builder usage、two doors）
- `docs/10-QR-CODE-DEMO-PLAN.md` — QR／deep link 畫面計畫
- `evidence/...` — 僅用公開遮罩版；勿塞 raw webhook / charge id / nsec

---

## 6. 本 repo 相關文件索引（一頁目錄）

### Fuel POC（我們的 bot）

| 檔 | 用途 |
|---|---|
| `README.md` | 產品契約、閱讀順序、本地指令 |
| `IMPLEMENTER-HANDOFF.md` | 凍結 SHA、live 指紋、PARTIAL 結論 |
| `docs/00-DECISIONS.md` | LOCK／OPEN／PARK／REJECT |
| `docs/01-SOURCE-MAP.md` | 從 countdown-bot 抄什麼、不抄什麼 |
| `docs/02-IMPLEMENTATION-PLAN.md` | 跨 repo change set |
| `docs/03-TEST-ACCEPTANCE.md` | 驗收矩陣 |
| `docs/04-RELEASE-RUNBOOK.md` | 發布／rollback／錄影 |
| `docs/10-DEMO-TUTORIAL-AND-LESSONS.md` | **影片腳本 + 可說不可說** |
| `docs/10-QR-CODE-DEMO-PLAN.md` | QR demo 畫面 |
| `docs/11-COMMUNITY-SECURITY-AND-LOCAL-RESOURCES.md` | 安全與本機資源 |
| `docs/12-HOSTED-OG-AGENT-ACCESS-MODEL.md` | hosted community vs local agent |
| `docs/13-BUZZ-BOT-DOCS-TALK-PACK.md` | **本文件：講稿／PDF 總包** |
| `LIVE-RUN-CHECKLIST.md` | 操作 checklist |
| `evidence/` | 可稽核證據包 |

### Research（策略語境，非 bot API）

| 檔 | 用途 |
|---|---|
| `research/00-INDEX.md` | 研究總索引 |
| `research/buzz-architecture-ascii.md` | 接線圖 |
| `research/buzz-tg-x402-plan.zh-TW.md` | P0 計畫語境 |
| `research/buzz-verdict-honest.zh-TW.md` | 商業判決（TG-first） |

### Upstream Buzz（官方）

| 檔 | 用途 |
|---|---|
| `buzz/examples/countdown-bot/README.md` | thin bot 聖經 |
| `buzz/crates/buzz-acp/README.md` | AI harness |
| `buzz/crates/buzz-agent/README.md` | ACP agent |
| `buzz/VISION_AGENT.md` | agent 願景 |
| `buzz/README.md` / `ARCHITECTURE.md` | 產品與架構 |

---

## 7. 與 buzz://message 連結的對應

使用者分享的 deep link 形如：

```text
buzz://message?channel=72ef2918-e8db-44de-9e28-10f504c44ac9&id=<event-id>
```

| 參數 | 意義 | 我們那次 |
|---|---|---|
| `channel` | channel UUID（NIP-29 群組／頻道） | `72ef2918-e8db-44de-9e28-10f504c44ac9` |
| `id` | 該則 Nostr event id（64 hex） | 見 §3.3 各階段 event |

在 **本機 Desktop 連 `ws://localhost:3000`** 時，此 link 指向本機 community 裡那則簽名訊息；不是 OG hosted 上的同一則（event 在哪個 relay 落地就屬於哪個 community）。

---

## 8. 下一步（出 PDF 時）

1. 確認講稿要 **中文技術分享** 還是 **英文 demo reel**（本檔已雙語可用）
2. 從 §5 挑 8–10 頁，補 1 張四步流程圖
3. 跑 `make-pdf`（或 gstack make-pdf skill）對本檔或精簡版
4. 影片主軸仍以 `10-DEMO-TUTORIAL-AND-LESSONS.md` 為準；本檔當講者備註 + PDF 正文

**本檔完成定義：** 講者只開這一份，就能找到官方 bot 文檔、我們 bot 的架構與 live 指紋、以及 PDF／影片章節順序——不必再掃整個 monorepo。
