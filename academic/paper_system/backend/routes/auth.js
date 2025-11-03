const express = require('express');
const jwt = require('jsonwebtoken');
const bcrypt = require('bcryptjs');
const Joi = require('joi');
const User = require('../models/User');
const auth = require('../middleware/auth');
const rateLimit = require('express-rate-limit');

const router = express.Router();

// Rate limiting for auth endpoints
const authLimiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 5, // limit each IP to 5 requests per windowMs for auth endpoints
  message: 'Too many authentication attempts, please try again later.'
});

const loginLimiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 10, // limit each IP to 10 login requests per windowMs
  message: 'Too many login attempts, please try again later.'
});

// Validation schemas
const registerSchema = Joi.object({
  email: Joi.string().email().required(),
  password: Joi.string().min(6).required(),
  firstName: Joi.string().trim().required(),
  lastName: Joi.string().trim().required(),
  title: Joi.string().valid('Dr.', 'Prof.', 'Mr.', 'Ms.', 'Mrs.', 'Mx.').default('Dr.'),
  affiliation: Joi.object({
    institution: Joi.string().trim(),
    department: Joi.string().trim(),
    position: Joi.string().trim(),
    orcid: Joi.string().pattern(/^\d{4}-\d{4}-\d{4}-\d{4}$/)
  }),
  researchInterests: Joi.array().items(Joi.string().trim()),
  currentResearch: Joi.string().trim()
});

const loginSchema = Joi.object({
  email: Joi.string().email().required(),
  password: Joi.string().required()
});

const updateProfileSchema = Joi.object({
  firstName: Joi.string().trim(),
  lastName: Joi.string().trim(),
  title: Joi.string().valid('Dr.', 'Prof.', 'Mr.', 'Ms.', 'Mrs.', 'Mx.'),
  affiliation: Joi.object({
    institution: Joi.string().trim(),
    department: Joi.string().trim(),
    position: Joi.string().trim(),
    orcid: Joi.string().pattern(/^\d{4}-\d{4}-\d{4}-\d{4}$/)
  }),
  researchInterests: Joi.array().items(Joi.string().trim()),
  currentResearch: Joi.string().trim(),
  reviewPreferences: Joi.object({
    willingToReview: Joi.boolean(),
    areas: Joi.array().items(Joi.string()),
    maximumReviewsPerYear: Joi.number().min(1).max(50),
    languages: Joi.array().items(Joi.string())
  }),
  emailNotifications: Joi.object({
    paperSubmissions: Joi.boolean(),
    reviewRequests: Joi.boolean(),
    conferenceDeadlines: Joi.boolean(),
    systemUpdates: Joi.boolean()
  }),
  privacySettings: Joi.object({
    profileVisible: Joi.boolean(),
    showEmail: Joi.boolean(),
    showAffiliation: Joi.boolean()
  })
});

// Register new user
router.post('/register', authLimiter, async (req, res) => {
  try {
    // Validate input
    const { error, value } = registerSchema.validate(req.body);
    if (error) {
      return res.status(400).json({ 
        error: 'Validation failed',
        details: error.details.map(detail => detail.message)
      });
    }

    const { email, password, firstName, lastName, title, affiliation, researchInterests, currentResearch } = value;

    // Check if user already exists
    const existingUser = await User.findOne({ email });
    if (existingUser) {
      return res.status(409).json({ 
        error: 'User already exists',
        message: 'An account with this email address already exists'
      });
    }

    // Create new user
    const user = new User({
      email,
      password,
      firstName,
      lastName,
      title,
      affiliation,
      researchInterests,
      currentResearch,
      role: 'researcher'
    });

    await user.save();

    // Generate JWT token
    const token = jwt.sign(
      { userId: user._id, email: user.email, role: user.role },
      process.env.JWT_SECRET || 'your-secret-key',
      { expiresIn: '7d' }
    );

    // Set secure cookie
    res.cookie('token', token, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'strict',
      maxAge: 7 * 24 * 60 * 60 * 1000 // 7 days
    });

    res.status(201).json({
      message: 'User registered successfully',
      token,
      user: {
        id: user._id,
        email: user.email,
        firstName: user.firstName,
        lastName: user.lastName,
        fullName: user.fullName,
        title: user.title,
        role: user.role,
        affiliation: user.affiliation,
        researchInterests: user.researchInterests
      }
    });

  } catch (error) {
    console.error('Registration error:', error);
    res.status(500).json({ 
      error: 'Registration failed',
      message: 'An error occurred during registration'
    });
  }
});

// Login user
router.post('/login', loginLimiter, async (req, res) => {
  try {
    // Validate input
    const { error, value } = loginSchema.validate(req.body);
    if (error) {
      return res.status(400).json({ 
        error: 'Validation failed',
        details: error.details.map(detail => detail.message)
      });
    }

    const { email, password } = value;

    // Find user
    const user = await User.findOne({ email }).select('+password');
    if (!user) {
      return res.status(401).json({ 
        error: 'Invalid credentials',
        message: 'Email or password is incorrect'
      });
    }

    // Check if account is active
    if (!user.isActive) {
      return res.status(403).json({ 
        error: 'Account inactive',
        message: 'Your account has been deactivated. Please contact support.'
      });
    }

    // Verify password
    const isValidPassword = await user.comparePassword(password);
    if (!isValidPassword) {
      return res.status(401).json({ 
        error: 'Invalid credentials',
        message: 'Email or password is incorrect'
      });
    }

    // Update login statistics
    user.lastLogin = new Date();
    user.loginCount += 1;
    await user.save();

    // Generate JWT token
    const token = jwt.sign(
      { userId: user._id, email: user.email, role: user.role },
      process.env.JWT_SECRET || 'your-secret-key',
      { expiresIn: '7d' }
    );

    // Set secure cookie
    res.cookie('token', token, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'strict',
      maxAge: 7 * 24 * 60 * 60 * 1000 // 7 days
    });

    res.json({
      message: 'Login successful',
      token,
      user: {
        id: user._id,
        email: user.email,
        firstName: user.firstName,
        lastName: user.lastName,
        fullName: user.fullName,
        title: user.title,
        role: user.role,
        affiliation: user.affiliation,
        researchInterests: user.researchInterests,
        reviewPreferences: user.reviewPreferences,
        lastLogin: user.lastLogin,
        loginCount: user.loginCount
      }
    });

  } catch (error) {
    console.error('Login error:', error);
    res.status(500).json({ 
      error: 'Login failed',
      message: 'An error occurred during login'
    });
  }
});

// Logout user
router.post('/logout', auth, (req, res) => {
  res.clearCookie('token', {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'strict'
  });

  res.json({ message: 'Logout successful' });
});

// Get current user
router.get('/me', auth, async (req, res) => {
  try {
    const user = await User.findById(req.user.userId);
    if (!user) {
      return res.status(404).json({ error: 'User not found' });
    }

    // Get user statistics
    const stats = await user.getStats();

    res.json({
      user: {
        id: user._id,
        email: user.email,
        firstName: user.firstName,
        lastName: user.lastName,
        fullName: user.fullName,
        title: user.title,
        role: user.role,
        affiliation: user.affiliation,
        researchInterests: user.researchInterests,
        expertise: user.expertise,
        currentResearch: user.currentResearch,
        publications: user.publications,
        reviewPreferences: user.reviewPreferences,
        emailNotifications: user.emailNotifications,
        privacySettings: user.privacySettings,
        isVerified: user.isVerified,
        createdAt: user.createdAt,
        lastLogin: user.lastLogin
      },
      stats
    });

  } catch (error) {
    console.error('Get user error:', error);
    res.status(500).json({ 
      error: 'Failed to fetch user data',
      message: 'An error occurred while fetching user information'
    });
  }
});

// Update user profile
router.put('/profile', auth, async (req, res) => {
  try {
    // Validate input
    const { error, value } = updateProfileSchema.validate(req.body);
    if (error) {
      return res.status(400).json({ 
        error: 'Validation failed',
        details: error.details.map(detail => detail.message)
      });
    }

    const user = await User.findById(req.user.userId);
    if (!user) {
      return res.status(404).json({ error: 'User not found' });
    }

    // Update user fields
    Object.keys(value).forEach(key => {
      if (value[key] !== undefined) {
        user[key] = value[key];
      }
    });

    await user.save();

    res.json({
      message: 'Profile updated successfully',
      user: {
        id: user._id,
        email: user.email,
        firstName: user.firstName,
        lastName: user.lastName,
        fullName: user.fullName,
        title: user.title,
        affiliation: user.affiliation,
        researchInterests: user.researchInterests,
        currentResearch: user.currentResearch,
        reviewPreferences: user.reviewPreferences,
        emailNotifications: user.emailNotifications,
        privacySettings: user.privacySettings
      }
    });

  } catch (error) {
    console.error('Profile update error:', error);
    res.status(500).json({ 
      error: 'Profile update failed',
      message: 'An error occurred while updating profile'
    });
  }
});

// Change password
router.put('/change-password', auth, async (req, res) => {
  try {
    const { currentPassword, newPassword } = req.body;

    if (!currentPassword || !newPassword) {
      return res.status(400).json({ 
        error: 'Missing fields',
        message: 'Current password and new password are required'
      });
    }

    if (newPassword.length < 6) {
      return res.status(400).json({ 
        error: 'Invalid password',
        message: 'New password must be at least 6 characters long'
      });
    }

    const user = await User.findById(req.user.userId).select('+password');
    if (!user) {
      return res.status(404).json({ error: 'User not found' });
    }

    // Verify current password
    const isValidPassword = await user.comparePassword(currentPassword);
    if (!isValidPassword) {
      return res.status(401).json({ 
        error: 'Invalid password',
        message: 'Current password is incorrect'
      });
    }

    // Update password
    user.password = newPassword;
    await user.save();

    res.json({ message: 'Password changed successfully' });

  } catch (error) {
    console.error('Password change error:', error);
    res.status(500).json({ 
      error: 'Password change failed',
      message: 'An error occurred while changing password'
    });
  }
});

// Request password reset
router.post('/forgot-password', authLimiter, async (req, res) => {
  try {
    const { email } = req.body;

    if (!email) {
      return res.status(400).json({ 
        error: 'Missing email',
        message: 'Email address is required'
      });
    }

    const user = await User.findOne({ email });
    if (!user) {
      // Don't reveal if user exists or not
      return res.json({ 
        message: 'If the email exists, a password reset link has been sent' 
      });
    }

    // Generate reset token
    const resetToken = user.generateResetToken();
    await user.save();

    // TODO: Send email with reset token
    // For now, return the token (in production, send via email)
    if (process.env.NODE_ENV === 'development') {
      res.json({ 
        message: 'Password reset token generated',
        resetToken // Remove this in production
      });
    } else {
      res.json({ 
        message: 'If the email exists, a password reset link has been sent' 
      });
    }

  } catch (error) {
    console.error('Forgot password error:', error);
    res.status(500).json({ 
      error: 'Request failed',
      message: 'An error occurred while processing password reset request'
    });
  }
});

// Reset password
router.post('/reset-password', authLimiter, async (req, res) => {
  try {
    const { token, newPassword } = req.body;

    if (!token || !newPassword) {
      return res.status(400).json({ 
        error: 'Missing fields',
        message: 'Reset token and new password are required'
      });
    }

    if (newPassword.length < 6) {
      return res.status(400).json({ 
        error: 'Invalid password',
        message: 'New password must be at least 6 characters long'
      });
    }

    // Find user by reset token
    const user = await User.findOne({
      resetPasswordToken: token,
      resetPasswordExpires: { $gt: Date.now() }
    });

    if (!user) {
      return res.status(400).json({ 
        error: 'Invalid token',
        message: 'Password reset token is invalid or has expired'
      });
    }

    // Update password
    user.password = newPassword;
    user.resetPasswordToken = undefined;
    user.resetPasswordExpires = undefined;
    await user.save();

    res.json({ message: 'Password reset successful' });

  } catch (error) {
    console.error('Reset password error:', error);
    res.status(500).json({ 
      error: 'Reset failed',
      message: 'An error occurred while resetting password'
    });
  }
});

// Refresh token
router.post('/refresh', auth, async (req, res) => {
  try {
    const user = await User.findById(req.user.userId);
    if (!user) {
      return res.status(404).json({ error: 'User not found' });
    }

    // Generate new JWT token
    const token = jwt.sign(
      { userId: user._id, email: user.email, role: user.role },
      process.env.JWT_SECRET || 'your-secret-key',
      { expiresIn: '7d' }
    );

    // Set secure cookie
    res.cookie('token', token, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'strict',
      maxAge: 7 * 24 * 60 * 60 * 1000 // 7 days
    });

    res.json({
      message: 'Token refreshed successfully',
      token
    });

  } catch (error) {
    console.error('Token refresh error:', error);
    res.status(500).json({ 
      error: 'Token refresh failed',
      message: 'An error occurred while refreshing token'
    });
  }
});

module.exports = router;