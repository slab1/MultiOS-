import React from 'react';
import { GameProvider, useGame } from './contexts/GameContext';
import { GameDashboard } from './components/GameDashboard';
import { User } from './types';
import './App.css';

// Sample user for demo purposes
const sampleUser: User = {
  id: 'demo-user-1',
  username: 'OSStudent',
  email: 'student@example.com',
  level: 5,
  xp: 2340,
  completedChallenges: ['memory-beginner', 'scheduling-beginner', 'filesystem-beginner'],
  badges: [
    {
      id: 'bronze_learner',
      name: 'Bronze Learner',
      description: 'Complete 10 challenges',
      icon: '🥉',
      rarity: 'common',
      earnedAt: new Date('2024-01-15'),
    },
    {
      id: 'memory_guru',
      name: 'Memory Management Guru',
      description: 'Excel at memory allocation algorithms',
      icon: '🧠',
      rarity: 'epic',
      earnedAt: new Date('2024-01-20'),
    },
  ],
  achievements: [
    {
      id: 'first_challenge',
      title: 'Getting Started',
      description: 'Complete your first coding challenge',
      requirement: 1,
      progress: 1,
      completed: true,
      reward: 50,
    },
  ],
  stats: {
    gamesPlayed: 25,
    totalPlayTime: 480, // 8 hours
    challengesCompleted: 8,
    averageScore: 78.5,
    streakCount: 3,
    ranking: 15,
    multiplayerWins: 2,
    multiplayerLosses: 1,
  },
};

const AppContent: React.FC = () => {
  const { dispatch } = useGame();

  React.useEffect(() => {
    // Initialize with sample user
    dispatch({ type: 'SET_USER', payload: sampleUser });
    
    // Load challenges
    const challenges = [
      {
        id: 'memory-basic',
        title: 'Memory Allocation Basics',
        description: 'Learn basic memory allocation concepts',
        difficulty: 'beginner' as const,
        category: 'memory' as const,
        estimatedTime: 15,
        xpReward: 50,
        prerequisites: [],
        testCases: [],
        hints: ['Consider using First-Fit algorithm'],
      },
    ];
    dispatch({ type: 'SET_CHALLENGES', payload: challenges });
  }, [dispatch]);

  return <GameDashboard />;
};

function App() {
  return (
    <GameProvider>
      <div className="min-h-screen bg-slate-900">
        <AppContent />
      </div>
    </GameProvider>
  );
}

export default App;
