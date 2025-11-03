import React from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { ErrorBoundary } from './components/ErrorBoundary';
import Navbar from './components/Navbar';
import Footer from './components/Footer';
import HomePage from './pages/HomePage';
import FeaturesPage from './pages/FeaturesPage';
import DemosPage from './pages/DemosPage';
import DocumentationPage from './pages/DocumentationPage';
import DownloadPage from './pages/DownloadPage';
import CommunityPage from './pages/CommunityPage';
import EducationPage from './pages/EducationPage';
import BlogPage from './pages/BlogPage';
import AboutPage from './pages/AboutPage';
import ContactPage from './pages/ContactPage';
import { LanguageProvider } from './contexts/LanguageContext';
import './App.css';

function App() {
  return (
    <ErrorBoundary>
      <LanguageProvider>
        <Router>
          <div className="min-h-screen bg-gradient-to-br from-blue-50 via-indigo-50 to-purple-50">
            <Navbar />
            <main className="pt-16">
              <Routes>
                <Route path="/" element={<HomePage />} />
                <Route path="/features" element={<FeaturesPage />} />
                <Route path="/demos" element={<DemosPage />} />
                <Route path="/documentation" element={<DocumentationPage />} />
                <Route path="/download" element={<DownloadPage />} />
                <Route path="/community" element={<CommunityPage />} />
                <Route path="/education" element={<EducationPage />} />
                <Route path="/blog" element={<BlogPage />} />
                <Route path="/about" element={<AboutPage />} />
                <Route path="/contact" element={<ContactPage />} />
              </Routes>
            </main>
            <Footer />
          </div>
        </Router>
      </LanguageProvider>
    </ErrorBoundary>
  );
}

export default App;
