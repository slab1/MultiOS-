import React, { useEffect, useRef, useState } from 'react';
import * as d3 from 'd3';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { MemoryStick, Search, Filter } from 'lucide-react';

interface MemoryRegion {
  id: string;
  start: number;
  end: number;
  size: number;
  type: 'code' | 'data' | 'heap' | 'stack' | 'library' | 'shared' | 'reserved';
  permissions: string;
  pid?: number;
  processName?: string;
  color: string;
  allocationTime: number;
  lastAccess: number;
}

interface MemoryMapVisualizationProps {
  realTimeData: boolean;
}

const MemoryMapVisualization: React.FC<MemoryMapVisualizationProps> = ({ realTimeData }) => {
  const svgRef = useRef<SVGSVGElement>(null);
  const [selectedRegion, setSelectedRegion] = useState<MemoryRegion | null>(null);
  const [filterType, setFilterType] = useState<string>('all');
  const [searchTerm, setSearchTerm] = useState('');
  const [memoryRegions, setMemoryRegions] = useState<MemoryRegion[]>([]);

  // Generate realistic memory layout
  const generateMemoryLayout = (): MemoryRegion[] => {
    const regions: MemoryRegion[] = [];
    const types = ['code', 'data', 'heap', 'stack', 'library', 'shared', 'reserved'] as const;
    const colors = {
      code: '#ef4444',
      data: '#3b82f6',
      heap: '#10b981',
      stack: '#f59e0b',
      library: '#8b5cf6',
      shared: '#06b6d4',
      reserved: '#6b7280'
    };

    // Add kernel space (0x00000000 - 0x7fffffff)
    regions.push({
      id: 'kernel-space',
      start: 0x00000000,
      end: 0x7fffffff,
      size: 0x80000000,
      type: 'reserved',
      permissions: '---',
      color: colors.reserved,
      allocationTime: 0,
      lastAccess: Date.now()
    });

    // Add user space processes
    for (let i = 0; i < 8; i++) {
      const base = 0x80000000 + i * 0x20000000;
      const pid = 1000 + i;
      const processName = i === 0 ? 'kernel' : `proc-${pid}`;
      
      // Code section
      regions.push({
        id: `code-${pid}`,
        start: base,
        end: base + 0x1000000,
        size: 0x1000000,
        type: 'code',
        permissions: 'r-x',
        pid,
        processName,
        color: colors.code,
        allocationTime: Date.now() - Math.random() * 100000,
        lastAccess: Date.now() - Math.random() * 5000
      });

      // Data section
      regions.push({
        id: `data-${pid}`,
        start: base + 0x1000000,
        end: base + 0x2000000,
        size: 0x1000000,
        type: 'data',
        permissions: 'rw-',
        pid,
        processName,
        color: colors.data,
        allocationTime: Date.now() - Math.random() * 100000,
        lastAccess: Date.now() - Math.random() * 5000
      });

      // Heap
      regions.push({
        id: `heap-${pid}`,
        start: base + 0x2000000,
        end: base + 0x6000000,
        size: 0x4000000,
        type: 'heap',
        permissions: 'rw-',
        pid,
        processName,
        color: colors.heap,
        allocationTime: Date.now() - Math.random() * 100000,
        lastAccess: Date.now() - Math.random() * 5000
      });

      // Shared libraries
      if (i > 0) {
        regions.push({
          id: `lib-${pid}`,
          start: base + 0x6000000,
          end: base + 0xb000000,
          size: 0x5000000,
          type: 'library',
          permissions: 'r-x',
          pid,
          processName,
          color: colors.library,
          allocationTime: Date.now() - Math.random() * 100000,
          lastAccess: Date.now() - Math.random() * 5000
        });
      }

      // Stack
      regions.push({
        id: `stack-${pid}`,
        start: base + 0xb000000,
        end: base + 0x20000000,
        size: 0x15000000,
        type: 'stack',
        permissions: 'rw-',
        pid,
        processName,
        color: colors.stack,
        allocationTime: Date.now() - Math.random() * 100000,
        lastAccess: Date.now() - Math.random() * 5000
      });
    }

    return regions;
  };

  // Initialize memory regions
  useEffect(() => {
    setMemoryRegions(generateMemoryLayout());
  }, []);

  // Update memory regions periodically for real-time effect
  useEffect(() => {
    if (!realTimeData) return;

    const interval = setInterval(() => {
      setMemoryRegions(prev => {
        const updated = [...prev];
        // Randomly update some heap regions to simulate dynamic allocation
        const heapRegions = updated.filter(r => r.type === 'heap');
        if (heapRegions.length > 0 && Math.random() > 0.8) {
          const randomHeap = heapRegions[Math.floor(Math.random() * heapRegions.length)];
          randomHeap.lastAccess = Date.now();
        }
        return updated;
      });
    }, 2000);

    return () => clearInterval(interval);
  }, [realTimeData]);

  // Filter and search regions
  const filteredRegions = memoryRegions.filter(region => {
    const typeMatch = filterType === 'all' || region.type === filterType;
    const searchMatch = !searchTerm || 
      region.processName?.toLowerCase().includes(searchTerm.toLowerCase()) ||
      region.pid?.toString().includes(searchTerm);
    return typeMatch && searchMatch;
  });

  // Render memory map visualization
  useEffect(() => {
    if (!svgRef.current || filteredRegions.length === 0) return;

    const svg = d3.select(svgRef.current);
    const margin = { top: 20, right: 20, bottom: 20, left: 150 };
    const width = 1000 - margin.left - margin.right;
    const height = 600 - margin.bottom - margin.top;

    svg.selectAll('*').remove();

    const g = svg.append('g')
      .attr('transform', `translate(${margin.left},${margin.top})`);

    // Create scales
    const xScale = d3.scaleLinear()
      .domain([0, 0xffffffff])
      .range([0, width]);

    const yScale = d3.scaleBand()
      .domain(filteredRegions.map(d => d.id))
      .range([0, height])
      .padding(0.1);

    // Add axes
    const xAxis = d3.axisBottom(xScale)
      .tickFormat((d) => {
        const value = Number(d);
        if (value === 0) return '0x00000000';
        if (value >= 0x80000000) return '0x' + value.toString(16).toUpperCase();
        return '0x' + value.toString(16).toUpperCase();
      })
      .ticks(10);

    g.append('g')
      .attr('transform', `translate(0,${height})`)
      .call(xAxis)
      .selectAll('text')
      .style('fill', '#9ca3af')
      .style('font-size', '10px');

    // Create memory regions
    const regions = g.selectAll('.memory-region')
      .data(filteredRegions)
      .enter()
      .append('g')
      .attr('class', 'memory-region')
      .attr('transform', d => `translate(0,${yScale(d.id)})`);

    // Add memory blocks
    regions.append('rect')
      .attr('x', d => xScale(d.start))
      .attr('y', 0)
      .attr('width', d => Math.max(2, xScale(d.end) - xScale(d.start)))
      .attr('height', yScale.bandwidth())
      .attr('fill', d => d.color)
      .attr('stroke', '#374151')
      .attr('stroke-width', 1)
      .style('cursor', 'pointer')
      .on('mouseover', function(event, d) {
        d3.select(this).attr('opacity', 0.7);
        setSelectedRegion(d);
      })
      .on('mouseout', function() {
        d3.select(this).attr('opacity', 1);
      });

    // Add region labels
    regions.append('text')
      .attr('x', -10)
      .attr('y', yScale.bandwidth() / 2)
      .attr('dy', '0.35em')
      .attr('text-anchor', 'end')
      .style('fill', '#f9fafb')
      .style('font-size', '11px')
      .text(d => {
        const label = d.processName || d.type;
        return label.length > 15 ? label.substring(0, 12) + '...' : label;
      });

    // Add size information on hover
    regions.append('title')
      .text(d => `${d.processName || d.type}\nSize: ${(d.size / 1024 / 1024).toFixed(1)} MB\nRange: 0x${d.start.toString(16)} - 0x${d.end.toString(16)}\nPermissions: ${d.permissions}`);

  }, [filteredRegions]);

  const totalSize = filteredRegions.reduce((sum, region) => sum + region.size, 0);
  const typeStats = filteredRegions.reduce((stats, region) => {
    stats[region.type] = (stats[region.type] || 0) + region.size;
    return stats;
  }, {} as Record<string, number>);

  return (
    <div className="space-y-4">
      {/* Controls */}
      <Card className="bg-gray-900 border-gray-700">
        <CardHeader>
          <CardTitle className="flex items-center">
            <MemoryStick className="h-5 w-5 mr-2" />
            Memory Map Controls
          </CardTitle>
          <CardDescription>Filter and search memory regions</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-4 items-center">
            <div className="flex items-center space-x-2">
              <Filter className="h-4 w-4" />
              <select
                value={filterType}
                onChange={(e) => setFilterType(e.target.value)}
                className="bg-gray-800 border border-gray-600 rounded px-3 py-1 text-sm"
              >
                <option value="all">All Types</option>
                <option value="code">Code</option>
                <option value="data">Data</option>
                <option value="heap">Heap</option>
                <option value="stack">Stack</option>
                <option value="library">Library</option>
                <option value="shared">Shared</option>
                <option value="reserved">Reserved</option>
              </select>
            </div>
            <div className="flex items-center space-x-2">
              <Search className="h-4 w-4" />
              <input
                type="text"
                placeholder="Search by process name or PID"
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="bg-gray-800 border border-gray-600 rounded px-3 py-1 text-sm w-48"
              />
            </div>
            <Badge variant="outline">
              {filteredRegions.length} regions
            </Badge>
            <Badge variant="outline">
              {(totalSize / 1024 / 1024 / 1024).toFixed(1)} GB total
            </Badge>
          </div>
        </CardContent>
      </Card>

      {/* Memory Map Visualization */}
      <Card className="bg-gray-900 border-gray-700">
        <CardHeader>
          <CardTitle>Memory Layout Visualization</CardTitle>
          <CardDescription>Real-time memory allocation tracking</CardDescription>
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

      {/* Region Details and Statistics */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {/* Selected Region Details */}
        {selectedRegion && (
          <Card className="bg-gray-900 border-gray-700">
            <CardHeader>
              <CardTitle>Selected Region</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                <div className="flex justify-between">
                  <span className="text-gray-400">Process:</span>
                  <span>{selectedRegion.processName || 'System'}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">PID:</span>
                  <span>{selectedRegion.pid || 'N/A'}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Type:</span>
                  <Badge>{selectedRegion.type}</Badge>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Size:</span>
                  <span>{(selectedRegion.size / 1024 / 1024).toFixed(1)} MB</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Start Address:</span>
                  <span>0x{selectedRegion.start.toString(16).toUpperCase()}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">End Address:</span>
                  <span>0x{selectedRegion.end.toString(16).toUpperCase()}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Permissions:</span>
                  <Badge variant="outline">{selectedRegion.permissions}</Badge>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Last Access:</span>
                  <span>{new Date(selectedRegion.lastAccess).toLocaleTimeString()}</span>
                </div>
              </div>
            </CardContent>
          </Card>
        )}

        {/* Memory Type Statistics */}
        <Card className="bg-gray-900 border-gray-700">
          <CardHeader>
            <CardTitle>Memory Usage by Type</CardTitle>
            <CardDescription>Distribution of memory allocation</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {Object.entries(typeStats).map(([type, size]) => {
                const percentage = (size / totalSize) * 100;
                const colors = {
                  code: '#ef4444',
                  data: '#3b82f6',
                  heap: '#10b981',
                  stack: '#f59e0b',
                  library: '#8b5cf6',
                  shared: '#06b6d4',
                  reserved: '#6b7280'
                };
                return (
                  <div key={type} className="space-y-1">
                    <div className="flex justify-between items-center">
                      <span className="text-sm capitalize flex items-center">
                        <div
                          className="w-3 h-3 rounded mr-2"
                          style={{ backgroundColor: colors[type as keyof typeof colors] }}
                        />
                        {type}
                      </span>
                      <span className="text-sm text-gray-400">
                        {(size / 1024 / 1024 / 1024).toFixed(1)} GB ({percentage.toFixed(1)}%)
                      </span>
                    </div>
                    <div className="w-full bg-gray-700 rounded-full h-2">
                      <div
                        className="h-2 rounded-full"
                        style={{
                          width: `${percentage}%`,
                          backgroundColor: colors[type as keyof typeof colors]
                        }}
                      />
                    </div>
                  </div>
                );
              })}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
};

export default MemoryMapVisualization;