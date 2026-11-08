# bevy_pbr 授權與修改聲明

Latch 使用 [Bevy bevy_pbr 0.20.1](https://crates.io/crates/bevy_pbr/0.20.1)，依隨附的 `LICENSE-MIT` 或 `LICENSE-APACHE` 授權散布。原始專案與著作權聲明見 [Bevy 原始碼](https://github.com/bevyengine/bevy)。

Latch 修改三個 PBR shader，完整採用 [上游修補 e909449efc07d4d47d8b20e4f3c026439b058d74](https://github.com/bevyengine/bevy/commit/e909449efc07d4d47d8b20e4f3c026439b058d74)，讓 texture 與 sampler 直接出現在取樣呼叫中，以處理 [Safari shader 相容性問題 #26378](https://github.com/bevyengine/bevy/issues/26378)。修改檔案為 `pbr_fragment.wesl`、`pbr_prepass.wesl` 與 `pbr_prepass_functions.wesl`。

完整來源版本、原始套件 checksum 與逐檔雜湊記錄於 Latch 原始碼的 `vendor/README.md` 與 `vendor/bevy_pbr-source.json`。
