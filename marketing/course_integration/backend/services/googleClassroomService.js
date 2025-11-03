const { google } = require('googleapis');

class GoogleClassroomService {
  constructor(config) {
    this.clientId = config.clientId || process.env.GOOGLE_CLIENT_ID;
    this.clientSecret = config.clientSecret || process.env.GOOGLE_CLIENT_SECRET;
    this.redirectUri = config.redirectUri || process.env.GOOGLE_REDIRECT_URI;
    this.calendarId = config.calendarId;
    
    this.oauth2Client = new google.auth.OAuth2(
      this.clientId,
      this.clientSecret,
      this.redirectUri
    );

    this.scopes = [
      'https://www.googleapis.com/auth/classroom.courses',
      'https://www.googleapis.com/auth/classroom.coursework.me',
      'https://www.googleapis.com/auth/classroom.coursework.students',
      'https://www.googleapis.com/auth/classroom.rosters',
      'https://www.googleapis.com/auth/classroom.announcements',
      'https://www.googleapis.com/auth/classroom.guardianlinks.students',
      'https://www.googleapis.com/auth/classroom.profile.emails',
      'https://www.googleapis.com/auth/classroom.profile.photos'
    ];
  }

  // Get authorization URL
  getAuthorizationUrl(state) {
    return this.oauth2Client.generateAuthUrl({
      access_type: 'offline',
      scope: this.scopes,
      state: state,
      prompt: 'consent'
    });
  }

  // Exchange authorization code for tokens
  async getTokens(code) {
    const { tokens } = await this.oauth2Client.getToken(code);
    this.oauth2Client.setCredentials(tokens);
    return tokens;
  }

  // Set access token
  setAccessToken(tokens) {
    this.oauth2Client.setCredentials(tokens);
  }

  // Get Google Classroom API instance
  getClassroomClient() {
    return google.classroom({ version: 'v1', auth: this.oauth2Client });
  }

  // Get Google Calendar API instance
  getCalendarClient() {
    return google.calendar({ version: 'v3', auth: this.oauth2Client });
  }

  // Get current user info
  async getCurrentUser() {
    const classroom = this.getClassroomClient();
    const response = await classroom.userProfiles.get({
      userId: 'me'
    });
    return response.data;
  }

  // Get courses
  async getCourses(options = {}) {
    const classroom = this.getClassroomClient();
    const params = {
      pageSize: options.pageSize || 100,
      courseStates: options.courseStates || ['ACTIVE']
    };

    if (options.teacherId) {
      params.teacherId = options.teacherId;
    }

    if (options.studentId) {
      params.studentId = options.studentId;
    }

    const response = await classroom.courses.list(params);
    return response.data.courses || [];
  }

  // Get course details
  async getCourse(courseId) {
    const classroom = this.getClassroomClient();
    const response = await classroom.courses.get({
      id: courseId
    });
    return response.data;
  }

  // Create course
  async createCourse(courseData) {
    const classroom = this.getClassroomClient();
    
    const course = {
      name: courseData.name,
      section: courseData.section || '',
      descriptionHeading: courseData.descriptionHeading || '',
      description: courseData.description || '',
      room: courseData.room || '',
      ownerId: courseData.ownerId || 'me',
      courseState: courseData.courseState || 'PROVISIONED'
    };

    const response = await classroom.courses.create({
      requestBody: course
    });
    return response.data;
  }

  // Update course
  async updateCourse(courseId, updateData) {
    const classroom = this.getClassroomClient();
    
    const updateMask = Object.keys(updateData).join(',');
    const response = await classroom.courses.patch({
      id: courseId,
      updateMask: updateMask,
      requestBody: updateData
    });
    return response.data;
  }

  // Delete course
  async deleteCourse(courseId) {
    const classroom = this.getClassroomClient();
    const response = await classroom.courses.delete({
      id: courseId
    });
    return response.status === 204;
  }

  // Get course students
  async getCourseStudents(courseId, options = {}) {
    const classroom = this.getClassroomClient();
    const params = {
      courseId: courseId,
      pageSize: options.pageSize || 100
    };

    const response = await classroom.courses.students.list(params);
    return response.data.students || [];
  }

  // Get course teachers
  async getCourseTeachers(courseId, options = {}) {
    const classroom = this.getClassroomClient();
    const params = {
      courseId: courseId,
      pageSize: options.pageSize || 100
    };

    const response = await classroom.courses.teachers.list(params);
    return response.data.teachers || [];
  }

  // Add student to course
  async addStudent(courseId, studentId) {
    const classroom = this.getClassroomClient();
    
    const student = {
      userId: studentId
    };

    const response = await classroom.courses.students.create({
      courseId: courseId,
      requestBody: student
    });
    return response.data;
  }

  // Add teacher to course
  async addTeacher(courseId, teacherId) {
    const classroom = this.getClassroomClient();
    
    const teacher = {
      userId: teacherId
    };

    const response = await classroom.courses.teachers.create({
      courseId: courseId,
      requestBody: teacher
    });
    return response.data;
  }

  // Get course announcements
  async getAnnouncements(courseId, options = {}) {
    const classroom = this.getClassroomClient();
    const params = {
      courseId: courseId,
      pageSize: options.pageSize || 100
    };

    if (options.announcementStates) {
      params.announcementStates = options.announcementStates;
    }

    const response = await classroom.courses.announcements.list(params);
    return response.data.announcements || [];
  }

  // Create announcement
  async createAnnouncement(courseId, announcementData) {
    const classroom = this.getClassroomClient();
    
    const announcement = {
      text: announcementData.text,
      materials: announcementData.materials || [],
      state: announcementData.state || 'PUBLISHED',
      scheduledDate: announcementData.scheduledDate
    };

    if (announcementData.dueDate) {
      announcement.dueDate = announcementData.dueDate;
    }

    if (announcementData.dueTime) {
      announcement.dueTime = announcementData.dueTime;
    }

    const response = await classroom.courses.announcements.create({
      courseId: courseId,
      requestBody: announcement
    });
    return response.data;
  }

  // Get course assignments
  async getCourseWork(courseId, options = {}) {
    const classroom = this.getClassroomClient();
    const params = {
      courseId: courseId,
      pageSize: options.pageSize || 100
    };

    if (options.courseWorkStates) {
      params.courseWorkStates = options.courseWorkStates;
    }

    const response = await classroom.courses.courseWork.list(params);
    return response.data.courseWork || [];
  }

  // Create assignment
  async createAssignment(courseId, assignmentData) {
    const classroom = this.getClassroomClient();
    
    const courseWork = {
      title: assignmentData.title,
      description: assignmentData.description || '',
      materials: assignmentData.materials || [],
      state: assignmentData.state || 'PUBLISHED',
      maxPoints: assignmentData.maxPoints || 100,
      workType: assignmentData.workType || 'ASSIGNMENT',
      submissionModificationMode: assignmentData.submissionModificationMode || 'SUBMISSION_MODIFICATION_MODE_UNSPECIFIED'
    };

    // Add due date if specified
    if (assignmentData.dueDate) {
      courseWork.dueDate = assignmentData.dueDate;
    }

    if (assignmentData.dueTime) {
      courseWork.dueTime = assignmentData.dueTime;
    }

    // Add scheduling if specified
    if (assignmentData.scheduleTime) {
      courseWork.scheduleTime = assignmentData.scheduleTime;
    }

    const response = await classroom.courses.courseWork.create({
      courseId: courseId,
      requestBody: courseWork
    });
    return response.data;
  }

  // Update assignment
  async updateAssignment(courseId, assignmentId, updateData) {
    const classroom = this.getClassroomClient();
    
    const updateMask = Object.keys(updateData).join(',');
    const response = await classroom.courses.courseWork.patch({
      courseId: courseId,
      id: assignmentId,
      updateMask: updateMask,
      requestBody: updateData
    });
    return response.data;
  }

  // Delete assignment
  async deleteAssignment(courseId, assignmentId) {
    const classroom = this.getClassroomClient();
    const response = await classroom.courses.courseWork.delete({
      courseId: courseId,
      id: assignmentId
    });
    return response.status === 204;
  }

  // Get assignment submissions
  async getSubmissions(courseId, assignmentId) {
    const classroom = this.getClassroomClient();
    const response = await classroom.courses.courseWork.studentSubmissions.list({
      courseId: courseId,
      courseWorkId: assignmentId
    });
    return response.data.studentSubmissions || [];
  }

  // Get student submission
  async getStudentSubmission(courseId, assignmentId, submissionId) {
    const classroom = this.getClassroomClient();
    const response = await classroom.courses.courseWork.studentSubmissions.get({
      courseId: courseId,
      courseWorkId: assignmentId,
      id: submissionId
    });
    return response.data;
  }

  // Grade submission
  async gradeSubmission(courseId, assignmentId, submissionId, gradeData) {
    const classroom = this.getClassroomClient();
    
    const updateMask = [];
    const updateBody = {};

    if (gradeData.assignedGrade !== undefined) {
      updateMask.push('assignedGrade');
      updateBody.assignedGrade = gradeData.assignedGrade;
    }

    if (gradeData.draftGrade !== undefined) {
      updateMask.push('draftGrade');
      updateBody.draftGrade = gradeData.draftGrade;
    }

    if (gradeData.driveFolder) {
      updateMask.push('driveFolder');
      updateBody.driveFolder = gradeData.driveFolder;
    }

    if (gradeData.teacherFolder) {
      updateMask.push('teacherFolder');
      updateBody.teacherFolder = gradeData.teacherFolder;
    }

    // Return submission after grading
    if (gradeData.returnSubmission) {
      updateMask.push('assignedGrade');
      await classroom.courses.courseWork.studentSubmissions.return({
        courseId: courseId,
        courseWorkId: assignmentId,
        id: submissionId
      });
    }

    if (updateMask.length === 0) {
      throw new Error('No grade data provided');
    }

    const response = await classroom.courses.courseWork.studentSubmissions.patch({
      courseId: courseId,
      courseWorkId: assignmentId,
      id: submissionId,
      updateMask: updateMask.join(','),
      requestBody: updateBody
    });
    return response.data;
  }

  // Create course topic
  async createTopic(courseId, topicData) {
    const classroom = this.getClassroomClient();
    
    const topic = {
      name: topicData.name
    };

    const response = await classroom.courses.topics.create({
      courseId: courseId,
      requestBody: topic
    });
    return response.data;
  }

  // Get course topics
  async getTopics(courseId) {
    const classroom = this.getClassroomClient();
    const response = await classroom.courses.topics.list({
      courseId: courseId
    });
    return response.data.topics || [];
  }

  // Get course materials
  async getCourseMaterials(courseId) {
    const classroom = this.getClassroomClient();
    const response = await classroom.courses.courseWorkMaterials.list({
      courseId: courseId
    });
    return response.data.courseWorkMaterial || [];
  }

  // Create course material
  async createCourseMaterial(courseId, materialData) {
    const classroom = this.getClassroomClient();
    
    const material = {
      title: materialData.title,
      description: materialData.description || '',
      materials: materialData.materials || [],
      state: materialData.state || 'PUBLISHED',
      topicId: materialData.topicId
    };

    if (materialData.scheduleTime) {
      material.scheduleTime = materialData.scheduleTime;
    }

    const response = await classroom.courses.courseWorkMaterials.create({
      courseId: courseId,
      requestBody: material
    });
    return response.data;
  }

  // Create calendar event for course
  async createCalendarEvent(eventData) {
    const calendar = this.getCalendarClient();
    
    const event = {
      summary: eventData.summary,
      description: eventData.description || '',
      start: eventData.start,
      end: eventData.end,
      attendees: eventData.attendees || [],
      conferenceData: eventData.conferenceData,
      location: eventData.location
    };

    const response = await calendar.events.insert({
      calendarId: this.calendarId || 'primary',
      requestBody: event,
      conferenceDataVersion: event.conferenceData ? 1 : 0
    });
    return response.data;
  }

  // Get calendar events
  async getCalendarEvents(options = {}) {
    const calendar = this.getCalendarClient();
    const params = {
      calendarId: this.calendarId || 'primary',
      timeMin: options.timeMin,
      timeMax: options.timeMax,
      maxResults: options.maxResults || 100,
      singleEvents: options.singleEvents || true,
      orderBy: options.orderBy || 'startTime'
    };

    const response = await calendar.events.list(params);
    return response.data.items || [];
  }

  // Sync course data to MultiOS format
  syncCourseData(classroomCourse) {
    return {
      external_id: classroomCourse.id,
      title: classroomCourse.name,
      description: classroomCourse.description,
      code: classroomCourse.section || classroomCourse.courseId,
      instructor: classroomCourse.ownerId,
      start_date: classroomCourse.enrollmentTime,
      end_date: classroomCourse.endTime,
      students_count: classroomCourse.courseState === 'ACTIVE' ? 0 : 0, // Needs additional API call
      syllabus: classroomCourse.descriptionHeading,
      alternateLink: classroomCourse.alternateLink,
      courseState: classroomCourse.courseState,
      room: classroomCourse.room
    };
  }

  // Sync assignment data
  syncAssignmentData(classroomAssignment) {
    return {
      external_id: classroomAssignment.id,
      title: classroomAssignment.title,
      description: classroomAssignment.description,
      due_date: classroomAssignment.dueDate ? `${classroomAssignment.dueDate.year}-${classroomAssignment.dueDate.month.toString().padStart(2, '0')}-${classroomAssignment.dueDate.day.toString().padStart(2, '0')}` : null,
      max_points: classroomAssignment.maxPoints,
      submission_types: this.getSubmissionTypes(classroomAssignment.workType),
      work_type: classroomAssignment.workType,
      state: classroomAssignment.state,
      alternate_link: classroomAssignment.alternateLink,
      course_work_material_id: classroomAssignment.courseWorkMaterialId
    };
  }

  // Get submission types based on work type
  getSubmissionTypes(workType) {
    switch (workType) {
      case 'ASSIGNMENT':
        return ['TURNED_IN'];
      case 'SHORT_ANSWER_QUESTION':
        return ['SHORT_ANSWER'];
      case 'MULTIPLE_CHOICE_QUESTION':
        return ['MULTIPLE_CHOICE'];
      default:
        return [];
    }
  }

  // Sync student enrollment data
  syncEnrollmentData(classroomStudent, courseId) {
    return {
      external_id: classroomStudent.userId,
      user_id: classroomStudent.userId,
      course_id: courseId,
      profile: {
        id: classroomStudent.profile.id,
        name: classroomStudent.profile.name,
        email_address: classroomStudent.profile.emailAddress,
        photo_url: classroomStudent.profile.photoUrl
      },
      course_id: classroomStudent.courseId,
      user_id: classroomStudent.userId
    };
  }

  // Batch sync course data
  async syncAllCourseData(courseId) {
    try {
      const [course, assignments, students, materials] = await Promise.all([
        this.getCourse(courseId),
        this.getCourseWork(courseId),
        this.getCourseStudents(courseId),
        this.getCourseMaterials(courseId)
      ]);

      return {
        course: this.syncCourseData(course),
        assignments: assignments.map(a => this.syncAssignmentData(a)),
        students: students.map(s => this.syncEnrollmentData(s, courseId)),
        materials: materials
      };
    } catch (error) {
      throw new Error(`Failed to sync Google Classroom course data: ${error.message}`);
    }
  }

  // Disconnect
  async disconnect() {
    this.oauth2Client.revokeCredentials();
    return true;
  }
}

module.exports = GoogleClassroomService;