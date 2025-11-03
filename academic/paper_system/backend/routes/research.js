const express = require('express');
const mongoose = require('mongoose');
const User = require('../models/User');
const Paper = require('../models/Paper');
const { auth, requireRole } = require('../middleware/auth');
const Joi = require('joi');

const router = express.Router();

// Research Area Model
const researchAreaSchema = new mongoose.Schema({
  name: {
    type: String,
    required: true,
    unique: true
  },
  description: String,
  subAreas: [String],
  keywords: [String],
  activePapers: { type: Number, default: 0 },
  activeResearchers: { type: Number, default: 0 },
  yearlyPublications: { type: Number, default: 0 },
  averageCitations: { type: Number, default: 0 },
  topJournals: [String],
  topConferences: [String],
  emergingTrends: [String],
  interdisciplinaryConnections: [{
    area: String,
    strength: { type: Number, min: 1, max: 10 }
  }],
  statistics: {
    totalPapers: { type: Number, default: 0 },
    totalResearchers: { type: Number, default: 0 },
    averageReviewTime: { type: Number, default: 0 },
    acceptanceRate: { type: Number, default: 0 }
  },
  createdAt: { type: Date, default: Date.now },
  updatedAt: { type: Date, default: Date.now }
});

const ResearchArea = mongoose.model('ResearchArea', researchAreaSchema);

// Validation schemas
const createResearchAreaSchema = Joi.object({
  name: Joi.string().required(),
  description: Joi.string(),
  subAreas: Joi.array().items(Joi.string()),
  keywords: Joi.array().items(Joi.string()),
  emergingTrends: Joi.array().items(Joi.string()),
  topJournals: Joi.array().items(Joi.string()),
  topConferences: Joi.array().items(Joi.string())
});

const updateResearchAreaSchema = Joi.object({
  name: Joi.string(),
  description: Joi.string(),
  subAreas: Joi.array().items(Joi.string()),
  keywords: Joi.array().items(Joi.string()),
  emergingTrends: Joi.array().items(Joi.string()),
  topJournals: Joi.array().items(Joi.string()),
  topConferences: Joi.array().items(Joi.string())
});

// Get all research areas
router.get('/', async (req, res) => {
  try {
    const { page = 1, limit = 20, search, sortBy = 'name', order = 'asc' } = req.query;
    
    const query = {};
    if (search) {
      query.$or = [
        { name: { $regex: search, $options: 'i' } },
        { description: { $regex: search, $options: 'i' } },
        { keywords: { $regex: search, $options: 'i' } }
      ];
    }
    
    const sortOptions = {};
    sortOptions[sortBy] = order === 'desc' ? -1 : 1;
    
    const areas = await ResearchArea.find(query)
      .sort(sortOptions)
      .limit(limit * 1)
      .skip((page - 1) * limit)
      .exec();
    
    const total = await ResearchArea.countDocuments(query);
    
    res.json({
      areas,
      totalPages: Math.ceil(total / limit),
      currentPage: page,
      total
    });
  } catch (error) {
    console.error('Error fetching research areas:', error);
    res.status(500).json({ error: 'Failed to fetch research areas' });
  }
});

// Get research area by ID
router.get('/:id', async (req, res) => {
  try {
    const area = await ResearchArea.findById(req.params.id);
    if (!area) {
      return res.status(404).json({ error: 'Research area not found' });
    }
    
    // Get related statistics
    const papers = await Paper.find({ researchArea: area.name }).populate('authors');
    const researchers = await User.find({ 
      'researchAreas.name': area.name,
      role: 'researcher'
    });
    
    const stats = {
      totalPapers: papers.length,
      totalResearchers: researchers.length,
      publishedPapers: papers.filter(p => p.status === 'published').length,
      underReview: papers.filter(p => p.status === 'under_review').length,
      averageCitations: papers.reduce((sum, p) => sum + p.citations.length, 0) / papers.length || 0,
      topResearchers: researchers
        .sort((a, b) => (b.statistics?.papersAccepted || 0) - (a.statistics?.papersAccepted || 0))
        .slice(0, 10)
        .map(u => ({
          id: u._id,
          name: u.fullName,
          institution: u.affiliation?.institution,
          papersAccepted: u.statistics?.papersAccepted || 0,
          hIndex: u.statistics?.hIndex || 0
        })),
      recentPapers: papers
        .sort((a, b) => new Date(b.createdAt) - new Date(a.createdAt))
        .slice(0, 5)
        .map(p => ({
          id: p._id,
          title: p.title,
          status: p.status,
          authors: p.authors.map(a => a.name),
          submissionDate: p.submissionDate
        }))
    };
    
    res.json({
      area,
      statistics: stats
    });
  } catch (error) {
    console.error('Error fetching research area:', error);
    res.status(500).json({ error: 'Failed to fetch research area' });
  }
});

// Create new research area (admin only)
router.post('/', auth, requireRole(['admin']), async (req, res) => {
  try {
    const { error, value } = createResearchAreaSchema.validate(req.body);
    if (error) {
      return res.status(400).json({ 
        error: 'Validation failed', 
        details: error.details.map(d => d.message) 
      });
    }
    
    const existingArea = await ResearchArea.findOne({ name: value.name });
    if (existingArea) {
      return res.status(409).json({ error: 'Research area already exists' });
    }
    
    const area = new ResearchArea(value);
    await area.save();
    
    res.status(201).json({ 
      message: 'Research area created successfully',
      area 
    });
  } catch (error) {
    console.error('Error creating research area:', error);
    res.status(500).json({ error: 'Failed to create research area' });
  }
});

// Update research area (admin only)
router.put('/:id', auth, requireRole(['admin']), async (req, res) => {
  try {
    const { error, value } = updateResearchAreaSchema.validate(req.body);
    if (error) {
      return res.status(400).json({ 
        error: 'Validation failed', 
        details: error.details.map(d => d.message) 
      });
    }
    
    const area = await ResearchArea.findByIdAndUpdate(
      req.params.id,
      { ...value, updatedAt: new Date() },
      { new: true, runValidators: true }
    );
    
    if (!area) {
      return res.status(404).json({ error: 'Research area not found' });
    }
    
    res.json({ 
      message: 'Research area updated successfully',
      area 
    });
  } catch (error) {
    console.error('Error updating research area:', error);
    res.status(500).json({ error: 'Failed to update research area' });
  }
});

// Get research trends and analytics
router.get('/analytics/trends', async (req, res) => {
  try {
    const { timeRange = '1year', area } = req.query;
    
    let dateFilter = {};
    const now = new Date();
    
    switch (timeRange) {
      case '1month':
        dateFilter = { createdAt: { $gte: new Date(now.getFullYear(), now.getMonth(), 1) } };
        break;
      case '6months':
        dateFilter = { createdAt: { $gte: new Date(now.getFullYear(), now.getMonth() - 6, now.getDate()) } };
        break;
      case '1year':
        dateFilter = { createdAt: { $gte: new Date(now.getFullYear() - 1, now.getMonth(), now.getDate()) } };
        break;
      case '5years':
        dateFilter = { createdAt: { $gte: new Date(now.getFullYear() - 5, now.getMonth(), now.getDate()) } };
        break;
    }
    
    if (area) {
      dateFilter.researchArea = area;
    }
    
    // Get paper submission trends
    const paperTrends = await Paper.aggregate([
      { $match: { ...dateFilter } },
      {
        $group: {
          _id: {
            year: { $year: '$createdAt' },
            month: { $month: '$createdAt' }
          },
          count: { $sum: 1 },
          accepted: { $sum: { $cond: [{ $eq: ['$status', 'accepted'] }, 1, 0] } },
          rejected: { $sum: { $cond: [{ $eq: ['$status', 'rejected'] }, 1, 0] } }
        }
      },
      { $sort: { '_id.year': 1, '_id.month': 1 } }
    ]);
    
    // Get research area popularity
    const areaPopularity = await Paper.aggregate([
      { $match: dateFilter },
      {
        $group: {
          _id: '$researchArea',
          count: { $sum: 1 },
          averageCitations: { $avg: { $size: '$citations' } },
          topPapers: { $push: { $slice: ['$citations', -5] } }
        }
      },
      { $sort: { count: -1 } },
      { $limit: 10 }
    ]);
    
    // Get researcher trends by area
    const researcherTrends = await User.aggregate([
      {
        $match: { 
          role: 'researcher',
          'researchAreas.0': { $exists: true }
        }
      },
      {
        $unwind: '$researchAreas'
      },
      {
        $group: {
          _id: '$researchAreas.name',
          researchers: { $sum: 1 },
          activeResearchers: {
            $sum: { $cond: [{ $gt: ['$lastLogin', new Date(Date.now() - 30 * 24 * 60 * 60 * 1000)] }, 1, 0] }
          },
          averageProductivity: {
            $avg: { $add: ['$statistics.papersSubmitted', '$statistics.papersAccepted'] }
          }
        }
      },
      { $sort: { researchers: -1 } }
    ]);
    
    res.json({
      paperTrends,
      areaPopularity,
      researcherTrends,
      timeRange,
      generatedAt: new Date()
    });
  } catch (error) {
    console.error('Error fetching research trends:', error);
    res.status(500).json({ error: 'Failed to fetch research trends' });
  }
});

// Get interdisciplinary connections
router.get('/:id/connections', async (req, res) => {
  try {
    const area = await ResearchArea.findById(req.params.id);
    if (!area) {
      return res.status(404).json({ error: 'Research area not found' });
    }
    
    // Find papers that cross multiple research areas
    const crossAreaPapers = await Paper.aggregate([
      { $match: { researchArea: area.name } },
      {
        $lookup: {
          from: 'papers',
          let: { currentId: '$_id', authors: '$authors' },
          pipeline: [
            {
              $match: {
                $expr: {
                  $and: [
                    { $ne: ['$_id', '$$currentId'] },
                    { $ne: ['$researchArea', area.name] },
                    { 
                      $or: [
                        { 'authors.user': { $in: '$$authors.user' } },
                        { 'authors.email': { $in: '$$authors.email' } }
                      ]
                    }
                  ]
                }
              }
            }
          ],
          as: 'crossAreaPapers'
        }
      },
      { $match: { 'crossAreaPapers.0': { $exists: true } } },
      {
        $group: {
          _id: '$researchArea',
          collaborationCount: { $sum: 1 },
          examplePapers: { $push: { title: '$title', authors: '$authors' } }
        }
      },
      { $sort: { collaborationCount: -1 } },
      { $limit: 10 }
    ]);
    
    res.json({
      area: area.name,
      connections: crossAreaPapers,
      interdisciplinaryScore: area.interdisciplinaryConnections
    });
  } catch (error) {
    console.error('Error fetching interdisciplinary connections:', error);
    res.status(500).json({ error: 'Failed to fetch interdisciplinary connections' });
  }
});

// Update area statistics (automated job)
router.post('/:id/update-stats', auth, requireRole(['admin']), async (req, res) => {
  try {
    const area = await ResearchArea.findById(req.params.id);
    if (!area) {
      return res.status(404).json({ error: 'Research area not found' });
    }
    
    // Calculate updated statistics
    const papers = await Paper.find({ researchArea: area.name });
    const researchers = await User.find({ 
      'researchAreas.name': area.name,
      role: 'researcher'
    });
    
    area.statistics.totalPapers = papers.length;
    area.statistics.totalResearchers = researchers.length;
    area.statistics.acceptanceRate = papers.length > 0 ? 
      (papers.filter(p => p.status === 'accepted').length / papers.length * 100) : 0;
    
    area.activePapers = papers.filter(p => p.status === 'under_review').length;
    area.activeResearchers = researchers.filter(r => 
      r.lastLogin && (new Date() - r.lastLogin) < 30 * 24 * 60 * 60 * 1000
    ).length;
    
    area.yearlyPublications = papers.filter(p => 
      p.createdAt >= new Date(new Date().getFullYear(), 0, 1)
    ).length;
    
    area.averageCitations = papers.length > 0 ? 
      papers.reduce((sum, p) => sum + (p.citations?.length || 0), 0) / papers.length : 0;
    
    await area.save();
    
    res.json({ 
      message: 'Statistics updated successfully',
      statistics: area.statistics 
    });
  } catch (error) {
    console.error('Error updating area statistics:', error);
    res.status(500).json({ error: 'Failed to update statistics' });
  }
});

// Search researchers by research area
router.get('/:name/researchers', async (req, res) => {
  try {
    const { page = 1, limit = 20, expertise, minHIndex } = req.query;
    
    const query = {
      'researchAreas.name': req.params.name,
      role: 'researcher',
      status: 'active'
    };
    
    if (expertise) {
      query['researchAreas.expertise'] = expertise;
    }
    
    if (minHIndex) {
      query['statistics.hIndex'] = { $gte: parseInt(minHIndex) };
    }
    
    const researchers = await User.find(query)
      .select('firstName lastName affiliation researchAreas statistics')
      .sort({ 'statistics.hIndex': -1, 'statistics.papersAccepted': -1 })
      .limit(limit * 1)
      .skip((page - 1) * limit)
      .exec();
    
    const total = await User.countDocuments(query);
    
    res.json({
      researchers,
      totalPages: Math.ceil(total / limit),
      currentPage: page,
      total
    });
  } catch (error) {
    console.error('Error fetching researchers:', error);
    res.status(500).json({ error: 'Failed to fetch researchers' });
  }
});

// Initialize default research areas
router.post('/initialize', async (req, res) => {
  try {
    const defaultAreas = [
      {
        name: 'Operating Systems',
        description: 'Research on operating system design, implementation, and optimization',
        subAreas: ['Kernel Design', 'Process Management', 'Memory Management', 'File Systems', 'Security'],
        keywords: ['kernel', 'scheduler', 'virtual memory', 'file system', 'process'],
        emergingTrends: ['Microkernel architectures', 'Real-time systems', 'Distributed operating systems'],
        topConferences: ['SOSP', 'OSDI', 'NSDI', 'EuroSys'],
        topJournals: ['ACM Transactions on Computer Systems', 'Operating Systems Review']
      },
      {
        name: 'Distributed Systems',
        description: 'Research on distributed computing, consensus, and fault tolerance',
        subAreas: ['Consensus Algorithms', 'Distributed Databases', 'Cloud Computing', 'Edge Computing'],
        keywords: ['consensus', 'replication', 'distributed database', 'fault tolerance'],
        emergingTrends: ['Blockchain consensus', 'Serverless computing', 'Edge AI'],
        topConferences: ['DISC', 'PODC', 'ICDCS', 'SRDS'],
        topJournals: ['Distributed Computing', 'IEEE Transactions on Parallel and Distributed Systems']
      },
      {
        name: 'System Security',
        description: 'Research on computer security, privacy, and trust',
        subAreas: ['Cryptography', 'Network Security', 'Software Security', 'Privacy Engineering'],
        keywords: ['cryptography', 'intrusion detection', 'privacy', 'vulnerability'],
        emergingTrends: ['Homomorphic encryption', 'Zero-trust architecture', 'Quantum cryptography'],
        topConferences: ['IEEE S&P', 'CCS', 'USENIX Security', 'NDSS'],
        topJournals: ['IEEE Security & Privacy', 'ACM Computing Surveys']
      },
      {
        name: 'Cloud Computing',
        description: 'Research on cloud infrastructure, services, and architectures',
        subAreas: ['Virtualization', 'Containerization', 'Serverless Computing', 'Multi-cloud'],
        keywords: ['virtualization', 'containers', 'serverless', 'auto-scaling'],
        emergingTrends: ['Function as a Service', 'Multi-cloud orchestration', 'Green cloud'],
        topConferences: ['IEEE Cloud', 'UCC', 'CloudCom'],
        topJournals: ['IEEE Cloud Computing', 'Journal of Cloud Computing']
      }
    ];
    
    const existingAreas = await ResearchArea.countDocuments();
    if (existingAreas > 0) {
      return res.status(409).json({ error: 'Research areas already exist' });
    }
    
    await ResearchArea.insertMany(defaultAreas);
    
    res.json({ 
      message: 'Default research areas initialized',
      count: defaultAreas.length 
    });
  } catch (error) {
    console.error('Error initializing research areas:', error);
    res.status(500).json({ error: 'Failed to initialize research areas' });
  }
});

module.exports = router;