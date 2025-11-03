const axios = require('axios');

class MoodleLMSService {
  constructor(config) {
    this.baseUrl = config.baseUrl || process.env.MOODLE_API_URL;
    this.wstoken = config.wstoken || process.env.MOODLE_WSTOKEN;
    this.format = 'json';
  }

  // Make Moodle Web Service API request
  async makeRequest(wsfunction, params = {}) {
    const requestParams = {
      wstoken: this.wstoken,
      wsfunction: wsfunction,
      moodlewsrestformat: this.format,
      ...params
    };

    try {
      const response = await axios.get(this.baseUrl, { params: requestParams });
      
      if (response.data.exception) {
        throw new Error(`Moodle API error: ${response.data.message}`);
      }
      
      return response.data;
    } catch (error) {
      throw new Error(`Moodle Web Service error: ${error.response?.data?.message || error.message}`);
    }
  }

  // Get site info
  async getSiteInfo() {
    return await this.makeRequest('core_webservice_get_site_info');
  }

  // Get courses
  async getCourses(options = {}) {
    const params = {};
    
    if (options.ids) {
      params.courseids = options.ids;
    }
    
    return await this.makeRequest('core_course_get_courses', params);
  }

  // Get course by field
  async getCourseByField(field, value) {
    return await this.makeRequest('core_course_get_courses_by_field', {
      field: field,
      value: value
    });
  }

  // Get course contents
  async getCourseContents(courseId) {
    return await this.makeRequest('core_course_get_contents', {
      courseid: courseId
    });
  }

  // Get course modules
  async getCourseModules(courseId) {
    return await this.makeRequest('core_course_get_course_module', {
      courseid: courseId
    });
  }

  // Create course
  async createCourse(courseData) {
    const params = {
      courses: [{
        fullname: courseData.fullname,
        shortname: courseData.shortname,
        categoryid: courseData.categoryid,
        summary: courseData.summary || '',
        startdate: courseData.startdate,
        enddate: courseData.enddate,
        visible: courseData.visible || 1,
        format: courseData.format || 'topics',
        numsections: courseData.numsections || 1
      }]
    };

    const response = await this.makeRequest('core_course_create_courses', params);
    return response[0]; // Returns array with one course
  }

  // Update course
  async updateCourse(courseId, updateData) {
    const params = {
      courses: [{
        id: courseId,
        ...updateData
      }]
    };

    return await this.makeRequest('core_course_update_courses', params);
  }

  // Delete course
  async deleteCourse(courseId) {
    const params = {
      courseids: [courseId]
    };

    return await this.makeRequest('core_course_delete_courses', params);
  }

  // Get enrolled users in course
  async getEnrolledUsers(courseId) {
    return await this.makeRequest('core_enrol_get_enrolled_users', {
      courseid: courseId
    });
  }

  // Enroll user in course
  async enrollUser(userId, courseId, roleId = 5) { // 5 = student role by default
    const params = {
      enrolments: [{
        roleid: roleId,
        userid: userId,
        courseid: courseId
      }]
    };

    return await this.makeRequest('enrol_manual_enrol_users', params);
  }

  // Unenroll user from course
  async unenrollUser(userId, courseId) {
    const params = {
      unenrolments: [{
        userid: userId,
        courseid: courseId
      }]
    };

    return await this.makeRequest('enrol_manual_unenrol_users', params);
  }

  // Create assignment
  async createAssignment(courseId, assignmentData) {
    const params = {
      assignments: [{
        course: courseId,
        name: assignmentData.name,
        intro: assignmentData.description || '',
        introformat: 1, // HTML format
        alwaysshowdescription: assignmentData.alwaysshowdescription || 0,
        submissiondrafts: assignmentData.submissiondrafts || 0,
        sendnotifications: assignmentData.sendnotifications || 0,
        sendlatenotifications: assignmentData.sendlatenotifications || 0,
        sendstudentnotifications: assignmentData.sendstudentnotifications || 1,
        duedate: assignmentData.duedate || 0,
        allowsubmissionsfromdate: assignmentData.allowsubmissionsfromdate || 0,
        grade: assignmentData.grade || 100,
        timemodified: Math.floor(Date.now() / 1000),
        completionsubmit: assignmentData.completionsubmit || 0
      }]
    };

    const response = await this.makeRequest('mod_assign_create_assignments', params);
    return response[0]; // Returns array with one assignment
  }

  // Update assignment
  async updateAssignment(assignmentId, updateData) {
    const params = {
      assignments: [{
        id: assignmentId,
        ...updateData
      }]
    };

    return await this.makeRequest('mod_assign_update_assignments', params);
  }

  // Get assignments
  async getAssignments(courseIds) {
    const params = {
      courseids: courseIds
    };

    return await this.makeRequest('mod_assign_get_assignments', params);
  }

  // Submit assignment
  async submitAssignment(assignmentId, userId, text = '', files = []) {
    const params = {
      assignmentids: [assignmentId],
      plugindata: {
        onlinetext_editor: {
          text: text,
          format: 1
        },
        file_filemanager: files
      }
    };

    return await this.makeRequest('mod_assign_save_submission', params);
  }

  // Grade assignment submission
  async gradeSubmission(assignmentId, userId, grade, feedback = '') {
    const params = {
      assignments: [{
        assignmentid: assignmentId,
        userid: userId,
        grade: grade,
        attemptnumber: -1,
        addattempt: 0,
        workflowstate: '',
        applytoall: 0,
        plugindata: {
          onlinetext_editor: {
            text: feedback,
            format: 1
          }
        }
      }]
    };

    return await this.makeRequest('mod_assign_save_grades', params);
  }

  // Get assignment submissions
  async getSubmissions(assignmentId) {
    const params = {
      assignmentids: [assignmentId]
    };

    return await this.makeRequest('mod_assign_get_submissions', params);
  }

  // Get quiz
  async getQuiz(quizId) {
    return await this.makeRequest('mod_quiz_get_quizzes_by_courses', {
      courseids: [quizId]
    });
  }

  // Create quiz
  async createQuiz(courseId, quizData) {
    const params = {
      quizzes: [{
        course: courseId,
        name: quizData.name,
        intro: quizData.description || '',
        introformat: 1,
        timeopen: quizData.timeopen || 0,
        timeclose: quizData.timeclose || 0,
        timelimit: quizData.timelimit || 0,
        preferencess: {
          shuffleanswers: quizData.shuffleanswers || 1,
          showcorrectresponses: quizData.showcorrectresponses || 1,
          showresults: quizData.showresults || 'afteropen',
          showfeedback: quizData.showfeedback || 1,
          attempts: quizData.attempts || 'unlimited',
          grademethod: quizData.grademethod || 1
        }
      }]
    };

    const response = await this.makeRequest('mod_quiz_create_quizzes', params);
    return response[0];
  }

  // Create course module (resource)
  async createModule(courseId, moduleData) {
    const params = {
      modules: [{
        course: courseId,
        modulename: moduleData.modulename || 'resource',
        section: moduleData.section || 1,
        name: moduleData.name,
        intro: moduleData.intro || '',
        introformat: 1,
        visible: moduleData.visible || 1,
        completion: moduleData.completion || 0
      }]
    };

    const response = await this.makeRequest('core_course_create_modules', params);
    return response[0];
  }

  // Update course module
  async updateModule(moduleId, updateData) {
    const params = {
      modules: [{
        id: moduleId,
        ...updateData
      }]
    };

    return await this.makeRequest('core_course_update_modules', params);
  }

  // Delete course module
  async deleteModule(moduleId) {
    const params = {
      ids: [moduleId]
    };

    return await this.makeRequest('core_course_delete_modules', params);
  }

  // Get course categories
  async getCategories() {
    return await this.makeRequest('core_course_get_categories');
  }

  // Create category
  async createCategory(categoryData) {
    const params = {
      categories: [{
        name: categoryData.name,
        parent: categoryData.parent || 0,
        description: categoryData.description || '',
        descriptionformat: 1
      }]
    };

    const response = await this.makeRequest('core_course_create_categories', params);
    return response[0];
  }

  // Get users
  async getUsers(criteria = []) {
    const params = {
      criteria: criteria.map(c => ({
        key: c.key,
        value: c.value
      }))
    };

    return await this.makeRequest('core_user_get_users', params);
  }

  // Create user
  async createUser(userData) {
    const params = {
      users: [{
        username: userData.username,
        firstname: userData.firstname,
        lastname: userData.lastname,
        email: userData.email,
        password: userData.password,
        lang: userData.lang || 'en',
        auth: userData.auth || 'manual',
        idnumber: userData.idnumber || '',
        department: userData.department || '',
        interests: userData.interests || '',
        institution: userData.institution || '',
        location: userData.location || ''
      }]
    };

    const response = await this.makeRequest('core_user_create_users', params);
    return response[0];
  }

  // Update user
  async updateUser(userId, updateData) {
    const params = {
      users: [{
        id: userId,
        ...updateData
      }]
    };

    return await this.makeRequest('core_user_update_users', params);
  }

  // Get discussions
  async getDiscussions(courseId) {
    return await this.makeRequest('mod_forum_get_forum_discussions', {
      forumid: courseId
    });
  }

  // Create discussion
  async createDiscussion(forumId, discussionData) {
    const params = {
      forumid: forumId,
      subject: discussionData.subject,
      message: discussionData.message,
      options: discussionData.options || []
    };

    return await this.makeRequest('mod_forum_add_discussion', params);
  }

  // Get calendar events
  async getCalendarEvents(courseId = null) {
    const params = {};
    
    if (courseId) {
      params.courseids = [courseId];
    }

    return await this.makeRequest('core_calendar_get_calendar_events', params);
  }

  // Create calendar event
  async createCalendarEvent(eventData) {
    const params = {
      events: [{
        name: eventData.name,
        description: eventData.description,
        format: 1,
        courseid: eventData.courseid,
        groupid: eventData.groupid || 0,
        userid: eventData.userid || 0,
        eventtype: eventData.eventtype || 'course',
        timestart: eventData.timestart,
        timeduration: eventData.timeduration || 0,
        visible: eventData.visible || 1,
        sequence: 1
      }]
    };

    return await this.makeRequest('core_calendar_create_calendar_events', params);
  }

  // Sync course data to MultiOS format
  syncCourseData(moodleCourse) {
    return {
      external_id: moodleCourse.id.toString(),
      title: moodleCourse.fullname,
      description: moodleCourse.summary,
      code: moodleCourse.shortname,
      start_date: moodleCourse.startdate ? new Date(moodleCourse.startdate * 1000).toISOString() : null,
      end_date: moodleCourse.enddate ? new Date(moodleCourse.enddate * 1000).toISOString() : null,
      students_count: moodleCourse.enrolledusercount || 0,
      visible: moodleCourse.visible === 1,
      format: moodleCourse.format,
      numsections: moodleCourse.numsections,
      category_id: moodleCourse.categoryid
    };
  }

  // Sync assignment data
  syncAssignmentData(moodleAssignment) {
    return {
      external_id: moodleAssignment.id.toString(),
      title: moodleAssignment.name,
      description: moodleAssignment.intro,
      due_date: moodleAssignment.duedate ? new Date(moodleAssignment.duedate * 1000).toISOString() : null,
      points_possible: moodleAssignment.grade,
      submission_types: ['online_text', 'online_upload'],
      grading_type: 'points',
      published: moodleAssignment.visible === 1,
      course_module_id: moodleAssignment.coursemodule
    };
  }

  // Sync module data
  syncModuleData(moodleModule) {
    return {
      external_id: moodleModule.id.toString(),
      title: moodleModule.name,
      description: moodleModule.description,
      module_type: moodleModule.modname,
      url: moodleModule.url,
      visible: moodleModule.visible === 1,
      section: moodleModule.section,
      position: moodleModule.indexposition,
      completion: {
        completion: moodleModule.completion,
        completiondata: moodleModule.completiondata
      }
    };
  }

  // Sync enrollment data
  syncEnrollmentData(moodleUser) {
    return {
      external_id: moodleUser.id.toString(),
      user_id: moodleUser.id.toString(),
      username: moodleUser.username,
      full_name: moodleUser.fullname,
      email: moodleUser.email,
      first_access: moodleUser.firstaccess,
      last_access: moodleUser.lastaccess,
      auth: moodleUser.auth
    };
  }

  // Batch sync course data
  async syncAllCourseData(courseId) {
    try {
      const [course, contents, assignments] = await Promise.all([
        this.getCourseByField('id', courseId),
        this.getCourseContents(courseId),
        this.getAssignments([courseId])
      ]);

      return {
        course: course.courses.length > 0 ? this.syncCourseData(course.courses[0]) : null,
        learning_modules: contents.map(c => this.syncModuleData(c)),
        assignments: assignments.courses[0]?.assignments?.map(a => this.syncAssignmentData(a)) || []
      };
    } catch (error) {
      throw new Error(`Failed to sync Moodle course data: ${error.message}`);
    }
  }

  // Disconnect
  async disconnect() {
    // Moodle doesn't require disconnection as it uses stateless API calls
    return true;
  }
}

module.exports = MoodleLMSService;