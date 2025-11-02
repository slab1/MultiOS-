import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { 
  ClipboardDocumentListIcon,
  CodeBracketIcon,
  TrophyIcon,
  ClockIcon,
  CheckCircleIcon,
  ExclamationCircleIcon,
  ChartBarIcon
} from '@heroicons/react/24/outline';
import { useVCS } from '../context/VCSContext';

const StudentDashboard = () => {
  const { currentUser, API_URL } = useVCS();
  const [dashboardData, setDashboardData] = useState(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (currentUser) {
      loadDashboardData();
    }
  }, [currentUser]);

  const loadDashboardData = async () => {
    try {
      // Simulate API call with mock data for demo
      const mockData = {
        student: currentUser,
        progress: {
          totalAssignments: 12,
          completedAssignments: 8,
          averageGrade: 85.5,
          streak: 5,
          skillsLearned: ['Git Basics', 'Branching', 'Merging', 'Code Review']
        },
        upcomingAssignments: [
          {
            id: 1,
            title: 'Advanced Git Workflows',
            dueDate: '2024-12-15',
            course: 'CS 201',
            status: 'pending'
          },
          {
            id: 2,
            title: 'Collaborative Project',
            dueDate: '2024-12-18',
            course: 'CS 301',
            status: 'in_progress'
          },
          {
            id: 3,
            title: 'Code Review Exercise',
            dueDate: '2024-12-20',
            course: 'CS 201',
            status: 'assigned'
          }
        ],
        recentSubmissions: [
          {
            id: 1,
            assignment: 'Git Basics Quiz',
            status: 'graded',
            grade: 95,
            submittedAt: '2024-12-01'
          },
          {
            id: 2,
            assignment: 'Branching Exercise',
            status: 'graded',
            grade: 88,
            submittedAt: '2024-11-28'
          },
          {
            id: 3,
            assignment: 'Merge Challenge',
            status: 'pending',
            grade: null,
            submittedAt: '2024-11-30'
          }
        ],
        achievements: [
          {
            id: 1,
            title: 'First Commit',
            description: 'Made your first commit',
            icon: '🎉',
            earnedAt: '2024-11-15'
          },
          {
            id: 2,
            title: 'Branch Master',
            description: 'Created 5 branches',
            icon: '🌳',
            earnedAt: '2024-11-20'
          },
          {
            id: 3,
            title: 'Code Reviewer',
            description: 'Completed 10 code reviews',
            icon: '👀',
            earnedAt: '2024-11-25'
          }
        ]
      };

      setDashboardData(mockData);
      setLoading(false);
    } catch (error) {
      console.error('Failed to load dashboard:', error);
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  if (!dashboardData) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <p className="text-gray-500">Failed to load dashboard data</p>
        </div>
      </div>
    );
  }

  const { progress, upcomingAssignments, recentSubmissions, achievements } = dashboardData;

  return (
    <div className="p-6 overflow-y-auto h-full">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-gray-900">
          Welcome back, {currentUser?.name}! 👋
        </h1>
        <p className="text-gray-600 mt-2">
          Here's your learning progress and upcoming assignments
        </p>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Completed</p>
              <p className="text-2xl font-bold text-gray-900">
                {progress.completedAssignments}/{progress.totalAssignments}
              </p>
            </div>
            <CheckCircleIcon className="w-8 h-8 text-green-500" />
          </div>
          <div className="mt-4">
            <div className="w-full bg-gray-200 rounded-full h-2">
              <div 
                className="bg-green-500 h-2 rounded-full"
                style={{ width: `${(progress.completedAssignments / progress.totalAssignments) * 100}%` }}
              ></div>
            </div>
          </div>
        </div>

        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Average Grade</p>
              <p className="text-2xl font-bold text-gray-900">{progress.averageGrade}%</p>
            </div>
            <TrophyIcon className="w-8 h-8 text-yellow-500" />
          </div>
          <div className="mt-2">
            <span className="text-sm text-green-600 font-medium">
              ▲ 3% from last week
            </span>
          </div>
        </div>

        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Learning Streak</p>
              <p className="text-2xl font-bold text-gray-900">{progress.streak} days</p>
            </div>
            <ClockIcon className="w-8 h-8 text-blue-500" />
          </div>
          <div className="mt-2">
            <span className="text-sm text-blue-600 font-medium">
              Keep it up!
            </span>
          </div>
        </div>

        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Skills Learned</p>
              <p className="text-2xl font-bold text-gray-900">{progress.skillsLearned.length}</p>
            </div>
            <ChartBarIcon className="w-8 h-8 text-purple-500" />
          </div>
          <div className="mt-2">
            <span className="text-sm text-purple-600 font-medium">
              +2 this week
            </span>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        {/* Upcoming Assignments */}
        <div className="bg-white rounded-lg shadow-sm border border-gray-200">
          <div className="p-6 border-b border-gray-200">
            <h2 className="text-xl font-semibold text-gray-900 flex items-center">
              <ClipboardDocumentListIcon className="w-6 h-6 mr-2" />
              Upcoming Assignments
            </h2>
          </div>
          <div className="p-6">
            <div className="space-y-4">
              {upcomingAssignments.map((assignment) => (
                <div key={assignment.id} className="flex items-center justify-between p-4 bg-gray-50 rounded-lg">
                  <div className="flex-1">
                    <h3 className="font-medium text-gray-900">{assignment.title}</h3>
                    <p className="text-sm text-gray-600">{assignment.course}</p>
                    <p className="text-xs text-gray-500 mt-1">
                      Due: {new Date(assignment.dueDate).toLocaleDateString()}
                    </p>
                  </div>
                  <div className="flex items-center space-x-2">
                    {assignment.status === 'pending' && (
                      <span className="px-2 py-1 bg-yellow-100 text-yellow-800 text-xs rounded-full">
                        Pending
                      </span>
                    )}
                    {assignment.status === 'in_progress' && (
                      <span className="px-2 py-1 bg-blue-100 text-blue-800 text-xs rounded-full">
                        In Progress
                      </span>
                    )}
                    {assignment.status === 'assigned' && (
                      <span className="px-2 py-1 bg-green-100 text-green-800 text-xs rounded-full">
                        Assigned
                      </span>
                    )}
                  </div>
                </div>
              ))}
            </div>
            <div className="mt-4">
              <Link 
                to="/assignments"
                className="text-blue-600 hover:text-blue-800 text-sm font-medium"
              >
                View all assignments →
              </Link>
            </div>
          </div>
        </div>

        {/* Recent Submissions */}
        <div className="bg-white rounded-lg shadow-sm border border-gray-200">
          <div className="p-6 border-b border-gray-200">
            <h2 className="text-xl font-semibold text-gray-900 flex items-center">
              <CodeBracketIcon className="w-6 h-6 mr-2" />
              Recent Submissions
            </h2>
          </div>
          <div className="p-6">
            <div className="space-y-4">
              {recentSubmissions.map((submission) => (
                <div key={submission.id} className="flex items-center justify-between p-4 bg-gray-50 rounded-lg">
                  <div className="flex-1">
                    <h3 className="font-medium text-gray-900">{submission.assignment}</h3>
                    <p className="text-xs text-gray-500 mt-1">
                      Submitted: {new Date(submission.submittedAt).toLocaleDateString()}
                    </p>
                  </div>
                  <div className="flex items-center space-x-2">
                    {submission.status === 'graded' ? (
                      <div className="text-right">
                        <p className="text-sm font-medium text-gray-900">{submission.grade}%</p>
                        <CheckCircleIcon className="w-4 h-4 text-green-500 ml-auto" />
                      </div>
                    ) : (
                      <div className="text-right">
                        <ExclamationCircleIcon className="w-4 h-4 text-yellow-500 ml-auto" />
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
            <div className="mt-4">
              <Link 
                to="/repository"
                className="text-blue-600 hover:text-blue-800 text-sm font-medium"
              >
                View repository →
              </Link>
            </div>
          </div>
        </div>
      </div>

      {/* Achievements */}
      <div className="mt-8">
        <div className="bg-white rounded-lg shadow-sm border border-gray-200">
          <div className="p-6 border-b border-gray-200">
            <h2 className="text-xl font-semibold text-gray-900 flex items-center">
              <TrophyIcon className="w-6 h-6 mr-2" />
              Your Achievements
            </h2>
          </div>
          <div className="p-6">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              {achievements.map((achievement) => (
                <div key={achievement.id} className="p-4 border border-gray-200 rounded-lg">
                  <div className="text-2xl mb-2">{achievement.icon}</div>
                  <h3 className="font-medium text-gray-900">{achievement.title}</h3>
                  <p className="text-sm text-gray-600 mt-1">{achievement.description}</p>
                  <p className="text-xs text-gray-500 mt-2">
                    Earned: {new Date(achievement.earnedAt).toLocaleDateString()}
                  </p>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* Skills Progress */}
      <div className="mt-8">
        <div className="bg-white rounded-lg shadow-sm border border-gray-200">
          <div className="p-6 border-b border-gray-200">
            <h2 className="text-xl font-semibold text-gray-900">Skills Progress</h2>
          </div>
          <div className="p-6">
            <div className="space-y-4">
              {progress.skillsLearned.map((skill, index) => (
                <div key={index} className="flex items-center justify-between">
                  <span className="text-gray-900 font-medium">{skill}</span>
                  <div className="flex items-center space-x-2">
                    <CheckCircleIcon className="w-5 h-5 text-green-500" />
                    <span className="text-sm text-green-600 font-medium">Completed</span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default StudentDashboard;
