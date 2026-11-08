# bevy_pbr 來源與 Safari 修補

`bevy_pbr/` 保留 crates.io 發行的 [bevy_pbr 0.20.1](https://crates.io/crates/bevy_pbr/0.20.1) 來源。原始套件 SHA-256 為 `1138d6ed9bbcc43a4d83878062869c20d4292f1112b177acfc1a20cf6ae14f3e`，取自修改前的根目錄 `Cargo.lock`。

原始專案為 [Bevy](https://github.com/bevyengine/bevy)，作者與著作權聲明保留於套件的 `LICENSE-MIT`、`LICENSE-APACHE` 與原始檔。套件 `.cargo_vcs_info.json` 記錄來源 commit `a0577b5c6a7b208719dd761c5c6a7a539e472a97`，路徑為 `crates/bevy_pbr`。保留發行套件的 `Cargo.toml`、`Cargo.toml.orig`、`Cargo.lock`、README 與授權檔，僅略過 Cargo 的快取標記 `.cargo-ok`。

Latch 套用 [上游修補 e909449efc07d4d47d8b20e4f3c026439b058d74](https://github.com/bevyengine/bevy/commit/e909449efc07d4d47d8b20e4f3c026439b058d74)，對應 [Bevy issue #26378](https://github.com/bevyengine/bevy/issues/26378)。此修補把 texture 與 sampler 直接傳入取樣函式，避免 Safari 的 WGSL 編譯器拒絕以 `let` 建立 texture 或 sampler 別名。僅修改以下三檔，其餘 149 檔與發行來源逐位元組相同。

- `src/render/pbr_fragment.wesl` 修正 PBR 材質貼圖取樣。
- `src/render/pbr_prepass.wesl` 修正 prepass 法線貼圖取樣。
- `src/render/pbr_prepass_functions.wesl` 修正 prepass 基底色與透明度取樣。

`bevy_pbr-source.json` 記錄全部 152 檔的原始與預期 SHA-256，以及三個 shader 的原始與修補後 Git blob SHA。三個 blob 與上游補丁的 before 與 after index 相符。`.gitattributes` 僅對此 crate 停用換行轉換與 whitespace 檢查，保留上游既有空白與跨平台檢出的原始位元組；第一方檔案仍採原本檢查。

在專案根目錄執行 `pwsh -File scripts/verify-pbr-patch.ps1` 可驗證檔案內容與清單。驗證器會拒絕缺失、新增或變更的檔案。若有原始 crates.io 解壓目錄，可加入 `-OriginalSourceDirectory <path>`，另外驗證原始雜湊。`-VendorDirectory <path>` 可針對測試副本執行同一檢查。

根目錄 `Cargo.toml` 的 `[patch.crates-io]` 讓原生與 WASM 使用同一份修補來源。待正式 Bevy 發行版包含此修正，且 Latch 在實際 Safari 驗證啟動、遊戲互動與音效後，改回正式依賴並移除此 vendor crate、來源清單、驗證腳本與 Cargo patch。此修補尚不代表 Safari 問題已經實機確認解決。
