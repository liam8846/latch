# Latch · 午夜書房

以 Rust、Bevy 0.20.1 與 WebGPU 製作的第一人稱 3D 密室逃脫。單一原創房間，約三分鐘遊玩，完整流程包括閱讀便箋、解開保險箱、取得鑰匙、打開房門與走出書房。支援重新開始。

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

開啟 <http://127.0.0.1:4173>。`dist/` 是完整靜態交付目錄，可供 Cloudflare Pages 使用；本次沒有部署。建置腳本將 WASM 壓縮為 `pkg/latch_bg.wasm.gz`，確認符合單檔 25 MiB 限制後移除原始 WASM。瀏覽器透過 DecompressionStream 展開後交給 wasm-bindgen。靜態伺服器應將 `.gz` 檔案以 `application/gzip` 提供，且不設定 `Content-Encoding`，避免瀏覽器重複解壓縮。開發迭代可用 `./scripts/build-web.ps1 -Dev`。

## 操作

| 操作 | 按鍵 |
| --- | --- |
| 移動 | W / A / S / D |
| 環顧 | 滑鼠，或方向鍵 |
| 互動 | 對準物件後按 E，或點擊畫面互動按鈕 |
| 暫停、解除滑鼠鎖定 | Esc |
| 繼續 | 暫停畫面的「繼續探索」 |
| 音效 | 右上角切換 |

首次點擊「走進書房」後啟用音效並請求滑鼠鎖定。方向鍵可在不鎖定滑鼠時操作視角。切換視窗、分頁或開啟便箋與密碼介面會暫停移動；回到畫面後按「繼續探索」。移動有牆面與家具碰撞，互動限制在約三公尺內並檢查視線遮擋。

## 程式結構

- `src/puzzle.rs` 是獨立於 Bevy 的唯一謎題進度 reducer，使用 enum 表達保險箱、鑰匙、門與逃脫狀態。
- `src/room.rs` 建立 PBR 場景與門的動畫。幾何、材質與音效均由本專案原創程序產生，沒有外部素材或網路字型。
- `src/player.rs` 處理第一人稱相機、時間步長移動、碰撞與出口判定。
- `src/interaction.rs` 檢查互動目標並將玩家動作套用至 reducer。
- `src/bridge.rs` 將唯讀呈現資料交給繁體中文 HTML 介面，並讀取動作信箱。JavaScript 不擁有或修改謎題進度。
- `app.js` 管理 DOM、滑鼠鎖定與 Web Audio 合成音效；`style.css` 定義介面。

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

原生 `cargo run` 僅供場景開發檢查；繁體中文介面與音效以網頁為交付平台。核心測試涵蓋完整流程、錯誤密碼、錯誤順序、重複動作、重新開始與外部密碼輸入邊界。尚未量測 branch coverage，不宣稱達成 95%。目前沒有存檔、後端、多關卡或手機觸控移動。
