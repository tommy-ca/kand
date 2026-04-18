# Kand WebAssembly Bindings

This crate provides high-performance WebAssembly bindings for the `kand` technical analysis library. It is designed for zero-copy data transfer between JavaScript and Rust using shared memory views.

## Zero-Copy Usage (JavaScript)

The `WasmBuffer` class manages memory that can be shared between the host (JS) and the guest (WASM).

```javascript
import init, { WasmBuffer, sma } from './pkg/kand.js';

async function run() {
    await init();

    const len = 1000000; // 1 million points

    // 1. Allocate shared memory (8 bytes per f64)
    // 8 is the byte-size of Float64
    const inputBuffer = new WasmBuffer(len, 8);
    const outputBuffer = new WasmBuffer(len, 8);

    // 2. Create JS views into WASM memory
    // This allows JS to write directly into WASM's heap
    const inputView = new Float64Array(
        WebAssembly.Memory.buffer,
        inputBuffer.ptr(),
        len
    );
    const outputView = new Float64Array(
        WebAssembly.Memory.buffer,
        outputBuffer.ptr(),
        len
    );

    // 3. Populate input data (No copy!)
    for (let i = 0; i < len; i++) {
        inputView[i] = Math.random();
    }

    // 4. Call WASM indicator
    // Pass references to the managed buffers
    sma(inputBuffer, 14, outputBuffer);

    // 5. Results are immediately available in outputView (No copy!)
    console.log(outputView.slice(0, 20));

    // 6. Cleanup
    inputBuffer.free();
    outputBuffer.free();
}
```

## Generic Type Support
The `WasmBuffer` supports `f64`, `i32`, and `i64` by specifying the correct `element_size` during construction:
- `8`: for `f64` (Standard Price Data)
- `4`: for `i32` (Pattern Signals)
- `8`: for `i64` (Extended Precision Signals)

## Warning on Memory Growth
If a WASM call causes the memory to grow, existing JavaScript `TypedArray` views (like `Float64Array`) will become **detached** and must be re-created. Always re-instantiate your views if you perform allocations between WASM calls.
