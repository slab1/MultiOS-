import React from 'react';
import { Code, BookOpen, TrendingUp, Bug } from 'lucide-react';

interface NavigationProps {
  currentView: 'browse' | 'modules' | 'performance' | 'debug';
  onViewChange: (view: 'browse' | 'modules' | 'performance' | 'debug') => void;
}

export const Navigation: React.FC<NavigationProps> = ({ currentView, onViewChange }) => {
  const navItems = [
    { id: 'browse' as const, label: 'Code Browser', icon: Code, description: 'Browse and analyze kernel code' },
    { id: 'modules' as const, label: 'Learning Modules', icon: BookOpen, description: 'Educational content and tutorials' },
    { id: 'performance' as const, label: 'Performance Analysis', icon: TrendingUp, description: 'Performance hotspots and optimization' },
    { id: 'debug' as const, label: 'Debug Integration', icon: Bug, description: 'Interactive debugging interface' },
  ];

  return (
    <nav className="bg-white shadow-lg border-b border-gray-200">
      <div className="container mx-auto px-4">
        <div className="flex items-center justify-between h-16">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <h1 className="text-xl font-bold text-gray-900">MultiOS Code Browser</h1>
            </div>
            <div className="ml-10 flex items-baseline space-x-4">
              {navItems.map((item) => {
                const Icon = item.icon;
                const isActive = currentView === item.id;
                
                return (
                  <button
                    key={item.id}
                    onClick={() => onViewChange(item.id)}
                    className={`flex items-center px-3 py-2 rounded-md text-sm font-medium transition-colors duration-200 ${
                      isActive
                        ? 'bg-blue-100 text-blue-700 border border-blue-200'
                        : 'text-gray-600 hover:text-gray-900 hover:bg-gray-100'
                    }`}
                    title={item.description}
                  >
                    <Icon className="w-4 h-4 mr-2" />
                    {item.label}
                  </button>
                );
              })}
            </div>
          </div>
          
          <div className="flex items-center space-x-4">
            <div className="text-sm text-gray-500">
              Educational OS Development Platform
            </div>
            <div className="flex items-center space-x-2">
              <div className="w-2 h-2 bg-green-400 rounded-full animate-pulse"></div>
              <span className="text-sm text-gray-600">Live Analysis</span>
            </div>
          </div>
        </div>
      </div>
    </nav>
  );
};
