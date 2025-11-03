import React, { useState } from 'react';
import { useLanguage } from '../contexts/LanguageContext';
import { 
  Download, 
  Monitor, 
  Cpu, 
  HardDrive, 
  CheckCircle, 
  AlertCircle,
  Github,
  ExternalLink,
  Zap,
  Shield,
  Server,
  Laptop,
  Smartphone,
  QrCode
} from 'lucide-react';

const DownloadPage: React.FC = () => {
  const { t } = useLanguage();
  const [selectedArch, setSelectedArch] = useState('x86_64');
  const [selectedType, setSelectedType] = useState('desktop');

  const architectures = [
    {
      id: 'x86_64',
      name: 'x86_64',
      description: 'Intel and AMD processors',
      icon: Monitor,
      color: 'from-blue-500 to-blue-600',
      systems: ['Intel', 'AMD', 'VirtualBox', 'VMware']
    },
    {
      id: 'aarch64',
      name: 'ARM64',
      description: 'ARM v8-A architecture',
      icon: Smartphone,
      color: 'from-green-500 to-green-600',
      systems: ['Apple Silicon', 'Raspberry Pi 4', 'ARM Servers']
    },
    {
      id: 'riscv64',
      name: 'RISC-V',
      description: 'RV64GC implementation',
      icon: Cpu,
      color: 'from-purple-500 to-purple-600',
      systems: ['SiFive', 'QEMU', 'Academic Platforms']
    }
  ];

  const installationTypes = [
    {
      id: 'desktop',
      name: 'Desktop/Laptop',
      description: 'Full desktop environment with GUI',
      icon: Laptop,
      minSpecs: '2GB RAM, 4GB storage',
      features: ['GUI Desktop', 'Development Tools', 'Educational Apps', 'Full IDE Support']
    },
    {
      id: 'server',
      name: 'Server Edition',
      description: 'Headless server environment',
      icon: Server,
      minSpecs: '1GB RAM, 2GB storage',
      features: ['CLI Interface', 'Development Server', 'Container Support', 'Network Services']
    },
    {
      id: 'minimal',
      name: 'Minimal',
      description: 'Core system for embedded/learning',
      icon: Shield,
      minSpecs: '512MB RAM, 1GB storage',
      features: ['Core OS', 'CLI Only', 'Educational Modules', 'Cross-compilation']
    }
  ];

  const downloadOptions = [
    {
      title: 'ISO Image',
      description: 'Bootable installation media',
      icon: Download,
      size: 'Varies by architecture',
      recommended: true
    },
    {
      title: 'Virtual Machine',
      description: 'Pre-configured VM images',
      icon: Monitor,
      size: '1-4 GB',
      recommended: false
    },
    {
      title: 'Docker Container',
      description: 'Containerized development environment',
      icon: Shield,
      size: '200-500 MB',
      recommended: true
    },
    {
      title: 'Source Code',
      description: 'Build from source',
      icon: Github,
      size: '50-100 MB',
      recommended: false
    }
  ];

  const systemRequirements = {
    minimum: {
      cpu: 'Any supported architecture',
      ram: '512 MB',
      storage: '1 GB free space',
      display: 'VGA or UEFI GOP',
      network: 'Optional'
    },
    recommended: {
      cpu: 'Multi-core (2+ cores)',
      ram: '4 GB',
      storage: '8 GB free space',
      display: '1920x1080 or higher',
      network: 'Ethernet or WiFi'
    },
    optimal: {
      cpu: 'Modern multi-core processor',
      ram: '8 GB+',
      storage: '16 GB+ SSD',
      display: '4K display support',
      network: 'Gigabit Ethernet'
    }
  };

  const quickStart = [
    {
      step: 1,
      title: 'Download MultiOS',
      description: 'Choose your architecture and installation type',
      action: 'Select and download the appropriate image'
    },
    {
      step: 2,
      title: 'Create Bootable Media',
      description: 'Burn ISO to USB drive or create VM',
      action: 'Use tools like Rufus, Balena Etcher, or VirtualBox'
    },
    {
      step: 3,
      title: 'Boot and Install',
      description: 'Boot from media and follow installer',
      action: 'Installation wizard guides you through setup'
    },
    {
      step: 4,
      title: 'Start Learning',
      description: 'Explore educational content and demos',
      action: 'Run interactive demos and tutorials'
    }
  ];

  const downloadUrl = (arch: string, type: string, format: string = 'iso') => {
    const baseUrl = 'https://releases.multios.org';
    const version = 'v1.0.0';
    return `${baseUrl}/${version}/${arch}-${type}-${format}.iso`;
  };

  return (
    <div className="min-h-screen bg-white">
      {/* Hero Section */}
      <section className="bg-gradient-to-br from-blue-600 via-purple-600 to-indigo-800 py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center text-white">
            <h1 className="text-4xl md:text-5xl font-bold mb-6">
              Download MultiOS
            </h1>
            <p className="text-xl md:text-2xl text-blue-100 max-w-3xl mx-auto mb-8">
              Get started with MultiOS - Choose your architecture, installation type, and format
            </p>
            <div className="flex items-center justify-center space-x-8 text-blue-100">
              <div className="flex items-center">
                <CheckCircle className="w-5 h-5 mr-2" />
                Free & Open Source
              </div>
              <div className="flex items-center">
                <CheckCircle className="w-5 h-5 mr-2" />
                No Registration Required
              </div>
              <div className="flex items-center">
                <CheckCircle className="w-5 h-5 mr-2" />
                Regular Updates
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Architecture Selection */}
      <section className="py-12 bg-gray-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <h2 className="text-2xl font-bold text-gray-900 mb-8 text-center">Choose Your Architecture</h2>
          <div className="grid md:grid-cols-3 gap-6">
            {architectures.map((arch) => {
              const Icon = arch.icon;
              return (
                <button
                  key={arch.id}
                  onClick={() => setSelectedArch(arch.id)}
                  className={`p-6 rounded-xl border-2 transition-all duration-200 text-left ${
                    selectedArch === arch.id
                      ? 'border-blue-500 bg-blue-50'
                      : 'border-gray-200 hover:border-gray-300 bg-white'
                  }`}
                >
                  <div className="flex items-center mb-4">
                    <div className={`w-12 h-12 bg-gradient-to-r ${arch.color} rounded-lg flex items-center justify-center mr-4`}>
                      <Icon className="w-6 h-6 text-white" />
                    </div>
                    <div>
                      <h3 className="text-lg font-semibold text-gray-900">{arch.name}</h3>
                      <p className="text-sm text-gray-600">{arch.description}</p>
                    </div>
                  </div>
                  <div className="space-y-1">
                    {arch.systems.map((system, index) => (
                      <div key={index} className="text-sm text-gray-500 flex items-center">
                        <CheckCircle className="w-3 h-3 text-green-500 mr-2" />
                        {system}
                      </div>
                    ))}
                  </div>
                </button>
              );
            })}
          </div>
        </div>
      </section>

      {/* Installation Type */}
      <section className="py-12">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <h2 className="text-2xl font-bold text-gray-900 mb-8 text-center">Choose Installation Type</h2>
          <div className="grid md:grid-cols-3 gap-6">
            {installationTypes.map((type) => {
              const Icon = type.icon;
              return (
                <button
                  key={type.id}
                  onClick={() => setSelectedType(type.id)}
                  className={`p-6 rounded-xl border-2 transition-all duration-200 text-left ${
                    selectedType === type.id
                      ? 'border-green-500 bg-green-50'
                      : 'border-gray-200 hover:border-gray-300 bg-white'
                  }`}
                >
                  <div className="flex items-center mb-4">
                    <Icon className="w-6 h-6 text-gray-700 mr-3" />
                    <h3 className="text-lg font-semibold text-gray-900">{type.name}</h3>
                  </div>
                  <p className="text-gray-600 mb-4">{type.description}</p>
                  <div className="text-sm text-gray-500 mb-4">
                    <strong>Minimum specs:</strong> {type.minSpecs}
                  </div>
                  <div className="space-y-1">
                    {type.features.map((feature, index) => (
                      <div key={index} className="text-sm text-gray-600 flex items-center">
                        <CheckCircle className="w-3 h-3 text-green-500 mr-2" />
                        {feature}
                      </div>
                    ))}
                  </div>
                </button>
              );
            })}
          </div>
        </div>
      </section>

      {/* Download Options */}
      <section className="py-12 bg-gray-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <h2 className="text-2xl font-bold text-gray-900 mb-8 text-center">Download Options</h2>
          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
            {downloadOptions.map((option, index) => {
              const Icon = option.icon;
              return (
                <div key={index} className="bg-white p-6 rounded-xl shadow-md hover:shadow-lg transition-shadow">
                  <div className="flex items-center mb-4">
                    <Icon className="w-8 h-8 text-blue-600 mr-3" />
                    <div>
                      <h3 className="font-semibold text-gray-900">{option.title}</h3>
                      {option.recommended && (
                        <span className="inline-block bg-green-100 text-green-800 text-xs px-2 py-1 rounded">
                          Recommended
                        </span>
                      )}
                    </div>
                  </div>
                  <p className="text-gray-600 text-sm mb-2">{option.description}</p>
                  <p className="text-gray-500 text-xs mb-4">Size: {option.size}</p>
                  <button 
                    onClick={() => window.open(downloadUrl(selectedArch, selectedType), '_blank')}
                    className="w-full bg-blue-600 text-white py-2 px-4 rounded hover:bg-blue-700 transition-colors"
                  >
                    Download {option.title}
                  </button>
                </div>
              );
            })}
          </div>
        </div>
      </section>

      {/* System Requirements */}
      <section className="py-12">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <h2 className="text-2xl font-bold text-gray-900 mb-8 text-center">System Requirements</h2>
          <div className="grid lg:grid-cols-3 gap-8">
            <div className="bg-red-50 border border-red-200 rounded-xl p-6">
              <div className="flex items-center mb-4">
                <AlertCircle className="w-6 h-6 text-red-600 mr-2" />
                <h3 className="text-lg font-semibold text-red-900">Minimum</h3>
              </div>
              <div className="space-y-2 text-sm">
                <div><strong>CPU:</strong> {systemRequirements.minimum.cpu}</div>
                <div><strong>RAM:</strong> {systemRequirements.minimum.ram}</div>
                <div><strong>Storage:</strong> {systemRequirements.minimum.storage}</div>
                <div><strong>Display:</strong> {systemRequirements.minimum.display}</div>
                <div><strong>Network:</strong> {systemRequirements.minimum.network}</div>
              </div>
            </div>

            <div className="bg-yellow-50 border border-yellow-200 rounded-xl p-6">
              <div className="flex items-center mb-4">
                <Zap className="w-6 h-6 text-yellow-600 mr-2" />
                <h3 className="text-lg font-semibold text-yellow-900">Recommended</h3>
              </div>
              <div className="space-y-2 text-sm">
                <div><strong>CPU:</strong> {systemRequirements.recommended.cpu}</div>
                <div><strong>RAM:</strong> {systemRequirements.recommended.ram}</div>
                <div><strong>Storage:</strong> {systemRequirements.recommended.storage}</div>
                <div><strong>Display:</strong> {systemRequirements.recommended.display}</div>
                <div><strong>Network:</strong> {systemRequirements.recommended.network}</div>
              </div>
            </div>

            <div className="bg-green-50 border border-green-200 rounded-xl p-6">
              <div className="flex items-center mb-4">
                <CheckCircle className="w-6 h-6 text-green-600 mr-2" />
                <h3 className="text-lg font-semibold text-green-900">Optimal</h3>
              </div>
              <div className="space-y-2 text-sm">
                <div><strong>CPU:</strong> {systemRequirements.optimal.cpu}</div>
                <div><strong>RAM:</strong> {systemRequirements.optimal.ram}</div>
                <div><strong>Storage:</strong> {systemRequirements.optimal.storage}</div>
                <div><strong>Display:</strong> {systemRequirements.optimal.display}</div>
                <div><strong>Network:</strong> {systemRequirements.optimal.network}</div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Quick Start Guide */}
      <section className="py-12 bg-gray-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <h2 className="text-2xl font-bold text-gray-900 mb-8 text-center">Quick Start Guide</h2>
          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
            {quickStart.map((step, index) => (
              <div key={index} className="text-center">
                <div className="w-12 h-12 bg-blue-600 text-white rounded-full flex items-center justify-center mx-auto mb-4 text-lg font-bold">
                  {step.step}
                </div>
                <h3 className="font-semibold text-gray-900 mb-2">{step.title}</h3>
                <p className="text-gray-600 text-sm mb-2">{step.description}</p>
                <p className="text-gray-500 text-xs">{step.action}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Download Buttons */}
      <section className="py-12">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <h2 className="text-2xl font-bold text-gray-900 mb-8">Ready to Get Started?</h2>
          <div className="grid md:grid-cols-2 gap-6">
            <a
              href={downloadUrl(selectedArch, selectedType)}
              className="inline-flex items-center justify-center px-8 py-4 bg-gradient-to-r from-blue-600 to-purple-600 text-white font-semibold rounded-lg hover:from-blue-700 hover:to-purple-700 transition-all duration-200 shadow-lg hover:shadow-xl"
            >
              <Download className="w-5 h-5 mr-2" />
              Download MultiOS {selectedArch} {selectedType}
            </a>
            <a
              href="https://github.com/multios-org"
              className="inline-flex items-center justify-center px-8 py-4 bg-gray-800 text-white font-semibold rounded-lg hover:bg-gray-900 transition-colors"
            >
              <Github className="w-5 h-5 mr-2" />
              View Source Code
              <ExternalLink className="w-4 h-4 ml-2" />
            </a>
          </div>
          
          <div className="mt-8 p-4 bg-blue-50 rounded-lg">
            <h3 className="font-semibold text-blue-900 mb-2">Alternative Downloads</h3>
            <div className="flex flex-wrap justify-center gap-4 text-sm">
              <a href="#docker" className="text-blue-700 hover:text-blue-800">
                Docker Hub
              </a>
              <a href="#vmware" className="text-blue-700 hover:text-blue-800">
                VMware Images
              </a>
              <a href="#virtualbox" className="text-blue-700 hover:text-blue-800">
                VirtualBox Images
              </a>
              <a href="#qemu" className="text-blue-700 hover:text-blue-800">
                QEMU Images
              </a>
            </div>
          </div>
        </div>
      </section>

      {/* QR Code Section */}
      <section className="py-12 bg-gray-900 text-white">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <h2 className="text-2xl font-bold mb-8">Mobile Downloads</h2>
          <div className="grid md:grid-cols-2 gap-8 items-center">
            <div>
              <h3 className="text-xl font-semibold mb-4">Scan QR Code</h3>
              <p className="text-gray-300 mb-4">
                Quick download link for mobile and tablet devices
              </p>
              <div className="w-32 h-32 bg-white mx-auto rounded-lg flex items-center justify-center">
                <QrCode className="w-24 h-24 text-gray-800" />
              </div>
            </div>
            <div>
              <h3 className="text-xl font-semibold mb-4">Development Tools</h3>
              <div className="space-y-3 text-left">
                <div className="flex items-center">
                  <CheckCircle className="w-5 h-5 text-green-400 mr-3" />
                  <span>Cross-compilation support</span>
                </div>
                <div className="flex items-center">
                  <CheckCircle className="w-5 h-5 text-green-400 mr-3" />
                  <span>Docker development environment</span>
                </div>
                <div className="flex items-center">
                  <CheckCircle className="w-5 h-5 text-green-400 mr-3" />
                  <span>VS Code extension available</span>
                </div>
                <div className="flex items-center">
                  <CheckCircle className="w-5 h-5 text-green-400 mr-3" />
                  <span>GitHub Actions integration</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
};

export default DownloadPage;