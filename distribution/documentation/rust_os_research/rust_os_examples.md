# Rust-based Operating Systems: Memory Safety, Kernel Design, and Practical Implementation in Redox OS, Theseus, and Tock

## Executive Summary

Three prominent Rust-based operating systems—Redox OS, Theseus, and Tock—demonstrate materially different strategies for achieving memory safety and structuring kernel code. These divergences are not accidents of implementation; they are architectural choices shaped by each project’s target domain, assumptions about hardware isolation, and stance on the role of the compiler versus the operating system.

- Redox OS is a Unix-like, general-purpose operating system built on a microkernel, with system services in user space. Its safety model combines Rust’s guarantees with traditional hardware isolation: kernel code is written in Rust and runs in kernel mode, while services run in user space behind a conventional syscall boundary. On x86-64, Redox uses the fast syscall/sysretq ABI to switch between user and kernel contexts.[^1][^2]

- Theseus is an intralingual operating system designed from first principles to let the compiler understand the entire system. It runs all components—including kernel and applications—in a single address space (SAS) and a single privilege level (SPL). Isolation comes from Rust’s type and memory safety, reinforced by a tightly engineered memory abstraction (MappedPages) and dependency metadata that enables live evolution and fault recovery.[^3]

- Tock is an embedded real-time operating system (RTOS) oriented around mutual distrust: applications and drivers must be isolated from each other and from the kernel. Tock combines Rust’s compile-time safety with hardware Memory Protection Units (MPUs) for spatial isolation on resource-constrained microcontrollers, enforcing separation between core kernel, chips (hardware support), boards (platforms), and capsules (system software).[^4][^5]

The projects’ strategic trade-offs are stark:

- Language-first isolation (Theseus) maximizes compile-time enforcement and enables advanced capabilities like live evolution and intralingual recovery, but requires rethinking OS structure and relying on strong language invariants across the codebase.

- Hardware-assisted isolation (Tock) aligns with microcontroller constraints, leveraging MPUs for robust boundaries where the language cannot fully enforce separation. This results in practical isolation but demands careful conventions to avoid unsafe aliasing in event-driven code.

- Microkernel + language safety (Redox) keeps the kernel small and moves services to user space, using traditional hardware boundaries and a stable syscall ABI. It provides a familiar, portable model but inherits the complexity and overhead of cross-address-space communication.

These lessons translate into actionable implementation strategies: when the language runtime and program structure can be controlled end-to-end, an intralingual, single-address-space design provides superior safety and evolvability; when hardware and ecosystem constraints dominate (e.g., microcontrollers with MPU only), combine Rust’s guarantees with MPU-based isolation and conservative driver conventions; and for general-purpose systems where POSIX-like expectations and application ecosystems matter, a microkernel with user-space services and Rust inside the kernel remains a pragmatic path.[^1][^2][^3][^4][^5]

## Methodology and Scope

This report synthesizes information from official documentation, academic papers, and source repositories. For Theseus, the OSDI 2020 paper is the canonical source for architecture, memory management, state management, and the intralingual design approach. Redox OS’s book provides the system design and kernel documentation, including syscall conventions on x86-64 and its repository structure. Tock’s official website and Getting Started guide present its architecture, isolation model, codebase organization, and development workflow. The early Tock experience paper (PLOS 2015) offers detailed lessons about applying Rust’s ownership model in embedded systems, including challenges and proposed language extensions.[^1][^3][^4][^5]

The inclusion criteria prioritize primary technical documentation and peer-reviewed research; references are consolidated to the minimum needed for each claim. Where a topic is not covered by the consulted sources, the report acknowledges gaps.

## Architectural Foundations: Roles of Language and Hardware

These three systems occupy different points on the spectrum from language-first to hardware-first isolation. Their architectural foundations reflect their goals.

- Redox OS organizes functionality in the microkernel tradition: the kernel provides minimal services (scheduling, IPC, memory management), while system servers run in user space. The design philosophy embraces a Unix-like API and Plan 9-inspired “schemes” for resource handling, with everythingPresented-as-a-file where practical.[^1]

- Theseus inverts the traditional model. It runs all software in one address space at a single privilege level, relying on Rust’s type and memory safety for isolation. The OS is designed around intralingual principles that empower the compiler to enforce invariants, minimize state spill, and enable live evolution and recovery.[^3]

- Tock targets microcontrollers where an MMU is often absent and resource budgets are tight. It uses Rust to protect the kernel and drivers, isolates applications and drivers from each other with MPUs, and structures the kernel to enforce clear trust boundaries—core kernel, system software capsules, chip-specific hardware abstraction, and boards.[^4][^5]

To situate these differences concretely, Table 1 compares architectural models, isolation mechanisms, and typical components across the three systems.

### Table 1: Architectural Model vs Isolation Mechanism vs Typical Components

| OS       | Architectural Model                            | Primary Isolation Mechanism                             | Typical Components/Subsystems                                        |
|----------|-------------------------------------------------|----------------------------------------------------------|-----------------------------------------------------------------------|
| Redox    | Microkernel with system services in user space  | Hardware address spaces + syscall boundary; kernel in Rust| Kernel (scheduling, IPC, memory), user-space servers, drivers, RedoxFS, graphics/windowing[^1] |
| Theseus  | Single Address Space (SAS), Single Privilege Level (SPL), intralingual | Language/type/memory safety via Rust; MappedPages; dependency metadata | Cells (crates/object files at runtime), dynamic loader/linker, cell metadata, unwinder, task lifecycle[^3] |
| Tock     | RTOS for embedded MCUs; capsules model          | MPU-based spatial isolation; Rust kernel/driver isolation | Kernel (scheduler, core), capsules (system services), chips (HAL), boards (platforms), applications[^4][^5] |

These foundational choices drive downstream design differences in memory management, concurrency models, and how safely to evolve or recover from faults.

### Redox OS: Microkernel Foundations

Redox’s kernel is small and modular, providing only essential services. System functionality—filesystems, networking, windowing—runs in user space, communicating with applications and the kernel via well-defined schemes and IPC. The kernel’s syscall ABI on x86-64 uses the syscall instruction to enter kernel mode (ring 0) and sysretq to return to user mode (ring 3). The design emphasizes modularity and system services outside the kernel, reinforcing isolation through hardware privilege and address spaces.[^1][^2]

### Theseus: Intralingual Single-Address-Space OS

Theseus is designed so that the compiler can view and enforce OS-level invariants. All components live in one address space and one privilege level. Isolation comes from Rust’s guarantees, and the OS employs a memory abstraction (MappedPages) that makes all memory access type-safe, bounds-checked, and subject to drop-based cleanup. Runtime dependency metadata enables precise tracking of inter-cell relationships and supports advanced features like cell swapping for live evolution and unwinding-based fault recovery.[^3]

### Tock: Embedded RTOS with MPU-backed Isolation

Tock assumes resource-constrained devices without MMUs, where MPUs provide spatial isolation between applications, drivers, and the kernel. The kernel and drivers are written in Rust to enforce memory and type safety, while capsules provide system services atop hardware support layers (chips) and board configurations. The result is a protection-centric architecture where Rust’s compile-time checks and MPU policies work together to contain faults and limit the blast radius of buggy or malicious components.[^4][^5]

## Memory Safety Approaches and Mechanisms

These systems leverage Rust’s compile-time guarantees in different ways, augmented by hardware mechanisms where appropriate.

- Theseus pushes the furthest: isolation is language-first, with memory abstraction built on MappedPages enforcing bijective virtual-to-physical mapping, bounds checks, exactly-once unmapping via Drop, and type-level guarantees for mutability and executability (MappedPagesMut and MappedPagesExec). Resource cleanup is handled through unwinding and Drop handlers, preventing leakage and enabling static deadlock prevention for locks.[^3]

- Tock relies on Rust to protect the kernel and drivers from each other and from untrusted code, while using MPUs to isolate applications and drivers spatially. In practice, event-driven concurrency and hardware resource sharing lead to careful conventions—e.g., interrupt handlers only enqueue work rather than perform sharing directly—to avoid unsafe aliasing and data races. Static lifetimes are used to share hardware resources, trading some compile-time race detection for practical embedded patterns.[^4][^5]

- Redox runs kernel code in Rust and services in user space, with isolation reinforced by the syscall boundary and hardware protections. The Redox documentation highlights memory management, but the specific page does not provide detailed implementation details; the kernel architecture and microkernel design remain the reference points for its safety approach.[^1][^2]

To make the differences explicit, Table 2 summarizes the memory safety mechanisms, protection layers, and isolation granularity.

### Table 2: Comparative Memory Safety Matrix

| OS       | Language-level guarantees | Hardware protections      | Isolation granularity                         | Cleanup strategy                                 |
|----------|---------------------------|---------------------------|-----------------------------------------------|--------------------------------------------------|
| Redox    | Rust kernel code          | Syscall boundary, rings   | Per-process address spaces; user/kernel split | Conventional kernel/user cleanup; unspecified in consulted docs[^1][^2] |
| Theseus  | Rust type/memory safety; MappedPages invariants | None (SAS/SPL)            | Per-section dependency metadata; cell bounds   | Drop-based cleanup via unwinding; single allocator[^3] |
| Tock     | Rust kernel/driver isolation | MPU policies              | Per-application/driver spatial isolation       | Rust ownership; conservative ISR conventions[^4][^5] |

### Theseus: Language-Level Isolation and MappedPages

MappedPages is Theseus’s fundamental abstraction for memory. It guarantees bijective mapping between virtual pages and physical frames, preventing covert aliasing beneath the language level. Access is only possible through methods that check bounds, preserve lifetimes, and ensure that mutability and executability are restricted to appropriately typed mappings. Drop semantics guarantee exactly-once unmapping after outstanding references expire. By unifying stacks, heaps, device memory, and loaded cells under a single, lossless interface, Theseus shifts much of the safety work to the compiler while retaining the ability to reason precisely about OS-level invariants.[^3]

### Tock: Rust Ownership and MPU-based Isolation

Tock’s embedded context demands spatial isolation with minimal runtime overhead. Rust enforces memory and type safety in the kernel and drivers, but the ownership model can conflict with event-driven concurrency and always-present hardware resources. Tock uses static lifetimes to share resources safely and adopts conservative conventions for interrupt handlers, which enqueue work rather than share mutable state directly, minimizing the risk of data races. MPUs enforce isolation between applications and between kernel components, containing faults and reducing the trusted computing base.[^4][^5]

## Kernel Design Patterns and Code Organization

Design patterns and code organization reflect each project’s architectural choices and development workflows.

- Redox’s repository structure uses GitLab with a root project that orchestrates Makefiles and submodules. Recipes in the Cookbook define how to build and package software for inclusion in Redox images, supporting Rust and C/C++ projects, dynamic/static linking, and cross-compilation through relibc. Contributions occur via public personal forks and merge requests, emphasizing reproducible builds and clear packaging workflows.[^6]

- Theseus decomposes the OS into many tiny components—“cells”—that exist as Rust crates at implementation time, as object files at compile time, and as loaded sections with metadata at runtime. Dynamic loading and linking occur at runtime, with per-section dependency tracking enabling live evolution and fault recovery. The design balances fine-grained modularity with manageability: a flat set of distinct object files, organized hierarchically in source, allows precise metadata-driven operations without heavyweight monolithic structures.[^3]

- Tock organizes code into /kernel (core), /capsules (system services), /chips (hardware support), and /boards (platform support). The project is structured to isolate drivers and system services from the kernel and to separate platform-specific code, facilitating portability and controlled trust boundaries. Documentation and build guidance orient new contributors quickly to these directories and the protective mechanisms of the system.[^4][^5]

Table 3 summarizes code organization, build systems, and extension mechanisms.

### Table 3: Code Organization and Build System Comparison

| OS       | Repository layout                        | Build system                      | Extension mechanism                               |
|----------|------------------------------------------|-----------------------------------|----------------------------------------------------|
| Redox    | GitLab root + submodules; Cookbook recipes| Make + Cargo; relibc; cross-compile | Recipes for packaging; user-space services; schemes[^6][^1] |
| Theseus  | Many tiny crates; runtime-linked object files | Custom runtime loader/linker; DWARF-based unwinder | Cells with metadata; namespaces; dynamic loading[^3] |
| Tock     | /kernel, /capsules, /chips, /boards      | Rust toolchain; board/chip-specific builds | Capsules; MPU policies; driver isolation[^4][^5] |

## Architecture Support and Platform Portability

Support across architectures and platforms varies by design goals.

- Redox targets x86-64 and documents boot processes, memory management, scheduling, and drivers. Its microkernel architecture lends itself to portability, but the consulted materials primarily describe x86-64 details. Broader architecture coverage is referenced conceptually and would require deeper inspection of specific build documentation.[^1]

- Theseus is implemented on x86_64 and includes multicore, preemptive multitasking, and I/O subsystems, with ongoing work exploring correctness properties and driver verification. Its intralingual design is inherently portable in principle but is currently documented with an x86_64 focus.[^3]

- Tock supports ARM Cortex-M platforms and provides ports to RISC-V as part of its ecosystem. MPU-based isolation is the standard protection mechanism, and the codebase is organized to accommodate multiple chips and boards.[^4][^5]

Table 4 summarizes supported architectures and boot/load mechanisms.

### Table 4: Architecture Support Matrix

| OS       | Supported ISAs (from consulted sources) | Boot/Load model                       | Protection mechanisms            |
|----------|------------------------------------------|---------------------------------------|----------------------------------|
| Redox    | x86-64 documented                        | Boot process documented; microkernel  | Syscall boundary; user/kernel rings[^1][^2] |
| Theseus  | x86_64                                   | nano_core bootstrap; runtime loading  | Language-based (SAS/SPL)[^3]     |
| Tock     | ARM Cortex-M; RISC-V ecosystem           | Embedded boot; board/chip setup       | MPU-based isolation[^4][^5]      |

## Development Patterns, Tooling, and Workflows

Development workflows are shaped by each system’s architecture and packaging philosophy.

- Redox emphasizes reproducible, recipe-driven builds. Cookbook templates cover Cargo, configure (Autotools), CMake, and Meson; a remote template pulls prebuilt packages; and a custom template enables arbitrary scripts. Cross-compilation is the default, with relibc as the C standard library. Environment variables and helper functions streamline complex builds. Contributions flow through public forks and merge requests, with SHA-pinned submodules ensuring build determinism.[^6]

- Theseus’s workflow is metadata-driven. The dynamic loader builds LoadedCrate and LoadedSection metadata, tracking bidirectional dependencies at a per-section granularity. Cell swapping—used for live evolution—depends on precise dependency verification, symbol map management, and runtime rewriting of relocations. The unwinder provides a reliable recovery mechanism even in exceptional control paths, making cleanup robust and static deadlock prevention practical for lock guards.[^3]

- Tock’s workflow centers on clear directory roles and hardware isolation. Contributors set up the Rust toolchain, build for specific boards/chips, and write capsules that extend system services while remaining isolated from the kernel and each other via MPUs. Documentation emphasizes how Rust protects the kernel from drivers and vice versa, and how MPU policies enforce application isolation.[^4][^5]

Table 5 outlines development workflow elements.

### Table 5: Development Workflow Elements

| OS       | Workflow highlights                                   |
|----------|--------------------------------------------------------|
| Redox    | Recipe-driven builds; cross-compilation; relibc; public forks + MRs; SHA-pinned submodules[^6] |
| Theseus  | Runtime metadata; dependency verification; unwinding; cell swapping for live evolution[^3] |
| Tock     | Rust toolchain; board/chip builds; MPU isolation; capsules as extensions[^4][^5] |

## Cross-System Comparative Analysis

These systems exemplify distinct points in the design space of OS safety and structure. At one end, Theseus demonstrates how much can be achieved when the OS is built around the language’s strengths and the compiler is given the information and authority to enforce invariants. In the middle, Tock shows how to combine Rust with MPU-based isolation to meet the constraints of microcontrollers and achieve practical containment. At the other end, Redox shows how to retain traditional, general-purpose OS structure while using Rust to harden the kernel, with system services isolated in user space.

### Memory Safety Strategies

MappedPages in Theseus is emblematic of language-first memory safety: the abstraction enforces bijection, bounds, and drop semantics, while making all sharing explicit at the type level. This reduces state spill, allows the compiler to validate use, and enables robust recovery and evolution.[^3]

Tock must bridge the gap between Rust’s ownership model and embedded reality. Static lifetimes allow sharing of hardware resources; MPUs enforce spatial isolation; and conservative ISR conventions prevent races at the cost of some compile-time guarantees. The result is a practical safety posture appropriate for low-resource, event-driven systems.[^4][^5]

Redox uses Rust in the kernel and keeps services in user space, relying on hardware boundaries for coarse-grained isolation. The approach is familiar and portable, aligning with existing ecosystems and POSIX-like expectations while minimizing the amount of kernel code and maximizing the benefits of language safety where it matters most.[^1][^2]

### State Management and Modularity

Theseus applies intralingual design to state management: opaque exportation pushes client progress state to clients, eschews handles in favor of owned objects, and uses metadata to reduce cross-cell state spill. The unwinder and Drop mechanisms ensure cleanup in all paths, and the minimized Task struct keeps OS state out of central structures. This architecture enables live evolution and recovery with fewer dependencies and less fate sharing.[^3]

Redox’s microkernel approach keeps kernel state small and pushes system state into user-space servers, minimizing kernel complexity and isolating faults across address spaces. Schemes and resources model system functionality predictably, though cross-address-space communication introduces overhead and complexity in IPC.[^1]

Tock compartmentalizes system state through capsules and hardware support layers, relying on MPU policies to prevent unauthorized access and ensure component isolation. The structure supports extensibility with controlled trust boundaries, appropriate for embedded ecosystems where drivers and applications must be mutually distrustful.[^4][^5]

Table 6 compares state management strategies and their impact on fault containment.

### Table 6: State Management Strategies vs Fault Containment

| OS       | Strategy                                         | Impact on fault containment                      |
|----------|---------------------------------------------------|--------------------------------------------------|
| Redox    | Microkernel + user-space servers                  | Faults isolated across address spaces; IPC boundaries[^1] |
| Theseus  | Opaque exportation; metadata-driven cell swapping | Minimal state spill; precise recovery; live evolution[^3] |
| Tock     | Capsules + MPU policies                           | Spatial isolation contains faults; conservative ISR patterns[^4][^5] |

## Practical Implementation Strategies and Lessons Learned

A set of pragmatic strategies emerges from the three projects:

1. Decide isolation based on environment and control. If you can co-design the runtime and program structure, adopt intralingual, single-address-space design to let the compiler enforce invariants. If you must accommodate untrusted applications on constrained hardware, use Rust plus MPU-based isolation and conservative concurrency conventions. For broad ecosystem compatibility, a microkernel with Rust in the kernel and user-space services is a balanced approach.[^1][^3][^4][^5]

2. Embrace lossless interfaces and metadata. Theseus shows how preserving type, lifetime, and ownership across boundaries empowers the compiler and reduces state spill. Runtime metadata for dependencies and per-section bounds enables precise operations like cell swapping and robust unwinding-driven cleanup.[^3]

3. Design cleanup and recovery around Drop and unwinding. A system where resource release is implemented only in Drop handlers, and where unwinding is supported in core contexts, simplifies code, prevents leaks, and enables static deadlock prevention for locks and guard-like primitives.[^3]

4. Use metadata-driven evolution for availability. Cell swapping with bidirectional dependency verification and symbol map management is a powerful mechanism for live evolution, provided the OS knows the bounds and dependencies of each component precisely.[^3]

5. Match the language model to OS structure. Theseus aligns OS execution with Rust’s runtime expectations (single address space, single allocator) to maximize compile-time checks. Redox aligns with the traditional user/kernel split to align with existing toolchains and expectations. Tock aligns with embedded constraints and MPU capabilities.[^1][^3][^4][^5]

6. Organize code to reflect trust boundaries. Tock’s directory layout (/kernel, /capsules, /chips, /boards) is a model for separating core kernel, system services, hardware support, and platforms. Redox’s Cookbook recipes provide a reproducible packaging model that simplifies inclusion of user-space services and applications.[^4][^5][^6]

7. Exploit language features to minimize unsafe code. While unsafe is unavoidable for hardware access, Theseus demonstrates how to keep unsafe confined to small, obviously necessary areas and ensure that most code—including critical OS subsystems—remains safe by design.[^3]

8. Apply conservative concurrency patterns when needed. Embedded systems benefit from top-half/bottom-half patterns: interrupt handlers enqueue work; the main scheduler performs actual processing. This avoids unsafe aliasing and data races when compile-time guarantees are hard to retain under event-driven constraints.[^5]

The following checklists summarize when to prefer each model and the pitfalls to avoid.

### Table 7: Model Selection Guide

| Context                                   | Prefer                                           | Rationale                                                     |
|-------------------------------------------|--------------------------------------------------|---------------------------------------------------------------|
| Co-designed runtime, strong language control | Intralingual, single-address-space (Theseus-like) | Compile-time enforcement, reduced state spill, live evolution[^3] |
| Constrained MCUs, MPU-only                 | Rust + MPU isolation (Tock-like)                 | Spatial isolation, low overhead, practical driver separation[^4][^5] |
| General-purpose OS, ecosystem compatibility | Microkernel + user-space services (Redox-like)   | Familiar model, isolation via address spaces, stable ABIs[^1][^2] |

### Table 8: Pitfalls and Mitigations

| Pitfall                                                | Mitigation                                                   |
|--------------------------------------------------------|--------------------------------------------------------------|
| Unsafe aliasing in event-driven embedded code          | Adopt conservative ISR conventions; consider language extensions (execution contexts) proposed for Rust[^5] |
| State spill across OS boundaries                       | Use opaque exportation; metadata-driven dependency tracking; minimized central task state[^3] |
| Lossy interfaces across kernel/user boundaries         | Preserve type/lifetime/ownership; avoid handles; prefer owned objects that encode provenance[^3] |
| Leakage and inconsistent cleanup                       | Implement cleanup only in Drop; rely on unwinding for exceptional paths; use guard-based lock release[^3] |
| Overly coarse-grained isolation                        | Combine language safety with hardware isolation appropriate to domain (MPUs or address spaces)[^4][^5] |
| Build and packaging complexity                         | Use recipe-based workflows (Cookbook); pin sources via hashes; templates for Cargo/CMake/Meson[^6] |

## Recommendations and Strategic Guidance

When building a new Rust-based OS or kernel component, align isolation and structure with your domain and constraints:

- For embedded MCUs: follow Tock’s approach. Use Rust for kernel and drivers, apply MPU-based isolation for applications and drivers, and adopt conservative ISR/top-half patterns. Organize code with clear trust boundaries and system service capsules.[^4][^5]

- For safe-language OS experimentation or high-availability systems where you control the program model: adopt Theseus’s intralingual design. Use a single address space and single privilege level with language-first isolation, MappedPages-like abstractions, and metadata-driven cell swapping for live evolution and recovery.[^3]

- For general-purpose OS or Unix-like environments: use a microkernel architecture (Redox-like) with Rust in the kernel, system services in user space, and stable syscall ABIs. This provides familiarity and ecosystem compatibility while benefitting from language safety where it has the most leverage.[^1][^2]

Practical steps to operationalize these choices include:

- Define isolation policies early. Decide whether language-first (SAS/SPL), hardware-first (MPU or MMU), or hybrid isolation best fits your domain.

- Establish code organization that encodes trust boundaries. Separate kernel, drivers, system services, and platform support; avoid mixing responsibilities in single modules.

- Embrace lossless interfaces and metadata. Preserve language-level context across boundaries; use runtime metadata to track dependencies and enable safe evolution.

- Implement cleanup solely in Drop and support unwinding in core contexts. This avoids leakage and enables static deadlock prevention and robust recovery.

- Adopt reproducible build systems and packaging. Use recipe-driven workflows, pin sources, and provide clear templates for multiple build systems.

## Information Gaps and Limitations

This analysis is constrained by the consulted sources:

- Redox OS memory management details beyond high-level architecture were not available on the referenced documentation page.

- Comprehensive architecture support beyond x86-64 for Redox and Theseus was not covered in the extracted materials.

- Tock’s internal driver isolation and kernel capsule protocols are not fully documented in the extracted Getting Started pages.

- Quantitative performance metrics across the three systems were not available in the consulted sources.

- The most recent updates to Theseus’s verification and intralingual correctness work beyond the OSDI 2020 paper were referenced at a high level but not covered in depth.[^3]

These gaps suggest avenues for deeper investigation: detailed Redox memory management documentation, current Theseus verification efforts, and Tock’s internal driver model and capsule protocols.

## Appendices

### A. Glossary of Terms

- Single Address Space (SAS): A design where all software—applications and OS components—share one address space.

- Single Privilege Level (SPL): A design where all code runs at one privilege level, relying on language safety rather than hardware rings for isolation.

- Intralingual Design: An approach that implements OS semantics using language-level mechanisms so the compiler can enforce invariants.

- MappedPages: Theseus’s memory abstraction representing virtually contiguous pages mapped to physical frames with strong invariants.

- Cell: Theseus’s unit of modularity, existing as a crate (implementation), object file (compile time), and loaded sections with metadata (runtime).

- Capsule: In Tock, a system software component that provides services atop drivers and kernel, isolated via MPU and Rust safety.

- Memory Protection Unit (MPU): Hardware mechanism for enforcing memory access policies, used to isolate processes or components in constrained systems.

- State Spill: One component holding state on behalf of another, causing fate sharing and hindering modularity.

### B. Build and Porting References

- Redox Cookbook recipes and build templates (Cargo, configure, CMake, Meson, remote, custom) support cross-compilation and packaging with relibc. Environment variables and helper functions enable complex build orchestration; contributions flow through public forks and merge requests.[^6]

- Tock build system and directory organization (/kernel, /capsules, /chips, /boards) provide a structured path for extending and porting the OS to new hardware platforms.[^4][^5]

### C. Citation Mapping

- Redox OS architecture and syscall mechanism: kernel documentation and book sections.[^1][^2]
- Theseus architecture, MappedPages, and state management: OSDI 2020 paper.[^3]
- Tock OS architecture, isolation, and development workflow: website and Getting Started materials.[^4][^5]
- Redox repository structure and Cookbook recipes: documentation pages.[^6]

---

## References

[^1]: The Redox Operating System (Book). https://doc.redox-os.org/book/

[^2]: Redox kernel - The Redox Operating System. https://doc.redox-os.org/book/kernel.html

[^3]: Kevin Boos et al., Theseus: an Experiment in Operating System Structure and State Management (OSDI 2020). https://www.usenix.org/system/files/osdi20-boos.pdf

[^4]: Tock Embedded Operating System (Official Website). https://www.tockos.org/

[^5]: Getting Started - Tock Embedded Operating System. https://www.tockos.org/documentation/getting-started/

[^6]: Repository Structure - The Redox Operating System. https://doc.redox-os.org/book/repository-structure.html

---

## Appendix: Detailed Tables

For clarity, the main report references several comparative tables. The full content of those tables is reproduced below for quick reference.

### Table A1: Architectural Model vs Isolation Mechanism vs Typical Components

| OS       | Architectural Model                            | Primary Isolation Mechanism                             | Typical Components/Subsystems                                        |
|----------|-------------------------------------------------|----------------------------------------------------------|-----------------------------------------------------------------------|
| Redox    | Microkernel with system services in user space  | Hardware address spaces + syscall boundary; kernel in Rust| Kernel (scheduling, IPC, memory), user-space servers, drivers, RedoxFS, graphics/windowing[^1] |
| Theseus  | Single Address Space (SAS), Single Privilege Level (SPL), intralingual | Language/type/memory safety via Rust; MappedPages; dependency metadata | Cells (crates/object files at runtime), dynamic loader/linker, cell metadata, unwinder, task lifecycle[^3] |
| Tock     | RTOS for embedded MCUs; capsules model          | MPU-based spatial isolation; Rust kernel/driver isolation | Kernel (scheduler, core), capsules (system services), chips (HAL), boards (platforms), applications[^4][^5] |

### Table A2: Comparative Memory Safety Matrix

| OS       | Language-level guarantees | Hardware protections      | Isolation granularity                         | Cleanup strategy                                 |
|----------|---------------------------|---------------------------|-----------------------------------------------|--------------------------------------------------|
| Redox    | Rust kernel code          | Syscall boundary, rings   | Per-process address spaces; user/kernel split | Conventional kernel/user cleanup; unspecified in consulted docs[^1][^2] |
| Theseus  | Rust type/memory safety; MappedPages invariants | None (SAS/SPL)            | Per-section dependency metadata; cell bounds   | Drop-based cleanup via unwinding; single allocator[^3] |
| Tock     | Rust kernel/driver isolation | MPU policies              | Per-application/driver spatial isolation       | Rust ownership; conservative ISR conventions[^4][^5] |

### Table A3: Code Organization and Build System Comparison

| OS       | Repository layout                        | Build system                      | Extension mechanism                               |
|----------|------------------------------------------|-----------------------------------|----------------------------------------------------|
| Redox    | GitLab root + submodules; Cookbook recipes| Make + Cargo; relibc; cross-compile | Recipes for packaging; user-space services; schemes[^6][^1] |
| Theseus  | Many tiny crates; runtime-linked object files | Custom runtime loader/linker; DWARF-based unwinder | Cells with metadata; namespaces; dynamic loading[^3] |
| Tock     | /kernel, /capsules, /chips, /boards      | Rust toolchain; board/chip-specific builds | Capsules; MPU policies; driver isolation[^4][^5] |

### Table A4: Architecture Support Matrix

| OS       | Supported ISAs (from consulted sources) | Boot/Load model                       | Protection mechanisms            |
|----------|------------------------------------------|---------------------------------------|----------------------------------|
| Redox    | x86-64 documented                        | Boot process documented; microkernel  | Syscall boundary; user/kernel rings[^1][^2] |
| Theseus  | x86_64                                   | nano_core bootstrap; runtime loading  | Language-based (SAS/SPL)[^3]     |
| Tock     | ARM Cortex-M; RISC-V ecosystem           | Embedded boot; board/chip setup       | MPU-based isolation[^4][^5]      |

### Table A5: Development Workflow Elements

| OS       | Workflow highlights                                   |
|----------|--------------------------------------------------------|
| Redox    | Recipe-driven builds; cross-compilation; relibc; public forks + MRs; SHA-pinned submodules[^6] |
| Theseus  | Runtime metadata; dependency verification; unwinding; cell swapping for live evolution[^3] |
| Tock     | Rust toolchain; board/chip builds; MPU isolation; capsules as extensions[^4][^5] |

### Table A6: State Management Strategies vs Fault Containment

| OS       | Strategy                                         | Impact on fault containment                      |
|----------|---------------------------------------------------|--------------------------------------------------|
| Redox    | Microkernel + user-space servers                  | Faults isolated across address spaces; IPC boundaries[^1] |
| Theseus  | Opaque exportation; metadata-driven cell swapping | Minimal state spill; precise recovery; live evolution[^3] |
| Tock     | Capsules + MPU policies                           | Spatial isolation contains faults; conservative ISR patterns[^4][^5] |

### Table A7: Model Selection Guide

| Context                                   | Prefer                                           | Rationale                                                     |
|-------------------------------------------|--------------------------------------------------|---------------------------------------------------------------|
| Co-designed runtime, strong language control | Intralingual, single-address-space (Theseus-like) | Compile-time enforcement, reduced state spill, live evolution[^3] |
| Constrained MCUs, MPU-only                 | Rust + MPU isolation (Tock-like)                 | Spatial isolation, low overhead, practical driver separation[^4][^5] |
| General-purpose OS, ecosystem compatibility | Microkernel + user-space services (Redox-like)   | Familiar model, isolation via address spaces, stable ABIs[^1][^2] |

### Table A8: Pitfalls and Mitigations

| Pitfall                                                | Mitigation                                                   |
|--------------------------------------------------------|--------------------------------------------------------------|
| Unsafe aliasing in event-driven embedded code          | Adopt conservative ISR conventions; consider language extensions (execution contexts) proposed for Rust[^5] |
| State spill across OS boundaries                       | Use opaque exportation; metadata-driven dependency tracking; minimized central task state[^3] |
| Lossy interfaces across kernel/user boundaries         | Preserve type/lifetime/ownership; avoid handles; prefer owned objects that encode provenance[^3] |
| Leakage and inconsistent cleanup                       | Implement cleanup only in Drop; rely on unwinding for exceptional paths; use guard-based lock release[^3] |
| Overly coarse-grained isolation                        | Combine language safety with hardware isolation appropriate to domain (MPU or address spaces)[^4][^5] |
| Build and packaging complexity                         | Use recipe-based workflows (Cookbook); pin sources via hashes; templates for Cargo/CMake/Meson[^6] |

---

This report distills the architectural foundations, memory safety strategies, design patterns, and development workflows of three influential Rust-based operating systems into actionable guidance for practitioners. By situating each project in its design context and identifying what generalizes, the analysis aims to help OS engineers choose strategies that best fit their constraints and goals.