import React, { useState, useEffect } from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { AuthProvider, useAuth } from './contexts/AuthContext';
import { NotificationProvider } from './contexts/NotificationContext';
import { Toaster } from 'react-hot-toast';

// Components
import Navbar from './components/Layout/Navbar';
import Sidebar from './components/Layout/Sidebar';
import LoadingSpinner from './components/Common/LoadingSpinner';
import PrivateRoute from './components/Common/PrivateRoute';

// Pages
import Login from './pages/Auth/Login';
import Register from './pages/Auth/Register';
import ForgotPassword from './pages/Auth/ForgotPassword';
import Dashboard from './pages/Dashboard/Dashboard';
import PapersList from './pages/Papers/PapersList';
import PaperDetail from './pages/Papers/PaperDetail';
import CreatePaper from './pages/Papers/CreatePaper';
import EditPaper from './pages/Papers/EditPaper';
import ReviewsList from './pages/Reviews/ReviewsList';
import ReviewDetail from './pages/Reviews/ReviewDetail';
import SubmitReview from './pages/Reviews/SubmitReview';
import ConferencesList from './pages/Conferences/ConferencesList';
import ConferenceDetail from './pages/Conferences/ConferenceDetail';
import CreateConference from './pages/Conferences/CreateConference';
import CitationsList from './pages/Citations/CitationsList';
import AnalyticsDashboard from './pages/Analytics/AnalyticsDashboard';
import UserProfile from './pages/Users/UserProfile';
import UsersList from './pages/Users/UsersList';
import LaTeXEditor from './pages/LaTeX/LaTeXEditor';
import SearchPapers from './pages/Search/SearchPapers';
import NotFound from './pages/Common/NotFound';

// Styles
import './App.css';

function AppLayout() {
  const { user, loading } = useAuth();
  const [sidebarOpen, setSidebarOpen] = useState(false);

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (!user) {
    return (
      <div className="min-h-screen bg-gray-50">
        <Routes>
          <Route path="/login" element={<Login />} />
          <Route path="/register" element={<Register />} />
          <Route path="/forgot-password" element={<ForgotPassword />} />
          <Route path="*" element={<Navigate to="/login" replace />} />
        </Routes>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <Navbar onMenuClick={() => setSidebarOpen(!sidebarOpen)} />
      
      <div className="flex">
        <Sidebar isOpen={sidebarOpen} onClose={() => setSidebarOpen(false)} />
        
        <main className="flex-1 lg:ml-64">
          <div className="px-4 py-6 sm:px-6 lg:px-8">
            <Routes>
              {/* Dashboard */}
              <Route path="/" element={<Dashboard />} />
              
              {/* Papers */}
              <Route path="/papers" element={<PapersList />} />
              <Route path="/papers/create" element={<CreatePaper />} />
              <Route path="/papers/:id" element={<PaperDetail />} />
              <Route path="/papers/:id/edit" element={<EditPaper />} />
              
              {/* Reviews */}
              <Route path="/reviews" element={<ReviewsList />} />
              <Route path="/reviews/:id" element={<ReviewDetail />} />
              <Route path="/reviews/:id/submit" element={<SubmitReview />} />
              
              {/* Conferences */}
              <Route path="/conferences" element={<ConferencesList />} />
              <Route path="/conferences/:id" element={<ConferenceDetail />} />
              <Route path="/conferences/create" element={<CreateConference />} />
              
              {/* Citations */}
              <Route path="/citations" element={<CitationsList />} />
              
              {/* Analytics */}
              <Route path="/analytics" element={<AnalyticsDashboard />} />
              
              {/* Users */}
              <Route path="/profile" element={<UserProfile />} />
              <Route path="/users" element={<UsersList />} />
              
              {/* Tools */}
              <Route path="/latex-editor" element={<LaTeXEditor />} />
              <Route path="/search" element={<SearchPapers />} />
              
              {/* 404 */}
              <Route path="*" element={<NotFound />} />
            </Routes>
          </div>
        </main>
      </div>
      
      <Toaster
        position="top-right"
        toastOptions={{
          duration: 4000,
          style: {
            background: '#363636',
            color: '#fff',
          },
        }}
      />
    </div>
  );
}

function App() {
  return (
    <Router>
      <AuthProvider>
        <NotificationProvider>
          <AppLayout />
        </NotificationProvider>
      </AuthProvider>
    </Router>
  );
}

export default App;
