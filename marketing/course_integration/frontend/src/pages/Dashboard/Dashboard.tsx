import React, { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { useAuth } from '../../contexts/AuthContext';
import {
  AcademicCapIcon,
  CloudIcon,
  ClipboardDocumentListIcon,
  UsersIcon,
  ChartBarIcon,
  BookOpenIcon,
  PlusIcon,
} from '@heroicons/react/24/outline';

interface DashboardStats {
  totalCourses: number;
  activeEnrollments: number;
  completedAssignments: number;
  totalStudents: number;
}

interface RecentActivity {
  id: string;
  type: 'enrollment' | 'assignment' | 'course' | 'announcement';
  title: string;
  description: string;
  timestamp: string;
  user: string;
}

const Dashboard: React.FC = () => {
  const { state: authState } = useAuth();
  const [stats, setStats] = useState<DashboardStats>({
    totalCourses: 0,
    activeEnrollments: 0,
    completedAssignments: 0,
    totalStudents: 0,
  });
  const [recentActivity, setRecentActivity] = useState<RecentActivity[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  const user = authState.user;
  const isInstructor = user?.role === 'instructor' || user?.role === 'administrator';
  const isStudent = user?.role === 'student';

  useEffect(() => {
    loadDashboardData();
  }, [user]);

  const loadDashboardData = async () => {
    try {
      setIsLoading(true);
      
      // Simulate API calls - replace with actual API calls
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      if (isInstructor) {
        setStats({
          totalCourses: 12,
          activeEnrollments: 148,
          completedAssignments: 234,
          totalStudents: 89,
        });
        
        setRecentActivity([
          {
            id: '1',
            type: 'enrollment',
            title: 'New Student Enrollment',
            description: 'John Doe enrolled in MultiOS Kernel Development',
            timestamp: '2 hours ago',
            user: 'System',
          },
          {
            id: '2',
            type: 'assignment',
            title: 'Assignment Submitted',
            description: 'Sarah Wilson submitted Assignment 3',
            timestamp: '3 hours ago',
            user: 'Sarah Wilson',
          },
          {
            id: '3',
            type: 'course',
            title: 'New Course Created',
            description: 'Advanced Driver Development course published',
            timestamp: '1 day ago',
            user: 'You',
          },
        ]);
      } else if (isStudent) {
        setStats({
          totalCourses: 6,
          activeEnrollments: 4,
          completedAssignments: 18,
          totalStudents: 0,
        });
        
        setRecentActivity([
          {
            id: '1',
            type: 'assignment',
            title: 'Assignment Due Soon',
            description: 'Kernel Programming Assignment 2 is due in 2 days',
            timestamp: '1 hour ago',
            user: 'System',
          },
          {
            id: '2',
            type: 'announcement',
            title: 'New Announcement',
            description: 'Week 5 materials are now available',
            timestamp: '4 hours ago',
            user: 'Dr. Smith',
          },
          {
            id: '3',
            type: 'enrollment',
            title: 'Course Completed',
            description: 'You have completed Operating Systems Fundamentals',
            timestamp: '2 days ago',
            user: 'System',
          },
        ]);
      }
    } catch (error) {
      console.error('Failed to load dashboard data:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const getActivityIcon = (type: string) => {
    switch (type) {
      case 'enrollment':
        return <UsersIcon className="h-5 w-5 text-blue-500" />;
      case 'assignment':
        return <ClipboardDocumentListIcon className="h-5 w-5 text-green-500" />;
      case 'course':
        return <AcademicCapIcon className="h-5 w-5 text-purple-500" />;
      case 'announcement':
        return <ChartBarIcon className="h-5 w-5 text-orange-500" />;
      default:
        return <BookOpenIcon className="h-5 w-5 text-gray-500" />;
    }
  };

  const statsCards = isInstructor ? [
    {
      title: 'Total Courses',
      value: stats.totalCourses,
      icon: AcademicCapIcon,
      color: 'text-blue-600',
      bgColor: 'bg-blue-100',
    },
    {
      title: 'Active Enrollments',
      value: stats.activeEnrollments,
      icon: UsersIcon,
      color: 'text-green-600',
      bgColor: 'bg-green-100',
    },
    {
      title: 'Completed Assignments',
      value: stats.completedAssignments,
      icon: ClipboardDocumentListIcon,
      color: 'text-purple-600',
      bgColor: 'bg-purple-100',
    },
    {
      title: 'Total Students',
      value: stats.totalStudents,
      icon: UsersIcon,
      color: 'text-indigo-600',
      bgColor: 'bg-indigo-100',
    },
  ] : [
    {
      title: 'My Courses',
      value: stats.totalCourses,
      icon: AcademicCapIcon,
      color: 'text-blue-600',
      bgColor: 'bg-blue-100',
    },
    {
      title: 'Active Courses',
      value: stats.activeEnrollments,
      icon: BookOpenIcon,
      color: 'text-green-600',
      bgColor: 'bg-green-100',
    },
    {
      title: 'Completed Assignments',
      value: stats.completedAssignments,
      icon: ClipboardDocumentListIcon,
      color: 'text-purple-600',
      bgColor: 'bg-purple-100',
    },
  ];

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Welcome section */}
      <div className="bg-white overflow-hidden shadow rounded-lg">
        <div className="px-4 py-5 sm:p-6">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-2xl font-bold text-gray-900">
                Welcome back, {user?.firstName}!
              </h1>
              <p className="mt-1 text-sm text-gray-500">
                Here's what's happening with your {isInstructor ? 'courses' : 'learning'} today.
              </p>
            </div>
            {isInstructor && (
              <div className="flex space-x-3">
                <Link
                  to="/courses/create"
                  className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                >
                  <PlusIcon className="h-4 w-4 mr-2" />
                  Create Course
                </Link>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Stats cards */}
      <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4">
        {statsCards.map((card, index) => (
          <div key={index} className="bg-white overflow-hidden shadow rounded-lg">
            <div className="p-5">
              <div className="flex items-center">
                <div className="flex-shrink-0">
                  <div className={`${card.bgColor} p-3 rounded-md`}>
                    <card.icon className={`h-6 w-6 ${card.color}`} />
                  </div>
                </div>
                <div className="ml-5 w-0 flex-1">
                  <dl>
                    <dt className="text-sm font-medium text-gray-500 truncate">
                      {card.title}
                    </dt>
                    <dd className="text-lg font-medium text-gray-900">
                      {card.value}
                    </dd>
                  </dl>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Recent Activity */}
        <div className="bg-white shadow rounded-lg">
          <div className="px-4 py-5 sm:p-6">
            <h3 className="text-lg leading-6 font-medium text-gray-900 mb-4">
              Recent Activity
            </h3>
            <div className="space-y-4">
              {recentActivity.map((activity) => (
                <div key={activity.id} className="flex space-x-3">
                  <div className="flex-shrink-0">
                    {getActivityIcon(activity.type)}
                  </div>
                  <div className="min-w-0 flex-1">
                    <p className="text-sm font-medium text-gray-900">
                      {activity.title}
                    </p>
                    <p className="text-sm text-gray-500">
                      {activity.description}
                    </p>
                    <p className="text-xs text-gray-400 mt-1">
                      {activity.timestamp} by {activity.user}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Quick Actions / LMS Status */}
        <div className="bg-white shadow rounded-lg">
          <div className="px-4 py-5 sm:p-6">
            <h3 className="text-lg leading-6 font-medium text-gray-900 mb-4">
              {isInstructor ? 'Quick Actions' : 'Learning Progress'}
            </h3>
            
            {isInstructor ? (
              <div className="grid grid-cols-2 gap-4">
                <Link
                  to="/courses"
                  className="flex items-center p-3 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors"
                >
                  <AcademicCapIcon className="h-6 w-6 text-indigo-600 mr-3" />
                  <div>
                    <p className="text-sm font-medium text-gray-900">Manage Courses</p>
                    <p className="text-xs text-gray-500">View and edit your courses</p>
                  </div>
                </Link>
                
                <Link
                  to="/lms"
                  className="flex items-center p-3 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors"
                >
                  <CloudIcon className="h-6 w-6 text-green-600 mr-3" />
                  <div>
                    <p className="text-sm font-medium text-gray-900">LMS Sync</p>
                    <p className="text-xs text-gray-500">Manage integrations</p>
                  </div>
                </Link>
                
                <Link
                  to="/assignments"
                  className="flex items-center p-3 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors"
                >
                  <ClipboardDocumentListIcon className="h-6 w-6 text-purple-600 mr-3" />
                  <div>
                    <p className="text-sm font-medium text-gray-900">Assignments</p>
                    <p className="text-xs text-gray-500">Create and grade</p>
                  </div>
                </Link>
                
                <Link
                  to="/analytics"
                  className="flex items-center p-3 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors"
                >
                  <ChartBarIcon className="h-6 w-6 text-orange-600 mr-3" />
                  <div>
                    <p className="text-sm font-medium text-gray-900">Analytics</p>
                    <p className="text-xs text-gray-500">View reports</p>
                  </div>
                </Link>
              </div>
            ) : (
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-gray-500">Overall Progress</span>
                  <span className="text-sm font-medium text-gray-900">68%</span>
                </div>
                <div className="w-full bg-gray-200 rounded-full h-2">
                  <div className="bg-indigo-600 h-2 rounded-full" style={{ width: '68%' }}></div>
                </div>
                
                <div className="space-y-2 mt-4">
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-600">Current Courses</span>
                    <span className="font-medium">4/6</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-600">Completed Assignments</span>
                    <span className="font-medium">18/25</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-600">This Week</span>
                    <span className="font-medium">3 assignments due</span>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default Dashboard;