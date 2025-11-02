import React, { useState, useEffect } from 'react';
import { ResizablePanelGroup, ResizablePanel, ResizableHandle } from '@/components/ui/resizable';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { 
  Activity, 
  Cpu, 
  Database, 
  Network, 
  FileTree, 
  GitBranch, 
  Layers, 
  Zap,
  Monitor,
  AlertTriangle,
  CheckCircle
} from 'lucide-react';

// Import visualization components
import MemoryMapVisualization from './components/visualizations/MemoryMapVisualization';
import ProcessTreeVisualization from './components/visualizations/ProcessTreeVisualization';
import CPUSchedulerVisualization from './components/visualizations/CPUSchedulerVisualization';
import FileSystemVisualization from './components/visualizations/FileSystemVisualization';
import NetworkStackVisualization from './components/visualizations/NetworkStackVisualization';
import KernelModuleGraph from './components/visualizations/KernelModuleGraph';
import SystemCallFlow from './components/visualizations/SystemCallFlow';
import PerformanceOverlay from './components/visualizations/PerformanceOverlay';

import './App.css';

interface SystemMetrics {
  cpuUsage: number;
  memoryUsage: number;
  networkActivity: number;
  activeProcesses: number;
  systemCallsPerSecond: number;
  diskIO: number;
}

const App: React.FC = () => {
  const [activeTab, setActiveTab] = useState('overview');
  const [systemMetrics, setSystemMetrics] = useState<SystemMetrics>({
    cpuUsage: 0,
    memoryUsage: 0,
    networkActivity: 0,
    activeProcesses: 0,
    systemCallsPerSecond: 0,
    diskIO: 0,
  });

  const [realTimeData, setRealTimeData] = useState(true);

  // Simulate real-time system metrics updates
  useEffect(() => {
    if (!realTimeData) return;

    const interval = setInterval(() => {
      setSystemMetrics(prev => ({
        cpuUsage: Math.max(0, Math.min(100, prev.cpuUsage + (Math.random() - 0.5) * 10)),
        memoryUsage: Math.max(0, Math.min(100, prev.memoryUsage + (Math.random() - 0.5) * 5)),
        networkActivity: Math.max(0, prev.networkActivity + (Math.random() - 0.5) * 20),
        activeProcesses: Math.max(1, prev.activeProcesses + Math.floor((Math.random() - 0.5) * 5)),
        systemCallsPerSecond: Math.max(0, prev.systemCallsPerSecond + (Math.random() - 0.5) * 100),
        diskIO: Math.max(0, prev.diskIO + (Math.random() - 0.5) * 50),
      }));
    }, 1000);

    return () => clearInterval(interval);
  }, [realTimeData]);

  // Initialize with realistic values
  useEffect(() => {
    setSystemMetrics({
      cpuUsage: 15 + Math.random() * 30,
      memoryUsage: 40 + Math.random() * 20,
      networkActivity: Math.random() * 100,
      activeProcesses: 50 + Math.floor(Math.random() * 20),
      systemCallsPerSecond: 1000 + Math.random() * 2000,
      diskIO: Math.random() * 100,
    });
  }, []);

  const getStatusColor = (value: number, thresholds: { warning: number; critical: number }) => {
    if (value >= thresholds.critical) return 'destructive';
    if (value >= thresholds.warning) return 'secondary';
    return 'default';
  };

  const getStatusIcon = (value: number, thresholds: { warning: number; critical: number }) => {
    if (value >= thresholds.critical) return <AlertTriangle className="h-4 w-4" />;
    return <CheckCircle className="h-4 w-4" />;
  };

  return (
    <div className="min-h-screen bg-gray-950 text-white">
      {/* Header */}
      <div className="border-b border-gray-800 p-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-4">
            <Monitor className="h-8 w-8 text-blue-400" />
            <div>
              <h1 className="text-2xl font-bold text-white">MultiOS Kernel Internals Visualization System</h1>
              <p className="text-gray-400">Real-time kernel monitoring and analysis</p>
            </div>
          </div>
          <div className="flex items-center space-x-2">
            <Badge variant={realTimeData ? "default" : "secondary"}>
              <Activity className="h-3 w-3 mr-1" />
              Real-time {realTimeData ? 'ON' : 'OFF'}
            </Badge>
            <button
              onClick={() => setRealTimeData(!realTimeData)}
              className="px-3 py-1 bg-blue-600 hover:bg-blue-700 rounded-md text-sm"
            >
              {realTimeData ? 'Pause' : 'Resume'}
            </button>
          </div>
        </div>
      </div>

      {/* Performance Metrics Overview */}
      <div className="p-4 border-b border-gray-800">
        <div className="grid grid-cols-2 md:grid-cols-6 gap-4">
          <Card className="bg-gray-900 border-gray-700">
            <CardHeader className="pb-2">
              <CardTitle className="text-sm flex items-center">
                <Cpu className="h-4 w-4 mr-2" />
                CPU Usage
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex items-center justify-between">
                <span className="text-2xl font-bold">{systemMetrics.cpuUsage.toFixed(1)}%</span>
                {getStatusIcon(systemMetrics.cpuUsage, { warning: 70, critical: 90 })}
              </div>
              <Progress 
                value={systemMetrics.cpuUsage} 
                className="mt-2" 
                indicatorClassName={
                  systemMetrics.cpuUsage >= 90 ? 'bg-red-500' : 
                  systemMetrics.cpuUsage >= 70 ? 'bg-yellow-500' : 'bg-green-500'
                }
              />
            </CardContent>
          </Card>

          <Card className="bg-gray-900 border-gray-700">
            <CardHeader className="pb-2">
              <CardTitle className="text-sm flex items-center">
                <Database className="h-4 w-4 mr-2" />
                Memory
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex items-center justify-between">
                <span className="text-2xl font-bold">{systemMetrics.memoryUsage.toFixed(1)}%</span>
                {getStatusIcon(systemMetrics.memoryUsage, { warning: 80, critical: 95 })}
              </div>
              <Progress 
                value={systemMetrics.memoryUsage} 
                className="mt-2"
                indicatorClassName={
                  systemMetrics.memoryUsage >= 95 ? 'bg-red-500' : 
                  systemMetrics.memoryUsage >= 80 ? 'bg-yellow-500' : 'bg-green-500'
                }
              />
            </CardContent>
          </Card>

          <Card className="bg-gray-900 border-gray-700">
            <CardHeader className="pb-2">
              <CardTitle className="text-sm flex items-center">
                <Network className="h-4 w-4 mr-2" />
                Network
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex items-center justify-between">
                <span className="text-2xl font-bold">{systemMetrics.networkActivity.toFixed(0)}</span>
                <Activity className="h-4 w-4 text-blue-400" />
              </div>
              <Progress value={systemMetrics.networkActivity} className="mt-2" />
            </CardContent>
          </Card>

          <Card className="bg-gray-900 border-gray-700">
            <CardHeader className="pb-2">
              <CardTitle className="text-sm flex items-center">
                <FileTree className="h-4 w-4 mr-2" />
                Processes
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex items-center justify-between">
                <span className="text-2xl font-bold">{systemMetrics.activeProcesses}</span>
                <Zap className="h-4 w-4 text-yellow-400" />
              </div>
              <div className="text-xs text-gray-400 mt-1">Active</div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900 border-gray-700">
            <CardHeader className="pb-2">
              <CardTitle className="text-sm flex items-center">
                <GitBranch className="h-4 w-4 mr-2" />
                Syscalls/s
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex items-center justify-between">
                <span className="text-2xl font-bold">{(systemMetrics.systemCallsPerSecond / 1000).toFixed(1)}K</span>
                <Layers className="h-4 w-4 text-purple-400" />
              </div>
              <div className="text-xs text-gray-400 mt-1">Rate</div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900 border-gray-700">
            <CardHeader className="pb-2">
              <CardTitle className="text-sm flex items-center">
                <Activity className="h-4 w-4 mr-2" />
                Disk I/O
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex items-center justify-between">
                <span className="text-2xl font-bold">{systemMetrics.diskIO.toFixed(0)}</span>
                <CheckCircle className="h-4 w-4 text-green-400" />
              </div>
              <div className="text-xs text-gray-400 mt-1">MB/s</div>
            </CardContent>
          </Card>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 p-4">
        <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
          <TabsList className="grid w-full grid-cols-8 bg-gray-800">
            <TabsTrigger value="overview" className="text-xs">Overview</TabsTrigger>
            <TabsTrigger value="memory" className="text-xs">Memory</TabsTrigger>
            <TabsTrigger value="processes" className="text-xs">Processes</TabsTrigger>
            <TabsTrigger value="cpu" className="text-xs">CPU</TabsTrigger>
            <TabsTrigger value="filesystem" className="text-xs">File System</TabsTrigger>
            <TabsTrigger value="network" className="text-xs">Network</TabsTrigger>
            <TabsTrigger value="modules" className="text-xs">Modules</TabsTrigger>
            <TabsTrigger value="syscalls" className="text-xs">Syscalls</TabsTrigger>
          </TabsList>

          <TabsContent value="overview" className="mt-4">
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              <Card className="bg-gray-900 border-gray-700">
                <CardHeader>
                  <CardTitle>System Call Flow</CardTitle>
                  <CardDescription>Real-time system call visualization</CardDescription>
                </CardHeader>
                <CardContent>
                  <SystemCallFlow realTimeData={realTimeData} />
                </CardContent>
              </Card>
              
              <Card className="bg-gray-900 border-gray-700">
                <CardHeader>
                  <CardTitle>Kernel Module Dependencies</CardTitle>
                  <CardDescription>Module dependency graph</CardDescription>
                </CardHeader>
                <CardContent>
                  <KernelModuleGraph realTimeData={realTimeData} />
                </CardContent>
              </Card>
            </div>
          </TabsContent>

          <TabsContent value="memory" className="mt-4">
            <MemoryMapVisualization realTimeData={realTimeData} />
          </TabsContent>

          <TabsContent value="processes" className="mt-4">
            <ProcessTreeVisualization realTimeData={realTimeData} />
          </TabsContent>

          <TabsContent value="cpu" className="mt-4">
            <CPUSchedulerVisualization realTimeData={realTimeData} />
          </TabsContent>

          <TabsContent value="filesystem" className="mt-4">
            <FileSystemVisualization realTimeData={realTimeData} />
          </TabsContent>

          <TabsContent value="network" className="mt-4">
            <NetworkStackVisualization realTimeData={realTimeData} />
          </TabsContent>

          <TabsContent value="modules" className="mt-4">
            <KernelModuleGraph realTimeData={realTimeData} />
          </TabsContent>

          <TabsContent value="syscalls" className="mt-4">
            <SystemCallFlow realTimeData={realTimeData} />
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
};

export default App;
