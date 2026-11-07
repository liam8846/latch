const byId = id => document.getElementById(id);
function fail(message) {
  byId('loading').hidden = true;
  byId('error').hidden = false;
  byId('error-message').textContent = message;
}
const startupError = event => {
  if (!String(event.message).includes('Using exceptions for control flow')) {
    fail('場景初始化失敗。請重新載入並檢查硬體加速。 / The scene could not start. Reload and check hardware acceleration.');
  }
};
window.addEventListener('error', startupError);
try {
  if (!navigator.gpu) throw new Error('無法使用 WebGPU。請使用新版桌面 Chrome 或 Edge 並啟用硬體加速。 / WebGPU is unavailable. Use current desktop Chrome or Edge with hardware acceleration enabled.');
  const adapter = await navigator.gpu.requestAdapter();
  if (!adapter) throw new Error('找不到可用的 WebGPU 顯示裝置。請檢查硬體加速與顯示卡驅動。 / No WebGPU adapter was found. Check hardware acceleration and graphics drivers.');
  if (!('DecompressionStream' in window)) throw new Error('瀏覽器無法展開遊戲檔案。請更新瀏覽器。 / This browser cannot decompress the game. Please update your browser.');
  const { default: init } = await import('./pkg/latch.js');
  const response = await fetch('./pkg/latch_bg.wasm.gz');
  if (!response.ok || !response.body) throw new Error('遊戲檔案下載失敗。請檢查連線後重新載入。 / The game download failed. Check your connection and reload.');
  const decoded = response.body.pipeThrough(new DecompressionStream('gzip'));
  const bytes = await new Response(decoded).arrayBuffer();
  await init({ module_or_path: bytes });
} catch (error) {
  fail(error instanceof Error ? error.message : '書房載入失敗。請重新載入。 / The study could not load. Please reload.');
}
finally {
  window.removeEventListener('error', startupError);
}
