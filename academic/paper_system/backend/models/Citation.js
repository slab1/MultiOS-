const mongoose = require('mongoose');

const citationSchema = new mongoose.Schema({
  // Citation Type and Source
  type: {
    type: String,
    enum: [
      'journal_article',
      'conference_paper',
      'book',
      'book_chapter',
      'thesis',
      'technical_report',
      'preprint',
      'dataset',
      'software',
      'patent',
      'webpage',
      'blog_post',
      'presentation',
      'other'
    ],
    required: true
  },
  
  // Basic Information
  title: {
    type: String,
    required: true,
    trim: true
  },
  authors: [{
    firstName: String,
    lastName: String,
    fullName: String,
    orcid: String,
    affiliations: [String],
    isCorrespondingAuthor: { type: Boolean, default: false }
  }],
  
  // Publication Details
  publication: {
    journal: String,
    conference: String,
    bookTitle: String,
    publisher: String,
    volume: String,
    issue: String,
    pages: {
      start: String,
      end: String
    },
    year: Number,
    month: String,
    day: Number,
    edition: String
  },
  
  // Identifiers
  identifiers: {
    doi: {
      type: String,
      index: true,
      unique: true,
      sparse: true
    },
    isbn: String,
    issn: String,
    arxivId: String,
    pubmedId: String,
    handle: String,
    url: String,
    pmid: String
  },
  
  // Abstract and Keywords
  abstract: String,
  keywords: [String],
  
  // BibTeX Entry
  bibtex: {
    key: String, // BibTeX citation key
    entry: String, // Complete BibTeX entry
    fields: {
      author: String,
      title: String,
      journal: String,
      booktitle: String,
      year: String,
      volume: String,
      number: String,
      pages: String,
      publisher: String,
      address: String,
      url: String,
      doi: String,
      isbn: String,
      issn: String,
      abstract: String,
      keywords: String
    }
  },
  
  // Research Paper Integration
  researchPapers: [{
    paper: {
      type: mongoose.Schema.Types.ObjectId,
      ref: 'Paper'
    },
    context: String, // How this citation is used in the paper
    relevance: {
      type: String,
      enum: ['background', 'related_work', 'methodology', 'results', 'discussion']
    }
  }],
  
  // Citation Metrics
  metrics: {
    totalCitations: { type: Number, default: 0 },
    selfCitations: { type: Number, default: 0 },
    googleScholarCitations: Number,
    semanticScholarCitations: Number,
    arxivCitations: Number,
    lastUpdated: Date
  },
  
  // Open Access and Licensing
  openAccess: {
    isOpenAccess: { type: Boolean, default: false },
    license: String,
    repository: String,
    version: String
  },
  
  // Review and Quality
  quality: {
    isVerified: { type: Boolean, default: false },
    verificationDate: Date,
    verifiedBy: {
      type: mongoose.Schema.Types.ObjectId,
      ref: 'User'
    },
    hasFullText: { type: Boolean, default: false },
    retrievalDate: Date,
    qualityScore: {
      type: Number,
      min: 0,
      max: 10,
      default: 0
    }
  },
  
  // Metadata
  addedBy: {
    type: mongoose.Schema.Types.ObjectId,
    ref: 'User'
  },
  addedDate: { type: Date, default: Date.now },
  lastModified: Date,
  
  // Research Impact
  impact: {
    journalImpactFactor: Number,
    citationVelocity: Number, // Citations per year
    altmetricsScore: Number,
    hIndex: Number,
    i10Index: Number
  },
  
  // Relationships
  relatedCitations: [{
    citation: {
      type: mongoose.Schema.Types.ObjectId,
      ref: 'Citation'
    },
    relationship: {
      type: String,
      enum: ['cites', 'cited_by', 'related', 'supplemental', 'contrasting']
    },
    strength: {
      type: Number,
      min: 0,
      max: 1
    }
  }],
  
  // System Fields
  isActive: { type: Boolean, default: true },
  tags: [String], // User-defined tags for organization
  notes: [{
    content: String,
    author: {
      type: mongoose.Schema.Types.ObjectId,
      ref: 'User'
    },
    createdAt: { type: Date, default: Date.now }
  }]
}, {
  timestamps: true,
  toJSON: { virtuals: true },
  toObject: { virtuals: true }
});

// Virtual for formatted citation
citationSchema.virtual('formattedCitation').get(function() {
  const authors = this.authors.map(author => author.fullName || `${author.firstName} ${author.lastName}`).join(', ');
  const year = this.publication.year || 'n.d.';
  
  switch (this.type) {
    case 'journal_article':
      return `${authors} (${year}). ${this.title}. ${this.publication.journal}`;
    case 'conference_paper':
      return `${authors} (${year}). ${this.title}. In ${this.publication.conference}`;
    case 'book':
      return `${authors} (${year}). ${this.title}. ${this.publication.publisher}`;
    case 'book_chapter':
      return `${authors} (${year}). ${this.title}. In ${this.publication.bookTitle}. ${this.publication.publisher}`;
    default:
      return `${authors} (${year}). ${this.title}.`;
  }
});

// Virtual for citation key (for BibTeX)
citationSchema.virtual('citationKey').get(function() {
  if (this.bibtex.key) return this.bibtex.key;
  
  // Generate key from first author and year
  const firstAuthor = this.authors[0]?.lastName || 'unknown';
  const year = this.publication.year || 'nodate';
  let titlePart = this.title.split(' ')[0].toLowerCase().replace(/[^a-z0-9]/g, '');
  
  return `${firstAuthor}${year}${titlePart}`;
});

// Virtual for Google Scholar URL
citationSchema.virtual('googleScholarUrl').get(function() {
  if (this.identifiers.arxivId) {
    return `https://scholar.google.com/scholar?q=arxiv:${this.identifiers.arxivId}`;
  }
  if (this.identifiers.doi) {
    const encodedDoi = encodeURIComponent(this.identifiers.doi);
    return `https://scholar.google.com/scholar?q=doi:${encodedDoi}`;
  }
  return `https://scholar.google.com/scholar?q="${encodeURIComponent(this.title)}"`;
});

// Pre-save middleware
citationSchema.pre('save', function(next) {
  this.lastModified = new Date();
  
  // Auto-generate BibTeX if not provided
  if (!this.bibtex.entry && this.type) {
    this.generateBibtex();
  }
  
  // Generate citation key
  if (!this.bibtex.key) {
    this.bibtex.key = this.citationKey;
  }
  
  next();
});

// Methods
citationSchema.methods.generateBibtex = function() {
  const key = this.citationKey;
  const fields = {};
  
  // Map fields based on type
  switch (this.type) {
    case 'journal_article':
      fields['@article'] = key;
      fields.journal = this.publication.journal || '';
      fields.volume = this.publication.volume || '';
      fields.number = this.publication.issue || '';
      fields.pages = this.publication.pages ? 
        `${this.publication.pages.start}-${this.publication.pages.end}` : '';
      break;
    case 'conference_paper':
      fields['@inproceedings'] = key;
      fields.booktitle = this.publication.conference || '';
      fields.pages = this.publication.pages ? 
        `${this.publication.pages.start}-${this.publication.pages.end}` : '';
      break;
    case 'book':
      fields['@book'] = key;
      fields.publisher = this.publication.publisher || '';
      fields.address = this.publication.address || '';
      fields.edition = this.publication.edition || '';
      break;
    case 'book_chapter':
      fields['@incollection'] = key;
      fields.booktitle = this.publication.bookTitle || '';
      fields.publisher = this.publication.publisher || '';
      fields.pages = this.publication.pages ? 
        `${this.publication.pages.start}-${this.publication.pages.end}` : '';
      break;
    default:
      fields['@misc'] = key;
  }
  
  // Add common fields
  fields.author = this.authors.map(author => 
    author.fullName || `${author.firstName} ${author.lastName}`
  ).join(' and ');
  fields.title = this.title;
  fields.year = this.publication.year?.toString() || '';
  
  // Add optional fields if available
  if (this.identifiers.doi) fields.doi = this.identifiers.doi;
  if (this.identifiers.isbn) fields.isbn = this.identifiers.isbn;
  if (this.identifiers.url) fields.url = this.identifiers.url;
  if (this.abstract) fields.abstract = this.abstract;
  if (this.keywords.length > 0) fields.keywords = this.keywords.join(', ');
  
  // Generate BibTeX string
  let entry = fields['@' + key.split(key)[0]] || '@misc';
  delete fields['@' + key.split(key)[0]];
  
  this.bibtex.entry = `${entry}{${key},\n` +
    Object.entries(fields)
      .filter(([_, value]) => value)
      .map(([field, value]) => `  ${field} = {${value}}`)
      .join(',\n') + '\n}';
  
  this.bibtex.fields = fields;
};

citationSchema.methods.addRelatedCitation = function(relatedCitationId, relationship, strength = 0.5) {
  const existingRelation = this.relatedCitations.find(
    rel => rel.citation.toString() === relatedCitationId.toString()
  );
  
  if (existingRelation) {
    existingRelation.relationship = relationship;
    existingRelation.strength = strength;
  } else {
    this.relatedCitations.push({
      citation: relatedCitationId,
      relationship,
      strength
    });
  }
  
  return this.save();
};

citationSchema.methods.updateMetrics = function(metrics) {
  this.metrics = { ...this.metrics, ...metrics, lastUpdated: new Date() };
  return this.save();
};

citationSchema.methods.verify = function(verifiedBy) {
  this.quality.isVerified = true;
  this.quality.verificationDate = new Date();
  this.quality.verifiedBy = verifiedBy;
  return this.save();
};

citationSchema.methods.addNote = function(content, authorId) {
  this.notes.push({
    content,
    author: authorId,
    createdAt: new Date()
  });
  return this.save();
};

// Static methods
citationSchema.statics.findByDOI = function(doi) {
  return this.findOne({ 'identifiers.doi': doi });
};

citationSchema.statics.findByArxivId = function(arxivId) {
  return this.findOne({ 'identifiers.arxivId': arxivId });
};

citationSchema.statics.searchByTitle = function(title, limit = 10) {
  return this.find({
    title: { $regex: title, $options: 'i' },
    isActive: true
  })
  .sort({ 'publication.year': -1 })
  .limit(limit);
};

citationSchema.statics.getCitationMetrics = function() {
  return this.aggregate([
    { $match: { isActive: true } },
    {
      $group: {
        _id: null,
        totalCitations: { $sum: '$metrics.totalCitations' },
        averageCitations: { $avg: '$metrics.totalCitations' },
        totalWorks: { $sum: 1 },
        verifiedWorks: {
          $sum: { $cond: ['$quality.isVerified', 1, 0] }
        }
      }
    }
  ]);
};

citationSchema.statics.getMostCited = function(limit = 10) {
  return this.find({ isActive: true })
    .sort({ 'metrics.totalCitations': -1 })
    .limit(limit)
    .populate('researchPapers.paper', 'title');
};

citationSchema.statics.importFromBibtex = function(bibtexEntry, addedBy) {
  // Basic BibTeX parsing (simplified)
  const bibtexRegex = /@(\w+)\{([^,]+),\s*([\s\S]*?)\}/g;
  const match = bibtexRegex.exec(bibtexEntry);
  
  if (!match) {
    throw new Error('Invalid BibTeX format');
  }
  
  const [, type, key, content] = match;
  
  // Parse fields
  const fieldRegex = /(\w+)\s*=\s*\{([^}]*)\}/g;
  const fields = {};
  let fieldMatch;
  
  while ((fieldMatch = fieldRegex.exec(content)) !== null) {
    fields[fieldMatch[1].toLowerCase()] = fieldMatch[2];
  }
  
  // Create citation object
  const citation = new this({
    type: this.mapBibtexType(type),
    bibtex: { key, entry: bibtexEntry },
    addedBy
  });
  
  // Map fields
  if (fields.title) citation.title = fields.title;
  if (fields.author) {
    citation.authors = fields.author.split(' and ').map(authorStr => {
      const parts = authorStr.trim().split(', ');
      if (parts.length === 2) {
        return { lastName: parts[0], firstName: parts[1] };
      }
      const nameParts = authorStr.trim().split(' ');
      return {
        firstName: nameParts.slice(0, -1).join(' '),
        lastName: nameParts.slice(-1).join(' ')
      };
    });
  }
  if (fields.year) citation.publication.year = parseInt(fields.year);
  if (fields.journal) citation.publication.journal = fields.journal;
  if (fields.booktitle) citation.publication.conference = fields.booktitle;
  if (fields.publisher) citation.publication.publisher = fields.publisher;
  if (fields.volume) citation.publication.volume = fields.volume;
  if (fields.number) citation.publication.issue = fields.number;
  if (fields.pages) {
    const pageParts = fields.pages.split('-');
    citation.publication.pages = {
      start: pageParts[0],
      end: pageParts[1] || pageParts[0]
    };
  }
  if (fields.doi) citation.identifiers.doi = fields.doi;
  if (fields.isbn) citation.identifiers.isbn = fields.isbn;
  if (fields.url) citation.identifiers.url = fields.url;
  if (fields.abstract) citation.abstract = fields.abstract;
  if (fields.keywords) citation.keywords = fields.keywords.split(',').map(k => k.trim());
  
  return citation.save();
};

citationSchema.statics.mapBibtexType = function(bibtexType) {
  const mapping = {
    'article': 'journal_article',
    'inproceedings': 'conference_paper',
    'book': 'book',
    'incollection': 'book_chapter',
    'phdthesis': 'thesis',
    'mastersthesis': 'thesis',
    'techreport': 'technical_report',
    'misc': 'other'
  };
  
  return mapping[bibtexType.toLowerCase()] || 'other';
};

// Indexes
citationSchema.index({ 'identifiers.doi': 1 }, { sparse: true });
citationSchema.index({ 'identifiers.arxivId': 1 }, { sparse: true });
citationSchema.index({ title: 'text', abstract: 'text', keywords: 'text' });
citationSchema.index({ type: 1 });
citationSchema.index({ 'publication.year': -1 });
citationSchema.index({ 'metrics.totalCitations': -1 });
citationSchema.index({ researchPapers: 1 });
citationSchema.index({ isActive: 1 });

module.exports = mongoose.model('Citation', citationSchema);