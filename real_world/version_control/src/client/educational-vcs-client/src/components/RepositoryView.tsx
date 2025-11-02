import React from 'react';
import { useParams, Link } from 'react-router-dom';
import { 
  FolderIcon,
  CodeBracketIcon,
  GitBranchIcon,
  ClockIcon,
  UserIcon
} from '@heroicons/react/24/outline';
import { useVCS } from '../context/VCSContext';

const RepositoryView = ({ currentRepo }) => {
  const { '*': filePath } = useParams();
  const { API_URL } = useVCS();
  
  // Mock data for demonstration
  const mockFiles = [
    { name: 'src', type: 'folder', size: '-', modified: '2024-12-04' },
    { name: 'README.md', type: 'file', size: '2.1 KB', modified: '2024-12-04' },
    { name: 'package.json', type: 'file', size: '856 B', modified: '2024-12-03' },
    { name: '.gitignore', type: 'file', size: '124 B', modified: '2024-12-02' }
  ];

  const mockCommits = [
    {
      hash: 'abc1234',
      message: 'Add user authentication feature',
      author: 'Alice Smith',
      timestamp: '2024-12-04 10:30:00',
      branch: 'feature/auth'
    },
    {
      hash: 'def5678',
      message: 'Fix bug in data validation',
      author: 'Bob Johnson',
      timestamp: '2024-12-03 15:45:00',
      branch: 'main'
    },
    {
      hash: 'ghi9012',
      message: 'Update documentation',
      author: 'Carol Davis',
      timestamp: '2024-12-03 09:20:00',
      branch: 'docs/update'
    }
  ];

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="bg-white border-b border-gray-200 p-6">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-4">
            <FolderIcon className="w-8 h-8 text-blue-600" />
            <div>
              <h1 className="text-2xl font-bold text-gray-900">
                {currentRepo?.name || 'Repository'}
              </h1>
              <p className="text-gray-600">
                {currentRepo?.path || '/path/to/repo'}
              </p>
            </div>
          </div>
          
          <div className="flex space-x-3">
            <button className="px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors">
              Clone
            </button>
            <button className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors">
              New File
            </button>
          </div>
        </div>
      </div>

      <div className="flex-1 flex overflow-hidden">
        {/* Sidebar - File Tree */}
        <div className="w-80 bg-white border-r border-gray-200 overflow-y-auto">
          <div className="p-4 border-b border-gray-200">
            <h3 className="text-sm font-semibold text-gray-700">Files</h3>
          </div>
          
          <div className="p-2">
            {mockFiles.map((file, index) => (
              <div
                key={index}
                className="flex items-center space-x-2 p-2 rounded hover:bg-gray-50 cursor-pointer"
              >
                <CodeBracketIcon className="w-4 h-4 text-gray-400" />
                <span className="text-sm text-gray-700 flex-1 truncate">
                  {file.name}
                </span>
                <span className="text-xs text-gray-400">{file.size}</span>
              </div>
            ))}
          </div>
        </div>

        {/* Main Content */}
        <div className="flex-1 overflow-y-auto">
          {/* Commit History */}
          <div className="p-6">
            <h2 className="text-xl font-semibold text-gray-900 mb-6">
              Recent Commits
            </h2>
            
            <div className="space-y-4">
              {mockCommits.map((commit, index) => (
                <div
                  key={index}
                  className="flex items-start space-x-4 p-4 bg-white border border-gray-200 rounded-lg hover:shadow-sm transition-shadow"
                >
                  <div className="w-10 h-10 bg-blue-100 rounded-full flex items-center justify-center">
                    <GitBranchIcon className="w-5 h-5 text-blue-600" />
                  </div>
                  
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center space-x-2 mb-1">
                      <code className="text-sm font-mono text-gray-600 bg-gray-100 px-2 py-1 rounded">
                        {commit.hash.substring(0, 7)}
                      </code>
                      <span className="text-sm text-gray-500">
                        {commit.branch}
                      </span>
                    </div>
                    
                    <h3 className="font-medium text-gray-900 mb-1">
                      {commit.message}
                    </h3>
                    
                    <div className="flex items-center space-x-4 text-sm text-gray-500">
                      <div className="flex items-center space-x-1">
                        <UserIcon className="w-4 h-4" />
                        <span>{commit.author}</span>
                      </div>
                      <div className="flex items-center space-x-1">
                        <ClockIcon className="w-4 h-4" />
                        <span>{commit.timestamp}</span>
                      </div>
                    </div>
                  </div>
                  
                  <div className="flex space-x-2">
                    <button className="text-sm text-blue-600 hover:text-blue-800">
                      View
                    </button>
                    <button className="text-sm text-gray-600 hover:text-gray-800">
                      Compare
                    </button>
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

export default RepositoryView;
