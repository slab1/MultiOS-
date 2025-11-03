import React from 'react';
import { Github, MessageCircle, Users, Star, GitFork, ExternalLink, Mail } from 'lucide-react';

const CommunityPage: React.FC = () => {
  return (
    <div className="min-h-screen bg-white">
      {/* Hero Section */}
      <section className="bg-gradient-to-br from-blue-600 via-purple-600 to-indigo-800 py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center text-white">
          <h1 className="text-4xl md:text-5xl font-bold mb-6">Join Our Community</h1>
          <p className="text-xl md:text-2xl text-blue-100 max-w-3xl mx-auto">
            Connect with thousands of developers, students, and educators advancing operating systems
          </p>
        </div>
      </section>

      {/* Community Stats */}
      <section className="py-12 bg-gray-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid md:grid-cols-4 gap-6 text-center">
            <div className="bg-white p-6 rounded-lg shadow-md">
              <Star className="w-8 h-8 text-yellow-500 mx-auto mb-2" />
              <div className="text-2xl font-bold text-gray-900">15.2k</div>
              <div className="text-gray-600">GitHub Stars</div>
            </div>
            <div className="bg-white p-6 rounded-lg shadow-md">
              <Users className="w-8 h-8 text-blue-500 mx-auto mb-2" />
              <div className="text-2xl font-bold text-gray-900">5.8k</div>
              <div className="text-gray-600">Contributors</div>
            </div>
            <div className="bg-white p-6 rounded-lg shadow-md">
              <MessageCircle className="w-8 h-8 text-green-500 mx-auto mb-2" />
              <div className="text-2xl font-bold text-gray-900">12.5k</div>
              <div className="text-gray-600">Discord Members</div>
            </div>
            <div className="bg-white p-6 rounded-lg shadow-md">
              <GitFork className="w-8 h-8 text-purple-500 mx-auto mb-2" />
              <div className="text-2xl font-bold text-gray-900">2.1k</div>
              <div className="text-gray-600">Forks</div>
            </div>
          </div>
        </div>
      </section>

      {/* Community Platforms */}
      <section className="py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <h2 className="text-3xl font-bold text-center text-gray-900 mb-12">Connect With Us</h2>
          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
            <div className="bg-white p-6 rounded-xl shadow-lg hover:shadow-xl transition-shadow text-center">
              <Github className="w-12 h-12 text-gray-900 mx-auto mb-4" />
              <h3 className="text-xl font-semibold text-gray-900 mb-2">GitHub</h3>
              <p className="text-gray-600 mb-4">Contribute code, report issues, and collaborate on development</p>
              <a href="https://github.com/multios-org" className="inline-flex items-center text-blue-600 hover:text-blue-700">
                Visit GitHub <ExternalLink className="w-4 h-4 ml-1" />
              </a>
            </div>

            <div className="bg-white p-6 rounded-xl shadow-lg hover:shadow-xl transition-shadow text-center">
              <MessageCircle className="w-12 h-12 text-blue-500 mx-auto mb-4" />
              <h3 className="text-xl font-semibold text-gray-900 mb-2">Discord</h3>
              <p className="text-gray-600 mb-4">Real-time chat, support, and community discussions</p>
              <a href="https://discord.gg/multios" className="inline-flex items-center text-blue-600 hover:text-blue-700">
                Join Discord <ExternalLink className="w-4 h-4 ml-1" />
              </a>
            </div>

            <div className="bg-white p-6 rounded-xl shadow-lg hover:shadow-xl transition-shadow text-center">
              <Mail className="w-12 h-12 text-green-500 mx-auto mb-4" />
              <h3 className="text-xl font-semibold text-gray-900 mb-2">Mailing List</h3>
              <p className="text-gray-600 mb-4">Stay updated with announcements and discussions</p>
              <a href="mailto:community@multios.org" className="inline-flex items-center text-blue-600 hover:text-blue-700">
                Subscribe <ExternalLink className="w-4 h-4 ml-1" />
              </a>
            </div>
          </div>
        </div>
      </section>

      {/* Contribution Guide */}
      <section className="py-20 bg-gray-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <h2 className="text-3xl font-bold text-center text-gray-900 mb-12">How to Contribute</h2>
          <div className="grid md:grid-cols-2 gap-12">
            <div>
              <h3 className="text-xl font-semibold text-gray-900 mb-4">Code Contributions</h3>
              <ul className="space-y-3">
                <li className="flex items-start">
                  <span className="text-green-500 mr-2">✓</span>
                  Fix bugs and improve performance
                </li>
                <li className="flex items-start">
                  <span className="text-green-500 mr-2">✓</span>
                  Add new features and drivers
                </li>
                <li className="flex items-start">
                  <span className="text-green-500 mr-2">✓</span>
                  Improve documentation
                </li>
                <li className="flex items-start">
                  <span className="text-green-500 mr-2">✓</span>
                  Write comprehensive tests
                </li>
              </ul>
            </div>
            <div>
              <h3 className="text-xl font-semibold text-gray-900 mb-4">Community Contributions</h3>
              <ul className="space-y-3">
                <li className="flex items-start">
                  <span className="text-blue-500 mr-2">✓</span>
                  Help other users and students
                </li>
                <li className="flex items-start">
                  <span className="text-blue-500 mr-2">✓</span>
                  Create educational content
                </li>
                <li className="flex items-start">
                  <span className="text-blue-500 mr-2">✓</span>
                  Translate documentation
                </li>
                <li className="flex items-start">
                  <span className="text-blue-500 mr-2">✓</span>
                  Organize events and workshops
                </li>
              </ul>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-20 bg-gradient-to-r from-blue-600 to-purple-600">
        <div className="max-w-4xl mx-auto text-center px-4 sm:px-6 lg:px-8">
          <h2 className="text-3xl font-bold text-white mb-6">Ready to Contribute?</h2>
          <p className="text-xl text-blue-100 mb-8">
            Join our community of developers and make your mark on operating systems education
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <a
              href="https://github.com/multios-org"
              className="inline-flex items-center px-8 py-4 bg-white text-blue-600 font-semibold rounded-lg hover:bg-blue-50 transition-colors"
            >
              <Github className="w-5 h-5 mr-2" />
              Start Contributing
            </a>
            <a
              href="https://discord.gg/multios"
              className="inline-flex items-center px-8 py-4 bg-transparent border-2 border-white text-white font-semibold rounded-lg hover:bg-white hover:text-blue-600 transition-colors"
            >
              <MessageCircle className="w-5 h-5 mr-2" />
              Join Discord
            </a>
          </div>
        </div>
      </section>
    </div>
  );
};

export default CommunityPage;