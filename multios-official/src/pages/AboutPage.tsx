import React from 'react';

const AboutPage = () => {
  const milestones = [
    { year: "2023", title: "Project Inception", description: "Initial architecture design and core team formation" },
    { year: "2023", title: "Core Implementation", description: "Kernel development, boot system, and basic drivers" },
    { year: "2024", title: "Multi-Architecture Support", description: "x86_64, ARM64, and RISC-V implementations completed" },
    { year: "2024", title: "Educational Integration", description: "Curriculum materials and learning resources developed" },
    { year: "2025", title: "Production Release", description: "First stable release with comprehensive feature set" },
    { year: "2025", title: "Community Launch", description: "Open-source release and community engagement initiatives" }
  ];

  const projectStats = [
    { label: "Lines of Code", value: "50,000+", description: "Comprehensive implementation across all subsystems" },
    { label: "Test Coverage", value: "95%+", description: "Enterprise-grade quality assurance" },
    { label: "Architectures", value: "3", description: "x86_64, ARM64, RISC-V support" },
    { label: "Subsystems", value: "15+", description: "Complete operating system components" }
  ];

  const technologyStack = [
    { category: "Language", items: ["Rust (Memory Safety)", "Assembly (Boot Code)", "C (Compatibility Layer)"] },
    { category: "Build System", items: ["Cargo (Rust Package Manager)", "Cross-compilation", "Link-time Optimization"] },
    { category: "Testing", items: ["Unit Testing", "Integration Testing", "QEMU Emulation", "Hardware Testing"] },
    { category: "Documentation", items: ["Markdown Documentation", "API Reference", "Tutorial Guides", "Architecture Diagrams"] }
  ];

  return (
    <div className="pt-16">
      {/* Page Header */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container text-center">
          <h1 className="text-h1 mb-6">About MultiOS</h1>
          <p className="text-large max-w-3xl mx-auto">
            MultiOS represents the future of operating systems education and development. 
            Built from the ground up in Rust, it demonstrates modern systems programming 
            while providing an ideal learning platform for students and professionals.
          </p>
        </div>
      </section>

      {/* Project Vision */}
      <section className="section">
        <div className="container">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-16 items-center">
            <div>
              <h2 className="text-h2 mb-6">Project Vision</h2>
              <p className="text-body mb-6">
                MultiOS aims to provide a universal, educational, and production-ready operating system that demonstrates 
                modern OS development practices while maintaining compatibility across diverse hardware platforms. 
                Built from the ground up in Rust, it showcases safe systems programming and provides a 
                comprehensive learning platform for OS development.
              </p>
              <div className="space-y-4">
                <div className="swiss-border-left pl-6">
                  <h4 className="text-h3 mb-2">Educational Excellence</h4>
                  <p className="text-body text-gray-600">Comprehensive documentation and examples for operating systems education</p>
                </div>
                <div className="swiss-border-left pl-6">
                  <h4 className="text-h3 mb-2">Cross-Platform Compatibility</h4>
                  <p className="text-body text-gray-600">Seamless operation across multiple architectures and hardware platforms</p>
                </div>
                <div className="swiss-border-left pl-6">
                  <h4 className="text-h3 mb-2">Modern Development Practices</h4>
                  <p className="text-body text-gray-600">Rust-based memory-safe development with production-quality implementation</p>
                </div>
              </div>
            </div>
            <div>
              <img 
                src="/images/minimalist_network_server_infrastructure_illustration.jpg"
                alt="MultiOS Vision"
                className="w-full h-auto border border-gray-light"
              />
            </div>
          </div>
        </div>
      </section>

      {/* Project Statistics */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Project Statistics</h2>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
            {projectStats.map((stat, index) => (
              <div key={index} className="card-white text-center swiss-border-top">
                <div className="text-4xl font-bold text-black mb-2">
                  {stat.value}
                </div>
                <div className="text-small font-bold uppercase tracking-wider text-gray-600 mb-3">
                  {stat.label}
                </div>
                <div className="text-body text-gray-600">
                  {stat.description}
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Milestones Timeline */}
      <section className="section">
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Project Milestones</h2>
          <div className="max-w-4xl mx-auto">
            <div className="space-y-8">
              {milestones.map((milestone, index) => (
                <div key={index} className="flex items-start">
                  <div className="flex-shrink-0 w-16 text-right mr-6">
                    <span className="text-h3 font-bold text-red-600">{milestone.year}</span>
                  </div>
                  <div className="flex-shrink-0 w-4 h-4 bg-red-600 rounded-full mt-2 mr-6"></div>
                  <div className="flex-1 pb-8">
                    <h3 className="text-h3 mb-2">{milestone.title}</h3>
                    <p className="text-body text-gray-600">{milestone.description}</p>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </section>

      {/* Open Source Philosophy */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-16 items-center">
            <div>
              <img 
                src="/images/team_collaboration_developers_office_meeting_coding.jpg"
                alt="Open Source Collaboration"
                className="w-full h-auto border border-gray-light"
              />
            </div>
            <div>
              <h2 className="text-h2 mb-6">Open Source Philosophy</h2>
              <p className="text-body mb-6">
                MultiOS is committed to open source development and community collaboration. 
                We believe that operating systems education and innovation should be accessible 
                to everyone, regardless of institutional affiliation or economic background.
              </p>
              <div className="space-y-4">
                <div className="flex items-center">
                  <div className="w-8 h-8 bg-red-600 text-white text-sm font-bold rounded-full flex items-center justify-center mr-4">
                    MIT
                  </div>
                  <div>
                    <h4 className="text-h3 mb-1">MIT License</h4>
                    <p className="text-body text-gray-600">Permissive license allowing commercial and educational use</p>
                  </div>
                </div>
                <div className="flex items-center">
                  <div className="w-8 h-8 bg-red-600 text-white text-sm font-bold rounded-full flex items-center justify-center mr-4">
                    0
                  </div>
                  <div>
                    <h4 className="text-h3 mb-1">Zero Vendor Lock-in</h4>
                    <p className="text-body text-gray-600">No proprietary dependencies or hidden restrictions</p>
                  </div>
                </div>
                <div className="flex items-center">
                  <div className="w-8 h-8 bg-red-600 text-white text-sm font-bold rounded-full flex items-center justify-center mr-4">
                    ∞
                  </div>
                  <div>
                    <h4 className="text-h3 mb-1">Perpetual Access</h4>
                    <p className="text-body text-gray-600">Free forever for educational and research purposes</p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Technology Stack */}
      <section className="section">
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Technology Stack</h2>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
            {technologyStack.map((stack, index) => (
              <div key={index} className="card-white">
                <h3 className="text-h3 mb-4">{stack.category}</h3>
                <ul className="space-y-2">
                  {stack.items.map((item, idx) => (
                    <li key={idx} className="text-small text-gray-600">
                      • {item}
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Contact Information */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container text-center">
          <h2 className="text-h2 mb-6">Get in Touch</h2>
          <p className="text-large mb-8 max-w-2xl mx-auto">
            Whether you're interested in using MultiOS for education, contributing to development, 
            or exploring research collaborations, we'd love to hear from you.
          </p>
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-6 max-w-3xl mx-auto">
            <div className="card-white">
              <h3 className="text-h3 mb-3">Email</h3>
              <a href="mailto:hello@multios.org" className="text-body text-red-600 hover:text-red-700">
                hello@multios.org
              </a>
            </div>
            <div className="card-white">
              <h3 className="text-h3 mb-3">GitHub</h3>
              <a href="https://github.com/multios" target="_blank" rel="noopener noreferrer" className="text-body text-red-600 hover:text-red-700">
                github.com/multios
              </a>
            </div>
            <div className="card-white">
              <h3 className="text-h3 mb-3">Discord</h3>
              <a href="https://discord.gg/multios" target="_blank" rel="noopener noreferrer" className="text-body text-red-600 hover:text-red-700">
                discord.gg/multios
              </a>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
};

export default AboutPage;