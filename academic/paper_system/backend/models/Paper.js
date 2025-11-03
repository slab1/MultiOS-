const mongoose = require('mongoose');

const authorSchema = new mongoose.Schema({
  user: {
    type: mongoose.Schema.Types.ObjectId,
    ref: 'User'
  },
  name: String,
  email: String,
  affiliation: String,
  isCorresponding: { type: Boolean, default: false },
  contribution: String,
  orcid: String,
  position: { type: Number, default: 0 } // Author order
}, { _id: false });

const fileSchema = new mongoose.Schema({
  filename: String,
  originalName: String,
  path: String,
  type: {
    type: String,
    enum: ['latex', 'pdf', 'supplementary', 'data', 'code']
  },
  size: Number,
  uploadDate: { type: Date, default: Date.now },
  compiledPdf: String, // Path to compiled PDF for LaTeX
  checksum: String
}, { _id: false });

const experimentSchema = new mongoose.Schema({
  name: String,
  description: String,
  code: String,
  dataSet: String,
  parameters: mongoose.Schema.Types.Mixed,
  results: mongoose.Schema.Types.Mixed,
  reproducibilityScore: {
    type: Number,
    min: 0,
    max: 10
  },
  validationStatus: {
    type: String,
    enum: ['pending', 'validating', 'valid', 'invalid', 'error'],
    default: 'pending'
  },
  validationResults: [{
    validator: {
      type: mongoose.Schema.Types.ObjectId,
      ref: 'User'
    },
    status: String,
    notes: String,
    date: Date
  }]
}, { _id: true });

const paperSchema = new mongoose.Schema({
  // Basic Information
  title: {
    type: String,
    required: true,
    trim: true,
    maxlength: 300
  },
  abstract: {
    type: String,
    required: true,
    maxlength: 5000
  },
  keywords: [String],
  
  // Authors
  authors: [authorSchema],
  correspondingAuthor: {
    type: mongoose.Schema.Types.ObjectId,
    ref: 'User'
  },
  
  // Content and Files
  content: {
    latex: String, // LaTeX source
    compiledPdf: String, // Compiled PDF path
    wordCount: Number,
    pageCount: Number
  },
  files: [fileSchema],
  
  // Research Details
  researchArea: {
    type: String,
    required: true,
    enum: [
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
    ]
  },
  methodology: {
    type: String,
    enum: ['theoretical', 'experimental', 'simulation', 'survey', 'case_study', 'mixed']
  },
  experiments: [experimentSchema],
  
  // Submission Status
  status: {
    type: String,
    enum: [
      'draft',
      'submitted',
      'under_review',
      'revision_requested',
      'accepted',
      'rejected',
      'withdrawn',
      'published'
    ],
    default: 'draft'
  },
  submissionDate: Date,
  reviewCycle: {
    current: { type: Number, default: 0 },
    total: { type: Number, default: 3 },
    maximum: { type: Number, default: 3 }
  },
  
  // Version Control
  version: {
    type: Number,
    default: 1
  },
  parentPaper: {
    type: mongoose.Schema.Types.ObjectId,
    ref: 'Paper'
  },
  changeLog: [{
    version: Number,
    date: Date,
    author: {
      type: mongoose.Schema.Types.ObjectId,
      ref: 'User'
    },
    changes: String
  }],
  
  // Conference/Workshop Submission
  submissionTarget: {
    type: {
      type: String,
      enum: ['conference', 'journal', 'workshop', 'preprint']
    },
    conference: {
      type: mongoose.Schema.Types.ObjectId,
      ref: 'Conference'
    },
    track: String,
    submissionDeadline: Date,
    notificationDate: Date,
    finalVersionDue: Date
  },
  
  // Citations and References
  citations: [{
    type: mongoose.Schema.Types.ObjectId,
    ref: 'Citation'
  }],
  bibliography: [{
    type: String, // BibTeX format
    citationId: {
      type: mongoose.Schema.Types.ObjectId,
      ref: 'Citation'
    }
  }],
  
  // Review Management
  assignedReviewers: [{
    reviewer: {
      type: mongoose.Schema.Types.ObjectId,
      ref: 'User'
    },
    assignedDate: Date,
    dueDate: Date,
    completedDate: Date,
    status: {
      type: String,
      enum: ['assigned', 'in_progress', 'completed', 'declined'],
      default: 'assigned'
    },
    blindReview: { type: Boolean, default: true }
  }],
  reviews: [{
    type: mongoose.Schema.Types.ObjectId,
    ref: 'Review'
  }],
  
  // Metadata
  doi: String,
  arxivId: String,
  handle: String,
  relatedPapers: [{
    type: mongoose.Schema.Types.ObjectId,
    ref: 'Paper'
  }],
  
  // Privacy and Access
  visibility: {
    type: String,
    enum: ['public', 'private', 'embargoed'],
    default: 'private'
  },
  embargoDate: Date,
  accessToken: String, // For private shared papers
  
  // System Fields
  createdBy: {
    type: mongoose.Schema.Types.ObjectId,
    ref: 'User',
    required: true
  },
  updatedBy: {
    type: mongoose.Schema.Types.ObjectId,
    ref: 'User'
  },
  createdAt: { type: Date, default: Date.now },
  updatedAt: { type: Date, default: Date.now },
  
  // Performance and Impact
  metrics: {
    views: { type: Number, default: 0 },
    downloads: { type: Number, default: 0 },
    citations: { type: Number, default: 0 },
    altmetricScore: { type: Number, default: 0 },
    impactFactor: { type: Number, default: 0 }
  },
  
  // Review Decision
  finalDecision: {
    decision: {
      type: String,
      enum: ['accept', 'minor_revision', 'major_revision', 'reject']
    },
    decisionDate: Date,
    decisionBy: {
      type: mongoose.Schema.Types.ObjectId,
      ref: 'User'
    },
    letterToAuthors: String
  }
}, {
  timestamps: true,
  toJSON: { virtuals: true },
  toObject: { virtuals: true }
});

// Virtual for all author names
paperSchema.virtual('authorNames').get(function() {
  return this.authors.map(author => author.name || `${author.user?.firstName} ${author.user?.lastName}`);
});

// Virtual for corresponding author details
paperSchema.virtual('correspondingAuthorDetails').get(function() {
  const corrAuthor = this.authors.find(author => author.isCorresponding);
  return corrAuthor ? {
    name: corrAuthor.name || `${corrAuthor.user?.firstName} ${corrAuthor.user?.lastName}`,
    email: corrAuthor.email || corrAuthor.user?.email,
    affiliation: corrAuthor.affiliation || corrAuthor.user?.affiliation?.institution
  } : null;
});

// Virtual for paper age
paperSchema.virtual('ageInDays').get(function() {
  return Math.floor((Date.now() - this.createdAt) / (1000 * 60 * 60 * 24));
});

// Virtual for review progress
paperSchema.virtual('reviewProgress').get(function() {
  const total = this.assignedReviewers.length;
  const completed = this.assignedReviewers.filter(r => r.status === 'completed').length;
  const inProgress = this.assignedReviewers.filter(r => r.status === 'in_progress').length;
  
  return {
    total,
    completed,
    inProgress,
    percentage: total > 0 ? Math.round((completed / total) * 100) : 0
  };
});

// Pre-save middleware
paperSchema.pre('save', function(next) {
  this.updatedAt = new Date();
  
  // Set submission date when status changes to submitted
  if (this.isModified('status') && this.status === 'submitted' && !this.submissionDate) {
    this.submissionDate = new Date();
  }
  
  // Auto-assign version for new versions
  if (this.isNew && this.parentPaper) {
    // This will be handled in the version creation logic
  }
  
  next();
});

// Methods
paperSchema.methods.submitForReview = async function(conferenceId, track = null) {
  this.status = 'submitted';
  this.submissionDate = new Date();
  
  if (conferenceId) {
    this.submissionTarget = {
      type: 'conference',
      conference: conferenceId,
      track
    };
  }
  
  return await this.save();
};

paperSchema.methods.assignReviewer = async function(reviewerId, dueDate, blindReview = true) {
  const existingAssignment = this.assignedReviewers.find(
    assignment => assignment.reviewer.toString() === reviewerId.toString()
  );
  
  if (existingAssignment) {
    throw new Error('Reviewer already assigned');
  }
  
  this.assignedReviewers.push({
    reviewer: reviewerId,
    assignedDate: new Date(),
    dueDate,
    blindReview
  });
  
  return await this.save();
};

paperSchema.methods.addReview = async function(reviewId) {
  this.reviews.push(reviewId);
  
  // Check if all reviews are completed
  const allCompleted = this.assignedReviewers.every(
    assignment => assignment.status === 'completed'
  );
  
  if (allCompleted && this.reviews.length >= this.assignedReviewers.length) {
    this.status = 'under_review';
  }
  
  return await this.save();
};

paperSchema.methods.createNewVersion = async function(changes, updatedBy) {
  const Paper = this.constructor;
  
  // Create new version
  const newVersion = new Paper({
    ...this.toObject(),
    _id: undefined,
    version: this.version + 1,
    parentPaper: this._id,
    status: 'draft',
    reviewCycle: {
      current: this.reviewCycle.current + 1,
      total: this.reviewCycle.total,
      maximum: this.reviewCycle.maximum
    },
    reviews: [],
    assignedReviewers: [],
    changeLog: [...this.changeLog, {
      version: this.version,
      date: new Date(),
      author: updatedBy,
      changes
    }],
    createdBy: this.createdBy,
    updatedBy
  });
  
  await newVersion.save();
  return newVersion;
};

paperSchema.methods.recordView = function() {
  this.metrics.views += 1;
  return this.save();
};

paperSchema.methods.recordDownload = function() {
  this.metrics.downloads += 1;
  return this.save();
};

// Indexes
paperSchema.index({ title: 'text', abstract: 'text', keywords: 'text' });
paperSchema.index({ authors: 1 });
paperSchema.index({ researchArea: 1 });
paperSchema.index({ status: 1 });
paperSchema.index({ submissionDate: -1 });
paperSchema.index({ createdAt: -1 });
paperSchema.index({ 'submissionTarget.conference': 1 });
paperSchema.index({ doi: 1 });
paperSchema.index({ arxivId: 1 });
paperSchema.index({ parentPaper: 1 });

module.exports = mongoose.model('Paper', paperSchema);