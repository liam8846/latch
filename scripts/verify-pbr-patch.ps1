param(
    [string]$VendorDirectory = (Join-Path $PSScriptRoot '../vendor/bevy_pbr'),
    [string]$OriginalSourceDirectory
)

$ErrorActionPreference = 'Stop'
$manifestPath = Join-Path $PSScriptRoot '../vendor/bevy_pbr-source.json'
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
if ($manifest.schema_version -ne 1 -or $manifest.package -ne 'bevy_pbr' -or $manifest.version -ne '0.20.1') {
    throw '不支援的 bevy_pbr 來源清單。'
}

function Get-GitBlobHash([string]$Path) {
    $bytes = [IO.File]::ReadAllBytes($Path)
    $header = [Text.Encoding]::ASCII.GetBytes("blob $($bytes.Length)`0")
    $hash = [Security.Cryptography.IncrementalHash]::CreateHash([Security.Cryptography.HashAlgorithmName]::SHA1)
    try {
        $hash.AppendData($header)
        $hash.AppendData($bytes)
        return [Convert]::ToHexString($hash.GetHashAndReset()).ToLowerInvariant()
    }
    finally { $hash.Dispose() }
}

function Assert-FileHash([string]$Path, [string]$ExpectedHash) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { throw "缺少來源檔案 $Path" }
    $actual = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -cne $ExpectedHash) { throw "SHA-256 不符 $Path" }
}

$vendorRoot = [IO.Path]::GetFullPath($VendorDirectory)
$paths = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
$patchedPaths = @(
    'src/render/pbr_fragment.wesl',
    'src/render/pbr_prepass.wesl',
    'src/render/pbr_prepass_functions.wesl'
)
$patchedCount = 0
foreach ($entry in $manifest.files) {
    if (-not $paths.Add($entry.path)) { throw "重複來源路徑 $($entry.path)" }
    $path = [IO.Path]::GetFullPath((Join-Path $vendorRoot $entry.path))
    if (-not $path.StartsWith($vendorRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::Ordinal)) {
        throw "來源路徑超出 crate $($entry.path)"
    }
    Assert-FileHash $path $entry.expected_sha256
    if ($patchedPaths -ccontains $entry.path) {
        if ($entry.original_sha256 -ceq $entry.expected_sha256) { throw "缺少預期修補 $($entry.path)" }
        if ((Get-GitBlobHash $path) -cne $entry.patched_git_blob) { throw "上游修補 blob 不符 $($entry.path)" }
        $patchedCount++
    }
    elseif ($entry.original_sha256 -cne $entry.expected_sha256) { throw "上游修補範圍以外的變更 $($entry.path)" }
    if ($OriginalSourceDirectory) {
        $originalPath = Join-Path $OriginalSourceDirectory $entry.path
        Assert-FileHash $originalPath $entry.original_sha256
        if ($patchedPaths -ccontains $entry.path) {
            if ((Get-GitBlobHash $originalPath) -cne $entry.original_git_blob) { throw "上游原始 blob 不符 $($entry.path)" }
        }
    }
}

if ($paths.Count -ne 152 -or $patchedCount -ne 3) { throw '來源檔案數量或修補數量不符。' }
$actualFiles = @(Get-ChildItem -LiteralPath $vendorRoot -Recurse -Force -File)
foreach ($file in $actualFiles) {
    $relativePath = [IO.Path]::GetRelativePath($vendorRoot, $file.FullName).Replace('\', '/')
    if (-not $paths.Contains($relativePath)) { throw "非預期來源檔案 $relativePath" }
}
if ($actualFiles.Count -ne $paths.Count) { throw '來源檔案清單不完整。' }
Write-Output "bevy_pbr 0.20.1 驗證通過，共 $($paths.Count) 檔，3 個上游 shader 修補，149 檔保留原始位元組。"
