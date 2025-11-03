const express = require('express');
const { body, query, validationResult } = require('express-validator');
const router = express.Router();

// Get all assignments
router.get('/', [
  query('courseId').optional().isUUID(),
  query('page').optional().isInt({ min: 1 }),
  query('limit').optional().isInt({ min: 1, max: 100 }),
  query('status').optional().isIn(['draft', 'published', 'graded', 'closed'])
], async (req, res) => {
  try {
    const errors = validationResult(req);
    if (!errors.isEmpty()) {
      return res.status(400).json({
        error: 'Validation failed',
        details: errors.array()
      });
    }

    const { courseId, page = 1, limit = 20, status } = req.query;
    const user = req.user;

    const conditions = {};
    
    // Students can only see assignments for courses they're enrolled in
    if (user.role === 'student') {
      const enrollments = await req.db.findMany('course_enrollments', {
        where: { student_id: user.userId, status: 'active' }
      });
      const enrolledCourseIds = enrollments.map(e => e.course_id);
      conditions.course_id = { $in: enrolledCourseIds };
    } else if (user.role === 'instructor') {
      // Instructors can see assignments for their courses
      conditions.course_id = await req.db.findMany('courses', {
        where: { instructor_id: user.userId },
        select: ['id']
      }).then(courses => courses.map(c => c.id));
    }

    if (courseId) conditions.course_id = courseId;
    if (status) conditions.status = status;

    const { rows: assignments, count: total } = await req.db.findManyWithCount('assignments', {
      where: conditions,
      limit: parseInt(limit),
      offset: (page - 1) * limit,
      include: [
        {
          model: 'courses',
          as: 'course',
          attributes: ['id', 'title', 'course_code']
        }
      ],
      orderBy: { created_at: 'DESC' }
    });

    res.json({
      assignments: assignments.map(assignment => ({
        id: assignment.id,
        title: assignment.title,
        description: assignment.description,
        dueDate: assignment.due_date,
        maxScore: assignment.max_score,
        weight: assignment.weight,
        status: assignment.status,
        course: assignment.course,
        createdAt: assignment.created_at
      })),
      pagination: {
        page: parseInt(page),
        limit: parseInt(limit),
        total,
        pages: Math.ceil(total / limit)
      }
    });

  } catch (error) {
    console.error('Get assignments error:', error);
    res.status(500).json({ error: 'Failed to fetch assignments' });
  }
});

// Get assignment by ID
router.get('/:assignmentId', async (req, res) => {
  try {
    const { assignmentId } = req.params;
    const user = req.user;

    const assignment = await req.db.findOne('assignments', {
      where: { id: assignmentId },
      include: [
        {
          model: 'courses',
          as: 'course',
          attributes: ['id', 'title', 'course_code', 'instructor_id']
        }
      ]
    });

    if (!assignment) {
      return res.status(404).json({ error: 'Assignment not found' });
    }

    // Check permissions
    if (user.role === 'student') {
      const enrollment = await req.db.findOne('course_enrollments', {
        where: {
          student_id: user.userId,
          course_id: assignment.course_id,
          status: 'active'
        }
      });
      if (!enrollment) {
        return res.status(403).json({ error: 'Permission denied' });
      }
    } else if (user.role === 'instructor' && assignment.course.instructor_id !== user.userId) {
      return res.status(403).json({ error: 'Permission denied' });
    }

    res.json({
      id: assignment.id,
      title: assignment.title,
      description: assignment.description,
      instructions: assignment.instructions,
      dueDate: assignment.due_date,
      maxScore: assignment.max_score,
      weight: assignment.weight,
      submissionRequirements: assignment.submission_requirements,
      rubric: assignment.rubric,
      status: assignment.status,
      course: assignment.course,
      createdAt: assignment.created_at
    });

  } catch (error) {
    console.error('Get assignment error:', error);
    res.status(500).json({ error: 'Failed to fetch assignment' });
  }
});

// Create assignment
router.post('/', [
  body('courseId').isUUID(),
  body('title').trim().isLength({ min: 1, max: 255 }),
  body('description').optional().isString(),
  body('instructions').optional().isString(),
  body('dueDate').optional().isISO8601(),
  body('maxScore').optional().isInt({ min: 1 }),
  body('weight').optional().isFloat({ min: 0, max: 1 })
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
    
    // Check if user is instructor or admin
    if (!['instructor', 'administrator'].includes(user.role)) {
      return res.status(403).json({ error: 'Permission denied' });
    }

    // Verify course ownership
    const course = await req.db.findOne('courses', {
      where: { id: req.body.courseId }
    });

    if (!course) {
      return res.status(404).json({ error: 'Course not found' });
    }

    if (user.role === 'instructor' && course.instructor_id !== user.userId) {
      return res.status(403).json({ error: 'Permission denied' });
    }

    const assignment = await req.db.insert('assignments', {
      course_id: req.body.courseId,
      title: req.body.title,
      description: req.body.description || '',
      instructions: req.body.instructions || '',
      due_date: req.body.dueDate ? new Date(req.body.dueDate) : null,
      max_score: req.body.maxScore || 100,
      weight: req.body.weight || 1.0,
      submission_requirements: req.body.submissionRequirements || {},
      rubric: req.body.rubric || {},
      status: 'draft',
      created_at: new Date(),
      updated_at: new Date()
    });

    res.status(201).json({
      message: 'Assignment created successfully',
      assignment: {
        id: assignment.id,
        title: assignment.title,
        dueDate: assignment.due_date,
        maxScore: assignment.max_score,
        status: assignment.status
      }
    });

  } catch (error) {
    console.error('Create assignment error:', error);
    res.status(500).json({ error: 'Failed to create assignment' });
  }
});

// Submit assignment
router.post('/:assignmentId/submit', [
  body('submissionText').optional().isString(),
  body('files').optional().isArray()
], async (req, res) => {
  try {
    const errors = validationResult(req);
    if (!errors.isEmpty()) {
      return res.status(400).json({
        error: 'Validation failed',
        details: errors.array()
      });
    }

    const { assignmentId } = req.params;
    const user = req.user;

    // Check if user is student
    if (user.role !== 'student') {
      return res.status(403).json({ error: 'Only students can submit assignments' });
    }

    const assignment = await req.db.findOne('assignments', {
      where: { id: assignmentId },
      include: [{
        model: 'courses',
        as: 'course'
      }]
    });

    if (!assignment) {
      return res.status(404).json({ error: 'Assignment not found' });
    }

    // Check if student is enrolled
    const enrollment = await req.db.findOne('course_enrollments', {
      where: {
        student_id: user.userId,
        course_id: assignment.course_id,
        status: 'active'
      }
    });

    if (!enrollment) {
      return res.status(403).json({ error: 'Not enrolled in this course' });
    }

    // Check due date
    if (assignment.due_date && new Date() > new Date(assignment.due_date)) {
      return res.status(400).json({ error: 'Assignment is past due' });
    }

    // Create or update submission
    const submission = await req.db.upsert('assignment_submissions', {
      where: {
        assignment_id: assignmentId,
        student_id: user.userId
      },
      defaults: {
        assignment_id: assignmentId,
        student_id: user.userId,
        submission_text: req.body.submissionText || '',
        files: req.body.files || [],
        submitted_at: new Date(),
        status: 'submitted'
      },
      update: {
        submission_text: req.body.submissionText || '',
        files: req.body.files || [],
        submitted_at: new Date(),
        status: 'submitted'
      }
    });

    res.json({
      message: 'Assignment submitted successfully',
      submission: {
        id: submission.id,
        submittedAt: submission.submitted_at,
        status: submission.status
      }
    });

  } catch (error) {
    console.error('Submit assignment error:', error);
    res.status(500).json({ error: 'Failed to submit assignment' });
  }
});

// Grade assignment
router.post('/:assignmentId/grade/:submissionId', [
  body('score').isInt({ min: 0 }),
  body('feedback').optional().isString()
], async (req, res) => {
  try {
    const errors = validationResult(req);
    if (!errors.isEmpty()) {
      return res.status(400).json({
        error: 'Validation failed',
        details: errors.array()
      });
    }

    const { assignmentId, submissionId } = req.params;
    const user = req.user;

    // Check if user is instructor or admin
    if (!['instructor', 'administrator'].includes(user.role)) {
      return res.status(403).json({ error: 'Permission denied' });
    }

    const submission = await req.db.findOne('assignment_submissions', {
      where: { id: submissionId, assignment_id: assignmentId },
      include: [{
        model: 'courses',
        as: 'course',
        attributes: ['instructor_id']
      }]
    });

    if (!submission) {
      return res.status(404).json({ error: 'Submission not found' });
    }

    // Check permissions
    if (user.role === 'instructor' && submission.course.instructor_id !== user.userId) {
      return res.status(403).json({ error: 'Permission denied' });
    }

    // Update grade
    await req.db.update('assignment_submissions', {
      where: { id: submissionId },
      data: {
        score: req.body.score,
        feedback: req.body.feedback || '',
        graded_by: user.userId,
        graded_at: new Date(),
        status: 'graded'
      }
    });

    res.json({
      message: 'Assignment graded successfully',
      grade: {
        score: req.body.score,
        feedback: req.body.feedback
      }
    });

  } catch (error) {
    console.error('Grade assignment error:', error);
    res.status(500).json({ error: 'Failed to grade assignment' });
  }
});

module.exports = router;