import React, { createContext, useContext, useReducer, ReactNode, useEffect } from 'react';
import { User, GameChallenge, GameSession, Badge, Achievement } from '../types';

interface GameState {
  user: User | null;
  currentChallenge: GameChallenge | null;
  gameSession: GameSession | null;
  allChallenges: GameChallenge[];
  userProgress: {
    completedChallenges: string[];
    totalXp: number;
    currentStreak: number;
    bestScores: Record<string, number>;
  };
  achievements: Achievement[];
  badges: Badge[];
  leaderboard: any[];
  isLoading: boolean;
  error: string | null;
}

type GameAction =
  | { type: 'SET_USER'; payload: User }
  | { type: 'SET_CURRENT_CHALLENGE'; payload: GameChallenge | null }
  | { type: 'START_GAME_SESSION'; payload: { challengeId: string; userId: string } }
  | { type: 'UPDATE_GAME_SESSION'; payload: Partial<GameSession> }
  | { type: 'END_GAME_SESSION'; payload: { score: number; passedTests: number; totalTests: number } }
  | { type: 'SET_CHALLENGES'; payload: GameChallenge[] }
  | { type: 'COMPLETE_CHALLENGE'; payload: { challengeId: string; xpEarned: number; badge?: Badge } }
  | { type: 'UNLOCK_ACHIEVEMENT'; payload: Achievement }
  | { type: 'SET_LOADING'; payload: boolean }
  | { type: 'SET_ERROR'; payload: string | null }
  | { type: 'UPDATE_USER_PROGRESS'; payload: Partial<GameState['userProgress']> };

const initialState: GameState = {
  user: null,
  currentChallenge: null,
  gameSession: null,
  allChallenges: [],
  userProgress: {
    completedChallenges: [],
    totalXp: 0,
    currentStreak: 0,
    bestScores: {},
  },
  achievements: [],
  badges: [],
  leaderboard: [],
  isLoading: false,
  error: null,
};

const gameReducer = (state: GameState, action: GameAction): GameState => {
  switch (action.type) {
    case 'SET_USER':
      return { ...state, user: action.payload };
    
    case 'SET_CURRENT_CHALLENGE':
      return { ...state, currentChallenge: action.payload };
    
    case 'START_GAME_SESSION':
      return {
        ...state,
        gameSession: {
          id: Date.now().toString(),
          userId: action.payload.userId,
          challengeId: action.payload.challengeId,
          startTime: new Date(),
          code: '',
          score: 0,
          passedTests: 0,
          totalTests: 0,
          hintsUsed: 0,
          timeBonus: 0,
        },
      };
    
    case 'UPDATE_GAME_SESSION':
      return {
        ...state,
        gameSession: state.gameSession ? { ...state.gameSession, ...action.payload } : null,
      };
    
    case 'END_GAME_SESSION':
      if (!state.gameSession) return state;
      
      const endTime = new Date();
      const timeSpent = Math.floor((endTime.getTime() - state.gameSession.startTime.getTime()) / 1000 / 60);
      const timeBonus = Math.max(0, 10 - timeSpent); // Bonus for speed
      const finalScore = action.payload.score + timeBonus;
      
      const rank = getScoreRank(finalScore);
      const xpEarned = calculateXp(action.payload.passedTests, action.payload.totalTests, timeSpent);
      
      return {
        ...state,
        gameSession: {
          ...state.gameSession,
          endTime,
          score: finalScore,
          passedTests: action.payload.passedTests,
          totalTests: action.payload.totalTests,
          rank,
        },
        userProgress: {
          ...state.userProgress,
          bestScores: {
            ...state.userProgress.bestScores,
            [state.gameSession.challengeId]: Math.max(
              state.userProgress.bestScores[state.gameSession.challengeId] || 0,
              finalScore
            ),
          },
        },
      };
    
    case 'SET_CHALLENGES':
      return { ...state, allChallenges: action.payload };
    
    case 'COMPLETE_CHALLENGE':
      const newCompletedChallenges = [...state.userProgress.completedChallenges];
      if (!newCompletedChallenges.includes(action.payload.challengeId)) {
        newCompletedChallenges.push(action.payload.challengeId);
      }
      
      return {
        ...state,
        userProgress: {
          ...state.userProgress,
          completedChallenges: newCompletedChallenges,
          totalXp: state.userProgress.totalXp + action.payload.xpEarned,
        },
        badges: action.payload.badge ? [...state.badges, action.payload.badge] : state.badges,
      };
    
    case 'UNLOCK_ACHIEVEMENT':
      return {
        ...state,
        achievements: state.achievements.map(achievement =>
          achievement.id === action.payload.id ? { ...achievement, completed: true } : achievement
        ),
      };
    
    case 'UPDATE_USER_PROGRESS':
      return {
        ...state,
        userProgress: { ...state.userProgress, ...action.payload },
      };
    
    case 'SET_LOADING':
      return { ...state, isLoading: action.payload };
    
    case 'SET_ERROR':
      return { ...state, error: action.payload };
    
    default:
      return state;
  }
};

function getScoreRank(score: number): 'S' | 'A' | 'B' | 'C' | 'D' | 'F' {
  if (score >= 90) return 'S';
  if (score >= 80) return 'A';
  if (score >= 70) return 'B';
  if (score >= 60) return 'C';
  if (score >= 50) return 'D';
  return 'F';
}

function calculateXp(passedTests: number, totalTests: number, timeSpent: number): number {
  const baseXp = Math.floor((passedTests / totalTests) * 100);
  const timeBonus = Math.max(0, 20 - timeSpent);
  return baseXp + timeBonus;
}

interface GameContextType {
  state: GameState;
  dispatch: React.Dispatch<GameAction>;
  // Helper functions
  startChallenge: (challenge: GameChallenge) => void;
  submitCode: (code: string) => Promise<{ passed: number; total: number; score: number }>;
  useHint: () => void;
  completeChallenge: () => void;
}

const GameContext = createContext<GameContextType | undefined>(undefined);

export const GameProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [state, dispatch] = useReducer(gameReducer, initialState);

  const startChallenge = (challenge: GameChallenge) => {
    if (state.user) {
      dispatch({ type: 'SET_CURRENT_CHALLENGE', payload: challenge });
      dispatch({ 
        type: 'START_GAME_SESSION', 
        payload: { challengeId: challenge.id, userId: state.user.id } 
      });
    }
  };

  const submitCode = async (code: string): Promise<{ passed: number; total: number; score: number }> => {
    // This would integrate with the actual code execution environment
    dispatch({ type: 'UPDATE_GAME_SESSION', payload: { code } });
    
    // Mock implementation - in real app, this would execute the code
    const passed = Math.floor(Math.random() * state.currentChallenge!.testCases.length) + 1;
    const total = state.currentChallenge!.testCases.length;
    const score = Math.floor((passed / total) * 100);
    
    dispatch({ type: 'END_GAME_SESSION', payload: { score, passedTests: passed, totalTests: total } });
    
    return { passed, total, score };
  };

  const useHint = () => {
    dispatch({ type: 'UPDATE_GAME_SESSION', payload: { hintsUsed: (state.gameSession?.hintsUsed || 0) + 1 } });
  };

  const completeChallenge = () => {
    if (state.gameSession && state.currentChallenge) {
      const xpEarned = calculateXp(
        state.gameSession.passedTests,
        state.gameSession.totalTests,
        Math.floor((new Date().getTime() - state.gameSession.startTime.getTime()) / 1000 / 60)
      );
      
      dispatch({ 
        type: 'COMPLETE_CHALLENGE', 
        payload: { 
          challengeId: state.currentChallenge.id, 
          xpEarned,
        } 
      });
    }
  };

  // Load user data from localStorage on mount
  useEffect(() => {
    const savedUser = localStorage.getItem('osGameUser');
    if (savedUser) {
      dispatch({ type: 'SET_USER', payload: JSON.parse(savedUser) });
    }
  }, []);

  // Save user data to localStorage when user changes
  useEffect(() => {
    if (state.user) {
      localStorage.setItem('osGameUser', JSON.stringify(state.user));
    }
  }, [state.user]);

  return (
    <GameContext.Provider value={{ 
      state, 
      dispatch, 
      startChallenge, 
      submitCode, 
      useHint, 
      completeChallenge 
    }}>
      {children}
    </GameContext.Provider>
  );
};

export const useGame = () => {
  const context = useContext(GameContext);
  if (context === undefined) {
    throw new Error('useGame must be used within a GameProvider');
  }
  return context;
};