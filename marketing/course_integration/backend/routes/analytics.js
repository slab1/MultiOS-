const express = require('express');
const { query } = require('express-validator');
const router = express.Router();

// Get student analytics
router.get('/student/:studentId', [
  query('courseId').optional().isUUID(),
  query('timeframe').optional().isIn(['week', 'month', 'semester', 'all'])
], async (req, res) => {
  try {
    const { studentId } = req.params;
    const { courseId, timeframe = 'month' } = req.query;
    const user = req.user;

    // Check permissions
    if (user.role === 'student' && user.userId !== studentId) {
      return res.status(403).json({ error: 'Permission denied' });
    }

    const conditions = { student_id: studentId };
    if (courseId) conditions.course_id = courseId;

    // Calculate timeframe
    const now = new Date();
    let startDate;
    switch (timeframe) {
      case 'week':
        startDate = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000);
        break;
      case 'month':
        startDate = new Date(now.getTime() - 30 * 24 * 60 * 60 * 1000);
        break;
      case 'semester':
        startDate = new Date(now.getTime() - 120 * 24 * 60 * 60 * 1000);
        break;
      default:
        startDate = new Date(0);
    }

    if (timeframe !== 'all') {
      conditions.timestamp = { $gte: startDate };
    }

    // Get progress data
    const progressData = await req.db.findMany('student_progress', {
      where: conditions,
      orderBy: { timestamp: 'ASC' }
    });

    // Calculate analytics
    const analytics = {
      totalTimeSpent: progressData.reduce((sum, p) => sum + (p.time_spent_minutes || 0), 0),
      completedModules: progressData.filter(p => p.completed).length,
      activitiesCompleted: progressData.length,
      averageSessionTime: progressData.length > 0 
        ? progressData.reduce((sum, p) => sum + (p.time_spent_minutes || 0), 0) / progressData.length 
        : 0,
      progressByDay: aggregateProgressByDay(progressData),
      activityBreakdown: getActivityBreakdown(progressData)
    };

    res.json(analytics);

  } catch (error) {
    console.error('Get student analytics error:', error);
    res.status(500).json({ error: 'Failed to fetch analytics' });
  }
});

// Get course analytics
router.get('/course/:courseId', async (req, res) => {
  try {
    const { courseId } = req.params;
    const user = req.user;

    // Check permissions
    if (!['instructor', 'administrator'].includes(user.role)) {
      return res.status(403).json({ error: 'Permission denied' });
    }

    // Verify course ownership
    const course = await req.db.findOne('courses', {
      where: { id: courseId }
    });

    if (!course) {
      return res.status(404).json({ error: 'Course not found' });
    }

    if (user.role === 'instructor' && course.instructor_id !== user.userId) {
      return res.status(403).json({ error: 'Permission denied' });
    }

    // Get course statistics
    const [
      totalEnrollments,
      activeEnrollments,
      completedEnrollments,
      assignments,
      submissions,
      averageGrade
    ] = await Promise.all([
      req.db.count('course_enrollments', { where: { course_id: courseId } }),
      req.db.count('course_enrollments', { where: { course_id: courseId, status: 'active' } }),
      req.db.count('course_enrollments', { where: { course_id: courseId, status: 'completed' } }),
      req.db.count('assignments', { where: { course_id: courseId } }),
      req.db.count('assignment_submissions', { where: { assignment_id: { $in: await req.db.findMany('assignments', { where: { course_id: courseId }, select: ['id'] }).then(a => a.map(x => x.id)) } } }),
      req.db.avg('assignment_submissions', 'score', { 
        where: { 
          assignment_id: { $in: await req.db.findMany('assignments', { where: { course_id: courseId }, select: ['id'] }).then(a => a.map(x => x.id)) },
          score: { $ne: null }
        } 
      })
    ]);

    const analytics = {
      enrollmentStats: {
        total: totalEnrollments,
        active: activeEnrollments,
        completed: completedEnrollments,
        completionRate: totalEnrollments > 0 ? (completedEnrollments / totalEnrollments * 100) : 0
      },
      assignmentStats: {
        total: assignments,
        totalSubmissions: submissions,
        submissionRate: assignments > 0 ? (submissions / assignments * 100) : 0
      },
      gradeStats: {
        averageGrade: averageGrade || 0
      }
    };

    res.json(analytics);

  } catch (error) {
    console.error('Get course analytics error:', error);
    res.status(500).json({ error: 'Failed to fetch analytics' });
  }
});

// Get system-wide analytics
router.get('/system', async (req, res) => {
  try {
    const user = req.user;

    // Only administrators can view system analytics
    if (user.role !== 'administrator' && user.role !== 'super_admin') {
      return res.status(403).json({ error: 'Permission denied' });
    }

    const [
      totalUsers,
      totalCourses,
      totalEnrollments,
      totalAssignments,
      lmsIntegrations,
      systemStats
    ] = await Promise.all([
      req.db.count('users'),
      req.db.count('courses'),
      req.db.count('course_enrollments'),
      req.db.count('assignments'),
      req.db.count('lms_integrations', { where: { is_active: true } }),
      getSystemStats()
    ]);

    const analytics = {
      userStats: {
        total: totalUsers,
        byRole: await req.db.raw(`
          SELECT role, COUNT(*) as count 
          FROM users 
          GROUP BY role
        `)
      },
      courseStats: {
        total: totalCourses,
        byLevel: await req.db.raw(`
          SELECT level, COUNT(*) as count 
          FROM courses 
          GROUP BY level
        `)
      },
      enrollmentStats: {
        total: totalEnrollments
      },
      assignmentStats: {
        total: totalAssignments
      },
      integrationStats: {
        activeLMS: lmsIntegrations
      },
      systemHealth: systemStats
    };

    res.json(analytics);

  } catch (error) {
    console.error('Get system analytics error:', error);
    res.status(500).json({ error: 'Failed to fetch analytics' });
  }
});

// Helper functions
function aggregateProgressByDay(progressData) {
  const dailyData = {};
  
  progressData.forEach(progress => {
    const date = new Date(progress.timestamp).toDateString();
    if (!dailyData[date]) {
      dailyData[date] = {
        date,
        timeSpent: 0,
        activitiesCompleted: 0,
        modulesCompleted: 0
      };
    }
    
    dailyData[date].timeSpent += progress.time_spent_minutes || 0;
    dailyData[date].activitiesCompleted += 1;
    if (progress.completed) {
      dailyData[date].modulesCompleted += 1;
    }
  });
  
  return Object.values(dailyData);
}

function getActivityBreakdown(progressData) {
  const breakdown = {};
  
  progressData.forEach(progress => {
    const type = progress.activity_type;
    if (!breakdown[type]) {
      breakdown[type] = 0;
    }
    breakdown[type]++;
  });
  
  return breakdown;
}

async function getSystemStats() {
  // Mock system stats - in real implementation, gather from monitoring tools
  return {
    uptime: process.uptime(),
    memoryUsage: process.memoryUsage(),
    cpuUsage: process.cpuUsage(),
    activeConnections: Math.floor(Math.random() * 100) // Mock data
  };
}

module.exports = router;