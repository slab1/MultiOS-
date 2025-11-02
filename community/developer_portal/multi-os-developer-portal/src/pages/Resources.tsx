import React, { useState } from 'react';
import { 
  Library, 
  BookOpen, 
  Video, 
  FileText, 
  ExternalLink, 
  Download,
  Search,
  Filter,
  Star,
  Clock,
  Users,
  Code,
  Globe,
  Zap,
  Target,
  Award,
  TrendingUp,
  Lightbulb,
  Database,
  Shield,
  Cloud,
  Smartphone,
  Monitor
} from 'lucide-react';

interface Resource {
  id: string;
  title: string;
  description: string;
  type: 'documentation' | 'video' | 'article' | 'ebook' | 'course' | 'tool';
  category: string;
  level: 'beginner' | 'intermediate' | 'advanced';
  duration?: string;
  rating?: number;
  downloads?: number;
  url: string;
  tags: string[];
  author: string;
  publishedAt: string;
  featured?: boolean;
  language: string;
}

interface LearningPath {
  id: string;
  title: string;
  description: string;
  duration: string;
  courses: number;
  difficulty: 'Beginner' | 'Intermediate' | 'Advanced';
  completionRate: number;
  prerequisites: string[];
  skills: string[];
  icon: string;
  color: string;
}

export const Resources: React.FC = () => {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedType, setSelectedType] = useState('All');
  const [selectedCategory, setSelectedCategory] = useState('All');
  const [selectedLevel, setSelectedLevel] = useState('All');
  const [activeTab, setActiveTab] = useState<'resources' | 'paths' | 'tools'>('resources');

  const resources: Resource[] = [
    {
      id: '1',
      title: 'MultiOS API Documentation',
      description: 'Comprehensive API reference documentation for MultiOS development, including endpoints, authentication, and examples.',
      type: 'documentation',
      category: 'API Reference',
      level: 'intermediate',
      rating: 4.9,
      downloads: 15000,
      url: 'https://docs.multios.com/api',
      tags: ['API', 'REST', 'Authentication', 'JSON'],
      author: 'MultiOS Team',
      publishedAt: '2025-01-15',
      featured: true,
      language: 'English'
    },
    {
      id: '2',
      title: 'Getting Started with MultiOS',
      description: 'Video series covering the fundamentals of MultiOS development, from installation to your first application.',
      type: 'video',
      category: 'Getting Started',
      level: 'beginner',
      duration: '2h 30m',
      rating: 4.8,
      url: 'https://youtube.com/playlist/multios-getting-started',
      tags: ['setup', 'basics', 'tutorial', 'beginner'],
      author: 'MultiOS Academy',
      publishedAt: '2025-01-10',
      language: 'English'
    },
    {
      id: '3',
      title: 'Advanced Rust Patterns',
      description: 'Deep dive into advanced Rust programming patterns and best practices for MultiOS system development.',
      type: 'article',
      category: 'System Programming',
      level: 'advanced',
      rating: 4.7,
      url: 'https://blog.multios.com/advanced-rust-patterns',
      tags: ['Rust', 'patterns', 'best-practices', 'systems'],
      author: 'Dr. Sarah Chen',
      publishedAt: '2025-01-08',
      language: 'English'
    },
    {
      id: '4',
      title: 'Python for MultiOS Developers',
      description: 'Complete ebook covering Python development specifically for MultiOS environments and applications.',
      type: 'ebook',
      category: 'Programming Languages',
      level: 'intermediate',
      downloads: 8500,
      url: 'https://books.multios.com/python-for-multios',
      tags: ['Python', 'ebook', 'programming', 'development'],
      author: 'Mike Rodriguez',
      publishedAt: '2025-01-05',
      language: 'English'
    },
    {
      id: '5',
      title: 'MultiOS Performance Optimization',
      description: 'Course on optimizing applications for MultiOS platforms, covering profiling, benchmarking, and optimization techniques.',
      type: 'course',
      category: 'Performance',
      level: 'advanced',
      duration: '8h 45m',
      rating: 4.9,
      url: 'https://academy.multios.com/performance-optimization',
      tags: ['performance', 'optimization', 'profiling', 'benchmarking'],
      author: 'Performance Team',
      publishedAt: '2025-01-03',
      language: 'English'
    },
    {
      id: '6',
      title: 'Database Design Patterns',
      description: 'Article series on database design patterns and best practices for MultiOS applications.',
      type: 'article',
      category: 'Database',
      level: 'intermediate',
      rating: 4.6,
      url: 'https://blog.multios.com/database-design-patterns',
      tags: ['database', 'design-patterns', 'SQL', 'NoSQL'],
      author: 'Anna Kowalski',
      publishedAt: '2025-01-01',
      language: 'English'
    },
    {
      id: '7',
      title: 'MultiOS Security Guide',
      description: 'Comprehensive security guide covering authentication, authorization, encryption, and secure coding practices.',
      type: 'documentation',
      category: 'Security',
      level: 'intermediate',
      rating: 4.8,
      downloads: 12000,
      url: 'https://docs.multios.com/security-guide',
      tags: ['security', 'authentication', 'encryption', 'best-practices'],
      author: 'Security Team',
      publishedAt: '2024-12-28',
      featured: true,
      language: 'English'
    },
    {
      id: '8',
      title: 'Real-time Communication Patterns',
      description: 'Video tutorial series on implementing real-time features using WebSockets and other communication protocols.',
      type: 'video',
      category: 'Real-time Systems',
      level: 'advanced',
      duration: '4h 15m',
      rating: 4.7,
      url: 'https://youtube.com/playlist/realtime-communication',
      tags: ['WebSockets', 'real-time', 'communication', 'patterns'],
      author: 'David Liu',
      publishedAt: '2024-12-25',
      language: 'English'
    }
  ];

  const learningPaths: LearningPath[] = [
    {
      id: '1',
      title: 'MultiOS Beginner Path',
      description: 'Complete beginner-friendly learning path covering all fundamentals needed to start building MultiOS applications.',
      duration: '6 weeks',
      courses: 12,
      difficulty: 'Beginner',
      completionRate: 89,
      prerequisites: ['Basic programming knowledge'],
      skills: ['Python Basics', 'JavaScript Fundamentals', 'API Design', 'Version Control'],
      icon: '🌱',
      color: 'from-green-500 to-emerald-500'
    },
    {
      id: '2',
      title: 'System Programming Expert',
      description: 'Advanced path focusing on system-level programming, performance optimization, and low-level development.',
      duration: '12 weeks',
      courses: 20,
      difficulty: 'Advanced',
      completionRate: 67,
      prerequisites: ['C/C++ experience', 'Operating systems knowledge'],
      skills: ['Rust Programming', 'Memory Management', 'Performance Optimization', 'System Design'],
      icon: '⚡',
      color: 'from-red-500 to-pink-500'
    },
    {
      id: '3',
      title: 'Web Development Specialist',
      description: 'Comprehensive path for building modern web applications with MultiOS backend services.',
      duration: '10 weeks',
      courses: 16,
      difficulty: 'Intermediate',
      completionRate: 78,
      prerequisites: ['HTML/CSS basics', 'JavaScript fundamentals'],
      skills: ['React Development', 'Node.js', 'Database Design', 'API Development'],
      icon: '🌐',
      color: 'from-blue-500 to-cyan-500'
    },
    {
      id: '4',
      title: 'Mobile Development Expert',
      description: 'Specialized path for developing cross-platform mobile applications using modern frameworks.',
      duration: '14 weeks',
      courses: 18,
      difficulty: 'Intermediate',
      completionRate: 72,
      prerequisites: ['JavaScript basics', 'Mobile app concepts'],
      skills: ['React Native', 'Flutter', 'Mobile UI/UX', 'App Store Deployment'],
      icon: '📱',
      color: 'from-purple-500 to-indigo-500'
    }
  ];

  const tools = [
    {
      name: 'MultiOS CLI',
      description: 'Command-line interface for project management and development tasks',
      category: 'Development Tools',
      level: 'essential',
      icon: Terminal,
      url: 'https://github.com/multios/cli',
      rating: 4.8
    },
    {
      name: 'Performance Profiler',
      description: 'Advanced profiling tool for analyzing application performance',
      category: 'Performance',
      level: 'advanced',
      icon: Zap,
      url: 'https://github.com/multios/profiler',
      rating: 4.6
    },
    {
      name: 'Database Migration Tool',
      description: 'Schema migration and database management utilities',
      category: 'Database',
      level: 'intermediate',
      icon: Database,
      url: 'https://github.com/multios/db-migrate',
      rating: 4.7
    },
    {
      name: 'Security Scanner',
      description: 'Automated security vulnerability scanning and reporting',
      category: 'Security',
      level: 'intermediate',
      icon: Shield,
      url: 'https://github.com/multios/security-scanner',
      rating: 4.5
    },
    {
      name: 'Cloud Deployment Manager',
      description: 'Streamlined cloud deployment and infrastructure management',
      category: 'DevOps',
      level: 'advanced',
      icon: Cloud,
      url: 'https://github.com/multios/cloud-manager',
      rating: 4.9
    },
    {
      name: 'Mobile Testing Suite',
      description: 'Comprehensive testing framework for mobile applications',
      category: 'Mobile',
      level: 'intermediate',
      icon: Smartphone,
      url: 'https://github.com/multios/mobile-testing',
      rating: 4.4
    }
  ];

  const types = ['All', 'documentation', 'video', 'article', 'ebook', 'course', 'tool'];
  const categories = ['All', 'API Reference', 'Getting Started', 'System Programming', 'Programming Languages', 'Performance', 'Database', 'Security', 'Real-time Systems'];
  const levels = ['All', 'beginner', 'intermediate', 'advanced'];

  const filteredResources = resources.filter(resource => {
    const matchesSearch = resource.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         resource.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         resource.tags.some(tag => tag.toLowerCase().includes(searchTerm.toLowerCase()));
    
    const matchesType = selectedType === 'All' || resource.type === selectedType;
    const matchesCategory = selectedCategory === 'All' || resource.category === selectedCategory;
    const matchesLevel = selectedLevel === 'All' || resource.level === selectedLevel;
    
    return matchesSearch && matchesType && matchesCategory && matchesLevel;
  });

  const featuredResources = resources.filter(r => r.featured);

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'documentation':
        return FileText;
      case 'video':
        return Video;
      case 'article':
        return BookOpen;
      case 'ebook':
        return BookOpen;
      case 'course':
        return Video;
      case 'tool':
        return Code;
      default:
        return FileText;
    }
  };

  const getLevelColor = (level: string) => {
    switch (level) {
      case 'beginner':
        return 'bg-green-100 text-green-800';
      case 'intermediate':
        return 'bg-yellow-100 text-yellow-800';
      case 'advanced':
        return 'bg-red-100 text-red-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  const getLevelBadgeColor = (level: string) => {
    switch (level) {
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

  return (
    <div className="min-h-screen bg-slate-50">
      {/* Header */}
      <div className="bg-white border-b border-slate-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <div className="text-center">
            <div className="flex justify-center mb-4">
              <div className="bg-gradient-to-r from-blue-600 to-purple-600 p-3 rounded-xl">
                <Library className="h-8 w-8 text-white" />
              </div>
            </div>
            <h1 className="text-4xl md:text-5xl font-bold text-slate-900 mb-4">
              Developer Resources
            </h1>
            <p className="text-xl text-slate-600 max-w-3xl mx-auto">
              Comprehensive collection of documentation, tutorials, tools, and learning materials 
              to help you master MultiOS development and build amazing applications.
            </p>
          </div>
        </div>
      </div>

      {/* Resource Stats */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
          <div className="bg-white rounded-xl shadow-lg border border-slate-200 p-6 text-center">
            <div className="bg-gradient-to-r from-blue-600 to-purple-600 p-3 rounded-xl w-fit mx-auto mb-4">
              <FileText className="h-6 w-6 text-white" />
            </div>
            <div className="text-2xl font-bold text-slate-900 mb-2">500+</div>
            <div className="text-slate-600">Documentation Pages</div>
          </div>
          
          <div className="bg-white rounded-xl shadow-lg border border-slate-200 p-6 text-center">
            <div className="bg-gradient-to-r from-green-600 to-emerald-600 p-3 rounded-xl w-fit mx-auto mb-4">
              <Video className="h-6 w-6 text-white" />
            </div>
            <div className="text-2xl font-bold text-slate-900 mb-2">200+</div>
            <div className="text-slate-600">Video Tutorials</div>
          </div>
          
          <div className="bg-white rounded-xl shadow-lg border border-slate-200 p-6 text-center">
            <div className="bg-gradient-to-r from-yellow-600 to-orange-600 p-3 rounded-xl w-fit mx-auto mb-4">
              <Code className="h-6 w-6 text-white" />
            </div>
            <div className="text-2xl font-bold text-slate-900 mb-2">100+</div>
            <div className="text-slate-600">Development Tools</div>
          </div>
          
          <div className="bg-white rounded-xl shadow-lg border border-slate-200 p-6 text-center">
            <div className="bg-gradient-to-r from-purple-600 to-pink-600 p-3 rounded-xl w-fit mx-auto mb-4">
              <BookOpen className="h-6 w-6 text-white" />
            </div>
            <div className="text-2xl font-bold text-slate-900 mb-2">50+</div>
            <div className="text-slate-600">Learning Paths</div>
          </div>
        </div>
      </div>

      {/* Navigation Tabs */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="bg-white rounded-xl shadow-lg border border-slate-200">
          <div className="border-b border-slate-200">
            <nav className="flex space-x-8 px-6">
              {[
                { id: 'resources', label: 'Resources', icon: Library },
                { id: 'paths', label: 'Learning Paths', icon: Target },
                { id: 'tools', label: 'Tools', icon: Code }
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

          {/* Resources Tab */}
          {activeTab === 'resources' && (
            <div className="p-6">
              {/* Filters */}
              <div className="flex flex-col lg:flex-row gap-4 items-center justify-between mb-6">
                <div className="relative flex-1 max-w-md">
                  <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-slate-400" />
                  <input
                    type="text"
                    placeholder="Search resources..."
                    value={searchTerm}
                    onChange={(e) => setSearchTerm(e.target.value)}
                    className="w-full pl-10 pr-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                  />
                </div>

                <div className="flex items-center space-x-4">
                  <select
                    value={selectedType}
                    onChange={(e) => setSelectedType(e.target.value)}
                    className="px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                  >
                    {types.map(type => (
                      <option key={type} value={type}>{type.charAt(0).toUpperCase() + type.slice(1)}</option>
                    ))}
                  </select>

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
                    value={selectedLevel}
                    onChange={(e) => setSelectedLevel(e.target.value)}
                    className="px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                  >
                    {levels.map(level => (
                      <option key={level} value={level}>{level.charAt(0).toUpperCase() + level.slice(1)}</option>
                    ))}
                  </select>
                </div>
              </div>

              {/* Featured Resources */}
              {featuredResources.length > 0 && (
                <div className="mb-8">
                  <h3 className="text-xl font-bold text-slate-900 mb-4 flex items-center space-x-2">
                    <Star className="h-5 w-5 text-yellow-500" />
                    <span>Featured Resources</span>
                  </h3>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    {featuredResources.map((resource) => {
                      const Icon = getTypeIcon(resource.type);
                      return (
                        <div
                          key={resource.id}
                          className="bg-gradient-to-br from-blue-50 to-purple-50 rounded-xl border border-blue-200 p-6 hover:shadow-lg transition-all duration-300"
                        >
                          <div className="flex items-center justify-between mb-4">
                            <div className="flex items-center space-x-3">
                              <div className="bg-gradient-to-r from-blue-600 to-purple-600 p-2 rounded-lg">
                                <Icon className="h-5 w-5 text-white" />
                              </div>
                              <div>
                                <div className="font-semibold text-slate-900">{resource.author}</div>
                                <div className="text-sm text-slate-600">{resource.category}</div>
                              </div>
                            </div>
                            <span className={`px-2 py-1 rounded-full text-xs font-medium ${getLevelColor(resource.level)}`}>
                              {resource.level}
                            </span>
                          </div>
                          
                          <h4 className="text-lg font-bold text-slate-900 mb-2">{resource.title}</h4>
                          <p className="text-slate-600 text-sm mb-4 line-clamp-2">{resource.description}</p>
                          
                          <div className="flex flex-wrap gap-2 mb-4">
                            {resource.tags.slice(0, 4).map((tag, index) => (
                              <span key={index} className="px-2 py-1 bg-white/80 text-slate-600 rounded-full text-xs">
                                {tag}
                              </span>
                            ))}
                          </div>
                          
                          <div className="flex items-center justify-between">
                            <div className="flex items-center space-x-4 text-sm text-slate-500">
                              {resource.duration && (
                                <div className="flex items-center space-x-1">
                                  <Clock className="h-4 w-4" />
                                  <span>{resource.duration}</span>
                                </div>
                              )}
                              {resource.rating && (
                                <div className="flex items-center space-x-1">
                                  <Star className="h-4 w-4 fill-current text-yellow-500" />
                                  <span>{resource.rating}</span>
                                </div>
                              )}
                              {resource.downloads && (
                                <div className="flex items-center space-x-1">
                                  <Download className="h-4 w-4" />
                                  <span>{resource.downloads.toLocaleString()}</span>
                                </div>
                              )}
                            </div>
                            
                            <a
                              href={resource.url}
                              target="_blank"
                              rel="noopener noreferrer"
                              className="flex items-center space-x-1 bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 transition-colors"
                            >
                              <span>View</span>
                              <ExternalLink className="h-4 w-4" />
                            </a>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>
              )}

              {/* All Resources */}
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {filteredResources.map((resource) => {
                  const Icon = getTypeIcon(resource.type);
                  return (
                    <div
                      key={resource.id}
                      className="bg-white rounded-xl shadow-lg border border-slate-200 p-6 hover:shadow-xl transition-all duration-300"
                    >
                      <div className="flex items-center justify-between mb-4">
                        <div className="flex items-center space-x-3">
                          <div className="bg-gradient-to-r from-blue-600 to-purple-600 p-2 rounded-lg">
                            <Icon className="h-5 w-5 text-white" />
                          </div>
                          <div>
                            <div className="font-semibold text-slate-900">{resource.author}</div>
                            <div className="text-sm text-slate-600">{resource.category}</div>
                          </div>
                        </div>
                        <span className={`px-2 py-1 rounded-full text-xs font-medium ${getLevelColor(resource.level)}`}>
                          {resource.level}
                        </span>
                      </div>
                      
                      <h3 className="text-lg font-bold text-slate-900 mb-2">{resource.title}</h3>
                      <p className="text-slate-600 text-sm mb-4 line-clamp-3">{resource.description}</p>
                      
                      <div className="flex flex-wrap gap-2 mb-4">
                        {resource.tags.slice(0, 3).map((tag, index) => (
                          <span key={index} className="px-2 py-1 bg-slate-100 text-slate-600 rounded-full text-xs">
                            {tag}
                          </span>
                        ))}
                        {resource.tags.length > 3 && (
                          <span className="px-2 py-1 bg-slate-100 text-slate-600 rounded-full text-xs">
                            +{resource.tags.length - 3}
                          </span>
                        )}
                      </div>
                      
                      <div className="flex items-center justify-between">
                        <div className="flex items-center space-x-3 text-sm text-slate-500">
                          {resource.duration && (
                            <div className="flex items-center space-x-1">
                              <Clock className="h-4 w-4" />
                              <span>{resource.duration}</span>
                            </div>
                          )}
                          {resource.rating && (
                            <div className="flex items-center space-x-1">
                              <Star className="h-4 w-4 fill-current text-yellow-500" />
                              <span>{resource.rating}</span>
                            </div>
                          )}
                          {resource.downloads && (
                            <div className="flex items-center space-x-1">
                              <Download className="h-4 w-4" />
                              <span>{resource.downloads.toLocaleString()}</span>
                            </div>
                          )}
                        </div>
                        
                        <a
                          href={resource.url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-blue-600 hover:text-blue-700"
                        >
                          <ExternalLink className="h-4 w-4" />
                        </a>
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* Learning Paths Tab */}
          {activeTab === 'paths' && (
            <div className="p-6">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {learningPaths.map((path) => (
                  <div
                    key={path.id}
                    className="bg-white rounded-xl shadow-lg border border-slate-200 overflow-hidden hover:shadow-xl transition-all duration-300"
                  >
                    <div className={`bg-gradient-to-r ${path.color} p-6 text-white`}>
                      <div className="flex items-center space-x-3 mb-4">
                        <span className="text-3xl">{path.icon}</span>
                        <div>
                          <h3 className="text-xl font-bold">{path.title}</h3>
                          <p className="text-white/90">{path.description}</p>
                        </div>
                      </div>
                    </div>
                    
                    <div className="p-6">
                      <div className="flex items-center justify-between mb-4">
                        <span className={`px-2 py-1 rounded-full text-xs font-medium ${getLevelBadgeColor(path.difficulty)}`}>
                          {path.difficulty}
                        </span>
                        <div className="text-sm text-slate-600">
                          {path.duration} • {path.courses} courses
                        </div>
                      </div>
                      
                      <div className="mb-4">
                        <div className="flex items-center justify-between text-sm text-slate-600 mb-2">
                          <span>Completion Rate</span>
                          <span>{path.completionRate}%</span>
                        </div>
                        <div className="w-full bg-slate-200 rounded-full h-2">
                          <div
                            className="bg-gradient-to-r from-blue-600 to-purple-600 h-2 rounded-full"
                            style={{ width: `${path.completionRate}%` }}
                          ></div>
                        </div>
                      </div>
                      
                      <div className="mb-4">
                        <h4 className="text-sm font-semibold text-slate-900 mb-2">Skills you'll learn:</h4>
                        <div className="flex flex-wrap gap-2">
                          {path.skills.map((skill, index) => (
                            <span key={index} className="px-2 py-1 bg-slate-100 text-slate-600 rounded-full text-xs">
                              {skill}
                            </span>
                          ))}
                        </div>
                      </div>
                      
                      <div className="mb-4">
                        <h4 className="text-sm font-semibold text-slate-900 mb-2">Prerequisites:</h4>
                        <ul className="text-sm text-slate-600 space-y-1">
                          {path.prerequisites.map((prereq, index) => (
                            <li key={index} className="flex items-center space-x-2">
                              <span className="w-1 h-1 bg-slate-400 rounded-full"></span>
                              <span>{prereq}</span>
                            </li>
                          ))}
                        </ul>
                      </div>
                      
                      <button className="w-full bg-gradient-to-r from-blue-600 to-purple-600 text-white py-3 rounded-lg hover:shadow-lg transition-all duration-300">
                        Start Learning Path
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Tools Tab */}
          {activeTab === 'tools' && (
            <div className="p-6">
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {tools.map((tool, index) => {
                  const Icon = tool.icon;
                  return (
                    <div
                      key={index}
                      className="bg-white rounded-xl shadow-lg border border-slate-200 p-6 hover:shadow-xl transition-all duration-300"
                    >
                      <div className="flex items-center space-x-3 mb-4">
                        <div className="bg-gradient-to-r from-blue-600 to-purple-600 p-2 rounded-lg">
                          <Icon className="h-5 w-5 text-white" />
                        </div>
                        <div>
                          <h3 className="text-lg font-bold text-slate-900">{tool.name}</h3>
                          <div className="text-sm text-slate-600">{tool.category}</div>
                        </div>
                      </div>
                      
                      <p className="text-slate-600 mb-4">{tool.description}</p>
                      
                      <div className="flex items-center justify-between mb-4">
                        <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                          tool.level === 'essential' ? 'bg-green-100 text-green-800' :
                          tool.level === 'intermediate' ? 'bg-yellow-100 text-yellow-800' :
                          'bg-red-100 text-red-800'
                        }`}>
                          {tool.level}
                        </span>
                        
                        <div className="flex items-center space-x-1">
                          <Star className="h-4 w-4 fill-current text-yellow-500" />
                          <span className="text-sm text-slate-600">{tool.rating}</span>
                        </div>
                      </div>
                      
                      <a
                        href={tool.url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="flex items-center justify-center space-x-2 bg-gradient-to-r from-blue-600 to-purple-600 text-white py-2 rounded-lg hover:shadow-lg transition-all duration-300"
                      >
                        <span>View Tool</span>
                        <ExternalLink className="h-4 w-4" />
                      </a>
                    </div>
                  );
                })}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};