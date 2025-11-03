import React, { useState, useEffect } from 'react';
import { Play, Pause, RotateCcw, Settings, Monitor, Cpu, HardDrive, FileText } from 'lucide-react';

const DemosPage: React.FC = () => {
  const [activeDemo, setActiveDemo] = useState('kernel');
  const [isPlaying, setIsPlaying] = useState(false);
  const [speed, setSpeed] = useState(1);

  const demos = [
    {
      id: 'kernel',
      title: 'Kernel Debugging',
      description: 'Step through kernel boot process and system calls',
      icon: Monitor,
      color: 'from-red-500 to-red-600',
    },
    {
      id: 'process',
      title: 'Process Management',
      description: 'Visualize process scheduling and lifecycle',
      icon: Cpu,
      color: 'from-blue-500 to-blue-600',
    },
    {
      id: 'memory',
      title: 'Memory Allocation',
      description: 'Dynamic memory allocation and garbage collection',
      icon: HardDrive,
      color: 'from-green-500 to-green-600',
    },
    {
      id: 'filesystem',
      title: 'File System',
      description: 'File operations and directory structure',
      icon: FileText,
      color: 'from-purple-500 to-purple-600',
    },
  ];

  const KernelDebuggingDemo: React.FC = () => {
    const [currentStep, setCurrentStep] = useState(0);
    const [logs, setLogs] = useState<string[]>([]);
    const [registers, setRegisters] = useState({
      rip: '0x1000',
      rax: '0x0',
      rbx: '0x0',
      rcx: '0x0',
      rdx: '0x0',
    });

    const bootSteps = [
      'Loading bootloader...',
      'Initializing memory management unit...',
      'Setting up page tables...',
      'Enabling interrupts...',
      'Initializing device drivers...',
      'Loading kernel modules...',
      'Starting system services...',
      'MultiOS boot complete!',
    ];

    useEffect(() => {
      if (isPlaying) {
        const interval = setInterval(() => {
          setCurrentStep(prev => {
            if (prev < bootSteps.length - 1) {
              setLogs(prevLogs => [...prevLogs, bootSteps[prev]]);
              setRegisters(prev => ({
                ...prev,
                rax: `0x${(prev.rip ? parseInt(prev.rip.slice(2), 16) : 0x1000 + prev * 0x10).toString(16)}`,
                rip: `0x${(prev.rip ? parseInt(prev.rip.slice(2), 16) : 0x1000 + prev * 0x10).toString(16)}`,
              }));
              return prev + 1;
            } else {
              clearInterval(interval);
              setIsPlaying(false);
              return prev;
            }
          });
        }, 1000 / speed);
        return () => clearInterval(interval);
      }
    }, [isPlaying, speed]);

    const handleReset = () => {
      setCurrentStep(0);
      setLogs([]);
      setRegisters({
        rip: '0x1000',
        rax: '0x0',
        rbx: '0x0',
        rcx: '0x0',
        rdx: '0x0',
      });
      setIsPlaying(false);
    };

    return (
      <div className="bg-gray-900 text-green-400 p-4 rounded-lg font-mono text-sm">
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
          {/* Terminal */}
          <div className="bg-black p-4 rounded border border-gray-700">
            <div className="flex items-center justify-between mb-2">
              <span className="text-gray-400">MultiOS Debugger</span>
              <div className="flex space-x-2">
                <button
                  onClick={() => setIsPlaying(!isPlaying)}
                  className="p-1 hover:bg-gray-800 rounded"
                >
                  {isPlaying ? <Pause size={16} /> : <Play size={16} />}
                </button>
                <button onClick={handleReset} className="p-1 hover:bg-gray-800 rounded">
                  <RotateCcw size={16} />
                </button>
              </div>
            </div>
            <div className="space-y-1 h-96 overflow-y-auto">
              {logs.map((log, index) => (
                <div key={index} className="text-green-400">
                  <span className="text-gray-500">[Step {index + 1}]</span> {log}
                </div>
              ))}
            </div>
          </div>

          {/* Registers */}
          <div className="bg-black p-4 rounded border border-gray-700">
            <div className="flex items-center justify-between mb-2">
              <span className="text-gray-400">CPU Registers</span>
              <div className="flex items-center space-x-2">
                <label className="text-gray-400 text-xs">Speed:</label>
                <select
                  value={speed}
                  onChange={(e) => setSpeed(Number(e.target.value))}
                  className="bg-gray-800 text-white text-xs p-1 rounded border border-gray-600"
                >
                  <option value={0.5}>0.5x</option>
                  <option value={1}>1x</option>
                  <option value={2}>2x</option>
                  <option value={4}>4x</option>
                </select>
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <div className="text-gray-400 mb-1">General Purpose</div>
                <div className="space-y-1">
                  <div>RIP: {registers.rip}</div>
                  <div>RAX: {registers.rax}</div>
                  <div>RBX: {registers.rbx}</div>
                  <div>RCX: {registers.rcx}</div>
                  <div>RDX: {registers.rdx}</div>
                </div>
              </div>
              <div>
                <div className="text-gray-400 mb-1">Memory</div>
                <div className="space-y-1">
                  <div>Stack: 0x7fff...</div>
                  <div>Heap: 0x4000...</div>
                  <div>Code: 0x1000...</div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  };

  const ProcessManagementDemo: React.FC = () => {
    const [processes, setProcesses] = useState([
      { id: 1, name: 'init', state: 'Running', pid: 1, priority: 0, cpu: 100 },
      { id: 2, name: 'scheduler', state: 'Running', pid: 2, priority: 5, cpu: 50 },
      { id: 3, name: 'memory-manager', state: 'Running', pid: 3, priority: 3, cpu: 75 },
    ]);

    const [scheduling, setScheduling] = useState('RR');

    const nextProcess = () => {
      setProcesses(prev => prev.map((p, index) => {
        if (index === 0) {
          return { ...p, cpu: p.cpu === 100 ? 0 : p.cpu + 25 };
        }
        return p;
      }));
    };

    useEffect(() => {
      if (isPlaying) {
        const interval = setInterval(nextProcess, 500 / speed);
        return () => clearInterval(interval);
      }
    }, [isPlaying, speed]);

    return (
      <div className="bg-gray-900 text-white p-4 rounded-lg">
        <div className="mb-4">
          <div className="flex items-center justify-between mb-2">
            <span className="text-gray-400">Process Scheduler</span>
            <div className="flex space-x-2">
              <button
                onClick={() => setIsPlaying(!isPlaying)}
                className="p-1 hover:bg-gray-800 rounded"
              >
                {isPlaying ? <Pause size={16} /> : <Play size={16} />}
              </button>
              <select
                value={scheduling}
                onChange={(e) => setScheduling(e.target.value)}
                className="bg-gray-800 text-white text-xs p-1 rounded border border-gray-600"
              >
                <option value="RR">Round Robin</option>
                <option value="FCFS">First Come First Serve</option>
                <option value="Priority">Priority</option>
              </select>
            </div>
          </div>
        </div>

        <div className="space-y-2">
          {processes.map((process, index) => (
            <div
              key={process.id}
              className={`p-3 rounded border ${
                index === 0 ? 'border-yellow-500 bg-yellow-500/10' : 'border-gray-700 bg-gray-800'
              }`}
            >
              <div className="flex justify-between items-center mb-1">
                <span className="font-semibold">{process.name}</span>
                <span className={`px-2 py-1 rounded text-xs ${
                  process.state === 'Running' ? 'bg-green-600' : 'bg-red-600'
                }`}>
                  {process.state}
                </span>
              </div>
              <div className="text-sm text-gray-400 mb-2">
                PID: {process.pid} | Priority: {process.priority}
              </div>
              <div className="w-full bg-gray-700 rounded-full h-2">
                <div
                  className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                  style={{ width: `${process.cpu}%` }}
                ></div>
              </div>
              <div className="text-xs text-gray-400 mt-1">
                CPU Usage: {process.cpu}%
              </div>
            </div>
          ))}
        </div>
      </div>
    );
  };

  const MemoryAllocationDemo: React.FC = () => {
    const [memoryBlocks, setMemoryBlocks] = useState([
      { id: 1, size: 1024, used: false, process: null },
      { id: 2, size: 2048, used: false, process: null },
      { id: 3, size: 4096, used: false, process: null },
    ]);
    const [allocationCount, setAllocationCount] = useState(0);

    const allocateMemory = () => {
      const freeBlock = memoryBlocks.find(block => !block.used);
      if (freeBlock) {
        setMemoryBlocks(prev => prev.map(block =>
          block.id === freeBlock.id
            ? { ...block, used: true, process: `Process ${allocationCount + 1}` }
            : block
        ));
        setAllocationCount(prev => prev + 1);
      }
    };

    const deallocateMemory = () => {
      const usedBlock = memoryBlocks.find(block => block.used);
      if (usedBlock) {
        setMemoryBlocks(prev => prev.map(block =>
          block.id === usedBlock.id
            ? { ...block, used: false, process: null }
            : block
        ));
      }
    };

    return (
      <div className="bg-gray-900 text-white p-4 rounded-lg">
        <div className="mb-4 flex space-x-2">
          <button
            onClick={allocateMemory}
            className="px-4 py-2 bg-green-600 hover:bg-green-700 rounded"
          >
            Allocate
          </button>
          <button
            onClick={deallocateMemory}
            className="px-4 py-2 bg-red-600 hover:bg-red-700 rounded"
          >
            Deallocate
          </button>
        </div>

        <div className="space-y-3">
          <div className="text-gray-400">Memory Map (KB)</div>
          {memoryBlocks.map((block) => (
            <div
              key={block.id}
              className={`p-3 rounded border ${
                block.used ? 'border-blue-500 bg-blue-500/10' : 'border-green-500 bg-green-500/10'
              }`}
            >
              <div className="flex justify-between items-center">
                <span>Block {block.id}: {block.size}KB</span>
                <span className={`px-2 py-1 rounded text-xs ${
                  block.used ? 'bg-blue-600' : 'bg-green-600'
                }`}>
                  {block.used ? 'Allocated' : 'Free'}
                </span>
              </div>
              {block.process && (
                <div className="text-sm text-gray-400 mt-1">
                  Allocated to: {block.process}
                </div>
              )}
            </div>
          ))}
        </div>

        <div className="mt-4 text-sm text-gray-400">
          Total Allocated: {allocationCount} processes
        </div>
      </div>
    );
  };

  const FileSystemDemo: React.FC = () => {
    const [currentDir, setCurrentDir] = useState('/');
    const [files, setFiles] = useState([
      { name: 'bin', type: 'directory', size: 4096 },
      { name: 'lib', type: 'directory', size: 8192 },
      { name: 'etc', type: 'directory', size: 2048 },
      { name: 'kernel.log', type: 'file', size: 1024 },
      { name: 'README.md', type: 'file', size: 512 },
    ]);

    const [fileContent, setFileContent] = useState('');

    const openFile = (filename: string) => {
      if (filename.endsWith('.log')) {
        setFileContent('kernel.log:\n[INFO] Boot sequence initiated...\n[INFO] Memory initialized\n[INFO] Devices detected\n[INFO] System ready');
      } else if (filename.endsWith('.md')) {
        setFileContent('# MultiOS File System\n\nThis is a demonstration of MultiOS file system operations.\n\n## Features\n- Directories\n- Files\n- Permissions\n- Real-time operations');
      }
    };

    return (
      <div className="bg-gray-900 text-white p-4 rounded-lg">
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
          {/* Directory listing */}
          <div>
            <div className="text-gray-400 mb-2">Current Directory: {currentDir}</div>
            <div className="space-y-1">
              {files.map((file) => (
                <div
                  key={file.name}
                  className="p-2 hover:bg-gray-800 rounded cursor-pointer flex items-center justify-between"
                  onClick={() => {
                    if (file.type === 'file') {
                      openFile(file.name);
                    }
                  }}
                >
                  <div className="flex items-center space-x-2">
                    <span>{file.type === 'directory' ? '📁' : '📄'}</span>
                    <span>{file.name}</span>
                  </div>
                  <span className="text-gray-400 text-sm">{file.size} bytes</span>
                </div>
              ))}
            </div>
          </div>

          {/* File content */}
          <div>
            <div className="text-gray-400 mb-2">File Viewer</div>
            <div className="bg-black p-3 rounded border border-gray-700 h-64 overflow-y-auto">
              <pre className="text-green-400 text-sm whitespace-pre-wrap">
                {fileContent || 'Select a file to view its contents'}
              </pre>
            </div>
          </div>
        </div>
      </div>
    );
  };

  const renderDemo = () => {
    switch (activeDemo) {
      case 'kernel':
        return <KernelDebuggingDemo />;
      case 'process':
        return <ProcessManagementDemo />;
      case 'memory':
        return <MemoryAllocationDemo />;
      case 'filesystem':
        return <FileSystemDemo />;
      default:
        return <KernelDebuggingDemo />;
    }
  };

  return (
    <div className="min-h-screen bg-gray-50 py-12">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Header */}
        <div className="text-center mb-12">
          <h1 className="text-4xl font-bold text-gray-900 mb-4">
            Interactive Operating Systems Demos
          </h1>
          <p className="text-xl text-gray-600 max-w-3xl mx-auto">
            Explore operating systems concepts through hands-on interactive demonstrations.
            Each demo provides real-time visualization of core OS functionality.
          </p>
        </div>

        {/* Demo selector */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
          {demos.map((demo) => {
            const Icon = demo.icon;
            return (
              <button
                key={demo.id}
                onClick={() => setActiveDemo(demo.id)}
                className={`p-4 rounded-lg border-2 transition-all duration-200 text-left ${
                  activeDemo === demo.id
                    ? 'border-blue-500 bg-blue-50'
                    : 'border-gray-200 hover:border-gray-300 bg-white'
                }`}
              >
                <div className={`w-10 h-10 bg-gradient-to-r ${demo.color} rounded-lg flex items-center justify-center mb-3`}>
                  <Icon className="w-5 h-5 text-white" />
                </div>
                <h3 className="font-semibold text-gray-900 mb-1">{demo.title}</h3>
                <p className="text-sm text-gray-600">{demo.description}</p>
              </button>
            );
          })}
        </div>

        {/* Demo content */}
        <div className="bg-white rounded-xl shadow-lg p-6">
          {renderDemo()}
        </div>

        {/* Instructions */}
        <div className="mt-8 bg-blue-50 rounded-lg p-6">
          <h3 className="text-lg font-semibold text-blue-900 mb-2">How to Use</h3>
          <ul className="text-blue-800 space-y-1">
            <li>• Click the play button to start the demonstration</li>
            <li>• Adjust the speed control to change the demonstration pace</li>
            <li>• Use the reset button to restart the demo from the beginning</li>
            <li>• Interact with different components to explore OS concepts</li>
          </ul>
        </div>

        {/* Code examples */}
        <div className="mt-8 bg-gray-900 rounded-lg p-6">
          <h3 className="text-lg font-semibold text-white mb-4">Related Code Examples</h3>
          <div className="bg-black p-4 rounded border border-gray-700">
            <pre className="text-green-400 text-sm">
{`// Kernel Debugging Example
use multios::kernel::{Kernel, BootConfig};

fn debug_kernel_boot() {
    let config = BootConfig::default()
        .with_memory_check(true)
        .with_device_enumeration(true);
        
    let kernel = Kernel::new(config);
    kernel.debug_boot_sequence();
    
    // Step through each boot phase
    kernel.with_debug(|debug| {
        debug.enable_step_mode();
        debug.set_breakpoint(0x1000);
    });
}

// Process Management Example
use multios::process::{Process, Scheduler};

fn demonstrate_scheduling() {
    let mut scheduler = Scheduler::new();
    
    // Create processes with different priorities
    let processes = [
        Process::new("init", Priority::System),
        Process::new("daemon", Priority::Low),
        Process::new("app", Priority::Normal),
    ];
    
    for proc in processes {
        scheduler.add_process(proc);
    }
    
    scheduler.start_scheduling();
}`}
            </pre>
          </div>
        </div>
      </div>
    </div>
  );
};

export default DemosPage;