// Gen Registry Web UI

// State
let registry = null;
let modules = [];
let installedModules = [];

// Initialize
async function init() {
    console.log('Initializing Gen Registry UI');

    // Setup event listeners
    setupEventListeners();

    // Load modules
    await loadModules();

    // Update peer count
    updatePeerCount();
}

function setupEventListeners() {
    // Search
    const searchButton = document.querySelector('.search-button');
    const searchInput = document.getElementById('search-input');

    searchButton.addEventListener('click', performSearch);
    searchInput.addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            performSearch();
        }
    });

    // Filter tags
    document.querySelectorAll('.filter-tag').forEach(tag => {
        tag.addEventListener('click', () => {
            const tagText = tag.textContent.toLowerCase();
            searchByTag(tagText);
        });
    });

    // Sort
    document.getElementById('sort-by').addEventListener('change', (e) => {
        sortModules(e.target.value);
    });

    // Modal close
    document.querySelector('.close').addEventListener('click', closeModal);
}

async function loadModules() {
    // In real implementation, this would fetch from WASM module
    // For now, use sample data
    modules = [
        {
            id: 'io.univrs.user',
            name: 'User Management',
            description: 'User authentication and profile management with JWT tokens, OAuth integration, and role-based access control.',
            version: '2.1.0',
            rating: 4.8,
            downloads: 15200,
            size: 95,
            tags: ['authentication', 'security'],
        },
        {
            id: 'io.univrs.database',
            name: 'Database Access',
            description: 'Type-safe database access with migrations, transactions, and connection pooling. Supports SQLite, PostgreSQL, MySQL.',
            version: '3.0.1',
            rating: 4.9,
            downloads: 23500,
            size: 128,
            tags: ['database', 'sql'],
        },
        {
            id: 'io.univrs.http',
            name: 'HTTP Client/Server',
            description: 'Modern HTTP client and server with async/await, middleware, routing, and WebSocket support.',
            version: '4.2.0',
            rating: 4.7,
            downloads: 31800,
            size: 142,
            tags: ['http', 'networking'],
        },
    ];

    renderModules();
}

function renderModules() {
    const grid = document.getElementById('modules-grid');
    grid.innerHTML = '';

    modules.forEach(module => {
        const card = createModuleCard(module);
        grid.appendChild(card);
    });
}

function createModuleCard(module) {
    const card = document.createElement('div');
    card.className = 'module-card';

    card.innerHTML = `
        <div class="module-header">
            <h3 class="module-name">${module.id}</h3>
            <span class="module-version">v${module.version}</span>
        </div>
        <p class="module-description">${module.description}</p>
        <div class="module-stats">
            <span class="stat">⭐ ${module.rating}</span>
            <span class="stat">⬇️ ${formatNumber(module.downloads)}</span>
            <span class="stat">📦 ${module.size} KB</span>
        </div>
        <div class="module-tags">
            ${module.tags.map(tag => `<span class="tag">${tag}</span>`).join('')}
        </div>
        <div class="module-actions">
            <button class="btn-primary" onclick="installModule('${module.id}')">Install</button>
            <button class="btn-secondary" onclick="showModuleInfo('${module.id}')">Info</button>
        </div>
    `;

    return card;
}

function formatNumber(num) {
    if (num >= 1000) {
        return (num / 1000).toFixed(1) + 'k';
    }
    return num.toString();
}

async function performSearch() {
    const query = document.getElementById('search-input').value;
    console.log('Searching for:', query);

    // In real implementation, call WASM search function
    // For now, filter locally
    if (query.trim() === '') {
        await loadModules();
        return;
    }

    modules = modules.filter(m =>
        m.id.toLowerCase().includes(query.toLowerCase()) ||
        m.description.toLowerCase().includes(query.toLowerCase()) ||
        m.tags.some(tag => tag.toLowerCase().includes(query.toLowerCase()))
    );

    renderModules();
}

function searchByTag(tag) {
    modules = modules.filter(m =>
        m.tags.some(t => t.toLowerCase() === tag)
    );
    renderModules();
}

function sortModules(sortBy) {
    switch (sortBy) {
        case 'popularity':
            modules.sort((a, b) => b.downloads - a.downloads);
            break;
        case 'recent':
            // Would sort by updated_at in real implementation
            break;
        case 'rating':
            modules.sort((a, b) => b.rating - a.rating);
            break;
        case 'name':
            modules.sort((a, b) => a.id.localeCompare(b.id));
            break;
    }
    renderModules();
}

async function installModule(moduleId) {
    console.log('Installing:', moduleId);

    // In real implementation, call WASM install function
    // Show loading state
    const btn = event.target;
    btn.textContent = 'Installing...';
    btn.disabled = true;

    // Simulate installation
    setTimeout(() => {
        btn.textContent = '✓ Installed';
        btn.disabled = false;
        installedModules.push(moduleId);
    }, 2000);
}

function showModuleInfo(moduleId) {
    const module = modules.find(m => m.id === moduleId);
    if (!module) return;

    document.getElementById('modal-title').textContent = module.id;
    document.getElementById('modal-version').textContent = `v${module.version}`;
    document.getElementById('modal-description').textContent = module.description;

    document.getElementById('module-modal').style.display = 'flex';
}

function closeModal() {
    document.getElementById('module-modal').style.display = 'none';
}

async function updatePeerCount() {
    // In real implementation, fetch from WASM P2P module
    const peerCount = 3;
    document.getElementById('peer-count').textContent = `${peerCount} peers`;
}

// Start app when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}
