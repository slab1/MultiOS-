# Rust Cross-Compilation Best Practices for x86_64, ARM64, and RISC‑V: Tooling, Targets, Build Systems, and Testing with QEMU

## Executive Summary

Cross-compilation is central to modern Rust workflows. It lets engineers build for multiple operating systems and architectures from a single host, integrate system libraries reliably, and validate behavior across heterogeneous environments without maintaining fleets of machines. In 2025, the Rust ecosystem offers mature, repeatable pathways to cross-compile for x86_64, ARM64 (AArch64), and RISC‑V. Two approaches dominate in practice: a container-first pipeline anchored by the cross tool, and a Cargo-first flow managed via rustup targets and explicit Cargo configuration.

- The container-first approach (cross-rs/cross) emphasizes reproducibility and “zero setup” builds by executing toolchains in prepared Docker or Podman images, handling system dependencies, and offering an experimental test runner that emulates foreign binaries via QEMU user-mode. It excels when your team wants hermetic, consistent builds and CI-friendly semantics across Linux, Windows, and ARM targets.[^6]

- The Cargo/rustup approach emphasizes native toolchains managed by rustup, explicit target additions via rustup target add, and Cargo’s target-specific configuration to specify linkers and flags. It shines for purely Rust code, Linux-to-Linux or Linux-to-Windows workflows, and when you need tight control over the toolchain and minimal layers between your code and the compiler.[^1]

Testing strategies hinge on the target. For std environments, cross test executes tests under QEMU user-mode emulation inside containers, trading some performance and edge-case fidelity for broad coverage. For bare-metal and no_std targets, QEMU system emulation remains the most practical way to run integration tests, boot kernels, and debug via GDB, drawing on established embedded workflows.[^10][^11]

Teams building multi-architecture operating systems and kernels in Rust should plan for both std and no_std realities: configure linkers, leverage Cross.toml to pin system libraries per target, consider custom target JSON when builtin targets are insufficient, and integrate QEMU both as a runner and a GDB server for debug sessions. This guide distills current best practices, shows how to wire rustup, Cargo, and cross together coherently, and provides a CI blueprint to make multi-arch releases routine rather than exceptional.[^4][^6][^9][^11]

Key recommendations:
- Prefer cross for containerized, reproducible builds and broad target coverage; adopt Cargo/rustup for lean, native workflows when you control dependencies tightly.[^1][^6]
- For macOS (Darwin) targets from Linux, use osxcross with a custom image and pin the SDK and toolchain versions; budget time for compatibility alignment.[^8][^12]
- For testing, use cross test for std targets and QEMU system emulation for no_std kernels; apply remote GDB debugging for deeper introspection.[^10][^11]
- Treat configuration as code: keep Cross.toml and Cargo target settings in the repo, pin toolchains via rust-toolchain.toml, and build reproducible CI pipelines with matrix strategies and controlled cache semantics.[^6][^9]

Finally, acknowledge known information gaps. QEMU user-mode reliability varies by architecture and target; macOS linking quirks evolve with Xcode and osxcross releases; CI cache tuning is project-specific; and custom target JSON needs nightly for build-std flows. Verify version notes and test your exact combination before locking your workflow.

[^1]: Cross-compilation - The rustup book.
[^6]: cross-rs/cross - “Zero setup” cross compilation and cross testing.
[^8]: Cross-Compiling Your Project in Rust - Tangram Visions Blog.
[^11]: The Embedded Rust Book: QEMU.
[^4]: The rustc book: Platform Support.

## Foundations: Rust Cross-Compilation Model and Target Semantics

Rust cross-compilation pivots on target triples of the form <arch>-<vendor>-<sys>-<abi> and on the Rust toolchain’s standard library components for those targets. rustup manages toolchains and installs target std components via rustup target add. Once installed, Cargo builds for the chosen target using --target, while the linker and system libraries are supplied either by the host environment (Cargo/rustup) or a container image (cross).[^1][^3]

Targets are grouped into tiers with different guarantees. Tier 1 platforms receive extensive testing and official builds; Tier 2 platforms “build” reliably with std available, sometimes without host tools; Tier 3 targets exist in the codebase but without official builds or automated testing guarantees. The tier you target influences both how much automation you can expect and the amount of bespoke setup required.[^4]

When the target you need is not directly supported, you can create a custom target specification (JSON) and—on nightly—compile core and alloc via Cargo’s build-std. This is common in embedded and OS-development contexts where you need precise control over atomics, panic strategy, linker flags, and data layout.[^13]

### Table: Rust target tiering quick-reference (x86_64, AArch64, RISC‑V)

To illustrate typical tier coverage, the following table summarizes commonly used triples and their tier classification as documented.

| Architecture | Example Target Triple                    | Tier Classification          | Notes |
|--------------|------------------------------------------|------------------------------|-------|
| x86_64       | x86_64-unknown-linux-gnu                 | Tier 1 with host tools       | Standard 64-bit Linux target.[^4] |
| x86_64       | x86_64-pc-windows-msvc                   | Tier 1 with host tools       | Standard 64-bit Windows MSVC target.[^4] |
| x86_64       | x86_64-unknown-linux-musl                | Tier 2 with host tools       | musl-based Linux target.[^4] |
| AArch64      | aarch64-unknown-linux-gnu                | Tier 1 with host tools       | Standard 64-bit ARM Linux target.[^4] |
| AArch64      | aarch64-apple-darwin                     | Tier 1 with host tools       | ARM64 macOS (11.0+).[^4] |
| AArch64      | aarch64-pc-windows-msvc                  | Tier 1 with host tools       | ARM64 Windows MSVC.[^4] |
| RISC‑V       | riscv64gc-unknown-linux-gnu              | Tier 2 with host tools       | RV64IMAFDC Linux (glibc).[^4] |
| RISC‑V       | riscv64gc-unknown-none-elf               | Tier 2 without host tools    | Bare-metal RV64IMAFDC.[^4] |

This classification guides expectations: Tier 1 targets have the smoothest path and the richest tooling; Tier 2 targets generally build and run but may lack host tools; Tier 3 targets require bespoke effort.

### Target Triples and libc variants

Two practical libc variants appear frequently in Linux targets: glibc and musl. For glibc-based targets, the OS target triple typically ends with -unknown-linux-gnu; for musl, -unknown-linux-musl. Musl is attractive for static linking and footprint reduction, but teams should verify system library availability inside containers and the runtime environment. In cross, pre-build steps can install architecture-specific packages using CROSS_DEB_ARCH (for Debian/Ubuntu images), which is essential when building against vendored or system libraries.[^6]

### cargo configuration: target stanza essentials

Cargo’s configuration model allows per-target settings in a target section, commonly used to specify linkers, ar, runner wrappers, and rustc flags. These settings can live in .cargo/config.toml or .cargo/config/runtime and be overridden by environment variables named CARGO_TARGET_<target triple>_<option>. When using cross, prefer environment-variable overrides for target-specific flags, as cross orchestrates the container environment and does not inherit all host environment variables by default.[^3][^6]

## Tooling Landscape: rustup, cargo, cross, Docker/Podman, osxcross

Rustup is the canonical toolchain manager. It installs target std components with rustup target add, and associates targets with toolchains (including nightly when build-std is needed). After installation, you can pass --target <triple> to Cargo build commands.[^1]

Cargo provides the configuration stanza for per-target linkers, ar, rustflags, and runner commands. When you need a custom linker for a target, declare it under [target.<triple>].cargo config target documentation provides authoritative semantics for these entries.[^3]

cross executes builds inside curated container images, managing system libraries and linkers for you. It supports Docker or Podman, allows custom images via Cross.toml, and exposes runners, including QEMU user-mode for cross test. cross test relies on QEMU emulation; this is convenient but not a perfect proxy for native execution, and failures can arise from QEMU-specific bugs or emulation limitations rather than your code.[^6]

Docker/Podman supply the isolation and reproducibility that cross leverages. On Linux, Podman is a common alternative to Docker Desktop; both integrate with cross seamlessly. These container engines provide consistent toolchains across developer machines and CI runners.[^2][^6]

osxcross packages the macOS SDK for Linux cross-builds, enabling linking for Darwin targets when Apple’s proprietary toolchain is required. This is often the missing piece in macOS targeting: cross does not provide official Apple images, so teams must build and maintain local images aligned to specific Xcode SDK versions.[^8][^12]

### When to use rustup/cargo vs cross

Prefer Cargo/rustup when building purely Rust code or when your host environment already contains the correct linkers and system libraries, such as Linux-to-Windows mingw or Linux-to-Linux glibc targets. This path offers simplicity and speed, directly leveraging installed toolchains without container overhead.[^1]

Prefer cross for multi-arch workflows involving system libraries, hermetic builds, and consistent CI execution. It reduces per-developer setup, enforces reproducible dependency versions, and centralizes target-specific knowledge in Cross.toml. It also helps teams converge on identical behavior across macOS, Linux, and Windows targets, including musl where packaging nuances can otherwise vary by host.[^6]

## Hands-on Setup for x86_64, ARM64 (AArch64), and RISC‑V

A proven baseline for cross-compilation includes: rustup managing toolchains and targets; Cargo controlling build flows; cross for containerized builds; and QEMU for testing. For macOS, add osxcross with pinned SDK.

### Table: Quick-start commands matrix

The following matrix summarizes baseline commands for installing targets and building on each approach.

| Operation                                | rustup/Cargo baseline                                 | cross baseline                                      | Container (Docker/Podman) baseline |
|------------------------------------------|--------------------------------------------------------|-----------------------------------------------------|------------------------------------|
| Install target std                       | rustup target add <triple>                             | cross handles target via its image                  | Ensure image has rustup target     |
| Build for target                         | cargo build --target <triple>                          | cross build --target <triple>                       | docker run … cargo build --target  |
| Run binary                               | Use native execution (if host matches) or QEMU runner  | cross run --target <triple>                         | docker run … ./binary              |
| Run tests (std)                          | cargo test --target <triple>                           | cross test --target <triple>                        | docker run … cargo test            |
| macOS targets from Linux                 | Requires osxcross toolchain and Cargo linker settings  | Custom image using osxcross; Cross.toml image field | Build custom image per osxcross    |

This baseline aligns teams on the minimal viable steps, while the nuances live in per-target configuration.

### x86_64 baselines

Linux and Windows are straightforward. For Windows, install mingw toolchains on Linux and build with the appropriate triple (e.g., x86_64-pc-windows-gnu). A practical pathway is:

- Use cross build --target x86_64-pc-windows-gnu if you prefer hermetic builds with system libraries handled inside the container.[^2]
- Or use Cargo/rustup with mingw installed on the host, and set the linker in Cargo’s target configuration.[^3]

Both routes are reliable; cross is recommended when you want CI consistency or system libraries beyond the Rust-only crate graph.[^2][^6]

### ARM64 (AArch64) baselines

For Linux on ARM64, install aarch64-unknown-linux-gnu with rustup and build using cargo build --target. On host systems lacking ARM64 linkers or system libraries, cross automates dependency installation and uses curated images to ensure consistent builds.[^1][^6]

macOS Apple Silicon (aarch64-apple-darwin) introduces linking constraints that favor custom images and osxcross on Linux. From macOS hosts, native builds are feasible; cross compiling from Linux to macOS requires the osxcross toolchain and a custom image, with careful version pinning of the SDK and clang.[^8][^12]

### RISC‑V baselines

Common Linux targets include riscv64gc-unknown-linux-gnu (RV64IMAFDC). For bare-metal no_std targets, use riscv64gc-unknown-none-elf and configure a custom target JSON when you need precise linker options, atomics, and panic strategies. The Embedonomicon offers the definitive guidance for custom target creation, including how to set llvm-target, features, and linker arguments.[^14]

## Build System Design for Multi-Architecture OS Development

A robust build system for multi-arch OS development should separate concerns and make configuration explicit:

- rust-toolchain.toml pins the Rust channel and, optionally, the targets installed by rustup. This eliminates “it works on my machine” drift across contributors and CI runners.[^8]
- Cargo’s [target.*] stanzas declare linkers, ar, rustflags, and runner wrappers per triple. Keep these in the repository and avoid relying on host-specific global Cargo configuration.[^3]
- Cross.toml defines container image usage, pre-build commands for system dependencies, environment passthrough, and runner selection (including QEMU wrappers). It is the canonical place to encode target-specific build semantics.[^6]

### Table: Configuration sources and precedence

The following precedence applies to cross’s configuration model:

| Source                       | Precedence | Applies To                          | Notes |
|------------------------------|------------|-------------------------------------|-------|
| Environment variables        | Highest    | Global or per-target options        | E.g., CROSS_BUILD_XARGO, CARGO_TARGET_<triple>_LINKER.[^6] |
| Cross.toml                   | Middle     | Global or per-target options        | image, pre-build, runner, zig, env passthrough.[^6] |
| Cargo.toml metadata          | Lowest     | Some cross settings                 | package.metadata.cross.KEY.[^6] |

This hierarchy helps teams avoid conflicts and “leaky” host configuration.

### Table: Candidate target matrix for OS development

A representative matrix clarifies linker expectations and emulator modes across architectures:

| Target Triple                     | Linker (typical)            | libc     | Emulator/Runtime                | Notes |
|-----------------------------------|-----------------------------|----------|---------------------------------|-------|
| x86_64-unknown-linux-gnu          | cc (host) or gcc in container | glibc    | native or QEMU user-mode        | Tier 1 host tools.[^4] |
| x86_64-pc-windows-gnu             | x86_64-w64-mingw32-gcc       | win32    | native (Windows) or Wine        | mingw path common.[^2][^3] |
| aarch64-unknown-linux-gnu         | aarch64-linux-gnu-gcc         | glibc    | QEMU user-mode or native ARM    | Tier 1 host tools.[^4][^6] |
| aarch64-apple-darwin              | osxcross clang                | Darwin   | macOS host; custom image from Linux | Requires osxcross and SDK.[^8][^12] |
| riscv64gc-unknown-linux-gnu       | riscv64-linux-gnu-gcc         | glibc    | QEMU user-mode or native RISC‑V | Tier 2 with host tools.[^4] |
| riscv64gc-unknown-none-elf        | Custom linker (ld/lld)        | none     | QEMU system emulation           | no_std, custom JSON.[^14] |
| aarch64-unknown-none-softfloat    | Custom linker (ld/lld)        | none     | QEMU system emulation           | Bare-metal ARM64, custom JSON.[^13][^14] |

The linker column indicates typical linker toolchains, not Cargo configuration. Linker selection must be declared in [target.<triple>] or via environment variables under cross.

### Per-target linkers and flags

Setting linkers and flags per target reduces friction across architectures. For example, on Linux to Windows, configure [target.x86_64-pc-windows-gnu] with the mingw linker and ar. On ARM64 Linux, ensure aarch64-linux-gnu-gcc is available (via host packages or cross images). These entries should be kept in .cargo/config.toml within the repository to enforce consistency across machines.[^3][^9]

### Custom images and pre-build dependencies

cross supports custom images defined in Cross.toml. Use pre-build to install system libraries, and passthrough to inject environment variables. For Debian/Ubuntu-derived images, CROSS_DEB_ARCH enables architecture-qualified package installs, e.g., libssl-dev:$CROSS_DEB_ARCH. This is vital for crates depending on system libs across musl and glibc targets.[^6]

### OS/kernel no_std considerations

For bare-metal kernels, you will build without the standard library (no_std), manage atomics widths explicitly, and pick a panic strategy (often abort). Linker flags and startup sequences must be encoded in the custom target JSON. When your target is not built-in, compile with Cargo’s unstable build-std to build core and alloc against your custom spec.[^13][^14]

### Reproducible builds and CI cache strategy

Make your build hermetic by pinning toolchains and controlling environment variables. In CI, build once per target and test separately to isolate compile-time from test-time regressions. sccache can materially speed up repeated builds and cross-target variants by caching compiler outputs; integrate it as a wrapper and ensure the cache persists across jobs.[^9]

## Testing Strategies: QEMU, Cross Testing, and CI Validation

Testing cross-compiled binaries hinges on the environment you target.

cross test executes tests inside containers using QEMU user-mode emulation for non-native architectures. This is efficient and broad, but not all syscalls, filesystem semantics, or threading behaviors are perfectly emulated. Sometimes QEMU bugs or emulation edge cases cause test failures unrelated to your crate.[^6]

For bare-metal and OS development, QEMU system emulation provides a realistic execution environment. The Embedded Rust Book demonstrates launching binaries in QEMU, capturing output via semihosting, and connecting a GDB server for stepwise debugging. This workflow scales well to kernel-style integration tests and controlled boot sequences.[^11]

### Table: Testing matrix by target and method

The table below maps typical targets to testing approaches.

| Target Category                  | Native            | QEMU user-mode (cross test)    | QEMU system emulation            |
|----------------------------------|-------------------|--------------------------------|----------------------------------|
| x86_64 Linux (std)               | Yes               | Optional                       | Rare                              |
| ARM64 Linux (std)                | On ARM hardware   | Yes via cross                  | Possible for OS-level tests       |
| RISC‑V Linux (std)               | On RISC‑V hardware| Yes via cross                  | Possible                          |
| macOS Darwin (std)               | On macOS          | Not via cross (no official img)| Use VMs only                      |
| Windows (std)                    | On Windows        | Optional via cross (Wine context)| Not typical                      |
| Bare-metal kernels (no_std)      | Hardware or simulator | Not applicable              | Yes, primary method               |

For macOS, cross does not provide official images due to licensing; expect to use native macOS hosts or maintain custom osxcross-based images. For bare-metal, QEMU system emulation is the backbone of integration testing.[^6][^11][^8]

### cross test in practice

Expect sequential test execution and occasional false negatives due to QEMU limitations. Use cross run for ad-hoc binary checks, and apply QEMU_STRACE to diagnose syscall-level behavior when debugging failures in foreign binaries.[^6]

### QEMU system emulation for kernels/OS

Define runner commands in Cargo config for QEMU system targets, enable semihosting to observe console output, and attach GDB for remote debugging. QEMU’s command-line flags (e.g., -cpu, -machine, -kernel, -gdb) give fine-grained control over the emulated board and debugging session. This approach is portable across Cortex-M and RISC‑V kernels alike, and integrates directly with Cargo runners.[^11][^14]

### CI test orchestration

In CI, build per target in parallel, then execute tests under the appropriate runner. On macOS, run native tests on macOS runners; on Linux, prefer cross test for std targets and QEMU for no_std. Keep cache keys distinct per target triple and toolchain to avoid cross-talk, and disable global Cargo configuration that could interfere with cross’s containerized environment.[^6][^9]

## CI/CD Implementation: Matrix Builds, Artifacts, and Reproducibility

A robust CI pipeline uses a matrix strategy across target triples, one job per target to enable parallelism and targeted artifact naming. The core steps are: checkout, set up Rust toolchain and target(s), install any cross-compilation system dependencies (e.g., mingw for Windows), build, rename artifacts, and upload release assets on tags. Separate build and test stages to isolate compilation from test execution and improve failure triage.[^9]

### Table: Example CI matrix and artifact naming

An illustrative matrix for a mixed Linux/Windows/x86_64 and ARM64 build:

| rust_target                         | runner               | artifact_name                      | notes |
|-------------------------------------|----------------------|------------------------------------|-------|
| x86_64-unknown-linux-gnu            | ubuntu-latest        | app-linux-amd64                    | Build with cargo build --release.[^9] |
| x86_64-pc-windows-gnu               | ubuntu-latest        | app-windows-amd64.exe              | Install mingw on the runner.[^9] |
| aarch64-unknown-linux-gnu           | ubuntu-latest        | app-linux-arm64                    | Build with cross or Cargo depending on deps.[^6][^9] |
| riscv64gc-unknown-linux-gnu         | ubuntu-latest        | app-linux-riscv64                  | Use cross for system libraries.[^6] |

Upload artifacts only on tagged releases to avoid CI noise. Maintain per-target cache keys and ensure sccache or equivalent is configured with a shared cache backend for multi-job efficiency.[^9]

### Reproducibility and cache efficiency

Reproducibility comes from hermetic builds and pinned toolchains. Cache compiler outputs with sccache and structure jobs so each target triple maps to a distinct cache key. Balance storage costs and hit rates by tuning the number of cached targets and the retention policy.[^9]

## Real-World Case Study: Cross-Compiling novops (Linux/macOS/Windows, x86_64 & ARM64)

The novops case study demonstrates a pragmatic path to multi-arch builds across Linux, macOS, and Windows using cross, quickemu for validation, and osxcross for Darwin targets. Key decisions included:

- Using cross with containerized toolchains for Linux targets and pre-build steps to install system libraries. This ensured predictable builds without polluting host systems.[^7]
- Enabling the vendored feature for OpenSSL when targeting musl,规避 system library version mismatches by building OpenSSL within the crate graph rather than depending on host-provided packages.[^7]
- Packaging a macOS SDK via osxcross and building a custom image to compile Darwin targets from Linux, accepting the operational complexity in exchange for consistent CI.[^12][^7]
- Managing Xcode SDK compatibility carefully: versions beyond a certain cutoff were incompatible with the osxcross commit used in cross-toolchains at the time, requiring version pinning and careful alignment.[^7]

Outcomes included successfully generating Linux, macOS, and Windows binaries across x86_64 and ARM64, with a working Linux-to-macOS cross-compilation path for Darwin targets in CI.[^7]

### Table: novops target matrix and configuration

| OS/Arch          | Target Triple                    | Build Tool      | Special Configuration            | Status |
|------------------|----------------------------------|-----------------|----------------------------------|--------|
| Linux x86_64     | x86_64-unknown-linux-musl        | cross           | pre-build apt packages           | Success[^7] |
| Linux ARM64      | aarch64-unknown-linux-musl       | cross           | CROSS_DEB_ARCH=arm64 packages    | Success[^7] |
| macOS x86_64     | x86_64-apple-darwin              | cross (custom)  | osxcross image, SDK pinning      | Success[^7][^12] |
| macOS ARM64      | aarch64-apple-darwin             | cross (custom)  | osxcross image, SDK pinning      | Success[^7][^12] |
| Windows x86_64   | x86_64-pc-windows-gnu            | cross or Cargo  | mingw linkage                    | Success[^2][^3][^7] |

### macOS/Darwin: osxcross and custom images

From Linux, macOS targeting requires osxcross: package the SDK, install clang toolchains, and add the bin directory to PATH. In cross, define a custom image in Cross.toml and pin toolchain versions to avoid drift. The known constraint is SDK compatibility: align the Xcode version with the osxcross commit used by cross-toolchains, and test explicitly before adopting new SDKs.[^8][^12][^7]

## Troubleshooting and Pitfalls

Global Cargo configuration can conflict with cross, particularly wrappers like sccache defined in $HOME/.cargo/config.toml. cross runs builds inside containers and does not inherit arbitrary host environment variables. Prefer repository-local configuration and, when needed, enable sccache inside cross via its documented mechanisms.[^8][^6]

build.rs scripts that presume host tooling can break cross-compilation. Configure linkers and flags in Cargo target stanzas, not inside build.rs, and keep build scripts focused on building native dependencies rather than managing the cross toolchain. This reduces confusion and ensures settings apply consistently across dependencies.[^3][^6]

musl linking issues, especially for C dependencies like OpenSSL, often resolve by vendoring those dependencies rather than relying on host packages. Toggle vendored features when targeting musl and ensure container images have appropriate development headers installed via pre-build commands.[^7][^6]

QEMU emulation can produce false negatives, particularly around threading and filesystem semantics. When cross test fails, reproduce on native hardware where feasible, or switch to QEMU system emulation for more faithful behavior. Apply QEMU_STRACE to inspect syscall behavior and isolate emulator issues from application bugs.[^6][^11]

## Best Practices and Recommendations

Prefer cross for multi-arch builds with system libraries and containerized reproducibility. Keep Cross.toml in the repository, pin custom images, and centralize per-target knowledge in this file. Use pre-build commands to install system libraries and declare environment passthrough explicitly.[^6]

Prefer Cargo/rustup for native, Rust-only workflows. Configure [target.*] stanzas for linkers and flags, keep configuration under version control, and avoid reliance on host-specific global settings. This keeps builds simple and transparent.[^3][^1]

Pin toolchains in rust-toolchain.toml and explicitly install targets for the chosen toolchain. This ensures contributors and CI operate with the same compiler version and std components.[^1][^8]

Separate build and test stages in CI to simplify triage. Cache effectively with sccache and structure matrix builds to avoid cross-target cache contamination. Upload release artifacts only on tags to maintain a clean, auditable release history.[^9]

For macOS targets from Linux, adopt osxcross with pinned SDKs and custom images. Budget time for version compatibility work and add verification steps to your CI to catch drift early.[^8][^12][^7]

## Appendices

### Appendix A: Per-architecture quick-start checklists

- Rust toolchain and targets
  - Install rustup.
  - Pin toolchain via rust-toolchain.toml (channel and targets).
  - rustup target add <triple> for each target in native workflows.[^1]

- x86_64
  - Linux glibc: cargo build --target x86_64-unknown-linux-gnu.
  - Linux musl: cargo build --target x86_64-unknown-linux-musl or cross build --target x86_64-unknown-linux-musl.
  - Windows: install mingw and configure linker in Cargo [target.x86_64-pc-windows-gnu], or use cross build.[^2][^3][^6]

- ARM64 (AArch64)
  - Linux: cargo build --target aarch64-unknown-linux-gnu or cross build --target aarch64-unknown-linux-gnu.
  - macOS (from macOS): native build with Xcode toolchains.
  - macOS (from Linux): prepare osxcross SDK and build custom cross image; cross build --target aarch64-apple-darwin.[^8][^12]

- RISC‑V
  - Linux: cargo build --target riscv64gc-unknown-linux-gnu.
  - Bare-metal: custom target JSON; cargo build -Z build-std=core,alloc --target <your-target>.json.[^13][^14]

### Appendix B: Cross.toml skeletons

- Global image and pre-build:

  [build]
  image = "my-org/rust-cross:stable"
  pre-build = [
    "dpkg --add-architecture $CROSS_DEB_ARCH",
    "apt-get update && apt-get --assume-yes install libssl-dev:$CROSS_DEB_ARCH"
  ]

- Target-specific runner and image:

  [target.aarch64-unknown-linux-gnu]
  image = { name = "my-org/rust-cross:aarch64", toolchain = ["x86_64-unknown-linux-musl", "linux/arm64=aarch64-unknown-linux-musl"] }
  runner = "qemu-aarch64"

- Zig integration (optional):

  [target.x86_64-unknown-linux-musl]
  zig.enable = true
  zig.version = "2.17"

These snippets illustrate key options; adjust names and commands for your environment.[^6]

### Appendix C: Example .cargo/config.toml target sections

- Windows (mingw):

  [target.x86_64-pc-windows-gnu]
  linker = "x86_64-w64-mingw32-gcc"
  ar = "x86_64-w64-mingw32-ar"

- ARM64 Linux:

  [target.aarch64-unknown-linux-gnu]
  linker = "aarch64-linux-gnu-gcc"
  ar = "aarch64-linux-gnu-ar"

- RISC‑V bare-metal (runner and flags may be set via environment under cross):

  [target.riscv64gc-unknown-none-elf]
  runner = "qemu-system-riscv64"
  rustflags = ["-C", "link-arg=-Tlayout.ld"]

Use environment variables under cross for runner and rustflags when building inside containers, following CARGO_TARGET_<target>_RUNNER and CARGO_TARGET_<target>_RUSTFLAGS patterns.[^3][^6]

### Appendix D: GitHub Actions template (matrix per target)

jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        include:
          - rust_target: x86_64-unknown-linux-gnu
            artifact_name: app-linux-amd64
          - rust_target: x86_64-pc-windows-gnu
            artifact_name: app-windows-amd64.exe
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable
          targets: ${{ matrix.rust_target }}
      - name: Install mingw (Windows target)
        if: matrix.rust_target == 'x86_64-pc-windows-gnu'
        run: sudo apt-get install -y gcc-mingw-w64-x86-64
      - name: Build
        run: cargo build --release --target ${{ matrix.rust_target }}
      - name: Rename artifact
        run: mv target/${{ matrix.rust_target }}/release/app ${{ matrix.artifact_name }}
      - name: Upload release assets
        if: startsWith(github.ref, 'refs/tags/')
        uses: svenstaro/upload-release-action@v2
        with:
          file: ${{ matrix.artifact_name }}
          overwrite: true
          prerelease: false
          tag: ${{ github.ref }}

This template balances clarity and flexibility, with separate steps for dependencies and artifact naming.[^9]

## Information Gaps and Caveats

- cross test and QEMU user-mode reliability vary by architecture and target; results can differ between emulated and native environments. Validate critical paths on native hardware when possible.[^6]
- macOS/Darwin linking and osxcross compatibility shift with Xcode and SDK releases; pin versions and test CI images regularly.[^8][^12][^7]
- CI caching strategies (sccache, registry caches) are highly project-dependent; measure cache hit rates and adjust policies to your build graph.[^9]
- Custom target JSON for no_std and build-std flows requires nightly and may interact with build scripts or system library crates; test thoroughly before adopting widely.[^13][^14]

These caveats are not blockers but demand disciplined configuration management and regular validation.

## References

[^1]: Cross-compilation - The rustup book. https://rust-lang.github.io/rustup/cross-compilation.html  
[^2]: A guide to cross-compilation in Rust - LogRocket Blog. https://blog.logrocket.com/guide-cross-compilation-rust/  
[^3]: Cargo Reference: Configuration (target section). https://doc.rust-lang.org/cargo/reference/config.html#target  
[^4]: The rustc book: Platform Support. https://doc.rust-lang.org/rustc/platform-support.html  
[^5]: A guide to cross-compilation in Rust (Greg Stoll) - Example repo. https://github.com/gregstoll/rust-crosscompile  
[^6]: cross-rs/cross - “Zero setup” cross compilation and cross testing. https://github.com/cross-rs/cross  
[^7]: A Rust cross compilation journey (novops case study). https://blog.crafteo.io/2024/02/29/my-rust-cross-compilation-journey/  
[^8]: Cross-Compiling Your Project in Rust - Tangram Visions Blog. https://www.tangramvision.com/blog/cross-compiling-your-project-in-rust  
[^9]: Write a GitHub Actions Workflow for Rust cross-compilation. https://medium.com/@mellomello2030/write-a-github-actions-workflow-for-rust-cross-compilation-44284dfa9597  
[^10]: Testing Cross Compiling with QEMU - Robopenguins. https://www.robopenguins.com/cross-compiling/  
[^11]: The Embedded Rust Book: QEMU. https://docs.rust-embedded.org/book/start/qemu.html  
[^12]: osxcross - Cross compile macOS toolchain on Linux. https://github.com/tpoechtrager/osxcross  
[^13]: The Embedonomicon: Creating a custom target. https://docs.rust-embedded.org/embedonomicon/custom-target.html  
[^14]: Adding a new target - Rust Compiler Development Guide. https://rustc-dev-guide.rust-lang.org/building/new-target.html