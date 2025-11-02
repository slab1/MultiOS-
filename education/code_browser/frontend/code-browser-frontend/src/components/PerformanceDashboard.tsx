import React, { useState, useEffect } from 'react';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, PieChart, Pie, Cell, LineChart, Line } from 'recharts';
import { TrendingUp, Clock, Cpu, HardDrive, Zap, Target, AlertTriangle } from 'lucide-react';

interface PerformanceMetric {
  name: string;
  value: number;
  unit: string;
  trend: 'up' | 'down' | 'stable';
  change: number;
}

interface PerformanceSnapshot {
  timestamp: number;
  cpu_usage: number;
  memory_usage: number;
  cache_hit_rate: number;
  context_switches: number;
  syscalls_per_sec: number;
}

export const PerformanceDashboard: React.FC = () => {
  const [timeRange, setTimeRange] = useState<'1h' | '24h' | '7d'>('24h');
  const [selectedMetric, setSelectedMetric] = useState<string>('cpu_usage');

  // Mock performance data
  const currentMetrics: PerformanceMetric[] = [
    { name: 'CPU Usage', value: 23.5, unit: '%', trend: 'down', change: -2.3 },
    { name: 'Memory Usage', value: 67.2, unit: '%', trend: 'up', change: 1.8 },
    { name: 'Cache Hit Rate', value: 94.7, unit: '%', trend: 'up', change: 0.5 },
    { name: 'Context Switches', value: 1250, unit: '/sec', trend: 'down', change: -45 },
    { name: 'System Calls', value: 340, unit: '/sec', trend: 'stable', change: 2 },
    { name: 'Interrupt Rate', value: 89, unit: '/sec', trend: 'up', change: 12 },
  ];

  const performanceData: PerformanceSnapshot[] = [
    { timestamp: Date.now() - 3600000 * 6, cpu_usage: 25, memory_usage: 65, cache_hit_rate: 93, context_switches: 1200, syscalls_per_sec: 320 },
    { timestamp: Date.now() - 3600000 * 5, cpu_usage: 28, memory_usage: 66, cache_hit_rate: 94, context_switches: 1350, syscalls_per_sec: 345 },
    { timestamp: Date.now() - 3600000 * 4, cpu_usage: 22, memory_usage: 67, cache_hit_rate: 95, context_switches: 1100, syscalls_per_sec: 330 },
    { timestamp: Date.now() - 3600000 * 3, cpu_usage: 26, memory_usage: 68, cache_hit_rate: 94, context_switches: 1400, syscalls_per_sec: 355 },
    { timestamp: Date.now() - 3600000 * 2, cpu_usage: 24, memory_usage: 67, cache_hit_rate: 94, context_switches: 1250, syscalls_per_sec: 340 },
    { timestamp: Date.now() - 3600000 * 1, cpu_usage: 23, memory_usage: 67, cache_hit_rate: 95, context_switches: 1150, syscalls_per_sec: 325 },
    { timestamp: Date.now(), cpu_usage: 24, memory_usage: 67, cache_hit_rate: 95, context_switches: 1250, syscalls_per_sec: 340 },
  ];

  const hotspotDistribution = [
    { name: 'System Calls', value: 35, color: '#ef4444' },
    { name: 'Memory Allocation', value: 28, color: '#f97316' },
    { name: 'Loop Processing', value: 22, color: '#eab308' },
    { name: 'Synchronization', value: 10, color: '#22c55e' },
    { name: 'Cache Misses', value: 5, color: '#3b82f6' },
  ];

  const optimizationOpportunities = [
    { function: 'schedule_next_task', impact: 'high', estimated_improvement: '15%', complexity: 'medium', effort: '2 days' },
    { function: 'memory_allocate', impact: 'medium', estimated_improvement: '8%', complexity: 'easy', effort: '1 day' },
    { function: 'syscall_handler', impact: 'high', estimated_improvement: '20%', complexity: 'hard', effort: '1 week' },
    { function: 'context_switch', impact: 'medium', estimated_improvement: '12%', complexity: 'medium', effort: '3 days' },
  ];

  const getTrendIcon = (trend: string) => {
    switch (trend) {
      case 'up': return <TrendingUp className="w-4 h-4 text-green-500" />;
      case 'down': return <TrendingUp className="w-4 h-4 text-red-500 rotate-180" />;
      default: return <div className="w-4 h-4 bg-gray-400 rounded-full" />;
    }
  };

  const getImpactColor = (impact: string) => {
    switch (impact) {
      case 'high': return 'text-red-700 bg-red-100 border-red-200';
      case 'medium': return 'text-yellow-700 bg-yellow-100 border-yellow-200';
      case 'low': return 'text-green-700 bg-green-100 border-green-200';
      default: return 'text-gray-700 bg-gray-100 border-gray-200';
    }
  };

  return (
    <div className="max-w-7xl mx-auto space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Performance Dashboard</h1>
          <p className="text-gray-600">Real-time performance metrics and optimization insights</p>
        </div>
        
        <div className="flex items-center space-x-4">
          <select
            value={timeRange}
            onChange={(e) => setTimeRange(e.target.value as any)}
            className="border border-gray-300 rounded-md px-3 py-2 text-sm"
          >
            <option value="1h">Last Hour</option>
            <option value="24h">Last 24 Hours</option>
            <option value="7d">Last 7 Days</option>
          </select>
          
          <div className="flex items-center space-x-2 text-sm text-gray-600">
            <div className="w-2 h-2 bg-green-400 rounded-full animate-pulse"></div>
            <span>Live Data</span>
          </div>
        </div>
      </div>

      {/* Current Metrics Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {currentMetrics.map((metric, index) => (
          <div key={index} className="bg-white rounded-lg shadow border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-2">
              <h3 className="text-sm font-medium text-gray-600">{metric.name}</h3>
              {getTrendIcon(metric.trend)}
            </div>
            
            <div className="flex items-end space-x-2">
              <span className="text-3xl font-bold text-gray-900">
                {metric.value}{metric.unit}
              </span>
              <span className={`text-sm pb-1 ${
                metric.change > 0 ? 'text-green-600' : 
                metric.change < 0 ? 'text-red-600' : 'text-gray-500'
              }`}>
                {metric.change > 0 ? '+' : ''}{metric.change}{metric.unit}
              </span>
            </div>
          </div>
        ))}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Performance Trends Chart */}
        <div className="bg-white rounded-lg shadow border border-gray-200 p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Performance Trends</h3>
          <ResponsiveContainer width="100%" height={300}>
            <LineChart data={performanceData}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis 
                dataKey="timestamp" 
                tickFormatter={(value) => new Date(value).toLocaleTimeString()}
              />
              <YAxis />
              <Tooltip 
                labelFormatter={(value) => new Date(value).toLocaleString()}
                formatter={(value, name) => [value, name.replace('_', ' ')]}
              />
              <Line 
                type="monotone" 
                dataKey="cpu_usage" 
                stroke="#3b82f6" 
                strokeWidth={2}
                name="CPU Usage (%)"
              />
              <Line 
                type="monotone" 
                dataKey="memory_usage" 
                stroke="#ef4444" 
                strokeWidth={2}
                name="Memory Usage (%)"
              />
            </LineChart>
          </ResponsiveContainer>
        </div>

        {/* Hotspot Distribution */}
        <div className="bg-white rounded-lg shadow border border-gray-200 p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Performance Hotspots</h3>
          <ResponsiveContainer width="100%" height={300}>
            <PieChart>
              <Pie
                data={hotspotDistribution}
                cx="50%"
                cy="50%"
                labelLine={false}
                label={({ name, percent }) => `${name} ${(percent * 100).toFixed(0)}%`}
                outerRadius={80}
                fill="#8884d8"
                dataKey="value"
              >
                {hotspotDistribution.map((entry, index) => (
                  <Cell key={`cell-${index}`} fill={entry.color} />
                ))}
              </Pie>
              <Tooltip />
            </PieChart>
          </ResponsiveContainer>
        </div>
      </div>

      {/* Optimization Opportunities */}
      <div className="bg-white rounded-lg shadow border border-gray-200">
        <div className="px-6 py-4 border-b border-gray-200">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-semibold text-gray-900">Optimization Opportunities</h3>
            <div className="flex items-center space-x-2 text-sm text-gray-600">
              <Target className="w-4 h-4" />
              <span>Based on real-time analysis</span>
            </div>
          </div>
        </div>
        
        <div className="p-6">
          <div className="space-y-4">
            {optimizationOpportunities.map((opportunity, index) => (
              <div key={index} className="flex items-center justify-between p-4 bg-gray-50 rounded-lg">
                <div className="flex items-center space-x-4">
                  <div className="w-10 h-10 bg-blue-100 rounded-lg flex items-center justify-center">
                    <Zap className="w-5 h-5 text-blue-600" />
                  </div>
                  <div>
                    <div className="font-medium text-gray-900">{opportunity.function}</div>
                    <div className="text-sm text-gray-600">Estimated improvement: {opportunity.estimated_improvement}</div>
                  </div>
                </div>
                
                <div className="flex items-center space-x-4">
                  <span className={`px-3 py-1 text-xs rounded-full border ${getImpactColor(opportunity.impact)}`}>
                    {opportunity.impact} impact
                  </span>
                  <div className="text-right">
                    <div className="text-sm text-gray-900">{opportunity.complexity} complexity</div>
                    <div className="text-xs text-gray-600">{opportunity.effort}</div>
                  </div>
                  <button className="px-4 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700">
                    Review
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* System Health Overview */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="bg-white rounded-lg shadow border border-gray-200 p-6">
          <div className="flex items-center space-x-3 mb-4">
            <Cpu className="w-6 h-6 text-blue-600" />
            <h3 className="text-lg font-semibold text-gray-900">CPU Performance</h3>
          </div>
          
          <div className="space-y-4">
            <div>
              <div className="flex justify-between text-sm mb-1">
                <span>Current Usage</span>
                <span>23.5%</span>
              </div>
              <div className="w-full bg-gray-200 rounded-full h-2">
                <div className="bg-blue-500 h-2 rounded-full" style={{ width: '23.5%' }}></div>
              </div>
            </div>
            
            <div className="text-sm text-gray-600">
              <div className="flex justify-between">
                <span>Idle Time:</span>
                <span>76.5%</span>
              </div>
              <div className="flex justify-between">
                <span>Context Switches:</span>
                <span>1,250/sec</span>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-white rounded-lg shadow border border-gray-200 p-6">
          <div className="flex items-center space-x-3 mb-4">
            <HardDrive className="w-6 h-6 text-green-600" />
            <h3 className="text-lg font-semibold text-gray-900">Memory Performance</h3>
          </div>
          
          <div className="space-y-4">
            <div>
              <div className="flex justify-between text-sm mb-1">
                <span>Current Usage</span>
                <span>67.2%</span>
              </div>
              <div className="w-full bg-gray-200 rounded-full h-2">
                <div className="bg-green-500 h-2 rounded-full" style={{ width: '67.2%' }}></div>
              </div>
            </div>
            
            <div className="text-sm text-gray-600">
              <div className="flex justify-between">
                <span>Cache Hit Rate:</span>
                <span>94.7%</span>
              </div>
              <div className="flex justify-between">
                <span>Page Faults:</span>
                <span>0.2/sec</span>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-white rounded-lg shadow border border-gray-200 p-6">
          <div className="flex items-center space-x-3 mb-4">
            <Clock className="w-6 h-6 text-purple-600" />
            <h3 className="text-lg font-semibold text-gray-900">I/O Performance</h3>
          </div>
          
          <div className="space-y-4">
            <div>
              <div className="flex justify-between text-sm mb-1">
                <span>System Calls</span>
                <span>340/sec</span>
              </div>
              <div className="w-full bg-gray-200 rounded-full h-2">
                <div className="bg-purple-500 h-2 rounded-full" style={{ width: '68%' }}></div>
              </div>
            </div>
            
            <div className="text-sm text-gray-600">
              <div className="flex justify-between">
                <span>Interrupt Rate:</span>
                <span>89/sec</span>
              </div>
              <div className="flex justify-between">
                <span>Avg Latency:</span>
                <span>12.3μs</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Performance Alerts */}
      <div className="bg-white rounded-lg shadow border border-gray-200 p-6">
        <div className="flex items-center space-x-3 mb-4">
          <AlertTriangle className="w-6 h-6 text-yellow-600" />
          <h3 className="text-lg font-semibold text-gray-900">Performance Alerts</h3>
        </div>
        
        <div className="space-y-3">
          <div className="flex items-center space-x-3 p-3 bg-yellow-50 rounded-lg border border-yellow-200">
            <AlertTriangle className="w-5 h-5 text-yellow-600" />
            <div>
              <div className="font-medium text-yellow-800">High Context Switch Rate</div>
              <div className="text-sm text-yellow-700">Context switches increased by 15% in the last hour</div>
            </div>
          </div>
          
          <div className="flex items-center space-x-3 p-3 bg-blue-50 rounded-lg border border-blue-200">
            <Clock className="w-5 h-5 text-blue-600" />
            <div>
              <div className="font-medium text-blue-800">Memory Usage Trending Up</div>
              <div className="text-sm text-blue-700">Gradual increase in memory usage detected</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
