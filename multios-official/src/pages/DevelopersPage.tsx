import React from 'react';

const DevelopersPage = () => {
  const quickStartSteps = [
    {
      step: "Prerequisites",
      description: "Install Rust toolchain and build dependencies",
      code: `# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install build dependencies
sudo apt update
sudo apt install qemu-system-x86 gcc-aarch64-linux-gnu gcc-riscv64-linux-gnu

# Verify installation
rustc --version
cargo --version`
    },
    {
      step: "Setup",
      description: "Clone repository and initialize development environment",
      code: `# Clone the repository
git clone https://github.com/multios/multios.git
cd multios

# Initialize submodules
git submodule update --init --recursive

# Install development tools
cargo install cargo-watch
cargo install cargo-audit

# Build the project
cargo build --release`
    },
    {
      step: "Test",
      description: "Run tests and verify your setup",
      code: `# Run unit tests
cargo test

# Run integration tests
cargo test --test integration

# Test on different architectures
cargo test --target x86_64-unknown-none
cargo test --target aarch64-unknown-none
cargo test --target riscv64gc-unknown-none

# Run on QEMU
make test-qemu-x86_64`
    },
    {
      step: "Contribute",
      description: "Start contributing to the project",
      code: `# Create a feature branch
git checkout -b feature/my-new-feature

# Make your changes
# ... edit files ...

# Run tests
cargo test

# Commit changes
git commit -m "Add: my new feature"

# Push and create PR
git push origin feature/my-new-feature

# Submit pull request on GitHub`
    }
  ];

  const contributionAreas = [
    {
      title: "Core Development",
      description: "Work on kernel subsystems and core OS components",
      areas: [
        "Memory management and virtual memory",
        "Process scheduling and synchronization",
        "Filesystem implementation and drivers",
        "System call interface and IPC",
        "Hardware abstraction layer"
      ],
      icon: "⚡"
    },
    {
      title: "Platform Support", 
      description: "Extend MultiOS to new architectures and hardware",
      areas: [
        "x86_64 optimization and features",
        "ARM64 implementation and tuning",
        "RISC-V development and extensions",
        "New processor architecture ports",
        "Platform-specific optimizations"
      ],
      icon: "🔧"
    },
    {
      title: "Testing & Quality",
      description: "Improve test coverage and ensure code quality",
      areas: [
        "Unit test implementation",
        "Integration test development",
        "Performance benchmarking",
        "Security audit and testing",
        "Documentation and examples"
      ],
      icon: "🧪"
    },
    {
      title: "Education",
      description: "Develop educational resources and learning materials",
      areas: [
        "Tutorial and guide writing",
        "Code example development",
        "Interactive demo creation",
        "Curriculum material development",
        "Documentation improvements"
      ],
      icon: "📚"
    }
  ];

  const developmentTools = [
    {
      title: "Build System",
      description: "Cargo-based build system with cross-compilation",
      tools: ["Rust Cargo", "Cross-compilation", "Link-time optimization", "Incremental builds"]
    },
    {
      title: "Testing Framework",
      description: "Comprehensive testing across multiple architectures",
      tools: ["Unit testing", "Integration testing", "QEMU testing", "Hardware testing"]
    },
    {
      title: "CI/CD Pipeline",
      description: "Automated testing and deployment",
      tools: ["GitHub Actions", "Automated builds", "Cross-platform testing", "Performance monitoring"]
    },
    {
      title: "Debugging Tools",
      description: "Professional debugging and profiling capabilities",
      tools: ["GDB integration", "QEMU debugging", "Systemtap", "Performance profiling"]
    }
  ];

  return (
    <div className="pt-16">
      {/* Page Header */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container text-center">
          <h1 className="text-h1 mb-6">For Developers</h1>
          <p className="text-large max-w-3xl mx-auto">
            Join the MultiOS development community. Contribute to a cutting-edge operating system 
            project while learning modern systems programming practices with Rust.
          </p>
        </div>
      </section>

      {/* Quick Start */}
      <section className="section">
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Quick Start Guide</h2>
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
            {quickStartSteps.map((step, index) => (
              <div key={index} className="card-white">
                <div className="flex items-center mb-4">
                  <div className="w-8 h-8 bg-red-600 text-white text-sm font-bold rounded-full flex items-center justify-center mr-3">
                    {index + 1}
                  </div>
                  <div>
                    <h3 className="text-h3">{step.step}</h3>
                    <p className="text-body text-gray-600">{step.description}</p>
                  </div>
                </div>
                <div className="code-block">
                  <pre className="text-sm text-white">{step.code}</pre>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Contribution Areas */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Contribution Areas</h2>
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
            {contributionAreas.map((area, index) => (
              <div key={index} className="card">
                <div className="flex items-center mb-4">
                  <div className="text-3xl mr-4">{area.icon}</div>
                  <div>
                    <h3 className="text-h3">{area.title}</h3>
                    <p className="text-body text-gray-600">{area.description}</p>
                  </div>
                </div>
                <ul className="space-y-2">
                  {area.areas.map((item, idx) => (
                    <li key={idx} className="flex items-start">
                      <div className="w-2 h-2 bg-red-600 mt-2 mr-3 flex-shrink-0"></div>
                      <span className="text-small text-gray-600">{item}</span>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Development Tools */}
      <section className="section">
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Development Tools</h2>
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
            {developmentTools.map((tool, index) => (
              <div key={index} className="card-white">
                <h3 className="text-h3 mb-4">{tool.title}</h3>
                <p className="text-body text-gray-600 mb-4">{tool.description}</p>
                <div className="grid grid-cols-2 gap-2">
                  {tool.tools.map((toolItem, idx) => (
                    <div key={idx} className="text-small text-gray-600 bg-gray-pale px-3 py-2 rounded">
                      {toolItem}
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* API Documentation */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-16 items-center">
            <div>
              <h2 className="text-h2 mb-6">API Documentation</h2>
              <p className="text-body mb-6">
                Comprehensive API documentation covering all MultiOS subsystems. 
                Interactive documentation with examples and reference implementations.
              </p>
              <div className="space-y-3">
                <a href="/docs/api/kernel" className="block text-body text-red-600 hover:text-red-700">
                  → Kernel API Reference
                </a>
                <a href="/docs/api/drivers" className="block text-body text-red-600 hover:text-red-700">
                  → Driver Framework API
                </a>
                <a href="/docs/api/fs" className="block text-body text-red-600 hover:text-red-700">
                  → Filesystem API
                </a>
                <a href="/docs/api/syscalls" className="block text-body text-red-600 hover:text-red-700">
                  → System Call Interface
                </a>
                <a href="/docs/examples" className="block text-body text-red-600 hover:text-red-700">
                  → Code Examples and Tutorials
                </a>
              </div>
            </div>
            <div>
              <img 
                src="/images/modern_minimal_vscode_programming_interface.jpg"
                alt="API Documentation"
                className="w-full h-auto border border-gray-light"
              />
            </div>
          </div>
        </div>
      </section>

      {/* Code Examples */}
      <section className="section">
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Code Examples</h2>
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
            <div className="card">
              <h3 className="text-h3 mb-4">Basic Kernel Module</h3>
              <div className="code-block">
                <pre className="text-sm text-white">{`use multios::prelude::*;

pub struct MyModule;

impl KernelModule for MyModule {
    fn init() -> Result<Self> {
        println!("Initializing my kernel module");
        
        // Register interrupt handler
        interrupt::register_handler(32, my_handler)?;
        
        Ok(MyModule)
    }
    
    fn exit() {
        println!("Cleaning up my module");
    }
}

kernel_module!(MyModule);`}</pre>
              </div>
            </div>

            <div className="card">
              <h3 className="text-h3 mb-4">Device Driver</h3>
              <div className="code-block">
                <pre className="text-sm text-white">{`use multios::drivers::*;

pub struct MyDevice {
    base_addr: *mut u8,
}

impl DeviceDriver for MyDevice {
    type Interrupt = usize;
    
    fn probe(hw_addr: usize) -> Result<Self> {
        Ok(MyDevice {
            base_addr: hw_addr as *mut u8,
        })
    }
    
    fn handle_interrupt(&self) -> Result<()> {
        let status = unsafe {
            self.base_addr.read_volatile()
        };
        
        // Handle device interrupt
        if status & 0x01 != 0 {
            self.process_data()?;
        }
        
        Ok(())
    }
}`}</pre>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Mentorship Program */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container text-center">
          <h2 className="text-h2 mb-6">Mentorship Program</h2>
          <p className="text-large mb-8 max-w-2xl mx-auto">
            New to operating systems development? Join our mentorship program to get personalized guidance 
            from experienced MultiOS developers.
          </p>
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-8 mb-8">
            <div className="card-white">
              <h3 className="text-h3 mb-4">Getting Help</h3>
              <p className="text-body text-gray-600 mb-4">
                Connect with mentors through our Discord server or GitHub Discussions for guidance and support.
              </p>
              <a href="https://discord.gg/multios" className="btn btn-secondary btn-small">
                Join Discord
              </a>
            </div>

            <div className="card-white">
              <h3 className="text-h3 mb-4">Code Reviews</h3>
              <p className="text-body text-gray-600 mb-4">
                All pull requests receive thorough review from maintainers with constructive feedback.
              </p>
              <a href="https://github.com/multios/multios/pulls" className="btn btn-secondary btn-small">
                View Pull Requests
              </a>
            </div>

            <div className="card-white">
              <h3 className="text-h3 mb-4">Documentation</h3>
              <p className="text-body text-gray-600 mb-4">
                Comprehensive guides and documentation to help you understand the codebase.
              </p>
              <a href="/docs" className="btn btn-secondary btn-small">
                Read Documentation
              </a>
            </div>
          </div>
          <div className="flex flex-col sm:flex-row justify-center gap-4">
            <a href="https://github.com/multios/multios" className="btn btn-primary btn-large">
              View Source on GitHub
            </a>
            <a href="https://discord.gg/multios" className="btn btn-secondary btn-large">
              Join Community Chat
            </a>
          </div>
        </div>
      </section>
    </div>
  );
};

export default DevelopersPage;