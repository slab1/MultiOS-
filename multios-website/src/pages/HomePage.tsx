import React from 'react';
import { Link } from 'react-router-dom';
import { useLanguage } from '../contexts/LanguageContext';
import { 
  ArrowRight, 
  Code, 
  Cpu, 
  Book, 
  Users, 
  Download, 
  Play,
  Shield,
  Zap,
  Globe,
  Award,
  GitBranch,
  Monitor
} from 'lucide-react';

const HomePage: React.FC = () => {
  const { t } = useLanguage();

  const stats = [
    { value: '50,000+', label: 'Lines of Code', icon: Code },
    { value: '3', label: 'Architectures Supported', icon: Cpu },
    { value: '95%', label: 'Test Coverage', icon: Award },
    { value: '4', label: 'Languages Supported', icon: Globe },
  ];

  const features = [
    {
      icon: Shield,
      title: t('features.kernel.title'),
      description: t('features.kernel.desc'),
      color: 'from-blue-500 to-blue-600',
    },
    {
      icon: GitBranch,
      title: t('features.cross.title'),
      description: t('features.cross.desc'),
      color: 'from-green-500 to-green-600',
    },
    {
      icon: Book,
      title: t('features.edu.title'),
      description: t('features.edu.desc'),
      color: 'from-purple-500 to-purple-600',
    },
    {
      icon: Users,
      title: t('features.community.title'),
      description: t('features.community.desc'),
      color: 'from-orange-500 to-orange-600',
    },
  ];

  const demoSections = [
    {
      title: 'Kernel Debugging',
      description: 'Interactive kernel debugging with step-through execution',
      image: '/api/placeholder/400/250',
      color: 'from-red-500 to-red-600',
    },
    {
      title: 'Process Management',
      description: 'Visualize process scheduling and management',
      image: '/api/placeholder/400/250',
      color: 'from-blue-500 to-blue-600',
    },
    {
      title: 'Memory Allocation',
      description: 'Dynamic memory allocation visualization',
      image: '/api/placeholder/400/250',
      color: 'from-green-500 to-green-600',
    },
    {
      title: 'File Systems',
      description: 'Explore file system operations in real-time',
      image: '/api/placeholder/400/250',
      color: 'from-purple-500 to-purple-600',
    },
  ];

  return (
    <div className="min-h-screen">
      {/* Hero Section */}
      <section className="relative py-20 lg:py-32 overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-br from-blue-600 via-purple-600 to-indigo-800"></div>
        <div className="absolute inset-0 bg-black/20"></div>
        
        <div className="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center text-white">
            <h1 className="text-4xl md:text-6xl font-bold mb-6 leading-tight">
              {t('hero.title')}
            </h1>
            <p className="text-xl md:text-2xl mb-8 text-blue-100 max-w-3xl mx-auto leading-relaxed">
              {t('hero.subtitle')}
            </p>
            
            <div className="flex flex-col sm:flex-row gap-4 justify-center mb-12">
              <Link
                to="/demos"
                className="inline-flex items-center px-8 py-4 bg-white text-blue-600 font-semibold rounded-lg hover:bg-blue-50 transition-all duration-200 shadow-lg hover:shadow-xl"
              >
                <Play className="w-5 h-5 mr-2" />
                {t('hero.cta')}
              </Link>
              <Link
                to="/download"
                className="inline-flex items-center px-8 py-4 bg-transparent border-2 border-white text-white font-semibold rounded-lg hover:bg-white hover:text-blue-600 transition-all duration-200"
              >
                <Download className="w-5 h-5 mr-2" />
                {t('hero.download')}
              </Link>
            </div>

            {/* Stats */}
            <div className="grid grid-cols-2 md:grid-cols-4 gap-8 max-w-4xl mx-auto">
              {stats.map((stat, index) => {
                const Icon = stat.icon;
                return (
                  <div key={index} className="text-center">
                    <Icon className="w-8 h-8 mx-auto mb-2 text-blue-200" />
                    <div className="text-2xl md:text-3xl font-bold text-white mb-1">
                      {stat.value}
                    </div>
                    <div className="text-blue-200 text-sm md:text-base">
                      {stat.label}
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        </div>

        {/* Animated background elements */}
        <div className="absolute top-20 left-10 w-20 h-20 bg-blue-400/20 rounded-full animate-pulse"></div>
        <div className="absolute bottom-20 right-10 w-32 h-32 bg-purple-400/20 rounded-full animate-pulse delay-1000"></div>
        <div className="absolute top-40 right-1/4 w-16 h-16 bg-indigo-400/20 rounded-full animate-pulse delay-500"></div>
      </section>

      {/* Features Section */}
      <section className="py-20 bg-white">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-3xl md:text-4xl font-bold text-gray-900 mb-4">
              {t('features.title')}
            </h2>
            <p className="text-xl text-gray-600 max-w-3xl mx-auto">
              {t('features.subtitle')}
            </p>
          </div>

          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-8">
            {features.map((feature, index) => {
              const Icon = feature.icon;
              return (
                <div
                  key={index}
                  className="p-6 bg-white rounded-xl shadow-lg hover:shadow-xl transition-all duration-300 border border-gray-100 hover:border-blue-200 group"
                >
                  <div className={`w-12 h-12 bg-gradient-to-r ${feature.color} rounded-lg flex items-center justify-center mb-4 group-hover:scale-110 transition-transform duration-300`}>
                    <Icon className="w-6 h-6 text-white" />
                  </div>
                  <h3 className="text-xl font-semibold text-gray-900 mb-2">
                    {feature.title}
                  </h3>
                  <p className="text-gray-600">
                    {feature.description}
                  </p>
                </div>
              );
            })}
          </div>
        </div>
      </section>

      {/* Demo Preview Section */}
      <section className="py-20 bg-gray-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-3xl md:text-4xl font-bold text-gray-900 mb-4">
              Interactive Learning Demos
            </h2>
            <p className="text-xl text-gray-600 max-w-3xl mx-auto">
              Explore operating systems concepts through hands-on interactive demonstrations
            </p>
          </div>

          <div className="grid md:grid-cols-2 gap-8">
            {demoSections.map((demo, index) => (
              <div
                key={index}
                className="bg-white rounded-xl shadow-lg overflow-hidden hover:shadow-xl transition-all duration-300 group"
              >
                <div className={`h-48 bg-gradient-to-r ${demo.color} relative overflow-hidden`}>
                  <div className="absolute inset-0 flex items-center justify-center">
                    <Monitor className="w-16 h-16 text-white opacity-80" />
                  </div>
                  <div className="absolute inset-0 bg-black/20 group-hover:bg-black/10 transition-colors duration-300"></div>
                </div>
                <div className="p-6">
                  <h3 className="text-xl font-semibold text-gray-900 mb-2">
                    {demo.title}
                  </h3>
                  <p className="text-gray-600 mb-4">
                    {demo.description}
                  </p>
                  <Link
                    to="/demos"
                    className="inline-flex items-center text-blue-600 font-medium hover:text-blue-700 transition-colors"
                  >
                    Try Demo
                    <ArrowRight className="w-4 h-4 ml-1" />
                  </Link>
                </div>
              </div>
            ))}
          </div>

          <div className="text-center mt-12">
            <Link
              to="/demos"
              className="inline-flex items-center px-8 py-4 bg-gradient-to-r from-blue-600 to-purple-600 text-white font-semibold rounded-lg hover:from-blue-700 hover:to-purple-700 transition-all duration-200 shadow-lg hover:shadow-xl"
            >
              View All Demos
              <ArrowRight className="w-5 h-5 ml-2" />
            </Link>
          </div>
        </div>
      </section>

      {/* Architecture Section */}
      <section className="py-20 bg-white">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-3xl md:text-4xl font-bold text-gray-900 mb-4">
              Cross-Platform Architecture
            </h2>
            <p className="text-xl text-gray-600 max-w-3xl mx-auto">
              MultiOS runs natively on multiple CPU architectures with zero-compromise performance
            </p>
          </div>

          <div className="bg-gray-50 rounded-2xl p-8 md:p-12">
            <div className="grid md:grid-cols-3 gap-8">
              <div className="text-center">
                <div className="w-16 h-16 bg-blue-100 rounded-full flex items-center justify-center mx-auto mb-4">
                  <Cpu className="w-8 h-8 text-blue-600" />
                </div>
                <h3 className="text-xl font-semibold text-gray-900 mb-2">x86_64</h3>
                <p className="text-gray-600 mb-4">
                  Full support with SSE, AVX, AES-NI features
                </p>
                <div className="flex items-center justify-center space-x-4 text-sm text-gray-500">
                  <span>Intel</span>
                  <span>AMD</span>
                  <span>VirtualBox</span>
                </div>
              </div>

              <div className="text-center">
                <div className="w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-4">
                  <Cpu className="w-8 h-8 text-green-600" />
                </div>
                <h3 className="text-xl font-semibold text-gray-900 mb-2">ARM64</h3>
                <p className="text-gray-600 mb-4">
                  Complete ARMv8-A support with NEON and crypto extensions
                </p>
                <div className="flex items-center justify-center space-x-4 text-sm text-gray-500">
                  <span>Apple M</span>
                  <span>ARMv8-A</span>
                  <span>Raspberry Pi</span>
                </div>
              </div>

              <div className="text-center">
                <div className="w-16 h-16 bg-purple-100 rounded-full flex items-center justify-center mx-auto mb-4">
                  <Cpu className="w-8 h-8 text-purple-600" />
                </div>
                <h3 className="text-xl font-semibold text-gray-900 mb-2">RISC-V</h3>
                <p className="text-gray-600 mb-4">
                  RV64GC implementation with standard extensions
                </p>
                <div className="flex items-center justify-center space-x-4 text-sm text-gray-500">
                  <span>SiFive</span>
                  <span>RV64GC</span>
                  <span>QEMU</span>
                </div>
              </div>
            </div>

            <div className="text-center mt-12">
              <Link
                to="/features"
                className="inline-flex items-center px-6 py-3 border border-gray-300 text-gray-700 font-medium rounded-lg hover:bg-gray-50 transition-colors"
              >
                Learn More About Architecture
                <ArrowRight className="w-4 h-4 ml-2" />
              </Link>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-20 bg-gradient-to-r from-blue-600 to-purple-600">
        <div className="max-w-4xl mx-auto text-center px-4 sm:px-6 lg:px-8">
          <h2 className="text-3xl md:text-4xl font-bold text-white mb-6">
            Ready to Start Learning?
          </h2>
          <p className="text-xl text-blue-100 mb-8">
            Join thousands of students and developers mastering operating systems development with MultiOS
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Link
              to="/demos"
              className="inline-flex items-center px-8 py-4 bg-white text-blue-600 font-semibold rounded-lg hover:bg-blue-50 transition-all duration-200 shadow-lg"
            >
              <Play className="w-5 h-5 mr-2" />
              Try Interactive Demos
            </Link>
            <Link
              to="/education"
              className="inline-flex items-center px-8 py-4 bg-transparent border-2 border-white text-white font-semibold rounded-lg hover:bg-white hover:text-blue-600 transition-all duration-200"
            >
              <Book className="w-5 h-5 mr-2" />
              Explore Curriculum
            </Link>
          </div>
        </div>
      </section>
    </div>
  );
};

export default HomePage;