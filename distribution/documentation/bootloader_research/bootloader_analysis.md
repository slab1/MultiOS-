# Modern Bootloaders and Cross-Architecture Boot Strategies for x86_64, ARM64, and RISC‑V (with a focus on educational OS development in Rust)

## Executive Summary

Boot firmware is the bridge between hardware复位 and an operating system. On contemporary platforms, this bridge is shaped by a small set of standards that determine how firmware initializes the system, how it hands control to a bootloader or OS loader, and what runtime services persist after boot. For x86_64, the industry has largely transitioned from legacy BIOS to the Unified Extensible Firmware Interface (UEFI), which defines aarchitected boot services, runtime services, and the System Table as the central discovery mechanism for OS loaders[^1][^2]. On ARM and RISC‑V, boot flows are more diverse: platforms often chain through multiple stages (e.g., ARM Trusted Firmware, U‑Boot, or EDK2/UEFI), and rely on Devicetree to describe hardware to the OS when hardware discovery is limited[^8][^6][^12]. RISC‑V introduces the Supervisor Binary Interface (SBI) as a portable supervisor-level firmware layer invoked via the ecall instruction; OpenSBI implements this interface and supports several firmware payload strategies to reach the OS[^10][^11][^13].

At a practical level for educators and hobbyists:

- UEFI offers a standardized, rich boot environment (System Table, Boot Services, Runtime Services, Secure Boot, GPT), but requires understanding PE/COFF image formats, memory map semantics, and calling conventions[^1][^2][^17].
- Legacy BIOS boot still has educational value—its simplicity makes real-mode transitions, memory detection, and protected mode entry very tangible—but lacks modern features and scalability[^4][^2].
- ARM boot flows commonly involve a first-stage loader (e.g., SPL) and a second-stage bootloader (U‑Boot or EDK2/UEFI), often passing a Devicetree Blob (DTB) to the kernel[^6][^8][^12].
- RISC‑V flows typically start with a Zero Stage Bootloader (ZSBL) that hands control to OpenSBI (M-mode), which then transitions to the OS kernel (S-mode), with parameters passed in registers and optionally via a Device Tree[^10][^11][^12].

For Rust-oriented educational OS projects, practical choices exist:

- The pure-Rust bootloader crate supports x86_64 for BIOS and UEFI, offering a BootInfo API and disk image creation workflows[^21].
- Limine is a cross-architecture bootloader with a simple protocol; Rust developers can use the limine crate to parse Limine structures or pair the kernel with the Limine bootimage tool[^18][^19][^20][^22].
- GNU-EFI and POSIX-UEFI enable writing UEFI applications in C/C++; Rust projects can consume UEFI via uefi and uefi-services crates and test with QEMU + OVMF[^16][^26][^25].

Recommended minimal boot stacks per architecture for teaching:

- x86_64: Start with BIOS + Multiboot2 to learn fundamentals; move to UEFI for modern workflows and Secure Boot awareness[^5][^2].
- ARM64: Use the rust-embedded Raspberry Pi tutorials as a guided path from bare metal to MMU, interrupts, and timer use; progress to U‑Boot or EDK2/UEFI on models where available[^7][^6][^8].
- RISC‑V: Start with the rust-embedded tutorials and QEMU virt; use OpenSBI FW_DYNAMIC to understand flexible payload handoff and pass the Device Tree in a1[^23][^10][^11].

Across architectures, secure boot, measured boot, and standard payload interfaces (UEFI System Table, Multiboot2 tags, Device Tree, RISC‑V SBI) shape the educational design space. A unified narrative emphasizes how firmware abstracts hardware, how boot protocols pass information to the kernel, and how portability can be achieved without sacrificing rigor[^1][^5][^12][^13].

## Foundations and Terminology

Boot firmware sits at the most privileged level of software, responsible for platform initialization and for loading the operating system or its bootloader. Standards exist to define these responsibilities so that OS code is insulated from platform-specific device details.

On UEFI systems, firmware presents Boot Services and Runtime Services to OS loaders and OS code, respectively. Every UEFI application receives the System Table—a struct containing handles to the EFI system configuration tables (e.g., ACPI, SMBIOS), console I/O protocols, and runtime services—upon entry. UEFI also defines a standard way to partition disks using GUID Partition Table (GPT), and an EFI System Partition (ESP) to host boot applications and support files[^1][^2]. Legacy BIOS, by contrast, provides INT services and relies on the Master Boot Record (MBR) to stage a tiny boot sector, from which the OS or a second-stage bootloader must set up protected/long mode and manage memory and devices itself[^2][^4].

On ARM and RISC‑V, Devicetree is the lingua franca for describing hardware at boot time when PCI-class enumeration is absent or insufficient. Devicetree’s textual source (DTS) compiles to a binary blob (DTB) with a defined structure—header, memory reservation block, structure block, and strings block—used by the OS to discover memory, CPUs, and peripheral devices[^12]. The kernel boot process on both ARM and RISC‑V typically expects a DTB to be passed by firmware or bootloader[^6][^10].

In the RISC‑V privilege stack, Machine Mode (M-mode) is the highest privilege and must be implemented on all cores; more capable systems add Supervisor Mode (S-mode) for OS kernels and User Mode (U-mode) for applications. The Supervisor Binary Interface (SBI) defines a portable set of supervisor-level calls that an OS can invoke via ecall, enabling firmware to abstract environment details (console, timer, harts) and provide standard entrypoints[^13]. This design allows the same OS kernel to boot across heterogeneous RISC‑V platforms, as long as SBI is implemented[^13][^10].

UEFI is intentionally designed for little-endian operation and to be processor-architecture agnostic over time, with support for PE/COFF images across x86, x86_64, ARM, AArch64, and RISC‑V[^1][^2]. Practical guidance for educational use includes understanding:

- The roles of Boot Services (available only pre–ExitBootServices) vs Runtime Services (persist across boot), and the implications for memory reuse by the kernel[^2].
- The UEFI memory map types and which regions are safe to reuse after boot services are exited[^2].
- The x86_64 calling convention mismatch between C foreign functions and Microsoft x64 ABI used by UEFI on x86_64; build systems must bridge this (e.g., via GNU-EFI’s uefi_call_wrapper or POSIX-UEFI’s make-based ABI normalization)[^2].
- The practical requirement that ESPs often be FAT32 with the fallback filename for x86_64 being EFI/BOOT/BOOTX64.EFI[^2].

To illustrate the different expectations across platforms for discovery and runtime services, Table 1 summarizes the typical mechanisms.

Table 1: Boot discovery and runtime mechanisms by architecture

| Architecture | Typical Firmware/First Stage | Boot Discovery Mechanism | Runtime Services |
|--------------|------------------------------|--------------------------|------------------|
| x86_64       | BIOS or UEFI                 | BIOS: MBR, INT 13h; UEFI: System Table, ESP, Boot Services | UEFI Runtime Services; ACPI tables via System Table[^1][^2] |
| ARM64        | TF‑A (optional), U‑Boot SPL, EDK2/UEFI | Devicetree (DTB) via bootloader; UEFI System Table on UEFI platforms | UEFI Runtime Services if using EDK2; otherwise firmware-specific services; ACPI on some platforms[^6][^8][^12] |
| RISC‑V       | ZSBL → OpenSBI               | Register a0/a1, Device Tree; SBI as firmware API | OpenSBI runtime services via SBI; Device Tree for hardware description[^10][^11][^12][^13] |

The design intent behind these mechanisms is consistent: pass a minimal, portable description of the platform to the kernel while exposing a stable interface boundary that allows OS developers to avoid hardware-specific code[^1][^12][^13].

## Architecture-Specific Boot Flows

The end-to-end boot sequence must take a system from reset to an OS kernel executing with memory, interrupt, and timer foundations in place. While the specifics vary, a useful pedagogical approach is to compare end-to-end flows and identify common patterns.

Table 2 presents a consolidated view of the boot flows for x86_64 (BIOS and UEFI), ARM64 (U‑Boot and EDK2/UEFI), and RISC‑V (OpenSBI), including key handoff structures and expectations.

Table 2: End-to-end boot flow by architecture and bootloader stack

| Architecture | Stack Components | Entry & Initialization | Key Handoff to Kernel | Notable Constraints/Notes |
|--------------|------------------|------------------------|-----------------------|----------------------------|
| x86_64 (BIOS/MBR) | BIOS → MBR boot sector → Second-stage bootloader | BIOS performs POST, enables A20, selects boot device; loads 512-byte sector to 0x7C00 | Bootloader enters protected/long mode, sets up GDT/IDT, loads kernel, may pass Multiboot info | Kernel must enable A20, configure paging; bootloader must respect BIOS memory map and disk geometry[^2][^4][^5] |
| x86_64 (UEFI) | Firmware → UEFI OS loader | UEFI firmware initializes; loads PE/COFF app from ESP; passes System Table | ExitBootServices with memory map; PE/COFF entry; UEFI protocols for console, simple file system | Must handle calling conventions; must avoid touching Runtime Services memory; Secure Boot affects unsigned loaders[^1][^2][^3] |
| ARM64 (U‑Boot) | SPL → U‑Boot → Kernel | U‑Boot runs at entry EL; relocates; may handle FDT placement; supports spin-table SMP | DTB via chosen node; kernel Image loaded at a prescribed address; initrd optional | Kernel’s FDT placement constraints relaxed since v4.2; spin-table encodes secondary CPU entry points in device tree[^6] |
| ARM64 (EDK2/UEFI) | EDK2/UEFI → Kernel | EDK2 builds UEFI firmware; boots via ESP; kernel and DTB loaded; startup script parameters | Kernel command line, DTB, initrd via UEFI variables or startup.nsh | Requires correct dtb parameter for platform; supports TFTP and local boot; Juno variants use specific DTBs[^8] |
| RISC‑V (OpenSBI) | ZSBL → OpenSBI (M-mode) → Kernel (S-mode) | ZSBL jumps to OpenSBI; OpenSBI initializes; determines next mode and address via FW_* | a0: hartid; a1: DTB address; satp=0; kernel at PMD boundary; SBI calls for console/timer | FW_PAYLOAD, FW_JUMP, FW_DYNAMIC differ in flexibility; DTB often supplied by platform/QSBL[^10][^11][^14] |

The remainder of this section explores each architecture’s flow in more depth.

### x86_64 Boot Flows (BIOS vs UEFI)

Legacy BIOS boot is a rite of passage for OS developers: on power-on, the firmware performs a Power-On Self-Test (POST), enables the A20 gate for accessing memory above 1 MB, and loads the first 512-byte boot sector from the MBR of the selected boot device to physical address 0x7C00, then transfers control. From there, the bootloader must switch the CPU into protected mode (and, for 64-bit kernels, long mode), set up a Global Descriptor Table (GDT) and Interrupt Descriptor Table (IDT), and load the kernel image and any boot modules. The simplicity of this path makes it ideal for teaching low-level initialization, memory detection via BIOS INT services, and disk I/O via INT 13h[^4][^2].

By contrast, UEFI abstracts device access and exposes a structured environment. The firmware:

- Loads a UEFI application from the EFI System Partition on a GPT- or MBR-partitioned disk (commonly FAT32) and calls its entry point with the System Table.
- Provides Boot Services for device access (via protocols like Block I/O, Simple File System) and Runtime Services that persist after ExitBootServices.
- Expects PE/COFF executables (PE32/PE32+) for applications and drivers; OS loaders are ordinary UEFI applications[^1][^2].

Memory management in UEFI requires careful treatment: after ExitBootServices, the kernel may reuse memory where the boot loader was loaded and non-read-only Boot Services data, but must never touch Runtime Services memory. The memory map types, returned by the firmware, convey which regions are safe to repurpose[^2].

Security models differ sharply: BIOS offers little beyond password protections, while UEFI includes Secure Boot with platform keys and allow/deny lists (PK/KEK/db/dbx). For educational OSes, signed loaders and key management can become a major hurdle; practical workflows often rely on disabling Secure Boot or using signed intermediaries in class or lab settings[^2][^3].

Finally, boot order in UEFI is managed via NVRAM variables. Tools such as efibootmgr (Linux) and bcfg (UEFI Shell) manipulate BootOrder and BootXXXX entries to change the firmware’s default selection of the ESP and the loader path (e.g., EFI/BOOT/BOOTX64.EFI)[^2].

### ARM64 Boot Flows (U‑Boot and EDK2/UEFI)

ARM platforms often employ multi-stage boot sequences. A common pattern is a first-stage loader (SPL) that fits within ROM constraints, which then loads a second-stage bootloader such as U‑Boot or a UEFI firmware like EDK2. U‑Boot can run at different Exception Levels (EL3 for firmware responsibilities, EL2 for hypervisor-like roles, dropping to EL1 before loading the OS) and supports SMP bringup via a spin-table mechanism: secondary CPUs poll a memory location for their entry point encoded in the device tree[^6].

Historically, Linux on ARM64 required strict Device Tree placement (e.g., 2 MB alignment and proximity to the kernel), but since kernel version 4.2 these constraints have been relaxed; DTBs may be placed anywhere in memory. In practice, U‑Boot loads the kernel Image and DTB and passes control with registers set appropriately; initrd loading is platform-dependent[^6].

EDK2 provides a UEFI firmware environment on ARM platforms, enabling standard UEFI boot flows—ESP-based loading, System Table discovery, and boot services usage. Arm’s reference guidance for platforms like Juno shows kernel and DTB loading via USB or TFTP, with command-line parameters for console, root, and debug options, as well as explicit dtb settings for platform variants. This provides a straightforward path for educators to demonstrate UEFI boot on ARM hardware or models[^8].

### RISC‑V Boot Flows (OpenSBI and SBI)

RISC‑V boot typically begins with a Zero Stage Bootloader (ZSBL) at a fixed address (e.g., 0x1000 in QEMU’s riscv64 virt machine), which performs minimal initialization and jumps to OpenSBI at a higher address (e.g., 0x80000000). OpenSBI, running in M-mode, completes environment setup and transitions to the OS kernel, commonly in S-mode. Parameters are passed via registers—hart ID in a0 and a Device Tree address in a1—while the SATP register is zeroed, indicating the kernel should set up its own address translation[^10][^14].

OpenSBI can be configured in three flavors to control how it hands off to the next stage:

- FW_PAYLOAD: OpenSBI splices the next payload (e.g., the kernel) into its binary, producing a monolithic firmware image.
- FW_JUMP: OpenSBI jumps to a fixed address where the next stage is expected, without embedding it.
- FW_DYNAMIC: The prior boot stage populates a fw_dynamic_info structure, and OpenSBI follows those instructions, including the next address and mode. This offers the greatest flexibility, particularly when the platform firmware or ZSBL controls placement and formatting[^10][^11].

OpenSBI also implements the SBI standard, which provides console I/O and timer services and supports extension IDs (EIDs) and function IDs (FIDs) encoded via ecall. This keeps early kernel code portable across diverse RISC‑V implementations[^13][^10].

Table 3 contrasts OpenSBI firmware flavors with scenarios where each is most appropriate.

Table 3: OpenSBI firmware flavors and usage

| Flavor | Build Control | Handoff Mechanism | Pros | Cons | Typical Use |
|--------|---------------|-------------------|------|------|-------------|
| FW_PAYLOAD | OpenSBI built with embedded payload path | Monolithic image; direct entry after OpenSBI init | Single binary, simple loading | Requires rebuilding OpenSBI; not flexible if firmware is ROM-resident | Embedded systems with tightly controlled images[^11] |
| FW_JUMP | OpenSBI configured with jump address | Fixed jump to next stage after init | Flexible placement, no payload embedding | Still requires OpenSBI rebuild per target | Lab environments with fixed memory layout[^11] |
| FW_DYNAMIC | Prior stage provides fw_dynamic_info | Dynamic next address/mode via struct | Highest flexibility; platform firmware controls placement | Requires coordination with ZSBL/platform | General-purpose testing and multi-platform support[^10][^11] |

## UEFI vs BIOS: Architecture, Security, and Capabilities

UEFI was designed to replace legacy PC‑AT BIOS constraints with a modern, extensible interface. The comparison touches memory models, partitioning, driver architecture, and security.

- Memory and mode: BIOS operates in 16-bit real mode with a 1 MB memory window; UEFI operates in 32-bit or 64-bit mode with larger address spaces and identity-mapped paging from entry[^2][^3].
- Partitioning: BIOS typically uses MBR (up to four primary partitions and 2.2 TB disks), while UEFI uses GPT (up to 128 partitions and exabyte-scale disks)[^2][^3].
- Driver model and option ROMs: BIOS relies on legacy option ROMs; UEFI defines a driver model and optional EFI Byte Code (EBC) interpreter to enable portable drivers and modular firmware[^1][^2].
- Network boot: UEFI’s preboot environment includes extensive protocols (HTTP, TCP/IP, DNS, PXE), while BIOS-based network boot is minimal and often vendor-specific[^1][^2].
- Security: UEFI Secure Boot, with PK/KEK/db/dbx, enforces signing and allows revocation; BIOS lacks an inherent secure boot capability[^2][^3].

Table 4 synthesizes the most salient differences for educational decision-making.

Table 4: BIOS vs UEFI feature comparison

| Feature | BIOS (Legacy) | UEFI |
|---------|----------------|------|
| Processor mode | 16-bit real mode; limited addressing | 32/64-bit; larger address spaces[^2][^3] |
| Partitioning | MBR; ≤4 primary partitions; ≤2.2 TB | GPT; ≤128 partitions; up to ~18 exabytes[^2][^3] |
| Boot image size | 512-byte boot sector | PE/COFF app; arbitrary size[^2] |
| Boot device discovery | MBR + INT 13h | ESP + System Table + Boot Services[^1][^2] |
| Driver model | Legacy option ROMs | UEFI Driver Model; optional EBC[^1] |
| Network protocols | Basic PXE | HTTP Boot, TCP/IP stack in firmware[^1] |
| Security | Minimal (passwords) | Secure Boot (PK/KEK/db/dbx)[^2][^3] |
| Runtime services | None standardized | Runtime Services persist after boot[^1][^2] |

For teaching environments, UEFI’s rich services and standardized interfaces offer greater productivity, but BIOS is still a valuable pedagogical tool for learning low-level initialization. In many cases, course designers use BIOS to introduce fundamentals before transitioning to UEFI workflows[^4][^2].

## Boot Protocols and Handoff Standards

Boot protocols define how information is passed from firmware or bootloader to the kernel, minimizing assumptions about hardware and reducing platform-specific code paths.

Multiboot2 is a widely used standard in education that defines a header with magic number and tags inside the first 8 KB of the kernel image. The bootloader places a structured information record—tags covering memory maps, ELF sections, framebuffer details, and boot modules—at a physical pointer passed to the kernel via registers (EAX=multiboot2 magic, EBX=info pointer on x86). This creates a clean interface for bootloaders like GRUB and supports cross-platform kernels where the same binary can be booted by different firmware stacks[^5].

UEFI abstracts firmware into Boot Services and Runtime Services and passes the System Table to the OS loader. The loader discovers console I/O, simple file system access, and block I/O via protocols. On exit from Boot Services, UEFI hands the final memory map and the responsibility for runtime services to the OS. This provides a structured interface with a well-defined lifecycle[^1][^2].

ACPI (Advanced Configuration and Power Interface) complements boot by defining power and performance objects and system wake/resume paths, including S-states and D-states. ACPI S-states influence boot resume behavior: waking from S2/S3/S4 can begin execution at the boot location and rely on firmware resume vectors; S5 is a complete boot. While ACPI is not a boot protocol per se, its mechanisms are integral to the system’s behavior around sleep/wake cycles and therefore to how boot code prepares for resume and transition[^9].

Device Tree provides a hardware description tree compiled to a binary blob (DTB) with a structured header and tokens (FDT_BEGIN_NODE, FDT_PROP, FDT_END_NODE, FDT_END). Devicetree is essential on ARM and RISC‑V for describing memory, CPUs, and peripherals that are not otherwise enumerable. Passing a DTB at boot is the standard approach on both architectures[^12].

RISC‑V SBI formalizes a supervisor-level firmware API invoked via ecall, with registers encoding extension IDs and function IDs. OpenSBI implements this standard, offering console output and timer services while leaving memory management and scheduling to the OS kernel[^13][^10].

Table 5 summarizes the handoff structures per standard.

Table 5: Boot handoff structures and discovery mechanisms

| Standard | Discovery/Handoff Mechanism | Data Passed |
|----------|-----------------------------|-------------|
| Multiboot2 | Header in first 8 KB; bootloader provides info structure pointer via EBX (x86) | Memory map, ELF sections, framebuffer info, boot modules; architecture tags[^5] |
| UEFI | System Table passed to PE/COFF app; Boot Services protocols for device access; ExitBootServices hands memory map | Console, file systems, block I/O; runtime services; ACPI/SMBIOS tables via System Table[^1][^2] |
| ACPI | Tables and AML methods define device/system power management | System S-states, device D-states, wake/resume paths[^9] |
| Device Tree | DTB compiled from DTS; boot loaders pass DTB address in registers | Memory, CPUs, peripheral descriptions via device tree nodes and properties[^12] |
| RISC‑V SBI | ecall interface with EID/FID; OpenSBI implements | Console/timer services; environment info abstracted by firmware[^13][^10] |

## Cross-Architecture Boot Strategies and Portability

Educational OS projects often aim to run across at least two architectures to demonstrate architectural abstraction. Four strategies emerge as practical:

- UEFI as a cross-architecture foundation. UEFI supports x86, x86_64, ARM, AArch64, and even RISC‑V as machine types in PE/COFF images. While vendors may vary in completeness and features, UEFI’s specification encourages portable boot code and stable interfaces for OS loaders[^1][^2].
- Limine as a minimal, cross-architecture bootloader. Limine’s protocol is intentionally simple and supports BIOS and UEFI flows across architectures. Rust developers can consume the limine crate to parse protocol structures without tying to a specific firmware environment[^18][^19][^20].
- Devicetree as a portable hardware description. On ARM and RISC‑V, passing a DTB normalizes hardware discovery across diverse SoCs and boards, allowing a single OS kernel to adapt to different peripherals without hardcoding support[^12].
- RISC‑V SBI as a portable supervisor interface. OpenSBI’s implementation of SBI insulates kernels from low-level platform details and provides a standard set of services that can be invoked consistently across platforms[^13][^10].

Table 6 frames these strategies by architecture and complexity.

Table 6: Portability strategies matrix

| Strategy | x86_64 | ARM64 | RISC‑V | Complexity Trade-offs |
|----------|--------|-------|--------|-----------------------|
| UEFI | Widely available; mature tooling (OVMF) | Available via EDK2 on many platforms | Supported as machine type in spec; implementations vary | High firmware complexity; strong interfaces; Secure Boot considerations[^1][^8][^2] |
| Limine | BIOS and UEFI support | Cross-architecture bootloader | Cross-architecture bootloader | Simpler protocol; less standardized ecosystem than UEFI[^18][^19][^20] |
| Devicetree | N/A | Standard for non-ACPI platforms | Standard for non-ACPI platforms | Requires DTS authoring and DTB compilation; well-supported tooling[^12] |
| SBI | N/A | N/A | Standardized supervisor interface via OpenSBI | Minimal runtime services; requires understanding ecall and register conventions[^13][^10] |

UEFI is not a panacea: Secure Boot policies, vendor-specific firmware features, and divergent implementations can complicate portability. Nevertheless, for teaching structured boot interfaces, UEFI’s System Table and Boot/Runtime Services provide an excellent foundation[^1][^2]. On ARM and RISC‑V, Devicetree and SBI provide complementary mechanisms to decouple kernels from hardware details[^12][^13].

## Rust Bootloader Ecosystem and Development Practices

Rust’s growing presence in systems programming extends naturally to bootloaders and OS development. Several crates and toolchains help educators bootstrap projects:

- bootloader (rust-osdev): a pure-Rust x86_64 bootloader supporting BIOS and UEFI. It provides a BootInfo API, entry_point macro, and disk image creation utilities. Kernels compile as freestanding binaries and link against bootloader_api to receive memory maps and framebuffer information[^21].
- limine (rust crate): bindings to parse Limine boot protocol structures; pairs well with the limine bootloader and limage tool to produce bootable images. Example kernels require nightly Rust[^18][^20][^22].
- uefi and uefi-services: Rust crates that expose UEFI services in a developer-friendly way, enabling UEFI applications to be written in Rust without relying on the standard library. Testing commonly uses QEMU with OVMF firmware (EDK2)[^26][^25].
- EDK2 and GNU-EFI/POSIX-UEFI: even for Rust kernels, building a UEFI loader in C can be instructive; these ecosystems provide crt0, linker scripts, headers, and calling-convention wrappers to satisfy UEFI entry requirements[^16][^2].

Common patterns in Rust no_std OS development include:

- Defining the entry point as efi_main with the correct calling convention on x86_64 (matching Microsoft x64 ABI) or using Rust crates that abstract it.
- Providing a panic handler (#[panic_handler]) and avoiding std APIs, opting for log-style interfaces via uefi-services for console output[^26].
- Embedding metadata in ELF sections for bootloader communication (e.g., BootInfo pointers), ensuring the kernel remains relocatable and firmware-agnostic[^21].
- Building bootable disk images and running them in emulators (QEMU + OVMF for UEFI), using FAT ESP layouts with fallback filenames (EFI/BOOT/BOOTX64.EFI)[^25][^2].

Table 7 summarizes capabilities across Rust crates and tools relevant to bootloaders and OS loaders.

Table 7: Rust crates and tools capability matrix

| Crate/Tool | Architectures | Boot Modes | Entry/Protocol | Disk Image Support | Notes |
|------------|---------------|------------|----------------|--------------------|-------|
| bootloader (rust-osdev) | x86_64 | BIOS, UEFI | BootInfo API; entry_point macro | FAT images; runtime kernel loading | Pure Rust; comprehensive docs; disk creation helpers[^21] |
| limine (crate) | Cross-architecture | BIOS, UEFI | Limine protocol parsing | Works with limage tool | Simple protocol; example kernels require nightly[^18][^20][^22] |
| uefi | Cross-architecture (UEFI hosts) | UEFI | System Table; Boot/Runtime Services | N/A | Rust-friendly abstractions; integrates with uefi-services[^26] |
| uefi-services | Cross-architecture (UEFI hosts) | UEFI | Console, log integration | N/A | Logging and runtime support for UEFI apps[^26] |
| EDK2/OVMF | x86_64/ARM | UEFI | PE/COFF apps; System Table | ESP via FAT | Reference firmware; robust QEMU support[^16][^25] |
| GNU-EFI/POSIX-UEFI | x86/x86_64 | UEFI | C ABI wrappers; PE/COFF output | N/A | Useful for building UEFI loaders in C and interop[^2] |

## Design Patterns and Minimal Boot Stacks for Educational OS

Minimal boot stacks should prioritize clarity and reproducibility. The following patterns have proven effective for classroom and hobbyist use.

On x86_64:

- Start with BIOS and Multiboot2. The bootloader’s simplicity lets students implement real-mode entry, memory detection, and protected/long mode transitions before introducing UEFI’s complexity. Multiboot2 provides structured boot information, supporting early kernel diagnostics and framebuffer experiments[^5][^4].
- Transition to UEFI for modern workflows. Use QEMU + OVMF to build and test a Rust-based UEFI loader with uefi and uefi-services. Emphasize System Table usage, ExitBootServices timing, and memory reuse constraints. Discuss Secure Boot policies and demonstrate how signed loaders and key enrollment affect workflows[^25][^2][^26].

On ARM64:

- Begin with the rust-embedded Raspberry Pi tutorials. These step-by-step guides develop a minimal kernel, bringing up UART, the MMU, interrupts, and timers; early labs run in QEMU and later ones deploy to real hardware, giving students a guided pathway[^7].
- Progress to U‑Boot or EDK2/UEFI. For platforms with EDK2 support, boot via ESP and demonstrate command-line kernel loading and DTB passing; for U‑Boot, explore spin-table SMP and DTB placement, reinforcing Linux boot requirements on ARM[^6][^8].

On RISC‑V:

- Start with rust-embedded tutorials for RISC‑V and QEMU’s virt machine. These introduce boot-from-zero patterns and RISC‑V assembly, then extend to interrupt and timer code paths[^23].
- Use OpenSBI FW_DYNAMIC. Teach how ZSBL or platform firmware passes the next stage address and mode via fw_dynamic_info, and how to pass the Device Tree in a1. Emphasize the SBI ecall interface and the kernel’s responsibility for memory management and scheduling[^10][^11][^13].

Across architectures, adhere to three principles:

- Separate firmware-agnostic kernel code from firmware-specific loaders. Favor standard protocols (Multiboot2, UEFI System Table, Devicetree, SBI) to minimize coupling.
- Use emulator workflows to simplify testing. QEMU’s support for OVMF (UEFI), ARM FVP models, and RISC‑V virt makes repeatable lab exercises possible without specialized hardware[^25][^8][^23].
- Embed diagnostics early. Framebuffer or serial output from the kernel should be present in the first week, allowing students to validate boot and explore memory, interrupts, and timers incrementally[^7][^23].

Table 8 outlines a suggested minimal boot stack per architecture.

Table 8: Suggested minimal boot stack per architecture

| Architecture | Bootloader(s) | Kernel Format | Handoff Mechanism | Emulator Target |
|--------------|---------------|---------------|-------------------|-----------------|
| x86_64 (intro) | BIOS + custom loader or GRUB (Multiboot2) | ELF (Multiboot2 header) | EAX/EBX Multiboot2 info; memory map and framebuffer tags | QEMU (BIOS) |
| x86_64 (UEFI) | OVMF/EDK2 UEFI | PE/COFF | System Table; ExitBootServices; Runtime Services | QEMU + OVMF |
| ARM64 (bare metal) | Rust-embedded tutorials; no firmware | ELF | DTB passed by loader; UART/MMU bring-up | QEMU (raspi3/raspi4) |
| ARM64 (UEFI) | EDK2/UEFI | PE/COFF | System Table; DTB via kernel cmdline or loader | Arm FVP or Juno |
| RISC‑V | OpenSBI (FW_DYNAMIC) | ELF | a0/a1 registers; DTB; SBI calls | QEMU virt |

## Security Considerations (Secure Boot, Measured Boot, Power Management)

Security concerns increasingly shape boot design. UEFI Secure Boot enforces cryptographic verification of boot applications and drivers, managed via PK (Platform Key), KEK (Key Exchange Keys), and allow/deny lists (db/dbx). In practice, educational environments may disable Secure Boot or rely on signed intermediaries (e.g., shim loaders) to maintain flexibility while preserving chain-of-trust concepts[^2][^3].

Measured Boot builds on the Trusted Platform Module (TPM) and TCG EFI protocols to record measurements of boot events into Platform Configuration Registers (PCRs). This enables attestation and can detect tampering in early boot components. While measured boot requires additional firmware and OS support, the core idea is straightforward: hash each boot step and record the measurement, forming a chain of trust anchored in the TPM[^28].

ACPI’s role in system wake and resume interacts with boot flows, particularly for S2–S4 states where the processor begins execution at the boot location and firmware coordinates resume vectors. For teaching, ACPI reinforces the boundary between OS power management (OSPM) and firmware runtime services, emphasizing that boot code must account for transitions that do not start from a full cold boot[^9].

Table 9 frames a security feature matrix across boot flows and highlights implications for educational labs.

Table 9: Security feature matrix and lab implications

| Feature | BIOS | UEFI | Measured Boot (TCG/TPM) |
|---------|------|------|--------------------------|
| Chain of trust | Weak (unsigned MBR sector) | Strong via Secure Boot (PK/KEK/db/dbx) | Strong via PCR measurements; attestation possible |
| Signature enforcement | None inherent | Required when Secure Boot enabled | Independent of Secure Boot; records measurements |
| Key management | N/A | Platform keys and enrollment; vendor policies | TPM provisioning; platform or OS responsibility |
| Lab implications | Easy to test unsigned kernels | May require disabling Secure Boot or using signed intermediaries | Requires TPM hardware and firmware support; more advanced labs[^2][^3][^28] |

## Tooling, Testing, and Debugging Workflows

Effective pedagogy depends on reliable tooling and repeatable workflows. The following environments are recommended:

- QEMU + OVMF for UEFI testing. OVMF is EDK2’s reference firmware for x86_64 and provides a practical UEFI environment. Emulator workflows enable FAT virtual disks as ESPs and logging to IO ports for debugging. Students can launch kernels via the fallback filename BOOTX64.EFI and use efibootmgr or UEFI Shell to manipulate boot order[^25][^2].
- ARM development models. Arm’s Fixed Virtual Platform (FVP) and Juno platforms support EDK2 firmware and OpenEmbedded boot. Tutorials outline building firmware, preparing USB sticks with kernel Image and DTB, and using startup.nsh scripts to parameterize console and root filesystem choices[^8].
- RISC‑V QEMU virt and OpenSBI. The riscv64 virt machine provides a canonical environment for OpenSBI, with fw_dynamic_info and DTB passing via a1. Serial consoles and GDB stubs support step-by-step debugging, allowing students to trace from ZSBL to kernel entry[^10][^23].

Debugging tips across environments:

- On UEFI, disable or reset watchdog timers to avoid resets during long initialization; use SetWatchdogTimer calls or OS-provided wrappers[^2].
- Use serial/UEFI console logging early; build logging into the kernel’s early boot sequence to confirm memory map handling and ExitBootServices timing[^26].
- For BIOS bootloaders, prefer virtual machines for debugging; mixing 16-bit and 32-bit code without proper breakpoints makes bare-metal debugging challenging[^4].
- For RISC‑V, instrument OpenSBI and kernel entry points; verify a0 and a1 values, ensure SATP is zeroed, and confirm DTB header structure before parsing[^10][^12].

Table 10 summarizes recommended toolchains and emulator configurations.

Table 10: Recommended toolchains/emulators per architecture

| Architecture | Emulator/Model | Firmware | Loader/App | Kernel Format | Debug Interfaces |
|--------------|-----------------|----------|------------|---------------|------------------|
| x86_64 | QEMU | OVMF (EDK2) | UEFI app (PE/COFF) | ELF/PE | QEMU debugcon; GDB; UEFI logs[^25][^2] |
| ARM64 | Arm FVP/Juno | EDK2/UEFI | UEFI app (PE/COFF) | Image + DTB | Arm DS; UEFI shell; serial[^8] |
| RISC‑V | QEMU virt | OpenSBI | FW_* payloads | ELF | GDB stub; serial; OpenSBI logs[^10][^23] |

## Recommendations and Roadmap for an Educational OS in Rust

A phased roadmap helps instructors and students build competence without overwhelming them:

- Phase 1: x86_64 BIOS + Multiboot2. Start with real-mode boot sectors, memory detection, and protected mode. Introduce Multiboot2 headers and tags to provide structured boot information. Emphasize hand-rolled loader development and minimal kernels to print to VGA or serial[^5][^4].
- Phase 2: x86_64 UEFI. Transition to a UEFI OS loader in Rust using uefi and uefi-services, building and testing with QEMU + OVMF. Discuss System Table discovery, Boot Services usage, ExitBootServices timing, and memory map reuse. Introduce Secure Boot concepts and demonstrate key enrollment and signature checks[^25][^26][^2][^3].
- Phase 3: ARM64 bare metal to EDK2/UEFI. Use rust-embedded tutorials to bring up MMU, interrupts, and timers. Progress to EDK2/UEFI boot flows on platforms where available, passing DTBs and kernel parameters. Explore U‑Boot spin-table and SMP bringup as an alternative path[^7][^6][^8].
- Phase 4: RISC‑V with OpenSBI. Boot on QEMU virt with OpenSBI FW_DYNAMIC; pass the DTB in a1 and verify SBI usage. Use rust-embedded RISC‑V tutorials to extend the kernel with interrupts and timers. Discuss privilege levels and the SBI ecall interface[^10][^23][^13].

Rust-first practices:

- Use bootloader (rust-osdev) on x86_64 for BIOS/UEFI to simplify disk image creation and to consume BootInfo. Leverage uefi and uefi-services for pure-Rust UEFI loaders and integrate logging early[^21][^26].
- Consider Limine for cross-architecture tests where a minimal protocol is preferred. Use limage to build boot images and limine crate bindings to parse protocol structures[^18][^20][^22].
- Keep kernels freestanding (no_std), provide panic handlers, and use ELF metadata for bootloader communication where appropriate[^21][^26].

Assessment criteria should emphasize:

- Boot reliability across emulators and hardware.
- Secure Boot interoperability and key management understanding on UEFI systems.
- Cross-architecture portability via standard handoff structures (Multiboot2, UEFI System Table, Devicetree, SBI).
- Debugging competency using serial/console logging, watchdogs, and GDB.

Table 11 provides a practical curriculum mapping.

Table 11: Curriculum map for a year-long educational OS course

| Phase | Weeks | Topics | Labs | Tooling | Assessments |
|-------|-------|--------|------|---------|-------------|
| 1: x86_64 BIOS + Multiboot2 | 4–6 | Real mode, A20, GDT/IDT, INT 13h; Multiboot2 | Implement boot sector; detect memory; load kernel via Multiboot2 | QEMU (BIOS), GRUB | Boot logs; memory map correctness; protected mode switch |
| 2: x86_64 UEFI | 4–6 | System Table; Boot Services; ExitBootServices; Secure Boot | Build Rust UEFI loader; parse memory map; handle watchdog; Secure Boot demo | QEMU + OVMF; uefi/uefi-services | BootServices usage; memory reuse correctness; Secure Boot discussion |
| 3: ARM64 bare metal to UEFI | 6–8 | MMU, interrupts, timers; DTB usage; spin-table; EDK2/UEFI | Rust-embedded tutorials; EDK2 boot on FVP/Juno; U‑Boot flow | QEMU (raspi), Arm FVP/Juno | DTB parsing; SMP bring-up; UEFI boot script correctness |
| 4: RISC‑V OpenSBI | 4–6 | SBI ecall; fw_dynamic_info; DTB header; SATP | QEMU virt boot; a0/a1 validation; SBI console/timer | QEMU virt; OpenSBI | DTB validation; SBI calls; kernel entry correctness |

## A brief note on information gaps

Two areas require further empirical investigation for a complete pedagogy:

- Limine’s formal specification and cross-architecture coverage should be confirmed against the upstream protocol document and implementation notes; classroom adoption would benefit from concrete hardware matrices[^18][^19][^20].
- RISC‑V Boot and Runtime Services (BRS) is an emerging specification; its status and implications for portable firmware services should be tracked for future course updates[^15].

Additionally, measured boot workflows with TPM integration and measured-launch documentation vary across platforms; lab exercises would benefit from step-by-step guides tailored to common emulators and hardware.

## References

[^1]: UEFI Specification 2.10: Introduction. UEFI Forum. https://uefi.org/specs/UEFI/2.10/01_Introduction.html  
[^2]: UEFI - OSDev Wiki. https://wiki.osdev.org/UEFI  
[^3]: Comparing BIOS vs UEFI: Which is the better Boot in 2025? CyberPanel. https://cyberpanel.net/blog/bios-vs-uefi  
[^4]: Rolling Your Own Bootloader - OSDev Wiki. http://wiki.osdev.org/Rolling_Your_Own_Bootloader  
[^5]: Multiboot Specification - OSDev Wiki. https://wiki.osdev.org/Multiboot  
[^6]: ARM64 — Das U-Boot documentation (v2022.04). https://docs.u-boot.org/en/v2022.04/arch/arm64.html  
[^7]: rust-embedded: Raspberry Pi OS Tutorials (ARM64). https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials  
[^8]: Booting OpenEmbedded using EDK II UEFI - Arm Developer. https://developer.arm.com/documentation/102158/0101/Booting-OpenEmbedded-using-EDK-II-UEFI  
[^9]: ACPI Specification 6.5: Power and Performance Management. UEFI. https://uefi.org/specs/ACPI/6.5/07_Power_and_Performance_Mgmt.html  
[^10]: RISC-V SBI and the full boot process. Uros Popovic. https://popovicu.com/posts/risc-v-sbi-and-full-boot-process/  
[^11]: OpenSBI - RISC-V Open Source Supervisor Binary Interface (GitHub). https://github.com/riscv-software-src/opensbi  
[^12]: An Introduction to Devicetree specification. https://krinkinmu.github.io/2021/01/17/devicetree.html  
[^13]: RISC-V Supervisor Binary Interface Specification. https://github.com/riscv-non-isa/riscv-sbi-doc  
[^14]: RISC-V Kernel Boot Requirements and Constraints. https://docs.kernel.org/arch/riscv/boot.html  
[^15]: RISC-V Boot and Runtime Services (BRS) Specification (Draft). https://lists.riscv.org/g/tech-server-soc/attachment/110/0/riscv-brs-spec-draft.pdf  
[^16]: EDK2 Project (OVMF). https://github.com/tianocore/edk2  
[^17]: UEFI Specifications | UEFI Forum. https://uefi.org/specifications  
[^18]: limine crate - crates.io. https://crates.io/crates/limine  
[^19]: Limine Boot Protocol Specification. https://github.com/limine-bootloader/limine/blob/trunk/PROTOCOL.md  
[^20]: limage - crates.io. https://crates.io/crates/limage  
[^21]: rust-osdev/bootloader: Rust bootloader for x86_64 (BIOS & UEFI). https://github.com/rust-osdev/bootloader  
[^22]: rust-osdev/bootimage: Tool to create bootable disk images. https://github.com/rust-osdev/bootimage  
[^23]: rust-embedded: RISC-V OS Tutorials. https://github.com/rust-embedded/rust-embedded  
[^24]: UEFI Forum Specifications. https://uefi.org/specifications  
[^25]: OVMF (Open Virtual Machine Firmware) - TianoCore Wiki. https://github.com/tianocore/tianocore.github.io/wiki/OVMF  
[^26]: OS Experiment in Rust (Part 1): Creating a UEFI Loader. https://blog.malware.re/2023/08/20/rust-os-part1/index.html  
[^27]: Device Tree Specification v0.3. https://github.com/devicetree-org/devicetree-specification/releases/download/v0.3/devicetree-specification-v0.3.pdf  
[^28]: TCG EFI Protocol Specification. Trusted Computing Group. https://trustedcomputinggroup.org/resource/tcg-efi-protocol-specification/