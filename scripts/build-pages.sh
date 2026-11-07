set -euo pipefail

project_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd -- "$project_root"

if [[ "$(uname -s)-$(uname -m)" != Linux-x86_64 ]]; then
    printf '%s\n' 'Cloudflare Pages build requires Linux x86_64.' >&2
    exit 1
fi

rust_version=1.99.1
bindgen_version=0.2.130
tool_dir="$(mktemp -d "${TMPDIR:-/tmp}/latch-pages.XXXXXXXX")"
trap 'rm -rf -- "$tool_dir"' EXIT
export PATH="$HOME/.cargo/bin:$PATH"

if ! command -v rustup >/dev/null 2>&1; then
    curl --fail --location --silent --show-error https://sh.rustup.rs --output "$tool_dir/rustup-init.sh"
    sh "$tool_dir/rustup-init.sh" -y --profile minimal --default-toolchain none
fi
rustup toolchain install "$rust_version" --profile minimal --target wasm32-unknown-unknown

archive="wasm-bindgen-$bindgen_version-x86_64-unknown-linux-musl.tar.gz"
release="https://github.com/wasm-bindgen/wasm-bindgen/releases/download/$bindgen_version"
curl --fail --location --silent --show-error "$release/$archive" --output "$tool_dir/$archive"
curl --fail --location --silent --show-error "$release/$archive.sha256sum" --output "$tool_dir/checksum"
read -r expected_hash _ < "$tool_dir/checksum"
[[ "$expected_hash" =~ ^[0-9a-f]{64}$ ]]
printf '%s  %s\n' "$expected_hash" "$tool_dir/$archive" | sha256sum --check --status
tar -xzf "$tool_dir/$archive" -C "$tool_dir"
bindgen="$tool_dir/${archive%.tar.gz}/wasm-bindgen"
[[ "$("$bindgen" --version)" == "wasm-bindgen $bindgen_version" ]]

cargo +"$rust_version" build --locked --release --target wasm32-unknown-unknown --bin latch
wasm="${CARGO_TARGET_DIR:-$project_root/target}/wasm32-unknown-unknown/release/latch.wasm"
mkdir -p dist/pkg
rm -f -- dist/app.js
"$bindgen" --target web --no-typescript --out-dir dist/pkg "$wasm"
cp -- index.html style.css bootstrap.js dist/
cp -R -- assets/. dist/assets/
gzip -9 -n -c dist/pkg/latch_bg.wasm > dist/pkg/latch_bg.wasm.gz
compressed_bytes="$(wc -c < dist/pkg/latch_bg.wasm.gz)"
if (( compressed_bytes > 25 * 1024 * 1024 )); then
    printf '%s\n' 'Compressed WASM exceeds the Pages 25 MiB asset limit.' >&2
    exit 1
fi
rm -- dist/pkg/latch_bg.wasm
printf 'Pages output is ready in dist. Compressed WASM size: %s bytes.\n' "$compressed_bytes"
