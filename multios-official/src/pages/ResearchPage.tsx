import React from 'react';

const ResearchPage = () => {
  const researchAreas = [
    {
      title: "Operating Systems Architecture",
      description: "Novel approaches to OS design, kernel architecture, and system organization",
      topics: [
        "Microkernel vs. Monolithic architectures",
        "Capability-based security systems",
        "Distributed operating systems",
        "Real-time operating systems"
      ]
    },
    {
      title: "Memory Management",
      description: "Advanced memory allocation strategies and virtual memory systems",
      topics: [
        "Non-uniform memory access (NUMA) optimization",
        "Persistent memory integration",
        "Memory compression techniques",
        "Garbage collection in systems programming"
      ]
    },
    {
      title: "Hardware-Software Co-design",
      description: "Integration of hardware innovations with operating system software",
      topics: [
        "Heterogeneous computing support",
        "Hardware acceleration frameworks",
        "Emerging memory technologies",
        "Quantum-classical hybrid systems"
      ]
    },
    {
      title: "Security and Privacy",
      description: "Security mechanisms and privacy-preserving technologies",
      topics: [
        "Zero-trust operating systems",
        "Homomorphic encryption integration",
        "Secure boot and trusted execution",
        "Privacy-preserving machine learning"
      ]
    }
  ];

  const academicProjects = [
    {
      title: "CPU Testing Framework",
      description: "Comprehensive testing and validation suite for CPU architectures",
      institution: "Multiple Universities",
      status: "Active",
      features: [
        "Automated CPU feature detection",
        "Instruction set verification",
        "Performance benchmarking",
        "Compatibility testing"
      ]
    },
    {
      title: "Educational Curriculum Integration",
      description: "Development of OS curriculum materials and assessment tools",
      institution: "Computer Science Departments",
      status: "Ongoing",
      features: [
        "Interactive learning modules",
        "Automated grading systems",
        "Progress tracking tools",
        "Curriculum assessment"
      ]
    },
    {
      title: "Research API Platform",
      description: "Standardized API for OS research experiments and data collection",
      institution: "Research Institutions",
      status: "Beta",
      features: [
        "Experiment orchestration",
        "Data collection framework",
        "Performance monitoring",
        "Reproducible research"
      ]
    }
  ];

  const publications = [
    {
      title: "MultiOS: A Comprehensive Educational Operating System",
      authors: "Research Team",
      venue: "OSDI 2025",
      year: "2025",
      status: "Published",
      type: "Conference Paper"
    },
    {
      title: "Cross-Platform Operating Systems Development with Rust",
      authors: "Development Team",
      venue: "USENIX ATC 2025",
      year: "2025",
      status: "Submitted",
      type: "Conference Paper"
    },
    {
      title: "Memory-Safe Systems Programming in Educational Context",
      authors: "Educational Team",
      venue: "SIGCSE 2025",
      year: "2025",
      status: "In Review",
      type: "Conference Paper"
    }
  ];

  return (
    <div className="pt-16">
      {/* Page Header */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container text-center">
          <h1 className="text-h1 mb-6">Research & Academic</h1>
          <p className="text-large max-w-3xl mx-auto">
            MultiOS serves as a research platform for advancing operating systems concepts and methodologies. 
            Our open-source approach facilitates reproducible research and accelerates innovation in systems programming.
          </p>
        </div>
      </section>

      {/* Research Platform */}
      <section className="section">
        <div className="container">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-16 items-center">
            <div>
              <h2 className="text-h2 mb-6">Research Platform</h2>
              <p className="text-body mb-6">
                MultiOS provides researchers with a modern, open-source platform for conducting operating systems research. 
                The codebase serves as both a learning tool and a research platform, enabling experimental work in 
                real-world operating system contexts.
              </p>
              <div className="space-y-4">
                <div className="swiss-border-left pl-6">
                  <h4 className="text-h3 mb-2">Reproducible Research</h4>
                  <p className="text-body text-gray-600">Standardized experimental frameworks and data collection</p>
                </div>
                <div className="swiss-border-left pl-6">
                  <h4 className="text-h3 mb-2">Industry Collaboration</h4>
                  <p className="text-body text-gray-600">Bridge between academic research and industry needs</p>
                </div>
                <div className="swiss-border-left pl-6">
                  <h4 className="text-h3 mb-2">Educational Integration</h4>
                  <p className="text-body text-gray-600">Research findings directly integrated into curriculum</p>
                </div>
              </div>
            </div>
            <div>
              <img 
                src="/images/data_center_network_infrastructure_data_flow.jpg"
                alt="Research Platform"
                className="w-full h-auto border border-gray-light"
              />
            </div>
          </div>
        </div>
      </section>

      {/* Research Areas */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Research Areas</h2>
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
            {researchAreas.map((area, index) => (
              <div key={index} className="card-white">
                <h3 className="text-h3 mb-4">{area.title}</h3>
                <p className="text-body text-gray-600 mb-6">{area.description}</p>
                <div className="space-y-2">
                  {area.topics.map((topic, idx) => (
                    <div key={idx} className="flex items-start">
                      <div className="w-2 h-2 bg-red-600 mt-2 mr-3 flex-shrink-0"></div>
                      <span className="text-small text-gray-600">{topic}</span>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Academic Projects */}
      <section className="section">
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Academic Projects</h2>
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
            {academicProjects.map((project, index) => (
              <div key={index} className="card">
                <div className="mb-4">
                  <h3 className="text-h3 mb-2">{project.title}</h3>
                  <div className="flex justify-between text-small text-gray-600 mb-2">
                    <span>{project.institution}</span>
                    <span className={`px-2 py-1 font-bold uppercase tracking-wider text-xs ${
                      project.status === 'Active' ? 'bg-green-100 text-green-800' : 
                      project.status === 'Ongoing' ? 'bg-blue-100 text-blue-800' : 
                      'bg-yellow-100 text-yellow-800'
                    }`}>
                      {project.status}
                    </span>
                  </div>
                </div>
                <p className="text-body text-gray-600 mb-4">{project.description}</p>
                <div className="space-y-2">
                  {project.features.map((feature, idx) => (
                    <div key={idx} className="text-small text-gray-600">
                      • {feature}
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Research API */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Research API</h2>
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
            <div className="card-white">
              <h3 className="text-h3 mb-4">API Features</h3>
              <ul className="space-y-3">
                <li className="flex items-start">
                  <div className="w-2 h-2 bg-red-600 mt-2 mr-3 flex-shrink-0"></div>
                  <div>
                    <span className="text-body font-medium">Experiment Orchestration</span>
                    <p className="text-small text-gray-600">Automated experiment setup and execution</p>
                  </div>
                </li>
                <li className="flex items-start">
                  <div className="w-2 h-2 bg-red-600 mt-2 mr-3 flex-shrink-0"></div>
                  <div>
                    <span className="text-body font-medium">Data Collection</span>
                    <p className="text-small text-gray-600">Standardized metrics and performance data</p>
                  </div>
                </li>
                <li className="flex items-start">
                  <div className="w-2 h-2 bg-red-600 mt-2 mr-3 flex-shrink-0"></div>
                  <div>
                    <span className="text-body font-medium">Performance Monitoring</span>
                    <p className="text-small text-gray-600">Real-time system performance tracking</p>
                  </div>
                </li>
                <li className="flex items-start">
                  <div className="w-2 h-2 bg-red-600 mt-2 mr-3 flex-shrink-0"></div>
                  <div>
                    <span className="text-body font-medium">Reproducible Research</span>
                    <p className="text-small text-gray-600">Experiment replay and validation</p>
                  </div>
                </li>
              </ul>
            </div>
            <div className="card-white">
              <h3 className="text-h3 mb-4">Usage Example</h3>
              <div className="code-block">
                <pre className="text-sm text-white">{`// Initialize research experiment
let experiment = Experiment::new("memory_performance")
    .with_config(MemoryConfig {
        allocation_size: 1_000_000,
        test_duration: Duration::from_secs(60),
        iterations: 100
    })?;

// Add performance counters
experiment.add_counter("cpu_usage");
experiment.add_counter("memory_footprint");
experiment.add_counter("context_switches");

// Run experiment
let results = experiment.run()?;

// Analyze results
let analysis = results.analyze()
    .with_comparison("baseline")?;

println!("Performance improvement: {:?}", 
         analysis.percentage_improvement());`}</pre>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Publications */}
      <section className="section">
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Publications</h2>
          <div className="space-y-6">
            {publications.map((pub, index) => (
              <div key={index} className="card-white">
                <div className="grid grid-cols-1 lg:grid-cols-4 gap-6 items-start">
                  <div className="lg:col-span-3">
                    <h3 className="text-h3 mb-2">{pub.title}</h3>
                    <p className="text-body text-gray-600 mb-2">{pub.authors}</p>
                    <p className="text-small text-gray-500">{pub.venue} • {pub.year}</p>
                  </div>
                  <div className="flex flex-col items-end">
                    <span className={`px-3 py-1 text-xs font-bold uppercase tracking-wider mb-2 ${
                      pub.status === 'Published' ? 'bg-green-100 text-green-800' : 
                      pub.status === 'Submitted' ? 'bg-blue-100 text-blue-800' : 
                      'bg-yellow-100 text-yellow-800'
                    }`}>
                      {pub.status}
                    </span>
                    <span className="text-small text-gray-600">{pub.type}</span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Collaboration Opportunities */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container">
          <div className="text-center max-w-3xl mx-auto">
            <h2 className="text-h2 mb-6">Collaboration Opportunities</h2>
            <p className="text-large mb-8">
              We welcome research collaborations with universities, industry partners, and individual researchers. 
              Join our community to advance operating systems research together.
            </p>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-6 mb-8">
              <div className="card-white">
                <h3 className="text-h3 mb-3">University Partnerships</h3>
                <p className="text-body text-gray-600">Collaborative research projects and student thesis supervision</p>
              </div>
              <div className="card-white">
                <h3 className="text-h3 mb-3">Grant Opportunities</h3>
                <p className="text-body text-gray-600">Joint applications for research funding and infrastructure support</p>
              </div>
              <div className="card-white">
                <h3 className="text-h3 mb-3">Industry Research</h3>
                <p className="text-body text-gray-600">Collaborative projects between academia and industry</p>
              </div>
            </div>
            <div className="flex flex-col sm:flex-row justify-center gap-4">
              <a href="mailto:research@multios.org" className="btn btn-primary btn-large">
                Contact Research Team
              </a>
              <a href="/docs/research" className="btn btn-secondary btn-large">
                Research Documentation
              </a>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
};

export default ResearchPage;