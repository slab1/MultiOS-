const CanvasLMSService = require('./canvasLMS');
const BlackboardLMSService = require('./blackboardLMS');
const MoodleLMSService = require('./moodleLMS');
const GoogleClassroomService = require('./googleClassroomService');
const MicrosoftTeamsService = require('./microsoftTeamsService');
const LTIProviderService = require('./ltiProviderService');

class LMSManager {
  constructor(db, redis) {
    this.db = db;
    this.redis = redis;
    this.instances = new Map();
  }

  // Initialize LMS instance
  async initializeLMS(integrationConfig) {
    const { id, type, settings } = integrationConfig;
    
    try {
      let service;
      
      switch (type) {
        case 'canvas':
          service = new CanvasLMSService(settings);
          break;
        
        case 'blackboard':
          service = new BlackboardLMSService(settings);
          break;
        
        case 'moodle':
          service = new MoodleLMSService(settings);
          break;
        
        case 'google_classroom':
          service = new GoogleClassroomService(settings);
          break;
        
        case 'microsoft_teams':
          service = new MicrosoftTeamsService(settings);
          break;
        
        case 'lti_custom':
          service = new LTIProviderService(settings);
          break;
        
        default:
          throw new Error(`Unsupported LMS type: ${type}`);
      }

      this.instances.set(id, service);
      
      // Test connection
      if (type !== 'lti_custom') {
        await this.testConnection(id);
      }

      return { success: true, service };
    } catch (error) {
      console.error(`Failed to initialize ${type} LMS:`, error.message);
      return { success: false, error: error.message };
    }
  }

  // Test LMS connection
  async testConnection(integrationId) {
    const service = this.instances.get(integrationId);
    if (!service) {
      throw new Error(`LMS instance not found: ${integrationId}`);
    }

    try {
      // Most LMS services have a getCurrentUser or getSiteInfo method
      if (typeof service.getCurrentUser === 'function') {
        await service.getCurrentUser();
      } else if (typeof service.getSiteInfo === 'function') {
        await service.getSiteInfo();
      } else if (typeof service.getUserProfile === 'function') {
        await service.getUserProfile();
      }
      
      return { success: true, message: 'Connection successful' };
    } catch (error) {
      throw new Error(`Connection test failed: ${error.message}`);
    }
  }

  // Sync courses from LMS to MultiOS
  async syncCourses(integrationId, options = {}) {
    const service = this.instances.get(integrationId);
    if (!service) {
      throw new Error(`LMS instance not found: ${integrationId}`);
    }

    try {
      const cacheKey = `sync:courses:${integrationId}`;
      
      // Check if sync is already in progress
      if (await this.redis.get(cacheKey)) {
        return { success: false, message: 'Sync already in progress' };
      }

      // Lock the sync process
      await this.redis.setex(cacheKey, 3600, 'in_progress');

      const courses = await service.getCourses({
        includeEnrollments: options.includeEnrollments !== false
      });

      const syncResults = [];
      
      for (const course of courses) {
        try {
          const courseData = service.syncCourseData ? service.syncCourseData(course) : course;
          
          // Upsert course in database
          const courseRecord = await this.db.upsert('lms_courses', {
            where: {
              lms_integration_id: integrationId,
              external_course_id: courseData.external_id
            },
            defaults: {
              lms_integration_id: integrationId,
              external_course_id: courseData.external_id,
              course_name: courseData.title || courseData.name,
              course_code: courseData.code || courseData.course_code,
              description: courseData.description,
              instructor_name: courseData.instructor,
              students_count: courseData.students_count || 0,
              last_sync: new Date(),
              sync_status: 'completed'
            },
            update: {
              course_name: courseData.title || courseData.name,
              course_code: courseData.code || courseData.course_code,
              description: courseData.description,
              instructor_name: courseData.instructor,
              students_count: courseData.students_count || 0,
              last_sync: new Date(),
              sync_status: 'completed'
            }
          });

          syncResults.push({ success: true, courseId: courseRecord.id, data: courseData });
        } catch (error) {
          console.error(`Failed to sync course ${course.id}:`, error.message);
          syncResults.push({ 
            success: false, 
            courseId: course.id, 
            error: error.message 
          });
        }
      }

      // Release sync lock
      await this.redis.del(cacheKey);

      const successCount = syncResults.filter(r => r.success).length;
      const errorCount = syncResults.filter(r => !r.success).length;

      return {
        success: true,
        summary: {
          total: syncResults.length,
          successful: successCount,
          failed: errorCount
        },
        details: syncResults
      };

    } catch (error) {
      // Release sync lock on error
      await this.redis.del(`sync:courses:${integrationId}`);
      throw error;
    }
  }

  // Sync assignments from LMS
  async syncAssignments(integrationId, courseId, options = {}) {
    const service = this.instances.get(integrationId);
    if (!service) {
      throw new Error(`LMS instance not found: ${integrationId}`);
    }

    try {
      const assignments = await service.getCourseAssignments(courseId);
      const syncResults = [];

      for (const assignment of assignments) {
        try {
          const assignmentData = service.syncAssignmentData 
            ? service.syncAssignmentData(assignment) 
            : assignment;

          const assignmentRecord = await this.db.upsert('lms_assignments', {
            where: {
              lms_integration_id: integrationId,
              external_assignment_id: assignmentData.external_id
            },
            defaults: {
              lms_integration_id: integrationId,
              external_assignment_id: assignmentData.external_id,
              course_id: courseId,
              title: assignmentData.title || assignmentData.name,
              description: assignmentData.description,
              due_date: assignmentData.due_date,
              points_possible: assignmentData.points_possible,
              sync_status: 'completed'
            },
            update: {
              title: assignmentData.title || assignmentData.name,
              description: assignmentData.description,
              due_date: assignmentData.due_date,
              points_possible: assignmentData.points_possible,
              sync_status: 'completed'
            }
          });

          syncResults.push({ success: true, assignmentId: assignmentRecord.id });
        } catch (error) {
          syncResults.push({ 
            success: false, 
            assignmentId: assignment.id, 
            error: error.message 
          });
        }
      }

      return {
        success: true,
        summary: {
          total: syncResults.length,
          successful: syncResults.filter(r => r.success).length,
          failed: syncResults.filter(r => !r.success).length
        }
      };
    } catch (error) {
      throw new Error(`Failed to sync assignments: ${error.message}`);
    }
  }

  // Sync student enrollments
  async syncEnrollments(integrationId, courseId, options = {}) {
    const service = this.instances.get(integrationId);
    if (!service) {
      throw new Error(`LMS instance not found: ${integrationId}`);
    }

    try {
      const students = await service.getCourseStudents(courseId);
      const syncResults = [];

      for (const student of students) {
        try {
          const enrollmentData = service.syncEnrollmentData 
            ? service.syncEnrollmentData(student) 
            : student;

          const enrollmentRecord = await this.db.upsert('lms_enrollments', {
            where: {
              lms_integration_id: integrationId,
              external_course_id: courseId,
              external_user_id: enrollmentData.user_id || enrollmentData.external_id
            },
            defaults: {
              lms_integration_id: integrationId,
              external_course_id: courseId,
              external_user_id: enrollmentData.user_id || enrollmentData.external_id,
              user_data: enrollmentData.user || enrollmentData,
              enrollment_status: 'active'
            },
            update: {
              user_data: enrollmentData.user || enrollmentData,
              enrollment_status: 'active'
            }
          });

          syncResults.push({ success: true, enrollmentId: enrollmentRecord.id });
        } catch (error) {
          syncResults.push({ 
            success: false, 
            studentId: student.user_id || student.id, 
            error: error.message 
          });
        }
      }

      return {
        success: true,
        summary: {
          total: syncResults.length,
          successful: syncResults.filter(r => r.success).length,
          failed: syncResults.filter(r => !r.success).length
        }
      };
    } catch (error) {
      throw new Error(`Failed to sync enrollments: ${error.message}`);
    }
  }

  // Full course sync
  async syncFullCourse(integrationId, courseId, options = {}) {
    const service = this.instances.get(integrationId);
    if (!service) {
      throw new Error(`LMS instance not found: ${integrationId}`);
    }

    const syncId = `full:${integrationId}:${courseId}:${Date.now()}`;
    
    try {
      // Start sync logging
      await this.db.insert('lms_sync_logs', {
        id: syncId,
        lms_integration_id: integrationId,
        operation: 'full_course_sync',
        status: 'in_progress',
        message: 'Starting full course synchronization',
        created_at: new Date()
      });

      const results = {
        course: null,
        assignments: null,
        enrollments: null,
        modules: null
      };

      // Sync course data
      try {
        const courseData = await service.getCourse(courseId);
        results.course = service.syncCourseData ? service.syncCourseData(courseData) : courseData;
        
        await this.db.upsert('lms_courses', {
          where: {
            lms_integration_id: integrationId,
            external_course_id: results.course.external_id
          },
          defaults: {
            lms_integration_id: integrationId,
            external_course_id: results.course.external_id,
            course_name: results.course.title,
            course_code: results.course.code,
            description: results.course.description,
            instructor_name: results.course.instructor,
            students_count: results.course.students_count,
            last_sync: new Date(),
            sync_status: 'completed'
          }
        });
      } catch (error) {
        throw new Error(`Course sync failed: ${error.message}`);
      }

      // Sync assignments
      try {
        results.assignments = await this.syncAssignments(integrationId, courseId, options);
      } catch (error) {
        console.warn(`Assignments sync failed: ${error.message}`);
        results.assignments = { success: false, error: error.message };
      }

      // Sync enrollments
      try {
        results.enrollments = await this.syncEnrollments(integrationId, courseId, options);
      } catch (error) {
        console.warn(`Enrollments sync failed: ${error.message}`);
        results.enrollments = { success: false, error: error.message };
      }

      // Sync modules (if service supports it)
      try {
        if (service.getCourseModules || service.getCourseContents) {
          const modules = service.getCourseModules 
            ? await service.getCourseModules(courseId)
            : await service.getCourseContents(courseId);
          
          results.modules = modules.map(m => 
            service.syncLearningModuleData ? service.syncLearningModuleData(m) : m
          );
        }
      } catch (error) {
        console.warn(`Modules sync failed: ${error.message}`);
        results.modules = { success: false, error: error.message };
      }

      // Update sync log
      await this.db.update('lms_sync_logs', {
        where: { id: syncId },
        data: {
          status: 'completed',
          message: 'Full course synchronization completed',
          details: results
        }
      });

      return {
        success: true,
        syncId,
        results
      };

    } catch (error) {
      // Update sync log with error
      await this.db.update('lms_sync_logs', {
        where: { id: syncId },
        data: {
          status: 'failed',
          message: error.message,
          details: { error: error.message }
        }
      });
      
      throw error;
    }
  }

  // Get sync status
  async getSyncStatus(integrationId) {
    const logs = await this.db.findMany('lms_sync_logs', {
      where: { lms_integration_id: integrationId },
      orderBy: { created_at: 'desc' },
      limit: 10
    });

    return {
      integrationId,
      recentSyncs: logs
    };
  }

  // Get all LMS instances
  async getLMSInstances() {
    const instances = [];
    
    for (const [id, service] of this.instances) {
      instances.push({
        id,
        type: service.constructor.name.replace('LMSService', ''),
        connected: true,
        lastTest: new Date()
      });
    }

    return instances;
  }

  // Remove LMS instance
  async removeLMSInstance(integrationId) {
    const service = this.instances.get(integrationId);
    if (service && typeof service.disconnect === 'function') {
      await service.disconnect();
    }
    
    this.instances.delete(integrationId);
    return { success: true };
  }

  // Batch operations for multiple LMS instances
  async batchSync(integrationIds, operation, options = {}) {
    const results = [];
    
    for (const integrationId of integrationIds) {
      try {
        let result;
        
        switch (operation) {
          case 'sync_courses':
            result = await this.syncCourses(integrationId, options);
            break;
          
          case 'test_connection':
            result = await this.testConnection(integrationId);
            break;
          
          default:
            throw new Error(`Unsupported batch operation: ${operation}`);
        }
        
        results.push({ integrationId, success: true, result });
      } catch (error) {
        results.push({ 
          integrationId, 
          success: false, 
          error: error.message 
        });
      }
    }

    return {
      operation,
      summary: {
        total: results.length,
        successful: results.filter(r => r.success).length,
        failed: results.filter(r => !r.success).length
      },
      results
    };
  }

  // Webhook handling for LMS events
  async handleWebhook(integrationId, eventType, payload) {
    const service = this.instances.get(integrationId);
    if (!service) {
      throw new Error(`LMS instance not found: ${integrationId}`);
    }

    try {
      let result;
      
      if (typeof service.handleWebhook === 'function') {
        result = await service.handleWebhook(eventType, payload);
      } else if (typeof service.handleWebhookEvent === 'function') {
        result = await service.handleWebhookEvent(eventType, payload);
      } else {
        result = { processed: false, message: 'Webhook handling not implemented' };
      }

      // Log webhook processing
      await this.db.insert('lms_webhook_logs', {
        lms_integration_id: integrationId,
        event_type: eventType,
        payload: payload,
        processed: !!result.processed,
        result: result,
        created_at: new Date()
      });

      return result;
    } catch (error) {
      // Log webhook error
      await this.db.insert('lms_webhook_logs', {
        lms_integration_id: integrationId,
        event_type: eventType,
        payload: payload,
        processed: false,
        error: error.message,
        created_at: new Date()
      });
      
      throw error;
    }
  }

  // Health check for all LMS instances
  async healthCheck() {
    const healthStatus = [];
    
    for (const [id, service] of this.instances) {
      try {
        await this.testConnection(id);
        healthStatus.push({
          id,
          status: 'healthy',
          lastCheck: new Date()
        });
      } catch (error) {
        healthStatus.push({
          id,
          status: 'unhealthy',
          error: error.message,
          lastCheck: new Date()
        });
      }
    }

    return {
      timestamp: new Date(),
      totalInstances: healthStatus.length,
      healthyInstances: healthStatus.filter(h => h.status === 'healthy').length,
      instances: healthStatus
    };
  }
}

module.exports = LMSManager;