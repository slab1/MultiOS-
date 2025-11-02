import React, { useState } from 'react';
import { Link } from 'react-router-dom';
import { 
  FolderOpen, 
  Download, 
  Star, 
  Code, 
  Web, 
  Terminal, 
  Smartphone,
  Database,
  Cloud,
  Shield,
  Search,
  Filter,
  Grid3X3,
  List,
  ExternalLink,
  Clock,
  Users
} from 'lucide-react';

interface Template {
  id: string;
  title: string;
  description: string;
  language: string;
  category: string;
  difficulty: 'Beginner' | 'Intermediate' | 'Advanced';
  downloads: number;
  stars: number;
  lastUpdated: string;
  preview: string;
  tags: string[];
  featured?: boolean;
}

export const Templates: React.FC = () => {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedCategory, setSelectedCategory] = useState('All');
  const [selectedDifficulty, setSelectedDifficulty] = useState('All');
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [sortBy, setSortBy] = useState<'name' | 'downloads' | 'stars' | 'recent'>('recent');

  const templates: Template[] = [
    {
      id: '1',
      title: 'MultiOS Web App',
      description: 'Complete web application template with modern JavaScript, responsive design, and API integration.',
      language: 'JavaScript',
      category: 'Web Development',
      difficulty: 'Intermediate',
      downloads: 2450,
      stars: 89,
      lastUpdated: '2025-01-15',
      preview: 'Modern web app with React components and API calls',
      tags: ['React', 'TypeScript', 'API', 'Responsive'],
      featured: true
    },
    {
      id: '2',
      title: 'Python CLI Tool',
      description: 'Professional command-line interface template with argument parsing, logging, and configuration.',
      language: 'Python',
      category: 'CLI Tools',
      difficulty: 'Beginner',
      downloads: 1890,
      stars: 67,
      lastUpdated: '2025-01-10',
      preview: 'Full-featured CLI with argument parsing and logging',
      tags: ['CLI', 'argparse', 'logging', 'config']
    },
    {
      id: '3',
      title: 'Rust System Library',
      description: 'High-performance system library template with comprehensive testing and documentation.',
      language: 'Rust',
      category: 'System Programming',
      difficulty: 'Advanced',
      downloads: 1340,
      stars: 92,
      lastUpdated: '2025-01-12',
      preview: 'Performance-critical library with async support',
      tags: ['async', 'testing', 'documentation', 'performance']
    },
    {
      id: '4',
      title: 'Mobile App Template',
      description: 'Cross-platform mobile application template with navigation, state management, and offline support.',
      language: 'JavaScript',
      category: 'Mobile Development',
      difficulty: 'Intermediate',
      downloads: 2100,
      stars: 78,
      lastUpdated: '2025-01-14',
      preview: 'Mobile app with navigation and offline capabilities',
      tags: ['mobile', 'navigation', 'offline', 'cross-platform']
    },
    {
      id: '5',
      title: 'Database ORM',
      description: 'Object-Relational Mapping template with migrations, seeding, and query builder.',
      language: 'Python',
      category: 'Database',
      difficulty: 'Intermediate',
      downloads: 1680,
      stars: 85,
      lastUpdated: '2025-01-13',
      preview: 'Full ORM with migrations and query builder',
      tags: ['ORM', 'migrations', 'database', 'query-builder']
    },
    {
      id: '6',
      title: 'REST API Server',
      description: 'Complete REST API server template with authentication, validation, and rate limiting.',
      language: 'Python',
      category: 'Backend',
      difficulty: 'Advanced',
      downloads: 3250,
      stars: 156,
      lastUpdated: '2025-01-11',
      preview: 'Production-ready API with authentication',
      tags: ['REST', 'API', 'authentication', 'validation']
    },
    {
      id: '7',
      title: 'Rust Async Server',
      description: 'High-performance asynchronous server template with WebSocket support and request routing.',
      language: 'Rust',
      category: 'Backend',
      difficulty: 'Advanced',
      downloads: 980,
      stars: 73,
      lastUpdated: '2025-01-09',
      preview: 'Async server with WebSocket and routing',
      tags: ['async', 'WebSocket', 'routing', 'high-performance']
    },
    {
      id: '8',
      title: 'Frontend Dashboard',
      description: 'Modern dashboard template with charts, data visualization, and real-time updates.',
      language: 'JavaScript',
      category: 'Web Development',
      difficulty: 'Intermediate',
      downloads: 1750,
      stars: 91,
      lastUpdated: '2025-01-08',
      preview: 'Dashboard with charts and real-time data',
      tags: ['dashboard', 'charts', 'real-time', 'visualization']
    },
    {
      id: '9',
      title: 'IoT Device Manager',
      description: 'Internet of Things device management template with telemetry and remote control.',
      language: 'Python',
      category: 'IoT',
      difficulty: 'Advanced',
      downloads: 890,
      stars: 64,
      lastUpdated: '2025-01-07',
      preview: 'IoT device management with telemetry',
      tags: ['IoT', 'telemetry', 'remote-control', 'MQTT']
    },
    {
      id: '10',
      title: 'Game Engine Core',
      description: '2D game engine core template with physics, collision detection, and rendering.',
      language: 'Rust',
      category: 'Game Development',
      difficulty: 'Advanced',
      downloads: 1120,
      stars: 87,
      lastUpdated: '2025-01-06',
      preview: '2D game engine with physics and rendering',
      tags: ['game-engine', '2D', 'physics', 'rendering']
    },
    {
      id: '11',
      title: 'DevOps Pipeline',
      description: 'Complete DevOps pipeline template with CI/CD, testing, and deployment automation.',
      language: 'YAML',
      category: 'DevOps',
      difficulty: 'Intermediate',
      downloads: 1450,
      stars: 79,
      lastUpdated: '2025-01-05',
      preview: 'Complete DevOps pipeline with automation',
      tags: ['CI/CD', 'testing', 'deployment', 'automation']
    },
    {
      id: '12',
      title: 'Security Scanner',
      description: 'Security vulnerability scanner template with report generation and compliance checking.',
      language: 'Python',
      category: 'Security',
      difficulty: 'Advanced',
      downloads: 760,
      stars: 58,
      lastUpdated: '2025-01-04',
      preview: 'Security scanner with report generation',
      tags: ['security', 'scanner', 'compliance', 'reporting']
    }
  ];

  const categories = ['All', 'Web Development', 'Backend', 'CLI Tools', 'Mobile Development', 'System Programming', 'Database', 'Game Development', 'DevOps', 'Security', 'IoT'];
  const difficulties = ['All', 'Beginner', 'Intermediate', 'Advanced'];

  const filteredTemplates = templates
    .filter(template => {
      const matchesSearch = template.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
                           template.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
                           template.tags.some(tag => tag.toLowerCase().includes(searchTerm.toLowerCase()));
      
      const matchesCategory = selectedCategory === 'All' || template.category === selectedCategory;
      const matchesDifficulty = selectedDifficulty === 'All' || template.difficulty === selectedDifficulty;
      
      return matchesSearch && matchesCategory && matchesDifficulty;
    })
    .sort((a, b) => {
      switch (sortBy) {
        case 'name':
          return a.title.localeCompare(b.title);
        case 'downloads':
          return b.downloads - a.downloads;
        case 'stars':
          return b.stars - a.stars;
        case 'recent':
          return new Date(b.lastUpdated).getTime() - new Date(a.lastUpdated).getTime();
        default:
          return 0;
      }
    });

  const getDifficultyColor = (difficulty: string) => {
    switch (difficulty) {
      case 'Beginner':
        return 'bg-green-100 text-green-800';
      case 'Intermediate':
        return 'bg-yellow-100 text-yellow-800';
      case 'Advanced':
        return 'bg-red-100 text-red-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  const getCategoryIcon = (category: string) => {
    switch (category) {
      case 'Web Development':
        return Web;
      case 'Backend':
        return Database;
      case 'CLI Tools':
        return Terminal;
      case 'Mobile Development':
        return Smartphone;
      case 'System Programming':
        return Code;
      case 'Security':
        return Shield;
      case 'DevOps':
        return Cloud;
      default:
        return FolderOpen;
    }
  };

  const featuredTemplates = templates.filter(t => t.featured);

  return (
    <div className="min-h-screen bg-slate-50">
      {/* Header */}
      <div className="bg-white border-b border-slate-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <div className="text-center">
            <div className="flex justify-center mb-4">
              <div className="bg-gradient-to-r from-blue-600 to-purple-600 p-3 rounded-xl">
                <FolderOpen className="h-8 w-8 text-white" />
              </div>
            </div>
            <h1 className="text-4xl md:text-5xl font-bold text-slate-900 mb-4">
              Project Templates
            </h1>
            <p className="text-xl text-slate-600 max-w-3xl mx-auto">
              Get started quickly with our collection of production-ready templates and starter kits 
              for MultiOS development across different domains and skill levels.
            </p>
          </div>
        </div>
      </div>

      {/* Featured Templates */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <h2 className="text-2xl font-bold text-slate-900 mb-6">Featured Templates</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {featuredTemplates.map((template) => {
            const Icon = getCategoryIcon(template.category);
            return (
              <div
                key={template.id}
                className="bg-white rounded-xl shadow-lg border border-slate-200 overflow-hidden hover:shadow-xl transition-all duration-300 transform hover:scale-105"
              >
                <div className="p-6">
                  <div className="flex items-center justify-between mb-4">
                    <div className="flex items-center space-x-3">
                      <div className="bg-gradient-to-r from-blue-600 to-purple-600 p-2 rounded-lg">
                        <Icon className="h-5 w-5 text-white" />
                      </div>
                      <span className="text-sm font-medium text-slate-600">{template.language}</span>
                    </div>
                    <span className={`px-2 py-1 rounded-full text-xs font-medium ${getDifficultyColor(template.difficulty)}`}>
                      {template.difficulty}
                    </span>
                  </div>
                  
                  <h3 className="text-xl font-bold text-slate-900 mb-2">{template.title}</h3>
                  <p className="text-slate-600 mb-4 line-clamp-3">{template.description}</p>
                  
                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-4 text-sm text-slate-500">
                      <div className="flex items-center space-x-1">
                        <Download className="h-4 w-4" />
                        <span>{template.downloads.toLocaleString()}</span>
                      </div>
                      <div className="flex items-center space-x-1">
                        <Star className="h-4 w-4 fill-current text-yellow-500" />
                        <span>{template.stars}</span>
                      </div>
                    </div>
                    
                    <button className="bg-gradient-to-r from-blue-600 to-purple-600 text-white px-4 py-2 rounded-lg hover:shadow-lg transition-all duration-300">
                      Use Template
                    </button>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Filters and Search */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="bg-white rounded-xl shadow-lg border border-slate-200 p-6">
          <div className="flex flex-col lg:flex-row gap-4 items-center justify-between">
            {/* Search */}
            <div className="relative flex-1 max-w-md">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-slate-400" />
              <input
                type="text"
                placeholder="Search templates..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="w-full pl-10 pr-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              />
            </div>

            {/* Filters */}
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
                value={selectedDifficulty}
                onChange={(e) => setSelectedDifficulty(e.target.value)}
                className="px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              >
                {difficulties.map(difficulty => (
                  <option key={difficulty} value={difficulty}>{difficulty}</option>
                ))}
              </select>

              <select
                value={sortBy}
                onChange={(e) => setSortBy(e.target.value as any)}
                className="px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              >
                <option value="recent">Most Recent</option>
                <option value="downloads">Most Downloaded</option>
                <option value="stars">Highest Rated</option>
                <option value="name">Name A-Z</option>
              </select>

              {/* View Mode Toggle */}
              <div className="flex items-center border border-slate-300 rounded-lg overflow-hidden">
                <button
                  onClick={() => setViewMode('grid')}
                  className={`p-2 ${viewMode === 'grid' ? 'bg-blue-600 text-white' : 'text-slate-600 hover:bg-slate-100'}`}
                >
                  <Grid3X3 className="h-4 w-4" />
                </button>
                <button
                  onClick={() => setViewMode('list')}
                  className={`p-2 ${viewMode === 'list' ? 'bg-blue-600 text-white' : 'text-slate-600 hover:bg-slate-100'}`}
                >
                  <List className="h-4 w-4" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Templates Grid/List */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pb-16">
        {viewMode === 'grid' ? (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {filteredTemplates.map((template) => {
              const Icon = getCategoryIcon(template.category);
              return (
                <div
                  key={template.id}
                  className="bg-white rounded-xl shadow-lg border border-slate-200 overflow-hidden hover:shadow-xl transition-all duration-300"
                >
                  <div className="p-6">
                    <div className="flex items-center justify-between mb-4">
                      <div className="flex items-center space-x-3">
                        <div className="bg-gradient-to-r from-blue-600 to-purple-600 p-2 rounded-lg">
                          <Icon className="h-5 w-5 text-white" />
                        </div>
                        <span className="text-sm font-medium text-slate-600">{template.language}</span>
                      </div>
                      <span className={`px-2 py-1 rounded-full text-xs font-medium ${getDifficultyColor(template.difficulty)}`}>
                        {template.difficulty}
                      </span>
                    </div>
                    
                    <h3 className="text-xl font-bold text-slate-900 mb-2">{template.title}</h3>
                    <p className="text-slate-600 mb-4 line-clamp-2">{template.description}</p>
                    
                    <div className="flex flex-wrap gap-2 mb-4">
                      {template.tags.slice(0, 3).map((tag, index) => (
                        <span key={index} className="px-2 py-1 bg-slate-100 text-slate-600 rounded-full text-xs">
                          {tag}
                        </span>
                      ))}
                      {template.tags.length > 3 && (
                        <span className="px-2 py-1 bg-slate-100 text-slate-600 rounded-full text-xs">
                          +{template.tags.length - 3} more
                        </span>
                      )}
                    </div>
                    
                    <div className="flex items-center justify-between">
                      <div className="flex items-center space-x-4 text-sm text-slate-500">
                        <div className="flex items-center space-x-1">
                          <Download className="h-4 w-4" />
                          <span>{template.downloads.toLocaleString()}</span>
                        </div>
                        <div className="flex items-center space-x-1">
                          <Star className="h-4 w-4 fill-current text-yellow-500" />
                          <span>{template.stars}</span>
                        </div>
                        <div className="flex items-center space-x-1">
                          <Clock className="h-4 w-4" />
                          <span>{new Date(template.lastUpdated).toLocaleDateString()}</span>
                        </div>
                      </div>
                      
                      <button className="bg-gradient-to-r from-blue-600 to-purple-600 text-white px-4 py-2 rounded-lg hover:shadow-lg transition-all duration-300">
                        Use Template
                      </button>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        ) : (
          <div className="space-y-4">
            {filteredTemplates.map((template) => {
              const Icon = getCategoryIcon(template.category);
              return (
                <div
                  key={template.id}
                  className="bg-white rounded-xl shadow-lg border border-slate-200 p-6 hover:shadow-xl transition-all duration-300"
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-4">
                      <div className="bg-gradient-to-r from-blue-600 to-purple-600 p-3 rounded-lg">
                        <Icon className="h-6 w-6 text-white" />
                      </div>
                      <div>
                        <div className="flex items-center space-x-3 mb-2">
                          <h3 className="text-xl font-bold text-slate-900">{template.title}</h3>
                          <span className="text-sm font-medium text-slate-600">{template.language}</span>
                          <span className={`px-2 py-1 rounded-full text-xs font-medium ${getDifficultyColor(template.difficulty)}`}>
                            {template.difficulty}
                          </span>
                        </div>
                        <p className="text-slate-600 mb-2">{template.description}</p>
                        <div className="flex flex-wrap gap-2">
                          {template.tags.map((tag, index) => (
                            <span key={index} className="px-2 py-1 bg-slate-100 text-slate-600 rounded-full text-xs">
                              {tag}
                            </span>
                          ))}
                        </div>
                      </div>
                    </div>
                    
                    <div className="flex items-center space-x-4">
                      <div className="text-right text-sm text-slate-500">
                        <div className="flex items-center space-x-1 mb-1">
                          <Download className="h-4 w-4" />
                          <span>{template.downloads.toLocaleString()}</span>
                        </div>
                        <div className="flex items-center space-x-1 mb-1">
                          <Star className="h-4 w-4 fill-current text-yellow-500" />
                          <span>{template.stars}</span>
                        </div>
                        <div className="flex items-center space-x-1">
                          <Clock className="h-4 w-4" />
                          <span>{new Date(template.lastUpdated).toLocaleDateString()}</span>
                        </div>
                      </div>
                      
                      <button className="bg-gradient-to-r from-blue-600 to-purple-600 text-white px-6 py-2 rounded-lg hover:shadow-lg transition-all duration-300">
                        Use Template
                      </button>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
};