import React, { useState } from 'react';
import { TrendingUp, AlertTriangle, Zap, Clock, Target, Lightbulb } from 'lucide-react';
import type { PerformanceHotspot, CodeLocation } from '../App';

interface PerformanceHotspotsProps {
  hotspots: PerformanceHotspot[];
  selectedFunction: string | null;
  isLoading: boolean;
}

export const PerformanceHotspots: React.FC<PerformanceHotspotsProps> = ({
  hotspots,
  selectedFunction,
  isLoading
}) => {
  const [sortBy, setSortBy] = useState<'severity' | 'impact' | 'line'>('severity');
  const [filterSeverity, setFilterSeverity] = useState<string>('all');
  const [showOptimizations, setShowOptimizations] = useState(true);

  const getSeverityColor = (severity: string) => {
    switch (severity.toLowerCase()) {
      case 'critical': return 'text-red-700 bg-red-100 border-red-200';
      case 'high': return 'text-orange-700 bg-orange-100 border-orange-200';
      case 'medium': return 'text-yellow-700 bg-yellow-100 border-yellow-200';
      case 'low': return 'text-green-700 bg-green-100 border-green-200';
      default: return 'text-blue-700 bg-blue-100 border-blue-200';
    }
  };

  const getSeverityIcon = (severity: string) => {
    switch (severity.toLowerCase()) {
      case 'critical': return <AlertTriangle className="w-4 h-4" />;
      case 'high': return <TrendingUp className="w-4 h-4" />;
      case 'medium': return <Clock className="w-4 h-4" />;
      case 'low': return <Target className="w-4 h-4" />;
      default: return <Zap className="w-4 h-4" />;
    }
  };

  const getHotspotTypeIcon = (type: string) => {
    switch (type.toLowerCase()) {
      case 'system_call': return '🔔';
      case 'memory_allocation': return '🧠';
      case 'loop': return '🔄';
      case 'synchronization': return '🔒';
      case 'io_bound': return '💾';
      case 'cpu_intensive': return '⚡';
      case 'cache_miss': return '❌';
      default: return '⚙️';
    }
  };

  const getOptimizationSuggestion = (hotspot: PerformanceHotspot) => {
    const suggestions: { [key: string]: string } = {
      'system_call': 'Consider batching system calls or implementing caching to reduce overhead',
      'memory_allocation': 'Use memory pooling or object reuse to reduce allocation overhead',
      'loop': 'Implement loop unrolling, vectorization, or parallel processing',
      'synchronization': 'Optimize lock granularity or use lock-free algorithms where possible',
      'io_bound': 'Implement buffering, asynchronous I/O, or reduce I/O operations',
      'cpu_intensive': 'Consider algorithmic optimization or parallel processing',
      'cache_miss': 'Improve data locality and cache-friendly data structures',
    };

    return suggestions[hotspot.hotspot_type.toLowerCase()] || 'Review code for optimization opportunities';
  };

  const filteredHotspots = hotspots.filter(hotspot => 
    filterSeverity === 'all' || hotspot.severity.toLowerCase() === filterSeverity.toLowerCase()
  );

  const sortedHotspots = [...filteredHotspots].sort((a, b) => {
    switch (sortBy) {
      case 'severity':
        const severityOrder = { critical: 4, high: 3, medium: 2, low: 1, info: 0 };
        return severityOrder[b.severity.toLowerCase() as keyof typeof severityOrder] - 
               severityOrder[a.severity.toLowerCase() as keyof typeof severityOrder];
      case 'impact':
        const impactOrder = { critical: 4, high: 3, medium: 2, low: 1 };
        return impactOrder[b.estimated_impact.toLowerCase() as keyof typeof impactOrder] - 
               impactOrder[a.estimated_impact.toLowerCase() as keyof typeof impactOrder];
      case 'line':
        return a.location.line_number - b.location.line_number;
      default:
        return 0;
    }
  });

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-4"></div>
          <p className="text-gray-600">Analyzing performance hotspots...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex">
      {/* Hotspots List */}
      <div className="flex-1 bg-white border-r border-gray-200">
        <div className="flex items-center justify-between px-4 py-3 bg-gray-50 border-b border-gray-200">
          <h3 className="text-sm font-medium text-gray-900">Performance Hotspots</h3>
          
          <div className="flex items-center space-x-4">
            <select
              value={sortBy}
              onChange={(e) => setSortBy(e.target.value as any)}
              className="text-sm border border-gray-300 rounded px-2 py-1"
            >
              <option value="severity">Sort by Severity</option>
              <option value="impact">Sort by Impact</option>
              <option value="line">Sort by Line</option>
            </select>

            <select
              value={filterSeverity}
              onChange={(e) => setFilterSeverity(e.target.value)}
              className="text-sm border border-gray-300 rounded px-2 py-1"
            >
              <option value="all">All Severities</option>
              <option value="critical">Critical</option>
              <option value="high">High</option>
              <option value="medium">Medium</option>
              <option value="low">Low</option>
            </select>
          </div>
        </div>

        <div className="overflow-auto h-full p-4">
          {sortedHotspots.length > 0 ? (
            <div className="space-y-3">
              {sortedHotspots.map((hotspot, index) => (
                <div
                  key={index}
                  className={`p-4 rounded-lg border-2 transition-all duration-200 hover:shadow-md ${
                    hotspot.severity.toLowerCase() === 'critical' ? 'hotspot-critical' :
                    hotspot.severity.toLowerCase() === 'high' ? 'hotspot-high' :
                    'hotspot-medium'
                  }`}
                >
                  <div className="flex items-start justify-between mb-3">
                    <div className="flex items-center space-x-3">
                      <span className="text-2xl">
                        {getHotspotTypeIcon(hotspot.hotspot_type)}
                      </span>
                      <div>
                        <h4 className="font-medium text-gray-900 capitalize">
                          {hotspot.hotspot_type.replace('_', ' ')} Issue
                        </h4>
                        <div className="text-sm text-gray-600">
                          Line {hotspot.location.line_number} in {hotspot.location.file_path?.split('/').pop()}
                        </div>
                      </div>
                    </div>
                    
                    <div className={`flex items-center space-x-2 px-3 py-1 rounded-full border ${getSeverityColor(hotspot.severity)}`}>
                      {getSeverityIcon(hotspot.severity)}
                      <span className="text-sm font-medium capitalize">{hotspot.severity}</span>
                    </div>
                  </div>

                  <div className="mb-3">
                    <p className="text-sm text-gray-700 mb-2">{hotspot.description}</p>
                    <p className="text-sm text-gray-600">{hotspot.educational_context}</p>
                  </div>

                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-4 text-sm">
                      <div className="flex items-center space-x-1">
                        <Zap className="w-4 h-4 text-yellow-500" />
                        <span className="text-gray-600">Impact:</span>
                        <span className="font-medium capitalize">{hotspot.estimated_impact}</span>
                      </div>
                      
                      <div className="flex items-center space-x-1">
                        <Target className="w-4 h-4 text-blue-500" />
                        <span className="text-gray-600">Potential:</span>
                        <span className="font-medium capitalize">{hotspot.optimization_potential}</span>
                      </div>
                    </div>

                    <button
                      onClick={() => setShowOptimizations(!showOptimizations)}
                      className="text-sm text-blue-600 hover:text-blue-700 flex items-center space-x-1"
                    >
                      <Lightbulb className="w-4 h-4" />
                      <span>Optimization Tips</span>
                    </button>
                  </div>

                  {showOptimizations && (
                    <div className="mt-4 p-3 bg-blue-50 rounded border border-blue-200">
                      <h5 className="text-sm font-medium text-blue-800 mb-2">Optimization Suggestions</h5>
                      <p className="text-sm text-blue-700">
                        {getOptimizationSuggestion(hotspot)}
                      </p>
                      
                      <div className="mt-3">
                        <h6 className="text-xs font-medium text-blue-800 mb-1">Learning Resources:</h6>
                        <ul className="text-xs text-blue-600 space-y-1">
                          {hotspot.hotspot_type === 'system_call' && (
                            <>
                              <li>• Study system call interface design</li>
                              <li>• Learn about kernel-user space communication</li>
                              <li>• Understand context switching costs</li>
                            </>
                          )}
                          {hotspot.hotspot_type === 'memory_allocation' && (
                            <>
                              <li>• Memory allocator algorithms and trade-offs</li>
                              <li>• Cache locality and memory access patterns</li>
                              <li>• Memory pooling techniques</li>
                            </>
                          )}
                          {hotspot.hotspot_type === 'loop' && (
                            <>
                              <li>• Loop optimization techniques</li>
                              <li>• Vectorization and SIMD instructions</li>
                              <li>• Parallel loop execution</li>
                            </>
                          )}
                          {(hotspot.hotspot_type === 'synchronization' || hotspot.hotspot_type === 'cache_miss') && (
                            <>
                              <li>• Lock-free programming patterns</li>
                              <li>• Cache coherence and false sharing</li>
                              <li>• Memory ordering and barriers</li>
                            </>
                          )}
                        </ul>
                      </div>
                    </div>
                  )}
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center text-gray-500 py-8">
              <TrendingUp className="w-8 h-8 mx-auto mb-2 text-gray-300" />
              <p>No performance hotspots detected</p>
              <p className="text-sm">Code analysis is either complete or no issues were found</p>
            </div>
          )}
        </div>
      </div>

      {/* Analysis Panel */}
      <div className="w-80 bg-gray-50 border-l border-gray-200 overflow-auto">
        <div className="p-4 border-b border-gray-200">
          <h3 className="text-sm font-medium text-gray-900 mb-3">Performance Analysis</h3>
          
          {hotspots.length > 0 && (
            <div className="space-y-4">
              {/* Severity Distribution */}
              <div className="bg-white p-3 rounded border">
                <h4 className="text-xs font-medium text-gray-700 mb-2">Severity Distribution</h4>
                <div className="space-y-2">
                  {(() => {
                    const severityCounts = hotspots.reduce((acc, h) => {
                      acc[h.severity] = (acc[h.severity] || 0) + 1;
                      return acc;
                    }, {} as { [key: string]: number });

                    return Object.entries(severityCounts).map(([severity, count]) => (
                      <div key={severity} className="flex items-center justify-between">
                        <div className="flex items-center space-x-2">
                          {getSeverityIcon(severity)}
                          <span className="text-xs text-gray-600 capitalize">{severity}</span>
                        </div>
                        <span className="text-xs font-medium">{count}</span>
                      </div>
                    ));
                  })()}
                </div>
              </div>

              {/* Hotspot Types */}
              <div className="bg-white p-3 rounded border">
                <h4 className="text-xs font-medium text-gray-700 mb-2">Hotspot Types</h4>
                <div className="space-y-2">
                  {(() => {
                    const typeCounts = hotspots.reduce((acc, h) => {
                      acc[h.hotspot_type] = (acc[h.hotspot_type] || 0) + 1;
                      return acc;
                    }, {} as { [key: string]: number });

                    return Object.entries(typeCounts).map(([type, count]) => (
                      <div key={type} className="flex items-center justify-between">
                        <div className="flex items-center space-x-2">
                          <span>{getHotspotTypeIcon(type)}</span>
                          <span className="text-xs text-gray-600 capitalize">{type.replace('_', ' ')}</span>
                        </div>
                        <span className="text-xs font-medium">{count}</span>
                      </div>
                    ));
                  })()}
                </div>
              </div>

              {/* Overall Health */}
              <div className="bg-white p-3 rounded border">
                <h4 className="text-xs font-medium text-gray-700 mb-2">Overall Health</h4>
                <div className="text-center">
                  <div className="text-2xl font-bold text-gray-900 mb-1">
                    {Math.round(((hotspots.length - hotspots.filter(h => 
                      h.severity === 'critical' || h.severity === 'high'
                    ).length) / hotspots.length) * 100)}%
                  </div>
                  <div className="text-xs text-gray-600">Good Performance Areas</div>
                </div>
                
                <div className="mt-3">
                  <div className="w-full bg-gray-200 rounded-full h-2">
                    <div 
                      className={`h-2 rounded-full transition-all duration-300 ${
                        hotspots.filter(h => h.severity === 'critical' || h.severity === 'high').length === 0 
                          ? 'bg-green-500' 
                          : 'bg-orange-500'
                      }`}
                      style={{ 
                        width: `${Math.max(
                          20, 
                          100 - (hotspots.filter(h => h.severity === 'critical' || h.severity === 'high').length / hotspots.length) * 100
                        )}%` 
                      }}
                    ></div>
                  </div>
                </div>
              </div>

              {/* Top Optimization Opportunities */}
              <div className="bg-blue-50 p-3 rounded border border-blue-200">
                <h4 className="text-xs font-medium text-blue-800 mb-2">Top Optimization Areas</h4>
                <div className="space-y-2">
                  {hotspots
                    .filter(h => h.optimization_potential === 'high')
                    .slice(0, 3)
                    .map((hotspot, index) => (
                      <div key={index} className="flex items-center space-x-2">
                        <Lightbulb className="w-3 h-3 text-blue-600" />
                        <span className="text-xs text-blue-700 capitalize">
                          {hotspot.hotspot_type.replace('_', ' ')} (Line {hotspot.location.line_number})
                        </span>
                      </div>
                    ))}
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Performance Tips */}
        <div className="p-4">
          <h3 className="text-sm font-medium text-gray-900 mb-3">Performance Tips</h3>
          
          <div className="space-y-3">
            <div className="bg-green-50 p-3 rounded border border-green-200">
              <h4 className="text-xs font-medium text-green-800 mb-1">Best Practices</h4>
              <ul className="text-xs text-green-700 space-y-1">
                <li>• Profile before optimizing</li>
                <li>• Focus on critical paths first</li>
                <li>• Consider cache locality</li>
                <li>• Measure actual performance gains</li>
              </ul>
            </div>

            <div className="bg-yellow-50 p-3 rounded border border-yellow-200">
              <h4 className="text-xs font-medium text-yellow-800 mb-1">Common Pitfalls</h4>
              <ul className="text-xs text-yellow-700 space-y-1">
                <li>• Premature optimization</li>
                <li>• Ignoring algorithmic complexity</li>
                <li>• Overusing synchronization</li>
                <li>• Poor memory access patterns</li>
              </ul>
            </div>

            <div className="bg-purple-50 p-3 rounded border border-purple-200">
              <h4 className="text-xs font-medium text-purple-800 mb-1">Learning Path</h4>
              <p className="text-xs text-purple-700 mb-2">
                Understanding performance bottlenecks is crucial for systems programming.
              </p>
              <div className="text-xs text-purple-600">
                → Performance Optimization Module
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
