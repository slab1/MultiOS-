import React, { useState, useEffect } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, BarChart, Bar, PieChart, Pie, Cell } from 'recharts';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Progress } from '@/components/ui/progress';
import { 
  Activity, 
  Cpu, 
  HardDrive, 
  MemoryStick, 
  Network, 
  Users, 
  AlertTriangle,
  RefreshCw,
  Download,
  Settings,
  Clock,
  TrendingUp,
  TrendingDown
} from 'lucide-react';

// WebSocket connection for real-time updates
const useWebSocket = (url: string) => {
  const [data, setData] = useState<any>(null);
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    const ws = new WebSocket(url);

    ws.onopen = () => {
      setConnected(true);
      ws.send(JSON.stringify({ event: 'request_metrics' }));
    };

    ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      if (message.event === 'metrics_update') {
        setData(message.data);
      }
    };

    ws.onclose = () => setConnected(false);
    ws.onerror = () => setConnected(false);

    return () => ws.close();
  }, [url]);

  return { data, connected };
};

const SystemOverview: React.FC = () => {
  const { data, connected } = useWebSocket('ws://localhost:5000/socket.io/');
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());

  useEffect(() => {
    if (data) {
      setLastUpdate(new Date());
    }
  }, [data]);

  const formatUptime = (seconds: number) => {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${days}d ${hours}h ${minutes}m`;
  };

  const getStatusColor = (value: number, type: 'cpu' | 'memory' | 'disk') => {
    const thresholds = {
      cpu: { warning: 80, critical: 95 },
      memory: { warning: 85, critical: 95 },
      disk: { warning: 85, critical: 95 }
    };
    
    const threshold = thresholds[type];
    if (value >= threshold.critical) return 'bg-red-500';
    if (value >= threshold.warning) return 'bg-yellow-500';
    return 'bg-green-500';
  };

  if (!data) {
    return (
      <div className="flex items-center justify-center h-64">
        <RefreshCw className="h-6 w-6 animate-spin mr-2" />
        <span>Loading system metrics...</span>
      </div>
    );
  }

  const cpu = data.cpu || {};
  const memory = data.memory || {};
  const disk = data.disk || {};
  const network = data.network || {};
  const kernel = data.kernel || {};

  return (
    <div className="space-y-6">
      {/* Connection Status */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-2">
          <div className={`w-3 h-3 rounded-full ${connected ? 'bg-green-500' : 'bg-red-500'}`} />
          <span className="text-sm text-gray-600">
            {connected ? 'Connected' : 'Disconnected'}
          </span>
        </div>
        <div className="text-sm text-gray-500">
          Last update: {lastUpdate.toLocaleTimeString()}
        </div>
      </div>

      {/* System Overview Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">CPU Usage</CardTitle>
            <Cpu className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{cpu.cpu_percent?.toFixed(1) || 0}%</div>
            <Progress 
              value={cpu.cpu_percent || 0} 
              className={`mt-2 h-2 ${getStatusColor(cpu.cpu_percent || 0, 'cpu')}`}
            />
            <p className="text-xs text-muted-foreground mt-2">
              {cpu.cpu_count} cores, {cpu.cpu_count_logical} logical
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Memory Usage</CardTitle>
            <MemoryStick className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{memory.virtual_memory?.percent?.toFixed(1) || 0}%</div>
            <Progress 
              value={memory.virtual_memory?.percent || 0} 
              className={`mt-2 h-2 ${getStatusColor(memory.virtual_memory?.percent || 0, 'memory')}`}
            />
            <p className="text-xs text-muted-foreground mt-2">
              {(memory.virtual_memory?.used / 1024**3).toFixed(1)} GB of {(memory.virtual_memory?.total / 1024**3).toFixed(1)} GB
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Disk Usage</CardTitle>
            <HardDrive className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {Object.keys(disk.disk_usage || {}).length > 0 
                ? Object.values(disk.disk_usage)[0]?.percent?.toFixed(1) || 0
                : 0}%
            </div>
            <Progress 
              value={Object.values(disk.disk_usage || {})[0]?.percent || 0} 
              className={`mt-2 h-2 ${getStatusColor(Object.values(disk.disk_usage || {})[0]?.percent || 0, 'disk')}`}
            />
            <p className="text-xs text-muted-foreground mt-2">
              {Object.keys(disk.disk_usage || {}).length} partitions
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Network</CardTitle>
            <Network className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {((network.network_io?.download_rate_mbps || 0) + (network.network_io?.upload_rate_mbps || 0)).toFixed(1)} MB/s
            </div>
            <div className="text-xs text-muted-foreground mt-2">
              ↓ {network.network_io?.download_rate_mbps?.toFixed(1) || 0} MB/s ↑ {network.network_io?.upload_rate_mbps?.toFixed(1) || 0} MB/s
            </div>
          </CardContent>
        </Card>
      </div>

      {/* System Information */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Activity className="h-5 w-5" />
            <span>System Information</span>
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <h4 className="font-medium text-sm text-gray-600">Uptime</h4>
              <p className="text-lg">{kernel.uptime_formatted || 'N/A'}</p>
            </div>
            <div>
              <h4 className="font-medium text-sm text-gray-600">Load Average</h4>
              <p className="text-lg">
                {cpu.load_avg ? `${cpu.load_avg['1min']?.toFixed(2)} / ${cpu.load_avg['5min']?.toFixed(2)} / ${cpu.load_avg['15min']?.toFixed(2)}` : 'N/A'}
              </p>
            </div>
            <div>
              <h4 className="font-medium text-sm text-gray-600">Active Users</h4>
              <p className="text-lg">{kernel.users || 0}</p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

const PerformanceCharts: React.FC = () => {
  const { data } = useWebSocket('ws://localhost:5000/socket.io/');
  const [historicalData, setHistoricalData] = useState<any[]>([]);

  useEffect(() => {
    // Fetch historical data on component mount
    const fetchHistoricalData = async () => {
      try {
        const response = await fetch('/api/metrics/history/system?hours=24');
        const result = await response.json();
        if (result.success) {
          setHistoricalData(result.data);
        }
      } catch (error) {
        console.error('Error fetching historical data:', error);
      }
    };

    fetchHistoricalData();
  }, []);

  // Process historical data for charts
  const cpuData = historicalData.map((item, index) => ({
    time: new Date(item.timestamp).toLocaleTimeString(),
    cpu: item.cpu_percent || 0,
    memory: typeof item.memory_percent === 'string' ? JSON.parse(item.memory_percent || '{}').percent || 0 : (item.memory_percent || 0),
    load: typeof item.load_avg === 'string' ? JSON.parse(item.load_avg || '{}').['1min'] || 0 : (JSON.parse(item.load_avg || '{}')['1min'] || 0)
  }));

  const processCpuData = (data?.processes?.top_cpu_processes || []).map((proc: any) => ({
    name: proc.name,
    cpu: proc.cpu_percent || 0,
    memory: proc.memory_percent || 0
  }));

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card>
          <CardHeader>
            <CardTitle>CPU Usage Over Time</CardTitle>
            <CardDescription>24-hour CPU usage trend</CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={300}>
              <LineChart data={cpuData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="time" />
                <YAxis domain={[0, 100]} />
                <Tooltip />
                <Line type="monotone" dataKey="cpu" stroke="#8884d8" strokeWidth={2} />
              </LineChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Memory Usage Over Time</CardTitle>
            <CardDescription>24-hour memory usage trend</CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={300}>
              <LineChart data={cpuData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="time" />
                <YAxis domain={[0, 100]} />
                <Tooltip />
                <Line type="monotone" dataKey="memory" stroke="#82ca9d" strokeWidth={2} />
              </LineChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card>
          <CardHeader>
            <CardTitle>Top CPU Consuming Processes</CardTitle>
            <CardDescription>Current top 10 processes by CPU usage</CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={300}>
              <BarChart data={processCpuData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="name" />
                <YAxis />
                <Tooltip />
                <Bar dataKey="cpu" fill="#8884d8" />
              </BarChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Load Average Trend</CardTitle>
            <CardDescription>System load average over 24 hours</CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={300}>
              <LineChart data={cpuData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="time" />
                <YAxis />
                <Tooltip />
                <Line type="monotone" dataKey="load" stroke="#ffc658" strokeWidth={2} />
              </LineChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>
      </div>
    </div>
  );
};

const ProcessMonitor: React.FC = () => {
  const { data } = useWebSocket('ws://localhost:5000/socket.io/');
  const [sortBy, setSortBy] = useState<'cpu' | 'memory'>('cpu');
  const [processes, setProcesses] = useState<any[]>([]);

  useEffect(() => {
    if (data?.processes) {
      if (sortBy === 'cpu') {
        setProcesses(data.processes.top_cpu_processes || []);
      } else {
        setProcesses(data.processes.top_memory_processes || []);
      }
    }
  }, [data, sortBy]);

  const handleSortChange = (newSort: 'cpu' | 'memory') => {
    setSortBy(newSort);
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>Process Monitor</CardTitle>
            <CardDescription>Real-time process performance monitoring</CardDescription>
          </div>
          <div className="flex space-x-2">
            <Button 
              variant={sortBy === 'cpu' ? 'default' : 'outline'} 
              size="sm"
              onClick={() => handleSortChange('cpu')}
            >
              Sort by CPU
            </Button>
            <Button 
              variant={sortBy === 'memory' ? 'default' : 'outline'} 
              size="sm"
              onClick={() => handleSortChange('memory')}
            >
              Sort by Memory
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <ScrollArea className="h-96">
          <div className="space-y-2">
            <div className="grid grid-cols-6 gap-2 text-sm font-medium border-b pb-2">
              <div>PID</div>
              <div>Name</div>
              <div>CPU %</div>
              <div>Memory %</div>
              <div>Status</div>
              <div>Threads</div>
            </div>
            {processes.slice(0, 20).map((proc, index) => (
              <div key={proc.pid} className="grid grid-cols-6 gap-2 text-sm py-1 hover:bg-gray-50 rounded">
                <div>{proc.pid}</div>
                <div className="truncate" title={proc.name}>{proc.name}</div>
                <div>
                  <Badge variant={proc.cpu_percent > 50 ? 'destructive' : 'secondary'}>
                    {proc.cpu_percent?.toFixed(1) || 0}%
                  </Badge>
                </div>
                <div>
                  <Badge variant={proc.memory_percent > 20 ? 'destructive' : 'secondary'}>
                    {proc.memory_percent?.toFixed(1) || 0}%
                  </Badge>
                </div>
                <div>
                  <Badge variant={proc.status === 'running' ? 'default' : 'secondary'}>
                    {proc.status || 'unknown'}
                  </Badge>
                </div>
                <div>{proc.num_threads || 0}</div>
              </div>
            ))}
          </div>
        </ScrollArea>
      </CardContent>
    </Card>
  );
};

const AlertCenter: React.FC = () => {
  const [alerts, setAlerts] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchAlerts = async () => {
      try {
        const response = await fetch('/api/alerts?hours=24&limit=50');
        const result = await response.json();
        if (result.success) {
          setAlerts(result.data);
        }
      } catch (error) {
        console.error('Error fetching alerts:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchAlerts();
    // Refresh alerts every 30 seconds
    const interval = setInterval(fetchAlerts, 30000);
    return () => clearInterval(interval);
  }, []);

  const handleAcknowledge = async (alertId: number) => {
    try {
      const response = await fetch(`/api/alerts/${alertId}/acknowledge`, { method: 'POST' });
      if (response.ok) {
        setAlerts(alerts.map(alert => 
          alert.id === alertId ? { ...alert, acknowledged: true } : alert
        ));
      }
    } catch (error) {
      console.error('Error acknowledging alert:', error);
    }
  };

  const getSeverityColor = (severity: string) => {
    switch (severity?.toLowerCase()) {
      case 'critical': return 'bg-red-100 text-red-800 border-red-200';
      case 'warning': return 'bg-yellow-100 text-yellow-800 border-yellow-200';
      case 'info': return 'bg-blue-100 text-blue-800 border-blue-200';
      default: return 'bg-gray-100 text-gray-800 border-gray-200';
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <RefreshCw className="h-6 w-6 animate-spin mr-2" />
        <span>Loading alerts...</span>
      </div>
    );
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="flex items-center space-x-2">
              <AlertTriangle className="h-5 w-5" />
              <span>Alert Center</span>
            </CardTitle>
            <CardDescription>System alerts and notifications</CardDescription>
          </div>
          <Badge variant="outline">{alerts.filter(a => !a.acknowledged).length} active</Badge>
        </div>
      </CardHeader>
      <CardContent>
        <ScrollArea className="h-96">
          {alerts.length === 0 ? (
            <div className="text-center text-gray-500 py-8">
              <AlertTriangle className="h-12 w-12 mx-auto mb-4 text-gray-300" />
              <p>No alerts found</p>
            </div>
          ) : (
            <div className="space-y-3">
              {alerts.map((alert) => (
                <Alert key={alert.id} className={`${getSeverityColor(alert.severity)} ${alert.acknowledged ? 'opacity-60' : ''}`}>
                  <AlertTriangle className="h-4 w-4" />
                  <div className="flex-1">
                    <div className="flex items-center justify-between mb-2">
                      <span className="font-medium">{alert.alert_type}</span>
                      <div className="flex items-center space-x-2">
                        <Badge variant="outline" className="text-xs">
                          {alert.severity}
                        </Badge>
                        {!alert.acknowledged && (
                          <Button 
                            size="sm" 
                            variant="outline"
                            onClick={() => handleAcknowledge(alert.id)}
                          >
                            Acknowledge
                          </Button>
                        )}
                      </div>
                    </div>
                    <AlertDescription>{alert.message}</AlertDescription>
                    <div className="text-xs text-gray-500 mt-1">
                      {new Date(alert.timestamp).toLocaleString()}
                    </div>
                  </div>
                </Alert>
              ))}
            </div>
          )}
        </ScrollArea>
      </CardContent>
    </Card>
  );
};

const NetworkMonitor: React.FC = () => {
  const { data } = useWebSocket('ws://localhost:5000/socket.io/');

  if (!data?.network) {
    return (
      <div className="flex items-center justify-center h-64">
        <RefreshCw className="h-6 w-6 animate-spin mr-2" />
        <span>Loading network data...</span>
      </div>
    );
  }

  const network = data.network;
  const interfaces = Object.entries(network.network_io?.interfaces || {});

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Card>
          <CardHeader>
            <CardTitle>Network Traffic</CardTitle>
            <CardDescription>Real-time network I/O</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <span>Download Rate</span>
                <span className="text-2xl font-bold">
                  {network.network_io?.download_rate_mbps?.toFixed(1) || 0} MB/s
                </span>
              </div>
              <div className="flex items-center justify-between">
                <span>Upload Rate</span>
                <span className="text-2xl font-bold">
                  {network.network_io?.upload_rate_mbps?.toFixed(1) || 0} MB/s
                </span>
              </div>
              <div className="flex items-center justify-between">
                <span>Total Downloaded</span>
                <span>{(network.network_io?.total_recv_mb || 0).toFixed(1)} MB</span>
              </div>
              <div className="flex items-center justify-between">
                <span>Total Uploaded</span>
                <span>{(network.network_io?.total_sent_mb || 0).toFixed(1)} MB</span>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Connection Statistics</CardTitle>
            <CardDescription>Active network connections</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <span>Total Connections</span>
                <span className="text-2xl font-bold">
                  {network.network_connections?.total || 0}
                </span>
              </div>
              {Object.entries(network.network_connections?.states || {}).map(([state, count]) => (
                <div key={state} className="flex items-center justify-between">
                  <span className="capitalize">{state.toLowerCase()}</span>
                  <Badge variant="outline">{count as number}</Badge>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Network Interfaces</CardTitle>
          <CardDescription>Network interface statistics</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {interfaces.map(([interface, stats]: [string, any]) => (
              <div key={interface} className="border rounded-lg p-4">
                <div className="flex items-center justify-between mb-2">
                  <h4 className="font-medium">{interface}</h4>
                  <Badge variant={stats.is_up ? 'default' : 'secondary'}>
                    {stats.is_up ? 'Up' : 'Down'}
                  </Badge>
                </div>
                <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                  <div>
                    <span className="text-gray-600">Bytes Sent</span>
                    <div className="font-medium">{(stats.bytes_sent / 1024 / 1024).toFixed(1)} MB</div>
                  </div>
                  <div>
                    <span className="text-gray-600">Bytes Received</span>
                    <div className="font-medium">{(stats.bytes_recv / 1024 / 1024).toFixed(1)} MB</div>
                  </div>
                  <div>
                    <span className="text-gray-600">Packets Sent</span>
                    <div className="font-medium">{stats.packets_sent?.toLocaleString()}</div>
                  </div>
                  <div>
                    <span className="text-gray-600">Packets Received</span>
                    <div className="font-medium">{stats.packets_recv?.toLocaleString()}</div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

const Dashboard: React.FC = () => {
  const [activeTab, setActiveTab] = useState('overview');
  const [isMonitoring, setIsMonitoring] = useState(false);

  const startMonitoring = async () => {
    try {
      const response = await fetch('/api/monitoring/start', { method: 'POST' });
      if (response.ok) {
        setIsMonitoring(true);
      }
    } catch (error) {
      console.error('Error starting monitoring:', error);
    }
  };

  const stopMonitoring = async () => {
    try {
      const response = await fetch('/api/monitoring/stop', { method: 'POST' });
      if (response.ok) {
        setIsMonitoring(false);
      }
    } catch (error) {
      console.error('Error stopping monitoring:', error);
    }
  };

  const exportReport = async (format: 'pdf' | 'html' | 'csv') => {
    try {
      const response = await fetch(`/api/export/report?format=${format}`);
      if (response.ok) {
        const blob = await response.blob();
        const url = window.URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `performance_report.${format}`;
        document.body.appendChild(a);
        a.click();
        window.URL.revokeObjectURL(url);
        document.body.removeChild(a);
      }
    } catch (error) {
      console.error('Error exporting report:', error);
    }
  };

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <header className="bg-white shadow-sm border-b">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            <div className="flex items-center space-x-4">
              <Activity className="h-8 w-8 text-blue-600" />
              <h1 className="text-xl font-semibold text-gray-900">
                Performance Monitoring Dashboard
              </h1>
            </div>
            <div className="flex items-center space-x-4">
              <Button
                variant={isMonitoring ? 'destructive' : 'default'}
                onClick={isMonitoring ? stopMonitoring : startMonitoring}
                className="flex items-center space-x-2"
              >
                {isMonitoring ? (
                  <>
                    <RefreshCw className="h-4 w-4" />
                    <span>Stop Monitoring</span>
                  </>
                ) : (
                  <>
                    <Clock className="h-4 w-4" />
                    <span>Start Monitoring</span>
                  </>
                )}
              </Button>
              <div className="relative group">
                <Button variant="outline">
                  <Download className="h-4 w-4 mr-2" />
                  Export
                </Button>
                <div className="absolute right-0 mt-2 w-32 bg-white rounded-md shadow-lg ring-1 ring-black ring-opacity-5 opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-200 z-10">
                  <div className="py-1">
                    <button
                      onClick={() => exportReport('pdf')}
                      className="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 w-full text-left"
                    >
                      PDF Report
                    </button>
                    <button
                      onClick={() => exportReport('html')}
                      className="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 w-full text-left"
                    >
                      HTML Report
                    </button>
                    <button
                      onClick={() => exportReport('csv')}
                      className="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 w-full text-left"
                    >
                      CSV Data
                    </button>
                  </div>
                </div>
              </div>
              <Button variant="outline">
                <Settings className="h-4 w-4" />
              </Button>
            </div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-6">
          <TabsList className="grid w-full grid-cols-5">
            <TabsTrigger value="overview">Overview</TabsTrigger>
            <TabsTrigger value="performance">Performance</TabsTrigger>
            <TabsTrigger value="processes">Processes</TabsTrigger>
            <TabsTrigger value="network">Network</TabsTrigger>
            <TabsTrigger value="alerts">Alerts</TabsTrigger>
          </TabsList>

          <TabsContent value="overview">
            <SystemOverview />
          </TabsContent>

          <TabsContent value="performance">
            <PerformanceCharts />
          </TabsContent>

          <TabsContent value="processes">
            <ProcessMonitor />
          </TabsContent>

          <TabsContent value="network">
            <NetworkMonitor />
          </TabsContent>

          <TabsContent value="alerts">
            <AlertCenter />
          </TabsContent>
        </Tabs>
      </main>
    </div>
  );
};

export default Dashboard;