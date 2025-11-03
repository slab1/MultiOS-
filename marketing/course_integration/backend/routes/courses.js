const express = require('express');
const { body, query, validationResult } = require('express-validator');
const router = express.Router();

// Get all courses
router.get('/', [
  query('page').optional().isInt({ min: 1 }),
  query('limit').optional().isInt({ min: 1, max: 100 }),
  query('search').optional().isString(),
  query('category').optional().isString(),
  query('level').optional().isIn(['beginner', 'intermediate', 'advanced', 'expert']),
  query('instructor').optional().isUUID(),
  query('published').optional().isBoolean()
], async (req, res) => {
  try {
    const errors = validationResult(req);
    if (!errors.isEmpty()) {
      return res.status(400).json({
        error: 'Validation failed',
        details: errors.array()
      });
    }

    const {
      page = 1,
      limit = 20,
      search,
      category,
      level,
      instructor,
      published
    } = req.query;

    const offset = (page - 1) * limit;

    // Build query conditions
    const conditions = {};
    const queryParams = {};

    if (search) {
      conditions.$or = [
        { title: { $ilike: `%${search}%` } },
        { description: { $ilike: `%${search}%` } },
        { course_code: { $ilike: `%${search}%` } }
      ];
    }

    if (category) conditions.category = category;
    if (level) conditions.level = level;
    if (instructor) conditions.instructor_id = instructor;
    if (published !== undefined) conditions.is_published = published === 'true';

    // Get courses with pagination
    const { rows: courses, count: total } = await req.db.findManyWithCount('courses', {
      where: conditions,
      limit: parseInt(limit),
      offset: parseInt(offset),
      include: [
        {
          model: 'users',
          as: 'instructor',
          attributes: ['id', 'first_name', 'last_name', 'email']
        },
        {
          model: 'institutions',
          as: 'institution',
          attributes: ['id', 'name', 'domain']
        },
        {
          model: 'lms_courses',
          as: 'lms_course',
          attributes: ['external_course_id', 'sync_status']
        }
      ],
      orderBy: [
        { column: 'created_at', direction: 'DESC' }
      ]
    });

    res.json({
      courses: courses.map(course => ({
        id: course.id,
        title: course.title,
        description: course.description,
        courseCode: course.course_code,
        level: course.level,
        category: course.category,
        instructor: course.instructor,
        institution: course.institution,
        startDate: course.start_date,
        endDate: course.end_date,
        estimatedDurationHours: course.estimated_duration_hours,
        maxEnrollment: course.max_enrollment,
        isPublished: course.is_published,
        lmsIntegration: course.lms_course,
        createdAt: course.created_at
      })),
      pagination: {
        page: parseInt(page),
        limit: parseInt(limit),
        total,
        pages: Math.ceil(total / limit)
      }
    });

  } catch (error) {
    console.error('Get courses error:', error);
    res.status(500).json({ error: 'Failed to fetch courses' });
  }
});

// Get course by ID
router.get('/:courseId', [
  query('includeModules').optional().isBoolean(),
  query('includeEnrollments').optional().isBoolean()
], async (req, res) => {
  try {
    const { courseId } = req.params;
    const { includeModules, includeEnrollments } = req.query;

    // Get course
    const course = await req.db.findOne('courses', {
      where: { id: courseId },
      include: [
        {
          model: 'users',
          as: 'instructor',
          attributes: ['id', 'first_name', 'last_name', 'email']
        },
        {
          model: 'institutions',
          as: 'institution',
          attributes: ['id', 'name', 'domain']
        },
        {
          model: 'lms_courses',
          as: 'lms_course'
        }
      ]
    });

    if (!course) {
      return res.status(404).json({ error: 'Course not found' });
    }

    const courseData = {
      id: course.id,
      title: course.title,
      description: course.description,
      courseCode: course.course_code,
      level: course.level,
      category: course.category,
      instructor: course.instructor,
      institution: course.institution,
      curriculum: course.curriculum,
      learningObjectives: course.learning_objectives,
      prerequisites: course.prerequisites,
      estimatedDurationHours: course.estimated_duration_hours,
      maxEnrollment: course.max_enrollment,
      isPublished: course.is_published,
      startDate: course.start_date,
      endDate: course.end_date,
      settings: course.settings,
      lmsIntegration: course.lms_course,
      createdAt: course.created_at,
      updatedAt: course.updated_at
    };

    // Include modules if requested
    if (includeModules) {
      const modules = await req.db.findMany('course_modules', {
        where: { course_id: courseId, is_published: true },
        orderBy: { order_index: 'ASC' }
      });

      courseData.modules = modules.map(module => ({
        id: module.id,
        title: module.title,
        description: module.description,
        moduleType: module.module_type,
        orderIndex: module.order_index,
        estimatedDurationMinutes: module.estimated_duration_minutes,
        learningObjectives: module.learning_objectives,
        assessmentCriteria: module.assessment_criteria
      }));
    }

    // Include enrollments if requested
    if (includeEnrollments) {
      const enrollments = await req.db.findMany('course_enrollments', {
        where: { course_id: courseId },
        include: [
          {
            model: 'users',
            as: 'student',
            attributes: ['id', 'first_name', 'last_name', 'email']
          }
        ]
      });

      courseData.enrollments = enrollments.map(enrollment => ({
        id: enrollment.id,
        student: enrollment.student,
        enrollmentDate: enrollment.enrollment_date,
        status: enrollment.status,
        progressPercentage: enrollment.progress_percentage,
        completionDate: enrollment.completion_date,
        grade: enrollment.grade
      }));
    }

    res.json(courseData);

  } catch (error) {
    console.error('Get course error:', error);
    res.status(500).json({ error: 'Failed to fetch course' });
  }
});

// Create new course
router.post('/', [
  body('title').trim().isLength({ min: 1, max: 255 }),
  body('description').optional().isString(),
  body('courseCode').trim().isLength({ min: 1, max: 100 }).matches(/^[A-Z0-9_-]+$/),
  body('level').isIn(['beginner', 'intermediate', 'advanced', 'expert']),
  body('category').optional().isString(),
  body('learningObjectives').optional().isArray(),
  body('prerequisites').optional().isArray(),
  body('estimatedDurationHours').optional().isInt({ min: 1 }),
  body('maxEnrollment').optional().isInt({ min: 1 }),
  body('startDate').optional().isISO8601(),
  body('endDate').optional().isISO8601()
], async (req, res) => {
  try {
    const errors = validationResult(req);
    if (!errors.isEmpty()) {
      return res.status(400).json({
        error: 'Validation failed',
        details: errors.array()
      });
    }

    // Check if course code already exists
    const existingCourse = await req.db.findOne('courses', {
      where: { course_code: req.body.courseCode }
    });

    if (existingCourse) {
      return res.status(409).json({ error: 'Course code already exists' });
    }

    const user = req.user; // From auth middleware

    // Create course
    const course = await req.db.insert('courses', {
      title: req.body.title,
      description: req.body.description || '',
      course_code: req.body.courseCode,
      level: req.body.level,
      category: req.body.category || 'General',
      instructor_id: user.userId,
      institution_id: user.institutionId || null,
      curriculum: req.body.curriculum || {},
      learning_objectives: req.body.learningObjectives || [],
      prerequisites: req.body.prerequisites || [],
      estimated_duration_hours: req.body.estimatedDurationHours || null,
      max_enrollment: req.body.maxEnrollment || null,
      start_date: req.body.startDate ? new Date(req.body.startDate) : null,
      end_date: req.body.endDate ? new Date(req.body.endDate) : null,
      is_published: false,
      settings: req.body.settings || {},
      created_at: new Date(),
      updated_at: new Date()
    });

    res.status(201).json({
      message: 'Course created successfully',
      course: {
        id: course.id,
        title: course.title,
        description: course.description,
        courseCode: course.course_code,
        level: course.level,
        category: course.category,
        createdAt: course.created_at
      }
    });

  } catch (error) {
    console.error('Create course error:', error);
    res.status(500).json({ error: 'Failed to create course' });
  }
});

// Update course
router.put('/:courseId', [
  body('title').optional().trim().isLength({ min: 1, max: 255 }),
  body('description').optional().isString(),
  body('level').optional().isIn(['beginner', 'intermediate', 'advanced', 'expert']),
  body('category').optional().isString(),
  body('learningObjectives').optional().isArray(),
  body('prerequisites').optional().isArray(),
  body('estimatedDurationHours').optional().isInt({ min: 1 }),
  body('maxEnrollment').optional().isInt({ min: 1 }),
  body('startDate').optional().isISO8601(),
  body('endDate').optional().isISO8601(),
  body('isPublished').optional().isBoolean()
], async (req, res) => {
  try {
    const errors = validationResult(req);
    if (!errors.isEmpty()) {
      return res.status(400).json({
        error: 'Validation failed',
        details: errors.array()
      });
    }

    const { courseId } = req.params;
    const user = req.user;

    // Check if course exists and user has permission
    const course = await req.db.findOne('courses', {
      where: { id: courseId }
    });

    if (!course) {
      return res.status(404).json({ error: 'Course not found' });
    }

    // Check permissions (only instructor or admin can update)
    if (user.role !== 'administrator' && course.instructor_id !== user.userId) {
      return res.status(403).json({ error: 'Permission denied' });
    }

    // Build update data
    const updateData = { updated_at: new Date() };
    
    if (req.body.title) updateData.title = req.body.title;
    if (req.body.description !== undefined) updateData.description = req.body.description;
    if (req.body.level) updateData.level = req.body.level;
    if (req.body.category) updateData.category = req.body.category;
    if (req.body.learningObjectives) updateData.learning_objectives = req.body.learningObjectives;
    if (req.body.prerequisites) updateData.prerequisites = req.body.prerequisites;
    if (req.body.estimatedDurationHours) updateData.estimated_duration_hours = req.body.estimatedDurationHours;
    if (req.body.maxEnrollment) updateData.max_enrollment = req.body.maxEnrollment;
    if (req.body.startDate) updateData.start_date = new Date(req.body.startDate);
    if (req.body.endDate) updateData.end_date = new Date(req.body.endDate);
    if (req.body.isPublished !== undefined) updateData.is_published = req.body.isPublished;
    if (req.body.settings) updateData.settings = req.body.settings;
    if (req.body.curriculum) updateData.curriculum = req.body.curriculum;

    // Update course
    const result = await req.db.update('courses', {
      where: { id: courseId },
      data: updateData
    });

    if (result[0] === 0) {
      return res.status(404).json({ error: 'Course not found' });
    }

    // Get updated course
    const updatedCourse = await req.db.findOne('courses', {
      where: { id: courseId },
      include: [
        {
          model: 'users',
          as: 'instructor',
          attributes: ['id', 'first_name', 'last_name', 'email']
        }
      ]
    });

    res.json({
      message: 'Course updated successfully',
      course: {
        id: updatedCourse.id,
        title: updatedCourse.title,
        description: updatedCourse.description,
        courseCode: updatedCourse.course_code,
        level: updatedCourse.level,
        category: updatedCourse.category,
        instructor: updatedCourse.instructor,
        isPublished: updatedCourse.is_published,
        updatedAt: updatedCourse.updated_at
      }
    });

  } catch (error) {
    console.error('Update course error:', error);
    res.status(500).json({ error: 'Failed to update course' });
  }
});

// Delete course
router.delete('/:courseId', async (req, res) => {
  try {
    const { courseId } = req.params;
    const user = req.user;

    // Check if course exists and user has permission
    const course = await req.db.findOne('courses', {
      where: { id: courseId }
    });

    if (!course) {
      return res.status(404).json({ error: 'Course not found' });
    }

    // Check permissions (only instructor or admin can delete)
    if (user.role !== 'administrator' && course.instructor_id !== user.userId) {
      return res.status(403).json({ error: 'Permission denied' });
    }

    // Check for active enrollments
    const activeEnrollments = await req.db.count('course_enrollments', {
      where: {
        course_id: courseId,
        status: 'active'
      }
    });

    if (activeEnrollments > 0) {
      return res.status(400).json({
        error: 'Cannot delete course with active enrollments',
        activeEnrollments
      });
    }

    // Delete course (cascade will handle related records)
    const result = await req.db.delete('courses', {
      where: { id: courseId }
    });

    if (result[0] === 0) {
      return res.status(404).json({ error: 'Course not found' });
    }

    res.json({ message: 'Course deleted successfully' });

  } catch (error) {
    console.error('Delete course error:', error);
    res.status(500).json({ error: 'Failed to delete course' });
  }
});

// Get course modules
router.get('/:courseId/modules', [
  query('published').optional().isBoolean()
], async (req, res) => {
  try {
    const { courseId } = req.params;
    const { published = true } = req.query;

    // Check if course exists
    const course = await req.db.findOne('courses', {
      where: { id: courseId }
    });

    if (!course) {
      return res.status(404).json({ error: 'Course not found' });
    }

    const conditions = { course_id: courseId };
    if (published) conditions.is_published = true;

    const modules = await req.db.findMany('course_modules', {
      where: conditions,
      orderBy: { order_index: 'ASC' }
    });

    res.json({
      modules: modules.map(module => ({
        id: module.id,
        title: module.title,
        description: module.description,
        moduleType: module.module_type,
        content: module.content,
        orderIndex: module.order_index,
        estimatedDurationMinutes: module.estimated_duration_minutes,
        prerequisites: module.prerequisites,
        learningObjectives: module.learning_objectives,
        assessmentCriteria: module.assessment_criteria,
        isPublished: module.is_published,
        createdAt: module.created_at
      }))
    });

  } catch (error) {
    console.error('Get course modules error:', error);
    res.status(500).json({ error: 'Failed to fetch course modules' });
  }
});

// Add module to course
router.post('/:courseId/modules', [
  body('title').trim().isLength({ min: 1, max: 255 }),
  body('description').optional().isString(),
  body('moduleType').isIn(['video', 'reading', 'assignment', 'lab', 'quiz', 'discussion', 'project']),
  body('orderIndex').isInt({ min: 1 }),
  body('estimatedDurationMinutes').optional().isInt({ min: 1 }),
  body('prerequisites').optional().isArray(),
  body('learningObjectives').optional().isArray(),
  body('assessmentCriteria').optional().isObject()
], async (req, res) => {
  try {
    const errors = validationResult(req);
    if (!errors.isEmpty()) {
      return res.status(400).json({
        error: 'Validation failed',
        details: errors.array()
      });
    }

    const { courseId } = req.params;
    const user = req.user;

    // Check if course exists and user has permission
    const course = await req.db.findOne('courses', {
      where: { id: courseId }
    });

    if (!course) {
      return res.status(404).json({ error: 'Course not found' });
    }

    if (user.role !== 'administrator' && course.instructor_id !== user.userId) {
      return res.status(403).json({ error: 'Permission denied' });
    }

    // Create module
    const module = await req.db.insert('course_modules', {
      course_id: courseId,
      title: req.body.title,
      description: req.body.description || '',
      module_type: req.body.moduleType,
      content: req.body.content || {},
      order_index: req.body.orderIndex,
      estimated_duration_minutes: req.body.estimatedDurationMinutes || null,
      prerequisites: req.body.prerequisites || [],
      learning_objectives: req.body.learningObjectives || [],
      assessment_criteria: req.body.assessmentCriteria || {},
      is_published: false,
      created_at: new Date(),
      updated_at: new Date()
    });

    res.status(201).json({
      message: 'Module created successfully',
      module: {
        id: module.id,
        title: module.title,
        description: module.description,
        moduleType: module.module_type,
        orderIndex: module.order_index,
        createdAt: module.created_at
      }
    });

  } catch (error) {
    console.error('Create module error:', error);
    res.status(500).json({ error: 'Failed to create module' });
  }
});

// Enroll in course
router.post('/:courseId/enroll', async (req, res) => {
  try {
    const { courseId } = req.params;
    const user = req.user;

    // Check if course exists and is published
    const course = await req.db.findOne('courses', {
      where: { id: courseId, is_published: true }
    });

    if (!course) {
      return res.status(404).json({ error: 'Course not found or not published' });
    }

    // Check if already enrolled
    const existingEnrollment = await req.db.findOne('course_enrollments', {
      where: {
        student_id: user.userId,
        course_id: courseId
      }
    });

    if (existingEnrollment) {
      return res.status(409).json({ error: 'Already enrolled in this course' });
    }

    // Check enrollment capacity
    if (course.max_enrollment) {
      const currentEnrollment = await req.db.count('course_enrollments', {
        where: {
          course_id: courseId,
          status: 'active'
        }
      });

      if (currentEnrollment >= course.max_enrollment) {
        return res.status(400).json({ error: 'Course enrollment is full' });
      }
    }

    // Create enrollment
    const enrollment = await req.db.insert('course_enrollments', {
      student_id: user.userId,
      course_id: courseId,
      enrollment_date: new Date(),
      status: 'active',
      progress_percentage: 0
    });

    res.status(201).json({
      message: 'Successfully enrolled in course',
      enrollment: {
        id: enrollment.id,
        courseId: enrollment.course_id,
        enrollmentDate: enrollment.enrollment_date,
        status: enrollment.status
      }
    });

  } catch (error) {
    console.error('Course enrollment error:', error);
    res.status(500).json({ error: 'Failed to enroll in course' });
  }
});

module.exports = router;