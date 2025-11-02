import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { useVCS } from '../context/VCSContext';
import { 
  PlayIcon,
  PauseIcon,
  UserGroupIcon,
  ChatBubbleLeftRightIcon,
  ExclamationTriangleIcon
} from '@heroicons/react/24/outline';
import { toast } from 'react-hot-toast';

const CollaborativeEditor = ({ socket }) => {
  const { filePath } = useParams();
  const { currentUser, API_URL, currentRepo } = useVCS();
  const [content, setContent] = useState('');
  const [isConnected, setIsConnected] = useState(false);
  const [activeUsers, setActiveUsers] = useState([]);
  const [sessionId, setSessionId] = useState(null);
  const [unsavedChanges, setUnsavedChanges] = useState(false);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    if (socket && currentUser && currentRepo && filePath) {
      initializeCollaborativeSession();
    }

    return () => {
      if (sessionId) {
        socket?.emit('leave_session', { session_id: sessionId });
      }
    };
  }, [socket, currentUser, currentRepo, filePath]);

  useEffect(() => {
    if (!socket) return;

    socket.on('connected', () => {
      setIsConnected(true);
    });

    socket.on('joined_session', (data) => {
      setActiveUsers(data.users || []);
      toast.success('Joined collaborative session!');
    });

    socket.on('content_updated', (data) => {
      if (data.user !== currentUser?.id) {
        setContent(data.content);
        setUnsavedChanges(false);
      }
    });

    socket.on('repository_updated', (data) => {
      toast.success('Repository updated!');
    });

    return () => {
      socket.off('connected');
      socket.off('joined_session');
      socket.off('content_updated');
      socket.off('repository_updated');
    };
  }, [socket, currentUser]);

  const initializeCollaborativeSession = async () => {
    const newSessionId = `${currentRepo.id}_${filePath}_${Date.now()}`;
    setSessionId(newSessionId);

    socket.emit('join_session', {
      session_id: newSessionId,
      file_path: filePath,
      user: currentUser?.id
    });
  };

  const handleContentChange = (e) => {
    const newContent = e.target.value;
    setContent(newContent);
    setUnsavedChanges(true);

    // Broadcast changes to other users
    socket.emit('edit_content', {
      session_id: sessionId,
      content: newContent,
      changes: [],
      user: currentUser?.id
    });
  };

  const saveToRepository = async () => {
    if (!content.trim()) {
      toast.error('No content to save');
      return;
    }

    setIsSaving(true);
    try {
      const response = await fetch(`${API_URL}/repos/${currentRepo.path}/add`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          file_path: filePath,
          content: content,
          user: currentUser?.name
        })
      });

      const result = await response.json();

      if (result.status === 'success') {
        // Create commit
        const commitResponse = await fetch(`${API_URL}/repos/${currentRepo.path}/commit`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({
            message: `Update ${filePath}`,
            user: currentUser?.name
          })
        });

        const commitResult = await commitResponse.json();

        if (commitResult.status === 'success') {
          setUnsavedChanges(false);
          toast.success('Changes saved to repository!');
        }
      }
    } catch (error) {
      console.error('Failed to save:', error);
      toast.error('Failed to save changes');
    } finally {
      setIsSaving(false);
    }
  };

  const getFileLanguage = (filePath) => {
    const extension = filePath.split('.').pop()?.toLowerCase();
    const languageMap = {
      'js': 'javascript',
      'jsx': 'javascript',
      'ts': 'typescript',
      'tsx': 'typescript',
      'py': 'python',
      'java': 'java',
      'cpp': 'cpp',
      'c': 'c',
      'html': 'html',
      'css': 'css'
    };
    return languageMap[extension] || 'text';
  };

  if (!currentRepo) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <p className="text-gray-500">Please select a repository first</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="bg-white border-b border-gray-200 p-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-4">
            <h1 className="text-lg font-semibold text-gray-900">
              {filePath || 'Untitled'}
            </h1>
            <div className="flex items-center space-x-2">
              {isConnected ? (
                <div className="flex items-center space-x-1 text-green-600">
                  <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
                  <span className="text-sm">Live</span>
                </div>
              ) : (
                <div className="flex items-center space-x-1 text-red-600">
                  <div className="w-2 h-2 bg-red-500 rounded-full"></div>
                  <span className="text-sm">Offline</span>
                </div>
              )}
              
              {unsavedChanges && (
                <div className="flex items-center space-x-1 text-yellow-600">
                  <ExclamationTriangleIcon className="w-4 h-4" />
                  <span className="text-sm">Unsaved</span>
                </div>
              )}
            </div>
          </div>

          <div className="flex items-center space-x-4">
            {/* Active Users */}
            <div className="flex items-center space-x-2">
              <UserGroupIcon className="w-5 h-5 text-gray-400" />
              <div className="flex -space-x-2">
                {activeUsers.slice(0, 3).map((user, index) => (
                  <div
                    key={user.sid}
                    className="w-8 h-8 bg-blue-500 rounded-full flex items-center justify-center text-white text-sm font-medium border-2 border-white"
                  >
                    {user.user?.charAt(0) || 'U'}
                  </div>
                ))}
                {activeUsers.length > 3 && (
                  <div className="w-8 h-8 bg-gray-500 rounded-full flex items-center justify-center text-white text-sm font-medium border-2 border-white">
                    +{activeUsers.length - 3}
                  </div>
                )}
              </div>
            </div>

            {/* Save Button */}
            <button
              onClick={saveToRepository}
              disabled={isSaving || !unsavedChanges}
              className={`
                px-4 py-2 rounded-lg font-medium transition-colors
                ${unsavedChanges && !isSaving
                  ? 'bg-blue-600 text-white hover:bg-blue-700'
                  : 'bg-gray-200 text-gray-500 cursor-not-allowed'
                }
              `}
            >
              {isSaving ? 'Saving...' : 'Save'}
            </button>
          </div>
        </div>
      </div>

      {/* Editor */}
      <div className="flex-1 flex">
        <div className="flex-1">
          <textarea
            value={content}
            onChange={handleContentChange}
            placeholder={`Start typing your ${getFileLanguage(filePath)} code...`}
            className="w-full h-full p-4 font-mono text-sm border-none outline-none resize-none"
            style={{ fontFamily: 'Monaco, Menlo, "Ubuntu Mono", monospace' }}
          />
        </div>

        {/* Side Panel */}
        <div className="w-64 bg-gray-50 border-l border-gray-200 p-4">
          <h3 className="font-semibold text-gray-900 mb-4">Collaboration</h3>
          
          {/* Real-time Status */}
          <div className="mb-6">
            <h4 className="text-sm font-medium text-gray-700 mb-2">Session Status</h4>
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-600">Connection</span>
                <span className={`text-sm font-medium ${
                  isConnected ? 'text-green-600' : 'text-red-600'
                }`}>
                  {isConnected ? 'Connected' : 'Disconnected'}
                </span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-600">Active Users</span>
                <span className="text-sm font-medium text-gray-900">
                  {activeUsers.length}
                </span>
              </div>
            </div>
          </div>

          {/* Active Users */}
          <div className="mb-6">
            <h4 className="text-sm font-medium text-gray-700 mb-2">Active Users</h4>
            <div className="space-y-2">
              {activeUsers.map((user) => (
                <div key={user.sid} className="flex items-center space-x-2">
                  <div className="w-6 h-6 bg-blue-500 rounded-full flex items-center justify-center text-white text-xs">
                    {user.user?.charAt(0) || 'U'}
                  </div>
                  <span className="text-sm text-gray-600 truncate">
                    {user.user || 'Unknown'}
                  </span>
                  <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
                </div>
              ))}
              {activeUsers.length === 0 && (
                <p className="text-sm text-gray-400">No other users online</p>
              )}
            </div>
          </div>

          {/* Educational Tips */}
          <div className="mb-6">
            <h4 className="text-sm font-medium text-gray-700 mb-2">💡 Tips</h4>
            <div className="space-y-2 text-xs text-gray-600">
              <p>• Changes are saved automatically</p>
              <p>• Use descriptive commit messages</p>
              <p>• Collaborate using comments</p>
              <p>• Review before merging</p>
            </div>
          </div>

          {/* File Info */}
          <div>
            <h4 className="text-sm font-medium text-gray-700 mb-2">File Info</h4>
            <div className="space-y-1 text-xs text-gray-600">
              <p><strong>Language:</strong> {getFileLanguage(filePath)}</p>
              <p><strong>Repository:</strong> {currentRepo.name}</p>
              <p><strong>Lines:</strong> {content.split('\n').length}</p>
              <p><strong>Characters:</strong> {content.length}</p>
            </div>
          </div>
        </div>
      </div>

      {/* Bottom Status Bar */}
      <div className="bg-white border-t border-gray-200 px-4 py-2">
        <div className="flex items-center justify-between text-sm text-gray-500">
          <div className="flex items-center space-x-4">
            <span>Line {content.split('\n').findIndex((line, index) => 
              index === content.split('\n').length - 1 || line.length > 0) + 1 || 1}</span>
            <span>Column 1</span>
            <span>{getFileLanguage(filePath)}</span>
          </div>
          <div className="flex items-center space-x-4">
            {unsavedChanges && <span>Unsaved changes</span>}
            <span>UTF-8</span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default CollaborativeEditor;
