import axios from 'axios';

// Create axios instance
export const api = axios.create({
  baseURL: process.env.REACT_APP_API_URL || 'http://localhost:5000/api',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Request interceptor
api.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem('token');
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

// Response interceptor
api.interceptors.response.use(
  (response) => {
    return response;
  },
  (error) => {
    if (error.response?.status === 401) {
      // Token expired or invalid
      localStorage.removeItem('token');
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);

// Authentication API
export const authAPI = {
  login: (credentials: { email: string; password: string }) =>
    api.post('/auth/login', credentials),
  
  register: (userData: any) =>
    api.post('/auth/register', userData),
  
  logout: () =>
    api.post('/auth/logout'),
  
  me: () =>
    api.get('/auth/me'),
  
  updateProfile: (userData: any) =>
    api.put('/auth/profile', userData),
  
  changePassword: (passwordData: { currentPassword: string; newPassword: string }) =>
    api.put('/auth/change-password', passwordData),
  
  forgotPassword: (email: string) =>
    api.post('/auth/forgot-password', { email }),
  
  resetPassword: (token: string, newPassword: string) =>
    api.post('/auth/reset-password', { token, newPassword }),
  
  refreshToken: () =>
    api.post('/auth/refresh'),
};

// Papers API
export const papersAPI = {
  getMyPapers: (params?: { page?: number; limit?: number }) =>
    api.get('/papers/my-papers', { params }),
  
  getPapers: (params?: any) =>
    api.get('/papers', { params }),
  
  getPaper: (id: string) =>
    api.get(`/papers/${id}`),
  
  createPaper: (paperData: any) =>
    api.post('/papers', paperData, {
      headers: { 'Content-Type': 'multipart/form-data' },
    }),
  
  updatePaper: (id: string, paperData: any) =>
    api.put(`/papers/${id}`, paperData, {
      headers: { 'Content-Type': 'multipart/form-data' },
    }),
  
  submitPaper: (id: string, submissionData: any) =>
    api.post(`/papers/${id}/submit`, submissionData),
  
  createVersion: (id: string, changes: string) =>
    api.post(`/papers/${id}/version`, { changes }),
  
  deletePaper: (id: string) =>
    api.delete(`/papers/${id}`),
  
  downloadFile: (paperId: string, fileId: string) =>
    api.get(`/papers/${paperId}/files/${fileId}/download`, {
      responseType: 'blob',
    }),
};

// Reviews API
export const reviewsAPI = {
  getMyAssignments: (params?: { page?: number; limit?: number; status?: string }) =>
    api.get('/reviews/my-assignments', { params }),
  
  getReview: (id: string) =>
    api.get(`/reviews/${id}`),
  
  updateReview: (id: string, reviewData: any) =>
    api.put(`/reviews/${id}`, reviewData),
  
  submitReview: (id: string, reviewData: any) =>
    api.post(`/reviews/${id}/submit`, reviewData),
  
  addComment: (id: string, commentData: { section: string; comment: string; pageNumber?: number }) =>
    api.post(`/reviews/${id}/comment`, commentData),
  
  assignReview: (assignmentData: {
    paperId: string;
    reviewerId: string;
    dueDate: string;
    blindReview?: boolean;
  }) =>
    api.post('/reviews/assign', assignmentData),
  
  declineReview: (id: string, reason?: string) =>
    api.post(`/reviews/${id}/decline`, { reason }),
  
  getOverdueReviews: () =>
    api.get('/reviews/overdue/list'),
  
  getReviewerStats: () =>
    api.get('/reviews/stats/reviewer'),
  
  getQualityMetrics: () =>
    api.get('/reviews/stats/quality'),
};

// Conferences API
export const conferencesAPI = {
  getConferences: (params?: any) =>
    api.get('/conferences', { params }),
  
  getConference: (id: string) =>
    api.get(`/conferences/${id}`),
  
  createConference: (conferenceData: any) =>
    api.post('/conferences', conferenceData),
  
  updateConference: (id: string, conferenceData: any) =>
    api.put(`/conferences/${id}`, conferenceData),
  
  addTrack: (id: string, trackData: any) =>
    api.post(`/conferences/${id}/tracks`, trackData),
  
  addImportantDate: (id: string, dateData: any) =>
    api.post(`/conferences/${id}/dates`, dateData),
  
  updateStatus: (id: string, status: string) =>
    api.put(`/conferences/${id}/status`, { status }),
  
  getSubmissions: (id: string, params?: any) =>
    api.get(`/conferences/${id}/submissions`, { params }),
  
  updateStats: (id: string) =>
    api.post(`/conferences/${id}/update-stats`),
  
  searchUpcoming: (limit?: number) =>
    api.get('/conferences/search/upcoming', { params: { limit } }),
  
  searchByArea: (area: string) =>
    api.get(`/conferences/search/by-area/${encodeURIComponent(area)}`),
  
  deleteConference: (id: string) =>
    api.delete(`/conferences/${id}`),
};

// Citations API
export const citationsAPI = {
  getMyCitations: (params?: any) =>
    api.get('/citations/my-citations', { params }),
  
  getCitations: (params?: any) =>
    api.get('/citations', { params }),
  
  getCitation: (id: string) =>
    api.get(`/citations/${id}`),
  
  createCitation: (citationData: any) =>
    api.post('/citations', citationData),
  
  updateCitation: (id: string, citationData: any) =>
    api.put(`/citations/${id}`, citationData),
  
  importBibTeX: (bibtex: string, citationIds?: string[]) =>
    api.post('/citations/import/bibtex', { bibtex, citationIds }),
  
  linkToPaper: (id: string, linkData: {
    paperId: string;
    context?: string;
    relevance?: string;
  }) =>
    api.post(`/citations/${id}/link-to-paper`, linkData),
  
  addRelatedCitation: (id: string, relatedData: {
    relatedCitationId: string;
    relationship: string;
    strength?: number;
  }) =>
    api.post(`/citations/${id}/related`, relatedData),
  
  addNote: (id: string, content: string) =>
    api.post(`/citations/${id}/notes`, { content }),
  
  verifyCitation: (id: string) =>
    api.post(`/citations/${id}/verify`),
  
  updateMetrics: (id: string, metrics: any) =>
    api.post(`/citations/${id}/metrics`, metrics),
  
  deleteCitation: (id: string) =>
    api.delete(`/citations/${id}`),
};

// Users API
export const usersAPI = {
  getUsers: (params?: any) =>
    api.get('/users', { params }),
  
  getUser: (id: string) =>
    api.get(`/users/${id}`),
  
  searchResearchers: (params?: any) =>
    api.get('/users/search/researchers', { params }),
  
  getAvailableReviewers: (params: {
    researchArea: string;
    expertise?: string;
    maxReviews?: number;
  }) =>
    api.get('/users/reviewers/available', { params }),
  
  updateProfile: (userData: any) =>
    api.put('/users/profile', userData),
  
  updateUserRole: (id: string, roleData: any) =>
    api.put(`/users/${id}/role`, roleData),
  
  verifyUser: (id: string) =>
    api.post(`/users/${id}/verify`),
  
  deactivateUser: (id: string, reason?: string) =>
    api.post(`/users/${id}/deactivate`, { reason }),
  
  getUserStats: (id: string) =>
    api.get(`/users/${id}/stats`),
  
  getTopResearchers: (params?: {
    metric?: string;
    researchArea?: string;
    limit?: number;
    timeRange?: string;
  }) =>
    api.get('/users/rankings/top-researchers', { params }),
};

// Analytics API
export const analyticsAPI = {
  getDashboard: () =>
    api.get('/analytics/dashboard'),
  
  getPaperAnalytics: (params?: {
    timeRange?: string;
    paperId?: string;
  }) =>
    api.get('/analytics/papers', { params }),
  
  getReviewAnalytics: (params?: {
    timeRange?: string;
  }) =>
    api.get('/analytics/reviews', { params }),
  
  getPlatformAnalytics: (params?: {
    timeRange?: string;
  }) =>
    api.get('/analytics/platform', { params }),
  
  getCitationAnalytics: (params?: {
    timeRange?: string;
  }) =>
    api.get('/analytics/citations', { params }),
  
  exportData: (params: {
    type: string;
    format: string;
    timeRange?: string;
  }) =>
    api.get('/analytics/export', { params }),
};

// LaTeX API
export const latexAPI = {
  compile: (compileData: {
    latex: string;
    mainFile?: string;
    packages?: string[];
    bibliographyEngine?: string;
  }) =>
    api.post('/latex/compile', compileData),
  
  validate: (validationData: {
    paperId: string;
    validationType?: string;
    strictMode?: boolean;
  }) =>
    api.post('/latex/validate', validationData),
  
  convert: (conversionData: {
    latex: string;
    targetFormat: string;
    options?: any;
  }) =>
    api.post('/latex/convert', conversionData),
  
  getTemplate: (templateType: string) =>
    api.get(`/latex/templates/${templateType}`),
};

export default api;