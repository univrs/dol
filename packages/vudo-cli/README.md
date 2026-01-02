# @vudo/cli

Command-line tool for running and inspecting DOL WASM modules.

## Installation

```bash
npm install -g @vudo/cli
# or
pnpm add -g @vudo/cli
```

## Commands

### `vudo run`

Execute a DOL WASM module.

```bash
vudo run <file.wasm> [function] [args...]
```

**Options:**
- `-d, --debug` - Enable debug output
- `-m, --memory <pages>` - Initial memory pages (default: 16)
- `-M, --max-memory <pages>` - Maximum memory pages (default: 256)
- `--json` - Output result as JSON

**Examples:**

```bash
# Run the main function
vudo run calculator.wasm

# Call a specific function with arguments
vudo run calculator.wasm add 10n 20n

# Enable debug output
vudo run -d calculator.wasm multiply 5n 7n

# Output as JSON
vudo run --json calculator.wasm get_config
```

**Argument Parsing:**
- `123` - Integer (converted to Number or BigInt for large values)
- `123n` - BigInt (explicit)
- `3.14` - Float
- `true/false` - Boolean
- `anything else` - String

### `vudo inspect`

Inspect a DOL WASM module.

```bash
vudo inspect <file.wasm>
```

**Options:**
- `--exports` - List exported functions only
- `--imports` - List required imports only
- `--memory` - Show memory configuration
- `--json` - Output as JSON

**Examples:**

```bash
# Show all module information
vudo inspect calculator.wasm

# List only exports
vudo inspect --exports calculator.wasm

# Get JSON output for scripting
vudo inspect --json calculator.wasm
```

## Usage with @vudo/runtime

The CLI uses [@vudo/runtime](../vudo-runtime) under the hood. For programmatic usage:

```typescript
import { loadSpirit } from '@vudo/runtime';

const spirit = await loadSpirit('./calculator.wasm');
const result = spirit.call<bigint>('add', [10n, 20n]);
console.log(result); // 30n
```

## Requirements

- Node.js 18+
- WASM modules compiled from DOL source

## License

MIT
