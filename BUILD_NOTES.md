# Build Notes - Live Server RS

## GitHub Actions Workflow Changes

The workflow was refined to avoid cross-compilation pitfalls and to make artifacts and caches deterministic across targets. This section documents the rationale and the exact behavior so you can reproduce it locally or adjust it safely.

### Key Improvements

1. **Use `cross` for Linux cross-compilation**
   - ARM targets (`aarch64`, `armv7`) and `i686` on Linux now use `cross` instead of native `cargo` builds on the CI host.
   - This prevents host/target architecture mismatches and toolchain incompatibilities when building multiple targets on the same runner.

2. **Per-target cache keys**
   - Each target uses its own cache key for Cargo registry/git index and build output.
   - This avoids mixing artifacts built for different architectures, which can cause hard-to-debug build failures.

3. **Tests only on native x86_64**
   - Tests run only on the native x86_64 targets for Linux, Windows, and macOS.
   - Cross-compiled targets skip tests because they cannot execute on the host runner.

4. **Unique artifact names**
   - Each build produces a uniquely named artifact to avoid collisions and to make downloads explicit.
   - Examples:
     - `live_server_rs-linux-x86_64`
     - `live_server_rs-linux-aarch64`
     - `live_server_rs-linux-armv7`
     - `live_server_rs-linux-i686`
     - `live_server_rs-windows-x86_64.exe`
     - `live_server_rs-windows-i686.exe`
     - `live_server_rs-windows-aarch64.exe`
     - `live_server_rs-macos-x86_64`
     - `live_server_rs-macos-aarch64`

### Build Matrix

| OS | Target | Method | Type |
|----|--------|--------|------|
| ubuntu-latest | x86_64-unknown-linux-gnu | cargo | native |
| ubuntu-latest | aarch64-unknown-linux-gnu | cross | cross-compile |
| ubuntu-latest | armv7-unknown-linux-gnueabihf | cross | cross-compile |
| ubuntu-latest | i686-unknown-linux-gnu | cross | cross-compile |
| windows-latest | x86_64-pc-windows-msvc | cargo | native |
| windows-latest | i686-pc-windows-msvc | cargo | native |
| windows-latest | aarch64-pc-windows-msvc | cargo | native |
| macos-latest | x86_64-apple-darwin | cargo | native |
| macos-latest | aarch64-apple-darwin | cargo | native |

### Automated Release

When you create and push a tag (for example `v1.0.0`), the workflow:

1. Builds all nine binaries defined in the matrix.
2. Creates a GitHub Release for the tag.
3. Uploads all binaries as release assets.

Example:

```bash
git tag v1.0.0
git push origin v1.0.0
```

### Local Cross-Compilation

To reproduce the Linux cross targets locally, install `cross` and build the specific targets:

```bash
# Install cross
cargo install cross

# Build for aarch64
cross build --release --target aarch64-unknown-linux-gnu

# Build for armv7
cross build --release --target armv7-unknown-linux-gnueabihf

# Build for i686
cross build --release --target i686-unknown-linux-gnu
```

### Compatibility Notes

- **Linux x86_64**: Suitable for most modern distributions.
- **Linux aarch64**: Targets 64-bit ARM devices (for example, Raspberry Pi 4 running a 64-bit OS).
- **Linux armv7**: Targets 32-bit ARM devices (for example, Raspberry Pi 3).
- **Linux i686**: Targets legacy 32-bit Intel/AMD systems.
- **Windows**: Built with the MSVC toolchain and requires the MSVC runtime.
- **macOS**: Supports both Intel and Apple Silicon (M1/M2/M3) systems.
