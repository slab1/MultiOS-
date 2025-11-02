import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { Badge as BadgeIcon, Trophy, Star, Target, Zap, Award, Crown } from 'lucide-react';
import { Badge as BadgeType, Achievement, User } from '../types';

interface AchievementSystemProps {
  user: User;
  onUserUpdate: (user: User) => void;
}

const ACHIEVEMENTS: Achievement[] = [
  {
    id: 'first_challenge',
    title: 'Getting Started',
    description: 'Complete your first coding challenge',
    requirement: 1,
    progress: 0,
    completed: false,
    reward: 50,
  },
  {
    id: 'memory_master',
    title: 'Memory Master',
    description: 'Score 90+ on 5 memory management challenges',
    requirement: 5,
    progress: 0,
    completed: false,
    reward: 200,
  },
  {
    id: 'scheduling_expert',
    title: 'CPU Scheduling Expert',
    description: 'Complete 10 scheduling challenges with A+ rank',
    requirement: 10,
    progress: 0,
    completed: false,
    reward: 300,
  },
  {
    id: 'filesystem_navigator',
    title: 'File System Navigator',
    description: 'Efficiently organize 50 files across directories',
    requirement: 50,
    progress: 0,
    completed: false,
    reward: 250,
  },
  {
    id: 'speed_demon',
    title: 'Speed Demon',
    description: 'Complete 5 challenges in under 5 minutes each',
    requirement: 5,
    progress: 0,
    completed: false,
    reward: 150,
  },
  {
    id: 'perfectionist',
    title: 'Perfectionist',
    description: 'Achieve 100% score on 3 challenges',
    requirement: 3,
    progress: 0,
    completed: false,
    reward: 400,
  },
  {
    id: 'hint_hero',
    title: 'Independent Learner',
    description: 'Complete 5 challenges without using hints',
    requirement: 5,
    progress: 0,
    completed: false,
    reward: 100,
  },
  {
    id: 'streak_champion',
    title: 'Streak Champion',
    description: 'Maintain a 7-day learning streak',
    requirement: 7,
    progress: 0,
    completed: false,
    reward: 500,
  },
  {
    id: 'code_artist',
    title: 'Code Artist',
    description: 'Write well-documented code (80+ quality score) 10 times',
    requirement: 10,
    progress: 0,
    completed: false,
    reward: 200,
  },
  {
    id: 'multiplayer_winner',
    title: 'Competitive Player',
    description: 'Win 10 multiplayer challenges',
    requirement: 10,
    progress: 0,
    completed: false,
    reward: 350,
  },
];

const BADGE_DEFINITIONS = [
  {
    id: 'bronze_learner',
    name: 'Bronze Learner',
    description: 'Complete 10 challenges',
    icon: '🥉',
    rarity: 'common' as const,
  },
  {
    id: 'silver_scholar',
    name: 'Silver Scholar',
    description: 'Complete 25 challenges with 70%+ average score',
    icon: '🥈',
    rarity: 'rare' as const,
  },
  {
    id: 'gold_expert',
    name: 'Gold Expert',
    description: 'Complete 50 challenges with 85%+ average score',
    icon: '🥇',
    rarity: 'epic' as const,
  },
  {
    id: 'platinum_master',
    name: 'Platinum Master',
    description: 'Complete 100 challenges with 90%+ average score',
    icon: '💎',
    rarity: 'legendary' as const,
  },
  {
    id: 'memory_guru',
    name: 'Memory Management Guru',
    description: 'Excel at memory allocation algorithms',
    icon: '🧠',
    rarity: 'epic' as const,
  },
  {
    id: 'scheduling_genius',
    name: 'CPU Scheduling Genius',
    description: 'Master all scheduling algorithms',
    icon: '⚡',
    rarity: 'epic' as const,
  },
  {
    id: 'filesystem_architect',
    name: 'File System Architect',
    description: 'Design efficient file organization systems',
    icon: '🏗️',
    rarity: 'rare' as const,
  },
  {
    id: 'debug_wizard',
    title: 'Debug Wizard',
    description: 'Find and fix complex bugs quickly',
    icon: '🧙‍♂️',
    rarity: 'legendary' as const,
  },
  {
    id: 'optimization_hero',
    name: 'Performance Optimizer',
    description: 'Optimize code for speed and efficiency',
    icon: '🚀',
    rarity: 'rare' as const,
  },
  {
    id: 'mentor',
    name: 'Peer Mentor',
    description: 'Help other students in multiplayer mode',
    icon: '👨‍🏫',
    rarity: 'legendary' as const,
  },
];

export const AchievementSystem: React.FC<AchievementSystemProps> = ({ user, onUserUpdate }) => {
  const [selectedTab, setSelectedTab] = useState<'achievements' | 'badges'>('achievements');
  const [achievements, setAchievements] = useState<Achievement[]>(ACHIEVEMENTS);

  // Update achievement progress based on user stats
  useEffect(() => {
    const updatedAchievements = achievements.map(achievement => {
      let newProgress = achievement.progress;
      
      switch (achievement.id) {
        case 'first_challenge':
          newProgress = Math.min(achievement.requirement, user.stats.challengesCompleted);
          break;
        case 'memory_master':
          // This would check memory management scores
          newProgress = Math.min(achievement.requirement, user.stats.challengesCompleted * 0.2);
          break;
        case 'perfectionist':
          newProgress = Math.min(achievement.requirement, user.stats.challengesCompleted * 0.1);
          break;
        case 'streak_champion':
          newProgress = Math.min(achievement.requirement, user.stats.streakCount);
          break;
        case 'multiplayer_winner':
          newProgress = Math.min(achievement.requirement, user.stats.multiplayerWins);
          break;
        default:
          newProgress = achievement.progress;
      }
      
      const completed = newProgress >= achievement.requirement;
      
      return {
        ...achievement,
        progress: newProgress,
        completed,
      };
    });
    
    setAchievements(updatedAchievements);
    
    // Award new achievements
    updatedAchievements.forEach(achievement => {
      if (achievement.completed && !user.achievements.find(a => a.id === achievement.id)) {
        onUserUpdate({
          ...user,
          achievements: [...user.achievements, achievement],
          xp: user.xp + achievement.reward,
        });
      }
    });
  }, [user.stats, user.achievements, user.xp, onUserUpdate]);

  const checkAndAwardBadges = () => {
    const newBadges: BadgeType[] = [];
    
    // Check badge conditions
    if (user.stats.challengesCompleted >= 10 && !user.badges.find(b => b.id === 'bronze_learner')) {
      const badgeDef = BADGE_DEFINITIONS.find(b => b.id === 'bronze_learner');
      if (badgeDef) {
        newBadges.push({
          ...badgeDef,
          earnedAt: new Date(),
        } as BadgeType);
      }
    }
    
    if (user.stats.challengesCompleted >= 25 && user.stats.averageScore >= 70 && 
        !user.badges.find(b => b.id === 'silver_scholar')) {
      const badgeDef = BADGE_DEFINITIONS.find(b => b.id === 'silver_scholar');
      if (badgeDef) {
        newBadges.push({
          ...badgeDef,
          earnedAt: new Date(),
        } as BadgeType);
      }
    }
    
    // Award badges if any new ones were earned
    if (newBadges.length > 0) {
      onUserUpdate({
        ...user,
        badges: [...user.badges, ...newBadges],
      });
    }
  };

  useEffect(() => {
    checkAndAwardBadges();
  }, [user.stats]);

  const getAchievementIcon = (achievementId: string) => {
    switch (achievementId) {
      case 'first_challenge':
      case 'perfectionist':
        return <Star className="w-5 h-5" />;
      case 'memory_master':
      case 'scheduling_expert':
        return <Target className="w-5 h-5" />;
      case 'speed_demon':
      case 'hint_hero':
        return <Zap className="w-5 h-5" />;
      case 'streak_champion':
        return <Trophy className="w-5 h-5" />;
      case 'multiplayer_winner':
        return <Crown className="w-5 h-5" />;
      default:
        return <Award className="w-5 h-5" />;
    }
  };

  const getRarityColor = (rarity: string) => {
    switch (rarity) {
      case 'common':
        return 'border-gray-500 bg-gray-900';
      case 'rare':
        return 'border-blue-500 bg-blue-900';
      case 'epic':
        return 'border-purple-500 bg-purple-900';
      case 'legendary':
        return 'border-yellow-500 bg-yellow-900';
      default:
        return 'border-gray-500 bg-gray-900';
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <Card className="bg-slate-800 border-slate-700">
        <CardHeader>
          <div className="flex justify-between items-center">
            <div>
              <h2 className="text-2xl font-bold text-white flex items-center gap-2">
                <Trophy className="w-6 h-6" />
                Achievement System
              </h2>
              <p className="text-gray-300">Track your progress and earn rewards</p>
            </div>
            <div className="text-right">
              <div className="text-2xl font-bold text-yellow-400">{user.xp} XP</div>
              <div className="text-sm text-gray-400">Level {user.level}</div>
            </div>
          </div>
        </CardHeader>
      </Card>

      {/* Tab Navigation */}
      <div className="flex space-x-2">
        <Button
          variant={selectedTab === 'achievements' ? 'default' : 'outline'}
          onClick={() => setSelectedTab('achievements')}
          className="flex-1"
        >
          <Award className="w-4 h-4 mr-2" />
          Achievements
        </Button>
        <Button
          variant={selectedTab === 'badges' ? 'default' : 'outline'}
          onClick={() => setSelectedTab('badges')}
          className="flex-1"
        >
          <BadgeIcon className="w-4 h-4 mr-2" />
          Badges ({user.badges.length})
        </Button>
      </div>

      {/* Achievements Tab */}
      {selectedTab === 'achievements' && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {achievements.map((achievement) => (
            <Card 
              key={achievement.id} 
              className={`${
                achievement.completed 
                  ? 'bg-green-900 border-green-700' 
                  : 'bg-slate-800 border-slate-700'
              } transition-all hover:scale-105`}
            >
              <CardContent className="pt-6">
                <div className="flex items-start gap-4">
                  <div className={`p-3 rounded-full ${
                    achievement.completed 
                      ? 'bg-green-600 text-white' 
                      : 'bg-slate-700 text-gray-400'
                  }`}>
                    {getAchievementIcon(achievement.id)}
                  </div>
                  
                  <div className="flex-1">
                    <div className="flex items-center gap-2 mb-1">
                      <h3 className={`font-bold ${
                        achievement.completed ? 'text-green-100' : 'text-white'
                      }`}>
                        {achievement.title}
                      </h3>
                      {achievement.completed && (
                        <Badge variant="secondary" className="bg-green-600">
                          ✓ Complete
                        </Badge>
                      )}
                    </div>
                    
                    <p className={`text-sm mb-3 ${
                      achievement.completed ? 'text-green-200' : 'text-gray-300'
                    }`}>
                      {achievement.description}
                    </p>
                    
                    <div className="space-y-2">
                      <div className="flex justify-between text-sm">
                        <span className="text-gray-400">Progress</span>
                        <span className={achievement.completed ? 'text-green-400' : 'text-gray-300'}>
                          {Math.floor(achievement.progress)} / {achievement.requirement}
                        </span>
                      </div>
                      <Progress 
                        value={(achievement.progress / achievement.requirement) * 100} 
                        className="h-2"
                      />
                      <div className="flex justify-between text-sm">
                        <span className="text-gray-400">Reward</span>
                        <span className="text-yellow-400 font-medium">
                          {achievement.reward} XP
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {/* Badges Tab */}
      {selectedTab === 'badges' && (
        <div className="space-y-4">
          {/* Summary */}
          <Card className="bg-slate-800 border-slate-700">
            <CardContent className="pt-6">
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-center">
                <div>
                  <div className="text-2xl font-bold text-white">{user.badges.length}</div>
                  <div className="text-sm text-gray-400">Total Badges</div>
                </div>
                <div>
                  <div className="text-2xl font-bold text-yellow-400">
                    {user.badges.filter(b => b.rarity === 'legendary').length}
                  </div>
                  <div className="text-sm text-gray-400">Legendary</div>
                </div>
                <div>
                  <div className="text-2xl font-bold text-purple-400">
                    {user.badges.filter(b => b.rarity === 'epic').length}
                  </div>
                  <div className="text-sm text-gray-400">Epic</div>
                </div>
                <div>
                  <div className="text-2xl font-bold text-blue-400">
                    {user.badges.filter(b => b.rarity === 'rare').length}
                  </div>
                  <div className="text-sm text-gray-400">Rare</div>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Badge Collection */}
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
            {/* Earned Badges */}
            {user.badges.map((badge) => (
              <Card 
                key={badge.id}
                className={`${getRarityColor(badge.rarity)} border-2`}
              >
                <CardContent className="pt-6 text-center">
                  <div className="text-4xl mb-2">{badge.icon}</div>
                  <h3 className="font-bold text-white mb-1">{badge.name}</h3>
                  <p className="text-xs text-gray-300 mb-2">{badge.description}</p>
                  <Badge 
                    variant="outline" 
                    className={`text-xs ${
                      badge.rarity === 'legendary' ? 'border-yellow-500 text-yellow-400' :
                      badge.rarity === 'epic' ? 'border-purple-500 text-purple-400' :
                      badge.rarity === 'rare' ? 'border-blue-500 text-blue-400' :
                      'border-gray-500 text-gray-400'
                    }`}
                  >
                    {badge.rarity}
                  </Badge>
                  <div className="text-xs text-gray-400 mt-2">
                    Earned {badge.earnedAt.toLocaleDateString()}
                  </div>
                </CardContent>
              </Card>
            ))}

            {/* Locked Badges */}
            {BADGE_DEFINITIONS
              .filter(badgeDef => !user.badges.find(b => b.id === badgeDef.id))
              .map((badgeDef) => (
                <Card 
                  key={badgeDef.id}
                  className="bg-slate-800 border-slate-700 opacity-50"
                >
                  <CardContent className="pt-6 text-center">
                    <div className="text-4xl mb-2 grayscale">🔒</div>
                    <h3 className="font-bold text-gray-500 mb-1">{badgeDef.name}</h3>
                    <p className="text-xs text-gray-600 mb-2">{badgeDef.description}</p>
                    <Badge 
                      variant="outline" 
                      className="text-xs border-gray-600 text-gray-500"
                    >
                      {badgeDef.rarity}
                    </Badge>
                    <div className="text-xs text-gray-600 mt-2">
                      Locked
                    </div>
                  </CardContent>
                </Card>
              ))}
          </div>
        </div>
      )}

      {/* Progress Overview */}
      <Card className="bg-slate-800 border-slate-700">
        <CardHeader>
          <CardTitle className="text-white">Overall Progress</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="text-center">
              <div className="text-3xl font-bold text-blue-400 mb-2">
                {Math.round((achievements.filter(a => a.completed).length / achievements.length) * 100)}%
              </div>
              <div className="text-gray-300">Achievements Complete</div>
              <div className="text-sm text-gray-400">
                {achievements.filter(a => a.completed).length} of {achievements.length}
              </div>
            </div>
            
            <div className="text-center">
              <div className="text-3xl font-bold text-green-400 mb-2">
                {user.badges.length}
              </div>
              <div className="text-gray-300">Badges Earned</div>
              <div className="text-sm text-gray-400">
                of {BADGE_DEFINITIONS.length} available
              </div>
            </div>
            
            <div className="text-center">
              <div className="text-3xl font-bold text-yellow-400 mb-2">
                {user.xp}
              </div>
              <div className="text-gray-300">Total XP</div>
              <div className="text-sm text-gray-400">
                Level {user.level}
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};