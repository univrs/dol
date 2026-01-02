// test-hello.ts - Test string literals in WASM
import { readFile } from 'fs/promises';

async function main() {
    console.log('=== Testing String Literals in WASM ===\n');

    // Load the compiled WASM
    const wasmBytes = await readFile('./examples/hello_world.wasm');
    console.log(`Loaded ${wasmBytes.length} bytes of WASM`);

    // We need to store the actual memory reference
    let actualMemory: WebAssembly.Memory | null = null;

    // Create Loa imports (WASM imports from "loa" module)
    // The import function uses actualMemory which we set after instantiation
    const loa = {
        vudo_print: (ptr: number, len: number) => {
            console.log(`  [vudo_print called] ptr=${ptr}, len=${len}`);
            if (!actualMemory) {
                console.log('  ERROR: memory not available');
                return;
            }
            const bytes = new Uint8Array(actualMemory.buffer, ptr, len);
            console.log(`  [bytes] ${Array.from(bytes).join(', ')}`);
            const text = new TextDecoder().decode(bytes);
            console.log(`  [text] "${text}"`);
        }
    };

    // Build imports object with loa module
    const imports: WebAssembly.Imports = {
        loa
    };

    // Compile and instantiate WASM
    console.log('Compiling WASM...');
    let instance: WebAssembly.Instance;
    try {
        const module = await WebAssembly.compile(wasmBytes);
        instance = await WebAssembly.instantiate(module, imports);
    } catch (e: any) {
        console.error('Failed to load WASM:', e.message);
        return;
    }

    // Get the module's exported memory
    const exportedMemory = instance.exports.memory as WebAssembly.Memory | undefined;
    if (!exportedMemory) {
        console.error('No memory exported from WASM');
        return;
    }
    actualMemory = exportedMemory;
    console.log('Using exported memory');

    // Debug: Check data section content
    console.log('\n--- Checking data section at offset 1024 ---');
    const dataView = new Uint8Array(actualMemory.buffer, 1024, 20);
    console.log(`Bytes at 1024: ${Array.from(dataView).join(', ')}`);
    console.log(`Text at 1024: "${new TextDecoder().decode(dataView)}"`);

    console.log('WASM loaded successfully');
    console.log('Exports:', Object.keys(instance.exports));

    // Test add function
    console.log('\n--- Testing add(3, 4) ---');
    try {
        const add = instance.exports.add as (a: bigint, b: bigint) => bigint;
        const result = add(3n, 4n);
        console.log(`Result: ${result}`);
        console.log(`Expected: 7`);
        console.log(`Status: ${result === 7n ? 'PASS' : 'FAIL'}`);
    } catch (e: any) {
        console.log(`Error: ${e.message}`);
    }

    // Test greet function
    console.log('\n--- Testing greet() ---');
    try {
        const greet = instance.exports.greet as () => void;
        greet();
        console.log('greet() completed');
    } catch (e: any) {
        console.log(`Error: ${e.message}`);
    }

    console.log('\n=== Test Complete ===');
}

main().catch(console.error);
