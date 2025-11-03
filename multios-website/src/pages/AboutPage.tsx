import React from 'react';
import { Target, Users, Award, TrendingUp, Calendar, MapPin } from 'lucide-react';

const AboutPage: React.FC = () => {
  const timeline = [
    {
      year: '2023',
      title: 'Project Inception',
      description: 'MultiOS project started with a vision to create an educational OS',
    },
    {
      year: '2024',
      title: 'Alpha Release',
      description: 'First working prototype with basic kernel functionality',
    },
    {
      year: '2024',
      title: 'Cross-Platform',
      description: 'Added support for ARM64 and RISC-V architectures',
    },
    {
      year: '2025',
      title: 'Beta Release',
      description: 'Production-ready release with comprehensive testing',
    },
  ];

  const team = [
    {
      name: 'Dr. Sarah Chen',
      role: 'Lead Architect',
      description: 'Former Google kernel engineer, 15+ years in OS development',
      image: '/api/placeholder/150/150'
    },
    {
      name: 'Prof. Michael Rodriguez',
      role: 'Educational Director',
      description: 'MIT professor specializing in systems education',
      image: '/api/placeholder/150/150'
    },
    {
      name: 'Emily Watson',
      role: 'Developer Relations',
      description: 'Community building and open source advocate',
      image: '/api/placeholder/150/150'
    },
  ];

  return (
    <div className="min-h-screen bg-white">
      {/* Hero Section */}
      <section className="bg-gradient-to-br from-blue-600 via-purple-600 to-indigo-800 py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center text-white">
          <h1 className="text-4xl md:text-5xl font-bold mb-6">About MultiOS</h1>
          <p className="text-xl md:text-2xl text-blue-100 max-w-3xl mx-auto">
            Revolutionizing operating systems education through modern development practices
          </p>
        </div>
      </section>

      {/* Mission */}
      <section className="py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid lg:grid-cols-2 gap-12 items-center">
            <div>
              <h2 className="text-3xl font-bold text-gray-900 mb-6">Our Mission</h2>
              <p className="text-lg text-gray-600 mb-6">
                MultiOS was created to bridge the gap between theoretical operating systems 
                education and practical, hands-on experience. We believe that learning OS 
                development should be accessible, comprehensive, and aligned with modern industry practices.
              </p>
              <div className="grid grid-cols-2 gap-4">
                <div className="flex items-center">
                  <Target className="w-5 h-5 text-blue-600 mr-2" />
                  <span>Educational Excellence</span>
                </div>
                <div className="flex items-center">
                  <Users className="w-5 h-5 text-green-600 mr-2" />
                  <span>Community Driven</span>
                </div>
                <div className="flex items-center">
                  <Award className="w-5 h-5 text-purple-600 mr-2" />
                  <span>Quality Assured</span>
                </div>
                <div className="flex items-center">
                  <TrendingUp className="w-5 h-5 text-orange-600 mr-2" />
                  <span>Future Ready</span>
                </div>
              </div>
            </div>
            <div className="bg-gray-100 rounded-xl p-8">
              <h3 className="text-xl font-semibold text-gray-900 mb-4">By the Numbers</h3>
              <div className="space-y-4">
                <div className="flex justify-between">
                  <span>Contributors</span>
                  <span className="font-semibold">5,800+</span>
                </div>
                <div className="flex justify-between">
                  <span>Lines of Code</span>
                  <span className="font-semibold">50,000+</span>
                </div>
                <div className="flex justify-between">
                  <span>Universities</span>
                  <span className="font-semibold">150+</span>
                </div>
                <div className="flex justify-between">
                  <span>Countries</span>
                  <span className="font-semibold">40+</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Timeline */}
      <section className="py-20 bg-gray-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <h2 className="text-3xl font-bold text-center text-gray-900 mb-12">Project Timeline</h2>
          <div className="max-w-4xl mx-auto">
            {timeline.map((event, index) => (
              <div key={index} className="flex items-center mb-8">
                <div className="flex-shrink-0 w-16 h-16 bg-blue-600 text-white rounded-full flex items-center justify-center font-bold">
                  {event.year}
                </div>
                <div className="ml-6">
                  <h3 className="text-xl font-semibold text-gray-900">{event.title}</h3>
                  <p className="text-gray-600">{event.description}</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Team */}
      <section className="py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <h2 className="text-3xl font-bold text-center text-gray-900 mb-12">Our Team</h2>
          <div className="grid md:grid-cols-3 gap-8">
            {team.map((member, index) => (
              <div key={index} className="text-center">
                <div className="w-32 h-32 bg-gray-300 rounded-full mx-auto mb-4"></div>
                <h3 className="text-xl font-semibold text-gray-900">{member.name}</h3>
                <p className="text-blue-600 font-medium mb-2">{member.role}</p>
                <p className="text-gray-600">{member.description}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Values */}
      <section className="py-20 bg-gray-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <h2 className="text-3xl font-bold text-center text-gray-900 mb-12">Our Values</h2>
          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-8">
            <div className="text-center">
              <div className="w-16 h-16 bg-blue-600 rounded-full flex items-center justify-center mx-auto mb-4">
                <Target className="w-8 h-8 text-white" />
              </div>
              <h3 className="text-lg font-semibold text-gray-900 mb-2">Excellence</h3>
              <p className="text-gray-600">We strive for the highest quality in everything we create</p>
            </div>
            <div className="text-center">
              <div className="w-16 h-16 bg-green-600 rounded-full flex items-center justify-center mx-auto mb-4">
                <Users className="w-8 h-8 text-white" />
              </div>
              <h3 className="text-lg font-semibold text-gray-900 mb-2">Community</h3>
              <p className="text-gray-600">We believe in the power of collaborative learning</p>
            </div>
            <div className="text-center">
              <div className="w-16 h-16 bg-purple-600 rounded-full flex items-center justify-center mx-auto mb-4">
                <Award className="w-8 h-8 text-white" />
              </div>
              <h3 className="text-lg font-semibold text-gray-900 mb-2">Innovation</h3>
              <p className="text-gray-600">We embrace new technologies and approaches</p>
            </div>
            <div className="text-center">
              <div className="w-16 h-16 bg-orange-600 rounded-full flex items-center justify-center mx-auto mb-4">
                <TrendingUp className="w-8 h-8 text-white" />
              </div>
              <h3 className="text-lg font-semibold text-gray-900 mb-2">Impact</h3>
              <p className="text-gray-600">We measure success by educational impact</p>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
};

export default AboutPage;