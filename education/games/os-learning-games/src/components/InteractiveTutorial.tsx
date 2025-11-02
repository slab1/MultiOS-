import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { 
  BookOpen, 
  Play, 
  CheckCircle, 
  ArrowRight, 
  Lightbulb, 
  Star,
  Trophy,
  Target,
  Code,
  Monitor
} from 'lucide-react';
import { Tutorial, TutorialStep } from '../types';

const TUTORIALS: Tutorial[] = [
  {
    id: 'intro-to-os',
    title: 'Introduction to Operating Systems',
    description: 'Learn the fundamental concepts of operating systems',
    category: 'basics',
    difficulty: 'beginner',
    xpReward: 100,
    steps: [
      {
        id: 'step1',
        title: 'What is an Operating System?',
        content: 'An operating system (OS) is software that manages computer hardware and software resources, and provides common services for computer programs. Think of it as the bridge between you and your computer!',
        interactive: false,
      },
      {
        id: 'step2',
        title: 'OS Components',
        content: 'Operating systems consist of several key components: the Kernel (core), Device Drivers, File System, and User Interface. Each plays a crucial role in system operation.',
        interactive: true,
        action: 'Click on each component to learn more',
        feedback: 'Great! You\'ve learned about the main OS components.',
      },
      {
        id: 'step3',
        title: 'Types of Operating Systems',
        content: 'There are several types of OS: Desktop (Windows, macOS, Linux), Mobile (Android, iOS), and Real-time Systems. Each is optimized for different purposes.',
        interactive: false,
      },
      {
        id: 'step4',
        title: 'OS Functions',
        content: 'Operating systems perform essential functions like Process Management, Memory Management, File Management, and Device Management.',
        interactive: true,
        action: 'Match each function with its description',
        feedback: 'Excellent! You understand the core OS functions.',
      },
    ],
  },
  {
    id: 'memory-management-basics',
    title: 'Memory Management Fundamentals',
    description: 'Understand how operating systems manage computer memory',
    category: 'memory',
    difficulty: 'beginner',
    xpReward: 150,
    steps: [
      {
        id: 'step1',
        title: 'What is Memory Management?',
        content: 'Memory management is the OS function that handles, controls, and coordinates computer memory, assigns portions called blocks to various running programs to optimize overall system performance.',
        interactive: false,
      },
      {
        id: 'step2',
        title: 'Physical vs Virtual Memory',
        content: 'Physical memory (RAM) is the actual hardware memory. Virtual memory extends this by using disk space to simulate additional RAM when physical memory is full.',
        interactive: true,
        action: 'Drag the memory blocks to see how they\'re allocated',
        feedback: 'Perfect! You understand the difference between physical and virtual memory.',
      },
      {
        id: 'step3',
        title: 'Memory Allocation',
        content: 'When a program runs, the OS allocates memory blocks using algorithms like First Fit, Best Fit, and Worst Fit. Each has different efficiency characteristics.',
        interactive: true,
        action: 'Try different allocation algorithms',
        feedback: 'Great job! You\'ve experienced different memory allocation strategies.',
      },
      {
        id: 'step4',
        title: 'Fragmentation',
        content: 'Memory fragmentation occurs when free memory is broken into small, non-contiguous blocks. This reduces efficiency and can cause allocation failures.',
        interactive: true,
        action: 'Identify fragmented memory patterns',
        feedback: 'Excellent! You can recognize memory fragmentation.',
      },
    ],
  },
  {
    id: 'process-scheduling',
    title: 'CPU Process Scheduling',
    description: 'Learn how operating systems schedule processes on the CPU',
    category: 'scheduling',
    difficulty: 'intermediate',
    xpReward: 200,
    steps: [
      {
        id: 'step1',
        title: 'What is Process Scheduling?',
        content: 'Process scheduling is the activity of the operating system that handles the removal of the running process from the CPU and the selection of another process on the basis of a particular strategy.',
        interactive: false,
      },
      {
        id: 'step2',
        title: 'Scheduling Algorithms',
        content: 'Different scheduling algorithms prioritize different goals: FCFS (fairness), SJF (efficiency), Priority (importance), and Round Robin (responsiveness).',
        interactive: true,
        action: 'Configure processes and see how different algorithms affect them',
        feedback: 'You\'re getting the hang of scheduling algorithms!',
      },
      {
        id: 'step3',
        title: 'Context Switching',
        content: 'Context switching is the process of storing and restoring the state of a CPU so that multiple processes can share a single CPU execution time.',
        interactive: true,
        action: 'Watch a context switch in action',
        feedback: 'Great! You understand how context switching works.',
      },
      {
        id: 'step4',
        title: 'Performance Metrics',
        content: 'Key performance metrics include CPU Utilization, Throughput, Turnaround Time, Waiting Time, and Response Time. Each measures different aspects of system performance.',
        interactive: true,
        action: 'Analyze performance metrics for different scenarios',
        feedback: 'Excellent! You can evaluate scheduling performance.',
      },
    ],
  },
  {
    id: 'file-systems',
    title: 'File System Organization',
    description: 'Master file system concepts and organization',
    category: 'filesystem',
    difficulty: 'intermediate',
    xpReward: 175,
    steps: [
      {
        id: 'step1',
        title: 'File System Basics',
        content: 'A file system controls how data is stored and retrieved from storage devices. It organizes files into directories and manages file metadata like permissions and timestamps.',
        interactive: false,
      },
      {
        id: 'step2',
        title: 'Directory Structure',
        content: 'File systems use hierarchical directory structures with a root directory containing files and subdirectories. This organization makes file management intuitive.',
        interactive: true,
        action: 'Create a file system hierarchy',
        feedback: 'Perfect! You understand directory structures.',
      },
      {
        id: 'step3',
        title: 'File Permissions',
        content: 'File permissions control who can read, write, or execute files. They include owner, group, and others permissions with read, write, and execute rights.',
        interactive: true,
        action: 'Set file permissions for different scenarios',
        feedback: 'Great! You understand file permission systems.',
      },
      {
        id: 'step4',
        title: 'File Operations',
        content: 'Common file operations include creating, opening, reading, writing, closing, and deleting files. Each operation has specific requirements and effects.',
        interactive: true,
        action: 'Perform various file operations',
        feedback: 'Excellent! You can perform file system operations.',
      },
    ],
  },
];

interface InteractiveTutorialProps {
  tutorialId: string;
  onComplete: (tutorialId: string, xpEarned: number) => void;
}

export const InteractiveTutorial: React.FC<InteractiveTutorialProps> = ({ 
  tutorialId, 
  onComplete 
}) => {
  const [tutorial, setTutorial] = useState<Tutorial | null>(null);
  const [currentStepIndex, setCurrentStepIndex] = useState(0);
  const [completedSteps, setCompletedSteps] = useState<Set<string>>(new Set());
  const [stepProgress, setStepProgress] = useState<Record<string, number>>({});
  const [showReward, setShowReward] = useState(false);
  const [earnedXp, setEarnedXp] = useState(0);

  useEffect(() => {
    const foundTutorial = TUTORIALS.find(t => t.id === tutorialId);
    setTutorial(foundTutorial || null);
  }, [tutorialId]);

  const currentStep = tutorial?.steps[currentStepIndex];
  const isLastStep = currentStepIndex === (tutorial?.steps.length || 0) - 1;
  const progress = tutorial ? ((currentStepIndex + 1) / tutorial.steps.length) * 100 : 0;

  const completeStep = (stepId: string) => {
    const newCompletedSteps = new Set(completedSteps);
    newCompletedSteps.add(stepId);
    setCompletedSteps(newCompletedSteps);

    // Calculate progress for interactive steps
    if (currentStep?.interactive) {
      setStepProgress(prev => ({
        ...prev,
        [stepId]: 100,
      }));
    }

    // Move to next step or complete tutorial
    if (isLastStep) {
      completeTutorial();
    } else {
      setCurrentStepIndex(prev => prev + 1);
    }
  };

  const completeTutorial = () => {
    if (tutorial) {
      const xpEarned = tutorial.xpReward;
      setEarnedXp(xpEarned);
      setShowReward(true);
      onComplete(tutorial.id, xpEarned);
    }
  };

  const retryStep = () => {
    setStepProgress(prev => ({
      ...prev,
      [currentStep!.id]: 0,
    }));
  };

  const resetTutorial = () => {
    setCurrentStepIndex(0);
    setCompletedSteps(new Set());
    setStepProgress({});
    setShowReward(false);
  };

  if (!tutorial) {
    return (
      <Card className="bg-slate-800 border-slate-700">
        <CardContent className="pt-6 text-center">
          <div className="text-white">Tutorial not found</div>
        </CardContent>
      </Card>
    );
  }

  if (showReward) {
    return (
      <Card className="bg-gradient-to-r from-green-600 to-blue-600 border-0">
        <CardContent className="pt-6 text-center">
          <div className="text-6xl mb-4">🎉</div>
          <h3 className="text-2xl font-bold text-white mb-2">Tutorial Complete!</h3>
          <p className="text-white/90 mb-4">{tutorial.title}</p>
          <div className="space-y-2">
            <div className="flex items-center justify-center gap-2 text-white">
              <Star className="w-5 h-5" />
              <span className="text-xl font-bold">+{earnedXp} XP Earned</span>
            </div>
            <div className="text-white/80">
              Great job! You've completed all steps in this tutorial.
            </div>
          </div>
          <div className="mt-6 space-x-2">
            <Button onClick={resetTutorial} variant="outline" className="text-white border-white">
              Restart Tutorial
            </Button>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-6">
      {/* Tutorial Header */}
      <Card className="bg-slate-800 border-slate-700">
        <CardHeader>
          <div className="flex justify-between items-start">
            <div>
              <div className="flex items-center gap-3 mb-2">
                <CardTitle className="text-xl text-white">{tutorial.title}</CardTitle>
                <Badge variant="outline" className="capitalize">
                  {tutorial.difficulty}
                </Badge>
                <Badge variant="secondary">
                  {tutorial.category}
                </Badge>
              </div>
              <p className="text-gray-300">{tutorial.description}</p>
            </div>
            <div className="text-right">
              <div className="text-sm text-gray-400">Progress</div>
              <div className="text-lg font-bold text-white">{Math.round(progress)}%</div>
              <div className="text-sm text-gray-400">{tutorial.xpReward} XP</div>
            </div>
          </div>
        </CardHeader>
      </Card>

      {/* Progress Bar */}
      <Card className="bg-slate-800 border-slate-700">
        <CardContent className="pt-6">
          <div className="space-y-3">
            <div className="flex justify-between text-sm">
              <span className="text-gray-300">Step {currentStepIndex + 1} of {tutorial.steps.length}</span>
              <span className="text-gray-300">
                {completedSteps.size} / {tutorial.steps.length} completed
              </span>
            </div>
            <Progress value={progress} className="h-3" />
          </div>
        </CardContent>
      </Card>

      {/* Step Navigation */}
      <div className="flex space-x-2 overflow-x-auto pb-2">
        {tutorial.steps.map((step, index) => (
          <Button
            key={step.id}
            variant={index === currentStepIndex ? 'default' : 'outline'}
            size="sm"
            onClick={() => setCurrentStepIndex(index)}
            disabled={index > currentStepIndex + 1}
            className="whitespace-nowrap"
          >
            {completedSteps.has(step.id) && (
              <CheckCircle className="w-4 h-4 mr-1" />
            )}
            {index + 1}. {step.title}
          </Button>
        ))}
      </div>

      {/* Current Step */}
      {currentStep && (
        <Card className="bg-slate-800 border-slate-700">
          <CardHeader>
            <CardTitle className="text-white flex items-center gap-2">
              {currentStep.interactive ? (
                <Target className="w-5 h-5 text-blue-400" />
              ) : (
                <BookOpen className="w-5 h-5 text-green-400" />
              )}
              {currentStep.title}
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-6">
            {/* Step Content */}
            <div className="prose prose-invert max-w-none">
              <p className="text-gray-300 leading-relaxed">{currentStep.content}</p>
            </div>

            {/* Interactive Elements */}
            {currentStep.interactive && (
              <div className="bg-slate-700 p-4 rounded-lg">
                <div className="flex items-center gap-2 mb-3">
                  <Lightbulb className="w-5 h-5 text-yellow-400" />
                  <span className="text-white font-medium">Interactive Activity</span>
                </div>
                <p className="text-gray-300 mb-4">{currentStep.action}</p>
                
                {/* Simulated Interactive Component */}
                <div className="bg-slate-800 p-4 rounded border-2 border-dashed border-gray-600">
                  <div className="text-center text-gray-400">
                    <Code className="w-12 h-12 mx-auto mb-2" />
                    <div>Interactive demonstration would be here</div>
                    <div className="text-sm">This is a simulation - in a real app, this would contain the actual interactive element</div>
                  </div>
                </div>
                
                {currentStep.feedback && stepProgress[currentStep.id] > 0 && (
                  <div className="mt-3 p-3 bg-green-900 border border-green-600 rounded">
                    <div className="text-green-200 text-sm">
                      <CheckCircle className="w-4 h-4 inline mr-1" />
                      {currentStep.feedback}
                    </div>
                  </div>
                )}
              </div>
            )}

            {/* Navigation Buttons */}
            <div className="flex justify-between">
              <Button
                variant="outline"
                onClick={() => setCurrentStepIndex(prev => Math.max(0, prev - 1))}
                disabled={currentStepIndex === 0}
              >
                Previous
              </Button>
              
              <div className="space-x-2">
                {!currentStep.interactive && (
                  <Button
                    variant="outline"
                    onClick={retryStep}
                  >
                    Retry
                  </Button>
                )}
                
                <Button
                  onClick={() => completeStep(currentStep.id)}
                  disabled={currentStep.interactive && stepProgress[currentStep.id] < 100}
                >
                  {isLastStep ? (
                    <>
                      <Trophy className="w-4 h-4 mr-2" />
                      Complete Tutorial
                    </>
                  ) : (
                    <>
                      Next Step
                      <ArrowRight className="w-4 h-4 ml-2" />
                    </>
                  )}
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Tutorial Overview */}
      <Card className="bg-slate-800 border-slate-700">
        <CardHeader>
          <CardTitle className="text-white">Tutorial Overview</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            {tutorial.steps.map((step, index) => (
              <div
                key={step.id}
                className={`p-3 rounded border transition-all ${
                  index === currentStepIndex
                    ? 'bg-blue-900 border-blue-500'
                    : completedSteps.has(step.id)
                    ? 'bg-green-900 border-green-600'
                    : 'bg-slate-700 border-slate-600'
                }`}
              >
                <div className="flex items-center gap-3">
                  <div className={`w-8 h-8 rounded-full flex items-center justify-center ${
                    index === currentStepIndex
                      ? 'bg-blue-600 text-white'
                      : completedSteps.has(step.id)
                      ? 'bg-green-600 text-white'
                      : 'bg-slate-600 text-gray-300'
                  }`}>
                    {completedSteps.has(step.id) ? (
                      <CheckCircle className="w-4 h-4" />
                    ) : (
                      <span className="text-sm font-bold">{index + 1}</span>
                    )}
                  </div>
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      <span className={`font-medium ${
                        index === currentStepIndex ? 'text-blue-300' : 
                        completedSteps.has(step.id) ? 'text-green-300' : 'text-white'
                      }`}>
                        {step.title}
                      </span>
                      {step.interactive && (
                        <Badge variant="outline" className="text-xs">
                          Interactive
                        </Badge>
                      )}
                    </div>
                    <div className="text-sm text-gray-400">
                      {step.interactive ? 'Hands-on activity' : 'Theory and concepts'}
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Completion Progress */}
      <Card className="bg-slate-800 border-slate-700">
        <CardContent className="pt-6">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-center">
            <div>
              <div className="text-2xl font-bold text-white">{completedSteps.size}</div>
              <div className="text-sm text-gray-400">Steps Completed</div>
            </div>
            <div>
              <div className="text-2xl font-bold text-blue-400">{Math.round(progress)}%</div>
              <div className="text-sm text-gray-400">Overall Progress</div>
            </div>
            <div>
              <div className="text-2xl font-bold text-yellow-400">{tutorial.xpReward}</div>
              <div className="text-sm text-gray-400">XP Reward</div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};