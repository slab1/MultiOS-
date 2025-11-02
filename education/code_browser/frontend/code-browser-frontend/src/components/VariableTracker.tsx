import React, { useState, useMemo } from 'react';
import { Layers, ArrowRight, GitBranch, Info, AlertTriangle } from 'lucide-react';
import type { CodeAnalysis, VariableInfo, DataFlow } from '../App';

interface VariableTrackerProps {
  code: string;
  analysis: CodeAnalysis | null;
  selectedFunction: string | null;
  isLoading: boolean;
}

interface DataFlowStep {
  line: number;
  operation: string;
  from: string;
  to: string;
  description: string;
}

export const VariableTracker: React.FC<VariableTrackerProps> = ({
  code,
  analysis,
  selectedFunction,
  isLoading
}) => {
  const [selectedVariable, setSelectedVariable] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<'list' | 'flow'>('list');

  const codeLines = useMemo(() => code.split('\n'), [code]);

  // Mock data flow for demonstration
  const mockDataFlow: { [key: string]: DataFlowStep[] } = {
    'current_task': [
      { line: 15, operation: 'declare', from: '', to: 'current_task', description: 'Variable declared and initialized' },
      { line: 20, operation: 'read', from: 'current_task', to: 'scheduler', description: 'Task state read by scheduler' },
      { line: 25, operation: 'write', from: 'scheduler', to: 'current_task', description: 'Task state updated' },
    ],
    'next_task': [
      { line: 16, operation: 'declare', from: '', to: 'next_task', description: 'Next task variable declared' },
      { line: 21, operation: 'write', from: 'select_next_task()', to: 'next_task', description: 'Next task assigned' },
      { line: 26, operation: 'read', from: 'next_task', to: 'load_context', description: 'Context loaded for next task' },
    ],
    'syscall_num': [
      { line: 32, operation: 'declare', from: '', to: 'syscall_num', description: 'System call number extracted' },
      { line: 33, operation: 'read', from: 'syscall_num', to: 'match', description: 'System call number used in dispatch' },
    ]
  };

  const getVariableFlow = (variableName: string): DataFlowStep[] => {
    return mockDataFlow[variableName] || [];
  };

  const getVariableUsage = (variableName: string) => {
    if (!analysis) return [];
    
    return analysis.variables.filter(v => v.name === variableName);
  };

  const getOperationColor = (operation: string) => {
    switch (operation.toLowerCase()) {
      case 'declare': return 'text-green-600 bg-green-100';
      case 'read': return 'text-blue-600 bg-blue-100';
      case 'write': return 'text-orange-600 bg-orange-100';
      case 'modify': return 'text-purple-600 bg-purple-100';
      default: return 'text-gray-600 bg-gray-100';
    }
  };

  const getOperationIcon = (operation: string) => {
    switch (operation.toLowerCase()) {
      case 'declare': return '•';
      case 'read': return '←';
      case 'write': return '→';
      case 'modify': return '↔';
      default: return '•';
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-4"></div>
          <p className="text-gray-600">Analyzing variable usage...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex">
      {/* Variables List */}
      <div className="flex-1 bg-white border-r border-gray-200">
        <div className="flex items-center justify-between px-4 py-3 bg-gray-50 border-b border-gray-200">
          <h3 className="text-sm font-medium text-gray-900">Variable Tracker</h3>
          
          <div className="flex items-center space-x-2">
            <button
              onClick={() => setViewMode('list')}
              className={`px-3 py-1 text-sm rounded ${
                viewMode === 'list' 
                  ? 'bg-blue-100 text-blue-700' 
                  : 'text-gray-600 hover:text-gray-900'
              }`}
            >
              List
            </button>
            <button
              onClick={() => setViewMode('flow')}
              className={`px-3 py-1 text-sm rounded ${
                viewMode === 'flow' 
                  ? 'bg-blue-100 text-blue-700' 
                  : 'text-gray-600 hover:text-gray-900'
              }`}
            >
              Data Flow
            </button>
          </div>
        </div>

        <div className="overflow-auto h-full p-4">
          {analysis?.variables.map((variable, index) => (
            <div
              key={index}
              className={`mb-3 p-3 rounded-lg border cursor-pointer transition-colors duration-200 ${
                selectedVariable === variable.name
                  ? 'bg-blue-50 border-blue-200'
                  : 'bg-white border-gray-200 hover:bg-gray-50'
              }`}
              onClick={() => setSelectedVariable(selectedVariable === variable.name ? null : variable.name)}
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-2">
                  <Layers className="w-4 h-4 text-gray-500" />
                  <span className="font-medium text-sm text-gray-900">{variable.name}</span>
                </div>
                <div className="flex items-center space-x-2">
                  {variable.is_mutable && (
                    <span className="px-2 py-1 bg-yellow-100 text-yellow-700 text-xs rounded">
                      mutable
                    </span>
                  )}
                  <span className="text-xs text-gray-500">{variable.var_type}</span>
                </div>
              </div>
              
              <div className="mt-2 text-xs text-gray-600">
                Line {variable.line} • {variable.scope} scope
                {variable.initialized_value && (
                  <div className="mt-1 font-mono text-gray-500">
                    = {variable.initialized_value}
                  </div>
                )}
              </div>

              {selectedVariable === variable.name && (
                <div className="mt-3 pt-3 border-t border-gray-200">
                  <h4 className="text-sm font-medium text-gray-900 mb-2">Variable Analysis</h4>
                  
                  <div className="space-y-2 text-sm">
                    <div>
                      <span className="text-gray-600">Declaration:</span>
                      <span className="ml-2 font-mono text-gray-800">
                        {variable.is_mutable ? 'let mut' : 'let'} {variable.name}: {variable.var_type}
                      </span>
                    </div>
                    
                    <div>
                      <span className="text-gray-600">Scope:</span>
                      <span className="ml-2 text-gray-800">{variable.scope}</span>
                    </div>

                    <div>
                      <span className="text-gray-600">Mutability:</span>
                      <span className="ml-2 text-gray-800">
                        {variable.is_mutable ? 'Mutable (can be modified)' : 'Immutable (read-only)'}
                      </span>
                    </div>
                  </div>

                  {viewMode === 'flow' && (
                    <div className="mt-4">
                      <h5 className="text-xs font-medium text-gray-700 mb-2">Data Flow Trace</h5>
                      <div className="space-y-2">
                        {getVariableFlow(variable.name).map((step, stepIndex) => (
                          <div key={stepIndex} className="flex items-center space-x-3 text-xs">
                            <span className="w-8 text-center font-mono text-gray-500">{step.line}</span>
                            <span className={`px-2 py-1 rounded text-xs ${getOperationColor(step.operation)}`}>
                              {getOperationIcon(step.operation)} {step.operation}
                            </span>
                            <div className="flex-1">
                              <div className="text-gray-700">{step.description}</div>
                              {step.from && step.to && (
                                <div className="text-gray-500 font-mono">
                                  {step.from} → {step.to}
                                </div>
                              )}
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              )}
            </div>
          )) || (
            <div className="text-center text-gray-500 py-8">
              <Layers className="w-8 h-8 mx-auto mb-2 text-gray-300" />
              <p>No variables found</p>
            </div>
          )}
        </div>
      </div>

      {/* Analysis Panel */}
      <div className="w-80 bg-gray-50 border-l border-gray-200 overflow-auto">
        <div className="p-4 border-b border-gray-200">
          <h3 className="text-sm font-medium text-gray-900 mb-3">Variable Analysis</h3>
          
          {selectedVariable ? (
            <div>
              <div className="flex items-center space-x-2 mb-3">
                <Layers className="w-4 h-4 text-blue-500" />
                <span className="font-medium text-gray-900">{selectedVariable}</span>
              </div>

              {(() => {
                const variable = analysis?.variables.find(v => v.name === selectedVariable);
                const flow = getVariableFlow(selectedVariable);
                
                if (!variable) return null;

                return (
                  <div className="space-y-4">
                    {/* Basic Info */}
                    <div className="bg-white p-3 rounded border">
                      <h4 className="text-xs font-medium text-gray-700 mb-2">Basic Information</h4>
                      <div className="space-y-1 text-xs">
                        <div className="flex justify-between">
                          <span className="text-gray-600">Type:</span>
                          <span className="font-mono">{variable.var_type}</span>
                        </div>
                        <div className="flex justify-between">
                          <span className="text-gray-600">Line:</span>
                          <span>{variable.line}</span>
                        </div>
                        <div className="flex justify-between">
                          <span className="text-gray-600">Scope:</span>
                          <span>{variable.scope}</span>
                        </div>
                        <div className="flex justify-between">
                          <span className="text-gray-600">Mutable:</span>
                          <span>{variable.is_mutable ? 'Yes' : 'No'}</span>
                        </div>
                      </div>
                    </div>

                    {/* Data Flow Summary */}
                    <div className="bg-white p-3 rounded border">
                      <h4 className="text-xs font-medium text-gray-700 mb-2">Data Flow Summary</h4>
                      <div className="text-xs text-gray-600">
                        <div className="mb-2">
                          <strong>{flow.length}</strong> operations tracked
                        </div>
                        <div className="space-y-1">
                          {Array.from(new Set(flow.map(s => s.operation))).map(op => {
                            const count = flow.filter(s => s.operation === op).length;
                            return (
                              <div key={op} className="flex justify-between">
                                <span>{op}:</span>
                                <span>{count}</span>
                              </div>
                            );
                          })}
                        </div>
                      </div>
                    </div>

                    {/* Educational Context */}
                    <div className="bg-blue-50 p-3 rounded border border-blue-200">
                      <h4 className="text-xs font-medium text-blue-800 mb-2">Educational Note</h4>
                      <p className="text-xs text-blue-700">
                        {selectedVariable === 'current_task' && 
                          "Current task pointer is crucial for context switching. It's accessed frequently by the scheduler."}
                        {selectedVariable === 'next_task' && 
                          "Next task pointer holds the process that will run next. It's critical for scheduling decisions."}
                        {selectedVariable === 'syscall_num' && 
                          "System call number is extracted from registers and used to dispatch to the appropriate handler."}
                        {![ 'current_task', 'next_task', 'syscall_num' ].includes(selectedVariable) && 
                          "Understanding variable scope and mutability is important for writing safe and efficient code."}
                      </p>
                    </div>

                    {/* Potential Issues */}
                    <div className="bg-yellow-50 p-3 rounded border border-yellow-200">
                      <h4 className="text-xs font-medium text-yellow-800 mb-2">Analysis</h4>
                      <div className="space-y-1 text-xs text-yellow-700">
                        {variable.is_mutable && (
                          <div className="flex items-center space-x-1">
                            <Info className="w-3 h-3" />
                            <span>Mutable variable - ensure proper synchronization if shared</span>
                          </div>
                        )}
                        {variable.scope === 'global' && (
                          <div className="flex items-center space-x-1">
                            <AlertTriangle className="w-3 h-3" />
                            <span>Global variable - consider encapsulation</span>
                          </div>
                        )}
                        {flow.length > 5 && (
                          <div className="flex items-center space-x-1">
                            <GitBranch className="w-3 h-3" />
                            <span>High usage count - may be optimization target</span>
                          </div>
                        )}
                      </div>
                    </div>
                  </div>
                );
              })()}
            </div>
          ) : (
            <div className="text-center text-gray-500 py-8">
              <Layers className="w-8 h-8 mx-auto mb-2 text-gray-300" />
              <p className="text-sm">Select a variable to view detailed analysis</p>
            </div>
          )}
        </div>

        {/* Variable Statistics */}
        {analysis && (
          <div className="p-4">
            <h3 className="text-sm font-medium text-gray-900 mb-3">Variable Statistics</h3>
            
            <div className="space-y-3">
              <div className="bg-white p-3 rounded border">
                <div className="text-sm text-gray-600 mb-2">Distribution by Scope</div>
                {(() => {
                  const scopeCounts = analysis.variables.reduce((acc, v) => {
                    acc[v.scope] = (acc[v.scope] || 0) + 1;
                    return acc;
                  }, {} as { [key: string]: number });

                  return Object.entries(scopeCounts).map(([scope, count]) => (
                    <div key={scope} className="flex justify-between text-sm">
                      <span className="text-gray-600">{scope}:</span>
                      <span className="font-medium">{count}</span>
                    </div>
                  ));
                })()}
              </div>

              <div className="bg-white p-3 rounded border">
                <div className="text-sm text-gray-600 mb-2">Mutability</div>
                <div className="space-y-1 text-sm">
                  <div className="flex justify-between">
                    <span className="text-gray-600">Mutable:</span>
                    <span className="font-medium">
                      {analysis.variables.filter(v => v.is_mutable).length}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-600">Immutable:</span>
                    <span className="font-medium">
                      {analysis.variables.filter(v => !v.is_mutable).length}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
