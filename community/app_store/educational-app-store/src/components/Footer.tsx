import React from 'react';
import { Link } from 'react-router-dom';
import { 
  BookOpen, 
  Mail, 
  Phone, 
  MapPin, 
  Github, 
  Twitter, 
  Linkedin,
  Heart
} from 'lucide-react';

const Footer: React.FC = () => {
  return (
    <footer className="bg-gray-900 text-white">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
          {/* Brand Section */}
          <div className="col-span-1 md:col-span-1">
            <Link to="/" className="flex items-center space-x-2 mb-4">
              <BookOpen className="h-8 w-8 text-blue-400" />
              <span className="text-xl font-bold">EduStore</span>
            </Link>
            <p className="text-gray-400 mb-4">
              Curating the best educational applications for learners and educators worldwide.
            </p>
            <div className="flex space-x-4">
              <a href="#" className="text-gray-400 hover:text-white transition-colors">
                <Github className="h-5 w-5" />
              </a>
              <a href="#" className="text-gray-400 hover:text-white transition-colors">
                <Twitter className="h-5 w-5" />
              </a>
              <a href="#" className="text-gray-400 hover:text-white transition-colors">
                <Linkedin className="h-5 w-5" />
              </a>
            </div>
          </div>

          {/* Quick Links */}
          <div>
            <h3 className="text-lg font-semibold mb-4">Quick Links</h3>
            <ul className="space-y-2">
              <li>
                <Link to="/apps" className="text-gray-400 hover:text-white transition-colors">
                  Browse Apps
                </Link>
              </li>
              <li>
                <Link to="/categories" className="text-gray-400 hover:text-white transition-colors">
                  Categories
                </Link>
              </li>
              <li>
                <Link to="/popular" className="text-gray-400 hover:text-white transition-colors">
                  Popular Apps
                </Link>
              </li>
              <li>
                <Link to="/featured" className="text-gray-400 hover:text-white transition-colors">
                  Featured Apps
                </Link>
              </li>
            </ul>
          </div>

          {/* For Educators */}
          <div>
            <h3 className="text-lg font-semibold mb-4">For Educators</h3>
            <ul className="space-y-2">
              <li>
                <Link to="/register?role=educator" className="text-gray-400 hover:text-white transition-colors">
                  Join as Educator
                </Link>
              </li>
              <li>
                <Link to="/educator-guide" className="text-gray-400 hover:text-white transition-colors">
                  Educator Guide
                </Link>
              </li>
              <li>
                <Link to="/classroom-integration" className="text-gray-400 hover:text-white transition-colors">
                  Classroom Integration
                </Link>
              </li>
              <li>
                <Link to="/educator-resources" className="text-gray-400 hover:text-white transition-colors">
                  Resources
                </Link>
              </li>
            </ul>
          </div>

          {/* For Developers */}
          <div>
            <h3 className="text-lg font-semibold mb-4">For Developers</h3>
            <ul className="space-y-2">
              <li>
                <Link to="/register?role=developer" className="text-gray-400 hover:text-white transition-colors">
                  Submit Your App
                </Link>
              </li>
              <li>
                <Link to="/developer-guide" className="text-gray-400 hover:text-white transition-colors">
                  Developer Guide
                </Link>
              </li>
              <li>
                <Link to="/api-docs" className="text-gray-400 hover:text-white transition-colors">
                  API Documentation
                </Link>
              </li>
              <li>
                <Link to="/submission-guidelines" className="text-gray-400 hover:text-white transition-colors">
                  Submission Guidelines
                </Link>
              </li>
            </ul>
          </div>
        </div>

        {/* Categories */}
        <div className="border-t border-gray-800 mt-8 pt-8">
          <h3 className="text-lg font-semibold mb-4">Popular Categories</h3>
          <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
            <Link to="/categories/mathematics" className="text-gray-400 hover:text-white transition-colors text-sm">
              Mathematics
            </Link>
            <Link to="/categories/science" className="text-gray-400 hover:text-white transition-colors text-sm">
              Science
            </Link>
            <Link to="/categories/language-arts" className="text-gray-400 hover:text-white transition-colors text-sm">
              Language Arts
            </Link>
            <Link to="/categories/social-studies" className="text-gray-400 hover:text-white transition-colors text-sm">
              Social Studies
            </Link>
            <Link to="/categories/art-music" className="text-gray-400 hover:text-white transition-colors text-sm">
              Art & Music
            </Link>
            <Link to="/categories/programming" className="text-gray-400 hover:text-white transition-colors text-sm">
              Programming
            </Link>
          </div>
        </div>

        {/* Contact Info */}
        <div className="border-t border-gray-800 mt-8 pt-8">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="flex items-center space-x-3">
              <Mail className="h-5 w-5 text-blue-400" />
              <span className="text-gray-400">support@edustore.com</span>
            </div>
            <div className="flex items-center space-x-3">
              <Phone className="h-5 w-5 text-blue-400" />
              <span className="text-gray-400">+1 (555) 123-4567</span>
            </div>
            <div className="flex items-center space-x-3">
              <MapPin className="h-5 w-5 text-blue-400" />
              <span className="text-gray-400">San Francisco, CA</span>
            </div>
          </div>
        </div>

        {/* Bottom Section */}
        <div className="border-t border-gray-800 mt-8 pt-8 flex flex-col md:flex-row justify-between items-center">
          <div className="flex items-center space-x-1 text-gray-400">
            <span>Made with</span>
            <Heart className="h-4 w-4 text-red-400 fill-current" />
            <span>for education</span>
          </div>
          
          <div className="flex space-x-6 mt-4 md:mt-0">
            <Link to="/privacy-policy" className="text-gray-400 hover:text-white transition-colors text-sm">
              Privacy Policy
            </Link>
            <Link to="/terms-of-service" className="text-gray-400 hover:text-white transition-colors text-sm">
              Terms of Service
            </Link>
            <Link to="/accessibility" className="text-gray-400 hover:text-white transition-colors text-sm">
              Accessibility
            </Link>
            <Link to="/help" className="text-gray-400 hover:text-white transition-colors text-sm">
              Help
            </Link>
          </div>
        </div>

        <div className="border-t border-gray-800 mt-4 pt-4 text-center">
          <p className="text-gray-400 text-sm">
            © 2024 EduStore. All rights reserved. Empowering education through technology.
          </p>
        </div>
      </div>
    </footer>
  );
};

export default Footer;