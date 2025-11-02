import React from 'react';
import { GitBranch, ArrowRight, Circle } from 'lucide-react';
import type { CallGraph as CallGraphType, CallGraphNode, CallGraphEdge } from '../App';

interface CallGraphProps {
  data: CallGraphType | null;
  selectedFunction: string | null;
  onFunctionSelect: (functionName: string | null) => void;
  isLoading: boolean;
}

export const CallGraph: React.FC<CallGraphProps> = ({
  data,
  selectedFunction,
  onFunctionSelect,
  isLoading
}) => {
  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-4"></div>
          <p className="text-gray-600">Generating call graph...</p>
        </div>
      </div>
    );
  }

  if (!data) {
    return (
      <div className="flex items-center justify-center h-full text-gray-500">
        <div className="text-center">
          <GitBranch className="w-12 h-12 mx-auto mb-4 text-gray-300" />
          <p>No call graph data available</p>
        </div>
      </div>
    );
  }

  const getNodeColor = (node: CallGraphNode) => {
    if (node.is_entry_point) return 'bg-green-100 border-green-500 text-green-800';
    if (node.is_extern) return 'bg-purple-100 border-purple-500 text-purple-800';
    if (node.performance_impact === 'critical') return 'bg-red-100 border-red-500 text-red-800';
    if (node.performance_impact === 'high') return 'bg-orange-100 border-orange-500 text-orange-800';
    return 'bg-blue-100 border-blue-500 text-blue-800';
  };

  const getEdgeStyle = (edge: CallGraphEdge) => {
    let style = 'stroke-2 ';
    if (edge.is_recursive) {
      style += 'stroke-red-400 ';
    } else if (edge.is_system_call) {
      style += 'stroke-purple-400 ';
    } else if (edge.is_cross_file) {
      style += 'stroke-orange-400 ';
    } else {
      style += 'stroke-blue-400 ';
    }
    return style;
  };

  const getEdgeLabel = (edge: CallGraphEdge) => {
    if (edge.is_recursive) return 'recursive';
    if (edge.is_system_call) return 'syscall';
    if (edge.is_cross_file) return 'external';
    return 'call';
  };

  return (
    <div className="h-full flex">
      {/* Call Graph Visualization */}
      <div className="flex-1 bg-white border-r border-gray-200">
        <div className="flex items-center justify-between px-4 py-3 bg-gray-50 border-b border-gray-200">
          <h3 className="text-sm font-medium text-gray-900">Function Call Graph</h3>
          <div className="flex items-center space-x-4 text-xs text-gray-500">
            <div className="flex items-center">
              <Circle className="w-3 h-3 mr-1 fill-current text-blue-500" />
              Internal
            </div>
            <div className="flex items-center">
              <Circle className="w-3 h-3 mr-1 fill-current text-green-500" />
              Entry Point
            </div>
            <div className="flex items-center">
              <Circle className="w-3 h-3 mr-1 fill-current text-purple-500" />
              External
            </div>
          </div>
        </div>

        <div className="call-graph-container h-full overflow-auto p-6">
          <svg width="800" height="600" className="mx-auto">
            {/* Render edges first (behind nodes) */}
            {data.edges.map((edge, index) => {
              const fromNode = data.nodes.find(n => n.id === edge.from);
              const toNode = data.nodes.find(n => n.id === edge.to);
              
              if (!fromNode || !toNode) return null;

              // Simple layout: arrange nodes in rows based on complexity
              const fromX = (data.nodes.indexOf(fromNode) % 3) * 250 + 125;
              const fromY = Math.floor(data.nodes.indexOf(fromNode) / 3) * 100 + 50;
              const toX = (data.nodes.indexOf(toNode) % 3) * 250 + 125;
              const toY = Math.floor(data.nodes.indexOf(toNode) / 3) * 100 + 50;

              return (
                <g key={`edge-${index}`}>
                  <line
                    x1={fromX}
                    y1={fromY + 20}
                    x2={toX}
                    y2={toY - 20}
                    className={getEdgeStyle(edge)}
                    markerEnd="url(#arrowhead)"
                  />
                  {/* Edge label */}
                  <text
                    x={(fromX + toX) / 2}
                    y={(fromY + toY) / 2}
                    className="text-xs fill-gray-600"
                    textAnchor="middle"
                  >
                    {getEdgeLabel(edge)}
                  </text>
                </g>
              );
            })}

            {/* Arrow marker definition */}
            <defs>
              <marker
                id="arrowhead"
                markerWidth="10"
                markerHeight="7"
                refX="9"
                refY="3.5"
                orient="auto"
              >
                <polygon
                  points="0 0, 10 3.5, 0 7"
                  className="fill-current text-blue-400"
                />
              </marker>
            </defs>

            {/* Render nodes */}
            {data.nodes.map((node, index) => {
              const x = (index % 3) * 250 + 125;
              const y = Math.floor(index / 3) * 100 + 50;
              const isSelected = selectedFunction === node.function_name;

              return (
                <g key={node.id}>
                  <circle
                    cx={x}
                    cy={y}
                    r="30"
                    className={`${getNodeColor(node)} border-2 cursor-pointer transition-all duration-200 ${
                      isSelected ? 'stroke-blue-600 stroke-3' : ''
                    }`}
                    onClick={() => onFunctionSelect(node.function_name)}
                  />
                  
                  <text
                    x={x}
                    y={y + 5}
                    className="text-xs font-medium text-center"
                    textAnchor="middle"
                  >
                    {node.function_name}
                  </text>
                  
                  {/* Complexity indicator */}
                  <text
                    x={x}
                    y={y + 35}
                    className="text-xs text-gray-500"
                    textAnchor="middle"
                  >
                    C:{node.complexity}
                  </text>
                </g>
              );
            })}
          </svg>
        </div>
      </div>

      {/* Information Panel */}
      <div className="w-80 bg-gray-50 border-l border-gray-200 overflow-auto">
        <div className="p-4 border-b border-gray-200">
          <h3 className="text-sm font-medium text-gray-900 mb-3">Graph Statistics</h3>
          
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-600">Total Functions:</span>
              <span className="font-medium">{data.nodes.length}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600">Call Relations:</span>
              <span className="font-medium">{data.edges.length}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600">Complexity Score:</span>
              <span className="font-medium">{data.complexity_score}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600">Entry Points:</span>
              <span className="font-medium">{data.entry_points.length}</span>
            </div>
          </div>
        </div>

        {selectedFunction && (
          <div className="p-4 border-b border-gray-200">
            <h3 className="text-sm font-medium text-gray-900 mb-3">Function Details</h3>
            
            {(() => {
              const node = data.nodes.find(n => n.function_name === selectedFunction);
              if (!node) return null;

              return (
                <div className="space-y-3">
                  <div>
                    <div className="text-sm font-medium text-gray-900">{node.function_name}</div>
                    <div className="text-xs text-gray-600">{node.file_path}:{node.line_number}</div>
                  </div>

                  <div className="grid grid-cols-2 gap-3 text-sm">
                    <div>
                      <div className="text-xs text-gray-500">Complexity</div>
                      <div className="font-medium">{node.complexity}</div>
                    </div>
                    <div>
                      <div className="text-xs text-gray-500">Call Count</div>
                      <div className="font-medium">{node.call_count}</div>
                    </div>
                  </div>

                  <div>
                    <div className="text-xs text-gray-500 mb-1">Properties</div>
                    <div className="flex flex-wrap gap-1">
                      {node.is_entry_point && (
                        <span className="px-2 py-1 bg-green-100 text-green-700 text-xs rounded">
                          Entry Point
                        </span>
                      )}
                      {node.is_extern && (
                        <span className="px-2 py-1 bg-purple-100 text-purple-700 text-xs rounded">
                          External
                        </span>
                      )}
                      <span className={`px-2 py-1 text-xs rounded ${
                        node.performance_impact === 'critical' ? 'bg-red-100 text-red-700' :
                        node.performance_impact === 'high' ? 'bg-orange-100 text-orange-700' :
                        'bg-blue-100 text-blue-700'
                      }`}>
                        {node.performance_impact} impact
                      </span>
                    </div>
                  </div>

                  {node.educational_description && (
                    <div>
                      <div className="text-xs text-gray-500 mb-1">Educational Context</div>
                      <div className="text-sm text-gray-700">{node.educational_description}</div>
                    </div>
                  )}
                </div>
              );
            })()}
          </div>
        )}

        {/* Call Depth Distribution */}
        <div className="p-4">
          <h3 className="text-sm font-medium text-gray-900 mb-3">Call Depth Distribution</h3>
          
          <div className="space-y-2">
            {Object.entries(data.call_depth_distribution).map(([depth, count]) => (
              <div key={depth} className="flex items-center justify-between">
                <span className="text-sm text-gray-600">Depth {depth}:</span>
                <div className="flex items-center">
                  <div 
                    className="w-16 h-2 bg-blue-200 rounded-full mr-2"
                    style={{ width: `${Math.min((count / Math.max(...Object.values(data.call_depth_distribution))) * 64, 64)}px` }}
                  ></div>
                  <span className="text-sm font-medium">{count}</span>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};
