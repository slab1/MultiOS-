import React, { useState, useEffect } from 'react';
import { Link, useParams } from 'react-router-dom';
import { Grid, List, Filter, Search } from 'lucide-react';

interface Category {
  id: string;
  name: string;
  description: string;
  icon: string;
  color: string;
  app_count: number;
  avg_rating: number;
  subcategories?: Subcategory[];
}

interface Subcategory {
  id: string;
  name: string;
  description: string;
  app_count: number;
}

interface App {
  id: string;
  title: string;
  icon: string;
  rating: number;
  download_count: number;
  price: number;
  developer_name: string;
}

const CategoriesPage: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const [categories, setCategories] = useState<Category[]>([]);
  const [selectedCategory, setSelectedCategory] = useState<Category | null>(null);
  const [apps, setApps] = useState<App[]>([]);
  const [loading, setLoading] = useState(true);
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');

  const API_BASE = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3001/api';

  useEffect(() => {
    loadCategories();
  }, []);

  useEffect(() => {
    if (id && categories.length > 0) {
      const category = categories.find(c => c.id === id);
      if (category) {
        setSelectedCategory(category);
        loadCategoryApps(id);
      }
    }
  }, [id, categories]);

  const loadCategories = async () => {
    try {
      const response = await fetch(`${API_BASE}/categories?includeSubcategories=true`);
      const data = await response.json();
      if (data.success) {
        setCategories(data.data);
      }
    } catch (error) {
      console.error('Error loading categories:', error);
    } finally {
      setLoading(false);
    }
  };

  const loadCategoryApps = async (categoryId: string) => {
    try {
      const response = await fetch(`${API_BASE}/categories/${categoryId}/apps?limit=20`);
      const data = await response.json();
      if (data.success) {
        setApps(data.data.items);
      }
    } catch (error) {
      console.error('Error loading category apps:', error);
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

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <h1 className="text-3xl font-bold text-gray-900 mb-8">
          {selectedCategory ? selectedCategory.name : 'Categories'}
        </h1>

        {selectedCategory ? (
          <div>
            <div className="mb-6">
              <Link to="/categories" className="text-blue-600 hover:text-blue-700">
                ← Back to all categories
              </Link>
            </div>

            <div className="bg-white rounded-lg shadow-md p-6 mb-8">
              <div className="flex items-center space-x-4">
                <div 
                  className="w-16 h-16 rounded-full flex items-center justify-center text-2xl"
                  style={{ backgroundColor: `${selectedCategory.color}20` }}
                >
                  {selectedCategory.icon}
                </div>
                <div>
                  <h2 className="text-2xl font-bold text-gray-900">{selectedCategory.name}</h2>
                  <p className="text-gray-600">{selectedCategory.description}</p>
                  <p className="text-sm text-gray-500 mt-1">{selectedCategory.app_count} apps available</p>
                </div>
              </div>
            </div>

            {selectedCategory.subcategories && selectedCategory.subcategories.length > 0 && (
              <div className="mb-8">
                <h3 className="text-lg font-semibold text-gray-900 mb-4">Subcategories</h3>
                <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
                  {selectedCategory.subcategories.map((subcategory) => (
                    <div key={subcategory.id} className="bg-white rounded-lg p-4 shadow-md hover:shadow-lg transition-shadow">
                      <h4 className="font-semibold text-gray-900 mb-2">{subcategory.name}</h4>
                      <p className="text-sm text-gray-600 mb-3">{subcategory.description}</p>
                      <p className="text-xs text-blue-600">{subcategory.app_count} apps</p>
                    </div>
                  ))}
                </div>
              </div>
            )}

            <div className="bg-white rounded-lg shadow-md p-6">
              <div className="flex items-center justify-between mb-6">
                <h3 className="text-lg font-semibold text-gray-900">
                  Apps in {selectedCategory.name}
                </h3>
                <div className="flex items-center space-x-2">
                  <button
                    onClick={() => setViewMode('grid')}
                    className={`p-2 ${viewMode === 'grid' ? 'bg-blue-600 text-white' : 'text-gray-600'}`}
                  >
                    <Grid className="h-4 w-4" />
                  </button>
                  <button
                    onClick={() => setViewMode('list')}
                    className={`p-2 ${viewMode === 'list' ? 'bg-blue-600 text-white' : 'text-gray-600'}`}
                  >
                    <List className="h-4 w-4" />
                  </button>
                </div>
              </div>

              <div className={viewMode === 'grid' ? "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6" : "space-y-4"}>
                {apps.map((app) => (
                  <Link
                    key={app.id}
                    to={`/apps/${app.id}`}
                    className="bg-white rounded-lg border border-gray-200 hover:shadow-md transition-shadow p-4"
                  >
                    <div className="flex items-center space-x-3">
                      <img
                        src={app.icon || '/placeholder-app-icon.png'}
                        alt={app.title}
                        className="w-12 h-12 rounded-lg object-cover"
                      />
                      <div className="flex-1">
                        <h4 className="font-semibold text-gray-900 truncate">{app.title}</h4>
                        <p className="text-sm text-gray-600">{app.developer_name}</p>
                        <div className="flex items-center space-x-2 mt-1">
                          <span className="text-sm text-yellow-600">★ {app.rating.toFixed(1)}</span>
                          <span className="text-sm text-gray-500">• {formatNumber(app.download_count)} downloads</span>
                          <span className={`text-sm ${app.price === 0 ? 'text-green-600' : 'text-gray-900'}`}>
                            {formatPrice(app.price)}
                          </span>
                        </div>
                      </div>
                    </div>
                  </Link>
                ))}
              </div>
            </div>
          </div>
        ) : (
          <div>
            <p className="text-gray-600 mb-8">Browse educational apps by subject area</p>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
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
                  <h3 className="font-semibold text-gray-900 group-hover:text-blue-600 transition-colors mb-2">
                    {category.name}
                  </h3>
                  <p className="text-sm text-gray-600 mb-4">{category.description}</p>
                  <div className="text-sm text-blue-600">
                    {category.app_count} apps
                  </div>
                </Link>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default CategoriesPage;