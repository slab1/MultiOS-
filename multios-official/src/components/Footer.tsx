import React from 'react';
import { Link } from 'react-router-dom';

const Footer = () => {
  return (
    <footer className="bg-surface py-16">
      <div className="container">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-8 mb-8">
          {/* Logo and Description */}
          <div className="col-span-1 md:col-span-2">
            <Link to="/" className="text-2xl font-bold text-black mb-4 block">
              MultiOS
            </Link>
            <p className="text-body mb-4 max-w-md">
              A revolutionary, educational, and production-ready operating system written entirely in Rust, 
              designed to run seamlessly across multiple CPU architectures.
            </p>
            <div className="flex space-x-4">
              <a 
                href="https://github.com/multios" 
                target="_blank" 
                rel="noopener noreferrer"
                className="text-sm font-bold uppercase tracking-wider text-gray-600 hover:text-black transition-colors"
              >
                GitHub
              </a>
              <a 
                href="https://discord.gg/multios" 
                target="_blank" 
                rel="noopener noreferrer"
                className="text-sm font-bold uppercase tracking-wider text-gray-600 hover:text-black transition-colors"
              >
                Discord
              </a>
              <a 
                href="mailto:hello@multios.org"
                className="text-sm font-bold uppercase tracking-wider text-gray-600 hover:text-black transition-colors"
              >
                Contact
              </a>
            </div>
          </div>

          {/* Quick Links */}
          <div>
            <h4 className="text-small font-bold uppercase tracking-wider text-black mb-4">
              Project
            </h4>
            <ul className="space-y-2">
              <li>
                <Link to="/features" className="text-body text-gray-600 hover:text-black transition-colors">
                  Features
                </Link>
              </li>
              <li>
                <Link to="/download" className="text-body text-gray-600 hover:text-black transition-colors">
                  Download
                </Link>
              </li>
              <li>
                <Link to="/demos" className="text-body text-gray-600 hover:text-black transition-colors">
                  Demos
                </Link>
              </li>
              <li>
                <Link to="/research" className="text-body text-gray-600 hover:text-black transition-colors">
                  Research
                </Link>
              </li>
            </ul>
          </div>

          {/* Community */}
          <div>
            <h4 className="text-small font-bold uppercase tracking-wider text-black mb-4">
              Community
            </h4>
            <ul className="space-y-2">
              <li>
                <Link to="/developers" className="text-body text-gray-600 hover:text-black transition-colors">
                  For Developers
                </Link>
              </li>
              <li>
                <Link to="/educators" className="text-body text-gray-600 hover:text-black transition-colors">
                  For Educators
                </Link>
              </li>
              <li>
                <Link to="/community" className="text-body text-gray-600 hover:text-black transition-colors">
                  Get Involved
                </Link>
              </li>
              <li>
                <Link to="/blog" className="text-body text-gray-600 hover:text-black transition-colors">
                  Blog
                </Link>
              </li>
            </ul>
          </div>
        </div>

        {/* Bottom Bar */}
        <div className="border-t border-gray-light pt-8">
          <div className="flex flex-col md:flex-row justify-between items-center">
            <p className="text-small text-gray-600 mb-4 md:mb-0">
              © 2025 MultiOS Project. Licensed under MIT.
            </p>
            <div className="flex space-x-6">
              <Link to="/about" className="text-small text-gray-600 hover:text-black transition-colors">
                About
              </Link>
              <a 
                href="/privacy" 
                className="text-small text-gray-600 hover:text-black transition-colors"
              >
                Privacy
              </a>
              <a 
                href="/terms" 
                className="text-small text-gray-600 hover:text-black transition-colors"
              >
                Terms
              </a>
            </div>
          </div>
        </div>
      </div>
    </footer>
  );
};

export default Footer;