import React, { useState, useEffect } from 'react';
import { useSearchParams, Link } from 'react-router-dom';
import { 
  Search, 
  Filter, 
  Star, 
  Download, 
  Heart,
  ChevronDown,
  X,
  SlidersHorizontal,
  Grid,
  List
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
  category_name: string;
  subcategory_name: string;
  developer_name: string;
  platform: string[];
  grade_levels: string[];
  subjects: string[];
  featured: boolean;
}

interface Category {
  id: string;
  name: string;
  icon: string;
  color: string;
}

interface Filters {
  categories: string[];
  subcategories: string[];
  gradeLevels: string[];
  subjects: string[];
  price: string;
  platform: string[];
  rating: number;
  searchQuery: string;
  sortBy: string;
  sortOrder: string;
}

const AppsPage: React.FC = () => {
  const [searchParams, setSearchParams] = useSearchParams();
  const [apps, setApps] = useState<App[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [loading, setLoading] = useState(true);
  const [showFilters, setShowFilters] = useState(false);
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [totalPages, setTotalPages] = useState(1);
  const [currentPage, setCurrentPage] = useState(1);

  const [filters, setFilters] = useState<Filters>({
    categories: [],
    subcategories: [],
    gradeLevels: [],
    subjects: [],
    price: 'all',
    platform: [],
    rating: 0,
    searchQuery: searchParams.get('search') || '',
    sortBy: 'rating',
    sortOrder: 'desc'
  });

  const API_BASE = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3001/api';

  useEffect(() => {
    loadCategories();
  }, []);

  useEffect(() => {
    loadApps();
  }, [searchParams, currentPage, filters]);

  const loadCategories = async () => {
    try {
      const response = await fetch(`${API_BASE}/categories?includeSubcategories=true`);
      const data = await response.json();
      if (data.success) {
        setCategories(data.data);
      }
    } catch (error) {
      console.error('Error loading categories:', error);
    }
  };

  const loadApps = async () => {
    try {
      setLoading(true);
      
      const params = new URLSearchParams();
      
      if (filters.searchQuery) params.append('searchQuery', filters.searchQuery);
      if (filters.categories.length > 0) params.append('categories', JSON.stringify(filters.categories));
      if (filters.subcategories.length > 0) params.append('subcategories', JSON.stringify(filters.subcategories));
      if (filters.gradeLevels.length > 0) params.append('gradeLevels', JSON.stringify(filters.gradeLevels));
      if (filters.subjects.length > 0) params.append('subjects', JSON.stringify(filters.subjects));
      if (filters.price !== 'all') params.append('price', filters.price);
      if (filters.platform.length > 0) params.append('platform', JSON.stringify(filters.platform));
      if (filters.rating > 0) params.append('rating', filters.rating.toString());
      params.append('sortBy', filters.sortBy);
      params.append('sortOrder', filters.sortOrder);
      params.append('page', currentPage.toString());
      params.append('limit', '20');

      const response = await fetch(`${API_BASE}/apps?${params.toString()}`);
      const data = await response.json();
      
      if (data.success) {
        setApps(data.data.items);
        setTotalPages(data.data.totalPages);
      }
    } catch (error) {
      console.error('Error loading apps:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleFilterChange = (key: keyof Filters, value: any) => {
    setFilters(prev => ({ ...prev, [key]: value }));
    setCurrentPage(1);
  };

  const clearFilters = () => {
    setFilters({
      categories: [],
      subcategories: [],
      gradeLevels: [],
      subjects: [],
      price: 'all',
      platform: [],
      rating: 0,
      searchQuery: '',
      sortBy: 'rating',
      sortOrder: 'desc'
    });
    setSearchParams({});
    setCurrentPage(1);
  };

  const formatPrice = (price: number) => {
    return price === 0 ? 'Free' : `$${price.toFixed(2)}`;
  };

  const formatNumber = (num: number) => {
    if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
    if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
    return num.toString();
  };

  const AppCard: React.FC<{ app: App }> = ({ app }) => (
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
            <div className="flex items-center space-x-2">
              <h3 className="font-semibold text-gray-900 truncate group-hover:text-blue-600 transition-colors">
                {app.title}
              </h3>
              {app.featured && (
                <span className="bg-yellow-100 text-yellow-800 text-xs px-2 py-1 rounded-full">
                  Featured
                </span>
              )}
            </div>
            <p className="text-sm text-gray-600 truncate">{app.developer_name}</p>
            <p className="text-xs text-blue-600">{app.category_name}</p>
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
        
        {app.grade_levels.length > 0 && (
          <div className="mt-3 flex flex-wrap gap-1">
            {app.grade_levels.slice(0, 3).map((level) => (
              <span key={level} className="bg-gray-100 text-gray-600 text-xs px-2 py-1 rounded">
                {level}
              </span>
            ))}
            {app.grade_levels.length > 3 && (
              <span className="text-xs text-gray-500">+{app.grade_levels.length - 3} more</span>
            )}
          </div>
        )}
      </div>
    </Link>
  );

  const AppListItem: React.FC<{ app: App }> = ({ app }) => (
    <Link 
      to={`/apps/${app.id}`}
      className="bg-white rounded-lg shadow-md hover:shadow-lg transition-shadow duration-300 p-6 flex items-center space-x-4 group"
    >
      <img
        src={app.icon || '/placeholder-app-icon.png'}
        alt={app.title}
        className="w-16 h-16 rounded-lg object-cover"
      />
      <div className="flex-1">
        <div className="flex items-center space-x-2 mb-1">
          <h3 className="font-semibold text-gray-900 group-hover:text-blue-600 transition-colors">
            {app.title}
          </h3>
          {app.featured && (
            <span className="bg-yellow-100 text-yellow-800 text-xs px-2 py-1 rounded-full">
              Featured
            </span>
          )}
        </div>
        <p className="text-sm text-gray-600 mb-2">{app.developer_name}</p>
        <p className="text-sm text-gray-600 line-clamp-2">
          {app.short_description || app.description}
        </p>
        <div className="flex items-center space-x-4 mt-2">
          <div className="flex items-center space-x-1">
            <Star className="h-4 w-4 text-yellow-400 fill-current" />
            <span className="text-sm font-medium">{app.rating.toFixed(1)}</span>
          </div>
          <div className="flex items-center space-x-1">
            <Download className="h-4 w-4 text-gray-400" />
            <span className="text-sm text-gray-600">{formatNumber(app.download_count)}</span>
          </div>
          <span className={`text-sm font-medium ${app.price === 0 ? 'text-green-600' : 'text-gray-900'}`}>
            {formatPrice(app.price)}
          </span>
        </div>
      </div>
    </Link>
  );

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <div className="bg-white shadow-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
          <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between space-y-4 lg:space-y-0">
            <div>
              <h1 className="text-3xl font-bold text-gray-900">Educational Apps</h1>
              <p className="text-gray-600 mt-1">
                Discover {apps.length} high-quality educational applications
              </p>
            </div>
            
            {/* Search Bar */}
            <div className="w-full lg:w-96">
              <div className="relative">
                <input
                  type="text"
                  value={filters.searchQuery}
                  onChange={(e) => handleFilterChange('searchQuery', e.target.value)}
                  placeholder="Search apps..."
                  className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
                <Search className="absolute left-3 top-2.5 h-5 w-5 text-gray-400" />
              </div>
            </div>
          </div>

          {/* Controls */}
          <div className="flex items-center justify-between mt-6">
            <div className="flex items-center space-x-4">
              <button
                onClick={() => setShowFilters(!showFilters)}
                className="flex items-center space-x-2 px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
              >
                <SlidersHorizontal className="h-4 w-4" />
                <span>Filters</span>
                {(filters.categories.length > 0 || filters.platform.length > 0 || filters.price !== 'all' || filters.rating > 0) && (
                  <span className="bg-blue-100 text-blue-800 text-xs px-2 py-1 rounded-full">
                    {filters.categories.length + filters.platform.length + (filters.price !== 'all' ? 1 : 0) + (filters.rating > 0 ? 1 : 0)}
                  </span>
                )}
              </button>
              
              {(filters.categories.length > 0 || filters.platform.length > 0 || filters.price !== 'all' || filters.rating > 0 || filters.searchQuery) && (
                <button
                  onClick={clearFilters}
                  className="text-blue-600 hover:text-blue-700 text-sm"
                >
                  Clear all filters
                </button>
              )}
            </div>

            <div className="flex items-center space-x-4">
              {/* Sort Options */}
              <select
                value={`${filters.sortBy}-${filters.sortOrder}`}
                onChange={(e) => {
                  const [sortBy, sortOrder] = e.target.value.split('-');
                  handleFilterChange('sortBy', sortBy);
                  handleFilterChange('sortOrder', sortOrder);
                }}
                className="px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
              >
                <option value="rating-desc">Highest Rated</option>
                <option value="download_count-desc">Most Popular</option>
                <option value="created_at-desc">Newest</option>
                <option value="title-asc">Name A-Z</option>
                <option value="price-asc">Price: Low to High</option>
                <option value="price-desc">Price: High to Low</option>
              </select>

              {/* View Mode Toggle */}
              <div className="flex border border-gray-300 rounded-lg">
                <button
                  onClick={() => setViewMode('grid')}
                  className={`p-2 ${viewMode === 'grid' ? 'bg-blue-600 text-white' : 'text-gray-600 hover:bg-gray-50'}`}
                >
                  <Grid className="h-4 w-4" />
                </button>
                <button
                  onClick={() => setViewMode('list')}
                  className={`p-2 ${viewMode === 'list' ? 'bg-blue-600 text-white' : 'text-gray-600 hover:bg-gray-50'}`}
                >
                  <List className="h-4 w-4" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="lg:grid lg:grid-cols-4 lg:gap-8">
          {/* Filters Sidebar */}
          {showFilters && (
            <div className="lg:col-span-1">
              <div className="bg-white rounded-lg shadow-md p-6 space-y-6">
                {/* Price Filter */}
                <div>
                  <h3 className="font-semibold text-gray-900 mb-3">Price</h3>
                  <div className="space-y-2">
                    {['all', 'free', 'paid'].map((price) => (
                      <label key={price} className="flex items-center">
                        <input
                          type="radio"
                          name="price"
                          value={price}
                          checked={filters.price === price}
                          onChange={(e) => handleFilterChange('price', e.target.value)}
                          className="mr-2"
                        />
                        <span className="text-sm capitalize">{price === 'all' ? 'All' : price}</span>
                      </label>
                    ))}
                  </div>
                </div>

                {/* Rating Filter */}
                <div>
                  <h3 className="font-semibold text-gray-900 mb-3">Minimum Rating</h3>
                  <div className="space-y-2">
                    {[0, 4, 3, 2, 1].map((rating) => (
                      <label key={rating} className="flex items-center">
                        <input
                          type="radio"
                          name="rating"
                          value={rating}
                          checked={filters.rating === rating}
                          onChange={(e) => handleFilterChange('rating', parseInt(e.target.value))}
                          className="mr-2"
                        />
                        <div className="flex items-center">
                          {rating > 0 && (
                            <>
                              {Array.from({ length: rating }).map((_, i) => (
                                <Star key={i} className="h-4 w-4 text-yellow-400 fill-current" />
                              ))}
                              <span className="text-sm ml-2">& up</span>
                            </>
                          )}
                          {rating === 0 && <span className="text-sm">Any rating</span>}
                        </div>
                      </label>
                    ))}
                  </div>
                </div>

                {/* Platform Filter */}
                <div>
                  <h3 className="font-semibold text-gray-900 mb-3">Platform</h3>
                  <div className="space-y-2">
                    {['web', 'mobile', 'desktop', 'cross-platform'].map((platform) => (
                      <label key={platform} className="flex items-center">
                        <input
                          type="checkbox"
                          checked={filters.platform.includes(platform)}
                          onChange={(e) => {
                            const newPlatforms = e.target.checked
                              ? [...filters.platform, platform]
                              : filters.platform.filter(p => p !== platform);
                            handleFilterChange('platform', newPlatforms);
                          }}
                          className="mr-2"
                        />
                        <span className="text-sm capitalize">{platform}</span>
                      </label>
                    ))}
                  </div>
                </div>

                {/* Categories */}
                <div>
                  <h3 className="font-semibold text-gray-900 mb-3">Categories</h3>
                  <div className="space-y-2 max-h-48 overflow-y-auto">
                    {categories.map((category) => (
                      <label key={category.id} className="flex items-center">
                        <input
                          type="checkbox"
                          checked={filters.categories.includes(category.id)}
                          onChange={(e) => {
                            const newCategories = e.target.checked
                              ? [...filters.categories, category.id]
                              : filters.categories.filter(c => c !== category.id);
                            handleFilterChange('categories', newCategories);
                          }}
                          className="mr-2"
                        />
                        <div className="flex items-center space-x-2">
                          <span>{category.icon}</span>
                          <span className="text-sm">{category.name}</span>
                        </div>
                      </label>
                    ))}
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Apps Grid/List */}
          <div className={showFilters ? "lg:col-span-3 mt-8 lg:mt-0" : "lg:col-span-4"}>
            {loading ? (
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
                {Array.from({ length: 8 }).map((_, i) => (
                  <div key={i} className="bg-white rounded-lg shadow-md p-4 animate-pulse">
                    <div className="flex items-center space-x-3">
                      <div className="w-12 h-12 bg-gray-200 rounded-lg"></div>
                      <div className="flex-1">
                        <div className="h-4 bg-gray-200 rounded mb-2"></div>
                        <div className="h-3 bg-gray-200 rounded w-2/3"></div>
                      </div>
                    </div>
                    <div className="mt-3 space-y-2">
                      <div className="h-3 bg-gray-200 rounded"></div>
                      <div className="h-3 bg-gray-200 rounded w-3/4"></div>
                    </div>
                  </div>
                ))}
              </div>
            ) : apps.length === 0 ? (
              <div className="text-center py-12">
                <Search className="h-16 w-16 text-gray-300 mx-auto mb-4" />
                <h3 className="text-lg font-medium text-gray-900 mb-2">No apps found</h3>
                <p className="text-gray-600">Try adjusting your search criteria or filters</p>
              </div>
            ) : (
              <>
                <div className={
                  viewMode === 'grid' 
                    ? "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6"
                    : "space-y-4"
                }>
                  {apps.map((app) => 
                    viewMode === 'grid' ? (
                      <AppCard key={app.id} app={app} />
                    ) : (
                      <AppListItem key={app.id} app={app} />
                    )
                  )}
                </div>

                {/* Pagination */}
                {totalPages > 1 && (
                  <div className="flex justify-center mt-8">
                    <nav className="flex items-center space-x-2">
                      <button
                        onClick={() => setCurrentPage(prev => Math.max(prev - 1, 1))}
                        disabled={currentPage === 1}
                        className="px-3 py-2 border border-gray-300 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-50"
                      >
                        Previous
                      </button>
                      
                      {Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
                        const page = i + 1;
                        return (
                          <button
                            key={page}
                            onClick={() => setCurrentPage(page)}
                            className={`px-3 py-2 border rounded-lg ${
                              currentPage === page
                                ? 'bg-blue-600 text-white border-blue-600'
                                : 'border-gray-300 hover:bg-gray-50'
                            }`}
                          >
                            {page}
                          </button>
                        );
                      })}
                      
                      <button
                        onClick={() => setCurrentPage(prev => Math.min(prev + 1, totalPages))}
                        disabled={currentPage === totalPages}
                        className="px-3 py-2 border border-gray-300 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-50"
                      >
                        Next
                      </button>
                    </nav>
                  </div>
                )}
              </>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default AppsPage;