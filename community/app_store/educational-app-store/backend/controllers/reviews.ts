// Reviews controller for managing app reviews and ratings

import express from 'express';
import { query } from '../database/connection';
import { AuthenticatedRequest } from '../utils/auth';
import { Review, APIResponse } from '../types/models';

const router = express.Router();

// GET /api/reviews/:appId - Get reviews for an app
router.get('/:appId', async (req: express.Request, res: express.Response) => {
  try {
    const { appId } = req.params;
    const { page = 1, limit = 10, sortBy = 'helpful', rating } = req.query;

    let sql = `
      SELECT 
        r.*,
        u.name as reviewer_name,
        u.avatar as reviewer_avatar
      FROM reviews r
      LEFT JOIN users u ON r.user_id = u.id
      WHERE r.app_id = $1
    `;

    const params: any[] = [appId];
    let paramCount = 1;

    // Filter by rating if specified
    if (rating) {
      paramCount++;
      sql += ` AND r.rating = $${paramCount}`;
      params.push(parseInt(rating as string));
    }

    // Add ordering
    const validSortFields = ['created_at', 'helpful', 'rating'];
    const sortField = validSortFields.includes(sortBy as string) ? sortBy : 'helpful';
    
    if (sortBy === 'helpful') {
      sql += ` ORDER BY r.helpful DESC, r.created_at DESC`;
    } else {
      sql += ` ORDER BY r.${sortField} DESC`;
    }

    // Add pagination
    const offset = (parseInt(page as string) - 1) * parseInt(limit as string);
    sql += ` LIMIT $${++paramCount} OFFSET $${++paramCount}`;
    params.push(parseInt(limit as string), offset);

    const result = await query(sql, params);

    // Get total count
    let countSql = 'SELECT COUNT(*) as total FROM reviews WHERE app_id = $1';
    const countParams = [appId];
    
    if (rating) {
      countSql += ' AND rating = $2';
      countParams.push(rating as string);
    }

    const countResult = await query(countSql, countParams);
    const total = parseInt(countResult.rows[0].total);

    const response: APIResponse<{
      items: Review[];
      total: number;
      page: number;
      limit: number;
      totalPages: number;
    }> = {
      success: true,
      data: {
        items: result.rows,
        total,
        page: parseInt(page as string),
        limit: parseInt(limit as string),
        totalPages: Math.ceil(total / parseInt(limit as string))
      }
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching reviews:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// GET /api/reviews/ratings/:appId - Get rating statistics for an app
router.get('/ratings/:appId', async (req: express.Request, res: express.Response) => {
  try {
    const { appId } = req.params;

    const sql = `
      SELECT 
        COUNT(*) as total_reviews,
        AVG(rating) as average_rating,
        COUNT(CASE WHEN rating = 5 THEN 1 END) as five_star,
        COUNT(CASE WHEN rating = 4 THEN 1 END) as four_star,
        COUNT(CASE WHEN rating = 3 THEN 1 END) as three_star,
        COUNT(CASE WHEN rating = 2 THEN 1 END) as two_star,
        COUNT(CASE WHEN rating = 1 THEN 1 END) as one_star
      FROM reviews
      WHERE app_id = $1
    `;

    const result = await query(sql, [appId]);
    const stats = result.rows[0];

    // Calculate percentages
    const totalReviews = parseInt(stats.total_reviews);
    const ratingDistribution = {
      5: totalReviews > 0 ? Math.round((parseInt(stats.five_star) / totalReviews) * 100) : 0,
      4: totalReviews > 0 ? Math.round((parseInt(stats.four_star) / totalReviews) * 100) : 0,
      3: totalReviews > 0 ? Math.round((parseInt(stats.three_star) / totalReviews) * 100) : 0,
      2: totalReviews > 0 ? Math.round((parseInt(stats.two_star) / totalReviews) * 100) : 0,
      1: totalReviews > 0 ? Math.round((parseInt(stats.one_star) / totalReviews) * 100) : 0
    };

    const response: APIResponse<{
      totalReviews: number;
      averageRating: number;
      ratingDistribution: Record<string, number>;
    }> = {
      success: true,
      data: {
        totalReviews,
        averageRating: parseFloat(stats.average_rating) || 0,
        ratingDistribution
      }
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching rating statistics:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// POST /api/reviews - Create a new review
router.post('/', async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    if (!req.user) {
      return res.status(401).json({ success: false, error: 'Authentication required' });
    }

    const { appId, rating, title, content } = req.body;

    // Validate input
    if (!appId || !rating || rating < 1 || rating > 5) {
      return res.status(400).json({
        success: false,
        error: 'App ID and valid rating (1-5) are required'
      });
    }

    // Check if app exists
    const appResult = await query(
      'SELECT id FROM apps WHERE id = $1 AND status = $2',
      [appId, 'approved']
    );

    if (appResult.rows.length === 0) {
      return res.status(404).json({ success: false, error: 'App not found' });
    }

    // Check if user already reviewed this app
    const existingReview = await query(
      'SELECT id FROM reviews WHERE app_id = $1 AND user_id = $2',
      [appId, req.user.id]
    );

    if (existingReview.rows.length > 0) {
      return res.status(400).json({
        success: false,
        error: 'You have already reviewed this app'
      });
    }

    // Check if user has downloaded the app (verified purchase)
    const downloadCheck = await query(
      'SELECT id FROM user_downloads WHERE user_id = $1 AND app_id = $2',
      [req.user.id, appId]
    );

    const isVerified = downloadCheck.rows.length > 0;

    // Create review
    const sql = `
      INSERT INTO reviews (app_id, user_id, rating, title, content, verified)
      VALUES ($1, $2, $3, $4, $5, $6)
      RETURNING *
    `;

    const params = [
      appId,
      req.user.id,
      rating,
      title || null,
      content || null,
      isVerified
    ];

    const result = await query(sql, params);

    // Update app rating and review count
    const updateSql = `
      UPDATE apps 
      SET 
        rating = (SELECT AVG(rating) FROM reviews WHERE app_id = $1),
        review_count = (SELECT COUNT(*) FROM reviews WHERE app_id = $1)
      WHERE id = $1
    `;

    await query(updateSql, [appId]);

    const response: APIResponse<Review> = {
      success: true,
      data: result.rows[0],
      message: 'Review created successfully'
    };

    res.status(201).json(response);
  } catch (error) {
    console.error('Error creating review:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// PUT /api/reviews/:id - Update a review
router.put('/:id', async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    if (!req.user) {
      return res.status(401).json({ success: false, error: 'Authentication required' });
    }

    const { id } = req.params;
    const { rating, title, content } = req.body;

    // Check if review exists and belongs to user
    const existingReview = await query(
      'SELECT * FROM reviews WHERE id = $1 AND user_id = $2',
      [id, req.user.id]
    );

    if (existingReview.rows.length === 0) {
      return res.status(404).json({ success: false, error: 'Review not found' });
    }

    // Build dynamic update query
    const updates = [];
    const params: any[] = [];
    let paramCount = 0;

    if (rating !== undefined) {
      if (rating < 1 || rating > 5) {
        return res.status(400).json({
          success: false,
          error: 'Rating must be between 1 and 5'
        });
      }
      paramCount++;
      updates.push(`rating = $${paramCount}`);
      params.push(rating);
    }

    if (title !== undefined) {
      paramCount++;
      updates.push(`title = $${paramCount}`);
      params.push(title);
    }

    if (content !== undefined) {
      paramCount++;
      updates.push(`content = $${paramCount}`);
      params.push(content);
    }

    if (updates.length === 0) {
      return res.status(400).json({ success: false, error: 'No valid fields to update' });
    }

    paramCount++;
    updates.push(`updated_at = NOW()`);

    const sql = `
      UPDATE reviews 
      SET ${updates.join(', ')}
      WHERE id = $${paramCount}
      RETURNING *
    `;

    params.push(id);
    const result = await query(sql, params);

    // Update app rating and review count
    const updateSql = `
      UPDATE apps 
      SET 
        rating = (SELECT AVG(rating) FROM reviews WHERE app_id = $1),
        review_count = (SELECT COUNT(*) FROM reviews WHERE app_id = $1)
      WHERE id = $1
    `;

    const appId = result.rows[0].app_id;
    await query(updateSql, [appId]);

    const response: APIResponse<Review> = {
      success: true,
      data: result.rows[0],
      message: 'Review updated successfully'
    };

    res.json(response);
  } catch (error) {
    console.error('Error updating review:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// DELETE /api/reviews/:id - Delete a review
router.delete('/:id', async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    if (!req.user) {
      return res.status(401).json({ success: false, error: 'Authentication required' });
    }

    const { id } = req.params;

    // Get review to find app_id before deletion
    const reviewResult = await query(
      'SELECT app_id FROM reviews WHERE id = $1 AND user_id = $2',
      [id, req.user.id]
    );

    if (reviewResult.rows.length === 0) {
      return res.status(404).json({ success: false, error: 'Review not found' });
    }

    const appId = reviewResult.rows[0].app_id;

    // Delete review
    await query('DELETE FROM reviews WHERE id = $1', [id]);

    // Update app rating and review count
    const updateSql = `
      UPDATE apps 
      SET 
        rating = COALESCE((SELECT AVG(rating) FROM reviews WHERE app_id = $1), 0),
        review_count = (SELECT COUNT(*) FROM reviews WHERE app_id = $1)
      WHERE id = $1
    `;

    await query(updateSql, [appId]);

    const response: APIResponse<null> = {
      success: true,
      message: 'Review deleted successfully'
    };

    res.json(response);
  } catch (error) {
    console.error('Error deleting review:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// POST /api/reviews/:id/helpful - Mark review as helpful
router.post('/:id/helpful', async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    if (!req.user) {
      return res.status(401).json({ success: false, error: 'Authentication required' });
    }

    const { id } = req.params;

    const result = await query(
      'UPDATE reviews SET helpful = helpful + 1 WHERE id = $1 RETURNING helpful',
      [id]
    );

    if (result.rows.length === 0) {
      return res.status(404).json({ success: false, error: 'Review not found' });
    }

    const response: APIResponse<{ helpful: number }> = {
      success: true,
      data: { helpful: result.rows[0].helpful }
    };

    res.json(response);
  } catch (error) {
    console.error('Error marking review as helpful:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// POST /api/reviews/:id/not-helpful - Mark review as not helpful
router.post('/:id/not-helpful', async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    if (!req.user) {
      return res.status(401).json({ success: false, error: 'Authentication required' });
    }

    const { id } = req.params;

    const result = await query(
      'UPDATE reviews SET not_helpful = not_helpful + 1 WHERE id = $1 RETURNING not_helpful',
      [id]
    );

    if (result.rows.length === 0) {
      return res.status(404).json({ success: false, error: 'Review not found' });
    }

    const response: APIResponse<{ notHelpful: number }> = {
      success: true,
      data: { notHelpful: result.rows[0].not_helpful }
    };

    res.json(response);
  } catch (error) {
    console.error('Error marking review as not helpful:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

export { router as ReviewRouter };