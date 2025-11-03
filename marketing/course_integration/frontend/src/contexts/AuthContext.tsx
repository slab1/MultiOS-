import React, { createContext, useContext, useReducer, useEffect, ReactNode } from 'react';

interface User {
  id: string;
  email: string;
  firstName: string;
  lastName: string;
  role: 'student' | 'instructor' | 'administrator' | 'super_admin';
  institutionId?: string;
  emailVerified: boolean;
  lastLogin?: string;
  timezone?: string;
  preferences?: Record<string, any>;
}

interface AuthTokens {
  accessToken: string;
  refreshToken: string;
}

interface AuthState {
  user: User | null;
  tokens: AuthTokens | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
}

type AuthAction =
  | { type: 'AUTH_START' }
  | { type: 'AUTH_SUCCESS'; payload: { user: User; tokens: AuthTokens } }
  | { type: 'AUTH_FAILURE'; payload: string }
  | { type: 'LOGOUT' }
  | { type: 'CLEAR_ERROR' }
  | { type: 'UPDATE_USER'; payload: Partial<User> }
  | { type: 'REFRESH_TOKENS'; payload: AuthTokens };

const initialState: AuthState = {
  user: null,
  tokens: null,
  isAuthenticated: false,
  isLoading: true,
  error: null,
};

function authReducer(state: AuthState, action: AuthAction): AuthState {
  switch (action.type) {
    case 'AUTH_START':
      return {
        ...state,
        isLoading: true,
        error: null,
      };
    case 'AUTH_SUCCESS':
      return {
        ...state,
        user: action.payload.user,
        tokens: action.payload.tokens,
        isAuthenticated: true,
        isLoading: false,
        error: null,
      };
    case 'AUTH_FAILURE':
      return {
        ...state,
        user: null,
        tokens: null,
        isAuthenticated: false,
        isLoading: false,
        error: action.payload,
      };
    case 'LOGOUT':
      return {
        ...state,
        user: null,
        tokens: null,
        isAuthenticated: false,
        isLoading: false,
        error: null,
      };
    case 'CLEAR_ERROR':
      return {
        ...state,
        error: null,
      };
    case 'UPDATE_USER':
      return {
        ...state,
        user: state.user ? { ...state.user, ...action.payload } : null,
      };
    case 'REFRESH_TOKENS':
      return {
        ...state,
        tokens: action.payload,
      };
    default:
      return state;
  }
}

interface AuthContextType {
  state: AuthState;
  login: (email: string, password: string) => Promise<void>;
  register: (userData: RegisterData) => Promise<void>;
  logout: () => void;
  clearError: () => void;
  updateUser: (userData: Partial<User>) => void;
  refreshTokens: () => Promise<boolean>;
  hasPermission: (requiredRoles: string | string[]) => boolean;
  hasRole: (role: string) => boolean;
}

interface RegisterData {
  email: string;
  password: string;
  firstName: string;
  lastName: string;
  role?: 'student' | 'instructor';
  institutionId?: string;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

interface AuthProviderProps {
  children: ReactNode;
}

export function AuthProvider({ children }: AuthProviderProps) {
  const [state, dispatch] = useReducer(authReducer, initialState);

  // API base URL
  const API_BASE_URL = process.env.REACT_APP_API_URL || 'http://localhost:5000/api';

  // Axios instance with interceptors
  const api = React.useMemo(() => {
    const instance = require('axios').default.create({
      baseURL: API_BASE_URL,
    });

    // Request interceptor to add auth token
    instance.interceptors.request.use(
      (config: any) => {
        const tokens = state.tokens;
        if (tokens?.accessToken) {
          config.headers.Authorization = `Bearer ${tokens.accessToken}`;
        }
        return config;
      },
      (error: any) => {
        return Promise.reject(error);
      }
    );

    // Response interceptor to handle token refresh
    instance.interceptors.response.use(
      (response: any) => {
        return response;
      },
      async (error: any) => {
        const originalRequest = error.config;

        if (error.response?.status === 401 && !originalRequest._retry) {
          originalRequest._retry = true;

          try {
            const refreshed = await refreshTokens();
            if (refreshed) {
              originalRequest.headers.Authorization = `Bearer ${state.tokens?.accessToken}`;
              return instance(originalRequest);
            }
          } catch (refreshError) {
            dispatch({ type: 'LOGOUT' });
            return Promise.reject(refreshError);
          }
        }

        return Promise.reject(error);
      }
    );

    return instance;
  }, [state.tokens]);

  // Check for existing session on mount
  useEffect(() => {
    const initializeAuth = async () => {
      const storedTokens = localStorage.getItem('authTokens');
      const storedUser = localStorage.getItem('user');

      if (storedTokens && storedUser) {
        try {
          const tokens = JSON.parse(storedTokens);
          const user = JSON.parse(storedUser);
          
          dispatch({
            type: 'AUTH_SUCCESS',
            payload: { user, tokens },
          });
        } catch (error) {
          // Clear invalid stored data
          localStorage.removeItem('authTokens');
          localStorage.removeItem('user');
          dispatch({ type: 'AUTH_FAILURE', payload: 'Invalid stored session' });
        }
      } else {
        dispatch({ type: 'AUTH_FAILURE', payload: 'No stored session' });
      }
    };

    initializeAuth();
  }, []);

  // Login function
  const login = async (email: string, password: string): Promise<void> => {
    try {
      dispatch({ type: 'AUTH_START' });

      const response = await api.post('/auth/login', {
        email,
        password,
      });

      const { user, tokens } = response.data;

      // Store in localStorage
      localStorage.setItem('authTokens', JSON.stringify(tokens));
      localStorage.setItem('user', JSON.stringify(user));

      dispatch({
        type: 'AUTH_SUCCESS',
        payload: { user, tokens },
      });
    } catch (error: any) {
      const errorMessage = error.response?.data?.error || 'Login failed';
      dispatch({ type: 'AUTH_FAILURE', payload: errorMessage });
      throw new Error(errorMessage);
    }
  };

  // Register function
  const register = async (userData: RegisterData): Promise<void> => {
    try {
      dispatch({ type: 'AUTH_START' });

      const response = await api.post('/auth/register', userData);

      const { user, tokens } = response.data;

      // Store in localStorage
      localStorage.setItem('authTokens', JSON.stringify(tokens));
      localStorage.setItem('user', JSON.stringify(user));

      dispatch({
        type: 'AUTH_SUCCESS',
        payload: { user, tokens },
      });
    } catch (error: any) {
      const errorMessage = error.response?.data?.error || 'Registration failed';
      dispatch({ type: 'AUTH_FAILURE', payload: errorMessage });
      throw new Error(errorMessage);
    }
  };

  // Logout function
  const logout = (): void => {
    // Clear localStorage
    localStorage.removeItem('authTokens');
    localStorage.removeItem('user');
    
    dispatch({ type: 'LOGOUT' });
  };

  // Clear error function
  const clearError = (): void => {
    dispatch({ type: 'CLEAR_ERROR' });
  };

  // Update user function
  const updateUser = (userData: Partial<User>): void => {
    const updatedUser = { ...state.user, ...userData } as User;
    localStorage.setItem('user', JSON.stringify(updatedUser));
    dispatch({ type: 'UPDATE_USER', payload: userData });
  };

  // Refresh tokens function
  const refreshTokens = async (): Promise<boolean> => {
    try {
      if (!state.tokens?.refreshToken) {
        return false;
      }

      const response = await api.post('/auth/refresh', {
        refreshToken: state.tokens.refreshToken,
      });

      const { tokens } = response.data;

      // Update stored tokens
      localStorage.setItem('authTokens', JSON.stringify(tokens));
      
      dispatch({ type: 'REFRESH_TOKENS', payload: tokens });
      
      return true;
    } catch (error) {
      return false;
    }
  };

  // Check permissions
  const hasPermission = (requiredRoles: string | string[]): boolean => {
    if (!state.user) return false;

    const roles = Array.isArray(requiredRoles) ? requiredRoles : [requiredRoles];
    return roles.includes(state.user.role);
  };

  // Check specific role
  const hasRole = (role: string): boolean => {
    return state.user?.role === role;
  };

  const contextValue: AuthContextType = {
    state,
    login,
    register,
    logout,
    clearError,
    updateUser,
    refreshTokens,
    hasPermission,
    hasRole,
  };

  return (
    <AuthContext.Provider value={contextValue}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthContextType {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}