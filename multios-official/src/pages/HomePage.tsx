import React from 'react';
import { Link } from 'react-router-dom';

const HomePage = () => {
  const keyMetrics = [
    {
      number: "50,000+",
      label: "Lines of Code",
      description: "Comprehensive implementation across all subsystems"
    },
    {
      number: "15+",
      label: "Major Subsystems",
      description: "Complete operating system components"
    },
    {
      number: "3",
      label: "CPU Architectures",
      description: "x86_64, ARM64, and RISC-V support"
    },
    {
      number: "95%",
      label: "Test Coverage",
      description: "Enterprise-grade quality assurance"
    }
  ];

  const valuePropositions = [
    {
      title: "Educational Excellence",
      description: "Comprehensive documentation and examples designed specifically for operating systems education. Perfect for curriculum integration and hands-on learning.",
      image: "/images/education_technology_multi_platform_learning.jpg"
    },
    {
      title: "Cross-Platform",
      description: "Seamless operation across x86_64, ARM64, and RISC-V architectures. Demonstrates modern compatibility patterns and hardware abstraction.",
      image: "/images/cross_platform_devices_education_technology.jpg"
    },
    {
      title: "Modern Development",
      description: "Written entirely in Rust for memory safety and performance. Showcases best practices in systems programming and modern development workflows.",
      image: "/images/modern_minimal_code_editor_javascript_programming.jpg"
    }
  ];

  return (
    <div className="pt-16">
      {/* Hero Section */}
      <section className="section-large">
        <div className="container">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-16 items-center">
            <div>
              <h1 className="text-display mb-6">
                MultiOS
              </h1>
              <p className="text-large mb-8 max-w-lg">
                A revolutionary, educational, and production-ready operating system written entirely in Rust, 
                designed to run seamlessly across multiple CPU architectures.
              </p>
              <div className="flex flex-col sm:flex-row gap-4">
                <Link to="/download" className="btn btn-primary btn-large">
                  Download MultiOS
                </Link>
                <Link to="/demos" className="btn btn-secondary btn-large">
                  Try Interactive Demo
                </Link>
              </div>
            </div>
            <div className="relative">
              <img 
                src="/images/operating_system_architecture_kernel_user_mode_diagram.jpg"
                alt="MultiOS Architecture Diagram"
                className="w-full h-auto border border-gray-light"
              />
            </div>
          </div>
        </div>
      </section>

      {/* Key Metrics Section */}
      <section className="section">
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Project Statistics</h2>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
            {keyMetrics.map((metric, index) => (
              <div key={index} className="card text-center">
                <div className="text-4xl font-bold text-black mb-2">
                  {metric.number}
                </div>
                <div className="text-small font-bold uppercase tracking-wider text-gray-600 mb-3">
                  {metric.label}
                </div>
                <div className="text-body text-gray-600">
                  {metric.description}
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Value Propositions Section */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Core Objectives Achieved</h2>
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
            {valuePropositions.map((item, index) => (
              <div key={index} className="card-white">
                <div className="mb-6">
                  <img 
                    src={item.image}
                    alt={item.title}
                    className="w-full h-48 object-cover border border-gray-light"
                  />
                </div>
                <h3 className="text-h3 mb-4">{item.title}</h3>
                <p className="text-body text-gray-600">
                  {item.description}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Architecture Overview Section */}
      <section className="section">
        <div className="container">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-16 items-center">
            <div>
              <h2 className="text-h2 mb-6">Architecture Overview</h2>
              <p className="text-body mb-6">
                MultiOS demonstrates modern operating system design with a clean separation between kernel and user space, 
                comprehensive device driver framework, and efficient system services architecture.
              </p>
              <div className="space-y-4">
                <div className="swiss-border-left pl-6">
                  <h4 className="text-h3 mb-2">Boot System</h4>
                  <p className="text-body text-gray-600">Multi-stage boot with UEFI and legacy BIOS support</p>
                </div>
                <div className="swiss-border-left pl-6">
                  <h4 className="text-h3 mb-2">Kernel Core</h4>
                  <p className="text-body text-gray-600">Memory management, process scheduling, and IPC</p>
                </div>
                <div className="swiss-border-left pl-6">
                  <h4 className="text-h3 mb-2">Driver Framework</h4>
                  <p className="text-body text-gray-600">Graphics, storage, network, and audio subsystems</p>
                </div>
                <div className="swiss-border-left pl-6">
                  <h4 className="text-h3 mb-2">System Services</h4>
                  <p className="text-body text-gray-600">Filesystem, GUI toolkit, and API interfaces</p>
                </div>
              </div>
              <div className="mt-8">
                <Link to="/features" className="btn btn-primary">
                  Explore All Features
                </Link>
              </div>
            </div>
            <div>
              <img 
                src="/images/operating_system_architecture_diagram_kernel_user_mode.jpg"
                alt="Detailed Architecture"
                className="w-full h-auto border border-gray-light"
              />
            </div>
          </div>
        </div>
      </section>

      {/* Call to Action Section */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container">
          <div className="text-center max-w-3xl mx-auto">
            <h2 className="text-h2 mb-6">Ready to Get Started?</h2>
            <p className="text-large mb-8">
              Whether you're an educator looking to enhance curriculum, a developer interested in contributing, 
              or an institution evaluating operating systems for educational use, MultiOS provides the tools and documentation you need.
            </p>
            <div className="flex flex-col sm:flex-row justify-center gap-4">
              <Link to="/download" className="btn btn-primary btn-large">
                Download Now
              </Link>
              <Link to="/educators" className="btn btn-secondary btn-large">
                For Educators
              </Link>
              <Link to="/developers" className="btn btn-secondary btn-large">
                For Developers
              </Link>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
};

export default HomePage;