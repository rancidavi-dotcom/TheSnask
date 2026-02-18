# Creating Libraries in Snask (without changing the compiler)

This guide explains how to create and share libraries written **100% in Snask** (`.snask` files) using `import` and the `module::function()` namespace — **without modifying the compiler source**.

## Library layout (recommended)
Required files:
- `package.json` — metadata (name, version, description, etc.)
- `package.snask` — the library code
- `README.md` — documentation for developers

## Usage in an app
```snask
import "your_lib";

class main
    fun start()
        your_lib::hello();
```

## Import-only native APIs
If a library uses low-level native entrypoints (runtime/LLVM builtins), those names are reserved and cannot be called directly from user apps. Wrap them in a `.snask` package and expose a clean `lib::...` API.

## Publishing (workflow)
Recommended workflow:
1. Create your library repo (or a fork).
2. Include the required files above.
3. Submit a PR to the registry repository.

