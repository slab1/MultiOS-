const express = require('express');
const Review = require('../models/Review');
const Paper = require('../models/Paper');
const User = require('../models/User');
const { auth, requireRole } = require('../middleware/auth');
const Joi = require('joi');

const router = express.Router();

// Validation schemas
const createReviewSchema = Joi.object({
  paperId: Joi.string().required(),
  dueDate: Joi.date().required(),
  isBlind: Joi.boolean().default(true)
});

const submitReviewSchema = Joi.object({
  rating: Joi.object({
    originality: Joi.object({
      score: Joi.number().min(1).max(5).required(),
      comment: Joi.string()
    }).required(),
    significance: Joi.object({
      score: Joi.number().min(1).max(5).required(),
      comment: Joi.string()
    }).required(),
    technicalQuality: Joi.object({
      score: Joi.number().min(1).max(5).required(),
      comment: Joi.string()
    }).required(),
    clarity: Joi.object({
      score: Joi.number().min(1).max(5).required(),
      comment: Joi.string()
    }).required(),
    overall: Joi.object({
      score: Joi.number().min(1).max(5).required(),
      comment: Joi.string()
    }).required()
  }).required(),
  reviewText: Joi.object({
    summary: Joi.string().required(),
    strengths: Joi.string().allow(''),
    weaknesses: Joi.string().allow(''),
    detailedComments: Joi.string().allow(''),
    confidentialComments: Joi.string().allow('')
  }).required(),
  technicalReview: Joi.object({
    methodology: Joi.string().allow(''),
    experiments: Joi.string().allow(''),
    results: Joi.string().allow(''),
    relatedWork: Joi.string().allow(''),
    correctness: Joi.string().allow(''),
    reproducibility: Joi.string().allow('')
  }),
  presentationReview: Joi.object({
    structure: Joi.string().allow(''),
    figures: Joi.string().allow(''),
    tables: Joi.string().allow(''),
    references: Joi.string().allow(''),
    grammar: Joi.string().allow(''),
    formatting: Joi.string().allow('')
  }),
  recommendation: Joi.object({
    decision: Joi.string().valid('accept', 'minor_revision', 'major_revision', 'reject').required(),
    confidence: Joi.number().min(1).max(3).default(2),
    rationale: Joi.string().allow('')
  }).required()
});

const updateReviewSchema = Joi.object({
  rating: Joi.object({
    originality: Joi.object({
      score: Joi.number().min(1).max(5),
      comment: Joi.string()
    }),
    significance: Joi.object({
      score: Joi.number().min(1).max(5),
      comment: Joi.string()
    }),
    technicalQuality: Joi.object({
      score: Joi.number().min(1).max(5),
      comment: Joi.string()
    }),
    clarity: Joi.object({
      score: Joi.number().min(1).max(5),
      comment: Joi.string()
    }),
    overall: Joi.object({
      score: Joi.number().min(1).max(5),
      comment: Joi.string()
    })
  }),
  reviewText: Joi.object({
    summary: Joi.string(),
    strengths: Joi.string(),
    weaknesses: Joi.string(),
    detailedComments: Joi.string(),
    confidentialComments: Joi.string()
  }),
  recommendation: Joi.object({
    decision: Joi.string().valid('accept', 'minor_revision', 'major_revision', 'reject'),
    confidence: Joi.number().min(1).max(3),
    rationale: Joi.string()
  })
});

// Assign review to reviewer
router.post('/assign', auth, requireRole(['editor', 'admin']), async (req, res) => {
  try {
    const { paperId, reviewerId, dueDate, blindReview = true } = req.body;

    // Validate inputs
    if (!paperId || !reviewerId || !dueDate) {
      return res.status(400).json({ 
        error: 'Missing required fields',
        message: 'Paper ID, reviewer ID, and due date are required'
      });
    }

    // Check if paper exists
    const paper = await Paper.findById(paperId);
    if (!paper) {
      return res.status(404).json({ error: 'Paper not found' });
    }

    // Check if reviewer exists
    const reviewer = await User.findById(reviewerId);
    if (!reviewer) {
      return res.status(404).json({ error: 'Reviewer not found' });
    }

    // Check if reviewer is willing to review
    if (!reviewer.reviewPreferences.willingToReview) {
      return res.status(400).json({ 
        error: 'Reviewer not available',
        message: 'This reviewer is not currently accepting review assignments'
      });
    }

    // Check if reviewer has expertise in the research area
    if (!reviewer.canReviewInArea(paper.researchArea)) {
      return res.status(400).json({ 
        error: 'Insufficient expertise',
        message: 'Reviewer does not have expertise in this research area'
      });
    }

    // Check if reviewer already assigned
    const existingAssignment = paper.assignedReviewers.find(
      assignment => assignment.reviewer.toString() === reviewerId
    );

    if (existingAssignment) {
      return res.status(400).json({ 
        error: 'Already assigned',
        message: 'This reviewer is already assigned to this paper'
      });
    }

    // Assign reviewer to paper
    await paper.assignReviewer(reviewerId, new Date(dueDate), blindReview);

    // Create review record
    const review = new Review({
      paper: paperId,
      reviewer: reviewerId,
      assignedBy: req.user.userId,
      dueDate: new Date(dueDate),
      isBlind: blindReview,
      cycle: paper.reviewCycle.current + 1
    });

    await review.save();

    // TODO: Send notification email to reviewer

    res.status(201).json({
      message: 'Reviewer assigned successfully',
      review: {
        id: review._id,
        paper: {
          id: paper._id,
          title: paper.title
        },
        reviewer: {
          id: reviewer._id,
          name: reviewer.fullName,
          email: reviewer.email
        },
        dueDate: review.dueDate,
        status: review.status
      }
    });

  } catch (error) {
    console.error('Assign review error:', error);
    res.status(500).json({ 
      error: 'Failed to assign review',
      message: 'An error occurred while assigning the review'
    });
  }
});

// Get reviews assigned to current user
router.get('/my-assignments', auth, async (req, res) => {
  try {
    const page = parseInt(req.query.page) || 1;
    const limit = parseInt(req.query.limit) || 10;
    const skip = (page - 1) * limit;
    const status = req.query.status; // assigned, in_progress, completed, late

    // Build query
    const query = { reviewer: req.user.userId };
    if (status) {
      query.status = status;
    }

    const reviews = await Review.find(query)
      .populate({
        path: 'paper',
        select: 'title abstract authors researchArea submissionTarget',
        populate: [
          { path: 'authors.user', select: 'firstName lastName affiliation' },
          { path: 'correspondingAuthor', select: 'firstName lastName' },
          { path: 'submissionTarget.conference', select: 'name shortName' }
        ]
      })
      .sort({ dueDate: 1 })
      .skip(skip)
      .limit(limit);

    const total = await Review.countDocuments(query);

    res.json({
      reviews: reviews.map(review => ({
        id: review._id,
        paper: {
          id: review.paper._id,
          title: review.paper.title,
          abstract: review.paper.abstract.substring(0, 200) + '...',
          authors: review.paper.authorNames,
          researchArea: review.paper.researchArea,
          conference: review.paper.submissionTarget?.conference?.name
        },
        assignmentDate: review.assignmentDate,
        dueDate: review.dueDate,
        completedDate: review.completedDate,
        status: review.status,
        cycle: review.cycle,
        daysUntilDue: review.daysUntilDue,
        completionTime: review.completionTimeInDays,
        averageRating: review.averageRating,
        recommendation: review.recommendation
      })),
      pagination: {
        current: page,
        total: Math.ceil(total / limit),
        count: reviews.length,
        totalRecords: total
      }
    });

  } catch (error) {
    console.error('Get my assignments error:', error);
    res.status(500).json({ 
      error: 'Failed to fetch assignments',
      message: 'An error occurred while fetching review assignments'
    });
  }
});

// Get single review
router.get('/:id', auth, async (req, res) => {
  try {
    const review = await Review.findById(req.params.id)
      .populate('paper', 'title abstract authors researchArea status createdBy')
      .populate('reviewer', 'firstName lastName email affiliation')
      .populate('assignedBy', 'firstName lastName email');

    if (!review) {
      return res.status(404).json({ error: 'Review not found' });
    }

    // Check access permissions
    const isReviewer = review.reviewer._id.toString() === req.user.userId;
    const isPaperOwner = review.paper.createdBy.toString() === req.user.userId;
    const isAssignedEditor = review.assignedBy && review.assignedBy._id.toString() === req.user.userId;
    const isEditor = req.user.role === 'editor' || req.user.role === 'admin';

    // Reviewer can see their own review
    // Paper owner can see reviews after completion
    // Assigned editor can see the review
    // Other editors can see reviews
    const canView = isReviewer || (isPaperOwner && review.status === 'completed') || 
                   isAssignedEditor || isEditor;

    if (!canView) {
      return res.status(403).json({ error: 'Access denied' });
    }

    // Update metrics
    review.metrics.lastAccessed = new Date();
    await review.save();

    res.json({
      review: {
        id: review._id,
        paper: {
          id: review.paper._id,
          title: review.paper.title,
          abstract: review.paper.abstract,
          authors: review.paper.authorNames,
          researchArea: review.paper.researchArea
        },
        reviewer: isEditor || isPaperOwner || isAssignedEditor ? 
          { name: review.reviewer.fullName, affiliation: review.reviewer.affiliation } : 
          review.anonymousIdentity,
        status: review.status,
        cycle: review.cycle,
        assignmentDate: review.assignmentDate,
        dueDate: review.dueDate,
        completedDate: review.completedDate,
        isBlind: review.isBlind,
        rating: review.rating,
        reviewText: review.reviewText,
        technicalReview: review.technicalReview,
        presentationReview: review.presentationReview,
        recommendation: review.recommendation,
        averageRating: review.averageRating,
        timeSpent: review.metrics.timeSpent,
        sessionCount: review.metrics.sessionCount
      }
    });

  } catch (error) {
    console.error('Get review error:', error);
    res.status(500).json({ 
      error: 'Failed to fetch review',
      message: 'An error occurred while fetching the review'
    });
  }
});

// Update review (draft mode)
router.put('/:id', auth, async (req, res) => {
  try {
    const review = await Review.findById(req.params.id);

    if (!review) {
      return res.status(404).json({ error: 'Review not found' });
    }

    // Check if user is the reviewer
    if (review.reviewer.toString() !== req.user.userId) {
      return res.status(403).json({ error: 'Access denied' });
    }

    // Check if review is still in progress
    if (review.status === 'completed') {
      return res.status(400).json({ 
        error: 'Review already submitted',
        message: 'Completed reviews cannot be edited'
      });
    }

    // Update status
    review.status = 'in_progress';

    // Update review fields
    const { error, value } = updateReviewSchema.validate(req.body);
    if (error) {
      return res.status(400).json({ 
        error: 'Validation failed',
        details: error.details.map(detail => detail.message)
      });
    }

    // Merge nested objects
    if (value.rating) {
      Object.keys(value.rating).forEach(category => {
        if (value.rating[category]) {
          Object.keys(value.rating[category]).forEach(field => {
            if (value.rating[category][field] !== undefined) {
              review.rating[category][field] = value.rating[category][field];
            }
          });
        }
      });
    }

    if (value.reviewText) {
      Object.keys(value.reviewText).forEach(field => {
        if (value.reviewText[field] !== undefined) {
          review.reviewText[field] = value.reviewText[field];
        }
      });
    }

    if (value.recommendation) {
      Object.keys(value.recommendation).forEach(field => {
        if (value.recommendation[field] !== undefined) {
          review.recommendation[field] = value.recommendation[field];
        }
      });
    }

    await review.save();

    res.json({
      message: 'Review updated successfully',
      review: {
        id: review._id,
        status: review.status,
        lastModified: review.lastModified
      }
    });

  } catch (error) {
    console.error('Update review error:', error);
    res.status(500).json({ 
      error: 'Failed to update review',
      message: 'An error occurred while updating the review'
    });
  }
});

// Submit review
router.post('/:id/submit', auth, async (req, res) => {
  try {
    const review = await Review.findById(req.params.id);

    if (!review) {
      return res.status(404).json({ error: 'Review not found' });
    }

    // Check if user is the reviewer
    if (review.reviewer.toString() !== req.user.userId) {
      return res.status(403).json({ error: 'Access denied' });
    }

    // Validate complete review data
    const { error, value } = submitReviewSchema.validate(req.body);
    if (error) {
      return res.status(400).json({ 
        error: 'Validation failed',
        details: error.details.map(detail => detail.message)
      });
    }

    // Update review with all data
    review.rating = value.rating;
    review.reviewText = value.reviewText;
    review.technicalReview = value.technicalReview || {};
    review.presentationReview = value.presentationReview || {};
    review.recommendation = value.recommendation;

    // Submit review
    await review.submitReview();

    // Update paper review progress
    const paper = await Paper.findById(review.paper);
    await paper.addReview(review._id);

    // TODO: Send notifications

    res.json({
      message: 'Review submitted successfully',
      review: {
        id: review._id,
        status: review.status,
        completedDate: review.completedDate,
        recommendation: review.recommendation,
        averageRating: review.averageRating
      }
    });

  } catch (error) {
    console.error('Submit review error:', error);
    res.status(500).json({ 
      error: 'Failed to submit review',
      message: 'An error occurred while submitting the review'
    });
  }
});

// Add comment to review
router.post('/:id/comment', auth, async (req, res) => {
  try {
    const { section, comment, pageNumber } = req.body;

    if (!section || !comment) {
      return res.status(400).json({ 
        error: 'Missing required fields',
        message: 'Section and comment are required'
      });
    }

    const review = await Review.findById(req.params.id);

    if (!review) {
      return res.status(404).json({ error: 'Review not found' });
    }

    // Check if user is the reviewer
    if (review.reviewer.toString() !== req.user.userId) {
      return res.status(403).json({ error: 'Access denied' });
    }

    // Add comment
    await review.addComment(section, comment, pageNumber);

    res.json({ message: 'Comment added successfully' });

  } catch (error) {
    console.error('Add comment error:', error);
    res.status(500).json({ 
      error: 'Failed to add comment',
      message: 'An error occurred while adding the comment'
    });
  }
});

// Get overdue reviews
router.get('/overdue/list', auth, requireRole(['editor', 'admin']), async (req, res) => {
  try {
    const overdueReviews = await Review.findOverdue()
      .populate('reviewer', 'firstName lastName email')
      .populate('paper', 'title authors researchArea dueDate');

    res.json({
      overdueReviews: overdueReviews.map(review => ({
        id: review._id,
        paper: {
          id: review.paper._id,
          title: review.paper.title,
          authors: review.paper.authorNames
        },
        reviewer: review.reviewer,
        dueDate: review.dueDate,
        daysOverdue: Math.floor((new Date() - review.dueDate) / (1000 * 60 * 60 * 24)),
        status: review.status
      }))
    });

  } catch (error) {
    console.error('Get overdue reviews error:', error);
    res.status(500).json({ 
      error: 'Failed to fetch overdue reviews',
      message: 'An error occurred while fetching overdue reviews'
    });
  }
});

// Get reviewer statistics
router.get('/stats/reviewer', auth, async (req, res) => {
  try {
    const stats = await Review.getReviewerStats(req.user.userId);

    if (stats.length === 0) {
      return res.json({
        stats: {
          totalReviews: 0,
          completedReviews: 0,
          averageTime: 0,
          averageRating: 0
        }
      });
    }

    const reviewerStats = stats[0];
    
    res.json({
      stats: {
        totalReviews: reviewerStats.totalReviews || 0,
        completedReviews: reviewerStats.completedReviews || 0,
        averageTime: Math.round(reviewerStats.averageTime || 0),
        averageRating: Math.round((reviewerStats.averageRating || 0) * 100) / 100
      }
    });

  } catch (error) {
    console.error('Get reviewer stats error:', error);
    res.status(500).json({ 
      error: 'Failed to fetch reviewer statistics',
      message: 'An error occurred while fetching statistics'
    });
  }
});

// Get overall review quality metrics
router.get('/stats/quality', auth, requireRole(['editor', 'admin']), async (req, res) => {
  try {
    const metrics = await Review.getQualityMetrics();

    if (metrics.length === 0) {
      return res.json({
        metrics: {
          totalReviews: 0,
          averageOverallRating: 0,
          averageOriginality: 0,
          averageSignificance: 0,
          averageTechnicalQuality: 0,
          averageClarity: 0
        }
      });
    }

    const qualityMetrics = metrics[0];

    res.json({
      metrics: {
        totalReviews: qualityMetrics.totalReviews || 0,
        averageOverallRating: Math.round((qualityMetrics.averageOverallRating || 0) * 100) / 100,
        averageOriginality: Math.round((qualityMetrics.averageOriginality || 0) * 100) / 100,
        averageSignificance: Math.round((qualityMetrics.averageSignificance || 0) * 100) / 100,
        averageTechnicalQuality: Math.round((qualityMetrics.averageTechnicalQuality || 0) * 100) / 100,
        averageClarity: Math.round((qualityMetrics.averageClarity || 0) * 100) / 100
      }
    });

  } catch (error) {
    console.error('Get quality metrics error:', error);
    res.status(500).json({ 
      error: 'Failed to fetch quality metrics',
      message: 'An error occurred while fetching quality metrics'
    });
  }
});

// Decline review assignment
router.post('/:id/decline', auth, async (req, res) => {
  try {
    const { reason } = req.body;

    const review = await Review.findById(req.params.id);

    if (!review) {
      return res.status(404).json({ error: 'Review not found' });
    }

    // Check if user is the reviewer
    if (review.reviewer.toString() !== req.user.userId) {
      return res.status(403).json({ error: 'Access denied' });
    }

    if (review.status !== 'assigned') {
      return res.status(400).json({ 
        error: 'Invalid status',
        message: 'Only assigned reviews can be declined'
      });
    }

    // Update review status
    review.status = 'withdrawn';
    
    // Update paper assignment
    const paper = await Paper.findById(review.paper);
    const assignment = paper.assignedReviewers.find(
      assignment => assignment.reviewer.toString() === req.user.userId
    );
    
    if (assignment) {
      assignment.status = 'declined';
      await paper.save();
    }

    await review.save();

    // TODO: Send notification to editor

    res.json({
      message: 'Review assignment declined successfully',
      review: {
        id: review._id,
        status: review.status
      }
    });

  } catch (error) {
    console.error('Decline review error:', error);
    res.status(500).json({ 
      error: 'Failed to decline review',
      message: 'An error occurred while declining the review'
    });
  }
});

module.exports = router;