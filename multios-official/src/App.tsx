import React from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import Navbar from './components/Navbar';
import Footer from './components/Footer';
import HomePage from './pages/HomePage';
import DemosPage from './pages/DemosPage';
import FeaturesPage from './pages/FeaturesPage';
import DownloadPage from './pages/DownloadPage';
import EducatorsPage from './pages/EducatorsPage';
import DevelopersPage from './pages/DevelopersPage';
import CommunityPage from './pages/CommunityPage';
import ResearchPage from './pages/ResearchPage';
import BlogPage from './pages/BlogPage';
import AboutPage from './pages/AboutPage';

function App() {
  return (
    <Router>
      <div className="min-h-screen bg-white">
        <Navbar />
        <main>
          <Routes>
            <Route path="/" element={<HomePage />} />
            <Route path="/demos" element={<DemosPage />} />
            <Route path="/features" element={<FeaturesPage />} />
            <Route path="/download" element={<DownloadPage />} />
            <Route path="/educators" element={<EducatorsPage />} />
            <Route path="/developers" element={<DevelopersPage />} />
            <Route path="/community" element={<CommunityPage />} />
            <Route path="/research" element={<ResearchPage />} />
            <Route path="/blog" element={<BlogPage />} />
            <Route path="/about" element={<AboutPage />} />
          </Routes>
        </main>
        <Footer />
      </div>
    </Router>
  );
}

export default App;
