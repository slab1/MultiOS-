import React, { useState, useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { toast } from 'react-hot-toast';
import {
  User,
  Mail,
  Building,
  Key,
  Save,
  Eye,
  EyeOff,
  Camera,
  FileText,
  Users,
  MessageCircle,
  Award,
  Calendar
} from 'lucide-react';

import LoadingSpinner from '../../components/Common/LoadingSpinner';
import { useAuth } from '../../contexts/AuthContext';
import { apiService } from '../../services/api';

const profileSchema = z.object({
  firstName: z.string().min(2, 'First name must be at least 2 characters'),
  lastName: z.string().min(2, 'Last name must be at least 2 characters'),
  email: z.string().email('Invalid email address'),
  affiliation: z.string().min(2, 'Affiliation is required'),
  bio: z.string().optional(),
  orcidId: z.string().optional(),
  researchInterests: z.string().optional(),
  website: z.string().url('Invalid website URL').optional().or(z.literal(''))
});

const passwordSchema = z.object({
  currentPassword: z.string().min(1, 'Current password is required'),
  newPassword: z.string().min(8, 'New password must be at least 8 characters'),
  confirmPassword: z.string()
}).refine((data) => data.newPassword === data.confirmPassword, {
  message: "Passwords don't match",
  path: ["confirmPassword"]
});

type ProfileFormData = z.infer<typeof profileSchema>;
type PasswordFormData = z.infer<typeof passwordSchema>;

export default function UserProfile() {
  const { user } = useAuth();
  const [activeTab, setActiveTab] = useState('profile');
  const [isLoading, setIsLoading] = useState(false);
  const [stats, setStats] = useState({
    papersCount: 0,
    reviewsCount: 0,
    citationsCount: 0
  });

  const profileForm = useForm<ProfileFormData>({
    resolver: zodResolver(profileSchema)
  });

  const passwordForm = useForm<PasswordFormData>({
    resolver: zodResolver(passwordSchema)
  });

  useEffect(() => {
    if (user) {
      profileForm.reset({
        firstName: user.firstName || '',
        lastName: user.lastName || '',
        email: user.email || '',
        affiliation: user.affiliation || '',
        bio: user.bio || '',
        orcidId: user.orcidId || '',
        researchInterests: user.researchInterests || '',
        website: user.website || ''
      });
    }
    fetchUserStats();
  }, [user]);

  const fetchUserStats = async () => {
    try {
      const [papersRes, reviewsRes] = await Promise.all([
        apiService.get('/papers', { params: { author: user?._id, limit: 0 } }),
        apiService.get('/reviews', { params: { reviewer: user?._id, limit: 0 } })
      ]);
      
      setStats({
        papersCount: papersRes.data.total || 0,
        reviewsCount: reviewsRes.data.total || 0,
        citationsCount: 0 // Will be calculated from analytics
      });
    } catch (error) {
      console.error('Failed to fetch user stats:', error);
    }
  };

  const onProfileSubmit = async (data: ProfileFormData) => {
    setIsLoading(true);
    try {
      await apiService.put('/users/profile', data);
      toast.success('Profile updated successfully!');
    } catch (error: any) {
      toast.error(error.message || 'Failed to update profile');
    } finally {
      setIsLoading(false);
    }
  };

  const onPasswordSubmit = async (data: PasswordFormData) => {
    setIsLoading(true);
    try {
      await apiService.post('/auth/change-password', {
        currentPassword: data.currentPassword,
        newPassword: data.newPassword
      });
      toast.success('Password changed successfully!');
      passwordForm.reset();
    } catch (error: any) {
      toast.error(error.message || 'Failed to change password');
    } finally {
      setIsLoading(false);
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric'
    });
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-white shadow rounded-lg">
        <div className="px-6 py-8">
          <div className="flex items-center space-x-6">
            <div className="relative">
              <div className="h-24 w-24 rounded-full bg-indigo-100 flex items-center justify-center">
                <User className="h-12 w-12 text-indigo-600" />
              </div>
              <button className="absolute bottom-0 right-0 h-8 w-8 rounded-full bg-indigo-600 flex items-center justify-center text-white hover:bg-indigo-700">
                <Camera className="h-4 w-4" />
              </button>
            </div>
            <div className="flex-1">
              <h1 className="text-2xl font-semibold text-gray-900">
                {user?.firstName} {user?.lastName}
              </h1>
              <p className="text-gray-600">{user?.email}</p>
              <p className="text-sm text-gray-500">{user?.affiliation}</p>
              <div className="mt-4 flex items-center space-x-6">
                <div className="flex items-center text-sm text-gray-600">
                  <FileText className="h-4 w-4 mr-1" />
                  <span>{stats.papersCount} Papers</span>
                </div>
                <div className="flex items-center text-sm text-gray-600">
                  <MessageCircle className="h-4 w-4 mr-1" />
                  <span>{stats.reviewsCount} Reviews</span>
                </div>
                <div className="flex items-center text-sm text-gray-600">
                  <Calendar className="h-4 w-4 mr-1" />
                  <span>Joined {user?.createdAt ? formatDate(user.createdAt) : 'N/A'}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="bg-white shadow rounded-lg">
        <div className="border-b border-gray-200">
          <nav className="-mb-px flex space-x-8 px-6">
            <button
              onClick={() => setActiveTab('profile')}
              className={`py-4 px-1 border-b-2 font-medium text-sm ${
                activeTab === 'profile'
                  ? 'border-indigo-500 text-indigo-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              Profile
            </button>
            <button
              onClick={() => setActiveTab('password')}
              className={`py-4 px-1 border-b-2 font-medium text-sm ${
                activeTab === 'password'
                  ? 'border-indigo-500 text-indigo-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              Security
            </button>
            <button
              onClick={() => setActiveTab('activity')}
              className={`py-4 px-1 border-b-2 font-medium text-sm ${
                activeTab === 'activity'
                  ? 'border-indigo-500 text-indigo-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              Activity
            </button>
          </nav>
        </div>

        <div className="p-6">
          {/* Profile Tab */}
          {activeTab === 'profile' && (
            <form onSubmit={profileForm.handleSubmit(onProfileSubmit)} className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div>
                  <label htmlFor="firstName" className="block text-sm font-medium text-gray-700">
                    First Name *
                  </label>
                  <input
                    {...profileForm.register('firstName')}
                    type="text"
                    className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                  />
                  {profileForm.formState.errors.firstName && (
                    <p className="mt-1 text-sm text-red-600">
                      {profileForm.formState.errors.firstName.message}
                    </p>
                  )}
                </div>

                <div>
                  <label htmlFor="lastName" className="block text-sm font-medium text-gray-700">
                    Last Name *
                  </label>
                  <input
                    {...profileForm.register('lastName')}
                    type="text"
                    className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                  />
                  {profileForm.formState.errors.lastName && (
                    <p className="mt-1 text-sm text-red-600">
                      {profileForm.formState.errors.lastName.message}
                    </p>
                  )}
                </div>

                <div>
                  <label htmlFor="email" className="block text-sm font-medium text-gray-700">
                    Email Address *
                  </label>
                  <input
                    {...profileForm.register('email')}
                    type="email"
                    className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                  />
                  {profileForm.formState.errors.email && (
                    <p className="mt-1 text-sm text-red-600">
                      {profileForm.formState.errors.email.message}
                    </p>
                  )}
                </div>

                <div>
                  <label htmlFor="affiliation" className="block text-sm font-medium text-gray-700">
                    Affiliation *
                  </label>
                  <input
                    {...profileForm.register('affiliation')}
                    type="text"
                    className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                  />
                  {profileForm.formState.errors.affiliation && (
                    <p className="mt-1 text-sm text-red-600">
                      {profileForm.formState.errors.affiliation.message}
                    </p>
                  )}
                </div>

                <div>
                  <label htmlFor="orcidId" className="block text-sm font-medium text-gray-700">
                    ORCID ID
                  </label>
                  <input
                    {...profileForm.register('orcidId')}
                    type="text"
                    placeholder="0000-0000-0000-0000"
                    className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                  />
                </div>

                <div>
                  <label htmlFor="website" className="block text-sm font-medium text-gray-700">
                    Website
                  </label>
                  <input
                    {...profileForm.register('website')}
                    type="url"
                    placeholder="https://example.com"
                    className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                  />
                  {profileForm.formState.errors.website && (
                    <p className="mt-1 text-sm text-red-600">
                      {profileForm.formState.errors.website.message}
                    </p>
                  )}
                </div>
              </div>

              <div>
                <label htmlFor="bio" className="block text-sm font-medium text-gray-700">
                  Bio
                </label>
                <textarea
                  {...profileForm.register('bio')}
                  rows={4}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                  placeholder="Tell us about yourself..."
                />
              </div>

              <div>
                <label htmlFor="researchInterests" className="block text-sm font-medium text-gray-700">
                  Research Interests
                </label>
                <textarea
                  {...profileForm.register('researchInterests')}
                  rows={3}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                  placeholder="What are your main research interests?"
                />
              </div>

              <div className="flex justify-end">
                <button
                  type="submit"
                  disabled={isLoading}
                  className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50"
                >
                  {isLoading ? (
                    <LoadingSpinner size="sm" />
                  ) : (
                    <>
                      <Save className="h-4 w-4 mr-2" />
                      Save Changes
                    </>
                  )}
                </button>
              </div>
            </form>
          )}

          {/* Password Tab */}
          {activeTab === 'password' && (
            <form onSubmit={passwordForm.handleSubmit(onPasswordSubmit)} className="space-y-6">
              <div className="max-w-md">
                <div>
                  <label htmlFor="currentPassword" className="block text-sm font-medium text-gray-700">
                    Current Password *
                  </label>
                  <input
                    {...passwordForm.register('currentPassword')}
                    type="password"
                    className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                  />
                  {passwordForm.formState.errors.currentPassword && (
                    <p className="mt-1 text-sm text-red-600">
                      {passwordForm.formState.errors.currentPassword.message}
                    </p>
                  )}
                </div>

                <div className="mt-4">
                  <label htmlFor="newPassword" className="block text-sm font-medium text-gray-700">
                    New Password *
                  </label>
                  <input
                    {...passwordForm.register('newPassword')}
                    type="password"
                    className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                  />
                  {passwordForm.formState.errors.newPassword && (
                    <p className="mt-1 text-sm text-red-600">
                      {passwordForm.formState.errors.newPassword.message}
                    </p>
                  )}
                </div>

                <div className="mt-4">
                  <label htmlFor="confirmPassword" className="block text-sm font-medium text-gray-700">
                    Confirm New Password *
                  </label>
                  <input
                    {...passwordForm.register('confirmPassword')}
                    type="password"
                    className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                  />
                  {passwordForm.formState.errors.confirmPassword && (
                    <p className="mt-1 text-sm text-red-600">
                      {passwordForm.formState.errors.confirmPassword.message}
                    </p>
                  )}
                </div>

                <div className="mt-6">
                  <button
                    type="submit"
                    disabled={isLoading}
                    className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50"
                  >
                    {isLoading ? (
                      <LoadingSpinner size="sm" />
                    ) : (
                      <>
                        <Key className="h-4 w-4 mr-2" />
                        Change Password
                      </>
                    )}
                  </button>
                </div>
              </div>
            </form>
          )}

          {/* Activity Tab */}
          {activeTab === 'activity' && (
            <div className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div className="bg-blue-50 p-6 rounded-lg">
                  <div className="flex items-center">
                    <FileText className="h-8 w-8 text-blue-600" />
                    <div className="ml-4">
                      <h3 className="text-lg font-semibold text-blue-900">{stats.papersCount}</h3>
                      <p className="text-blue-700">Papers Created</p>
                    </div>
                  </div>
                </div>
                <div className="bg-green-50 p-6 rounded-lg">
                  <div className="flex items-center">
                    <MessageCircle className="h-8 w-8 text-green-600" />
                    <div className="ml-4">
                      <h3 className="text-lg font-semibold text-green-900">{stats.reviewsCount}</h3>
                      <p className="text-green-700">Reviews Submitted</p>
                    </div>
                  </div>
                </div>
                <div className="bg-purple-50 p-6 rounded-lg">
                  <div className="flex items-center">
                    <Award className="h-8 w-8 text-purple-600" />
                    <div className="ml-4">
                      <h3 className="text-lg font-semibold text-purple-900">{stats.citationsCount}</h3>
                      <p className="text-purple-700">Total Citations</p>
                    </div>
                  </div>
                </div>
              </div>

              <div className="bg-gray-50 p-6 rounded-lg">
                <h3 className="text-lg font-semibold text-gray-900 mb-4">Recent Activity</h3>
                <div className="space-y-4">
                  <div className="flex items-center space-x-3 text-sm">
                    <div className="h-2 w-2 bg-blue-600 rounded-full"></div>
                    <span className="text-gray-600">Joined the platform</span>
                    <span className="text-gray-400">{user?.createdAt ? formatDate(user.createdAt) : 'N/A'}</span>
                  </div>
                  <div className="text-sm text-gray-500 italic">
                    Activity tracking will be available once you start creating papers and submitting reviews.
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}