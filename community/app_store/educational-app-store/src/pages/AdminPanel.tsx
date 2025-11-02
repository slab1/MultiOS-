import React, { useState, useEffect } from 'react';
import { useAuth } from '../contexts/AuthContext';
import { 
  Users, 
  BookOpen, 
  Star, 
  TrendingUp,
  Eye,
  CheckCircle,
  XCircle,
  Clock,
  AlertCircle,
  BarChart3,
  Shield,
  Settings
} from 'lucide-react';

interface AdminStats {
  totalUsers: number;
  totalApps: number;
  pendingReviews: number;
  avgRating: number;
  totalDownloads: number;
  recentSignups: number;
}

interface PendingSubmission {
  id: string;
  title: string;
  developer_name: string;
  submitted_at: string;
  status: string;
}

const AdminPanel: React.FC = () => {
  const { user } = useAuth();
  const [stats, setStats] = useState<AdminStats | null>(null);
  const [pendingSubmissions, setPendingSubmissions] = useState<PendingSubmission[]>([]);
  const [activeTab, setActiveTab] = useState<'overview' | 'submissions' | 'users' | 'analytics'>('overview');
  const [loading, setLoading] = useState(true);

  const API_BASE = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3001/api';

  useEffect(() => {
    if (user?.role === 'admin') {
      loadAdminData();
    }
  }, [user]);

  const loadAdminData = async () => {
    try {
      setLoading(true);
      
      // Load platform statistics
      const statsResponse = await fetch(`${API_BASE}/analytics/platform/overview`, {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('authToken')}`
        }
      });
      const statsData = await statsResponse.json();
      if (statsData.success) {
        setStats({
          totalUsers: statsData.data.platformStats.totalDevelopers + statsData.data.platformStats.totalEducators,
          totalApps: statsData.data.platformStats.totalApprovedApps,
          pendingReviews: statsData.data.platformStats.pendingApps,
          avgRating: statsData.data.platformStats.avgPlatformRating,
          totalDownloads: statsData.data.platformStats.totalDownloads,
          recentSignups: 45 // Mock data
        });
      }

      // Load pending submissions
      const submissionsResponse = await fetch(`${API_BASE}/submissions/pending/all`, {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('authToken')}`
        }
      });
      const submissionsData = await submissionsResponse.json();
      if (submissionsData.success) {
        setPendingSubmissions(submissionsData.data.items);
      }
    } catch (error) {
      console.error('Error loading admin data:', error);
    } finally {
      setLoading(false);
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric'
    });
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'submitted': return 'text-yellow-600 bg-yellow-100';
      case 'under_review': return 'text-blue-600 bg-blue-100';
      case 'approved': return 'text-green-600 bg-green-100';
      case 'rejected': return 'text-red-600 bg-red-100';
      default: return 'text-gray-600 bg-gray-100';
    }
  };

  if (user?.role !== 'admin') {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-center">
          <Shield className="h-16 w-16 text-red-500 mx-auto mb-4" />
          <h2 className="text-2xl font-bold text-gray-900 mb-4">Access Denied</h2>
          <p className="text-gray-600">You need administrator privileges to access this page.</p>
        </div>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900">Admin Dashboard</h1>
          <p className="text-gray-600 mt-2">Manage the educational app store</p>
        </div>

        {/* Tabs */}
        <div className="bg-white rounded-lg shadow-md mb-8">
          <div className="border-b border-gray-200">
            <nav className="flex space-x-8 px-6">
              {[
                { id: 'overview', label: 'Overview', icon: BarChart3 },
                { id: 'submissions', label: 'Submissions', icon: BookOpen },
                { id: 'users', label: 'Users', icon: Users },
                { id: 'analytics', label: 'Analytics', icon: TrendingUp }
              ].map((tab) => (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id as any)}
                  className={`flex items-center space-x-2 py-4 border-b-2 font-medium text-sm ${
                    activeTab === tab.id
                      ? 'border-blue-500 text-blue-600'
                      : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                  }`}
                >
                  <tab.icon className="h-4 w-4" />
                  <span>{tab.label}</span>
                </button>
              ))}
            </nav>
          </div>

          <div className="p-6">
            {/* Overview Tab */}
            {activeTab === 'overview' && stats && (
              <div className="space-y-6">
                <h2 className="text-xl font-bold text-gray-900">Platform Overview</h2>
                
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                  <div className="bg-blue-50 rounded-lg p-6">
                    <div className="flex items-center">
                      <Users className="h-8 w-8 text-blue-600" />
                      <div className="ml-4">
                        <p className="text-sm font-medium text-blue-600">Total Users</p>
                        <p className="text-2xl font-bold text-blue-900">{stats.totalUsers}</p>
                      </div>
                    </div>
                  </div>

                  <div className="bg-green-50 rounded-lg p-6">
                    <div className="flex items-center">
                      <BookOpen className="h-8 w-8 text-green-600" />
                      <div className="ml-4">
                        <p className="text-sm font-medium text-green-600">Total Apps</p>
                        <p className="text-2xl font-bold text-green-900">{stats.totalApps}</p>
                      </div>
                    </div>
                  </div>

                  <div className="bg-yellow-50 rounded-lg p-6">
                    <div className="flex items-center">
                      <Clock className="h-8 w-8 text-yellow-600" />
                      <div className="ml-4">
                        <p className="text-sm font-medium text-yellow-600">Pending Reviews</p>
                        <p className="text-2xl font-bold text-yellow-900">{stats.pendingReviews}</p>
                      </div>
                    </div>
                  </div>

                  <div className="bg-purple-50 rounded-lg p-6">
                    <div className="flex items-center">
                      <Star className="h-8 w-8 text-purple-600" />
                      <div className="ml-4">
                        <p className="text-sm font-medium text-purple-600">Average Rating</p>
                        <p className="text-2xl font-bold text-purple-900">{stats.avgRating.toFixed(1)}</p>
                      </div>
                    </div>
                  </div>

                  <div className="bg-indigo-50 rounded-lg p-6">
                    <div className="flex items-center">
                      <TrendingUp className="h-8 w-8 text-indigo-600" />
                      <div className="ml-4">
                        <p className="text-sm font-medium text-indigo-600">Total Downloads</p>
                        <p className="text-2xl font-bold text-indigo-900">{stats.totalDownloads.toLocaleString()}</p>
                      </div>
                    </div>
                  </div>

                  <div className="bg-pink-50 rounded-lg p-6">
                    <div className="flex items-center">
                      <Users className="h-8 w-8 text-pink-600" />
                      <div className="ml-4">
                        <p className="text-sm font-medium text-pink-600">Recent Signups</p>
                        <p className="text-2xl font-bold text-pink-900">{stats.recentSignups}</p>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            )}

            {/* Submissions Tab */}
            {activeTab === 'submissions' && (
              <div className="space-y-6">
                <div className="flex items-center justify-between">
                  <h2 className="text-xl font-bold text-gray-900">Pending Submissions</h2>
                  <span className="text-sm text-gray-600">{pendingSubmissions.length} pending</span>
                </div>

                {pendingSubmissions.length === 0 ? (
                  <div className="text-center py-12">
                    <CheckCircle className="h-16 w-16 text-gray-300 mx-auto mb-4" />
                    <h3 className="text-lg font-medium text-gray-900 mb-2">All caught up!</h3>
                    <p className="text-gray-600">No pending submissions to review.</p>
                  </div>
                ) : (
                  <div className="space-y-4">
                    {pendingSubmissions.map((submission) => (
                      <div key={submission.id} className="border border-gray-200 rounded-lg p-6">
                        <div className="flex items-center justify-between">
                          <div>
                            <h3 className="font-semibold text-gray-900">{submission.title}</h3>
                            <p className="text-sm text-gray-600">by {submission.developer_name}</p>
                            <p className="text-sm text-gray-500">Submitted {formatDate(submission.submitted_at)}</p>
                          </div>
                          <div className="flex items-center space-x-3">
                            <span className={`text-xs px-3 py-1 rounded-full ${getStatusColor(submission.status)}`}>
                              {submission.status.replace('_', ' ')}
                            </span>
                            <button className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 text-sm">
                              Review
                            </button>
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}

            {/* Users Tab */}
            {activeTab === 'users' && (
              <div className="space-y-6">
                <h2 className="text-xl font-bold text-gray-900">User Management</h2>
                
                <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-6">
                  <div className="flex items-start space-x-3">
                    <AlertCircle className="h-6 w-6 text-yellow-600 mt-0.5" />
                    <div>
                      <h3 className="font-semibold text-yellow-900 mb-2">User Management Features</h3>
                      <p className="text-yellow-800 text-sm">
                        Advanced user management features including user roles, permissions, and account management 
                        will be implemented in a future update.
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            )}

            {/* Analytics Tab */}
            {activeTab === 'analytics' && (
              <div className="space-y-6">
                <h2 className="text-xl font-bold text-gray-900">Platform Analytics</h2>
                
                <div className="bg-blue-50 border border-blue-200 rounded-lg p-6">
                  <div className="flex items-start space-x-3">
                    <BarChart3 className="h-6 w-6 text-blue-600 mt-0.5" />
                    <div>
                      <h3 className="font-semibold text-blue-900 mb-2">Advanced Analytics Dashboard</h3>
                      <p className="text-blue-800 text-sm">
                        Detailed analytics including user behavior, app performance metrics, revenue tracking, 
                        and comprehensive reporting will be available in the full version.
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Quick Actions */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="bg-white rounded-lg shadow-md p-6">
            <div className="flex items-center space-x-3 mb-4">
              <BookOpen className="h-8 w-8 text-blue-600" />
              <h3 className="font-semibold text-gray-900">Review Submissions</h3>
            </div>
            <p className="text-gray-600 text-sm mb-4">Review and approve app submissions</p>
            <button 
              onClick={() => setActiveTab('submissions')}
              className="w-full bg-blue-600 text-white py-2 rounded-lg hover:bg-blue-700 transition-colors"
            >
              Go to Reviews
            </button>
          </div>

          <div className="bg-white rounded-lg shadow-md p-6">
            <div className="flex items-center space-x-3 mb-4">
              <Settings className="h-8 w-8 text-green-600" />
              <h3 className="font-semibold text-gray-900">Platform Settings</h3>
            </div>
            <p className="text-gray-600 text-sm mb-4">Configure platform settings and policies</p>
            <button className="w-full bg-green-600 text-white py-2 rounded-lg hover:bg-green-700 transition-colors">
              Manage Settings
            </button>
          </div>

          <div className="bg-white rounded-lg shadow-md p-6">
            <div className="flex items-center space-x-3 mb-4">
              <Users className="h-8 w-8 text-purple-600" />
              <h3 className="font-semibold text-gray-900">User Support</h3>
            </div>
            <p className="text-gray-600 text-sm mb-4">Handle user support requests</p>
            <button className="w-full bg-purple-600 text-white py-2 rounded-lg hover:bg-purple-700 transition-colors">
              View Support
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default AdminPanel;