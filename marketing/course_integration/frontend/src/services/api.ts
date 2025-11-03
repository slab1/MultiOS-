// API base configuration
const API_BASE_URL = process.env.REACT_APP_API_URL || 'http://localhost:5000/api';

// Generic API request function
const apiRequest = async (endpoint: string, options: RequestInit = {}) => {
  const url = `${API_BASE_URL}${endpoint}`;
  
  const config: RequestInit = {
    headers: {
      'Content-Type': 'application/json',
      ...options.headers,
    },
    ...options,
  };

  // Add auth token if available
  const token = localStorage.getItem('token');
  if (token) {
    config.headers = {
      ...config.headers,
      Authorization: `Bearer ${token}`,
    };
  }

  try {
    const response = await fetch(url, config);
    
    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.message || `HTTP error! status: ${response.status}`);
    }
    
    return await response.json();
  } catch (error) {
    console.error('API request failed:', error);
    throw error;
  }
};

// Auth API
export const authAPI = {
  login: async (email: string, password: string) => {
    return apiRequest('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ email, password }),
    });
  },

  register: async (userData: {
    firstName: string;
    lastName: string;
    email: string;
    password: string;
    role?: string;
  }) => {
    return apiRequest('/auth/register', {
      method: 'POST',
      body: JSON.stringify(userData),
    });
  },

  logout: () => {
    localStorage.removeItem('token');
    localStorage.removeItem('user');
  },

  getCurrentUser: async () => {
    return apiRequest('/auth/me');
  },

  refreshToken: async () => {
    return apiRequest('/auth/refresh', {
      method: 'POST',
    });
  },
};

// LMS API
export const lmsAPI = {
  getIntegrations: async () => {
    return apiRequest('/lms/integrations');
  },

  connectLMS: async (lmsData: {
    platform: string;
    apiKey: string;
    baseUrl: string;
    credentials: Record<string, any>;
  }) => {
    return apiRequest('/lms/connect', {
      method: 'POST',
      body: JSON.stringify(lmsData),
    });
  },

  disconnectLMS: async (integrationId: string) => {
    return apiRequest(`/lms/integrations/${integrationId}`, {
      method: 'DELETE',
    });
  },

  syncLMS: async (integrationId: string) => {
    return apiRequest(`/lms/sync/${integrationId}`, {
      method: 'POST',
    });
  },

  getSyncStatus: async (integrationId: string) => {
    return apiRequest(`/lms/sync-status/${integrationId}`);
  },

  updateCredentials: async (integrationId: string, credentials: Record<string, any>) => {
    return apiRequest(`/lms/integrations/${integrationId}/credentials`, {
      method: 'PUT',
      body: JSON.stringify({ credentials }),
    });
  },

  testConnection: async (lmsData: {
    platform: string;
    baseUrl: string;
    credentials: Record<string, any>;
  }) => {
    return apiRequest('/lms/test-connection', {
      method: 'POST',
      body: JSON.stringify(lmsData),
    });
  },
};

// Courses API
export const coursesAPI = {
  getCourses: async (filters?: {
    lms?: string;
    search?: string;
    page?: number;
    limit?: number;
  }) => {
    const queryParams = new URLSearchParams();
    if (filters) {
      Object.entries(filters).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          queryParams.append(key, value.toString());
        }
      });
    }
    
    return apiRequest(`/courses?${queryParams.toString()}`);
  },

  getCourse: async (courseId: string) => {
    return apiRequest(`/courses/${courseId}`);
  },

  createCourse: async (courseData: {
    title: string;
    description: string;
    lmsPlatform: string;
    startDate: string;
    endDate: string;
    instructor: string;
    enrollmentLimit?: number;
  }) => {
    return apiRequest('/courses', {
      method: 'POST',
      body: JSON.stringify(courseData),
    });
  },

  updateCourse: async (courseId: string, courseData: Partial<{
    title: string;
    description: string;
    startDate: string;
    endDate: string;
    instructor: string;
    enrollmentLimit: number;
    isActive: boolean;
  }>) => {
    return apiRequest(`/courses/${courseId}`, {
      method: 'PUT',
      body: JSON.stringify(courseData),
    });
  },

  deleteCourse: async (courseId: string) => {
    return apiRequest(`/courses/${courseId}`, {
      method: 'DELETE',
    });
  },

  getEnrollments: async (courseId: string) => {
    return apiRequest(`/courses/${courseId}/enrollments`);
  },

  enrollStudent: async (courseId: string, studentId: string) => {
    return apiRequest(`/courses/${courseId}/enroll`, {
      method: 'POST',
      body: JSON.stringify({ studentId }),
    });
  },

  unenrollStudent: async (courseId: string, studentId: string) => {
    return apiRequest(`/courses/${courseId}/unenroll/${studentId}`, {
      method: 'DELETE',
    });
  },
};

// Assignments API
export const assignmentsAPI = {
  getAssignments: async (filters?: {
    course?: string;
    status?: string;
    lms?: string;
    search?: string;
    page?: number;
    limit?: number;
  }) => {
    const queryParams = new URLSearchParams();
    if (filters) {
      Object.entries(filters).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          queryParams.append(key, value.toString());
        }
      });
    }
    
    return apiRequest(`/assignments?${queryParams.toString()}`);
  },

  getAssignment: async (assignmentId: string) => {
    return apiRequest(`/assignments/${assignmentId}`);
  },

  createAssignment: async (assignmentData: {
    title: string;
    description: string;
    courseId: string;
    dueDate: string;
    lmsPlatform: string;
    maxSubmissions?: number;
    points?: number;
    attachments?: File[];
  }) => {
    const formData = new FormData();
    Object.entries(assignmentData).forEach(([key, value]) => {
      if (key === 'attachments' && Array.isArray(value)) {
        value.forEach((file) => formData.append('attachments', file));
      } else if (value !== undefined && value !== null) {
        formData.append(key, typeof value === 'string' ? value : JSON.stringify(value));
      }
    });

    return apiRequest('/assignments', {
      method: 'POST',
      body: formData,
      headers: {}, // Let browser set Content-Type for FormData
    });
  },

  updateAssignment: async (assignmentId: string, assignmentData: Partial<{
    title: string;
    description: string;
    dueDate: string;
    maxSubmissions: number;
    points: number;
    isActive: boolean;
  }>) => {
    return apiRequest(`/assignments/${assignmentId}`, {
      method: 'PUT',
      body: JSON.stringify(assignmentData),
    });
  },

  deleteAssignment: async (assignmentId: string) => {
    return apiRequest(`/assignments/${assignmentId}`, {
      method: 'DELETE',
    });
  },

  publishAssignment: async (assignmentId: string) => {
    return apiRequest(`/assignments/${assignmentId}/publish`, {
      method: 'POST',
    });
  },

  getSubmissions: async (assignmentId: string) => {
    return apiRequest(`/assignments/${assignmentId}/submissions`);
  },

  gradeSubmission: async (assignmentId: string, submissionId: string, grade: {
    score: number;
    feedback: string;
    rubric?: Record<string, any>;
  }) => {
    return apiRequest(`/assignments/${assignmentId}/submissions/${submissionId}/grade`, {
      method: 'POST',
      body: JSON.stringify(grade),
    });
  },

  downloadSubmission: async (assignmentId: string, submissionId: string) => {
    return apiRequest(`/assignments/${assignmentId}/submissions/${submissionId}/download`, {
      method: 'GET',
    });
  },
};

// Students API
export const studentsAPI = {
  getStudents: async (filters?: {
    status?: string;
    search?: string;
    course?: string;
    page?: number;
    limit?: number;
  }) => {
    const queryParams = new URLSearchParams();
    if (filters) {
      Object.entries(filters).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          queryParams.append(key, value.toString());
        }
      });
    }
    
    return apiRequest(`/students?${queryParams.toString()}`);
  },

  getStudent: async (studentId: string) => {
    return apiRequest(`/students/${studentId}`);
  },

  createStudent: async (studentData: {
    firstName: string;
    lastName: string;
    email: string;
    phone?: string;
    location?: string;
  }) => {
    return apiRequest('/students', {
      method: 'POST',
      body: JSON.stringify(studentData),
    });
  },

  updateStudent: async (studentId: string, studentData: Partial<{
    firstName: string;
    lastName: string;
    email: string;
    phone: string;
    location: string;
    status: string;
  }>) => {
    return apiRequest(`/students/${studentId}`, {
      method: 'PUT',
      body: JSON.stringify(studentData),
    });
  },

  deleteStudent: async (studentId: string) => {
    return apiRequest(`/students/${studentId}`, {
      method: 'DELETE',
    });
  },

  getStudentProgress: async (studentId: string) => {
    return apiRequest(`/students/${studentId}/progress`);
  },

  getStudentGrades: async (studentId: string) => {
    return apiRequest(`/students/${studentId}/grades`);
  },

  sendEmail: async (studentId: string, emailData: {
    subject: string;
    message: string;
    template?: string;
  }) => {
    return apiRequest(`/students/${studentId}/email`, {
      method: 'POST',
      body: JSON.stringify(emailData),
    });
  },
};

// Analytics API
export const analyticsAPI = {
  getDashboardStats: async (filters?: {
    period?: string;
    lms?: string;
  }) => {
    const queryParams = new URLSearchParams();
    if (filters) {
      Object.entries(filters).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          queryParams.append(key, value.toString());
        }
      });
    }
    
    return apiRequest(`/analytics/dashboard?${queryParams.toString()}`);
  },

  getCourseAnalytics: async (courseId: string, filters?: {
    period?: string;
  }) => {
    const queryParams = new URLSearchParams();
    if (filters) {
      Object.entries(filters).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          queryParams.append(key, value.toString());
        }
      });
    }
    
    return apiRequest(`/analytics/courses/${courseId}?${queryParams.toString()}`);
  },

  getLMSAnalytics: async (filters?: {
    period?: string;
    platform?: string;
  }) => {
    const queryParams = new URLSearchParams();
    if (filters) {
      Object.entries(filters).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          queryParams.append(key, value.toString());
        }
      });
    }
    
    return apiRequest(`/analytics/lms?${queryParams.toString()}`);
  },

  exportAnalytics: async (type: 'csv' | 'pdf', filters?: Record<string, any>) => {
    const queryParams = new URLSearchParams();
    if (filters) {
      Object.entries(filters).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          queryParams.append(key, value.toString());
        }
      });
    }
    
    return fetch(`${API_BASE_URL}/analytics/export/${type}?${queryParams.toString()}`, {
      headers: {
        Authorization: `Bearer ${localStorage.getItem('token')}`,
      },
    });
  },

  getStudentEngagement: async (studentId: string) => {
    return apiRequest(`/analytics/students/${studentId}/engagement`);
  },

  getCourseCompletionRates: async (filters?: {
    period?: string;
    lms?: string;
  }) => {
    const queryParams = new URLSearchParams();
    if (filters) {
      Object.entries(filters).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          queryParams.append(key, value.toString());
        }
      });
    }
    
    return apiRequest(`/analytics/completion-rates?${queryParams.toString()}`);
  },
};

// Settings API
export const settingsAPI = {
  getSettings: async () => {
    return apiRequest('/settings');
  },

  updateSettings: async (settings: Record<string, any>) => {
    return apiRequest('/settings', {
      method: 'PUT',
      body: JSON.stringify(settings),
    });
  },

  getNotificationSettings: async () => {
    return apiRequest('/settings/notifications');
  },

  updateNotificationSettings: async (notifications: Record<string, any>) => {
    return apiRequest('/settings/notifications', {
      method: 'PUT',
      body: JSON.stringify(notifications),
    });
  },

  getSecuritySettings: async () => {
    return apiRequest('/settings/security');
  },

  updateSecuritySettings: async (security: {
    twoFactorEnabled: boolean;
    sessionTimeout: number;
    passwordPolicy: Record<string, any>;
  }) => {
    return apiRequest('/settings/security', {
      method: 'PUT',
      body: JSON.stringify(security),
    });
  },

  changePassword: async (passwordData: {
    currentPassword: string;
    newPassword: string;
    confirmPassword: string;
  }) => {
    return apiRequest('/settings/change-password', {
      method: 'POST',
      body: JSON.stringify(passwordData),
    });
  },
};

// Utility functions
export const handleApiError = (error: any) => {
  if (error.message === 'Unauthorized') {
    authAPI.logout();
    window.location.href = '/login';
    return;
  }
  
  console.error('API Error:', error);
  throw error;
};

// File upload utility
export const uploadFile = async (file: File, endpoint: string) => {
  const formData = new FormData();
  formData.append('file', file);

  const token = localStorage.getItem('token');
  const response = await fetch(`${API_BASE_URL}${endpoint}`, {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${token}`,
    },
    body: formData,
  });

  if (!response.ok) {
    throw new Error('File upload failed');
  }

  return await response.json();
};

export default {
  auth: authAPI,
  lms: lmsAPI,
  courses: coursesAPI,
  assignments: assignmentsAPI,
  students: studentsAPI,
  analytics: analyticsAPI,
  settings: settingsAPI,
  uploadFile,
  handleApiError,
};