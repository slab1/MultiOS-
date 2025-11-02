import React, { useState, useEffect } from 'react';
import { Play, Pause, Square, SkipForward, RotateCcw, Bug, Monitor, Terminal, Zap } from 'lucide-react';

interface DebugState {
  isDebugging: boolean;
  isPaused: boolean;
  currentLine: number;
  callStack: StackFrame[];
  variables: VariableState[];
  breakpoints: Breakpoint[];
  watchpoints: Watchpoint[];
  executionPoint: ExecutionPoint | null;
}

interface StackFrame {
  functionName: string;
  filePath: string;
  lineNumber: number;
  localVariables: VariableState[];
}

interface VariableState {
  name: string;
  value: string;
  type: string;
  isModified: boolean;
}

interface Breakpoint {
  id: string;
  filePath: string;
  lineNumber: number;
  isEnabled: boolean;
  condition?: string;
  hitCount: number;
}

interface Watchpoint {
  id: string;
  variableName: string;
  watchType: 'read' | 'write' | 'readwrite';
  isEnabled: boolean;
}

interface ExecutionPoint {
  functionName: string;
  filePath: string;
  lineNumber: number;
  instruction: string;
}

export const DebugInterface: React.FC = () => {
  const [debugState, setDebugState] = useState<DebugState>({
    isDebugging: false,
    isPaused: true,
    currentLine: 15,
    callStack: [],
    variables: [],
    breakpoints: [],
    watchpoints: [],
    executionPoint: null,
  });

  const [selectedTab, setSelectedTab] = useState<'variables' | 'memory' | 'registers' | 'callstack'>('variables');
  const [debugLog, setDebugLog] = useState<string[]>([]);

  // Mock debug data
  useEffect(() => {
    if (debugState.isDebugging) {
      setDebugState(prev => ({
        ...prev,
        callStack: [
          {
            functionName: 'schedule_next_task',
            filePath: 'kernel/src/scheduler/mod.rs',
            lineNumber: 45,
            localVariables: [
              { name: 'current_task', value: '0x7fff1234', type: 'Task*', isModified: false },
              { name: 'next_task', value: '0x7fff1250', type: 'Task*', isModified: true },
              { name: 'priority', value: '5', type: 'i32', isModified: false },
            ]
          },
          {
            functionName: 'timer_handler',
            filePath: 'kernel/src/interrupts/handler.rs',
            lineNumber: 23,
            localVariables: [
              { name: 'interrupt_number', value: '0', type: 'u32', isModified: false },
              { name: 'task_queue', value: '0x80001234', type: 'Queue*', isModified: true },
            ]
          },
          {
            functionName: 'main',
            filePath: 'kernel/src/main.rs',
            lineNumber: 15,
            localVariables: [
              { name: 'initialized', value: 'true', type: 'bool', isModified: true },
            ]
          }
        ],
        variables: [
          { name: 'current_task', value: '0x7fff1234', type: 'Task*', isModified: false },
          { name: 'next_task', value: '0x7fff1250', type: 'Task*', isModified: true },
          { name: 'scheduler_lock', value: 'unlocked', type: 'Mutex', isModified: false },
          { name: 'system_time', value: '1640995200', type: 'u64', isModified: true },
        ],
        executionPoint: {
          functionName: 'schedule_next_task',
          filePath: 'kernel/src/scheduler/mod.rs',
          lineNumber: 45,
          instruction: 'mov rax, [rsp+8]'
        }
      }));

      addDebugLog('Breakpoint hit at schedule_next_task:45');
      addDebugLog('Variable current_task = 0x7fff1234');
      addDebugLog('Variable next_task = 0x7fff1250 (modified)');
    }
  }, [debugState.isDebugging, debugState.currentLine]);

  const addDebugLog = (message: string) => {
    const timestamp = new Date().toLocaleTimeString();
    setDebugLog(prev => [...prev.slice(-100), `[${timestamp}] ${message}`]);
  };

  const handleStartDebug = () => {
    setDebugState(prev => ({
      ...prev,
      isDebugging: true,
      isPaused: false,
    }));
    addDebugLog('Started debugging session');
  };

  const handlePauseDebug = () => {
    setDebugState(prev => ({
      ...prev,
      isPaused: true,
    }));
    addDebugLog('Debugging paused');
  };

  const handleStepOver = () => {
    setDebugState(prev => ({
      ...prev,
      currentLine: prev.currentLine + 1,
    }));
    addDebugLog('Stepped over');
  };

  const handleStepInto = () => {
    setDebugState(prev => ({
      ...prev,
      currentLine: prev.currentLine + 1,
    }));
    addDebugLog('Stepped into function');
  };

  const handleStepOut = () => {
    setDebugState(prev => ({
      ...prev,
      currentLine: Math.max(1, prev.currentLine - 1),
    }));
    addDebugLog('Stepped out of function');
  };

  const handleContinue = () => {
    setDebugState(prev => ({
      ...prev,
      isPaused: false,
    }));
    addDebugLog('Continuing execution...');
  };

  const handleReset = () => {
    setDebugState({
      isDebugging: false,
      isPaused: true,
      currentLine: 15,
      callStack: [],
      variables: [],
      breakpoints: [],
      watchpoints: [],
      executionPoint: null,
    });
    setDebugLog([]);
    addDebugLog('Debug session reset');
  };

  const addBreakpoint = () => {
    const newBreakpoint: Breakpoint = {
      id: Date.now().toString(),
      filePath: 'kernel/src/scheduler/mod.rs',
      lineNumber: debugState.currentLine,
      isEnabled: true,
      hitCount: 0,
    };

    setDebugState(prev => ({
      ...prev,
      breakpoints: [...prev.breakpoints, newBreakpoint]
    }));
    addDebugLog(`Breakpoint added at line ${debugState.currentLine}`);
  };

  const addWatchpoint = (variableName: string) => {
    const newWatchpoint: Watchpoint = {
      id: Date.now().toString(),
      variableName,
      watchType: 'write',
      isEnabled: true,
    };

    setDebugState(prev => ({
      ...prev,
      watchpoints: [...prev.watchpoints, newWatchpoint]
    }));
    addDebugLog(`Watchpoint added for variable ${variableName}`);
  };

  const toggleBreakpoint = (breakpointId: string) => {
    setDebugState(prev => ({
      ...prev,
      breakpoints: prev.breakpoints.map(bp => 
        bp.id === breakpointId ? { ...bp, isEnabled: !bp.isEnabled } : bp
      )
    }));
  };

  return (
    <div className="h-screen flex flex-col bg-gray-50">
      {/* Debug Header */}
      <div className="bg-white border-b border-gray-200 px-6 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-4">
            <Bug className="w-6 h-6 text-blue-600" />
            <h1 className="text-xl font-bold text-gray-900">Debug Interface</h1>
            <div className="flex items-center space-x-2">
              <div className={`w-2 h-2 rounded-full ${debugState.isDebugging ? 'bg-green-400 animate-pulse' : 'bg-gray-400'}`}></div>
              <span className="text-sm text-gray-600">
                {debugState.isDebugging ? 'Debugging Active' : 'Debugging Inactive'}
              </span>
            </div>
          </div>

          <div className="flex items-center space-x-4">
            {/* Debug Controls */}
            <div className="flex items-center space-x-2">
              {!debugState.isDebugging ? (
                <button
                  onClick={handleStartDebug}
                  className="flex items-center space-x-2 px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700"
                >
                  <Play className="w-4 h-4" />
                  <span>Start Debug</span>
                </button>
              ) : (
                <>
                  {debugState.isPaused ? (
                    <button
                      onClick={handleContinue}
                      className="flex items-center space-x-2 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
                    >
                      <Play className="w-4 h-4" />
                      <span>Continue</span>
                    </button>
                  ) : (
                    <button
                      onClick={handlePauseDebug}
                      className="flex items-center space-x-2 px-4 py-2 bg-yellow-600 text-white rounded-md hover:bg-yellow-700"
                    >
                      <Pause className="w-4 h-4" />
                      <span>Pause</span>
                    </button>
                  )}

                  <button
                    onClick={handleStepOver}
                    disabled={!debugState.isPaused}
                    className="flex items-center space-x-2 px-3 py-2 bg-gray-600 text-white rounded-md hover:bg-gray-700 disabled:opacity-50"
                    title="Step Over (F10)"
                  >
                    <SkipForward className="w-4 h-4" />
                    <span>Step Over</span>
                  </button>

                  <button
                    onClick={handleStepInto}
                    disabled={!debugState.isPaused}
                    className="flex items-center space-x-2 px-3 py-2 bg-gray-600 text-white rounded-md hover:bg-gray-700 disabled:opacity-50"
                    title="Step Into (F11)"
                  >
                    <SkipForward className="w-4 h-4" />
                    <span>Step Into</span>
                  </button>

                  <button
                    onClick={handleStepOut}
                    disabled={!debugState.isPaused}
                    className="flex items-center space-x-2 px-3 py-2 bg-gray-600 text-white rounded-md hover:bg-gray-700 disabled:opacity-50"
                    title="Step Out (Shift+F11)"
                  >
                    <RotateCcw className="w-4 h-4" />
                    <span>Step Out</span>
                  </button>

                  <button
                    onClick={handleReset}
                    className="flex items-center space-x-2 px-3 py-2 bg-red-600 text-white rounded-md hover:bg-red-700"
                    title="Reset"
                  >
                    <Square className="w-4 h-4" />
                    <span>Stop</span>
                  </button>
                </>
              )}
            </div>
          </div>
        </div>

        {/* Execution Status */}
        {debugState.executionPoint && (
          <div className="mt-4 flex items-center space-x-6 text-sm">
            <div className="flex items-center space-x-2">
              <Monitor className="w-4 h-4 text-blue-600" />
              <span className="text-gray-600">Current:</span>
              <span className="font-mono">{debugState.executionPoint.functionName}()</span>
              <span className="text-gray-400">at</span>
              <span className="font-mono">{debugState.executionPoint.filePath}:{debugState.executionPoint.lineNumber}</span>
            </div>
            <div className="flex items-center space-x-2">
              <Zap className="w-4 h-4 text-purple-600" />
              <span className="text-gray-600">Instruction:</span>
              <span className="font-mono text-purple-700">{debugState.executionPoint.instruction}</span>
            </div>
          </div>
        )}
      </div>

      <div className="flex-1 flex overflow-hidden">
        {/* Code Viewer with Debugging */}
        <div className="flex-1 bg-white border-r border-gray-200 flex flex-col">
          <div className="px-4 py-2 bg-gray-50 border-b border-gray-200">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-medium text-gray-900">Code - kernel/src/scheduler/mod.rs</h3>
              <div className="flex items-center space-x-2">
                <button
                  onClick={addBreakpoint}
                  className="text-xs px-2 py-1 bg-red-100 text-red-700 rounded hover:bg-red-200"
                >
                  Add Breakpoint
                </button>
              </div>
            </div>
          </div>

          {/* Code with Line Numbers and Debug Indicators */}
          <div className="flex-1 overflow-auto p-4">
            <div className="font-mono text-sm leading-relaxed">
              {Array.from({ length: 50 }, (_, i) => i + 1).map(lineNumber => (
                <div
                  key={lineNumber}
                  className={`flex hover:bg-gray-50 ${
                    debugState.executionPoint?.lineNumber === lineNumber 
                      ? 'bg-yellow-100 border-l-4 border-yellow-500' 
                      : ''
                  } ${
                    debugState.breakpoints.some(bp => bp.lineNumber === lineNumber && bp.isEnabled)
                      ? 'bg-red-50 border-l-4 border-red-500'
                      : ''
                  }`}
                >
                  <div className="w-12 text-right pr-4 text-gray-400 select-none">
                    {lineNumber}
                  </div>
                  <div className="flex-1">
                    {lineNumber === 45 && debugState.isDebugging ? (
                      <span className="bg-yellow-200">
                        {`let next_task = select_next_task(); // Debug: Variable 'next_task' will be modified`}
                      </span>
                    ) : lineNumber === 40 ? (
                      <span className="text-gray-500">
                        {`// Check if preemption is enabled`}
                      </span>
                    ) : lineNumber === 41 ? (
                      <span className="text-gray-500">
                        {`if !current_task.preemption_enabled {`}
                      </span>
                    ) : lineNumber === 42 ? (
                      <span className="text-gray-500">
                        {`    return None; // No task switching needed`}
                      </span>
                    ) : lineNumber === 43 ? (
                      <span className="text-gray-500">
                        {`}`}
                      </span>
                    ) : lineNumber === 44 ? (
                      <span className="text-gray-500">
                        {``}
                      </span>
                    ) : lineNumber === 46 ? (
                      <span className="text-gray-500">
                        {`// Update task statistics`}
                      </span>
                    ) : lineNumber === 47 ? (
                      <span className="text-gray-500">
                        {`current_task.update_execution_stats();`}
                      </span>
                    ) : (
                      <span className="text-gray-700">
                        {`// Line ${lineNumber} content...`}
                      </span>
                    )}
                  </div>
                  {/* Debug Line Indicator */}
                  {debugState.executionPoint?.lineNumber === lineNumber && (
                    <div className="w-4 flex items-center justify-center">
                      <div className="w-2 h-2 bg-yellow-500 rounded-full animate-pulse"></div>
                    </div>
                  )}
                  {/* Breakpoint Indicator */}
                  {debugState.breakpoints.some(bp => bp.lineNumber === lineNumber && bp.isEnabled) && (
                    <div className="w-4 flex items-center justify-center">
                      <div className="w-2 h-2 bg-red-500 rounded-full"></div>
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Debug Panels */}
        <div className="w-96 bg-gray-50 border-l border-gray-200 flex flex-col">
          {/* Tab Navigation */}
          <div className="flex border-b border-gray-200">
            {[
              { id: 'variables', label: 'Variables', icon: '📊' },
              { id: 'callstack', label: 'Call Stack', icon: '🔄' },
              { id: 'memory', label: 'Memory', icon: '💾' },
              { id: 'registers', label: 'Registers', icon: '⚡' },
            ].map(tab => (
              <button
                key={tab.id}
                onClick={() => setSelectedTab(tab.id as any)}
                className={`flex-1 py-3 px-2 text-sm font-medium border-b-2 transition-colors duration-200 ${
                  selectedTab === tab.id
                    ? 'border-blue-500 text-blue-600 bg-white'
                    : 'border-transparent text-gray-500 hover:text-gray-700'
                }`}
              >
                <div className="flex items-center justify-center space-x-1">
                  <span>{tab.icon}</span>
                  <span>{tab.label}</span>
                </div>
              </button>
            ))}
          </div>

          {/* Tab Content */}
          <div className="flex-1 overflow-auto">
            {selectedTab === 'variables' && (
              <div className="p-4">
                <h3 className="text-sm font-medium text-gray-900 mb-3">Local Variables</h3>
                <div className="space-y-2">
                  {debugState.variables.map((variable, index) => (
                    <div
                      key={index}
                      className="flex items-center justify-between p-2 bg-white rounded border hover:bg-gray-50 cursor-pointer"
                      onClick={() => addWatchpoint(variable.name)}
                      title="Click to add watchpoint"
                    >
                      <div>
                        <div className="flex items-center space-x-2">
                          <span className="text-sm font-mono text-gray-900">{variable.name}</span>
                          {variable.isModified && (
                            <span className="text-xs bg-orange-100 text-orange-700 px-1 rounded">modified</span>
                          )}
                        </div>
                        <div className="text-xs text-gray-500">{variable.type}</div>
                      </div>
                      <div className="text-right">
                        <div className="text-sm font-mono text-blue-600">{variable.value}</div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {selectedTab === 'callstack' && (
              <div className="p-4">
                <h3 className="text-sm font-medium text-gray-900 mb-3">Call Stack</h3>
                <div className="space-y-2">
                  {debugState.callStack.map((frame, index) => (
                    <div
                      key={index}
                      className={`p-3 bg-white rounded border ${
                        index === 0 ? 'border-blue-500 bg-blue-50' : 'border-gray-200'
                      }`}
                    >
                      <div className="flex items-center justify-between mb-2">
                        <span className="text-sm font-medium text-gray-900">{frame.functionName}()</span>
                        {index === 0 && (
                          <span className="text-xs bg-blue-100 text-blue-700 px-2 py-1 rounded">Current</span>
                        )}
                      </div>
                      <div className="text-xs text-gray-600 mb-2">
                        {frame.filePath}:{frame.lineNumber}
                      </div>
                      <div className="text-xs text-gray-500">
                        {frame.localVariables.length} local variables
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {selectedTab === 'memory' && (
              <div className="p-4">
                <h3 className="text-sm font-medium text-gray-900 mb-3">Memory View</h3>
                <div className="bg-black text-green-400 p-3 rounded font-mono text-sm">
                  <div className="mb-2">Address: 0x7fff1234</div>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <div>00 00 00 00 01 23 45 67</div>
                      <div>89 ab cd ef 00 11 22 33</div>
                      <div>44 55 66 77 88 99 aa bb</div>
                      <div>cc dd ee ff 00 00 00 00</div>
                    </div>
                    <div>
                      <div className="text-gray-400">null pointer</div>
                      <div className="text-blue-400">Task struct</div>
                      <div className="text-yellow-400">next pointer</div>
                      <div className="text-purple-400">padding</div>
                    </div>
                  </div>
                </div>
              </div>
            )}

            {selectedTab === 'registers' && (
              <div className="p-4">
                <h3 className="text-sm font-medium text-gray-900 mb-3">CPU Registers</h3>
                <div className="grid grid-cols-2 gap-2 text-sm font-mono">
                  <div className="bg-white p-2 rounded border">
                    <div className="text-gray-600">RAX</div>
                    <div>0x0000000000000001</div>
                  </div>
                  <div className="bg-white p-2 rounded border">
                    <div className="text-gray-600">RBX</div>
                    <div>0x00007fff12345678</div>
                  </div>
                  <div className="bg-white p-2 rounded border">
                    <div className="text-gray-600">RCX</div>
                    <div>0x0000000000000040</div>
                  </div>
                  <div className="bg-white p-2 rounded border">
                    <div className="text-gray-600">RDX</div>
                    <div>0x0000000000000000</div>
                  </div>
                  <div className="bg-white p-2 rounded border">
                    <div className="text-gray-600">RSI</div>
                    <div>0x00007fff87654321</div>
                  </div>
                  <div className="bg-white p-2 rounded border">
                    <div className="text-gray-600">RDI</div>
                    <div>0x0000000000000000</div>
                  </div>
                  <div className="bg-white p-2 rounded border">
                    <div className="text-gray-600">RSP</div>
                    <div>0x00007fffffffe8a0</div>
                  </div>
                  <div className="bg-white p-2 rounded border">
                    <div className="text-gray-600">RBP</div>
                    <div>0x00007fffffffe900</div>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Debug Console */}
      <div className="h-40 bg-black text-green-400 border-t border-gray-200 flex flex-col">
        <div className="px-4 py-2 bg-gray-900 border-b border-gray-700 flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <Terminal className="w-4 h-4" />
            <span className="text-sm font-medium">Debug Console</span>
          </div>
          <button
            onClick={() => setDebugLog([])}
            className="text-xs text-gray-400 hover:text-gray-200"
          >
            Clear
          </button>
        </div>
        <div className="flex-1 overflow-auto p-4 font-mono text-sm">
          {debugLog.map((log, index) => (
            <div key={index} className="mb-1">
              {log}
            </div>
          ))}
          <div className="flex items-center space-x-2">
            <span className="text-green-400">&gt;</span>
            <input
              type="text"
              placeholder="Enter debugger command..."
              className="flex-1 bg-transparent text-green-400 outline-none"
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  const command = (e.target as HTMLInputElement).value;
                  addDebugLog(`> ${command}`);
                  (e.target as HTMLInputElement).value = '';
                  // Mock command processing
                  if (command === 'help') {
                    addDebugLog('Available commands: help, info, print, watch, breakpoint, continue, step');
                  }
                }
              }}
            />
          </div>
        </div>
      </div>
    </div>
  );
};
