[English](README.md) · [繁體中文](README.zh-TW.md)

# Clip-Claude (即貼即克)

把截圖直接 Ctrl+V 進終端機 AI agent —— Claude Code、Gemini CLI、Codex —— 就像它們原生支援圖片貼上一樣。**而且**同一張截圖,還是能照常貼進 Photoshop、Telegram、Discord、任何其他地方,完全不受影響。

姊妹專案:[Kaikou-Claude(開口即克)](https://github.com/GeniusPudding/Kaikou-Claude)(語音 → agent)與 [Listen-Claude(聽聲即克)](https://github.com/GeniusPudding/Listen-Claude)(agent → 語音)。三個外掛、同一個工作流:把終端機 agent 當聊天 app 用。

```
                              ┌───────────────────────────────────────────┐
                              │  Clip-Claude 在剪貼簿上「補一格」:           │
[ 剪貼簿裡的截圖 ]            ─►│    原始圖片 (CF_DIB) 保留不動            │
                              │    再多寫一份文字路徑 (CF_UNICODE)        │
                              └───────────────────────────────────────────┘
                                              │
                  ┌───────────────────────────┴───────────────────────────┐
                  ▼                                                       ▼
       Ctrl+V 進 Photoshop /                                  Ctrl+V 進 agent 終端機 ——
       Telegram / Discord —— 跑出來                          路徑文字進去,agent 用
       還是原本的圖,像從沒被動過。                            內建讀檔工具打開,多模態
                                                              模型看見這張圖。
```

純 Rust 原生小執行檔——沒有 runtime、沒有 Python、沒有 Electron。Windows 提供完整安裝路徑與無視窗背景常駐;macOS / Linux 可以編譯但沒有自動啟動(Claude Code 在 macOS 原生支援 Cmd+V 貼圖)。

## 為什麼需要這個

終端機 AI agent CLI 普遍不接受二進位剪貼簿貼上——Windows 上多半直接失敗;macOS 上 Claude Code 自己接好,但 Gemini / Codex 表現不一(Gemini 要 Alt+V 而非 Ctrl+V,且有已知的 regression)。Clip-Claude 不需要改任何 agent CLI,用最自然的快捷鍵—— **Ctrl+V**——直接補上這個缺口。

## Agent 相容性

| Agent | 狀態 | 備註 |
|---|---|---|
| **Claude Code** | ✅ 已驗證 | TUI 會自動把 4 行 payload 摺成 `[Pasted text #1, +4 lines]`,送出時 Read tool 打開檔案。 |
| **Gemini CLI** | ⚠️ 設計上支援,尚未端到端實測 | Gemini 是多模態,LLM 應該會照著 payload 內的指示讀路徑。4 行文字在 TUI 上不會被摺起來,會直接展開顯示。 |
| **Codex (OpenAI)** | ⚠️ 設計上支援,尚未端到端實測 | 與 Gemini 同——多模態模型 + 讀檔工具,應該能正確處理路徑。 |
| **任何其他多模態 + 有讀檔工具的 agent** | 應該都可以 | 機制與 agent 無關:剪貼簿放一段文字 payload 指向檔案路徑而已。 |

歡迎在 Issues 回報實際的 Gemini / Codex 使用結果。

## 不會破壞既有貼圖

Clip-Claude **不會把你原本的圖弄不見**。處理完之後,剪貼簿同時帶有兩種格式:

- `CF_DIB` —— 原始圖片,bytes 對 bytes 等同於 Snipping Tool / .NET `Clipboard.SetImage` 的輸出(32-bit BI_BITFIELDS,bottom-up)。系統會自動從這個合成 `CF_BITMAP` 與 `CF_DIBV5`。
- `CF_UNICODETEXT` —— 一段自說明用的 4 行文字 payload,指向已存檔的 PNG。

吃圖的 app(Photoshop、Telegram、Discord、Word、瀏覽器輸入框)拿到圖;只吃文字的 app(Claude Code、Gemini CLI、Codex、Notepad、VS Code)拿到文字 payload。沒有任何 app 會看到「錯的那一份」。

## 安裝

每個平台都是一行指令(clone + 跑安裝腳本):

```bash
git clone https://github.com/GeniusPudding/Clip-Claude.git
cd Clip-Claude
.\install.ps1   # Windows
./install.sh    # macOS / Linux
```

冪等——可以隨時重跑來升級或修復。

### Windows

安裝腳本做的事:

1. 沒有 `rustup` 就裝(stable,minimal profile)。
2. `cargo build --release` 編譯 release binaries。
3. 把 `clip-claude.exe` + `clip-claude-bg.exe` 複製到 `%LOCALAPPDATA%\Clip-Claude\`。
4. 寫入 `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\Clip-Claude` 指向 `clip-claude-bg.exe`,登入時自動啟動無視窗背景程式。
5. 馬上 spawn daemon——不用 reboot、不用重開 shell。

驗證:

```powershell
& "$env:LOCALAPPDATA\Clip-Claude\clip-claude.exe" status
```

預期輸出:
```
  ok    installed at C:\Users\you\AppData\Local\Clip-Claude
  ok    auto-start registry entry present
  ok    daemon running
```

### macOS / Linux

`./install.sh` 會編出 `target/release/clip-claude`。**本版尚未支援登入自動啟動**——把 binary 放到 PATH,需要時手動跑 `clip-claude start`。macOS 上 Claude Code 已原生支援 Cmd+V 貼圖,所以這主要影響 Gemini / Codex 的使用情境。

多型剪貼簿寫入(image + text 並存)目前只在 Windows 完整實作。macOS / Linux 走 `arboard` 的 text-only 路徑——貼上時原圖會被路徑文字取代。Roadmap 項目:NSPasteboard / xclip 多型支援。

## 解除安裝

```bash
.\uninstall.ps1   # Windows —— 停 daemon、移除 Run key、保留 binaries
./uninstall.sh    # macOS / Linux —— 殺掉執行中的 watcher
```

Repo 檔案不會被動。要完整清掉的話,手動刪 `%LOCALAPPDATA%\Clip-Claude\`(或整個 clone 來的資料夾)。

## 子指令

| 指令                       | 說明                                                                     |
|----------------------------|--------------------------------------------------------------------------|
| `clip-claude install`      | 複製 binary 到 `%LOCALAPPDATA%\Clip-Claude\`、註冊自動啟動、立即啟動。     |
| `clip-claude uninstall`    | 停 daemon、移除自動啟動。Binary 不刪。                                   |
| `clip-claude status`       | 回報安裝狀態 + 自動啟動 + daemon 執行狀態。                              |
| `clip-claude`              | `clip-claude start` 的 alias。前景 watcher(會顯示 console)。           |
| `clip-claude start`        | 前景跑 watcher 直到 Ctrl+C。                                             |
| `clip-claude run -- CMD`   | 包住 `CMD`。Watcher 與 `CMD` 同生共死。                                  |
| `clip-claude doctor`       | 檢查剪貼簿存取與 cache 目錄。                                            |
| `clip-claude --version`    | 印版本。                                                                 |
| `clip-claude --help`       | 印 help。                                                                |

`clip-claude-bg.exe` 是同一份 watcher,但編譯時加 `windows_subsystem = "windows"`,所以不會開出 console 視窗。`install` 註冊到 Run key 的就是這個。

## 運作原理

1. 透過 `arboard` 每 150 ms 輪詢系統剪貼簿。
2. 剪貼簿如果已經有文字,就跳過——可能是使用者複製了文字,或者已經是我們處理過的圖。
3. 剪貼簿只有圖片、**沒有文字**(全新截圖)→ 讀取 RGBA buffer。
4. 存成 PNG 到 `~/.clip-claude/cache/clip_<timestamp>.png`。
5. 組出一份 byte 對 byte 等同 `.NET` 參考輸出的 CF_DIB(40-byte `BITMAPINFOHEADER` · `biCompression = BI_BITFIELDS` · 32-bit · 正值 `biHeight` 表 bottom-up · R/G/B color masks · BGRA pixel 反向 row)。
6. 組一份 4 行 UTF-16 文字 payload:
   ```
   [Clip-Claude] Pasted image (1920x1080)
   File: C:\Users\you\.clip-claude\cache\clip_20260515_143022_815.png
   Please open and analyze this file using your image-reading tool.
   (This text was auto-injected because the terminal cannot display images directly.)
   ```
7. 直接打 Win32:`OpenClipboard` → `EmptyClipboard` → `SetClipboardData(CF_DIB, ...)` → `SetClipboardData(CF_UNICODETEXT, ...)` → `CloseClipboard`。系統會自動合成 `CF_BITMAP` 與 `CF_DIBV5`。
8. 在 agent CLI 裡貼上 → 拿到文字(Claude Code 自動摺成 `[Pasted text #1, +4 lines]`),送出後 agent 的 Read tool 打開檔案路徑,多模態模型看見圖。
9. 在 Photoshop / Telegram / Discord / Word 貼上 → 拿到 CF_DIB 或合成出來的 CF_BITMAP——聊天 app 收這個,因為 bytes 跟 Snipping Tool 的輸出完全一樣。
10. 每次新截圖時順手清除 cache 中 7 天以上的舊檔。

### 與其他外掛共存

- **[Kaikou-Claude](https://github.com/GeniusPudding/Kaikou-Claude)(語音 → agent)**:語音 daemon 把語音轉成的文字寫入剪貼簿後送 Ctrl+V。Clip-Claude 看到「剪貼簿有文字」就靜止不動。先後順序不論,都能乾淨共存。
- **[Listen-Claude](https://github.com/GeniusPudding/Listen-Claude)(agent → 語音)**:跑在 Claude Code 的 `Stop` hook 裡,完全不碰剪貼簿——正交,毫無衝突。
- **通用剪貼簿管理工具**:Clip-Claude 寫的 CF_DIB 跟一般截圖一模一樣,所以剪貼簿歷史工具就把它當普通截圖記下來——沒有任何驚喜。

### 平台支援現況

- **Windows**:完整安裝路徑、byte-matched CF_DIB、無視窗背景常駐、HKCU Run key 自動啟動。
- **macOS / Linux**:可編譯成功;image+text 多型寫入與自動啟動尚未實作。目前的 fallback 是文字 only(原圖會被取代),這在 macOS 已經被 Claude Code 原生 Cmd+V 蓋掉,所以可接受。

## 架構

```
src/
├── lib.rs            # 模組宣告
├── main.rs           # `clip-claude.exe` 進入點 —— 分派子指令、doctor
├── bg.rs             # `clip-claude-bg.exe` 進入點 —— windows_subsystem = "windows"
├── cli.rs            # clap 參數定義
├── watcher.rs        # 150 ms 輪詢迴圈,有文字就跳過
├── clipboard_io.rs   # Win32 CF_DIB builder(byte-match .NET)+ 多型寫入
├── cache.rs          # 存 PNG 到 ~/.clip-claude/cache/,清 7 天以上的舊檔
├── inject.rs         # 組文字 payload
├── runner.rs         # `run -- CMD` 子行程包裝
└── install.rs        # install / uninstall / status(基於 HKCU Run key)
```

`clip-claude.exe` 與 `clip-claude-bg.exe` 透過 `clip_claude` lib 共用所有模組。

## Toolchain

見 [docs/toolchain.md](docs/toolchain.md)。

## 開發

```bash
./scripts/dev.sh -- start         # cargo run --bin clip-claude -- start
./scripts/dev.sh -- doctor
cargo test
cargo fmt && cargo clippy --all-targets -- -D warnings
```

## Roadmap

- [x] 多型剪貼簿 —— image + text 並存,各家 app 貼上都拿到對的格式。
- [x] Byte-match .NET CF_DIB —— 已驗證相容 LINE / Telegram / Discord / Photoshop / 瀏覽器貼上目標。
- [x] Windows 安裝指令 —— 一行安裝、無視窗背景常駐、登入自動啟動。
- [ ] 端到端實測 Gemini CLI 與 Codex 的貼上行為;記錄各 TUI 的怪癖(文字展開、讀檔工具命名)。
- [ ] `--agent <claude|gemini|codex>` 旗標 —— 為每個 agent 客製文字 payload(例:Gemini 用 `@path` 語法)。
- [ ] `clip-claude restore` —— 把最近一次 cache 的 PNG 重新以 image-only 形式寫回剪貼簿。
- [ ] macOS 安裝:NSPasteboard 多型寫入 + `launchctl` LaunchAgent。
- [ ] Linux 安裝:xclip / wl-clipboard 多型寫入 + systemd user service。

## License

MIT.
