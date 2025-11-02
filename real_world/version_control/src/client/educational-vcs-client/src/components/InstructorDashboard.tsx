import React, { useState, useEffect } from 'react';
import { 
  AcademicCapIcon,
  UsersIcon,
  ClipboardDocumentListIcon,
  ChartBarIcon,
  PlusIcon,
  EyeIcon
} from '@heroicons/react/24/outline';

const InstructorDashboard = () => {
  const [stats, setStats] = useState({
    totalStudents: 45,
    activeRepositories: 32,
    pendingReviews: 8,
    completedAssignments: 156
  });

  const [recentActivity] = useState([
    {
      id: 1,
      type: 'submission',
      student: 'John Doe',
      assignment: 'Git Basics Exercise',
      timestamp: '2024-12-04 10:30:00',
      status: 'graded'
    },
    {
      id: 2,
      type: 'review',
      student: 'Jane Smith',
      assignment: 'Collaborative Project',
      timestamp: '2024-12-04 09:15:00',
      status: 'pending'
    },
    {
      id: 3,
      type: 'repository',
      student: 'Bob Johnson',
      action: 'Created new repository',
      timestamp: '2024-12-04 08:45:00',
      status: 'created'
    }
  ]);

  const [assignments] = useState([
    {
      id: 1,
      title: 'Git Basics Exercise',
      course: 'CS 201',
      submissions: 38,
      totalStudents: 45,
      averageGrade: 85.2,
      dueDate: '2024-12-15'
    },
    {
      id: 2,
      title: 'Collaborative Project',
      course: 'CS 301',
      submissions: 25,
      totalStudents: 30,
      averageGrade: 78.9,
      dueDate: '2024-12-20'
    }
  ]);

  const getActivityIcon = (type) => {
    switch (type) {
      case 'submission': return <ClipboardDocumentListIcon className="w-5 h-5" />;
      case 'review': return <EyeIcon className="w-5 h-5" />;
      case 'repository': return <ChartBarIcon className="w-5 h-5" />;
      default: return <UsersIcon className="w-5 h-5" />;
    }
  };

  const getStatusColor = (status) => {
    switch (status) {
      case 'graded': return 'text-green-600 bg-green-50';
      case 'pending': return 'text-yellow-600 bg-yellow-50';
      case 'created': return 'text-blue-600 bg-blue-50';
      default: return 'text-gray-600 bg-gray-50';
    }
  };

  return (
    <div className="p-6 overflow-y-auto h-full">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-gray-900">Instructor Dashboard</h1>
        <p className="text-gray-600 mt-2">
          Monitor student progress and manage your courses
        </p>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Total Students</p>
              <p className="text-2xl font-bold text-gray-900">{stats.totalStudents}</p>
            </div>
            <UsersIcon className="w-8 h-8 text-blue-500" />
          </div>
        </div>

        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Active Repos</p>
              <p className="text-2xl font-bold text-gray-900">{stats.activeRepositories}</p>
            </div>
            <ChartBarIcon className="w-8 h-8 text-green-500" />
          </div>
        </div>

        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Pending Reviews</p>
              <p className="text-2xl font-bold text-gray-900">{stats.pendingReviews}</p>
            </div>
            <ClipboardDocumentListIcon className="w-8 h-8 text-yellow-500" />
          </div>
        </div>

        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Completed</p>
              <p className="text-2xl font-bold text-gray-900">{stats.completedAssignments}</p>
            </div>
            <AcademicCapIcon className="w-8 h-8 text-purple-500" />
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        {/* Recent Activity */}
        <div className="bg-white rounded-lg shadow-sm border border-gray-200">
          <div className="p-6 border-b border-gray-200">
            <h2 className="text-xl font-semibold text-gray-900">Recent Activity</h2>
          </div>
          <div className="p-6">
            <div className="space-y-4">
              {recentActivity.map((activity) => (
                <div key={activity.id} className="flex items-center space-x-4 p-4 bg-gray-50 rounded-lg">
                  <div className="text-gray-500">
                    {getActivityIcon(activity.type)}
                  </div>
                  <div className="flex-1">
                    <div className="flex items-center justify-between">
                      <p className="font-medium text-gray-900">
                        {activity.student} {activity.action}
                      </p>
                      <span className={`px-2 py-1 text-xs rounded-full ${getStatusColor(activity.status)}`}>
                        {activity.status}
                      </span>
                    </div>
                    <p className="text-sm text-gray-600">{activity.assignment}</p>
                    <p className="text-xs text-gray-500 mt-1">{activity.timestamp}</p>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Assignment Overview */}
        <div className="bg-white rounded-lg shadow-sm border border-gray-200">
          <div className="p-6 border-b border-gray-200">
            <div className="flex items-center justify-between">
              <h2 className="text-xl font-semibold text-gray-900">Assignments</h2>
              <button className="flex items-center space-x-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors">
                <PlusIcon className="w-4 h-4" />
                <span>New Assignment</span>
              </button>
            </div>
          </div>
          <div className="p-6">
            <div className="space-y-4">
              {assignments.map((assignment) => (
                <div key={assignment.id} className="border border-gray-200 rounded-lg p-4">
                  <div className="flex items-center justify-between mb-2">
                    <h3 className="font-medium text-gray-900">{assignment.title}</h3>
                    <span className="text-sm text-gray-500">{assignment.course}</span>
                  </div>
                  
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <span className="text-gray-600">Submissions:</span>
                      <span className="ml-2 font-medium">
                        {assignment.submissions}/{assignment.totalStudents}
                      </span>
                    </div>
                    <div>
                      <span className="text-gray-600">Average Grade:</span>
                      <span className="ml-2 font-medium">{assignment.averageGrade}%</span>
                    </div>
                  </div>
                  
                  <div className="mt-3 flex items-center justify-between">
                    <span className="text-xs text-gray-500">
                      Due: {new Date(assignment.dueDate).toLocaleDateString()}
                    </span>
                    <button className="text-sm text-blue-600 hover:text-blue-800">
                      View Details
                    </button>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* Quick Actions */}
      <div className="mt-8">
        <div className="bg-white rounded-lg shadow-sm border border-gray-200">
          <div className="p-6 border-b border-gray-200">
            <h2 className="text-xl font-semibold text-gray-900">Quick Actions</h2>
          </div>
          <div className="p-6">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <button className="p-4 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors text-left">
                <AcademicCapIcon className="w-8 h-8 text-blue-500 mb-2" />
                <h3 className="font-medium text-gray-900">Create Course</h3>
                <p className="text-sm text-gray-600">Set up a new course with assignments</p>
              </button>
              
              <button className="p-4 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors text-left">
                <UsersIcon className="w-8 h-8 text-green-500 mb-2" />
                <h3 className="font-medium text-gray-900">Manage Students</h3>
                <p className="text-sm text-gray-600">Add or remove students from courses</p>
              </button>
              
              <button className="p-4 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors text-left">
                <ChartBarIcon className="w-8 h-8 text-purple-500 mb-2" />
                <h3 className="font-medium text-gray-900">View Analytics</h3>
                <p className="text-sm text-gray-600">Analyze student performance data</p>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default InstructorDashboard;
