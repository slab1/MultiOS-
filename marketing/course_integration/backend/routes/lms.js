const express = require('express');
const { body, query, validationResult } = require('express-validator');
const router = express.Router();

// Get all LMS integrations
router.get('/', async (req, res) => {
  try {
    const user = req.user;
    let integrations;

    if (user.role === 'administrator') {
      // Admin can see all integrations
      integrations = await req.db.findMany('lms_integrations', {
        orderBy: { created_at: 'DESC' },
        include: [
          {
            model: 'institutions',
            as: 'institution',
            attributes: ['id', 'name', 'domain']
          }
        ]
      });
    } else {
      // Regular users can only see their institution's integrations
      integrations = await req.db.findMany('lms_integrations', {
        where: { institution_id: user.institutionId },
        orderBy: { created_at: 'DESC' },
        include: [
          {
            model: 'institutions',
            as: 'institution',
            attributes: ['id', 'name', 'domain']
          }
        ]
      });
    }

    res.json({
      integrations: integrations.map(integration => ({
        id: integration.id,
        name: integration.name,
        type: integration.type,
        baseUrl: integration.base_url,
        isActive: integration.is_active,
        institution: integration.institution,
        lastSync: integration.last_sync,
        createdAt: integration.created_at
      }))
    });

  } catch (error) {
    console.error('Get LMS integrations error:', error);
    res.status(500).json({ error: 'Failed to fetch LMS integrations' });
  }
});

// Get specific LMS integration
router.get('/:integrationId', async (req, res) => {
  try {
    const { integrationId } = req.params;
    const user = req.user;

    const integration = await req.db.findOne('lms_integrations', {
      where: { id: integrationId },
      include: [
        {
          model: 'institutions',
          as: 'institution',
          attributes: ['id', 'name', 'domain']
        },
        {
          model: 'lms_courses',
          as: 'courses',
          limit: 10,
          orderBy: { last_sync: 'DESC' }
        }
      ]
    });

    if (!integration) {
      return res.status(404).json({ error: 'LMS integration not found' });
    }

    // Check permissions
    if (user.role !== 'administrator' && integration.institution_id !== user.institutionId) {
      return res.status(403).json({ error: 'Permission denied' });
    }

    res.json({
      id: integration.id,
      name: integration.name,
      type: integration.type,
      baseUrl: integration.base_url,
      isActive: integration.is_active,
      institution: integration.institution,
      settings: integration.settings,
      lastSync: integration.last_sync,
      courses: integration.courses,
      createdAt: integration.created_at
    });

  } catch (error) {
    console.error('Get LMS integration error:', error);
    res.status(500).json({ error: 'Failed to fetch LMS integration' });
  }
});

// Create new LMS integration
router.post('/', [
  body('name').trim().isLength({ min: 1, max: 255 }),
  body('type').isIn(['canvas', 'blackboard', 'moodle', 'google_classroom', 'microsoft_teams', 'lti_custom']),
  body('baseUrl').isURL(),
  body('settings').optional().isObject()
], async (req, res) => {
  try {
    const errors = validationResult(req);
    if (!errors.isEmpty()) {
      return res.status(400).json({
        error: 'Validation failed',
        details: errors.array()
      });
    }

    const user = req.user;

    // Only administrators and instructors can create integrations
    if (!['administrator', 'instructor'].includes(user.role)) {
      return res.status(403).json({ error: 'Permission denied' });
    }

    const { name, type, baseUrl, settings = {} } = req.body;

    // Validate integration type specific settings
    const validationResult = validateIntegrationSettings(type, settings);
    if (!validationResult.valid) {
      return res.status(400).json({
        error: 'Invalid settings for integration type',
        details: validationResult.errors
      });
    }

    // Create integration
    const integration = await req.db.insert('lms_integrations', {
      name,
      type,
      base_url: baseUrl,
      settings,
      is_active: true,
      institution_id: user.institutionId,
      created_at: new Date(),
      updated_at: new Date()
    });

    // Initialize LMS service (if not LTI)
    if (type !== 'lti_custom' && req.lmsManager) {
      try {
        await req.lmsManager.initializeLMS({
          id: integration.id,
          type: type,
          settings: {
            baseUrl: baseUrl,
            ...settings
          }
        });
      } catch (error) {
        console.warn(`Failed to initialize ${type} service:`, error.message);
      }
    }

    res.status(201).json({
      message: 'LMS integration created successfully',
      integration: {
        id: integration.id,
        name: integration.name,
        type: integration.type,
        baseUrl: integration.base_url,
        isActive: integration.is_active,
        createdAt: integration.created_at
      }
    });

  } catch (error) {
    console.error('Create LMS integration error:', error);
    res.status(500).json({ error: 'Failed to create LMS integration' });
  }
});

// Update LMS integration
router.put('/:integrationId', [
  body('name').optional().trim().isLength({ min: 1, max: 255 }),
  body('baseUrl').optional().isURL(),
  body('settings').optional().isObject(),
  body('isActive').optional().isBoolean()
], async (req, res) => {
  try {
    const errors = validationResult(req);
    if (!errors.isEmpty()) {
      return res.status(400).json({
        error: 'Validation failed',
        details: errors.array()
      });
    }

    const { integrationId } = req.params;
    const user = req.user;

    // Check if integration exists and user has permission
    const integration = await req.db.findOne('lms_integrations', {
      where: { id: integrationId }
    });

    if (!integration) {
      return res.status(404).json({ error: 'LMS integration not found' });
    }

    if (user.role !== 'administrator' && integration.institution_id !== user.institutionId) {
      return res.status(403).json({ error: 'Permission denied' });
    }

    const updateData = { updated_at: new Date() };

    if (req.body.name) updateData.name = req.body.name;
    if (req.body.baseUrl) updateData.base_url = req.body.baseUrl;
    if (req.body.settings) updateData.settings = req.body.settings;
    if (req.body.isActive !== undefined) updateData.is_active = req.body.isActive;

    // Update integration
    const result = await req.db.update('lms_integrations', {
      where: { id: integrationId },
      data: updateData
    });

    if (result[0] === 0) {
      return res.status(404).json({ error: 'LMS integration not found' });
    }

    // Reinitialize service if needed
    if (req.lmsManager && integration.type !== 'lti_custom') {
      try {
        await req.lmsManager.initializeLMS({
          id: integrationId,
          type: integration.type,
          settings: {
            baseUrl: updateData.base_url || integration.base_url,
            ...updateData.settings || integration.settings
          }
        });
      } catch (error) {
        console.warn(`Failed to reinitialize ${integration.type} service:`, error.message);
      }
    }

    res.json({
      message: 'LMS integration updated successfully',
      integration: {
        id: integrationId,
        name: updateData.name || integration.name,
        isActive: updateData.is_active !== undefined ? updateData.is_active : integration.is_active,
        updatedAt: new Date()
      }
    });

  } catch (error) {
    console.error('Update LMS integration error:', error);
    res.status(500).json({ error: 'Failed to update LMS integration' });
  }
});

// Delete LMS integration
router.delete('/:integrationId', async (req, res) => {
  try {
    const { integrationId } = req.params;
    const user = req.user;

    // Check if integration exists and user has permission
    const integration = await req.db.findOne('lms_integrations', {
      where: { id: integrationId }
    });

    if (!integration) {
      return res.status(404).json({ error: 'LMS integration not found' });
    }

    if (user.role !== 'administrator' && integration.institution_id !== user.institutionId) {
      return res.status(403).json({ error: 'Permission denied' });
    }

    // Check for related courses
    const relatedCourses = await req.db.count('lms_courses', {
      where: { lms_integration_id: integrationId }
    });

    if (relatedCourses > 0) {
      return res.status(400).json({
        error: 'Cannot delete integration with related courses',
        relatedCourses
      });
    }

    // Remove from LMS manager
    if (req.lmsManager) {
      await req.lmsManager.removeLMSInstance(integrationId);
    }

    // Delete integration
    const result = await req.db.delete('lms_integrations', {
      where: { id: integrationId }
    });

    if (result[0] === 0) {
      return res.status(404).json({ error: 'LMS integration not found' });
    }

    res.json({ message: 'LMS integration deleted successfully' });

  } catch (error) {
    console.error('Delete LMS integration error:', error);
    res.status(500).json({ error: 'Failed to delete LMS integration' });
  }
});

// Test LMS connection
router.post('/:integrationId/test', async (req, res) => {
  try {
    const { integrationId } = req.params;
    const user = req.user;

    // Check if integration exists and user has permission
    const integration = await req.db.findOne('lms_integrations', {
      where: { id: integrationId }
    });

    if (!integration) {
      return res.status(404).json({ error: 'LMS integration not found' });
    }

    if (user.role !== 'administrator' && integration.institution_id !== user.institutionId) {
      return res.status(403).json({ error: 'Permission denied' });
    }

    if (!req.lmsManager) {
      return res.status(503).json({ error: 'LMS manager not available' });
    }

    // Test connection
    const result = await req.lmsManager.testConnection(integrationId);

    res.json({
      message: 'Connection test completed',
      result
    });

  } catch (error) {
    console.error('Test LMS connection error:', error);
    res.status(500).json({ 
      error: 'Connection test failed',
      details: error.message 
    });
  }
});

// Sync courses from LMS
router.post('/:integrationId/sync-courses', [
  query('includeEnrollments').optional().isBoolean()
], async (req, res) => {
  try {
    const { integrationId } = req.params;
    const { includeEnrollments = true } = req.query;
    const user = req.user;

    // Check if integration exists and user has permission
    const integration = await req.db.findOne('lms_integrations', {
      where: { id: integrationId }
    });

    if (!integration) {
      return res.status(404).json({ error: 'LMS integration not found' });
    }

    if (user.role !== 'administrator' && integration.institution_id !== user.institutionId) {
      return res.status(403).json({ error: 'Permission denied' });
    }

    if (!req.lmsManager) {
      return res.status(503).json({ error: 'LMS manager not available' });
    }

    // Start sync process
    res.status(202).json({
      message: 'Course synchronization started',
      syncId: `${integrationId}:${Date.now()}`
    });

    // Run sync in background
    try {
      const result = await req.lmsManager.syncCourses(integrationId, { includeEnrollments });
      
      // Log sync result
      await req.db.insert('lms_sync_logs', {
        lms_integration_id: integrationId,
        operation: 'sync_courses',
        status: result.success ? 'completed' : 'failed',
        message: result.summary ? `Synced ${result.summary.successful}/${result.summary.total} courses` : 'Sync completed',
        details: result,
        created_at: new Date()
      });

    } catch (syncError) {
      console.error('Background sync error:', syncError);
      
      // Log sync error
      await req.db.insert('lms_sync_logs', {
        lms_integration_id: integrationId,
        operation: 'sync_courses',
        status: 'failed',
        message: syncError.message,
        created_at: new Date()
      });
    }

  } catch (error) {
    console.error('Sync courses error:', error);
    res.status(500).json({ error: 'Failed to start course synchronization' });
  }
});

// Sync specific course
router.post('/:integrationId/sync-course/:externalCourseId', async (req, res) => {
  try {
    const { integrationId, externalCourseId } = req.params;
    const user = req.user;

    // Check if integration exists and user has permission
    const integration = await req.db.findOne('lms_integrations', {
      where: { id: integrationId }
    });

    if (!integration) {
      return res.status(404).json({ error: 'LMS integration not found' });
    }

    if (user.role !== 'administrator' && integration.institution_id !== user.institutionId) {
      return res.status(403).json({ error: 'Permission denied' });
    }

    if (!req.lmsManager) {
      return res.status(503).json({ error: 'LMS manager not available' });
    }

    // Get LMS course record
    const lmsCourse = await req.db.findOne('lms_courses', {
      where: {
        lms_integration_id: integrationId,
        external_course_id: externalCourseId
      }
    });

    if (!lmsCourse) {
      return res.status(404).json({ error: 'LMS course not found' });
    }

    // Start full sync for specific course
    res.status(202).json({
      message: 'Course synchronization started',
      courseId: lmsCourse.id
    });

    // Run sync in background
    try {
      const result = await req.lmsManager.syncFullCourse(integrationId, externalCourseId);
      
    } catch (syncError) {
      console.error('Background course sync error:', syncError);
    }

  } catch (error) {
    console.error('Sync course error:', error);
    res.status(500).json({ error: 'Failed to start course synchronization' });
  }
});

// Get LMS courses
router.get('/:integrationId/courses', [
  query('page').optional().isInt({ min: 1 }),
  query('limit').optional().isInt({ min: 1, max: 100 }),
  query('search').optional().isString(),
  query('syncStatus').optional().isIn(['pending', 'in_progress', 'completed', 'failed'])
], async (req, res) => {
  try {
    const { integrationId } = req.params;
    const { page = 1, limit = 20, search, syncStatus } = req.query;
    const user = req.user;

    // Check permissions
    const integration = await req.db.findOne('lms_integrations', {
      where: { id: integrationId }
    });

    if (!integration) {
      return res.status(404).json({ error: 'LMS integration not found' });
    }

    if (user.role !== 'administrator' && integration.institution_id !== user.institutionId) {
      return res.status(403).json({ error: 'Permission denied' });
    }

    const offset = (page - 1) * limit;
    const conditions = { lms_integration_id: integrationId };

    if (search) {
      conditions.$or = [
        { course_name: { $ilike: `%${search}%` } },
        { course_code: { $ilike: `%${search}%` } },
        { instructor_name: { $ilike: `%${search}%` } }
      ];
    }

    if (syncStatus) conditions.sync_status = syncStatus;

    const { rows: courses, count: total } = await req.db.findManyWithCount('lms_courses', {
      where: conditions,
      limit: parseInt(limit),
      offset: parseInt(offset),
      orderBy: [
        { column: 'last_sync', direction: 'DESC' },
        { column: 'course_name', direction: 'ASC' }
      ]
    });

    res.json({
      courses: courses.map(course => ({
        id: course.id,
        externalCourseId: course.external_course_id,
        courseName: course.course_name,
        courseCode: course.course_code,
        description: course.description,
        instructorName: course.instructor_name,
        studentsCount: course.students_count,
        lastSync: course.last_sync,
        syncStatus: course.sync_status,
        syncError: course.sync_error
      })),
      pagination: {
        page: parseInt(page),
        limit: parseInt(limit),
        total,
        pages: Math.ceil(total / limit)
      }
    });

  } catch (error) {
    console.error('Get LMS courses error:', error);
    res.status(500).json({ error: 'Failed to fetch LMS courses' });
  }
});

// Get sync logs
router.get('/:integrationId/sync-logs', [
  query('page').optional().isInt({ min: 1 }),
  query('limit').optional().isInt({ min: 1, max: 50 }),
  query('operation').optional().isString(),
  query('status').optional().isIn(['pending', 'in_progress', 'completed', 'failed'])
], async (req, res) => {
  try {
    const { integrationId } = req.params;
    const { page = 1, limit = 20, operation, status } = req.query;
    const user = req.user;

    // Check permissions
    const integration = await req.db.findOne('lms_integrations', {
      where: { id: integrationId }
    });

    if (!integration) {
      return res.status(404).json({ error: 'LMS integration not found' });
    }

    if (user.role !== 'administrator' && integration.institution_id !== user.institutionId) {
      return res.status(403).json({ error: 'Permission denied' });
    }

    const offset = (page - 1) * limit;
    const conditions = { lms_integration_id: integrationId };

    if (operation) conditions.operation = operation;
    if (status) conditions.status = status;

    const { rows: logs, count: total } = await req.db.findManyWithCount('lms_sync_logs', {
      where: conditions,
      limit: parseInt(limit),
      offset: parseInt(offset),
      orderBy: { created_at: 'DESC' }
    });

    res.json({
      logs: logs.map(log => ({
        id: log.id,
        operation: log.operation,
        status: log.status,
        message: log.message,
        details: log.details,
        createdAt: log.created_at
      })),
      pagination: {
        page: parseInt(page),
        limit: parseInt(limit),
        total,
        pages: Math.ceil(total / limit)
      }
    });

  } catch (error) {
    console.error('Get sync logs error:', error);
    res.status(500).json({ error: 'Failed to fetch sync logs' });
  }
});

// LTI configuration
router.get('/:integrationId/lti-config', async (req, res) => {
  try {
    const { integrationId } = req.params;
    const user = req.user;

    // Check if integration exists and is LTI type
    const integration = await req.db.findOne('lms_integrations', {
      where: { id: integrationId, type: 'lti_custom' }
    });

    if (!integration) {
      return res.status(404).json({ error: 'LTI integration not found' });
    }

    if (user.role !== 'administrator' && integration.institution_id !== user.institutionId) {
      return res.status(403).json({ error: 'Permission denied' });
    }

    // Generate LTI configuration XML
    if (req.lmsManager && req.lmsManager.ltiService) {
      const configXML = req.lmsManager.ltiService.generateConfigXML({
        title: integration.name,
        description: integration.settings.description || 'MultiOS Learning Tool',
        launch_url: integration.settings.launch_url || `${req.protocol}://${req.get('host')}/api/lms/lti/launch`,
        custom_parameters: integration.settings.custom_parameters || {}
      });

      res.set('Content-Type', 'application/xml');
      res.send(configXML);
    } else {
      res.status(503).json({ error: 'LTI service not available' });
    }

  } catch (error) {
    console.error('Get LTI config error:', error);
    res.status(500).json({ error: 'Failed to generate LTI configuration' });
  }
});

// Helper function to validate integration settings
function validateIntegrationSettings(type, settings) {
  const errors = [];
  let valid = true;

  switch (type) {
    case 'canvas':
      if (!settings.clientId) {
        errors.push('Canvas client ID is required');
        valid = false;
      }
      if (!settings.clientSecret) {
        errors.push('Canvas client secret is required');
        valid = false;
      }
      break;

    case 'blackboard':
      if (!settings.clientId) {
        errors.push('Blackboard client ID is required');
        valid = false;
      }
      if (!settings.clientSecret) {
        errors.push('Blackboard client secret is required');
        valid = false;
      }
      break;

    case 'moodle':
      if (!settings.wstoken) {
        errors.push('Moodle web service token is required');
        valid = false;
      }
      break;

    case 'google_classroom':
      if (!settings.clientId) {
        errors.push('Google client ID is required');
        valid = false;
      }
      if (!settings.clientSecret) {
        errors.push('Google client secret is required');
        valid = false;
      }
      break;

    case 'microsoft_teams':
      if (!settings.clientId) {
        errors.push('Microsoft client ID is required');
        valid = false;
      }
      if (!settings.clientSecret) {
        errors.push('Microsoft client secret is required');
        valid = false;
      }
      if (!settings.tenantId) {
        errors.push('Microsoft tenant ID is required');
        valid = false;
      }
      break;

    case 'lti_custom':
      if (!settings.consumerKey) {
        errors.push('LTI consumer key is required');
        valid = false;
      }
      if (!settings.consumerSecret) {
        errors.push('LTI consumer secret is required');
        valid = false;
      }
      break;
  }

  return { valid, errors };
}

module.exports = router;