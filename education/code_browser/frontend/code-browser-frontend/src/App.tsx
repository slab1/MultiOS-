import React from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { CodeBrowser } from './components/CodeBrowser';
import { LandingPage } from './components/LandingPage';
import { EducationalModules } from './components/EducationalModules';
import { PerformanceDashboard } from './components/PerformanceDashboard';
import { Navigation } from './components/Navigation';
import { DebugInterface } from './components/DebugInterface';
import './App.css';

export interface CodeAnalysis {
  syntax_highlighting: SyntaxHighlight[];
  functions: FunctionInfo[];
  variables: VariableInfo[];
  types: TypeInfo[];
  imports: ImportInfo[];
  inline_explanations: InlineExplanation[];
  complexity_score: number;
  educational_comments: EducationalComment[];
}

export interface SyntaxHighlight {
  line: number;
  start_col: number;
  end_col: number;
  token_type: string;
  token_value: string;
}

export interface FunctionInfo {
  name: string;
  signature: string;
  start_line: number;
  end_line: number;
  parameters: string[];
  return_type: string;
  complexity: number;
  educational_description?: string;
}

export interface VariableInfo {
  name: string;
  var_type: string;
  line: number;
  scope: string;
  is_mutable: boolean;
  initialized_value?: string;
}

export interface TypeInfo {
  name: string;
  definition: string;
  line: number;
  fields: TypeField[];
  is_builtin: boolean;
}

export interface TypeField {
  name: string;
  field_type: string;
  is_public: boolean;
}

export interface ImportInfo {
  module: string;
  items: string[];
  is_external: boolean;
  line: number;
}

export interface InlineExplanation {
  line: number;
  start_col: number;
  end_col: number;
  explanation: string;
  complexity_level: string;
  related_concepts: string[];
}

export interface EducationalComment {
  line: number;
  comment: string;
  category: string;
  difficulty_level: string;
  learning_objectives: string[];
}

export interface CallGraph {
  nodes: CallGraphNode[];
  edges: CallGraphEdge[];
  entry_points: string[];
  complexity_score: number;
  call_depth_distribution: { [key: string]: number };
}

export interface CallGraphNode {
  id: string;
  function_name: string;
  file_path: string;
  line_number: number;
  complexity: number;
  is_extern: boolean;
  is_entry_point: boolean;
  call_count: number;
  educational_description?: string;
  performance_impact: string;
}

export interface CallGraphEdge {
  from: string;
  to: string;
  call_count: number;
  is_recursive: boolean;
  is_cross_file: boolean;
  is_system_call: boolean;
}

export interface PerformanceHotspot {
  location: CodeLocation;
  hotspot_type: string;
  severity: string;
  estimated_impact: string;
  description: string;
  educational_context: string;
  optimization_potential: string;
}

export interface CodeLocation {
  file_path: string;
  line_number: number;
  column: number;
  function_name?: string;
}

function App() {
  const [currentView, setCurrentView] = React.useState<'browse' | 'modules' | 'performance' | 'debug'>('browse');
  const [isLoading, setIsLoading] = React.useState(false);

  const handleViewChange = React.useCallback((view: 'browse' | 'modules' | 'performance' | 'debug') => {
    setCurrentView(view);
  }, []);

  return (
    <div className="min-h-screen bg-gray-50">
      <Navigation currentView={currentView} onViewChange={handleViewChange} />
      
      {isLoading && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white p-6 rounded-lg shadow-lg">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mb-4"></div>
            <p className="text-gray-700">Analyzing code...</p>
          </div>
        </div>
      )}

      <main className="container mx-auto px-4 py-6">
        {currentView === 'browse' && <CodeBrowser isLoading={isLoading} setIsLoading={setIsLoading} />}
        {currentView === 'modules' && <EducationalModules />}
        {currentView === 'performance' && <PerformanceDashboard />}
        {currentView === 'debug' && <DebugInterface />}
      </main>

      {/* Global styles */}
      <style>{`
        .code-viewer {
          font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
        }
        
        .syntax-keyword {
          color: #0000ff;
          font-weight: bold;
        }
        
        .syntax-string {
          color: #008000;
        }
        
        .syntax-comment {
          color: #808080;
          font-style: italic;
        }
        
        .syntax-function {
          color: #795e26;
        }
        
        .syntax-number {
          color: #09885a;
        }
        
        .inline-explanation {
          position: relative;
          border-bottom: 1px dotted #007acc;
          cursor: help;
        }
        
        .explanation-tooltip {
          position: absolute;
          background: #333;
          color: white;
          padding: 8px 12px;
          border-radius: 4px;
          font-size: 14px;
          max-width: 300px;
          z-index: 1000;
          box-shadow: 0 2px 10px rgba(0,0,0,0.2);
        }
        
        .hotspot-critical {
          background-color: #ffebee;
          border-left: 4px solid #f44336;
        }
        
        .hotspot-high {
          background-color: #fff3e0;
          border-left: 4px solid #ff9800;
        }
        
        .hotspot-medium {
          background-color: #e8f5e8;
          border-left: 4px solid #4caf50;
        }
        
        .call-graph-container {
          background: linear-gradient(45deg, #f0f0f0 25%, transparent 25%), 
                      linear-gradient(-45deg, #f0f0f0 25%, transparent 25%),
                      linear-gradient(45deg, transparent 75%, #f0f0f0 75%), 
                      linear-gradient(-45deg, transparent 75%, #f0f0f0 75%);
          background-size: 20px 20px;
          background-position: 0 0, 0 10px, 10px -10px, -10px 0px;
        }
      `}</style>
    </div>
  );
}

export default App;