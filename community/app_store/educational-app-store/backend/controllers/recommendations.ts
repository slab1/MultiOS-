// Recommendations controller for app discovery and recommendation engine

import express from 'express';
import { query, pool } from '../database/connection';
import { AuthenticatedRequest } from '../utils/auth';
import { RecommendationData, APIResponse } from '../types/models';

const router = express.Router();

// GET /api/recommendations/personalized - Get personalized recommendations for user
router.get('/personalized', async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    const { limit = 20 } = req.query;

    let sql = '';
    let params: any[] = [];

    if (req.user) {
      // Personalized recommendations for authenticated users
      sql = `
        WITH user_history AS (
          -- Get user's download history
          SELECT DISTINCT app_id FROM user_downloads WHERE user_id = $1
          UNION
          -- Get user's review history
          SELECT DISTINCT app_id FROM reviews WHERE user_id = $1
          UNION
          -- Get user's favorite apps
          SELECT DISTINCT app_id FROM user_favorites WHERE user_id = $1
        ),
        similar_users AS (
          -- Find users with similar app usage
          SELECT DISTINCT uhd.user_id
          FROM user_downloads uhd
          JOIN user_downloads target ON uhd.app_id = target.app_id
          WHERE target.user_id = $1 AND uhd.user_id != $1
          LIMIT 100
        ),
        category_preferences AS (
          -- Get user's preferred categories based on their history
          SELECT 
            a.category_id,
            COUNT(*) as category_score
          FROM apps a
          JOIN user_history uh ON a.id = uh.app_id
          GROUP BY a.category_id
        ),
        recommendation_scores AS (
          SELECT 
            a.id as app_id,
            a.title,
            a.description,
            a.short_description,
            a.icon,
            a.rating,
            a.review_count,
            a.download_count,
            a.price,
            a.platform,
            c.name as category_name,
            sc.name as subcategory_name,
            u.name as developer_name,
            -- Scoring factors
            CASE WHEN uh.app_id IS NOT NULL THEN 0 ELSE 1 END as not_viewed_score,
            a.featured::int * 0.3 as featured_score,
            a.rating * 0.2 as rating_score,
            LEAST(a.download_count::decimal / 1000, 1) * 0.3 as popularity_score,
            CASE WHEN a.category_id IN (SELECT category_id FROM category_preferences) THEN 0.2 ELSE 0 END as category_match_score,
            COALESCE((
              SELECT COUNT(*) * 0.1
              FROM reviews r2 
              WHERE r2.app_id = a.id AND r2.user_id IN (SELECT user_id FROM similar_users)
            ), 0) as similar_user_score
          FROM apps a
          LEFT JOIN categories c ON a.category_id = c.id
          LEFT JOIN subcategories sc ON a.subcategory_id = sc.id
          LEFT JOIN users u ON a.developer_id = u.id
          LEFT JOIN user_history uh ON a.id = uh.app_id
          WHERE a.status = 'approved'
          AND a.id NOT IN (SELECT app_id FROM user_history)
        )
        SELECT 
          app_id,
          title,
          description,
          short_description,
          icon,
          rating,
          review_count,
          download_count,
          price,
          platform,
          category_name,
          subcategory_name,
          developer_name,
          (
            not_viewed_score + featured_score + rating_score + 
            popularity_score + category_match_score + similar_user_score
          ) as recommendation_score,
          CASE 
            WHEN similar_user_score > 0 THEN 'similar_users'
            WHEN category_match_score > 0 THEN 'category_match'
            WHEN featured_score > 0 THEN 'featured'
            WHEN popularity_score > 0.2 THEN 'popular'
            ELSE 'general'
          END as recommendation_reason
        FROM recommendation_scores
        WHERE recommendation_score > 0.3
        ORDER BY recommendation_score DESC, rating DESC
        LIMIT $2
      `;
      params = [req.user.id, parseInt(limit as string)];
    } else {
      // Generic recommendations for anonymous users
      sql = `
        SELECT 
          a.id as app_id,
          a.title,
          a.description,
          a.short_description,
          a.icon,
          a.rating,
          a.review_count,
          a.download_count,
          a.price,
          a.platform,
          c.name as category_name,
          sc.name as subcategory_name,
          u.name as developer_name,
          (
            a.featured::int * 0.4 +
            a.rating * 0.3 +
            LEAST(a.download_count::decimal / 1000, 1) * 0.3
          ) as recommendation_score,
          CASE 
            WHEN a.featured THEN 'featured'
            WHEN a.download_count > 1000 THEN 'popular'
            ELSE 'trending'
          END as recommendation_reason
        FROM apps a
        LEFT JOIN categories c ON a.category_id = c.id
        LEFT JOIN subcategories sc ON a.subcategory_id = sc.id
        LEFT JOIN users u ON a.developer_id = u.id
        WHERE a.status = 'approved'
        AND a.rating >= 4.0
        ORDER BY recommendation_score DESC, a.download_count DESC
        LIMIT $1
      `;
      params = [parseInt(limit as string)];
    }

    const result = await query(sql, params);

    const response: APIResponse<{
      recommendations: any[];
      personalized: boolean;
    }> = {
      success: true,
      data: {
        recommendations: result.rows,
        personalized: !!req.user
      }
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching personalized recommendations:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// GET /api/recommendations/similar/:appId - Get similar apps
router.get('/similar/:appId', async (req: express.Request, res: express.Response) => {
  try {
    const { appId } = req.params;
    const { limit = 10 } = req.query;

    // Get the target app details
    const targetAppResult = await query(
      'SELECT * FROM apps WHERE id = $1 AND status = $2',
      [appId, 'approved']
    );

    if (targetAppResult.rows.length === 0) {
      return res.status(404).json({ success: false, error: 'App not found' });
    }

    const targetApp = targetAppResult.rows[0];

    const sql = `
      SELECT 
        a.id as app_id,
        a.title,
        a.description,
        a.short_description,
        a.icon,
        a.rating,
        a.review_count,
        a.download_count,
        a.price,
        a.platform,
        c.name as category_name,
        sc.name as subcategory_name,
        u.name as developer_name,
        -- Calculate similarity score
        (
          CASE WHEN a.category_id = $2 THEN 30 ELSE 0 END +
          CASE WHEN a.subcategory_id = $3 THEN 20 ELSE 0 END +
          CASE WHEN a.platform && $4 THEN 15 ELSE 0 END +
          CASE WHEN a.grade_levels && $5 THEN 10 ELSE 0 END +
          CASE WHEN a.subjects && $6 THEN 10 ELSE 0 END +
          CASE WHEN a.price <= $7 THEN 5 ELSE 0 END
        ) as similarity_score
      FROM apps a
      LEFT JOIN categories c ON a.category_id = c.id
      LEFT JOIN subcategories sc ON a.subcategory_id = sc.id
      LEFT JOIN users u ON a.developer_id = u.id
      WHERE a.id != $1 
      AND a.status = 'approved'
      AND (
        a.category_id = $2 OR 
        a.subcategory_id = $3 OR
        a.platform && $4 OR
        a.grade_levels && $5 OR
        a.subjects && $6
      )
      ORDER BY similarity_score DESC, a.rating DESC
      LIMIT $8
    `;

    const params = [
      appId,
      targetApp.category_id,
      targetApp.subcategory_id,
      JSON.stringify(targetApp.platform),
      JSON.stringify(targetApp.grade_levels),
      JSON.stringify(targetApp.subjects),
      targetApp.price,
      parseInt(limit as string)
    ];

    const result = await query(sql, params);

    const response: APIResponse<any[]> = {
      success: true,
      data: result.rows
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching similar apps:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// GET /api/recommendations/trending - Get trending apps
router.get('/trending', async (req: express.Request, res: express.Response) => {
  try {
    const { period = '7d', limit = 20 } = req.query;

    let dateCondition = '';
    switch (period) {
      case '1d':
        dateCondition = 'aa.date >= CURRENT_DATE - INTERVAL \'1 day\'';
        break;
      case '7d':
        dateCondition = 'aa.date >= CURRENT_DATE - INTERVAL \'7 days\'';
        break;
      case '30d':
        dateCondition = 'aa.date >= CURRENT_DATE - INTERVAL \'30 days\'';
        break;
      default:
        dateCondition = 'aa.date >= CURRENT_DATE - INTERVAL \'7 days\'';
    }

    const sql = `
      SELECT 
        a.id as app_id,
        a.title,
        a.description,
        a.short_description,
        a.icon,
        a.rating,
        a.review_count,
        a.download_count,
        a.price,
        a.platform,
        c.name as category_name,
        sc.name as subcategory_name,
        u.name as developer_name,
        SUM(aa.views) as recent_views,
        SUM(aa.downloads) as recent_downloads,
        (
          SUM(aa.downloads)::decimal / NULLIF(SUM(aa.views), 0) * 100 +
          a.rating * 10
        ) as trending_score
      FROM apps a
      LEFT JOIN app_analytics aa ON a.id = aa.app_id
      LEFT JOIN categories c ON a.category_id = c.id
      LEFT JOIN subcategories sc ON a.subcategory_id = sc.id
      LEFT JOIN users u ON a.developer_id = u.id
      WHERE a.status = 'approved' AND ${dateCondition}
      GROUP BY a.id, a.title, a.description, a.short_description, a.icon, 
               a.rating, a.review_count, a.download_count, a.price, a.platform,
               c.name, sc.name, u.name
      HAVING SUM(aa.views) > 10
      ORDER BY trending_score DESC, a.rating DESC
      LIMIT $1
    `;

    const result = await query(sql, [parseInt(limit as string)]);

    const response: APIResponse<any[]> = {
      success: true,
      data: result.rows
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching trending apps:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// GET /api/recommendations/featured - Get featured apps
router.get('/featured', async (req: express.Request, res: express.Response) => {
  try {
    const { limit = 10 } = req.query;

    const sql = `
      SELECT 
        a.id as app_id,
        a.title,
        a.description,
        a.short_description,
        a.icon,
        a.rating,
        a.review_count,
        a.download_count,
        a.price,
        a.platform,
        c.name as category_name,
        sc.name as subcategory_name,
        u.name as developer_name
      FROM apps a
      LEFT JOIN categories c ON a.category_id = c.id
      LEFT JOIN subcategories sc ON a.subcategory_id = sc.id
      LEFT JOIN users u ON a.developer_id = u.id
      WHERE a.status = 'approved' AND a.featured = true
      ORDER BY a.rating DESC, a.download_count DESC
      LIMIT $1
    `;

    const result = await query(sql, [parseInt(limit as string)]);

    const response: APIResponse<any[]> = {
      success: true,
      data: result.rows
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching featured apps:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// GET /api/recommendations/category/:categoryId - Get recommendations for category
router.get('/category/:categoryId', async (req: express.Request, res: express.Response) => {
  try {
    const { categoryId } = req.params;
    const { limit = 20, subcategoryId } = req.query;

    let sql = `
      SELECT 
        a.id as app_id,
        a.title,
        a.description,
        a.short_description,
        a.icon,
        a.rating,
        a.review_count,
        a.download_count,
        a.price,
        a.platform,
        c.name as category_name,
        sc.name as subcategory_name,
        u.name as developer_name,
        (
          a.rating * 0.4 +
          LEAST(a.download_count::decimal / 500, 1) * 0.3 +
          a.review_count * 0.1 +
          a.featured::int * 0.2
        ) as recommendation_score
      FROM apps a
      LEFT JOIN categories c ON a.category_id = c.id
      LEFT JOIN subcategories sc ON a.subcategory_id = sc.id
      LEFT JOIN users u ON a.developer_id = u.id
      WHERE a.status = 'approved' AND a.category_id = $1
    `;

    const params = [categoryId];

    if (subcategoryId) {
      sql += ` AND a.subcategory_id = $2`;
      params.push(subcategoryId as string);
    }

    sql += ` ORDER BY recommendation_score DESC, a.rating DESC LIMIT $${params.length + 1}`;
    params.push(parseInt(limit as string));

    const result = await query(sql, params);

    const response: APIResponse<any[]> = {
      success: true,
      data: result.rows
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching category recommendations:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// GET /api/recommendations/grade-level/:gradeLevel - Get apps for specific grade level
router.get('/grade-level/:gradeLevel', async (req: express.Request, res: express.Response) => {
  try {
    const { gradeLevel } = req.params;
    const { limit = 20 } = req.query;

    const sql = `
      SELECT 
        a.id as app_id,
        a.title,
        a.description,
        a.short_description,
        a.icon,
        a.rating,
        a.review_count,
        a.download_count,
        a.price,
        a.platform,
        a.grade_levels,
        c.name as category_name,
        sc.name as subcategory_name,
        u.name as developer_name
      FROM apps a
      LEFT JOIN categories c ON a.category_id = c.id
      LEFT JOIN subcategories sc ON a.subcategory_id = sc.id
      LEFT JOIN users u ON a.developer_id = u.id
      WHERE a.status = 'approved' AND $1 = ANY(a.grade_levels)
      ORDER BY a.rating DESC, a.download_count DESC
      LIMIT $2
    `;

    const result = await query(sql, [gradeLevel, parseInt(limit as string)]);

    const response: APIResponse<any[]> = {
      success: true,
      data: result.rows
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching grade level recommendations:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// GET /api/recommendations/free - Get free apps
router.get('/free', async (req: express.Request, res: express.Response) => {
  try {
    const { limit = 20, categoryId } = req.query;

    let sql = `
      SELECT 
        a.id as app_id,
        a.title,
        a.description,
        a.short_description,
        a.icon,
        a.rating,
        a.review_count,
        a.download_count,
        a.platform,
        c.name as category_name,
        sc.name as subcategory_name,
        u.name as developer_name
      FROM apps a
      LEFT JOIN categories c ON a.category_id = c.id
      LEFT JOIN subcategories sc ON a.subcategory_id = sc.id
      LEFT JOIN users u ON a.developer_id = u.id
      WHERE a.status = 'approved' AND a.price = 0
    `;

    const params = [];

    if (categoryId) {
      sql += ` AND a.category_id = $1`;
      params.push(categoryId);
    }

    sql += ` ORDER BY a.rating DESC, a.download_count DESC LIMIT $${params.length + 1}`;
    params.push(parseInt(limit as string));

    const result = await query(sql, params);

    const response: APIResponse<any[]> = {
      success: true,
      data: result.rows
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching free apps:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// POST /api/recommendations/feedback - Record recommendation feedback
router.post('/feedback', async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    const { appId, recommendationType, helpful, userId } = req.body;

    if (!appId || recommendationType || helpful === undefined) {
      return res.status(400).json({
        success: false,
        error: 'App ID, recommendation type, and feedback are required'
      });
    }

    // Store recommendation feedback in a simple table or log it
    // For now, we'll just acknowledge the feedback
    console.log('Recommendation feedback:', {
      appId,
      recommendationType,
      helpful,
      userId: userId || req.user?.id,
      timestamp: new Date()
    });

    const response: APIResponse<null> = {
      success: true,
      message: 'Feedback recorded successfully'
    };

    res.json(response);
  } catch (error) {
    console.error('Error recording recommendation feedback:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// GET /api/recommendations/search - Search-based recommendations
router.get('/search/recommendations', async (req: express.Request, res: express.Response) => {
  try {
    const { query: searchQuery, limit = 20 } = req.query;

    if (!searchQuery) {
      return res.status(400).json({
        success: false,
        error: 'Search query is required'
      });
    }

    const sql = `
      SELECT 
        a.id as app_id,
        a.title,
        a.description,
        a.short_description,
        a.icon,
        a.rating,
        a.review_count,
        a.download_count,
        a.price,
        a.platform,
        a.tags,
        c.name as category_name,
        sc.name as subcategory_name,
        u.name as developer_name,
        -- Full-text search relevance score
        ts_rank(
          to_tsvector('english', a.title || ' ' || a.description || ' ' || COALESCE(a.tags, '')), 
          plainto_tsquery('english', $1)
        ) as relevance_score
      FROM apps a
      LEFT JOIN categories c ON a.category_id = c.id
      LEFT JOIN subcategories sc ON a.subcategory_id = sc.id
      LEFT JOIN users u ON a.developer_id = u.id
      WHERE a.status = 'approved' 
      AND to_tsvector('english', a.title || ' ' || a.description || ' ' || COALESCE(a.tags, '')) 
          @@ plainto_tsquery('english', $1)
      ORDER BY relevance_score DESC, a.rating DESC, a.download_count DESC
      LIMIT $2
    `;

    const result = await query(sql, [searchQuery, parseInt(limit as string)]);

    const response: APIResponse<any[]> = {
      success: true,
      data: result.rows
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching search recommendations:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

export { router as RecommendationRouter };