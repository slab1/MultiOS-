const axios = require('axios');

class BlackboardLMSService {
  constructor(config) {
    this.baseUrl = config.baseUrl || process.env.BLACKBOARD_API_URL;
    this.clientId = config.clientId || process.env.BLACKBOARD_CLIENT_ID;
    this.clientSecret = config.clientId || process.env.BLACKBOARD_CLIENT_SECRET;
    this.accessToken = null;
  }

  // OAuth2 Token acquisition
  async getAccessToken() {
    const auth = Buffer.from(`${this.clientId}:${this.clientSecret}`).toString('base64');
    
    try {
      const response = await axios.post(
        `${this.baseUrl.replace('/learn/api/public/v3', '')}/learn/api/public/v1/oauth2/token`,
        'grant_type=client_credentials',
        {
          headers: {
            'Authorization': `Basic ${auth}`,
            'Content-Type': 'application/x-www-form-urlencoded'
          }
        }
      );

      this.accessToken = response.data.access_token;
      return response.data;
    } catch (error) {
      throw new Error(`Blackboard OAuth error: ${error.response?.data?.error || error.message}`);
    }
  }

  // Make authenticated API request
  async makeRequest(endpoint, options = {}) {
    if (!this.accessToken) {
      await this.getAccessToken();
    }

    const config = {
      method: options.method || 'GET',
      url: `${this.baseUrl}${endpoint}`,
      headers: {
        'Authorization': `Bearer ${this.accessToken}`,
        'Content-Type': 'application/json',
        'BBlackboard-API-Request': 'true',
        ...options.headers
      },
      ...options
    };

    try {
      const response = await axios(config);
      return response.data;
    } catch (error) {
      if (error.response?.status === 401) {
        // Refresh token
        await this.getAccessToken();
        config.headers.Authorization = `Bearer ${this.accessToken}`;
        const retryResponse = await axios(config);
        return retryResponse.data;
      }
      throw new Error(`Blackboard API error: ${error.response?.data?.message || error.message}`);
    }
  }

  // Get user profile
  async getUserProfile(userId = '@self') {
    return await this.makeRequest(`/users/${userId}`);
  }

  // Get courses
  async getCourses(limit = 100) {
    return await this.makeRequest(`/courses?limit=${limit}`);
  }

  // Get course details
  async getCourse(courseId) {
    return await this.makeRequest(`/courses/${courseId}`);
  }

  // Get course contents (learning modules)
  async getCourseContents(courseId) {
    return await this.makeRequest(`/courses/${courseId}/contents`);
  }

  // Get content details
  async getContent(courseId, contentId) {
    return await this.makeRequest(`/courses/${courseId}/contents/${contentId}`);
  }

  // Create learning module
  async createLearningModule(courseId, moduleData) {
    return await this.makeRequest(`/courses/${courseId}/contents`, {
      method: 'POST',
      data: {
        title: moduleData.title,
        description: moduleData.description,
        position: moduleData.position || 1,
        availability: {
          available: moduleData.available !== false,
          allowGuests: moduleData.allowGuests || false,
          adaptiveRelease: moduleData.adaptiveRelease || {},
          releaseDate: moduleData.releaseDate,
          endDate: moduleData.endDate
        },
        contentHandler: {
          id: 'resource/x-bb-document',
          type: 'Resource',
          payload: {
            body: moduleData.content || '',
            draftViewable: true
          }
        }
      }
    });
  }

  // Update learning module
  async updateLearningModule(courseId, contentId, updateData) {
    return await this.makeRequest(`/courses/${courseId}/contents/${contentId}`, {
      method: 'PATCH',
      data: {
        title: updateData.title,
        description: updateData.description,
        availability: updateData.availability
      }
    });
  }

  // Get course assignments
  async getCourseAssignments(courseId) {
    return await this.makeRequest(`/courses/${courseId}/gradebook/columns`);
  }

  // Create assignment
  async createAssignment(courseId, assignmentData) {
    return await this.makeRequest(`/courses/${courseId}/gradebook/columns`, {
      method: 'POST',
      data: {
        name: assignmentData.name,
        description: assignmentData.description,
        instructions: assignmentData.instructions,
        position: assignmentData.position || 1,
        score: {
          possible: assignmentData.points_possible || 100
        },
        availability: {
          available: assignmentData.available !== false,
          adaptiveRelease: assignmentData.adaptiveRelease || {}
        },
        due: assignmentData.due_at,
        text: assignmentData.name,
        attemptAllowed: assignmentData.attemptAllowed || 1
      }
    });
  }

  // Grade assignment
  async gradeAssignment(courseId, columnId, userId, gradeData) {
    return await this.makeRequest(
      `/courses/${courseId}/gradebook/columns/${columnId}/users/${userId}`,
      {
        method: 'PATCH',
        data: {
          score: {
            score: gradeData.score,
            text: gradeData.feedback || ''
          }
        }
      }
    );
  }

  // Get course students
  async getCourseStudents(courseId) {
    return await this.makeRequest(`/courses/${courseId}/users?role=Student`);
  }

  // Get course enrollments
  async getCourseEnrollments(courseId) {
    return await this.makeRequest(`/courses/${courseId}/users`);
  }

  // Create discussion board
  async createDiscussionBoard(courseId, forumData) {
    return await this.makeRequest(`/courses/${courseId}/discussion-forums`, {
      method: 'POST',
      data: {
        title: forumData.title,
        description: forumData.description,
        availability: {
          available: forumData.available !== false
        },
        type: forumData.type || 'Discussion',
        position: forumData.position || 1
      }
    });
  }

  // Get discussion topics
  async getDiscussionTopics(courseId, forumId) {
    return await this.makeRequest(`/courses/${courseId}/discussion-forums/${forumId}/posts`);
  }

  // Create discussion topic
  async createDiscussionTopic(courseId, forumId, topicData) {
    return await this.makeRequest(
      `/courses/${courseId}/discussion-forums/${forumId}/posts`,
      {
        method: 'POST',
        data: {
          text: topicData.text,
          title: topicData.title,
          parentId: topicData.parentId,
          position: topicData.position || 1
        }
      }
    );
  }

  // Get announcements
  async getAnnouncements(courseId) {
    return await this.makeRequest(`/courses/${courseId}/announcements`);
  }

  // Create announcement
  async createAnnouncement(courseId, announcementData) {
    return await this.makeRequest(`/courses/${courseId}/announcements`, {
      method: 'POST',
      data: {
        title: announcementData.title,
        body: {
          text: announcementData.text,
          type: 'Text'
        },
        availability: {
          duration: announcementData.duration || 'permanent',
          allowGuests: announcementData.allowGuests || false
        },
        startDate: announcementData.startDate,
        endDate: announcementData.endDate
      }
    });
  }

  // Get gradebook columns
  async getGradebookColumns(courseId) {
    return await this.makeRequest(`/courses/${courseId}/gradebook/columns`);
  }

  // Get gradebook column details
  async getGradebookColumn(courseId, columnId) {
    return await this.makeRequest(`/courses/${courseId}/gradebook/columns/${columnId}`);
  }

  // Get student grades
  async getStudentGrades(courseId, userId) {
    return await this.makeRequest(`/courses/${courseId}/gradebook/columns/users/${userId}`);
  }

  // Get course calendar
  async getCourseCalendar(courseId) {
    return await this.makeRequest(`/courses/${courseId}/calendar`);
  }

  // Create calendar event
  async createCalendarEvent(courseId, eventData) {
    return await this.makeRequest(`/courses/${courseId}/calendar`, {
      method: 'POST',
      data: {
        title: eventData.title,
        description: eventData.description,
        start: eventData.start,
        end: eventData.end,
        location: eventData.location,
        type: eventData.type || 'Course'
      }
    });
  }

  // Sync course data to MultiOS format
  syncCourseData(blackboardCourse) {
    return {
      external_id: blackboardCourse.id,
      title: blackboardCourse.name,
      description: blackboardCourse.description,
      code: blackboardCourse.courseId,
      instructor: blackboardCourse.created?.createdBy?.givenName + ' ' + blackboardCourse.created?.createdBy?.familyName || 'Unknown',
      start_date: blackboardCourse.startDate,
      end_date: blackboardCourse.endDate,
      students_count: blackboardCourse.enrollment?.studentCount || 0,
      syllabus: blackboardCourse.syllabus,
      workflow_state: blackboardCourse.availability?.available ? 'available' : 'unavailable'
    };
  }

  // Sync assignment/gradebook column to MultiOS format
  syncAssignmentData(blackboardColumn) {
    return {
      external_id: blackboardColumn.id,
      title: blackboardColumn.name,
      description: blackboardColumn.description,
      due_date: blackboardColumn.due,
      points_possible: blackboardColumn.score?.possible,
      submission_types: ['online_text_entry', 'online_upload'],
      grading_type: 'points',
      published: blackboardColumn.availability?.available || false,
      workflow_state: blackboardColumn.availability?.available ? 'available' : 'unavailable'
    };
  }

  // Sync learning module data
  syncLearningModuleData(blackboardContent) {
    return {
      external_id: blackboardContent.id,
      title: blackboardContent.title,
      description: blackboardContent.description,
      content: blackboardContent.contentHandler?.payload?.body,
      position: blackboardContent.position,
      available: blackboardContent.availability?.available,
      release_date: blackboardContent.availability?.adaptiveRelease?.releaseDate,
      end_date: blackboardContent.availability?.adaptiveRelease?.endDate,
      type: 'learning_module'
    };
  }

  // Sync enrollment data
  syncEnrollmentData(enrollment) {
    return {
      external_id: enrollment.id,
      user_id: enrollment.userId,
      course_id: enrollment.courseId,
      type: enrollment.courseRoleId,
      role: enrollment.courseRoleId,
      enrollment_state: enrollment.availability?.available ? 'active' : 'inactive',
      user: {
        name: enrollment.user?.name?.given + ' ' + enrollment.user?.name?.family,
        email: enrollment.user?.contact?.email,
        student_id: enrollment.user?.studentId
      }
    };
  }

  // Batch sync course data
  async syncAllCourseData(courseId) {
    try {
      const [course, contents, assignments, students] = await Promise.all([
        this.getCourse(courseId),
        this.getCourseContents(courseId),
        this.getCourseAssignments(courseId),
        this.getCourseStudents(courseId)
      ]);

      return {
        course: this.syncCourseData(course),
        learning_modules: contents.map(c => this.syncLearningModuleData(c)),
        assignments: assignments.map(a => this.syncAssignmentData(a)),
        students: students.map(s => this.syncEnrollmentData(s))
      };
    } catch (error) {
      throw new Error(`Failed to sync Blackboard course data: ${error.message}`);
    }
  }

  // Webhook validation (if implemented)
  validateWebhook(signature, timestamp, body, key) {
    // Implement Blackboard webhook validation if needed
    return true;
  }

  // Disconnect
  async disconnect() {
    this.accessToken = null;
    return true;
  }
}

module.exports = BlackboardLMSService;