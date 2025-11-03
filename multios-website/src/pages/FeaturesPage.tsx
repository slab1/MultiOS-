import React from 'react';
import { useLanguage } from '../contexts/LanguageContext';
import { 
  Cpu, 
  Shield, 
  Zap, 
  Globe, 
  Book, 
  Users, 
  Code, 
  Database,
  Network,
  Monitor,
  HardDrive,
  Settings,
  Award,
  GitBranch,
  Layers,
  Lock,
  BarChart3
} from 'lucide-react';

const FeaturesPage: React.FC = () => {
  const { t } = useLanguage();

  const featureCategories = [
    {
      title: 'Core Features',
      icon: Cpu,
      color: 'from-blue-500 to-blue-600',
      features: [
        {
          icon: Layers,
          title: 'Multi-Stage Bootloader',
          description: 'UEFI and legacy BIOS support with memory initialization and hardware enumeration',
          details: [
            'UEFI firmware integration with GOP graphics',
            'Legacy BIOS compatibility layer',
            'Automatic memory map detection',
            'Hardware device enumeration',
            'Configurable boot options'
          ]
        },
        {
          icon: Shield,
          title: 'Memory Safety',
          description: 'Rust-based memory safety with zero-cost abstractions',
          details: [
            'Stack canaries and bounds checking',
            'Use-after-free prevention',
            'MMU-based process isolation',
            'Hardware-accelerated encryption',
            'Secure memory allocation'
          ]
        },
        {
          icon: Zap,
          title: 'Performance Optimization',
          description: 'Highly optimized algorithms with hardware acceleration',
          details: [
            'Linear scaling up to 64+ cores',
            'NVMe Gen4 support (32 GB/s)',
            '10G Ethernet with offload',
            'Sub-microsecond context switches',
            'Real-time interrupt handling'
          ]
        }
      ]
    },
    {
      title: 'Architecture Support',
      icon: GitBranch,
      color: 'from-green-500 to-green-600',
      features: [
        {
          icon: Cpu,
          title: 'x86_64 Architecture',
          description: 'Full support with modern CPU extensions',
          details: [
            'Intel and AMD processor support',
            'SSE, AVX, AES-NI acceleration',
            'VirtualBox and VMware compatibility',
            'Hypervisor integration',
            'Legacy BIOS and UEFI support'
          ]
        },
        {
          icon: HardDrive,
          title: 'ARM64 Support',
          description: 'Complete ARMv8-A implementation',
          details: [
            'Apple Silicon (M1/M2) support',
            'ARMv8-A architecture features',
            'NEON SIMD instructions',
            'Crypto extension acceleration',
            'Raspberry Pi 4+ compatibility'
          ]
        },
        {
          icon: Database,
          title: 'RISC-V Implementation',
          description: 'RV64GC with standard extensions',
          details: [
            'SiFive and QEMU support',
            'RV64GC instruction set',
            'Extensible architecture',
            'Academic research platform',
            'Open standard compliance'
          ]
        }
      ]
    },
    {
      title: 'Educational Excellence',
      icon: Book,
      color: 'from-purple-500 to-purple-600',
      features: [
        {
          icon: Code,
          title: 'Comprehensive Curriculum',
          description: 'Structured learning paths from beginner to expert',
          details: [
            'ACM/IEEE standards alignment',
            'Hands-on coding exercises',
            'Progressive skill building',
            'Automated assessment system',
            'Certification programs'
          ]
        },
        {
          icon: Users,
          title: 'Community Learning',
          description: 'Open source collaboration and peer learning',
          details: [
            'Discord community support',
            'GitHub collaboration tools',
            'Monthly workshops and webinars',
            'Mentorship program',
            'Student project showcase'
          ]
        },
        {
          icon: Award,
          title: 'Assessment & Certification',
          description: 'Validated learning outcomes and skill recognition',
          details: [
            'MultiOS Fundamentals Certificate',
            'Cross-Platform Development Certificate',
            'Advanced Systems Programming Certificate',
            'Research Contributor Recognition',
            'Industry partnership programs'
          ]
        }
      ]
    },
    {
      title: 'Advanced Systems',
      icon: Settings,
      color: 'from-orange-500 to-orange-600',
      features: [
        {
          icon: Monitor,
          title: 'User Interface',
          description: 'Complete CLI and GUI toolkit',
          details: [
            'Comprehensive shell interface',
            'GUI toolkit with widgets',
            'Cross-platform UI components',
            'Accessibility compliance',
            'Interactive tutorials'
          ]
        },
        {
          icon: Network,
          title: 'IPC & Networking',
          description: 'Inter-process communication and networking stack',
          details: [
            'Message passing systems',
            'Shared memory management',
            'Network protocol stack',
            'WebSocket and socket APIs',
            'Distributed computing support'
          ]
        },
        {
          icon: Database,
          title: 'File Systems',
          description: 'Advanced file system with testing framework',
          details: [
            'MultiOS File System (MFS)',
            'Virtual File System (VFS)',
            'Comprehensive testing suite',
            'Performance optimization',
            'Integrity checking and recovery'
          ]
        }
      ]
    }
  ];

  const systemSpecs = [
    { label: 'Boot Time', value: '<5 seconds', description: 'On modern hardware' },
    { label: 'Memory Footprint', value: '2-50MB', description: 'Configuration dependent' },
    { label: 'Context Switch', value: '<1μs', description: 'On supported hardware' },
    { label: 'Interrupt Latency', value: '<10μs', description: 'Typical response time' },
    { label: 'Max Processes', value: '10,000+', description: 'Concurrent processes' },
    { label: 'Supported Devices', value: '1000+', description: 'Hot-plug capable' },
  ];

  const securityFeatures = [
    { icon: Lock, title: 'Secure Boot', description: 'Hardware-verified boot process' },
    { icon: Shield, title: 'Memory Protection', description: 'MMU-based isolation' },
    { icon: Database, title: 'Encryption Support', description: 'Hardware-accelerated crypto' },
    { icon: Network, title: 'Network Security', description: 'WPA3, TLS, secure protocols' },
  ];

  return (
    <div className="min-h-screen bg-white">
      {/* Hero Section */}
      <section className="bg-gradient-to-br from-blue-600 via-purple-600 to-indigo-800 py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center text-white">
            <h1 className="text-4xl md:text-5xl font-bold mb-6">
              {t('features.title')}
            </h1>
            <p className="text-xl md:text-2xl text-blue-100 max-w-3xl mx-auto">
              {t('features.subtitle')}
            </p>
          </div>
        </div>
      </section>

      {/* Feature Categories */}
      <section className="py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          {featureCategories.map((category, categoryIndex) => {
            const Icon = category.icon;
            return (
              <div key={categoryIndex} className="mb-20">
                <div className="text-center mb-12">
                  <div className={`w-16 h-16 bg-gradient-to-r ${category.color} rounded-full flex items-center justify-center mx-auto mb-4`}>
                    <Icon className="w-8 h-8 text-white" />
                  </div>
                  <h2 className="text-3xl font-bold text-gray-900 mb-4">{category.title}</h2>
                </div>

                <div className="grid lg:grid-cols-3 gap-8">
                  {category.features.map((feature, featureIndex) => {
                    const FeatureIcon = feature.icon;
                    return (
                      <div key={featureIndex} className="bg-white border border-gray-200 rounded-xl p-6 shadow-lg hover:shadow-xl transition-shadow">
                        <div className="flex items-center mb-4">
                          <div className={`w-10 h-10 bg-gradient-to-r ${category.color} rounded-lg flex items-center justify-center mr-3`}>
                            <FeatureIcon className="w-5 h-5 text-white" />
                          </div>
                          <h3 className="text-xl font-semibold text-gray-900">{feature.title}</h3>
                        </div>
                        <p className="text-gray-600 mb-4">{feature.description}</p>
                        <ul className="space-y-2">
                          {feature.details.map((detail, detailIndex) => (
                            <li key={detailIndex} className="text-sm text-gray-500 flex items-start">
                              <span className="text-green-500 mr-2">✓</span>
                              {detail}
                            </li>
                          ))}
                        </ul>
                      </div>
                    );
                  })}
                </div>
              </div>
            );
          })}
        </div>
      </section>

      {/* Performance Specifications */}
      <section className="py-20 bg-gray-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-12">
            <h2 className="text-3xl font-bold text-gray-900 mb-4">Performance Specifications</h2>
            <p className="text-xl text-gray-600">
              Enterprise-grade performance metrics and capabilities
            </p>
          </div>

          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
            {systemSpecs.map((spec, index) => (
              <div key={index} className="bg-white p-6 rounded-lg shadow-md">
                <div className="text-center">
                  <div className="text-3xl font-bold text-blue-600 mb-2">{spec.value}</div>
                  <div className="text-lg font-semibold text-gray-900 mb-1">{spec.label}</div>
                  <div className="text-sm text-gray-500">{spec.description}</div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Security Features */}
      <section className="py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-12">
            <h2 className="text-3xl font-bold text-gray-900 mb-4">Security & Privacy</h2>
            <p className="text-xl text-gray-600">
              Built-in security features with privacy-first design
            </p>
          </div>

          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
            {securityFeatures.map((feature, index) => {
              const Icon = feature.icon;
              return (
                <div key={index} className="text-center">
                  <div className="w-16 h-16 bg-gradient-to-r from-red-500 to-red-600 rounded-full flex items-center justify-center mx-auto mb-4">
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

      {/* Architecture Diagram */}
      <section className="py-20 bg-gray-900 text-white">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-12">
            <h2 className="text-3xl font-bold mb-4">System Architecture</h2>
            <p className="text-xl text-gray-300">
              Modular design with clear separation of concerns
            </p>
          </div>

          <div className="bg-gray-800 rounded-lg p-8 overflow-x-auto">
            <div className="min-w-4xl">
              <div className="grid grid-cols-4 gap-4 text-center">
                <div className="bg-blue-600 p-4 rounded-lg">
                  <div className="font-semibold mb-2">User Applications</div>
                  <div className="text-sm text-blue-100">CLI, GUI, Educational Tools</div>
                </div>
                <div className="bg-purple-600 p-4 rounded-lg">
                  <div className="font-semibold mb-2">System Services</div>
                  <div className="text-sm text-purple-100">IPC, File System, Network</div>
                </div>
                <div className="bg-green-600 p-4 rounded-lg">
                  <div className="font-semibold mb-2">Kernel Core</div>
                  <div className="text-sm text-green-100">Scheduler, Memory, IPC</div>
                </div>
                <div className="bg-orange-600 p-4 rounded-lg">
                  <div className="font-semibold mb-2">Hardware Layer</div>
                  <div className="text-sm text-orange-100">Drivers, HAL, Firmware</div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Testing & Quality */}
      <section className="py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid lg:grid-cols-2 gap-12 items-center">
            <div>
              <h2 className="text-3xl font-bold text-gray-900 mb-6">Quality Assurance</h2>
              <div className="space-y-6">
                <div className="flex items-start">
                  <BarChart3 className="w-6 h-6 text-green-600 mr-3 mt-1" />
                  <div>
                    <h3 className="font-semibold text-gray-900 mb-1">95%+ Test Coverage</h3>
                    <p className="text-gray-600">Comprehensive testing across all subsystems</p>
                  </div>
                </div>
                <div className="flex items-start">
                  <Shield className="w-6 h-6 text-green-600 mr-3 mt-1" />
                  <div>
                    <h3 className="font-semibold text-gray-900 mb-1">Security Audited</h3>
                    <p className="text-gray-600">Regular security reviews and vulnerability assessments</p>
                  </div>
                </div>
                <div className="flex items-start">
                  <Award className="w-6 h-6 text-green-600 mr-3 mt-1" />
                  <div>
                    <h3 className="font-semibold text-gray-900 mb-1">Enterprise Grade</h3>
                    <p className="text-gray-600">Production-ready code with comprehensive documentation</p>
                  </div>
                </div>
              </div>
            </div>
            <div className="bg-gray-50 p-8 rounded-lg">
              <h3 className="text-xl font-semibold text-gray-900 mb-4">Testing Infrastructure</h3>
              <ul className="space-y-3 text-gray-600">
                <li>• Unit testing for all components</li>
                <li>• Integration testing across subsystems</li>
                <li>• QEMU testing for multiple architectures</li>
                <li>• Performance benchmarking suite</li>
                <li>• Automated CI/CD pipeline</li>
                <li>• Cross-platform compatibility testing</li>
                <li>• Driver validation framework</li>
                <li>• Memory safety verification</li>
              </ul>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-20 bg-gradient-to-r from-blue-600 to-purple-600">
        <div className="max-w-4xl mx-auto text-center px-4 sm:px-6 lg:px-8">
          <h2 className="text-3xl font-bold text-white mb-6">
            Experience MultiOS Features Today
          </h2>
          <p className="text-xl text-blue-100 mb-8">
            Start exploring these features through our interactive demos and educational content
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <a
              href="/demos"
              className="inline-flex items-center px-8 py-4 bg-white text-blue-600 font-semibold rounded-lg hover:bg-blue-50 transition-colors"
            >
              Try Interactive Demos
            </a>
            <a
              href="/documentation"
              className="inline-flex items-center px-8 py-4 bg-transparent border-2 border-white text-white font-semibold rounded-lg hover:bg-white hover:text-blue-600 transition-colors"
            >
              View Documentation
            </a>
          </div>
        </div>
      </section>
    </div>
  );
};

export default FeaturesPage;