const express = require('express');
const multer = require('multer');
const path = require('path');
const fs = require('fs').promises;
const Paper = require('../models/Paper');
const User = require('../models/User');
const Review = require('../models/Review');
const Citation = require('../models/Citation');
const { auth, requireRole } = require('../middleware/auth');
const Joi = require('joi');

const router = express.Router();

// Configure multer for file uploads
const storage = multer.diskStorage({
  destination: async (req, file, cb) => {
    const uploadDir = path.join(__dirname, '../uploads/papers', req.user.userId);
    try {
      await fs.mkdir(uploadDir, { recursive: true });
      cb(null, uploadDir);
    } catch (error) {
      cb(error);
    }
  },
  filename: (req, file, cb) => {
    const uniqueSuffix = Date.now() + '-' + Math.round(Math.random() * 1E9);
    cb(null, file.fieldname + '-' + uniqueSuffix + path.extname(file.originalname));
  }
});

const fileFilter = (req, file, cb) => {
  const allowedTypes = ['.tex', '.pdf', '.zip', '.tar', '.gz', '.txt', '.csv', '.json'];
  const extname = path.extname(file.originalname).toLowerCase();
  
  if (allowedTypes.includes(extname)) {
    cb(null, true);
  } else {
    cb(new Error('Invalid file type'), false);
  }
};

const upload = multer({
  storage,
  fileFilter,
  limits: {
    fileSize: 100 * 1024 * 1024 // 100MB limit
  }
});

// Validation schemas
const createPaperSchema = Joi.object({
  title: Joi.string().max(300).required(),
  abstract: Joi.string().max(5000).required(),
  keywords: Joi.array().items(Joi.string().trim()).max(10),
  researchArea: Joi.string().valid(
    'Operating Systems',
    'Distributed Systems', 
    'Real-time Systems',
    'System Security',
    'Network Protocols',
    'Database Systems',
    'Virtualization',
    'Embedded Systems',
    'Cloud Computing',
    'Performance Analysis'
  ).required(),
  methodology: Joi.string().valid(
    'theoretical', 'experimental', 'simulation', 'survey', 'case_study', 'mixed'
  ).required(),
  content: Joi.object({
    latex: Joi.string(),
    wordCount: Joi.number(),
    pageCount: Joi.number()
  }),
  authors: Joi.array().items(Joi.object({
    user: Joi.string(),
    name: Joi.string(),
    email: Joi.string().email(),
    affiliation: Joi.string(),
    isCorresponding: Joi.boolean(),
    contribution: Joi.string(),
    orcid: Joi.string(),
    position: Joi.number()
  })).min(1).required()
});

const updatePaperSchema = Joi.object({
  title: Joi.string().max(300),
  abstract: Joi.string().max(5000),
  keywords: Joi.array().items(Joi.string().trim()).max(10),
  researchArea: Joi.string().valid(
    'Operating Systems', 'Distributed Systems', 'Real-time Systems',
    'System Security', 'Network Protocols', 'Database Systems',
    'Virtualization', 'Embedded Systems', 'Cloud Computing', 'Performance Analysis'
  ),
  methodology: Joi.string().valid(
    'theoretical', 'experimental', 'simulation', 'survey', 'case_study', 'mixed'
  ),
  content: Joi.object({
    latex: Joi.string(),
    wordCount: Joi.number(),
    pageCount: Joi.number()
  }),
  visibility: Joi.string().valid('public', 'private', 'embargoed')
});

// Create new paper
router.post('/', auth, upload.fields([
  { name: 'latex', maxCount: 1 },
  { name: 'pdf', maxCount: 1 },
  { name: 'supplementary', maxCount: 10 },
  { name: 'data', maxCount: 5 },
  { name: 'code', maxCount: 5 }
]), async (req, res) => {
  try {
    // Validate input
    const { error, value } = createPaperSchema.validate(req.body);
    if (error) {
      return res.status(400).json({ 
        error: 'Validation failed',
        details: error.details.map(detail => detail.message)
      });
    }

    // Process files
    const files = [];
    
    if (req.files) {
      Object.keys(req.files).forEach(fieldName => {
        req.files[fieldName].forEach(file => {
          files.push({
            filename: file.filename,
            originalName: file.originalname,
            path: file.path,
            type: fieldName === 'latex' ? 'latex' : 
                  fieldName === 'pdf' ? 'pdf' :
                  fieldName === 'supplementary' ? 'supplementary' :
                  fieldName === 'data' ? 'data' : 'code',
            size: file.size
          });
        });
      });
    }

    // Find corresponding author
    const correspondingAuthor = value.authors.find(author => author.isCorresponding) || value.authors[0];
    
    // Create paper
    const paper = new Paper({
      ...value,
      content: value.content || {},
      files,
      correspondingAuthor: correspondingAuthor.user ? correspondingAuthor.user : null,
      createdBy: req.user.userId,
      authors: value.authors.map(author => ({
        ...author,
        user: author.user && author.user === req.user.userId ? author.user : null
      }))
    });

    await paper.save();

    // Populate author information
    await paper.populate('authors.user', 'firstName lastName email affiliation');
    await paper.populate('correspondingAuthor', 'firstName lastName email affiliation');

    res.status(201).json({
      message: 'Paper created successfully',
      paper: {
        id: paper._id,
        title: paper.title,
        abstract: paper.abstract,
        keywords: paper.keywords,
        researchArea: paper.researchArea,
        methodology: paper.methodology,
        status: paper.status,
        authors: paper.authorNames,
        correspondingAuthor: paper.correspondingAuthorDetails,
        createdAt: paper.createdAt,
        version: paper.version
      }
    });

  } catch (error) {
    console.error('Create paper error:', error);
    res.status(500).json({ 
      error: 'Failed to create paper',
      message: 'An error occurred while creating the paper'
    });
  }
});

// Get user's papers
router.get('/my-papers', auth, async (req, res) => {
  try {
    const page = parseInt(req.query.page) || 1;
    const limit = parseInt(req.query.limit) || 10;
    const skip = (page - 1) * limit;

    const papers = await Paper.find({ createdBy: req.user.userId })
      .populate('correspondingAuthor', 'firstName lastName email')
      .populate('reviews')
      .sort({ createdAt: -1 })
      .skip(skip)
      .limit(limit);

    const total = await Paper.countDocuments({ createdBy: req.user.userId });

    res.json({
      papers: papers.map(paper => ({
        id: paper._id,
        title: paper.title,
        abstract: paper.abstract.substring(0, 200) + '...',
        keywords: paper.keywords,
        researchArea: paper.researchArea,
        status: paper.status,
        submissionDate: paper.submissionDate,
        reviewProgress: paper.reviewProgress,
        version: paper.version,
        metrics: paper.metrics,
        createdAt: paper.createdAt
      })),
      pagination: {
        current: page,
        total: Math.ceil(total / limit),
        count: papers.length,
        totalRecords: total
      }
    });

  } catch (error) {
    console.error('Get my papers error:', error);
    res.status(500).json({ 
      error: 'Failed to fetch papers',
      message: 'An error occurred while fetching papers'
    });
  }
});

// Get single paper
router.get('/:id', auth, async (req, res) => {
  try {
    const paper = await Paper.findById(req.params.id)
      .populate('createdBy', 'firstName lastName email affiliation')
      .populate('correspondingAuthor', 'firstName lastName email affiliation orcid')
      .populate('authors.user', 'firstName lastName email affiliation orcid')
      .populate('assignedReviewers.reviewer', 'firstName lastName email affiliation')
      .populate('reviews')
      .populate('parentPaper', 'title version')
      .populate('relatedPapers', 'title authors version')
      .populate('submissionTarget.conference', 'name shortName dates');

    if (!paper) {
      return res.status(404).json({ error: 'Paper not found' });
    }

    // Check access permissions
    const isOwner = paper.createdBy._id.toString() === req.user.userId;
    const isAuthor = paper.authors.some(author => 
      author.user && author.user._id.toString() === req.user.userId
    );
    const isReviewer = paper.assignedReviewers.some(assignment => 
      assignment.reviewer._id.toString() === req.user.userId
    );
    const isEditor = req.user.role === 'editor' || req.user.role === 'admin';
    const canView = isOwner || isAuthor || isReviewer || isEditor || 
                   paper.visibility === 'public';

    if (!canView) {
      return res.status(403).json({ error: 'Access denied' });
    }

    // Record view if not owner/reviewer/editor
    if (!isOwner && !isAuthor && !isReviewer && !isEditor) {
      await paper.recordView();
    }

    res.json({
      paper: {
        id: paper._id,
        title: paper.title,
        abstract: paper.abstract,
        keywords: paper.keywords,
        researchArea: paper.researchArea,
        methodology: paper.methodology,
        status: paper.status,
        version: paper.version,
        authors: paper.authors.map(author => ({
          id: author.user?._id,
          name: author.name || `${author.user?.firstName} ${author.user?.lastName}`,
          email: author.email || author.user?.email,
          affiliation: author.affiliation || author.user?.affiliation?.institution,
          isCorresponding: author.isCorresponding,
          contribution: author.contribution,
          orcid: author.orcid || author.user?.affiliation?.orcid
        })),
        correspondingAuthor: paper.correspondingAuthorDetails,
        content: {
          latex: (isOwner || isAuthor || isEditor) ? paper.content.latex : undefined,
          compiledPdf: paper.content.compiledPdf,
          wordCount: paper.content.wordCount,
          pageCount: paper.content.pageCount
        },
        files: paper.files.map(file => ({
          filename: file.originalName,
          type: file.type,
          size: file.size,
          uploadDate: file.uploadDate,
          path: (isOwner || isAuthor || isEditor) ? file.path : undefined
        })),
        submissionTarget: paper.submissionTarget,
        reviewProgress: paper.reviewProgress,
        assignedReviewers: isOwner || isAuthor || isEditor ? 
          paper.assignedReviewers.map(assignment => ({
            reviewer: assignment.reviewer,
            status: assignment.status,
            dueDate: assignment.dueDate
          })) : undefined,
        reviews: isOwner || isAuthor || isEditor || isReviewer ?
          paper.reviews.map(review => ({
            id: review._id,
            status: review.status,
            completedDate: review.completedDate,
            reviewer: isReviewer || isOwner || isAuthor || isEditor ? 
              review.reviewer : undefined,
            recommendation: review.recommendation,
            rating: review.rating
          })) : undefined,
        experiments: paper.experiments,
        changeLog: paper.changeLog,
        citations: paper.citations,
        bibliography: paper.bibliography,
        metrics: paper.metrics,
        finalDecision: paper.finalDecision,
        visibility: paper.visibility,
        createdAt: paper.createdAt,
        updatedAt: paper.updatedAt,
        ageInDays: paper.ageInDays
      }
    });

  } catch (error) {
    console.error('Get paper error:', error);
    res.status(500).json({ 
      error: 'Failed to fetch paper',
      message: 'An error occurred while fetching the paper'
    });
  }
});

// Update paper
router.put('/:id', auth, upload.fields([
  { name: 'latex', maxCount: 1 },
  { name: 'pdf', maxCount: 1 },
  { name: 'supplementary', maxCount: 10 }
]), async (req, res) => {
  try {
    const paper = await Paper.findById(req.params.id);
    
    if (!paper) {
      return res.status(404).json({ error: 'Paper not found' });
    }

    // Check ownership
    const isOwner = paper.createdBy.toString() === req.user.userId;
    if (!isOwner) {
      return res.status(403).json({ error: 'Access denied' });
    }

    // Check if paper can be edited
    if (['accepted', 'rejected', 'published'].includes(paper.status)) {
      return res.status(400).json({ 
        error: 'Cannot edit paper',
        message: 'Paper with this status cannot be edited'
      });
    }

    // Validate input
    const { error, value } = updatePaperSchema.validate(req.body);
    if (error) {
      return res.status(400).json({ 
        error: 'Validation failed',
        details: error.details.map(detail => detail.message)
      });
    }

    // Process new files
    const newFiles = [];
    
    if (req.files) {
      Object.keys(req.files).forEach(fieldName => {
        req.files[fieldName].forEach(file => {
          newFiles.push({
            filename: file.filename,
            originalName: file.originalname,
            path: file.path,
            type: fieldName === 'latex' ? 'latex' : 
                  fieldName === 'pdf' ? 'pdf' : 'supplementary',
            size: file.size
          });
        });
      });
    }

    // Update paper
    Object.keys(value).forEach(key => {
      if (value[key] !== undefined) {
        paper[key] = value[key];
      }
    });

    // Add new files
    if (newFiles.length > 0) {
      paper.files.push(...newFiles);
    }

    paper.updatedBy = req.user.userId;
    await paper.save();

    // Populate author information
    await paper.populate('correspondingAuthor authors.user', 'firstName lastName email');

    res.json({
      message: 'Paper updated successfully',
      paper: {
        id: paper._id,
        title: paper.title,
        abstract: paper.abstract,
        keywords: paper.keywords,
        researchArea: paper.researchArea,
        status: paper.status,
        authors: paper.authorNames,
        correspondingAuthor: paper.correspondingAuthorDetails,
        updatedAt: paper.updatedAt
      }
    });

  } catch (error) {
    console.error('Update paper error:', error);
    res.status(500).json({ 
      error: 'Failed to update paper',
      message: 'An error occurred while updating the paper'
    });
  }
});

// Submit paper for review
router.post('/:id/submit', auth, async (req, res) => {
  try {
    const paper = await Paper.findById(req.params.id);
    
    if (!paper) {
      return res.status(404).json({ error: 'Paper not found' });
    }

    // Check ownership
    const isOwner = paper.createdBy.toString() === req.user.userId;
    const isAuthor = paper.authors.some(author => 
      author.user && author.user.toString() === req.user.userId
    );
    
    if (!isOwner && !isAuthor) {
      return res.status(403).json({ error: 'Access denied' });
    }

    // Validate submission
    if (paper.status !== 'draft' && paper.status !== 'revision_requested') {
      return res.status(400).json({ 
        error: 'Invalid status',
        message: 'Paper can only be submitted from draft or revision_requested status'
      });
    }

    const { conferenceId, track } = req.body;

    // Submit paper
    await paper.submitForReview(conferenceId, track);

    res.json({
      message: 'Paper submitted successfully',
      paper: {
        id: paper._id,
        title: paper.title,
        status: paper.status,
        submissionDate: paper.submissionDate,
        submissionTarget: paper.submissionTarget
      }
    });

  } catch (error) {
    console.error('Submit paper error:', error);
    res.status(500).json({ 
      error: 'Failed to submit paper',
      message: 'An error occurred while submitting the paper'
    });
  }
});

// Create new version of paper
router.post('/:id/version', auth, async (req, res) => {
  try {
    const paper = await Paper.findById(req.params.id);
    
    if (!paper) {
      return res.status(404).json({ error: 'Paper not found' });
    }

    // Check ownership
    const isOwner = paper.createdBy.toString() === req.user.userId;
    if (!isOwner) {
      return res.status(403).json({ error: 'Access denied' });
    }

    const { changes } = req.body;
    
    if (!changes) {
      return res.status(400).json({ 
        error: 'Missing changes',
        message: 'Description of changes is required'
      });
    }

    // Create new version
    const newVersion = await paper.createNewVersion(changes, req.user.userId);

    res.status(201).json({
      message: 'New version created successfully',
      newVersion: {
        id: newVersion._id,
        title: newVersion.title,
        version: newVersion.version,
        status: newVersion.status,
        parentPaper: newVersion.parentPaper,
        createdAt: newVersion.createdAt
      }
    });

  } catch (error) {
    console.error('Create version error:', error);
    res.status(500).json({ 
      error: 'Failed to create new version',
      message: 'An error occurred while creating the new version'
    });
  }
});

// Delete paper
router.delete('/:id', auth, async (req, res) => {
  try {
    const paper = await Paper.findById(req.params.id);
    
    if (!paper) {
      return res.status(404).json({ error: 'Paper not found' });
    }

    // Check ownership
    const isOwner = paper.createdBy.toString() === req.user.userId;
    if (!isOwner) {
      return res.status(403).json({ error: 'Access denied' });
    }

    // Check if paper can be deleted
    if (['submitted', 'under_review', 'accepted', 'published'].includes(paper.status)) {
      return res.status(400).json({ 
        error: 'Cannot delete paper',
        message: 'Paper in review or published status cannot be deleted'
      });
    }

    // Delete files
    for (const file of paper.files) {
      try {
        await fs.unlink(file.path);
      } catch (error) {
        console.warn(`Failed to delete file ${file.path}:`, error);
      }
    }

    await Paper.findByIdAndDelete(req.params.id);

    res.json({ message: 'Paper deleted successfully' });

  } catch (error) {
    console.error('Delete paper error:', error);
    res.status(500).json({ 
      error: 'Failed to delete paper',
      message: 'An error occurred while deleting the paper'
    });
  }
});

// Get papers for review (assigned to current user)
router.get('/review/assigned', auth, async (req, res) => {
  try {
    const page = parseInt(req.query.page) || 1;
    const limit = parseInt(req.query.limit) || 10;
    const skip = (page - 1) * limit;

    const papers = await Paper.find({
      'assignedReviewers.reviewer': req.user.userId,
      'assignedReviewers.status': { $ne: 'completed' }
    })
      .populate('createdBy', 'firstName lastName affiliation')
      .populate('correspondingAuthor', 'firstName lastName')
      .sort({ 'assignedReviewers.dueDate': 1 })
      .skip(skip)
      .limit(limit);

    const total = await Paper.countDocuments({
      'assignedReviewers.reviewer': req.user.userId,
      'assignedReviewers.status': { $ne: 'completed' }
    });

    res.json({
      papers: papers.map(paper => {
        const assignment = paper.assignedReviewers.find(
          a => a.reviewer._id.toString() === req.user.userId
        );
        
        return {
          id: paper._id,
          title: paper.title,
          abstract: paper.abstract.substring(0, 200) + '...',
          authors: paper.authorNames,
          correspondingAuthor: paper.correspondingAuthorDetails,
          status: paper.status,
          researchArea: paper.researchArea,
          assignment: {
            assignedDate: assignment.assignedDate,
            dueDate: assignment.dueDate,
            status: assignment.status,
            daysRemaining: Math.ceil((assignment.dueDate - new Date()) / (1000 * 60 * 60 * 24))
          },
          createdAt: paper.createdAt
        };
      }),
      pagination: {
        current: page,
        total: Math.ceil(total / limit),
        count: papers.length,
        totalRecords: total
      }
    });

  } catch (error) {
    console.error('Get review assignments error:', error);
    res.status(500).json({ 
      error: 'Failed to fetch review assignments',
      message: 'An error occurred while fetching review assignments'
    });
  }
});

// Search papers
router.get('/', async (req, res) => {
  try {
    const page = parseInt(req.query.page) || 1;
    const limit = parseInt(req.query.limit) || 20;
    const skip = (page - 1) * limit;

    const { 
      query, 
      researchArea, 
      status, 
      year, 
      author,
      sortBy = 'createdAt',
      sortOrder = 'desc'
    } = req.query;

    // Build search query
    const searchQuery = {
      visibility: 'public',
      status: { $in: ['submitted', 'under_review', 'accepted', 'published'] }
    };

    if (query) {
      searchQuery.$text = { $search: query };
    }

    if (researchArea) {
      searchQuery.researchArea = researchArea;
    }

    if (status) {
      searchQuery.status = status;
    }

    if (year) {
      const startDate = new Date(parseInt(year), 0, 1);
      const endDate = new Date(parseInt(year) + 1, 0, 1);
      searchQuery.createdAt = { $gte: startDate, $lt: endDate };
    }

    if (author) {
      searchQuery.$or = [
        { 'authors.name': { $regex: author, $options: 'i' } },
        { 'authors.email': { $regex: author, $options: 'i' } }
      ];
    }

    // Build sort object
    const sortObj = {};
    sortObj[sortBy] = sortOrder === 'desc' ? -1 : 1;

    const papers = await Paper.find(searchQuery)
      .populate('authors.user', 'firstName lastName affiliation')
      .populate('correspondingAuthor', 'firstName lastName affiliation')
      .sort(sortObj)
      .skip(skip)
      .limit(limit);

    const total = await Paper.countDocuments(searchQuery);

    res.json({
      papers: papers.map(paper => ({
        id: paper._id,
        title: paper.title,
        abstract: paper.abstract.substring(0, 300) + '...',
        keywords: paper.keywords,
        researchArea: paper.researchArea,
        status: paper.status,
        authors: paper.authorNames,
        correspondingAuthor: paper.correspondingAuthorDetails,
        metrics: {
          views: paper.metrics.views,
          downloads: paper.metrics.downloads,
          citations: paper.metrics.citations
        },
        createdAt: paper.createdAt
      })),
      pagination: {
        current: page,
        total: Math.ceil(total / limit),
        count: papers.length,
        totalRecords: total
      }
    });

  } catch (error) {
    console.error('Search papers error:', error);
    res.status(500).json({ 
      error: 'Failed to search papers',
      message: 'An error occurred while searching papers'
    });
  }
});

module.exports = router;