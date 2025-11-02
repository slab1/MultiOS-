import React from 'react';
import { Link, useLocation } from 'react-router-dom';
import { 
  Code, 
  BookOpen, 
  Users, 
  FolderOpen, 
  Home, 
  Library,
  Play,
  Github,
  Menu,
  X
} from 'lucide-react';
import { useState } from 'react';

interface LayoutProps {
  children: React.ReactNode;
}

export const Layout: React.FC<LayoutProps> = ({ children }) => {
  const location = useLocation();
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);

  const navigation = [
    { name: 'Home', href: '/', icon: Home },
    { name: 'Code Editor', href: '/editor', icon: Code },
    { name: 'Templates', href: '/templates', icon: FolderOpen },
    { name: 'Tutorials', href: '/tutorials', icon: BookOpen },
    { name: 'Community', href: '/community', icon: Users },
    { name: 'Resources', href: '/resources', icon: Library },
  ];

  const isActive = (path: string) => {
    if (path === '/' && location.pathname === '/') return true;
    if (path !== '/' && location.pathname.startsWith(path)) return true;
    return false;
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 to-blue-50">
      {/* Header */}
      <header className="bg-white shadow-lg border-b border-slate-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            {/* Logo */}
            <Link to="/" className="flex items-center space-x-2">
              <div className="bg-gradient-to-r from-blue-600 to-purple-600 p-2 rounded-lg">
                <Play className="h-6 w-6 text-white" />
              </div>
              <span className="text-xl font-bold bg-gradient-to-r from-blue-600 to-purple-600 bg-clip-text text-transparent">
                MultiOS Dev Portal
              </span>
            </Link>

            {/* Desktop Navigation */}
            <nav className="hidden md:flex space-x-1">
              {navigation.map((item) => {
                const Icon = item.icon;
                return (
                  <Link
                    key={item.name}
                    to={item.href}
                    className={`flex items-center space-x-2 px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 ${
                      isActive(item.href)
                        ? 'bg-blue-100 text-blue-700 shadow-sm'
                        : 'text-slate-600 hover:text-blue-600 hover:bg-slate-100'
                    }`}
                  >
                    <Icon className="h-4 w-4" />
                    <span>{item.name}</span>
                  </Link>
                );
              })}
            </nav>

            {/* GitHub Link */}
            <div className="hidden md:flex items-center space-x-4">
              <a
                href="https://github.com"
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center space-x-2 px-4 py-2 bg-slate-800 text-white rounded-lg hover:bg-slate-700 transition-colors"
              >
                <Github className="h-4 w-4" />
                <span>GitHub</span>
              </a>
            </div>

            {/* Mobile menu button */}
            <button
              onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
              className="md:hidden p-2 rounded-lg text-slate-600 hover:bg-slate-100"
            >
              {isMobileMenuOpen ? (
                <X className="h-6 w-6" />
              ) : (
                <Menu className="h-6 w-6" />
              )}
            </button>
          </div>

          {/* Mobile Navigation */}
          {isMobileMenuOpen && (
            <div className="md:hidden py-4 border-t border-slate-200">
              <div className="space-y-2">
                {navigation.map((item) => {
                  const Icon = item.icon;
                  return (
                    <Link
                      key={item.name}
                      to={item.href}
                      onClick={() => setIsMobileMenuOpen(false)}
                      className={`flex items-center space-x-3 px-4 py-3 rounded-lg text-sm font-medium transition-all duration-200 ${
                        isActive(item.href)
                          ? 'bg-blue-100 text-blue-700'
                          : 'text-slate-600 hover:text-blue-600 hover:bg-slate-100'
                      }`}
                    >
                      <Icon className="h-5 w-5" />
                      <span>{item.name}</span>
                    </Link>
                  );
                })}
                <a
                  href="https://github.com"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center space-x-3 px-4 py-3 bg-slate-800 text-white rounded-lg hover:bg-slate-700 transition-colors"
                >
                  <Github className="h-5 w-5" />
                  <span>GitHub</span>
                </a>
              </div>
            </div>
          )}
        </div>
      </header>

      {/* Main Content */}
      <main className="flex-1">
        {children}
      </main>

      {/* Footer */}
      <footer className="bg-slate-900 text-white">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
            <div>
              <div className="flex items-center space-x-2 mb-4">
                <div className="bg-gradient-to-r from-blue-600 to-purple-600 p-2 rounded-lg">
                  <Play className="h-5 w-5 text-white" />
                </div>
                <span className="text-lg font-bold">MultiOS Dev Portal</span>
              </div>
              <p className="text-slate-400 text-sm">
                A comprehensive platform for developers to learn, code, and collaborate on MultiOS projects.
              </p>
            </div>
            
            <div>
              <h3 className="text-sm font-semibold uppercase tracking-wider mb-4">Platform</h3>
              <ul className="space-y-2">
                <li><Link to="/editor" className="text-slate-400 hover:text-white text-sm">Code Editor</Link></li>
                <li><Link to="/templates" className="text-slate-400 hover:text-white text-sm">Templates</Link></li>
                <li><Link to="/tutorials" className="text-slate-400 hover:text-white text-sm">Tutorials</Link></li>
                <li><Link to="/community" className="text-slate-400 hover:text-white text-sm">Community</Link></li>
              </ul>
            </div>

            <div>
              <h3 className="text-sm font-semibold uppercase tracking-wider mb-4">Resources</h3>
              <ul className="space-y-2">
                <li><Link to="/resources" className="text-slate-400 hover:text-white text-sm">Documentation</Link></li>
                <li><a href="#" className="text-slate-400 hover:text-white text-sm">API Reference</a></li>
                <li><a href="#" className="text-slate-400 hover:text-white text-sm">Best Practices</a></li>
                <li><a href="#" className="text-slate-400 hover:text-white text-sm">Examples</a></li>
              </ul>
            </div>

            <div>
              <h3 className="text-sm font-semibold uppercase tracking-wider mb-4">Community</h3>
              <ul className="space-y-2">
                <li><a href="#" className="text-slate-400 hover:text-white text-sm">Discord</a></li>
                <li><a href="#" className="text-slate-400 hover:text-white text-sm">Forum</a></li>
                <li><a href="#" className="text-slate-400 hover:text-white text-sm">GitHub</a></li>
                <li><a href="#" className="text-slate-400 hover:text-white text-sm">Contribute</a></li>
              </ul>
            </div>
          </div>
          
          <div className="border-t border-slate-800 mt-8 pt-8 text-center">
            <p className="text-slate-400 text-sm">
              © 2025 MultiOS Developer Portal. Built with ❤️ for the developer community.
            </p>
          </div>
        </div>
      </footer>
    </div>
  );
};