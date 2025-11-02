import React, { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';
import { useUser } from '../contexts/UserContext';
import { 
  Star, 
  Download, 
  Heart, 
  Share2, 
  ExternalLink,
  Play,
  BookOpen,
  Users,
  Shield,
  CheckCircle,
  AlertTriangle,
  MessageSquare,
  ThumbsUp,
  ThumbsDown,
  ArrowLeft
} from 'lucide-react';

interface App {
  id: string;
  title: string;
  description: string;
  short_description: string;
  icon: string;
  screenshots: string[];
  rating: number;
  review_count: number;
  download_count: number;
  price: number;
  platform: string[];
  version: string;
  app_size: number;
  category_name: string;
  subcategory_name: string;
  developer_name: string;
  developer_institution: string;
  website_url: string;
  download_url: string;
  video_url: string;
  featured: boolean;
  verified: boolean;
  educational_impact: any;
  accessibility: any;
  grade_levels: string[];
  subjects: string[];
  tags: string[];
}

interface Review {
  id: string;
  rating: number;
  title: string;
  content: string;
  helpful: number;
  not_helpful: number;
  verified: boolean;
  created_at: string;
  reviewer_name: string;
  reviewer_avatar?: string;
}

const AppDetailPage: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const { isAuthenticated, user } = useAuth();
  const { addToFavorites, removeFromFavorites } = useUser();
  const [app, setApp] = useState<App | null>(null);
  const [reviews, setReviews] = useState<Review[]>([]);
  const [similarApps, setSimilarApps] = useState<App[]>([]);
  const [loading, setLoading] = useState(true);
  const [isFavorited, setIsFavorited] = useState(false);
  const [showReviewForm, setShowReviewForm] = useState(false);
  const [newReview, setNewReview] = useState({ rating: 5, title: '', content: '' });
  const [submittingReview, setSubmittingReview] = useState(false);

  const API_BASE = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3001/api';

  useEffect(() => {
    if (id) {
      loadAppDetails();
      loadReviews();
      loadSimilarApps();
    }
  }, [id]);

  const loadAppDetails = async () => {
    try {
      const response = await fetch(`${API_BASE}/apps/${id}`);
      const data = await response.json();
      if (data.success) {
        setApp(data.data);
        // Track view
        if (isAuthenticated) {
          fetch(`${API_BASE}/analytics/track`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              appId: id,
              eventType: 'view',
              userId: user?.id
            })
          });
        }
      }
    } catch (error) {
      console.error('Error loading app details:', error);
    } finally {
      setLoading(false);
    }
  };

  const loadReviews = async () => {
    try {
      const response = await fetch(`${API_BASE}/reviews/${id}?limit=5`);
      const data = await response.json();
      if (data.success) {
        setReviews(data.data.items);
      }
    } catch (error) {
      console.error('Error loading reviews:', error);
    }
  };

  const loadSimilarApps = async () => {
    try {
      const response = await fetch(`${API_BASE}/recommendations/similar/${id}?limit=4`);
      const data = await response.json();
      if (data.success) {
        setSimilarApps(data.data);
      }
    } catch (error) {
      console.error('Error loading similar apps:', error);
    }
  };

  const handleFavoriteToggle = async () => {
    if (!isAuthenticated || !user || !app) return;

    try {
      if (isFavorited) {
        const success = await removeFromFavorites(app.id);
        if (success) setIsFavorited(false);
      } else {
        const success = await addToFavorites(app.id);
        if (success) setIsFavorited(true);
      }
    } catch (error) {
      console.error('Error toggling favorite:', error);
    }
  };

  const handleDownload = async () => {
    if (!app) return;

    // Track download
    if (isAuthenticated && user) {
      fetch(`${API_BASE}/analytics/track`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          appId: app.id,
          eventType: 'download',
          userId: user.id
        })
      });
    }

    // Open download URL
    if (app.download_url) {
      window.open(app.download_url, '_blank');
    } else if (app.website_url) {
      window.open(app.website_url, '_blank');
    }
  };

  const handleReviewSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!user || !app) return;

    setSubmittingReview(true);
    try {
      const response = await fetch(`${API_BASE}/reviews`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('authToken')}`
        },
        body: JSON.stringify({
          appId: app.id,
          ...newReview
        })
      });

      const data = await response.json();
      if (data.success) {
        setShowReviewForm(false);
        setNewReview({ rating: 5, title: '', content: '' });
        loadReviews(); // Reload reviews
        loadAppDetails(); // Reload app to update rating
      }
    } catch (error) {
      console.error('Error submitting review:', error);
    } finally {
      setSubmittingReview(false);
    }
  };

  const formatPrice = (price: number) => {
    return price === 0 ? 'Free' : `$${price.toFixed(2)}`;
  };

  const formatNumber = (num: number) => {
    if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
    if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
    return num.toString();
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric'
    });
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  if (!app) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-center">
          <h2 className="text-2xl font-bold text-gray-900 mb-4">App not found</h2>
          <Link to="/apps" className="text-blue-600 hover:text-blue-700">
            Browse all apps
          </Link>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Breadcrumb */}
      <div className="bg-white shadow-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
          <nav className="flex items-center space-x-2 text-sm">
            <Link to="/" className="text-gray-500 hover:text-gray-700">Home</Link>
            <span className="text-gray-300">/</span>
            <Link to="/apps" className="text-gray-500 hover:text-gray-700">Apps</Link>
            <span className="text-gray-300">/</span>
            <Link to={`/categories/${app.category_name?.toLowerCase().replace(' ', '-')}`} className="text-gray-500 hover:text-gray-700">
              {app.category_name}
            </Link>
            <span className="text-gray-300">/</span>
            <span className="text-gray-900">{app.title}</span>
          </nav>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="lg:grid lg:grid-cols-3 lg:gap-8">
          {/* Main Content */}
          <div className="lg:col-span-2">
            {/* App Header */}
            <div className="bg-white rounded-lg shadow-md p-6 mb-8">
              <div className="flex items-start space-x-4">
                <img
                  src={app.icon || '/placeholder-app-icon.png'}
                  alt={app.title}
                  className="w-20 h-20 rounded-xl object-cover"
                />
                <div className="flex-1">
                  <div className="flex items-center space-x-2 mb-2">
                    <h1 className="text-2xl font-bold text-gray-900">{app.title}</h1>
                    {app.featured && (
                      <span className="bg-yellow-100 text-yellow-800 text-xs px-2 py-1 rounded-full">
                        Featured
                      </span>
                    )}
                    {app.verified && (
                      <CheckCircle className="h-5 w-5 text-blue-500" />
                    )}
                  </div>
                  <p className="text-gray-600 mb-2">{app.developer_name}</p>
                  <p className="text-sm text-blue-600">{app.category_name}</p>
                </div>
                <div className="text-right">
                  <div className="flex items-center space-x-1 mb-2">
                    <Star className="h-5 w-5 text-yellow-400 fill-current" />
                    <span className="font-semibold">{app.rating.toFixed(1)}</span>
                    <span className="text-gray-500">({formatNumber(app.review_count)} reviews)</span>
                  </div>
                  <div className="flex items-center space-x-4 text-sm text-gray-600">
                    <div className="flex items-center space-x-1">
                      <Download className="h-4 w-4" />
                      <span>{formatNumber(app.download_count)}</span>
                    </div>
                    <div className="flex items-center space-x-1">
                      <Heart className="h-4 w-4" />
                      <span>{formatNumber(app.review_count * 2)}</span>
                    </div>
                  </div>
                </div>
              </div>
              
              <p className="mt-4 text-gray-700">{app.short_description}</p>
              
              <div className="flex items-center space-x-4 mt-6">
                <button
                  onClick={handleDownload}
                  className="bg-blue-600 text-white px-6 py-3 rounded-lg font-semibold hover:bg-blue-700 transition-colors flex items-center space-x-2"
                >
                  <Download className="h-5 w-5" />
                  <span>{app.price === 0 ? 'Get Free' : 'Buy Now'}</span>
                </button>
                
                {isAuthenticated && (
                  <button
                    onClick={handleFavoriteToggle}
                    className={`px-4 py-3 rounded-lg border transition-colors flex items-center space-x-2 ${
                      isFavorited
                        ? 'bg-red-50 border-red-200 text-red-600'
                        : 'bg-white border-gray-300 text-gray-700 hover:bg-gray-50'
                    }`}
                  >
                    <Heart className={`h-5 w-5 ${isFavorited ? 'fill-current' : ''}`} />
                    <span>{isFavorited ? 'Favorited' : 'Add to Favorites'}</span>
                  </button>
                )}
                
                <button className="px-4 py-3 rounded-lg border border-gray-300 text-gray-700 hover:bg-gray-50 transition-colors flex items-center space-x-2">
                  <Share2 className="h-5 w-5" />
                  <span>Share</span>
                </button>
                
                {app.website_url && (
                  <a
                    href={app.website_url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="px-4 py-3 rounded-lg border border-gray-300 text-gray-700 hover:bg-gray-50 transition-colors flex items-center space-x-2"
                  >
                    <ExternalLink className="h-5 w-5" />
                    <span>Website</span>
                  </a>
                )}
              </div>
              
              <div className="mt-6 flex items-center justify-between">
                <div className="flex items-center space-x-6">
                  <span className={`text-lg font-semibold ${app.price === 0 ? 'text-green-600' : 'text-gray-900'}`}>
                    {formatPrice(app.price)}
                  </span>
                  <span className="text-sm text-gray-600">Version {app.version}</span>
                  {app.app_size && (
                    <span className="text-sm text-gray-600">{(app.app_size / (1024 * 1024)).toFixed(1)} MB</span>
                  )}
                </div>
                
                <div className="flex items-center space-x-2">
                  {app.platform.map((platform) => (
                    <span key={platform} className="bg-gray-100 text-gray-600 text-xs px-2 py-1 rounded">
                      {platform}
                    </span>
                  ))}
                </div>
              </div>
            </div>

            {/* Screenshots and Video */}
            {(app.screenshots?.length > 0 || app.video_url) && (
              <div className="bg-white rounded-lg shadow-md p-6 mb-8">
                <h2 className="text-xl font-bold text-gray-900 mb-4">Screenshots & Video</h2>
                
                {app.video_url && (
                  <div className="mb-6">
                    <div className="relative bg-gray-900 rounded-lg aspect-video flex items-center justify-center">
                      <button className="bg-blue-600 rounded-full p-4 hover:bg-blue-700 transition-colors">
                        <Play className="h-8 w-8 text-white fill-current" />
                      </button>
                    </div>
                  </div>
                )}
                
                {app.screenshots?.length > 0 && (
                  <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
                    {app.screenshots.map((screenshot, index) => (
                      <img
                        key={index}
                        src={screenshot}
                        alt={`${app.title} screenshot ${index + 1}`}
                        className="rounded-lg object-cover aspect-video cursor-pointer hover:opacity-80 transition-opacity"
                      />
                    ))}
                  </div>
                )}
              </div>
            )}

            {/* Description */}
            <div className="bg-white rounded-lg shadow-md p-6 mb-8">
              <h2 className="text-xl font-bold text-gray-900 mb-4">Description</h2>
              <div className="prose max-w-none text-gray-700">
                {app.description.split('\n').map((paragraph, index) => (
                  <p key={index} className="mb-4">{paragraph}</p>
                ))}
              </div>
            </div>

            {/* Educational Information */}
            {(app.grade_levels?.length > 0 || app.subjects?.length > 0 || app.tags?.length > 0) && (
              <div className="bg-white rounded-lg shadow-md p-6 mb-8">
                <h2 className="text-xl font-bold text-gray-900 mb-4">Educational Information</h2>
                
                {app.grade_levels?.length > 0 && (
                  <div className="mb-4">
                    <h3 className="font-semibold text-gray-900 mb-2">Grade Levels</h3>
                    <div className="flex flex-wrap gap-2">
                      {app.grade_levels.map((level) => (
                        <span key={level} className="bg-blue-100 text-blue-800 text-sm px-3 py-1 rounded-full">
                          {level}
                        </span>
                      ))}
                    </div>
                  </div>
                )}
                
                {app.subjects?.length > 0 && (
                  <div className="mb-4">
                    <h3 className="font-semibold text-gray-900 mb-2">Subjects</h3>
                    <div className="flex flex-wrap gap-2">
                      {app.subjects.map((subject) => (
                        <span key={subject} className="bg-green-100 text-green-800 text-sm px-3 py-1 rounded-full">
                          {subject}
                        </span>
                      ))}
                    </div>
                  </div>
                )}
                
                {app.tags?.length > 0 && (
                  <div>
                    <h3 className="font-semibold text-gray-900 mb-2">Tags</h3>
                    <div className="flex flex-wrap gap-2">
                      {app.tags.map((tag) => (
                        <span key={tag} className="bg-gray-100 text-gray-600 text-sm px-3 py-1 rounded-full">
                          {tag}
                        </span>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            )}

            {/* Reviews Section */}
            <div className="bg-white rounded-lg shadow-md p-6">
              <div className="flex items-center justify-between mb-6">
                <h2 className="text-xl font-bold text-gray-900">Reviews</h2>
                {isAuthenticated && (
                  <button
                    onClick={() => setShowReviewForm(!showReviewForm)}
                    className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 transition-colors"
                  >
                    Write Review
                  </button>
                )}
              </div>

              {/* Review Form */}
              {showReviewForm && (
                <form onSubmit={handleReviewSubmit} className="mb-6 p-4 border border-gray-200 rounded-lg">
                  <div className="mb-4">
                    <label className="block text-sm font-medium text-gray-700 mb-2">Rating</label>
                    <div className="flex space-x-1">
                      {[1, 2, 3, 4, 5].map((rating) => (
                        <button
                          key={rating}
                          type="button"
                          onClick={() => setNewReview(prev => ({ ...prev, rating }))}
                          className={`p-1 ${newReview.rating >= rating ? 'text-yellow-400' : 'text-gray-300'}`}
                        >
                          <Star className="h-6 w-6 fill-current" />
                        </button>
                      ))}
                    </div>
                  </div>
                  
                  <div className="mb-4">
                    <label className="block text-sm font-medium text-gray-700 mb-2">Review Title</label>
                    <input
                      type="text"
                      value={newReview.title}
                      onChange={(e) => setNewReview(prev => ({ ...prev, title: e.target.value }))}
                      className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
                      placeholder="Brief summary of your experience"
                      required
                    />
                  </div>
                  
                  <div className="mb-4">
                    <label className="block text-sm font-medium text-gray-700 mb-2">Review</label>
                    <textarea
                      value={newReview.content}
                      onChange={(e) => setNewReview(prev => ({ ...prev, content: e.target.value }))}
                      rows={4}
                      className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
                      placeholder="Share your detailed experience with this app..."
                      required
                    />
                  </div>
                  
                  <div className="flex space-x-3">
                    <button
                      type="submit"
                      disabled={submittingReview}
                      className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 disabled:opacity-50"
                    >
                      {submittingReview ? 'Submitting...' : 'Submit Review'}
                    </button>
                    <button
                      type="button"
                      onClick={() => setShowReviewForm(false)}
                      className="bg-gray-300 text-gray-700 px-4 py-2 rounded-lg hover:bg-gray-400"
                    >
                      Cancel
                    </button>
                  </div>
                </form>
              )}

              {/* Reviews List */}
              <div className="space-y-6">
                {reviews.map((review) => (
                  <div key={review.id} className="border-b border-gray-200 pb-6 last:border-b-0">
                    <div className="flex items-start space-x-3">
                      <img
                        src={review.reviewer_avatar || '/default-avatar.png'}
                        alt={review.reviewer_name}
                        className="w-10 h-10 rounded-full object-cover"
                      />
                      <div className="flex-1">
                        <div className="flex items-center space-x-2 mb-1">
                          <h4 className="font-semibold text-gray-900">{review.reviewer_name}</h4>
                          {review.verified && (
                            <CheckCircle className="h-4 w-4 text-blue-500" />
                          )}
                          <div className="flex items-center space-x-1">
                            {[...Array(5)].map((_, i) => (
                              <Star
                                key={i}
                                className={`h-4 w-4 ${i < review.rating ? 'text-yellow-400 fill-current' : 'text-gray-300'}`}
                              />
                            ))}
                          </div>
                          <span className="text-sm text-gray-500">{formatDate(review.created_at)}</span>
                        </div>
                        {review.title && <h5 className="font-medium text-gray-900 mb-2">{review.title}</h5>}
                        <p className="text-gray-700 mb-3">{review.content}</p>
                        <div className="flex items-center space-x-4">
                          <button className="flex items-center space-x-1 text-sm text-gray-600 hover:text-blue-600">
                            <ThumbsUp className="h-4 w-4" />
                            <span>Helpful ({review.helpful})</span>
                          </button>
                          <button className="flex items-center space-x-1 text-sm text-gray-600 hover:text-red-600">
                            <ThumbsDown className="h-4 w-4" />
                            <span>Not helpful ({review.not_helpful})</span>
                          </button>
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
              
              {reviews.length === 0 && (
                <div className="text-center py-8">
                  <MessageSquare className="h-12 w-12 text-gray-300 mx-auto mb-4" />
                  <p className="text-gray-600">No reviews yet. Be the first to review this app!</p>
                </div>
              )}
            </div>
          </div>

          {/* Sidebar */}
          <div className="lg:col-span-1 mt-8 lg:mt-0">
            {/* Developer Info */}
            <div className="bg-white rounded-lg shadow-md p-6 mb-6">
              <h3 className="text-lg font-bold text-gray-900 mb-4">About the Developer</h3>
              <div className="flex items-center space-x-3 mb-4">
                <div className="w-12 h-12 bg-gray-200 rounded-full flex items-center justify-center">
                  <Users className="h-6 w-6 text-gray-600" />
                </div>
                <div>
                  <p className="font-semibold text-gray-900">{app.developer_name}</p>
                  {app.developer_institution && (
                    <p className="text-sm text-gray-600">{app.developer_institution}</p>
                  )}
                </div>
              </div>
            </div>

            {/* App Features */}
            <div className="bg-white rounded-lg shadow-md p-6 mb-6">
              <h3 className="text-lg font-bold text-gray-900 mb-4">Features</h3>
              <div className="space-y-3">
                {app.educational_impact?.collaboration && (
                  <div className="flex items-center space-x-2">
                    <Users className="h-5 w-5 text-green-500" />
                    <span className="text-sm">Collaboration Tools</span>
                  </div>
                )}
                {app.educational_impact?.adaptiveLearning && (
                  <div className="flex items-center space-x-2">
                    <BookOpen className="h-5 w-5 text-blue-500" />
                    <span className="text-sm">Adaptive Learning</span>
                  </div>
                )}
                {app.accessibility?.screenReader && (
                  <div className="flex items-center space-x-2">
                    <Shield className="h-5 w-5 text-purple-500" />
                    <span className="text-sm">Screen Reader Support</span>
                  </div>
                )}
              </div>
            </div>

            {/* Similar Apps */}
            {similarApps.length > 0 && (
              <div className="bg-white rounded-lg shadow-md p-6">
                <h3 className="text-lg font-bold text-gray-900 mb-4">Similar Apps</h3>
                <div className="space-y-4">
                  {similarApps.map((similarApp) => (
                    <Link
                      key={similarApp.id}
                      to={`/apps/${similarApp.id}`}
                      className="flex items-center space-x-3 p-3 rounded-lg hover:bg-gray-50 transition-colors"
                    >
                      <img
                        src={similarApp.icon || '/placeholder-app-icon.png'}
                        alt={similarApp.title}
                        className="w-12 h-12 rounded-lg object-cover"
                      />
                      <div className="flex-1">
                        <h4 className="font-medium text-gray-900 truncate">{similarApp.title}</h4>
                        <p className="text-sm text-gray-600">{similarApp.category_name}</p>
                        <div className="flex items-center space-x-1 mt-1">
                          <Star className="h-4 w-4 text-yellow-400 fill-current" />
                          <span className="text-sm text-gray-600">{similarApp.rating.toFixed(1)}</span>
                        </div>
                      </div>
                    </Link>
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

export default AppDetailPage;