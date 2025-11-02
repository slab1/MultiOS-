import React, { useState } from 'react';
import { Link } from 'react-router-dom';
import { 
  Users, 
  Heart, 
  MessageCircle, 
  Share2, 
  Star, 
  Eye, 
  Code, 
  GitBranch,
  Calendar,
  TrendingUp,
  Filter,
  Search,
  Plus,
  Award,
  Zap,
  Target,
  BookOpen,
  ExternalLink
} from 'lucide-react';

interface Project {
  id: string;
  title: string;
  description: string;
  author: {
    name: string;
    avatar: string;
    level: string;
  };
  language: string;
  category: string;
  tags: string[];
  likes: number;
  views: number;
  comments: number;
  createdAt: string;
  updatedAt: string;
  featured: boolean;
  status: 'active' | 'completed' | 'archived';
  demoUrl?: string;
  githubUrl?: string;
  preview: string;
}

interface Achievement {
  id: string;
  title: string;
  description: string;
  icon: string;
  rarity: 'common' | 'rare' | 'epic' | 'legendary';
  earned: boolean;
}

export const Community: React.FC = () => {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedCategory, setSelectedCategory] = useState('All');
  const [selectedLanguage, setSelectedLanguage] = useState('All');
  const [sortBy, setSortBy] = useState<'recent' | 'popular' | 'trending'>('recent');
  const [activeTab, setActiveTab] = useState<'projects' | 'discussions' | 'achievements'>('projects');

  const projects: Project[] = [
    {
      id: '1',
      title: 'MultiOS Task Manager',
      description: 'A sophisticated task management application built with Python and React, featuring real-time collaboration, project tracking, and team management capabilities.',
      author: {
        name: 'Alex Chen',
        avatar: '👨‍💻',
        level: 'Senior Developer'
      },
      language: 'Python',
      category: 'Productivity',
      tags: ['React', 'Python', 'Real-time', 'Collaboration'],
      likes: 124,
      views: 2340,
      comments: 18,
      createdAt: '2025-01-10',
      updatedAt: '2025-01-15',
      featured: true,
      status: 'active',
      demoUrl: 'https://demo.multios.app/task-manager',
      githubUrl: 'https://github.com/alexchen/multios-task-manager',
      preview: 'A comprehensive task management solution'
    },
    {
      id: '2',
      title: 'Rust System Monitor',
      description: 'High-performance system monitoring tool written in Rust, providing real-time metrics, alerting, and visualization for MultiOS environments.',
      author: {
        name: 'Sarah Kim',
        avatar: '👩‍🚀',
        level: 'System Architect'
      },
      language: 'Rust',
      category: 'System Tools',
      tags: ['Rust', 'Performance', 'Monitoring', 'Visualization'],
      likes: 89,
      views: 1567,
      comments: 12,
      createdAt: '2025-01-08',
      updatedAt: '2025-01-14',
      featured: true,
      status: 'active',
      githubUrl: 'https://github.com/sarahkim/rust-system-monitor',
      preview: 'Real-time system performance monitoring'
    },
    {
      id: '3',
      title: 'JavaScript Game Engine',
      description: 'Lightweight 2D game engine built with JavaScript, featuring physics simulation, sprite management, and MultiOS-optimized rendering.',
      author: {
        name: 'Mike Rodriguez',
        avatar: '🎮',
        level: 'Game Developer'
      },
      language: 'JavaScript',
      category: 'Game Development',
      tags: ['JavaScript', 'Game Engine', 'Physics', 'Canvas'],
      likes: 156,
      views: 3890,
      comments: 31,
      createdAt: '2025-01-05',
      updatedAt: '2025-01-12',
      featured: false,
      status: 'active',
      demoUrl: 'https://demo.multios.app/game-engine',
      preview: '2D game engine with physics simulation'
    },
    {
      id: '4',
      title: 'Python Data Visualization Suite',
      description: 'Comprehensive data visualization library for Python, supporting multiple chart types, interactive features, and MultiOS-specific optimizations.',
      author: {
        name: 'Emily Watson',
        avatar: '📊',
        level: 'Data Scientist'
      },
      language: 'Python',
      category: 'Data Science',
      tags: ['Python', 'Data Viz', 'Charts', 'Interactive'],
      likes: 203,
      views: 4567,
      comments: 45,
      createdAt: '2025-01-03',
      updatedAt: '2025-01-11',
      featured: true,
      status: 'active',
      githubUrl: 'https://github.com/emilywatson/python-data-viz',
      preview: 'Interactive data visualization toolkit'
    },
    {
      id: '5',
      title: 'MultiOS CLI Framework',
      description: 'Command-line interface framework for building modern CLI applications in multiple programming languages with consistent design patterns.',
      author: {
        name: 'David Liu',
        avatar: '⚡',
        level: 'Framework Developer'
      },
      language: 'TypeScript',
      category: 'Development Tools',
      tags: ['TypeScript', 'CLI', 'Framework', 'Cross-platform'],
      likes: 78,
      views: 1234,
      comments: 9,
      createdAt: '2025-01-07',
      updatedAt: '2025-01-09',
      featured: false,
      status: 'active',
      githubUrl: 'https://github.com/davidliu/multios-cli-framework',
      preview: 'Modern CLI application framework'
    },
    {
      id: '6',
      title: 'Rust WebAssembly Compiler',
      description: 'Experimental WebAssembly compiler written in Rust, targeting MultiOS platforms with advanced optimization techniques.',
      author: {
        name: 'Anna Kowalski',
        avatar: '🔬',
        level: 'Compiler Engineer'
      },
      language: 'Rust',
      category: 'Compiler Tools',
      tags: ['Rust', 'WebAssembly', 'Compiler', 'Optimization'],
      likes: 167,
      views: 2789,
      comments: 22,
      createdAt: '2025-01-01',
      updatedAt: '2025-01-10',
      featured: false,
      status: 'active',
      preview: 'Experimental WASM compiler for MultiOS'
    }
  ];

  const achievements: Achievement[] = [
    {
      id: '1',
      title: 'First Project',
      description: 'Published your first MultiOS project',
      icon: '🚀',
      rarity: 'common',
      earned: true
    },
    {
      id: '2',
      title: 'Code Master',
      description: 'Published 10 projects',
      icon: '💎',
      rarity: 'rare',
      earned: true
    },
    {
      id: '3',
      title: 'Community Leader',
      description: 'Received 1000+ likes across projects',
      icon: '👑',
      rarity: 'epic',
      earned: false
    },
    {
      id: '4',
      title: 'Innovation Pioneer',
      description: 'Created a featured project',
      icon: '🏆',
      rarity: 'legendary',
      earned: true
    },
    {
      id: '5',
      title: 'Helper',
      description: 'Posted 50+ helpful comments',
      icon: '🤝',
      rarity: 'rare',
      earned: false
    },
    {
      id: '6',
      title: 'Tutorial Creator',
      description: 'Created 5 tutorials with 100+ completions',
      icon: '📚',
      rarity: 'epic',
      earned: true
    }
  ];

  const categories = ['All', 'Productivity', 'System Tools', 'Game Development', 'Data Science', 'Development Tools', 'Compiler Tools'];
  const languages = ['All', 'Python', 'JavaScript', 'TypeScript', 'Rust', 'Go', 'C++'];

  const filteredProjects = projects
    .filter(project => {
      const matchesSearch = project.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
                           project.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
                           project.tags.some(tag => tag.toLowerCase().includes(searchTerm.toLowerCase()));
      
      const matchesCategory = selectedCategory === 'All' || project.category === selectedCategory;
      const matchesLanguage = selectedLanguage === 'All' || project.language === selectedLanguage;
      
      return matchesSearch && matchesCategory && matchesLanguage;
    })
    .sort((a, b) => {
      switch (sortBy) {
        case 'popular':
          return b.likes - a.likes;
        case 'trending':
          return b.views - a.views;
        case 'recent':
        default:
          return new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime();
      }
    });

  const featuredProjects = projects.filter(p => p.featured);

  const getRarityColor = (rarity: string) => {
    switch (rarity) {
      case 'common':
        return 'border-slate-300 bg-slate-50';
      case 'rare':
        return 'border-blue-300 bg-blue-50';
      case 'epic':
        return 'border-purple-300 bg-purple-50';
      case 'legendary':
        return 'border-yellow-300 bg-yellow-50';
      default:
        return 'border-slate-300 bg-slate-50';
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active':
        return 'bg-green-100 text-green-800';
      case 'completed':
        return 'bg-blue-100 text-blue-800';
      case 'archived':
        return 'bg-gray-100 text-gray-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  return (
    <div className="min-h-screen bg-slate-50">
      {/* Header */}
      <div className="bg-white border-b border-slate-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <div className="text-center">
            <div className="flex justify-center mb-4">
              <div className="bg-gradient-to-r from-blue-600 to-purple-600 p-3 rounded-xl">
                <Users className="h-8 w-8 text-white" />
              </div>
            </div>
            <h1 className="text-4xl md:text-5xl font-bold text-slate-900 mb-4">
              Developer Community
            </h1>
            <p className="text-xl text-slate-600 max-w-3xl mx-auto">
              Discover amazing projects, connect with fellow developers, share your work, 
              and grow together in our vibrant MultiOS developer community.
            </p>
          </div>
        </div>
      </div>

      {/* Community Stats */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
          <div className="bg-white rounded-xl shadow-lg border border-slate-200 p-6 text-center">
            <div className="bg-gradient-to-r from-blue-600 to-purple-600 p-3 rounded-xl w-fit mx-auto mb-4">
              <Code className="h-6 w-6 text-white" />
            </div>
            <div className="text-2xl font-bold text-slate-900 mb-2">2,500+</div>
            <div className="text-slate-600">Community Projects</div>
          </div>
          
          <div className="bg-white rounded-xl shadow-lg border border-slate-200 p-6 text-center">
            <div className="bg-gradient-to-r from-green-600 to-emerald-600 p-3 rounded-xl w-fit mx-auto mb-4">
              <Users className="h-6 w-6 text-white" />
            </div>
            <div className="text-2xl font-bold text-slate-900 mb-2">8,000+</div>
            <div className="text-slate-600">Active Developers</div>
          </div>
          
          <div className="bg-white rounded-xl shadow-lg border border-slate-200 p-6 text-center">
            <div className="bg-gradient-to-r from-yellow-600 to-orange-600 p-3 rounded-xl w-fit mx-auto mb-4">
              <Heart className="h-6 w-6 text-white" />
            </div>
            <div className="text-2xl font-bold text-slate-900 mb-2">50,000+</div>
            <div className="text-slate-600">Project Likes</div>
          </div>
          
          <div className="bg-white rounded-xl shadow-lg border border-slate-200 p-6 text-center">
            <div className="bg-gradient-to-r from-purple-600 to-pink-600 p-3 rounded-xl w-fit mx-auto mb-4">
              <MessageCircle className="h-6 w-6 text-white" />
            </div>
            <div className="text-2xl font-bold text-slate-900 mb-2">12,000+</div>
            <div className="text-slate-600">Discussions</div>
          </div>
        </div>
      </div>

      {/* Navigation Tabs */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="bg-white rounded-xl shadow-lg border border-slate-200">
          <div className="border-b border-slate-200">
            <nav className="flex space-x-8 px-6">
              {[
                { id: 'projects', label: 'Projects', icon: Code },
                { id: 'discussions', label: 'Discussions', icon: MessageCircle },
                { id: 'achievements', label: 'Achievements', icon: Award }
              ].map((tab) => {
                const Icon = tab.icon;
                return (
                  <button
                    key={tab.id}
                    onClick={() => setActiveTab(tab.id as any)}
                    className={`flex items-center space-x-2 py-4 border-b-2 font-medium text-sm ${
                      activeTab === tab.id
                        ? 'border-blue-600 text-blue-600'
                        : 'border-transparent text-slate-500 hover:text-slate-700'
                    }`}
                  >
                    <Icon className="h-4 w-4" />
                    <span>{tab.label}</span>
                  </button>
                );
              })}
            </nav>
          </div>

          {/* Projects Tab */}
          {activeTab === 'projects' && (
            <div className="p-6">
              {/* Filters */}
              <div className="flex flex-col lg:flex-row gap-4 items-center justify-between mb-6">
                <div className="relative flex-1 max-w-md">
                  <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-slate-400" />
                  <input
                    type="text"
                    placeholder="Search projects..."
                    value={searchTerm}
                    onChange={(e) => setSearchTerm(e.target.value)}
                    className="w-full pl-10 pr-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                  />
                </div>

                <div className="flex items-center space-x-4">
                  <select
                    value={selectedCategory}
                    onChange={(e) => setSelectedCategory(e.target.value)}
                    className="px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                  >
                    {categories.map(category => (
                      <option key={category} value={category}>{category}</option>
                    ))}
                  </select>

                  <select
                    value={selectedLanguage}
                    onChange={(e) => setSelectedLanguage(e.target.value)}
                    className="px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                  >
                    {languages.map(language => (
                      <option key={language} value={language}>{language}</option>
                    ))}
                  </select>

                  <select
                    value={sortBy}
                    onChange={(e) => setSortBy(e.target.value as any)}
                    className="px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                  >
                    <option value="recent">Most Recent</option>
                    <option value="popular">Most Popular</option>
                    <option value="trending">Most Viewed</option>
                  </select>

                  <button className="bg-gradient-to-r from-blue-600 to-purple-600 text-white px-4 py-2 rounded-lg flex items-center space-x-2">
                    <Plus className="h-4 w-4" />
                    <span>Submit Project</span>
                  </button>
                </div>
              </div>

              {/* Featured Projects */}
              {featuredProjects.length > 0 && (
                <div className="mb-8">
                  <h3 className="text-xl font-bold text-slate-900 mb-4 flex items-center space-x-2">
                    <Star className="h-5 w-5 text-yellow-500" />
                    <span>Featured Projects</span>
                  </h3>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    {featuredProjects.slice(0, 2).map((project) => (
                      <div
                        key={project.id}
                        className="bg-gradient-to-br from-blue-50 to-purple-50 rounded-xl border border-blue-200 p-6 hover:shadow-lg transition-all duration-300"
                      >
                        <div className="flex items-center justify-between mb-4">
                          <div className="flex items-center space-x-3">
                            <div className="text-2xl">{project.author.avatar}</div>
                            <div>
                              <div className="font-semibold text-slate-900">{project.author.name}</div>
                              <div className="text-sm text-slate-600">{project.author.level}</div>
                            </div>
                          </div>
                          <span className={`px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(project.status)}`}>
                            {project.status}
                          </span>
                        </div>
                        
                        <h4 className="text-xl font-bold text-slate-900 mb-2">{project.title}</h4>
                        <p className="text-slate-600 mb-4 line-clamp-2">{project.description}</p>
                        
                        <div className="flex flex-wrap gap-2 mb-4">
                          {project.tags.slice(0, 4).map((tag, index) => (
                            <span key={index} className="px-2 py-1 bg-white/80 text-slate-600 rounded-full text-xs">
                              {tag}
                            </span>
                          ))}
                        </div>
                        
                        <div className="flex items-center justify-between">
                          <div className="flex items-center space-x-4 text-sm text-slate-500">
                            <div className="flex items-center space-x-1">
                              <Heart className="h-4 w-4" />
                              <span>{project.likes}</span>
                            </div>
                            <div className="flex items-center space-x-1">
                              <Eye className="h-4 w-4" />
                              <span>{project.views}</span>
                            </div>
                            <div className="flex items-center space-x-1">
                              <MessageCircle className="h-4 w-4" />
                              <span>{project.comments}</span>
                            </div>
                          </div>
                          
                          <div className="flex items-center space-x-2">
                            {project.demoUrl && (
                              <a
                                href={project.demoUrl}
                                target="_blank"
                                rel="noopener noreferrer"
                                className="text-blue-600 hover:text-blue-700"
                              >
                                <ExternalLink className="h-4 w-4" />
                              </a>
                            )}
                            {project.githubUrl && (
                              <a
                                href={project.githubUrl}
                                target="_blank"
                                rel="noopener noreferrer"
                                className="text-slate-600 hover:text-slate-700"
                              >
                                <GitBranch className="h-4 w-4" />
                              </a>
                            )}
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* All Projects */}
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {filteredProjects.map((project) => (
                  <div
                    key={project.id}
                    className="bg-white rounded-xl shadow-lg border border-slate-200 overflow-hidden hover:shadow-xl transition-all duration-300"
                  >
                    <div className="p-6">
                      <div className="flex items-center justify-between mb-4">
                        <div className="flex items-center space-x-3">
                          <div className="text-xl">{project.author.avatar}</div>
                          <div>
                            <div className="font-semibold text-slate-900">{project.author.name}</div>
                            <div className="text-sm text-slate-600">{project.author.level}</div>
                          </div>
                        </div>
                        <span className={`px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(project.status)}`}>
                          {project.status}
                        </span>
                      </div>
                      
                      <h3 className="text-lg font-bold text-slate-900 mb-2">{project.title}</h3>
                      <p className="text-slate-600 text-sm mb-4 line-clamp-3">{project.description}</p>
                      
                      <div className="flex flex-wrap gap-2 mb-4">
                        {project.tags.slice(0, 3).map((tag, index) => (
                          <span key={index} className="px-2 py-1 bg-slate-100 text-slate-600 rounded-full text-xs">
                            {tag}
                          </span>
                        ))}
                        {project.tags.length > 3 && (
                          <span className="px-2 py-1 bg-slate-100 text-slate-600 rounded-full text-xs">
                            +{project.tags.length - 3}
                          </span>
                        )}
                      </div>
                      
                      <div className="flex items-center justify-between">
                        <div className="flex items-center space-x-3 text-sm text-slate-500">
                          <div className="flex items-center space-x-1">
                            <Heart className="h-4 w-4" />
                            <span>{project.likes}</span>
                          </div>
                          <div className="flex items-center space-x-1">
                            <Eye className="h-4 w-4" />
                            <span>{project.views}</span>
                          </div>
                          <div className="flex items-center space-x-1">
                            <MessageCircle className="h-4 w-4" />
                            <span>{project.comments}</span>
                          </div>
                        </div>
                        
                        <div className="flex items-center space-x-2">
                          {project.demoUrl && (
                            <a
                              href={project.demoUrl}
                              target="_blank"
                              rel="noopener noreferrer"
                              className="text-blue-600 hover:text-blue-700"
                            >
                              <ExternalLink className="h-4 w-4" />
                            </a>
                          )}
                          {project.githubUrl && (
                            <a
                              href={project.githubUrl}
                              target="_blank"
                              rel="noopener noreferrer"
                              className="text-slate-600 hover:text-slate-700"
                            >
                              <GitBranch className="h-4 w-4" />
                            </a>
                          )}
                          <button className="text-slate-600 hover:text-slate-700">
                            <Share2 className="h-4 w-4" />
                          </button>
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Achievements Tab */}
          {activeTab === 'achievements' && (
            <div className="p-6">
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {achievements.map((achievement) => (
                  <div
                    key={achievement.id}
                    className={`rounded-xl border-2 p-6 transition-all duration-300 ${
                      achievement.earned
                        ? getRarityColor(achievement.rarity) + ' transform hover:scale-105'
                        : 'border-slate-200 bg-slate-50 opacity-60'
                    }`}
                  >
                    <div className="text-center">
                      <div className="text-4xl mb-4">{achievement.icon}</div>
                      <h3 className="text-lg font-bold text-slate-900 mb-2">{achievement.title}</h3>
                      <p className="text-slate-600 text-sm mb-4">{achievement.description}</p>
                      
                      <div className="flex items-center justify-center space-x-2">
                        <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                          achievement.earned ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-600'
                        }`}>
                          {achievement.rarity}
                        </span>
                        <span className={`text-xs font-medium ${
                          achievement.earned ? 'text-green-600' : 'text-gray-500'
                        }`}>
                          {achievement.earned ? 'Earned' : 'Locked'}
                        </span>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};