// Submissions controller for managing app submission workflow

import express from 'express';
import { query } from '../database/connection';
import { AuthenticatedRequest, AdminMiddleware, DeveloperMiddleware } from '../utils/auth';
import { DeveloperSubmission, APIResponse } from '../types/models';

const router = express.Router();

// GET /api/submissions/my - Get current user's submissions (developers)
router.get('/my', DeveloperMiddleware, async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    if (!req.user) {
      return res.status(401).json({ success: false, error: 'Authentication required' });
    }

    const { status, page = 1, limit = 20 } = req.query;

    let sql = `
      SELECT 
        ds.*,
        a.title as app_title,
        a.icon as app_icon
      FROM developer_submissions ds
      LEFT JOIN apps a ON ds.app_id = a.id
      WHERE ds.developer_id = $1
    `;

    const params = [req.user.id];

    if (status && status !== 'all') {
      sql += ` AND ds.status = $2`;
      params.push(status as string);
    }

    sql += ` ORDER BY ds.updated_at DESC LIMIT $3 OFFSET $4`;
    params.push(parseInt(limit as string), (parseInt(page as string) - 1) * parseInt(limit as string));

    const result = await query(sql, params);

    // Get total count
    let countSql = `SELECT COUNT(*) as total FROM developer_submissions WHERE developer_id = $1`;
    const countParams = [req.user.id];
    
    if (status && status !== 'all') {
      countSql += ` AND status = $2`;
      countParams.push(status as string);
    }

    const countResult = await query(countSql, countParams);
    const total = parseInt(countResult.rows[0].total);

    const response: APIResponse<{
      items: DeveloperSubmission[];
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
    console.error('Error fetching submissions:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// GET /api/submissions/:id - Get submission details
router.get('/:id', async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    const { id } = req.params;

    const sql = `
      SELECT 
        ds.*,
        a.title as app_title,
        u.name as developer_name
      FROM developer_submissions ds
      LEFT JOIN users u ON ds.developer_id = u.id
      LEFT JOIN apps a ON ds.app_id = a.id
      WHERE ds.id = $1
    `;

    const result = await query(sql, [id]);

    if (result.rows.length === 0) {
      return res.status(404).json({ success: false, error: 'Submission not found' });
    }

    const submission = result.rows[0];

    // Check access permissions
    if (req.user && req.user.role === 'developer' && submission.developer_id !== req.user.id) {
      return res.status(403).json({ success: false, error: 'Access denied' });
    }

    const response: APIResponse<DeveloperSubmission> = {
      success: true,
      data: submission
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching submission:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// POST /api/submissions - Create new submission (developers)
router.post('/', DeveloperMiddleware, async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    if (!req.user) {
      return res.status(401).json({ success: false, error: 'Authentication required' });
    }

    const {
      appId,
      title,
      description,
      version,
      changelog,
      technicalRequirements,
      educationalContent,
      mediaFiles,
      complianceData
    } = req.body;

    // Validate required fields
    if (!title || !description || !version) {
      return res.status(400).json({
        success: false,
        error: 'Title, description, and version are required'
      });
    }

    // If appId is provided, check if it belongs to user
    if (appId) {
      const appResult = await query(
        'SELECT id FROM apps WHERE id = $1 AND developer_id = $2',
        [appId, req.user.id]
      );

      if (appResult.rows.length === 0) {
        return res.status(404).json({
          success: false,
          error: 'App not found or access denied'
        });
      }
    }

    const sql = `
      INSERT INTO developer_submissions (
        developer_id, app_id, title, description, version, changelog,
        technical_requirements, educational_content, media_files, compliance_data,
        status
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
      RETURNING *
    `;

    const params = [
      req.user.id,
      appId || null,
      title,
      description,
      version,
      changelog || null,
      technicalRequirements || {},
      educationalContent || {},
      mediaFiles || {},
      complianceData || {},
      'draft'
    ];

    const result = await query(sql, params);

    const response: APIResponse<DeveloperSubmission> = {
      success: true,
      data: result.rows[0],
      message: 'Submission created successfully'
    };

    res.status(201).json(response);
  } catch (error) {
    console.error('Error creating submission:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// PUT /api/submissions/:id - Update submission (developers, their own submissions)
router.put('/:id', DeveloperMiddleware, async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    if (!req.user) {
      return res.status(401).json({ success: false, error: 'Authentication required' });
    }

    const { id } = req.params;
    const {
      title,
      description,
      version,
      changelog,
      technicalRequirements,
      educationalContent,
      mediaFiles,
      complianceData
    } = req.body;

    // Check if submission exists and belongs to user
    const existingSubmission = await query(
      'SELECT * FROM developer_submissions WHERE id = $1 AND developer_id = $2',
      [id, req.user.id]
    );

    if (existingSubmission.rows.length === 0) {
      return res.status(404).json({
        success: false,
        error: 'Submission not found or access denied'
      });
    }

    const submission = existingSubmission.rows[0];

    // Can't update submitted submissions
    if (['submitted', 'under_review', 'approved', 'rejected'].includes(submission.status)) {
      return res.status(400).json({
        success: false,
        error: 'Cannot update submitted or reviewed submissions'
      });
    }

    // Build dynamic update query
    const updates = [];
    const params: any[] = [];
    let paramCount = 0;

    const allowedFields = [
      'title', 'description', 'version', 'changelog',
      'technical_requirements', 'educational_content', 'media_files', 'compliance_data'
    ];

    for (const [field, value] of Object.entries({
      title,
      description,
      version,
      changelog,
      technicalRequirements,
      educationalContent,
      mediaFiles,
      complianceData
    })) {
      if (value !== undefined) {
        paramCount++;
        const dbField = field.replace(/([A-Z])/g, '_$1').toLowerCase();
        updates.push(`${dbField} = $${paramCount}`);
        
        if (['technicalRequirements', 'educationalContent', 'mediaFiles', 'complianceData'].includes(field)) {
          params.push(value || {});
        } else {
          params.push(value);
        }
      }
    }

    if (updates.length === 0) {
      return res.status(400).json({ success: false, error: 'No valid fields to update' });
    }

    paramCount++;
    updates.push(`updated_at = NOW()`);

    const sql = `
      UPDATE developer_submissions 
      SET ${updates.join(', ')}
      WHERE id = $${paramCount}
      RETURNING *
    `;

    params.push(id);
    const result = await query(sql, params);

    const response: APIResponse<DeveloperSubmission> = {
      success: true,
      data: result.rows[0],
      message: 'Submission updated successfully'
    };

    res.json(response);
  } catch (error) {
    console.error('Error updating submission:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// POST /api/submissions/:id/submit - Submit for review (developers)
router.post('/:id/submit', DeveloperMiddleware, async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    if (!req.user) {
      return res.status(401).json({ success: false, error: 'Authentication required' });
    }

    const { id } = req.params;

    // Check if submission exists and belongs to user
    const existingSubmission = await query(
      'SELECT * FROM developer_submissions WHERE id = $1 AND developer_id = $2',
      [id, req.user.id]
    );

    if (existingSubmission.rows.length === 0) {
      return res.status(404).json({
        success: false,
        error: 'Submission not found or access denied'
      });
    }

    const submission = existingSubmission.rows[0];

    if (submission.status !== 'draft') {
      return res.status(400).json({
        success: false,
        error: 'Only draft submissions can be submitted for review'
      });
    }

    // Basic validation before submission
    if (!submission.title || !submission.description || !submission.version) {
      return res.status(400).json({
        success: false,
        error: 'Title, description, and version are required before submission'
      });
    }

    // Update submission status
    const sql = `
      UPDATE developer_submissions 
      SET status = $1, submitted_at = NOW(), updated_at = NOW()
      WHERE id = $2
      RETURNING *
    `;

    const result = await query(sql, ['submitted', id]);

    const response: APIResponse<DeveloperSubmission> = {
      success: true,
      data: result.rows[0],
      message: 'Submission submitted for review successfully'
    };

    res.json(response);
  } catch (error) {
    console.error('Error submitting for review:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// GET /api/submissions/pending - Get pending submissions (admin only)
router.get('/pending/all', AdminMiddleware, async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    const { page = 1, limit = 20 } = req.query;

    const offset = (parseInt(page as string) - 1) * parseInt(limit as string);

    const sql = `
      SELECT 
        ds.*,
        u.name as developer_name,
        u.email as developer_email,
        a.title as existing_app_title
      FROM developer_submissions ds
      JOIN users u ON ds.developer_id = u.id
      LEFT JOIN apps a ON ds.app_id = a.id
      WHERE ds.status IN ('submitted', 'under_review')
      ORDER BY ds.submitted_at ASC
      LIMIT $1 OFFSET $2
    `;

    const result = await query(sql, [parseInt(limit as string), offset]);

    // Get total count
    const countSql = `
      SELECT COUNT(*) as total 
      FROM developer_submissions 
      WHERE status IN ('submitted', 'under_review')
    `;

    const countResult = await query(countSql);
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
    console.error('Error fetching pending submissions:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// PUT /api/submissions/:id/review - Review submission (admin only)
router.put('/:id/review', AdminMiddleware, async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    if (!req.user) {
      return res.status(401).json({ success: false, error: 'Authentication required' });
    }

    const { id } = req.params;
    const { status, reviewerNotes } = req.body;

    if (!['under_review', 'needs_changes', 'approved', 'rejected'].includes(status)) {
      return res.status(400).json({
        success: false,
        error: 'Invalid review status'
      });
    }

    // Check if submission exists
    const existingSubmission = await query(
      'SELECT * FROM developer_submissions WHERE id = $1',
      [id]
    );

    if (existingSubmission.rows.length === 0) {
      return res.status(404).json({
        success: false,
        error: 'Submission not found'
      });
    }

    const submission = existingSubmission.rows[0];

    if (!['submitted', 'under_review'].includes(submission.status)) {
      return res.status(400).json({
        success: false,
        error: 'Only submitted or under review submissions can be reviewed'
      });
    }

    // Update submission
    const sql = `
      UPDATE developer_submissions 
      SET status = $1, reviewer_notes = $2, reviewed_at = NOW(), updated_at = NOW()
      WHERE id = $3
      RETURNING *
    `;

    const result = await query(sql, [status, reviewerNotes || null, id]);

    // If approved, create or update the app
    if (status === 'approved') {
      await approveSubmission(result.rows[0], req.user.id);
    }

    const response: APIResponse<DeveloperSubmission> = {
      success: true,
      data: result.rows[0],
      message: 'Submission reviewed successfully'
    };

    res.json(response);
  } catch (error) {
    console.error('Error reviewing submission:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// Helper function to approve submission and create/update app
const approveSubmission = async (submission: any, reviewerId: string) => {
  try {
    if (submission.app_id) {
      // Update existing app
      await query(
        `UPDATE apps SET 
          title = $1, description = $2, version = $3, 
          technical_requirements = $4, educational_content = $5,
          status = 'approved', updated_at = NOW(), approved_by = $6
         WHERE id = $7`,
        [
          submission.title,
          submission.description,
          submission.version,
          submission.technical_requirements,
          submission.educational_content,
          reviewerId,
          submission.app_id
        ]
      );
    } else {
      // Create new app
      await query(
        `INSERT INTO apps (
          title, description, developer_id, status, approved_by, approved_at
         ) VALUES ($1, $2, $3, 'approved', $4, NOW())`,
        [
          submission.title,
          submission.description,
          submission.developer_id,
          reviewerId
        ]
      );
    }
  } catch (error) {
    console.error('Error approving submission:', error);
    throw error;
  }
};

// GET /api/submissions/stats - Get submission statistics (admin)
router.get('/stats/overview', AdminMiddleware, async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    const statsSql = `
      SELECT 
        COUNT(*) as total_submissions,
        COUNT(CASE WHEN status = 'draft' THEN 1 END) as draft_count,
        COUNT(CASE WHEN status = 'submitted' THEN 1 END) as submitted_count,
        COUNT(CASE WHEN status = 'under_review' THEN 1 END) as under_review_count,
        COUNT(CASE WHEN status = 'needs_changes' THEN 1 END) as needs_changes_count,
        COUNT(CASE WHEN status = 'approved' THEN 1 END) as approved_count,
        COUNT(CASE WHEN status = 'rejected' THEN 1 END) as rejected_count
      FROM developer_submissions
    `;

    const result = await query(statsSql);
    const stats = result.rows[0];

    // Recent submissions
    const recentSql = `
      SELECT ds.*, u.name as developer_name
      FROM developer_submissions ds
      JOIN users u ON ds.developer_id = u.id
      WHERE ds.submitted_at >= NOW() - INTERVAL '7 days'
      ORDER BY ds.submitted_at DESC
      LIMIT 10
    `;

    const recentResult = await query(recentSql);

    const response: APIResponse<{
      stats: any;
      recentSubmissions: any[];
    }> = {
      success: true,
      data: {
        stats: {
          total: parseInt(stats.total_submissions),
          draft: parseInt(stats.draft_count),
          submitted: parseInt(stats.submitted_count),
          underReview: parseInt(stats.under_review_count),
          needsChanges: parseInt(stats.needs_changes_count),
          approved: parseInt(stats.approved_count),
          rejected: parseInt(stats.rejected_count)
        },
        recentSubmissions: recentResult.rows
      }
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching submission stats:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// DELETE /api/submissions/:id - Delete submission (developers, their own, draft only)
router.delete('/:id', DeveloperMiddleware, async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    if (!req.user) {
      return res.status(401).json({ success: false, error: 'Authentication required' });
    }

    const { id } = req.params;

    // Check if submission exists and belongs to user
    const existingSubmission = await query(
      'SELECT * FROM developer_submissions WHERE id = $1 AND developer_id = $2',
      [id, req.user.id]
    );

    if (existingSubmission.rows.length === 0) {
      return res.status(404).json({
        success: false,
        error: 'Submission not found or access denied'
      });
    }

    const submission = existingSubmission.rows[0];

    if (submission.status !== 'draft') {
      return res.status(400).json({
        success: false,
        error: 'Only draft submissions can be deleted'
      });
    }

    // Delete submission
    await query('DELETE FROM developer_submissions WHERE id = $1', [id]);

    const response: APIResponse<null> = {
      success: true,
      message: 'Submission deleted successfully'
    };

    res.json(response);
  } catch (error) {
    console.error('Error deleting submission:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

export { router as SubmissionRouter };