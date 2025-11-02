import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Trophy, Medal, Award, Crown, TrendingUp, Clock, Target } from 'lucide-react';
import { LeaderboardEntry, User } from '../types';

interface LeaderboardProps {
  currentUser: User;
  category: 'global' | 'weekly' | 'daily' | 'category' | 'multiplayer';
  categoryFilter?: string;
}

interface LeaderboardData extends Omit<LeaderboardEntry, 'category'> {
  rank: number;
  level: number;
  badgeCount: number;
  completionRate: number;
  averageScore: number;
  category: string;
}

const mockLeaderboardData: LeaderboardData[] = [
  {
    userId: '1',
    username: 'OSMaster2024',
    score: 15234,
    category: 'all',
    timestamp: new Date(),
    rank: 1,
    level: 15,
    badgeCount: 23,
    completionRate: 95.5,
    averageScore: 87.3,
  },
  {
    userId: '2',
    username: 'MemoryGuru',
    score: 14890,
    category: 'memory',
    timestamp: new Date(),
    rank: 2,
    level: 14,
    badgeCount: 20,
    completionRate: 92.1,
    averageScore: 89.1,
  },
  {
    userId: '3',
    username: 'SchedulingPro',
    score: 14567,
    category: 'scheduling',
    timestamp: new Date(),
    rank: 3,
    level: 14,
    badgeCount: 19,
    completionRate: 88.7,
    averageScore: 86.5,
  },
  {
    userId: '4',
    username: 'CodeNinja',
    score: 14234,
    category: 'all',
    timestamp: new Date(),
    rank: 4,
    level: 13,
    badgeCount: 18,
    completionRate: 91.2,
    averageScore: 84.8,
  },
  {
    userId: '5',
    username: 'FileSystemExpert',
    score: 13956,
    category: 'filesystem',
    timestamp: new Date(),
    rank: 5,
    level: 13,
    badgeCount: 17,
    completionRate: 89.4,
    averageScore: 85.2,
  },
  {
    userId: '6',
    username: 'DebugWizard',
    score: 13678,
    category: 'all',
    timestamp: new Date(),
    rank: 6,
    level: 12,
    badgeCount: 16,
    completionRate: 87.8,
    averageScore: 83.9,
  },
  {
    userId: '7',
    username: 'PerformanceGuru',
    score: 13412,
    category: 'optimization',
    timestamp: new Date(),
    rank: 7,
    level: 12,
    badgeCount: 15,
    completionRate: 86.3,
    averageScore: 82.7,
  },
  {
    userId: '8',
    username: 'OSStudent',
    score: 13145,
    category: 'all',
    timestamp: new Date(),
    rank: 8,
    level: 11,
    badgeCount: 14,
    completionRate: 84.9,
    averageScore: 81.4,
  },
  {
    userId: '9',
    username: 'MemoryLearner',
    score: 12890,
    category: 'memory',
    timestamp: new Date(),
    rank: 9,
    level: 11,
    badgeCount: 13,
    completionRate: 83.2,
    averageScore: 80.8,
  },
  {
    userId: '10',
    username: 'SchedulerBeginner',
    score: 12634,
    category: 'scheduling',
    timestamp: new Date(),
    rank: 10,
    level: 10,
    badgeCount: 12,
    completionRate: 81.7,
    averageScore: 79.3,
  },
];

export const Leaderboard: React.FC<LeaderboardProps> = ({ 
  currentUser, 
  category, 
  categoryFilter 
}) => {
  const [leaderboardData, setLeaderboardData] = useState<LeaderboardData[]>([]);
  const [timeFilter, setTimeFilter] = useState<'all' | 'week' | 'month'>('all');
  const [userRank, setUserRank] = useState<number | null>(null);

  useEffect(() => {
    // In a real application, this would fetch data from an API
    // For demo purposes, we'll use mock data
    let filteredData = [...mockLeaderboardData];

    // Filter by category
    if (categoryFilter && categoryFilter !== 'all') {
      filteredData = filteredData.filter(entry => 
        entry.category === categoryFilter || entry.category === 'all'
      );
    }

    // Sort by score (highest first)
    filteredData.sort((a, b) => b.score - a.score);

    // Add rank to each entry
    filteredData = filteredData.map((entry, index) => ({
      ...entry,
      rank: index + 1,
    }));

    // Add current user to leaderboard if not present
    const currentUserInLeaderboard = filteredData.find(entry => entry.userId === currentUser.id);
    if (!currentUserInLeaderboard) {
      const currentUserEntry: LeaderboardData = {
        userId: currentUser.id,
        username: currentUser.username,
        score: currentUser.xp,
        category: 'all',
        timestamp: new Date(),
        rank: 0, // Will be calculated
        level: currentUser.level,
        badgeCount: currentUser.badges.length,
        completionRate: currentUser.stats.averageScore,
        averageScore: currentUser.stats.averageScore,
      };
      
      // Insert current user at appropriate position
      filteredData.push(currentUserEntry);
      filteredData.sort((a, b) => b.score - a.score);
      filteredData = filteredData.map((entry, index) => ({
        ...entry,
        rank: index + 1,
      }));
    }

    setLeaderboardData(filteredData);

    // Find current user's rank
    const userRank = filteredData.find(entry => entry.userId === currentUser.id)?.rank;
    setUserRank(userRank || null);
  }, [currentUser, categoryFilter]);

  const getRankIcon = (rank: number) => {
    switch (rank) {
      case 1:
        return <Crown className="w-6 h-6 text-yellow-400" />;
      case 2:
        return <Medal className="w-6 h-6 text-gray-300" />;
      case 3:
        return <Medal className="w-6 h-6 text-amber-600" />;
      default:
        return <Award className="w-5 h-5 text-gray-500" />;
    }
  };

  const getRankBadgeColor = (rank: number) => {
    if (rank <= 3) return 'bg-gradient-to-r from-yellow-400 to-yellow-600';
    if (rank <= 10) return 'bg-gradient-to-r from-blue-500 to-blue-700';
    if (rank <= 25) return 'bg-gradient-to-r from-green-500 to-green-700';
    return 'bg-gradient-to-r from-gray-500 to-gray-700';
  };

  const formatScore = (score: number) => {
    if (score >= 1000000) return `${(score / 1000000).toFixed(1)}M`;
    if (score >= 1000) return `${(score / 1000).toFixed(1)}K`;
    return score.toString();
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
                Leaderboard
              </h2>
              <p className="text-gray-300">
                {category === 'global' && 'Global Rankings'}
                {category === 'weekly' && 'Weekly Rankings'}
                {category === 'daily' && 'Daily Rankings'}
                {category === 'category' && `${categoryFilter?.charAt(0).toUpperCase() + categoryFilter?.slice(1)} Rankings`}
                {category === 'multiplayer' && 'Multiplayer Rankings'}
              </p>
            </div>
            <div className="text-right">
              {userRank && (
                <div className="text-center">
                  <div className="text-lg font-bold text-white">Your Rank</div>
                  <div className={`text-2xl font-bold ${userRank <= 3 ? 'text-yellow-400' : 'text-blue-400'}`}>
                    #{userRank}
                  </div>
                </div>
              )}
            </div>
          </div>
        </CardHeader>
      </Card>

      {/* Time Filter Tabs */}
      {category !== 'category' && (
        <div className="flex space-x-2">
          <Button
            variant={timeFilter === 'all' ? 'default' : 'outline'}
            size="sm"
            onClick={() => setTimeFilter('all')}
          >
            All Time
          </Button>
          <Button
            variant={timeFilter === 'month' ? 'default' : 'outline'}
            size="sm"
            onClick={() => setTimeFilter('month')}
          >
            This Month
          </Button>
          <Button
            variant={timeFilter === 'week' ? 'default' : 'outline'}
            size="sm"
            onClick={() => setTimeFilter('week')}
          >
            This Week
          </Button>
        </div>
      )}

      {/* Leaderboard */}
      <Card className="bg-slate-800 border-slate-700">
        <CardHeader>
          <CardTitle className="text-white">Top Performers</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            {leaderboardData.slice(0, 20).map((entry) => (
              <div
                key={entry.userId}
                className={`p-4 rounded-lg transition-all hover:scale-105 ${
                  entry.userId === currentUser.id
                    ? 'bg-blue-900 border-2 border-blue-500'
                    : 'bg-slate-700 hover:bg-slate-600'
                }`}
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-4">
                    {/* Rank */}
                    <div className="flex items-center justify-center w-12 h-12 rounded-full bg-slate-600">
                      {entry.rank <= 3 ? (
                        getRankIcon(entry.rank)
                      ) : (
                        <span className="text-lg font-bold text-white">#{entry.rank}</span>
                      )}
                    </div>

                    {/* User Info */}
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        <h3 className={`font-bold ${
                          entry.userId === currentUser.id ? 'text-blue-300' : 'text-white'
                        }`}>
                          {entry.username}
                          {entry.userId === currentUser.id && (
                            <span className="text-sm text-blue-300 ml-2">(You)</span>
                          )}
                        </h3>
                        {entry.rank <= 3 && (
                          <Badge className={getRankBadgeColor(entry.rank)}>
                            Top {entry.rank}
                          </Badge>
                        )}
                      </div>
                      
                      <div className="flex items-center gap-4 text-sm text-gray-300 mt-1">
                        <span className="flex items-center gap-1">
                          <Target className="w-3 h-3" />
                          Level {entry.level}
                        </span>
                        <span className="flex items-center gap-1">
                          <Award className="w-3 h-3" />
                          {entry.badgeCount} badges
                        </span>
                        <span className="flex items-center gap-1">
                          <TrendingUp className="w-3 h-3" />
                          {entry.averageScore.toFixed(1)}% avg
                        </span>
                      </div>
                    </div>
                  </div>

                  {/* Score */}
                  <div className="text-right">
                    <div className="text-2xl font-bold text-yellow-400">
                      {formatScore(entry.score)}
                    </div>
                    <div className="text-sm text-gray-400">XP</div>
                  </div>
                </div>

                {/* Additional Stats */}
                {entry.rank <= 10 && (
                  <div className="mt-3 pt-3 border-t border-slate-600">
                    <div className="grid grid-cols-3 gap-4 text-sm">
                      <div className="text-center">
                        <div className="text-white font-medium">{entry.completionRate.toFixed(1)}%</div>
                        <div className="text-gray-400">Completion</div>
                      </div>
                      <div className="text-center">
                        <div className="text-white font-medium">{entry.badgeCount}</div>
                        <div className="text-gray-400">Badges</div>
                      </div>
                      <div className="text-center">
                        <div className="text-white font-medium">{entry.averageScore.toFixed(1)}</div>
                        <div className="text-gray-400">Avg Score</div>
                      </div>
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Your Stats Summary */}
      {userRank && userRank > 20 && (
        <Card className="bg-gradient-to-r from-blue-900 to-purple-900 border-0">
          <CardContent className="pt-6">
            <div className="text-center">
              <h3 className="text-xl font-bold text-white mb-4">Your Position</h3>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div>
                  <div className="text-2xl font-bold text-white">#{userRank}</div>
                  <div className="text-blue-200">Current Rank</div>
                </div>
                <div>
                  <div className="text-2xl font-bold text-white">{currentUser.xp}</div>
                  <div className="text-blue-200">Total XP</div>
                </div>
                <div>
                  <div className="text-2xl font-bold text-white">{currentUser.level}</div>
                  <div className="text-blue-200">Level</div>
                </div>
                <div>
                  <div className="text-2xl font-bold text-white">{currentUser.badges.length}</div>
                  <div className="text-blue-200">Badges</div>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Ranking Categories */}
      <Card className="bg-slate-800 border-slate-700">
        <CardHeader>
          <CardTitle className="text-white">Ranking Categories</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="text-center p-4 bg-slate-700 rounded">
              <Trophy className="w-8 h-8 mx-auto text-yellow-400 mb-2" />
              <div className="font-bold text-white">Overall</div>
              <div className="text-sm text-gray-400">All challenges</div>
            </div>
            <div className="text-center p-4 bg-slate-700 rounded">
              <Target className="w-8 h-8 mx-auto text-blue-400 mb-2" />
              <div className="font-bold text-white">Memory</div>
              <div className="text-sm text-gray-400">Memory management</div>
            </div>
            <div className="text-center p-4 bg-slate-700 rounded">
              <Clock className="w-8 h-8 mx-auto text-green-400 mb-2" />
              <div className="font-bold text-white">Scheduling</div>
              <div className="text-sm text-gray-400">CPU scheduling</div>
            </div>
            <div className="text-center p-4 bg-slate-700 rounded">
              <Award className="w-8 h-8 mx-auto text-purple-400 mb-2" />
              <div className="font-bold text-white">File System</div>
              <div className="text-sm text-gray-400">File organization</div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};