// Users controller for user management and authentication

import express from 'express';
import bcrypt from 'bcryptjs';
import { query } from '../database/connection';
import { AuthenticatedRequest, generateToken } from '../utils/auth';
import { User, APIResponse } from '../types/models';

const router = express.Router();

// POST /api/users/register - Register new user
router.post('/register', async (req: express.Request, res: express.Response) => {
  try {
    const { email, password, name, role, institution, bio } = req.body;

    // Validate input
    if (!email || !password || !name || !role) {
      return res.status(400).json({
        success: false,
        error: 'Email, password, name, and role are required'
      });
    }

    if (!['developer', 'educator'].includes(role)) {
      return res.status(400).json({
        success: false,
        error: 'Role must be either developer or educator'
      });
    }

    // Check if user already exists
    const existingUser = await query(
      'SELECT id FROM users WHERE email = $1',
      [email]
    );

    if (existingUser.rows.length > 0) {
      return res.status(400).json({
        success: false,
        error: 'User with this email already exists'
      });
    }

    // Hash password
    const saltRounds = 10;
    const passwordHash = await bcrypt.hash(password, saltRounds);

    // Create user
    const sql = `
      INSERT INTO users (email, password_hash, name, role, institution, bio)
      VALUES ($1, $2, $3, $4, $5, $6)
      RETURNING id, email, name, role, institution, bio, created_at
    `;

    const params = [
      email,
      passwordHash,
      name,
      role,
      institution || null,
      bio || null
    ];

    const result = await query(sql, params);
    const newUser = result.rows[0];

    // Generate token
    const token = generateToken(newUser);

    const response: APIResponse<{ user: User; token: string }> = {
      success: true,
      data: {
        user: newUser,
        token
      },
      message: 'User registered successfully'
    };

    res.status(201).json(response);
  } catch (error) {
    console.error('Error registering user:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// POST /api/users/login - Login user
router.post('/login', async (req: express.Request, res: express.Response) => {
  try {
    const { email, password } = req.body;

    // Validate input
    if (!email || !password) {
      return res.status(400).json({
        success: false,
        error: 'Email and password are required'
      });
    }

    // Find user
    const result = await query(
      'SELECT * FROM users WHERE email = $1',
      [email]
    );

    if (result.rows.length === 0) {
      return res.status(401).json({
        success: false,
        error: 'Invalid email or password'
      });
    }

    const user = result.rows[0];

    // Check password
    const passwordMatch = await bcrypt.compare(password, user.password_hash);
    
    if (!passwordMatch) {
      return res.status(401).json({
        success: false,
        error: 'Invalid email or password'
      });
    }

    // Generate token
    const token = generateToken(user);

    // Remove password from response
    const { password_hash, ...userWithoutPassword } = user;

    const response: APIResponse<{ user: User; token: string }> = {
      success: true,
      data: {
        user: userWithoutPassword,
        token
      },
      message: 'Login successful'
    };

    res.json(response);
  } catch (error) {
    console.error('Error logging in:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// GET /api/users/profile - Get current user profile
router.get('/profile', async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    if (!req.user) {
      return res.status(401).json({ success: false, error: 'Authentication required' });
    }

    const result = await query(
      'SELECT id, email, name, role, institution, bio, avatar, created_at FROM users WHERE id = $1',
      [req.user.id]
    );

    if (result.rows.length === 0) {
      return res.status(404).json({ success: false, error: 'User not found' });
    }

    const response: APIResponse<User> = {
      success: true,
      data: result.rows[0]
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching user profile:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// PUT /api/users/profile - Update user profile
router.put('/profile', async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    if (!req.user) {
      return res.status(401).json({ success: false, error: 'Authentication required' });
    }

    const { name, institution, bio, avatar } = req.body;

    // Build dynamic update query
    const updates = [];
    const params: any[] = [];
    let paramCount = 0;

    if (name !== undefined) {
      paramCount++;
      updates.push(`name = $${paramCount}`);
      params.push(name);
    }

    if (institution !== undefined) {
      paramCount++;
      updates.push(`institution = $${paramCount}`);
      params.push(institution);
    }

    if (bio !== undefined) {
      paramCount++;
      updates.push(`bio = $${paramCount}`);
      params.push(bio);
    }

    if (avatar !== undefined) {
      paramCount++;
      updates.push(`avatar = $${paramCount}`);
      params.push(avatar);
    }

    if (updates.length === 0) {
      return res.status(400).json({ success: false, error: 'No valid fields to update' });
    }

    paramCount++;
    updates.push(`updated_at = NOW()`);

    const sql = `
      UPDATE users 
      SET ${updates.join(', ')}
      WHERE id = $${paramCount}
      RETURNING id, email, name, role, institution, bio, avatar, updated_at
    `;

    params.push(req.user.id);
    const result = await query(sql, params);

    if (result.rows.length === 0) {
      return res.status(404).json({ success: false, error: 'User not found' });
    }

    const response: APIResponse<User> = {
      success: true,
      data: result.rows[0],
      message: 'Profile updated successfully'
    };

    res.json(response);
  } catch (error) {
    console.error('Error updating user profile:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// GET /api/users/:id - Get user public profile
router.get('/:id', async (req: express.Request, res: express.Response) => {
  try {
    const { id } = req.params;

    const result = await query(
      'SELECT id, name, role, institution, bio, avatar, created_at FROM users WHERE id = $1',
      [id]
    );

    if (result.rows.length === 0) {
      return res.status(404).json({ success: false, error: 'User not found' });
    }

    const response: APIResponse<any> = {
      success: true,
      data: result.rows[0]
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching user:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// GET /api/users/:id/apps - Get apps developed by user
router.get('/:id/apps', async (req: express.Request, res: express.Response) => {
  try {
    const { id } = req.params;
    const { status = 'approved', page = 1, limit = 20 } = req.query;

    const offset = (parseInt(page as string) - 1) * parseInt(limit as string);

    let sql = `
      SELECT 
        a.*,
        c.name as category_name,
        sc.name as subcategory_name
      FROM apps a
      LEFT JOIN categories c ON a.category_id = c.id
      LEFT JOIN subcategories sc ON a.subcategory_id = sc.id
      WHERE a.developer_id = $1
    `;

    const params = [id];

    if (status !== 'all') {
      sql += ` AND a.status = $2`;
      params.push(status as string);
    }

    sql += ` ORDER BY a.created_at DESC LIMIT $3 OFFSET $4`;
    params.push(parseInt(limit as string), offset);

    const result = await query(sql, params);

    // Get total count
    let countSql = `SELECT COUNT(*) as total FROM apps WHERE developer_id = $1`;
    const countParams = [id];
    
    if (status !== 'all') {
      countSql += ` AND status = $2`;
      countParams.push(status as string);
    }

    const countResult = await query(countSql, countParams);
    const total = parseInt(countResult.rows[0].total);

    const response: APIResponse<{
      items: any[];
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
    console.error('Error fetching user apps:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// GET /api/users/:id/reviews - Get reviews by user
router.get('/:id/reviews', async (req: express.Request, res: express.Response) => {
  try {
    const { id } = req.params;
    const { page = 1, limit = 10 } = req.query;

    const offset = (parseInt(page as string) - 1) * parseInt(limit as string);

    const sql = `
      SELECT 
        r.*,
        a.title as app_title,
        a.icon as app_icon,
        c.name as category_name
      FROM reviews r
      LEFT JOIN apps a ON r.app_id = a.id
      LEFT JOIN categories c ON a.category_id = c.id
      WHERE r.user_id = $1
      ORDER BY r.created_at DESC
      LIMIT $2 OFFSET $3
    `;

    const result = await query(sql, [id, parseInt(limit as string), offset]);

    // Get total count
    const countSql = `SELECT COUNT(*) as total FROM reviews WHERE user_id = $1`;
    const countResult = await query(countSql, [id]);
    const total = parseInt(countResult.rows[0].total);

    const response: APIResponse<{
      items: any[];
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
    console.error('Error fetching user reviews:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// GET /api/users/:id/favorites - Get user's favorite apps
router.get('/:id/favorites', async (req: express.Request, res: express.Response) => {
  try {
    const { id } = req.params;
    const { page = 1, limit = 20 } = req.query;

    const offset = (parseInt(page as string) - 1) * parseInt(limit as string);

    const sql = `
      SELECT 
        a.*,
        c.name as category_name,
        sc.name as subcategory_name,
        u.name as developer_name,
        f.created_at as favorited_at
      FROM user_favorites f
      JOIN apps a ON f.app_id = a.id
      LEFT JOIN categories c ON a.category_id = c.id
      LEFT JOIN subcategories sc ON a.subcategory_id = sc.id
      LEFT JOIN users u ON a.developer_id = u.id
      WHERE f.user_id = $1 AND a.status = 'approved'
      ORDER BY f.created_at DESC
      LIMIT $2 OFFSET $3
    `;

    const result = await query(sql, [id, parseInt(limit as string), offset]);

    // Get total count
    const countSql = `
      SELECT COUNT(*) as total 
      FROM user_favorites f
      JOIN apps a ON f.app_id = a.id
      WHERE f.user_id = $1 AND a.status = 'approved'
    `;

    const countResult = await query(countSql, [id]);
    const total = parseInt(countResult.rows[0].total);

    const response: APIResponse<{
      items: any[];
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
    console.error('Error fetching user favorites:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// POST /api/users/:id/favorites - Add app to favorites
router.post('/:id/favorites', async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    if (!req.user) {
      return res.status(401).json({ success: false, error: 'Authentication required' });
    }

    const { id } = req.params;
    const { appId } = req.body;

    if (id !== req.user.id) {
      return res.status(403).json({ success: false, error: 'Access denied' });
    }

    // Check if app exists
    const appResult = await query(
      'SELECT id FROM apps WHERE id = $1 AND status = $2',
      [appId, 'approved']
    );

    if (appResult.rows.length === 0) {
      return res.status(404).json({ success: false, error: 'App not found' });
    }

    // Check if already favorited
    const existingFavorite = await query(
      'SELECT id FROM user_favorites WHERE user_id = $1 AND app_id = $2',
      [id, appId]
    );

    if (existingFavorite.rows.length > 0) {
      return res.status(400).json({
        success: false,
        error: 'App already in favorites'
      });
    }

    // Add to favorites
    await query(
      'INSERT INTO user_favorites (user_id, app_id) VALUES ($1, $2)',
      [id, appId]
    );

    const response: APIResponse<null> = {
      success: true,
      message: 'App added to favorites'
    };

    res.status(201).json(response);
  } catch (error) {
    console.error('Error adding to favorites:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// DELETE /api/users/:id/favorites/:appId - Remove app from favorites
router.delete('/:id/favorites/:appId', async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    if (!req.user) {
      return res.status(401).json({ success: false, error: 'Authentication required' });
    }

    const { id, appId } = req.params;

    if (id !== req.user.id) {
      return res.status(403).json({ success: false, error: 'Access denied' });
    }

    // Check if favorite exists
    const existingFavorite = await query(
      'SELECT id FROM user_favorites WHERE user_id = $1 AND app_id = $2',
      [id, appId]
    );

    if (existingFavorite.rows.length === 0) {
      return res.status(404).json({
        success: false,
        error: 'App not in favorites'
      });
    }

    // Remove from favorites
    await query(
      'DELETE FROM user_favorites WHERE user_id = $1 AND app_id = $2',
      [id, appId]
    );

    const response: APIResponse<null> = {
      success: true,
      message: 'App removed from favorites'
    };

    res.json(response);
  } catch (error) {
    console.error('Error removing from favorites:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

export { router as UserRouter };