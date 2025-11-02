import React, { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import { 
  ArrowLeft, 
  Heart, 
  MessageCircle, 
  Share2, 
  Star, 
  Eye, 
  GitBranch,
  ExternalLink,
  Download,
  User,
  Calendar,
  Code,
  Play,
  Users,
  Award,
  Tag,
  Globe,
  Monitor,
  Smartphone,
  Zap
} from 'lucide-react';

interface Project {
  id: string;
  title: string;
  description: string;
  fullDescription: string;
  author: {
    name: string;
    avatar: string;
    level: string;
    bio: string;
    projectsCount: number;
    followers: number;
  };
  language: string;
  category: string;
  tags: string[];
  likes: number;
  views: number;
  comments: number;
  createdAt: string;
  updatedAt: string;
  status: 'active' | 'completed' | 'archived';
  demoUrl?: string;
  githubUrl?: string;
  websiteUrl?: string;
  technologies: string[];
  features: string[];
  screenshots: string[];
  license: string;
  contributors?: {
    name: string;
    avatar: string;
    contribution: string;
  }[];
  documentation?: {
    title: string;
    url: string;
  }[];
}

export const ProjectViewer: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const [project, setProject] = useState<Project | null>(null);
  const [activeTab, setActiveTab] = useState<'overview' | 'code' | 'screenshots' | 'discussions'>('overview');
  const [isLiked, setIsLiked] = useState(false);
  const [likedCount, setLikedCount] = useState(0);

  useEffect(() => {
    // Mock data - in a real app, this would fetch from an API
    const mockProject: Project = {
      id: id || '1',
      title: 'MultiOS Task Manager',
      description: 'A sophisticated task management application built with Python and React, featuring real-time collaboration, project tracking, and team management capabilities.',
      fullDescription: `MultiOS Task Manager is a comprehensive project management solution designed specifically for MultiOS environments. Built with modern web technologies including React, Python Flask, and real-time WebSocket communication, this application provides teams with powerful tools to manage their projects efficiently.

## Key Features

- **Real-time Collaboration**: Multiple team members can work on the same projects simultaneously with live updates
- **Advanced Project Tracking**: Visual kanban boards, Gantt charts, and detailed progress reporting
- **Team Management**: Role-based permissions, team notifications, and activity feeds
- **Time Tracking**: Built-in timer functionality with detailed reporting
- **Integration Ready**: RESTful API with comprehensive documentation for third-party integrations

## Technical Architecture

The application follows a modern microservices architecture with:
- Frontend: React 18 with TypeScript and Tailwind CSS
- Backend: Python Flask with SQLAlchemy ORM
- Real-time: Socket.io for live updates
- Database: PostgreSQL with Redis caching
- Authentication: JWT-based with refresh tokens

## Performance Optimizations

- Lazy loading for improved initial load times
- Virtual scrolling for large project lists
- Optimized database queries with proper indexing
- CDN integration for static assets
- Compression and minification for production builds

This project serves as an excellent example of building scalable, maintainable applications for the MultiOS ecosystem.`,
      author: {
        name: 'Alex Chen',
        avatar: '👨‍💻',
        level: 'Senior Full-Stack Developer',
        bio: 'Passionate about building scalable web applications and mentoring junior developers. 8+ years of experience in full-stack development.',
        projectsCount: 15,
        followers: 234
      },
      language: 'Python',
      category: 'Productivity',
      tags: ['React', 'Python', 'Real-time', 'Collaboration', 'Project Management', 'WebSocket'],
      likes: 124,
      views: 2340,
      comments: 18,
      createdAt: '2025-01-10',
      updatedAt: '2025-01-15',
      status: 'active',
      demoUrl: 'https://demo.multios.app/task-manager',
      githubUrl: 'https://github.com/alexchen/multios-task-manager',
      technologies: ['React', 'TypeScript', 'Python', 'Flask', 'PostgreSQL', 'Redis', 'Socket.io', 'Docker'],
      features: [
        'Real-time collaboration with live updates',
        'Kanban boards with drag-and-drop functionality',
        'Time tracking with detailed reports',
        'Team management with role-based permissions',
        'File attachments and comments system',
        'Advanced search and filtering',
        'API documentation with Swagger UI',
        'Automated testing with 95% coverage'
      ],
      screenshots: [
        'https://via.placeholder.com/800x500/3B82F6/FFFFFF?text=Dashboard+View',
        'https://via.placeholder.com/800x500/8B5CF6/FFFFFF?text=Project+Board',
        'https://via.placeholder.com/800x500/10B981/FFFFFF?text=Team+Management'
      ],
      license: 'MIT',
      contributors: [
        {
          name: 'Sarah Kim',
          avatar: '👩‍🚀',
          contribution: 'Frontend Development'
        },
        {
          name: 'Mike Rodriguez',
          avatar: '🎮',
          contribution: 'Backend API'
        },
        {
          name: 'Emily Watson',
          avatar: '📊',
          contribution: 'Database Design'
        }
      ],
      documentation: [
        {
          title: 'Installation Guide',
          url: 'https://docs.multios.app/task-manager/installation'
        },
        {
          title: 'API Reference',
          url: 'https://docs.multios.app/task-manager/api'
        },
        {
          title: 'Deployment Guide',
          url: 'https://docs.multios.app/task-manager/deployment'
        }
      ]
    };

    setProject(mockProject);
    setLikedCount(mockProject.likes);
  }, [id]);

  const handleLike = () => {
    setIsLiked(!isLiked);
    setLikedCount(prev => isLiked ? prev - 1 : prev + 1);
  };

  const handleShare = async () => {
    if (navigator.share) {
      try {
        await navigator.share({
          title: project?.title,
          text: project?.description,
          url: window.location.href,
        });
      } catch (err) {
        console.log('Error sharing:', err);
      }
    } else {
      // Fallback to clipboard
      navigator.clipboard.writeText(window.location.href);
      alert('Link copied to clipboard!');
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric'
    });
  };

  if (!project) {
    return (
      <div className="min-h-screen bg-slate-50 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-blue-600 mx-auto mb-4"></div>
          <p className="text-slate-600">Loading project...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-slate-50">
      {/* Header */}
      <div className="bg-white border-b border-slate-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
          <div className="flex items-center justify-between">
            <Link
              to="/community"
              className="flex items-center space-x-2 text-slate-600 hover:text-slate-900"
            >
              <ArrowLeft className="h-5 w-5" />
              <span>Back to Community</span>
            </Link>
            
            <div className="flex items-center space-x-2">
              <button
                onClick={handleLike}
                className={`flex items-center space-x-2 px-4 py-2 rounded-lg transition-colors ${
                  isLiked
                    ? 'bg-red-100 text-red-700 hover:bg-red-200'
                    : 'bg-slate-100 text-slate-700 hover:bg-slate-200'
                }`}
              >
                <Heart className={`h-4 w-4 ${isLiked ? 'fill-current' : ''}`} />
                <span>{likedCount}</span>
              </button>
              
              <button
                onClick={handleShare}
                className="flex items-center space-x-2 bg-slate-100 text-slate-700 px-4 py-2 rounded-lg hover:bg-slate-200 transition-colors"
              >
                <Share2 className="h-4 w-4" />
                <span>Share</span>
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Project Header */}
      <div className="bg-white border-b border-slate-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
            <div className="lg:col-span-2">
              <div className="flex items-center space-x-3 mb-4">
                <div className="text-4xl">{project.author.avatar}</div>
                <div>
                  <h1 className="text-3xl font-bold text-slate-900">{project.title}</h1>
                  <div className="flex items-center space-x-4 mt-2">
                    <span className="text-slate-600">by {project.author.name}</span>
                    <span className="px-2 py-1 bg-blue-100 text-blue-800 rounded-full text-xs font-medium">
                      {project.status}
                    </span>
                    <span className="text-slate-500">•</span>
                    <span className="text-slate-600">{project.language}</span>
                  </div>
                </div>
              </div>
              
              <p className="text-lg text-slate-600 mb-6">{project.description}</p>
              
              <div className="flex flex-wrap gap-2 mb-6">
                {project.tags.map((tag, index) => (
                  <span key={index} className="px-3 py-1 bg-slate-100 text-slate-700 rounded-full text-sm">
                    {tag}
                  </span>
                ))}
              </div>
              
              <div className="flex items-center space-x-6 text-sm text-slate-500">
                <div className="flex items-center space-x-1">
                  <Eye className="h-4 w-4" />
                  <span>{project.views.toLocaleString()} views</span>
                </div>
                <div className="flex items-center space-x-1">
                  <MessageCircle className="h-4 w-4" />
                  <span>{project.comments} comments</span>
                </div>
                <div className="flex items-center space-x-1">
                  <Calendar className="h-4 w-4" />
                  <span>Updated {formatDate(project.updatedAt)}</span>
                </div>
              </div>
            </div>
            
            {/* Action Buttons */}
            <div className="space-y-4">
              {project.demoUrl && (
                <a
                  href={project.demoUrl}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="w-full bg-gradient-to-r from-blue-600 to-purple-600 text-white py-3 px-6 rounded-lg hover:shadow-lg transition-all duration-300 flex items-center justify-center space-x-2"
                >
                  <Monitor className="h-5 w-5" />
                  <span>View Demo</span>
                  <ExternalLink className="h-4 w-4" />
                </a>
              )}
              
              {project.githubUrl && (
                <a
                  href={project.githubUrl}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="w-full bg-slate-800 text-white py-3 px-6 rounded-lg hover:bg-slate-700 transition-colors flex items-center justify-center space-x-2"
                >
                  <GitBranch className="h-5 w-5" />
                  <span>View Code</span>
                  <ExternalLink className="h-4 w-4" />
                </a>
              )}
              
              {project.websiteUrl && (
                <a
                  href={project.websiteUrl}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="w-full bg-white border-2 border-slate-300 text-slate-700 py-3 px-6 rounded-lg hover:border-slate-400 transition-colors flex items-center justify-center space-x-2"
                >
                  <Globe className="h-5 w-5" />
                  <span>Visit Website</span>
                  <ExternalLink className="h-4 w-4" />
                </a>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Navigation Tabs */}
      <div className="bg-white border-b border-slate-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <nav className="flex space-x-8">
            {[
              { id: 'overview', label: 'Overview', icon: Globe },
              { id: 'code', label: 'Code', icon: Code },
              { id: 'screenshots', label: 'Screenshots', icon: Monitor },
              { id: 'discussions', label: 'Discussions', icon: MessageCircle }
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
      </div>

      {/* Main Content */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          {/* Main Content */}
          <div className="lg:col-span-2">
            {activeTab === 'overview' && (
              <div className="bg-white rounded-xl shadow-lg border border-slate-200 p-8">
                <div className="prose max-w-none">
                  <div className="whitespace-pre-wrap text-slate-700 leading-relaxed">
                    {project.fullDescription}
                  </div>
                </div>
                
                <div className="mt-8 pt-8 border-t border-slate-200">
                  <h3 className="text-xl font-bold text-slate-900 mb-4">Key Features</h3>
                  <ul className="space-y-2">
                    {project.features.map((feature, index) => (
                      <li key={index} className="flex items-center space-x-3">
                        <Zap className="h-4 w-4 text-yellow-500 flex-shrink-0" />
                        <span className="text-slate-700">{feature}</span>
                      </li>
                    ))}
                  </ul>
                </div>
                
                <div className="mt-8 pt-8 border-t border-slate-200">
                  <h3 className="text-xl font-bold text-slate-900 mb-4">Technologies Used</h3>
                  <div className="grid grid-cols-2 md:grid-cols-3 gap-3">
                    {project.technologies.map((tech, index) => (
                      <div key={index} className="flex items-center space-x-2 p-3 bg-slate-50 rounded-lg">
                        <Code className="h-4 w-4 text-blue-600" />
                        <span className="text-slate-700 font-medium">{tech}</span>
                      </div>
                    ))}
                  </div>
                </div>
                
                {project.documentation && (
                  <div className="mt-8 pt-8 border-t border-slate-200">
                    <h3 className="text-xl font-bold text-slate-900 mb-4">Documentation</h3>
                    <div className="space-y-2">
                      {project.documentation.map((doc, index) => (
                        <a
                          key={index}
                          href={doc.url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="flex items-center space-x-2 text-blue-600 hover:text-blue-700"
                        >
                          <ExternalLink className="h-4 w-4" />
                          <span>{doc.title}</span>
                        </a>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            )}
            
            {activeTab === 'code' && (
              <div className="bg-white rounded-xl shadow-lg border border-slate-200 p-8">
                <div className="text-center py-16">
                  <Code className="h-16 w-16 text-slate-400 mx-auto mb-4" />
                  <h3 className="text-xl font-semibold text-slate-900 mb-2">Code Viewer</h3>
                  <p className="text-slate-600 mb-6">
                    Browse the source code of this project directly in your browser.
                  </p>
                  <a
                    href={project.githubUrl}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="bg-gradient-to-r from-blue-600 to-purple-600 text-white px-6 py-3 rounded-lg hover:shadow-lg transition-all duration-300 inline-flex items-center space-x-2"
                  >
                    <GitBranch className="h-5 w-5" />
                    <span>View on GitHub</span>
                    <ExternalLink className="h-4 w-4" />
                  </a>
                </div>
              </div>
            )}
            
            {activeTab === 'screenshots' && (
              <div className="space-y-6">
                {project.screenshots.map((screenshot, index) => (
                  <div key={index} className="bg-white rounded-xl shadow-lg border border-slate-200 overflow-hidden">
                    <img
                      src={screenshot}
                      alt={`Screenshot ${index + 1}`}
                      className="w-full h-auto"
                    />
                  </div>
                ))}
              </div>
            )}
            
            {activeTab === 'discussions' && (
              <div className="bg-white rounded-xl shadow-lg border border-slate-200 p-8">
                <div className="text-center py-16">
                  <MessageCircle className="h-16 w-16 text-slate-400 mx-auto mb-4" />
                  <h3 className="text-xl font-semibold text-slate-900 mb-2">Discussions</h3>
                  <p className="text-slate-600">
                    Join the conversation about this project with the community.
                  </p>
                </div>
              </div>
            )}
          </div>

          {/* Sidebar */}
          <div className="space-y-6">
            {/* Author Info */}
            <div className="bg-white rounded-xl shadow-lg border border-slate-200 p-6">
              <h3 className="text-lg font-bold text-slate-900 mb-4">Author</h3>
              <div className="flex items-center space-x-3 mb-4">
                <div className="text-2xl">{project.author.avatar}</div>
                <div>
                  <div className="font-semibold text-slate-900">{project.author.name}</div>
                  <div className="text-sm text-slate-600">{project.author.level}</div>
                </div>
              </div>
              <p className="text-sm text-slate-600 mb-4">{project.author.bio}</p>
              <div className="flex items-center justify-between text-sm text-slate-500">
                <div className="flex items-center space-x-1">
                  <Code className="h-4 w-4" />
                  <span>{project.author.projectsCount} projects</span>
                </div>
                <div className="flex items-center space-x-1">
                  <Users className="h-4 w-4" />
                  <span>{project.author.followers} followers</span>
                </div>
              </div>
            </div>

            {/* Project Stats */}
            <div className="bg-white rounded-xl shadow-lg border border-slate-200 p-6">
              <h3 className="text-lg font-bold text-slate-900 mb-4">Project Stats</h3>
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <span className="text-slate-600">License</span>
                  <span className="font-medium text-slate-900">{project.license}</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-slate-600">Language</span>
                  <span className="font-medium text-slate-900">{project.language}</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-slate-600">Category</span>
                  <span className="font-medium text-slate-900">{project.category}</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-slate-600">Created</span>
                  <span className="font-medium text-slate-900">{formatDate(project.createdAt)}</span>
                </div>
              </div>
            </div>

            {/* Contributors */}
            {project.contributors && project.contributors.length > 0 && (
              <div className="bg-white rounded-xl shadow-lg border border-slate-200 p-6">
                <h3 className="text-lg font-bold text-slate-900 mb-4">Contributors</h3>
                <div className="space-y-3">
                  {project.contributors.map((contributor, index) => (
                    <div key={index} className="flex items-center space-x-3">
                      <div className="text-xl">{contributor.avatar}</div>
                      <div>
                        <div className="font-medium text-slate-900">{contributor.name}</div>
                        <div className="text-sm text-slate-600">{contributor.contribution}</div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};