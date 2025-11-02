import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';
import { useUser } from '../contexts/UserContext';
import { 
  BarChart3, 
  Download, 
  Star, 
  Eye, 
  TrendingUp,
  Calendar,
  DollarSign,
  Users,
  BookOpen,
  Settings,
  Plus,
  ArrowRight
} from 'lucide-react';

interface DashboardStats {
  totalApps: number;
  totalViews: number;
  totalDownloads: number;
  totalReviews: number;
  avgRating: number;
  conversionRate: number;
}

interface RecentApp {
  id: string;
  title: string;
  status: string;
  rating: number;
  download_count: number;
  created_at: string;
}

const DashboardPage: React.FC = () => {
  const { user } = useAuth();
  const { favorites, downloads, reviews } = useUser();
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [recentApps, setRecentApps] = useState<RecentApp[]>([]);
  const [loading, setLoading] = useState(true);

  const API_BASE = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3001/api';

  useEffect(() => {
    if (user) {
      loadDashboardData();
    }
  }, [user]);

  const loadDashboardData = async () => {
    try {
      setLoading(true);
      
      if (user?.role === 'developer') {
        // Load developer analytics
        const analyticsResponse = await fetch(`${API_BASE}/analytics/developer/summary`, {
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('authToken')}`
          }
        });
        const analyticsData = await analyticsResponse.json();
        if (analyticsData.success) {
          setStats(analyticsData.data.summary);
        }

        // Load recent apps
        const appsResponse = await fetch(`${API_BASE}/users/${user.id}/apps?limit=5`, {
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('authToken')}`
          }
        });
        const appsData = await appsResponse.json();
        if (appsData.success) {
          setRecentApps(appsData.data.items);
        }
      } else if (user?.role === 'educator') {
        // Load educator dashboard stats
        setStats({
          totalApps: downloads.length,
          totalViews: downloads.reduce((sum, app) => sum + (app.rating * 100), 0),
          totalDownloads: downloads.length,
          totalReviews: reviews.length,
          avgRating: reviews.length > 0 ? reviews.reduce((sum, review) => sum + review.rating, 0) / reviews.length : 0,
          conversionRate: 85 // Mock data
        });
      }
    } catch (error) {
      console.error('Error loading dashboard data:', error);
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
      case 'approved': return 'text-green-600 bg-green-100';
      case 'pending': return 'text-yellow-600 bg-yellow-100';
      case 'rejected': return 'text-red-600 bg-red-100';
      default: return 'text-gray-600 bg-gray-100';
    }
  };

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
          <h1 className="text-3xl font-bold text-gray-900">
            Welcome back, {user?.name}!
          </h1>
          <p className="text-gray-600 mt-2">
            {user?.role === 'developer' 
              ? 'Manage your educational apps and track performance'
              : 'Discover and manage your educational resources'
            }
          </p>
        </div>

        {/* Quick Actions */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
          {user?.role === 'developer' && (
            <>
              <Link
                to="/submit"
                className="bg-blue-600 text-white p-6 rounded-lg hover:bg-blue-700 transition-colors flex items-center space-x-3"
              >
                <Plus className="h-8 w-8" />
                <div>
                  <h3 className="font-semibold text-lg">Submit New App</h3>
                  <p className="text-blue-100 text-sm">Add your educational app to the store</p>
                </div>
              </Link>
              
              <Link
                to="/submissions"
                className="bg-white border border-gray-300 p-6 rounded-lg hover:bg-gray-50 transition-colors flex items-center space-x-3"
              >
                <BookOpen className="h-8 w-8 text-blue-600" />
                <div>
                  <h3 className="font-semibold text-lg">My Submissions</h3>
                  <p className="text-gray-600 text-sm">Track app review status</p>
                </div>
              </Link>
              
              <Link
                to="/developer-analytics"
                className="bg-white border border-gray-300 p-6 rounded-lg hover:bg-gray-50 transition-colors flex items-center space-x-3"
              >
                <BarChart3 className="h-8 w-8 text-green-600" />
                <div>
                  <h3 className="font-semibold text-lg">Analytics</h3>
                  <p className="text-gray-600 text-sm">View detailed performance metrics</p>
                </div>
              </Link>
            </>
          )}

          {user?.role === 'educator' && (
            <>
              <Link
                to="/apps"
                className="bg-blue-600 text-white p-6 rounded-lg hover:bg-blue-700 transition-colors flex items-center space-x-3"
              >
                <BookOpen className="h-8 w-8" />
                <div>
                  <h3 className="font-semibold text-lg">Browse Apps</h3>
                  <p className="text-blue-100 text-sm">Discover new educational tools</p>
                </div>
              </Link>
              
              <Link
                to="/favorites"
                className="bg-white border border-gray-300 p-6 rounded-lg hover:bg-gray-50 transition-colors flex items-center space-x-3"
              >
                <Star className="h-8 w-8 text-yellow-500" />
                <div>
                  <h3 className="font-semibold text-lg">My Favorites</h3>
                  <p className="text-gray-600 text-sm">{favorites.length} saved apps</p>
                </div>
              </Link>
              
              <Link
                to="/downloads"
                className="bg-white border border-gray-300 p-6 rounded-lg hover:bg-gray-50 transition-colors flex items-center space-x-3"
              >
                <Download className="h-8 w-8 text-green-600" />
                <div>
                  <h3 className="font-semibold text-lg">Downloads</h3>
                  <p className="text-gray-600 text-sm">{downloads.length} downloaded apps</p>
                </div>
              </Link>
            </>
          )}
        </div>

        {/* Stats Cards */}
        {stats && (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
            <div className="bg-white p-6 rounded-lg shadow-md">
              <div className="flex items-center">
                <div className="bg-blue-100 p-3 rounded-full">
                  {user?.role === 'developer' ? <BookOpen className="h-6 w-6 text-blue-600" /> : <Download className="h-6 w-6 text-blue-600" />}
                </div>
                <div className="ml-4">
                  <p className="text-sm font-medium text-gray-600">
                    {user?.role === 'developer' ? 'Total Apps' : 'Downloaded Apps'}
                  </p>
                  <p className="text-2xl font-bold text-gray-900">{stats.totalApps}</p>
                </div>
              </div>
            </div>

            <div className="bg-white p-6 rounded-lg shadow-md">
              <div className="flex items-center">
                <div className="bg-green-100 p-3 rounded-full">
                  <Eye className="h-6 w-6 text-green-600" />
                </div>
                <div className="ml-4">
                  <p className="text-sm font-medium text-gray-600">Total Views</p>
                  <p className="text-2xl font-bold text-gray-900">{stats.totalViews.toLocaleString()}</p>
                </div>
              </div>
            </div>

            <div className="bg-white p-6 rounded-lg shadow-md">
              <div className="flex items-center">
                <div className="bg-yellow-100 p-3 rounded-full">
                  <Star className="h-6 w-6 text-yellow-600" />
                </div>
                <div className="ml-4">
                  <p className="text-sm font-medium text-gray-600">
                    {user?.role === 'developer' ? 'Avg Rating' : 'Avg Rating Given'}
                  </p>
                  <p className="text-2xl font-bold text-gray-900">{stats.avgRating.toFixed(1)}</p>
                </div>
              </div>
            </div>

            <div className="bg-white p-6 rounded-lg shadow-md">
              <div className="flex items-center">
                <div className="bg-purple-100 p-3 rounded-full">
                  <TrendingUp className="h-6 w-6 text-purple-600" />
                </div>
                <div className="ml-4">
                  <p className="text-sm font-medium text-gray-600">Conversion Rate</p>
                  <p className="text-2xl font-bold text-gray-900">{stats.conversionRate}%</p>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Recent Activity */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
          {/* Recent Apps (for developers) */}
          {user?.role === 'developer' && recentApps.length > 0 && (
            <div className="bg-white rounded-lg shadow-md p-6">
              <div className="flex items-center justify-between mb-6">
                <h2 className="text-xl font-bold text-gray-900">Recent Apps</h2>
                <Link
                  to="/developer/apps"
                  className="text-blue-600 hover:text-blue-700 flex items-center text-sm"
                >
                  View all <ArrowRight className="h-4 w-4 ml-1" />
                </Link>
              </div>
              
              <div className="space-y-4">
                {recentApps.map((app) => (
                  <div key={app.id} className="flex items-center justify-between p-4 border border-gray-200 rounded-lg">
                    <div className="flex items-center space-x-3">
                      <img
                        src={app.icon || '/placeholder-app-icon.png'}
                        alt={app.title}
                        className="w-10 h-10 rounded-lg object-cover"
                      />
                      <div>
                        <h3 className="font-medium text-gray-900">{app.title}</h3>
                        <p className="text-sm text-gray-600">Created {formatDate(app.created_at)}</p>
                      </div>
                    </div>
                    <div className="flex items-center space-x-3">
                      <span className={`text-xs px-2 py-1 rounded-full ${getStatusColor(app.status)}`}>
                        {app.status}
                      </span>
                      <div className="text-right">
                        <div className="flex items-center space-x-1">
                          <Star className="h-4 w-4 text-yellow-400 fill-current" />
                          <span className="text-sm">{app.rating.toFixed(1)}</span>
                        </div>
                        <p className="text-xs text-gray-600">{app.download_count} downloads</p>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Recent Reviews (for educators) */}
          {user?.role === 'educator' && reviews.length > 0 && (
            <div className="bg-white rounded-lg shadow-md p-6">
              <div className="flex items-center justify-between mb-6">
                <h2 className="text-xl font-bold text-gray-900">Recent Reviews</h2>
                <Link
                  to="/profile/reviews"
                  className="text-blue-600 hover:text-blue-700 flex items-center text-sm"
                >
                  View all <ArrowRight className="h-4 w-4 ml-1" />
                </Link>
              </div>
              
              <div className="space-y-4">
                {reviews.slice(0, 5).map((review) => (
                  <div key={review.id} className="flex items-start space-x-3 p-4 border border-gray-200 rounded-lg">
                    <img
                      src={review.app_icon || '/placeholder-app-icon.png'}
                      alt="App"
                      className="w-10 h-10 rounded-lg object-cover"
                    />
                    <div className="flex-1">
                      <div className="flex items-center space-x-2 mb-1">
                        <h3 className="font-medium text-gray-900">{review.app_title}</h3>
                        <div className="flex items-center space-x-1">
                          {[...Array(5)].map((_, i) => (
                            <Star
                              key={i}
                              className={`h-3 w-3 ${i < review.rating ? 'text-yellow-400 fill-current' : 'text-gray-300'}`}
                            />
                          ))}
                        </div>
                      </div>
                      <p className="text-sm text-gray-600 line-clamp-2">{review.content}</p>
                      <p className="text-xs text-gray-500 mt-1">{formatDate(review.created_at)}</p>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Favorites (for educators) */}
          {user?.role === 'educator' && favorites.length > 0 && (
            <div className="bg-white rounded-lg shadow-md p-6">
              <div className="flex items-center justify-between mb-6">
                <h2 className="text-xl font-bold text-gray-900">Recent Favorites</h2>
                <Link
                  to="/favorites"
                  className="text-blue-600 hover:text-blue-700 flex items-center text-sm"
                >
                  View all <ArrowRight className="h-4 w-4 ml-1" />
                </Link>
              </div>
              
              <div className="space-y-4">
                {favorites.slice(0, 5).map((app) => (
                  <Link
                    key={app.id}
                    to={`/apps/${app.id}`}
                    className="flex items-center space-x-3 p-4 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors"
                  >
                    <img
                      src={app.icon || '/placeholder-app-icon.png'}
                      alt={app.title}
                      className="w-10 h-10 rounded-lg object-cover"
                    />
                    <div className="flex-1">
                      <h3 className="font-medium text-gray-900">{app.title}</h3>
                      <p className="text-sm text-gray-600">{app.category_name}</p>
                      <div className="flex items-center space-x-1 mt-1">
                        <Star className="h-4 w-4 text-yellow-400 fill-current" />
                        <span className="text-sm text-gray-600">{app.rating.toFixed(1)}</span>
                      </div>
                    </div>
                    <p className="text-xs text-gray-500">{formatDate(app.favorited_at)}</p>
                  </Link>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* Account Information */}
        <div className="mt-8 bg-white rounded-lg shadow-md p-6">
          <div className="flex items-center justify-between mb-6">
            <h2 className="text-xl font-bold text-gray-900">Account Information</h2>
            <Link
              to="/profile"
              className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 transition-colors flex items-center space-x-2"
            >
              <Settings className="h-4 w-4" />
              <span>Edit Profile</span>
            </Link>
          </div>
          
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <h3 className="font-semibold text-gray-900 mb-2">Personal Details</h3>
              <div className="space-y-2 text-sm">
                <p><span className="text-gray-600">Name:</span> {user?.name}</p>
                <p><span className="text-gray-600">Email:</span> {user?.email}</p>
                <p><span className="text-gray-600">Role:</span> <span className="capitalize">{user?.role}</span></p>
                {user?.institution && (
                  <p><span className="text-gray-600">Institution:</span> {user.institution}</p>
                )}
              </div>
            </div>
            
            <div>
              <h3 className="font-semibold text-gray-900 mb-2">Account Stats</h3>
              <div className="space-y-2 text-sm">
                <p><span className="text-gray-600">Member since:</span> {user?.createdAt ? formatDate(user.createdAt) : 'N/A'}</p>
                <p><span className="text-gray-600">Last login:</span> Recently</p>
                <p><span className="text-gray-600">Account type:</span> {user?.role === 'developer' ? 'App Developer' : 'Educator'}</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default DashboardPage;