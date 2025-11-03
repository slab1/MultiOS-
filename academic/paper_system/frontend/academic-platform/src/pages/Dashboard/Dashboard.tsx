import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { useAuth } from '../../contexts/AuthContext';
import { useNotifications } from '../../contexts/NotificationContext';
import { analyticsAPI, papersAPI, reviewsAPI } from '../../services/api';
import LoadingSpinner from '../Common/LoadingSpinner';
import {
  DocumentTextIcon,
  ChatBubbleLeftRightIcon,
  AcademicCapIcon,
  ChartBarIcon,
  ExclamationTriangleIcon,
  ArrowTrendingUpIcon,
  UsersIcon,
  BookOpenIcon,
  PlusIcon,
  EyeIcon,
} from '@heroicons/react/24/outline';

interface DashboardData {
  papers: {
    total: number;
    published: number;
    accepted: number;
    underReview: number;
    recent: number;
  };
  reviews: {
    completed: number;
    active: number;
    pending: number;
  };
  metrics: {
    acceptanceRate: number;
    averageRating: number;
    hIndex: number;
  };
  platform?: {
    papers: {
      total: number;
      published: number;
      underReview: number;
      submitted: number;
    };
    reviews: {
      completed: number;
      active: number;
      overdue: number;
    };
    users: {
      total: number;
      reviewers: number;
      activeReviewers: number;
    };
    conferences: {
      upcoming: number;
      completed: number;
    };
    citations: number;
  };
}

interface RecentActivity {
  id: string;
  type: 'paper_submitted' | 'review_assigned' | 'review_completed' | 'paper_accepted';
  title: string;
  date: string;
  status?: string;
}

const Dashboard: React.FC = () => {
  const { user } = useAuth();
  const { showError } = useNotifications();
  const [dashboardData, setDashboardData] = useState<DashboardData | null>(null);
  const [recentActivity, setRecentActivity] = useState<RecentActivity[]>([]);
  const [loading, setLoading] = useState(true);
  const [timeRange, setTimeRange] = useState('30days');

  useEffect(() => {
    fetchDashboardData();
  }, [timeRange]);

  const fetchDashboardData = async () => {
    try {
      setLoading(true);
      const [dashboardResponse, recentPapersResponse] = await Promise.all([
        analyticsAPI.getDashboard(),
        papersAPI.getMyPapers({ limit: 5 })
      ]);

      setDashboardData(dashboardResponse.data.dashboard);
      setRecentActivity(transformRecentActivity(recentPapersResponse.data.papers));
    } catch (error: any) {
      showError('Failed to load dashboard data');
      console.error('Dashboard error:', error);
    } finally {
      setLoading(false);
    }
  };

  const transformRecentActivity = (papers: any[]): RecentActivity[] => {
    return papers.map(paper => ({
      id: paper.id,
      type: 'paper_submitted',
      title: paper.title,
      date: paper.createdAt,
      status: paper.status
    }));
  };

  const getActivityIcon = (type: string) => {
    switch (type) {
      case 'paper_submitted':
        return <DocumentTextIcon className="h-5 w-5 text-blue-500" />;
      case 'review_assigned':
        return <ChatBubbleLeftRightIcon className="h-5 w-5 text-orange-500" />;
      case 'review_completed':
        return <ChatBubbleLeftRightIcon className="h-5 w-5 text-green-500" />;
      case 'paper_accepted':
        return <AcademicCapIcon className="h-5 w-5 text-green-500" />;
      default:
        return <DocumentTextIcon className="h-5 w-5 text-gray-500" />;
    }
  };

  const getStatusBadge = (status: string) => {
    const statusConfig = {
      draft: { label: 'Draft', className: 'bg-gray-100 text-gray-800' },
      submitted: { label: 'Submitted', className: 'bg-blue-100 text-blue-800' },
      under_review: { label: 'Under Review', className: 'bg-yellow-100 text-yellow-800' },
      accepted: { label: 'Accepted', className: 'bg-green-100 text-green-800' },
      rejected: { label: 'Rejected', className: 'bg-red-100 text-red-800' },
      published: { label: 'Published', className: 'bg-purple-100 text-purple-800' }
    };

    const config = statusConfig[status as keyof typeof statusConfig] || statusConfig.draft;
    
    return (
      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${config.className}`}>
        {config.label}
      </span>
    );
  };

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="md:flex md:items-center md:justify-between">
        <div className="flex-1 min-w-0">
          <h2 className="text-2xl font-bold leading-7 text-gray-900 sm:text-3xl sm:truncate">
            Welcome back, {user?.firstName}!
          </h2>
          <p className="mt-1 text-sm text-gray-500">
            Here's what's happening with your research and reviews
          </p>
        </div>
        <div className="mt-4 flex md:mt-0 md:ml-4">
          <Link
            to="/papers/create"
            className="ml-3 inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
          >
            <PlusIcon className="h-4 w-4 mr-2" />
            New Paper
          </Link>
        </div>
      </div>

      {/* Quick Stats */}
      <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4">
        {/* Papers */}
        <div className="bg-white overflow-hidden shadow rounded-lg">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <DocumentTextIcon className="h-6 w-6 text-gray-400" />
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 truncate">
                    Total Papers
                  </dt>
                  <dd className="text-lg font-medium text-gray-900">
                    {dashboardData?.papers.total || 0}
                  </dd>
                </dl>
              </div>
            </div>
          </div>
          <div className="bg-gray-50 px-5 py-3">
            <div className="text-sm">
              <Link to="/papers" className="font-medium text-indigo-700 hover:text-indigo-900">
                View all papers
              </Link>
            </div>
          </div>
        </div>

        {/* Reviews */}
        <div className="bg-white overflow-hidden shadow rounded-lg">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <ChatBubbleLeftRightIcon className="h-6 w-6 text-gray-400" />
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 truncate">
                    Active Reviews
                  </dt>
                  <dd className="text-lg font-medium text-gray-900">
                    {dashboardData?.reviews.active || 0}
                  </dd>
                </dl>
              </div>
            </div>
          </div>
          <div className="bg-gray-50 px-5 py-3">
            <div className="text-sm">
              <Link to="/reviews" className="font-medium text-indigo-700 hover:text-indigo-900">
                View assignments
              </Link>
            </div>
          </div>
        </div>

        {/* Acceptance Rate */}
        <div className="bg-white overflow-hidden shadow rounded-lg">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <ArrowTrendingUpIcon className="h-6 w-6 text-gray-400" />
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 truncate">
                    Acceptance Rate
                  </dt>
                  <dd className="text-lg font-medium text-gray-900">
                    {dashboardData?.metrics.acceptanceRate || 0}%
                  </dd>
                </dl>
              </div>
            </div>
          </div>
          <div className="bg-gray-50 px-5 py-3">
            <div className="text-sm">
              <span className="text-gray-500">Based on submitted papers</span>
            </div>
          </div>
        </div>

        {/* h-index */}
        <div className="bg-white overflow-hidden shadow rounded-lg">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <ChartBarIcon className="h-6 w-6 text-gray-400" />
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 truncate">
                    h-index
                  </dt>
                  <dd className="text-lg font-medium text-gray-900">
                    {dashboardData?.metrics.hIndex || 0}
                  </dd>
                </dl>
              </div>
            </div>
          </div>
          <div className="bg-gray-50 px-5 py-3">
            <div className="text-sm">
              <Link to="/analytics" className="font-medium text-indigo-700 hover:text-indigo-900">
                View analytics
              </Link>
            </div>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        {/* Recent Activity */}
        <div className="bg-white shadow rounded-lg">
          <div className="px-4 py-5 sm:p-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg leading-6 font-medium text-gray-900">
                Recent Activity
              </h3>
              <Link
                to="/papers"
                className="text-sm font-medium text-indigo-600 hover:text-indigo-500"
              >
                View all
              </Link>
            </div>
            <div className="flow-root">
              <ul className="-my-5 divide-y divide-gray-200">
                {recentActivity.map((activity) => (
                  <li key={activity.id} className="py-4">
                    <div className="flex items-center space-x-4">
                      <div className="flex-shrink-0">
                        {getActivityIcon(activity.type)}
                      </div>
                      <div className="flex-1 min-w-0">
                        <p className="text-sm font-medium text-gray-900 truncate">
                          {activity.title}
                        </p>
                        <p className="text-sm text-gray-500">
                          {new Date(activity.date).toLocaleDateString()}
                        </p>
                      </div>
                      <div className="flex-shrink-0">
                        {activity.status && getStatusBadge(activity.status)}
                      </div>
                    </div>
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </div>

        {/* Quick Actions */}
        <div className="bg-white shadow rounded-lg">
          <div className="px-4 py-5 sm:p-6">
            <h3 className="text-lg leading-6 font-medium text-gray-900 mb-4">
              Quick Actions
            </h3>
            <div className="grid grid-cols-1 gap-4">
              <Link
                to="/papers/create"
                className="relative group bg-white p-6 focus-within:ring-2 focus-within:ring-inset focus-within:ring-indigo-500 hover:bg-gray-50 rounded-lg border-2 border-dashed border-gray-300"
              >
                <div>
                  <span className="rounded-lg inline-flex p-3 bg-indigo-50 text-indigo-700 ring-4 ring-white">
                    <PlusIcon className="h-6 w-6" />
                  </span>
                </div>
                <div className="mt-8">
                  <h3 className="text-lg font-medium">
                    <span className="absolute inset-0" aria-hidden="true" />
                    Create New Paper
                  </h3>
                  <p className="mt-2 text-sm text-gray-500">
                    Submit a new research paper for peer review
                  </p>
                </div>
              </Link>

              <Link
                to="/latex-editor"
                className="relative group bg-white p-6 focus-within:ring-2 focus-within:ring-inset focus-within:ring-indigo-500 hover:bg-gray-50 rounded-lg border-2 border-dashed border-gray-300"
              >
                <div>
                  <span className="rounded-lg inline-flex p-3 bg-indigo-50 text-indigo-700 ring-4 ring-white">
                    <DocumentTextIcon className="h-6 w-6" />
                  </span>
                </div>
                <div className="mt-8">
                  <h3 className="text-lg font-medium">
                    <span className="absolute inset-0" aria-hidden="true" />
                    LaTeX Editor
                  </h3>
                  <p className="mt-2 text-sm text-gray-500">
                    Write and format your paper using LaTeX
                  </p>
                </div>
              </Link>

              <Link
                to="/conferences"
                className="relative group bg-white p-6 focus-within:ring-2 focus-within:ring-inset focus-within:ring-indigo-500 hover:bg-gray-50 rounded-lg border-2 border-dashed border-gray-300"
              >
                <div>
                  <span className="rounded-lg inline-flex p-3 bg-indigo-50 text-indigo-700 ring-4 ring-white">
                    <AcademicCapIcon className="h-6 w-6" />
                  </span>
                </div>
                <div className="mt-8">
                  <h3 className="text-lg font-medium">
                    <span className="absolute inset-0" aria-hidden="true" />
                    Browse Conferences
                  </h3>
                  <p className="mt-2 text-sm text-gray-500">
                    Find upcoming conferences and workshops
                  </p>
                </div>
              </Link>
            </div>
          </div>
        </div>
      </div>

      {/* Platform Stats (for Editors/Admins) */}
      {user?.role === 'editor' || user?.role === 'admin' ? (
        <div className="bg-white shadow rounded-lg">
          <div className="px-4 py-5 sm:p-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg leading-6 font-medium text-gray-900">
                Platform Overview
              </h3>
              <Link
                to="/analytics"
                className="text-sm font-medium text-indigo-600 hover:text-indigo-500"
              >
                View detailed analytics
              </Link>
            </div>
            
            {dashboardData?.platform && (
              <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4">
                <div className="bg-gray-50 rounded-lg p-4">
                  <div className="flex items-center">
                    <DocumentTextIcon className="h-8 w-8 text-gray-400" />
                    <div className="ml-3">
                      <p className="text-sm font-medium text-gray-500">Total Papers</p>
                      <p className="text-2xl font-semibold text-gray-900">
                        {dashboardData.platform.papers.total}
                      </p>
                    </div>
                  </div>
                </div>
                
                <div className="bg-gray-50 rounded-lg p-4">
                  <div className="flex items-center">
                    <UsersIcon className="h-8 w-8 text-gray-400" />
                    <div className="ml-3">
                      <p className="text-sm font-medium text-gray-500">Active Users</p>
                      <p className="text-2xl font-semibold text-gray-900">
                        {dashboardData.platform.users.total}
                      </p>
                    </div>
                  </div>
                </div>
                
                <div className="bg-gray-50 rounded-lg p-4">
                  <div className="flex items-center">
                    <ChatBubbleLeftRightIcon className="h-8 w-8 text-gray-400" />
                    <div className="ml-3">
                      <p className="text-sm font-medium text-gray-500">Active Reviews</p>
                      <p className="text-2xl font-semibold text-gray-900">
                        {dashboardData.platform.reviews.active}
                      </p>
                    </div>
                  </div>
                </div>
                
                <div className="bg-gray-50 rounded-lg p-4">
                  <div className="flex items-center">
                    <ExclamationTriangleIcon className="h-8 w-8 text-gray-400" />
                    <div className="ml-3">
                      <p className="text-sm font-medium text-gray-500">Overdue Reviews</p>
                      <p className="text-2xl font-semibold text-gray-900">
                        {dashboardData.platform.reviews.overdue}
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      ) : null}
    </div>
  );
};

export default Dashboard;