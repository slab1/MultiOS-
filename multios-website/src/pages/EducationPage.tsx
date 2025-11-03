import React from 'react';
import { Link } from 'react-router-dom';
import { useLanguage } from '../contexts/LanguageContext';
import { 
  Book, 
  Users, 
  Award, 
  Play, 
  Download, 
  ExternalLink,
  Star,
  CheckCircle,
  Target,
  Clock,
  TrendingUp,
  GraduationCap,
  Code,
  Lightbulb,
  Globe,
  BarChart3
} from 'lucide-react';

const EducationPage: React.FC = () => {
  const { t } = useLanguage();

  const learningPaths = [
    {
      level: 'Beginner',
      description: 'Introduction to OS concepts and Rust programming',
      duration: '4-6 weeks',
      courses: 6,
      color: 'from-green-500 to-green-600',
      topics: [
        'Computer Architecture Fundamentals',
        'Rust Programming Basics',
        'Operating System Concepts',
        'Memory Management Introduction',
        'Process and Thread Basics',
        'File System Fundamentals'
      ]
    },
    {
      level: 'Intermediate',
      description: 'Kernel development and systems programming',
      duration: '8-12 weeks',
      courses: 8,
      color: 'from-blue-500 to-blue-600',
      topics: [
        'Kernel Architecture Design',
        'System Call Implementation',
        'Interrupt Handling',
        'Device Driver Development',
        'Cross-Platform Programming',
        'Performance Optimization',
        'Testing and Debugging',
        'Security Fundamentals'
      ]
    },
    {
      level: 'Advanced',
      description: 'Multi-architecture and optimization techniques',
      duration: '12-16 weeks',
      courses: 10,
      color: 'from-purple-500 to-purple-600',
      topics: [
        'Multi-Architecture Support',
        'Advanced Memory Management',
        'Distributed Systems',
        'Real-Time Operating Systems',
        'Virtualization',
        'Container Technologies',
        'Compiler Integration',
        'Research Methodologies'
      ]
    },
    {
      level: 'Expert',
      description: 'Research projects and cutting-edge topics',
      duration: '16+ weeks',
      courses: 12,
      color: 'from-red-500 to-red-600',
      topics: [
        'Novel OS Architectures',
        'Quantum Computing Integration',
        'AI-Native Operating Systems',
        'Edge Computing Systems',
        'Academic Research Projects',
        'Industry Collaboration',
        'Open Source Contributions',
        'Technical Leadership'
      ]
    }
  ];

  const certifications = [
    {
      name: 'MultiOS Fundamentals',
      description: 'Core operating systems concepts and basic implementation',
      badge: '🏁',
      requirements: [
        'Complete Beginner pathway',
        'Pass comprehensive assessment',
        'Submit final project',
        'Community participation'
      ],
      validity: '3 years'
    },
    {
      name: 'Cross-Platform Development',
      description: 'Multi-architecture development expertise',
      badge: '🏗️',
      requirements: [
        'Complete Intermediate pathway',
        'Implement cross-platform features',
        'Performance optimization project',
        'Code review contributions'
      ],
      validity: '3 years'
    },
    {
      name: 'Advanced Systems Programming',
      description: 'Deep systems programming and optimization',
      badge: '🚀',
      requirements: [
        'Complete Advanced pathway',
        'Research project completion',
        'Technical presentation',
        'Mentoring contribution'
      ],
      validity: '3 years'
    },
    {
      name: 'Research Contributor',
      description: 'Significant research or contribution to the ecosystem',
      badge: '🎓',
      requirements: [
        'Original research publication',
        'Significant feature contribution',
        'Community leadership',
        'Technical mentorship'
      ],
      validity: 'Lifetime'
    }
  ];

  const caseStudies = [
    {
      university: 'Stanford University',
      course: 'CS 240 - Operating Systems',
      students: 120,
      semester: 'Fall 2024',
      outcome: '92% pass rate, 40% increase in practical skills',
      quote: 'MultiOS has revolutionized how we teach operating systems. Students gain hands-on experience with real-world OS development.',
      professor: 'Dr. Sarah Chen, CS Department Chair'
    },
    {
      university: 'MIT',
      course: '6.828 - Operating System Engineering',
      students: 80,
      semester: 'Spring 2024',
      outcome: '3x increase in research projects, 2x more industry internships',
      quote: 'The cross-platform nature and educational focus make MultiOS an ideal teaching platform.',
      professor: 'Prof. Michael Rodriguez, Systems Group'
    },
    {
      university: 'Carnegie Mellon',
      course: '15-410 - Operating Systems Design',
      students: 60,
      semester: 'Fall 2023',
      outcome: '100% graduation rate, 5 papers published',
      quote: 'MultiOS provides the perfect balance of theory and practice for advanced OS education.',
      professor: 'Dr. Emily Watson, CS Department'
    }
  ];

  const curriculumFeatures = [
    {
      icon: Target,
      title: 'Standards Aligned',
      description: 'ACM/IEEE curriculum standards compliance'
    },
    {
      icon: Code,
      title: 'Hands-On Coding',
      description: 'Every concept reinforced with practical exercises'
    },
    {
      icon: BarChart3,
      title: 'Progress Tracking',
      description: 'Automated assessment and progress monitoring'
    },
    {
      icon: Users,
      title: 'Peer Learning',
      description: 'Collaborative projects and code reviews'
    },
    {
      icon: Lightbulb,
      title: 'Project-Based',
      description: 'Real-world projects and research opportunities'
    },
    {
      icon: Globe,
      title: 'Multi-Platform',
      description: 'Learn across x86, ARM, and RISC-V architectures'
    }
  ];

  return (
    <div className="min-h-screen bg-white">
      {/* Hero Section */}
      <section className="bg-gradient-to-br from-blue-600 via-purple-600 to-indigo-800 py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center text-white">
            <h1 className="text-4xl md:text-5xl font-bold mb-6">
              Educational Excellence
            </h1>
            <p className="text-xl md:text-2xl text-blue-100 max-w-3xl mx-auto mb-8">
              Master operating systems development through our comprehensive curriculum, 
              hands-on exercises, and industry-recognized certifications
            </p>
            <div className="flex flex-col sm:flex-row gap-4 justify-center">
              <Link
                to="/demos"
                className="inline-flex items-center px-8 py-4 bg-white text-blue-600 font-semibold rounded-lg hover:bg-blue-50 transition-colors"
              >
                <Play className="w-5 h-5 mr-2" />
                Start Learning
              </Link>
              <a
                href="#curriculum"
                className="inline-flex items-center px-8 py-4 bg-transparent border-2 border-white text-white font-semibold rounded-lg hover:bg-white hover:text-blue-600 transition-colors"
              >
                <Book className="w-5 h-5 mr-2" />
                Explore Curriculum
              </a>
            </div>
          </div>
        </div>
      </section>

      {/* Curriculum Features */}
      <section id="curriculum" className="py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-3xl font-bold text-gray-900 mb-4">Curriculum Highlights</h2>
            <p className="text-xl text-gray-600 max-w-3xl mx-auto">
              Comprehensive educational framework designed for modern operating systems development
            </p>
          </div>

          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
            {curriculumFeatures.map((feature, index) => {
              const Icon = feature.icon;
              return (
                <div key={index} className="text-center p-6 bg-gray-50 rounded-xl hover:bg-gray-100 transition-colors">
                  <div className="w-16 h-16 bg-gradient-to-r from-blue-500 to-purple-600 rounded-full flex items-center justify-center mx-auto mb-4">
                    <Icon className="w-8 h-8 text-white" />
                  </div>
                  <h3 className="text-lg font-semibold text-gray-900 mb-2">{feature.title}</h3>
                  <p className="text-gray-600">{feature.description}</p>
                </div>
              );
            })}
          </div>
        </div>
      </section>

      {/* Learning Paths */}
      <section className="py-20 bg-gray-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-3xl font-bold text-gray-900 mb-4">Learning Pathways</h2>
            <p className="text-xl text-gray-600">
              Structured progression from fundamentals to advanced research
            </p>
          </div>

          <div className="grid lg:grid-cols-2 gap-8">
            {learningPaths.map((path, index) => (
              <div key={index} className="bg-white rounded-xl shadow-lg overflow-hidden hover:shadow-xl transition-shadow">
                <div className={`h-2 bg-gradient-to-r ${path.color}`}></div>
                <div className="p-6">
                  <div className="flex items-center justify-between mb-4">
                    <h3 className="text-2xl font-bold text-gray-900">{path.level}</h3>
                    <div className="flex items-center space-x-4 text-sm text-gray-500">
                      <div className="flex items-center">
                        <Clock className="w-4 h-4 mr-1" />
                        {path.duration}
                      </div>
                      <div className="flex items-center">
                        <Book className="w-4 h-4 mr-1" />
                        {path.courses} courses
                      </div>
                    </div>
                  </div>
                  <p className="text-gray-600 mb-4">{path.description}</p>
                  <div className="space-y-2">
                    <h4 className="font-semibold text-gray-900">Key Topics:</h4>
                    <div className="grid grid-cols-1 gap-2">
                      {path.topics.map((topic, topicIndex) => (
                        <div key={topicIndex} className="flex items-center text-sm text-gray-600">
                          <CheckCircle className="w-4 h-4 text-green-500 mr-2 flex-shrink-0" />
                          {topic}
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

      {/* Certifications */}
      <section className="py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-3xl font-bold text-gray-900 mb-4">Certification Programs</h2>
            <p className="text-xl text-gray-600">
              Industry-recognized credentials to validate your expertise
            </p>
          </div>

          <div className="grid md:grid-cols-2 gap-8">
            {certifications.map((cert, index) => (
              <div key={index} className="bg-white border border-gray-200 rounded-xl p-6 shadow-lg hover:shadow-xl transition-shadow">
                <div className="flex items-center mb-4">
                  <span className="text-4xl mr-4">{cert.badge}</span>
                  <div>
                    <h3 className="text-xl font-bold text-gray-900">{cert.name}</h3>
                    <p className="text-gray-600">{cert.description}</p>
                  </div>
                </div>
                <div className="mb-4">
                  <h4 className="font-semibold text-gray-900 mb-2">Requirements:</h4>
                  <ul className="space-y-1">
                    {cert.requirements.map((req, reqIndex) => (
                      <li key={reqIndex} className="text-sm text-gray-600 flex items-center">
                        <Star className="w-3 h-3 text-yellow-500 mr-2 flex-shrink-0" />
                        {req}
                      </li>
                    ))}
                  </ul>
                </div>
                <div className="flex items-center justify-between text-sm">
                  <span className="text-gray-500">Valid for {cert.validity}</span>
                  <button className="text-blue-600 hover:text-blue-700 font-medium">
                    Learn More
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Case Studies */}
      <section className="py-20 bg-gray-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-3xl font-bold text-gray-900 mb-4">Academic Partnerships</h2>
            <p className="text-xl text-gray-600">
              Real results from universities using MultiOS in their curriculum
            </p>
          </div>

          <div className="space-y-8">
            {caseStudies.map((study, index) => (
              <div key={index} className="bg-white rounded-xl shadow-lg p-8">
                <div className="grid lg:grid-cols-3 gap-6">
                  <div className="lg:col-span-2">
                    <div className="flex items-center mb-4">
                      <h3 className="text-2xl font-bold text-gray-900 mr-4">{study.university}</h3>
                      <span className="bg-blue-100 text-blue-800 px-3 py-1 rounded-full text-sm">
                        {study.semester}
                      </span>
                    </div>
                    <p className="text-gray-600 mb-4">{study.course}</p>
                    <blockquote className="text-lg italic text-gray-700 mb-4">
                      "{study.quote}"
                    </blockquote>
                    <p className="text-sm text-gray-500">— {study.professor}</p>
                  </div>
                  <div className="space-y-4">
                    <div className="text-center p-4 bg-gray-50 rounded-lg">
                      <div className="text-3xl font-bold text-blue-600">{study.students}</div>
                      <div className="text-sm text-gray-600">Students</div>
                    </div>
                    <div className="text-center p-4 bg-green-50 rounded-lg">
                      <div className="text-2xl font-bold text-green-600">{study.outcome}</div>
                      <div className="text-sm text-gray-600">Outcome</div>
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Educational Resources */}
      <section className="py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-3xl font-bold text-gray-900 mb-4">Educational Resources</h2>
            <p className="text-xl text-gray-600">
              Comprehensive tools and materials for educators and students
            </p>
          </div>

          <div className="grid md:grid-cols-3 gap-8">
            <div className="text-center">
              <div className="w-16 h-16 bg-gradient-to-r from-blue-500 to-blue-600 rounded-full flex items-center justify-center mx-auto mb-4">
                <GraduationCap className="w-8 h-8 text-white" />
              </div>
              <h3 className="text-xl font-semibold text-gray-900 mb-2">For Educators</h3>
              <p className="text-gray-600 mb-4">
                Complete curriculum materials, assessment tools, and instructor guides
              </p>
              <Link to="/documentation" className="text-blue-600 hover:text-blue-700 font-medium">
                Access Educator Resources →
              </Link>
            </div>

            <div className="text-center">
              <div className="w-16 h-16 bg-gradient-to-r from-green-500 to-green-600 rounded-full flex items-center justify-center mx-auto mb-4">
                <Users className="w-8 h-8 text-white" />
              </div>
              <h3 className="text-xl font-semibold text-gray-900 mb-2">For Students</h3>
              <p className="text-gray-600 mb-4">
                Interactive tutorials, practice exercises, and collaborative learning tools
              </p>
              <Link to="/demos" className="text-green-600 hover:text-green-700 font-medium">
                Start Learning →
              </Link>
            </div>

            <div className="text-center">
              <div className="w-16 h-16 bg-gradient-to-r from-purple-500 to-purple-600 rounded-full flex items-center justify-center mx-auto mb-4">
                <TrendingUp className="w-8 h-8 text-white" />
              </div>
              <h3 className="text-xl font-semibold text-gray-900 mb-2">For Institutions</h3>
              <p className="text-gray-600 mb-4">
                Course templates, integration tools, and institutional reporting
              </p>
              <a href="mailto:education@multios.org" className="text-purple-600 hover:text-purple-700 font-medium">
                Contact Our Team →
              </a>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-20 bg-gradient-to-r from-blue-600 to-purple-600">
        <div className="max-w-4xl mx-auto text-center px-4 sm:px-6 lg:px-8">
          <h2 className="text-3xl font-bold text-white mb-6">
            Ready to Transform Your OS Education?
          </h2>
          <p className="text-xl text-blue-100 mb-8">
            Join leading universities and thousands of students in learning modern operating systems development
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Link
              to="/demos"
              className="inline-flex items-center px-8 py-4 bg-white text-blue-600 font-semibold rounded-lg hover:bg-blue-50 transition-colors"
            >
              <Play className="w-5 h-5 mr-2" />
              Try Interactive Demos
            </Link>
            <a
              href="mailto:education@multios.org"
              className="inline-flex items-center px-8 py-4 bg-transparent border-2 border-white text-white font-semibold rounded-lg hover:bg-white hover:text-blue-600 transition-colors"
            >
              <Users className="w-5 h-5 mr-2" />
              Academic Partnerships
            </a>
          </div>
        </div>
      </section>
    </div>
  );
};

export default EducationPage;