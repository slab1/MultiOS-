const express = require('express');
const Conference = require('../models/Conference');
const Paper = require('../models/Paper');
const { auth, requireRole } = require('../middleware/auth');
const Joi = require('joi');

const router = express.Router();

// Validation schemas
const createConferenceSchema = Joi.object({
  name: Joi.string().max(200).required(),
  shortName: Joi.string().max(50).required(),
  acronym: Joi.string().max(10).required(),
  description: Joi.string().required(),
  theme: Joi.string(),
  scope: Joi.array().items(Joi.string()).max(20),
  keywords: Joi.array().items(Joi.string()).max(20),
  location: Joi.object({
    city: Joi.string(),
    country: Joi.string().required(),
    venue: Joi.string(),
    virtual: Joi.boolean().default(false),
    hybrid: Joi.boolean().default(false),
    timezone: Joi.string().default('UTC')
  }).required(),
  dates: Joi.object({
    conferenceStart: Joi.date().required(),
    conferenceEnd: Joi.date().required(),
    registrationStart: Joi.date(),
    registrationEnd: Joi.date()
  }).required(),
  researchAreas: Joi.array().items(Joi.string()).required(),
  reviewProcess: Joi.object({
    reviewerCount: Joi.number().min(1).max(10).default(3),
    reviewRounds: Joi.number().min(1).max(3).default(2),
    reviewPeriod: Joi.number().min(1).max(60).default(21),
    discussionPeriod: Joi.number().min(1).max(30).default(7),
    consensusRequired: Joi.boolean().default(false)
  }),
  reviewerManagement: Joi.object({
    minimumReviewsPerReviewer: Joi.number().min(1).max(10).default(3),
    maximumReviewsPerReviewer: Joi.number().min(3).max(20).default(6),
    automaticAssignment: Joi.boolean().default(true),
    conflictOfInterest: Joi.boolean().default(true)
  }),
  publication: Joi.object({
    proceedingsPublisher: Joi.string(),
    proceedingsISBN: Joi.string(),
    doiPrefix: Joi.string(),
    indexedIn: Joi.array().items(Joi.string()),
    hasSpecialIssue: Joi.boolean().default(false),
    specialIssueJournal: Joi.string(),
    openAccess: Joi.boolean().default(false)
  }),
  contact: Joi.object({
    email: Joi.string().email().required(),
    website: Joi.string().uri(),
    socialMedia: Joi.object({
      twitter: Joi.string(),
      linkedin: Joi.string(),
      facebook: Joi.string()
    })
  }).required()
});

const updateConferenceSchema = Joi.object({
  name: Joi.string().max(200),
  shortName: Joi.string().max(50),
  description: Joi.string(),
  theme: Joi.string(),
  scope: Joi.array().items(Joi.string()).max(20),
  keywords: Joi.array().items(Joi.string()).max(20),
  location: Joi.object({
    city: Joi.string(),
    country: Joi.string(),
    venue: Joi.string(),
    virtual: Joi.boolean(),
    hybrid: Joi.boolean(),
    timezone: Joi.string()
  }),
  dates: Joi.object({
    conferenceStart: Joi.date(),
    conferenceEnd: Joi.date(),
    registrationStart: Joi.date(),
    registrationEnd: Joi.date()
  }),
  researchAreas: Joi.array().items(Joi.string()),
  status: Joi.string().valid(
    'planning', 'cfp_announced', 'submissions_open', 'submissions_closed',
    'under_review', 'reviews_completed', 'decisions_sent', 
    'camera_ready_deadline', 'proceedings_final', 'completed', 'cancelled'
  ),
  visibility: Joi.string().valid('public', 'private', 'unlisted')
});

const addTrackSchema = Joi.object({
  name: Joi.string().required(),
  description: Joi.string(),
  chairs: Joi.array().items(Joi.object({
    name: Joi.string().required(),
    email: Joi.string().email().required(),
    affiliation: Joi.string().required(),
    role: Joi.string().valid('chair', 'co-chair').default('co-chair')
  }))
});

const addImportantDateSchema = Joi.object({
  name: Joi.string().required(),
  date: Joi.date().required(),
  timezone: Joi.string().default('UTC'),
  description: Joi.string(),
  isHardDeadline: Joi.boolean().default(true)
});

// Create new conference
router.post('/', auth, requireRole(['admin', 'editor']), async (req, res) => {
  try {
    // Validate input
    const { error, value } = createConferenceSchema.validate(req.body);
    if (error) {
      return res.status(400).json({ 
        error: 'Validation failed',
        details: error.details.map(detail => detail.message)
      });
    }

    // Check if conference with same acronym exists
    const existingConference = await Conference.findOne({ 
      $or: [
        { acronym: value.acronym },
        { shortName: value.shortName }
      ]
    });

    if (existingConference) {
      return res.status(409).json({ 
        error: 'Conference already exists',
        message: 'A conference with this acronym or short name already exists'
      });
    }

    // Create conference
    const conference = new Conference({
      ...value,
      createdBy: req.user.userId,
      organizingCommittee: {
        generalChairs: [],
        programChairs: [],
        publicChairs: [],
        localChairs: [],
        workshopChairs: []
      },
      tracks: [],
      importantDates: [],
      statistics: {
        totalSubmissions: 0,
        totalAcceptances: 0,
        acceptanceRate: 0,
        trackBreakdown: []
      },
      awards: [],
      sponsors: []
    });

    await conference.save();

    res.status(201).json({
      message: 'Conference created successfully',
      conference: {
        id: conference._id,
        name: conference.name,
        shortName: conference.shortName,
        acronym: conference.acronym,
        description: conference.description,
        location: conference.location,
        dates: conference.dates,
        researchAreas: conference.researchAreas,
        status: conference.status,
        contact: conference.contact,
        createdAt: conference.createdAt
      }
    });

  } catch (error) {
    console.error('Create conference error:', error);
    res.status(500).json({ 
      error: 'Failed to create conference',
      message: 'An error occurred while creating the conference'
    });
  }
});

// Get all conferences
router.get('/', async (req, res) => {
  try {
    const page = parseInt(req.query.page) || 1;
    const limit = parseInt(req.query.limit) || 20;
    const skip = (page - 1) * limit;

    const { 
      status, 
      year, 
      country, 
      researchArea,
      search,
      sortBy = 'dates.conferenceStart',
      sortOrder = 'asc'
    } = req.query;

    // Build search query
    const query = { visibility: 'public' };

    if (status) {
      query.status = status;
    }

    if (year) {
      const startDate = new Date(parseInt(year), 0, 1);
      const endDate = new Date(parseInt(year) + 1, 0, 1);
      query.$and = [
        { 'dates.conferenceStart': { $gte: startDate } },
        { 'dates.conferenceStart': { $lt: endDate } }
      ];
    }

    if (country) {
      query['location.country'] = new RegExp(country, 'i');
    }

    if (researchArea) {
      query.researchAreas = researchArea;
    }

    if (search) {
      query.$or = [
        { name: { $regex: search, $options: 'i' } },
        { shortName: { $regex: search, $options: 'i' } },
        { acronym: { $regex: search, $options: 'i' } },
        { description: { $regex: search, $options: 'i' } },
        { keywords: { $regex: search, $options: 'i' } }
      ];
    }

    // Build sort object
    const sortObj = {};
    sortObj[sortBy] = sortOrder === 'desc' ? -1 : 1;

    const conferences = await Conference.find(query)
      .populate('organizingCommittee.generalChairs', 'name affiliation')
      .populate('organizingCommittee.programChairs', 'name affiliation')
      .sort(sortObj)
      .skip(skip)
      .limit(limit);

    const total = await Conference.countDocuments(query);

    res.json({
      conferences: conferences.map(conf => ({
        id: conf._id,
        name: conf.name,
        shortName: conf.shortName,
        acronym: conf.acronym,
        description: conf.description.substring(0, 200) + '...',
        location: {
          city: conf.location.city,
          country: conf.location.country,
          virtual: conf.location.virtual,
          hybrid: conf.location.hybrid
        },
        dates: {
          conferenceStart: conf.dates.conferenceStart,
          conferenceEnd: conf.dates.conferenceEnd,
          registrationStart: conf.dates.registrationStart,
          registrationEnd: conf.dates.registrationEnd
        },
        researchAreas: conf.researchAreas,
        status: conf.status,
        nextDeadline: conf.nextDeadline,
        daysUntilNextDeadline: conf.daysUntilNextDeadline,
        isAcceptingSubmissions: conf.isAcceptingSubmissions,
        statistics: {
          totalSubmissions: conf.statistics.totalSubmissions,
          acceptanceRate: conf.statistics.acceptanceRate
        },
        contact: {
          email: conf.contact.email,
          website: conf.contact.website
        }
      })),
      pagination: {
        current: page,
        total: Math.ceil(total / limit),
        count: conferences.length,
        totalRecords: total
      }
    });

  } catch (error) {
    console.error('Get conferences error:', error);
    res.status(500).json({ 
      error: 'Failed to fetch conferences',
      message: 'An error occurred while fetching conferences'
    });
  }
});

// Get single conference
router.get('/:id', async (req, res) => {
  try {
    const conference = await Conference.findById(req.params.id)
      .populate('organizingCommittee.generalChairs')
      .populate('organizingCommittee.programChairs')
      .populate('organizingCommittee.publicChairs')
      .populate('organizingCommittee.localChairs')
      .populate('organizingCommittee.workshopChairs')
      .populate('tracks.chairs')
      .populate('tracks.reviewers')
      .populate('awards.winner.paper', 'title authors')
      .populate('createdBy', 'firstName lastName');

    if (!conference) {
      return res.status(404).json({ error: 'Conference not found' });
    }

    // Check access for private conferences
    if (conference.visibility === 'private') {
      const isEditor = req.user && (req.user.role === 'editor' || req.user.role === 'admin');
      const isOwner = req.user && conference.createdBy._id.toString() === req.user.userId;
      
      if (!isEditor && !isOwner) {
        return res.status(403).json({ error: 'Access denied' });
      }
    }

    res.json({
      conference: {
        id: conference._id,
        name: conference.name,
        shortName: conference.shortName,
        acronym: conference.acronym,
        description: conference.description,
        theme: conference.theme,
        scope: conference.scope,
        keywords: conference.keywords,
        organizingCommittee: conference.organizingCommittee,
        location: conference.location,
        dates: conference.dates,
        importantDates: conference.importantDates.sort((a, b) => a.date - b.date),
        researchAreas: conference.researchAreas,
        tracks: conference.tracks,
        status: conference.status,
        submissionGuidelines: conference.submissionGuidelines,
        reviewProcess: conference.reviewProcess,
        reviewerManagement: conference.reviewerManagement,
        statistics: conference.statistics,
        publication: conference.publication,
        awards: conference.awards,
        sponsors: conference.sponsors,
        contact: conference.contact,
        isAcceptingSubmissions: conference.isAcceptingSubmissions,
        nextDeadline: conference.nextDeadline,
        daysUntilNextDeadline: conference.daysUntilNextDeadline,
        currentAcceptanceRate: conference.currentAcceptanceRate,
        isRegistrationOpen: conference.isRegistrationOpen,
        createdAt: conference.createdAt
      }
    });

  } catch (error) {
    console.error('Get conference error:', error);
    res.status(500).json({ 
      error: 'Failed to fetch conference',
      message: 'An error occurred while fetching the conference'
    });
  }
});

// Update conference
router.put('/:id', auth, requireRole(['admin', 'editor']), async (req, res) => {
  try {
    const conference = await Conference.findById(req.params.id);

    if (!conference) {
      return res.status(404).json({ error: 'Conference not found' });
    }

    // Check ownership for editing
    const isOwner = conference.createdBy.toString() === req.user.userId;
    const isAdmin = req.user.role === 'admin';
    
    if (!isOwner && !isAdmin) {
      return res.status(403).json({ error: 'Access denied' });
    }

    // Validate input
    const { error, value } = updateConferenceSchema.validate(req.body);
    if (error) {
      return res.status(400).json({ 
        error: 'Validation failed',
        details: error.details.map(detail => detail.message)
      });
    }

    // Update conference
    Object.keys(value).forEach(key => {
      if (value[key] !== undefined) {
        conference[key] = value[key];
      }
    });

    conference.updatedBy = req.user.userId;
    await conference.save();

    res.json({
      message: 'Conference updated successfully',
      conference: {
        id: conference._id,
        name: conference.name,
        status: conference.status,
        updatedAt: conference.updatedAt
      }
    });

  } catch (error) {
    console.error('Update conference error:', error);
    res.status(500).json({ 
      error: 'Failed to update conference',
      message: 'An error occurred while updating the conference'
    });
  }
});

// Add track to conference
router.post('/:id/tracks', auth, requireRole(['admin', 'editor']), async (req, res) => {
  try {
    const conference = await Conference.findById(req.params.id);

    if (!conference) {
      return res.status(404).json({ error: 'Conference not found' });
    }

    // Check ownership
    const isOwner = conference.createdBy.toString() === req.user.userId;
    const isAdmin = req.user.role === 'admin';
    
    if (!isOwner && !isAdmin) {
      return res.status(403).json({ error: 'Access denied' });
    }

    // Validate input
    const { error, value } = addTrackSchema.validate(req.body);
    if (error) {
      return res.status(400).json({ 
        error: 'Validation failed',
        details: error.details.map(detail => detail.message)
      });
    }

    await conference.addTrack(value);

    res.status(201).json({
      message: 'Track added successfully',
      track: value
    });

  } catch (error) {
    console.error('Add track error:', error);
    res.status(500).json({ 
      error: 'Failed to add track',
      message: 'An error occurred while adding the track'
    });
  }
});

// Add important date
router.post('/:id/dates', auth, requireRole(['admin', 'editor']), async (req, res) => {
  try {
    const conference = await Conference.findById(req.params.id);

    if (!conference) {
      return res.status(404).json({ error: 'Conference not found' });
    }

    // Check ownership
    const isOwner = conference.createdBy.toString() === req.user.userId;
    const isAdmin = req.user.role === 'admin';
    
    if (!isOwner && !isAdmin) {
      return res.status(403).json({ error: 'Access denied' });
    }

    // Validate input
    const { error, value } = addImportantDateSchema.validate(req.body);
    if (error) {
      return res.status(400).json({ 
        error: 'Validation failed',
        details: error.details.map(detail => detail.message)
      });
    }

    await conference.addImportantDate(value);

    res.status(201).json({
      message: 'Important date added successfully',
      date: value
    });

  } catch (error) {
    console.error('Add date error:', error);
    res.status(500).json({ 
      error: 'Failed to add date',
      message: 'An error occurred while adding the date'
    });
  }
});

// Update conference status
router.put('/:id/status', auth, requireRole(['admin', 'editor']), async (req, res) => {
  try {
    const { status } = req.body;

    if (!status) {
      return res.status(400).json({ 
        error: 'Missing status',
        message: 'Status is required'
      });
    }

    const conference = await Conference.findById(req.params.id);

    if (!conference) {
      return res.status(404).json({ error: 'Conference not found' });
    }

    // Check ownership
    const isOwner = conference.createdBy.toString() === req.user.userId;
    const isAdmin = req.user.role === 'admin';
    
    if (!isOwner && !isAdmin) {
      return res.status(403).json({ error: 'Access denied' });
    }

    await conference.updateStatus(status);

    res.json({
      message: 'Conference status updated successfully',
      conference: {
        id: conference._id,
        name: conference.name,
        status: conference.status
      }
    });

  } catch (error) {
    console.error('Update status error:', error);
    res.status(500).json({ 
      error: 'Failed to update status',
      message: error.message || 'An error occurred while updating the status'
    });
  }
});

// Get conference submissions
router.get('/:id/submissions', auth, requireRole(['admin', 'editor']), async (req, res) => {
  try {
    const { status, track, researchArea } = req.query;

    // Build query
    const query = { 'submissionTarget.conference': req.params.id };

    if (status) {
      query.status = status;
    }

    if (track) {
      query['submissionTarget.track'] = track;
    }

    if (researchArea) {
      query.researchArea = researchArea;
    }

    const submissions = await Paper.find(query)
      .populate('createdBy', 'firstName lastName affiliation')
      .populate('correspondingAuthor', 'firstName lastName affiliation')
      .populate('authors.user', 'firstName lastName affiliation')
      .sort({ submissionDate: -1 });

    res.json({
      submissions: submissions.map(paper => ({
        id: paper._id,
        title: paper.title,
        authors: paper.authorNames,
        correspondingAuthor: paper.correspondingAuthorDetails,
        researchArea: paper.researchArea,
        status: paper.status,
        submissionDate: paper.submissionDate,
        submissionTarget: {
          track: paper.submissionTarget.track
        },
        reviewProgress: paper.reviewProgress,
        metrics: {
          views: paper.metrics.views,
          downloads: paper.metrics.downloads
        }
      })),
      totalSubmissions: submissions.length
    });

  } catch (error) {
    console.error('Get submissions error:', error);
    res.status(500).json({ 
      error: 'Failed to fetch submissions',
      message: 'An error occurred while fetching submissions'
    });
  }
});

// Update conference statistics
router.post('/:id/update-stats', auth, requireRole(['admin', 'editor']), async (req, res) => {
  try {
    const conference = await Conference.findById(req.params.id);

    if (!conference) {
      return res.status(404).json({ error: 'Conference not found' });
    }

    // Check ownership
    const isOwner = conference.createdBy.toString() === req.user.userId;
    const isAdmin = req.user.role === 'admin';
    
    if (!isOwner && !isAdmin) {
      return res.status(403).json({ error: 'Access denied' });
    }

    await conference.updateStatistics();

    res.json({
      message: 'Statistics updated successfully',
      statistics: conference.statistics
    });

  } catch (error) {
    console.error('Update statistics error:', error);
    res.status(500).json({ 
      error: 'Failed to update statistics',
      message: 'An error occurred while updating statistics'
    });
  }
});

// Search upcoming conferences
router.get('/search/upcoming', async (req, res) => {
  try {
    const limit = parseInt(req.query.limit) || 10;

    const upcomingConferences = await Conference.findUpcoming(limit);

    res.json({
      conferences: upcomingConferences.map(conf => ({
        id: conf._id,
        name: conf.name,
        shortName: conf.shortName,
        acronym: conf.acronym,
        location: conf.location,
        dates: conf.dates,
        nextDeadline: conf.nextDeadline,
        daysUntilNextDeadline: conf.daysUntilNextDeadline,
        isAcceptingSubmissions: conf.isAcceptingSubmissions,
        contact: {
          email: conf.contact.email,
          website: conf.contact.website
        }
      }))
    });

  } catch (error) {
    console.error('Search upcoming error:', error);
    res.status(500).json({ 
      error: 'Failed to search conferences',
      message: 'An error occurred while searching conferences'
    });
  }
});

// Get conferences by research area
router.get('/search/by-area/:area', async (req, res) => {
  try {
    const area = decodeURIComponent(req.params.area);
    const conferences = await Conference.findByResearchArea(area);

    res.json({
      researchArea: area,
      conferences: conferences.map(conf => ({
        id: conf._id,
        name: conf.name,
        shortName: conf.shortName,
        acronym: conf.acronym,
        location: conf.location,
        dates: conf.dates,
        status: conf.status,
        isAcceptingSubmissions: conf.isAcceptingSubmissions,
        nextDeadline: conf.nextDeadline,
        contact: conf.contact
      }))
    });

  } catch (error) {
    console.error('Get by area error:', error);
    res.status(500).json({ 
      error: 'Failed to fetch conferences by area',
      message: 'An error occurred while fetching conferences'
    });
  }
});

// Delete conference
router.delete('/:id', auth, requireRole(['admin']), async (req, res) => {
  try {
    const conference = await Conference.findById(req.params.id);

    if (!conference) {
      return res.status(404).json({ error: 'Conference not found' });
    }

    // Check if conference has submissions
    const submissionCount = await Paper.countDocuments({
      'submissionTarget.conference': req.params.id
    });

    if (submissionCount > 0) {
      return res.status(400).json({ 
        error: 'Cannot delete conference',
        message: 'Conference has submissions and cannot be deleted'
      });
    }

    await Conference.findByIdAndDelete(req.params.id);

    res.json({ message: 'Conference deleted successfully' });

  } catch (error) {
    console.error('Delete conference error:', error);
    res.status(500).json({ 
      error: 'Failed to delete conference',
      message: 'An error occurred while deleting the conference'
    });
  }
});

module.exports = router;