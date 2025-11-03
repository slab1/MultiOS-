const jwt = require('jsonwebtoken');
const User = require('../models/User');

const auth = async (req, res, next) => {
  try {
    // Get token from header or cookie
    const token = req.header('Authorization')?.replace('Bearer ', '') || req.cookies?.token;
    
    if (!token) {
      return res.status(401).json({ 
        error: 'Access denied',
        message: 'No token provided'
      });
    }

    // Verify token
    const decoded = jwt.verify(token, process.env.JWT_SECRET || 'your-secret-key');
    
    // Check if user still exists and is active
    const user = await User.findById(decoded.userId);
    if (!user) {
      return res.status(401).json({ 
        error: 'Access denied',
        message: 'User not found'
      });
    }
    
    if (!user.isActive) {
      return res.status(403).json({ 
        error: 'Access denied',
        message: 'Account is inactive'
      });
    }
    
    // Add user info to request
    req.user = {
      userId: user._id,
      email: user.email,
      role: user.role,
      permissions: user.permissions
    };
    
    next();
  } catch (error) {
    if (error.name === 'JsonWebTokenError') {
      return res.status(401).json({ 
        error: 'Access denied',
        message: 'Invalid token'
      });
    }
    
    if (error.name === 'TokenExpiredError') {
      return res.status(401).json({ 
        error: 'Access denied',
        message: 'Token expired'
      });
    }
    
    console.error('Auth middleware error:', error);
    res.status(500).json({ 
      error: 'Authentication failed',
      message: 'An error occurred during authentication'
    });
  }
};

// Middleware to check for specific roles
const requireRole = (roles) => {
  return (req, res, next) => {
    if (!req.user) {
      return res.status(401).json({ 
        error: 'Access denied',
        message: 'Authentication required'
      });
    }
    
    const userRoles = Array.isArray(req.user.role) ? req.user.role : [req.user.role];
    const allowedRoles = Array.isArray(roles) ? roles : [roles];
    
    const hasRole = allowedRoles.some(role => userRoles.includes(role));
    
    if (!hasRole) {
      return res.status(403).json({ 
        error: 'Access denied',
        message: `Insufficient permissions. Required role: ${allowedRoles.join(' or ')}`
      });
    }
    
    next();
  };
};

// Middleware to check for specific permissions
const requirePermission = (resource, action) => {
  return (req, res, next) => {
    if (!req.user) {
      return res.status(401).json({ 
        error: 'Access denied',
        message: 'Authentication required'
      });
    }
    
    const hasPermission = req.user.permissions?.some(permission => 
      permission.resource === resource && permission.actions.includes(action)
    );
    
    if (!hasPermission && req.user.role !== 'admin') {
      return res.status(403).json({ 
        error: 'Access denied',
        message: `Insufficient permissions for ${action} on ${resource}`
      });
    }
    
    next();
  };
};

// Middleware to check if user owns resource or has elevated privileges
const requireOwnershipOrRole = (roles = []) => {
  return (req, res, next) => {
    if (!req.user) {
      return res.status(401).json({ 
        error: 'Access denied',
        message: 'Authentication required'
      });
    }
    
    // Check if user has elevated role
    const userRoles = Array.isArray(req.user.role) ? req.user.role : [req.user.role];
    const hasRole = roles.some(role => userRoles.includes(role));
    
    if (hasRole || req.user.role === 'admin') {
      return next();
    }
    
    // Check ownership (to be implemented in specific routes)
    next();
  };
};

// Middleware for optional authentication
const optionalAuth = async (req, res, next) => {
  try {
    const token = req.header('Authorization')?.replace('Bearer ', '') || req.cookies?.token;
    
    if (token) {
      const decoded = jwt.verify(token, process.env.JWT_SECRET || 'your-secret-key');
      const user = await User.findById(decoded.userId);
      
      if (user && user.isActive) {
        req.user = {
          userId: user._id,
          email: user.email,
          role: user.role,
          permissions: user.permissions
        };
      }
    }
    
    next();
  } catch (error) {
    // Continue without authentication for optional auth
    next();
  }
};

// Middleware to validate API key (for system-level operations)
const requireApiKey = (req, res, next) => {
  const apiKey = req.header('X-API-Key');
  
  if (!apiKey || apiKey !== process.env.API_KEY) {
    return res.status(401).json({ 
      error: 'Access denied',
      message: 'Invalid API key'
    });
  }
  
  next();
};

module.exports = {
  auth,
  requireRole,
  requirePermission,
  requireOwnershipOrRole,
  optionalAuth,
  requireApiKey
};