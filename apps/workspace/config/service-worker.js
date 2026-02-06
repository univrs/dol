// VUDO Workspace Service Worker
// Handles offline caching, P2P sync coordination, and background operations

const CACHE_NAME = 'vudo-workspace-v1';
const RUNTIME_CACHE = 'vudo-runtime-v1';

// Files to cache on install
const PRECACHE_URLS = [
  '/',
  '/index.html',
  '/styles.css',
  '/app.js',
  '/manifest.json',
  '/wasm/vudo_workspace_bg.wasm',
  '/icons/icon-192x192.png',
  '/icons/icon-512x512.png'
];

// Install event - cache static assets
self.addEventListener('install', event => {
  event.waitUntil(
    caches.open(CACHE_NAME)
      .then(cache => cache.addAll(PRECACHE_URLS))
      .then(() => self.skipWaiting())
  );
});

// Activate event - clean up old caches
self.addEventListener('activate', event => {
  const currentCaches = [CACHE_NAME, RUNTIME_CACHE];
  event.waitUntil(
    caches.keys().then(cacheNames => {
      return cacheNames.filter(cacheName => !currentCaches.includes(cacheName));
    }).then(cachesToDelete => {
      return Promise.all(cachesToDelete.map(cacheToDelete => {
        return caches.delete(cacheToDelete);
      }));
    }).then(() => self.clients.claim())
  );
});

// Fetch event - serve from cache, fallback to network
self.addEventListener('fetch', event => {
  // Skip non-GET requests
  if (event.request.method !== 'GET') {
    return;
  }

  // Skip chrome-extension and other non-http(s) requests
  if (!event.request.url.startsWith('http')) {
    return;
  }

  event.respondWith(
    caches.match(event.request).then(cachedResponse => {
      if (cachedResponse) {
        return cachedResponse;
      }

      return caches.open(RUNTIME_CACHE).then(cache => {
        return fetch(event.request).then(response => {
          // Only cache successful responses
          if (response.status === 200) {
            cache.put(event.request, response.clone());
          }
          return response;
        }).catch(() => {
          // Return offline page if available
          return caches.match('/offline.html');
        });
      });
    })
  );
});

// Background sync for P2P operations
self.addEventListener('sync', event => {
  if (event.tag === 'sync-documents') {
    event.waitUntil(syncDocuments());
  } else if (event.tag === 'sync-tasks') {
    event.waitUntil(syncTasks());
  } else if (event.tag === 'sync-credits') {
    event.waitUntil(syncCreditTransactions());
  }
});

async function syncDocuments() {
  console.log('[Service Worker] Syncing documents...');

  // Get pending document changes from IndexedDB
  const db = await openDatabase();
  const pendingChanges = await db.getAll('pending_document_changes');

  // Send changes to peers via P2P sync
  for (const change of pendingChanges) {
    try {
      await broadcastChange('document', change);
      await db.delete('pending_document_changes', change.id);
    } catch (error) {
      console.error('[Service Worker] Failed to sync document:', error);
    }
  }
}

async function syncTasks() {
  console.log('[Service Worker] Syncing tasks...');

  const db = await openDatabase();
  const pendingChanges = await db.getAll('pending_task_changes');

  for (const change of pendingChanges) {
    try {
      await broadcastChange('task', change);
      await db.delete('pending_task_changes', change.id);
    } catch (error) {
      console.error('[Service Worker] Failed to sync task:', error);
    }
  }
}

async function syncCreditTransactions() {
  console.log('[Service Worker] Syncing credit transactions...');

  const db = await openDatabase();
  const pendingTransactions = await db.getAll('pending_credit_transactions');

  for (const transaction of pendingTransactions) {
    try {
      await broadcastChange('credit', transaction);
      await db.delete('pending_credit_transactions', transaction.id);
    } catch (error) {
      console.error('[Service Worker] Failed to sync credit transaction:', error);
    }
  }
}

async function broadcastChange(type, change) {
  // Send message to all clients to broadcast via Iroh P2P
  const clients = await self.clients.matchAll();
  for (const client of clients) {
    client.postMessage({
      type: 'broadcast-change',
      changeType: type,
      change: change
    });
  }
}

function openDatabase() {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open('vudo-workspace', 1);

    request.onerror = () => reject(request.error);
    request.onsuccess = () => resolve(request.result);

    request.onupgradeneeded = event => {
      const db = event.target.result;

      if (!db.objectStoreNames.contains('pending_document_changes')) {
        db.createObjectStore('pending_document_changes', { keyPath: 'id' });
      }
      if (!db.objectStoreNames.contains('pending_task_changes')) {
        db.createObjectStore('pending_task_changes', { keyPath: 'id' });
      }
      if (!db.objectStoreNames.contains('pending_credit_transactions')) {
        db.createObjectStore('pending_credit_transactions', { keyPath: 'id' });
      }
    };
  });
}

// Push notifications for credit transfers and task assignments
self.addEventListener('push', event => {
  const data = event.data ? event.data.json() : {};

  const options = {
    body: data.body || 'You have a new notification',
    icon: '/icons/icon-192x192.png',
    badge: '/icons/badge-72x72.png',
    tag: data.tag || 'default',
    data: data,
    actions: data.actions || []
  };

  event.waitUntil(
    self.registration.showNotification(data.title || 'VUDO Workspace', options)
  );
});

// Notification click handler
self.addEventListener('notificationclick', event => {
  event.notification.close();

  event.waitUntil(
    clients.openWindow(event.notification.data.url || '/')
  );
});

// Periodic background sync for continuous P2P sync
self.addEventListener('periodicsync', event => {
  if (event.tag === 'continuous-sync') {
    event.waitUntil(performContinuousSync());
  }
});

async function performContinuousSync() {
  console.log('[Service Worker] Performing continuous sync...');

  // Sync all pending changes
  await syncDocuments();
  await syncTasks();
  await syncCreditTransactions();

  // Check for incoming changes from peers
  const clients = await self.clients.matchAll();
  for (const client of clients) {
    client.postMessage({
      type: 'check-peer-updates'
    });
  }
}

console.log('[Service Worker] VUDO Workspace service worker loaded');
