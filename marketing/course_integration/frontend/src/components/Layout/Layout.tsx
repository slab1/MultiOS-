import React, { useState } from 'react';
import { Outlet, useLocation } from 'react-router-dom';
import { useAuth } from '../../contexts/AuthContext';
import Sidebar from './Sidebar';
import Header from './Header';

const Layout: React.FC = () => {
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const location = useLocation();
  const { state: authState } = useAuth();

  const pageTitle = getPageTitle(location.pathname);

  return (
    <div className="h-screen flex overflow-hidden bg-gray-100">
      {/* Sidebar */}
      <Sidebar
        isOpen={sidebarOpen}
        onClose={() => setSidebarOpen(false)}
        currentPath={location.pathname}
        userRole={authState.user?.role}
      />

      {/* Main content */}
      <div className="flex flex-col flex-1 overflow-hidden">
        {/* Header */}
        <Header
          onMenuClick={() => setSidebarOpen(true)}
          title={pageTitle}
          user={authState.user}
        />

        {/* Main content area */}
        <main className="flex-1 relative overflow-y-auto focus:outline-none">
          <div className="py-6">
            <div className="max-w-7xl mx-auto px-4 sm:px-6 md:px-8">
              <Outlet />
            </div>
          </div>
        </main>
      </div>
    </div>
  );
};

function getPageTitle(pathname: string): string {
  const routeTitles: Record<string, string> = {
    '/dashboard': 'Dashboard',
    '/courses': 'Courses',
    '/lms': 'LMS Integrations',
    '/assignments': 'Assignments',
    '/students': 'Students',
    '/analytics': 'Analytics',
    '/content': 'Content Library',
    '/collaboration': 'Collaboration',
    '/profile': 'Profile',
    '/settings': 'Settings',
  };

  // Handle dynamic routes
  if (pathname.startsWith('/courses/create')) return 'Create Course';
  if (pathname.match(/^\/courses\/[^/]+\/edit$/)) return 'Edit Course';
  if (pathname.match(/^\/courses\/[^/]+$/)) return 'Course Details';
  if (pathname.match(/^\/lms\/[^/]+$/)) return 'LMS Integration Details';
  if (pathname.startsWith('/lms/create')) return 'Create LMS Integration';
  if (pathname.match(/^\/assignments\/[^/]+$/)) return 'Assignment Details';
  if (pathname.startsWith('/assignments/create')) return 'Create Assignment';

  return routeTitles[pathname] || 'MultiOS Course Platform';
}

export default Layout;