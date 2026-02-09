/**
 * Multi-tab synchronization tests.
 *
 * Tests document sync across multiple browser tabs:
 * - BroadcastChannel sync
 * - SharedWorker coordination
 * - Cross-tab updates
 * - Tab isolation
 */

import { test, expect, Page, BrowserContext } from '@playwright/test';

test.describe('Multi-Tab Sync', () => {
  test('should sync document changes across tabs', async ({ browser }) => {
    const context = await browser.newContext();

    // Open two tabs
    const tab1 = await context.newPage();
    const tab2 = await context.newPage();

    await tab1.goto('http://localhost:3000');
    await tab2.goto('http://localhost:3000');

    // Create document in tab 1
    await tab1.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      await vudo.createDocument('shared', 'counter', { value: 0 });
    });

    // Wait for sync
    await tab1.waitForTimeout(500);

    // Read from tab 2
    const docInTab2 = await tab2.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      return await vudo.loadDocument('shared', 'counter');
    });

    expect(docInTab2).toBeTruthy();
    expect(docInTab2.value).toBe(0);

    // Update in tab 1
    await tab1.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      await vudo.updateDocument('shared', 'counter', (doc) => {
        doc.value = 42;
      });
    });

    // Wait for sync
    await tab1.waitForTimeout(500);

    // Verify in tab 2
    const updated = await tab2.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      return await vudo.loadDocument('shared', 'counter');
    });

    expect(updated.value).toBe(42);

    await context.close();
  });

  test('should handle concurrent edits from multiple tabs', async ({ browser }) => {
    const context = await browser.newContext();

    // Open three tabs
    const tabs = [
      await context.newPage(),
      await context.newPage(),
      await context.newPage(),
    ];

    for (const tab of tabs) {
      await tab.goto('http://localhost:3000');
    }

    // Create shared document in tab 0
    await tabs[0].evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      await vudo.createDocument('concurrent', 'test', { edits: [] });
    });

    await tabs[0].waitForTimeout(500);

    // Concurrent edits from all tabs
    await Promise.all(
      tabs.map((tab, index) =>
        tab.evaluate(async (tabIndex) => {
          // @ts-ignore
          const vudo = window.VUDO;
          await vudo.updateDocument('concurrent', 'test', (doc) => {
            doc.edits.push(`tab_${tabIndex}`);
          });
        }, index)
      )
    );

    // Wait for convergence
    await tabs[0].waitForTimeout(1000);

    // Verify all edits present in tab 0
    const result = await tabs[0].evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      return await vudo.loadDocument('concurrent', 'test');
    });

    expect(result.edits).toHaveLength(3);
    expect(result.edits).toContain('tab_0');
    expect(result.edits).toContain('tab_1');
    expect(result.edits).toContain('tab_2');

    await context.close();
  });

  test('should use BroadcastChannel for tab communication', async ({ browser }) => {
    const context = await browser.newContext();

    const tab1 = await context.newPage();
    const tab2 = await context.newPage();

    await tab1.goto('http://localhost:3000');
    await tab2.goto('http://localhost:3000');

    // Listen for broadcast messages in tab 2
    await tab2.evaluate(() => {
      // @ts-ignore
      window.receivedMessages = [];
      const channel = new BroadcastChannel('vudo-sync');
      channel.onmessage = (event) => {
        // @ts-ignore
        window.receivedMessages.push(event.data);
      };
    });

    // Send update from tab 1
    await tab1.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      await vudo.createDocument('broadcast', 'test', { data: 'hello' });
    });

    await tab1.waitForTimeout(500);

    // Check messages received in tab 2
    const messages = await tab2.evaluate(() => {
      // @ts-ignore
      return window.receivedMessages;
    });

    expect(messages.length).toBeGreaterThan(0);

    await context.close();
  });

  test('should handle tab closing gracefully', async ({ browser }) => {
    const context = await browser.newContext();

    const tab1 = await context.newPage();
    const tab2 = await context.newPage();
    const tab3 = await context.newPage();

    for (const tab of [tab1, tab2, tab3]) {
      await tab.goto('http://localhost:3000');
    }

    // Create document in tab 1
    await tab1.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      await vudo.createDocument('closing', 'test', { active_tabs: 3 });
    });

    await tab1.waitForTimeout(500);

    // Close tab 2
    await tab2.close();

    await tab1.waitForTimeout(500);

    // Tab 1 and 3 should still work
    await tab1.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      await vudo.updateDocument('closing', 'test', (doc) => {
        doc.active_tabs = 2;
      });
    });

    const result = await tab3.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      return await vudo.loadDocument('closing', 'test');
    });

    expect(result.active_tabs).toBe(2);

    await context.close();
  });

  test('should isolate private documents per tab', async ({ browser }) => {
    const context = await browser.newContext();

    const tab1 = await context.newPage();
    const tab2 = await context.newPage();

    await tab1.goto('http://localhost:3000');
    await tab2.goto('http://localhost:3000');

    // Create private document in tab 1
    await tab1.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      await vudo.createDocument('private', 'tab1-only', { secret: 'tab1' }, {
        sync: false, // Don't sync across tabs
      });
    });

    // Try to load in tab 2
    const result = await tab2.evaluate(async () => {
      try {
        // @ts-ignore
        const vudo = window.VUDO;
        return await vudo.loadDocument('private', 'tab1-only');
      } catch (error) {
        return null;
      }
    });

    // Should not be accessible in tab 2
    expect(result).toBeNull();

    await context.close();
  });

  test('should handle SharedWorker for tab coordination', async ({ browser }) => {
    const context = await browser.newContext();

    const tab1 = await context.newPage();
    const tab2 = await context.newPage();

    await tab1.goto('http://localhost:3000');
    await tab2.goto('http://localhost:3000');

    // Initialize SharedWorker in both tabs
    await Promise.all([
      tab1.evaluate(() => {
        if ('SharedWorker' in window) {
          // @ts-ignore
          window.worker = new SharedWorker('/sync-worker.js');
        }
      }),
      tab2.evaluate(() => {
        if ('SharedWorker' in window) {
          // @ts-ignore
          window.worker = new SharedWorker('/sync-worker.js');
        }
      }),
    ]);

    // Create document in tab 1
    await tab1.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      await vudo.createDocument('worker', 'shared', { count: 0 });
    });

    await tab1.waitForTimeout(500);

    // Increment from both tabs
    await Promise.all([
      tab1.evaluate(async () => {
        // @ts-ignore
        const vudo = window.VUDO;
        await vudo.updateDocument('worker', 'shared', (doc) => {
          doc.count += 1;
        });
      }),
      tab2.evaluate(async () => {
        // @ts-ignore
        const vudo = window.VUDO;
        await vudo.updateDocument('worker', 'shared', (doc) => {
          doc.count += 1;
        });
      }),
    ]);

    await tab1.waitForTimeout(1000);

    // Verify final count
    const result = await tab1.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      return await vudo.loadDocument('worker', 'shared');
    });

    expect(result.count).toBe(2);

    await context.close();
  });

  test('should sync across 10 tabs', async ({ browser }) => {
    const context = await browser.newContext();

    // Open 10 tabs
    const tabs = await Promise.all(
      Array.from({ length: 10 }, () => context.newPage())
    );

    for (const tab of tabs) {
      await tab.goto('http://localhost:3000');
    }

    // Create document in first tab
    await tabs[0].evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      await vudo.createDocument('many-tabs', 'test', { updates: [] });
    });

    await tabs[0].waitForTimeout(1000);

    // Each tab makes an update
    await Promise.all(
      tabs.map((tab, index) =>
        tab.evaluate(async (tabIndex) => {
          // @ts-ignore
          const vudo = window.VUDO;
          await vudo.updateDocument('many-tabs', 'test', (doc) => {
            doc.updates.push(tabIndex);
          });
        }, index)
      )
    );

    await tabs[0].waitForTimeout(2000);

    // Verify all updates in last tab
    const result = await tabs[9].evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      return await vudo.loadDocument('many-tabs', 'test');
    });

    expect(result.updates).toHaveLength(10);
    for (let i = 0; i < 10; i++) {
      expect(result.updates).toContain(i);
    }

    await context.close();
  });
});
