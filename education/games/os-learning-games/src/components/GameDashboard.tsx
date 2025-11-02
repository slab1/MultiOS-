import React, { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { 
  GamepadIcon, 
  BookOpen, 
  Trophy, 
  Users, 
  Zap,
  Brain,
  Cpu,
  HardDrive,
  Code,
  Target,
  Star,
  TrendingUp,
  Clock
} from 'lucide-react';
import { MemoryManagementGame } from './games/MemoryManagementGame';
import { CPUSchedulingGame } from './games/CPUSchedulingGame';
import { FileSystemGame } from './games/FileSystemGame';
import { CodeChallengeGame } from './games/CodeChallengeGame';
import { AchievementSystem } from './AchievementSystem';
import { Leaderboard } from './Leaderboard';
import { InteractiveTutorial } from './InteractiveTutorial';
import { useGame } from '../contexts/GameContext';
import { GameChallenge } from '../types';

const GAME_CHALLENGES: GameChallenge[] = [
  {
    id: 'memory-allocation-1',
    title: 'Basic Memory Allocation',
    description: 'Allocate memory blocks efficiently using First-Fit algorithm',
    difficulty: 'beginner',
    category: 'memory',
    estimatedTime: 10,
    xpReward: 50,
    prerequisites: [],
    testCases: [
      {
        id: 'test1',
        input: 'processes=[{size:100},{size:50},{size:75}]',
        expectedOutput: 'Success',
        description: 'Basic allocation test',
        weight: 1,
      },
    ],
    hints: [
      'Start by allocating the largest process first',
      'Consider using First-Fit algorithm',
    ],
  },
  {
    id: 'process-scheduling-1',
    title: 'FCFS Scheduling',
    description: 'Implement First-Come-First-Served scheduling algorithm',
    difficulty: 'beginner',
    category: 'scheduling',
    estimatedTime: 15,
    xpReward: 75,
    prerequisites: [],
    testCases: [
      {
        id: 'test1',
        input: 'processes=[{arrival:0,burst:5},{arrival:1,burst:3}]',
        expectedOutput: '[0,5]',
        description: 'Basic FCFS test',
        weight: 1,
      },
    ],
    hints: [
      'Process that arrives first gets CPU first',
      'No preemption in FCFS',
    ],
  },
  {
    id: 'file-organization-1',
    title: 'Directory Structure',
    description: 'Create an efficient directory hierarchy',
    difficulty: 'intermediate',
    category: 'filesystem',
    estimatedTime: 20,
    xpReward: 100,
    prerequisites: [],
    testCases: [
      {
        id: 'test1',
        input: 'structure={"files":10,"directories":3}',
        expectedOutput: 'Organized',
        description: 'Basic organization test',
        weight: 1,
      },
    ],
    hints: [
      'Group related files together',
      'Use meaningful directory names',
    ],
  },
];

export const GameDashboard: React.FC = () => {
  const { state, dispatch } = useGame();
  const [currentView, setCurrentView] = useState<'dashboard' | 'game' | 'achievements' | 'leaderboard' | 'tutorials'>('dashboard');
  const [selectedGame, setSelectedGame] = useState<'memory' | 'scheduling' | 'filesystem' | 'challenge'>('memory');
  const [selectedChallenge, setSelectedChallenge] = useState<GameChallenge | null>(null);
  const [selectedDifficulty, setSelectedDifficulty] = useState<'beginner' | 'intermediate' | 'advanced'>('beginner');

  const handleGameStart = (gameType: typeof selectedGame, difficulty: typeof selectedDifficulty) => {
    setSelectedGame(gameType);
    setSelectedDifficulty(difficulty);
    setCurrentView('game');
  };

  const handleChallengeComplete = (score: number, passedTests: number, totalTests: number) => {
    // Update user progress
    if (selectedChallenge) {
      dispatch({
        type: 'COMPLETE_CHALLENGE',
        payload: {
          challengeId: selectedChallenge.id,
          xpEarned: selectedChallenge.xpReward,
        },
      });
    }
    
    // Show completion message and return to dashboard
    setTimeout(() => {
      setCurrentView('dashboard');
    }, 3000);
  };

  const renderGameContent = () => {
    if (selectedGame === 'memory') {
      return (
        <MemoryManagementGame
          difficulty={selectedDifficulty}
          onComplete={(score) => handleGameComplete(score, 0, 0)}
        />
      );
    }
    
    if (selectedGame === 'scheduling') {
      return (
        <CPUSchedulingGame
          difficulty={selectedDifficulty}
          onComplete={(score, metrics) => handleGameComplete(score, 0, 0)}
        />
      );
    }
    
    if (selectedGame === 'filesystem') {
      return (
        <FileSystemGame
          difficulty={selectedDifficulty}
          onComplete={(score, metrics) => handleGameComplete(score, 0, 0)}
        />
      );
    }
    
    if (selectedGame === 'challenge' && selectedChallenge) {
      return (
        <CodeChallengeGame
          challenge={selectedChallenge}
          onComplete={handleChallengeComplete}
        />
      );
    }

    return null;
  };

  const handleGameComplete = (score: number, passed: number, total: number) => {
    // Simulate completion
    dispatch({
      type: 'UPDATE_USER_PROGRESS',
      payload: {
        totalXp: state.userProgress.totalXp + Math.floor(score / 10),
        completedChallenges: [...state.userProgress.completedChallenges, selectedGame + '-' + selectedDifficulty],
      },
    });
    
    setTimeout(() => {
      setCurrentView('dashboard');
    }, 2000);
  };

  const getDifficultyStats = (difficulty: string) => {
    const completed = state.userProgress.completedChallenges.filter(id => id.includes(difficulty)).length;
    return { completed, total: 3 };
  };

  if (currentView === 'achievements') {
    return (
      <div className="min-h-screen bg-slate-900 p-6">
        <div className="max-w-7xl mx-auto">
          <Button
            onClick={() => setCurrentView('dashboard')}
            className="mb-6"
            variant="outline"
          >
            ← Back to Dashboard
          </Button>
          {state.user && (
            <AchievementSystem
              user={state.user}
              onUserUpdate={(user) => dispatch({ type: 'SET_USER', payload: user })}
            />
          )}
        </div>
      </div>
    );
  }

  if (currentView === 'leaderboard') {
    return (
      <div className="min-h-screen bg-slate-900 p-6">
        <div className="max-w-7xl mx-auto">
          <Button
            onClick={() => setCurrentView('dashboard')}
            className="mb-6"
            variant="outline"
          >
            ← Back to Dashboard
          </Button>
          {state.user && (
            <Leaderboard
              currentUser={state.user}
              category="global"
            />
          )}
        </div>
      </div>
    );
  }

  if (currentView === 'tutorials') {
    return (
      <div className="min-h-screen bg-slate-900 p-6">
        <div className="max-w-7xl mx-auto">
          <Button
            onClick={() => setCurrentView('dashboard')}
            className="mb-6"
            variant="outline"
          >
            ← Back to Dashboard
          </Button>
          <InteractiveTutorial
            tutorialId="intro-to-os"
            onComplete={(id, xp) => {
              dispatch({
                type: 'UPDATE_USER_PROGRESS',
                payload: { totalXp: state.userProgress.totalXp + xp },
              });
            }}
          />
        </div>
      </div>
    );
  }

  if (currentView === 'game') {
    return (
      <div className="min-h-screen bg-slate-900 p-6">
        <div className="max-w-7xl mx-auto">
          <Button
            onClick={() => setCurrentView('dashboard')}
            className="mb-6"
            variant="outline"
          >
            ← Back to Dashboard
          </Button>
          {renderGameContent()}
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-slate-900 p-6">
      <div className="max-w-7xl mx-auto space-y-6">
        {/* Header */}
        <div className="flex justify-between items-center">
          <div>
            <h1 className="text-3xl font-bold text-white flex items-center gap-3">
              <GamepadIcon className="w-8 h-8" />
              OS Learning Games
            </h1>
            <p className="text-gray-300 mt-1">Master operating system concepts through interactive games</p>
          </div>
          <div className="text-right">
            <div className="text-2xl font-bold text-yellow-400">{state.userProgress.totalXp} XP</div>
            <div className="text-sm text-gray-400">Level {Math.floor(state.userProgress.totalXp / 1000) + 1}</div>
          </div>
        </div>

        {/* Quick Stats */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <Card className="bg-slate-800 border-slate-700">
            <CardContent className="pt-6 text-center">
              <Trophy className="w-8 h-8 mx-auto text-yellow-400 mb-2" />
              <div className="text-2xl font-bold text-white">{state.userProgress.completedChallenges.length}</div>
              <div className="text-sm text-gray-400">Challenges Completed</div>
            </CardContent>
          </Card>
          
          <Card className="bg-slate-800 border-slate-700">
            <CardContent className="pt-6 text-center">
              <Star className="w-8 h-8 mx-auto text-blue-400 mb-2" />
              <div className="text-2xl font-bold text-white">{state.user?.badges.length || 0}</div>
              <div className="text-sm text-gray-400">Badges Earned</div>
            </CardContent>
          </Card>
          
          <Card className="bg-slate-800 border-slate-700">
            <CardContent className="pt-6 text-center">
              <TrendingUp className="w-8 h-8 mx-auto text-green-400 mb-2" />
              <div className="text-2xl font-bold text-white">
                {Math.round((state.userProgress.completedChallenges.length / 20) * 100)}%
              </div>
              <div className="text-sm text-gray-400">Progress</div>
            </CardContent>
          </Card>
          
          <Card className="bg-slate-800 border-slate-700">
            <CardContent className="pt-6 text-center">
              <Clock className="w-8 h-8 mx-auto text-purple-400 mb-2" />
              <div className="text-2xl font-bold text-white">{state.userProgress.currentStreak}</div>
              <div className="text-sm text-gray-400">Day Streak</div>
            </CardContent>
          </Card>
        </div>

        {/* Navigation */}
        <div className="flex flex-wrap gap-2">
          <Button
            onClick={() => setCurrentView('dashboard')}
            variant="default"
            className="flex items-center gap-2"
          >
            <GamepadIcon className="w-4 h-4" />
            Games
          </Button>
          <Button
            onClick={() => setCurrentView('achievements')}
            variant="outline"
            className="flex items-center gap-2"
          >
            <Trophy className="w-4 h-4" />
            Achievements
          </Button>
          <Button
            onClick={() => setCurrentView('leaderboard')}
            variant="outline"
            className="flex items-center gap-2"
          >
            <Users className="w-4 h-4" />
            Leaderboard
          </Button>
          <Button
            onClick={() => setCurrentView('tutorials')}
            variant="outline"
            className="flex items-center gap-2"
          >
            <BookOpen className="w-4 h-4" />
            Tutorials
          </Button>
        </div>

        {/* Main Game Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {/* Memory Management */}
          <Card className="bg-slate-800 border-slate-700 hover:border-blue-500 transition-all cursor-pointer">
            <CardHeader>
              <CardTitle className="text-white flex items-center gap-2">
                <Brain className="w-6 h-6 text-blue-400" />
                Memory Management
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <p className="text-gray-300">
                Learn memory allocation algorithms and fragmentation management through interactive simulations.
              </p>
              <div className="space-y-2">
                <div className="flex justify-between items-center">
                  <span className="text-sm text-gray-400">Beginner</span>
                  <Badge variant="outline">{getDifficultyStats('beginner').completed}/3</Badge>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm text-gray-400">Intermediate</span>
                  <Badge variant="outline">{getDifficultyStats('intermediate').completed}/3</Badge>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm text-gray-400">Advanced</span>
                  <Badge variant="outline">{getDifficultyStats('advanced').completed}/3</Badge>
                </div>
              </div>
              <div className="flex gap-2">
                <Button onClick={() => handleGameStart('memory', 'beginner')} size="sm">
                  Start Beginner
                </Button>
                <Button onClick={() => handleGameStart('memory', 'intermediate')} size="sm" variant="outline">
                  Start Intermediate
                </Button>
                <Button onClick={() => handleGameStart('memory', 'advanced')} size="sm" variant="outline">
                  Start Advanced
                </Button>
              </div>
            </CardContent>
          </Card>

          {/* CPU Scheduling */}
          <Card className="bg-slate-800 border-slate-700 hover:border-green-500 transition-all cursor-pointer">
            <CardHeader>
              <CardTitle className="text-white flex items-center gap-2">
                <Cpu className="w-6 h-6 text-green-400" />
                CPU Scheduling
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <p className="text-gray-300">
                Master CPU scheduling algorithms and optimize process execution times.
              </p>
              <div className="space-y-2">
                <div className="flex justify-between items-center">
                  <span className="text-sm text-gray-400">Beginner</span>
                  <Badge variant="outline">{getDifficultyStats('beginner').completed}/3</Badge>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm text-gray-400">Intermediate</span>
                  <Badge variant="outline">{getDifficultyStats('intermediate').completed}/3</Badge>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm text-gray-400">Advanced</span>
                  <Badge variant="outline">{getDifficultyStats('advanced').completed}/3</Badge>
                </div>
              </div>
              <div className="flex gap-2">
                <Button onClick={() => handleGameStart('scheduling', 'beginner')} size="sm">
                  Start Beginner
                </Button>
                <Button onClick={() => handleGameStart('scheduling', 'intermediate')} size="sm" variant="outline">
                  Start Intermediate
                </Button>
                <Button onClick={() => handleGameStart('scheduling', 'advanced')} size="sm" variant="outline">
                  Start Advanced
                </Button>
              </div>
            </CardContent>
          </Card>

          {/* File Systems */}
          <Card className="bg-slate-800 border-slate-700 hover:border-purple-500 transition-all cursor-pointer">
            <CardHeader>
              <CardTitle className="text-white flex items-center gap-2">
                <HardDrive className="w-6 h-6 text-purple-400" />
                File Systems
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <p className="text-gray-300">
                Navigate and organize file systems, learn about permissions and directory structures.
              </p>
              <div className="space-y-2">
                <div className="flex justify-between items-center">
                  <span className="text-sm text-gray-400">Beginner</span>
                  <Badge variant="outline">{getDifficultyStats('beginner').completed}/3</Badge>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm text-gray-400">Intermediate</span>
                  <Badge variant="outline">{getDifficultyStats('intermediate').completed}/3</Badge>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm text-gray-400">Advanced</span>
                  <Badge variant="outline">{getDifficultyStats('advanced').completed}/3</Badge>
                </div>
              </div>
              <div className="flex gap-2">
                <Button onClick={() => handleGameStart('filesystem', 'beginner')} size="sm">
                  Start Beginner
                </Button>
                <Button onClick={() => handleGameStart('filesystem', 'intermediate')} size="sm" variant="outline">
                  Start Intermediate
                </Button>
                <Button onClick={() => handleGameStart('filesystem', 'advanced')} size="sm" variant="outline">
                  Start Advanced
                </Button>
              </div>
            </CardContent>
          </Card>

          {/* Code Challenges */}
          <Card className="bg-slate-800 border-slate-700 hover:border-yellow-500 transition-all cursor-pointer">
            <CardHeader>
              <CardTitle className="text-white flex items-center gap-2">
                <Code className="w-6 h-6 text-yellow-400" />
                Code Challenges
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <p className="text-gray-300">
                Solve OS-related programming problems with immediate visual feedback.
              </p>
              <div className="space-y-2">
                <div className="text-sm text-gray-400">Available Challenges</div>
                {GAME_CHALLENGES.map((challenge) => (
                  <div key={challenge.id} className="flex justify-between items-center">
                    <div>
                      <div className="text-sm text-white">{challenge.title}</div>
                      <div className="text-xs text-gray-400">{challenge.category}</div>
                    </div>
                    <div className="flex items-center gap-2">
                      <Badge variant="outline">{challenge.difficulty}</Badge>
                      <Button
                        size="sm"
                        onClick={() => {
                          setSelectedChallenge(challenge);
                          setSelectedGame('challenge');
                          setCurrentView('game');
                        }}
                      >
                        <Zap className="w-3 h-3 mr-1" />
                        {challenge.xpReward} XP
                      </Button>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Quick Access */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <Card className="bg-slate-800 border-slate-700">
            <CardHeader>
              <CardTitle className="text-white flex items-center gap-2">
                <Target className="w-5 h-5" />
                Performance Optimization
              </CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-gray-300 mb-4">
                Challenge yourself with performance optimization scenarios and benchmark improvements.
              </p>
              <Button variant="outline" className="w-full" disabled>
                Coming Soon
              </Button>
            </CardContent>
          </Card>

          <Card className="bg-slate-800 border-slate-700">
            <CardHeader>
              <CardTitle className="text-white flex items-center gap-2">
                <Users className="w-5 h-5" />
                Multiplayer Debugging
              </CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-gray-300 mb-4">
                Compete with friends in real-time debugging competitions and collaborative problem solving.
              </p>
              <Button variant="outline" className="w-full" disabled>
                Coming Soon
              </Button>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
};