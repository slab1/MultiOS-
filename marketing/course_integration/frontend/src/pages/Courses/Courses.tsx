import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { useAuth } from '../../contexts/AuthContext';
import {
  PlusIcon,
  MagnifyingGlassIcon,
  FunnelIcon,
  AcademicCapIcon,
  UsersIcon,
  ClockIcon,
  BookOpenIcon,
} from '@heroicons/react/24/outline';

interface Course {
  id: string;
  title: string;
  description: string;
  courseCode: string;
  level: 'beginner' | 'intermediate' | 'advanced' | 'expert';
  category: string;
  instructor: {
    firstName: string;
    lastName: string;
  };
  startDate?: string;
  endDate?: string;
  estimatedDurationHours?: number;
  maxEnrollment?: number;
  isPublished: boolean;
  createdAt: string;
}

const Courses: React.FC = () => {
  const { user, hasPermission } = useAuth();
  const [courses, setCourses] = useState<Course[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [searchTerm, setSearchTerm] = useState('');
  const [filterLevel, setFilterLevel] = useState<string>('');
  const [filterCategory, setFilterCategory] = useState<string>('');

  const isInstructor = hasPermission(['instructor', 'administrator']);

  useEffect(() => {
    loadCourses();
  }, [searchTerm, filterLevel, filterCategory]);

  const loadCourses = async () => {
    try {
      setIsLoading(true);
      
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 800));
      
      const mockCourses: Course[] = [
        {
          id: '1',
          title: 'MultiOS Kernel Development Fundamentals',
          description: 'Learn the core concepts of operating system kernel development, memory management, and process scheduling.',
          courseCode: 'MOS-KERNEL-101',
          level: 'intermediate',
          category: 'System Programming',
          instructor: {
            firstName: 'Dr. Sarah',
            lastName: 'Chen'
          },
          startDate: '2024-02-01',
          endDate: '2024-05-15',
          estimatedDurationHours: 120,
          maxEnrollment: 50,
          isPublished: true,
          createdAt: '2024-01-15T10:00:00Z'
        },
        {
          id: '2',
          title: 'Advanced Driver Development',
          description: 'Deep dive into hardware driver development for MultiOS, including device driver architecture and debugging.',
          courseCode: 'MOS-DRIVER-201',
          level: 'advanced',
          category: 'Driver Development',
          instructor: {
            firstName: 'Prof. Michael',
            lastName: 'Rodriguez'
          },
          startDate: '2024-03-01',
          endDate: '2024-06-15',
          estimatedDurationHours: 150,
          maxEnrollment: 30,
          isPublished: true,
          createdAt: '2024-02-01T14:30:00Z'
        },
        {
          id: '3',
          title: 'MultiOS System Administration',
          description: 'Comprehensive guide to administering MultiOS systems in enterprise environments.',
          courseCode: 'MOS-ADMIN-101',
          level: 'beginner',
          category: 'System Administration',
          instructor: {
            firstName: 'Admin',
            lastName: 'Team'
          },
          startDate: '2024-01-15',
          endDate: '2024-04-30',
          estimatedDurationHours: 80,
          maxEnrollment: 100,
          isPublished: true,
          createdAt: '2024-01-01T09:00:00Z'
        }
      ];
      
      let filteredCourses = mockCourses;
      
      if (searchTerm) {
        filteredCourses = filteredCourses.filter(course =>
          course.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
          course.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
          course.courseCode.toLowerCase().includes(searchTerm.toLowerCase())
        );
      }
      
      if (filterLevel) {
        filteredCourses = filteredCourses.filter(course => course.level === filterLevel);
      }
      
      if (filterCategory) {
        filteredCourses = filteredCourses.filter(course => course.category === filterCategory);
      }
      
      setCourses(filteredCourses);
    } catch (error) {
      console.error('Failed to load courses:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const getLevelColor = (level: string) => {
    switch (level) {
      case 'beginner':
        return 'bg-green-100 text-green-800';
      case 'intermediate':
        return 'bg-blue-100 text-blue-800';
      case 'advanced':
        return 'bg-purple-100 text-purple-800';
      case 'expert':
        return 'bg-red-100 text-red-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  const formatDuration = (hours?: number) => {
    if (!hours) return 'Self-paced';
    if (hours < 10) return `${hours}h`;
    return `${Math.floor(hours / 10) * 10}+h`;
  };

  const formatDate = (dateString?: string) => {
    if (!dateString) return 'Self-paced';
    return new Date(dateString).toLocaleDateString();
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">
            {isInstructor ? 'Course Management' : 'Available Courses'}
          </h1>
          <p className="mt-1 text-sm text-gray-500">
            {isInstructor 
              ? 'Create and manage your MultiOS courses' 
              : 'Explore and enroll in MultiOS courses'
            }
          </p>
        </div>
        
        {isInstructor && (
          <Link
            to="/courses/create"
            className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
          >
            <PlusIcon className="h-4 w-4 mr-2" />
            Create Course
          </Link>
        )}
      </div>

      {/* Search and Filters */}
      <div className="bg-white shadow rounded-lg">
        <div className="p-6">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            {/* Search */}
            <div className="md:col-span-2">
              <div className="relative">
                <MagnifyingGlassIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-gray-400" />
                <input
                  type="text"
                  placeholder="Search courses..."
                  className="pl-10 w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                />
              </div>
            </div>
            
            {/* Level Filter */}
            <div>
              <select
                className="w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
                value={filterLevel}
                onChange={(e) => setFilterLevel(e.target.value)}
              >
                <option value="">All Levels</option>
                <option value="beginner">Beginner</option>
                <option value="intermediate">Intermediate</option>
                <option value="advanced">Advanced</option>
                <option value="expert">Expert</option>
              </select>
            </div>
            
            {/* Category Filter */}
            <div>
              <select
                className="w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
                value={filterCategory}
                onChange={(e) => setFilterCategory(e.target.value)}
              >
                <option value="">All Categories</option>
                <option value="System Programming">System Programming</option>
                <option value="Driver Development">Driver Development</option>
                <option value="System Administration">System Administration</option>
                <option value="Performance Optimization">Performance Optimization</option>
              </select>
            </div>
          </div>
        </div>
      </div>

      {/* Courses Grid */}
      {isLoading ? (
        <div className="flex items-center justify-center h-64">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {courses.map((course) => (
            <div
              key={course.id}
              className="bg-white overflow-hidden shadow rounded-lg hover:shadow-md transition-shadow"
            >
              <div className="p-6">
                {/* Header */}
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <h3 className="text-lg font-medium text-gray-900 mb-2">
                      {course.title}
                    </h3>
                    <p className="text-sm text-gray-500 mb-3 line-clamp-2">
                      {course.description}
                    </p>
                  </div>
                </div>

                {/* Course info */}
                <div className="space-y-3">
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-gray-500">Code</span>
                    <span className="font-medium text-gray-900">{course.courseCode}</span>
                  </div>
                  
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-gray-500">Level</span>
                    <span className={`inline-flex px-2 py-1 text-xs font-semibold rounded-full ${getLevelColor(course.level)}`}>
                      {course.level}
                    </span>
                  </div>
                  
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-gray-500">Instructor</span>
                    <span className="font-medium text-gray-900">
                      {course.instructor.firstName} {course.instructor.lastName}
                    </span>
                  </div>
                  
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-gray-500">Duration</span>
                    <div className="flex items-center">
                      <ClockIcon className="h-4 w-4 text-gray-400 mr-1" />
                      <span className="text-gray-900">{formatDuration(course.estimatedDurationHours)}</span>
                    </div>
                  </div>
                  
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-gray-500">Category</span>
                    <span className="text-gray-900">{course.category}</span>
                  </div>
                  
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-gray-500">Start Date</span>
                    <span className="text-gray-900">{formatDate(course.startDate)}</span>
                  </div>
                  
                  {course.maxEnrollment && (
                    <div className="flex items-center justify-between text-sm">
                      <span className="text-gray-500">Enrollment</span>
                      <div className="flex items-center">
                        <UsersIcon className="h-4 w-4 text-gray-400 mr-1" />
                        <span className="text-gray-900">{course.maxEnrollment} max</span>
                      </div>
                    </div>
                  )}
                </div>

                {/* Actions */}
                <div className="mt-6 flex space-x-3">
                  <Link
                    to={`/courses/${course.id}`}
                    className="flex-1 bg-white py-2 px-3 border border-gray-300 rounded-md shadow-sm text-sm leading-4 font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 text-center"
                  >
                    View Details
                  </Link>
                  
                  {!isInstructor && course.isPublished && (
                    <button className="flex-1 bg-indigo-600 py-2 px-3 border border-transparent rounded-md shadow-sm text-sm leading-4 font-medium text-white hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 text-center">
                      Enroll
                    </button>
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Empty state */}
      {!isLoading && courses.length === 0 && (
        <div className="text-center py-12">
          <AcademicCapIcon className="mx-auto h-12 w-12 text-gray-400" />
          <h3 className="mt-2 text-sm font-medium text-gray-900">
            No courses found
          </h3>
          <p className="mt-1 text-sm text-gray-500">
            {searchTerm || filterLevel || filterCategory
              ? 'Try adjusting your search or filters.'
              : 'Get started by creating a new course.'
            }
          </p>
        </div>
      )}

      {/* Help section */}
      <div className="bg-indigo-50 border border-indigo-200 rounded-md p-4">
        <div className="flex">
          <div className="flex-shrink-0">
            <BookOpenIcon className="h-5 w-5 text-indigo-400" />
          </div>
          <div className="ml-3">
            <h3 className="text-sm font-medium text-indigo-800">
              About MultiOS Courses
            </h3>
            <div className="mt-2 text-sm text-indigo-700">
              <p>
                Our courses are designed to take you from beginner to expert in MultiOS development. 
                Each course includes hands-on labs, real-world projects, and interactive assignments.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Courses;