import React, { useState } from 'react';
import { useParams } from 'react-router-dom';
import { 
  CodeBracketIcon,
  ChatBubbleLeftIcon,
  CheckCircleIcon,
  ExclamationTriangleIcon,
  ClockIcon,
  UserIcon
} from '@heroicons/react/24/outline';

const CodeReview = () => {
  const { reviewId } = useParams();
  const [reviews] = useState([
    {
      id: '1',
      title: 'Add user authentication feature',
      author: 'Alice Smith',
      branch: 'feature/auth',
      status: 'pending',
      reviewers: ['Bob Johnson', 'Carol Davis'],
      comments: [
        {
          id: 1,
          author: 'Bob Johnson',
          content: 'Great implementation! Consider adding error handling for failed login attempts.',
          type: 'suggestion',
          line: 45,
          timestamp: '2024-12-04 10:30:00'
        },
        {
          id: 2,
          author: 'Carol Davis',
          content: 'The code structure looks good. Maybe add unit tests for the auth functions?',
          type: 'question',
          line: 32,
          timestamp: '2024-12-04 11:15:00'
        }
      ]
    }
  ]);

  const currentReview = reviews[0]; // For demo

  const getStatusColor = (status) => {
    switch (status) {
      case 'pending': return 'bg-yellow-50 text-yellow-800';
      case 'approved': return 'bg-green-50 text-green-800';
      case 'changes_requested': return 'bg-red-50 text-red-800';
      default: return 'bg-gray-50 text-gray-800';
    }
  };

  const getCommentIcon = (type) => {
    switch (type) {
      case 'suggestion': return <ExclamationTriangleIcon className="w-4 h-4" />;
      case 'issue': return <ExclamationTriangleIcon className="w-4 h-4" />;
      case 'question': return <ChatBubbleLeftIcon className="w-4 h-4" />;
      case 'praise': return <CheckCircleIcon className="w-4 h-4" />;
      default: return <ChatBubbleLeftIcon className="w-4 h-4" />;
    }
  };

  return (
    <div className="h-full flex flex-col">
      <div className="bg-white border-b border-gray-200 p-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-gray-900">Code Review</h1>
            <p className="text-gray-600 mt-1">Review and provide feedback on code changes</p>
          </div>
          
          <div className="flex space-x-3">
            <button className="px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors">
              View Changes
            </button>
            <button className="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors">
              Approve
            </button>
            <button className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors">
              Request Changes
            </button>
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        {currentReview && (
          <div className="max-w-4xl mx-auto space-y-6">
            {/* Review Header */}
            <div className="bg-white border border-gray-200 rounded-lg p-6">
              <div className="flex items-start justify-between mb-4">
                <div className="flex-1">
                  <h2 className="text-xl font-semibold text-gray-900 mb-2">
                    {currentReview.title}
                  </h2>
                  <div className="flex items-center space-x-4 text-sm text-gray-600">
                    <div className="flex items-center space-x-1">
                      <UserIcon className="w-4 h-4" />
                      <span>{currentReview.author}</span>
                    </div>
                    <span>Branch: {currentReview.branch}</span>
                  </div>
                </div>
                
                <span className={`px-3 py-1 text-sm font-medium rounded-full ${getStatusColor(currentReview.status)}`}>
                  {currentReview.status.replace('_', ' ')}
                </span>
              </div>

              <div className="flex items-center space-x-4 text-sm text-gray-600">
                <span>Reviewers:</span>
                {currentReview.reviewers.map((reviewer, index) => (
                  <span key={index} className="px-2 py-1 bg-gray-100 rounded">
                    {reviewer}
                  </span>
                ))}
              </div>
            </div>

            {/* Code Changes */}
            <div className="bg-white border border-gray-200 rounded-lg">
              <div className="p-4 border-b border-gray-200">
                <h3 className="font-semibold text-gray-900">Changes</h3>
              </div>
              <div className="p-4">
                <div className="bg-gray-50 p-4 rounded font-mono text-sm">
                  <div className="text-green-600">+ def authenticate_user(username, password):</div>
                  <div className="text-green-600">+     """Authenticate user with username and password."""</div>
                  <div className="text-green-600">+     user = get_user_by_username(username)</div>
                  <div className="text-green-600">+     if user and check_password(user.password_hash, password):</div>
                  <div className="text-green-600">+         return user</div>
                  <div className="text-green-600">+     return None</div>
                </div>
              </div>
            </div>

            {/* Comments */}
            <div className="bg-white border border-gray-200 rounded-lg">
              <div className="p-4 border-b border-gray-200">
                <h3 className="font-semibold text-gray-900">Comments ({currentReview.comments.length})</h3>
              </div>
              <div className="p-4 space-y-4">
                {currentReview.comments.map((comment) => (
                  <div key={comment.id} className="border border-gray-200 rounded-lg p-4">
                    <div className="flex items-start justify-between mb-2">
                      <div className="flex items-center space-x-2">
                        <div className="w-8 h-8 bg-blue-500 rounded-full flex items-center justify-center text-white text-sm">
                          {comment.author.charAt(0)}
                        </div>
                        <div>
                          <span className="font-medium text-gray-900">{comment.author}</span>
                          <span className="text-sm text-gray-500 ml-2">
                            Line {comment.line}
                          </span>
                        </div>
                      </div>
                      
                      <div className="flex items-center space-x-2">
                        {getCommentIcon(comment.type)}
                        <span className="text-xs text-gray-500">{comment.type}</span>
                      </div>
                    </div>
                    
                    <p className="text-gray-700 mb-2">{comment.content}</p>
                    
                    <div className="flex items-center space-x-4 text-sm text-gray-500">
                      <span>{comment.timestamp}</span>
                      <button className="text-blue-600 hover:text-blue-800">Reply</button>
                      <button className="text-green-600 hover:text-green-800">Resolve</button>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Add Comment */}
            <div className="bg-white border border-gray-200 rounded-lg p-4">
              <h3 className="font-semibold text-gray-900 mb-4">Add Comment</h3>
              <div className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Line Number
                    </label>
                    <input
                      type="number"
                      className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                      placeholder="e.g., 45"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Comment Type
                    </label>
                    <select className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500">
                      <option value="general">General</option>
                      <option value="suggestion">Suggestion</option>
                      <option value="issue">Issue</option>
                      <option value="question">Question</option>
                      <option value="praise">Praise</option>
                    </select>
                  </div>
                </div>
                
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Comment
                  </label>
                  <textarea
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                    rows={4}
                    placeholder="Provide your feedback..."
                  />
                </div>
                
                <div className="flex justify-end">
                  <button className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors">
                    Add Comment
                  </button>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default CodeReview;
