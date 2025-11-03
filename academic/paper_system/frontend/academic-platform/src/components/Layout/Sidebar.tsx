import React from 'react';
import { Link, useLocation } from 'react-router-dom';
import { useAuth } from '../../contexts/AuthContext';
import {
  HomeIcon,
  DocumentTextIcon,
  ChatBubbleLeftRightIcon,
  AcademicCapIcon,
  BookOpenIcon,
  ChartBarIcon,
  UsersIcon,
  CodeBracketSquareIcon,
  MagnifyingGlassIcon,
  Cog6ToothIcon,
  XMarkIcon,
} from '@heroicons/react/24/outline';

interface SidebarProps {
  isOpen: boolean;
  onClose: () => void;
}

const Sidebar: React.FC<SidebarProps> = ({ isOpen, onClose }) => {
  const { user } = useAuth();
  const location = useLocation();

  const navigation = [
    { name: 'Dashboard', href: '/', icon: HomeIcon, current: location.pathname === '/' },
    { name: 'Papers', href: '/papers', icon: DocumentTextIcon, current: location.pathname.startsWith('/papers') },
    { name: 'Reviews', href: '/reviews', icon: ChatBubbleLeftRightIcon, current: location.pathname.startsWith('/reviews') },
    { name: 'Conferences', href: '/conferences', icon: AcademicCapIcon, current: location.pathname.startsWith('/conferences') },
    { name: 'Citations', href: '/citations', icon: BookOpenIcon, current: location.pathname.startsWith('/citations') },
    { name: 'Analytics', href: '/analytics', icon: ChartBarIcon, current: location.pathname.startsWith('/analytics') },
    { name: 'LaTeX Editor', href: '/latex-editor', icon: CodeBracketSquareIcon, current: location.pathname === '/latex-editor' },
    { name: 'Search', href: '/search', icon: MagnifyingGlassIcon, current: location.pathname === '/search' },
  ];

  // Add admin/editor only navigation items
  if (user?.role === 'admin' || user?.role === 'editor') {
    navigation.push(
      { name: 'Users', href: '/users', icon: UsersIcon, current: location.pathname.startsWith('/users') }
    );
  }

  const secondaryNavigation = [
    { name: 'Profile', href: '/profile', icon: UsersIcon },
    { name: 'Settings', href: '/settings', icon: Cog6ToothIcon },
  ];

  const isActive = (href: string) => {
    if (href === '/') {
      return location.pathname === '/';
    }
    return location.pathname.startsWith(href);
  };

  return (
    <>
      {/* Mobile backdrop */}
      {isOpen && (
        <div className="fixed inset-0 z-40 lg:hidden">
          <div className="fixed inset-0 bg-gray-600 bg-opacity-75" onClick={onClose} />
        </div>
      )}

      {/* Sidebar */}
      <div className={`
        fixed inset-y-0 left-0 z-50 w-64 bg-white shadow-lg transform transition-transform duration-300 ease-in-out lg:translate-x-0 lg:static lg:inset-0
        ${isOpen ? 'translate-x-0' : '-translate-x-full'}
      `}>
        <div className="flex h-16 items-center justify-between px-6 lg:hidden">
          <h2 className="text-lg font-semibold text-gray-900">Menu</h2>
          <button
            type="button"
            className="text-gray-400 hover:text-gray-600"
            onClick={onClose}
          >
            <XMarkIcon className="h-6 w-6" />
          </button>
        </div>

        {/* Navigation */}
        <nav className="flex-1 space-y-1 px-4 py-6">
          <div className="space-y-1">
            {navigation.map((item) => {
              const Icon = item.icon;
              return (
                <Link
                  key={item.name}
                  to={item.href}
                  className={`
                    group flex items-center px-2 py-2 text-sm font-medium rounded-md transition-colors duration-200
                    ${isActive(item.href)
                      ? 'bg-indigo-50 text-indigo-700 border-r-2 border-indigo-700'
                      : 'text-gray-700 hover:bg-gray-50 hover:text-gray-900'
                    }
                  `}
                  onClick={() => {
                    // Close mobile sidebar when navigating
                    if (window.innerWidth < 1024) {
                      onClose();
                    }
                  }}
                >
                  <Icon
                    className={`
                      mr-3 h-5 w-5 flex-shrink-0
                      ${isActive(item.href) ? 'text-indigo-700' : 'text-gray-400 group-hover:text-gray-500'}
                    `}
                  />
                  {item.name}
                </Link>
              );
            })}
          </div>

          {/* Divider */}
          <div className="border-t border-gray-200 pt-4 mt-6">
            <h3 className="px-2 text-xs font-semibold text-gray-500 uppercase tracking-wider">
              Account
            </h3>
            <div className="mt-2 space-y-1">
              {secondaryNavigation.map((item) => {
                const Icon = item.icon;
                return (
                  <Link
                    key={item.name}
                    to={item.href}
                    className="
                      group flex items-center px-2 py-2 text-sm font-medium rounded-md text-gray-700 hover:bg-gray-50 hover:text-gray-900
                    "
                    onClick={() => {
                      if (window.innerWidth < 1024) {
                        onClose();
                      }
                    }}
                  >
                    <Icon className="mr-3 h-5 w-5 text-gray-400 group-hover:text-gray-500" />
                    {item.name}
                  </Link>
                );
              })}
            </div>
          </div>

          {/* User role indicator */}
          {user && (
            <div className="border-t border-gray-200 pt-4 mt-6">
              <div className="px-2">
                <div className="text-xs font-semibold text-gray-500 uppercase tracking-wider">
                  Your Role
                </div>
                <div className="mt-1">
                  <span className={`
                    inline-flex items-center px-2 py-1 rounded-full text-xs font-medium
                    ${user.role === 'admin' ? 'bg-red-100 text-red-800' :
                      user.role === 'editor' ? 'bg-purple-100 text-purple-800' :
                      user.role === 'reviewer' ? 'bg-blue-100 text-blue-800' :
                      'bg-green-100 text-green-800'
                    }
                  `}>
                    {user.role.charAt(0).toUpperCase() + user.role.slice(1)}
                  </span>
                </div>
              </div>
            </div>
          )}

          {/* Quick stats */}
          {user && (
            <div className="border-t border-gray-200 pt-4 mt-6">
              <div className="px-2">
                <h3 className="text-xs font-semibold text-gray-500 uppercase tracking-wider">
                  Quick Stats
                </h3>
                <div className="mt-2 space-y-2 text-sm text-gray-600">
                  <div className="flex justify-between">
                    <span>Papers:</span>
                    <span className="font-medium">-</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Reviews:</span>
                    <span className="font-medium">-</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Verified:</span>
                    <span className={`font-medium ${user.isVerified ? 'text-green-600' : 'text-orange-600'}`}>
                      {user.isVerified ? 'Yes' : 'No'}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          )}
        </nav>
      </div>
    </>
  );
};

export default Sidebar;