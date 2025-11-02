import React, { useEffect, useRef, useState } from 'react';
import * as d3 from 'd3';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { Cpu, Zap, Clock, PlayCircle } from 'lucide-react';

interface Process {
  id: string;
  pid: number;
  name: string;
  priority: number;
  state: 'running' | 'ready' | 'waiting';
  timeSlice: number;
  remainingTime: number;
  burstTime: number;
  arrivalTime: number;
  color: string;
}

interface CPU {
  id: string;
  name: string;
  currentProcess: Process | null;
  load: number;
  temperature: number;
}

interface CPUSchedulerVisualizationProps {
  realTimeData: boolean;
}

const CPUSchedulerVisualization: React.FC<CPUSchedulerVisualizationProps> = ({ realTimeData }) => {
  const cpuRef = useRef<SVGSVGElement>(null);
  const queueRef = useRef<SVGSVGElement>(null);
  const [cpus, setCpus] = useState<CPU[]>([]);
  const [runningProcesses, setRunningProcesses] = useState<Process[]>([]);
  const [readyQueue, setReadyQueue] = useState<Process[]>([]);
  const [waitingQueue, setWaitingQueue] = useState<Process[]>([]);
  const [schedulerStats, setSchedulerStats] = useState({
    contextSwitches: 0,
    averageWaitTime: 0,
    throughput: 0,
    cpuUtilization: 0
  });

  // Generate realistic CPU cores and processes
  const generateInitialState = () => {
    const numCpus = 8;
    const newCpus: CPU[] = [];

    for (let i = 0; i < numCpus; i++) {
      newCpus.push({
        id: `cpu-${i}`,
        name: `CPU ${i}`,
        currentProcess: null,
        load: Math.random() * 100,
        temperature: 35 + Math.random() * 25
      });
    }

    // Generate processes with realistic scheduling behavior
    const processes: Process[] = [];
    const processNames = [
      'kernel_task', 'systemd', 'firefox', 'chrome', 'code', 'terminal',
      'node', 'python', 'docker', 'mysql', 'nginx', 'redis', 'mongodb'
    ];

    for (let i = 0; i < 50; i++) {
      const name = processNames[i % processNames.length];
      const burstTime = Math.random() * 50 + 10; // 10-60 units
      const priority = Math.floor(Math.random() * 40) - 20; // -20 to 20
      const arrivalTime = Math.floor(Math.random() * 100);

      processes.push({
        id: `proc-${i}`,
        pid: 1000 + i,
        name,
        priority,
        state: 'ready',
        timeSlice: Math.random() * 10 + 5, // 5-15 time units
        remainingTime: burstTime,
        burstTime,
        arrivalTime,
        color: d3.schemeCategory10[i % 10]
      });
    }

    // Sort by arrival time and initialize queues
    processes.sort((a, b) => a.arrivalTime - b.arrivalTime);
    const firstProcesses = processes.slice(0, 16); // Start with 16 processes
    const remainingProcesses = processes.slice(16);

    return { newCpus, firstProcesses, remainingProcesses };
  };

  // Initialize state
  useEffect(() => {
    const { newCpus, firstProcesses } = generateInitialState();
    setCpus(newCpus);
    setReadyQueue(firstProcesses);
  }, []);

  // CPU scheduling simulation
  useEffect(() => {
    if (!realTimeData) return;

    const interval = setInterval(() => {
      setCpus(prevCpus => {
        const updatedCpus = [...prevCpus];
        
        updatedCpus.forEach(cpu => {
          // Update CPU load and temperature
          cpu.load = Math.max(0, Math.min(100, cpu.load + (Math.random() - 0.5) * 10));
          cpu.temperature = Math.max(30, Math.min(80, cpu.temperature + (Math.random() - 0.5) * 3));

          // Process current task if exists
          if (cpu.currentProcess) {
            cpu.currentProcess.remainingTime -= 1;
            cpu.currentProcess.timeSlice -= 1;

            if (cpu.currentProcess.remainingTime <= 0 || cpu.currentProcess.timeSlice <= 0) {
              // Process completed or time slice expired
              if (cpu.currentProcess.remainingTime <= 0) {
                // Process completed - could generate a new one
                cpu.currentProcess = null;
              } else {
                // Time slice expired - move to ready queue
                cpu.currentProcess.state = 'ready';
                cpu.currentProcess.timeSlice = Math.random() * 10 + 5;
                setReadyQueue(prev => [...prev, cpu.currentProcess!]);
                cpu.currentProcess = null;
              }
            }
          }

          // Assign new process if CPU is idle
          if (!cpu.currentProcess && readyQueue.length > 0) {
            // Simple priority scheduling with round-robin for same priority
            const availableProcesses = readyQueue.filter(p => p.arrivalTime <= Date.now() / 1000);
            if (availableProcesses.length > 0) {
              // Sort by priority (higher priority first)
              availableProcesses.sort((a, b) => b.priority - a.priority);
              const selectedProcess = availableProcesses[0];
              
              selectedProcess.state = 'running';
              selectedProcess.timeSlice = Math.random() * 10 + 5;
              
              setReadyQueue(prev => prev.filter(p => p.id !== selectedProcess.id));
              cpu.currentProcess = selectedProcess;
              
              setSchedulerStats(prev => ({
                ...prev,
                contextSwitches: prev.contextSwitches + 1
              }));
            }
          }
        });

        return updatedCpus;
      });

      // Update queue statistics
      setReadyQueue(prev => {
        const avgWaitTime = prev.length > 0 ? prev.reduce((sum, p) => sum + p.timeSlice, 0) / prev.length : 0;
        setSchedulerStats(prevStats => ({
          ...prevStats,
          averageWaitTime: avgWaitTime,
          cpuUtilization: cpus.filter(c => c.currentProcess).length / cpus.length * 100,
          throughput: (schedulerStats.contextSwitches / (Date.now() / 1000)).toFixed(2)
        }));
        return prev;
      });

      // Randomly add new processes
      if (Math.random() > 0.9 && readyQueue.length < 30) {
        const newProcess: Process = {
          id: `proc-${Date.now()}`,
          pid: 1000 + Date.now(),
          name: `task-${Math.floor(Math.random() * 100)}`,
          priority: Math.floor(Math.random() * 40) - 20,
          state: 'ready',
          timeSlice: Math.random() * 10 + 5,
          remainingTime: Math.random() * 50 + 10,
          burstTime: Math.random() * 50 + 10,
          arrivalTime: Date.now() / 1000,
          color: d3.schemeCategory10[Math.floor(Math.random() * 10)]
        };
        setReadyQueue(prev => [...prev, newProcess]);
      }

    }, 1000);

    return () => clearInterval(interval);
  }, [realTimeData, readyQueue, cpus, schedulerStats.contextSwitches]);

  // Render CPU visualization
  useEffect(() => {
    if (!cpuRef.current || cpus.length === 0) return;

    const svg = d3.select(cpuRef.current);
    const width = 1000;
    const height = 300;

    svg.selectAll('*').remove();

    const g = svg.append('g')
      .attr('transform', 'translate(20,20)');

    // Create CPU layout
    const cpuWidth = (width - 40) / cpus.length;
    const cpuHeight = height - 40;

    const cpuGroups = g.selectAll('.cpu-group')
      .data(cpus)
      .enter()
      .append('g')
      .attr('class', 'cpu-group')
      .attr('transform', (d, i) => `translate(${i * cpuWidth},0)`);

    // CPU boxes
    cpuGroups.append('rect')
      .attr('width', cpuWidth - 10)
      .attr('height', cpuHeight)
      .attr('fill', d => {
        if (d.currentProcess) return '#1f2937';
        return '#374151';
      })
      .attr('stroke', '#4b5563')
      .attr('stroke-width', 2)
      .attr('rx', 8);

    // CPU labels
    cpuGroups.append('text')
      .attr('x', (cpuWidth - 10) / 2)
      .attr('y', 25)
      .attr('text-anchor', 'middle')
      .style('fill', '#f9fafb')
      .style('font-size', '12px')
      .style('font-weight', 'bold')
      .text(d => d.name);

    // CPU load bars
    cpuGroups.append('rect')
      .attr('x', 10)
      .attr('y', 40)
      .attr('width', (cpuWidth - 30) * (d => d.load / 100))
      .attr('height', 20)
      .attr('fill', d => {
        if (d.load > 80) return '#ef4444';
        if (d.load > 60) return '#f59e0b';
        return '#10b981';
      })
      .attr('rx', 4);

    // Load percentage
    cpuGroups.append('text')
      .attr('x', (cpuWidth - 10) / 2)
      .attr('y', 55)
      .attr('text-anchor', 'middle')
      .style('fill', '#f9fafb')
      .style('font-size', '10px')
      .text(d => `${d.load.toFixed(1)}%`);

    // Temperature indicators
    cpuGroups.append('text')
      .attr('x', (cpuWidth - 10) / 2)
      .attr('y', 75)
      .attr('text-anchor', 'middle')
      .style('fill', d => {
        if (d.temperature > 70) return '#ef4444';
        if (d.temperature > 60) return '#f59e0b';
        return '#10b981';
      })
      .style('font-size', '10px')
      .text(d => `${d.temperature.toFixed(1)}°C`);

    // Current process display
    cpuGroups.append('text')
      .attr('x', (cpuWidth - 10) / 2)
      .attr('y', 95)
      .attr('text-anchor', 'middle')
      .style('fill', d => d.currentProcess ? '#10b981' : '#6b7280')
      .style('font-size', '10px')
      .text(d => d.currentProcess ? d.currentProcess.name : 'Idle');

    // Process progress bar
    cpuGroups.append('rect')
      .attr('x', 10)
      .attr('y', 105)
      .attr('width', cpuWidth - 30)
      .attr('height', 10)
      .attr('fill', '#374151')
      .attr('rx', 4);

    cpuGroups.append('rect')
      .attr('x', 10)
      .attr('y', 105)
      .attr('width', d => {
        if (!d.currentProcess) return 0;
        const progress = (d.currentProcess.burstTime - d.currentProcess.remainingTime) / d.currentProcess.burstTime;
        return (cpuWidth - 30) * progress;
      })
      .attr('height', 10)
      .attr('fill', '#3b82f6')
      .attr('rx', 4);

    // Time slice indicator
    cpuGroups.append('rect')
      .attr('x', 10)
      .attr('y', 120)
      .attr('width', cpuWidth - 30)
      .attr('height', 5)
      .attr('fill', '#374151')
      .attr('rx', 2);

    cpuGroups.append('rect')
      .attr('x', 10)
      .attr('y', 120)
      .attr('width', d => {
        if (!d.currentProcess) return 0;
        const progress = (15 - d.currentProcess.timeSlice) / 15; // Assuming max time slice of 15
        return (cpuWidth - 30) * Math.max(0, Math.min(1, progress));
      })
      .attr('height', 5)
      .attr('fill', '#f59e0b')
      .attr('rx', 2);

  }, [cpus]);

  // Render queue visualization
  useEffect(() => {
    if (!queueRef.current) return;

    const svg = d3.select(queueRef.current);
    const width = 1000;
    const height = 200;

    svg.selectAll('*').remove();

    const g = svg.append('g')
      .attr('transform', 'translate(20,20)');

    // Ready Queue visualization
    const queueY = 30;
    const queueHeight = 60;

    // Queue background
    g.append('rect')
      .attr('x', 0)
      .attr('y', queueY)
      .attr('width', width - 40)
      .attr('height', queueHeight)
      .attr('fill', '#1f2937')
      .attr('stroke', '#4b5563')
      .attr('rx', 8);

    // Queue label
    g.append('text')
      .attr('x', 10)
      .attr('y', queueY - 10)
      .style('fill', '#f9fafb')
      .style('font-size', '12px')
      .style('font-weight', 'bold')
      .text(`Ready Queue (${readyQueue.length} processes)`);

    // Process blocks in ready queue
    const processWidth = Math.max(80, (width - 60) / Math.max(1, readyQueue.length));
    const maxProcesses = Math.floor((width - 60) / 80);

    const displayProcesses = readyQueue.slice(0, maxProcesses);

    g.selectAll('.ready-process')
      .data(displayProcesses)
      .enter()
      .append('rect')
      .attr('class', 'ready-process')
      .attr('x', (d, i) => 10 + i * processWidth)
      .attr('y', queueY + 5)
      .attr('width', Math.min(75, processWidth - 5))
      .attr('height', queueHeight - 10)
      .attr('fill', '#374151')
      .attr('stroke', '#6b7280')
      .attr('rx', 4);

    g.selectAll('.ready-process-label')
      .data(displayProcesses)
      .enter()
      .append('text')
      .attr('class', 'ready-process-label')
      .attr('x', (d, i) => 10 + i * processWidth + Math.min(75, processWidth - 5) / 2)
      .attr('y', queueY + queueHeight / 2)
      .attr('text-anchor', 'middle')
      .attr('dominant-baseline', 'middle')
      .style('fill', '#f9fafb')
      .style('font-size', '10px')
      .style('font-weight', 'bold')
      .text(d => d.name.length > 8 ? d.name.substring(0, 6) + '...' : d.name);

    // Priority indicator
    g.selectAll('.priority-indicator')
      .data(displayProcesses)
      .enter()
      .append('rect')
      .attr('class', 'priority-indicator')
      .attr('x', (d, i) => 10 + i * processWidth)
      .attr('y', queueY + 5)
      .attr('width', Math.min(75, processWidth - 5))
      .attr('height', 3)
      .attr('fill', d => {
        if (d.priority > 10) return '#10b981';
        if (d.priority > 0) return '#f59e0b';
        return '#ef4444';
      });

  }, [readyQueue]);

  const runningCount = cpus.filter(cpu => cpu.currentProcess).length;
  const totalProcesses = runningCount + readyQueue.length + waitingQueue.length;

  return (
    <div className="space-y-4">
      {/* Scheduler Statistics */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <Card className="bg-gray-900 border-gray-700">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm flex items-center">
              <Cpu className="h-4 w-4 mr-2" />
              Active CPUs
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{runningCount}/{cpus.length}</div>
            <div className="text-xs text-gray-400">Running</div>
          </CardContent>
        </Card>

        <Card className="bg-gray-900 border-gray-700">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm flex items-center">
              <Zap className="h-4 w-4 mr-2" />
              Context Switches
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{schedulerStats.contextSwitches}</div>
            <div className="text-xs text-gray-400">Total</div>
          </CardContent>
        </Card>

        <Card className="bg-gray-900 border-gray-700">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm flex items-center">
              <Clock className="h-4 w-4 mr-2" />
              Avg Wait Time
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{schedulerStats.averageWaitTime.toFixed(1)}</div>
            <div className="text-xs text-gray-400">Time units</div>
          </CardContent>
        </Card>

        <Card className="bg-gray-900 border-gray-700">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm flex items-center">
              <PlayCircle className="h-4 w-4 mr-2" />
              CPU Utilization
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{schedulerStats.cpuUtilization.toFixed(1)}%</div>
            <Progress value={schedulerStats.cpuUtilization} className="mt-2" />
          </CardContent>
        </Card>
      </div>

      {/* CPU Layout Visualization */}
      <Card className="bg-gray-900 border-gray-700">
        <CardHeader>
          <CardTitle className="flex items-center">
            <Cpu className="h-5 w-5 mr-2" />
            CPU Scheduler - Multi-Core Processing
          </CardTitle>
          <CardDescription>Real-time CPU assignment and load balancing</CardDescription>
        </CardHeader>
        <CardContent>
          <svg
            ref={cpuRef}
            width={1000}
            height={300}
            className="border border-gray-600 rounded"
            style={{ background: '#1f2937' }}
          />
        </CardContent>
      </Card>

      {/* Ready Queue Visualization */}
      <Card className="bg-gray-900 border-gray-700">
        <CardHeader>
          <CardTitle>Ready Queue</CardTitle>
          <CardDescription>Processes waiting for CPU allocation</CardDescription>
        </CardHeader>
        <CardContent>
          <svg
            ref={queueRef}
            width={1000}
            height={200}
            className="border border-gray-600 rounded"
            style={{ background: '#1f2937' }}
          />
        </CardContent>
      </Card>

      {/* Process Details */}
      <Card className="bg-gray-900 border-gray-700">
        <CardHeader>
          <CardTitle>Current Process Details</CardTitle>
          <CardDescription>Active processes on each CPU core</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            {cpus.map(cpu => (
              <div key={cpu.id} className="bg-gray-800 p-4 rounded-lg">
                <div className="flex items-center justify-between mb-2">
                  <h4 className="font-semibold">{cpu.name}</h4>
                  <Badge 
                    variant={cpu.currentProcess ? "default" : "secondary"}
                    className={cpu.currentProcess ? "bg-green-500" : ""}
                  >
                    {cpu.currentProcess ? "Active" : "Idle"}
                  </Badge>
                </div>
                
                {cpu.currentProcess ? (
                  <div className="space-y-2">
                    <div>
                      <div className="text-sm text-gray-400">Process</div>
                      <div className="font-mono">{cpu.currentProcess.name}</div>
                    </div>
                    <div>
                      <div className="text-sm text-gray-400">PID</div>
                      <div className="font-mono">{cpu.currentProcess.pid}</div>
                    </div>
                    <div>
                      <div className="text-sm text-gray-400">Priority</div>
                      <div className="font-mono">{cpu.currentProcess.priority}</div>
                    </div>
                    <div>
                      <div className="text-sm text-gray-400">Progress</div>
                      <Progress 
                        value={((cpu.currentProcess.burstTime - cpu.currentProcess.remainingTime) / cpu.currentProcess.burstTime) * 100}
                        className="mt-1"
                      />
                    </div>
                    <div>
                      <div className="text-sm text-gray-400">Time Slice</div>
                      <div className="text-xs">{cpu.currentProcess.timeSlice.toFixed(1)} units</div>
                    </div>
                  </div>
                ) : (
                  <div className="text-center text-gray-400 py-4">
                    No active process
                  </div>
                )}
                
                <div className="mt-4 pt-4 border-t border-gray-700">
                  <div className="grid grid-cols-2 gap-2 text-xs">
                    <div>
                      <div className="text-gray-400">Load</div>
                      <div className={cpu.load > 80 ? "text-red-400" : cpu.load > 60 ? "text-yellow-400" : "text-green-400"}>
                        {cpu.load.toFixed(1)}%
                      </div>
                    </div>
                    <div>
                      <div className="text-gray-400">Temp</div>
                      <div className={cpu.temperature > 70 ? "text-red-400" : cpu.temperature > 60 ? "text-yellow-400" : "text-green-400"}>
                        {cpu.temperature.toFixed(1)}°C
                      </div>
                    </div>
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

export default CPUSchedulerVisualization;