import React, { createContext, useContext } from 'react';
import toast, { Toast } from 'react-hot-toast';

// Types
interface NotificationContextType {
  showSuccess: (message: string) => void;
  showError: (message: string) => void;
  showInfo: (message: string) => void;
  showWarning: (message: string) => void;
  showLoading: (message: string) => string;
  dismiss: (toastId: string) => void;
  custom: (element: React.ReactNode, options?: any) => string;
}

// Create context
const NotificationContext = createContext<NotificationContextType | undefined>(undefined);

// Notification provider component
export function NotificationProvider({ children }: { children: React.ReactNode }) {
  const showSuccess = (message: string): void => {
    toast.success(message, {
      duration: 4000,
      style: {
        background: '#10B981',
        color: '#fff',
        fontSize: '14px',
      },
      iconTheme: {
        primary: '#fff',
        secondary: '#10B981',
      },
    });
  };

  const showError = (message: string): void => {
    toast.error(message, {
      duration: 6000,
      style: {
        background: '#EF4444',
        color: '#fff',
        fontSize: '14px',
      },
      iconTheme: {
        primary: '#fff',
        secondary: '#EF4444',
      },
    });
  };

  const showInfo = (message: string): void => {
    toast(message, {
      duration: 4000,
      style: {
        background: '#3B82F6',
        color: '#fff',
        fontSize: '14px',
      },
      icon: 'ℹ️',
    });
  };

  const showWarning = (message: string): void => {
    toast(message, {
      duration: 5000,
      style: {
        background: '#F59E0B',
        color: '#fff',
        fontSize: '14px',
      },
      icon: '⚠️',
    });
  };

  const showLoading = (message: string): string => {
    return toast.loading(message, {
      style: {
        background: '#6B7280',
        color: '#fff',
        fontSize: '14px',
      },
    });
  };

  const dismiss = (toastId: string): void => {
    toast.dismiss(toastId);
  };

  const custom = (element: React.ReactNode, options?: any): string => {
    return toast(element, options);
  };

  const value: NotificationContextType = {
    showSuccess,
    showError,
    showInfo,
    showWarning,
    showLoading,
    dismiss,
    custom,
  };

  return (
    <NotificationContext.Provider value={value}>
      {children}
    </NotificationContext.Provider>
  );
}

// Hook to use notification context
export function useNotifications(): NotificationContextType {
  const context = useContext(NotificationContext);
  if (context === undefined) {
    throw new Error('useNotifications must be used within a NotificationProvider');
  }
  return context;
}

export default NotificationContext;