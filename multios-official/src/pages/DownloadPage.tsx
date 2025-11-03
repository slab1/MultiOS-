import React, { useState } from 'react';

const DownloadPage = () => {
  const [selectedVersion, setSelectedVersion] = useState('1.0.0');

  const versions = [
    { 
      version: '1.0.0', 
      codename: 'Release Candidate',
      status: 'Stable',
      date: '2025-11-03',
      description: 'Production-ready release with complete feature set'
    },
    { 
      version: '1.1.0', 
      codename: 'Development',
      status: 'Beta',
      date: '2025-12-15',
      description: 'Next release with enhanced features (coming soon)'
    }
  ];

  const architectures = [
    {
      name: 'x86_64',
      description: 'Intel and AMD 64-bit processors',
      requirements: 'Intel VT-x or AMD-V, 4GB RAM, 2GB storage',
      size: '512 MB',
      filename: 'multios-x86_64-1.0.0.iso'
    },
    {
      name: 'ARM64',
      description: 'ARMv8-A 64-bit processors',
      requirements: 'ARMv8-A, 4GB RAM, 2GB storage',
      size: '480 MB',
      filename: 'multios-arm64-1.0.0.iso'
    },
    {
      name: 'RISC-V',
      description: 'RISC-V 64-bit processors',
      requirements: 'RISC-V64, 4GB RAM, 2GB storage',
      size: '520 MB',
      filename: 'multios-riscv64-1.0.0.iso'
    }
  ];

  const systemRequirements = [
    { component: 'CPU', minimum: '64-bit processor', recommended: 'Multi-core 64-bit processor' },
    { component: 'Memory', minimum: '2GB RAM', recommended: '4GB+ RAM' },
    { component: 'Storage', minimum: '2GB free space', recommended: '8GB+ free space' },
    { component: 'Graphics', minimum: 'VGA compatible', recommended: 'Modern graphics card' },
    { component: 'Network', minimum: 'Optional', recommended: 'Ethernet or WiFi' }
  ];

  const installationMethods = [
    {
      title: 'ISO Installation',
      description: 'Create a bootable USB drive or burn to DVD',
      steps: [
        'Download the appropriate ISO file for your architecture',
        'Create bootable media using tools like Rufus or dd command',
        'Boot from the USB drive or DVD',
        'Follow the installation wizard'
      ]
    },
    {
      title: 'Virtual Machine',
      description: 'Run MultiOS in VirtualBox, QEMU, or VMware',
      steps: [
        'Create a new VM with 2GB+ RAM and 8GB+ storage',
        'Set boot device to the downloaded ISO file',
        'Start the VM and select "Install MultiOS"',
        'Follow the installation process within the VM'
      ]
    },
    {
      title: 'Docker Container',
      description: 'Run MultiOS container for development and testing',
      steps: [
        'Pull the MultiOS container image',
        'Run the container with appropriate hardware access',
        'Access the MultiOS shell through container console',
        'Use for development and testing purposes'
      ]
    }
  ];

  return (
    <div className="pt-16">
      {/* Page Header */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container text-center">
          <h1 className="text-h1 mb-6">Download MultiOS</h1>
          <p className="text-large max-w-3xl mx-auto">
            Download MultiOS for your preferred architecture. Choose from stable releases 
            or cutting-edge development builds. All downloads include comprehensive documentation.
          </p>
        </div>
      </section>

      {/* Version Selector */}
      <section className="section">
        <div className="container">
          <h2 className="text-h2 text-center mb-8">Available Versions</h2>
          <div className="flex flex-wrap justify-center gap-4">
            {versions.map((version) => (
              <button
                key={version.version}
                onClick={() => setSelectedVersion(version.version)}
                className={`px-6 py-4 border-2 text-left max-w-sm ${
                  selectedVersion === version.version
                    ? 'border-red-600 bg-red-50'
                    : 'border-gray-light bg-white hover:border-gray-400'
                }`}
              >
                <div className="flex justify-between items-start mb-2">
                  <span className="text-h3">{version.version}</span>
                  <span className={`text-small px-2 py-1 font-bold uppercase tracking-wider ${
                    version.status === 'Stable' ? 'bg-green-100 text-green-800' : 'bg-yellow-100 text-yellow-800'
                  }`}>
                    {version.status}
                  </span>
                </div>
                <div className="text-caption text-gray-600 mb-2">{version.codename}</div>
                <div className="text-caption text-gray-500 mb-2">Released: {version.date}</div>
                <div className="text-body text-gray-600">{version.description}</div>
              </button>
            ))}
          </div>
        </div>
      </section>

      {/* Architecture Downloads */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Downloads by Architecture</h2>
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
            {architectures.map((arch, index) => (
              <div key={index} className="card-white">
                <h3 className="text-h3 mb-3">{arch.name}</h3>
                <p className="text-body text-gray-600 mb-4">{arch.description}</p>
                
                <div className="border-t border-gray-light pt-4 mb-4">
                  <h4 className="text-small font-bold uppercase tracking-wider text-gray-600 mb-2">Requirements</h4>
                  <p className="text-small text-gray-600">{arch.requirements}</p>
                </div>

                <div className="flex justify-between items-center mb-6">
                  <span className="text-body font-medium">File Size: {arch.size}</span>
                </div>

                <div className="space-y-3">
                  <button className="w-full btn btn-primary">
                    Download {arch.filename}
                  </button>
                  <button className="w-full btn btn-secondary">
                    View Checksums
                  </button>
                  <button className="w-full btn btn-secondary">
                    Installation Guide
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* System Requirements */}
      <section className="section">
        <div className="container">
          <h2 className="text-h2 text-center mb-12">System Requirements</h2>
          <div className="max-w-4xl mx-auto">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
              <div className="card">
                <h3 className="text-h3 mb-4">Minimum Requirements</h3>
                <div className="space-y-3">
                  {systemRequirements.map((req, index) => (
                    <div key={index} className="flex justify-between">
                      <span className="text-small font-medium">{req.component}:</span>
                      <span className="text-small text-gray-600">{req.minimum}</span>
                    </div>
                  ))}
                </div>
              </div>

              <div className="card">
                <h3 className="text-h3 mb-4">Recommended Specifications</h3>
                <div className="space-y-3">
                  {systemRequirements.map((req, index) => (
                    <div key={index} className="flex justify-between">
                      <span className="text-small font-medium">{req.component}:</span>
                      <span className="text-small text-gray-600">{req.recommended}</span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Installation Methods */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Installation Methods</h2>
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
            {installationMethods.map((method, index) => (
              <div key={index} className="card-white">
                <h3 className="text-h3 mb-4">{method.title}</h3>
                <p className="text-body text-gray-600 mb-6">{method.description}</p>
                <ol className="space-y-2">
                  {method.steps.map((step, stepIndex) => (
                    <li key={stepIndex} className="flex items-start">
                      <span className="flex-shrink-0 w-6 h-6 bg-red-600 text-white text-xs font-bold rounded-full flex items-center justify-center mr-3 mt-0.5">
                        {stepIndex + 1}
                      </span>
                      <span className="text-small text-gray-600">{step}</span>
                    </li>
                  ))}
                </ol>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Quick Start Guide */}
      <section className="section">
        <div className="container">
          <div className="max-w-3xl mx-auto text-center">
            <h2 className="text-h2 mb-6">Quick Start Guide</h2>
            <div className="code-block text-left mb-8">
              <pre className="text-sm text-white">{`# Download and verify
wget https://releases.multios.org/1.0.0/multios-x86_64-1.0.0.iso
sha256sum -c multios-x86_64-1.0.0.iso.sha256

# Create bootable USB (Linux)
sudo dd if=multios-x86_64-1.0.0.iso of=/dev/sdX bs=4M status=progress

# Boot and install
# 1. Boot from USB/DVD
# 2. Select "Install MultiOS"
# 3. Choose disk partition
# 4. Create user account
# 5. Complete installation

# First boot
Welcome to MultiOS 1.0.0!
Your user account has been created.
Explore with: help`}</pre>
            </div>
            <div className="flex flex-col sm:flex-row justify-center gap-4">
              <a href="/docs/installation" className="btn btn-primary btn-large">
                Full Installation Guide
              </a>
              <a href="/demos" className="btn btn-secondary btn-large">
                Try Online Demo
              </a>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
};

export default DownloadPage;