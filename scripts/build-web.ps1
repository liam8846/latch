param([switch]$Dev)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path $PSScriptRoot -Parent
Push-Location $projectRoot
try {
    $bindgenVersion = (& wasm-bindgen --version)
    if ($bindgenVersion -ne 'wasm-bindgen 0.2.130') {
        throw '需要 wasm-bindgen-cli 0.2.130。執行 cargo install wasm-bindgen-cli --version 0.2.130 --locked。'
    }
    $cargoArguments = @('build', '--locked', '--target', 'wasm32-unknown-unknown', '--bin', 'latch')
    $profileDirectory = 'debug'
    if (-not $Dev) {
        $cargoArguments += '--release'
        $profileDirectory = 'release'
    }
    & cargo @cargoArguments
    if ($LASTEXITCODE -ne 0) { throw 'Rust WASM 編譯失敗。' }
    $targetDirectory = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $projectRoot 'target' }
    $wasmPath = Join-Path $targetDirectory "wasm32-unknown-unknown/$profileDirectory/latch.wasm"
    $dist = Join-Path $projectRoot 'dist'
    New-Item -ItemType Directory -Path (Join-Path $dist 'pkg') -Force | Out-Null
    $legacyScript = [IO.Path]::GetFullPath((Join-Path $dist 'app.js'))
    $resolvedDist = [IO.Path]::GetFullPath($dist)
    if (-not $legacyScript.StartsWith($resolvedDist + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
        throw '舊版啟動檔清理路徑超出 dist。'
    }
    if (Test-Path -LiteralPath $legacyScript -PathType Leaf) { Remove-Item -LiteralPath $legacyScript }

    & wasm-bindgen --target web --no-typescript --out-dir (Join-Path $dist 'pkg') $wasmPath
    if ($LASTEXITCODE -ne 0) { throw 'wasm-bindgen 匯出失敗。' }
    Copy-Item -LiteralPath 'index.html', 'style.css', 'bootstrap.js' -Destination $dist -Force
    Copy-Item -LiteralPath 'assets' -Destination $dist -Recurse -Force
    $artifact = Get-Item -LiteralPath (Join-Path $dist 'pkg/latch_bg.wasm')
    $gzipPath = Join-Path $dist 'pkg/latch_bg.wasm.gz'
    $inputStream = [IO.File]::OpenRead($artifact.FullName)
    try {
        $outputStream = [IO.File]::Create($gzipPath)
        try {
            $gzipStream = [IO.Compression.GZipStream]::new($outputStream, [IO.Compression.CompressionLevel]::SmallestSize)
            try { $inputStream.CopyTo($gzipStream) }
            finally { $gzipStream.Dispose() }
        }
        finally { $outputStream.Dispose() }
    }
    finally { $inputStream.Dispose() }
    $compressed = Get-Item -LiteralPath $gzipPath
    if ($compressed.Length -gt 25MB) { throw '壓縮 WASM 超過 Cloudflare Pages 單檔 25 MiB 限制。' }
    if (-not $artifact.FullName.StartsWith($dist + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'WASM 清理路徑超出 dist。'
    }
    Remove-Item -LiteralPath $artifact.FullName
    Write-Output ("已建立 dist，WASM 原始大小 {0:N2} MiB，靜態 gzip 交付檔案 {1:N2} MiB。" -f ($artifact.Length / 1MB), ($compressed.Length / 1MB))
}
finally { Pop-Location }
