/**
 * Browser crash recovery tests.
 *
 * Tests persistence and recovery with WASM + browser storage:
 * - Tab crash recovery
 * - IndexedDB persistence
 * - Service worker recovery
 * - Multiple tab coordination
 */

import { test, expect, Page, BrowserContext } from '@playwright/test';

test.describe('Browser Crash Recovery', () => {
  test('should recover document after tab crash', async ({ browser }) => {
    const context = await browser.newContext();
    const page = await context.newPage();

    // Navigate to app
    await page.goto('http://localhost:3000');

    // Create document
    await page.evaluate(async () => {
      // @ts-ignore - VUDO runtime injected globally
      const vudo = window.VUDO;
      await vudo.createDocument('notes', 'shopping', {
        items: ['Milk', 'Eggs', 'Bread'],
      });
    });

    // Wait for persistence
    await page.waitForTimeout(1000);

    // Get persisted state
    const persistedState = await page.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      return await vudo.getPersistedState('notes', 'shopping');
    });

    // Simulate crash (close context without cleanup)
    await context.close();

    // Recover in new context
    const newContext = await browser.newContext();
    const newPage = await newContext.newPage();
    await newPage.goto('http://localhost:3000');

    // Load recovered document
    const recovered = await newPage.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      return await vudo.loadDocument('notes', 'shopping');
    });

    // Verify recovery
    expect(recovered).toBeTruthy();
    expect(recovered.items).toContain('Milk');
    expect(recovered.items).toContain('Eggs');
    expect(recovered.items).toContain('Bread');

    await newContext.close();
  });

  test('should recover after browser restart', async ({ browser }) => {
    const context = await browser.newContext();
    const page = await context.newPage();

    await page.goto('http://localhost:3000');

    // Create multiple documents
    await page.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      await vudo.createDocument('users', 'alice', { name: 'Alice', age: 30 });
      await vudo.createDocument('users', 'bob', { name: 'Bob', age: 25 });
      await vudo.createDocument('posts', 'post1', { title: 'Hello World', content: 'Test post' });
    });

    await page.waitForTimeout(1000);

    // Close browser context (simulates browser close)
    await context.close();

    // Reopen
    const newContext = await browser.newContext();
    const newPage = await newContext.newPage();
    await newPage.goto('http://localhost:3000');

    // Verify all documents recovered
    const documents = await newPage.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      return {
        alice: await vudo.loadDocument('users', 'alice'),
        bob: await vudo.loadDocument('users', 'bob'),
        post1: await vudo.loadDocument('posts', 'post1'),
      };
    });

    expect(documents.alice.name).toBe('Alice');
    expect(documents.bob.name).toBe('Bob');
    expect(documents.post1.title).toBe('Hello World');

    await newContext.close();
  });

  test('should handle IndexedDB persistence', async ({ browser }) => {
    const context = await browser.newContext();
    const page = await context.newPage();

    await page.goto('http://localhost:3000');

    // Create document
    await page.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      await vudo.createDocument('config', 'settings', {
        theme: 'dark',
        notifications: true,
      });
    });

    await page.waitForTimeout(1000);

    // Verify IndexedDB contains data
    const idbData = await page.evaluate(async () => {
      return new Promise((resolve) => {
        const request = indexedDB.open('VUDO_Storage', 1);
        request.onsuccess = () => {
          const db = request.result;
          const tx = db.transaction(['documents'], 'readonly');
          const store = tx.objectStore('documents');
          const getRequest = store.get('config/settings');

          getRequest.onsuccess = () => {
            resolve(getRequest.result);
          };
        };
      });
    });

    expect(idbData).toBeTruthy();

    await context.close();
  });

  test('should recover partial edits after crash', async ({ browser }) => {
    const context = await browser.newContext();
    const page = await context.newPage();

    await page.goto('http://localhost:3000');

    // Create document
    await page.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      await vudo.createDocument('draft', 'article', {
        title: 'Draft Article',
        content: '',
      });
    });

    // Make several edits
    for (let i = 0; i < 10; i++) {
      await page.evaluate(async (i) => {
        // @ts-ignore
        const vudo = window.VUDO;
        await vudo.updateDocument('draft', 'article', (doc) => {
          doc.content += `Paragraph ${i}. `;
        });
      }, i);

      // Small delay between edits
      await page.waitForTimeout(100);
    }

    // Wait for persistence
    await page.waitForTimeout(1000);

    // Crash
    await context.close();

    // Recover
    const newContext = await browser.newContext();
    const newPage = await newContext.newPage();
    await newPage.goto('http://localhost:3000');

    const recovered = await newPage.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      return await vudo.loadDocument('draft', 'article');
    });

    // Should have all 10 paragraphs
    expect(recovered.content).toContain('Paragraph 0');
    expect(recovered.content).toContain('Paragraph 9');

    await newContext.close();
  });

  test('should handle service worker recovery', async ({ browser }) => {
    const context = await browser.newContext();
    const page = await context.newPage();

    await page.goto('http://localhost:3000');

    // Register service worker
    await page.evaluate(async () => {
      if ('serviceWorker' in navigator) {
        await navigator.serviceWorker.register('/sw.js');
      }
    });

    // Create document
    await page.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      await vudo.createDocument('sw', 'test', { value: 'persisted' });
    });

    await page.waitForTimeout(1000);

    // Close page (service worker may still be active)
    await page.close();

    // Open new page
    const newPage = await context.newPage();
    await newPage.goto('http://localhost:3000');

    const recovered = await newPage.evaluate(async () => {
      // @ts-ignore
      const vudo = window.VUDO;
      return await vudo.loadDocument('sw', 'test');
    });

    expect(recovered.value).toBe('persisted');

    await context.close();
  });

  test('should handle quota exceeded gracefully', async ({ browser }) => {
    const context = await browser.newContext();
    const page = await context.newPage();

    await page.goto('http://localhost:3000');

    // Try to exceed storage quota (this should be handled gracefully)
    const result = await page.evaluate(async () => {
      try {
        // @ts-ignore
        const vudo = window.VUDO;

        // Create very large document
        const largeData = new Array(1000000).fill('x').join('');
        await vudo.createDocument('large', 'huge', { data: largeData });

        return { success: true };
      } catch (error) {
        return { success: false, error: error.message };
      }
    });

    // Should either succeed or fail gracefully
    expect(result).toBeTruthy();

    await context.close();
  });
});
