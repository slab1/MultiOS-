import React from 'react';
import { Link } from 'react-router-dom';
import { 
  Code, 
  BookOpen, 
  Users, 
  FolderOpen, 
  Play, 
  ArrowRight, 
  Zap, 
  Shield, 
  Globe,
  Star,
  Download,
  Terminal
} from 'lucide-react';

export const Home: React.FC = () => {
  const features = [
    {
      icon: Code,
      title: 'Advanced Code Editor',
      description: 'Monaco-powered editor with syntax highlighting for Rust, Python, JavaScript, and more.',
      color: 'from-blue-500 to-cyan-500'
    },
    {
      icon: Zap,
      title: 'Real-time Execution',
      description: 'Execute and test code in real-time with instant feedback and error reporting.',
      color: 'from-purple-500 to-pink-500'
    },
    {
      icon: BookOpen,
      title: 'Interactive Tutorials',
      description: 'Step-by-step tutorials with embedded coding exercises and progressive learning.',
      color: 'from-green-500 to-emerald-500'
    },
    {
      icon: Users,
      title: 'Community Driven',
      description: 'Share projects, collaborate with peers, and learn from the developer community.',
      color: 'from-orange-500 to-red-500'
    },
    {
      icon: FolderOpen,
      title: 'Project Templates',
      description: 'Ready-to-use templates and starter kits for various MultiOS projects.',
      color: 'from-indigo-500 to-purple-500'
    },
    {
      icon: Shield,
      title: 'Secure Environment',
      description: 'Safe execution environment with proper sandboxing and security measures.',
      color: 'from-teal-500 to-cyan-500'
    }
  ];

  const languages = [
    { name: 'Rust', percentage: 95, color: 'bg-orange-500' },
    { name: 'Python', percentage: 90, color: 'bg-blue-500' },
    { name: 'JavaScript', percentage: 88, color: 'bg-yellow-500' },
    { name: 'TypeScript', percentage: 85, color: 'bg-blue-600' },
    { name: 'C++', percentage: 80, color: 'bg-purple-500' },
    { name: 'Go', percentage: 75, color: 'bg-cyan-500' }
  ];

  const stats = [
    { label: 'Active Developers', value: '10,000+' },
    { label: 'Code Templates', value: '500+' },
    { label: 'Tutorials', value: '1,200+' },
    { label: 'Community Projects', value: '2,500+' }
  ];

  return (
    <div className="min-h-screen">
      {/* Hero Section */}
      <section className="relative py-20 px-4 sm:px-6 lg:px-8 overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-br from-blue-600/10 to-purple-600/10"></div>
        <div className="relative max-w-7xl mx-auto">
          <div className="text-center">
            <div className="flex justify-center mb-8">
              <div className="bg-gradient-to-r from-blue-600 to-purple-600 p-4 rounded-2xl">
                <Play className="h-12 w-12 text-white" />
              </div>
            </div>
            <h1 className="text-5xl md:text-7xl font-bold text-slate-900 mb-6">
              <span className="bg-gradient-to-r from-blue-600 to-purple-600 bg-clip-text text-transparent">
                MultiOS
              </span>
              <br />
              <span className="text-slate-800">Developer Portal</span>
            </h1>
            <p className="text-xl md:text-2xl text-slate-600 mb-12 max-w-3xl mx-auto leading-relaxed">
              A comprehensive platform for learning, coding, and collaborating on MultiOS projects. 
              Write, test, and deploy code with real-time execution and community features.
            </p>
            
            <div className="flex flex-col sm:flex-row gap-4 justify-center">
              <Link
                to="/editor"
                className="group bg-gradient-to-r from-blue-600 to-purple-600 text-white px-8 py-4 rounded-xl font-semibold text-lg hover:shadow-xl transition-all duration-300 transform hover:scale-105"
              >
                <div className="flex items-center space-x-2">
                  <Terminal className="h-5 w-5" />
                  <span>Start Coding</span>
                  <ArrowRight className="h-5 w-5 group-hover:translate-x-1 transition-transform" />
                </div>
              </Link>
              <Link
                to="/tutorials"
                className="bg-white text-slate-700 px-8 py-4 rounded-xl font-semibold text-lg border-2 border-slate-200 hover:border-blue-300 hover:shadow-lg transition-all duration-300"
              >
                <div className="flex items-center space-x-2">
                  <BookOpen className="h-5 w-5" />
                  <span>Learn Tutorials</span>
                </div>
              </Link>
            </div>
          </div>
        </div>
      </section>

      {/* Stats Section */}
      <section className="py-16 bg-white">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-8">
            {stats.map((stat, index) => (
              <div key={index} className="text-center">
                <div className="text-3xl md:text-4xl font-bold text-slate-900 mb-2">
                  {stat.value}
                </div>
                <div className="text-slate-600 font-medium">{stat.label}</div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="py-20 px-4 sm:px-6 lg:px-8">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-16">
            <h2 className="text-4xl md:text-5xl font-bold text-slate-900 mb-6">
              Everything you need to build amazing
              <span className="bg-gradient-to-r from-blue-600 to-purple-600 bg-clip-text text-transparent">
                {' '}MultiOS applications
              </span>
            </h2>
            <p className="text-xl text-slate-600 max-w-3xl mx-auto">
              Our platform provides all the tools, resources, and community support you need 
              to create, learn, and share MultiOS applications.
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
            {features.map((feature, index) => {
              const Icon = feature.icon;
              return (
                <div
                  key={index}
                  className="group bg-white rounded-2xl p-8 shadow-lg hover:shadow-xl transition-all duration-300 transform hover:scale-105 border border-slate-100"
                >
                  <div className={`bg-gradient-to-r ${feature.color} p-3 rounded-xl w-fit mb-6 group-hover:scale-110 transition-transform duration-300`}>
                    <Icon className="h-8 w-8 text-white" />
                  </div>
                  <h3 className="text-xl font-bold text-slate-900 mb-4">
                    {feature.title}
                  </h3>
                  <p className="text-slate-600 leading-relaxed">
                    {feature.description}
                  </p>
                </div>
              );
            })}
          </div>
        </div>
      </section>

      {/* Language Support Section */}
      <section className="py-20 bg-slate-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-4xl md:text-5xl font-bold text-slate-900 mb-6">
              Multi-language
              <span className="bg-gradient-to-r from-blue-600 to-purple-600 bg-clip-text text-transparent">
                {' '}Support
              </span>
            </h2>
            <p className="text-xl text-slate-600 max-w-3xl mx-auto">
              Write and execute code in your favorite programming languages with full syntax highlighting and IntelliSense.
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-8 max-w-4xl mx-auto">
            {languages.map((language, index) => (
              <div key={index} className="bg-white rounded-xl p-6 shadow-md">
                <div className="flex items-center justify-between mb-3">
                  <span className="text-lg font-semibold text-slate-800">{language.name}</span>
                  <span className="text-sm font-medium text-slate-600">{language.percentage}%</span>
                </div>
                <div className="w-full bg-slate-200 rounded-full h-3">
                  <div
                    className={`${language.color} h-3 rounded-full transition-all duration-1000 ease-out`}
                    style={{ width: `${language.percentage}%` }}
                  ></div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-20 px-4 sm:px-6 lg:px-8 bg-gradient-to-r from-blue-600 to-purple-600">
        <div className="max-w-4xl mx-auto text-center">
          <h2 className="text-4xl md:text-5xl font-bold text-white mb-6">
            Ready to start building?
          </h2>
          <p className="text-xl text-blue-100 mb-12 max-w-2xl mx-auto">
            Join thousands of developers who are already building amazing MultiOS applications. 
            Start coding now and see your ideas come to life.
          </p>
          
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Link
              to="/editor"
              className="group bg-white text-blue-600 px-8 py-4 rounded-xl font-semibold text-lg hover:shadow-xl transition-all duration-300 transform hover:scale-105"
            >
              <div className="flex items-center space-x-2">
                <Code className="h-5 w-5" />
                <span>Open Code Editor</span>
                <ArrowRight className="h-5 w-5 group-hover:translate-x-1 transition-transform" />
              </div>
            </Link>
            <Link
              to="/templates"
              className="bg-transparent text-white px-8 py-4 rounded-xl font-semibold text-lg border-2 border-white hover:bg-white hover:text-blue-600 transition-all duration-300"
            >
              <div className="flex items-center space-x-2">
                <FolderOpen className="h-5 w-5" />
                <span>Browse Templates</span>
              </div>
            </Link>
          </div>
        </div>
      </section>
    </div>
  );
};