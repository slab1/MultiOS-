// Analytics controller for tracking app usage and providing insights

import express from 'express';
import { query } from '../database/connection';
import { AuthenticatedRequest, AdminMiddleware } from '../utils/auth';
import { AppAnalytics, APIResponse } from '../types/models';

const router = express.Router();

// POST /api/analytics/track - Track app interaction (public endpoint)
router.post('/track', async (req: express.Request, res: express.Response) => {
  try {
    const { 
      appId, 
      eventType, 
      userId, 
      sessionId, 
      metadata,
      deviceInfo,
      location
    } = req.body;

    if (!appId || !eventType) {
      return res.status(400).json({
        success: false,
        error: 'App ID and event type are required'
      });
    }

    // Update daily analytics
    const today = new Date().toISOString().split('T')[0];
    
    const updateAnalyticsSql = `
      INSERT INTO app_analytics (app_id, date, views, downloads, reviews, device_types, geographic_data)
      VALUES ($1, $2, 
        CASE WHEN $3 = 'view' THEN 1 ELSE 0 END,
        CASE WHEN $3 = 'download' THEN 1 ELSE 0 END,
        CASE WHEN $3 = 'review' THEN 1 ELSE 0 END,
        CASE WHEN $4 IS NOT NULL THEN $4 ELSE '[]' END,
        CASE WHEN $5 IS NOT NULL THEN $5 ELSE '[]' END
      )
      ON CONFLICT (app_id, date) 
      DO UPDATE SET
        views = app_analytics.views + CASE WHEN $3 = 'view' THEN 1 ELSE 0 END,
        downloads = app_analytics.downloads + CASE WHEN $3 = 'download' THEN 1 ELSE 0 END,
        reviews = app_analytics.reviews + CASE WHEN $3 = 'review' THEN 1 ELSE 0 END,
        device_types = CASE 
          WHEN $4 IS NOT NULL THEN (
            SELECT jsonb_agg(
              jsonb_build_object(
                'device', device_data->>'device',
                'count', (device_data->>'count')::int + CASE WHEN $3 = 'view' AND device_data->>'device' = $4->>'device' THEN 1 ELSE 0 END,
                'percentage', device_data->>'percentage'
              )
            )
            FROM jsonb_array_elements(app_analytics.device_types) AS device_data
          )
          ELSE app_analytics.device_types
        END,
        geographic_data = app_analytics.geographic_data
    `;

    await query(updateAnalyticsSql, [
      appId,
      today,
      eventType,
      deviceInfo ? JSON.stringify([{ device: deviceInfo.device || 'unknown', count: 1, percentage: 0 }]) : null,
      location ? JSON.stringify([{ country: location.country || 'unknown', count: 1, percentage: 0 }]) : null
    ]);

    // Track user downloads if applicable
    if (eventType === 'download' && userId) {
      await query(
        'INSERT INTO user_downloads (user_id, app_id) VALUES ($1, $2) ON CONFLICT (user_id, app_id) DO NOTHING',
        [userId, appId]
      );
    }

    const response: APIResponse<null> = {
      success: true,
      message: 'Event tracked successfully'
    };

    res.json(response);
  } catch (error) {
    console.error('Error tracking event:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// GET /api/analytics/app/:appId - Get analytics for specific app
router.get('/app/:appId', async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    const { appId } = req.params;
    const { period = '30d' } = req.query;

    // Calculate date range
    let dateCondition = '';
    const now = new Date();
    
    switch (period) {
      case '7d':
        dateCondition = `date >= CURRENT_DATE - INTERVAL '7 days'`;
        break;
      case '30d':
        dateCondition = `date >= CURRENT_DATE - INTERVAL '30 days'`;
        break;
      case '90d':
        dateCondition = `date >= CURRENT_DATE - INTERVAL '90 days'`;
        break;
      case '1y':
        dateCondition = `date >= CURRENT_DATE - INTERVAL '1 year'`;
        break;
      default:
        dateCondition = `date >= CURRENT_DATE - INTERVAL '30 days'`;
    }

    // Get app info
    const appSql = `
      SELECT a.title, a.developer_id, u.name as developer_name
      FROM apps a
      LEFT JOIN users u ON a.developer_id = u.id
      WHERE a.id = $1
    `;

    const appResult = await query(appSql, [appId]);
    
    if (appResult.rows.length === 0) {
      return res.status(404).json({ success: false, error: 'App not found' });
    }

    const app = appResult.rows[0];

    // Check if user has access (developer can see their own app analytics, admin can see all)
    if (req.user?.role === 'developer' && app.developer_id !== req.user.id) {
      return res.status(403).json({ success: false, error: 'Access denied' });
    }

    // Get analytics data
    const analyticsSql = `
      SELECT 
        date,
        views,
        downloads,
        reviews,
        average_rating,
        bounce_rate,
        session_duration,
        device_types,
        geographic_data
      FROM app_analytics
      WHERE app_id = $1 AND ${dateCondition}
      ORDER BY date ASC
    `;

    const analyticsResult = await query(analyticsSql, [appId]);

    // Calculate summary statistics
    const summarySql = `
      SELECT 
        SUM(views) as total_views,
        SUM(downloads) as total_downloads,
        SUM(reviews) as total_reviews,
        AVG(average_rating) as avg_rating,
        AVG(bounce_rate) as avg_bounce_rate,
        AVG(session_duration) as avg_session_duration
      FROM app_analytics
      WHERE app_id = $1 AND ${dateCondition}
    `;

    const summaryResult = await query(summarySql, [appId]);
    const summary = summaryResult.rows[0];

    // Get top performing days
    const topDaysSql = `
      SELECT date, views, downloads, (downloads::float / NULLIF(views, 0)) * 100 as conversion_rate
      FROM app_analytics
      WHERE app_id = $1 AND ${dateCondition}
      ORDER BY downloads DESC
      LIMIT 5
    `;

    const topDaysResult = await query(topDaysSql, [appId]);

    const response: APIResponse<{
      app: any;
      analytics: AppAnalytics[];
      summary: {
        totalViews: number;
        totalDownloads: number;
        totalReviews: number;
        avgRating: number;
        avgBounceRate: number;
        avgSessionDuration: number;
        conversionRate: number;
      };
      topDays: any[];
    }> = {
      success: true,
      data: {
        app,
        analytics: analyticsResult.rows,
        summary: {
          totalViews: parseInt(summary.total_views) || 0,
          totalDownloads: parseInt(summary.total_downloads) || 0,
          totalReviews: parseInt(summary.total_reviews) || 0,
          avgRating: parseFloat(summary.avg_rating) || 0,
          avgBounceRate: parseFloat(summary.avg_bounce_rate) || 0,
          avgSessionDuration: parseInt(summary.avg_session_duration) || 0,
          conversionRate: summary.total_views > 0 
            ? (parseInt(summary.total_downloads) / parseInt(summary.total_views)) * 100 
            : 0
        },
        topDays: topDaysResult.rows
      }
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching app analytics:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// GET /api/analytics/developer - Get analytics for developer's apps
router.get('/developer/summary', AdminMiddleware, async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    if (!req.user) {
      return res.status(401).json({ success: false, error: 'Authentication required' });
    }

    const { period = '30d' } = req.query;

    // Calculate date range
    let dateCondition = '';
    const now = new Date();
    
    switch (period) {
      case '7d':
        dateCondition = `date >= CURRENT_DATE - INTERVAL '7 days'`;
        break;
      case '30d':
        dateCondition = `date >= CURRENT_DATE - INTERVAL '30 days'`;
        break;
      case '90d':
        dateCondition = `date >= CURRENT_DATE - INTERVAL '90 days'`;
        break;
      default:
        dateCondition = `date >= CURRENT_DATE - INTERVAL '30 days'`;
    }

    // Get all apps by developer
    const appsSql = `
      SELECT id, title FROM apps WHERE developer_id = $1
    `;

    const appsResult = await query(appsSql, [req.user.id]);
    const appIds = appsResult.rows.map(app => app.id);

    if (appIds.length === 0) {
      return res.json({
        success: true,
        data: {
          summary: {
            totalApps: 0,
            totalViews: 0,
            totalDownloads: 0,
            totalReviews: 0,
            avgRating: 0,
            conversionRate: 0
          },
          appPerformance: [],
          trends: []
        }
      });
    }

    // Get overall analytics
    const overallSql = `
      SELECT 
        COUNT(DISTINCT aa.app_id) as total_apps,
        SUM(aa.views) as total_views,
        SUM(aa.downloads) as total_downloads,
        SUM(aa.reviews) as total_reviews,
        AVG(aa.average_rating) as avg_rating,
        AVG(aa.bounce_rate) as avg_bounce_rate
      FROM app_analytics aa
      WHERE aa.app_id = ANY($1) AND ${dateCondition}
    `;

    const overallResult = await query(overallSql, [appIds]);

    // Get per-app performance
    const appPerformanceSql = `
      SELECT 
        a.title,
        a.rating,
        a.review_count,
        a.download_count,
        SUM(aa.views) as total_views,
        SUM(aa.downloads) as total_downloads,
        SUM(aa.reviews) as total_reviews,
        (SUM(aa.downloads)::float / NULLIF(SUM(aa.views), 0)) * 100 as conversion_rate
      FROM apps a
      LEFT JOIN app_analytics aa ON a.id = aa.app_id
      WHERE a.id = ANY($1) AND ${dateCondition.replace('aa.date', 'aa.date')}
      GROUP BY a.id, a.title, a.rating, a.review_count, a.download_count
      ORDER BY total_downloads DESC
    `;

    const appPerformanceResult = await query(appPerformanceSql, [appIds]);

    // Get trend data (daily totals)
    const trendsSql = `
      SELECT 
        aa.date,
        SUM(aa.views) as views,
        SUM(aa.downloads) as downloads,
        SUM(aa.reviews) as reviews
      FROM app_analytics aa
      WHERE aa.app_id = ANY($1) AND ${dateCondition}
      GROUP BY aa.date
      ORDER BY aa.date ASC
    `;

    const trendsResult = await query(trendsSql, [appIds]);

    const summary = overallResult.rows[0];

    const response: APIResponse<{
      summary: {
        totalApps: number;
        totalViews: number;
        totalDownloads: number;
        totalReviews: number;
        avgRating: number;
        avgBounceRate: number;
        conversionRate: number;
      };
      appPerformance: any[];
      trends: any[];
    }> = {
      success: true,
      data: {
        summary: {
          totalApps: parseInt(summary.total_apps) || 0,
          totalViews: parseInt(summary.total_views) || 0,
          totalDownloads: parseInt(summary.total_downloads) || 0,
          totalReviews: parseInt(summary.total_reviews) || 0,
          avgRating: parseFloat(summary.avg_rating) || 0,
          avgBounceRate: parseFloat(summary.avg_bounce_rate) || 0,
          conversionRate: summary.total_views > 0 
            ? (parseInt(summary.total_downloads) / parseInt(summary.total_views)) * 100 
            : 0
        },
        appPerformance: appPerformanceResult.rows,
        trends: trendsResult.rows
      }
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching developer analytics:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// GET /api/analytics/platform - Get platform-wide analytics (admin only)
router.get('/platform/overview', AdminMiddleware, async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    const { period = '30d' } = req.query;

    // Calculate date range
    let dateCondition = '';
    
    switch (period) {
      case '7d':
        dateCondition = `date >= CURRENT_DATE - INTERVAL '7 days'`;
        break;
      case '30d':
        dateCondition = `date >= CURRENT_DATE - INTERVAL '30 days'`;
        break;
      case '90d':
        dateCondition = `date >= CURRENT_DATE - INTERVAL '90 days'`;
        break;
      default:
        dateCondition = `date >= CURRENT_DATE - INTERVAL '30 days'`;
    }

    // Overall platform stats
    const platformStatsSql = `
      SELECT 
        (SELECT COUNT(*) FROM apps WHERE status = 'approved') as total_approved_apps,
        (SELECT COUNT(*) FROM apps WHERE status = 'pending') as pending_apps,
        (SELECT COUNT(*) FROM users WHERE role = 'developer') as total_developers,
        (SELECT COUNT(*) FROM users WHERE role = 'educator') as total_educators,
        (SELECT COUNT(*) FROM reviews) as total_reviews,
        (SELECT AVG(rating) FROM apps WHERE status = 'approved' AND rating > 0) as avg_platform_rating,
        SUM(aa.views) as total_views,
        SUM(aa.downloads) as total_downloads,
        COUNT(DISTINCT aa.app_id) as apps_with_analytics
      FROM app_analytics aa
      WHERE ${dateCondition}
    `;

    const platformStatsResult = await query(platformStatsSql);
    const platformStats = platformStatsResult.rows[0];

    // Top performing categories
    const categoryStatsSql = `
      SELECT 
        c.name as category_name,
        COUNT(a.id) as app_count,
        SUM(a.download_count) as total_downloads,
        AVG(a.rating) as avg_rating
      FROM categories c
      LEFT JOIN apps a ON c.id = a.category_id AND a.status = 'approved'
      GROUP BY c.id, c.name
      ORDER BY total_downloads DESC NULLS LAST
      LIMIT 10
    `;

    const categoryStatsResult = await query(categoryStatsSql);

    // Recent activity trends
    const trendsSql = `
      SELECT 
        aa.date,
        SUM(aa.views) as views,
        SUM(aa.downloads) as downloads,
        SUM(aa.reviews) as reviews,
        COUNT(DISTINCT aa.app_id) as active_apps
      FROM app_analytics aa
      WHERE ${dateCondition}
      GROUP BY aa.date
      ORDER BY aa.date ASC
    `;

    const trendsResult = await query(trendsSql);

    // Device and geographic breakdown
    const deviceBreakdownSql = `
      SELECT 
        jsonb_array_elements_text(
          jsonb_path_query_array(
            jsonb_agg(DISTINCT aa.device_types),
            '$.[*].device'
          )
        ) as device_type,
        COUNT(*) as usage_count
      FROM app_analytics aa
      WHERE ${dateCondition}
      GROUP BY device_type
      ORDER BY usage_count DESC
    `;

    const deviceBreakdownResult = await query(deviceBreakdownSql);

    const response: APIResponse<{
      platformStats: any;
      topCategories: any[];
      trends: any[];
      deviceBreakdown: any[];
    }> = {
      success: true,
      data: {
        platformStats: {
          totalApprovedApps: parseInt(platformStats.total_approved_apps) || 0,
          pendingApps: parseInt(platformStats.pending_apps) || 0,
          totalDevelopers: parseInt(platformStats.total_developers) || 0,
          totalEducators: parseInt(platformStats.total_educators) || 0,
          totalReviews: parseInt(platformStats.total_reviews) || 0,
          avgPlatformRating: parseFloat(platformStats.avg_platform_rating) || 0,
          totalViews: parseInt(platformStats.total_views) || 0,
          totalDownloads: parseInt(platformStats.total_downloads) || 0,
          appsWithAnalytics: parseInt(platformStats.apps_with_analytics) || 0
        },
        topCategories: categoryStatsResult.rows,
        trends: trendsResult.rows,
        deviceBreakdown: deviceBreakdownResult.rows
      }
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching platform analytics:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

// GET /api/analytics/educator/:educatorId - Get educator-specific analytics
router.get('/educator/:educatorId', async (req: AuthenticatedRequest, res: express.Response) => {
  try {
    const { educatorId } = req.params;

    // Check access permissions
    if (req.user?.role === 'educator' && req.user.id !== educatorId) {
      return res.status(403).json({ success: false, error: 'Access denied' });
    }

    // Get educator's downloaded apps
    const downloadedAppsSql = `
      SELECT 
        a.*,
        c.name as category_name,
        u.name as developer_name,
        ud.downloaded_at
      FROM user_downloads ud
      JOIN apps a ON ud.app_id = a.id
      LEFT JOIN categories c ON a.category_id = c.id
      LEFT JOIN users u ON a.developer_id = u.id
      WHERE ud.user_id = $1 AND a.status = 'approved'
      ORDER BY ud.downloaded_at DESC
    `;

    const downloadedAppsResult = await query(downloadedAppsSql, [educatorId]);

    // Get analytics for downloaded apps
    const appIds = downloadedAppsResult.rows.map(app => app.id);
    let analytics = [];
    
    if (appIds.length > 0) {
      const analyticsSql = `
        SELECT 
          a.title,
          a.category_id,
          c.name as category_name,
          aa.date,
          aa.views,
          aa.downloads,
          aa.average_rating,
          aa.bounce_rate
        FROM apps a
        LEFT JOIN app_analytics aa ON a.id = aa.app_id
        LEFT JOIN categories c ON a.category_id = c.id
        WHERE a.id = ANY($1) AND aa.date >= CURRENT_DATE - INTERVAL '30 days'
        ORDER BY aa.date DESC
      `;

      const analyticsResult = await query(analyticsSql, [appIds]);
      analytics = analyticsResult.rows;
    }

    // Get usage statistics by category
    const categoryUsageSql = `
      SELECT 
        c.name as category_name,
        COUNT(ud.app_id) as downloaded_apps,
        AVG(a.rating) as avg_rating,
        AVG(a.download_count) as avg_downloads
      FROM user_downloads ud
      JOIN apps a ON ud.app_id = a.id
      JOIN categories c ON a.category_id = c.id
      WHERE ud.user_id = $1 AND a.status = 'approved'
      GROUP BY c.id, c.name
      ORDER BY downloaded_apps DESC
    `;

    const categoryUsageResult = await query(categoryUsageSql, [educatorId]);

    const response: APIResponse<{
      downloadedApps: any[];
      analytics: any[];
      categoryUsage: any[];
    }> = {
      success: true,
      data: {
        downloadedApps: downloadedAppsResult.rows,
        analytics,
        categoryUsage: categoryUsageResult.rows
      }
    };

    res.json(response);
  } catch (error) {
    console.error('Error fetching educator analytics:', error);
    res.status(500).json({ success: false, error: 'Internal server error' });
  }
});

export { router as AnalyticsRouter };