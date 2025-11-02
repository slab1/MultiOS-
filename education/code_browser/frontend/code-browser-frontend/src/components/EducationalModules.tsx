import React, { useState } from 'react';
import { BookOpen, Clock, Users, CheckCircle, Play, Star, Award } from 'lucide-react';

interface EducationalModule {
  id: string;
  title: string;
  description: string;
  difficulty_level: 'beginner' | 'intermediate' | 'advanced' | 'expert';
  estimated_time: string;
  prerequisites: string[];
  topics: string[];
  progress?: number;
  is_completed?: boolean;
  is_locked?: boolean;
}

export const EducationalModules: React.FC = () => {
  const [selectedDifficulty, setSelectedDifficulty] = useState<string>('all');
  const [selectedModule, setSelectedModule] = useState<string | null>(null);

  const modules: EducationalModule[] = [
    {
      id: 'kernel_basics',
      title: 'Kernel Architecture Fundamentals',
      description: 'Introduction to operating system kernels, their structure, and core responsibilities. Learn how kernels manage resources and provide services to user applications.',
      difficulty_level: 'beginner',
      estimated_time: '2 hours',
      prerequisites: [],
      topics: ['Kernel design principles', 'System calls', 'Memory management basics', 'Process model'],
      progress: 100,
      is_completed: true,
    },
    {
      id: 'memory_management',
      title: 'Memory Management Systems',
      description: 'Deep dive into virtual memory, page tables, memory allocation strategies, and memory protection mechanisms.',
      difficulty_level: 'intermediate',
      estimated_time: '4 hours',
      prerequisites: ['kernel_basics'],
      topics: ['Virtual memory', 'Page tables', 'Memory allocation', 'Memory protection'],
      progress: 75,
    },
    {
      id: 'process_scheduling',
      title: 'Process Scheduling Algorithms',
      description: 'Understanding how operating systems decide which process runs when, including various scheduling algorithms and their trade-offs.',
      difficulty_level: 'intermediate',
      estimated_time: '3 hours',
      prerequisites: ['kernel_basics'],
      topics: ['Scheduling algorithms', 'Priority systems', 'Context switching', 'Preemption'],
      progress: 60,
    },
    {
      id: 'device_drivers',
      title: 'Device Driver Development',
      description: 'Building and integrating device drivers, understanding hardware abstraction layers, and interrupt handling.',
      difficulty_level: 'advanced',
      estimated_time: '6 hours',
      prerequisites: ['memory_management', 'process_scheduling'],
      topics: ['Driver architecture', 'Interrupt handling', 'I/O operations', 'Hardware abstraction'],
      progress: 30,
    },
    {
      id: 'synchronization',
      title: 'Synchronization and Concurrency',
      description: 'Advanced topics in process synchronization, mutexes, semaphores, and deadlock prevention.',
      difficulty_level: 'advanced',
      estimated_time: '5 hours',
      prerequisites: ['process_scheduling'],
      topics: ['Mutexes and semaphores', 'Deadlock prevention', 'Race conditions', 'Lock-free programming'],
      progress: 0,
      is_locked: true,
    },
    {
      id: 'performance_optimization',
      title: 'Performance Optimization Techniques',
      description: 'Advanced performance analysis, profiling, cache optimization, and systems performance tuning.',
      difficulty_level: 'expert',
      estimated_time: '8 hours',
      prerequisites: ['device_drivers', 'synchronization'],
      topics: ['Profiling tools', 'Cache optimization', 'Benchmarking', 'Performance analysis'],
      progress: 0,
      is_locked: true,
    },
    {
      id: 'multicore_systems',
      title: 'Multicore and Parallel Systems',
      description: 'Understanding multiprocessor systems, cache coherence, and parallel programming in kernel space.',
      difficulty_level: 'expert',
      estimated_time: '6 hours',
      prerequisites: ['performance_optimization'],
      topics: ['Multicore architecture', 'Cache coherence', 'Parallel algorithms', 'NUMA systems'],
      progress: 0,
      is_locked: true,
    },
    {
      id: 'security_mechanisms',
      title: 'Kernel Security and Safety',
      description: 'Security mechanisms in operating systems, privilege separation, and secure coding practices.',
      difficulty_level: 'expert',
      estimated_time: '5 hours',
      prerequisites: ['performance_optimization'],
      topics: ['Privilege levels', 'Security domains', 'Safe coding practices', 'Vulnerability prevention'],
      progress: 0,
      is_locked: true,
    },
  ];

  const getDifficultyColor = (level: string) => {
    switch (level) {
      case 'beginner': return 'text-green-700 bg-green-100 border-green-200';
      case 'intermediate': return 'text-blue-700 bg-blue-100 border-blue-200';
      case 'advanced': return 'text-orange-700 bg-orange-100 border-orange-200';
      case 'expert': return 'text-red-700 bg-red-100 border-red-200';
      default: return 'text-gray-700 bg-gray-100 border-gray-200';
    }
  };

  const getDifficultyIcon = (level: string) => {
    switch (level) {
      case 'beginner': return '🌱';
      case 'intermediate': return '🌿';
      case 'advanced': return '🌳';
      case 'expert': return '🏆';
      default: return '📚';
    }
  };

  const filteredModules = modules.filter(module => 
    selectedDifficulty === 'all' || module.difficulty_level === selectedDifficulty
  );

  const selectedModuleData = modules.find(m => m.id === selectedModule);

  const getProgressColor = (progress: number) => {
    if (progress === 100) return 'bg-green-500';
    if (progress >= 75) return 'bg-blue-500';
    if (progress >= 50) return 'bg-yellow-500';
    if (progress >= 25) return 'bg-orange-500';
    return 'bg-gray-300';
  };

  return (
    <div className="max-w-7xl mx-auto">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-gray-900 mb-4">Educational Modules</h1>
        <p className="text-lg text-gray-600">
          Progressive learning paths designed to take you from kernel basics to advanced optimization techniques.
          Each module includes interactive code examples, exercises, and real-world case studies.
        </p>
      </div>

      {/* Filters */}
      <div className="mb-6">
        <div className="flex items-center space-x-4">
          <span className="text-sm font-medium text-gray-700">Filter by difficulty:</span>
          <div className="flex space-x-2">
            {['all', 'beginner', 'intermediate', 'advanced', 'expert'].map(level => (
              <button
                key={level}
                onClick={() => setSelectedDifficulty(level)}
                className={`px-3 py-1 text-sm rounded-full border transition-colors duration-200 ${
                  selectedDifficulty === level
                    ? 'bg-blue-100 text-blue-700 border-blue-200'
                    : 'bg-white text-gray-600 border-gray-300 hover:bg-gray-50'
                }`}
              >
                {level === 'all' ? 'All Levels' : level.charAt(0).toUpperCase() + level.slice(1)}
              </button>
            ))}
          </div>
        </div>
      </div>

      <div className="flex gap-8">
        {/* Modules Grid */}
        <div className="flex-1">
          <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6">
            {filteredModules.map((module) => (
              <div
                key={module.id}
                className={`bg-white rounded-lg shadow-md border-2 transition-all duration-200 hover:shadow-lg cursor-pointer ${
                  selectedModule === module.id ? 'border-blue-500' : 'border-gray-200'
                } ${module.is_locked ? 'opacity-60' : ''}`}
                onClick={() => !module.is_locked && setSelectedModule(module.id)}
              >
                <div className="p-6">
                  {/* Header */}
                  <div className="flex items-start justify-between mb-4">
                    <div className="flex items-center space-x-2">
                      <span className="text-2xl">{getDifficultyIcon(module.difficulty_level)}</span>
                      <span className={`px-2 py-1 text-xs rounded-full border ${getDifficultyColor(module.difficulty_level)}`}>
                        {module.difficulty_level}
                      </span>
                    </div>
                    {module.is_completed && (
                      <CheckCircle className="w-5 h-5 text-green-500" />
                    )}
                    {module.is_locked && (
                      <div className="w-5 h-5 bg-gray-400 rounded-full flex items-center justify-center">
                        <span className="text-white text-xs">🔒</span>
                      </div>
                    )}
                  </div>

                  {/* Title and Description */}
                  <h3 className="text-lg font-semibold text-gray-900 mb-2">{module.title}</h3>
                  <p className="text-sm text-gray-600 mb-4 line-clamp-3">{module.description}</p>

                  {/* Metadata */}
                  <div className="flex items-center space-x-4 text-sm text-gray-500 mb-4">
                    <div className="flex items-center space-x-1">
                      <Clock className="w-4 h-4" />
                      <span>{module.estimated_time}</span>
                    </div>
                    <div className="flex items-center space-x-1">
                      <BookOpen className="w-4 h-4" />
                      <span>{module.topics.length} topics</span>
                    </div>
                  </div>

                  {/* Prerequisites */}
                  {module.prerequisites.length > 0 && (
                    <div className="mb-4">
                      <div className="text-xs text-gray-500 mb-1">Prerequisites:</div>
                      <div className="flex flex-wrap gap-1">
                        {module.prerequisites.slice(0, 2).map(prereq => (
                          <span
                            key={prereq}
                            className="px-2 py-1 bg-gray-100 text-gray-600 text-xs rounded"
                          >
                            {prereq.replace('_', ' ')}
                          </span>
                        ))}
                        {module.prerequisites.length > 2 && (
                          <span className="px-2 py-1 bg-gray-100 text-gray-600 text-xs rounded">
                            +{module.prerequisites.length - 2} more
                          </span>
                        )}
                      </div>
                    </div>
                  )}

                  {/* Progress Bar */}
                  {module.progress !== undefined && (
                    <div className="mb-4">
                      <div className="flex items-center justify-between text-sm mb-1">
                        <span className="text-gray-600">Progress</span>
                        <span className="font-medium">{module.progress}%</span>
                      </div>
                      <div className="w-full bg-gray-200 rounded-full h-2">
                        <div
                          className={`h-2 rounded-full transition-all duration-300 ${getProgressColor(module.progress)}`}
                          style={{ width: `${module.progress}%` }}
                        ></div>
                      </div>
                    </div>
                  )}

                  {/* Action Button */}
                  <button
                    disabled={module.is_locked}
                    className={`w-full py-2 px-4 rounded-md text-sm font-medium transition-colors duration-200 flex items-center justify-center space-x-2 ${
                      module.is_locked
                        ? 'bg-gray-100 text-gray-400 cursor-not-allowed'
                        : module.is_completed
                        ? 'bg-green-100 text-green-700 hover:bg-green-200'
                        : 'bg-blue-100 text-blue-700 hover:bg-blue-200'
                    }`}
                  >
                    {module.is_locked ? (
                      <>
                        <span>Locked</span>
                      </>
                    ) : module.is_completed ? (
                      <>
                        <CheckCircle className="w-4 h-4" />
                        <span>Review</span>
                      </>
                    ) : (
                      <>
                        <Play className="w-4 h-4" />
                        <span>Start Module</span>
                      </>
                    )}
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Module Details Panel */}
        {selectedModuleData && (
          <div className="w-96 bg-white rounded-lg shadow-lg border border-gray-200 p-6">
            <h2 className="text-xl font-bold text-gray-900 mb-4">{selectedModuleData.title}</h2>
            
            <div className="space-y-6">
              {/* Module Overview */}
              <div>
                <h3 className="text-sm font-medium text-gray-700 mb-2">Overview</h3>
                <p className="text-sm text-gray-600">{selectedModuleData.description}</p>
              </div>

              {/* Learning Objectives */}
              <div>
                <h3 className="text-sm font-medium text-gray-700 mb-2">Topics Covered</h3>
                <ul className="space-y-1">
                  {selectedModuleData.topics.map((topic, index) => (
                    <li key={index} className="flex items-center space-x-2 text-sm text-gray-600">
                      <CheckCircle className="w-4 h-4 text-green-500" />
                      <span>{topic}</span>
                    </li>
                  ))}
                </ul>
              </div>

              {/* Prerequisites */}
              {selectedModuleData.prerequisites.length > 0 && (
                <div>
                  <h3 className="text-sm font-medium text-gray-700 mb-2">Prerequisites</h3>
                  <div className="space-y-2">
                    {selectedModuleData.prerequisites.map(prereq => {
                      const prereqModule = modules.find(m => m.id === prereq);
                      const isCompleted = prereqModule?.is_completed;
                      
                      return (
                        <div key={prereq} className="flex items-center justify-between">
                          <span className="text-sm text-gray-600">
                            {prereq.replace('_', ' ').replace(/\b\w/g, l => l.toUpperCase())}
                          </span>
                          {isCompleted ? (
                            <CheckCircle className="w-4 h-4 text-green-500" />
                          ) : (
                            <Clock className="w-4 h-4 text-yellow-500" />
                          )}
                        </div>
                      );
                    })}
                  </div>
                </div>
              )}

              {/* Skills You'll Gain */}
              <div>
                <h3 className="text-sm font-medium text-gray-700 mb-2">Skills You'll Gain</h3>
                <div className="flex flex-wrap gap-2">
                  {selectedModuleData.topics.map((topic, index) => (
                    <span
                      key={index}
                      className="px-2 py-1 bg-blue-100 text-blue-700 text-xs rounded-full"
                    >
                      {topic}
                    </span>
                  ))}
                </div>
              </div>

              {/* Module Stats */}
              <div className="grid grid-cols-2 gap-4 p-4 bg-gray-50 rounded-lg">
                <div className="text-center">
                  <div className="text-lg font-semibold text-gray-900">{selectedModuleData.estimated_time}</div>
                  <div className="text-xs text-gray-600">Estimated Time</div>
                </div>
                <div className="text-center">
                  <div className="text-lg font-semibold text-gray-900">
                    {selectedModuleData.topics.length}
                  </div>
                  <div className="text-xs text-gray-600">Topics</div>
                </div>
              </div>

              {/* Action Button */}
              <button
                disabled={selectedModuleData.is_locked}
                className={`w-full py-3 px-4 rounded-md font-medium transition-colors duration-200 flex items-center justify-center space-x-2 ${
                  selectedModuleData.is_locked
                    ? 'bg-gray-100 text-gray-400 cursor-not-allowed'
                    : selectedModuleData.is_completed
                    ? 'bg-green-100 text-green-700 hover:bg-green-200'
                    : 'bg-blue-600 text-white hover:bg-blue-700'
                }`}
              >
                {selectedModuleData.is_locked ? (
                  <>
                    <span>🔒 Prerequisites Required</span>
                  </>
                ) : selectedModuleData.is_completed ? (
                  <>
                    <Award className="w-5 h-5" />
                    <span>Review Module</span>
                  </>
                ) : (
                  <>
                    <Play className="w-5 h-5" />
                    <span>Start Learning</span>
                  </>
                )}
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
