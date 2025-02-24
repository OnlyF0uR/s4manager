# S4Manager

Provides essential functionality for creating and maintaining Sims 4 Mods. Requires Python 3.7.x and uncompyle6 (for decompilation).

### Features

- [x] Compiling Mods (Async)
- [x] Decompilation of standard scripts (Async)
- [x] Decompilation of mods (Async)

### Example

If you want to compile both the example Mod and this Manager use the following:

```bash
cargo run -- compile example_mod
```

If you want to use a binary, then donwload the binary from releases, add it to path and run:

```bash
s4m compile example_mod
```
