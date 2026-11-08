# Latch · 午夜書房

以 Rust、Bevy 0.20.1 與 WebGPU 製作的第一人稱 3D 密室逃脫。單一原創房間，約五分鐘遊玩，完整流程包括閱讀便箋、端詳海邊站台照片、解開保險箱、讀完未寄出的信、取得鑰匙、打開房門與走出書房。支援重新開始與繁體中文、英文即時切換。

正式網站為 [liam652.com](https://liam652.com/)，原始碼為 [liam8846/latch](https://github.com/liam8846/latch)。

## 在瀏覽器遊玩

需要支援 WebGPU、DecompressionStream 且已啟用硬體加速的新版桌面 Chrome 或 Edge。開發時可使用 `localhost`；正式站點需要 HTTPS。本專案不提供 WebGL fallback。

安裝 Rust、WASM target、Node.js 與版本相符的匯出工具。

```powershell
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130 --locked
```

建立正式靜態檔案並啟動本機伺服器。

```powershell
./scripts/build-web.ps1
node scripts/serve.mjs
```

開啟 <http://127.0.0.1:4173>。`dist/` 是完整靜態交付目錄，可供 Cloudflare Pages 使用。建置腳本將 WASM 壓縮為 `pkg/latch_bg.wasm.gz`，確認符合單檔 25 MiB 限制後移除原始 WASM。瀏覽器透過 DecompressionStream 展開後交給 wasm-bindgen。靜態伺服器應將 `.gz` 檔案以 `application/gzip` 提供，且不設定 `Content-Encoding`，避免瀏覽器重複解壓縮。開發迭代可用 `./scripts/build-web.ps1 -Dev`。

## GitHub 自動部署

Cloudflare Pages 的 Git 專案 `latch-liam652-git` 連接 `liam8846/latch`，正式分支為 `master`。建置命令為 `bash scripts/build-pages.sh`，輸出目錄為 `dist`。每次推送至 `master` 都會由 Cloudflare 取得該 commit 並重新建置。

Linux 建置腳本使用固定 Rust `1.99.1` 與官方預編譯 `wasm-bindgen 0.2.130`，下載後檢查 SHA-256 與版本。它保留與本機 PowerShell 相同的 WASM gzip 交付方式。建置失敗或壓縮產物超過 25 MiB 時，腳本以錯誤結束。

正式網域只在 Git 建置與瀏覽器驗證成功後綁定此專案。Pages 單次建置有 20 分鐘上限，不能假設它會快取 Cargo 的相依套件與編譯產物。

## 操作

| 操作 | 按鍵 |
| --- | --- |
| 移動 | W / A / S / D |
| 環顧 | 滑鼠，或方向鍵 |
| 互動 | 對準物件後按 E，或點擊畫面互動按鈕 |
| 暫停、解除滑鼠鎖定 | Esc |
| 繼續 | 暫停畫面的「繼續探索」 |
| 音效 | 右上角切換 |
| 語言 | 右上角「繁體中文／English」選單 |

首次點擊「走進書房」後預設啟用音效並請求滑鼠鎖定；若已在開場關閉音效，遊戲保留靜音選擇。方向鍵可在不鎖定滑鼠時操作視角。切換視窗、分頁或開啟便箋與密碼介面會暫停移動；回到畫面後按「繼續探索」。移動有牆面與家具碰撞，互動限制在約三公尺內並檢查視線遮擋。

## 程式結構

- `src/puzzle.rs` 是獨立於 Bevy 的唯一謎題進度 reducer，使用 enum 表達保險箱、鑰匙、門與逃脫狀態。
- `src/room.rs` 與 `src/room/` 建立 PBR 場景、木紋與布紋小貼圖、桌面紀念物與門的動畫。站台照片由 image_gen 原創生成，詳見 `assets/artwork/README.md`；無網路字型。
- `src/score.rs` 保存原創十二小節 3/4 柔鍵盤主題、首次取鑰匙答句與短 lookahead 時鐘。Rust Web Audio 合成雙正弦諧波；閱讀時降低音量，暫停與失焦同步靜音。音樂音量、事件尾音共享 master，語言切換不重播樂句。
- `src/player.rs` 處理第一人稱相機、時間步長移動、碰撞與出口判定。
- `src/interaction.rs` 檢查互動目標並將玩家動作套用至 reducer。
- `src/session.rs` 擁有唯一遊戲 Session，使用 `UiCommand`、`Notice` 與 `SoundCue` 表達輸入及事件。
- `src/bridge.rs` 接線 Bevy systems 與 `src/browser/` 的 NonSend `BrowserUi`。Rust 擁有 DOM、事件 listeners、焦點、滑鼠鎖定與 Web Audio 合成音效；callbacks 只送出型別化命令，不共用 Session。
- `src/i18n.rs` 使用 Fluent 格式化 `assets/locales/zh-Hant.ftl`、`en.ftl`。資源內嵌至 WASM，切換語言不重載場景、不清除密碼、不重播音效。缺漏訊息先回退繁中，再顯示 key；嚴格 API 回報解析與格式化錯誤。
- `index.html` 與 `style.css` 保留原生 HTML 輸入與介面排版。`bootstrap.js` 只做啟動前 WebGPU 偵測、gzip 下載與解壓、WASM 初始化及雙語失敗提示。其餘 JavaScript 是 wasm-bindgen 產生的必要膠水。

## 檢查

```powershell
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo test --doc
cargo check --target wasm32-unknown-unknown --bin latch
./scripts/build-web.ps1
```

原生 `cargo run` 僅供場景開發檢查；雙語介面與音效以網頁為交付平台。閱讀狀態統一為 `Screen::Inspect(Exhibit)`；只有 `Letter + SafeOpen` 能領取鑰匙，之後可重讀信。新增守衛與樂譜時間邊界測試；互動、實際聲音與畫面仍須在瀏覽器驗證。核心測試涵蓋完整流程、錯誤密碼、錯誤順序、重複動作、重新開始與外部密碼輸入邊界。翻譯測試驗證兩語 key 一致、完整資源可格式化、變數、英文複數、缺漏回退與錯誤回報。尚未量測 branch coverage，不宣稱達成 95%。目前沒有存檔、後端、多關卡或手機觸控移動。
