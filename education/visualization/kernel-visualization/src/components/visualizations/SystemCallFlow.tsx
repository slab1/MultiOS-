import React, { useEffect, useRef, useState } from 'react';
import * as d3 from 'd3';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { GitBranch, Activity, Clock, Zap, ArrowRight, Filter } from 'lucide-react';

interface SystemCall {
  id: string;
  name: string;
  category: 'file' | 'process' | 'network' | 'memory' | 'device' | 'sync';
  frequency: number;
  averageTime: number;
  successRate: number;
  parameters: string[];
  description: string;
  color: string;
}

interface SystemCallTrace {
  id: string;
  timestamp: number;
  pid: number;
  processName: string;
  callName: string;
  duration: number;
  result: 'success' | 'error' | 'timeout';
  parameters: Record<string, any>;
  returnValue: string | number;
  userStack: string[];
  kernelStack: string[];
}

interface SystemCallFlowProps {
  realTimeData: boolean;
}

const SystemCallFlow: React.FC<SystemCallFlowProps> = ({ realTimeData }) => {
  const svgRef = useRef<SVGSVGElement>(null);
  const flowRef = useRef<SVGSVGElement>(null);
  const [systemCalls, setSystemCalls] = useState<SystemCall[]>([]);
  const [callTraces, setCallTraces] = useState<SystemCallTrace[]>([]);
  const [selectedCall, setSelectedCall] = useState<SystemCall | null>(null);
  const [selectedTrace, setSelectedTrace] = useState<SystemCallTrace | null>(null);
  const [filterCategory, setFilterCategory] = useState<string>('all');
  const [flowData, setFlowData] = useState<{ nodes: any[]; links: any[] }>({ nodes: [], links: [] });
  const [stats, setStats] = useState({
    totalCalls: 0,
    callsPerSecond: 0,
    averageLatency: 0,
    errorRate: 0,
    topCaller: '',
    topCall: ''
  });

  // Generate realistic system calls
  const generateSystemCalls = (): SystemCall[] => {
    const calls: SystemCall[] = [
      // File operations
      {
        id: 'open',
        name: 'open',
        category: 'file',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: ['pathname', 'flags', 'mode'],
        description: 'Open a file',
        color: '#ef4444'
      },
      {
        id: 'read',
        name: 'read',
        category: 'file',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: ['fd', 'buf', 'count'],
        description: 'Read from file descriptor',
        color: '#ef4444'
      },
      {
        id: 'write',
        name: 'write',
        category: 'file',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: ['fd', 'buf', 'count'],
        description: 'Write to file descriptor',
        color: '#ef4444'
      },
      {
        id: 'close',
        name: 'close',
        category: 'file',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: ['fd'],
        description: 'Close a file descriptor',
        color: '#ef4444'
      },
      {
        id: 'stat',
        name: 'stat',
        category: 'file',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: ['pathname', 'statbuf'],
        description: 'Get file status',
        color: '#ef4444'
      },
      {
        id: 'mmap',
        name: 'mmap',
        category: 'memory',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: ['addr', 'length', 'prot', 'flags', 'fd', 'offset'],
        description: 'Memory mapping',
        color: '#f59e0b'
      },
      {
        id: 'munmap',
        name: 'munmap',
        category: 'memory',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: ['addr', 'length'],
        description: 'Unmap memory',
        color: '#f59e0b'
      },

      // Process operations
      {
        id: 'fork',
        name: 'fork',
        category: 'process',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: [],
        description: 'Create a child process',
        color: '#3b82f6'
      },
      {
        id: 'execve',
        name: 'execve',
        category: 'process',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: ['filename', 'argv', 'envp'],
        description: 'Execute program',
        color: '#3b82f6'
      },
      {
        id: 'exit',
        name: 'exit',
        category: 'process',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: ['status'],
        description: 'Terminate process',
        color: '#3b82f6'
      },
      {
        id: 'wait4',
        name: 'wait4',
        category: 'process',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: ['pid', 'status', 'options', 'rusage'],
        description: 'Wait for process termination',
        color: '#3b82f6'
      },
      {
        id: 'getpid',
        name: 'getpid',
        category: 'process',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: [],
        description: 'Get process ID',
        color: '#3b82f6'
      },
      {
        id: 'getppid',
        name: 'getppid',
        category: 'process',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: [],
        description: 'Get parent process ID',
        color: '#3b82f6'
      },

      // Network operations
      {
        id: 'socket',
        name: 'socket',
        category: 'network',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: ['domain', 'type', 'protocol'],
        description: 'Create socket',
        color: '#10b981'
      },
      {
        id: 'bind',
        name: 'bind',
        category: 'network',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: ['sockfd', 'addr', 'addrlen'],
        description: 'Bind socket to address',
        color: '#10b981'
      },
      {
        id: 'listen',
        name: 'listen',
        category: 'network',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: ['sockfd', 'backlog'],
        description: 'Listen on socket',
        color: '#10b981'
      },
      {
        id: 'accept',
        name: 'accept',
        category: 'network',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: ['sockfd', 'addr', 'addrlen'],
        description: 'Accept connection',
        color: '#10b981'
      },
      {
        id: 'connect',
        name: 'connect',
        category: 'network',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: ['sockfd', 'addr', 'addrlen'],
        description: 'Connect to remote host',
        color: '#10b981'
      },
      {
        id: 'send',
        name: 'send',
        category: 'network',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: ['sockfd', 'buf', 'len', 'flags'],
        description: 'Send data',
        color: '#10b981'
      },
      {
        id: 'recv',
        name: 'recv',
        category: 'network',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: ['sockfd', 'buf', 'len', 'flags'],
        description: 'Receive data',
        color: '#10b981'
      },

      // Synchronization
      {
        id: 'pthread_mutex_lock',
        name: 'pthread_mutex_lock',
        category: 'sync',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: ['mutex'],
        description: 'Lock mutex',
        color: '#8b5cf6'
      },
      {
        id: 'pthread_mutex_unlock',
        name: 'pthread_mutex_unlock',
        category: 'sync',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: ['mutex'],
        description: 'Unlock mutex',
        color: '#8b5cf6'
      },
      {
        id: 'pthread_cond_wait',
        name: 'pthread_cond_wait',
        category: 'sync',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: ['cond', 'mutex'],
        description: 'Wait on condition',
        color: '#8b5cf6'
      },
      {
        id: 'pthread_cond_signal',
        name: 'pthread_cond_signal',
        category: 'sync',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: ['cond'],
        description: 'Signal condition',
        color: '#8b5cf6'
      },

      // Device operations
      {
        id: 'ioctl',
        name: 'ioctl',
        category: 'device',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: ['fd', 'request', '...'],
        description: 'Device control',
        color: '#06b6d4'
      },
      {
        id: 'poll',
        name: 'poll',
        category: 'device',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: ['fds', 'nfds', 'timeout'],
        description: 'Poll for I/O events',
        color: '#06b6d4'
      },
      {
        id: 'epoll_wait',
        name: 'epoll_wait',
        category: 'device',
        frequency: 0,
        averageTime: 0,
        successRate: 0,
        parameters: ['epfd', 'events', 'maxevents', 'timeout'],
        description: 'Wait for I/O events',
        color: '#06b6d4'
      }
    ];

    return calls;
  };

  // Generate system call traces
  const generateSystemCallTraces = (): SystemCallTrace[] => {
    const traces: SystemCallTrace[] = [];
    const processes = ['firefox', 'chrome', 'code', 'terminal', 'node', 'python', 'nginx', 'mysql'];
    const callNames = ['open', 'read', 'write', 'close', 'socket', 'bind', 'connect', 'send', 'recv', 'mmap', 'fork', 'execve'];

    for (let i = 0; i < 100; i++) {
      const callName = callNames[Math.floor(Math.random() * callNames.length)];
      const processName = processes[Math.floor(Math.random() * processes.length)];
      const pid = 1000 + Math.floor(Math.random() * 1000);
      const duration = Math.random() * 10000; // 0-10ms
      const results: Array<SystemCallTrace['result']> = ['success', 'success', 'success', 'error', 'timeout'];
      const result = results[Math.floor(Math.random() * results.length)];

      traces.push({
        id: `trace-${i}`,
        timestamp: Date.now() - Math.random() * 60000,
        pid,
        processName,
        callName,
        duration,
        result,
        parameters: {
          fd: Math.floor(Math.random() * 100),
          count: Math.floor(Math.random() * 4096),
          timeout: Math.floor(Math.random() * 5000)
        },
        returnValue: result === 'success' ? 0 : -1,
        userStack: [`${processName}`, `libc.so.6`, `kernel_call`],
        kernelStack: [`sys_${callName}`, `do_${callName}`, `entry_${callName}`]
      });
    }

    return traces.sort((a, b) => a.timestamp - b.timestamp);
  };

  // Initialize system calls and traces
  useEffect(() => {
    setSystemCalls(generateSystemCalls());
    setCallTraces(generateSystemCallTraces());
  }, []);

  // Update call statistics and traces in real-time
  useEffect(() => {
    if (!realTimeData) return;

    const interval = setInterval(() => {
      // Generate new traces
      const newTraces: SystemCallTrace[] = [];
      const processes = ['firefox', 'chrome', 'code', 'terminal', 'node', 'python'];
      const callNames = systemCalls.map(call => call.name);
      
      for (let i = 0; i < 3; i++) {
        const callName = callNames[Math.floor(Math.random() * callNames.length)];
        const processName = processes[Math.floor(Math.random() * processes.length)];
        const pid = 1000 + Math.floor(Math.random() * 1000);
        const duration = Math.random() * 5000;
        const results: Array<SystemCallTrace['result']> = ['success', 'success', 'success', 'error'];
        const result = results[Math.floor(Math.random() * results.length)];

        newTraces.push({
          id: `trace-${Date.now()}-${i}`,
          timestamp: Date.now(),
          pid,
          processName,
          callName,
          duration,
          result,
          parameters: {
            fd: Math.floor(Math.random() * 100),
            count: Math.floor(Math.random() * 4096)
          },
          returnValue: result === 'success' ? 0 : -1,
          userStack: [`${processName}`, `libc.so.6`, `kernel_call`],
          kernelStack: [`sys_${callName}`, `do_${callName}`, `entry_${callName}`]
        });
      }

      setCallTraces(prev => {
        const updated = [...prev, ...newTraces];
        // Keep only last 200 traces
        return updated.slice(-200);
      });

      // Update call statistics
      setSystemCalls(prev => {
        const now = Date.now();
        const recentTraces = callTraces.filter(t => now - t.timestamp < 60000);
        
        return prev.map(call => {
          const callTraces = recentTraces.filter(t => t.callName === call.name);
          const totalCalls = callTraces.length;
          const totalTime = callTraces.reduce((sum, t) => sum + t.duration, 0);
          const successfulCalls = callTraces.filter(t => t.result === 'success').length;
          
          return {
            ...call,
            frequency: totalCalls,
            averageTime: totalCalls > 0 ? totalTime / totalCalls : 0,
            successRate: totalCalls > 0 ? (successfulCalls / totalCalls) * 100 : 100
          };
        });
      });

      // Update overall statistics
      const now = Date.now();
      const lastMinuteTraces = callTraces.filter(t => now - t.timestamp < 60000);
      const totalErrors = lastMinuteTraces.filter(t => t.result === 'error' || t.result === 'timeout').length;
      
      setStats({
        totalCalls: callTraces.length,
        callsPerSecond: lastMinuteTraces.length / 60,
        averageLatency: lastMinuteTraces.length > 0 ? 
          lastMinuteTraces.reduce((sum, t) => sum + t.duration, 0) / lastMinuteTraces.length : 0,
        errorRate: lastMinuteTraces.length > 0 ? (totalErrors / lastMinuteTraces.length) * 100 : 0,
        topCaller: '',
        topCall: ''
      });

    }, 1000);

    return () => clearInterval(interval);
  }, [realTimeData, systemCalls, callTraces]);

  // Update flow visualization data
  useEffect(() => {
    const filteredCalls = filterCategory === 'all' 
      ? systemCalls 
      : systemCalls.filter(call => call.category === filterCategory);

    const nodes = filteredCalls.map(call => ({
      id: call.name,
      name: call.name,
      category: call.category,
      frequency: call.frequency,
      color: call.color,
      size: Math.max(10, Math.min(50, 10 + call.frequency / 10))
    }));

    const links: any[] = [];
    // Create flow relationships between common call sequences
    const commonSequences = [
      ['open', 'read', 'close'],
      ['socket', 'bind', 'listen', 'accept'],
      ['connect', 'send', 'recv'],
      ['mmap', 'munmap'],
      ['fork', 'execve', 'exit']
    ];

    commonSequences.forEach(sequence => {
      for (let i = 0; i < sequence.length - 1; i++) {
        if (nodes.some(n => n.id === sequence[i]) && nodes.some(n => n.id === sequence[i + 1])) {
          links.push({
            source: sequence[i],
            target: sequence[i + 1],
            color: '#6b7280',
            width: 2
          });
        }
      }
    });

    setFlowData({ nodes, links });
  }, [systemCalls, filterCategory]);

  // Render flow visualization
  useEffect(() => {
    if (!flowRef.current || flowData.nodes.length === 0) return;

    const svg = d3.select(flowRef.current);
    const width = 1000;
    const height = 400;

    svg.selectAll('*').remove();

    const g = svg.append('g')
      .attr('transform', 'translate(50,50)');

    // Create force simulation
    const simulation = d3.forceSimulation(flowData.nodes)
      .force('link', d3.forceLink(flowData.links).id((d: any) => d.id).distance(100))
      .force('charge', d3.forceManyBody().strength(-300))
      .force('center', d3.forceCenter(width / 2 - 50, height / 2 - 50))
      .force('collision', d3.forceCollide().radius((d: any) => d.size));

    // Create links
    const links = g.selectAll('.flow-link')
      .data(flowData.links)
      .enter()
      .append('line')
      .attr('class', 'flow-link')
      .attr('stroke', (d: any) => d.color)
      .attr('stroke-width', (d: any) => d.width)
      .attr('marker-end', 'url(#arrow)');

    // Create nodes
    const nodes = g.selectAll('.flow-node')
      .data(flowData.nodes)
      .enter()
      .append('g')
      .attr('class', 'flow-node')
      .style('cursor', 'pointer')
      .call(d3.drag<any, any>()
        .on('start', (event, d) => {
          if (!event.active) simulation.alphaTarget(0.3).restart();
          d.fx = d.x;
          d.fy = d.y;
        })
        .on('drag', (event, d) => {
          d.fx = event.x;
          d.fy = event.y;
        })
        .on('end', (event, d) => {
          if (!event.active) simulation.alphaTarget(0);
          d.fx = null;
          d.fy = null;
        })
      );

    // Add node circles
    nodes.append('circle')
      .attr('r', (d: any) => d.size)
      .attr('fill', (d: any) => d.color)
      .attr('stroke', '#1f2937')
      .attr('stroke-width', 2)
      .on('click', (event, d) => {
        const call = systemCalls.find(c => c.name === d.id);
        setSelectedCall(call || null);
      });

    // Add node labels
    nodes.append('text')
      .attr('dy', '0.35em')
      .attr('text-anchor', 'middle')
      .style('fill', '#f9fafb')
      .style('font-size', '10px')
      .style('pointer-events', 'none')
      .text((d: any) => d.name);

    // Add frequency labels
    nodes.append('text')
      .attr('dy', '1.2em')
      .attr('text-anchor', 'middle')
      .style('fill', '#9ca3af')
      .style('font-size', '8px')
      .style('pointer-events', 'none')
      .text((d: any) => `${d.frequency}/min`);

    // Update positions on tick
    simulation.on('tick', () => {
      links
        .attr('x1', (d: any) => d.source.x)
        .attr('y1', (d: any) => d.source.y)
        .attr('x2', (d: any) => d.target.x)
        .attr('y2', (d: any) => d.target.y);

      nodes
        .attr('transform', (d: any) => `translate(${d.x},${d.y})`);
    });

    // Add arrow marker
    svg.append('defs')
      .append('marker')
      .attr('id', 'arrow')
      .attr('viewBox', '0 -5 10 10')
      .attr('refX', 15)
      .attr('refY', 0)
      .attr('markerWidth', 6)
      .attr('markerHeight', 6)
      .attr('orient', 'auto')
      .append('path')
      .attr('d', 'M0,-5L10,0L0,5')
      .attr('fill', '#6b7280');

  }, [flowData, systemCalls]);

  const getCategoryColor = (category: string) => {
    const colors = {
      file: '#ef4444',
      process: '#3b82f6',
      network: '#10b981',
      memory: '#f59e0b',
      device: '#06b6d4',
      sync: '#8b5cf6'
    };
    return colors[category as keyof typeof colors] || '#6b7280';
  };

  const filteredCalls = filterCategory === 'all' 
    ? systemCalls 
    : systemCalls.filter(call => call.category === filterCategory);

  return (
    <div className="space-y-4">
      {/* Statistics */}
      <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
        <Card className="bg-gray-900 border-gray-700">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm flex items-center">
              <Activity className="h-4 w-4 mr-2" />
              Total Calls
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.totalCalls}</div>
            <div className="text-xs text-gray-400">All time</div>
          </CardContent>
        </Card>

        <Card className="bg-gray-900 border-gray-700">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm flex items-center">
              <Zap className="h-4 w-4 mr-2" />
              Calls/sec
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.callsPerSecond.toFixed(1)}</div>
            <div className="text-xs text-gray-400">Current rate</div>
          </CardContent>
        </Card>

        <Card className="bg-gray-900 border-gray-700">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm flex items-center">
              <Clock className="h-4 w-4 mr-2" />
              Avg Latency
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{(stats.averageLatency / 1000).toFixed(2)}ms</div>
            <div className="text-xs text-gray-400">Response time</div>
          </CardContent>
        </Card>

        <Card className="bg-gray-900 border-gray-700">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm flex items-center">
              <GitBranch className="h-4 w-4 mr-2" />
              Error Rate
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.errorRate.toFixed(1)}%</div>
            <Progress value={stats.errorRate} className="mt-2" />
          </CardContent>
        </Card>

        <Card className="bg-gray-900 border-gray-700">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm flex items-center">
              <ArrowRight className="h-4 w-4 mr-2" />
              Success Rate
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{(100 - stats.errorRate).toFixed(1)}%</div>
            <Progress value={100 - stats.errorRate} className="mt-2" />
          </CardContent>
        </Card>
      </div>

      {/* Controls */}
      <Card className="bg-gray-900 border-gray-700">
        <CardHeader>
          <CardTitle className="flex items-center">
            <GitBranch className="h-5 w-5 mr-2" />
            System Call Flow Controls
          </CardTitle>
          <CardDescription>Filter and analyze system call patterns</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-4 items-center">
            <div className="flex items-center space-x-2">
              <Filter className="h-4 w-4" />
              <select
                value={filterCategory}
                onChange={(e) => setFilterCategory(e.target.value)}
                className="bg-gray-800 border border-gray-600 rounded px-3 py-1 text-sm"
              >
                <option value="all">All Categories</option>
                <option value="file">File Operations</option>
                <option value="process">Process Management</option>
                <option value="network">Network Operations</option>
                <option value="memory">Memory Management</option>
                <option value="device">Device I/O</option>
                <option value="sync">Synchronization</option>
              </select>
            </div>
            <Badge variant="outline">
              {filteredCalls.length} system calls
            </Badge>
          </div>
        </CardContent>
      </Card>

      {/* System Call Flow Graph */}
      <Card className="bg-gray-900 border-gray-700">
        <CardHeader>
          <CardTitle>System Call Flow Graph</CardTitle>
          <CardDescription>Interactive visualization of system call relationships and patterns</CardDescription>
        </CardHeader>
        <CardContent>
          <svg
            ref={flowRef}
            width={1000}
            height={400}
            className="border border-gray-600 rounded"
            style={{ background: '#1f2937' }}
          />
        </CardContent>
      </Card>

      {/* System Calls Table and Recent Traces */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {/* System Calls Table */}
        <Card className="bg-gray-900 border-gray-700">
          <CardHeader>
            <CardTitle>System Call Statistics</CardTitle>
            <CardDescription>Frequency and performance metrics</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="max-h-96 overflow-y-auto">
              <table className="w-full text-sm">
                <thead className="sticky top-0 bg-gray-800">
                  <tr className="border-b border-gray-700">
                    <th className="text-left p-2">System Call</th>
                    <th className="text-left p-2">Category</th>
                    <th className="text-left p-2">Freq/min</th>
                    <th className="text-left p-2">Avg Time</th>
                    <th className="text-left p-2">Success</th>
                  </tr>
                </thead>
                <tbody>
                  {filteredCalls
                    .sort((a, b) => b.frequency - a.frequency)
                    .slice(0, 20)
                    .map(call => (
                      <tr 
                        key={call.id}
                        className="border-b border-gray-700 hover:bg-gray-800 cursor-pointer"
                        onClick={() => setSelectedCall(call)}
                      >
                        <td className="p-2">
                          <div>
                            <div className="font-medium">{call.name}</div>
                            <div className="text-xs text-gray-400">{call.description}</div>
                          </div>
                        </td>
                        <td className="p-2">
                          <Badge 
                            style={{ backgroundColor: getCategoryColor(call.category), color: 'white' }}
                            className="text-xs"
                          >
                            {call.category}
                          </Badge>
                        </td>
                        <td className="p-2 font-mono">{call.frequency}</td>
                        <td className="p-2 font-mono">{(call.averageTime / 1000).toFixed(2)}ms</td>
                        <td className="p-2">
                          <div className="flex items-center space-x-2">
                            <span className="text-xs">{call.successRate.toFixed(1)}%</span>
                            <div className="w-12 bg-gray-700 rounded-full h-2">
                              <div
                                className="bg-green-500 h-2 rounded-full"
                                style={{ width: `${call.successRate}%` }}
                              />
                            </div>
                          </div>
                        </td>
                      </tr>
                    ))}
                </tbody>
              </table>
            </div>
          </CardContent>
        </Card>

        {/* Recent System Call Traces */}
        <Card className="bg-gray-900 border-gray-700">
          <CardHeader>
            <CardTitle>Recent System Call Traces</CardTitle>
            <CardDescription>Real-time call execution details</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="max-h-96 overflow-y-auto">
              <div className="space-y-2">
                {callTraces.slice(-20).reverse().map(trace => (
                  <div 
                    key={trace.id}
                    className="p-3 bg-gray-800 rounded hover:bg-gray-700 cursor-pointer"
                    onClick={() => setSelectedTrace(trace)}
                  >
                    <div className="flex items-center justify-between mb-1">
                      <span className="font-medium">{trace.callName}</span>
                      <Badge 
                        className={
                          trace.result === 'success' ? 'bg-green-500' :
                          trace.result === 'error' ? 'bg-red-500' : 'bg-yellow-500'
                        }
                      >
                        {trace.result}
                      </Badge>
                    </div>
                    <div className="text-xs text-gray-400">
                      {trace.processName} (PID: {trace.pid}) • {(trace.duration / 1000).toFixed(2)}ms
                    </div>
                    <div className="text-xs text-gray-500">
                      {new Date(trace.timestamp).toLocaleTimeString()}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Selected Call/Trace Details */}
      {(selectedCall || selectedTrace) && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
          {/* System Call Details */}
          {selectedCall && (
            <Card className="bg-gray-900 border-gray-700">
              <CardHeader>
                <CardTitle className="flex items-center justify-between">
                  {selectedCall.name}
                  <Badge style={{ backgroundColor: getCategoryColor(selectedCall.category), color: 'white' }}>
                    {selectedCall.category}
                  </Badge>
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div>
                    <div className="text-sm text-gray-400">Description</div>
                    <div>{selectedCall.description}</div>
                  </div>

                  <div>
                    <div className="text-sm text-gray-400">Parameters</div>
                    <div className="flex flex-wrap gap-1 mt-1">
                      {selectedCall.parameters.map(param => (
                        <code key={param} className="bg-gray-800 px-2 py-1 rounded text-xs">
                          {param}
                        </code>
                      ))}
                    </div>
                  </div>

                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <div className="text-sm text-gray-400">Frequency</div>
                      <div className="text-xl font-bold">{selectedCall.frequency}</div>
                      <div className="text-xs text-gray-400">calls/minute</div>
                    </div>
                    <div>
                      <div className="text-sm text-gray-400">Avg Duration</div>
                      <div className="text-xl font-bold">{(selectedCall.averageTime / 1000).toFixed(2)}ms</div>
                      <div className="text-xs text-gray-400">execution time</div>
                    </div>
                  </div>

                  <div>
                    <div className="text-sm text-gray-400">Success Rate</div>
                    <div className="flex items-center space-x-2 mt-1">
                      <span className="text-xl font-bold">{selectedCall.successRate.toFixed(1)}%</span>
                      <div className="flex-1 bg-gray-700 rounded-full h-3">
                        <div
                          className="bg-green-500 h-3 rounded-full"
                          style={{ width: `${selectedCall.successRate}%` }}
                        />
                      </div>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          )}

          {/* System Call Trace Details */}
          {selectedTrace && (
            <Card className="bg-gray-900 border-gray-700">
              <CardHeader>
                <CardTitle className="flex items-center justify-between">
                  Trace: {selectedTrace.callName}
                  <Badge 
                    className={
                      selectedTrace.result === 'success' ? 'bg-green-500' :
                      selectedTrace.result === 'error' ? 'bg-red-500' : 'bg-yellow-500'
                    }
                  >
                    {selectedTrace.result}
                  </Badge>
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <div className="text-sm text-gray-400">Process</div>
                      <div className="font-medium">{selectedTrace.processName}</div>
                    </div>
                    <div>
                      <div className="text-sm text-gray-400">PID</div>
                      <div className="font-mono">{selectedTrace.pid}</div>
                    </div>
                  </div>

                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <div className="text-sm text-gray-400">Duration</div>
                      <div className="font-mono">{(selectedTrace.duration / 1000).toFixed(2)} ms</div>
                    </div>
                    <div>
                      <div className="text-sm text-gray-400">Return Value</div>
                      <div className="font-mono">{selectedTrace.returnValue}</div>
                    </div>
                  </div>

                  <div>
                    <div className="text-sm text-gray-400">Parameters</div>
                    <div className="bg-gray-800 p-2 rounded mt-1">
                      <pre className="text-xs">{JSON.stringify(selectedTrace.parameters, null, 2)}</pre>
                    </div>
                  </div>

                  <div>
                    <div className="text-sm text-gray-400">User Stack</div>
                    <div className="bg-gray-800 p-2 rounded mt-1">
                      <div className="text-xs space-y-1">
                        {selectedTrace.userStack.map((frame, i) => (
                          <div key={i} className="font-mono">{frame}</div>
                        ))}
                      </div>
                    </div>
                  </div>

                  <div>
                    <div className="text-sm text-gray-400">Kernel Stack</div>
                    <div className="bg-gray-800 p-2 rounded mt-1">
                      <div className="text-xs space-y-1">
                        {selectedTrace.kernelStack.map((frame, i) => (
                          <div key={i} className="font-mono">{frame}</div>
                        ))}
                      </div>
                    </div>
                  </div>

                  <div>
                    <div className="text-sm text-gray-400">Timestamp</div>
                    <div>{new Date(selectedTrace.timestamp).toLocaleString()}</div>
                  </div>
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      )}
    </div>
  );
};

export default SystemCallFlow;