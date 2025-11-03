import React, { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import {
  PlusIcon,
  CloudIcon,
  CpuChipIcon,
  AcademicCapIcon,
  UserGroupIcon,
  CheckCircleIcon,
  ExclamationCircleIcon,
  ClockIcon,
} from '@heroicons/react/24/outline';
import { useAuth } from '../../contexts/AuthContext';
import { useToast } from '../../contexts/ToastContext';

interface LMSIntegration {
  id: string;
  name: string;
  type: 'canvas' | 'blackboard' | 'moodle' | 'google_classroom' | 'microsoft_teams' | 'lti_custom';
  baseUrl: string;
  isActive: boolean;
  lastSync?: string;
  institution?: {
    name: string;
    domain: string;
  };
  createdAt: string;
}

const LMSIntegrations: React.FC = () => {
  const { hasPermission } = useAuth();
  const { success, error } = useToast();
  const [integrations, setIntegrations] = useState<LMSIntegration[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  const canCreateIntegrations = hasPermission(['instructor', 'administrator']);

  useEffect(() => {
    loadIntegrations();
  }, []);

  const loadIntegrations = async () => {
    try {
      setIsLoading(true);
      
      // Simulate API call - replace with actual API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const mockIntegrations: LMSIntegration[] = [
        {
          id: '1',
          name: 'Canvas LMS',
          type: 'canvas',
          baseUrl: 'https://institution.instructure.com',
          isActive: true,
          lastSync: '2024-01-15T10:30:00Z',
          institution: {
            name: 'Tech University',
            domain: 'tech.edu'
          },
          createdAt: '2024-01-10T09:00:00Z'
        },
        {
          id: '2',
          name: 'Blackboard Learn',
          type: 'blackboard',
          baseUrl: 'https://learn.institution.edu',
          isActive: false,
          lastSync: '2024-01-12T14:15:00Z',
          institution: {
            name: 'Tech University',
            domain: 'tech.edu'
          },
          createdAt: '2024-01-08T11:30:00'
        },
        {
          id: '3',
          name: 'Google Classroom',
          type: 'google_classroom',
          baseUrl: 'https://classroom.google.com',
          isActive: true,
          lastSync: '2024-01-15T11:45:00Z',
          createdAt: '2024-01-14T16:20:00'
        },
        {
          id: '4',
          name: 'Moodle Platform',
          type: 'moodle',
          baseUrl: 'https://moodle.institution.edu',
          isActive: true,
          createdAt: '2024-01-11T13:45:00'
        }
      ];
      
      setIntegrations(mockIntegrations);
    } catch (err) {
      error('Failed to Load Integrations', 'Could not load LMS integrations');
    } finally {
      setIsLoading(false);
    }
  };

  const getLMSIcon = (type: string) => {
    switch (type) {
      case 'canvas':
        return <CpuChipIcon className="h-8 w-8 text-indigo-600" />;
      case 'blackboard':
        return <AcademicCapIcon className="h-8 w-8 text-blue-600" />;
      case 'moodle':
        return <CloudIcon className="h-8 w-8 text-green-600" />;
      case 'google_classroom':
        return <div className="h-8 w-8 text-red-600 font-bold">G</div>;
      case 'microsoft_teams':
        return <div className="h-8 w-8 text-blue-800 font-bold">T</div>;
      case 'lti_custom':
        return <CloudIcon className="h-8 w-8 text-purple-600" />;
      default:
        return <CloudIcon className="h-8 w-8 text-gray-600" />;
    }
  };

  const getLMSName = (type: string) => {
    switch (type) {
      case 'canvas':
        return 'Canvas LMS';
      case 'blackboard':
        return 'Blackboard Learn';
      case 'moodle':
        return 'Moodle';
      case 'google_classroom':
        return 'Google Classroom';
      case 'microsoft_teams':
        return 'Microsoft Teams';
      case 'lti_custom':
        return 'LTI Custom';
      default:
        return type;
    }
  };

  const getStatusIcon = (isActive: boolean) => {
    return isActive ? (
      <CheckCircleIcon className="h-5 w-5 text-green-500" />
    ) : (
      <ExclamationCircleIcon className="h-5 w-5 text-red-500" />
    );
  };

  const getStatusText = (isActive: boolean) => {
    return isActive ? 'Active' : 'Inactive';
  };

  const formatLastSync = (lastSync?: string) => {
    if (!lastSync) return 'Never';
    
    const date = new Date(lastSync);
    const now = new Date();
    const diffInHours = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60));
    
    if (diffInHours < 1) return 'Just now';
    if (diffInHours < 24) return `${diffInHours} hours ago`;
    
    const diffInDays = Math.floor(diffInHours / 24);
    if (diffInDays < 7) return `${diffInDays} days ago`;
    
    return date.toLocaleDateString();
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">LMS Integrations</h1>
          <p className="mt-1 text-sm text-gray-500">
            Manage connections to external Learning Management Systems
          </p>
        </div>
        
        {canCreateIntegrations && (
          <Link
            to="/lms/create"
            className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
          >
            <PlusIcon className="h-4 w-4 mr-2" />
            Add Integration
          </Link>
        )}
      </div>

      {/* Integrations Grid */}
      <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
        {integrations.map((integration) => (
          <div
            key={integration.id}
            className="bg-white overflow-hidden shadow rounded-lg hover:shadow-md transition-shadow"
          >
            <div className="p-6">
              {/* Header */}
              <div className="flex items-center justify-between">
                <div className="flex items-center">
                  <div className="flex-shrink-0">
                    {getLMSIcon(integration.type)}
                  </div>
                  <div className="ml-3">
                    <h3 className="text-lg font-medium text-gray-900">
                      {integration.name}
                    </h3>
                    <p className="text-sm text-gray-500">
                      {getLMSName(integration.type)}
                    </p>
                  </div>
                </div>
                <div className="flex items-center">
                  {getStatusIcon(integration.isActive)}
                </div>
              </div>

              {/* URL */}
              <div className="mt-4">
                <p className="text-sm text-gray-500">Base URL</p>
                <p className="text-sm font-medium text-gray-900 break-all">
                  {integration.baseUrl}
                </p>
              </div>

              {/* Status and sync info */}
              <div className="mt-4 grid grid-cols-2 gap-4">
                <div>
                  <p className="text-sm text-gray-500">Status</p>
                  <div className="flex items-center mt-1">
                    {getStatusIcon(integration.isActive)}
                    <span className="ml-1 text-sm font-medium text-gray-900">
                      {getStatusText(integration.isActive)}
                    </span>
                  </div>
                </div>
                <div>
                  <p className="text-sm text-gray-500">Last Sync</p>
                  <div className="flex items-center mt-1">
                    <ClockIcon className="h-4 w-4 text-gray-400" />
                    <span className="ml-1 text-sm text-gray-900">
                      {formatLastSync(integration.lastSync)}
                    </span>
                  </div>
                </div>
              </div>

              {/* Institution */}
              {integration.institution && (
                <div className="mt-4">
                  <p className="text-sm text-gray-500">Institution</p>
                  <p className="text-sm font-medium text-gray-900">
                    {integration.institution.name}
                  </p>
                </div>
              )}

              {/* Actions */}
              <div className="mt-6 flex space-x-3">
                <Link
                  to={`/lms/${integration.id}`}
                  className="flex-1 bg-white py-2 px-3 border border-gray-300 rounded-md shadow-sm text-sm leading-4 font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 text-center"
                >
                  View Details
                </Link>
                <button
                  onClick={() => {
                    success('Sync Started', `Synchronization started for ${integration.name}`);
                  }}
                  className="flex-1 bg-indigo-600 py-2 px-3 border border-transparent rounded-md shadow-sm text-sm leading-4 font-medium text-white hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 text-center"
                  disabled={!integration.isActive}
                >
                  Sync Now
                </button>
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Empty state */}
      {integrations.length === 0 && (
        <div className="text-center py-12">
          <CloudIcon className="mx-auto h-12 w-12 text-gray-400" />
          <h3 className="mt-2 text-sm font-medium text-gray-900">
            No LMS integrations
          </h3>
          <p className="mt-1 text-sm text-gray-500">
            Get started by connecting to a Learning Management System.
          </p>
          {canCreateIntegrations && (
            <div className="mt-6">
              <Link
                to="/lms/create"
                className="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
              >
                <PlusIcon className="h-4 w-4 mr-2" />
                Add Integration
              </Link>
            </div>
          )}
        </div>
      )}

      {/* Help section */}
      <div className="bg-blue-50 border border-blue-200 rounded-md p-4">
        <div className="flex">
          <div className="flex-shrink-0">
            <CloudIcon className="h-5 w-5 text-blue-400" />
          </div>
          <div className="ml-3">
            <h3 className="text-sm font-medium text-blue-800">
              Need help with LMS integration?
            </h3>
            <div className="mt-2 text-sm text-blue-700">
              <p>
                LMS integrations allow you to sync courses, assignments, and student data 
                between MultiOS and external learning management systems. Each integration 
                supports real-time synchronization and automatic grade passing back.
              </p>
            </div>
            <div className="mt-4">
              <div className="flex space-x-4">
                <a
                  href="#"
                  className="text-sm font-medium text-blue-800 hover:text-blue-700"
                >
                  Integration Guide →
                </a>
                <a
                  href="#"
                  className="text-sm font-medium text-blue-800 hover:text-blue-700"
                >
                  API Documentation →
                </a>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default LMSIntegrations;