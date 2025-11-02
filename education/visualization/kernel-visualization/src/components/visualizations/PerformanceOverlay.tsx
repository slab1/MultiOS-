import React, { useEffect, useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { 
  Activity, 
  Cpu, 
  Database, 
  Network, 
  HardDrive, 
  Thermometer,
  Zap,
  AlertTriangle,
  CheckCircle,
  TrendingUp,
  TrendingDown,
  Minus
} from 'lucide-react';

interface PerformanceMetric {
  id: string;
  name: string;
  value: number;
  unit: string;
  threshold: { warning: number; critical: number };
  icon: React.ReactNode;
  color: string;
  trend: 'up' | 'down' | 'stable';
  history: { timestamp: number; value: number }[];
}

interface PerformanceOverlayProps {
  realTimeData: boolean;
  className?: string;
}

const PerformanceOverlay: React.FC<PerformanceOverlayProps> = ({ realTimeData, className = '' }) => {
  const [metrics, setMetrics] = useState<PerformanceMetric[]>([]);
  const [alerts, setAlerts] = useState<Array<{ id: string; message: string; type: 'warning' | 'critical' }>>([]);

  // Generate performance metrics
  const generateMetrics = (): PerformanceMetric[] => {
    return [
      {
        id: 'cpu',
        name: 'CPU Usage',
        value: 15 + Math.random() * 40,
        unit: '%',
        threshold: { warning: 70, critical: 90 },
        icon: <Cpu className="h-4 w-4" />,
        color: '#3b82f6',
        trend: 'stable',
        history: []
      },
      {
        id: 'memory',
        name: 'Memory Usage',
        value: 40 + Math.random() * 30,
        unit: '%',
        threshold: { warning: 80, critical: 95 },
        icon: <Database className="h-4 w-4" />,
        color: '#10b981',
        trend: 'stable',
        history: []
      },
      {
        id: 'disk_io',
        name: 'Disk I/O',
        value: Math.random() * 100,
        unit: 'MB/s',
        threshold: { warning: 80, critical: 95 },
        icon: <HardDrive className="h-4 w-4" />,
        color: '#f59e0b',
        trend: 'stable',
        history: []
      },
      {
        id: 'network',
        name: 'Network I/O',
        value: Math.random() * 200,
        unit: 'Mbps',
        threshold: { warning: 150, critical: 180 },
        icon: <Network className="h-4 w-4" />,
        color: '#8b5cf6',
        trend: 'stable',
        history: []
      },
      {
        id: 'temperature',
        name: 'CPU Temperature',
        value: 45 + Math.random() * 20,
        unit: '°C',
        threshold: { warning: 70, critical: 85 },
        icon: <Thermometer className="h-4 w-4" />,
        color: '#ef4444',
        trend: 'stable',
        history: []
      },
      {
        id: 'processes',
        name: 'Active Processes',
        value: 45 + Math.floor(Math.random() * 20),
        unit: '',
        threshold: { warning: 80, critical: 120 },
        icon: <Activity className="h-4 w-4" />,
        color: '#06b6d4',
        trend: 'stable',
        history: []
      },
      {
        id: 'syscalls',
        name: 'System Calls/sec',
        value: 1000 + Math.random() * 2000,
        unit: 'ops',
        threshold: { warning: 5000, critical: 8000 },
        icon: <Zap className="h-4 w-4" />,
        color: '#ec4899',
        trend: 'stable',
        history: []
      },
      {
        id: 'load_avg',
        name: 'Load Average',
        value: 1 + Math.random() * 3,
        unit: '',
        threshold: { warning: 4, critical: 8 },
        icon: <TrendingUp className="h-4 w-4" />,
        color: '#14b8a6',
        trend: 'stable',
        history: []
      }
    ];
  };

  // Update metrics in real-time
  useEffect(() => {
    setMetrics(generateMetrics());
  }, []);

  useEffect(() => {
    if (!realTimeData) return;

    const interval = setInterval(() => {
      setMetrics(prev => {
        const updated = prev.map(metric => {
          // Generate realistic changes based on metric type
          let change = 0;
          switch (metric.id) {
            case 'cpu':
              change = (Math.random() - 0.5) * 10;
              break;
            case 'memory':
              change = (Math.random() - 0.5) * 5;
              break;
            case 'disk_io':
              change = (Math.random() - 0.5) * 20;
              break;
            case 'network':
              change = (Math.random() - 0.5) * 30;
              break;
            case 'temperature':
              change = (Math.random() - 0.5) * 3;
              break;
            case 'processes':
              change = Math.floor((Math.random() - 0.5) * 3);
              break;
            case 'syscalls':
              change = (Math.random() - 0.5) * 500;
              break;
            case 'load_avg':
              change = (Math.random() - 0.5) * 0.5;
              break;
          }

          const newValue = Math.max(0, metric.value + change);
          
          // Determine trend
          let trend: 'up' | 'down' | 'stable' = 'stable';
          if (metric.history.length > 0) {
            const recent = metric.history.slice(-5);
            const avgRecent = recent.reduce((sum, h) => sum + h.value, 0) / recent.length;
            const avgCurrent = recent.reduce((sum, h) => sum + h.value, 0) / recent.length + change;
            
            if (avgCurrent > avgRecent * 1.05) trend = 'up';
            else if (avgCurrent < avgRecent * 0.95) trend = 'down';
          }

          // Add to history
          const newHistory = [...metric.history, { timestamp: Date.now(), value: newValue }];
          // Keep only last 20 data points
          if (newHistory.length > 20) {
            newHistory.shift();
          }

          return {
            ...metric,
            value: newValue,
            trend,
            history: newHistory
          };
        });

        // Check for alerts
        const newAlerts = updated
          .filter(m => m.value >= m.threshold.critical || m.value >= m.threshold.warning)
          .map(m => ({
            id: m.id,
            message: `${m.name}: ${m.value.toFixed(1)}${m.unit} (threshold: ${m.threshold.warning}${m.unit})`,
            type: m.value >= m.threshold.critical ? 'critical' as const : 'warning' as const
          }));

        setAlerts(newAlerts);

        return updated;
      });
    }, 2000);

    return () => clearInterval(interval);
  }, [realTimeData]);

  const getStatusLevel = (metric: PerformanceMetric) => {
    if (metric.value >= metric.threshold.critical) return 'critical';
    if (metric.value >= metric.threshold.warning) return 'warning';
    return 'normal';
  };

  const getStatusColor = (level: string) => {
    switch (level) {
      case 'critical': return 'bg-red-500';
      case 'warning': return 'bg-yellow-500';
      default: return 'bg-green-500';
    }
  };

  const getTrendIcon = (trend: string) => {
    switch (trend) {
      case 'up': return <TrendingUp className="h-3 w-3" />;
      case 'down': return <TrendingDown className="h-3 w-3" />;
      default: return <Minus className="h-3 w-3" />;
    }
  };

  const getTrendColor = (trend: string) => {
    switch (trend) {
      case 'up': return 'text-red-400';
      case 'down': return 'text-green-400';
      default: return 'text-gray-400';
    }
  };

  // Calculate overall system health
  const criticalMetrics = metrics.filter(m => getStatusLevel(m) === 'critical').length;
  const warningMetrics = metrics.filter(m => getStatusLevel(m) === 'warning').length;
  const normalMetrics = metrics.filter(m => getStatusLevel(m) === 'normal').length;
  const systemHealth = normalMetrics / metrics.length * 100;

  return (
    <div className={`space-y-4 ${className}`}>
      {/* System Health Overview */}
      <Card className="bg-gray-900 border-gray-700">
        <CardHeader>
          <CardTitle className="flex items-center">
            <Activity className="h-5 w-5 mr-2" />
            System Performance Overview
          </CardTitle>
          <CardDescription>Real-time performance metrics overlay</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-4">
            <div className="text-center">
              <div className="text-3xl font-bold text-green-400">{normalMetrics}</div>
              <div className="text-sm text-gray-400">Normal</div>
            </div>
            <div className="text-center">
              <div className="text-3xl font-bold text-yellow-400">{warningMetrics}</div>
              <div className="text-sm text-gray-400">Warning</div>
            </div>
            <div className="text-center">
              <div className="text-3xl font-bold text-red-400">{criticalMetrics}</div>
              <div className="text-sm text-gray-400">Critical</div>
            </div>
            <div className="text-center">
              <div className="text-3xl font-bold text-blue-400">{systemHealth.toFixed(0)}%</div>
              <div className="text-sm text-gray-400">Health Score</div>
            </div>
          </div>

          {/* System Health Bar */}
          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span>Overall System Health</span>
              <span>{systemHealth.toFixed(1)}%</span>
            </div>
            <Progress value={systemHealth} className="h-3" />
          </div>
        </CardContent>
      </Card>

      {/* Performance Metrics Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {metrics.map(metric => {
          const statusLevel = getStatusLevel(metric);
          return (
            <Card key={metric.id} className="bg-gray-900 border-gray-700">
              <CardHeader className="pb-2">
                <CardTitle className="text-sm flex items-center justify-between">
                  <div className="flex items-center">
                    <span className="mr-2" style={{ color: metric.color }}>
                      {metric.icon}
                    </span>
                    {metric.name}
                  </div>
                  <div className="flex items-center space-x-1">
                    <Badge 
                      variant="outline" 
                      className={`text-xs ${getStatusColor(statusLevel)} text-white`}
                    >
                      {getTrendIcon(metric.trend)}
                    </Badge>
                  </div>
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div className="flex items-end justify-between">
                    <span className="text-2xl font-bold">
                      {metric.id === 'processes' ? Math.floor(metric.value) : metric.value.toFixed(1)}
                    </span>
                    <span className="text-sm text-gray-400">{metric.unit}</span>
                  </div>

                  {/* Progress bar */}
                  <div className="space-y-1">
                    <div className="w-full bg-gray-700 rounded-full h-2">
                      <div
                        className={`h-2 rounded-full ${getStatusColor(statusLevel)}`}
                        style={{ 
                          width: `${Math.min(100, (metric.value / (metric.threshold.critical || 100)) * 100)}%` 
                        }}
                      />
                    </div>
                    <div className="flex justify-between text-xs text-gray-400">
                      <span>0</span>
                      <span>Warning: {metric.threshold.warning}{metric.unit}</span>
                      <span>Critical: {metric.threshold.critical}{metric.unit}</span>
                    </div>
                  </div>

                  {/* Trend indicator */}
                  <div className="flex items-center justify-between text-xs">
                    <span className="text-gray-400">Trend</span>
                    <span className={`flex items-center ${getTrendColor(metric.trend)}`}>
                      {getTrendIcon(metric.trend)}
                      <span className="ml-1 capitalize">{metric.trend}</span>
                    </span>
                  </div>

                  {/* Status indicator */}
                  <div className="flex items-center justify-between text-xs">
                    <span className="text-gray-400">Status</span>
                    <Badge 
                      variant={statusLevel === 'critical' ? 'destructive' : statusLevel === 'warning' ? 'secondary' : 'default'}
                      className="text-xs"
                    >
                      {statusLevel === 'critical' && <AlertTriangle className="h-3 w-3 mr-1" />}
                      {statusLevel === 'normal' && <CheckCircle className="h-3 w-3 mr-1" />}
                      <span className="capitalize">{statusLevel}</span>
                    </Badge>
                  </div>
                </div>
              </CardContent>
            </Card>
          );
        })}
      </div>

      {/* Alerts */}
      {alerts.length > 0 && (
        <Card className="bg-red-900/20 border-red-700">
          <CardHeader>
            <CardTitle className="flex items-center text-red-400">
              <AlertTriangle className="h-5 w-5 mr-2" />
              Performance Alerts
            </CardTitle>
            <CardDescription>System performance issues requiring attention</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {alerts.map(alert => (
                <div 
                  key={alert.id} 
                  className="flex items-center justify-between p-3 bg-red-900/30 rounded border border-red-700"
                >
                  <span className="text-sm">{alert.message}</span>
                  <Badge 
                    variant={alert.type === 'critical' ? 'destructive' : 'secondary'}
                    className="text-xs"
                  >
                    {alert.type}
                  </Badge>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Historical Trends */}
      <Card className="bg-gray-900 border-gray-700">
        <CardHeader>
          <CardTitle>Performance Trends</CardTitle>
          <CardDescription>Historical performance data for key metrics</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            {metrics.slice(0, 4).map(metric => (
              <div key={metric.id} className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium">{metric.name}</span>
                  <span className="text-xs text-gray-400">
                    {metric.value.toFixed(1)}{metric.unit}
                  </span>
                </div>
                <div className="h-20 bg-gray-800 rounded p-2 relative overflow-hidden">
                  <svg width="100%" height="100%" className="absolute inset-0">
                    {metric.history.length > 1 && (
                      <polyline
                        fill="none"
                        stroke={metric.color}
                        strokeWidth="2"
                        points={metric.history
                          .map((point, i) => {
                            const x = (i / (metric.history.length - 1)) * 100;
                            const y = 100 - (point.value / (metric.threshold.critical || 100)) * 80;
                            return `${x},${y}`;
                          })
                          .join(' ')}
                      />
                    )}
                  </svg>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Performance Score */}
      <Card className="bg-gray-900 border-gray-700">
        <CardHeader>
          <CardTitle>Performance Score Breakdown</CardTitle>
          <CardDescription>Detailed performance scoring by subsystem</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {[
              { name: 'CPU Performance', score: 100 - Math.max(0, metrics.find(m => m.id === 'cpu')?.value || 0) },
              { name: 'Memory Efficiency', score: 100 - Math.max(0, metrics.find(m => m.id === 'memory')?.value || 0) },
              { name: 'I/O Performance', score: 100 - Math.max(0, (metrics.find(m => m.id === 'disk_io')?.value || 0) / 2) },
              { name: 'Network Performance', score: 100 - Math.max(0, (metrics.find(m => m.id === 'network')?.value || 0) / 3) },
              { name: 'Thermal Performance', score: 100 - Math.max(0, (metrics.find(m => m.id === 'temperature')?.value || 0) * 1.2) },
              { name: 'Process Management', score: 100 - Math.max(0, (metrics.find(m => m.id === 'processes')?.value || 0)) }
            ].map((subsystem, index) => (
              <div key={index} className="space-y-2">
                <div className="flex justify-between items-center">
                  <span className="text-sm">{subsystem.name}</span>
                  <span className="text-sm font-medium">{subsystem.score.toFixed(0)}/100</span>
                </div>
                <div className="w-full bg-gray-700 rounded-full h-2">
                  <div
                    className={`h-2 rounded-full ${
                      subsystem.score >= 80 ? 'bg-green-500' :
                      subsystem.score >= 60 ? 'bg-yellow-500' : 'bg-red-500'
                    }`}
                    style={{ width: `${Math.max(0, Math.min(100, subsystem.score))}%` }}
                  />
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default PerformanceOverlay;