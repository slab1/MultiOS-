import React from 'react';

const FeaturesPage = () => {
  const architectureFeatures = [
    {
      title: "x86_64 Support",
      description: "Full implementation for Intel and AMD 64-bit processors with modern features including SSE, AVX, and multi-core optimization.",
      technical: "Intel VT-x, AMD-V virtualization, UEFI support, ACPI power management"
    },
    {
      title: "ARM64 Support",
      description: "Complete ARM64 (AArch64) implementation for modern ARM processors including mobile and server architectures.",
      technical: "ARMv8-A architecture, NEON SIMD, TrustZone security, UEFI runtime services"
    },
    {
      title: "RISC-V Support",
      description: "Comprehensive RISC-V support showcasing this open-source ISA with multiple privilege levels and extensions.",
      technical: "RISC-V ISA, machine/user/supervisor modes, extensions A/M/D, OpenSBI support"
    }
  ];

  const bootFeatures = [
    {
      title: "UEFI Boot",
      description: "Modern UEFI firmware support with secure boot capabilities",
      icon: "🔧"
    },
    {
      title: "Legacy BIOS",
      description: "Backward compatibility with traditional BIOS systems",
      icon: "⚙️"
    },
    {
      title: "Multi-stage",
      description: "Intelligent boot loading with hardware detection",
      icon: "🚀"
    },
    {
      title: "Fast Boot",
      description: "Optimized boot process with parallel initialization",
      icon: "⚡"
    }
  ];

  const driverSupport = [
    {
      title: "Graphics Drivers",
      description: "VGA, VESA, and UEFI GOP support with framebuffer management",
      components: ["VGA Text Mode", "VESA Graphics Mode", "UEFI GOP", "2D Graphics Primitives"]
    },
    {
      title: "Storage Controllers",
      description: "SATA, NVMe, and USB Mass Storage device drivers",
      components: ["AHCI SATA", "NVMe NVMe", "USB Mass Storage", "Block Device I/O"]
    },
    {
      title: "Network Interfaces",
      description: "Ethernet and WiFi networking with protocol stack",
      components: ["Ethernet MAC", "WiFi 802.11", "TCP/IP Stack", "Packet Routing"]
    },
    {
      title: "Audio Subsystems",
      description: "AC'97, Intel HDA, and USB Audio support",
      components: ["AC'97 Codec", "Intel HDA", "USB Audio", "Audio Processing"]
    }
  ];

  const coreSubsystems = [
    {
      title: "Memory Management",
      description: "Virtual memory, page allocation, and memory protection",
      features: ["Virtual Address Space", "Page Tables", "Memory Protection", "DMA Management"]
    },
    {
      title: "Process Scheduling",
      description: "Priority-based multiprocessor scheduling with fair queuing",
      features: ["Preemptive Multitasking", "CPU Affinity", "Priority Scheduling", "Load Balancing"]
    },
    {
      title: "Inter-Process Communication",
      description: "Message passing, shared memory, and synchronization primitives",
      features: ["Message Queues", "Shared Memory", "Semaphores", "Event Notifications"]
    },
    {
      title: "Filesystem Support",
      description: "MultiFS with journaling, caching, and POSIX compatibility",
      features: ["MultiFS Filesystem", "Journaling", "Buffer Cache", "POSIX API"]
    },
    {
      title: "Device Management",
      description: "Hardware abstraction layer with driver framework",
      features: ["Device Tree", "Driver Framework", "Hot-plugging", "Power Management"]
    },
    {
      title: "GUI Toolkit",
      description: "Window management, graphics rendering, and user input",
      features: ["Window Manager", "2D Graphics", "Input Handling", "Theme System"]
    }
  ];

  const enterpriseFeatures = [
    {
      title: "Security",
      description: "Memory-safe Rust implementation with security hardening",
      features: ["Memory Safety", "Sandboxing", "Access Control", "Secure Boot"]
    },
    {
      title: "Reliability",
      description: "Fault tolerance with crash recovery and system monitoring",
      features: ["Crash Recovery", "Health Monitoring", "Error Handling", "System Recovery"]
    },
    {
      title: "Performance",
      description: "Optimized for high performance with benchmarking tools",
      features: ["Zero-Copy I/O", "Lock-Free Algorithms", "Performance Profiling", "Optimization"]
    },
    {
      title: "Monitoring",
      description: "Comprehensive system monitoring and debugging tools",
      features: ["System Metrics", "Performance Counters", "Debug Interface", "Logging System"]
    }
  ];

  return (
    <div className="pt-16">
      {/* Page Header */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container text-center">
          <h1 className="text-h1 mb-6">Technical Features</h1>
          <p className="text-large max-w-3xl mx-auto">
            MultiOS demonstrates comprehensive operating system implementation across all major subsystems. 
            Every component is designed with educational value and production quality in mind.
          </p>
        </div>
      </section>

      {/* Multi-Architecture Support */}
      <section className="section">
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Multi-Architecture Support</h2>
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
            {architectureFeatures.map((arch, index) => (
              <div key={index} className="card-white">
                <h3 className="text-h3 mb-4">{arch.title}</h3>
                <p className="text-body mb-4">{arch.description}</p>
                <div className="border-t border-gray-light pt-4">
                  <p className="text-caption font-medium text-gray-600 uppercase tracking-wider">Technical Details</p>
                  <p className="text-small text-gray-600">{arch.technical}</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Boot System */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Boot System</h2>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
            {bootFeatures.map((feature, index) => (
              <div key={index} className="text-center">
                <div className="text-4xl mb-4">{feature.icon}</div>
                <h3 className="text-h3 mb-3">{feature.title}</h3>
                <p className="text-body text-gray-600">{feature.description}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Driver Support */}
      <section className="section">
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Driver Support</h2>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-8">
            {driverSupport.map((driver, index) => (
              <div key={index} className="card">
                <h3 className="text-h3 mb-4">{driver.title}</h3>
                <p className="text-body mb-4">{driver.description}</p>
                <div className="space-y-2">
                  {driver.components.map((component, idx) => (
                    <div key={idx} className="flex items-center">
                      <div className="w-2 h-2 bg-red-600 mr-3"></div>
                      <span className="text-small text-gray-600">{component}</span>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Core Subsystems */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Core Subsystems</h2>
          <div className="space-y-8">
            {coreSubsystems.map((subsystem, index) => (
              <div key={index} className="card-white">
                <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
                  <div>
                    <h3 className="text-h3 mb-4">{subsystem.title}</h3>
                    <p className="text-body text-gray-600">{subsystem.description}</p>
                  </div>
                  <div>
                    <h4 className="text-small font-bold uppercase tracking-wider text-gray-600 mb-3">Key Features</h4>
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                      {subsystem.features.map((feature, idx) => (
                        <div key={idx} className="flex items-center">
                          <div className="w-2 h-2 bg-red-600 mr-2"></div>
                          <span className="text-small text-gray-600">{feature}</span>
                        </div>
                      ))}
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Enterprise Features */}
      <section className="section">
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Enterprise Features</h2>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
            {enterpriseFeatures.map((feature, index) => (
              <div key={index} className="card-white text-center">
                <h3 className="text-h3 mb-4">{feature.title}</h3>
                <p className="text-body mb-4">{feature.description}</p>
                <div className="space-y-2">
                  {feature.features.map((item, idx) => (
                    <div key={idx} className="border-t border-gray-light pt-3 first:border-t-0 first:pt-0">
                      <span className="text-small text-gray-600">{item}</span>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>
    </div>
  );
};

export default FeaturesPage;