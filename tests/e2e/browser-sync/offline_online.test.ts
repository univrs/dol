/**
 * Offline/online transition tests in browser.
 *
 * Tests browser-specific offline behavior:
 * - navigator.onLine detection
 * - Offline queue persistence
 * - Background sync
 * - Network change events
 */

import { test, expect, Page, BrowserContext } from '@playwright/test';

test.describe('Browser Offline/Online', () => {
  test('should detect online/offline status', async ({ browser }) => {
    const context = await browser.newContext();
    const page = await context.newPage();

    await page.goto('http://localhost:3000');

    // Check initial status
    const initialStatus = await page.evaluate(() => navigator.onLine);
    expect(initialStatus).toBe(true);

    // Go offline
    await context.setOffline(true);

    const offlineStatus = await page.evaluate(() => navigator.onLine);
    expect(offlineStatus).toBe(false);

    // Come back online
    await context.setOffline(false);

    const onlineStatus = await page.evaluate(() => navigator.onLine);
    expect(onlineStatus).toBe(true);

    await context.close();
  });

  test('should queue operations while offline', async ({ browser }) => {
    const context = await browser.newContext();
    const page = await context.newPage();

    await page.goto('http://localhost:3000');

    // Create initial document
    await page.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      await vudo.createDocument('offline', 'queue', { counter: 0 });
    });

    // Go offline
    await context.setOffline(true);

    // Make offline edits
    for (let i = 0; i < 5; i++) {
      await page.evaluate(async (i) => {
        // @ts-ignore
        const vudo = window.VUDO;
        await vudo.updateDocument('offline', 'queue', (doc) => {
          doc.counter = i + 1;
        });
      }, i);
    }

    // Check queue length
    const queueLength = await page.evaluate(() => {
      // @ts-ignore
      return window.VUDO.getQueueLength();
    });

    expect(queueLength).toBeGreaterThan(0);

    // Come back online
    await context.setOffline(false);

    // Wait for queue to drain
    await page.waitForTimeout(2000);

    // Verify queue is empty
    const finalQueueLength = await page.evaluate(() => {
      // @ts-ignore
      return window.VUDO.getQueueLength();
    });

    expect(finalQueueLength).toBe(0);

    // Verify final state
    const doc = await page.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      return await vudo.loadDocument('offline', 'queue');
    });

    expect(doc.counter).toBe(5);

    await context.close();
  });

  test('should handle background sync', async ({ browser }) => {
    const context = await browser.newContext();
    const page = await context.newPage();

    await page.goto('http://localhost:3000');

    // Register background sync
    await page.evaluate(async () => {
      if ('serviceWorker' in navigator && 'sync' in ServiceWorkerRegistration.prototype) {
        const registration = await navigator.serviceWorker.ready;
        await registration.sync.register('vudo-sync');
      }
    });

    // Create document
    await page.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      await vudo.createDocument('bg-sync', 'test', { synced: false });
    });

    // Go offline
    await context.setOffline(true);

    // Update document
    await page.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      await vudo.updateDocument('bg-sync', 'test', (doc) => {
        doc.synced = true;
      });
    });

    // Come back online (background sync should trigger)
    await context.setOffline(false);

    await page.waitForTimeout(2000);

    // Verify sync occurred
    const doc = await page.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      return await vudo.loadDocument('bg-sync', 'test');
    });

    expect(doc.synced).toBe(true);

    await context.close();
  });

  test('should persist offline queue across page reload', async ({ browser }) => {
    const context = await browser.newContext();
    const page = await context.newPage();

    await page.goto('http://localhost:3000');

    // Go offline
    await context.setOffline(true);

    // Create documents while offline
    await page.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      await vudo.createDocument('persist', 'doc1', { value: 1 });
      await vudo.createDocument('persist', 'doc2', { value: 2 });
      await vudo.createDocument('persist', 'doc3', { value: 3 });
    });

    // Reload page (still offline)
    await page.reload();

    // Check queue persisted
    const queueLength = await page.evaluate(() => {
      // @ts-ignore
      return window.VUDO.getQueueLength();
    });

    expect(queueLength).toBeGreaterThanOrEqual(3);

    await context.close();
  });

  test('should handle rapid online/offline transitions', async ({ browser }) => {
    const context = await browser.newContext();
    const page = await context.newPage();

    await page.goto('http://localhost:3000');

    // Create document
    await page.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      await vudo.createDocument('rapid', 'test', { transitions: 0 });
    });

    // Rapid transitions
    for (let i = 0; i < 10; i++) {
      await context.setOffline(true);
      await page.waitForTimeout(100);

      await page.evaluate(async (i) => {
        // @ts-ignore
        const vudo = window.VUDO;
        await vudo.updateDocument('rapid', 'test', (doc) => {
          doc.transitions = i + 1;
        });
      }, i);

      await context.setOffline(false);
      await page.waitForTimeout(100);
    }

    // Wait for stabilization
    await page.waitForTimeout(1000);

    // Verify final state
    const doc = await page.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      return await vudo.loadDocument('rapid', 'test');
    });

    expect(doc.transitions).toBe(10);

    await context.close();
  });

  test('should listen to online/offline events', async ({ browser }) => {
    const context = await browser.newContext();
    const page = await context.newPage();

    await page.goto('http://localhost:3000');

    // Setup event listeners
    await page.evaluate(() => {
      // @ts-ignore
      window.networkEvents = [];

      window.addEventListener('online', () => {
        // @ts-ignore
        window.networkEvents.push('online');
      });

      window.addEventListener('offline', () => {
        // @ts-ignore
        window.networkEvents.push('offline');
      });
    });

    // Go offline
    await context.setOffline(true);
    await page.waitForTimeout(500);

    // Go online
    await context.setOffline(false);
    await page.waitForTimeout(500);

    // Check events
    const events = await page.evaluate(() => {
      // @ts-ignore
      return window.networkEvents;
    });

    expect(events).toContain('offline');
    expect(events).toContain('online');

    await context.close();
  });

  test('should sync with remote peer after reconnection', async ({ browser }) => {
    const context = await browser.newContext();

    // Two browser instances (simulating two users)
    const page1 = await context.newPage();
    const page2 = await context.newPage();

    await page1.goto('http://localhost:3000');
    await page2.goto('http://localhost:3000');

    // Create shared document on page 1
    await page1.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      await vudo.createDocument('remote', 'shared', { version: 1 });
    });

    await page1.waitForTimeout(500);

    // Page 1 goes offline
    await page1.context().setOffline(true);

    // Page 1 makes offline edits
    await page1.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      await vudo.updateDocument('remote', 'shared', (doc) => {
        doc.version = 2;
        doc.offline_edit = true;
      });
    });

    // Page 2 makes online edits
    await page2.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      await vudo.updateDocument('remote', 'shared', (doc) => {
        doc.online_edit = true;
      });
    });

    // Page 1 comes back online
    await page1.context().setOffline(false);

    // Wait for sync
    await page1.waitForTimeout(2000);

    // Both pages should have both edits
    const doc1 = await page1.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      return await vudo.loadDocument('remote', 'shared');
    });

    const doc2 = await page2.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      return await vudo.loadDocument('remote', 'shared');
    });

    expect(doc1.offline_edit).toBe(true);
    expect(doc1.online_edit).toBe(true);
    expect(doc2.offline_edit).toBe(true);
    expect(doc2.online_edit).toBe(true);

    await context.close();
  });

  test('should handle connection timeout gracefully', async ({ browser }) => {
    const context = await browser.newContext();
    const page = await context.newPage();

    await page.goto('http://localhost:3000');

    // Create document
    await page.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      await vudo.createDocument('timeout', 'test', { status: 'pending' });
    });

    // Simulate slow network (offline for extended period)
    await context.setOffline(true);

    // Try to sync (should timeout gracefully)
    const syncResult = await page.evaluate(async () => {
      try {
        // @ts-ignore
        const vudo = window.VUDO;
        await vudo.syncNow(); // Force sync attempt
        return { success: true };
      } catch (error) {
        return { success: false, error: error.message };
      }
    });

    // Should fail gracefully, not crash
    expect(syncResult.success).toBe(false);

    await context.close();
  });
});
