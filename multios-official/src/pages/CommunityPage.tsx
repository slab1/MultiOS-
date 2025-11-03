import React from 'react';

const CommunityPage = () => {
  const communityStats = [
    { number: "1,200+", label: "Contributors", description: "Active community members worldwide" },
    { number: "15,000+", label: "Commits", description: "Code contributions and improvements" },
    { number: "250+", label: "Issues Closed", description: "Bugs fixed and features implemented" },
    { number: "50+", label: "Pull Requests", description: "Code reviews and contributions" }
  ];

  const getInvolvedAreas = [
    {
      title: "Code Contributions",
      description: "Contribute to the core MultiOS codebase",
      ways: [
        "Submit pull requests with new features",
        "Fix bugs and improve code quality",
        "Add test coverage and documentation",
        "Optimize performance and memory usage"
      ],
      icon: "💻"
    },
    {
      title: "Documentation",
      description: "Help improve documentation and guides",
      ways: [
        "Write tutorials and code examples",
        "Improve API documentation",
        "Create visual diagrams and illustrations",
        "Translate documentation to other languages"
      ],
      icon: "📝"
    },
    {
      title: "Testing & QA",
      description: "Help ensure MultiOS quality and reliability",
      ways: [
        "Test on different hardware platforms",
        "Report bugs and performance issues",
        "Create automated test suites",
        "Perform security audits"
      ],
      icon: "🧪"
    },
    {
      title: "Community Support",
      description: "Help other users and contribute to discussions",
      ways: [
        "Answer questions in Discord and forums",
        "Mentor new contributors",
        "Organize local meetups and workshops",
        "Share your MultiOS experience"
      ],
      icon: "🤝"
    }
  ];

  const communicationChannels = [
    {
      name: "Discord",
      description: "Real-time chat for discussions and support",
      url: "https://discord.gg/multios",
      members: "2,500+ members",
      icon: "💬"
    },
    {
      name: "GitHub Discussions",
      description: "Structured discussions and feature proposals",
      url: "https://github.com/multios/multios/discussions",
      members: "Active discussions",
      icon: "📋"
    },
    {
      name: "GitHub Issues",
      description: "Bug reports and feature requests",
      url: "https://github.com/multios/multios/issues",
      members: "Track progress",
      icon: "🐛"
    },
    {
      name: "Weekly Meetings",
      description: "Virtual meetings for contributors",
      url: "https://calendar.google.com/calendar/multios",
      members: "Open to all",
      icon: "📅"
    }
  ];

  return (
    <div className="pt-16">
      {/* Page Header */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container text-center">
          <h1 className="text-h1 mb-6">Community</h1>
          <p className="text-large max-w-3xl mx-auto">
            Join a vibrant community of operating systems enthusiasts, educators, and developers 
            working together to advance the future of systems programming and education.
          </p>
        </div>
      </section>

      {/* Community Stats */}
      <section className="section">
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Community Stats</h2>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
            {communityStats.map((stat, index) => (
              <div key={index} className="card text-center">
                <div className="text-4xl font-bold text-black mb-2">
                  {stat.number}
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

      {/* Get Involved */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Get Involved</h2>
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
            {getInvolvedAreas.map((area, index) => (
              <div key={index} className="card-white">
                <div className="flex items-center mb-4">
                  <div className="text-3xl mr-4">{area.icon}</div>
                  <div>
                    <h3 className="text-h3">{area.title}</h3>
                    <p className="text-body text-gray-600">{area.description}</p>
                  </div>
                </div>
                <ul className="space-y-2">
                  {area.ways.map((way, idx) => (
                    <li key={idx} className="flex items-start">
                      <div className="w-2 h-2 bg-red-600 mt-2 mr-3 flex-shrink-0"></div>
                      <span className="text-small text-gray-600">{way}</span>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Developer Portal */}
      <section className="section">
        <div className="container">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-16 items-center">
            <div>
              <h2 className="text-h2 mb-6">Developer Portal</h2>
              <p className="text-body mb-6">
                Our developer portal provides comprehensive tools and resources for MultiOS development. 
                Access APIs, documentation, testing frameworks, and contribution guidelines all in one place.
              </p>
              <div className="space-y-3">
                <a href="/docs/api" className="block text-body text-red-600 hover:text-red-700">
                  → API Reference and Documentation
                </a>
                <a href="/docs/development" className="block text-body text-red-600 hover:text-red-700">
                  → Development Guidelines
                </a>
                <a href="/docs/testing" className="block text-body text-red-600 hover:text-red-700">
                  → Testing Frameworks
                </a>
                <a href="/docs/contributing" className="block text-body text-red-600 hover:text-red-700">
                  → Contribution Guide
                </a>
                <a href="/tools" className="block text-body text-red-600 hover:text-red-700">
                  → Development Tools
                </a>
              </div>
            </div>
            <div>
              <img 
                src="/images/vscode_modern_programming_editor_interface_components.jpg"
                alt="Developer Portal"
                className="w-full h-auto border border-gray-light"
              />
            </div>
          </div>
        </div>
      </section>

      {/* App Store */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container">
          <h2 className="text-h2 text-center mb-12">App Store</h2>
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
            <div className="card-white">
              <h3 className="text-h3 mb-4">Educational Apps</h3>
              <p className="text-body text-gray-600 mb-4">
                Educational applications and tools built specifically for learning operating systems concepts.
              </p>
              <ul className="space-y-2">
                <li className="text-small text-gray-600">• OS Simulation Tools</li>
                <li className="text-small text-gray-600">• Algorithm Visualizers</li>
                <li className="text-small text-gray-600">• Interactive Tutorials</li>
                <li className="text-small text-gray-600">• Practice Exercises</li>
              </ul>
            </div>

            <div className="card-white">
              <h3 className="text-h3 mb-4">System Utilities</h3>
              <p className="text-body text-gray-600 mb-4">
                Essential utilities and tools for system administration and development.
              </p>
              <ul className="space-y-2">
                <li className="text-small text-gray-600">• System Monitors</li>
                <li className="text-small text-gray-600">• Debugging Tools</li>
                <li className="text-small text-gray-600">• Performance Analyzers</li>
                <li className="text-small text-gray-600">• Development IDEs</li>
              </ul>
            </div>

            <div className="card-white">
              <h3 className="text-h3 mb-4">Research Tools</h3>
              <p className="text-body text-gray-600 mb-4">
                Specialized tools for operating systems research and academic work.
              </p>
              <ul className="space-y-2">
                <li className="text-small text-gray-600">• Benchmarking Suites</li>
                <li className="text-small text-gray-600">• Research Frameworks</li>
                <li className="text-small text-gray-600">• Data Collection Tools</li>
                <li className="text-small text-gray-600">• Analysis Platforms</li>
              </ul>
            </div>
          </div>
          <div className="text-center mt-8">
            <a href="/app-store" className="btn btn-primary">
              Explore App Store
            </a>
          </div>
        </div>
      </section>

      {/* Package Manager */}
      <section className="section">
        <div className="container">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-16 items-center">
            <div>
              <img 
                src="/images/clean_modern_data_center_server_infrastructure.jpg"
                alt="Package Manager"
                className="w-full h-auto border border-gray-light"
              />
            </div>
            <div>
              <h2 className="text-h2 mb-6">Package Manager</h2>
              <p className="text-body mb-6">
                MultiOS Package Manager provides a centralized ecosystem for distributing and managing 
                applications, libraries, and development tools specifically optimized for MultiOS.
              </p>
              <div className="space-y-4">
                <div className="swiss-border-left pl-6">
                  <h4 className="text-h3 mb-2">Easy Installation</h4>
                  <p className="text-body text-gray-600">Simple command-line interface for package management</p>
                </div>
                <div className="swiss-border-left pl-6">
                  <h4 className="text-h3 mb-2">Dependency Resolution</h4>
                  <p className="text-body text-gray-600">Intelligent dependency management and conflict resolution</p>
                </div>
                <div className="swiss-border-left pl-6">
                  <h4 className="text-h3 mb-2">Security</h4>
                  <p className="text-body text-gray-600">Cryptographic verification and sandboxed execution</p>
                </div>
              </div>
              <div className="mt-6">
                <a href="/pkg-manager" className="btn btn-primary">
                  Learn More
                </a>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Communication Channels */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Communication Channels</h2>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
            {communicationChannels.map((channel, index) => (
              <div key={index} className="card-white text-center">
                <div className="text-4xl mb-4">{channel.icon}</div>
                <h3 className="text-h3 mb-3">{channel.name}</h3>
                <p className="text-body text-gray-600 mb-4">{channel.description}</p>
                <div className="text-small text-gray-500 mb-4">{channel.members}</div>
                <a href={channel.url} target="_blank" rel="noopener noreferrer" className="btn btn-secondary">
                  Join
                </a>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Join Community CTA */}
      <section className="section">
        <div className="container text-center">
          <h2 className="text-h2 mb-6">Join the MultiOS Community</h2>
          <p className="text-large mb-8 max-w-2xl mx-auto">
            Whether you're a student, educator, researcher, or industry professional, 
            there's a place for you in the MultiOS community. Start contributing today!
          </p>
          <div className="flex flex-col sm:flex-row justify-center gap-4">
            <a href="https://github.com/multios/multios" className="btn btn-primary btn-large">
              Start Contributing
            </a>
            <a href="https://discord.gg/multios" className="btn btn-secondary btn-large">
              Join Discord
            </a>
          </div>
        </div>
      </section>
    </div>
  );
};

export default CommunityPage;