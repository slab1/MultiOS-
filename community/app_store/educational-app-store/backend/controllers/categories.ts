// Categories controller for managing app categories and subcategories

import express from 'express';
import { query } from '../database/connection';
import { Category, Subcategory, APIResponse } from '../types/models';
import { AdminMiddleware } from '../utils/auth';

const router = express.Router();

// GET /api/categories - Get all categories
router.get('/', async (req: express.Request, res: express.Response) => {
  try {
    const { includeSubcategories = 'false' } = req.query;

    let sql = `
      SELECT c.*, 
        COUNT(a.id) as app_count,
        AVG(a.rating) as avg_rating
      FROM categories c
      LEFT JOIN apps a ON c.id = a.category_id AND a.status = 'approved'
      WHERE c.active = true
      GROUP BY c.id
      ORDER BY c.sort_order ASC, c.name ASC
    `;

    const result = await query(sql);

    let categories = result.rows;

    // Optionally include subcategories
    if (includeSubcategories === 'true') {
      const subcategorySql = `
        SELECT sc.*, 
          COUNT(a.id) as app_count,
          AVG(a.rating) as avg_rating
        FROM subcategories sc
        LEFT JOIN apps a ON sc.id = a.subcategory_id AND a.status = 'approved'
        WHERE sc.active = true
        GROUP BY sc.id
        ORDER BY sc.sort_order ASC, sc.name ASC
      `;

      const subcategoryResult = await query(subcategorySql);
      const subcategories = subcategoryResult.rows;

      // Group subcategories by category
      categories = categories.map(category => ({
        ...category,
        subcategories: subcategories.filter(sc => sc.category_id === category.id)
      }));
    }

    const response: APIResponse<Category[]> = {
      success: true,
      data: categories
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching categories:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// GET /api/categories/:id - Get category by ID
router.get('/:id', async (req: express.Request, res: express.Response) => {
  try {
    const { id } = req.params;

    const sql = `
      SELECT c.*, 
        COUNT(a.id) as app_count,
        AVG(a.rating) as avg_rating
      FROM categories c
      LEFT JOIN apps a ON c.id = a.category_id AND a.status = 'approved'
      WHERE c.id = $1
      GROUP BY c.id
    `;

    const result = await query(sql, [id]);

    if (result.rows.length === 0) {
      return res.status(404).json({ success: false, error: 'Category not found' });
    }

    const response: APIResponse<Category> = {
      success: true,
      data: result.rows[0]
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching category:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// GET /api/categories/:id/apps - Get apps in a category
router.get('/:id/apps', async (req: express.Request, res: express.Response) => {
  try {
    const { id } = req.params;
    const { page = 1, limit = 20, sortBy = 'rating' } = req.query;

    const offset = (parseInt(page as string) - 1) * parseInt(limit as string);

    const sql = `
      SELECT 
        a.*,
        c.name as category_name,
        sc.name as subcategory_name,
        u.name as developer_name
      FROM apps a
      LEFT JOIN categories c ON a.category_id = c.id
      LEFT JOIN subcategories sc ON a.subcategory_id = sc.id
      LEFT JOIN users u ON a.developer_id = u.id
      WHERE a.category_id = $1 AND a.status = 'approved'
      ORDER BY a.${sortBy} DESC
      LIMIT $2 OFFSET $3
    `;

    const appsResult = await query(sql, [id, parseInt(limit as string), offset]);

    // Get total count
    const countSql = `SELECT COUNT(*) as total FROM apps WHERE category_id = $1 AND status = 'approved'`;
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
        items: appsResult.rows,
        total,
        page: parseInt(page as string),
        limit: parseInt(limit as string),
        totalPages: Math.ceil(total / parseInt(limit as string))
      }
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching category apps:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// POST /api/categories - Create new category (admin only)
router.post('/', AdminMiddleware, async (req: express.Request, res: express.Response) => {
  try {
    const { name, description, icon, color, parentId, sortOrder } = req.body;

    const sql = `
      INSERT INTO categories (name, description, icon, color, parent_id, sort_order)
      VALUES ($1, $2, $3, $4, $5, $6)
      RETURNING *
    `;

    const params = [
      name,
      description,
      icon,
      color,
      parentId || null,
      sortOrder || 0
    ];

    const result = await query(sql, params);

    const response: APIResponse<Category> = {
      success: true,
      data: result.rows[0],
      message: 'Category created successfully'
    };

    res.status(201).json(response);
  } catch (error) {
    console.error('Error creating category:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// PUT /api/categories/:id - Update category (admin only)
router.put('/:id', AdminMiddleware, async (req: express.Request, res: express.Response) => {
  try {
    const { id } = req.params;
    const { name, description, icon, color, parentId, sortOrder, active } = req.body;

    // Build dynamic update query
    const updates = [];
    const params: any[] = [];
    let paramCount = 0;

    if (name !== undefined) {
      paramCount++;
      updates.push(`name = $${paramCount}`);
      params.push(name);
    }

    if (description !== undefined) {
      paramCount++;
      updates.push(`description = $${paramCount}`);
      params.push(description);
    }

    if (icon !== undefined) {
      paramCount++;
      updates.push(`icon = $${paramCount}`);
      params.push(icon);
    }

    if (color !== undefined) {
      paramCount++;
      updates.push(`color = $${paramCount}`);
      params.push(color);
    }

    if (parentId !== undefined) {
      paramCount++;
      updates.push(`parent_id = $${paramCount}`);
      params.push(parentId);
    }

    if (sortOrder !== undefined) {
      paramCount++;
      updates.push(`sort_order = $${paramCount}`);
      params.push(sortOrder);
    }

    if (active !== undefined) {
      paramCount++;
      updates.push(`active = $${paramCount}`);
      params.push(active);
    }

    if (updates.length === 0) {
      return res.status(400).json({ success: false, error: 'No valid fields to update' });
    }

    paramCount++;
    updates.push(`id = $${paramCount}`);

    const sql = `
      UPDATE categories 
      SET ${updates.join(', ')}
      WHERE id = $${paramCount}
      RETURNING *
    `;

    params.push(id);
    const result = await query(sql, params);

    if (result.rows.length === 0) {
      return res.status(404).json({ success: false, error: 'Category not found' });
    }

    const response: APIResponse<Category> = {
      success: true,
      data: result.rows[0],
      message: 'Category updated successfully'
    };

    res.json(response);
  } catch (error) {
    console.error('Error updating category:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// DELETE /api/categories/:id - Delete category (admin only)
router.delete('/:id', AdminMiddleware, async (req: express.Request, res: express.Response) => {
  try {
    const { id } = req.params;

    // Check if category has apps
    const appCountResult = await query(
      'SELECT COUNT(*) as count FROM apps WHERE category_id = $1',
      [id]
    );

    if (parseInt(appCountResult.rows[0].count) > 0) {
      return res.status(400).json({
        success: false,
        error: 'Cannot delete category with existing apps. Please move or delete the apps first.'
      });
    }

    // Soft delete by setting active to false
    const result = await query(
      'UPDATE categories SET active = false WHERE id = $1 RETURNING *',
      [id]
    );

    if (result.rows.length === 0) {
      return res.status(404).json({ success: false, error: 'Category not found' });
    }

    const response: APIResponse<null> = {
      success: true,
      message: 'Category deleted successfully'
    };

    res.json(response);
  } catch (error) {
    console.error('Error deleting category:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// GET /api/categories/tree - Get category tree
router.get('/tree/all', async (req: express.Request, res: express.Response) => {
  try {
    // Get all categories
    const categoriesSql = `
      SELECT c.*, 
        COUNT(a.id) as app_count,
        AVG(a.rating) as avg_rating
      FROM categories c
      LEFT JOIN apps a ON c.id = a.category_id AND a.status = 'approved'
      WHERE c.active = true
      GROUP BY c.id
      ORDER BY c.sort_order ASC, c.name ASC
    `;

    const categoriesResult = await query(categoriesSql);
    const categories = categoriesResult.rows;

    // Get all subcategories
    const subcategoriesSql = `
      SELECT sc.*, 
        COUNT(a.id) as app_count,
        AVG(a.rating) as avg_rating
      FROM subcategories sc
      LEFT JOIN apps a ON sc.id = a.subcategory_id AND a.status = 'approved'
      WHERE sc.active = true
      GROUP BY sc.id
      ORDER BY sc.sort_order ASC, sc.name ASC
    `;

    const subcategoriesResult = await query(subcategoriesSql);
    const subcategories = subcategoriesResult.rows;

    // Build tree structure
    const categoryTree = categories.map(category => ({
      ...category,
      subcategories: subcategories.filter(sc => sc.category_id === category.id)
    }));

    const response: APIResponse<any[]> = {
      success: true,
      data: categoryTree
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching category tree:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

export { router as CategoryRouter };