# 語音免打字工具

按熱鍵錄音 → Groq Whisper 轉文字 → Groq LLM 輕度校正 → 模擬鍵盤打進當前焦點視窗。
常駐 Windows 系統列（工作列右下角通知區域）。完整需求見 [`voicetyping-spec.md`](./voicetyping-spec.md)。

## 安裝（一般使用者，推薦）

不想自己編譯的話，直接下載安裝包即可：

1. 到 [Releases](https://github.com/dr-chenglung/voice-typing/releases/latest) 頁面。
2. 下載最新版的 `VoiceTyping_x.y.z_x64-setup.exe`（NSIS 安裝程式，`x.y.z` 為版本號）。
3. 執行安裝程式，一路下一步完成安裝（若系統缺少 WebView2 Runtime，安裝程式會自動下載補裝）。
4. 從開始選單啟動 **VoiceTyping**，程式會常駐系統列（工作列右下角）。
5. 首次使用請先在系統列圖示按右鍵 →「設定 Settings」填入 API key，見下方 [設定](#設定)。

> 首次執行時 Windows SmartScreen 可能因執行檔未經數位簽章而跳出警告，點「其他資訊 → 仍要執行」即可。

### 免安裝版（綠色版）

不想安裝、想直接執行的話，可改下載 `VoiceTyping_x.y.z_x64_portable.exe`：

1. 到 [Releases](https://github.com/dr-chenglung/voice-typing/releases/latest) 下載該檔，放到任一資料夾。
2. 直接雙擊執行即可，程式常駐系統列，不寫入登錄檔、不需安裝。
3. 首次使用一樣在系統列圖示按右鍵 →「設定 Settings」填入 API key。

> - **需系統已安裝 WebView2 Runtime**（Win10/11 多半已預裝；免安裝版不會自動補裝，缺少時可自行安裝 [Evergreen Runtime](https://developer.microsoft.com/microsoft-edge/webview2/)）。
> - 想做到「完全可攜」（設定跟著隨身碟走）：在 exe 旁邊放一份 `config.toml`，程式會優先讀取執行檔旁的設定，不必寫到 `%APPDATA%`。

## 從原始碼建置的環境需求

- Windows 10/11（含 WebView2 Runtime；Win10/11 多數已預裝，沒裝的話安裝程式會自動補裝）
- Rust（穩定版，MSVC toolchain）
- Tauri CLI：`cargo install tauri-cli`（一次性安裝；不需要 Node.js／npm）
- 一支 Groq API key（<https://console.groq.com>）

## 設定

API key 一律透過**設定視窗**填入：常駐系統列圖示按右鍵 → 「設定 Settings」，填入 API key 與其他選項後存檔，立即生效（熱鍵變更需重啟才生效）。不支援環境變數。

**權威存放位置是 `%APPDATA%\com.clhuang.voicetyping\config.toml`**，由設定視窗存檔時自動建立與寫入，不需要使用者自行準備範本檔；若該處沒有檔案，程式會向後相容讀取執行檔旁或目前工作目錄下的 `config.toml`。

STT（語音轉文字）與 LLM（校正）**各自獨立設定**，預設皆為 Groq，可改成其他 OpenAI 相容供應商：

| 欄位 | 說明 | 預設 |
|------|------|------|
| `stt_api_key` | STT 用的 API key | （空，需自行填入） |
| `stt_api_url` | STT 端點 URL | `https://api.groq.com/openai/v1/audio/transcriptions` |
| `stt_model` | 語音轉文字模型 | `whisper-large-v3-turbo` |
| `llm_api_key` | 校正用的 API key | （空，需自行填入） |
| `llm_api_url` | 校正端點 URL | `https://api.groq.com/openai/v1/chat/completions` |
| `llm_model` | 校正用模型 | `llama-3.1-8b-instant` |
| `enable_correction` | 是否做 LLM 校正 | `true` |
| `hotkey` | 觸發鍵，可改 `right_ctrl` 等 | `right_alt` |

## 建置與執行

```powershell
cargo install tauri-cli   # 一次性安裝
cd src-tauri
cargo tauri dev            # 開發模式
cargo tauri build          # 產出 release 安裝包（NSIS）與獨立 exe
```

## 使用方式

1. 啟動後常駐系統列（黑色麥克風圖示＝閒置）。
2. 把游標點進任一輸入框（記事本、瀏覽器…）。
3. 按 **右 Alt** → 螢幕底部中央浮現**麥克風圖示**，隨你說話的音量發光/微微放大 → 說一句話 → 再按 **右 Alt** 停止。
4. 圖示消失、系統列圖示變黃（處理中），數秒後校正完的文字自動打進游標處。
5. 系統列圖示按右鍵可開啟：
   - **設定 Settings**：調整 API key、模型、是否校正、熱鍵（語音語言一律自動偵測，無需設定）。
   - **歷史紀錄 History**：瀏覽過去轉錄出的文字（持久化存於磁碟），可一鍵清除。
   - **結束 Quit**：離開程式。

> 底部麥克風圖示不奪取輸入焦點、可點擊穿透，不會影響你正在輸入的視窗。

| 系統列圖示顏色 | 狀態 |
|----------|------|
| 黑 | 閒置 Idle |
| 紅 | 錄音中 Recording |
| 黃 | 處理中 Processing |
| 橘（短暫） | 發生錯誤（同時會跳出 Windows 系統通知說明原因） |

## 模組結構

後端（`src-tauri/src/`）：

| 檔案 | 職責 |
|------|------|
| `main.rs` | 進入點、執行緒接線、Tauri 事件迴圈 |
| `config.rs` | 設定檔讀取／儲存（`app_config_dir()` 為權威位置，向後相容舊位置） |
| `commands.rs` | 設定／歷史紀錄視窗用的 IPC commands |
| `hotkey.rs` | rdev 熱鍵監聽（toggle 偵測） |
| `audio.rs` | cpal 錄音 + 降取樣 + hound 封裝 WAV |
| `state.rs` | AppState 狀態定義 |
| `transcribe.rs` | Groq STT 與 LLM 校正 |
| `controller.rs` | 狀態機協調、管線、容錯降級 |
| `typer.rs` | 剪貼簿貼上輸出（Ctrl+V，繞過中文輸入法組字；失敗時退回 enigo 打字） |
| `tray.rs` | 系統列圖示、狀態顯示、選單（設定／歷史紀錄／結束） |
| `overlay.rs` | 麥克風圖示疊加視窗的生命週期管理（顯示／隱藏／定位／NOACTIVATE） |
| `history.rs` | 轉錄文字歷史紀錄讀寫（`app_data_dir()/history.json`） |
| `notify.rs` | 背景執行緒失敗時發 Windows 系統通知 |

前端（純靜態 HTML/CSS/JS，無建置步驟，`ui/`）：

| 目錄 | 用途 |
|------|------|
| `ui/overlay/` | 麥克風圖示視窗：canvas 繪製黑色麥克風剪影，隨音量發光/縮放 |
| `ui/settings/` | 設定視窗表單 |
| `ui/history/` | 歷史紀錄清單與清除按鈕 |

## 已知限制（MVP）

- 僅 Windows。
- 校正失敗時會降級為輸出原始辨識文字（不中止）。
- 單次錄音上限約 12 分鐘（25MB），超過會報錯。
- 右 Alt 在部分歐語系鍵盤等同 AltGr；受影響者請改用 `hotkey = "right_ctrl"`。
- 輸出採剪貼簿貼上：輸出當下會短暫佔用剪貼簿（事後自動還原文字內容）；若原本剪貼簿是圖片等非文字內容，還原會略過。
- 熱鍵變更存檔後需重啟程式才會生效（`rdev::listen` 沒有乾淨的取消機制）。
- 歷史紀錄只保存**轉錄出的文字**（上限 500 筆，超過時 FIFO 淘汰最舊），錄音音訊本身仍是用完即丟、只存在記憶體，不落地。
