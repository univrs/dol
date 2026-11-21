# @vudo/runtime Plan Comparison

> **Planning document (historical).** Status claims here are not authoritative. See [docs/DOL-STATUS-2026-11.md](../DOL-STATUS-2026-11.md).

## Side-by-Side Analysis

| Aspect | My Spec | Your Plan | Verdict |
|--------|---------|-----------|---------|
| **Naming** | Generic (SpiritLoader, SpiritInstance) | VUDO vocabulary (Spirit, Séance, Loa) | ✅ **Your plan** - consistent with HyphalNetwork |
| **Scope** | Phases 1-2 (load + memory) | Phases 1-4 (full ecosystem) | ✅ **Your plan** - more complete vision |
| **Session mgmt** | Not included | Séance class | ✅ **Your plan** - essential for Year 2 |
| **Service registry** | Basic host functions | Loa + LoaRegistry | ✅ **Your plan** - extensible |
| **Memory detail** | BumpAllocator, encode/decode | MemoryManager class | 🤝 **Similar** - can merge |
| **Type bridge** | GeneLayout interface | Generated from dol-codegen | ✅ **Your plan** - leverages existing tooling |
| **Ghost concept** | Not included | Spirit composition | ✅ **Your plan** - needed for complex apps |
| **API ergonomics** | spirit.call('name', ...args) | spirit.as<T>() + typed calls | ✅ **Your plan** - better DX |
| **Implementation detail** | More code examples | More architecture focus | 🤝 **Complementary** |

## Key Differences

### 1. Vocabulary Alignment

**My spec:**
```typescript
const loader = new SpiritLoader();
const instance = await loader.load(wasmBytes);
const result = instance.call<number>('add', 1, 2);
```

**Your plan:**
```typescript
const spirit = await loadSpirit(wasmBytes);
const calc = spirit.as<Calculator>();
const result = await calc.add(1, 2);  // Type-safe!
```

**Rationale:** Your plan's vocabulary matches the VUDO OS vision (HyphalNetwork.md) and provides better developer ergonomics with the `as<T>()` pattern for type-safe calls.

### 2. Session Management (Séance)

**My spec:** Not included - focused only on single Spirit loading.

**Your plan:**
```typescript
const seance = new Seance();
await seance.summon('calc', '/spirits/calculator.wasm');
await seance.summon('logger', '/spirits/logger.wasm');
await seance.invoke('calc', 'add', [5, 3]);
await seance.dismiss();
```

**Rationale:** Séance is essential for:
- Multi-Spirit applications
- Shared state management
- Resource cleanup
- Year 2 collaborative features

### 3. Service Injection (Loa)

**My spec:** Hardcoded host functions
```typescript
const imports = {
  vudo_print: (ptr, len) => { ... },
  vudo_alloc: (size) => { ... },
};
```

**Your plan:** Pluggable service registry
```typescript
const logLoa: Loa = {
  name: 'logging',
  capabilities: ['log', 'error', 'debug'],
  provides: (imports) => ({ ... }),
};
registry.register(logLoa);
```

**Rationale:** Loa pattern enables:
- User-defined host capabilities
- Capability-based security
- Service versioning
- Clean separation of concerns

### 4. Memory Management

**My spec:** Low-level detail
```typescript
export class BumpAllocator {
  alloc(size: number, align = 8): number { ... }
  reset(): void { ... }
}

export function writeGene(memory, ptr, layout, values) { ... }
export function readGene(memory, ptr, layout) { ... }
```

**Your plan:** Higher-level abstraction
```typescript
const memory = new MemoryManager(spirit.memory);
const point = memory.readGene<Point>(ptr, PointLayout);
memory.writeGene(ptr, { x: 10, y: 20 }, PointLayout);
```

**Rationale:** Both are similar - my spec has more implementation detail, yours has cleaner API. **Merge recommended.**

## Recommended Merged Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         @vudo/runtime                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌────────────┐  │
│  │   Spirit    │  │   Séance    │  │    Loa      │  │  Memory    │  │
│  │   (loader)  │  │ (sessions)  │  │ (services)  │  │  Manager   │  │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └─────┬──────┘  │
│         │                │                │               │          │
│         │    ┌───────────┴────────────────┴───────────────┘          │
│         │    │                                                       │
│         ▼    ▼                                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    Internal Utilities                        │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │    │
│  │  │ BumpAllocator │  │ TypeBridge   │  │ WasmLoader   │       │    │
│  │  │ (from my spec)│  │ (gene r/w)   │  │ (fetch/load) │       │    │
│  │  └──────────────┘  └──────────────┘  └──────────────┘       │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Merged API Design

```typescript
// ===== Spirit Loading =====
import { Spirit, loadSpirit } from '@vudo/runtime';

// Simple loading
const spirit = await loadSpirit('/spirits/math.wasm');

// With options (from my spec's detail)
const spirit = await loadSpirit(wasmBytes, {
  memory: { initial: 16, maximum: 256 },
  debug: true,
});

// Type-safe calls (your plan's ergonomics)
import type { Math } from './generated/math.types';
const math = spirit.as<Math>();
const sum = await math.add(1n, 2n);  // bigint for i64


// ===== Session Management =====
import { Seance } from '@vudo/runtime';

const seance = new Seance();
await seance.summon('calc', '/spirits/calculator.wasm');
await seance.summon('log', '/spirits/logger.wasm');

// Cross-Spirit interaction
const result = await seance.invoke('calc', 'multiply', [6, 7]);
await seance.invoke('log', 'info', [`Result: ${result}`]);

await seance.dismiss();


// ===== Service Registry =====
import { Loa, LoaRegistry } from '@vudo/runtime';

// Define custom Loa
const httpLoa: Loa = {
  name: 'http',
  version: '1.0.0',
  capabilities: ['fetch', 'post'],
  provides: () => ({
    fetch: async (url: string) => { /* ... */ },
    post: async (url: string, body: string) => { /* ... */ },
  }),
};

const registry = new LoaRegistry();
registry.register(httpLoa);

// Spirits can request capabilities
const spirit = await loadSpirit(wasmBytes, { loas: registry });


// ===== Memory Management =====
import { MemoryManager, type GeneLayout } from '@vudo/runtime';

// Using generated layout from dol-codegen
import { PointLayout } from './generated/math.types';

const memory = spirit.memory;  // MemoryManager instance

// Allocate and write
const ptr = memory.alloc(PointLayout.size);
memory.writeGene(ptr, { x: 10n, y: 20n }, PointLayout);

// Call function with pointer
const distance = await spirit.call('distance', [ptr, otherPtr]);

// Read back
const point = memory.readGene<Point>(ptr, PointLayout);
memory.free(ptr);
```

## Implementation Priority (Merged)

| Week | Focus | From My Spec | From Your Plan |
|------|-------|--------------|----------------|
| **1** | Core Loading | WASM instantiation, host functions | Spirit class API, error handling |
| **2** | Memory | BumpAllocator, encode/decode | MemoryManager interface, GeneLayout |
| **3** | Sessions | — | Séance, multi-Spirit coordination |
| **4** | Services | — | Loa, LoaRegistry, capability injection |
| **5** | DX | — | CLI, browser bundle, npm publish |

## Recommendation

**Use your VUDO-RUNTIME-PLAN.md as the primary architecture** with these additions from my spec:

1. **BumpAllocator implementation** - Internal detail for memory.alloc()
2. **Host function implementations** - vudo_print, vudo_now, vudo_random
3. **WASM opcode reference** - For debugging and documentation
4. **Detailed test cases** - Memory read/write edge cases

## Files to Merge

| From My Spec | Into Your Plan's |
|--------------|------------------|
| `src/memory/allocator.ts` (BumpAllocator) | `src/memory.ts` (internal) |
| `src/memory/strings.ts` | `src/utils/type-bridge.ts` |
| `src/host/imports.ts` | `src/loa.ts` (as default Loa) |
| Test cases | `tests/` |

## Conclusion

Your plan is **more complete and better aligned with VUDO OS vision**. My spec adds **implementation detail** that can be incorporated as internal utilities. The merged approach gives you:

- ✅ Consistent VUDO vocabulary (Spirit, Séance, Loa)
- ✅ Type-safe API with `spirit.as<T>()`
- ✅ Session management for multi-Spirit apps
- ✅ Extensible service registry
- ✅ Solid memory management internals
- ✅ Clear 4-week implementation path

**Verdict: Proceed with your VUDO-RUNTIME-PLAN.md, incorporate my implementation details as internal utilities.**
