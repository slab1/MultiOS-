import React from 'react';
import { Link } from 'react-router-dom';

const EducatorsPage = () => {
  const educationalBenefits = [
    {
      title: "Comprehensive Learning Materials",
      description: "Extensive documentation, tutorials, and hands-on examples designed specifically for operating systems education.",
      features: ["Step-by-step tutorials", "Interactive demonstrations", "Complete API documentation", "Troubleshooting guides"]
    },
    {
      title: "Cross-Platform Understanding",
      description: "Study operating system concepts across x86_64, ARM64, and RISC-V architectures in one codebase.",
      features: ["Comparative architecture study", "Hardware abstraction concepts", "Platform-specific optimizations", "Universal OS principles"]
    },
    {
      title: "Modern Development Practices",
      description: "Learn with Rust programming language, featuring memory safety and modern systems programming techniques.",
      features: ["Memory-safe systems programming", "Modern language features", "Best practices implementation", "Production-quality code"]
    },
    {
      title: "Real-World Applications",
      description: "Connect theoretical concepts with practical implementation through production-ready code examples.",
      features: ["Working code examples", "Real performance metrics", "Industry-standard patterns", "Scalable architectures"]
    }
  ];

  const curriculumResources = [
    {
      title: "Course Materials",
      description: "Complete curriculum packages for undergraduate and graduate OS courses",
      items: [
        "Syllabus templates and course outlines",
        "Weekly lecture materials and slides",
        "Hands-on lab exercises and projects",
        "Assessment rubrics and grading guides"
      ],
      image: "/images/educational_technology_multi_platform_learning.jpg"
    },
    {
      title: "Lab Exercises",
      description: "Progressive lab exercises from basic concepts to advanced implementation",
      items: [
        "Memory management simulation",
        "Process scheduling algorithms",
        "Filesystem implementation",
        "Device driver development",
        "System call interface design"
      ],
      image: "/images/education-multi-device-cross-platform-learning-infographic.jpg"
    },
    {
      title: "Assignment Templates",
      description: "Ready-to-use assignment templates for various skill levels",
      items: [
        "Beginner: Basic system calls",
        "Intermediate: Kernel module development",
        "Advanced: Complete subsystem implementation",
        "Expert: Optimization and performance tuning",
        "Research: Novel algorithm development"
      ],
      image: "/images/education_technology_multi_platform_e_learning_devices.jpg"
    }
  ];

  const certificationPrograms = [
    {
      level: "MultiOS Fundamentals",
      duration: "40 hours",
      prerequisites: "Basic programming experience",
      description: "Introduction to operating systems concepts using MultiOS",
      modules: [
        "Operating System Basics",
        "Process and Thread Management", 
        "Memory Management",
        "File Systems",
        "Device Drivers"
      ]
    },
    {
      level: "MultiOS Development",
      duration: "80 hours", 
      prerequisites: "C/Rust programming, Systems knowledge",
      description: "Hands-on development of OS components",
      modules: [
        "Kernel Development",
        "Driver Implementation",
        "Performance Optimization",
        "Testing and Debugging",
        "Cross-platform Development"
      ]
    },
    {
      level: "MultiOS Expert",
      duration: "120 hours",
      prerequisites: "Advanced programming, OS course completion", 
      description: "Advanced topics and research projects",
      modules: [
        "Advanced Architecture",
        "Security Implementation",
        "Research Projects",
        "Industry Applications",
        "Teaching Assistant Training"
      ]
    }
  ];

  const teachingTools = [
    {
      title: "Interactive Labs",
      description: "Browser-based lab environments with real-time feedback",
      icon: "🧪"
    },
    {
      title: "Code Browser",
      description: "Navigate MultiOS codebase with educational annotations",
      icon: "🔍"
    },
    {
      title: "Visualization Tools",
      description: "Interactive diagrams for complex concepts",
      icon: "📊"
    },
    {
      title: "Assessment Platform",
      description: "Automated grading and progress tracking",
      icon: "📝"
    }
  ];

  return (
    <div className="pt-16">
      {/* Page Header */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container text-center">
          <h1 className="text-h1 mb-6">For Educators</h1>
          <p className="text-large max-w-3xl mx-auto">
            MultiOS provides educators with comprehensive tools and materials for teaching operating systems concepts. 
            From introductory courses to advanced research projects, MultiOS offers practical, hands-on learning experiences.
          </p>
        </div>
      </section>

      {/* Why MultiOS */}
      <section className="section">
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Why Choose MultiOS for Education</h2>
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
            {educationalBenefits.map((benefit, index) => (
              <div key={index} className="card-white">
                <h3 className="text-h3 mb-4">{benefit.title}</h3>
                <p className="text-body text-gray-600 mb-6">{benefit.description}</p>
                <div className="space-y-2">
                  {benefit.features.map((feature, idx) => (
                    <div key={idx} className="flex items-center">
                      <div className="w-2 h-2 bg-red-600 mr-3"></div>
                      <span className="text-small text-gray-600">{feature}</span>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Curriculum Resources */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Curriculum Resources</h2>
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
            {curriculumResources.map((resource, index) => (
              <div key={index} className="card">
                <div className="mb-6">
                  <img 
                    src={resource.image}
                    alt={resource.title}
                    className="w-full h-48 object-cover border border-gray-light"
                  />
                </div>
                <h3 className="text-h3 mb-4">{resource.title}</h3>
                <p className="text-body text-gray-600 mb-4">{resource.description}</p>
                <ul className="space-y-2">
                  {resource.items.map((item, idx) => (
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

      {/* Certification Programs */}
      <section className="section">
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Certification Programs</h2>
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
            {certificationPrograms.map((program, index) => (
              <div key={index} className="card-white">
                <div className="mb-4">
                  <h3 className="text-h3 mb-2">{program.level}</h3>
                  <div className="flex justify-between text-small text-gray-600 mb-2">
                    <span>Duration: {program.duration}</span>
                    <span>Prerequisites: {program.prerequisites}</span>
                  </div>
                </div>
                <p className="text-body text-gray-600 mb-4">{program.description}</p>
                <div className="border-t border-gray-light pt-4">
                  <h4 className="text-small font-bold uppercase tracking-wider text-gray-600 mb-3">Modules</h4>
                  <ul className="space-y-1">
                    {program.modules.map((module, idx) => (
                      <li key={idx} className="text-small text-gray-600">• {module}</li>
                    ))}
                  </ul>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Academic Partnerships */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Academic Partnerships</h2>
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-16 items-center">
            <div>
              <h3 className="text-h2 mb-6">Collaborate with the MultiOS Project</h3>
              <p className="text-body mb-6">
                Join leading universities in advancing operating systems education. Our partnership program 
                provides institutions with early access to new features, direct collaboration opportunities, 
                and research support.
              </p>
              <div className="space-y-4">
                <div className="swiss-border-left pl-6">
                  <h4 className="text-h3 mb-2">Research Collaboration</h4>
                  <p className="text-body text-gray-600">Contribute to cutting-edge OS research with access to development resources</p>
                </div>
                <div className="swiss-border-left pl-6">
                  <h4 className="text-h3 mb-2">Curriculum Development</h4>
                  <p className="text-body text-gray-600">Co-develop educational materials and share best practices</p>
                </div>
                <div className="swiss-border-left pl-6">
                  <h4 className="text-h3 mb-2">Student Projects</h4>
                  <p className="text-body text-gray-600">Engage students with real-world OS development opportunities</p>
                </div>
              </div>
            </div>
            <div>
              <img 
                src="/images/professional_developers_team_collaboration_meeting_office.jpg"
                alt="Academic Collaboration"
                className="w-full h-auto border border-gray-light"
              />
            </div>
          </div>
        </div>
      </section>

      {/* Teaching Tools */}
      <section className="section">
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Teaching Tools</h2>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
            {teachingTools.map((tool, index) => (
              <div key={index} className="card-white text-center">
                <div className="text-4xl mb-4">{tool.icon}</div>
                <h3 className="text-h3 mb-3">{tool.title}</h3>
                <p className="text-body text-gray-600">{tool.description}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Get Started for Educators */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container text-center">
          <h2 className="text-h2 mb-6">Get Started for Educators</h2>
          <p className="text-large mb-8 max-w-2xl mx-auto">
            Ready to enhance your operating systems curriculum? Download our educator package 
            or contact our academic team for personalized support.
          </p>
          <div className="flex flex-col sm:flex-row justify-center gap-4">
            <a href="/download" className="btn btn-primary btn-large">
              Download Educator Package
            </a>
            <a href="mailto:educators@multios.org" className="btn btn-secondary btn-large">
              Contact Academic Team
            </a>
          </div>
        </div>
      </section>
    </div>
  );
};

export default EducatorsPage;