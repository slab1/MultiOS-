import React from 'react';
import { Link, useLocation } from 'react-router-dom';
import { useAuth } from '../../contexts/AuthContext';
import {
  HomeIcon,
  AcademicCapIcon,
  CloudIcon,
  ClipboardDocumentListIcon,
  UsersIcon,
  ChartBarIcon,
  BookOpenIcon,
  ChatBubbleLeftRightIcon,
  CogIcon,
  UserIcon,
  XMarkIcon,
} from '@heroicons/react/24/outline';

interface SidebarProps {
  isOpen: boolean;
  onClose: () => void;
  currentPath: string;
  userRole?: string;
}

interface NavigationItem {
  name: string;
  href: string;
  icon: React.ComponentType<{ className?: string }>;
  allowedRoles: string[];
  badge?: string;
}

const navigation: NavigationItem[] = [
  {
    name: 'Dashboard',
    href: '/dashboard',
    icon: HomeIcon,
    allowedRoles: ['student', 'instructor', 'administrator', 'super_admin'],
  },
  {
    name: 'Courses',
    href: '/courses',
    icon: AcademicCapIcon,
    allowedRoles: ['student', 'instructor', 'administrator', 'super_admin'],
  },
  {
    name: 'LMS Integrations',
    href: '/lms',
    icon: CloudIcon,
    allowedRoles: ['instructor', 'administrator', 'super_admin'],
  },
  {
    name: 'Assignments',
    href: '/assignments',
    icon: ClipboardDocumentListIcon,
    allowedRoles: ['student', 'instructor', 'administrator', 'super_admin'],
  },
  {
    name: 'Students',
    href: '/students',
    icon: UsersIcon,
    allowedRoles: ['instructor', 'administrator', 'super_admin'],
  },
  {
    name: 'Analytics',
    href: '/analytics',
    icon: ChartBarIcon,
    allowedRoles: ['instructor', 'administrator', 'super_admin'],
  },
  {
    name: 'Content Library',
    href: '/content',
    icon: BookOpenIcon,
    allowedRoles: ['student', 'instructor', 'administrator', 'super_admin'],
  },
  {
    name: 'Collaboration',
    href: '/collaboration',
    icon: ChatBubbleLeftRightIcon,
    allowedRoles: ['student', 'instructor', 'administrator', 'super_admin'],
  },
];

const secondaryNavigation: NavigationItem[] = [
  {
    name: 'Profile',
    href: '/profile',
    icon: UserIcon,
    allowedRoles: ['student', 'instructor', 'administrator', 'super_admin'],
  },
  {
    name: 'Settings',
    href: '/settings',
    icon: CogIcon,
    allowedRoles: ['administrator', 'super_admin'],
  },
];

const Sidebar: React.FC<SidebarProps> = ({ isOpen, onClose, currentPath, userRole }) => {
  const location = useLocation();
  const { user } = useAuth();

  const hasPermission = (roles: string[]): boolean => {
    if (!userRole) return false;
    return roles.includes(userRole);
  };

  const isActive = (href: string): boolean => {
    if (href === '/dashboard') {
      return location.pathname === href;
    }
    return location.pathname.startsWith(href);
  };

  return (
    <>
      {/* Mobile sidebar overlay */}
      {isOpen && (
        <div className="fixed inset-0 flex z-40 md:hidden">
          <div className="fixed inset-0 bg-gray-600 bg-opacity-75" onClick={onClose} />
          <div className="relative flex-1 flex flex-col max-w-xs w-full bg-white">
            <div className="absolute top-0 right-0 -mr-12 pt-2">
              <button
                className="ml-1 flex items-center justify-center h-10 w-10 rounded-full focus:outline-none focus:ring-2 focus:ring-inset focus:ring-white"
                onClick={onClose}
              >
                <XMarkIcon className="h-6 w-6 text-white" />
              </button>
            </div>
            <SidebarContent 
              navigation={navigation}
              secondaryNavigation={secondaryNavigation}
              currentPath={currentPath}
              userRole={userRole}
              onItemClick={onClose}
              hasPermission={hasPermission}
              isActive={isActive}
            />
          </div>
        </div>
      )}

      {/* Desktop sidebar */}
      <div className="hidden md:flex md:flex-shrink-0">
        <div className="flex flex-col w-64">
          <SidebarContent
            navigation={navigation}
            secondaryNavigation={secondaryNavigation}
            currentPath={currentPath}
            userRole={userRole}
            hasPermission={hasPermission}
            isActive={isActive}
          />
        </div>
      </div>
    </>
  );
};

interface SidebarContentProps {
  navigation: NavigationItem[];
  secondaryNavigation: NavigationItem[];
  currentPath: string;
  userRole?: string;
  onItemClick?: () => void;
  hasPermission: (roles: string[]) => boolean;
  isActive: (href: string) => boolean;
}

const SidebarContent: React.FC<SidebarContentProps> = ({
  navigation,
  secondaryNavigation,
  userRole,
  hasPermission,
  isActive,
  onItemClick
}) => {
  return (
    <div className="flex flex-col h-full bg-white border-r border-gray-200">
      {/* Logo */}
      <div className="flex items-center h-16 flex-shrink-0 px-4 bg-indigo-600">
        <div className="flex items-center">
          <div className="flex-shrink-0">
            <div className="h-8 w-8 bg-white rounded-lg flex items-center justify-center">
              <span className="text-indigo-600 font-bold text-lg">M</span>
            </div>
          </div>
          <div className="ml-3">
            <p className="text-white font-semibold text-lg">MultiOS</p>
            <p className="text-indigo-200 text-sm">Course Platform</p>
          </div>
        </div>
      </div>

      {/* Navigation */}
      <div className="flex-1 flex flex-col overflow-y-auto">
        <nav className="flex-1 px-2 py-4 space-y-1">
          {navigation.map((item) => {
            if (!hasPermission(item.allowedRoles)) {
              return null;
            }

            return (
              <Link
                key={item.name}
                to={item.href}
                onClick={onItemClick}
                className={`
                  group flex items-center px-2 py-2 text-sm font-medium rounded-md transition-colors
                  ${isActive(item.href)
                    ? 'bg-indigo-100 text-indigo-900'
                    : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900'
                  }
                `}
              >
                <item.icon
                  className={`
                    mr-3 flex-shrink-0 h-6 w-6
                    ${isActive(item.href) ? 'text-indigo-500' : 'text-gray-400 group-hover:text-gray-500'}
                  `}
                />
                {item.name}
                {item.badge && (
                  <span className="ml-auto inline-block py-0.5 px-2 text-xs rounded-full bg-indigo-100 text-indigo-800">
                    {item.badge}
                  </span>
                )}
              </Link>
            );
          })}
        </nav>

        {/* Secondary navigation */}
        <div className="flex-shrink-0 border-t border-gray-200 p-4">
          <nav className="space-y-1">
            {secondaryNavigation.map((item) => {
              if (!hasPermission(item.allowedRoles)) {
                return null;
              }

              return (
                <Link
                  key={item.name}
                  to={item.href}
                  onClick={onItemClick}
                  className={`
                    group flex items-center px-2 py-2 text-sm font-medium rounded-md transition-colors
                    ${isActive(item.href)
                      ? 'bg-indigo-100 text-indigo-900'
                      : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900'
                    }
                  `}
                >
                  <item.icon
                    className={`
                      mr-3 flex-shrink-0 h-6 w-6
                      ${isActive(item.href) ? 'text-indigo-500' : 'text-gray-400 group-hover:text-gray-500'}
                    `}
                  />
                  {item.name}
                </Link>
              );
            })}
          </nav>
        </div>
      </div>
    </div>
  );
};

export default Sidebar;