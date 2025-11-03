import React from 'react';
import { Book, Code, Settings, Download, Search, ExternalLink } from 'lucide-react';

const DocumentationPage: React.FC = () => {
  const docSections = [
    {
      title: 'Getting Started',
      icon: Book,
      description: 'Quick start guide and installation instructions',
      links: [
        'Installation Guide',
        'First Steps',
        'Development Setup',
        'System Requirements'
      ]
    },
    {
      title: 'API Reference',
      icon: Code,
      description: 'Complete API documentation and examples',
      links: [
        'Kernel API',
        'Device Drivers',
        'File System API',
        'Memory Management'
      ]
    },
    {
      title: 'Architecture',
      icon: Settings,
      description: 'System architecture and design patterns',
      links: [
        'Kernel Design',
        'Multi-Architecture',
        'Driver Framework',
        'Memory Layout'
      ]
    },
    {
      title: 'Tutorials',
      icon: Download,
      description: 'Step-by-step learning tutorials',
      links: [
        'OS Development Basics',
        'Creating Device Drivers',
        'Memory Management',
        'Process Scheduling'
      ]
    }
  ];

  return (
    <div className="min-h-screen bg-white">
      {/* Hero Section */}
      <section className="bg-gradient-to-br from-blue-600 via-purple-600 to-indigo-800 py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center text-white">
          <h1 className="text-4xl md:text-5xl font-bold mb-6">Documentation</h1>
          <p className="text-xl md:text-2xl text-blue-100 max-w-3xl mx-auto mb-8">
            Comprehensive guides, tutorials, and API references for MultiOS development
          </p>
          
          {/* Search Bar */}
          <div className="max-w-2xl mx-auto">
            <div className="relative">
              <Search className="absolute left-4 top-1/2 transform -translate-y-1/2 text-gray-400 w-5 h-5" />
              <input
                type="text"
                placeholder="Search documentation..."
                className="w-full pl-12 pr-4 py-4 rounded-lg text-gray-900 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-white"
              />
            </div>
          </div>
        </div>
      </section>

      {/* Documentation Sections */}
      <section className="py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-8">
            {docSections.map((section, index) => {
              const Icon = section.icon;
              return (
                <div key={index} className="bg-white border border-gray-200 rounded-xl p-6 hover:shadow-lg transition-shadow">
                  <Icon className="w-12 h-12 text-blue-600 mb-4" />
                  <h3 className="text-xl font-semibold text-gray-900 mb-2">{section.title}</h3>
                  <p className="text-gray-600 mb-4">{section.description}</p>
                  <ul className="space-y-2">
                    {section.links.map((link, linkIndex) => (
                      <li key={linkIndex}>
                        <a href="#" className="text-blue-600 hover:text-blue-700 text-sm">
                          {link}
                        </a>
                      </li>
                    ))}
                  </ul>
                </div>
              );
            })}
          </div>
        </div>
      </section>

      {/* Quick Links */}
      <section className="py-12 bg-gray-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <h2 className="text-2xl font-bold text-gray-900 mb-8 text-center">Quick Links</h2>
          <div className="grid md:grid-cols-3 gap-6">
            <a href="#" className="bg-white p-6 rounded-lg shadow-md hover:shadow-lg transition-shadow flex items-center">
              <ExternalLink className="w-6 h-6 text-blue-600 mr-3" />
              <div>
                <h3 className="font-semibold text-gray-900">Interactive API Docs</h3>
                <p className="text-sm text-gray-600">Live documentation with examples</p>
              </div>
            </a>
            <a href="#" className="bg-white p-6 rounded-lg shadow-md hover:shadow-lg transition-shadow flex items-center">
              <ExternalLink className="w-6 h-6 text-green-600 mr-3" />
              <div>
                <h3 className="font-semibold text-gray-900">Video Tutorials</h3>
                <p className="text-sm text-gray-600">Video guides and walkthroughs</p>
              </div>
            </a>
            <a href="#" className="bg-white p-6 rounded-lg shadow-md hover:shadow-lg transition-shadow flex items-center">
              <ExternalLink className="w-6 h-6 text-purple-600 mr-3" />
              <div>
                <h3 className="font-semibold text-gray-900">Code Examples</h3>
                <p className="text-sm text-gray-600">Practical code samples</p>
              </div>
            </a>
          </div>
        </div>
      </section>
    </div>
  );
};

export default DocumentationPage;