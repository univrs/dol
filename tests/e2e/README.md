# VUDO Runtime E2E Tests

End-to-end browser tests for VUDO Runtime local-first synchronization.

## Prerequisites

```bash
cd tests/e2e
npm install
npx playwright install
```

## Running Tests

```bash
# Run all E2E tests
npm test

# Run specific test suite
npm run test:crash         # Crash recovery tests
npm run test:multi-tab     # Multi-tab sync tests
npm run test:offline       # Offline/online tests

# Run with UI
npm run test:ui

# Debug mode
npm run test:debug

# Run in headed mode (see browser)
npm run test:headed
```

## Test Categories

### Crash Recovery (`crash_recovery.test.ts`)

Tests browser crash recovery and persistence:
- Tab crash recovery from IndexedDB
- Browser restart recovery
- Partial edit recovery
- Service worker recovery
- Storage quota handling

### Multi-Tab Sync (`multi_tab_sync.test.ts`)

Tests cross-tab synchronization:
- BroadcastChannel communication
- SharedWorker coordination
- Concurrent edits across tabs
- Tab closing/opening
- Private document isolation

### Offline/Online (`offline_online.test.ts`)

Tests offline-first workflows in browser:
- navigator.onLine detection
- Offline queue persistence
- Background sync API
- Network event handling
- Remote peer sync after reconnection

## Configuration

Edit `playwright.config.ts` to:
- Change test directory
- Configure browsers (Chrome, Firefox, Safari, Mobile)
- Set retry logic
- Configure reporters
- Add web server

## Test Architecture

```
tests/e2e/
├── browser-sync/           # Test suites
│   ├── crash_recovery.test.ts
│   ├── multi_tab_sync.test.ts
│   └── offline_online.test.ts
├── playwright.config.ts    # Playwright configuration
├── package.json            # Dependencies
└── README.md               # This file
```

## CI Integration

Tests run in CI with:
- Retry on failure (2 retries)
- Video recording on failure
- HTML and JSON reports
- All browsers (Chromium, Firefox, WebKit)

## Debugging

```bash
# Debug specific test
npx playwright test crash_recovery --debug

# Show test report
npx playwright show-report

# Open trace viewer
npx playwright show-trace trace.zip
```

## Requirements

- Node.js 18+
- VUDO Runtime development server running on http://localhost:3000
- Playwright browsers installed

## Performance Targets

- Crash recovery: < 2s
- Cross-tab sync: < 500ms
- Offline queue drain: < 5s for 100 operations
- 10 concurrent tabs: stable operation
