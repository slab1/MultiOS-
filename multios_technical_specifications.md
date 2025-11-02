# MultiOS: Technical Specifications

## 1. Executive Summary

MultiOS is a universal educational operating system designed from the ground up in Rust to provide a safe, modular, and portable learning environment. The project’s vision is to create a transparent, well-documented OS that empowers students and educators to explore core computer science concepts on a wide range of modern hardware, from x86_64 laptops to ARM64 tablets and RISC-V development boards.

The primary goals of MultiOS are:
*   **Educational Value**: To prioritize clarity, simplicity, and inspectability over performance optimizations, making the system an ideal tool for teaching OS principles.
*   **Memory Safety**: To leverage Rust’s ownership model and type system to eliminate entire classes of memory safety vulnerabilities, creating a robust and reliable platform.
*   **Modularity**: To build a system from small, isolated components that are easy to understand, modify, and extend.
*   **Cross-Platform Compatibility**: To design for multi-architecture support from day one, ensuring that MultiOS can run on the diverse hardware found in educational settings.

To achieve these goals, MultiOS will adopt a **hybrid microkernel architecture**. This design combines the security and modularity of a microkernel with the performance of a monolithic kernel by selectively co-locating trusted, performance-critical services in kernel space while isolating other services and drivers in user space. This approach, inspired by modern production microkernels like HongMeng, provides a pragmatic balance of safety, performance, and compatibility.

The multi-device strategy will follow a phased approach, starting with x86_64 and progressively adding support for ARM64 and RISC-V. A well-defined **Hardware Abstraction Layer (HAL)**, inspired by Android’s architecture, will be a cornerstone of this strategy, ensuring that the core OS remains portable and independent of vendor-specific hardware.

This document serves as the definitive technical blueprint for the 6-12 month full-time development of MultiOS, covering system architecture, implementation details, development roadmap, and educational strategy.

## 2. System Architecture

The architectural foundation of MultiOS is a **hybrid microkernel**, implemented in Rust. This choice is a deliberate balance between the strict isolation of a pure microkernel and the performance of a monolithic design. It allows MultiOS to be both robust and efficient, making it suitable for a wide range of educational devices.

### 2.1. Kernel Design: A Hybrid Approach

MultiOS’s kernel will be inspired by modern, production-tested microkernels that have successfully bridged the performance gap with monolithic kernels. The design will incorporate the following principles:

*   **Minimal Kernel Core**: The core kernel will be kept as small as possible, responsible only for essential services such as scheduling, inter-process communication (IPC), and basic memory management.
*   **User-Space Services**: The majority of OS services, including file systems, network stacks, and device drivers, will run as isolated user-space processes. This sandboxing is a critical safety feature, as a crash in a driver will not bring down the entire system.
*   **Differentiated Isolation Classes**: To optimize performance, MultiOS will adopt a model of differentiated isolation, similar to HongMeng’s. Validated, performance-critical services can be placed in a more privileged kernel-space context (IC1) with hardware-enforced isolation, while less trusted components like third-party drivers will run in a fully isolated user-space context (IC2).
*   **Selective Coalescing**: For tightly coupled services where IPC overhead is a bottleneck (e.g., the file system and memory manager), MultiOS will support coalescing these services into a single process to improve performance on resource-constrained devices.

### 2.2. The Role of Rust: Enforcing Safety

Rust is not just an implementation language for MultiOS; it is a core part of its security architecture. The language’s features will be leveraged to build a safer OS:

*   **Ownership and Borrowing**: Rust's compile-time checks for memory safety will be the first line of defense against bugs like null pointer dereferences, buffer overflows, and data races.
*   **Explicit `unsafe`**: All `unsafe` code, which is necessary for low-level hardware interaction, will be explicitly marked and kept to a minimum. This will create a small, auditable Trusted Computing Base (TCB).
*   **Safe Abstractions**: Safe, high-level abstractions will be built on top of the minimal `unsafe` code, providing safe APIs for the rest of the kernel and user-space services to use.

This approach, which mirrors the philosophy of systems like Redox OS and Tock, will allow MultiOS to achieve a high degree of safety without sacrificing the ability to perform low-level system programming.

### 2.3. Architecture Overview

The overall architecture of MultiOS can be visualized as a series of layers:

![MultiOS Architecture Diagram (Conceptual)](/workspace/charts/multios_architecture.png)
*Figure 1: A conceptual diagram of the MultiOS hybrid microkernel architecture. Note: This image is illustrative.*

*   **Hardware**: The physical devices (CPU, memory, peripherals).
*   **Kernel Core (Rust)**: The minimal microkernel running in privileged mode.
*   **Hardware Abstraction Layer (HAL)**: The interface that separates the kernel from platform-specific details.
*   **Kernel-Space Services (IC1)**: Trusted, performance-critical services with hardware-enforced isolation.
*   **User-Space Services (IC2)**: Isolated services, including drivers, file systems, and network stacks.
*   **Applications**: Educational applications and user-facing software.

This modular, layered design is key to achieving the project's goals of safety, portability, and educational value.

## 3. Multi-Device Strategy

MultiOS is designed to be a universal OS, capable of running on the diverse range of hardware found in educational environments. The strategy for achieving this is centered on a phased implementation across architectures and a robust Hardware Abstraction Layer (HAL).

### 3.1. Phased Architecture Support

Development will proceed in three phases, allowing for focused effort on one platform at a time while building the abstractions necessary for portability:

*   **Phase 1: x86_64**: The initial development target will be the ubiquitous x86_64 architecture. This will allow the team to focus on building the core OS and tooling in a familiar environment.
*   **Phase 2: ARM64**: Once the x86_64 version is stable, the focus will shift to ARM64, the architecture powering most tablets and many modern laptops. This phase will validate the portability of the HAL and core OS.
*   **Phase 3: RISC-V**: The final phase will target RISC-V, an open-standard instruction set architecture that is gaining traction in both educational and commercial settings. Supporting RISC-V will fulfill the project's vision of being a truly universal educational OS.

### 3.2. Hardware Abstraction Layer (HAL)

The cornerstone of the multi-device strategy is the Hardware Abstraction Layer (HAL). The HAL will be a stable, well-defined interface that separates the portable OS components from the platform-specific code. This design, inspired by the successful HAL in Android, offers several advantages:

*   **Portability**: The core OS and its services will be written against the HAL, not directly against the hardware. This means the majority of the OS code will be portable across architectures.
*   **Modularity**: Platform-specific code will be encapsulated within HAL implementations for each target architecture. This makes it easier to add support for new hardware without modifying the core OS.
*   **Vendor Independence**: The HAL will provide a clear contract for hardware vendors, allowing them to write drivers for MultiOS without needing to understand the internals of the OS.

The HAL will define interfaces for essential hardware functions, including:
*   Boot Process and Firmware Interaction
*   Memory Management and Mapping
*   Interrupt Handling
*   Timers and Clocks
*   Device Discovery and Power Management

### 3.3. Cross-Compilation and Tooling

A robust cross-compilation toolchain is essential for a multi-device OS. While the `cross_compilation_guide.md` document was not available for this report, the development team will prioritize the setup of a seamless cross-compilation environment using tools like `cross` and pre-configured Docker images. This will enable developers to build and test for all target architectures from a single development machine.

## 4. Technical Implementation

The implementation of MultiOS will be guided by the architectural principles outlined above. This section details the initial plans for the core components of the OS.

### 4.1. Boot Process

The boot process will be tailored to each supported architecture, using a combination of established bootloaders and modern boot protocols to ensure portability and robustness.

*   **x86_64**: The initial bootloader will be based on the `bootloader` crate, which supports both BIOS and UEFI. For UEFI, MultiOS will interface with the firmware using the `uefi` and `uefi-services` crates. The long-term goal is to support UEFI Secure Boot.
*   **ARM64**: The boot process will rely on standard bootloaders like U-Boot or EDK2/UEFI. The kernel will expect a DeviceTree Blob (DTB) to be passed from the bootloader, which will be used to discover and configure hardware.
*   **RISC-V**: The boot process will be designed to work with OpenSBI. The kernel will expect the hart ID and DTB address to be passed in registers `a0` and `a1`, respectively.

Across all architectures, the bootloader will be responsible for loading the kernel and providing it with essential information, including a memory map and framebuffer details. The Limine boot protocol and its associated Rust crate will also be evaluated as a potential cross-architecture boot solution.

### 4.2. Memory Management

Memory management will be a cornerstone of MultiOS's safety and stability. The design will draw inspiration from Theseus OS, which leverages Rust's type system to create strong guarantees around memory safety.

*   **Memory Abstraction**: A core `MappedPages` abstraction will be implemented to provide a type-safe and bounds-checked interface to memory. This will prevent many common memory errors at the language level.
*   **Ownership-Based Resource Management**: Rust's `Drop` trait will be used for automatic resource cleanup. When a memory mapping goes out of scope, its underlying physical memory will be automatically freed. This will prevent memory leaks and simplify resource management.
*   **Single Address Space (Intra-Kernel)**: Within the kernel and its trusted (IC1) services, a single address space will be used to maximize performance and leverage Rust's safety guarantees for intra-kernel communication.

### 4.3. Process Scheduling

The scheduler will be a core component of the microkernel, responsible for managing the execution of user-space processes. The initial implementation will be a simple round-robin scheduler, with the goal of evolving to a more sophisticated pre-emptive priority-based scheduler over time. The scheduler will be designed to be fair and responsive, ensuring that no single process can monopolize the CPU.

### 4.4. Device Drivers

Device drivers are a common source of instability in operating systems. MultiOS will mitigate this risk by isolating drivers in user-space processes.

*   **User-Space Drivers**: The majority of device drivers will run as unprivileged user-space processes. A crash in a driver will simply cause the driver process to be restarted, without affecting the kernel or other parts of the system.
*   **Driver-Container Model**: To maximize hardware compatibility, MultiOS will adopt a driver-container model, similar to that of HongMeng. This will allow the use of unmodified Linux drivers within a secure, isolated environment.
*   **Safe Driver APIs**: Driver APIs will be defined in Rust, using the language's features to create safe and robust interfaces for hardware interaction.

## 5. Development Roadmap

Development will be organized into a series of phases, each with clear deliverables and success metrics.

*   **Phase 1: MVP Microkernel (Months 1-3)**
    *   **Deliverables**: A minimal Rust-based microkernel that boots on x86_64 in QEMU. This will include a basic scheduler, IPC mechanism, and memory management.
    *   **Success Metrics**: Successful boot, stable execution of a few simple user-space processes.
*   **Phase 2: HAL and ARM64 Support (Months 4-6)**
    *   **Deliverables**: A well-defined Hardware Abstraction Layer (HAL) and a port of the OS to ARM64.
    *   **Success Metrics**: Successful boot on ARM64 hardware (e.g., Raspberry Pi), validation of the HAL's portability.
*   **Phase 3: RISC-V Support and Driver Model (Months 7-9)**
    *   **Deliverables**: A port of the OS to RISC-V and the implementation of the user-space driver model.
    *   **Success Metrics**: Successful boot on RISC-V hardware, successful operation of a few simple user-space drivers.
*   **Phase 4: Advanced Features and Educational Tools (Months 10-12)**
    *   **Deliverables**: Implementation of advanced features like the driver-container model and the development of educational tools and documentation.
    *   **Success Metrics**: Successful use of a Linux driver within a container, publication of initial tutorials and documentation.

## 6. Technology Stack

MultiOS will be built using a modern, open-source technology stack centered around Rust.

### 6.1. Language and Build System

*   **Language**: Rust will be the primary implementation language for the entire OS.
*   **Build System**: The OS will be built using Cargo, the standard Rust build tool. The build system will be configured for cross-compilation to all target architectures.
*   **Cross-Compilation**: The `cross` tool and/or custom Docker images will be used to create a seamless cross-compilation environment.

### 6.2. Key Crates

The project will make use of the vibrant Rust embedded and OS development ecosystem. Key crates will include:

*   `bootloader`: For the x86_64 bootloader.
*   `uefi` and `uefi-services`: For UEFI interaction.
*   `limine`: As a potential cross-architecture boot solution.
*   `tock-registers`: For type-safe access to memory-mapped I/O registers.

### 6.3. Testing Infrastructure

Testing will be a core part of the development process. The testing infrastructure will include:

*   **Unit and Integration Tests**: Carried out using Rust's built-in testing framework.
*   **Emulation**: QEMU will be used for testing the OS on all target architectures.
*   **Continuous Integration (CI)**: A CI pipeline will be set up to automatically build and test the OS on every commit.

## 7. Educational Focus

MultiOS is first and foremost an educational tool. The design and development process will be guided by the goal of making the OS as easy to understand and learn from as possible.

### 7.1. Learning Objectives

MultiOS will be designed to teach students about:

*   **Core OS Concepts**: Memory management, process scheduling, concurrency, and filesystems.
*   **Low-Level Systems Programming**: Interacting with hardware, writing drivers, and understanding the boot process.
*   **The Rust Programming Language**: How to use Rust's advanced features to write safe and efficient systems code.

### 7.2. Debugging and Documentation

To support these learning objectives, the project will invest heavily in debugging tools and documentation.

*   **Debugging**: The OS will have built-in support for debugging via GDB. All boot and kernel messages will be available over a serial console.
*   **Documentation**: The entire codebase will be thoroughly documented. In addition, a series of tutorials and guides will be written to walk students through the process of building and modifying the OS.

This focus on education will ensure that MultiOS is not just a functional operating system, but a valuable resource for the next generation of systems programmers.

## 8. Sources

This report was synthesized from a wide range of technical documentation, research papers, and best practices guides. The following sources were instrumental in shaping the technical specifications for MultiOS.

[1] [An Overview of Monolithic and Microkernel Architectures](https://www.windriver.com/sites/default/files/2024-05/monolithic-and-microkernel-architectures.pdf) - High Reliability - A detailed technical comparison from a reputable industry leader in embedded systems.

[2] [Microkernel Goes General: Performance and Compatibility in the HongMeng Production Microkernel](https://www.usenix.org/system/files/osdi24-chen-haibo.pdf) - High Reliability - A peer-reviewed paper from a top academic conference (USENIX OSDI 2024) detailing a production microkernel.

[3] [Why Rust? - The Redox Operating System](https://doc.redox-os.org/book/why-rust.html) - High Reliability - Official documentation from a well-known Rust-based OS project, outlining the language's benefits for kernel design.

[4] [Cross-compilation - The rustup book](https://rust-lang.github.io/rustup/cross-compilation.html) - High Reliability - Official documentation from the Rust project on cross-compilation.

[5] [A guide to cross-compilation in Rust](https://blog.logrocket.com/guide-cross-compilation-rust/) - Medium Reliability - A comprehensive guide from a well-regarded technical blog.

[6] [UEFI Specification 2.10 - Introduction](https://uefi.org/specs/UEFI/2.10/01_Introduction.html) - High Reliability - The official specification from the UEFI Forum.

[7] [UEFI Implementation Guide](https://wiki.osdev.org/UEFI) - High Reliability - A widely used and respected resource for OS developers from the OSDev community.

[8] [ARM64 U-Boot Architecture Documentation](https://docs.u-boot.org/en/v2022.04/arch/arm64.html) - High Reliability - Official documentation for the U-Boot bootloader.

[9] [RISC-V SBI and the Full Boot Process](https://popovicu.com/posts/risc-v-sbi-and-full-boot-process/) - Medium Reliability - A detailed and well-explained blog post on the RISC-V boot process.

[10] [Rust Bootloader Crate](https://github.com/rust-osdev/bootloader) - High Reliability - The official repository for a widely used Rust bootloader crate from the rust-osdev organization.

[11] [Theseus: an Experiment in Operating System Structure and State Management](https://www.usenix.org/system/files/osdi20-boos.pdf) - High Reliability - A peer-reviewed academic paper from OSDI 2020 describing an innovative Rust-based OS.

[12] [Tock Embedded Operating System](https://www.tockos.org/) - High Reliability - The official website for the Tock OS, a mature and widely used embedded OS written in Rust.
