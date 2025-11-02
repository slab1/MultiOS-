import React, { useState, useEffect } from 'react';
import { 
  ChartBarIcon,
  ExclamationTriangleIcon,
  CheckCircleIcon,
  InformationCircleIcon,
  LightBulbIcon,
  CodeBracketIcon,
  DocumentTextIcon
} from '@heroicons/react/24/outline';
import { useVCS } from '../context/VCSContext';
import { toast } from 'react-hot-toast';

const QualityAnalysis = () => {
  const { API_URL, currentRepo } = useVCS();
  const [selectedFile, setSelectedFile] = useState('');
  const [codeContent, setCodeContent] = useState('');
  const [analysis, setAnalysis] = useState(null);
  const [loading, setLoading] = useState(false);
  const [files, setFiles] = useState([]);

  useEffect(() => {
    if (currentRepo) {
      loadRepositoryFiles();
    }
  }, [currentRepo]);

  const loadRepositoryFiles = () => {
    // Mock files for demo
    const mockFiles = [
      { name: 'main.py', path: 'main.py', type: 'python', size: '2.4 KB' },
      { name: 'utils.py', path: 'utils.py', type: 'python', size: '1.8 KB' },
      { name: 'models.py', path: 'models.py', type: 'python', size: '3.2 KB' },
      { name: 'app.js', path: 'app.js', type: 'javascript', size: '4.1 KB' },
      { name: 'README.md', path: 'README.md', type: 'markdown', size: '1.2 KB' }
    ];
    setFiles(mockFiles);
  };

  const analyzeCode = async () => {
    if (!codeContent.trim()) {
      toast.error('Please enter some code to analyze');
      return;
    }

    setLoading(true);
    try {
      const response = await fetch(`${API_URL}/quality/analyze`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          code: codeContent,
          file_path: selectedFile || 'analyzed_code.py',
          language: 'python'
        })
      });

      const result = await response.json();
      
      if (response.ok) {
        setAnalysis(result);
        toast.success('Code analysis completed!');
      } else {
        throw new Error(result.error || 'Analysis failed');
      }
    } catch (error) {
      console.error('Analysis error:', error);
      toast.error('Failed to analyze code: ' + error.message);
      setAnalysis(null);
    } finally {
      setLoading(false);
    }
  };

  const getSeverityColor = (severity) => {
    switch (severity) {
      case 'poor': return 'text-red-600 bg-red-50 border-red-200';
      case 'needs_improvement': return 'text-yellow-600 bg-yellow-50 border-yellow-200';
      case 'good': return 'text-blue-600 bg-blue-50 border-blue-200';
      case 'excellent': return 'text-green-600 bg-green-50 border-green-200';
      default: return 'text-gray-600 bg-gray-50 border-gray-200';
    }
  };

  const getSeverityIcon = (severity) => {
    switch (severity) {
      case 'poor': return <ExclamationTriangleIcon className="w-5 h-5" />;
      case 'needs_improvement': return <ExclamationTriangleIcon className="w-5 h-5" />;
      case 'good': return <InformationCircleIcon className="w-5 h-5" />;
      case 'excellent': return <CheckCircleIcon className="w-5 h-5" />;
      default: return <InformationCircleIcon className="w-5 h-5" />;
    }
  };

  const getScoreColor = (score) => {
    if (score >= 90) return 'text-green-600';
    if (score >= 75) return 'text-blue-600';
    if (score >= 60) return 'text-yellow-600';
    return 'text-red-600';
  };

  const getScoreGrade = (score) => {
    if (score >= 90) return 'A';
    if (score >= 80) return 'B';
    if (score >= 70) return 'C';
    if (score >= 60) return 'D';
    return 'F';
  };

  const sampleCodeSnippets = {
    'good': `# Good code example
def calculate_average(numbers):
    """
    Calculate the average of a list of numbers.
    
    Args:
        numbers (list): List of numerical values
        
    Returns:
        float: The average value
    """
    if not numbers:
        return 0.0
    
    return sum(numbers) / len(numbers)


def validate_input(user_input):
    """Validate user input is a positive number."""
    try:
        value = float(user_input)
        return value > 0
    except (ValueError, TypeError):
        return False


# Main function
if __name__ == "__main__":
    numbers = [1, 2, 3, 4, 5]
    average = calculate_average(numbers)
    print(f"Average: {average}")`,

    'poor': `# Poor code example - has many issues
def calc(n):
    x=sum(n)/len(n)
    return x

# No docstring, poor naming, no input validation
def f(i):
    if i>0:
        return True
    else:
        return False

# Very long function with deep nesting
def process_data(data, settings, options, config, parameters, flags, debug, verbose):
    result = []
    for item in data:
        if item:
            if settings.get('enabled'):
                if options.get('option1'):
                    if config.get('setting1'):
                        if parameters.get('param1'):
                            if flags.get('flag1'):
                                if debug:
                                    if verbose:
                                        # Deep nesting continues...
                                        value = item['value'] * 2
                                        if value > 100:
                                            if value < 200:
                                                # More nesting...
                                                processed = value + 10
                                                if processed < 150:
                                                    result.append(processed)
    return result`
  };

  return (
    <div className="h-full flex">
      {/* Sidebar - File List */}
      <div className="w-80 bg-white border-r border-gray-200 overflow-y-auto">
        <div className="p-6 border-b border-gray-200">
          <h2 className="text-xl font-semibold text-gray-900 flex items-center">
            <ChartBarIcon className="w-6 h-6 mr-2" />
            Code Analysis
          </h2>
          <p className="text-gray-600 text-sm mt-1">
            Analyze your code quality and get educational feedback
          </p>
        </div>

        <div className="p-4">
          {/* Quick Actions */}
          <div className="mb-6">
            <h3 className="text-sm font-semibold text-gray-700 mb-3">Quick Examples</h3>
            <div className="space-y-2">
              <button
                onClick={() => setCodeContent(sampleCodeSnippets.good)}
                className="w-full text-left p-3 bg-green-50 border border-green-200 rounded-lg hover:bg-green-100 transition-colors"
              >
                <div className="flex items-center space-x-2">
                  <CheckCircleIcon className="w-4 h-4 text-green-600" />
                  <span className="text-sm text-green-800">Good Code Example</span>
                </div>
              </button>
              
              <button
                onClick={() => setCodeContent(sampleCodeSnippets.poor)}
                className="w-full text-left p-3 bg-red-50 border border-red-200 rounded-lg hover:bg-red-100 transition-colors"
              >
                <div className="flex items-center space-x-2">
                  <ExclamationTriangleIcon className="w-4 h-4 text-red-600" />
                  <span className="text-sm text-red-800">Problematic Code</span>
                </div>
              </button>
            </div>
          </div>

          {/* Repository Files */}
          {currentRepo && (
            <div>
              <h3 className="text-sm font-semibold text-gray-700 mb-3">Repository Files</h3>
              <div className="space-y-1">
                {files.map((file) => (
                  <button
                    key={file.path}
                    onClick={() => {
                      setSelectedFile(file.path);
                      setCodeContent(`// ${file.name}\n// This is a placeholder for ${file.name}\n// Select the file to load its actual content`);
                    }}
                    className={`
                      w-full text-left p-3 rounded-lg border transition-colors
                      ${selectedFile === file.path
                        ? 'bg-blue-50 border-blue-200 text-blue-900'
                        : 'bg-white border-gray-200 text-gray-700 hover:bg-gray-50'
                      }
                    `}
                  >
                    <div className="flex items-center space-x-2">
                      <CodeBracketIcon className="w-4 h-4" />
                      <div className="flex-1 min-w-0">
                        <p className="text-sm font-medium truncate">{file.name}</p>
                        <p className="text-xs text-gray-500">{file.size}</p>
                      </div>
                    </div>
                  </button>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 flex flex-col">
        {/* Editor */}
        <div className="flex-1 p-6 bg-gray-50">
          <div className="bg-white rounded-lg shadow-sm border border-gray-200 h-full flex flex-col">
            <div className="p-4 border-b border-gray-200">
              <h3 className="text-lg font-semibold text-gray-900">Code Editor</h3>
              <p className="text-sm text-gray-600 mt-1">
                Enter your code below for quality analysis
              </p>
            </div>
            
            <div className="flex-1 p-4">
              <textarea
                value={codeContent}
                onChange={(e) => setCodeContent(e.target.value)}
                placeholder={`# Enter your ${selectedFile || 'Python'} code here...

# Example:
def hello_world():
    print("Hello, World!")

if __name__ == "__main__":
    hello_world()`}
                className="w-full h-full p-4 font-mono text-sm border border-gray-300 rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-blue-500"
                style={{ fontFamily: 'Monaco, Menlo, "Ubuntu Mono", monospace' }}
              />
            </div>
            
            <div className="p-4 border-t border-gray-200">
              <div className="flex justify-between items-center">
                <div className="text-sm text-gray-600">
                  {codeContent.split('\n').length} lines, {codeContent.length} characters
                </div>
                <button
                  onClick={analyzeCode}
                  disabled={loading || !codeContent.trim()}
                  className={`
                    px-6 py-2 rounded-lg font-medium transition-colors
                    ${!loading && codeContent.trim()
                      ? 'bg-blue-600 text-white hover:bg-blue-700'
                      : 'bg-gray-200 text-gray-500 cursor-not-allowed'
                    }
                  `}
                >
                  {loading ? 'Analyzing...' : 'Analyze Code'}
                </button>
              </div>
            </div>
          </div>
        </div>

        {/* Analysis Results */}
        {analysis && (
          <div className="border-t border-gray-200 bg-white">
            <div className="p-6">
              <h3 className="text-xl font-semibold text-gray-900 mb-6">
                Quality Analysis Results
              </h3>

              {/* Overall Score */}
              <div className="mb-8 p-6 bg-gray-50 rounded-lg">
                <div className="flex items-center justify-between">
                  <div>
                    <h4 className="text-lg font-semibold text-gray-900">Overall Quality Score</h4>
                    <p className="text-gray-600 mt-1">
                      This score reflects code readability, complexity, and best practices
                    </p>
                  </div>
                  <div className="text-center">
                    <div className={`text-4xl font-bold ${getScoreColor(analysis.overall_score)}`}>
                      {analysis.overall_score.toFixed(1)}
                    </div>
                    <div className="text-sm text-gray-600">Grade: {getScoreGrade(analysis.overall_score)}</div>
                  </div>
                </div>
                
                <div className="mt-4">
                  <div className="w-full bg-gray-200 rounded-full h-3">
                    <div 
                      className={`h-3 rounded-full transition-all duration-500 ${
                        analysis.overall_score >= 90 ? 'bg-green-500' :
                        analysis.overall_score >= 75 ? 'bg-blue-500' :
                        analysis.overall_score >= 60 ? 'bg-yellow-500' : 'bg-red-500'
                      }`}
                      style={{ width: `${analysis.overall_score}%` }}
                    ></div>
                  </div>
                </div>
              </div>

              <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
                {/* Issues */}
                <div>
                  <h4 className="text-lg font-semibold text-gray-900 mb-4 flex items-center">
                    <ExclamationTriangleIcon className="w-5 h-5 mr-2 text-red-500" />
                    Issues Found ({analysis.issues.length})
                  </h4>
                  
                  {analysis.issues.length === 0 ? (
                    <div className="p-4 bg-green-50 border border-green-200 rounded-lg">
                      <div className="flex items-center space-x-2">
                        <CheckCircleIcon className="w-5 h-5 text-green-600" />
                        <p className="text-green-800 font-medium">No issues found!</p>
                      </div>
                      <p className="text-green-700 text-sm mt-1">
                        Your code follows good practices. Great job! 🎉
                      </p>
                    </div>
                  ) : (
                    <div className="space-y-3">
                      {analysis.issues.map((issue, index) => (
                        <div 
                          key={index}
                          className={`p-4 border rounded-lg ${getSeverityColor(issue.severity)}`}
                        >
                          <div className="flex items-start space-x-3">
                            {getSeverityIcon(issue.severity)}
                            <div className="flex-1">
                              <div className="flex items-center justify-between mb-1">
                                <h5 className="font-medium">{issue.type.replace('_', ' ').toUpperCase()}</h5>
                                <span className="text-xs font-medium px-2 py-1 rounded-full bg-white/50">
                                  {issue.severity.replace('_', ' ')}
                                </span>
                              </div>
                              <p className="text-sm mb-2">{issue.message}</p>
                              <p className="text-xs opacity-90 mb-2">{issue.suggestion}</p>
                              {issue.educational_note && (
                                <div className="text-xs opacity-80 bg-white/30 p-2 rounded">
                                  <strong>💡 Learning Note:</strong> {issue.educational_note}
                                </div>
                              )}
                              {issue.line_number && (
                                <div className="text-xs mt-1 opacity-70">
                                  Line {issue.line_number}
                                </div>
                              )}
                            </div>
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>

                {/* Strengths and Suggestions */}
                <div>
                  <h4 className="text-lg font-semibold text-gray-900 mb-4 flex items-center">
                    <CheckCircleIcon className="w-5 h-5 mr-2 text-green-500" />
                    Strengths
                  </h4>
                  
                  {analysis.strengths.length === 0 ? (
                    <p className="text-gray-500 italic">No specific strengths identified. Keep learning!</p>
                  ) : (
                    <div className="space-y-2 mb-6">
                      {analysis.strengths.map((strength, index) => (
                        <div key={index} className="flex items-center space-x-2 text-green-700">
                          <CheckCircleIcon className="w-4 h-4" />
                          <span className="text-sm">{strength}</span>
                        </div>
                      ))}
                    </div>
                  )}

                  <h4 className="text-lg font-semibold text-gray-900 mb-4 flex items-center">
                    <LightBulbIcon className="w-5 h-5 mr-2 text-yellow-500" />
                    Suggestions
                  </h4>
                  
                  {analysis.suggestions.length === 0 ? (
                    <p className="text-gray-500 italic">No additional suggestions. Excellent work!</p>
                  ) : (
                    <div className="space-y-2 mb-6">
                      {analysis.suggestions.map((suggestion, index) => (
                        <div key={index} className="flex items-start space-x-2 text-yellow-700">
                          <LightBulbIcon className="w-4 h-4 mt-0.5 flex-shrink-0" />
                          <span className="text-sm">{suggestion}</span>
                        </div>
                      ))}
                    </div>
                  )}

                  {/* Educational Feedback */}
                  {analysis.educational_feedback && (
                    <div className="mt-6 p-4 bg-blue-50 border border-blue-200 rounded-lg">
                      <h5 className="font-semibold text-blue-900 mb-3 flex items-center">
                        <DocumentTextIcon className="w-4 h-4 mr-2" />
                        Learning Feedback
                      </h5>
                      
                      {analysis.educational_feedback.learning_objectives && (
                        <div className="mb-3">
                          <h6 className="font-medium text-blue-800 text-sm mb-1">Learning Objectives</h6>
                          <ul className="text-sm text-blue-700 space-y-1">
                            {analysis.educational_feedback.learning_objectives.map((obj, index) => (
                              <li key={index}>• {obj}</li>
                            ))}
                          </ul>
                        </div>
                      )}
                      
                      {analysis.educational_feedback.concepts_to_review && (
                        <div>
                          <h6 className="font-medium text-blue-800 text-sm mb-1">Concepts to Review</h6>
                          <ul className="text-sm text-blue-700 space-y-1">
                            {analysis.educational_feedback.concepts_to_review.map((concept, index) => (
                              <li key={index}>• {concept}</li>
                            ))}
                          </ul>
                        </div>
                      )}
                    </div>
                  )}
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default QualityAnalysis;
