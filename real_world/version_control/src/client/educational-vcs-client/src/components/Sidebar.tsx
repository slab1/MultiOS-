import React, { useState, useEffect } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { 
  HomeIcon, 
  FolderIcon, 
  CodeBracketIcon, 
  ClipboardDocumentListIcon,
  AcademicCapIcon,
  ChartBarIcon,
  BookOpenIcon,
  PlusIcon,
  UserCircleIcon,
  CogIcon
} from '@heroicons/react/24/outline';
import { useVCS } from '../context/VCSContext';

const Sidebar = ({ currentRepo, onRepoChange, currentUser }) => {
  const location = useLocation();
  const { API_URL } = useVCS();
  const [repositories, setRepositories] = useState([]);
  const [showCreateRepo, setShowCreateRepo] = useState(false);

  useEffect(() => {
    // Load repositories from localStorage
    const savedRepos = localStorage.getItem('vcs_repositories');
    if (savedRepos) {
      setRepositories(JSON.parse(savedRepos));
    }
  }, []);

  const navigationItems = [
    {
      name: 'Dashboard',
      href: '/dashboard',
      icon: HomeIcon,
      description: 'Your learning progress'
    },
    {
      name: 'Tutorials',
      href: '/tutorial',
      icon: BookOpenIcon,
      description: 'Learn version control'
    },
    {
      name: 'Quality Analysis',
      href: '/quality',
      icon: ChartBarIcon,
      description: 'Code quality insights'
    },
    {
      name: 'Assignments',
      href: '/assignments',
      icon: ClipboardDocumentListIcon,
      description: 'Course assignments'
    },
    {
      name: 'Code Reviews',
      href: '/review',
      icon: CodeBracketIcon,
      description: 'Peer review center'
    }
  ];

  const instructorItems = [
    {
      name: 'Instructor Panel',
      href: '/instructor',
      icon: AcademicCapIcon,
      description: 'Teaching tools'
    }
  ];

  const handleCreateRepository = async (e) => {
    e.preventDefault();
    const formData = new FormData(e.target);
    const repoName = formData.get('repoName');
    const repoPath = `/workspace/demo-repos/${repoName.toLowerCase().replace(/\s+/g, '-')}`;

    try {
      const response = await fetch(`${API_URL}/repos`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          path: repoPath,
          name: repoName
        })
      });

      const result = await response.json();

      if (result.status === 'success') {
        const newRepo = {
          id: Date.now().toString(),
          name: repoName,
          path: repoPath,
          createdAt: new Date().toISOString()
        };

        const updatedRepos = [...repositories, newRepo];
        setRepositories(updatedRepos);
        localStorage.setItem('vcs_repositories', JSON.stringify(updatedRepos));
        onRepoChange(newRepo);
        setShowCreateRepo(false);
      }
    } catch (error) {
      console.error('Failed to create repository:', error);
    }
  };

  return (
    <div className="w-64 bg-white border-r border-gray-200 flex flex-col">
      {/* Header */}
      <div className="p-4 border-b border-gray-200">
        <div className="flex items-center space-x-3">
          <div className="w-10 h-10 bg-gradient-to-r from-blue-500 to-purple-600 rounded-lg flex items-center justify-center">
            <CodeBracketIcon className="w-6 h-6 text-white" />
          </div>
          <div>
            <h1 className="text-lg font-bold text-gray-900">EduVCS</h1>
            <p className="text-sm text-gray-500">Learning Platform</p>
          </div>
        </div>
      </div>

      {/* User Profile */}
      {currentUser && (
        <div className="p-4 border-b border-gray-200">
          <div className="flex items-center space-x-3">
            <div className="text-2xl">{currentUser.avatar}</div>
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-gray-900 truncate">
                {currentUser.name}
              </p>
              <p className="text-xs text-gray-500 truncate">
                {currentUser.role === 'student' ? 'Student' : 'Instructor'}
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Navigation */}
      <nav className="flex-1 overflow-y-auto p-4">
        <div className="space-y-1">
          {navigationItems.map((item) => {
            const Icon = item.icon;
            const isActive = location.pathname === item.href || 
                           (item.href !== '/dashboard' && location.pathname.startsWith(item.href));
            
            return (
              <Link
                key={item.name}
                to={item.href}
                className={`
                  flex items-center space-x-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors
                  ${isActive 
                    ? 'bg-blue-50 text-blue-700 border-r-2 border-blue-700' 
                    : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900'
                  }
                `}
              >
                <Icon className="w-5 h-5" />
                <div>
                  <div>{item.name}</div>
                  <div className="text-xs text-gray-400">{item.description}</div>
                </div>
              </Link>
            );
          })}
        </div>

        {/* Instructor Section */}
        {currentUser?.role === 'instructor' && (
          <div className="mt-8">
            <div className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-3">
              Teaching Tools
            </div>
            <div className="space-y-1">
              {instructorItems.map((item) => {
                const Icon = item.icon;
                const isActive = location.pathname === item.href;
                
                return (
                  <Link
                    key={item.name}
                    to={item.href}
                    className={`
                      flex items-center space-x-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors
                      ${isActive 
                        ? 'bg-blue-50 text-blue-700 border-r-2 border-blue-700' 
                        : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900'
                      }
                    `}
                  >
                    <Icon className="w-5 h-5" />
                    <div>
                      <div>{item.name}</div>
                      <div className="text-xs text-gray-400">{item.description}</div>
                    </div>
                  </Link>
                );
              })}
            </div>
          </div>
        )}

        {/* Repositories */}
        <div className="mt-8">
          <div className="flex items-center justify-between mb-3">
            <div className="text-xs font-semibold text-gray-400 uppercase tracking-wider">
              Repositories
            </div>
            <button
              onClick={() => setShowCreateRepo(true)}
              className="p-1 text-gray-400 hover:text-gray-600 transition-colors"
            >
              <PlusIcon className="w-4 h-4" />
            </button>
          </div>
          
          <div className="space-y-1">
            {repositories.map((repo) => (
              <button
                key={repo.id}
                onClick={() => onRepoChange(repo)}
                className={`
                  w-full flex items-center space-x-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors
                  ${currentRepo?.id === repo.id
                    ? 'bg-blue-50 text-blue-700 border-r-2 border-blue-700'
                    : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900'
                  }
                `}
              >
                <FolderIcon className="w-5 h-5" />
                <div className="flex-1 text-left min-w-0">
                  <div className="truncate">{repo.name}</div>
                  <div className="text-xs text-gray-400 truncate">
                    {new Date(repo.createdAt).toLocaleDateString()}
                  </div>
                </div>
              </button>
            ))}
            
            {repositories.length === 0 && (
              <div className="text-center py-4">
                <FolderIcon className="w-8 h-8 text-gray-300 mx-auto mb-2" />
                <p className="text-sm text-gray-500">No repositories yet</p>
                <p className="text-xs text-gray-400">Create one to get started</p>
              </div>
            )}
          </div>
        </div>
      </nav>

      {/* Create Repository Modal */}
      {showCreateRepo && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-96">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">
              Create New Repository
            </h3>
            
            <form onSubmit={handleCreateRepository}>
              <div className="mb-4">
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Repository Name
                </label>
                <input
                  type="text"
                  name="repoName"
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                  placeholder="e.g., My Project"
                  required
                />
              </div>
              
              <div className="flex justify-end space-x-3">
                <button
                  type="button"
                  onClick={() => setShowCreateRepo(false)}
                  className="px-4 py-2 text-gray-600 hover:text-gray-800 transition-colors"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                >
                  Create
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};

export default Sidebar;
