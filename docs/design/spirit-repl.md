# DOL Spirit REPL: Dynamic Compilation Architecture

## Vision

Transform VUDO from a static AST viewer into a **fully dynamic REPL** capable of:
1. **DOL → Rust → WASM** compilation pipeline
2. **Spirit loading** - modular runtime extensions
3. **MCP integration** - AI-assisted development
4. **Hot reloading** - instant feedback loop

```
┌─────────────────────────────────────────────────────────────────────┐
│                        VUDO Spirit REPL                              │
├─────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐             │
│  │   Editor    │───▶│  Compiler   │───▶│   Runtime   │             │
│  │   (DOL)     │    │  (Spirit)   │    │   (WASM)    │             │
│  └─────────────┘    └─────────────┘    └─────────────┘             │
│         │                  │                  │                     │
│         ▼                  ▼                  ▼                     │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐             │
│  │    AST      │    │    Rust     │    │   Output    │             │
│  │   Panel     │    │   Preview   │    │   Console   │             │
│  └─────────────┘    └─────────────┘    └─────────────┘             │
├─────────────────────────────────────────────────────────────────────┤
│  Spirit Loader  │  MCP Tools  │  Mod Registry  │  State Manager    │
└─────────────────────────────────────────────────────────────────────┘
```

## Current State vs Target State

| Feature | Current | Target |
|---------|---------|--------|
| DOL Parsing | ✅ metadol WASM | ✅ Keep |
| AST Display | ✅ Tree view | ✅ Enhanced |
| WASM Codegen | ⚠️ Basic (pure fns) | Full Spirit codegen |
| Execution | ⚠️ Simulated | Real WASM execution |
| Hot Reload | ❌ Manual compile | Auto-compile on change |
| Mod Loading | ❌ None | Spirit registry |
| Cloud Compile | ❌ None | Rust→WASM API |
| MCP Integration | ❌ None | Tool ecosystem |

## Architecture: The Spirit REPL

### 1. Compilation Pipeline

```
DOL Source
    │
    ▼
┌─────────────────┐
│  Stage 1: Parse │  ← metadol (existing WASM)
│  DOL → AST      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Stage 2: Lower │  ← NEW: AST → Rust IR
│  AST → Rust     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Stage 3: Compile│  ← Cloud API or local rustc
│  Rust → WASM    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Stage 4: Load  │  ← Spirit instantiation
│  WASM → Spirit  │
└─────────────────┘
```

### 2. Spirit Module System

A **Spirit** is the fundamental unit of DOL runtime:

```typescript
interface Spirit {
  // Identity
  id: string;
  name: string;
  version: string;

  // WASM Module
  module: WebAssembly.Module;
  instance: WebAssembly.Instance;

  // Exports
  exports: SpiritExports;

  // Lifecycle
  init(): Promise<void>;
  destroy(): void;

  // Hot reload
  reload(newModule: WebAssembly.Module): Promise<void>;
}

interface SpiritExports {
  // Functions exported from WASM
  functions: Map<string, Function>;

  // Memory access
  memory: WebAssembly.Memory;

  // Gene instances
  genes: Map<string, GeneInstance>;
}
```

### 3. Cloud Compilation API

For Rust → WASM compilation, we need a backend service:

```
┌─────────────────────────────────────────────────────────┐
│                 Compilation Service                      │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  POST /api/compile                                       │
│  ┌─────────────────────────────────────────────────┐    │
│  │ Request:                                         │    │
│  │ {                                                │    │
│  │   "source": "// DOL source or Rust IR",         │    │
│  │   "target": "wasm32-unknown-unknown",           │    │
│  │   "optimize": "release",                        │    │
│  │   "features": ["spirit-runtime"]                │    │
│  │ }                                                │    │
│  └─────────────────────────────────────────────────┘    │
│                                                          │
│  ┌─────────────────────────────────────────────────┐    │
│  │ Response:                                        │    │
│  │ {                                                │    │
│  │   "success": true,                              │    │
│  │   "wasm": "base64-encoded-wasm",                │    │
│  │   "exports": ["add", "multiply", "main"],       │    │
│  │   "size": 1234,                                 │    │
│  │   "compile_time_ms": 150                        │    │
│  │ }                                                │    │
│  └─────────────────────────────────────────────────┘    │
│                                                          │
│  Implementation Options:                                 │
│  • Cloudflare Workers + wasm-pack                       │
│  • AWS Lambda with Rust toolchain                       │
│  • Self-hosted with Docker                              │
│  • WebContainer (Stackblitz-style)                      │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### 4. MCP Integration

Model Context Protocol enables AI-assisted development:

```typescript
// MCP Server: dol-repl-tools
const mcpTools = {
  // Compile DOL to WASM
  "dol.compile": {
    description: "Compile DOL source to WebAssembly",
    input: { source: "string", optimize: "boolean" },
    output: { wasm: "bytes", exports: "string[]" }
  },

  // Execute Spirit function
  "dol.execute": {
    description: "Execute a function in a loaded Spirit",
    input: { spirit: "string", function: "string", args: "any[]" },
    output: { result: "any", logs: "string[]" }
  },

  // Load Spirit mod
  "dol.loadMod": {
    description: "Load a Spirit module from registry",
    input: { name: "string", version: "string" },
    output: { spirit: "Spirit", exports: "string[]" }
  },

  // Analyze DOL code
  "dol.analyze": {
    description: "Analyze DOL code for types, errors, suggestions",
    input: { source: "string" },
    output: { ast: "object", diagnostics: "Diagnostic[]" }
  },

  // Generate DOL from description
  "dol.generate": {
    description: "Generate DOL code from natural language",
    input: { description: "string", context: "string" },
    output: { source: "string", explanation: "string" }
  }
};
```

### 5. Mod Registry

Spirits can be published and shared:

```
┌─────────────────────────────────────────────────────────┐
│                   Spirit Registry                        │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  registry.univrs.io/spirits                              │
│                                                          │
│  ┌─────────────────────────────────────────────────┐    │
│  │ @univrs/math           v1.0.0                   │    │
│  │ Pure mathematical functions                      │    │
│  │ Exports: add, subtract, multiply, divide, pow   │    │
│  └─────────────────────────────────────────────────┘    │
│                                                          │
│  ┌─────────────────────────────────────────────────┐    │
│  │ @univrs/crypto         v0.2.0                   │    │
│  │ Cryptographic primitives                         │    │
│  │ Exports: hash, sign, verify, encrypt, decrypt   │    │
│  └─────────────────────────────────────────────────┘    │
│                                                          │
│  ┌─────────────────────────────────────────────────┐    │
│  │ @univrs/json           v1.1.0                   │    │
│  │ JSON parsing and serialization                   │    │
│  │ Exports: parse, stringify, query                │    │
│  └─────────────────────────────────────────────────┘    │
│                                                          │
│  Spirit Manifest (spirit.toml):                          │
│  ┌─────────────────────────────────────────────────┐    │
│  │ [spirit]                                        │    │
│  │ name = "math"                                   │    │
│  │ version = "1.0.0"                               │    │
│  │ author = "univrs"                               │    │
│  │                                                  │    │
│  │ [exports]                                       │    │
│  │ functions = ["add", "multiply", "pow"]          │    │
│  │                                                  │    │
│  │ [dependencies]                                  │    │
│  │ # other spirits this depends on                 │    │
│  └─────────────────────────────────────────────────┘    │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

## Implementation Phases

### Phase 1: Enhanced WASM Execution (Week 1-2)

**Goal**: Execute real WASM in the browser, not just simulation

```typescript
// src/lib/spirit-runtime.ts
export class SpiritRuntime {
  private spirits: Map<string, Spirit> = new Map();
  private memory: WebAssembly.Memory;

  async loadWasm(wasm: Uint8Array): Promise<Spirit> {
    const module = await WebAssembly.compile(wasm);
    const imports = this.createImports();
    const instance = await WebAssembly.instantiate(module, imports);

    return this.registerSpirit(module, instance);
  }

  private createImports(): WebAssembly.Imports {
    return {
      env: {
        // Console I/O
        print_i64: (value: bigint) => console.log(value),
        print_str: (ptr: number, len: number) => {
          const bytes = new Uint8Array(this.memory.buffer, ptr, len);
          console.log(new TextDecoder().decode(bytes));
        },

        // Event emission
        emit_event: (namePtr: number, nameLen: number, dataPtr: number, dataLen: number) => {
          // Handle DOL emit statements
        },

        // Memory allocation
        alloc: (size: number) => { /* ... */ },
        dealloc: (ptr: number, size: number) => { /* ... */ },
      },

      // WASI preview for broader compatibility
      wasi_snapshot_preview1: {
        fd_write: () => 0,
        fd_close: () => 0,
        proc_exit: () => {},
      }
    };
  }
}
```

### Phase 2: DOL → Rust Transpilation (Week 3-4)

**Goal**: Generate valid Rust code from DOL AST

```rust
// In univrs-dol: src/codegen/rust.rs
pub struct RustCodegen {
    output: String,
    indent: usize,
}

impl RustCodegen {
    pub fn generate(&mut self, ast: &[Declaration]) -> String {
        self.emit_prelude();

        for decl in ast {
            match decl {
                Declaration::Function(f) => self.emit_function(f),
                Declaration::Gene(g) => self.emit_gene(g),
                Declaration::Trait(t) => self.emit_trait(t),
                _ => {}
            }
        }

        self.output.clone()
    }

    fn emit_function(&mut self, f: &FunctionDecl) {
        // DOL: fun add(a: Int64, b: Int64) -> Int64 { return a + b }
        // Rust: #[no_mangle] pub extern "C" fn add(a: i64, b: i64) -> i64 { a + b }

        let export_attr = if f.is_public {
            "#[no_mangle]\npub extern \"C\" "
        } else {
            ""
        };

        self.emit(&format!("{}fn {}(", export_attr, f.name));
        // ... params, return type, body
    }

    fn emit_gene(&mut self, g: &GeneDecl) {
        // DOL Gene → Rust struct
        self.emit(&format!("pub struct {} {{", g.name));
        for field in &g.fields {
            self.emit(&format!("    pub {}: {},", field.name, self.map_type(&field.ty)));
        }
        self.emit("}");

        // Generate impl block with methods
        self.emit(&format!("impl {} {{", g.name));
        // ... constructor, getters, setters
        self.emit("}");
    }
}
```

### Phase 3: Cloud Compilation Service (Week 5-6)

**Goal**: API endpoint for Rust → WASM compilation

```typescript
// Cloudflare Worker: compile-service
export default {
  async fetch(request: Request): Promise<Response> {
    if (request.method !== 'POST') {
      return new Response('Method not allowed', { status: 405 });
    }

    const { source, target, optimize } = await request.json();

    // Option A: Use pre-compiled wasm-pack in worker
    // Option B: Call out to build service
    // Option C: Use WebContainer API

    const result = await compileRustToWasm(source, {
      target: target || 'wasm32-unknown-unknown',
      optimize: optimize || 'release',
    });

    return Response.json({
      success: true,
      wasm: base64Encode(result.wasm),
      exports: result.exports,
      size: result.wasm.byteLength,
    });
  }
};
```

### Phase 4: Spirit Registry & Mods (Week 7-8)

**Goal**: Publish and load Spirit modules

```typescript
// Spirit loader in VUDO
class SpiritLoader {
  private registry = 'https://registry.univrs.io';
  private cache: Map<string, Spirit> = new Map();

  async load(name: string, version?: string): Promise<Spirit> {
    const cacheKey = `${name}@${version || 'latest'}`;

    if (this.cache.has(cacheKey)) {
      return this.cache.get(cacheKey)!;
    }

    // Fetch from registry
    const manifest = await this.fetchManifest(name, version);
    const wasm = await this.fetchWasm(manifest.wasmUrl);

    // Instantiate
    const spirit = await this.runtime.loadWasm(wasm);
    spirit.name = manifest.name;
    spirit.version = manifest.version;

    this.cache.set(cacheKey, spirit);
    return spirit;
  }

  async loadFromSource(dolSource: string): Promise<Spirit> {
    // Full pipeline: DOL → AST → Rust → WASM → Spirit
    const ast = await this.compiler.parse(dolSource);
    const rust = await this.compiler.toRust(ast);
    const wasm = await this.compileService.compile(rust);
    return this.runtime.loadWasm(wasm);
  }
}
```

### Phase 5: MCP Tools Server (Week 9-10)

**Goal**: Expose REPL capabilities via MCP

```typescript
// packages/mcp-dol-tools/src/server.ts
import { Server } from '@modelcontextprotocol/sdk/server';

const server = new Server({
  name: 'dol-repl-tools',
  version: '1.0.0',
});

server.setRequestHandler('tools/list', async () => ({
  tools: [
    {
      name: 'dol.compile',
      description: 'Compile DOL source code to WebAssembly',
      inputSchema: {
        type: 'object',
        properties: {
          source: { type: 'string', description: 'DOL source code' },
          optimize: { type: 'boolean', default: true },
        },
        required: ['source'],
      },
    },
    {
      name: 'dol.execute',
      description: 'Execute a function in the REPL',
      inputSchema: {
        type: 'object',
        properties: {
          function: { type: 'string' },
          args: { type: 'array' },
        },
        required: ['function'],
      },
    },
    // ... more tools
  ],
}));

server.setRequestHandler('tools/call', async (request) => {
  const { name, arguments: args } = request.params;

  switch (name) {
    case 'dol.compile':
      return await handleCompile(args);
    case 'dol.execute':
      return await handleExecute(args);
    // ...
  }
});
```

## VUDO UI Evolution

### Current UI
```
┌─────────────────────┬─────────────────────┐
│                     │                     │
│   Code Editor       │   Output Panel      │
│   (DOL)             │   (AST + Simulated) │
│                     │                     │
├─────────────────────┴─────────────────────┤
│              Status Bar                    │
└───────────────────────────────────────────┘
```

### Enhanced REPL UI
```
┌─────────────────────┬─────────────────────┐
│   Code Editor       │   Multi-Tab Output  │
│   (DOL)             │   ┌───┬───┬───┬───┐ │
│                     │   │AST│Rust│WASM│Run││
│   ┌───────────────┐ │   └───┴───┴───┴───┘ │
│   │ Autocomplete  │ │                     │
│   │ ─────────────│ │   ▶ Execution Log   │
│   │ add()        │ │   > add(40, 2)      │
│   │ multiply()   │ │   42                │
│   │ log_result() │ │   > multiply(6, 7)  │
│   └───────────────┘ │   42                │
├─────────────────────┼─────────────────────┤
│ Spirit Browser      │ REPL Console        │
│ ┌─────────────────┐ │ ┌─────────────────┐ │
│ │ @univrs/math    │ │ │ > add(1, 2)     │ │
│ │ @univrs/crypto  │ │ │ 3               │ │
│ │ [+ Load More]   │ │ │ > _             │ │
│ └─────────────────┘ │ └─────────────────┘ │
├─────────────────────┴─────────────────────┤
│ Status │ Spirits: 2 │ WASM: 1.2KB │ Ready │
└───────────────────────────────────────────┘
```

## Technology Stack

| Layer | Technology | Rationale |
|-------|------------|-----------|
| Editor | CodeMirror 6 | Extensible, WASM-friendly |
| Parser | metadol (WASM) | Already built, fast |
| Codegen | Rust (new) | DOL → Rust transpilation |
| Compile | Cloud API | Rust toolchain too heavy for browser |
| Runtime | WebAssembly | Native speed, sandboxed |
| UI | React + Tailwind | Current stack |
| Registry | Cloudflare R2 | CDN for WASM modules |
| MCP | TypeScript | AI integration |

## Security Considerations

1. **WASM Sandboxing**: All Spirit code runs in WASM sandbox
2. **Memory Limits**: Configurable memory caps per Spirit
3. **Execution Timeout**: Kill long-running computations
4. **Code Signing**: Verify Spirit authenticity from registry
5. **CSP Headers**: Strict content security policy

## Success Metrics

| Metric | Target |
|--------|--------|
| Parse → Execute | < 500ms |
| Cloud Compile | < 3s |
| Spirit Load | < 100ms (cached) |
| Memory per Spirit | < 1MB |
| Concurrent Spirits | 10+ |

## Open Questions

1. **Compilation backend**: Cloudflare Workers vs AWS Lambda vs WebContainer?
2. **Registry hosting**: Self-hosted vs npm-like service?
3. **Monetization**: Free tier + paid compilation credits?
4. **Offline mode**: Bundle common Spirits for offline use?

## Next Steps

1. [ ] Implement SpiritRuntime with real WASM execution
2. [ ] Build DOL → Rust codegen in univrs-dol
3. [ ] Set up cloud compilation API
4. [ ] Design Spirit manifest format
5. [ ] Create MCP tools package
6. [ ] Evolve VUDO UI with REPL console

---

*This design enables DOL to become a true runtime platform, where Spirits are first-class citizens that can be composed, shared, and executed anywhere WebAssembly runs.*
