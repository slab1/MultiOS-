// Apps controller for managing educational applications

import express from 'express';
import { query } from '../database/connection';
import { AuthenticatedRequest } from '../utils/auth';
import { App, SearchFilters, APIResponse, PaginatedResponse } from '../types/models';
import multer from 'multer';
import path from 'path';

const router = express.Router();

// Configure multer for file uploads
const storage = multer.diskStorage({
  destination: (req, file, cb) => {
    cb(null, 'uploads/');
  },
  filename: (req, file, cb) => {
    const uniqueSuffix = Date.now() + '-' + Math.round(Math.random() * 1E9);
    cb(null, file.fieldname + '-' + uniqueSuffix + path.extname(file.originalname));
  }
});

const upload = multer({ 
  storage,
  limits: {
    fileSize: 10 * 1024 * 1024, // 10MB limit
  },
  fileFilter: (req, file, cb) => {
    const allowedTypes = ['image/jpeg', 'image/png', 'image/gif', 'video/mp4'];
    if (allowedTypes.includes(file.mimetype)) {
      cb(null, true);
    } else {
      cb(new Error('Invalid file type'));
    }
  }
});

// GET /api/apps - Search and filter apps
router.get('/', async (req: express.Request, res: express.Response) => {
  try {
    const {
      categories,
      subcategories,
      gradeLevels,
      subjects,
      price,
      platform,
      rating,
      searchQuery,
      sortBy = 'rating',
      sortOrder = 'desc',
      page = 1,
      limit = 20
    } = req.query as SearchFilters & { page?: string; limit?: string };

    let sql = `
      SELECT 
        a.*,
        c.name as category_name,
        sc.name as subcategory_name,
        u.name as developer_name,
        AVG(r.rating) as avg_rating,
        COUNT(r.id) as review_count
      FROM apps a
      LEFT JOIN categories c ON a.category_id = c.id
      LEFT JOIN subcategories sc ON a.subcategory_id = sc.id
      LEFT JOIN users u ON a.developer_id = u.id
      LEFT JOIN reviews r ON a.id = r.app_id
      WHERE a.status = 'approved'
    `;

    const params: any[] = [];
    let paramCount = 0;

    // Apply filters
    if (categories) {
      paramCount++;
      sql += ` AND a.category_id = ANY($${paramCount})`;
      params.push(categories);
    }

    if (subcategories) {
      paramCount++;
      sql += ` AND a.subcategory_id = ANY($${paramCount})`;
      params.push(subcategories);
    }

    if (gradeLevels) {
      paramCount++;
      sql += ` AND a.grade_levels && $${paramCount}`;
      params.push(gradeLevels);
    }

    if (subjects) {
      paramCount++;
      sql += ` AND a.subjects && $${paramCount}`;
      params.push(subjects);
    }

    if (price) {
      if (price === 'free') {
        sql += ` AND a.price = 0`;
      } else if (price === 'paid') {
        sql += ` AND a.price > 0`;
      }
    }

    if (platform) {
      paramCount++;
      sql += ` AND a.platform && $${paramCount}`;
      params.push(platform);
    }

    if (rating) {
      sql += ` AND a.rating >= $${++paramCount}`;
      params.push(rating);
    }

    if (searchQuery) {
      paramCount++;
      sql += ` AND to_tsvector('english', a.title || ' ' || a.description) @@ plainto_tsquery('english', $${paramCount})`;
      params.push(searchQuery);
    }

    // Group by to avoid duplicate apps from reviews join
    sql += ` GROUP BY a.id, c.name, sc.name, u.name`;

    // Add sorting
    const validSortFields = ['rating', 'download_count', 'created_at', 'title', 'price'];
    const sortField = validSortFields.includes(sortBy) ? sortBy : 'rating';
    sql += ` ORDER BY a.${sortField} ${sortOrder.toUpperCase() === 'ASC' ? 'ASC' : 'DESC'}`;

    // Add pagination
    const offset = (parseInt(page.toString()) - 1) * parseInt(limit.toString());
    sql += ` LIMIT $${++paramCount} OFFSET $${++paramCount}`;
    params.push(parseInt(limit.toString()), offset);

    const result = await query(sql, params);
    
    // Get total count for pagination
    let countSql = `SELECT COUNT(DISTINCT a.id) as total FROM apps a WHERE a.status = 'approved'`;
    const countParams: any[] = [];
    let countParamCount = 0;

    // Apply same filters to count query
    if (categories) {
      countParamCount++;
      countSql += ` AND a.category_id = ANY($${countParamCount})`;
      countParams.push(categories);
    }

    if (searchQuery) {
      countParamCount++;
      countSql += ` AND to_tsvector('english', a.title || ' ' || a.description) @@ plainto_tsquery('english', $${countParamCount})`;
      countParams.push(searchQuery);
    }

    const countResult = await query(countSql, countParams);
    const total = parseInt(countResult.rows[0].total);

    const response: APIResponse<PaginatedResponse<App>> = {
      success: true,
      data: {
        items: result.rows,
        total,
        page: parseInt(page.toString()),
        limit: parseInt(limit.toString()),
        totalPages: Math.ceil(total / parseInt(limit.toString()))
      }
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching apps:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// GET /api/apps/:id - Get app details
router.get('/:id', async (req: express.Request, res: express.Response) => {
  try {
    const { id } = req.params;

    const sql = `
      SELECT 
        a.*,
        c.name as category_name,
        sc.name as subcategory_name,
        u.name as developer_name,
        u.institution as developer_institution,
        AVG(r.rating) as avg_rating,
        COUNT(r.id) as review_count
      FROM apps a
      LEFT JOIN categories c ON a.category_id = c.id
      LEFT JOIN subcategories sc ON a.subcategory_id = sc.id
      LEFT JOIN users u ON a.developer_id = u.id
      LEFT JOIN reviews r ON a.id = r.app_id
      WHERE a.id = $1
      GROUP BY a.id, c.name, sc.name, u.name, u.institution
    `;

    const result = await query(sql, [id]);

    if (result.rows.length === 0) {
      return res.status(404).json({ success: false, error: 'App not found' });
    }

    // Increment view count
    await query(
      'UPDATE apps SET download_count = download_count + 1 WHERE id = $1',
      [id]
    );

    const response: APIResponse<App> = {
      success: true,
      data: result.rows[0]
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching app:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// POST /api/apps - Create new app (developers only)
router.post('/', upload.fields([
  { name: 'icon', maxCount: 1 },
  { name: 'screenshots', maxCount: 10 },
  { name: 'video', maxCount: 1 }
]), async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    const {
      title,
      description,
      shortDescription,
      categoryId,
      subcategoryId,
      gradeLevels,
      subjects,
      tags,
      price,
      currency,
      platform,
      version,
      websiteUrl,
      downloadUrl
    } = req.body;

    if (!req.user) {
      return res.status(401).json({ success: false, error: 'Authentication required' });
    }

    const sql = `
      INSERT INTO apps (
        title, description, short_description, developer_id, category_id, subcategory_id,
        grade_levels, subjects, tags, price, currency, platform, version, 
        screenshots, icon, video_url, website_url, download_url, status
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
      RETURNING *
    `;

    const files = req.files as { [fieldname: string]: Express.Multer.File[] };
    
    const params = [
      title,
      description,
      shortDescription,
      req.user.id,
      categoryId,
      subcategoryId || null,
      gradeLevels ? JSON.parse(gradeLevels) : [],
      subjects ? JSON.parse(subjects) : [],
      tags ? JSON.parse(tags) : [],
      parseFloat(price) || 0,
      currency || 'USD',
      platform ? JSON.parse(platform) : ['web'],
      version || '1.0.0',
      files.screenshots?.map(f => f.filename) || [],
      files.icon?.[0]?.filename || null,
      files.video?.[0]?.filename || null,
      websiteUrl,
      downloadUrl,
      'pending' // New apps start as pending review
    ];

    const result = await query(sql, params);

    const response: APIResponse<App> = {
      success: true,
      data: result.rows[0],
      message: 'App submitted for review successfully'
    };

    res.status(201).json(response);
  } catch (error) {
    console.error('Error creating app:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// PUT /api/apps/:id - Update app (developers only, their own apps)
router.put('/:id', upload.fields([
  { name: 'icon', maxCount: 1 },
  { name: 'screenshots', maxCount: 10 },
  { name: 'video', maxCount: 1 }
]), async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    const { id } = req.params;
    const updateData = req.body;

    // Check if app exists and belongs to user
    const existingApp = await query(
      'SELECT * FROM apps WHERE id = $1 AND developer_id = $2',
      [id, req.user?.id]
    );

    if (existingApp.rows.length === 0) {
      return res.status(404).json({ success: false, error: 'App not found or access denied' });
    }

    const files = req.files as { [fieldname: string]: Express.Multer.File[] };
    
    // Build dynamic update query
    const updates = [];
    const params: any[] = [];
    let paramCount = 0;

    const allowedFields = [
      'title', 'description', 'short_description', 'category_id', 'subcategory_id',
      'grade_levels', 'subjects', 'tags', 'price', 'currency', 'platform', 'version',
      'website_url', 'download_url', 'educational_impact', 'accessibility'
    ];

    for (const [field, value] of Object.entries(updateData)) {
      if (allowedFields.includes(field)) {
        paramCount++;
        updates.push(`${field} = $${paramCount}`);
        
        if (['grade_levels', 'subjects', 'tags', 'platform'].includes(field)) {
          params.push(JSON.parse(value as string));
        } else {
          params.push(value);
        }
      }
    }

    // Handle file uploads
    if (files.icon?.[0]) {
      paramCount++;
      updates.push(`icon = $${paramCount}`);
      params.push(files.icon[0].filename);
    }

    if (files.screenshots?.length > 0) {
      paramCount++;
      updates.push(`screenshots = $${paramCount}`);
      params.push(files.screenshots.map(f => f.filename));
    }

    if (files.video?.[0]) {
      paramCount++;
      updates.push(`video_url = $${paramCount}`);
      params.push(files.video[0].filename);
    }

    if (updates.length === 0) {
      return res.status(400).json({ success: false, error: 'No valid fields to update' });
    }

    paramCount++;
    updates.push(`updated_at = NOW()`);

    const sql = `
      UPDATE apps 
      SET ${updates.join(', ')}
      WHERE id = $${paramCount}
      RETURNING *
    `;

    params.push(id);
    const result = await query(sql, params);

    const response: APIResponse<App> = {
      success: true,
      data: result.rows[0],
      message: 'App updated successfully'
    };

    res.json(response);
  } catch (error) {
    console.error('Error updating app:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// DELETE /api/apps/:id - Delete app (developers only, their own apps)
router.delete('/:id', async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    const { id } = req.params;

    // Check if app exists and belongs to user
    const existingApp = await query(
      'SELECT * FROM apps WHERE id = $1 AND developer_id = $2',
      [id, req.user?.id]
    );

    if (existingApp.rows.length === 0) {
      return res.status(404).json({ success: false, error: 'App not found or access denied' });
    }

    // Soft delete by setting status to 'suspended'
    await query('UPDATE apps SET status = $1, updated_at = NOW() WHERE id = $2', ['suspended', id]);

    const response: APIResponse<null> = {
      success: true,
      message: 'App deleted successfully'
    };

    res.json(response);
  } catch (error) {
    console.error('Error deleting app:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// GET /api/apps/featured - Get featured apps
router.get('/featured/list', async (req: express.Request, res: express.Response) => {
  try {
    const sql = `
      SELECT 
        a.*,
        c.name as category_name,
        u.name as developer_name
      FROM apps a
      LEFT JOIN categories c ON a.category_id = c.id
      LEFT JOIN users u ON a.developer_id = u.id
      WHERE a.featured = true AND a.status = 'approved'
      ORDER BY a.rating DESC, a.download_count DESC
      LIMIT 10
    `;

    const result = await query(sql);

    const response: APIResponse<App[]> = {
      success: true,
      data: result.rows
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching featured apps:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// GET /api/apps/popular - Get popular apps
router.get('/popular/list', async (req: express.Request, res: express.Response) => {
  try {
    const { limit = 10 } = req.query;
    
    const sql = `
      SELECT 
        a.*,
        c.name as category_name,
        u.name as developer_name
      FROM apps a
      LEFT JOIN categories c ON a.category_id = c.id
      LEFT JOIN users u ON a.developer_id = u.id
      WHERE a.status = 'approved'
      ORDER BY a.download_count DESC, a.rating DESC
      LIMIT $1
    `;

    const result = await query(sql, [limit]);

    const response: APIResponse<App[]> = {
      success: true,
      data: result.rows
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching popular apps:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

export { router as AppRouter };