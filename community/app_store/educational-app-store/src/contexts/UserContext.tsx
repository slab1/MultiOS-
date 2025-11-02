// User context for managing user-specific data

import React, { createContext, useContext, useState, useEffect } from 'react';
import { useAuth } from './AuthContext';

interface FavoriteApp {
  id: string;
  title: string;
  icon: string;
  rating: number;
  category_name: string;
  favorited_at: string;
}

interface UserContextType {
  favorites: FavoriteApp[];
  downloads: any[];
  reviews: any[];
  addToFavorites: (appId: string) => Promise<boolean>;
  removeFromFavorites: (appId: string) => Promise<boolean>;
  loadFavorites: () => Promise<void>;
  loadDownloads: () => Promise<void>;
  loadReviews: () => Promise<void>;
  loading: boolean;
}

const UserContext = createContext<UserContextType | undefined>(undefined);

export const useUser = () => {
  const context = useContext(UserContext);
  if (context === undefined) {
    throw new Error('useUser must be used within a UserProvider');
  }
  return context;
};

export const UserProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const { user, token, isAuthenticated } = useAuth();
  const [favorites, setFavorites] = useState<FavoriteApp[]>([]);
  const [downloads, setDownloads] = useState<any[]>([]);
  const [reviews, setReviews] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);

  const API_BASE = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3001/api';

  const authHeaders = token ? {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json',
  } : {};

  const loadFavorites = async () => {
    if (!user || !isAuthenticated) return;

    try {
      const response = await fetch(`${API_BASE}/users/${user.id}/favorites`, {
        headers: authHeaders,
      });

      const data = await response.json();
      if (data.success) {
        setFavorites(data.data.items || []);
      }
    } catch (error) {
      console.error('Error loading favorites:', error);
    }
  };

  const loadDownloads = async () => {
    if (!user || !isAuthenticated) return;

    try {
      const response = await fetch(`${API_BASE}/analytics/educator/${user.id}`, {
        headers: authHeaders,
      });

      const data = await response.json();
      if (data.success) {
        setDownloads(data.data.downloadedApps || []);
      }
    } catch (error) {
      console.error('Error loading downloads:', error);
    }
  };

  const loadReviews = async () => {
    if (!user || !isAuthenticated) return;

    try {
      const response = await fetch(`${API_BASE}/users/${user.id}/reviews`, {
        headers: authHeaders,
      });

      const data = await response.json();
      if (data.success) {
        setReviews(data.data.items || []);
      }
    } catch (error) {
      console.error('Error loading reviews:', error);
    }
  };

  const addToFavorites = async (appId: string): Promise<boolean> => {
    if (!user || !isAuthenticated) return false;

    try {
      const response = await fetch(`${API_BASE}/users/${user.id}/favorites`, {
        method: 'POST',
        headers: authHeaders,
        body: JSON.stringify({ appId }),
      });

      const data = await response.json();
      if (data.success) {
        await loadFavorites(); // Reload favorites
        return true;
      }
      return false;
    } catch (error) {
      console.error('Error adding to favorites:', error);
      return false;
    }
  };

  const removeFromFavorites = async (appId: string): Promise<boolean> => {
    if (!user || !isAuthenticated) return false;

    try {
      const response = await fetch(`${API_BASE}/users/${user.id}/favorites/${appId}`, {
        method: 'DELETE',
        headers: authHeaders,
      });

      const data = await response.json();
      if (data.success) {
        await loadFavorites(); // Reload favorites
        return true;
      }
      return false;
    } catch (error) {
      console.error('Error removing from favorites:', error);
      return false;
    }
  };

  // Load user data when authenticated user changes
  useEffect(() => {
    if (isAuthenticated && user) {
      setLoading(true);
      Promise.all([
        loadFavorites(),
        loadDownloads(),
        loadReviews(),
      ]).finally(() => {
        setLoading(false);
      });
    } else {
      // Clear data when user logs out
      setFavorites([]);
      setDownloads([]);
      setReviews([]);
    }
  }, [user, isAuthenticated]);

  const value: UserContextType = {
    favorites,
    downloads,
    reviews,
    addToFavorites,
    removeFromFavorites,
    loadFavorites,
    loadDownloads,
    loadReviews,
    loading,
  };

  return <UserContext.Provider value={value}>{children}</UserContext.Provider>;
};