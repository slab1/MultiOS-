import React, { useState, useEffect, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { MemoryBlock, Process } from '../../types';
import { Play, RotateCcw, Lightbulb, Zap } from 'lucide-react';

interface MemoryManagementGameProps {
  onComplete: (score: number) => void;
  difficulty: 'beginner' | 'intermediate' | 'advanced';
}

export const MemoryManagementGame: React.FC<MemoryManagementGameProps> = ({ 
  onComplete, 
  difficulty 
}) => {
  const [memoryBlocks, setMemoryBlocks] = useState<MemoryBlock[]>([]);
  const [processes, setProcesses] = useState<Process[]>([]);
  const [selectedProcess, setSelectedProcess] = useState<Process | null>(null);
  const [currentAlgorithm, setCurrentAlgorithm] = useState<'first-fit' | 'best-fit' | 'worst-fit'>('first-fit');
  const [score, setScore] = useState(100);
  const [gamePhase, setGamePhase] = useState<'allocating' | 'deallocating' | 'completed'>('allocating');
  const [moves, setMoves] = useState(0);
  const [hintUsed, setHintUsed] = useState(false);

  const memorySize = difficulty === 'beginner' ? 100 : difficulty === 'intermediate' ? 200 : 500;

  const initializeMemory = useCallback(() => {
    const blocks: MemoryBlock[] = [
      { id: '1', start: 0, size: memorySize, allocated: false }
    ];
    setMemoryBlocks(blocks);
  }, [memorySize]);

  const generateProcess = useCallback(() => {
    const size = difficulty === 'beginner' ? 10 + Math.random() * 30 : 
                 difficulty === 'intermediate' ? 15 + Math.random() * 40 : 
                 20 + Math.random() * 50;
    
    return {
      id: Date.now().toString(),
      name: `P${Date.now()}`,
      arrivalTime: 0,
      burstTime: 5 + Math.random() * 10,
      memoryRequired: Math.floor(size),
      state: 'ready' as const
    };
  }, [difficulty]);

  useEffect(() => {
    initializeMemory();
    const initialProcesses = Array.from({ length: difficulty === 'beginner' ? 3 : difficulty === 'intermediate' ? 5 : 8 }, 
      () => generateProcess());
    setProcesses(initialProcesses);
  }, [initializeMemory, generateProcess, difficulty]);

  const allocateMemory = (process: Process): boolean => {
    const availableBlocks = memoryBlocks.filter(block => !block.allocated && block.size >= process.memoryRequired);
    
    if (availableBlocks.length === 0) return false;

    let targetBlock: MemoryBlock | null = null;

    switch (currentAlgorithm) {
      case 'first-fit':
        targetBlock = availableBlocks.find(block => block.size >= process.memoryRequired) || null;
        break;
      case 'best-fit':
        targetBlock = availableBlocks.reduce((best, block) => 
          block.size < (best?.size || Infinity) && block.size >= process.memoryRequired ? block : best, null);
        break;
      case 'worst-fit':
        targetBlock = availableBlocks.reduce((worst, block) => 
          block.size > (worst?.size || 0) && block.size >= process.memoryRequired ? block : worst, null);
        break;
    }

    if (!targetBlock) return false;

    // Update memory blocks
    const newBlocks = [...memoryBlocks];
    const blockIndex = newBlocks.findIndex(block => block.id === targetBlock!.id);
    
    if (targetBlock.size === process.memoryRequired) {
      newBlocks[blockIndex].allocated = true;
      newBlocks[blockIndex].processId = process.id;
    } else {
      newBlocks[blockIndex] = {
        ...newBlocks[blockIndex],
        allocated: true,
        processId: process.id,
        size: process.memoryRequired
      };
      
      newBlocks.splice(blockIndex + 1, 0, {
        id: (Date.now() + Math.random()).toString(),
        start: targetBlock.start + process.memoryRequired,
        size: targetBlock.size - process.memoryRequired,
        allocated: false
      });
    }

    setMemoryBlocks(newBlocks);
    setMoves(prev => prev + 1);
    return true;
  };

  const handleProcessClick = (process: Process) => {
    if (selectedProcess?.id === process.id) {
      setSelectedProcess(null);
    } else {
      setSelectedProcess(process);
    }
  };

  const attemptAllocation = () => {
    if (!selectedProcess) return;

    const success = allocateMemory(selectedProcess);
    if (success) {
      setProcesses(prev => prev.filter(p => p.id !== selectedProcess.id));
      setSelectedProcess(null);
      setScore(prev => Math.max(prev - 2, 0)); // Deduct points for each move
      
      // Check if all processes are allocated
      if (processes.length <= 1) {
        const finalScore = Math.max(score - moves * 2, 0);
        setGamePhase('completed');
        onComplete(finalScore);
      }
    }
  };

  const getAlgorithmHint = () => {
    const hints = {
      'first-fit': 'First-fit allocates to the first available block that fits the process.',
      'best-fit': 'Best-fit searches for the smallest block that can fit the process.',
      'worst-fit': 'Worst-fit uses the largest available block to minimize fragmentation.'
    };
    return hints[currentAlgorithm];
  };

  const calculateFragmentation = () => {
    const freeBlocks = memoryBlocks.filter(block => !block.allocated);
    if (freeBlocks.length <= 1) return 0;
    
    const totalFreeSpace = freeBlocks.reduce((sum, block) => sum + block.size, 0);
    const largestBlock = Math.max(...freeBlocks.map(block => block.size));
    
    return totalFreeSpace > 0 ? ((totalFreeSpace - largestBlock) / totalFreeSpace) * 100 : 0;
  };

  const resetGame = () => {
    initializeMemory();
    const newProcesses = Array.from({ length: difficulty === 'beginner' ? 3 : difficulty === 'intermediate' ? 5 : 8 }, 
      () => generateProcess());
    setProcesses(newProcesses);
    setSelectedProcess(null);
    setScore(100);
    setMoves(0);
    setGamePhase('allocating');
    setHintUsed(false);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-bold text-white">Memory Management Simulator</h2>
          <p className="text-gray-300">Allocate memory efficiently using different algorithms</p>
        </div>
        <Badge variant={gamePhase === 'completed' ? 'default' : 'secondary'}>
          Score: {score} | Moves: {moves}
        </Badge>
      </div>

      {/* Algorithm Selection */}
      <Card className="bg-slate-800 border-slate-700">
        <CardHeader>
          <CardTitle className="text-white flex items-center gap-2">
            <Zap className="w-5 h-5" />
            Memory Allocation Algorithm
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-3 gap-2">
            {(['first-fit', 'best-fit', 'worst-fit'] as const).map((algo) => (
              <Button
                key={algo}
                variant={currentAlgorithm === algo ? 'default' : 'outline'}
                size="sm"
                onClick={() => setCurrentAlgorithm(algo)}
                className="capitalize"
              >
                {algo.replace('-', ' ')}
              </Button>
            ))}
          </div>
          <div className="mt-3 p-3 bg-slate-700 rounded text-sm text-gray-300">
            <strong>Current:</strong> {getAlgorithmHint()}
          </div>
        </CardContent>
      </Card>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Memory Visualization */}
        <Card className="bg-slate-800 border-slate-700">
          <CardHeader>
            <CardTitle className="text-white">Memory Layout</CardTitle>
            <p className="text-sm text-gray-400">Total Memory: {memorySize} units</p>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {memoryBlocks.map((block) => (
                <div
                  key={block.id}
                  className={`p-3 rounded border-2 transition-all ${
                    block.allocated
                      ? 'bg-blue-600 border-blue-500 text-white'
                      : 'bg-green-600 border-green-500 text-white'
                  }`}
                  style={{ width: `${Math.max((block.size / memorySize) * 100, 10)}%` }}
                >
                  <div className="text-sm font-medium">
                    {block.allocated ? `${block.processId}` : 'FREE'}
                  </div>
                  <div className="text-xs opacity-75">Size: {block.size}</div>
                </div>
              ))}
            </div>
            <div className="mt-4 text-sm text-gray-400">
              <div>Fragmentation: {calculateFragmentation().toFixed(1)}%</div>
              <div>Free Memory: {memoryBlocks.filter(b => !b.allocated).reduce((sum, b) => sum + b.size, 0)}</div>
            </div>
          </CardContent>
        </Card>

        {/* Process Queue */}
        <Card className="bg-slate-800 border-slate-700">
          <CardHeader>
            <CardTitle className="text-white">Process Queue</CardTitle>
            <p className="text-sm text-gray-400">Select a process and allocate memory</p>
          </CardHeader>
          <CardContent>
            <div className="space-y-2 max-h-64 overflow-y-auto">
              {processes.map((process) => (
                <div
                  key={process.id}
                  className={`p-3 rounded cursor-pointer transition-all ${
                    selectedProcess?.id === process.id
                      ? 'bg-yellow-600 border-yellow-500'
                      : 'bg-slate-700 border-slate-600 hover:bg-slate-600'
                  }`}
                  onClick={() => handleProcessClick(process)}
                >
                  <div className="text-white font-medium">{process.name}</div>
                  <div className="text-sm text-gray-300">
                    Memory: {process.memoryRequired} units
                  </div>
                </div>
              ))}
            </div>
            <div className="mt-4 space-y-2">
              <Button 
                onClick={attemptAllocation}
                disabled={!selectedProcess || gamePhase !== 'allocating'}
                className="w-full"
              >
                <Play className="w-4 h-4 mr-2" />
                Allocate Memory
              </Button>
              <Button 
                onClick={() => setHintUsed(true)}
                variant="outline" 
                size="sm" 
                className="w-full"
                disabled={hintUsed}
              >
                <Lightbulb className="w-4 h-4 mr-2" />
                {hintUsed ? 'Hint Used' : 'Get Hint'}
              </Button>
              <Button onClick={resetGame} variant="outline" size="sm" className="w-full">
                <RotateCcw className="w-4 h-4 mr-2" />
                Reset Game
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Game Progress */}
      <Card className="bg-slate-800 border-slate-700">
        <CardContent className="pt-6">
          <div className="space-y-3">
            <div className="flex justify-between text-sm">
              <span className="text-gray-300">Progress</span>
              <span className="text-gray-300">
                {memoryBlocks.filter(b => b.allocated).length} / {memoryBlocks.length} blocks allocated
              </span>
            </div>
            <Progress 
              value={(memoryBlocks.filter(b => b.allocated).length / memoryBlocks.length) * 100} 
              className="h-2"
            />
            <div className="text-xs text-gray-400">
              Goal: Allocate all processes with minimal moves and low fragmentation
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Completion Modal */}
      {gamePhase === 'completed' && (
        <Card className="bg-gradient-to-r from-green-600 to-blue-600 border-0">
          <CardContent className="pt-6 text-center">
            <h3 className="text-2xl font-bold text-white mb-2">Challenge Complete!</h3>
            <p className="text-white/90 mb-4">
              Final Score: {score} | Total Moves: {moves} | Fragmentation: {calculateFragmentation().toFixed(1)}%
            </p>
            <div className="text-sm text-white/80">
              {score >= 80 ? 'Excellent memory management!' :
               score >= 60 ? 'Good job! You can improve efficiency.' :
               'Keep practicing to improve your memory allocation strategy.'}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
};