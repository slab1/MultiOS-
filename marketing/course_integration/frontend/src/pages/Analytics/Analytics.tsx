import React, { useState, useEffect } from 'react';
import { BarChart3, TrendingUp, Users, BookOpen, Calendar, Download, Filter, RefreshCw } from 'lucide-react';

interface AnalyticsData {
  totalCourses: number;
  totalStudents: number;
  totalAssignments: number;
  completionRate: number;
  monthlyData: {
    courses: number[];
    students: number[];
    assignments: number[];
    months: string[];
  };
  lmsData: {
    platform: string;
    courses: number;
    students: number;
    engagement: number;
  }[];
  courseStats: {
    id: string;
    name: string;
    enrollment: number;
    completion: number;
    averageScore: number;
    lastActivity: string;
  }[];
}

const Analytics: React.FC = () => {
  const [analyticsData, setAnalyticsData] = useState<AnalyticsData | null>(null);
  const [loading, setLoading] = useState(true);
  const [selectedPeriod, setSelectedPeriod] = useState('last30days');
  const [selectedLMS, setSelectedLMS] = useState('all');

  useEffect(() => {
    // Mock data - replace with actual API call
    const mockData: AnalyticsData = {
      totalCourses: 24,
      totalStudents: 1247,
      totalAssignments: 156,
      completionRate: 87.5,
      monthlyData: {
        courses: [18, 19, 21, 22, 24, 24, 24, 24, 24, 24, 24, 24],
        students: [980, 1020, 1050, 1080, 1120, 1150, 1180, 1210, 1230, 1240, 1245, 1247],
        assignments: [120, 125, 130, 135, 140, 145, 148, 150, 152, 154, 155, 156],
        months: ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec']
      },
      lmsData: [
        {
          platform: 'Blackboard',
          courses: 8,
          students: 420,
          engagement: 85.2
        },
        {
          platform: 'Moodle',
          courses: 7,
          students: 385,
          engagement: 89.1
        },
        {
          platform: 'Google Classroom',
          courses: 6,
          students: 312,
          engagement: 91.5
        },
        {
          platform: 'Microsoft Teams',
          courses: 3,
          students: 130,
          engagement: 82.3
        }
      ],
      courseStats: [
        {
          id: '1',
          name: 'Introduction to Computer Science',
          enrollment: 156,
          completion: 89.2,
          averageScore: 85.4,
          lastActivity: '2025-11-03'
        },
        {
          id: '2',
          name: 'Data Structures and Algorithms',
          enrollment: 142,
          completion: 94.1,
          averageScore: 88.7,
          lastActivity: '2025-11-02'
        },
        {
          id: '3',
          name: 'Web Development Fundamentals',
          enrollment: 128,
          completion: 92.8,
          averageScore: 86.2,
          lastActivity: '2025-11-03'
        },
        {
          id: '4',
          name: 'Database Management Systems',
          enrollment: 118,
          completion: 87.3,
          averageScore: 83.9,
          lastActivity: '2025-11-01'
        },
        {
          id: '5',
          name: 'Software Engineering Principles',
          enrollment: 98,
          completion: 91.8,
          averageScore: 90.1,
          lastActivity: '2025-11-02'
        }
      ]
    };
    
    setTimeout(() => {
      setAnalyticsData(mockData);
      setLoading(false);
    }, 1000);
  }, [selectedPeriod, selectedLMS]);

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric'
    });
  };

  const formatPercentage = (value: number) => {
    return `${value.toFixed(1)}%`;
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  if (!analyticsData) return null;

  const maxValue = Math.max(...analyticsData.monthlyData.students);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Analytics Dashboard</h1>
          <p className="text-gray-600">Insights and performance metrics across your learning platform</p>
        </div>
        <div className="flex items-center space-x-3">
          <button
            className="flex items-center space-x-2 px-4 py-2 text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200 transition-colors"
          >
            <Filter className="w-4 h-4" />
            <span>Filters</span>
          </button>
          <button
            className="flex items-center space-x-2 px-4 py-2 text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200 transition-colors"
          >
            <Download className="w-4 h-4" />
            <span>Export</span>
          </button>
          <button
            className="flex items-center space-x-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            <RefreshCw className="w-4 h-4" />
            <span>Refresh</span>
          </button>
        </div>
      </div>

      {/* Filter Controls */}
      <div className="bg-white p-4 rounded-lg shadow-sm border">
        <div className="flex items-center space-x-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Time Period</label>
            <select
              value={selectedPeriod}
              onChange={(e) => setSelectedPeriod(e.target.value)}
              className="px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            >
              <option value="last7days">Last 7 days</option>
              <option value="last30days">Last 30 days</option>
              <option value="last3months">Last 3 months</option>
              <option value="last6months">Last 6 months</option>
              <option value="lastyear">Last year</option>
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">LMS Platform</label>
            <select
              value={selectedLMS}
              onChange={(e) => setSelectedLMS(e.target.value)}
              className="px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            >
              <option value="all">All Platforms</option>
              <option value="blackboard">Blackboard</option>
              <option value="moodle">Moodle</option>
              <option value="google">Google Classroom</option>
              <option value="microsoft">Microsoft Teams</option>
            </select>
          </div>
        </div>
      </div>

      {/* Key Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <div className="bg-white p-6 rounded-lg shadow-sm border">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-600">Total Courses</p>
              <p className="text-3xl font-bold text-gray-900">{analyticsData.totalCourses}</p>
              <p className="text-sm text-green-600 flex items-center mt-1">
                <TrendingUp className="w-4 h-4 mr-1" />
                +2.1% from last month
              </p>
            </div>
            <BookOpen className="w-12 h-12 text-blue-500" />
          </div>
        </div>

        <div className="bg-white p-6 rounded-lg shadow-sm border">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-600">Total Students</p>
              <p className="text-3xl font-bold text-gray-900">{analyticsData.totalStudents.toLocaleString()}</p>
              <p className="text-sm text-green-600 flex items-center mt-1">
                <TrendingUp className="w-4 h-4 mr-1" />
                +5.3% from last month
              </p>
            </div>
            <Users className="w-12 h-12 text-green-500" />
          </div>
        </div>

        <div className="bg-white p-6 rounded-lg shadow-sm border">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-600">Total Assignments</p>
              <p className="text-3xl font-bold text-gray-900">{analyticsData.totalAssignments}</p>
              <p className="text-sm text-green-600 flex items-center mt-1">
                <TrendingUp className="w-4 h-4 mr-1" />
                +3.7% from last month
              </p>
            </div>
            <BarChart3 className="w-12 h-12 text-purple-500" />
          </div>
        </div>

        <div className="bg-white p-6 rounded-lg shadow-sm border">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-600">Completion Rate</p>
              <p className="text-3xl font-bold text-gray-900">{formatPercentage(analyticsData.completionRate)}</p>
              <p className="text-sm text-green-600 flex items-center mt-1">
                <TrendingUp className="w-4 h-4 mr-1" />
                +1.2% from last month
              </p>
            </div>
            <TrendingUp className="w-12 h-12 text-orange-500" />
          </div>
        </div>
      </div>

      {/* Charts Section */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Student Growth Chart */}
        <div className="bg-white p-6 rounded-lg shadow-sm border">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-semibold text-gray-900">Student Growth</h3>
            <span className="text-sm text-gray-500">Last 12 months</span>
          </div>
          <div className="space-y-4">
            {analyticsData.monthlyData.students.map((value, index) => (
              <div key={index} className="flex items-center space-x-4">
                <span className="text-sm text-gray-600 w-12">
                  {analyticsData.monthlyData.months[index]}
                </span>
                <div className="flex-1 bg-gray-200 rounded-full h-8 relative">
                  <div
                    className="bg-blue-600 h-8 rounded-full flex items-center justify-end pr-2"
                    style={{ width: `${(value / maxValue) * 100}%` }}
                  >
                    <span className="text-xs text-white font-medium">{value}</span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* LMS Platform Distribution */}
        <div className="bg-white p-6 rounded-lg shadow-sm border">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-semibold text-gray-900">LMS Platform Distribution</h3>
            <span className="text-sm text-gray-500">Current period</span>
          </div>
          <div className="space-y-4">
            {analyticsData.lmsData.map((platform, index) => (
              <div key={index} className="border-l-4 pl-4" style={{
                borderColor: ['#3B82F6', '#10B981', '#F59E0B', '#8B5CF6'][index] || '#6B7280'
              }}>
                <div className="flex justify-between items-start">
                  <div>
                    <h4 className="font-medium text-gray-900">{platform.platform}</h4>
                    <div className="flex items-center space-x-4 mt-1">
                      <span className="text-sm text-gray-600">{platform.courses} courses</span>
                      <span className="text-sm text-gray-600">{platform.students} students</span>
                    </div>
                  </div>
                  <div className="text-right">
                    <div className="text-lg font-bold text-gray-900">
                      {formatPercentage(platform.engagement)}
                    </div>
                    <div className="text-xs text-gray-500">engagement</div>
                  </div>
                </div>
                <div className="mt-2">
                  <div className="w-full bg-gray-200 rounded-full h-2">
                    <div
                      className="h-2 rounded-full"
                      style={{
                        width: `${platform.engagement}%`,
                        backgroundColor: ['#3B82F6', '#10B981', '#F59E0B', '#8B5CF6'][index] || '#6B7280'
                      }}
                    ></div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Course Performance Table */}
      <div className="bg-white rounded-lg shadow-sm border">
        <div className="p-6 border-b">
          <h3 className="text-lg font-semibold text-gray-900">Top Performing Courses</h3>
          <p className="text-sm text-gray-600 mt-1">Based on completion rate and student engagement</p>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead className="bg-gray-50">
              <tr>
                <th className="text-left px-6 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Course Name
                </th>
                <th className="text-left px-6 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Enrollment
                </th>
                <th className="text-left px-6 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Completion Rate
                </th>
                <th className="text-left px-6 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Average Score
                </th>
                <th className="text-left px-6 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Last Activity
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200">
              {analyticsData.courseStats.map((course) => (
                <tr key={course.id} className="hover:bg-gray-50">
                  <td className="px-6 py-4">
                    <div className="text-sm font-medium text-gray-900">{course.name}</div>
                  </td>
                  <td className="px-6 py-4 text-sm text-gray-900">
                    {course.enrollment} students
                  </td>
                  <td className="px-6 py-4">
                    <div className="flex items-center space-x-2">
                      <div className="w-16 bg-gray-200 rounded-full h-2">
                        <div
                          className="bg-green-500 h-2 rounded-full"
                          style={{ width: `${course.completion}%` }}
                        ></div>
                      </div>
                      <span className="text-sm text-gray-900">{formatPercentage(course.completion)}</span>
                    </div>
                  </td>
                  <td className="px-6 py-4">
                    <div className="flex items-center space-x-2">
                      <span className="text-sm text-gray-900">{formatPercentage(course.averageScore)}</span>
                      <div className="w-12 bg-gray-200 rounded-full h-2">
                        <div
                          className="bg-blue-500 h-2 rounded-full"
                          style={{ width: `${course.averageScore}%` }}
                        ></div>
                      </div>
                    </div>
                  </td>
                  <td className="px-6 py-4">
                    <div className="flex items-center space-x-2">
                      <Calendar className="w-4 h-4 text-gray-400" />
                      <span className="text-sm text-gray-900">{formatDate(course.lastActivity)}</span>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Activity Feed */}
      <div className="bg-white rounded-lg shadow-sm border">
        <div className="p-6 border-b">
          <h3 className="text-lg font-semibold text-gray-900">Recent Activity</h3>
          <p className="text-sm text-gray-600 mt-1">Latest updates and engagement metrics</p>
        </div>
        <div className="p-6">
          <div className="space-y-4">
            <div className="flex items-start space-x-3">
              <div className="w-2 h-2 bg-green-500 rounded-full mt-2"></div>
              <div>
                <p className="text-sm text-gray-900">New course <span className="font-medium">"Advanced Machine Learning"</span> created on Blackboard</p>
                <p className="text-xs text-gray-500 mt-1">2 hours ago</p>
              </div>
            </div>
            <div className="flex items-start space-x-3">
              <div className="w-2 h-2 bg-blue-500 rounded-full mt-2"></div>
              <div>
                <p className="text-sm text-gray-900">Assignment <span className="font-medium">"Final Project Proposal"</span> graded for 45 students</p>
                <p className="text-xs text-gray-500 mt-1">4 hours ago</p>
              </div>
            </div>
            <div className="flex items-start space-x-3">
              <div className="w-2 h-2 bg-purple-500 rounded-full mt-2"></div>
              <div>
                <p className="text-sm text-gray-900">156 new student enrollments across all LMS platforms</p>
                <p className="text-xs text-gray-500 mt-1">1 day ago</p>
              </div>
            </div>
            <div className="flex items-start space-x-3">
              <div className="w-2 h-2 bg-orange-500 rounded-full mt-2"></div>
              <div>
                <p className="text-sm text-gray-900">LMS synchronization completed successfully</p>
                <p className="text-xs text-gray-500 mt-1">2 days ago</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Analytics;