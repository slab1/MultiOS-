import React, { useEffect, useRef, useState } from 'react';
import * as d3 from 'd3';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { Network, Wifi, Globe, Server, ArrowUp, ArrowDown, Activity } from 'lucide-react';

interface NetworkConnection {
  id: string;
  localIP: string;
  localPort: number;
  remoteIP: string;
  remotePort: number;
  protocol: 'TCP' | 'UDP' | 'ICMP';
  state: 'ESTABLISHED' | 'LISTENING' | 'TIME_WAIT' | 'CLOSE_WAIT' | 'SYN_SENT' | 'SYN_RECV';
  processName: string;
  pid: number;
  bytesIn: number;
  bytesOut: number;
  packetsIn: number;
  packetsOut: number;
  timestamp: number;
}

interface NetworkLayer {
  id: string;
  name: string;
  description: string;
  active: boolean;
  throughput: number;
  color: string;
  connections: NetworkConnection[];
}

interface NetworkStackVisualizationProps {
  realTimeData: boolean;
}

const NetworkStackVisualization: React.FC<NetworkStackVisualizationProps> = ({ realTimeData }) => {
  const svgRef = useRef<SVGSVGElement>(null);
  const [networkLayers, setNetworkLayers] = useState<NetworkLayer[]>([]);
  const [connections, setConnections] = useState<NetworkConnection[]>([]);
  const [selectedConnection, setSelectedConnection] = useState<NetworkConnection | null>(null);
  const [networkStats, setNetworkStats] = useState({
    totalBytesIn: 0,
    totalBytesOut: 0,
    totalConnections: 0,
    activeConnections: 0,
    throughput: 0,
    packetLoss: 0
  });

  // Generate realistic network layer structure
  const generateNetworkLayers = (): NetworkLayer[] => {
    const layers: NetworkLayer[] = [
      {
        id: 'application',
        name: 'Application Layer',
        description: 'HTTP, HTTPS, DNS, SSH, FTP',
        active: true,
        throughput: 0,
        color: '#ef4444',
        connections: []
      },
      {
        id: 'transport',
        name: 'Transport Layer',
        description: 'TCP, UDP',
        active: true,
        throughput: 0,
        color: '#f59e0b',
        connections: []
      },
      {
        id: 'network',
        name: 'Network Layer',
        description: 'IP, ICMP, Routing',
        active: true,
        throughput: 0,
        color: '#3b82f6',
        connections: []
      },
      {
        id: 'datalink',
        name: 'Data Link Layer',
        description: 'Ethernet, ARP, VLAN',
        active: true,
        throughput: 0,
        color: '#10b981',
        connections: []
      },
      {
        id: 'physical',
        name: 'Physical Layer',
        description: 'NIC, Cable, Wireless',
        active: true,
        throughput: 0,
        color: '#8b5cf6',
        connections: []
      }
    ];
    return layers;
  };

  // Generate realistic network connections
  const generateNetworkConnections = (): NetworkConnection[] => {
    const connections: NetworkConnection[] = [];
    const protocols: Array<'TCP' | 'UDP' | 'ICMP'> = ['TCP', 'TCP', 'TCP', 'UDP', 'UDP'];
    const states: Array<NetworkConnection['state']> = ['ESTABLISHED', 'LISTENING', 'TIME_WAIT', 'CLOSE_WAIT'];
    const processes = [
      { name: 'firefox', pid: 1000 },
      { name: 'chrome', pid: 1001 },
      { name: 'ssh', pid: 1002 },
      { name: 'nginx', pid: 1003 },
      { name: 'mysql', pid: 1004 },
      { name: 'redis', pid: 1005 },
      { name: 'node', pid: 1006 },
      { name: 'docker', pid: 1007 },
      { name: 'systemd', pid: 1 }
    ];

    // Local connections
    const localPorts = [80, 443, 22, 3306, 6379, 3000, 8080, 5432, 27017];
    const remoteIPs = [
      '192.168.1.1', '192.168.1.100', '8.8.8.8', '1.1.1.1',
      '142.250.191.14', '52.94.76.1', '151.101.193.140'
    ];

    for (let i = 0; i < 25; i++) {
      const protocol = protocols[Math.floor(Math.random() * protocols.length)];
      const process = processes[Math.floor(Math.random() * processes.length)];
      const state = states[Math.floor(Math.random() * states.length)];
      const localIP = '192.168.1.50';
      const remoteIP = remoteIPs[Math.floor(Math.random() * remoteIPs.length)];
      
      connections.push({
        id: `conn-${i}`,
        localIP,
        localPort: protocol === 'TCP' ? localPorts[Math.floor(Math.random() * localPorts.length)] : 
                   Math.floor(Math.random() * 65535),
        remoteIP: state === 'LISTENING' ? '0.0.0.0' : remoteIP,
        remotePort: state === 'LISTENING' ? 0 : Math.floor(Math.random() * 65535),
        protocol,
        state,
        processName: process.name,
        pid: process.pid,
        bytesIn: Math.floor(Math.random() * 1000000),
        bytesOut: Math.floor(Math.random() * 500000),
        packetsIn: Math.floor(Math.random() * 10000),
        packetsOut: Math.floor(Math.random() * 5000),
        timestamp: Date.now() - Math.random() * 3600000
      });
    }

    return connections;
  };

  // Initialize network stack
  useEffect(() => {
    setNetworkLayers(generateNetworkLayers());
    setConnections(generateNetworkConnections());
  }, []);

  // Update network connections and statistics in real-time
  useEffect(() => {
    if (!realTimeData) return;

    const interval = setInterval(() => {
      setConnections(prev => {
        const updated = [...prev];
        
        // Update existing connections
        updated.forEach(conn => {
          if (conn.state === 'ESTABLISHED') {
            const inTraffic = Math.floor(Math.random() * 10000);
            const outTraffic = Math.floor(Math.random() * 5000);
            conn.bytesIn += inTraffic;
            conn.bytesOut += outTraffic;
            conn.packetsIn += Math.floor(inTraffic / 1500); // Typical packet size
            conn.packetsOut += Math.floor(outTraffic / 1500);
            conn.timestamp = Date.now();
          }
        });

        // Randomly change connection states
        if (Math.random() > 0.95 && updated.length < 30) {
          const protocol = ['TCP', 'UDP'][Math.floor(Math.random() * 2)] as 'TCP' | 'UDP';
          const processes = [
            { name: 'firefox', pid: 1000 },
            { name: 'chrome', pid: 1001 },
            { name: 'nginx', pid: 1003 },
            { name: 'node', pid: 1006 }
          ];
          const process = processes[Math.floor(Math.random() * processes.length)];
          
          updated.push({
            id: `conn-${Date.now()}`,
            localIP: '192.168.1.50',
            localPort: Math.floor(Math.random() * 65535),
            remoteIP: '8.8.8.8',
            remotePort: Math.floor(Math.random() * 65535),
            protocol,
            state: 'ESTABLISHED',
            processName: process.name,
            pid: process.pid,
            bytesIn: 0,
            bytesOut: 0,
            packetsIn: 0,
            packetsOut: 0,
            timestamp: Date.now()
          });
        }

        // Remove old connections
        const filtered = updated.filter(conn => 
          Date.now() - conn.timestamp < 300000 // 5 minutes
        );

        return filtered.length > 0 ? filtered : updated;
      });

      // Update network statistics
      setNetworkStats(prev => {
        const newStats = {
          totalBytesIn: prev.totalBytesIn + Math.floor(Math.random() * 50000),
          totalBytesOut: prev.totalBytesOut + Math.floor(Math.random() * 25000),
          totalConnections: connections.length,
          activeConnections: connections.filter(c => c.state === 'ESTABLISHED').length,
          throughput: Math.random() * 100,
          packetLoss: Math.random() * 2
        };
        return newStats;
      });

    }, 2000);

    return () => clearInterval(interval);
  }, [realTimeData, connections.length]);

  // Update layer throughput
  useEffect(() => {
    setNetworkLayers(prev => {
      return prev.map(layer => {
        let throughput = 0;
        connections.forEach(conn => {
          if (layer.id === 'application' && (conn.processName.includes('firefox') || conn.processName.includes('chrome'))) {
            throughput += (conn.bytesIn + conn.bytesOut) / 1000;
          } else if (layer.id === 'transport' && conn.protocol === 'TCP') {
            throughput += (conn.bytesIn + conn.bytesOut) / 2000;
          } else if (layer.id === 'network' && conn.state === 'ESTABLISHED') {
            throughput += (conn.bytesIn + conn.bytesOut) / 5000;
          }
        });
        return { ...layer, throughput: Math.min(throughput, 100) };
      });
    });
  }, [connections]);

  // Render network stack visualization
  useEffect(() => {
    if (!svgRef.current || networkLayers.length === 0) return;

    const svg = d3.select(svgRef.current);
    const width = 1000;
    const height = 400;

    svg.selectAll('*').remove();

    const g = svg.append('g')
      .attr('transform', 'translate(50,50)');

    // Create layer rectangles
    const layerHeight = 60;
    const layerSpacing = 10;

    const layers = g.selectAll('.network-layer')
      .data(networkLayers)
      .enter()
      .append('g')
      .attr('class', 'network-layer')
      .attr('transform', (d, i) => `translate(0,${i * (layerHeight + layerSpacing)})`);

    // Layer backgrounds
    layers.append('rect')
      .attr('width', 900)
      .attr('height', layerHeight)
      .attr('fill', d => d.active ? d.color : '#374151')
      .attr('opacity', d => d.active ? 0.3 : 0.1)
      .attr('stroke', d => d.color)
      .attr('stroke-width', 2)
      .attr('rx', 8);

    // Layer titles
    layers.append('text')
      .attr('x', 20)
      .attr('y', 25)
      .style('fill', '#f9fafb')
      .style('font-size', '14px')
      .style('font-weight', 'bold')
      .text(d => d.name);

    // Layer descriptions
    layers.append('text')
      .attr('x', 20)
      .attr('y', 45)
      .style('fill', '#9ca3af')
      .style('font-size', '11px')
      .text(d => d.description);

    // Throughput bars
    layers.append('rect')
      .attr('x', 600)
      .attr('y', 15)
      .attr('width', 280)
      .attr('height', 30)
      .attr('fill', '#374151')
      .attr('rx', 4);

    layers.append('rect')
      .attr('x', 600)
      .attr('y', 15)
      .attr('width', d => 2.8 * d.throughput)
      .attr('height', 30)
      .attr('fill', d => d.color)
      .attr('rx', 4);

    layers.append('text')
      .attr('x', 870)
      .attr('y', 32)
      .attr('text-anchor', 'end')
      .style('fill', '#f9fafb')
      .style('font-size', '10px')
      .text(d => `${d.throughput.toFixed(1)} MB/s`);

    // Active indicator
    layers.append('circle')
      .attr('cx', 50)
      .attr('cy', 30)
      .attr('r', 6)
      .attr('fill', d => d.active ? '#10b981' : '#6b7280');

    // Data flow arrows between layers
    for (let i = 0; i < networkLayers.length - 1; i++) {
      const y1 = i * (layerHeight + layerSpacing) + layerHeight;
      const y2 = (i + 1) * (layerHeight + layerSpacing);

      g.append('path')
        .attr('d', `M 450 ${y1} L 450 ${y2}`)
        .attr('stroke', '#6b7280')
        .attr('stroke-width', 2)
        .attr('marker-end', 'url(#arrowhead)')
        .attr('opacity', 0.5);

      g.append('path')
        .attr('d', `M 470 ${y2} L 470 ${y1}`)
        .attr('stroke', '#6b7280')
        .attr('stroke-width', 2)
        .attr('marker-end', 'url(#arrowhead)')
        .attr('opacity', 0.3);
    }

    // Add arrow marker definition
    svg.append('defs')
      .append('marker')
      .attr('id', 'arrowhead')
      .attr('viewBox', '0 -5 10 10')
      .attr('refX', 8)
      .attr('refY', 0)
      .attr('markerWidth', 6)
      .attr('markerHeight', 6)
      .attr('orient', 'auto')
      .append('path')
      .attr('d', 'M0,-5L10,0L0,5')
      .attr('fill', '#6b7280');

  }, [networkLayers]);

  const activeConnections = connections.filter(c => c.state === 'ESTABLISHED');
  const listeningConnections = connections.filter(c => c.state === 'LISTENING');

  const getProtocolColor = (protocol: string) => {
    switch (protocol) {
      case 'TCP': return 'bg-blue-500';
      case 'UDP': return 'bg-green-500';
      case 'ICMP': return 'bg-red-500';
      default: return 'bg-gray-500';
    }
  };

  const getStateColor = (state: string) => {
    switch (state) {
      case 'ESTABLISHED': return 'bg-green-500';
      case 'LISTENING': return 'bg-blue-500';
      case 'TIME_WAIT': return 'bg-yellow-500';
      case 'CLOSE_WAIT': return 'bg-orange-500';
      case 'SYN_SENT': return 'bg-purple-500';
      case 'SYN_RECV': return 'bg-pink-500';
      default: return 'bg-gray-500';
    }
  };

  return (
    <div className="space-y-4">
      {/* Network Statistics */}
      <div className="grid grid-cols-2 md:grid-cols-6 gap-4">
        <Card className="bg-gray-900 border-gray-700">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm flex items-center">
              <Globe className="h-4 w-4 mr-2" />
              Total Connections
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{networkStats.totalConnections}</div>
            <div className="text-xs text-gray-400">All states</div>
          </CardContent>
        </Card>

        <Card className="bg-gray-900 border-gray-700">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm flex items-center">
              <Activity className="h-4 w-4 mr-2" />
              Active
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{networkStats.activeConnections}</div>
            <div className="text-xs text-gray-400">Established</div>
          </CardContent>
        </Card>

        <Card className="bg-gray-900 border-gray-700">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm flex items-center">
              <ArrowDown className="h-4 w-4 mr-2" />
              Bytes In
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{(networkStats.totalBytesIn / 1024 / 1024).toFixed(1)}M</div>
            <div className="text-xs text-gray-400">Received</div>
          </CardContent>
        </Card>

        <Card className="bg-gray-900 border-gray-700">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm flex items-center">
              <ArrowUp className="h-4 w-4 mr-2" />
              Bytes Out
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{(networkStats.totalBytesOut / 1024 / 1024).toFixed(1)}M</div>
            <div className="text-xs text-gray-400">Sent</div>
          </CardContent>
        </Card>

        <Card className="bg-gray-900 border-gray-700">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm flex items-center">
              <Wifi className="h-4 w-4 mr-2" />
              Throughput
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{networkStats.throughput.toFixed(1)}</div>
            <div className="text-xs text-gray-400">MB/s</div>
          </CardContent>
        </Card>

        <Card className="bg-gray-900 border-gray-700">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm flex items-center">
              <Network className="h-4 w-4 mr-2" />
              Packet Loss
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{networkStats.packetLoss.toFixed(2)}%</div>
            <Progress value={networkStats.packetLoss} className="mt-2" />
          </CardContent>
        </Card>
      </div>

      {/* Network Stack Visualization */}
      <Card className="bg-gray-900 border-gray-700">
        <CardHeader>
          <CardTitle className="flex items-center">
            <Network className="h-5 w-5 mr-2" />
            Network Stack Layers
          </CardTitle>
          <CardDescription>OSI model with real-time throughput monitoring</CardDescription>
        </CardHeader>
        <CardContent>
          <svg
            ref={svgRef}
            width={1000}
            height={400}
            className="border border-gray-600 rounded"
            style={{ background: '#1f2937' }}
          />
        </CardContent>
      </Card>

      {/* Active Connections Table */}
      <Card className="bg-gray-900 border-gray-700">
        <CardHeader>
          <CardTitle>Active Network Connections</CardTitle>
          <CardDescription>Real-time connection monitoring with packet tracking</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="max-h-96 overflow-y-auto">
            <table className="w-full text-sm">
              <thead className="sticky top-0 bg-gray-800">
                <tr className="border-b border-gray-700">
                  <th className="text-left p-2">Process</th>
                  <th className="text-left p-2">Local Address</th>
                  <th className="text-left p-2">Remote Address</th>
                  <th className="text-left p-2">Protocol</th>
                  <th className="text-left p-2">State</th>
                  <th className="text-left p-2">Bytes In</th>
                  <th className="text-left p-2">Bytes Out</th>
                </tr>
              </thead>
              <tbody>
                {connections.slice(0, 20).map(conn => (
                  <tr 
                    key={conn.id} 
                    className="border-b border-gray-700 hover:bg-gray-800 cursor-pointer"
                    onClick={() => setSelectedConnection(conn)}
                  >
                    <td className="p-2">
                      <div>
                        <div className="font-medium">{conn.processName}</div>
                        <div className="text-xs text-gray-400">PID: {conn.pid}</div>
                      </div>
                    </td>
                    <td className="p-2 font-mono text-xs">
                      {conn.localIP}:{conn.localPort}
                    </td>
                    <td className="p-2 font-mono text-xs">
                      {conn.remoteIP}:{conn.remotePort}
                    </td>
                    <td className="p-2">
                      <Badge className={`${getProtocolColor(conn.protocol)} text-white`}>
                        {conn.protocol}
                      </Badge>
                    </td>
                    <td className="p-2">
                      <Badge className={`${getStateColor(conn.state)} text-white text-xs`}>
                        {conn.state}
                      </Badge>
                    </td>
                    <td className="p-2 font-mono text-xs">
                      {(conn.bytesIn / 1024).toFixed(1)}K
                    </td>
                    <td className="p-2 font-mono text-xs">
                      {(conn.bytesOut / 1024).toFixed(1)}K
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>

      {/* Connection Details and Protocol Distribution */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {/* Connection Details */}
        {selectedConnection && (
          <Card className="bg-gray-900 border-gray-700">
            <CardHeader>
              <CardTitle>Connection Details</CardTitle>
              <CardDescription>Detailed information about selected connection</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <div className="text-sm text-gray-400">Process</div>
                    <div className="font-medium">{selectedConnection.processName}</div>
                  </div>
                  <div>
                    <div className="text-sm text-gray-400">PID</div>
                    <div className="font-mono">{selectedConnection.pid}</div>
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <div className="text-sm text-gray-400">Protocol</div>
                    <Badge className={`${getProtocolColor(selectedConnection.protocol)} text-white`}>
                      {selectedConnection.protocol}
                    </Badge>
                  </div>
                  <div>
                    <div className="text-sm text-gray-400">State</div>
                    <Badge className={`${getStateColor(selectedConnection.state)} text-white`}>
                      {selectedConnection.state}
                    </Badge>
                  </div>
                </div>

                <div>
                  <div className="text-sm text-gray-400">Local Address</div>
                  <div className="font-mono text-xs bg-gray-800 p-2 rounded">
                    {selectedConnection.localIP}:{selectedConnection.localPort}
                  </div>
                </div>

                <div>
                  <div className="text-sm text-gray-400">Remote Address</div>
                  <div className="font-mono text-xs bg-gray-800 p-2 rounded">
                    {selectedConnection.remoteIP}:{selectedConnection.remotePort}
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <div className="text-sm text-gray-400">Packets In</div>
                    <div className="font-mono">{selectedConnection.packetsIn.toLocaleString()}</div>
                  </div>
                  <div>
                    <div className="text-sm text-gray-400">Packets Out</div>
                    <div className="font-mono">{selectedConnection.packetsOut.toLocaleString()}</div>
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <div className="text-sm text-gray-400">Bytes In</div>
                    <div className="font-mono">{(selectedConnection.bytesIn / 1024).toFixed(2)} KB</div>
                  </div>
                  <div>
                    <div className="text-sm text-gray-400">Bytes Out</div>
                    <div className="font-mono">{(selectedConnection.bytesOut / 1024).toFixed(2)} KB</div>
                  </div>
                </div>

                <div>
                  <div className="text-sm text-gray-400">Established</div>
                  <div>{new Date(selectedConnection.timestamp).toLocaleString()}</div>
                </div>
              </div>
            </CardContent>
          </Card>
        )}

        {/* Protocol and State Distribution */}
        <Card className="bg-gray-900 border-gray-700">
          <CardHeader>
            <CardTitle>Connection Distribution</CardTitle>
            <CardDescription>Protocol and state statistics</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-6">
              {/* Protocol Distribution */}
              <div>
                <h4 className="text-sm font-medium mb-3">Protocol Distribution</h4>
                <div className="space-y-2">
                  {Object.entries(
                    connections.reduce((acc, conn) => {
                      acc[conn.protocol] = (acc[conn.protocol] || 0) + 1;
                      return acc;
                    }, {} as Record<string, number>)
                  ).map(([protocol, count]) => {
                    const percentage = (count / connections.length) * 100;
                    return (
                      <div key={protocol} className="space-y-1">
                        <div className="flex justify-between items-center">
                          <span className="text-sm">{protocol}</span>
                          <span className="text-xs text-gray-400">{count} ({percentage.toFixed(1)}%)</span>
                        </div>
                        <div className="w-full bg-gray-700 rounded-full h-2">
                          <div
                            className={`h-2 rounded-full ${getProtocolColor(protocol).replace('bg-', 'bg-')}`}
                            style={{ width: `${percentage}%` }}
                          />
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>

              {/* State Distribution */}
              <div>
                <h4 className="text-sm font-medium mb-3">Connection States</h4>
                <div className="space-y-2">
                  {Object.entries(
                    connections.reduce((acc, conn) => {
                      acc[conn.state] = (acc[conn.state] || 0) + 1;
                      return acc;
                    }, {} as Record<string, number>)
                  ).map(([state, count]) => {
                    const percentage = (count / connections.length) * 100;
                    return (
                      <div key={state} className="space-y-1">
                        <div className="flex justify-between items-center">
                          <span className="text-sm">{state}</span>
                          <span className="text-xs text-gray-400">{count} ({percentage.toFixed(1)}%)</span>
                        </div>
                        <div className="w-full bg-gray-700 rounded-full h-2">
                          <div
                            className={`h-2 rounded-full ${getStateColor(state).replace('bg-', 'bg-')}`}
                            style={{ width: `${percentage}%` }}
                          />
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
};

export default NetworkStackVisualization;