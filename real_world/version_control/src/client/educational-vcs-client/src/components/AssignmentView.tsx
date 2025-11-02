import React, { useState } from 'react';
import { 
  ClipboardDocumentListIcon,
  PlusIcon,
  ClockIcon,
  CheckCircleIcon,
  ExclamationCircleIcon
} from '@heroicons/react/24/outline';

const AssignmentView = () => {
  const [assignments] = useState([
    {
      id: 1,
      title: 'Git Basics Exercise',
      description: 'Practice creating commits, branches, and basic Git operations',
      dueDate: '2024-12-15',
      status: 'assigned',
      course: 'CS 201',
      maxPoints: 100
    },
    {
      id: 2,
      title: 'Collaborative Project',
      description: 'Work in teams to build a small application using version control',
      dueDate: '2024-12-20',
      status: 'in_progress',
      course: 'CS 301',
      maxPoints: 200
    },
    {
      id: 3,
      title: 'Code Review Assignment',
      description: 'Practice peer code review and provide constructive feedback',
      dueDate: '2024-12-18',
      status: 'submitted',
      course: 'CS 201',
      maxPoints: 50
    }
  ]);

  const getStatusIcon = (status) => {
    switch (status) {
      case 'assigned': return <ClockIcon className="w-5 h-5 text-blue-500" />;
      case 'in_progress': return <ExclamationCircleIcon className="w-5 h-5 text-yellow-500" />;
      case 'submitted': return <CheckCircleIcon className="w-5 h-5 text-green-500" />;
      default: return <ClockIcon className="w-5 h-5 text-gray-500" />;
    }
  };

  const getStatusColor = (status) => {
    switch (status) {
      case 'assigned': return 'bg-blue-50 text-blue-800 border-blue-200';
      case 'in_progress': return 'bg-yellow-50 text-yellow-800 border-yellow-200';
      case 'submitted': return 'bg-green-50 text-green-800 border-green-200';
      default: return 'bg-gray-50 text-gray-800 border-gray-200';
    }
  };

  return (
    <div className="p-6 overflow-y-auto h-full">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-gray-900">Assignments</h1>
        <p className="text-gray-600 mt-2">
          View and manage your course assignments
        </p>
      </div>

      <div className="space-y-6">
        {assignments.map((assignment) => (
          <div
            key={assignment.id}
            className="bg-white border border-gray-200 rounded-lg p-6 hover:shadow-sm transition-shadow"
          >
            <div className="flex items-start justify-between mb-4">
              <div className="flex items-start space-x-3">
                {getStatusIcon(assignment.status)}
                <div className="flex-1">
                  <h3 className="text-lg font-semibold text-gray-900">
                    {assignment.title}
                  </h3>
                  <p className="text-gray-600 mt-1">{assignment.description}</p>
                  <div className="flex items-center space-x-4 mt-2 text-sm text-gray-500">
                    <span>{assignment.course}</span>
                    <span>Due: {new Date(assignment.dueDate).toLocaleDateString()}</span>
                    <span>{assignment.maxPoints} points</span>
                  </div>
                </div>
              </div>
              
              <div className="flex items-center space-x-3">
                <span className={`px-3 py-1 text-xs font-medium rounded-full border ${getStatusColor(assignment.status)}`}>
                  {assignment.status.replace('_', ' ')}
                </span>
                
                <button className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors text-sm">
                  {assignment.status === 'assigned' ? 'Start' : 
                   assignment.status === 'in_progress' ? 'Continue' : 'View'}
                </button>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default AssignmentView;
