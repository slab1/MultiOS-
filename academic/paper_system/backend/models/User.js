const mongoose = require('mongoose');
const bcrypt = require('bcryptjs');

const userSchema = new mongoose.Schema({
  // Basic Information
  email: {
    type: String,
    required: true,
    unique: true,
    lowercase: true,
    trim: true
  },
  password: {
    type: String,
    required: true,
    minlength: 6
  },
  firstName: {
    type: String,
    required: true,
    trim: true
  },
  lastName: {
    type: String,
    required: true,
    trim: true
  },
  title: {
    type: String,
    enum: ['Dr.', 'Prof.', 'Mr.', 'Ms.', 'Mrs.', 'Mx.'],
    default: 'Dr.'
  },
  
  // Academic Information
  affiliation: {
    institution: String,
    department: String,
    position: String,
    orcid: {
      type: String,
      unique: true,
      sparse: true
    },
    scholarId: String,
    researchId: String
  },
  
  // Research Profile
  researchInterests: [String],
  expertise: [String],
  currentResearch: String,
  publications: [{
    title: String,
    journal: String,
    year: Number,
    doi: String,
    url: String
  }],
  
  // Review Profile
  reviewPreferences: {
    willingToReview: {
      type: Boolean,
      default: true
    },
    areas: [String], // Research areas willing to review
    maximumReviewsPerYear: {
      type: Number,
      default: 12
    },
    languages: [String]
  },
  
  // System Roles and Permissions
  role: {
    type: String,
    enum: ['researcher', 'reviewer', 'editor', 'admin'],
    default: 'researcher'
  },
  permissions: [{
    type: String,
    resource: String,
    actions: [String]
  }],
  
  // Preferences
  emailNotifications: {
    paperSubmissions: { type: Boolean, default: true },
    reviewRequests: { type: Boolean, default: true },
    conferenceDeadlines: { type: Boolean, default: true },
    systemUpdates: { type: Boolean, default: false }
  },
  privacySettings: {
    profileVisible: { type: Boolean, default: true },
    showEmail: { type: Boolean, default: false },
    showAffiliation: { type: Boolean, default: true }
  },
  
  // Activity Tracking
  lastLogin: Date,
  loginCount: { type: Number, default: 0 },
  createdAt: { type: Date, default: Date.now },
  updatedAt: { type: Date, default: Date.now },
  
  // Profile Status
  isActive: { type: Boolean, default: true },
  isVerified: { type: Boolean, default: false },
  verificationToken: String,
  resetPasswordToken: String,
  resetPasswordExpires: Date
}, {
  timestamps: true,
  toJSON: { virtuals: true },
  toObject: { virtuals: true }
});

// Virtual for full name
userSchema.virtual('fullName').get(function() {
  return `${this.firstName} ${this.lastName}`;
});

// Virtual for author profile
userSchema.virtual('authorProfile').get(function() {
  return {
    id: this._id,
    name: this.fullName,
    affiliation: this.affiliation?.institution,
    orcid: this.affiliation?.orcid,
    researchInterests: this.researchInterests
  };
});

// Hash password before saving
userSchema.pre('save', async function(next) {
  if (!this.isModified('password')) return next();
  
  try {
    const salt = await bcrypt.genSalt(12);
    this.password = await bcrypt.hash(this.password, salt);
    next();
  } catch (error) {
    next(error);
  }
});

// Update updatedAt on save
userSchema.pre('save', function(next) {
  this.updatedAt = new Date();
  next();
});

// Compare password method
userSchema.methods.comparePassword = async function(candidatePassword) {
  return await bcrypt.compare(candidatePassword, this.password);
};

// Generate reset token
userSchema.methods.generateResetToken = function() {
  const crypto = require('crypto');
  this.resetPasswordToken = crypto.randomBytes(32).toString('hex');
  this.resetPasswordExpires = Date.now() + 24 * 60 * 60 * 1000; // 24 hours
  return this.resetPasswordToken;
};

// Check if user can review in area
userSchema.methods.canReviewInArea = function(area) {
  return this.reviewPreferences.willingToReview && 
         this.reviewPreferences.areas.includes(area);
};

// Get user statistics
userSchema.methods.getStats = async function() {
  const Paper = mongoose.model('Paper');
  const Review = mongoose.model('Review');
  
  const [papersCount, reviewsGiven, reviewsReceived] = await Promise.all([
    Paper.countDocuments({ authors: this._id }),
    Review.countDocuments({ reviewer: this._id }),
    Review.countDocuments({ 
      'authorFeedback.author': this._id 
    })
  ]);
  
  return {
    papersCount,
    reviewsGiven,
    reviewsReceived,
    averageRating: await Review.aggregate([
      { $match: { reviewer: this._id } },
      { $group: { _id: null, avgRating: { $avg: '$rating.overall' } } }
    ])
  };
};

// Indexes
userSchema.index({ email: 1 });
userSchema.index({ 'affiliation.orcid': 1 });
userSchema.index({ researchInterests: 1 });
userSchema.index({ 'reviewPreferences.areas': 1 });

module.exports = mongoose.model('User', userSchema);