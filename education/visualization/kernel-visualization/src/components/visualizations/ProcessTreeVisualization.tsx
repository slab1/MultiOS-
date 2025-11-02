import React, { useEffect, useRef, useState } from 'react';
import * as d3 from 'd3';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { GitBranch, Search, Play, Pause, Square, Zap } from 'lucide-react';

interface Process {
  id: string;
  pid: number;
  ppid: number;
  name: string;
  state: 'running' | 'sleeping' | 'waiting' | 'stopped' | 'zombie';
  cpuPercent: number;
  memoryPercent: number;
  children: Process[];
  priority: number;
  startTime: number;
  command: string;
  user: string;
}

interface ProcessTreeVisualizationProps {
  realTimeData: boolean;
}

const ProcessTreeVisualization: React.FC<ProcessTreeVisualizationProps> = ({ realTimeData }) => {
  const svgRef = useRef<SVGSVGElement>(null);
  const [processes, setProcesses] = useState<Process[]>([]);
  const [selectedProcess, setSelectedProcess] = useState<Process | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(new Set());

  // Generate realistic process tree
  const generateProcessTree = (): Process[] => {
    const processes: Process[] = [
      {
        id: '1',
        pid: 1,
        ppid: 0,
        name: 'kernel',
        state: 'running',
        cpuPercent: 2.5,
        memoryPercent: 1.2,
        priority: -100,
        startTime: Date.now() - 86400000,
        command: '/sbin/kernel',
        user: 'root',
        children: []
      }
    ];

    // System processes
    const systemProcesses = [
      { pid: 2, name: 'kthreadd', cpuPercent: 0.1, memoryPercent: 0.1 },
      { pid: 3, name: 'rcu_gp', cpuPercent: 0.0, memoryPercent: 0.1 },
      { pid: 4, name: 'rcu_par_gp', cpuPercent: 0.0, memoryPercent: 0.1 },
      { pid: 5, name: 'kworker/0:0H', cpuPercent: 0.1, memoryPercent: 0.1 },
      { pid: 6, name: 'kworker/0:1', cpuPercent: 0.2, memoryPercent: 0.1 },
      { pid: 7, name: 'mm_percpu_wq', cpuPercent: 0.0, memoryPercent: 0.1 },
      { pid: 8, name: 'ksoftirqd/0', cpuPercent: 0.1, memoryPercent: 0.2 },
      { pid: 9, name: 'rcu_sched', cpuPercent: 0.0, memoryPercent: 0.1 },
      { pid: 10, name: 'migration/0', cpuPercent: 0.0, memoryPercent: 0.1 },
      { pid: 11, name: 'watchdog/0', cpuPercent: 0.0, memoryPercent: 0.1 }
    ];

    systemProcesses.forEach(proc => {
      processes.push({
        ...proc,
        ppid: 2,
        state: 'sleeping',
        priority: 0,
        startTime: Date.now() - 86400000,
        command: proc.name,
        user: 'root',
        children: []
      });
    });

    // User space processes
    const userProcesses = [
      { pid: 100, name: 'systemd', ppid: 1, cpuPercent: 0.3, memoryPercent: 1.5 },
      { pid: 101, name: 'systemd-journal', ppid: 100, cpuPercent: 0.2, memoryPercent: 0.8 },
      { pid: 102, name: 'systemd-udevd', ppid: 100, cpuPercent: 0.1, memoryPercent: 0.6 },
      { pid: 103, name: 'systemd-network', ppid: 100, cpuPercent: 0.1, memoryPercent: 0.4 },
      { pid: 104, name: 'systemd-resolve', ppid: 100, cpuPercent: 0.1, memoryPercent: 0.3 },
      { pid: 110, name: 'sshd', ppid: 100, cpuPercent: 0.1, memoryPercent: 0.5 },
      { pid: 120, name: 'networkd', ppid: 100, cpuPercent: 0.2, memoryPercent: 1.2 },
      { pid: 200, name: 'firefox', ppid: 100, cpuPercent: 15.3, memoryPercent: 12.8 },
      { pid: 201, name: 'chrome', ppid: 200, cpuPercent: 8.7, memoryPercent: 6.4 },
      { pid: 202, name: 'gpu-process', ppid: 201, cpuPercent: 3.2, memoryPercent: 2.1 },
      { pid: 203, name: 'utility', ppid: 201, cpuPercent: 1.1, memoryPercent: 0.8 },
      { pid: 220, name: 'code', ppid: 100, cpuPercent: 5.4, memoryPercent: 8.9 },
      { pid: 221, name: 'extensionHost', ppid: 220, cpuPercent: 2.3, memoryPercent: 3.2 },
      { pid: 222, name: 'renderer', ppid: 220, cpuPercent: 4.1, memoryPercent: 5.7 },
      { pid: 230, name: 'terminal', ppid: 100, cpuPercent: 0.8, memoryPercent: 1.2 },
      { pid: 231, name: 'bash', ppid: 230, cpuPercent: 0.3, memoryPercent: 0.5 },
      { pid: 250, name: 'node', ppid: 100, cpuPercent: 2.1, memoryPercent: 4.6 },
      { pid: 260, name: 'python', pppid: 100, cpuPercent: 1.2, memoryPercent: 2.3 },
      { pid: 270, name: 'docker', ppid: 100, cpuPercent: 0.9, memoryPercent: 1.8 }
    ];

    userProcesses.forEach(proc => {
      const states: Array<Process['state']> = ['running', 'sleeping', 'waiting'];
      const randomState = states[Math.floor(Math.random() * states.length)];
      
      processes.push({
        ...proc,
        state: randomState,
        priority: Math.floor(Math.random() * 40) - 20,
        startTime: Date.now() - Math.random() * 86400000,
        command: proc.name,
        user: Math.random() > 0.5 ? 'user' : 'root',
        children: []
      });
    });

    return processes;
  };

  // Build hierarchical tree structure
  const buildTree = (processes: Process[]): Process[] => {
    const processMap = new Map<string, Process>();
    const rootProcesses: Process[] = [];

    // Create map for quick lookup
    processes.forEach(proc => {
      processMap.set(proc.pid.toString(), { ...proc, children: [] });
    });

    // Build parent-child relationships
    processes.forEach(proc => {
      const process = processMap.get(proc.pid.toString());
      if (!process) return;

      if (proc.ppid === 0 || proc.ppid === 1) {
        rootProcesses.push(process);
      } else {
        const parent = processMap.get(proc.ppid.toString());
        if (parent) {
          parent.children.push(process);
        }
      }
    });

    return rootProcesses;
  };

  // Initialize processes
  useEffect(() => {
    const processTree = buildTree(generateProcessTree());
    setProcesses(processTree);
  }, []);

  // Update process metrics periodically
  useEffect(() => {
    if (!realTimeData) return;

    const interval = setInterval(() => {
      setProcesses(prev => {
        const updateProcessMetrics = (processes: Process[]): Process[] => {
          return processes.map(proc => ({
            ...proc,
            cpuPercent: Math.max(0, proc.cpuPercent + (Math.random() - 0.5) * 5),
            memoryPercent: Math.max(0, Math.min(100, proc.memoryPercent + (Math.random() - 0.5) * 2)),
            children: updateProcessMetrics(proc.children)
          }));
        };
        return updateProcessMetrics(prev);
      });
    }, 3000);

    return () => clearInterval(interval);
  }, [realTimeData]);

  // Render process tree visualization
  useEffect(() => {
    if (!svgRef.current || processes.length === 0) return;

    const svg = d3.select(svgRef.current);
    const width = 1000;
    const height = 600;

    svg.selectAll('*').remove();

    const g = svg.append('g')
      .attr('transform', 'translate(50,50)');

    // Create tree layout
    const tree = d3.tree<Process>()
      .size([height - 100, width - 200]);

    // Flatten processes for visualization
    const flattenProcessTree = (processes: Process[]): Process[] => {
      const result: Process[] = [];
      const traverse = (procs: Process[], parentX = 0) => {
        procs.forEach(proc => {
          result.push({ ...proc });
          if (proc.children.length > 0) {
            traverse(proc.children);
          }
        });
      };
      traverse(processes);
      return result;
    };

    const flattenedProcesses = flattenProcessTree(processes);
    const root = d3.hierarchy({ children: processes } as any);
    const treeData = tree(root);

    // Create links
    const links = g.selectAll('.tree-link')
      .data(treeData.links())
      .enter()
      .append('path')
      .attr('class', 'tree-link')
      .attr('d', d3.linkHorizontal<any, any>()
        .x(d => d.y)
        .y(d => d.x)
      )
      .attr('stroke', '#374151')
      .attr('stroke-width', 1.5)
      .attr('fill', 'none');

    // Create nodes
    const nodes = g.selectAll('.tree-node')
      .data(treeData.descendants())
      .enter()
      .append('g')
      .attr('class', 'tree-node')
      .attr('transform', d => `translate(${d.y},${d.x})`);

    // Add node circles
    nodes.append('circle')
      .attr('r', d => {
        const proc = d.data as Process;
        return Math.max(4, Math.min(12, 4 + proc.cpuPercent * 0.3));
      })
      .attr('fill', d => {
        const proc = d.data as Process;
        const colors = {
          running: '#10b981',
          sleeping: '#3b82f6',
          waiting: '#f59e0b',
          stopped: '#ef4444',
          zombie: '#6b7280'
        };
        return colors[proc.state];
      })
      .attr('stroke', '#1f2937')
      .attr('stroke-width', 2)
      .style('cursor', 'pointer')
      .on('click', (event, d) => {
        setSelectedProcess(d.data as Process);
      })
      .on('mouseover', function(event, d) {
        d3.select(this).attr('r', Number(d3.select(this).attr('r')) + 2);
      })
      .on('mouseout', function(event, d) {
        const originalR = Math.max(4, Math.min(12, 4 + (d.data as Process).cpuPercent * 0.3));
        d3.select(this).attr('r', originalR);
      });

    // Add node labels
    nodes.append('text')
      .attr('dx', 15)
      .attr('dy', '0.35em')
      .style('fill', '#f9fafb')
      .style('font-size', '11px')
      .style('cursor', 'pointer')
      .text(d => {
        const proc = d.data as Process;
        return `${proc.name} (${proc.pid})`;
      })
      .on('click', (event, d) => {
        setSelectedProcess(d.data as Process);
      });

    // Add CPU usage bars
    nodes.append('rect')
      .attr('x', 15)
      .attr('y', 10)
      .attr('width', d => Math.max(1, (d.data as Process).cpuPercent * 2))
      .attr('height', 3)
      .attr('fill', '#ef4444');

    // Add memory usage bars
    nodes.append('rect')
      .attr('x', 15)
      .attr('y', 15)
      .attr('width', d => Math.max(1, (d.data as Process).memoryPercent * 2))
      .attr('height', 3)
      .attr('fill', '#3b82f6');

  }, [processes]);

  const getStateIcon = (state: Process['state']) => {
    switch (state) {
      case 'running': return <Play className="h-3 w-3" />;
      case 'sleeping': return <Pause className="h-3 w-3" />;
      case 'waiting': return <Zap className="h-3 w-3" />;
      case 'stopped': return <Square className="h-3 w-3" />;
      case 'zombie': return <Square className="h-3 w-3" />;
      default: return <Play className="h-3 w-3" />;
    }
  };

  const getStateColor = (state: Process['state']) => {
    const colors = {
      running: 'bg-green-500',
      sleeping: 'bg-blue-500',
      waiting: 'bg-yellow-500',
      stopped: 'bg-red-500',
      zombie: 'bg-gray-500'
    };
    return colors[state];
  };

  const totalCpu = processes.reduce((sum, proc) => sum + proc.cpuPercent, 0);
  const totalMemory = processes.reduce((sum, proc) => sum + proc.memoryPercent, 0);

  return (
    <div className="space-y-4">
      {/* Controls */}
      <Card className="bg-gray-900 border-gray-700">
        <CardHeader>
          <CardTitle className="flex items-center">
            <GitBranch className="h-5 w-5 mr-2" />
            Process Tree Controls
          </CardTitle>
          <CardDescription>Interactive process hierarchy with parent-child relationships</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-4 items-center">
            <div className="flex items-center space-x-2">
              <Search className="h-4 w-4" />
              <Input
                type="text"
                placeholder="Search processes..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="bg-gray-800 border-gray-600 text-sm w-48"
              />
            </div>
            <Badge variant="outline">
              {processes.length} processes
            </Badge>
            <Badge variant="outline">
              CPU: {totalCpu.toFixed(1)}%
            </Badge>
            <Badge variant="outline">
              Memory: {totalMemory.toFixed(1)}%
            </Badge>
          </div>
        </CardContent>
      </Card>

      {/* Process Tree Visualization */}
      <Card className="bg-gray-900 border-gray-700">
        <CardHeader>
          <CardTitle>Interactive Process Tree</CardTitle>
          <CardDescription>Click on nodes to view detailed information</CardDescription>
        </CardHeader>
        <CardContent>
          <svg
            ref={svgRef}
            width={1000}
            height={600}
            className="border border-gray-600 rounded"
            style={{ background: '#1f2937' }}
          />
        </CardContent>
      </Card>

      {/* Selected Process Details and Legend */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {/* Process Details */}
        {selectedProcess && (
          <Card className="bg-gray-900 border-gray-700">
            <CardHeader>
              <CardTitle className="flex items-center justify-between">
                {selectedProcess.name}
                <Badge className={`${getStateColor(selectedProcess.state)} text-white`}>
                  {getStateIcon(selectedProcess.state)}
                  <span className="ml-1 capitalize">{selectedProcess.state}</span>
                </Badge>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <div className="text-sm text-gray-400">Process ID</div>
                    <div className="font-mono text-lg">{selectedProcess.pid}</div>
                  </div>
                  <div>
                    <div className="text-sm text-gray-400">Parent PID</div>
                    <div className="font-mono text-lg">{selectedProcess.ppid}</div>
                  </div>
                </div>
                
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <div className="text-sm text-gray-400">CPU Usage</div>
                    <div className="text-lg">{selectedProcess.cpuPercent.toFixed(1)}%</div>
                    <div className="w-full bg-gray-700 rounded-full h-2 mt-1">
                      <div
                        className="bg-red-500 h-2 rounded-full"
                        style={{ width: `${Math.min(100, selectedProcess.cpuPercent)}%` }}
                      />
                    </div>
                  </div>
                  <div>
                    <div className="text-sm text-gray-400">Memory Usage</div>
                    <div className="text-lg">{selectedProcess.memoryPercent.toFixed(1)}%</div>
                    <div className="w-full bg-gray-700 rounded-full h-2 mt-1">
                      <div
                        className="bg-blue-500 h-2 rounded-full"
                        style={{ width: `${Math.min(100, selectedProcess.memoryPercent)}%` }}
                      />
                    </div>
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <div className="text-sm text-gray-400">Priority</div>
                    <div className="font-mono">{selectedProcess.priority}</div>
                  </div>
                  <div>
                    <div className="text-sm text-gray-400">User</div>
                    <div>{selectedProcess.user}</div>
                  </div>
                </div>

                <div>
                  <div className="text-sm text-gray-400">Command</div>
                  <div className="font-mono text-xs bg-gray-800 p-2 rounded">
                    {selectedProcess.command}
                  </div>
                </div>

                <div>
                  <div className="text-sm text-gray-400">Start Time</div>
                  <div>{new Date(selectedProcess.startTime).toLocaleString()}</div>
                </div>
              </div>
            </CardContent>
          </Card>
        )}

        {/* Legend */}
        <Card className="bg-gray-900 border-gray-700">
          <CardHeader>
            <CardTitle>Process States Legend</CardTitle>
            <CardDescription>Node colors indicate process state</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              <div className="flex items-center space-x-3">
                <div className="w-4 h-4 rounded-full bg-green-500"></div>
                <div>
                  <div className="text-sm font-medium">Running</div>
                  <div className="text-xs text-gray-400">Process actively using CPU</div>
                </div>
              </div>
              
              <div className="flex items-center space-x-3">
                <div className="w-4 h-4 rounded-full bg-blue-500"></div>
                <div>
                  <div className="text-sm font-medium">Sleeping</div>
                  <div className="text-xs text-gray-400">Waiting for I/O or event</div>
                </div>
              </div>
              
              <div className="flex items-center space-x-3">
                <div className="w-4 h-4 rounded-full bg-yellow-500"></div>
                <div>
                  <div className="text-sm font-medium">Waiting</div>
                  <div className="text-xs text-gray-400">Waiting for resource</div>
                </div>
              </div>
              
              <div className="flex items-center space-x-3">
                <div className="w-4 h-4 rounded-full bg-red-500"></div>
                <div>
                  <div className="text-sm font-medium">Stopped</div>
                  <div className="text-xs text-gray-400">Process suspended</div>
                </div>
              </div>
              
              <div className="flex items-center space-x-3">
                <div className="w-4 h-4 rounded-full bg-gray-500"></div>
                <div>
                  <div className="text-sm font-medium">Zombie</div>
                  <div className="text-xs text-gray-400">Terminated but not cleaned</div>
                </div>
              </div>
            </div>

            <div className="mt-6 pt-4 border-t border-gray-700">
              <h4 className="text-sm font-medium mb-2">Visual Indicators</h4>
              <div className="space-y-2">
                <div className="flex items-center space-x-2">
                  <div className="w-6 h-1 bg-red-500"></div>
                  <span className="text-xs">CPU usage bar</span>
                </div>
                <div className="flex items-center space-x-2">
                  <div className="w-6 h-1 bg-blue-500"></div>
                  <span className="text-xs">Memory usage bar</span>
                </div>
                <div className="flex items-center space-x-2">
                  <div className="w-4 h-4 rounded-full border-2 border-white"></div>
                  <span className="text-xs">Node size = CPU usage</span>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
};

export default ProcessTreeVisualization;