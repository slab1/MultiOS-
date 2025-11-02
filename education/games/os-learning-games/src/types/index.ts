// Core types for the OS Learning Games system

export interface User {
  id: string;
  username: string;
  email: string;
  level: number;
  xp: number;
  completedChallenges: string[];
  badges: Badge[];
  achievements: Achievement[];
  stats: UserStats;
}

export interface Badge {
  id: string;
  name: string;
  description: string;
  icon: string;
  rarity: 'common' | 'rare' | 'epic' | 'legendary';
  earnedAt: Date;
}

export interface Achievement {
  id: string;
  title: string;
  description: string;
  requirement: number;
  progress: number;
  completed: boolean;
  reward: number; // XP reward
}

export interface UserStats {
  gamesPlayed: number;
  totalPlayTime: number; // in minutes
  challengesCompleted: number;
  averageScore: number;
  streakCount: number; // consecutive days
  ranking: number;
  multiplayerWins: number;
  multiplayerLosses: number;
}

export interface GameChallenge {
  id: string;
  title: string;
  description: string;
  difficulty: 'beginner' | 'intermediate' | 'advanced' | 'expert';
  category: 'memory' | 'scheduling' | 'filesystem' | 'process' | 'synchronization' | 'optimization';
  estimatedTime: number; // in minutes
  xpReward: number;
  prerequisites: string[];
  codeTemplate?: string;
  testCases: TestCase[];
  hints: string[];
  storyContext?: string;
}

export interface TestCase {
  id: string;
  input: string;
  expectedOutput: string;
  description: string;
  weight: number; // importance of this test case
}

export interface GameSession {
  id: string;
  userId: string;
  challengeId: string;
  startTime: Date;
  endTime?: Date;
  code: string;
  score: number;
  passedTests: number;
  totalTests: number;
  hintsUsed: number;
  timeBonus: number;
  rank?: 'S' | 'A' | 'B' | 'C' | 'D' | 'F';
}

export interface LeaderboardEntry {
  userId: string;
  username: string;
  score: number;
  category: string;
  timestamp: Date;
}

export interface TutorialStep {
  id: string;
  title: string;
  content: string;
  interactive: boolean;
  action?: string;
  feedback?: string;
}

export interface Tutorial {
  id: string;
  title: string;
  description: string;
  category: string;
  difficulty: string;
  steps: TutorialStep[];
  xpReward: number;
}

export interface SimulationConfig {
  memorySize: number;
  pageSize: number;
  algorithms: string[];
  processes: Process[];
}

export interface Process {
  id: string;
  name: string;
  arrivalTime: number;
  burstTime: number;
  priority?: number;
  memoryRequired: number;
  state: 'ready' | 'running' | 'waiting' | 'terminated';
}

export interface MemoryBlock {
  id: string;
  start: number;
  size: number;
  processId?: string;
  allocated: boolean;
}

export interface GameMode {
  id: string;
  name: string;
  description: string;
  type: 'single' | 'multiplayer' | 'tournament';
  maxPlayers?: number;
  duration?: number; // in minutes
  rules: string[];
}