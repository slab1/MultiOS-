import React, { useState, useMemo } from 'react';
import { Info, Lightbulb, AlertTriangle, BookOpen } from 'lucide-react';
import type { CodeAnalysis, FunctionInfo, InlineExplanation, EducationalComment } from '../App';

interface CodeViewerProps {
  code: string;
  analysis: CodeAnalysis | null;
  selectedFunction: string | null;
  onFunctionSelect: (functionName: string | null) => void;
  isLoading: boolean;
}

export const CodeViewer: React.FC<CodeViewerProps> = ({
  code,
  analysis,
  selectedFunction,
  onFunctionSelect,
  isLoading
}) => {
  const [hoveredLine, setHoveredLine] = useState<number | null>(null);
  const [showEducationalComments, setShowEducationalComments] = useState(true);
  const [contextLines, setContextLines] = useState(3);

  const codeLines = useMemo(() => code.split('\n'), [code]);

  const getSyntaxHighlight = (lineNumber: number, line: string) => {
    if (!analysis) return [{ text: line, className: '' }];

    const highlights = analysis.syntax_highlighting
      .filter(h => h.line === lineNumber)
      .sort((a, b) => a.start_col - b.start_col);

    if (highlights.length === 0) {
      return [{ text: line, className: '' }];
    }

    const parts = [];
    let lastIndex = 0;

    highlights.forEach((highlight, index) => {
      // Add text before highlight
      if (highlight.start_col > lastIndex) {
        parts.push({
          text: line.slice(lastIndex, highlight.start_col),
          className: ''
        });
      }

      // Add highlighted text
      const className = getSyntaxClass(highlight.token_type);
      parts.push({
        text: line.slice(highlight.start_col, highlight.end_col),
        className
      });

      lastIndex = highlight.end_col;

      // Add remaining text after last highlight
      if (index === highlights.length - 1 && lastIndex < line.length) {
        parts.push({
          text: line.slice(lastIndex),
          className: ''
        });
      }
    });

    return parts;
  };

  const getSyntaxClass = (tokenType: string): string => {
    switch (tokenType) {
      case 'keyword': return 'syntax-keyword';
      case 'string': return 'syntax-string';
      case 'comment': return 'syntax-comment';
      case 'function': return 'syntax-function';
      case 'number': return 'syntax-number';
      default: return '';
    }
  };

  const getLineExplanations = (lineNumber: number): InlineExplanation[] => {
    if (!analysis) return [];
    return analysis.inline_explanations.filter(ex => ex.line === lineNumber);
  };

  const getLineEducationalComments = (lineNumber: number): EducationalComment[] => {
    if (!analysis) return [];
    return analysis.educational_comments.filter(ex => ex.line === lineNumber);
  };

  const getFunctionForLine = (lineNumber: number): FunctionInfo | null => {
    if (!analysis) return null;
    return analysis.functions.find(f => 
      lineNumber >= f.start_line && lineNumber <= f.end_line
    ) || null;
  };

  const getComplexityColor = (complexity: number): string => {
    if (complexity < 5) return 'text-green-600';
    if (complexity < 10) return 'text-yellow-600';
    return 'text-red-600';
  };

  const getSeverityIcon = (severity: string) => {
    switch (severity.toLowerCase()) {
      case 'critical': return <AlertTriangle className="w-4 h-4 text-red-500" />;
      case 'high': return <AlertTriangle className="w-4 h-4 text-orange-500" />;
      case 'medium': return <Info className="w-4 h-4 text-yellow-500" />;
      default: return <Info className="w-4 h-4 text-blue-500" />;
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-4"></div>
          <p className="text-gray-600">Analyzing code...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-full">
      {/* Code Panel */}
      <div className="flex-1 bg-white border-r border-gray-200">
        <div className="flex items-center justify-between px-4 py-3 bg-gray-50 border-b border-gray-200">
          <div className="flex items-center space-x-4">
            <h3 className="text-sm font-medium text-gray-900">Code Viewer</h3>
            {analysis && (
              <div className="flex items-center space-x-2 text-xs text-gray-500">
                <span>Complexity: {analysis.complexity_score}</span>
                <span>•</span>
                <span>{analysis.functions.length} functions</span>
                <span>•</span>
                <span>{analysis.variables.length} variables</span>
              </div>
            )}
          </div>
          
          <div className="flex items-center space-x-2">
            <label className="flex items-center text-sm text-gray-600">
              <input
                type="checkbox"
                checked={showEducationalComments}
                onChange={(e) => setShowEducationalComments(e.target.checked)}
                className="mr-2"
              />
              Show educational content
            </label>
            <select
              value={contextLines}
              onChange={(e) => setContextLines(Number(e.target.value))}
              className="text-sm border border-gray-300 rounded px-2 py-1"
            >
              <option value={1}>1 line context</option>
              <option value={3}>3 lines context</option>
              <option value={5}>5 lines context</option>
            </select>
          </div>
        </div>

        <div className="code-viewer overflow-auto h-full p-4">
          {codeLines.map((line, index) => {
            const lineNumber = index + 1;
            const parts = getSyntaxHighlight(lineNumber, line);
            const explanations = getLineExplanations(lineNumber);
            const comments = getLineEducationalComments(lineNumber);
            const functionInfo = getFunctionForLine(lineNumber);
            const isSelected = selectedFunction === functionInfo?.name;

            return (
              <div
                key={lineNumber}
                className={`group flex hover:bg-gray-50 transition-colors duration-200 ${
                  functionInfo ? 'cursor-pointer' : ''
                } ${isSelected ? 'bg-blue-50 border-l-4 border-blue-500' : ''}`}
                onClick={() => functionInfo && onFunctionSelect(functionInfo.name)}
                onMouseEnter={() => setHoveredLine(lineNumber)}
                onMouseLeave={() => setHoveredLine(null)}
              >
                {/* Line Number */}
                <div className="flex-shrink-0 w-12 text-right pr-4 text-gray-400 text-sm font-mono select-none">
                  {lineNumber}
                </div>

                {/* Line Content */}
                <div className="flex-1 font-mono text-sm leading-relaxed">
                  {parts.map((part, partIndex) => (
                    <span key={partIndex} className={part.className}>
                      {part.text}
                    </span>
                  ))}
                </div>

                {/* Inline Indicators */}
                <div className="flex-shrink-0 flex items-center space-x-1 ml-4 opacity-0 group-hover:opacity-100 transition-opacity duration-200">
                  {explanations.map((explanation, expIndex) => (
                    <div
                      key={expIndex}
                      className="relative"
                      title={explanation.explanation}
                    >
                      <Lightbulb className="w-4 h-4 text-yellow-500" />
                    </div>
                  ))}
                  
                  {functionInfo && (
                    <div className="relative" title={functionInfo.educational_description || functionInfo.signature}>
                      <BookOpen className="w-4 h-4 text-blue-500" />
                    </div>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Side Panel */}
      <div className="w-80 bg-gray-50 border-l border-gray-200 overflow-auto">
        {/* Analysis Summary */}
        {analysis && (
          <div className="p-4 border-b border-gray-200">
            <h3 className="text-sm font-medium text-gray-900 mb-3">Analysis Summary</h3>
            
            <div className="space-y-3">
              <div>
                <div className="flex items-center justify-between text-sm">
                  <span className="text-gray-600">Overall Complexity</span>
                  <span className={`font-medium ${getComplexityColor(analysis.complexity_score)}`}>
                    {analysis.complexity_score}
                  </span>
                </div>
                <div className="w-full bg-gray-200 rounded-full h-2 mt-1">
                  <div 
                    className={`h-2 rounded-full transition-all duration-300 ${
                      analysis.complexity_score < 20 ? 'bg-green-500' :
                      analysis.complexity_score < 40 ? 'bg-yellow-500' : 'bg-red-500'
                    }`}
                    style={{ width: `${Math.min((analysis.complexity_score / 100) * 100, 100)}%` }}
                  ></div>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4 text-sm">
                <div className="text-center">
                  <div className="text-lg font-semibold text-gray-900">{analysis.functions.length}</div>
                  <div className="text-gray-600">Functions</div>
                </div>
                <div className="text-center">
                  <div className="text-lg font-semibold text-gray-900">{analysis.variables.length}</div>
                  <div className="text-gray-600">Variables</div>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Inline Explanations */}
        {hoveredLine && (
          <div className="p-4 border-b border-gray-200">
            <h3 className="text-sm font-medium text-gray-900 mb-3">Line {hoveredLine} Details</h3>
            
            {getLineExplanations(hoveredLine).map((explanation, index) => (
              <div key={index} className="mb-3 p-3 bg-white rounded-lg border">
                <div className="flex items-start space-x-2">
                  <Lightbulb className="w-4 h-4 text-yellow-500 mt-0.5 flex-shrink-0" />
                  <div>
                    <div className="text-sm text-gray-900 mb-1">{explanation.explanation}</div>
                    <div className="text-xs text-gray-500">
                      Complexity: {explanation.complexity_level}
                    </div>
                    {explanation.related_concepts.length > 0 && (
                      <div className="mt-2">
                        <div className="text-xs text-gray-500 mb-1">Related concepts:</div>
                        <div className="flex flex-wrap gap-1">
                          {explanation.related_concepts.map((concept, idx) => (
                            <span
                              key={idx}
                              className="px-2 py-1 bg-blue-100 text-blue-700 text-xs rounded"
                            >
                              {concept}
                            </span>
                          ))}
                        </div>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            ))}

            {getLineEducationalComments(hoveredLine).map((comment, index) => (
              <div key={index} className="mb-3 p-3 bg-white rounded-lg border">
                <div className="flex items-start space-x-2">
                  {getSeverityIcon(comment.difficulty_level)}
                  <div>
                    <div className="text-sm text-gray-900 mb-1">{comment.comment}</div>
                    <div className="text-xs text-gray-500">
                      Category: {comment.category} • Level: {comment.difficulty_level}
                    </div>
                    {comment.learning_objectives.length > 0 && (
                      <div className="mt-2">
                        <div className="text-xs text-gray-500 mb-1">Learning objectives:</div>
                        <ul className="text-xs text-gray-600 list-disc list-inside">
                          {comment.learning_objectives.map((objective, idx) => (
                            <li key={idx}>{objective}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}

        {/* Functions List */}
        {analysis && analysis.functions.length > 0 && (
          <div className="p-4 border-b border-gray-200">
            <h3 className="text-sm font-medium text-gray-900 mb-3">Functions</h3>
            
            <div className="space-y-2">
              {analysis.functions.map((func, index) => (
                <div
                  key={index}
                  className={`p-3 rounded-lg cursor-pointer transition-colors duration-200 ${
                    selectedFunction === func.name 
                      ? 'bg-blue-100 border border-blue-200' 
                      : 'bg-white hover:bg-gray-50 border border-gray-200'
                  }`}
                  onClick={() => onFunctionSelect(func.name)}
                >
                  <div className="flex items-center justify-between">
                    <div className="font-medium text-sm text-gray-900">{func.name}</div>
                    <div className={`text-xs px-2 py-1 rounded ${getComplexityColor(func.complexity)} bg-gray-100`}>
                      {func.complexity}
                    </div>
                  </div>
                  <div className="text-xs text-gray-600 mt-1">
                    Lines {func.start_line}-{func.end_line}
                  </div>
                  {func.educational_description && (
                    <div className="text-xs text-gray-500 mt-1">
                      {func.educational_description}
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Variables List */}
        {analysis && analysis.variables.length > 0 && (
          <div className="p-4">
            <h3 className="text-sm font-medium text-gray-900 mb-3">Variables</h3>
            
            <div className="space-y-2">
              {analysis.variables.slice(0, 10).map((variable, index) => (
                <div key={index} className="p-2 bg-white rounded border">
                  <div className="flex items-center justify-between">
                    <span className="text-sm font-medium text-gray-900">{variable.name}</span>
                    <span className="text-xs text-gray-500">{variable.var_type}</span>
                  </div>
                  <div className="text-xs text-gray-500">
                    Line {variable.line} • {variable.scope}
                    {variable.is_mutable && ' • mutable'}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
