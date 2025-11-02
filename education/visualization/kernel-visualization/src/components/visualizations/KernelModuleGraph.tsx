import React, { useEffect, useRef, useState } from 'react';
import ForceGraph2D from 'react-force-graph-2d';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Layers, Package, GitBranch, Activity, AlertCircle } from 'lucide-react';

interface Module {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  license: string;
  dependencies: string[];
  dependents: string[];
  size: number;
  status: 'loaded' | 'unloaded' | 'error' | 'pending';
  memoryUsage: number;
  loadTime: number;
  color: string;
}

interface DependencyLink {
  source: string;
  target: string;
  type: 'depends' | 'required_by';
  weight: number;
}

interface KernelModuleGraphProps {
  realTimeData: boolean;
}

const KernelModuleGraph: React.FC<KernelModuleGraphProps> = ({ realTimeData }) => {
  const [modules, setModules] = useState<Module[]>([]);
  const [links, setLinks] = useState<DependencyLink[]>([]);
  const [selectedModule, setSelectedModule] = useState<Module | null>(null);
  const [filterStatus, setFilterStatus] = useState<string>('all');
  const [viewMode, setViewMode] = useState<'graph' | 'list'>('graph');
  const [graphData, setGraphData] = useState<{ nodes: any[]; links: any[] }>({ nodes: [], links: [] });

  // Generate realistic kernel modules
  const generateKernelModules = (): Module[] => {
    const modules: Module[] = [
      {
        id: 'kernel',
        name: 'kernel',
        version: '5.15.0',
        description: 'Linux Kernel',
        author: 'Linus Torvalds',
        license: 'GPL v2',
        dependencies: [],
        dependents: [],
        size: 1024 * 1024, // 1MB
        memoryUsage: 1024 * 1024,
        loadTime: 0,
        status: 'loaded',
        color: '#ef4444'
      },
      {
        id: 'core',
        name: 'module_core',
        version: '1.0.0',
        description: 'Core kernel module functionality',
        author: 'Kernel Team',
        license: 'GPL v2',
        dependencies: ['kernel'],
        dependents: [],
        size: 256 * 1024,
        memoryUsage: 256 * 1024,
        loadTime: Date.now() - 1000,
        status: 'loaded',
        color: '#f59e0b'
      }
    ];

    // System modules
    const systemModules = [
      { name: 'ip_tables', desc: 'IP packet filtering', deps: ['kernel'] },
      { name: 'iptable_filter', desc: 'Packet filter table', deps: ['ip_tables'] },
      { name: 'iptable_nat', desc: 'NAT table', deps: ['ip_tables'] },
      { name: 'netfilter', desc: 'Network filtering framework', deps: ['kernel'] },
      { name: 'nf_conntrack', desc: 'Connection tracking', deps: ['netfilter'] },
      { name: 'nf_defrag_ipv4', desc: 'IPv4 defragmentation', deps: ['nf_conntrack'] },
      { name: 'nf_defrag_ipv6', desc: 'IPv6 defragmentation', deps: ['nf_conntrack'] },
      { name: 'nf_nat', desc: 'Network address translation', deps: ['nf_conntrack'] },
      { name: 'bridge', desc: 'Bridge networking', deps: ['kernel'] },
      { name: 'br_netfilter', desc: 'Bridge netfilter', deps: ['bridge', 'netfilter'] },
      
      // Storage modules
      { name: 'scsi_mod', desc: 'SCSI subsystem', deps: ['kernel'] },
      { name: 'sd_mod', desc: 'SCSI disk driver', deps: ['scsi_mod'] },
      { name: 'ahci', desc: 'AHCI SATA controller', deps: ['scsi_mod'] },
      { name: 'libata', desc: 'ATA library', deps: ['scsi_mod'] },
      { name: 'dm_mod', desc: 'Device mapper', deps: ['kernel'] },
      { name: 'dm_crypt', desc: 'Device mapper crypto', deps: ['dm_mod'] },
      { name: 'dm_snapshot', desc: 'Device mapper snapshot', deps: ['dm_mod'] },
      { name: 'dm_mirror', desc: 'Device mapper mirror', deps: ['dm_mod'] },
      
      // File system modules
      { name: 'ext4', desc: 'Extended filesystem v4', deps: ['kernel'] },
      { name: 'xfs', desc: 'XFS filesystem', deps: ['kernel'] },
      { name: 'btrfs', desc: 'B-tree filesystem', deps: ['kernel'] },
      { name: 'vfat', desc: 'VFAT/FAT filesystem', deps: ['kernel'] },
      { name: 'ntfs', desc: 'NTFS filesystem', deps: ['kernel'] },
      { name: 'fuse', desc: 'Filesystem in userspace', deps: ['kernel'] },
      
      // Network modules
      { name: 'tcp_bbr', desc: 'BBR TCP congestion control', deps: ['kernel'] },
      { name: 'tcp_congestion', desc: 'TCP congestion control', deps: ['kernel'] },
      { name: 'tun', desc: 'Universal TUN/TAP driver', deps: ['kernel'] },
      { name: 'tap', desc: 'Universal TAP driver', deps: ['tun'] },
      
      // Graphics modules
      { name: 'drm', desc: 'Direct Rendering Manager', deps: ['kernel'] },
      { name: 'i915', desc: 'Intel graphics', deps: ['drm'] },
      { name: 'radeon', desc: 'AMD graphics', deps: ['drm'] },
      { name: 'nouveau', desc: 'NVIDIA graphics', deps: ['drm'] },
      
      // Sound modules
      { name: 'snd', desc: 'ALSA sound subsystem', deps: ['kernel'] },
      { name: 'snd_hda_intel', desc: 'Intel HDA audio', deps: ['snd'] },
      { name: 'snd_hda_codec', desc: 'HD audio codec', deps: ['snd'] },
      { name: 'snd_pcm', desc: 'PCM sound layer', deps: ['snd'] },
      
      // USB modules
      { name: 'usbcore', desc: 'USB core', deps: ['kernel'] },
      { name: 'xhci_hcd', desc: 'USB 3.0 host controller', deps: ['usbcore'] },
      { name: 'ehci_hcd', desc: 'USB 2.0 host controller', deps: ['usbcore'] },
      { name: 'ohci_hcd', desc: 'USB 1.1 host controller', deps: ['usbcore'] },
      { name: 'usbhid', desc: 'USB HID device support', deps: ['usbcore'] },
      
      // Power management
      { name: 'acpi', desc: 'ACPI support', deps: ['kernel'] },
      { name: 'battery', desc: 'Battery support', deps: ['acpi'] },
      { name: 'cpufreq', desc: 'CPU frequency scaling', deps: ['kernel'] },
      { name: 'thermal', desc: 'Thermal management', deps: ['kernel'] },
      
      // Security modules
      { name: 'capability', desc: 'Capability support', deps: ['kernel'] },
      { name: 'commoncap', desc: 'Common capabilities', deps: ['capability'] },
      { name: 'selinux', desc: 'SELinux security', deps: ['kernel'] },
      { name: 'apparmor', desc: 'AppArmor security', deps: ['kernel'] },
      
      // Bluetooth
      { name: 'bluetooth', desc: 'Bluetooth subsystem', deps: ['kernel'] },
      { name: 'btusb', desc: 'Bluetooth USB device', deps: ['bluetooth'] },
      { name: 'rfcomm', desc: 'Bluetooth RFCOMM', deps: ['bluetooth'] },
      { name: 'hidp', desc: 'Bluetooth HIDP', deps: ['bluetooth'] },
      
      // WiFi
      { name: 'mac80211', desc: 'Wireless MAC layer', deps: ['kernel'] },
      { name: 'cfg80211', desc: 'Wireless configuration', deps: ['mac80211'] },
      { name: 'iwlwifi', desc: 'Intel wireless', deps: ['cfg80211'] },
      { name: 'ath9k', desc: 'Atheros wireless', deps: ['cfg80211'] },
      
      // Virtualization
      { name: 'kvm', desc: 'Kernel Virtual Machine', deps: ['kernel'] },
      { name: 'kvm_intel', desc: 'Intel KVM', deps: ['kvm'] },
      { name: 'virtio', desc: 'VirtIO paravirtual', deps: ['kernel'] },
      { name: 'vhost_net', desc: 'VirtIO network host', deps: ['virtio'] }
    ];

    systemModules.forEach((mod, index) => {
      const statuses: Array<Module['status']> = ['loaded', 'loaded', 'loaded', 'unloaded', 'pending'];
      const status = statuses[Math.floor(Math.random() * statuses.length)];
      
      modules.push({
        id: mod.name,
        name: mod.name,
        version: '1.0.' + Math.floor(Math.random() * 10),
        description: mod.desc,
        author: 'Kernel Team',
        license: Math.random() > 0.7 ? 'Proprietary' : 'GPL v2',
        dependencies: mod.deps,
        dependents: [],
        size: Math.floor(Math.random() * 500 * 1024) + 50 * 1024, // 50KB to 500KB
        memoryUsage: Math.floor(Math.random() * 200 * 1024) + 20 * 1024,
        loadTime: Date.now() - Math.random() * 86400000,
        status,
        color: status === 'loaded' ? '#10b981' : status === 'error' ? '#ef4444' : status === 'pending' ? '#f59e0b' : '#6b7280'
      });
    });

    // Calculate dependents
    modules.forEach(module => {
      modules.forEach(other => {
        if (other.dependencies.includes(module.id)) {
          module.dependents.push(other.id);
        }
      });
    });

    return modules;
  };

  // Generate dependency links
  const generateLinks = (modules: Module[]): DependencyLink[] => {
    const links: DependencyLink[] = [];
    
    modules.forEach(module => {
      module.dependencies.forEach(dep => {
        links.push({
          source: module.id,
          target: dep,
          type: 'depends',
          weight: 1
        });
      });
    });

    return links;
  };

  // Initialize modules and links
  useEffect(() => {
    const initialModules = generateKernelModules();
    const initialLinks = generateLinks(initialModules);
    
    setModules(initialModules);
    setLinks(initialLinks);
  }, []);

  // Update graph data for visualization
  useEffect(() => {
    const filteredModules = filterStatus === 'all' 
      ? modules 
      : modules.filter(m => m.status === filterStatus);

    const graphNodes = filteredModules.map(module => ({
      id: module.id,
      name: module.name,
      status: module.status,
      color: module.color,
      size: Math.max(8, Math.min(20, 8 + module.dependents.length * 2)),
      dependencies: module.dependents.length,
      memoryUsage: module.memoryUsage,
      module: module
    }));

    const graphLinks = links.filter(link => {
      return filteredModules.some(m => m.id === link.source) && 
             filteredModules.some(m => m.id === link.target);
    }).map(link => ({
      ...link,
      color: '#6b7280',
      width: link.weight
    }));

    setGraphData({ nodes: graphNodes, links: graphLinks });
  }, [modules, links, filterStatus]);

  // Update module status and metrics in real-time
  useEffect(() => {
    if (!realTimeData) return;

    const interval = setInterval(() => {
      setModules(prev => {
        const updated = [...prev];
        
        // Randomly update module status
        if (Math.random() > 0.9) {
          const randomModule = updated[Math.floor(Math.random() * updated.length)];
          if (randomModule.id !== 'kernel' && randomModule.id !== 'core') {
            const statuses: Array<Module['status']> = ['loaded', 'unloaded', 'error', 'pending'];
            const newStatus = statuses[Math.floor(Math.random() * statuses.length)];
            randomModule.status = newStatus;
            randomModule.color = newStatus === 'loaded' ? '#10b981' : 
                               newStatus === 'error' ? '#ef4444' : 
                               newStatus === 'pending' ? '#f59e0b' : '#6b7280';
          }
        }

        // Update memory usage for loaded modules
        updated.forEach(module => {
          if (module.status === 'loaded') {
            module.memoryUsage = Math.max(0, 
              module.memoryUsage + (Math.random() - 0.5) * 1000
            );
          }
        });

        return updated;
      });
    }, 3000);

    return () => clearInterval(interval);
  }, [realTimeData]);

  // Node painting function for force graph
  const paintNode = (node: any, ctx: CanvasRenderingContext2D, globalScale: number) => {
    const label = node.name;
    const fontSize = Math.max(12, 16 / globalScale);
    
    // Draw node circle
    ctx.beginPath();
    ctx.arc(node.x, node.y, node.size, 0, 2 * Math.PI, false);
    ctx.fillStyle = node.color;
    ctx.fill();
    ctx.strokeStyle = '#1f2937';
    ctx.lineWidth = 2;
    ctx.stroke();

    // Draw label
    ctx.font = `${fontSize}px Sans-Serif`;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillStyle = '#f9fafb';
    ctx.fillText(label, node.x, node.y - node.size - 8);

    // Draw dependency count
    ctx.font = `${fontSize - 2}px Sans-Serif`;
    ctx.fillStyle = '#9ca3af';
    ctx.fillText(`${node.dependencies}`, node.x, node.y + node.size + 8);
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'loaded': return <Activity className="h-3 w-3" />;
      case 'error': return <AlertCircle className="h-3 w-3" />;
      case 'pending': return <GitBranch className="h-3 w-3" />;
      case 'unloaded': return <Package className="h-3 w-3" />;
      default: return <Package className="h-3 w-3" />;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'loaded': return 'bg-green-500';
      case 'error': return 'bg-red-500';
      case 'pending': return 'bg-yellow-500';
      case 'unloaded': return 'bg-gray-500';
      default: return 'bg-gray-500';
    }
  };

  const filteredModules = filterStatus === 'all' 
    ? modules 
    : modules.filter(m => m.status === filterStatus);

  const statusCounts = modules.reduce((counts, module) => {
    counts[module.status] = (counts[module.status] || 0) + 1;
    return counts;
  }, {} as Record<string, number>);

  return (
    <div className="space-y-4">
      {/* Controls and Statistics */}
      <Card className="bg-gray-900 border-gray-700">
        <CardHeader>
          <CardTitle className="flex items-center">
            <Layers className="h-5 w-5 mr-2" />
            Kernel Module Dependency Controls
          </CardTitle>
          <CardDescription>Interactive dependency graph with module relationships</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-4 items-center justify-between">
            <div className="flex items-center space-x-4">
              <div className="flex items-center space-x-2">
                <label className="text-sm text-gray-400">Filter:</label>
                <select
                  value={filterStatus}
                  onChange={(e) => setFilterStatus(e.target.value)}
                  className="bg-gray-800 border border-gray-600 rounded px-3 py-1 text-sm"
                >
                  <option value="all">All Modules</option>
                  <option value="loaded">Loaded</option>
                  <option value="unloaded">Unloaded</option>
                  <option value="error">Error</option>
                  <option value="pending">Pending</option>
                </select>
              </div>
              <div className="flex space-x-2">
                <Button
                  variant={viewMode === 'graph' ? 'default' : 'outline'}
                  size="sm"
                  onClick={() => setViewMode('graph')}
                >
                  Graph View
                </Button>
                <Button
                  variant={viewMode === 'list' ? 'default' : 'outline'}
                  size="sm"
                  onClick={() => setViewMode('list')}
                >
                  List View
                </Button>
              </div>
            </div>
            
            <div className="flex space-x-2">
              <Badge variant="outline">
                Total: {modules.length}
              </Badge>
              <Badge className="bg-green-500 text-white">
                Loaded: {statusCounts.loaded || 0}
              </Badge>
              <Badge className="bg-gray-500 text-white">
                Unloaded: {statusCounts.unloaded || 0}
              </Badge>
              <Badge className="bg-red-500 text-white">
                Error: {statusCounts.error || 0}
              </Badge>
              <Badge className="bg-yellow-500 text-white">
                Pending: {statusCounts.pending || 0}
              </Badge>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Dependency Graph */}
      {viewMode === 'graph' && (
        <Card className="bg-gray-900 border-gray-700">
          <CardHeader>
            <CardTitle>Module Dependency Graph</CardTitle>
            <CardDescription>Interactive force-directed graph of kernel module dependencies</CardDescription>
          </CardHeader>
          <CardContent>
            <div style={{ height: '600px' }}>
              <ForceGraph2D
                graphData={graphData}
                nodeCanvasObject={paintNode}
                onNodeClick={(node) => setSelectedModule(node.module)}
                linkColor={() => '#6b7280'}
                linkWidth={2}
                linkDirectionalParticles={2}
                linkDirectionalParticleSpeed={0.01}
                linkDirectionalParticleWidth={2}
                linkDirectionalParticleColor={() => '#9ca3af'}
                backgroundColor="#1f2937"
                width={960}
                height={550}
                d3AlphaDecay={0.02}
                d3VelocityDecay={0.3}
                cooldownTicks={100}
              />
            </div>
          </CardContent>
        </Card>
      )}

      {/* Module List */}
      {viewMode === 'list' && (
        <Card className="bg-gray-900 border-gray-700">
          <CardHeader>
            <CardTitle>Module List</CardTitle>
            <CardDescription>Tabular view of kernel modules</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="max-h-96 overflow-y-auto">
              <table className="w-full text-sm">
                <thead className="sticky top-0 bg-gray-800">
                  <tr className="border-b border-gray-700">
                    <th className="text-left p-2">Module</th>
                    <th className="text-left p-2">Status</th>
                    <th className="text-left p-2">Size</th>
                    <th className="text-left p-2">Memory</th>
                    <th className="text-left p-2">Dependencies</th>
                    <th className="text-left p-2">Dependents</th>
                    <th className="text-left p-2">License</th>
                  </tr>
                </thead>
                <tbody>
                  {filteredModules.slice(0, 50).map(module => (
                    <tr 
                      key={module.id}
                      className="border-b border-gray-700 hover:bg-gray-800 cursor-pointer"
                      onClick={() => setSelectedModule(module)}
                    >
                      <td className="p-2">
                        <div>
                          <div className="font-medium">{module.name}</div>
                          <div className="text-xs text-gray-400">{module.description}</div>
                        </div>
                      </td>
                      <td className="p-2">
                        <Badge className={`${getStatusColor(module.status)} text-white`}>
                          {getStatusIcon(module.status)}
                          <span className="ml-1 capitalize">{module.status}</span>
                        </Badge>
                      </td>
                      <td className="p-2 font-mono text-xs">
                        {(module.size / 1024).toFixed(1)} KB
                      </td>
                      <td className="p-2 font-mono text-xs">
                        {(module.memoryUsage / 1024).toFixed(1)} KB
                      </td>
                      <td className="p-2 text-center">
                        <Badge variant="outline">{module.dependencies.length}</Badge>
                      </td>
                      <td className="p-2 text-center">
                        <Badge variant="outline">{module.dependents.length}</Badge>
                      </td>
                      <td className="p-2 text-xs">
                        <Badge 
                          variant="outline" 
                          className={module.license === 'GPL v2' ? 'border-green-500' : 'border-orange-500'}
                        >
                          {module.license}
                        </Badge>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Module Details and Dependency Information */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {/* Module Details */}
        {selectedModule && (
          <Card className="bg-gray-900 border-gray-700">
            <CardHeader>
              <CardTitle className="flex items-center justify-between">
                {selectedModule.name}
                <Badge className={`${getStatusColor(selectedModule.status)} text-white`}>
                  {getStatusIcon(selectedModule.status)}
                  <span className="ml-1 capitalize">{selectedModule.status}</span>
                </Badge>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                <div>
                  <div className="text-sm text-gray-400">Description</div>
                  <div>{selectedModule.description}</div>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <div className="text-sm text-gray-400">Version</div>
                    <div className="font-mono">{selectedModule.version}</div>
                  </div>
                  <div>
                    <div className="text-sm text-gray-400">Author</div>
                    <div>{selectedModule.author}</div>
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <div className="text-sm text-gray-400">Size</div>
                    <div className="font-mono">{(selectedModule.size / 1024).toFixed(1)} KB</div>
                  </div>
                  <div>
                    <div className="text-sm text-gray-400">Memory Usage</div>
                    <div className="font-mono">{(selectedModule.memoryUsage / 1024).toFixed(1)} KB</div>
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <div className="text-sm text-gray-400">Dependencies</div>
                    <div className="text-2xl font-bold">{selectedModule.dependencies.length}</div>
                  </div>
                  <div>
                    <div className="text-sm text-gray-400">Dependents</div>
                    <div className="text-2xl font-bold">{selectedModule.dependents.length}</div>
                  </div>
                </div>

                <div>
                  <div className="text-sm text-gray-400">License</div>
                  <Badge 
                    variant="outline" 
                    className={selectedModule.license === 'GPL v2' ? 'border-green-500' : 'border-orange-500'}
                  >
                    {selectedModule.license}
                  </Badge>
                </div>

                <div>
                  <div className="text-sm text-gray-400">Load Time</div>
                  <div>{new Date(selectedModule.loadTime).toLocaleString()}</div>
                </div>
              </div>
            </CardContent>
          </Card>
        )}

        {/* Dependency Chain */}
        {selectedModule && (
          <Card className="bg-gray-900 border-gray-700">
            <CardHeader>
              <CardTitle>Dependency Chain</CardTitle>
              <CardDescription>Dependencies and dependents of selected module</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {/* Dependencies */}
                <div>
                  <h4 className="text-sm font-medium mb-2">Dependencies ({selectedModule.dependencies.length})</h4>
                  {selectedModule.dependencies.length > 0 ? (
                    <div className="space-y-1">
                      {selectedModule.dependencies.map(depId => {
                        const depModule = modules.find(m => m.id === depId);
                        if (!depModule) return null;
                        return (
                          <div 
                            key={depId}
                            className="flex items-center justify-between p-2 bg-gray-800 rounded cursor-pointer hover:bg-gray-700"
                            onClick={() => setSelectedModule(depModule)}
                          >
                            <span className="text-sm">{depModule.name}</span>
                            <Badge className={`${getStatusColor(depModule.status)} text-white text-xs`}>
                              {depModule.status}
                            </Badge>
                          </div>
                        );
                      })}
                    </div>
                  ) : (
                    <div className="text-gray-400 text-sm">No dependencies</div>
                  )}
                </div>

                {/* Dependents */}
                <div>
                  <h4 className="text-sm font-medium mb-2">Dependents ({selectedModule.dependents.length})</h4>
                  {selectedModule.dependents.length > 0 ? (
                    <div className="space-y-1">
                      {selectedModule.dependents.slice(0, 10).map(depId => {
                        const depModule = modules.find(m => m.id === depId);
                        if (!depModule) return null;
                        return (
                          <div 
                            key={depId}
                            className="flex items-center justify-between p-2 bg-gray-800 rounded cursor-pointer hover:bg-gray-700"
                            onClick={() => setSelectedModule(depModule)}
                          >
                            <span className="text-sm">{depModule.name}</span>
                            <Badge className={`${getStatusColor(depModule.status)} text-white text-xs`}>
                              {depModule.status}
                            </Badge>
                          </div>
                        );
                      })}
                      {selectedModule.dependents.length > 10 && (
                        <div className="text-xs text-gray-400 text-center">
                          ... and {selectedModule.dependents.length - 10} more
                        </div>
                      )}
                    </div>
                  ) : (
                    <div className="text-gray-400 text-sm">No dependents</div>
                  )}
                </div>
              </div>
            </CardContent>
          </Card>
        )}
      </div>
    </div>
  );
};

export default KernelModuleGraph;