// Authentication middleware for Educational App Store

import { Request, Response, NextFunction } from 'express';
import jwt from 'jsonwebtoken';
import { User } from '../types/models';

export interface AuthenticatedRequest extends Request {
  user?: User;
}

export const AuthMiddleware = (req: AuthenticatedRequest, res: Response, next: NextFunction) => {
  try {
    const token = req.header('Authorization')?.replace('Bearer ', '');
    
    if (!token) {
      return res.status(401).json({ error: 'Access denied. No token provided.' });
    }

    const decoded = jwt.verify(token, process.env.JWT_SECRET || 'fallback-secret') as User;
    req.user = decoded;
    next();
  } catch (error) {
    res.status(401).json({ error: 'Invalid token.' });
  }
};

export const AdminMiddleware = (req: AuthenticatedRequest, res: Response, next: NextFunction) => {
  if (req.user?.role !== 'admin') {
    return res.status(403).json({ error: 'Access denied. Admin privileges required.' });
  }
  next();
};

export const DeveloperMiddleware = (req: AuthenticatedRequest, res: Response, next: NextFunction) => {
  if (req.user?.role !== 'developer' && req.user?.role !== 'admin') {
    return res.status(403).json({ error: 'Access denied. Developer privileges required.' });
  }
  next();
};

export const EducatorMiddleware = (req: AuthenticatedRequest, res: Response, next: NextFunction) => {
  if (req.user?.role !== 'educator' && req.user?.role !== 'admin') {
    return res.status(403).json({ error: 'Access denied. Educator privileges required.' });
  }
  next();
};

export const generateToken = (user: User): string => {
  return jwt.sign(
    { 
      id: user.id, 
      email: user.email, 
      role: user.role 
    },
    process.env.JWT_SECRET || 'fallback-secret',
    { expiresIn: '7d' }
  );
};

export const verifyToken = (token: string): User | null => {
  try {
    return jwt.verify(token, process.env.JWT_SECRET || 'fallback-secret') as User;
  } catch (error) {
    return null;
  }
};