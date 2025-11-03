const mongoose = require('mongoose');

const reviewSchema = new mongoose.Schema({
  // Basic Review Information
  paper: {
    type: mongoose.Schema.Types.ObjectId,
    ref: 'Paper',
    required: true
  },
  reviewer: {
    type: mongoose.Schema.Types.ObjectId,
    ref: 'User',
    required: true
  },
  assignedBy: {
    type: mongoose.Schema.Types.ObjectId,
    ref: 'User'
  },
  
  // Review Assignment Details
  assignmentDate: {
    type: Date,
    default: Date.now
  },
  dueDate: {
    type: Date,
    required: true
  },
  completedDate: Date,
  
  // Review Status
  status: {
    type: String,
    enum: ['assigned', 'in_progress', 'completed', 'late', 'withdrawn'],
    default: 'assigned'
  },
  isBlind: {
    type: Boolean,
    default: true
  },
  cycle: {
    type: Number,
    default: 1
  },
  
  // Review Ratings and Scores
  rating: {
    originality: {
      score: { type: Number, min: 1, max: 5 },
      comment: String
    },
    significance: {
      score: { type: Number, min: 1, max: 5 },
      comment: String
    },
    technicalQuality: {
      score: { type: Number, min: 1, max: 5 },
      comment: String
    },
    clarity: {
      score: { type: Number, min: 1, max: 5 },
      comment: String
    },
    overall: {
      score: { type: Number, min: 1, max: 5 },
      comment: String
    }
  },
  
  // Detailed Feedback
  reviewText: {
    summary: String, // Brief summary of the paper
    strengths: String, // What the reviewer liked
    weaknesses: String, // What needs improvement
    detailedComments: String, // Page-by-page or section-by-section comments
    confidentialComments: String // Comments only visible to editors
  },
  
  // Specific Review Areas
  technicalReview: {
    methodology: String,
    experiments: String,
    results: String,
    relatedWork: String,
    correctness: String,
    reproducibility: String
  },
  
  // Writing and Presentation Review
  presentationReview: {
    structure: String,
    figures: String,
    tables: String,
    references: String,
    grammar: String,
    formatting: String
  },
  
  // Decision Recommendation
  recommendation: {
    decision: {
      type: String,
      enum: ['accept', 'minor_revision', 'major_revision', 'reject'],
      required: true
    },
    confidence: {
      type: Number,
      min: 1,
      max: 3,
      default: 2,
      comment: String
    },
    rationale: String // Explanation for the recommendation
  },
  
  // Review Quality Assessment (Editor Only)
  qualityAssessment: {
    overallQuality: {
      type: Number,
      min: 1,
      max: 5
    },
    constructiveness: {
      type: Number,
      min: 1,
      max: 5
    },
    expertise: {
      type: Number,
      min: 1,
      max: 5
    },
    timeliness: {
      type: Number,
      min: 1,
      max: 5
    },
    editorComments: String
  },
  
  // Revision Review
  isRevisionReview: {
    type: Boolean,
    default: false
  },
  revisionChanges: {
    revisionNumber: Number,
    addressMajorComments: String,
    addressMinorComments: String,
    additionalComments: String,
    originalRecommendation: String,
    revisedRecommendation: {
      type: String,
      enum: ['accept', 'minor_revision', 'major_revision', 'reject']
    }
  },
  
  // Reviewer Experience
  reviewerExperience: {
    isFirstReview: { type: Boolean, default: false },
    experienceLevel: {
      type: String,
      enum: ['novice', 'experienced', 'expert'],
      default: 'experienced'
    },
    confidenceLevel: {
      type: Number,
      min: 1,
      max: 5,
      default: 3
    }
  },
  
  // Anonymous Identity (for blind review)
  anonymousIdentity: {
    reviewerNumber: String, // e.g., "Reviewer 1"
    maskedIdentity: String // Additional masking if needed
  },
  
  // Collaboration Review
  collaborativeReview: {
    isCollaborative: { type: Boolean, default: false },
    collaborators: [{
      user: {
        type: mongoose.Schema.Types.ObjectId,
        ref: 'User'
      },
      contribution: String,
      permission: {
        type: String,
        enum: ['read', 'comment', 'edit'],
        default: 'comment'
      }
    }]
  },
  
  // Review Metrics
  metrics: {
    wordCount: Number,
    timeSpent: Number, // in minutes
    sessionCount: Number,
    lastAccessed: Date
  },
  
  // Metadata
  lastModified: Date,
  version: {
    type: Number,
    default: 1
  },
  
  // System Fields
  createdAt: { type: Date, default: Date.now },
  updatedAt: { type: Date, default: Date.now }
}, {
  timestamps: true,
  toJSON: { virtuals: true },
  toObject: { virtuals: true }
});

// Virtual for average rating
reviewSchema.virtual('averageRating').get(function() {
  const scores = [
    this.rating.originality.score,
    this.rating.significance.score,
    this.rating.technicalQuality.score,
    this.rating.clarity.score
  ].filter(score => score);
  
  return scores.length > 0 ? 
    scores.reduce((sum, score) => sum + score, 0) / scores.length : null;
});

// Virtual for review age
reviewSchema.virtual('daysSinceAssignment').get(function() {
  return Math.floor((Date.now() - this.assignmentDate) / (1000 * 60 * 60 * 24));
});

// Virtual for days until due
reviewSchema.virtual('daysUntilDue').get(function() {
  return Math.ceil((this.dueDate - Date.now()) / (1000 * 60 * 60 * 24));
});

// Virtual for completion time
reviewSchema.virtual('completionTimeInDays').get(function() {
  if (!this.completedDate) return null;
  return Math.floor((this.completedDate - this.assignmentDate) / (1000 * 60 * 60 * 24));
});

// Pre-save middleware
reviewSchema.pre('save', function(next) {
  this.lastModified = new Date();
  
  // Update status based on completion
  if (this.completedDate && this.status !== 'completed') {
    this.status = 'completed';
  }
  
  // Mark as late if due date has passed and not completed
  if (!this.completedDate && Date.now() > this.dueDate && this.status !== 'late') {
    this.status = 'late';
  }
  
  next();
});

// Methods
reviewSchema.methods.submitReview = async function() {
  // Validate required fields
  if (!this.recommendation.decision) {
    throw new Error('Decision recommendation is required');
  }
  
  if (!this.reviewText.summary) {
    throw new Error('Summary is required');
  }
  
  // Calculate metrics
  const wordCount = this.reviewText.summary?.length || 0;
  const detailedWordCount = this.reviewText.detailedComments?.length || 0;
  
  this.metrics.wordCount = wordCount + detailedWordCount;
  this.completedDate = new Date();
  this.status = 'completed';
  
  return await this.save();
};

reviewSchema.methods.addComment = function(section, comment, pageNumber = null) {
  // Add structured comment for specific section
  if (!this.reviewText.detailedComments) {
    this.reviewText.detailedComments = '';
  }
  
  const commentEntry = `[${section}${pageNumber ? `, p.${pageNumber}` : ''}] ${comment}`;
  this.reviewText.detailedComments += `\n${commentEntry}`;
  
  return this.save();
};

reviewSchema.methods.updateRating = function(category, score, comment = '') {
  if (!this.rating[category]) {
    throw new Error(`Invalid rating category: ${category}`);
  }
  
  this.rating[category].score = score;
  if (comment) {
    this.rating[category].comment = comment;
  }
  
  return this.save();
};

reviewSchema.methods.createRevisionResponse = function(revisionNumber, addressesMajor, addressesMinor) {
  this.isRevisionReview = true;
  this.revisionChanges = {
    revisionNumber,
    addressMajorComments: addressesMajor,
    addressMinorComments: addressesMinor
  };
  
  return this.save();
};

reviewSchema.methods.finalizeRevisionRecommendation = function(revisedRecommendation, additionalComments = '') {
  if (!this.isRevisionReview) {
    throw new Error('This is not a revision review');
  }
  
  this.revisionChanges.revisedRecommendation = revisedRecommendation;
  this.revisionChanges.additionalComments = additionalComments;
  this.recommendation.decision = revisedRecommendation;
  
  return this.save();
};

// Static methods
reviewSchema.statics.findOverdue = function() {
  return this.find({
    status: { $in: ['assigned', 'in_progress'] },
    dueDate: { $lt: new Date() }
  }).populate('paper reviewer');
};

reviewSchema.statics.getReviewerStats = function(reviewerId) {
  return this.aggregate([
    { $match: { reviewer: new mongoose.Types.ObjectId(reviewerId) } },
    {
      $group: {
        _id: null,
        totalReviews: { $sum: 1 },
        completedReviews: {
          $sum: { $cond: [{ $eq: ['$status', 'completed'] }, 1, 0] }
        },
        averageTime: {
          $avg: {
            $divide: [
              { $subtract: ['$completedDate', '$assignmentDate'] },
              1000 * 60 * 60 * 24 // Convert to days
            ]
          }
        },
        averageRating: { $avg: '$rating.overall.score' }
      }
    }
  ]);
};

reviewSchema.statics.getQualityMetrics = function() {
  return this.aggregate([
    { $match: { status: 'completed' } },
    {
      $group: {
        _id: null,
        totalReviews: { $sum: 1 },
        averageOverallRating: { $avg: '$rating.overall.score' },
        averageOriginality: { $avg: '$rating.originality.score' },
        averageSignificance: { $avg: '$rating.significance.score' },
        averageTechnicalQuality: { $avg: '$rating.technicalQuality.score' },
        averageClarity: { $avg: '$rating.clarity.score' }
      }
    }
  ]);
};

// Indexes
reviewSchema.index({ paper: 1, reviewer: 1 }, { unique: true });
reviewSchema.index({ reviewer: 1 });
reviewSchema.index({ status: 1 });
reviewSchema.index({ dueDate: 1 });
reviewSchema.index({ assignmentDate: -1 });
reviewSchema.index({ 'recommendation.decision': 1 });
reviewSchema.index({ cycle: 1 });

module.exports = mongoose.model('Review', reviewSchema);