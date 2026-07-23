# Buzz Fuel Demo、教學與開發復盤

**用途：** 在 canonical live run 通過後，直接用本文件製作 demo 影片、社群貼文與技術教學。  
**目前狀態：** 教學骨架已鎖；最終數字、連結與 PASS 結果必須由獨立 reviewer 的 evidence bundle 填入。  
**原則：** 先證明一條真實、可稽核的核心路徑；不要把 POC 包裝成完整 Agent Fuel Runtime。

## 1. 教學的一句話

> A human pays a Telegram Stars invoice; MakeReel uses its existing platform payer to settle one x402 video job; signed progress and the final result return to the Buzz room.

可以使用的短句：

> Stars in. x402 out. Proof back to the room.

只有在 evidence bundle 證明真實 x402 settlement 時，才可以使用第二句。

## 2. Demo 前的硬門檻

開始拍攝前，operator 與 reviewer 必須逐項確認：

- 使用非白名單 Telegram 帳號；invoice 顯示 canonical `75 Stars`。
- 本次 run 有新的 `funding_intent_id`，不可沿用 1-Star whitelist run。
- 平台 signer 有足夠 USDC／gas；operator 已明確批准這一次真實支出。
- Buzz community、channel、bot identity 與 Telegram bot 都已 smoke test。
- evidence 會保留 x402 response／transaction hash、完整簽名 Buzz events，以及安全遮罩後的 Telegram charge correlation。
- feature flag 的開啟人、關閉人與關閉條件已指定。

任何一項不成立，就先做 no-payment rehearsal，不要開始真實付款。

## 3. 建議影片腳本（60–90 秒）

### Shot 1 — 問題與場景（5–8 秒）

畫面：Buzz room 與 MakeReel Fuel Bot。

旁白：

> 在 Buzz 裡，一個人可以用 Telegram Stars fuel 一次具體任務，不用自己準備信用卡、API key 或 x402 wallet。

### Shot 2 — 建立 funding intent（8–12 秒）

畫面：

1. 在指定 Buzz channel 輸入 `/fuel`，或點固定 mission button。
2. Bot 回覆 funding card 與 `t.me/MakeReel_xyz_bot?start=fuel_<opaque-token>`。
3. 清楚拍到 mission／SKU；不要露出 secret 或完整 opaque token。

旁白：

> Buzz 只建立一個有限用途的 funding intent；它不是在聊天室裡理解任意指令。

### Shot 3 — Canonical Stars 付款（10–15 秒）

畫面：

1. 用第二個、非白名單 Telegram 帳號開啟 deep link。
2. invoice 明確顯示 `75 Stars`。
3. 使用者確認付款。

必須一鏡保留 invoice 金額與付款確認。不得用舊的 1-Star whitelist 畫面代替。

### Shot 4 — Fueled 與 x402 執行（8–12 秒）

畫面：

1. Buzz 出現帶簽名的 `Fueled`／`Running` 更新。
2. 顯示 job ID 的安全縮寫。
3. 生成等待可剪接，但不可剪掉 Stars 付款與 `Fueled` 的因果順序。

旁白：

> 成功付款只解鎖這個 intent；既有 MakeReel platform payer 再為固定 SKU 完成 x402 settlement。

### Shot 5 — 雙邊交付（10–15 秒）

畫面：

1. Telegram 收到結果 URL。
2. 同一結果回到 Buzz，狀態為 `Delivered`。
3. 快速展示 Base transaction receipt 或遮罩後的 evidence summary。

### Shot 6 — 誠實收尾（5–8 秒）

旁白：

> 這是一個 reference experiment：human Stars → platform-settled x402 video → signed receipt in Buzz。它驗證支付與交付 plumbing，不代表已驗證市場需求或去中心化 runtime。

## 4. Demo 排練順序

1. **No-payment rehearsal（10–15 分鐘）**  
   檢查畫面、手機、deep link、channel 與錄影收音；停止在付款確認前。
2. **Evidence rehearsal（5 分鐘）**  
   確認每一種 evidence 的保存位置與遮罩規則。
3. **One canonical live run（約 5–15 分鐘）**  
   只允許一個 operator 操作付款與 feature flag。
4. **Independent review（15–30 分鐘）**  
   reviewer 依 `03-TEST-ACCEPTANCE.md` 判定，不接受口頭補證。
5. **剪輯與發布（30–60 分鐘）**  
   先剪故事，再附 technical appendix；不可修改 run 的金額或授權歷史。

## 5. 最小 evidence bundle

| 關聯 | 最低證據 |
|---|---|
| Telegram payer → intent | `funding_intent_id`、canonical Stars amount、遮罩後 charge suffix/hash、原始 webhook 的受控留存位置 |
| intent → x402 job | job ID、固定 SKU、payer address、amount |
| x402 settlement | transaction hash 或遮罩後 payment response、network、public receipt |
| job → Buzz | 完整 Nostr event `id`、`pubkey`、`created_at`、`kind`、`tags`、`content`、`sig` |
| job → delivery | Telegram 與 Buzz 的同一 result URL／artifact digest |
| safety closure | feature flag 恢復為 false 的證據 |

公開版 bundle 只能包含安全遮罩後資料；原始 webhook、完整 charge ID、token、private key 與內部 credential 不可放入 repo 或影片。

## 6. 這次開發值得教的五個問題

### 6.1 白名單價格不是 canonical pricing

1-Star run 證明 live payment plumbing 可運作，但不能證明 75-Star pricing path。兩者必須分開記錄；不可事後重寫授權。

### 6.2 成功畫面不是可驗收證據

一個 `Delivered` 畫面不能證明 Stars、intent、x402 transaction 與 Buzz event 是同一條鏈。關聯欄位與原始 evidence 必須在執行當下保留。

### 6.3 Nostr event 必須保存簽名

只匯出訊息文字或 event ID 不足以獨立驗證。正式 evidence 必須保留完整 `sig` 與可重算 event ID 的所有欄位。

### 6.4 Payment webhook 要兼顧稽核與隱私

至少保存一個安全 charge suffix 或穩定 hash，並在受控環境保留原始 webhook。公開 evidence 不應暴露完整 Telegram charge ID 或使用者身份。

### 6.5 Reviewer 不是第二位 implementer

Implementer 負責交付 evidence bundle 與 handoff；reviewer 只依既定標準判定 `PASS`、`PARTIAL` 或 `FAIL`。若 evidence 缺失，reviewer 不應自行補碼或重新解釋產品決策。

## 7. 發布時可說與不可說

### 可以說

- 「這是一次真實 Stars payment 與 x402 settlement 的 reference experiment。」前提是 canonical live run PASS。
- 「使用者不需要自行提供 API key 或 x402 wallet。」
- 「結果與可驗證的進度回到 Buzz room。」

### 不可以說

- 「已完成 decentralized agent runtime／ACP SDK。」
- 「已證明 PMF 或社群有付費需求。」
- 「Telegram Stars 直接在鏈上結算 x402。」實際上是平台 payer 代為 settlement。
- 用 1-Star whitelist run 冒充 canonical 75-Star pricing run。

## 8. 發布素材清單

- 60–90 秒主影片。
- 一張四步流程圖：Buzz intent → Telegram Stars → x402 job → Buzz/TG delivery。
- 一段不超過 120 字的誠實摘要。
- 遮罩後 evidence summary 與 Base explorer link。
- 已知限制：固定 SKU、單次支付、平台 payer、尚未驗證市場需求。
- 技術 appendix：架構、測試、失敗補償與 evidence schema。

## 9. 最終填空（reviewer PASS 後）

```text
Live run time:
Telegram account class: non-whitelist
Stars charged:
Funding intent:
x402 job:
Network:
Transaction:
Buzz event IDs:
Result:
Reviewer verdict:
Feature flag restored:
Evidence bundle:
```

