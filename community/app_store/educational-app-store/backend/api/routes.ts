// API routes for Educational App Store

import express from 'express';
import { AppRouter } from './controllers/apps';
import { CategoryRouter } from './controllers/categories';
import { ReviewRouter } from './controllers/reviews';
import { UserRouter } from './controllers/users';
import { SubmissionRouter } from './controllers/submissions';
import { AnalyticsRouter } from './controllers/analytics';
import { RecommendationRouter } from './controllers/recommendations';
import { AuthMiddleware } from '../utils/auth';

const router = express.Router();

// Public routes
router.use('/categories', CategoryRouter);
router.use('/apps', AppRouter);
router.use('/reviews', ReviewRouter);

// Protected routes (require authentication)
router.use('/users', AuthMiddleware, UserRouter);
router.use('/submissions', AuthMiddleware, SubmissionRouter);
router.use('/analytics', AuthMiddleware, AnalyticsRouter);
router.use('/recommendations', AuthMiddleware, RecommendationRouter);

// Health check
router.get('/health', (req, res) => {
  res.json({ status: 'OK', timestamp: new Date().toISOString() });
});

export { router as ApiRoutes };