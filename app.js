const root = document.querySelector('#interface');
const canvas = document.querySelector('#game');
const byId = id => document.getElementById(id);
let audio;
let ambientGain;
let muted = true;
let lastScreen;
let lastRevision = '0';
let toastTimer;

function command(action) {
  root.dataset.action = action;
}

function fail(message) {
  byId('loading').hidden = true;
  byId('error').hidden = false;
  byId('error-message').textContent = message;
}

function tone(frequency, duration, delay = 0, volume = 0.035) {
  if (!audio || muted) return;
  const oscillator = audio.createOscillator();
  const gain = audio.createGain();
  const start = audio.currentTime + delay;
  oscillator.frequency.value = frequency;
  oscillator.type = 'sine';
  gain.gain.setValueAtTime(0, start);
  gain.gain.linearRampToValueAtTime(volume, start + 0.012);
  gain.gain.exponentialRampToValueAtTime(0.001, start + duration);
  oscillator.connect(gain).connect(audio.destination);
  oscillator.start(start);
  oscillator.stop(start + duration);
}

function unlockAudio() {
  if (!audio) {
    audio = new AudioContext();
    ambientGain = audio.createGain();
    ambientGain.gain.value = 0;
    ambientGain.connect(audio.destination);
    for (const frequency of [61.735, 92.5]) {
      const oscillator = audio.createOscillator();
      oscillator.frequency.value = frequency;
      oscillator.connect(ambientGain);
      oscillator.start();
    }
  }
  audio.resume().catch(() => {});
}

function setSound(enabled) {
  muted = !enabled;
  byId('sound').textContent = enabled ? '音效 開' : '音效 關';
  byId('sound').setAttribute('aria-pressed', String(enabled));
  byId('sound').setAttribute('aria-label', enabled ? '關閉音效' : '啟用音效');
  if (ambientGain) ambientGain.gain.setTargetAtTime(enabled && root.dataset.screen === 'playing' ? 0.008 : 0, audio.currentTime, 0.3);
}

function enter(action) {
  unlockAudio();
  canvas.focus();
  try {
    const lock = canvas.requestPointerLock?.();
    lock?.catch(() => {});
  } catch {}
  command(action);
}

function render() {
  const screen = root.dataset.screen;
  if (screen === 'loading') return;
  byId('loading').hidden = true;
  for (const name of ['intro', 'paused', 'note', 'safe', 'won']) byId(name).hidden = name !== screen;
  byId('shade').hidden = screen === 'playing';
  byId('hud').hidden = screen !== 'playing';
  byId('pause').hidden = screen !== 'playing';
  byId('objective').textContent = root.dataset.objective || '';
  byId('inventory').textContent = root.dataset.inventory || '';
  byId('target').textContent = root.dataset.target || '';
  byId('interact').hidden = !root.dataset.target;
  if (lastScreen !== screen) {
    if (screen !== 'playing' && document.pointerLockElement) document.exitPointerLock();
    if (screen === 'playing') canvas.focus({ preventScroll: true });
    if (screen === 'safe') {
      byId('code').value = '';
      byId('safe-message').textContent = '線索也許就在書桌上。';
      byId('code').focus();
    }
    if (screen === 'intro') byId('begin').focus({ preventScroll: true });
    if (screen === 'paused') byId('resume').focus({ preventScroll: true });
    if (ambientGain) ambientGain.gain.setTargetAtTime(!muted && screen === 'playing' ? 0.008 : 0, audio.currentTime, 0.3);
    lastScreen = screen;
  }
  if (root.dataset.revision !== lastRevision) {
    lastRevision = root.dataset.revision;
    byId('toast').textContent = root.dataset.message || '';
    byId('safe-message').textContent = root.dataset.message || '';
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => { byId('toast').textContent = ''; }, 5500);
    const sound = root.dataset.sound;
    if (sound === 'wrong') {
      tone(140, 0.2);
      tone(110, 0.25, 0.12);
      if (screen === 'safe') byId('code').focus();
    }
    else if (sound === 'win') { [261.63, 329.63, 392, 523.25].forEach((f, i) => tone(f, 1.2, i * 0.16)); }
    else if (sound === 'unlock' || sound === 'key') { tone(440, 0.45); tone(659.25, 0.7, 0.12); }
    else tone(330, 0.15);
  }
}

new MutationObserver(render).observe(root, { attributes: true, attributeFilter: ['data-screen', 'data-target', 'data-stage', 'data-objective', 'data-inventory', 'data-revision'] });
byId('begin').addEventListener('click', () => { setSound(true); enter('begin'); tone(261.63, 0.6); });
byId('resume').addEventListener('click', () => enter('resume'));
byId('pause').addEventListener('click', () => command('pause'));
byId('interact').addEventListener('click', () => { canvas.focus(); command('interact'); });
byId('sound').addEventListener('click', () => {
  unlockAudio();
  setSound(muted);
  if (root.dataset.screen === 'playing') canvas.focus({ preventScroll: true });
});
document.querySelectorAll('[data-command]').forEach(button => button.addEventListener('click', () => enter(button.dataset.command)));
byId('code-form').addEventListener('submit', event => {
  event.preventDefault();
  const code = byId('code').value;
  canvas.focus({ preventScroll: true });
  command(`code:${code}`);
});
byId('code').addEventListener('input', event => { event.target.value = event.target.value.replace(/[^0-9]/g, '').slice(0, 4); });
canvas.addEventListener('click', () => { if (root.dataset.screen === 'playing' && !document.pointerLockElement) enter('resume'); });
document.addEventListener('keydown', event => {
  if (event.code === 'Escape') { command('pause'); event.preventDefault(); }
  if (root.dataset.screen === 'playing' && ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Space'].includes(event.code)) event.preventDefault();
});
window.addEventListener('blur', () => { if (root.dataset.screen === 'playing') command('pause'); if (document.pointerLockElement) document.exitPointerLock(); });
document.addEventListener('visibilitychange', () => { if (document.hidden && root.dataset.screen === 'playing') command('pause'); });
document.addEventListener('pointerlockchange', () => { if (!document.pointerLockElement && root.dataset.screen === 'playing') command('pause'); });
window.addEventListener('error', event => { if (!String(event.message).includes('Using exceptions for control flow')) fail('場景初始化失敗。請重新載入，並確認瀏覽器已開啟硬體加速。'); });
try {
  if (!navigator.gpu) throw new Error('這個瀏覽器目前無法使用 WebGPU。請以支援 WebGPU 的新版桌面 Chrome 或 Edge 開啟，並啟用硬體加速。');
  const adapter = await navigator.gpu.requestAdapter();
  if (!adapter) throw new Error('找不到可用的 WebGPU 顯示裝置。請檢查硬體加速與顯示卡驅動後再試。');
  if (!('DecompressionStream' in window)) throw new Error('這個瀏覽器無法展開遊戲檔案。請使用支援 WebGPU 的新版桌面 Chrome 或 Edge。');
  byId('load-detail').textContent = '正在準備光影與房間';
  const { default: init } = await import('./pkg/latch.js');
  const response = await fetch('./pkg/latch_bg.wasm.gz');
  if (!response.ok || !response.body) throw new Error('遊戲檔案下載失敗。請確認網路連線後重新載入。');
  const decoded = response.body.pipeThrough(new DecompressionStream('gzip'));
  const bytes = await new Response(decoded).arrayBuffer();
  await init({ module_or_path: bytes });
} catch (error) {
  fail(error instanceof Error ? error.message : '書房載入失敗。請重新整理後再試。');
}
