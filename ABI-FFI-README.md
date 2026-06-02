<!--
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: 2025-2026 Jonathan D.A. Jewell (hyperpolymath)
-->

# echidnabot ABI/FFI

This document describes the **ABI** (Application Binary Interface) and **FFI**
(Foreign Function Interface) layers of echidnabot, in line with the
[Hyperpolymath estate convention](https://github.com/hyperpolymath/standards):

> **Zig = APIs + FFIs. Idris2 = ABIs.**

The Idris2 layer defines the binary contract (types, layout, foreign
declarations); the Zig layer is the C-ABI implementation that any other
language can link against.

---

## Current Status

The ABI/FFI surface in this repository is a **scaffold**, not a shipped
artefact. The skeleton is in place — what is missing is:

1. **Idris2 ABI definitions** (`src/abi/Types.idr`, `Layout.idr`,
   `Foreign.idr`) exist but are not yet wired to Rust types in `src/`.
   They will define the binary contract for the proof-job lifecycle
   (`ProofJob`, `JobResult`, `ProverKind`, `Verdict`).
2. **Zig FFI implementation** (`ffi/zig/src/main.zig`,
   `ffi/zig/build.zig`) is a **template scaffold** carrying
   `{{project}}` placeholders. It will not build until those are
   substituted to `echidnabot` and the export functions are realised
   against the real `ProofJob` API.
3. **Generated C header** (`generated/abi/echidnabot.h`) does not yet
   exist; will be produced by `idris2 --cg c-header` once the ABI is
   sealed.

Track this work via the [ABI/FFI epic](https://github.com/hyperpolymath/echidnabot/issues?q=is%3Aissue+label%3Aabi-ffi)
on the issue tracker. The intended end-state is what this document
describes; the current code reflects scaffold-only readiness.

---

## Intended Architecture

```
+---------------------------------------------+
|  ABI Definitions (Idris2)                   |
|  src/abi/                                   |
|  - Types.idr      (binary type definitions) |
|  - Layout.idr     (memory-layout proofs)    |
|  - Foreign.idr    (FFI function declarations)|
+----------------------+----------------------+
                       |
                       | generates (compile-time)
                       v
+---------------------------------------------+
|  C Headers (generated)                      |
|  generated/abi/echidnabot.h                 |
+----------------------+----------------------+
                       |
                       | imported by
                       v
+---------------------------------------------+
|  FFI Implementation (Zig)                   |
|  ffi/zig/src/main.zig                       |
|  - Implements C-compatible functions        |
|  - Memory-safe by default                   |
+----------------------+----------------------+
                       |
                       | compiled to libechidnabot.{so,dylib,a}
                       v
+---------------------------------------------+
|  Any Language via C ABI                     |
|  Rust, AffineScript, Julia, OCaml, ...      |
+---------------------------------------------+
```

Why this split? See the
[estate boundary memo](https://github.com/hyperpolymath/standards/blob/main/docs/boundary-conventions.md):
Idris2 is the only language in the estate toolchain that can carry the
*proofs* of layout compatibility, alignment, and forward-compatibility
that an ABI demands. Zig is the only language that can emit a clean C
ABI **without dragging in a runtime** — the resulting `.so` has no
hidden Rust or libc surprises.

---

## Directory Structure

```
echidnabot/
├── src/abi/                         # ABI definitions (Idris2)
│   ├── Types.idr                    # Core type definitions with proofs
│   ├── Layout.idr                   # Memory-layout verification
│   └── Foreign.idr                  # FFI function declarations
│
├── ffi/zig/                         # FFI implementation (Zig)
│   ├── build.zig                    # Build configuration (template, see above)
│   ├── src/
│   │   └── main.zig                 # C-compatible FFI implementation (template)
│   └── test/
│       └── integration_test.zig     # FFI integration tests
│
└── generated/                       # (not yet present)
    └── abi/
        └── echidnabot.h             # Generated C header from src/abi/
```

---

## Why Idris2 for ABI?

### Formal verification

Idris2's dependent types let us prove ABI properties at compile time:

```idris
-- Prove struct size is correct
public export
proofJobSize : HasSize ProofJob 64

-- Prove field alignment is correct
public export
priorityAligned : Divides 4 (offsetOf ProofJob.priority)

-- Prove backwards compatibility
public export
abiCompatible : Compatible (ABI 0) (ABI 1)
```

### Type-level invariants

Encode invariants C and Zig cannot express:

```idris
-- Non-null pointer guaranteed at type level
data Handle : Type where
  MkHandle : (ptr : Bits64) -> {auto 0 nonNull : So (ptr /= 0)} -> Handle

-- Buffer with length proof
data Buffer : (n : Nat) -> Type where
  MkBuffer : Vect n Byte -> Buffer n
```

### Platform abstraction

```idris
CInt : Platform -> Type
CInt Linux   = Bits32
CInt Windows = Bits32

CSize : Platform -> Type
CSize Linux   = Bits64
CSize Windows = Bits64
```

### Safe evolution

```idris
-- Compiler enforces compatibility
abiUpgrade : ABI 0 -> ABI 1
abiUpgrade old = MkABI1 {
  v0_compat    = old,
  new_features = defaults
}
```

---

## Why Zig for FFI?

### C ABI compatibility

Zig exports C-compatible functions naturally:

```zig
export fn echidnabot_init() ?*Handle {
    // ...
}
```

### Memory safety

Compile-time safety without runtime overhead:

```zig
const handle = init() orelse return error.InitFailed;
defer free(handle);
```

### Cross-compilation built in

```bash
zig build -Dtarget=x86_64-linux-gnu
zig build -Dtarget=aarch64-macos-none
zig build -Dtarget=x86_64-windows-gnu
```

### No runtime dependency

Zig only includes what you `@import`. The resulting `.so` is the smallest
possible footprint that still honours the C ABI.

---

## Intended Building (when scaffold is realised)

### Build the FFI library

```bash
cd ffi/zig
zig build                              # debug
zig build -Doptimize=ReleaseFast       # optimised
zig build test                         # unit tests
zig build test-integration             # integration tests
```

### Generate the C header from the Idris2 ABI

```bash
cd src/abi
idris2 --cg c-header Types.idr -o ../../generated/abi/echidnabot.h
```

### Cross-compile

```bash
cd ffi/zig
zig build -Dtarget=x86_64-linux-gnu      # Linux x86_64
zig build -Dtarget=aarch64-macos-none    # macOS ARM64
zig build -Dtarget=x86_64-windows-gnu    # Windows x86_64
```

---

## Intended Usage (post-scaffold)

### From C

```c
#include "echidnabot.h"

int main(void) {
    echidnabot_handle_t* h = echidnabot_init();
    if (!h) return 1;

    echidnabot_result_t r = echidnabot_dispatch(h, "coq", "proofs/foo.v");
    if (r != ECHIDNABOT_OK) {
        const char* err = echidnabot_last_error();
        fprintf(stderr, "error: %s\n", err);
    }

    echidnabot_free(h);
    return 0;
}
```

Compile:

```bash
gcc example.c -lechidnabot -L./zig-out/lib -o example
```

### From Idris2

```idris
import Echidnabot.ABI.Foreign

main : IO ()
main = do
  Just h <- init
    | Nothing => putStrLn "failed to initialise"
  Right result <- dispatch h "coq" "proofs/foo.v"
    | Left err => putStrLn $ "error: " ++ errorDescription err
  free h
```

### From Rust

```rust
#[link(name = "echidnabot")]
extern "C" {
    fn echidnabot_init() -> *mut std::ffi::c_void;
    fn echidnabot_free(handle: *mut std::ffi::c_void);
    fn echidnabot_dispatch(
        handle: *mut std::ffi::c_void,
        prover: *const std::os::raw::c_char,
        path:   *const std::os::raw::c_char,
    ) -> i32;
}
```

### From Julia

```julia
const libechidnabot = "libechidnabot"

function init()
    h = ccall((:echidnabot_init, libechidnabot), Ptr{Cvoid}, ())
    h == C_NULL && error("init failed")
    h
end

function dispatch(h, prover::String, path::String)
    ccall((:echidnabot_dispatch, libechidnabot), Cint,
          (Ptr{Cvoid}, Cstring, Cstring), h, prover, path)
end
```

### From AffineScript

Once the AffineScript FFI surface lands, the call shape will match the
`extern "C"` block above via the standard
`affine.ffi.bind_c` helper. See the
[affinescript bindings tracker](https://github.com/hyperpolymath/affinescript/issues/446).

---

## Testing

### Zig unit tests

```bash
cd ffi/zig && zig build test
```

### Integration tests

```bash
cd ffi/zig && zig build test-integration
```

### Idris2 ABI verification

```idris
-- Compile-time verification (elaborator reflection)
%runElab verifyABI

-- Runtime smoke checks
main : IO ()
main = do
  verifyLayoutsCorrect
  verifyAlignmentsCorrect
  putStrLn "ABI verification passed"
```

---

## Contributing to ABI / FFI

When modifying the surface:

1. **Update the ABI first** (`src/abi/*.idr`)
   - Modify type definitions
   - Update layout proofs
   - Ensure backwards compatibility (`Compatible (ABI n) (ABI n+1)`)
2. **Regenerate the C header**
   ```bash
   idris2 --cg c-header src/abi/Types.idr -o generated/abi/echidnabot.h
   ```
3. **Update the FFI implementation** (`ffi/zig/src/main.zig`)
   - Implement / amend exported functions
   - Match ABI types exactly
4. **Add tests**
   - Zig unit + integration tests
   - Idris2 ABI verification
5. **Update this document** when the surface shape changes (function
   signatures, struct layout, ownership rules).

---

## License

This document and the surrounding scaffold are MPL-2.0, matching the
rest of echidnabot. See [`LICENSE`](LICENSE).

---

## See Also

- [Idris2 documentation](https://idris2.readthedocs.io)
- [Zig documentation](https://ziglang.org/documentation/master/)
- [Hyperpolymath standards](https://github.com/hyperpolymath/standards)
- [Estate boundary memo (Zig=APIs+FFIs, Idris2=ABIs)](https://github.com/hyperpolymath/standards/blob/main/docs/boundary-conventions.md)
