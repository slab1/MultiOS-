import React, { useState, useEffect, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { Process } from '../../types';
import { Play, Pause, RotateCcw, Clock, Zap, Target } from 'lucide-react';

interface CPU {
  currentProcess: Process | null;
  time: number;
  isRunning: boolean;
}

interface CPUSchedulingGameProps {
  onComplete: (score: number, metrics: any) => void;
  difficulty: 'beginner' | 'intermediate' | 'advanced';
}

export const CPUSchedulingGame: React.FC<CPUSchedulingGameProps> = ({ 
  onComplete, 
  difficulty 
}) => {
  const [processes, setProcesses] = useState<Process[]>([]);
  const [cpu, setCpu] = useState<CPU>({ currentProcess: null, time: 0, isRunning: false });
  const [completedProcesses, setCompletedProcesses] = useState<Process[]>([]);
  const [selectedAlgorithm, setSelectedAlgorithm] = useState<
    'FCFS' | 'SJF' | 'Priority' | 'Round Robin' | 'Preemptive SJF'
  >('FCFS');
  const [timeQuantum, setTimeQuantum] = useState(2);
  const [score, setScore] = useState(100);
  const [isPaused, setIsPaused] = useState(true);
  const [gameMetrics, setGameMetrics] = useState({
    totalTime: 0,
    avgWaitingTime: 0,
    avgTurnaroundTime: 0,
    cpuUtilization: 0,
    contextSwitches: 0,
  });
  const [currentTime, setCurrentTime] = useState(0);
  const [waitingQueue, setWaitingQueue] = useState<Process[]>([]);
  const [readyQueue, setReadyQueue] = useState<Process[]>([]);

  const initializeProcesses = useCallback(() => {
    const numProcesses = difficulty === 'beginner' ? 4 : difficulty === 'intermediate' ? 6 : 8;
    const newProcesses: Process[] = Array.from({ length: numProcesses }, (_, i) => ({
      id: `P${i + 1}`,
      name: `Process ${i + 1}`,
      arrivalTime: Math.floor(Math.random() * 10), // Random arrival time
      burstTime: difficulty === 'beginner' ? 3 + Math.random() * 7 : 
                 difficulty === 'intermediate' ? 2 + Math.random() * 8 : 
                 1 + Math.random() * 10,
      priority: 1 + Math.floor(Math.random() * 5),
      memoryRequired: 50 + Math.random() * 100,
      state: 'ready',
    }));
    
    // Sort by arrival time initially
    newProcesses.sort((a, b) => a.arrivalTime - b.arrivalTime);
    setProcesses(newProcesses);
    setReadyQueue(newProcesses.filter(p => p.arrivalTime === 0));
  }, [difficulty]);

  useEffect(() => {
    initializeProcesses();
  }, [initializeProcesses]);

  const getNextProcess = (): Process | null => {
    const availableProcesses = [...waitingQueue, ...readyQueue];
    if (availableProcesses.length === 0) return null;

    switch (selectedAlgorithm) {
      case 'FCFS':
        return availableProcesses[0];
      
      case 'SJF':
        return availableProcesses.reduce((shortest, process) => 
          process.burstTime < shortest.burstTime ? process : shortest
        );
      
      case 'Priority':
        return availableProcesses.reduce((highest, process) => 
          (process.priority || 1) < (highest.priority || 1) ? process : highest
        );
      
      case 'Round Robin':
        if (cpu.currentProcess && selectedAlgorithm === 'Round Robin') {
          const currentInQueue = availableProcesses.find(p => p.id === cpu.currentProcess!.id);
          if (currentInQueue) {
            const currentIndex = availableProcesses.indexOf(currentInQueue);
            return availableProcesses[(currentIndex + 1) % availableProcesses.length];
          }
        }
        return availableProcesses[0];
      
      case 'Preemptive SJF':
        if (cpu.currentProcess) {
          const shortest = availableProcesses.reduce((shortest, process) => 
            process.burstTime < shortest.burstTime ? process : shortest
          );
          if (shortest.id !== cpu.currentProcess.id && shortest.burstTime < cpu.currentProcess.burstTime) {
            return shortest;
          }
          return cpu.currentProcess;
        }
        return availableProcesses[0];
      
      default:
        return availableProcesses[0];
    }
  };

  const executeStep = useCallback(() => {
    setCurrentTime(prev => prev + 1);
    setCpu(prev => ({ ...prev, time: prev.time + 1 }));

    // Add new arrivals to ready queue
    const newArrivals = processes.filter(p => p.arrivalTime === currentTime);
    setReadyQueue(prev => [...prev, ...newArrivals]);

    if (!cpu.currentProcess) {
      const nextProcess = getNextProcess();
      if (nextProcess) {
        setCpu(prev => ({ 
          ...prev, 
          currentProcess: nextProcess,
          isRunning: true 
        }));
        
        if (waitingQueue.find(p => p.id === nextProcess.id)) {
          setWaitingQueue(prev => prev.filter(p => p.id !== nextProcess.id));
        } else {
          setReadyQueue(prev => prev.filter(p => p.id !== nextProcess.id));
        }
        
        setGameMetrics(prev => ({ ...prev, contextSwitches: prev.contextSwitches + 1 }));
      }
    }

    if (cpu.currentProcess) {
      const updatedProcess = { 
        ...cpu.currentProcess, 
        burstTime: cpu.currentProcess.burstTime - 1 
      };

      if (updatedProcess.burstTime <= 0) {
        // Process completed
        setCompletedProcesses(prev => [...prev, { ...updatedProcess, burstTime: 0 }]);
        setCpu(prev => ({ ...prev, currentProcess: null, isRunning: false }));
        
        if (completedProcesses.length + 1 === processes.length) {
          // All processes completed
          calculateFinalMetrics();
          const finalScore = calculateScore();
          onComplete(finalScore, { ...gameMetrics, totalTime: currentTime });
        }
      } else {
        // Check for preemption (Round Robin)
        if (selectedAlgorithm === 'Round Robin' && timeQuantum > 0 && 
            (cpu.time % timeQuantum) === 0 && cpu.time > 0) {
          setWaitingQueue(prev => [...prev, updatedProcess]);
          setCpu(prev => ({ ...prev, currentProcess: null, isRunning: false }));
        } else {
          setCpu(prev => ({ 
            ...prev, 
            currentProcess: updatedProcess 
          }));
        }
      }
    }

    // Calculate waiting times
    const waitingProcesses = [...waitingQueue, ...readyQueue.filter(p => p.id !== cpu.currentProcess?.id)];
    const totalWaitingTime = waitingProcesses.reduce((sum, p) => sum + 1, 0);
    setGameMetrics(prev => ({
      ...prev,
      avgWaitingTime: waitingProcesses.length > 0 ? totalWaitingTime / waitingProcesses.length : 0,
      cpuUtilization: completedProcesses.length / processes.length * 100,
    }));
  }, [currentTime, cpu, processes, waitingQueue, readyQueue, selectedAlgorithm, timeQuantum, completedProcesses.length, gameMetrics, onComplete, processes.length]);

  const calculateScore = (): number => {
    const baseScore = Math.max(0, 100 - gameMetrics.avgWaitingTime - (gameMetrics.contextSwitches * 2));
    const utilizationBonus = Math.min(20, gameMetrics.cpuUtilization / 5);
    return Math.floor(baseScore + utilizationBonus);
  };

  const calculateFinalMetrics = () => {
    const avgTurnaroundTime = completedProcesses.reduce((sum, p) => {
      const turnaroundTime = p.arrivalTime ? (currentTime - p.arrivalTime) : currentTime;
      return sum + turnaroundTime;
    }, 0) / completedProcesses.length;

    setGameMetrics(prev => ({
      ...prev,
      avgTurnaroundTime,
      cpuUtilization: 100,
    }));
  };

  const toggleSimulation = () => {
    setIsPaused(!isPaused);
  };

  const resetSimulation = () => {
    initializeProcesses();
    setCompletedProcesses([]);
    setCpu({ currentProcess: null, time: 0, isRunning: false });
    setCurrentTime(0);
    setWaitingQueue([]);
    setReadyQueue([]);
    setScore(100);
    setIsPaused(true);
    setGameMetrics({
      totalTime: 0,
      avgWaitingTime: 0,
      avgTurnaroundTime: 0,
      cpuUtilization: 0,
      contextSwitches: 0,
    });
  };

  useEffect(() => {
    let interval: NodeJS.Timeout;
    if (!isPaused && completedProcesses.length < processes.length) {
      interval = setInterval(executeStep, 500); // Step every 500ms
    }
    return () => clearInterval(interval);
  }, [isPaused, executeStep, completedProcesses.length, processes.length]);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-bold text-white">CPU Scheduling Simulator</h2>
          <p className="text-gray-300">Learn different CPU scheduling algorithms</p>
        </div>
        <div className="flex gap-2">
          <Badge variant="secondary">Time: {currentTime}</Badge>
          <Badge variant="secondary">Score: {score}</Badge>
        </div>
      </div>

      {/* Algorithm Selection */}
      <Card className="bg-slate-800 border-slate-700">
        <CardHeader>
          <CardTitle className="text-white flex items-center gap-2">
            <Target className="w-5 h-5" />
            Scheduling Algorithm
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-5 gap-2 mb-4">
            {(['FCFS', 'SJF', 'Priority', 'Round Robin', 'Preemptive SJF'] as const).map((algo) => (
              <Button
                key={algo}
                variant={selectedAlgorithm === algo ? 'default' : 'outline'}
                size="sm"
                onClick={() => setSelectedAlgorithm(algo)}
                className="text-xs"
              >
                {algo.replace(' ', '\n')}
              </Button>
            ))}
          </div>
          {selectedAlgorithm === 'Round Robin' && (
            <div className="flex items-center gap-2">
              <label className="text-sm text-gray-300">Time Quantum:</label>
              <input
                type="range"
                min="1"
                max="5"
                value={timeQuantum}
                onChange={(e) => setTimeQuantum(parseInt(e.target.value))}
                className="flex-1"
              />
              <span className="text-sm text-white w-8">{timeQuantum}</span>
            </div>
          )}
        </CardContent>
      </Card>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* CPU State */}
        <Card className="bg-slate-800 border-slate-700">
          <CardHeader>
            <CardTitle className="text-white flex items-center gap-2">
              <Zap className="w-5 h-5" />
              CPU
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-center">
              {cpu.currentProcess ? (
                <div className="bg-blue-600 p-4 rounded-lg">
                  <div className="text-white font-bold text-lg">{cpu.currentProcess.name}</div>
                  <div className="text-blue-100 text-sm">Remaining: {cpu.currentProcess.burstTime}</div>
                  <div className="text-blue-100 text-sm">Priority: {cpu.currentProcess.priority}</div>
                </div>
              ) : (
                <div className="bg-gray-600 p-4 rounded-lg">
                  <div className="text-gray-300">CPU Idle</div>
                </div>
              )}
              <div className="mt-4 flex gap-2">
                <Button onClick={toggleSimulation} size="sm" className="flex-1">
                  {isPaused ? <Play className="w-4 h-4" /> : <Pause className="w-4 h-4" />}
                  {isPaused ? 'Start' : 'Pause'}
                </Button>
                <Button onClick={resetSimulation} size="sm" variant="outline">
                  <RotateCcw className="w-4 h-4" />
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Ready Queue */}
        <Card className="bg-slate-800 border-slate-700">
          <CardHeader>
            <CardTitle className="text-white flex items-center gap-2">
              <Clock className="w-5 h-5" />
              Ready Queue
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2 max-h-48 overflow-y-auto">
              {readyQueue.map((process) => (
                <div key={process.id} className="bg-green-600 p-2 rounded text-white text-sm">
                  <div className="font-medium">{process.name}</div>
                  <div>Burst: {process.burstTime} | Priority: {process.priority}</div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>

        {/* Completed Processes */}
        <Card className="bg-slate-800 border-slate-700">
          <CardHeader>
            <CardTitle className="text-white">Completed ({completedProcesses.length}/{processes.length})</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2 max-h-48 overflow-y-auto">
              {completedProcesses.map((process) => (
                <div key={process.id} className="bg-purple-600 p-2 rounded text-white text-sm">
                  <div className="font-medium">{process.name}</div>
                  <div>Turnaround: {currentTime - process.arrivalTime}</div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Metrics Dashboard */}
      <Card className="bg-slate-800 border-slate-700">
        <CardHeader>
          <CardTitle className="text-white">Performance Metrics</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="text-center">
              <div className="text-2xl font-bold text-white">{gameMetrics.avgWaitingTime.toFixed(1)}</div>
              <div className="text-sm text-gray-400">Avg Waiting Time</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-white">{gameMetrics.avgTurnaroundTime.toFixed(1)}</div>
              <div className="text-sm text-gray-400">Avg Turnaround Time</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-white">{gameMetrics.cpuUtilization.toFixed(1)}%</div>
              <div className="text-sm text-gray-400">CPU Utilization</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-white">{gameMetrics.contextSwitches}</div>
              <div className="text-sm text-gray-400">Context Switches</div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Gantt Chart */}
      <Card className="bg-slate-800 border-slate-700">
        <CardHeader>
          <CardTitle className="text-white">Execution Timeline</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-1">
            <div className="text-sm text-gray-400">Time: {Array.from({ length: 20 }, (_, i) => i).join(' ')}</div>
            <div className="flex gap-1 overflow-x-auto">
              {Array.from({ length: Math.min(20, currentTime + 1) }, (_, i) => (
                <div
                  key={i}
                  className={`w-8 h-8 flex items-center justify-center text-xs rounded ${
                    cpu.time >= i + 1 && cpu.currentProcess
                      ? 'bg-blue-600 text-white'
                      : 'bg-gray-600 text-gray-400'
                  }`}
                >
                  {completedProcesses.find(p => 
                    i >= p.arrivalTime && i < p.arrivalTime + (p.burstTime + 1)
                  )?.name.slice(-1) || (waitingQueue.find(p => 
                    i >= p.arrivalTime && i < p.arrivalTime + p.burstTime
                  )?.name.slice(-1)) || '-'}
                </div>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};