const axios = require('axios');
const crypto = require('crypto');

class CanvasLMSService {
  constructor(config) {
    this.baseUrl = config.baseUrl || process.env.CANVAS_API_URL;
    this.clientId = config.clientId || process.env.CANVAS_CLIENT_ID;
    this.clientSecret = config.clientSecret || process.env.CANVAS_CLIENT_SECRET;
    this.redirectUri = config.redirectUri || process.env.CANVAS_REDIRECT_URI;
    this.accessToken = null;
    this.refreshToken = null;
  }

  // OAuth2 Authorization
  getAuthorizationUrl(state) {
    const params = new URLSearchParams({
      client_id: this.clientId,
      response_type: 'code',
      redirect_uri: this.redirectUri,
      scope: 'url:GET|/api/v1/courses url:GET|/api/v1/users/self profile:read',
      state: state
    });
    
    return `${this.baseUrl}/login/oauth2/auth?${params.toString()}`;
  }

  async exchangeCodeForTokens(code) {
    try {
      const response = await axios.post(`${this.baseUrl}/login/oauth2/token`, {
        grant_type: 'authorization_code',
        client_id: this.clientId,
        client_secret: this.clientSecret,
        redirect_uri: this.redirectUri,
        code: code
      });

      this.accessToken = response.data.access_token;
      this.refreshToken = response.data.refresh_token;
      
      return {
        access_token: response.data.access_token,
        refresh_token: response.data.refresh_token,
        expires_in: response.data.expires_in
      };
    } catch (error) {
      throw new Error(`Canvas OAuth error: ${error.response?.data?.error || error.message}`);
    }
  }

  // Refresh access token
  async refreshAccessToken() {
    if (!this.refreshToken) {
      throw new Error('No refresh token available');
    }

    try {
      const response = await axios.post(`${this.baseUrl}/login/oauth2/token`, {
        grant_type: 'refresh_token',
        client_id: this.clientId,
        client_secret: this.clientSecret,
        refresh_token: this.refreshToken
      });

      this.accessToken = response.data.access_token;
      this.refreshToken = response.data.refresh_token || this.refreshToken;
      
      return response.data;
    } catch (error) {
      throw new Error(`Canvas token refresh error: ${error.response?.data?.error || error.message}`);
    }
  }

  // Make authenticated API request
  async makeRequest(endpoint, options = {}) {
    const config = {
      method: options.method || 'GET',
      url: `${this.baseUrl}${endpoint}`,
      headers: {
        'Authorization': `Bearer ${this.accessToken}`,
        'Content-Type': 'application/json',
        ...options.headers
      },
      ...options
    };

    try {
      const response = await axios(config);
      return response.data;
    } catch (error) {
      if (error.response?.status === 401) {
        // Try to refresh token and retry
        await this.refreshAccessToken();
        config.headers.Authorization = `Bearer ${this.accessToken}`;
        const retryResponse = await axios(config);
        return retryResponse.data;
      }
      throw new Error(`Canvas API error: ${error.response?.data?.errors || error.message}`);
    }
  }

  // Get current user info
  async getCurrentUser() {
    return await this.makeRequest('/api/v1/users/self/profile');
  }

  // Get user's courses
  async getCourses(includeEnrollments = true) {
    const params = new URLSearchParams({
      enrollment_type: 'teacher,student',
      per_page: '100'
    });
    
    if (includeEnrollments) {
      params.append('include[]', 'enrollments');
    }
    
    return await this.makeRequest(`/api/v1/courses?${params.toString()}`);
  }

  // Get course details
  async getCourse(courseId) {
    return await this.makeRequest(`/api/v1/courses/${courseId}`);
  }

  // Get course assignments
  async getCourseAssignments(courseId) {
    return await this.makeRequest(`/api/v1/courses/${courseId}/assignments`);
  }

  // Get course modules
  async getCourseModules(courseId) {
    return await this.makeRequest(`/api/v1/courses/${courseId}/modules`);
  }

  // Get course students
  async getCourseStudents(courseId) {
    return await this.makeRequest(`/api/v1/courses/${courseId}/enrollments?type[]=StudentEnrollment&per_page=100`);
  }

  // Create assignment
  async createAssignment(courseId, assignmentData) {
    return await this.makeRequest(`/api/v1/courses/${courseId}/assignments`, {
      method: 'POST',
      data: {
        assignment: {
          name: assignmentData.name,
          description: assignmentData.description,
          due_at: assignmentData.due_at,
          points_possible: assignmentData.points_possible,
          submission_types: assignmentData.submission_types || ['online_upload'],
          published: assignmentData.published || false
        }
      }
    });
  }

  // Update assignment
  async updateAssignment(courseId, assignmentId, assignmentData) {
    return await this.makeRequest(`/api/v1/courses/${courseId}/assignments/${assignmentId}`, {
      method: 'PUT',
      data: {
        assignment: assignmentData
      }
    });
  }

  // Get assignment submissions
  async getAssignmentSubmissions(courseId, assignmentId) {
    return await this.makeRequest(`/api/v1/courses/${courseId}/assignments/${assignmentId}/submissions`);
  }

  // Grade submission
  async gradeSubmission(courseId, assignmentId, userId, gradeData) {
    return await this.makeRequest(
      `/api/v1/courses/${courseId}/assignments/${assignmentId}/submissions/${userId}`,
      {
        method: 'PUT',
        data: {
          submission: {
            posted_grade: gradeData.posted_grade,
            comment: gradeData.comment
          }
        }
      }
    );
  }

  // Get course announcements
  async getCourseAnnouncements(courseId) {
    return await this.makeRequest(`/api/v1/courses/${courseId}/discussion_topics?only_announcements=true`);
  }

  // Create announcement
  async createAnnouncement(courseId, announcementData) {
    return await this.makeRequest(`/api/v1/courses/${courseId}/discussion_topics`, {
      method: 'POST',
      data: {
        title: announcementData.title,
        message: announcementData.message,
        is_announcement: true,
        published: announcementData.published || true
      }
    });
  }

  // Get course pages (content)
  async getCoursePages(courseId) {
    return await this.makeRequest(`/api/v1/courses/${courseId}/pages`);
  }

  // Create course page
  async createCoursePage(courseId, pageData) {
    return await this.makeRequest(`/api/v1/courses/${courseId}/pages`, {
      method: 'POST',
      data: {
        wiki_page: {
          title: pageData.title,
          body: pageData.body,
          published: pageData.published || false
        }
      }
    });
  }

  // Get course groups
  async getCourseGroups(courseId) {
    return await this.makeRequest(`/api/v1/courses/${courseId}/groups`);
  }

  // Sync course data to MultiOS format
  syncCourseData(canvasCourse) {
    return {
      external_id: canvasCourse.id.toString(),
      title: canvasCourse.name,
      description: canvasCourse.course_code,
      code: canvasCourse.course_code,
      instructor: canvasCourse.teacher_enrollments?.[0]?.user?.name || 'Unknown',
      start_date: canvasCourse.start_at,
      end_date: canvasCourse.end_at,
      students_count: canvasCourse.enrollments?.filter(e => e.type === 'StudentEnrollment').length || 0,
      syllabus: canvasCourse.syllabus_body,
      workflow_state: canvasCourse.workflow_state,
      enrollment_term_id: canvasCourse.enrollment_term_id
    };
  }

  // Sync assignment data to MultiOS format
  syncAssignmentData(canvasAssignment) {
    return {
      external_id: canvasAssignment.id.toString(),
      title: canvasAssignment.name,
      description: canvasAssignment.description,
      due_date: canvasAssignment.due_at,
      points_possible: canvasAssignment.points_possible,
      submission_types: canvasAssignment.submission_types,
      grading_type: canvasAssignment.grading_type,
      published: canvasAssignment.published,
      workflow_state: canvasAssignment.workflow_state
    };
  }

  // Sync student enrollment data
  syncEnrollmentData(enrollment) {
    return {
      external_id: enrollment.id.toString(),
      user_id: enrollment.user_id.toString(),
      course_id: enrollment.course_id.toString(),
      type: enrollment.type,
      role: enrollment.role,
      enrollment_state: enrollment.enrollment_state,
      user: {
        name: enrollment.user?.name,
        email: enrollment.user?.login_id,
        sortable_name: enrollment.user?.sortable_name
      }
    };
  }

  // Batch sync operations
  async syncAllCourseData(courseId) {
    try {
      const [course, assignments, students, modules] = await Promise.all([
        this.getCourse(courseId),
        this.getCourseAssignments(courseId),
        this.getCourseStudents(courseId),
        this.getCourseModules(courseId)
      ]);

      return {
        course: this.syncCourseData(course),
        assignments: assignments.map(a => this.syncAssignmentData(a)),
        students: students.map(s => this.syncEnrollmentData(s)),
        modules: modules
      };
    } catch (error) {
      throw new Error(`Failed to sync course data: ${error.message}`);
    }
  }

  // Webhook verification
  verifyWebhookSignature(payload, signature, secret) {
    const computedSignature = crypto
      .createHmac('sha256', secret)
      .update(payload)
      .digest('hex');
    
    return computedSignature === signature;
  }

  // Handle webhook events
  async handleWebhook(eventType, eventData) {
    switch (eventType) {
      case 'course_created':
      case 'course_updated':
        return this.syncCourseData(eventData.course);
      
      case 'assignment_created':
      case 'assignment_updated':
        return this.syncAssignmentData(eventData.assignment);
      
      case 'submission_created':
      case 'submission_updated':
        return this.syncSubmissionData(eventData.submission);
      
      default:
        console.log(`Unhandled Canvas webhook event: ${eventType}`);
        return null;
    }
  }

  // Disconnect/cleanup
  async disconnect() {
    this.accessToken = null;
    this.refreshToken = null;
    return true;
  }
}

module.exports = CanvasLMSService;