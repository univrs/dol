// ═══════════════════════════════════════════════════════════════════
// Game of Life Spirit - JavaScript Frontend (Fixed WASM Loading)
// ═══════════════════════════════════════════════════════════════════

// Configuration
const CONFIG = {
  gridSize: 100,
  cellSize: 8,
  wrapEdges: true,
  colors: { dead: '#1a1a2e', alive: '#00ff88' }
};

let wasm = null;
let running = false;
let animationId = null;
let lastFrameTime = 0;
let targetFps = 10;
let isDrawing = false;

const canvas = document.getElementById('canvas');
const ctx = canvas.getContext('2d');

// ═══════════════════════════════════════════════════════════════════
// WASM Loading (with better error handling)
// ═══════════════════════════════════════════════════════════════════

async function loadWasm() {
  try {
    // Try to load the WASM module
    const wasmPath = './game_of_life_bg.wasm';

    // First check if the file exists and is actually WASM
    const response = await fetch(wasmPath);

    if (!response.ok) {
      throw new Error(`Failed to fetch WASM: ${response.status} ${response.statusText}`);
    }

    const contentType = response.headers.get('content-type');
    if (contentType && contentType.includes('text/html')) {
      throw new Error('Server returned HTML instead of WASM. Check that game_of_life_bg.wasm exists.');
    }

    const wasmBytes = await response.arrayBuffer();

    if (wasmBytes.byteLength < 8) {
      throw new Error('WASM file is too small or empty');
    }

    // Check WASM magic number
    const magic = new Uint8Array(wasmBytes.slice(0, 4));
    if (magic[0] !== 0x00 || magic[1] !== 0x61 || magic[2] !== 0x73 || magic[3] !== 0x6d) {
      throw new Error('Invalid WASM file (bad magic number)');
    }

    // Import the JS bindings
    const js = await import('./game_of_life.js');

    // Initialize with the fetched bytes
    await js.default(wasmBytes);

    return js;
  } catch (error) {
    console.error('WASM loading failed:', error);
    document.body.innerHTML = `
      <div style="color: #ff4444; background: #1a1a2e; padding: 40px; font-family: monospace; max-width: 600px; margin: 50px auto; border-radius: 10px;">
        <h2>⚠️ WASM Loading Error</h2>
        <p style="color: #ffaa00;">${error.message}</p>
        <h3>Troubleshooting:</h3>
        <ol style="line-height: 2;">
          <li>Make sure you've run <code style="background:#333;padding:2px 6px;">./build.sh</code></li>
          <li>Check that these files exist in the web/ folder:
            <ul>
              <li><code>game_of_life.js</code></li>
              <li><code>game_of_life_bg.wasm</code></li>
            </ul>
          </li>
          <li>Serve files with a local server (not file://):
            <br><code style="background:#333;padding:2px 6px;">python3 -m http.server 8080</code>
          </li>
        </ol>
      </div>
    `;
    throw error;
  }
}

// ═══════════════════════════════════════════════════════════════════
// Rendering
// ═══════════════════════════════════════════════════════════════════

function resizeCanvas() {
  const container = canvas.parentElement;
  const size = Math.min(container.clientWidth, 800);
  canvas.width = size;
  canvas.height = size;
  CONFIG.cellSize = size / CONFIG.gridSize;
  render();
}

function render() {
  if (!wasm) return;

  const cells = wasm.get_cells();
  const width = wasm.get_width();
  const cs = CONFIG.cellSize;

  // Clear
  ctx.fillStyle = CONFIG.colors.dead;
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  // Draw alive cells
  ctx.fillStyle = CONFIG.colors.alive;
  for (let i = 0; i < cells.length; i++) {
    if (cells[i] === 1) {
      const x = (i % width) * cs;
      const y = Math.floor(i / width) * cs;
      ctx.fillRect(x, y, cs - 0.5, cs - 0.5);
    }
  }

  // Update stats
  document.getElementById('gen').textContent = wasm.get_generation();
  document.getElementById('alive').textContent = wasm.get_alive_count();
}

// ═══════════════════════════════════════════════════════════════════
// Game Loop
// ═══════════════════════════════════════════════════════════════════

function gameLoop(timestamp) {
  if (!running) return;

  const elapsed = timestamp - lastFrameTime;
  const frameInterval = 1000 / targetFps;

  if (elapsed >= frameInterval) {
    wasm.step();
    render();
    lastFrameTime = timestamp - (elapsed % frameInterval);
  }

  animationId = requestAnimationFrame(gameLoop);
}

function start() {
  if (!running) {
    running = true;
    lastFrameTime = performance.now();
    animationId = requestAnimationFrame(gameLoop);
    document.getElementById('start').textContent = '⏸ Pause';
  } else {
    stop();
  }
}

function stop() {
  running = false;
  if (animationId) {
    cancelAnimationFrame(animationId);
    animationId = null;
  }
  document.getElementById('start').textContent = '▶ Start';
}

// ═══════════════════════════════════════════════════════════════════
// Mouse Interaction
// ═══════════════════════════════════════════════════════════════════

function getCellPos(event) {
  const rect = canvas.getBoundingClientRect();
  const x = Math.floor((event.clientX - rect.left) / CONFIG.cellSize);
  const y = Math.floor((event.clientY - rect.top) / CONFIG.cellSize);
  return { x, y };
}

canvas.addEventListener('mousedown', (e) => {
  if (!wasm) return;
  isDrawing = true;
  const pos = getCellPos(e);
  wasm.toggle_cell(pos.x, pos.y);
  render();
});

canvas.addEventListener('mousemove', (e) => {
  if (!isDrawing || !wasm) return;
  const pos = getCellPos(e);
  wasm.set_cell_state(pos.x, pos.y, true);
  render();
});

canvas.addEventListener('mouseup', () => {
  isDrawing = false;
});

canvas.addEventListener('mouseleave', () => {
  isDrawing = false;
});

// ═══════════════════════════════════════════════════════════════════
// Control Handlers
// ═══════════════════════════════════════════════════════════════════

document.getElementById('start').addEventListener('click', start);

document.getElementById('stop').addEventListener('click', () => {
  stop();
});

document.getElementById('step').addEventListener('click', () => {
  if (!wasm) return;
  wasm.step();
  render();
});

document.getElementById('clear').addEventListener('click', () => {
  if (!wasm) return;
  stop();
  wasm.reset();
  render();
});

document.getElementById('random').addEventListener('click', () => {
  if (!wasm) return;
  wasm.randomize_grid(0.3);
  render();
});

document.getElementById('speed').addEventListener('input', (e) => {
  targetFps = parseInt(e.target.value);
  document.getElementById('speed-value').textContent = targetFps;
});

document.getElementById('place-pattern').addEventListener('click', () => {
  if (!wasm) return;
  const pattern = document.getElementById('pattern').value;
  if (pattern) {
    const cx = Math.floor(CONFIG.gridSize / 2) - 5;
    const cy = Math.floor(CONFIG.gridSize / 2) - 5;
    wasm.load_pattern(pattern, cx, cy);
    render();
  }
});

// ═══════════════════════════════════════════════════════════════════
// Initialization
// ═══════════════════════════════════════════════════════════════════

async function main() {
  console.log('🎮 Loading Game of Life Spirit...');

  // Load WASM
  wasm = await loadWasm();

  // Initialize game
  wasm.init(CONFIG.gridSize, CONFIG.gridSize, CONFIG.wrapEdges);

  // Update grid size display
  document.getElementById('grid-size').textContent = `${CONFIG.gridSize}×${CONFIG.gridSize}`;

  // Setup canvas
  resizeCanvas();
  window.addEventListener('resize', resizeCanvas);

  // Initial render
  render();

  console.log('✅ Game of Life Spirit initialized');
}

main().catch(err => console.error('Failed to initialize:', err));