const express = require('express');
const User = require('../models/User');
const Paper = require('../models/Paper');
const Review = require('../models/Review');
const { auth, requireRole } = require('../middleware/auth');
const Joi = require('joi');

const router = express.Router();

// Validation schemas
const updateProfileSchema = Joi.object({
  title: Joi.string().valid('Dr.', 'Prof.', 'Mr.', 'Ms.', 'Mrs.', 'Mx.'),
  affiliation: Joi.object({
    institution: Joi.string().trim(),
    department: Joi.string().trim(),
    position: Joi.string().trim(),
    orcid: Joi.string().pattern(/^\d{4}-\d{4}-\d{4}-\d{4}$/)
  }),
  researchInterests: Joi.array().items(Joi.string().trim()).max(20),
  expertise: Joi.array().items(Joi.string().trim()).max(20),
  currentResearch: Joi.string().trim(),
  publications: Joi.array().items(Joi.object({
    title: Joi.string().trim().required(),
    journal: Joi.string().trim(),
    year: Joi.number().integer().min(1900).max(2030),
    doi: Joi.string().trim(),
    url: Joi.string().uri()
  })).max(100),
  reviewPreferences: Joi.object({
    willingToReview: Joi.boolean(),
    areas: Joi.array().items(Joi.string()),
    maximumReviewsPerYear: Joi.number().min(1).max(50),
    languages: Joi.array().items(Joi.string())
  }),
  emailNotifications: Joi.object({
    paperSubmissions: Joi.boolean(),
    reviewRequests: Joi.boolean(),
    conferenceDeadlines: Joi.boolean(),
    systemUpdates: Joi.boolean()
  }),
  privacySettings: Joi.object({
    profileVisible: Joi.boolean(),
    showEmail: Joi.boolean(),
    showAffiliation: Joi.boolean()
  })
});

const updateRoleSchema = Joi.object({
  role: Joi.string().valid('researcher', 'reviewer', 'editor', 'admin').required(),
  permissions: Joi.array().items(Joi.object({
    type: Joi.string().required(),
    resource: Joi.string().required(),
    actions: Joi.array().items(Joi.string()).required()
  })),
  reviewPreferences: Joi.object({
    willingToReview: Joi.boolean(),
    areas: Joi.array().items(Joi.string()),
    maximumReviewsPerYear: Joi.number().min(1).max(50)
  })
});

// Get all users (Admin/Editor only)
router.get('/', auth, requireRole(['editor', 'admin']), async (req, res) => {
  try {
    const page = parseInt(req.query.page) || 1;
    const limit = parseInt(req.query.limit) || 20;
    const skip = (page - 1) * limit;

    const { 
      role, 
      search, 
      affiliation,
      willingToReview,
      verified,
      sortBy = 'createdAt',
      sortOrder = 'desc'
    } = req.query;

    // Build search query
    const query = { isActive: true };

    if (role) {
      query.role = role;
    }

    if (affiliation) {
      query['affiliation.institution'] = { $regex: affiliation, $options: 'i' };
    }

    if (willingToReview !== undefined) {
      query['reviewPreferences.willingToReview'] = willingToReview === 'true';
    }

    if (verified !== undefined) {
      query.isVerified = verified === 'true';
    }

    if (search) {
      query.$or = [
        { firstName: { $regex: search, $options: 'i' } },
        { lastName: { $regex: search, $options: 'i' } },
        { email: { $regex: search, $options: 'i' } },
        { 'affiliation.institution': { $regex: search, $options: 'i' } }
      ];
    }

    // Build sort object
    const sortObj = {};
    sortObj[sortBy] = sortOrder === 'desc' ? -1 : 1;

    const users = await User.find(query)
      .select('-password') // Exclude password from results
      .sort(sortObj)
      .skip(skip)
      .limit(limit);

    const total = await User.countDocuments(query);

    res.json({
      users: users.map(user => ({
        id: user._id,
        email: user.email,
        firstName: user.firstName,
        lastName: user.lastName,
        fullName: user.fullName,
        title: user.title,
        role: user.role,
        affiliation: {
          institution: user.affiliation?.institution,
          department: user.affiliation?.department,
          position: user.affiliation?.position,
          orcid: user.affiliation?.orcid
        },
        researchInterests: user.researchInterests,
        reviewPreferences: {
          willingToReview: user.reviewPreferences.willingToReview,
          areas: user.reviewPreferences.areas,
          maximumReviewsPerYear: user.reviewPreferences.maximumReviewsPerYear
        },
        isVerified: user.isVerified,
        loginCount: user.loginCount,
        lastLogin: user.lastLogin,
        createdAt: user.createdAt
      })),
      pagination: {
        current: page,
        total: Math.ceil(total / limit),
        count: users.length,
        totalRecords: total
      }
    });

  } catch (error) {
    console.error('Get users error:', error);
    res.status(500).json({ 
      error: 'Failed to fetch users',
      message: 'An error occurred while fetching users'
    });
  }
});

// Get user profile by ID
router.get('/:id', auth, async (req, res) => {
  try {
    const user = await User.findById(req.params.id)
      .select('-password');

    if (!user) {
      return res.status(404).json({ error: 'User not found' });
    }

    // Check access permissions
    const isOwner = user._id.toString() === req.user.userId;
    const isEditor = req.user.role === 'editor' || req.user.role === 'admin';
    const canView = isOwner || isEditor || user.privacySettings.profileVisible;

    if (!canView) {
      return res.status(403).json({ error: 'Access denied' });
    }

    // Get user statistics
    const stats = await user.getStats();

    res.json({
      user: {
        id: user._id,
        email: isOwner || isEditor ? user.email : undefined,
        firstName: user.firstName,
        lastName: user.lastName,
        fullName: user.fullName,
        title: user.title,
        role: user.role,
        affiliation: user.privacySettings.showAffiliation || isOwner || isEditor ? user.affiliation : undefined,
        researchInterests: user.researchInterests,
        expertise: user.expertise,
        currentResearch: user.currentResearch,
        publications: user.publications,
        reviewPreferences: isOwner || isEditor ? user.reviewPreferences : {
          willingToReview: user.reviewPreferences.willingToReview,
          areas: user.reviewPreferences.areas
        },
        isVerified: user.isVerified,
        stats,
        createdAt: user.createdAt
      }
    });

  } catch (error) {
    console.error('Get user profile error:', error);
    res.status(500).json({ 
      error: 'Failed to fetch user profile',
      message: 'An error occurred while fetching the user profile'
    });
  }
});

// Search researchers
router.get('/search/researchers', auth, async (req, res) => {
  try {
    const page = parseInt(req.query.page) || 1;
    const limit = parseInt(req.query.limit) || 20;
    const skip = (page - 1) * limit;

    const { 
      query,
      researchArea,
      institution,
      expertise,
      willingToReview,
      hasORCID,
      sortBy = 'lastName',
      sortOrder = 'asc'
    } = req.query;

    // Build search query
    const searchQuery = { 
      isActive: true,
      privacySettings: { $ne: { profileVisible: false } }
    };

    if (query) {
      searchQuery.$or = [
        { firstName: { $regex: query, $options: 'i' } },
        { lastName: { $regex: query, $options: 'i' } },
        { 'affiliation.institution': { $regex: query, $options: 'i' } },
        { researchInterests: { $regex: query, $options: 'i' } }
      ];
    }

    if (researchArea) {
      searchQuery.researchInterests = { $regex: researchArea, $options: 'i' };
    }

    if (institution) {
      searchQuery['affiliation.institution'] = { $regex: institution, $options: 'i' };
    }

    if (expertise) {
      searchQuery.expertise = { $regex: expertise, $options: 'i' };
    }

    if (willingToReview === 'true') {
      searchQuery['reviewPreferences.willingToReview'] = true;
    }

    if (hasORCID === 'true') {
      searchQuery['affiliation.orcid'] = { $exists: true, $ne: null };
    }

    // Build sort object
    const sortObj = {};
    sortObj[sortBy] = sortOrder === 'desc' ? -1 : 1;

    const researchers = await User.find(searchQuery)
      .select('firstName lastName affiliation researchInterests expertise reviewPreferences isVerified createdAt')
      .sort(sortObj)
      .skip(skip)
      .limit(limit);

    const total = await User.countDocuments(searchQuery);

    res.json({
      researchers: researchers.map(researcher => ({
        id: researcher._id,
        name: researcher.fullName,
        affiliation: researcher.affiliation,
        researchInterests: researcher.researchInterests,
        expertise: researcher.expertise,
        reviewPreferences: {
          willingToReview: researcher.reviewPreferences.willingToReview,
          areas: researcher.reviewPreferences.areas
        },
        isVerified: researcher.isVerified,
        hasORCID: researcher.affiliation?.orcid ? true : false,
        hasProfileImage: false // Would be implemented if profile images are added
      })),
      pagination: {
        current: page,
        total: Math.ceil(total / limit),
        count: researchers.length,
        totalRecords: total
      }
    });

  } catch (error) {
    console.error('Search researchers error:', error);
    res.status(500).json({ 
      error: 'Failed to search researchers',
      message: 'An error occurred while searching researchers'
    });
  }
});

// Get available reviewers for a research area
router.get('/reviewers/available', auth, requireRole(['editor', 'admin']), async (req, res) => {
  try {
    const { researchArea, expertise, maxReviews = 3 } = req.query;

    if (!researchArea) {
      return res.status(400).json({ 
        error: 'Missing research area',
        message: 'Research area is required to find available reviewers'
      });
    }

    // Build query for available reviewers
    const query = {
      isActive: true,
      isVerified: true,
      role: { $in: ['researcher', 'reviewer', 'editor', 'admin'] },
      'reviewPreferences.willingToReview': true,
      'reviewPreferences.areas': { $regex: researchArea, $options: 'i' }
    };

    // If expertise is specified, prioritize reviewers with matching expertise
    let reviewers = await User.find(query)
      .select('firstName lastName affiliation expertise reviewPreferences')
      .lean();

    // Calculate current review load for each reviewer
    const reviewerLoad = await Review.aggregate([
      {
        $match: {
          status: { $in: ['assigned', 'in_progress'] },
          dueDate: { $gte: new Date() }
        }
      },
      {
        $group: {
          _id: '$reviewer',
          currentReviews: { $sum: 1 }
        }
      }
    ]);

    // Add current review count to reviewer data
    reviewers = reviewers.map(reviewer => {
      const load = reviewerLoad.find(l => l._id.toString() === reviewer._id.toString());
      return {
        ...reviewer,
        currentReviews: load ? load.currentReviews : 0,
        availableSlots: Math.max(0, reviewer.reviewPreferences.maximumReviewsPerYear - 
                                (load ? load.currentReviews : 0))
      };
    });

    // Filter by availability and expertise
    const availableReviewers = reviewers
      .filter(reviewer => reviewer.availableSlots > 0)
      .filter(reviewer => {
        if (!expertise) return true;
        return reviewer.expertise.some(exp => 
          exp.toLowerCase().includes(expertise.toLowerCase())
        );
      })
      .sort((a, b) => {
        // Sort by availability (more available first), then by verification status
        if (b.availableSlots !== a.availableSlots) {
          return b.availableSlots - a.availableSlots;
        }
        return b.isVerified - a.isVerified;
      })
      .slice(0, maxReviews);

    res.json({
      researchArea,
      availableReviewers: availableReviewers.map(reviewer => ({
        id: reviewer._id,
        name: reviewer.fullName,
        affiliation: reviewer.affiliation,
        expertise: reviewer.expertise,
        currentReviews: reviewer.currentReviews,
        availableSlots: reviewer.availableSlots,
        maximumReviews: reviewer.reviewPreferences.maximumReviewsPerYear,
        reviewAreas: reviewer.reviewPreferences.areas
      }))
    });

  } catch (error) {
    console.error('Get available reviewers error:', error);
    res.status(500).json({ 
      error: 'Failed to fetch available reviewers',
      message: 'An error occurred while fetching available reviewers'
    });
  }
});

// Update user profile
router.put('/profile', auth, async (req, res) => {
  try {
    const user = await User.findById(req.user.userId);
    if (!user) {
      return res.status(404).json({ error: 'User not found' });
    }

    // Validate input
    const { error, value } = updateProfileSchema.validate(req.body);
    if (error) {
      return res.status(400).json({ 
        error: 'Validation failed',
        details: error.details.map(detail => detail.message)
      });
    }

    // Update user fields
    Object.keys(value).forEach(key => {
      if (value[key] !== undefined) {
        user[key] = value[key];
      }
    });

    await user.save();

    // Get updated user statistics
    const stats = await user.getStats();

    res.json({
      message: 'Profile updated successfully',
      user: {
        id: user._id,
        email: user.email,
        firstName: user.firstName,
        lastName: user.lastName,
        fullName: user.fullName,
        title: user.title,
        affiliation: user.affiliation,
        researchInterests: user.researchInterests,
        expertise: user.expertise,
        currentResearch: user.currentResearch,
        publications: user.publications,
        reviewPreferences: user.reviewPreferences,
        emailNotifications: user.emailNotifications,
        privacySettings: user.privacySettings,
        isVerified: user.isVerified
      },
      stats
    });

  } catch (error) {
    console.error('Update profile error:', error);
    res.status(500).json({ 
      error: 'Failed to update profile',
      message: 'An error occurred while updating the profile'
    });
  }
});

// Update user role and permissions (Admin only)
router.put('/:id/role', auth, requireRole(['admin']), async (req, res) => {
  try {
    const user = await User.findById(req.params.id);
    if (!user) {
      return res.status(404).json({ error: 'User not found' });
    }

    // Validate input
    const { error, value } = updateRoleSchema.validate(req.body);
    if (error) {
      return res.status(400).json({ 
        error: 'Validation failed',
        details: error.details.map(detail => detail.message)
      });
    }

    // Update role and related fields
    user.role = value.role;
    if (value.permissions) {
      user.permissions = value.permissions;
    }
    if (value.reviewPreferences) {
      user.reviewPreferences = { ...user.reviewPreferences, ...value.reviewPreferences };
    }

    await user.save();

    res.json({
      message: 'User role updated successfully',
      user: {
        id: user._id,
        email: user.email,
        firstName: user.firstName,
        lastName: user.lastName,
        fullName: user.fullName,
        role: user.role,
        permissions: user.permissions,
        reviewPreferences: user.reviewPreferences
      }
    });

  } catch (error) {
    console.error('Update user role error:', error);
    res.status(500).json({ 
      error: 'Failed to update user role',
      message: 'An error occurred while updating the user role'
    });
  }
});

// Verify user (Admin/Editor only)
router.post('/:id/verify', auth, requireRole(['editor', 'admin']), async (req, res) => {
  try {
    const user = await User.findById(req.params.id);
    if (!user) {
      return res.status(404).json({ error: 'User not found' });
    }

    user.isVerified = true;
    await user.save();

    res.json({
      message: 'User verified successfully',
      user: {
        id: user._id,
        email: user.email,
        firstName: user.firstName,
        lastName: user.lastName,
        isVerified: user.isVerified
      }
    });

  } catch (error) {
    console.error('Verify user error:', error);
    res.status(500).json({ 
      error: 'Failed to verify user',
      message: 'An error occurred while verifying the user'
    });
  }
});

// Deactivate user (Admin only)
router.post('/:id/deactivate', auth, requireRole(['admin']), async (req, res) => {
  try {
    const { reason } = req.body;

    const user = await User.findById(req.params.id);
    if (!user) {
      return res.status(404).json({ error: 'User not found' });
    }

    // Prevent self-deactivation
    if (user._id.toString() === req.user.userId) {
      return res.status(400).json({ 
        error: 'Cannot deactivate own account',
        message: 'You cannot deactivate your own account'
      });
    }

    user.isActive = false;
    await user.save();

    // TODO: Send notification email about deactivation

    res.json({
      message: 'User deactivated successfully',
      user: {
        id: user._id,
        email: user.email,
        firstName: user.firstName,
        lastName: user.lastName,
        isActive: user.isActive,
        reason: reason || 'Administrative action'
      }
    });

  } catch (error) {
    console.error('Deactivate user error:', error);
    res.status(500).json({ 
      error: 'Failed to deactivate user',
      message: 'An error occurred while deactivating the user'
    });
  }
});

// Get user statistics
router.get('/:id/stats', auth, async (req, res) => {
  try {
    const user = await User.findById(req.params.id);
    if (!user) {
      return res.status(404).json({ error: 'User not found' });
    }

    // Check access permissions
    const isOwner = user._id.toString() === req.user.userId;
    const isEditor = req.user.role === 'editor' || req.user.role === 'admin';

    if (!isOwner && !isEditor) {
      return res.status(403).json({ error: 'Access denied' });
    }

    const stats = await user.getStats();

    // Additional detailed statistics
    const detailedStats = await Promise.all([
      // Paper statistics by status
      Paper.aggregate([
        { $match: { createdBy: user._id } },
        {
          $group: {
            _id: '$status',
            count: { $sum: 1 }
          }
        }
      ]),
      
      // Review statistics by status
      Review.aggregate([
        { $match: { reviewer: user._id } },
        {
          $group: {
            _id: '$status',
            count: { $sum: 1 }
          }
        }
      ]),
      
      // Citation statistics
      Paper.aggregate([
        { $match: { createdBy: user._id } },
        { $group: { _id: null, totalCitations: { $sum: '$metrics.citations' } } }
      ])
    ]);

    const [paperStats, reviewStats, citationStats] = detailedStats;

    res.json({
      user: {
        id: user._id,
        name: user.fullName,
        role: user.role
      },
      statistics: {
        papers: {
          ...stats,
          byStatus: paperStats.reduce((acc, stat) => {
            acc[stat._id] = stat.count;
            return acc;
          }, {})
        },
        reviews: {
          total: stats.reviewsGiven,
          completed: stats.reviewsReceived,
          averageRating: stats.averageRating,
          byStatus: reviewStats.reduce((acc, stat) => {
            acc[stat._id] = stat.count;
            return acc;
          }, {})
        },
        impact: {
          totalCitations: citationStats[0]?.totalCitations || 0,
          hIndex: stats.hIndex || 0,
          i10Index: stats.i10Index || 0
        },
        activity: {
          memberSince: user.createdAt,
          lastLogin: user.lastLogin,
          totalLoginCount: user.loginCount
        }
      }
    });

  } catch (error) {
    console.error('Get user stats error:', error);
    res.status(500).json({ 
      error: 'Failed to fetch user statistics',
      message: 'An error occurred while fetching user statistics'
    });
  }
});

// Get top researchers
router.get('/rankings/top-researchers', auth, async (req, res) => {
  try {
    const { 
      metric = 'citations',
      researchArea,
      limit = 10,
      timeRange = '1year'
    } = req.query;

    // Calculate time filter
    const now = new Date();
    let dateFilter = {};
    
    switch (timeRange) {
      case '6months':
        dateFilter = { $gte: new Date(now.getFullYear(), now.getMonth() - 6, now.getDate()) };
        break;
      case '1year':
      default:
        dateFilter = { $gte: new Date(now.getFullYear() - 1, now.getMonth(), now.getDate()) };
        break;
    }

    // Build aggregation pipeline based on metric
    let pipeline = [];

    if (metric === 'citations') {
      pipeline = [
        { $match: { isActive: true, isVerified: true } },
        {
          $lookup: {
            from: 'papers',
            localField: '_id',
            foreignField: 'createdBy',
            as: 'papers'
          }
        },
        { $unwind: { path: '$papers', preserveNullAndEmptyArrays: true } },
        {
          $group: {
            _id: '$_id',
            user: { $first: '$$ROOT' },
            totalCitations: { $sum: '$papers.metrics.citations' },
            paperCount: { $sum: 1 }
          }
        },
        { $sort: { totalCitations: -1 } },
        { $limit: parseInt(limit) }
      ];
    } else if (metric === 'publications') {
      pipeline = [
        { $match: { isActive: true } },
        {
          $lookup: {
            from: 'papers',
            localField: '_id',
            foreignField: 'createdBy',
            as: 'papers'
          }
        },
        {
          $addFields: {
            paperCount: { $size: '$papers' }
          }
        },
        { $sort: { paperCount: -1 } },
        { $limit: parseInt(limit) }
      ];
    } else if (metric === 'reviews') {
      pipeline = [
        { $match: { isActive: true, isVerified: true } },
        {
          $lookup: {
            from: 'reviews',
            localField: '_id',
            foreignField: 'reviewer',
            as: 'reviews'
          }
        },
        {
          $addFields: {
            reviewCount: { $size: '$reviews' }
          }
        },
        { $sort: { reviewCount: -1 } },
        { $limit: parseInt(limit) }
      ];
    }

    // Add research area filter if specified
    if (researchArea) {
      pipeline.splice(1, 0, {
        $match: {
          researchInterests: { $regex: researchArea, $options: 'i' }
        }
      });
    }

    const topResearchers = await User.aggregate(pipeline);

    res.json({
      metric,
      timeRange,
      researchArea: researchArea || 'All',
      rankings: topResearchers.map((item, index) => {
        const user = item.user;
        return {
          rank: index + 1,
          id: user._id,
          name: user.fullName,
          affiliation: user.affiliation?.institution,
          orcid: user.affiliation?.orcid,
          score: item.totalCitations || item.paperCount || item.reviewCount,
          additionalData: {
            totalCitations: item.totalCitations || 0,
            paperCount: item.paperCount || 0,
            reviewCount: item.reviewCount || 0
          }
        };
      })
    });

  } catch (error) {
    console.error('Get top researchers error:', error);
    res.status(500).json({ 
      error: 'Failed to fetch top researchers',
      message: 'An error occurred while fetching rankings'
    });
  }
});

module.exports = router;