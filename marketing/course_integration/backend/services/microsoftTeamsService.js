const axios = require('axios');
const { Client } = require('@microsoft/microsoft-graph-client');
require('isomorphic-fetch');

class MicrosoftTeamsService {
  constructor(config) {
    this.clientId = config.clientId || process.env.MICROSOFT_CLIENT_ID;
    this.clientSecret = config.clientSecret || process.env.MICROSOFT_CLIENT_SECRET;
    this.tenantId = config.tenantId || process.env.MICROSOFT_TENANT_ID;
    this.redirectUri = config.redirectUri || process.env.MICROSOFT_REDIRECT_URI;
    this.accessToken = null;
    this.refreshToken = null;
    this.baseUrl = 'https://graph.microsoft.com/v1.0';
  }

  // OAuth2 Authorization
  getAuthorizationUrl(state) {
    const params = new URLSearchParams({
      client_id: this.clientId,
      response_type: 'code',
      redirect_uri: this.redirectUri,
      response_mode: 'query',
      scope: [
        'https://graph.microsoft.com/User.Read',
        'https://graph.microsoft.com/Edu.ReadWrite',
        'https://graph.microsoft.com/Group.ReadWrite.All',
        'https://graph.microsoft.com/Calendars.ReadWrite',
        'https://graph.microsoft.com/Files.ReadWrite.All'
      ].join(' '),
      state: state
    });
    
    return `https://login.microsoftonline.com/${this.tenantId}/oauth2/v2.0/authorize?${params.toString()}`;
  }

  // Exchange authorization code for tokens
  async getTokens(code) {
    const tokenUrl = `https://login.microsoftonline.com/${this.tenantId}/oauth2/v2.0/token`;
    const params = new URLSearchParams({
      client_id: this.clientId,
      client_secret: this.clientSecret,
      code: code,
      redirect_uri: this.redirectUri,
      grant_type: 'authorization_code'
    });

    try {
      const response = await axios.post(tokenUrl, params.toString(), {
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded'
        }
      });

      this.accessToken = response.data.access_token;
      this.refreshToken = response.data.refresh_token;
      
      return {
        access_token: response.data.access_token,
        refresh_token: response.data.refresh_token,
        expires_in: response.data.expires_in
      };
    } catch (error) {
      throw new Error(`Microsoft OAuth error: ${error.response?.data?.error_description || error.message}`);
    }
  }

  // Refresh access token
  async refreshAccessToken() {
    if (!this.refreshToken) {
      throw new Error('No refresh token available');
    }

    const tokenUrl = `https://login.microsoftonline.com/${this.tenantId}/oauth2/v2.0/token`;
    const params = new URLSearchParams({
      client_id: this.clientId,
      client_secret: this.clientSecret,
      refresh_token: this.refreshToken,
      grant_type: 'refresh_token'
    });

    try {
      const response = await axios.post(tokenUrl, params.toString(), {
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded'
        }
      });

      this.accessToken = response.data.access_token;
      this.refreshToken = response.data.refresh_token || this.refreshToken;
      
      return response.data;
    } catch (error) {
      throw new Error(`Microsoft token refresh error: ${error.response?.data?.error_description || error.message}`);
    }
  }

  // Get Microsoft Graph client
  getGraphClient() {
    return Client.init({
      authProvider: async (done) => {
        if (!this.accessToken) {
          await this.refreshAccessToken();
        }
        done(null, this.accessToken);
      }
    });
  }

  // Get current user profile
  async getCurrentUser() {
    const client = this.getGraphClient();
    return await client.api('/me').get();
  }

  // Get education classes
  async getEducationClasses() {
    const client = this.getGraphClient();
    const response = await client.api('/education/classes').get();
    return response.value || [];
  }

  // Get education class details
  async getEducationClass(classId) {
    const client = this.getGraphClient();
    return await client.api(`/education/classes/${classId}`).get();
  }

  // Create education class
  async createEducationClass(classData) {
    const client = this.getGraphClient();
    const classObj = {
      displayName: classData.displayName,
      mailNickname: classData.mailNickname || classData.displayName.toLowerCase().replace(/\s+/g, ''),
      description: classData.description || '',
      externalId: classData.externalId || classData.displayName,
      externalSource: 'multios',
      externalSourceId: classData.externalSourceId,
      createdBy: {
        user: {
          id: classData.createdBy || 'me'
        }
      },
      teachers: classData.teachers || [],
      students: classData.students || []
    };

    return await client.api('/education/classes').post(classObj);
  }

  // Update education class
  async updateEducationClass(classId, updateData) {
    const client = this.getGraphClient();
    return await client.api(`/education/classes/${classId}`).patch(updateData);
  }

  // Delete education class
  async deleteEducationClass(classId) {
    const client = this.getGraphClient();
    await client.api(`/education/classes/${classId}`).delete();
    return true;
  }

  // Get class members (teachers and students)
  async getClassMembers(classId) {
    const client = this.getGraphClient();
    const [teachers, students] = await Promise.all([
      client.api(`/education/classes/${classId}/teachers`).get(),
      client.api(`/education/classes/${classId}/students`).get()
    ]);

    return {
      teachers: teachers.value || [],
      students: students.value || []
    };
  }

  // Add teacher to class
  async addTeacherToClass(classId, teacherId) {
    const client = this.getGraphClient();
    return await client.api(`/education/classes/${classId}/teachers/${teacherId}`).put({});
  }

  // Add student to class
  async addStudentToClass(classId, studentId) {
    const client = this.getGraphClient();
    return await client.api(`/education/classes/${classId}/students/${studentId}`).put({});
  }

  // Remove teacher from class
  async removeTeacherFromClass(classId, teacherId) {
    const client = this.getGraphClient();
    await client.api(`/education/classes/${classId}/teachers/${teacherId}`).delete();
    return true;
  }

  // Remove student from class
  async removeStudentFromClass(classId, studentId) {
    const client = this.getGraphClient();
    await client.api(`/education/classes/${classId}/students/${studentId}`).delete();
    return true;
  }

  // Get class assignments
  async getClassAssignments(classId) {
    const client = this.getGraphClient();
    const response = await client.api(`/education/classes/${classId}/assignments`).get();
    return response.value || [];
  }

  // Create assignment
  async createAssignment(classId, assignmentData) {
    const client = this.getGraphClient();
    
    const assignment = {
      displayName: assignmentData.displayName,
      dueDateTime: assignmentData.dueDateTime,
      instructions: {
        contentType: 'text',
        content: assignmentData.instructions || ''
      },
      assignmentCategory: assignmentData.assignmentCategory || 'assignment',
      grading: {
        '@odata.type': '#microsoft.graph.educationAssignmentPointsGradeType',
        maxPoints: assignmentData.maxPoints || 100
      },
      status: 'draft',
      allowStudentsToAddResourcesToSubmission: assignmentData.allowStudentsToAddResourcesToSubmission || true,
      assignTo: assignmentData.assignTo || {
        '@odata.type': '#microsoft.graph.educationAssignmentClassRecipient'
      },
      createdBy: {
        user: {
          id: 'me'
        }
      }
    };

    return await client.api(`/education/classes/${classId}/assignments`).post(assignment);
  }

  // Update assignment
  async updateAssignment(classId, assignmentId, updateData) {
    const client = this.getGraphClient();
    return await client.api(`/education/classes/${classId}/assignments/${assignmentId}`).patch(updateData);
  }

  // Publish assignment
  async publishAssignment(classId, assignmentId) {
    const client = this.getGraphClient();
    return await client.api(`/education/classes/${classId}/assignments/${assignmentId}/publish`).post({});
  }

  // Delete assignment
  async deleteAssignment(classId, assignmentId) {
    const client = this.getGraphClient();
    await client.api(`/education/classes/${classId}/assignments/${assignmentId}`).delete();
    return true;
  }

  // Get assignment submissions
  async getAssignmentSubmissions(classId, assignmentId) {
    const client = this.getGraphClient();
    const response = await client.api(`/education/classes/${classId}/assignments/${assignmentId}/submissions`).get();
    return response.value || [];
  }

  // Grade submission
  async gradeSubmission(classId, assignmentId, submissionId, gradeData) {
    const client = this.getGraphClient();
    
    // Update submission grade
    const updateData = {
      points: gradeData.points || 0,
      feedback: {
        text: gradeData.feedback || ''
      }
    };

    const submission = await client.api(`/education/classes/${classId}/assignments/${assignmentId}/submissions/${submissionId}`).patch(updateData);

    // Return submission
    await client.api(`/education/classes/${classId}/assignments/${assignmentId}/submissions/${submissionId}/return`).post({});
    
    return submission;
  }

  // Get assignment resources
  async getAssignmentResources(classId, assignmentId) {
    const client = this.getGraphClient();
    const response = await client.api(`/education/classes/${classId}/assignments/${assignmentId}/resources`).get();
    return response.value || [];
  }

  // Create assignment resource
  async createAssignmentResource(classId, assignmentId, resourceData) {
    const client = this.getGraphClient();
    
    const resource = {
      displayName: resourceData.displayName,
      assignmentResourceFolder: {
        '@odata.type': '#microsoft.graph.educationAssignmentResourceFolder'
      },
      resources@odata.bind: `education/classes/${classId}/assignments/${assignmentId}/resources`
    };

    return await client.api(`/education/classes/${classId}/assignments/${assignmentId}/resources`).post(resource);
  }

  // Get class teams
  async getClassTeams(classId) {
    const client = this.getGraphClient();
    const response = await client.api(`/education/classes/${classId}/team`).get();
    return response;
  }

  // Create team for class
  async createTeamForClass(classId, teamData) {
    const client = this.getGraphClient();
    
    const team = {
      template@odata.bind: "https://graph.microsoft.com/v1.0/teamsTemplates('educationClass')",
      displayName: teamData.displayName,
      description: teamData.description || `Team for ${classId}`,
      visibility: teamData.visibility || 'Private',
      members: teamData.members || []
    };

    return await client.api('/teams').post(team);
  }

  // Get class channels
  async getClassChannels(classId) {
    const client = this.getGraphClient();
    const response = await client.api(`/education/classes/${classId}/team/channels`).get();
    return response.value || [];
  }

  // Create channel for class
  async createClassChannel(classId, channelData) {
    const client = this.getGraphClient();
    
    const channel = {
      displayName: channelData.displayName,
      description: channelData.description || '',
      membershipType: channelData.membershipType || 'standard'
    };

    return await client.api(`/education/classes/${classId}/team/channels`).post(channel);
  }

  // Get class calendar events
  async getClassCalendarEvents(classId) {
    const client = this.getGraphClient();
    const response = await client.api(`/education/classes/${classId}/calendar/events`).get();
    return response.value || [];
  }

  // Create calendar event
  async createCalendarEvent(classId, eventData) {
    const client = this.getGraphClient();
    
    const event = {
      subject: eventData.subject,
      body: {
        contentType: 'HTML',
        content: eventData.content || ''
      },
      start: {
        dateTime: eventData.startDateTime,
        timeZone: eventData.timeZone || 'UTC'
      },
      end: {
        dateTime: eventData.endDateTime,
        timeZone: eventData.timeZone || 'UTC'
      },
      attendees: eventData.attendees || [],
      location: {
        displayName: eventData.location || ''
      }
    };

    return await client.api(`/education/classes/${classId}/calendar/events`).post(event);
  }

  // Get class notebooks
  async getClassNotebooks(classId) {
    const client = this.getGraphClient();
    const response = await client.api(`/education/classes/${classId}/notebooks`).get();
    return response.value || [];
  }

  // Create class notebook
  async createClassNotebook(classId, notebookData) {
    const client = this.getGraphClient();
    
    const notebook = {
      displayName: notebookData.displayName,
      isDefault: notebookData.isDefault || false,
      userRole: notebookData.userRole || 'Student'
    };

    return await client.api(`/education/classes/${classId}/notebooks`).post(notebook);
  }

  // Sync education class data to MultiOS format
  syncClassData(msClass) {
    return {
      external_id: msClass.id,
      title: msClass.displayName,
      description: msClass.description,
      code: msClass.externalId,
      instructor: msClass.createdBy?.user?.displayName || 'Unknown',
      start_date: msClass.createdDateTime,
      external_source: msClass.externalSource,
      external_source_id: msClass.externalSourceId,
      external_name: msClass.externalName,
      grade_level: msClass.gradeLevel,
      term: msClass.term
    };
  }

  // Sync assignment data
  syncAssignmentData(msAssignment) {
    return {
      external_id: msAssignment.id,
      title: msAssignment.displayName,
      description: msAssignment.instructions?.content || '',
      due_date: msAssignment.dueDateTime,
      max_points: msAssignment.grading?.['@odata.type'] === '#microsoft.graph.educationAssignmentPointsGradeType' 
        ? msAssignment.grading.maxPoints : 100,
      assignment_category: msAssignment.assignmentCategory,
      status: msAssignment.status,
      allow_students_to_add_resources: msAssignment.allowStudentsToAddResourcesToSubmission,
      created_date_time: msAssignment.createdDateTime,
      modified_date_time: msAssignment.modifiedDateTime
    };
  }

  // Sync member data
  syncMemberData(msMember, role) {
    return {
      external_id: msMember.id,
      user_id: msMember.id,
      role: role,
      user: {
        display_name: msMember.displayName,
        mail: msMember.mail,
        user_principal_name: msMember.userPrincipalName
      }
    };
  }

  // Batch sync class data
  async syncAllClassData(classId) {
    try {
      const [classInfo, assignments, members] = await Promise.all([
        this.getEducationClass(classId),
        this.getClassAssignments(classId),
        this.getClassMembers(classId)
      ]);

      return {
        class: this.syncClassData(classInfo),
        assignments: assignments.map(a => this.syncAssignmentData(a)),
        teachers: members.teachers.map(t => this.syncMemberData(t, 'teacher')),
        students: members.students.map(s => this.syncMemberData(s, 'student'))
      };
    } catch (error) {
      throw new Error(`Failed to sync Microsoft Teams class data: ${error.message}`);
    }
  }

  // Webhook validation
  validateWebhook(notificationUrl, validationToken) {
    // Microsoft Teams webhook validation
    return {
      response: validationToken,
      status: 'ok'
    };
  }

  // Handle webhook events
  async handleWebhook(subscriptionId, notificationData) {
    // Handle Microsoft Teams education events
    console.log('Received Teams webhook:', subscriptionId, notificationData);
    return { processed: true };
  }

  // Disconnect
  async disconnect() {
    this.accessToken = null;
    this.refreshToken = null;
    return true;
  }
}

module.exports = MicrosoftTeamsService;