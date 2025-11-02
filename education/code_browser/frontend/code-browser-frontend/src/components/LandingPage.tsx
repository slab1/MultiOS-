import React from 'react';
import { Code, BookOpen, TrendingUp, Target, Users, Award } from 'lucide-react';

interface LandingPageProps {
  onStart: () => void;
}

export const LandingPage: React.FC<LandingPageProps> = ({ onStart }) => {
  const features = [
    {
      icon: Code,
      title: 'Interactive Code Browser',
      description: 'Navigate through MultiOS kernel code with syntax highlighting, function call graphs, and real-time explanations.',
    },
    {
      icon: BookOpen,
      title: 'Educational Modules',
      description: 'Progressive learning paths from kernel basics to advanced optimization techniques.',
    },
    {
      icon: TrendingUp,
      title: 'Performance Analysis',
      description: 'Identify performance hotspots and get optimization suggestions with educational context.',
    },
    {
      icon: Target,
      title: 'Data Flow Tracking',
      description: 'Track variable usage and understand data dependencies across the codebase.',
    },
  ];

  const stats = [
    { label: 'Code Files', value: '1,200+', description: 'Kernel source files analyzed' },
    { label: 'Functions', value: '5,000+', description: 'Function definitions with explanations' },
    { label: 'Learning Modules', value: '25', description: 'Educational modules available' },
    { label: 'Performance Insights', value: '150+', description: 'Optimization opportunities identified' },
  ];

  return (
    <div className="max-w-7xl mx-auto">
      {/* Hero Section */}
      <div className="text-center mb-16">
        <h1 className="text-4xl font-bold text-gray-900 mb-4">
          MultiOS Interactive Code Browser
        </h1>
        <p className="text-xl text-gray-600 mb-8 max-w-3xl mx-auto">
          Explore the MultiOS kernel with real-time explanations, performance analysis, and educational content. 
          Perfect for learning operating systems development and understanding complex kernel code.
        </p>
        <button
          onClick={onStart}
          className="bg-blue-600 hover:bg-blue-700 text-white font-bold py-3 px-8 rounded-lg transition-colors duration-200 text-lg"
        >
          Start Exploring Code
        </button>
      </div>

      {/* Features Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8 mb-16">
        {features.map((feature, index) => {
          const Icon = feature.icon;
          return (
            <div key={index} className="bg-white p-6 rounded-lg shadow-md hover:shadow-lg transition-shadow duration-200">
              <div className="flex items-center justify-center w-12 h-12 bg-blue-100 rounded-lg mb-4">
                <Icon className="w-6 h-6 text-blue-600" />
              </div>
              <h3 className="text-lg font-semibold text-gray-900 mb-2">{feature.title}</h3>
              <p className="text-gray-600">{feature.description}</p>
            </div>
          );
        })}
      </div>

      {/* Stats Section */}
      <div className="bg-gradient-to-r from-blue-50 to-indigo-50 rounded-2xl p-8 mb-16">
        <h2 className="text-2xl font-bold text-center text-gray-900 mb-8">Platform Statistics</h2>
        <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
          {stats.map((stat, index) => (
            <div key={index} className="text-center">
              <div className="text-3xl font-bold text-blue-600 mb-2">{stat.value}</div>
              <div className="text-lg font-semibold text-gray-900 mb-1">{stat.label}</div>
              <div className="text-sm text-gray-600">{stat.description}</div>
            </div>
          ))}
        </div>
      </div>

      {/* Learning Paths */}
      <div className="mb-16">
        <h2 className="text-2xl font-bold text-center text-gray-900 mb-8">Learning Paths</h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
          <div className="bg-green-50 p-6 rounded-lg border-2 border-green-200">
            <div className="flex items-center mb-4">
              <div className="w-8 h-8 bg-green-500 rounded-full flex items-center justify-center text-white font-bold mr-3">B</div>
              <h3 className="text-lg font-semibold text-green-800">Beginner Path</h3>
            </div>
            <p className="text-green-700 mb-4">
              Introduction to kernel architecture, memory management basics, and system calls.
            </p>
            <ul className="text-sm text-green-600 space-y-1">
              <li>• Operating System Concepts</li>
              <li>• Rust for Systems Programming</li>
              <li>• Kernel Entry Points</li>
              <li>• Basic Memory Management</li>
            </ul>
          </div>

          <div className="bg-yellow-50 p-6 rounded-lg border-2 border-yellow-200">
            <div className="flex items-center mb-4">
              <div className="w-8 h-8 bg-yellow-500 rounded-full flex items-center justify-center text-white font-bold mr-3">I</div>
              <h3 className="text-lg font-semibold text-yellow-800">Intermediate Path</h3>
            </div>
            <p className="text-yellow-700 mb-4">
              Process scheduling, interrupt handling, and device driver development.
            </p>
            <ul className="text-sm text-yellow-600 space-y-1">
              <li>• Process Management</li>
              <li>• Interrupt Handling</li>
              <li>• Device Drivers</li>
              <li>• Synchronization</li>
            </ul>
          </div>

          <div className="bg-red-50 p-6 rounded-lg border-2 border-red-200">
            <div className="flex items-center mb-4">
              <div className="w-8 h-8 bg-red-500 rounded-full flex items-center justify-center text-white font-bold mr-3">A</div>
              <h3 className="text-lg font-semibold text-red-800">Advanced Path</h3>
            </div>
            <p className="text-red-700 mb-4">
              Performance optimization, multicore systems, and advanced kernel features.
            </p>
            <ul className="text-sm text-red-600 space-y-1">
              <li>• Performance Optimization</li>
              <li>• Multicore Architecture</li>
              <li>• Advanced Scheduling</li>
              <li>• Security Mechanisms</li>
            </ul>
          </div>
        </div>
      </div>

      {/* Call to Action */}
      <div className="text-center bg-gray-900 text-white rounded-2xl p-12">
        <h2 className="text-3xl font-bold mb-4">Ready to Explore the Kernel?</h2>
        <p className="text-xl text-gray-300 mb-8 max-w-2xl mx-auto">
          Join thousands of developers learning operating systems development through interactive code exploration.
        </p>
        <button
          onClick={onStart}
          className="bg-blue-600 hover:bg-blue-700 text-white font-bold py-3 px-8 rounded-lg transition-colors duration-200 text-lg"
        >
          Begin Your Journey
        </button>
      </div>
    </div>
  );
};
