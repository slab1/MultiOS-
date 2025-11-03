const express = require('express');
const Citation = require('../models/Citation');
const Paper = require('../models/Paper');
const { auth, requireRole } = require('../middleware/auth');
const Joi = require('joi');

const router = express.Router();

// Validation schemas
const createCitationSchema = Joi.object({
  type: Joi.string().valid(
    'journal_article', 'conference_paper', 'book', 'book_chapter',
    'thesis', 'technical_report', 'preprint', 'dataset', 'software',
    'patent', 'webpage', 'blog_post', 'presentation', 'other'
  ).required(),
  title: Joi.string().max(500).required(),
  authors: Joi.array().items(Joi.object({
    firstName: Joi.string().trim(),
    lastName: Joi.string().trim(),
    fullName: Joi.string().trim(),
    orcid: Joi.string(),
    affiliations: Joi.array().items(Joi.string())
  })).min(1).required(),
  publication: Joi.object({
    journal: Joi.string().trim(),
    conference: Joi.string().trim(),
    bookTitle: Joi.string().trim(),
    publisher: Joi.string().trim(),
    volume: Joi.string().trim(),
    issue: Joi.string().trim(),
    pages: Joi.object({
      start: Joi.string().trim(),
      end: Joi.string().trim()
    }),
    year: Joi.number().integer().min(1000).max(2030),
    month: Joi.string().trim(),
    day: Joi.number().integer().min(1).max(31),
    edition: Joi.string().trim()
  }),
  identifiers: Joi.object({
    doi: Joi.string().trim(),
    isbn: Joi.string().trim(),
    issn: Joi.string().trim(),
    arxivId: Joi.string().trim(),
    pubmedId: Joi.string().trim(),
    handle: Joi.string().trim(),
    url: Joi.string().uri(),
    pmid: Joi.string().trim()
  }),
  abstract: Joi.string(),
  keywords: Joi.array().items(Joi.string().trim()),
  tags: Joi.array().items(Joi.string().trim())
});

const updateCitationSchema = Joi.object({
  title: Joi.string().max(500),
  authors: Joi.array().items(Joi.object({
    firstName: Joi.string().trim(),
    lastName: Joi.string().trim(),
    fullName: Joi.string().trim(),
    orcid: Joi.string(),
    affiliations: Joi.array().items(Joi.string())
  })),
  publication: Joi.object({
    journal: Joi.string().trim(),
    conference: Joi.string().trim(),
    bookTitle: Joi.string().trim(),
    publisher: Joi.string().trim(),
    volume: Joi.string().trim(),
    issue: Joi.string().trim(),
    pages: Joi.object({
      start: Joi.string().trim(),
      end: Joi.string().trim()
    }),
    year: Joi.number().integer().min(1000).max(2030),
    month: Joi.string().trim(),
    day: Joi.number().integer().min(1).max(31),
    edition: Joi.string().trim()
  }),
  identifiers: Joi.object({
    doi: Joi.string().trim(),
    isbn: Joi.string().trim(),
    issn: Joi.string().trim(),
    arxivId: Joi.string().trim(),
    pubmedId: Joi.string().trim(),
    handle: Joi.string().trim(),
    url: Joi.string().uri(),
    pmid: Joi.string().trim()
  }),
  abstract: Joi.string(),
  keywords: Joi.array().items(Joi.string().trim()),
  tags: Joi.array().items(Joi.string().trim())
});

// Create new citation
router.post('/', auth, async (req, res) => {
  try {
    // Validate input
    const { error, value } = createCitationSchema.validate(req.body);
    if (error) {
      return res.status(400).json({ 
        error: 'Validation failed',
        details: error.details.map(detail => detail.message)
      });
    }

    // Check for duplicate DOI or arXiv ID
    if (value.identifiers?.doi) {
      const existingCitation = await Citation.findOne({ 'identifiers.doi': value.identifiers.doi });
      if (existingCitation) {
        return res.status(409).json({ 
          error: 'Duplicate citation',
          message: 'A citation with this DOI already exists'
        });
      }
    }

    if (value.identifiers?.arxivId) {
      const existingCitation = await Citation.findOne({ 'identifiers.arxivId': value.identifiers.arxivId });
      if (existingCitation) {
        return res.status(409).json({ 
          error: 'Duplicate citation',
          message: 'A citation with this arXiv ID already exists'
        });
      }
    }

    // Create citation
    const citation = new Citation({
      ...value,
      addedBy: req.user.userId,
      quality: {
        isVerified: false,
        hasFullText: false,
        qualityScore: 0
      },
      metrics: {
        totalCitations: 0,
        selfCitations: 0
      },
      openAccess: {
        isOpenAccess: false
      },
      researchPapers: [],
      relatedCitations: [],
      notes: []
    });

    await citation.save();

    res.status(201).json({
      message: 'Citation created successfully',
      citation: {
        id: citation._id,
        type: citation.type,
        title: citation.title,
        authors: citation.authors,
        publication: citation.publication,
        identifiers: citation.identifiers,
        formattedCitation: citation.formattedCitation,
        bibtex: citation.bibtex.entry,
        addedDate: citation.addedDate
      }
    });

  } catch (error) {
    console.error('Create citation error:', error);
    res.status(500).json({ 
      error: 'Failed to create citation',
      message: 'An error occurred while creating the citation'
    });
  }
});

// Get user's citations
router.get('/my-citations', auth, async (req, res) => {
  try {
    const page = parseInt(req.query.page) || 1;
    const limit = parseInt(req.query.limit) || 20;
    const skip = (page - 1) * limit;

    const { type, year, search, sortBy = 'addedDate', sortOrder = 'desc' } = req.query;

    // Build query
    const query = { addedBy: req.user.userId, isActive: true };

    if (type) {
      query.type = type;
    }

    if (year) {
      query['publication.year'] = parseInt(year);
    }

    if (search) {
      query.$or = [
        { title: { $regex: search, $options: 'i' } },
        { abstract: { $regex: search, $options: 'i' } },
        { keywords: { $regex: search, $options: 'i' } },
        { 'authors.fullName': { $regex: search, $options: 'i' } },
        { 'publication.journal': { $regex: search, $options: 'i' } },
        { 'publication.conference': { $regex: search, $options: 'i' } }
      ];
    }

    // Build sort object
    const sortObj = {};
    sortObj[sortBy] = sortOrder === 'desc' ? -1 : 1;

    const citations = await Citation.find(query)
      .sort(sortObj)
      .skip(skip)
      .limit(limit);

    const total = await Citation.countDocuments(query);

    res.json({
      citations: citations.map(citation => ({
        id: citation._id,
        type: citation.type,
        title: citation.title,
        authors: citation.authors,
        publication: citation.publication,
        identifiers: citation.identifiers,
        formattedCitation: citation.formattedCitation,
        bibtex: citation.bibtex.entry,
        abstract: citation.abstract,
        keywords: citation.keywords,
        tags: citation.tags,
        metrics: {
          totalCitations: citation.metrics.totalCitations,
          selfCitations: citation.metrics.selfCitations
        },
        quality: {
          isVerified: citation.quality.isVerified,
          hasFullText: citation.quality.hasFullText,
          qualityScore: citation.quality.qualityScore
        },
        openAccess: citation.openAccess,
        addedDate: citation.addedDate
      })),
      pagination: {
        current: page,
        total: Math.ceil(total / limit),
        count: citations.length,
        totalRecords: total
      }
    });

  } catch (error) {
    console.error('Get my citations error:', error);
    res.status(500).json({ 
      error: 'Failed to fetch citations',
      message: 'An error occurred while fetching citations'
    });
  }
});

// Get single citation
router.get('/:id', auth, async (req, res) => {
  try {
    const citation = await Citation.findById(req.params.id)
      .populate('addedBy', 'firstName lastName')
      .populate('quality.verifiedBy', 'firstName lastName')
      .populate('researchPapers.paper', 'title authors')
      .populate('relatedCitations.citation', 'title authors publication.year')
      .populate('notes.author', 'firstName lastName');

    if (!citation) {
      return res.status(404).json({ error: 'Citation not found' });
    }

    // Check access permissions
    const isOwner = citation.addedBy._id.toString() === req.user.userId;
    const isEditor = req.user.role === 'editor' || req.user.role === 'admin';

    if (!isOwner && !isEditor) {
      return res.status(403).json({ error: 'Access denied' });
    }

    res.json({
      citation: {
        id: citation._id,
        type: citation.type,
        title: citation.title,
        authors: citation.authors,
        publication: citation.publication,
        identifiers: citation.identifiers,
        abstract: citation.abstract,
        keywords: citation.keywords,
        formattedCitation: citation.formattedCitation,
        citationKey: citation.citationKey,
        bibtex: {
          key: citation.bibtex.key,
          entry: citation.bibtex.entry
        },
        researchPapers: citation.researchPapers.map(rp => ({
          paper: rp.paper,
          context: rp.context,
          relevance: rp.relevance
        })),
        metrics: citation.metrics,
        impact: citation.impact,
        openAccess: citation.openAccess,
        quality: {
          isVerified: citation.quality.isVerified,
          verificationDate: citation.quality.verificationDate,
          verifiedBy: citation.quality.verifiedBy,
          hasFullText: citation.quality.hasFullText,
          retrievalDate: citation.quality.retrievalDate,
          qualityScore: citation.quality.qualityScore
        },
        relatedCitations: citation.relatedCitations.map(rc => ({
          citation: rc.citation,
          relationship: rc.relationship,
          strength: rc.strength
        })),
        notes: citation.notes.map(note => ({
          content: note.content,
          author: note.author,
          createdAt: note.createdAt
        })),
        tags: citation.tags,
        addedBy: citation.addedBy,
        addedDate: citation.addedDate,
        lastModified: citation.lastModified
      }
    });

  } catch (error) {
    console.error('Get citation error:', error);
    res.status(500).json({ 
      error: 'Failed to fetch citation',
      message: 'An error occurred while fetching the citation'
    });
  }
});

// Update citation
router.put('/:id', auth, async (req, res) => {
  try {
    const citation = await Citation.findById(req.params.id);

    if (!citation) {
      return res.status(404).json({ error: 'Citation not found' });
    }

    // Check ownership
    if (citation.addedBy.toString() !== req.user.userId) {
      return res.status(403).json({ error: 'Access denied' });
    }

    // Validate input
    const { error, value } = updateCitationSchema.validate(req.body);
    if (error) {
      return res.status(400).json({ 
        error: 'Validation failed',
        details: error.details.map(detail => detail.message)
      });
    }

    // Update citation fields
    Object.keys(value).forEach(key => {
      if (value[key] !== undefined) {
        citation[key] = value[key];
      }
    });

    await citation.save();

    res.json({
      message: 'Citation updated successfully',
      citation: {
        id: citation._id,
        title: citation.title,
        formattedCitation: citation.formattedCitation,
        lastModified: citation.lastModified
      }
    });

  } catch (error) {
    console.error('Update citation error:', error);
    res.status(500).json({ 
      error: 'Failed to update citation',
      message: 'An error occurred while updating the citation'
    });
  }
});

// Import from BibTeX
router.post('/import/bibtex', auth, async (req, res) => {
  try {
    const { bibtex, citationIds } = req.body;

    if (!bibtex) {
      return res.status(400).json({ 
        error: 'Missing BibTeX',
        message: 'BibTeX entry is required'
      });
    }

    const results = [];
    const errors = [];

    try {
      const citation = await Citation.importFromBibtex(bibtex, req.user.userId);
      
      if (citationIds && Array.isArray(citationIds)) {
        // Link to papers if provided
        for (const paperId of citationIds) {
          const paper = await Paper.findById(paperId);
          if (paper && paper.createdBy.toString() === req.user.userId) {
            citation.researchPapers.push({
              paper: paperId,
              context: 'Imported from BibTeX',
              relevance: 'background'
            });
          }
        }
        await citation.save();
      }

      results.push({
        id: citation._id,
        title: citation.title,
        formattedCitation: citation.formattedCitation
      });

    } catch (bibtexError) {
      errors.push({
        error: 'Invalid BibTeX format',
        message: bibtexError.message,
        input: bibtex
      });
    }

    res.status(201).json({
      imported: results.length,
      errors: errors.length,
      citations: results,
      errorDetails: errors
    });

  } catch (error) {
    console.error('Import BibTeX error:', error);
    res.status(500).json({ 
      error: 'Failed to import BibTeX',
      message: 'An error occurred while importing the BibTeX entry'
    });
  }
});

// Search citations
router.get('/', auth, async (req, res) => {
  try {
    const page = parseInt(req.query.page) || 1;
    const limit = parseInt(req.query.limit) || 20;
    const skip = (page - 1) * limit;

    const { 
      query,
      type,
      year,
      author,
      journal,
      conference,
      hasDOI,
      openAccess,
      verified,
      sortBy = 'publication.year',
      sortOrder = 'desc'
    } = req.query;

    // Build search query
    const searchQuery = { isActive: true };

    if (query) {
      searchQuery.$text = { $search: query };
    }

    if (type) {
      searchQuery.type = type;
    }

    if (year) {
      searchQuery['publication.year'] = parseInt(year);
    }

    if (author) {
      searchQuery['authors.fullName'] = { $regex: author, $options: 'i' };
    }

    if (journal) {
      searchQuery['publication.journal'] = { $regex: journal, $options: 'i' };
    }

    if (conference) {
      searchQuery['publication.conference'] = { $regex: conference, $options: 'i' };
    }

    if (hasDOI === 'true') {
      searchQuery['identifiers.doi'] = { $exists: true, $ne: null };
    }

    if (openAccess === 'true') {
      searchQuery['openAccess.isOpenAccess'] = true;
    }

    if (verified === 'true') {
      searchQuery['quality.isVerified'] = true;
    }

    // Build sort object
    const sortObj = {};
    sortObj[sortBy] = sortOrder === 'desc' ? -1 : 1;

    const citations = await Citation.find(searchQuery)
      .sort(sortObj)
      .skip(skip)
      .limit(limit);

    const total = await Citation.countDocuments(searchQuery);

    res.json({
      citations: citations.map(citation => ({
        id: citation._id,
        type: citation.type,
        title: citation.title,
        authors: citation.authors,
        publication: citation.publication,
        formattedCitation: citation.formattedCitation,
        googleScholarUrl: citation.googleScholarUrl,
        identifiers: {
          doi: citation.identifiers.doi,
          arxivId: citation.identifiers.arxivId
        },
        metrics: {
          totalCitations: citation.metrics.totalCitations
        },
        openAccess: citation.openAccess,
        quality: {
          isVerified: citation.quality.isVerified,
          hasFullText: citation.quality.hasFullText
        }
      })),
      pagination: {
        current: page,
        total: Math.ceil(total / limit),
        count: citations.length,
        totalRecords: total
      }
    });

  } catch (error) {
    console.error('Search citations error:', error);
    res.status(500).json({ 
      error: 'Failed to search citations',
      message: 'An error occurred while searching citations'
    });
  }
});

// Link citation to paper
router.post('/:id/link-to-paper', auth, async (req, res) => {
  try {
    const { paperId, context, relevance } = req.body;

    if (!paperId) {
      return res.status(400).json({ 
        error: 'Missing paper ID',
        message: 'Paper ID is required'
      });
    }

    const citation = await Citation.findById(req.params.id);
    const paper = await Paper.findById(paperId);

    if (!citation) {
      return res.status(404).json({ error: 'Citation not found' });
    }

    if (!paper) {
      return res.status(404).json({ error: 'Paper not found' });
    }

    // Check if user owns the paper
    if (paper.createdBy.toString() !== req.user.userId) {
      return res.status(403).json({ error: 'Access denied' });
    }

    // Check if citation is already linked to this paper
    const existingLink = citation.researchPapers.find(
      rp => rp.paper.toString() === paperId
    );

    if (existingLink) {
      return res.status(400).json({ 
        error: 'Already linked',
        message: 'This citation is already linked to the specified paper'
      });
    }

    // Add citation to paper's bibliography
    if (!paper.bibliography.includes(citation._id.toString())) {
      paper.bibliography.push({
        type: citation.bibtex.entry,
        citationId: citation._id
      });
      await paper.save();
    }

    // Link citation to paper
    citation.researchPapers.push({
      paper: paperId,
      context: context || 'User added',
      relevance: relevance || 'background'
    });

    await citation.save();

    res.json({
      message: 'Citation linked to paper successfully',
      citation: {
        id: citation._id,
        title: citation.title
      },
      paper: {
        id: paper._id,
        title: paper.title
      }
    });

  } catch (error) {
    console.error('Link citation error:', error);
    res.status(500).json({ 
      error: 'Failed to link citation',
      message: 'An error occurred while linking the citation to the paper'
    });
  }
});

// Add related citation
router.post('/:id/related', auth, async (req, res) => {
  try {
    const { relatedCitationId, relationship, strength } = req.body;

    if (!relatedCitationId || !relationship) {
      return res.status(400).json({ 
        error: 'Missing required fields',
        message: 'Related citation ID and relationship type are required'
      });
    }

    const citation = await Citation.findById(req.params.id);
    const relatedCitation = await Citation.findById(relatedCitationId);

    if (!citation) {
      return res.status(404).json({ error: 'Citation not found' });
    }

    if (!relatedCitation) {
      return res.status(404).json({ error: 'Related citation not found' });
    }

    // Check ownership
    if (citation.addedBy.toString() !== req.user.userId) {
      return res.status(403).json({ error: 'Access denied' });
    }

    await citation.addRelatedCitation(relatedCitationId, relationship, strength);

    res.json({
      message: 'Related citation added successfully',
      relatedCitation: {
        id: relatedCitation._id,
        title: relatedCitation.title
      },
      relationship,
      strength
    });

  } catch (error) {
    console.error('Add related citation error:', error);
    res.status(500).json({ 
      error: 'Failed to add related citation',
      message: 'An error occurred while adding the related citation'
    });
  }
});

// Add note to citation
router.post('/:id/notes', auth, async (req, res) => {
  try {
    const { content } = req.body;

    if (!content) {
      return res.status(400).json({ 
        error: 'Missing note content',
        message: 'Note content is required'
      });
    }

    const citation = await Citation.findById(req.params.id);

    if (!citation) {
      return res.status(404).json({ error: 'Citation not found' });
    }

    // Check ownership
    if (citation.addedBy.toString() !== req.user.userId) {
      return res.status(403).json({ error: 'Access denied' });
    }

    await citation.addNote(content, req.user.userId);

    res.status(201).json({
      message: 'Note added successfully',
      note: {
        content,
        author: req.user.userId,
        createdAt: new Date()
      }
    });

  } catch (error) {
    console.error('Add note error:', error);
    res.status(500).json({ 
      error: 'Failed to add note',
      message: 'An error occurred while adding the note'
    });
  }
});

// Verify citation
router.post('/:id/verify', auth, requireRole(['editor', 'admin']), async (req, res) => {
  try {
    const citation = await Citation.findById(req.params.id);

    if (!citation) {
      return res.status(404).json({ error: 'Citation not found' });
    }

    await citation.verify(req.user.userId);

    res.json({
      message: 'Citation verified successfully',
      citation: {
        id: citation._id,
        quality: {
          isVerified: citation.quality.isVerified,
          verificationDate: citation.quality.verificationDate
        }
      }
    });

  } catch (error) {
    console.error('Verify citation error:', error);
    res.status(500).json({ 
      error: 'Failed to verify citation',
      message: 'An error occurred while verifying the citation'
    });
  }
});

// Update citation metrics
router.post('/:id/metrics', auth, requireRole(['editor', 'admin']), async (req, res) => {
  try {
    const metrics = req.body;

    const citation = await Citation.findById(req.params.id);

    if (!citation) {
      return res.status(404).json({ error: 'Citation not found' });
    }

    await citation.updateMetrics(metrics);

    res.json({
      message: 'Citation metrics updated successfully',
      metrics: citation.metrics
    });

  } catch (error) {
    console.error('Update metrics error:', error);
    res.status(500).json({ 
      error: 'Failed to update metrics',
      message: 'An error occurred while updating the metrics'
    });
  }
});

// Delete citation
router.delete('/:id', auth, async (req, res) => {
  try {
    const citation = await Citation.findById(req.params.id);

    if (!citation) {
      return res.status(404).json({ error: 'Citation not found' });
    }

    // Check ownership
    if (citation.addedBy.toString() !== req.user.userId) {
      return res.status(403).json({ error: 'Access denied' });
    }

    // Soft delete
    citation.isActive = false;
    await citation.save();

    res.json({ message: 'Citation deleted successfully' });

  } catch (error) {
    console.error('Delete citation error:', error);
    res.status(500).json({ 
      error: 'Failed to delete citation',
      message: 'An error occurred while deleting the citation'
    });
  }
});

module.exports = router;