import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Textarea } from '@/components/ui/textarea';
import { Play, CheckCircle, XCircle, Lightbulb, Clock, Star } from 'lucide-react';
import { GameChallenge, TestCase } from '../../types';

interface CodeChallengeGameProps {
  challenge: GameChallenge;
  onComplete: (score: number, passedTests: number, totalTests: number) => void;
}

interface ExecutionResult {
  passed: number;
  total: number;
  testResults: Array<{
    testCase: TestCase;
    passed: boolean;
    output: string;
    error?: string;
  }>;
  executionTime: number;
  memoryUsage: number;
}

export const CodeChallengeGame: React.FC<CodeChallengeGameProps> = ({ 
  challenge, 
  onComplete 
}) => {
  const [code, setCode] = useState(challenge.codeTemplate || '');
  const [isRunning, setIsRunning] = useState(false);
  const [result, setResult] = useState<ExecutionResult | null>(null);
  const [hintsUsed, setHintsUsed] = useState(0);
  const [startTime] = useState(Date.now());
  const [currentHint, setCurrentHint] = useState('');
  const [showHints, setShowHints] = useState(false);

  // Simulated code execution environment
  const executeCode = async (userCode: string, testCases: TestCase[]): Promise<ExecutionResult> => {
    setIsRunning(true);
    
    // Simulate execution delay
    await new Promise(resolve => setTimeout(resolve, 1000 + Math.random() * 2000));
    
    const testResults = testCases.map(testCase => {
      try {
        // In a real implementation, this would execute the user's code
        // For demo purposes, we'll simulate random pass/fail based on code complexity
        const hasComplexity = userCode.includes('if') && userCode.includes('for');
        const hasReturn = userCode.includes('return');
        const hasComments = userCode.includes('//') || userCode.includes('/*');
        
        const basePassRate = 0.3;
        const complexityBonus = hasComplexity ? 0.3 : 0;
        const returnBonus = hasReturn ? 0.2 : 0;
        const commentBonus = hasComments ? 0.1 : 0;
        
        const passProbability = Math.min(0.95, basePassRate + complexityBonus + returnBonus + commentBonus);
        const passed = Math.random() < passProbability;
        
        let output = '';
        if (passed) {
          output = testCase.expectedOutput;
        } else {
          // Generate plausible wrong outputs
          const wrongOutputs = ['0', 'null', 'undefined', 'NaN', 'false', testCase.expectedOutput + '1'];
          output = wrongOutputs[Math.floor(Math.random() * wrongOutputs.length)];
        }
        
        return {
          testCase,
          passed,
          output,
        };
      } catch (error) {
        return {
          testCase,
          passed: false,
          output: '',
          error: error instanceof Error ? error.message : 'Execution error',
        };
      }
    });
    
    const passed = testResults.filter(r => r.passed).length;
    const executionTime = Math.floor(Math.random() * 100) + 50; // 50-150ms
    const memoryUsage = Math.floor(Math.random() * 1000) + 500; // 500-1500KB
    
    const finalResult = {
      passed,
      total: testCases.length,
      testResults,
      executionTime,
      memoryUsage,
    };
    
    setResult(finalResult);
    setIsRunning(false);
    
    return finalResult;
  };

  const runCode = async () => {
    if (!code.trim()) return;
    
    const executionResult = await executeCode(code, challenge.testCases);
    
    // Calculate score
    const timeElapsed = (Date.now() - startTime) / 1000 / 60; // minutes
    const timeBonus = Math.max(0, 10 - timeElapsed); // Bonus for speed
    const hintPenalty = hintsUsed * 5; // Penalty for using hints
    const codeQuality = calculateCodeQuality(code);
    
    const score = Math.min(100, 
      Math.max(0, 
        (executionResult.passed / executionResult.total) * 100 + 
        timeBonus + 
        codeQuality - 
        hintPenalty
      )
    );
    
    onComplete(score, executionResult.passed, executionResult.total);
  };

  const calculateCodeQuality = (userCode: string): number => {
    let quality = 0;
    
    // Code structure analysis
    if (userCode.includes('if') && userCode.includes('else')) quality += 10;
    if (userCode.includes('for') || userCode.includes('while')) quality += 10;
    if (userCode.includes('function') || userCode.includes('def')) quality += 10;
    
    // Comments and documentation
    if (userCode.includes('//') || userCode.includes('/*')) quality += 15;
    if (userCode.includes('"""') || userCode.includes("'''")) quality += 15;
    
    // Error handling
    if (userCode.includes('try') && userCode.includes('catch')) quality += 20;
    if (userCode.includes('throw')) quality += 10;
    
    // Code formatting
    const lines = userCode.split('\n');
    const emptyLines = lines.filter(line => line.trim() === '').length;
    const properIndentation = lines.filter(line => line.includes('  ') || line.includes('\t')).length;
    
    if (emptyLines > 2) quality += 5; // Good spacing
    if (properIndentation > lines.length * 0.5) quality += 10; // Good indentation
    
    return quality;
  };

  const useHint = () => {
    if (hintsUsed < challenge.hints.length) {
      setCurrentHint(challenge.hints[hintsUsed]);
      setHintsUsed(prev => prev + 1);
      setShowHints(true);
    }
  };

  const getScoreColor = (score: number): string => {
    if (score >= 90) return 'text-green-400';
    if (score >= 70) return 'text-yellow-400';
    if (score >= 50) return 'text-orange-400';
    return 'text-red-400';
  };

  const getScoreBadge = (score: number): { text: string; variant: any } => {
    if (score >= 90) return { text: 'Excellent', variant: 'default' };
    if (score >= 70) return { text: 'Good', variant: 'secondary' };
    if (score >= 50) return { text: 'Fair', variant: 'outline' };
    return { text: 'Needs Work', variant: 'destructive' };
  };

  return (
    <div className="space-y-6">
      {/* Challenge Header */}
      <Card className="bg-slate-800 border-slate-700">
        <CardHeader>
          <div className="flex justify-between items-start">
            <div className="flex-1">
              <div className="flex items-center gap-3 mb-2">
                <CardTitle className="text-xl text-white">{challenge.title}</CardTitle>
                <Badge variant="outline" className="capitalize">
                  {challenge.difficulty}
                </Badge>
                <Badge variant="secondary">
                  {challenge.category}
                </Badge>
              </div>
              <p className="text-gray-300 mb-2">{challenge.description}</p>
              {challenge.storyContext && (
                <div className="bg-slate-700 p-3 rounded text-sm text-gray-300 italic">
                  {challenge.storyContext}
                </div>
              )}
            </div>
            <div className="text-right">
              <div className="text-sm text-gray-400">XP Reward</div>
              <div className="text-lg font-bold text-yellow-400">{challenge.xpReward}</div>
              <div className="text-sm text-gray-400">~{challenge.estimatedTime} min</div>
            </div>
          </div>
        </CardHeader>
      </Card>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Code Editor */}
        <Card className="bg-slate-800 border-slate-700">
          <CardHeader>
            <CardTitle className="text-white flex items-center gap-2">
              <Star className="w-5 h-5" />
              Code Editor
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <Textarea
              value={code}
              onChange={(e) => setCode(e.target.value)}
              className="min-h-96 font-mono bg-slate-900 text-green-400 border-slate-600 focus:border-blue-500"
              placeholder="Write your solution here..."
              spellCheck={false}
            />
            
            <div className="flex gap-2">
              <Button 
                onClick={runCode} 
                disabled={isRunning || !code.trim()}
                className="flex-1"
              >
                {isRunning ? (
                  <>
                    <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2" />
                    Running...
                  </>
                ) : (
                  <>
                    <Play className="w-4 h-4 mr-2" />
                    Run Code
                  </>
                )}
              </Button>
              
              <Button 
                onClick={useHint}
                disabled={hintsUsed >= challenge.hints.length}
                variant="outline"
              >
                <Lightbulb className="w-4 h-4 mr-2" />
                Hint ({hintsUsed}/{challenge.hints.length})
              </Button>
            </div>

            {showHints && currentHint && (
              <div className="bg-yellow-900 border border-yellow-600 p-3 rounded">
                <div className="text-yellow-200 text-sm">
                  <strong>Hint:</strong> {currentHint}
                </div>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Test Results & Hints */}
        <Card className="bg-slate-800 border-slate-700">
          <CardHeader>
            <CardTitle className="text-white">Test Results</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {result ? (
              <>
                {/* Summary */}
                <div className="bg-slate-700 p-4 rounded">
                  <div className="grid grid-cols-2 gap-4 mb-4">
                    <div>
                      <div className="text-2xl font-bold text-center">
                        <span className={getScoreColor((result.passed / result.total) * 100)}>
                          {result.passed}/{result.total}
                        </span>
                      </div>
                      <div className="text-sm text-gray-400 text-center">Tests Passed</div>
                    </div>
                    <div>
                      <div className="text-2xl font-bold text-center">
                        <span className={getScoreColor((result.passed / result.total) * 100)}>
                          {Math.round((result.passed / result.total) * 100)}%
                        </span>
                      </div>
                      <div className="text-sm text-gray-400 text-center">Success Rate</div>
                    </div>
                  </div>
                  
                  <div className="grid grid-cols-3 gap-2 text-sm">
                    <div className="text-center">
                      <div className="text-white font-medium">{result.executionTime}ms</div>
                      <div className="text-gray-400">Exec Time</div>
                    </div>
                    <div className="text-center">
                      <div className="text-white font-medium">{result.memoryUsage}KB</div>
                      <div className="text-gray-400">Memory</div>
                    </div>
                    <div className="text-center">
                      <div className="text-white font-medium">{hintsUsed}</div>
                      <div className="text-gray-400">Hints Used</div>
                    </div>
                  </div>
                </div>

                {/* Individual Test Cases */}
                <div className="space-y-2">
                  <h4 className="text-white font-medium">Test Cases</h4>
                  {result.testResults.map((testResult, index) => (
                    <div
                      key={index}
                      className={`p-3 rounded border ${
                        testResult.passed
                          ? 'bg-green-900 border-green-700'
                          : 'bg-red-900 border-red-700'
                      }`}
                    >
                      <div className="flex items-center gap-2 mb-2">
                        {testResult.passed ? (
                          <CheckCircle className="w-4 h-4 text-green-400" />
                        ) : (
                          <XCircle className="w-4 h-4 text-red-400" />
                        )}
                        <span className="text-sm font-medium text-white">
                          Test Case {index + 1}: {testResult.testCase.description}
                        </span>
                      </div>
                      
                      {testResult.error ? (
                        <div className="text-red-300 text-sm font-mono">
                          Error: {testResult.error}
                        </div>
                      ) : (
                        <div className="text-sm space-y-1">
                          <div className="flex justify-between">
                            <span className="text-gray-400">Expected:</span>
                            <span className="text-white font-mono">{testResult.testCase.expectedOutput}</span>
                          </div>
                          <div className="flex justify-between">
                            <span className="text-gray-400">Got:</span>
                            <span className={`font-mono ${
                              testResult.passed ? 'text-green-400' : 'text-red-400'
                            }`}>
                              {testResult.output}
                            </span>
                          </div>
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              </>
            ) : (
              <div className="text-center text-gray-400 py-8">
                <Clock className="w-12 h-12 mx-auto mb-4 opacity-50" />
                <div>Run your code to see test results</div>
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Code Quality Feedback */}
      {result && (
        <Card className="bg-slate-800 border-slate-700">
          <CardHeader>
            <CardTitle className="text-white">Code Quality Analysis</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div className="text-center">
                <div className="text-2xl font-bold text-blue-400">
                  {calculateCodeQuality(code)}
                </div>
                <div className="text-sm text-gray-400">Quality Score</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-purple-400">
                  {code.split('\n').length}
                </div>
                <div className="text-sm text-gray-400">Lines of Code</div>
              </div>
              <div className="text-center">
                <Badge {...getScoreBadge((result.passed / result.total) * 100)}>
                  {getScoreBadge((result.passed / result.total) * 100).text}
                </Badge>
                <div className="text-sm text-gray-400 mt-1">Performance</div>
              </div>
            </div>
            
            <div className="mt-4 space-y-2">
              <h4 className="text-white font-medium">Suggestions:</h4>
              <ul className="text-sm text-gray-300 space-y-1">
                {code.includes('if') && !code.includes('else') && (
                  <li>• Consider adding else clauses for better completeness</li>
                )}
                {!code.includes('//') && !code.includes('/*') && (
                  <li>• Add comments to explain your code logic</li>
                )}
                {code.split('\n').length > 50 && (
                  <li>• Consider breaking down long functions into smaller ones</li>
                )}
                {!code.includes('try') && !code.includes('except') && (
                  <li>• Add error handling for robustness</li>
                )}
                {calculateCodeQuality(code) < 50 && (
                  <li>• Focus on code structure and documentation</li>
                )}
              </ul>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
};