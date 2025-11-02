import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';
import { 
  BookOpen, 
  Star, 
  Download, 
  Users, 
  Award,
  TrendingUp,
  Search,
  ArrowRight,
  Play,
  Heart,
  Filter
} from 'lucide-react';

interface App {
  id: string;
  title: string;
  description: string;
  short_description: string;
  icon: string;
  rating: number;
  review_count: number;
  download_count: number;
  price: number;
  featured: boolean;
  category_name: string;
  developer_name: string;
}

interface Category {
  id: string;
  name: string;
  description: string;
  icon: string;
  color: string;
  app_count: number;
  avg_rating: number;
}

const HomePage: React.FC = () => {
  const { isAuthenticated } = useAuth();
  const [featuredApps, setFeaturedApps] = useState<App[]>([]);
  const [popularApps, setPopularApps] = useState<App[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [recommendedApps, setRecommendedApps] = useState<App[]>([]);
  const [loading, setLoading] = useState(true);

  const API_BASE = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3001/api';

  useEffect(() => {
    loadHomePageData();
  }, []);

  const loadHomePageData = async () => {
    try {
      setLoading(true);

      // Load featured apps
      const featuredResponse = await fetch(`${API_BASE}/apps/featured/list`);
      const featuredData = await featuredResponse.json();
      if (featuredData.success) {
        setFeaturedApps(featuredData.data.slice(0, 6));
      }

      // Load popular apps
      const popularResponse = await fetch(`${API_BASE}/apps/popular/list?limit=8`);
      const popularData = await popularResponse.json();
      if (popularData.success) {
        setPopularApps(popularData.data);
      }

      // Load categories
      const categoriesResponse = await fetch(`${API_BASE}/categories?includeSubcategories=false`);
      const categoriesData = await categoriesResponse.json();
      if (categoriesData.success) {
        setCategories(categoriesData.data.slice(0, 8));
      }

      // Load recommendations
      try {
        const recommendationsResponse = await fetch(`${API_BASE}/recommendations/personalized?limit=6`);
        const recommendationsData = await recommendationsResponse.json();
        if (recommendationsData.success) {
          setRecommendedApps(recommendationsData.data.recommendations);
        }
      } catch (error) {
        console.log('No personalized recommendations available');
      }

    } catch (error) {
      console.error('Error loading home page data:', error);
    } finally {
      setLoading(false);
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

  const AppCard: React.FC<{ app: App; showCategory?: boolean }> = ({ app, showCategory = false }) => (
    <Link 
      to={`/apps/${app.id}`}
      className="bg-white rounded-lg shadow-md hover:shadow-lg transition-shadow duration-300 overflow-hidden group"
    >
      <div className="p-4">
        <div className="flex items-start space-x-3">
          <img
            src={app.icon || '/placeholder-app-icon.png'}
            alt={app.title}
            className="w-12 h-12 rounded-lg object-cover"
          />
          <div className="flex-1 min-w-0">
            <h3 className="font-semibold text-gray-900 truncate group-hover:text-blue-600 transition-colors">
              {app.title}
            </h3>
            <p className="text-sm text-gray-600 truncate">{app.developer_name}</p>
            {showCategory && (
              <p className="text-xs text-blue-600">{app.category_name}</p>
            )}
          </div>
          <div className="text-right">
            <div className="flex items-center space-x-1">
              <Star className="h-4 w-4 text-yellow-400 fill-current" />
              <span className="text-sm font-medium">{app.rating.toFixed(1)}</span>
            </div>
            <p className="text-xs text-gray-500">{formatNumber(app.review_count)} reviews</p>
          </div>
        </div>
        
        <p className="mt-3 text-sm text-gray-600 line-clamp-2">
          {app.short_description || app.description}
        </p>
        
        <div className="mt-4 flex items-center justify-between">
          <div className="flex items-center space-x-4">
            <div className="flex items-center space-x-1">
              <Download className="h-4 w-4 text-gray-400" />
              <span className="text-sm text-gray-600">{formatNumber(app.download_count)}</span>
            </div>
            <div className="flex items-center space-x-1">
              <Heart className="h-4 w-4 text-gray-400" />
              <span className="text-sm text-gray-600">{formatNumber(app.review_count * 2)}</span>
            </div>
          </div>
          <span className={`text-sm font-medium ${app.price === 0 ? 'text-green-600' : 'text-gray-900'}`}>
            {formatPrice(app.price)}
          </span>
        </div>
      </div>
    </Link>
  );

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Hero Section */}
      <section className="bg-gradient-to-r from-blue-600 to-purple-700 text-white">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-16">
          <div className="text-center">
            <h1 className="text-4xl md:text-6xl font-bold mb-6">
              Discover Educational Apps
            </h1>
            <p className="text-xl md:text-2xl text-blue-100 mb-8 max-w-3xl mx-auto">
              Explore our curated collection of high-quality educational applications designed to enhance learning experiences
            </p>
            <div className="flex flex-col sm:flex-row items-center justify-center space-y-4 sm:space-y-0 sm:space-x-4">
              <Link
                to="/apps"
                className="bg-white text-blue-600 px-8 py-3 rounded-lg font-semibold hover:bg-gray-100 transition-colors flex items-center"
              >
                <Search className="h-5 w-5 mr-2" />
                Browse All Apps
              </Link>
              <Link
                to="/categories"
                className="border border-white text-white px-8 py-3 rounded-lg font-semibold hover:bg-white hover:text-blue-600 transition-colors"
              >
                Explore Categories
              </Link>
            </div>
          </div>
        </div>
      </section>

      {/* Stats Section */}
      <section className="py-16 bg-white">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
            <div className="text-center">
              <div className="bg-blue-100 rounded-full w-16 h-16 flex items-center justify-center mx-auto mb-4">
                <BookOpen className="h-8 w-8 text-blue-600" />
              </div>
              <h3 className="text-3xl font-bold text-gray-900">500+</h3>
              <p className="text-gray-600">Educational Apps</p>
            </div>
            <div className="text-center">
              <div className="bg-green-100 rounded-full w-16 h-16 flex items-center justify-center mx-auto mb-4">
                <Users className="h-8 w-8 text-green-600" />
              </div>
              <h3 className="text-3xl font-bold text-gray-900">10K+</h3>
              <p className="text-gray-600">Active Educators</p>
            </div>
            <div className="text-center">
              <div className="bg-purple-100 rounded-full w-16 h-16 flex items-center justify-center mx-auto mb-4">
                <Download className="h-8 w-8 text-purple-600" />
              </div>
              <h3 className="text-3xl font-bold text-gray-900">1M+</h3>
              <p className="text-gray-600">App Downloads</p>
            </div>
            <div className="text-center">
              <div className="bg-yellow-100 rounded-full w-16 h-16 flex items-center justify-center mx-auto mb-4">
                <Star className="h-8 w-8 text-yellow-600" />
              </div>
              <h3 className="text-3xl font-bold text-gray-900">4.8</h3>
              <p className="text-gray-600">Average Rating</p>
            </div>
          </div>
        </div>
      </section>

      {/* Featured Apps Section */}
      <section className="py-16">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between mb-8">
            <div>
              <h2 className="text-3xl font-bold text-gray-900">Featured Apps</h2>
              <p className="text-gray-600 mt-2">Handpicked educational applications</p>
            </div>
            <Link 
              to="/apps?featured=true"
              className="text-blue-600 hover:text-blue-700 font-medium flex items-center"
            >
              View All <ArrowRight className="h-4 w-4 ml-1" />
            </Link>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {featuredApps.map((app) => (
              <AppCard key={app.id} app={app} showCategory />
            ))}
          </div>
        </div>
      </section>

      {/* Categories Section */}
      <section className="py-16 bg-white">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between mb-8">
            <div>
              <h2 className="text-3xl font-bold text-gray-900">Explore Categories</h2>
              <p className="text-gray-600 mt-2">Find apps by subject and grade level</p>
            </div>
            <Link 
              to="/categories"
              className="text-blue-600 hover:text-blue-700 font-medium flex items-center"
            >
              View All <ArrowRight className="h-4 w-4 ml-1" />
            </Link>
          </div>

          <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
            {categories.map((category) => (
              <Link
                key={category.id}
                to={`/categories/${category.id}`}
                className="bg-white rounded-lg p-6 shadow-md hover:shadow-lg transition-shadow text-center group"
              >
                <div 
                  className="w-16 h-16 rounded-full flex items-center justify-center mx-auto mb-4"
                  style={{ backgroundColor: `${category.color}20` }}
                >
                  <span className="text-2xl">{category.icon}</span>
                </div>
                <h3 className="font-semibold text-gray-900 group-hover:text-blue-600 transition-colors">
                  {category.name}
                </h3>
                <p className="text-sm text-gray-600 mt-1">
                  {category.app_count} apps
                </p>
                <div className="flex items-center justify-center mt-2">
                  <Star className="h-4 w-4 text-yellow-400 fill-current" />
                  <span className="text-sm text-gray-600 ml-1">
                    {category.avg_rating ? category.avg_rating.toFixed(1) : 'N/A'}
                  </span>
                </div>
              </Link>
            ))}
          </div>
        </div>
      </section>

      {/* Personalized Recommendations */}
      {isAuthenticated && recommendedApps.length > 0 && (
        <section className="py-16">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="flex items-center justify-between mb-8">
              <div>
                <h2 className="text-3xl font-bold text-gray-900">Recommended for You</h2>
                <p className="text-gray-600 mt-2">Based on your preferences and usage</p>
              </div>
              <button className="text-blue-600 hover:text-blue-700 font-medium flex items-center">
                <Filter className="h-4 w-4 mr-1" />
                Customize
              </button>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              {recommendedApps.map((app) => (
                <AppCard key={app.id} app={app} />
              ))}
            </div>
          </div>
        </section>
      )}

      {/* Popular Apps Section */}
      <section className="py-16 bg-white">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between mb-8">
            <div>
              <h2 className="text-3xl font-bold text-gray-900">Popular This Week</h2>
              <p className="text-gray-600 mt-2">Most downloaded educational apps</p>
            </div>
            <Link 
              to="/apps?sort=popularity"
              className="text-blue-600 hover:text-blue-700 font-medium flex items-center"
            >
              View All <ArrowRight className="h-4 w-4 ml-1" />
            </Link>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            {popularApps.map((app, index) => (
              <div key={app.id} className="relative">
                <div className="absolute -top-2 -left-2 bg-blue-600 text-white rounded-full w-6 h-6 flex items-center justify-center text-sm font-bold z-10">
                  {index + 1}
                </div>
                <AppCard app={app} showCategory />
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* CTA Section */}
      {!isAuthenticated && (
        <section className="py-16 bg-gradient-to-r from-blue-600 to-purple-700 text-white">
          <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
            <h2 className="text-3xl font-bold mb-4">Join Our Community</h2>
            <p className="text-xl text-blue-100 mb-8">
              Discover, share, and rate the best educational applications with educators worldwide
            </p>
            <div className="flex flex-col sm:flex-row items-center justify-center space-y-4 sm:space-y-0 sm:space-x-4">
              <Link
                to="/register?role=educator"
                className="bg-white text-blue-600 px-8 py-3 rounded-lg font-semibold hover:bg-gray-100 transition-colors"
              >
                Join as Educator
              </Link>
              <Link
                to="/register?role=developer"
                className="border border-white text-white px-8 py-3 rounded-lg font-semibold hover:bg-white hover:text-blue-600 transition-colors"
              >
                Submit Your App
              </Link>
            </div>
          </div>
        </section>
      )}
    </div>
  );
};

export default HomePage;