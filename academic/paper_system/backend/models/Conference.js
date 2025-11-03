const mongoose = require('mongoose');

const submissionGuidelinesSchema = new mongoose.Schema({
  maxPages: Number,
  pageLimit: String,
  template: String,
  latexTemplate: String,
  wordTemplate: String,
  anonymousSubmission: { type: Boolean, default: true },
  doubleBlindReview: { type: Boolean, default: false },
  supplementaryMaterialAllowed: { type: Boolean, default: true },
  supplementaryPageLimit: Number,
  fileFormats: [String],
  authorInstructions: String
}, { _id: false });

const importantDateSchema = new mongoose.Schema({
  name: String, // "Paper Submission", "Author Notification", etc.
  date: Date,
  timezone: { type: String, default: 'UTC' },
  description: String,
  isHardDeadline: { type: Boolean, default: true }
}, { _id: true });

const trackSchema = new mongoose.Schema({
  name: { type: String, required: true },
  description: String,
  chairs: [{
    name: String,
    email: String,
    affiliation: String,
    role: { type: String, enum: ['chair', 'co-chair'] }
  }],
  reviewers: [{
    type: mongoose.Schema.Types.ObjectId,
    ref: 'User'
  }],
  submissionCount: { type: Number, default: 0 },
  maxSubmissionsPerAuthor: { type: Number, default: 3 }
}, { _id: true });

const reviewerAssignmentSchema = new mongoose.Schema({
  track: mongoose.Schema.Types.ObjectId,
  reviewer: {
    type: mongoose.Schema.Types.ObjectId,
    ref: 'User'
  },
  expertise: [String],
  maximumPapers: { type: Number, default: 6 },
  assignedPapers: [{ type: mongoose.Schema.Types.ObjectId, ref: 'Paper' }],
  isActive: { type: Boolean, default: true },
  assignmentDate: Date
}, { _id: true });

const conferenceSchema = new mongoose.Schema({
  // Basic Information
  name: {
    type: String,
    required: true,
    trim: true
  },
  shortName: {
    type: String,
    required: true,
    trim: true,
    uppercase: true
  },
  acronym: {
    type: String,
    required: true,
    uppercase: true
  },
  
  // Conference Details
  description: {
    type: String,
    required: true
  },
  theme: String,
  scope: [String], // Research areas covered
  keywords: [String],
  
  // Organizing Committee
  organizingCommittee: {
    generalChairs: [{
      name: String,
      email: String,
      affiliation: String,
      photo: String
    }],
    programChairs: [{
      name: String,
      email: String,
      affiliation: String,
      photo: String
    }],
    publicChairs: [{
      name: String,
      email: String,
      affiliation: String
    }],
    localChairs: [{
      name: String,
      email: String,
      affiliation: String
    }],
    workshopChairs: [{
      name: String,
      email: String,
      affiliation: String
    }]
  },
  
  // Conference Information
  location: {
    city: String,
    country: String,
    venue: String,
    virtual: { type: Boolean, default: false },
    hybrid: { type: Boolean, default: false },
    timezone: { type: String, default: 'UTC' }
  },
  dates: {
    conferenceStart: Date,
    conferenceEnd: Date,
    registrationStart: Date,
    registrationEnd: Date
  },
  
  // Deadlines and Important Dates
  importantDates: [importantDateSchema],
  submissionGuidelines: submissionGuidelinesSchema,
  
  // Tracks and Areas
  tracks: [trackSchema],
  researchAreas: [String],
  
  // Review Management
  reviewProcess: {
    reviewerCount: { type: Number, default: 3 },
    reviewRounds: { type: Number, default: 2 },
    reviewPeriod: Number, // in days
    discussionPeriod: Number, // in days
    consensusRequired: { type: Boolean, default: false }
  },
  reviewerManagement: {
    minimumReviewsPerReviewer: { type: Number, default: 3 },
    maximumReviewsPerReviewer: { type: Number, default: 6 },
    automaticAssignment: { type: Boolean, default: true },
    conflictOfInterest: { type: Boolean, default: true }
  },
  
  // Submission Statistics
  statistics: {
    totalSubmissions: { type: Number, default: 0 },
    totalAcceptances: { type: Number, default: 0 },
    acceptanceRate: { type: Number, default: 0 },
    trackBreakdown: [{
      track: String,
      submissions: Number,
      acceptances: Number
    }]
  },
  
  // Publication
  publication: {
    proceedingsPublisher: String,
    proceedingsISBN: String,
    doiPrefix: String,
    indexedIn: [String], // DBLP, ACM Digital Library, IEEE Xplore, etc.
    hasSpecialIssue: { type: Boolean, default: false },
    specialIssueJournal: String,
    openAccess: { type: Boolean, default: false }
  },
  
  // Awards
  awards: [{
    name: String,
    description: String,
    criteria: String,
    winner: {
      paper: {
        type: mongoose.Schema.Types.ObjectId,
        ref: 'Paper'
      },
      authors: [String],
      awardDate: Date
    }
  }],
  
  // Sponsorship
  sponsors: [{
    name: String,
    logo: String,
    url: String,
    level: {
      type: String,
      enum: ['platinum', 'gold', 'silver', 'bronze', 'supporter']
    }
  }],
  
  // Contact Information
  contact: {
    email: String,
    website: String,
    socialMedia: {
      twitter: String,
      linkedin: String,
      facebook: String
    }
  },
  
  // Configuration
  status: {
    type: String,
    enum: [
      'planning',
      'cfp_announced',
      'submissions_open',
      'submissions_closed',
      'under_review',
      'reviews_completed',
      'decisions_sent',
      'camera_ready_deadline',
      'proceedings_final',
      'completed',
      'cancelled'
    ],
    default: 'planning'
  },
  visibility: {
    type: String,
    enum: ['public', 'private', 'unlisted'],
    default: 'public'
  },
  
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
  updatedAt: { type: Date, default: Date.now }
}, {
  timestamps: true,
  toJSON: { virtuals: true },
  toObject: { virtuals: true }
});

// Virtual for acceptance rate calculation
conferenceSchema.virtual('currentAcceptanceRate').get(function() {
  if (this.statistics.totalSubmissions === 0) return 0;
  return Math.round((this.statistics.totalAcceptances / this.statistics.totalSubmissions) * 100);
});

// Virtual for next important deadline
conferenceSchema.virtual('nextDeadline').get(function() {
  const now = new Date();
  const futureDeadlines = this.importantDates
    .filter(date => date.date > now)
    .sort((a, b) => a.date - b.date);
  
  return futureDeadlines[0] || null;
});

// Virtual for days until next deadline
conferenceSchema.virtual('daysUntilNextDeadline').get(function() {
  const nextDeadline = this.nextDeadline;
  if (!nextDeadline) return null;
  
  return Math.ceil((nextDeadline.date - new Date()) / (1000 * 60 * 60 * 24));
});

// Virtual for submission status
conferenceSchema.virtual('isAcceptingSubmissions').get(function() {
  const now = new Date();
  const submissionDeadline = this.importantDates.find(date => 
    date.name.toLowerCase().includes('submission') && 
    date.name.toLowerCase().includes('deadline')
  );
  
  if (!submissionDeadline) return false;
  
  return now < submissionDeadline.date && 
         ['cfp_announced', 'submissions_open'].includes(this.status);
});

// Pre-save middleware
conferenceSchema.pre('save', function(next) {
  this.updatedAt = new Date();
  
  // Update acceptance rate when statistics change
  if (this.isModified('statistics.totalSubmissions') || this.isModified('statistics.totalAcceptances')) {
    if (this.statistics.totalSubmissions > 0) {
      this.statistics.acceptanceRate = 
        (this.statistics.totalAcceptances / this.statistics.totalSubmissions) * 100;
    }
  }
  
  next();
});

// Methods
conferenceSchema.methods.updateStatistics = async function() {
  const Paper = mongoose.model('Paper');
  
  const pipeline = [
    {
      $match: {
        'submissionTarget.conference': this._id,
        status: { $in: ['accepted', 'published'] }
      }
    },
    {
      $group: {
        _id: '$submissionTarget.track',
        count: { $sum: 1 }
      }
    }
  ];
  
  const trackStats = await Paper.aggregate(pipeline);
  const totalAcceptances = trackStats.reduce((sum, track) => sum + track.count, 0);
  
  // Get total submissions
  const totalSubmissions = await Paper.countDocuments({
    'submissionTarget.conference': this._id,
    status: { $in: ['submitted', 'under_review', 'accepted', 'rejected', 'published'] }
  });
  
  this.statistics = {
    totalSubmissions,
    totalAcceptances,
    acceptanceRate: totalSubmissions > 0 ? (totalAcceptances / totalSubmissions) * 100 : 0,
    trackBreakdown: trackStats.map(track => ({
      track: track._id,
      submissions: track.count,
      acceptances: track.count
    }))
  };
  
  return await this.save();
};

conferenceSchema.methods.addTrack = function(trackData) {
  const newTrack = new this.tracks.constructor(trackData);
  this.tracks.push(newTrack);
  return this.save();
};

conferenceSchema.methods.assignTrackChair = function(trackId, chairData) {
  const track = this.tracks.id(trackId);
  if (!track) {
    throw new Error('Track not found');
  }
  
  track.chairs.push(chairData);
  return this.save();
};

conferenceSchema.methods.addImportantDate = function(dateData) {
  this.importantDates.push(dateData);
  this.importantDates.sort((a, b) => a.date - b.date);
  return this.save();
};

conferenceSchema.methods.updateStatus = function(newStatus) {
  const validTransitions = {
    'planning': ['cfp_announced'],
    'cfp_announced': ['submissions_open'],
    'submissions_open': ['submissions_closed'],
    'submissions_closed': ['under_review'],
    'under_review': ['reviews_completed'],
    'reviews_completed': ['decisions_sent'],
    'decisions_sent': ['camera_ready_deadline'],
    'camera_ready_deadline': ['proceedings_final', 'completed'],
    'proceedings_final': ['completed']
  };
  
  if (!validTransitions[this.status].includes(newStatus)) {
    throw new Error(`Invalid status transition from ${this.status} to ${newStatus}`);
  }
  
  this.status = newStatus;
  return this.save();
};

conferenceSchema.methods.isRegistrationOpen = function() {
  const now = new Date();
  return this.status === 'cfp_announced' || 
         this.status === 'submissions_open' ||
         (this.dates.registrationStart && this.dates.registrationEnd &&
          now >= this.dates.registrationStart && now <= this.dates.registrationEnd);
};

// Static methods
conferenceSchema.statics.findUpcoming = function(limit = 10) {
  const now = new Date();
  return this.find({
    status: { $in: ['cfp_announced', 'submissions_open', 'under_review'] },
    'dates.conferenceStart': { $gte: now }
  })
  .sort({ 'dates.conferenceStart': 1 })
  .limit(limit);
};

conferenceSchema.statics.findByResearchArea = function(area) {
  return this.find({
    researchAreas: area,
    status: { $in: ['cfp_announced', 'submissions_open'] }
  }).sort({ 'importantDates.date': 1 });
};

conferenceSchema.statics.search = function(query, limit = 20) {
  return this.find({
    $or: [
      { name: { $regex: query, $options: 'i' } },
      { shortName: { $regex: query, $options: 'i' } },
      { description: { $regex: query, $options: 'i' } },
      { keywords: { $regex: query, $options: 'i' } }
    ],
    visibility: 'public'
  })
  .sort({ 'dates.conferenceStart': 1 })
  .limit(limit);
};

// Indexes
conferenceSchema.index({ shortName: 1 }, { unique: true });
conferenceSchema.index({ acronym: 1 }, { unique: true });
conferenceSchema.index({ 'dates.conferenceStart': -1 });
conferenceSchema.index({ status: 1 });
conferenceSchema.index({ visibility: 1 });
conferenceSchema.index({ researchAreas: 1 });
conferenceSchema.index({ 'location.country': 1 });

module.exports = mongoose.model('Conference', conferenceSchema);