const express = require('express');
const Paper = require('../models/Paper');
const Review = require('../models/Review');
const User = require('../models/User');
const Citation = require('../models/Citation');
const Conference = require('../models/Conference');
const { auth, requireRole } = require('../middleware/auth');

const router = express.Router();

// Get dashboard overview statistics
router.get('/dashboard', auth, async (req, res) => {
  try {
    const userId = req.user.userId;
    const isEditor = req.user.role === 'editor' || req.user.role === 'admin';

    // User-specific statistics
    const userStats = await Promise.all([
      // User papers
      Paper.countDocuments({ createdBy: userId }),
      Paper.countDocuments({ createdBy: userId, status: 'published' }),
      Paper.countDocuments({ createdBy: userId, status: 'accepted' }),
      Paper.countDocuments({ createdBy: userId, status: 'under_review' }),
      
      // User reviews
      Review.countDocuments({ reviewer: userId, status: 'completed' }),
      Review.countDocuments({ reviewer: userId, status: { $in: ['assigned', 'in_progress'] } }),
      
      // Recent activity
      Paper.countDocuments({ 
        createdBy: userId, 
        createdAt: { $gte: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000) }
      })
    ]);

    const [
      totalPapers,
      publishedPapers,
      acceptedPapers,
      reviewPendingPapers,
      completedReviews,
      activeReviews,
      recentPapers
    ] = userStats;

    const dashboardData = {
      papers: {
        total: totalPapers,
        published: publishedPapers,
        accepted: acceptedPapers,
        underReview: reviewPendingPapers,
        recent: recentPapers
      },
      reviews: {
        completed: completedReviews,
        active: activeReviews,
        pending: Math.max(activeReviews - 2, 0) // Rough calculation
      },
      metrics: {
        acceptanceRate: totalPapers > 0 ? Math.round((acceptedPapers / totalPapers) * 100) : 0,
        averageRating: 0, // Will be populated below
        hIndex: 0 // Will be calculated below
      }
    };

    // Get average review rating for user
    const reviewStats = await Review.getReviewerStats(userId);
    if (reviewStats.length > 0 && reviewStats[0].averageRating) {
      dashboardData.metrics.averageRating = Math.round(reviewStats[0].averageRating * 100) / 100;
    }

    // Get user's papers with citations to calculate h-index
    const userPapers = await Paper.find({ 
      createdBy: userId, 
      'metrics.citations': { $gt: 0 } 
    }).sort({ 'metrics.citations': -1 });

    let hIndex = 0;
    for (let i = 0; i < userPapers.length; i++) {
      if (userPapers[i].metrics.citations >= i + 1) {
        hIndex = i + 1;
      } else {
        break;
      }
    }
    dashboardData.metrics.hIndex = hIndex;

    // Editor/Admin specific statistics
    if (isEditor) {
      const editorStats = await Promise.all([
        // Overall platform statistics
        Paper.countDocuments({}),
        Paper.countDocuments({ status: 'published' }),
        Paper.countDocuments({ status: 'under_review' }),
        Paper.countDocuments({ status: 'submitted' }),
        
        // Review statistics
        Review.countDocuments({ status: 'completed' }),
        Review.countDocuments({ status: { $in: ['assigned', 'in_progress'] } }),
        Review.countDocuments({ status: 'late' }),
        
        // User statistics
        User.countDocuments({}),
        User.countDocuments({ role: 'reviewer' }),
        User.countDocuments({ 'reviewPreferences.willingToReview': true }),
        
        // Conference statistics
        Conference.countDocuments({ status: { $in: ['cfp_announced', 'submissions_open'] } }),
        Conference.countDocuments({ status: 'completed' }),
        
        // Citation statistics
        Citation.countDocuments({})
      ]);

      const [
        totalPapersPlatform,
        publishedPlatform,
        underReviewPlatform,
        submittedPlatform,
        completedReviewsPlatform,
        activeReviewsPlatform,
        overdueReviews,
        totalUsers,
        totalReviewers,
        activeReviewers,
        upcomingConferences,
        completedConferences,
        totalCitations
      ] = editorStats;

      dashboardData.platform = {
        papers: {
          total: totalPapersPlatform,
          published: publishedPlatform,
          underReview: underReviewPlatform,
          submitted: submittedPlatform
        },
        reviews: {
          completed: completedReviewsPlatform,
          active: activeReviewsPlatform,
          overdue: overdueReviews
        },
        users: {
          total: totalUsers,
          reviewers: totalReviewers,
          activeReviewers
        },
        conferences: {
          upcoming: upcomingConferences,
          completed: completedConferences
        },
        citations: totalCitations
      };
    }

    res.json({
      dashboard: dashboardData,
      lastUpdated: new Date()
    });

  } catch (error) {
    console.error('Dashboard analytics error:', error);
    res.status(500).json({ 
      error: 'Failed to fetch dashboard data',
      message: 'An error occurred while fetching analytics data'
    });
  }
});

// Get paper analytics
router.get('/papers', auth, async (req, res) => {
  try {
    const { timeRange = '12months', paperId } = req.query;
    const userId = req.user.userId;
    const isEditor = req.user.role === 'editor' || req.user.role === 'admin';

    // Calculate date range
    const now = new Date();
    let startDate;
    
    switch (timeRange) {
      case '1month':
        startDate = new Date(now.getFullYear(), now.getMonth() - 1, now.getDate());
        break;
      case '3months':
        startDate = new Date(now.getFullYear(), now.getMonth() - 3, now.getDate());
        break;
      case '6months':
        startDate = new Date(now.getFullYear(), now.getMonth() - 6, now.getDate());
        break;
      case '1year':
      default:
        startDate = new Date(now.getFullYear() - 1, now.getMonth(), now.getDate());
        break;
    }

    let query = {
      createdAt: { $gte: startDate }
    };

    // Filter by paper if specified
    if (paperId) {
      query._id = paperId;
    }

    // User can only see their own papers unless they're an editor
    if (!isEditor) {
      query.createdBy = userId;
    }

    const papers = await Paper.find(query)
      .populate('createdBy', 'firstName lastName affiliation')
      .sort({ createdAt: -1 });

    // Aggregate analytics
    const analytics = {
      overview: {
        totalPapers: papers.length,
        published: papers.filter(p => p.status === 'published').length,
        underReview: papers.filter(p => p.status === 'under_review').length,
        rejected: papers.filter(p => p.status === 'rejected').length,
        totalViews: papers.reduce((sum, p) => sum + p.metrics.views, 0),
        totalDownloads: papers.reduce((sum, p) => sum + p.metrics.downloads, 0),
        totalCitations: papers.reduce((sum, p) => sum + p.metrics.citations, 0)
      },
      submissions: {
        byMonth: [],
        byResearchArea: {},
        byStatus: {},
        acceptanceRate: 0
      },
      engagement: {
        viewsOverTime: [],
        downloadsOverTime: [],
        topPerformingPapers: []
      }
    };

    // Group submissions by month
    const submissionsByMonth = {};
    papers.forEach(paper => {
      const monthKey = paper.createdAt.toISOString().substring(0, 7); // YYYY-MM
      if (!submissionsByMonth[monthKey]) {
        submissionsByMonth[monthKey] = 0;
      }
      submissionsByMonth[monthKey]++;
    });

    analytics.submissions.byMonth = Object.entries(submissionsByMonth)
      .map(([month, count]) => ({ month, count }))
      .sort((a, b) => a.month.localeCompare(b.month));

    // Group by research area
    const areaStats = {};
    papers.forEach(paper => {
      const area = paper.researchArea;
      if (!areaStats[area]) {
        areaStats[area] = { total: 0, accepted: 0, published: 0 };
      }
      areaStats[area].total++;
      if (['accepted', 'published'].includes(paper.status)) {
        areaStats[area].accepted++;
      }
      if (paper.status === 'published') {
        areaStats[area].published++;
      }
    });

    analytics.submissions.byResearchArea = areaStats;

    // Group by status
    const statusStats = {};
    papers.forEach(paper => {
      const status = paper.status;
      if (!statusStats[status]) {
        statusStats[status] = 0;
      }
      statusStats[status]++;
    });

    analytics.submissions.byStatus = statusStats;

    // Calculate acceptance rate
    const totalWithDecisions = papers.filter(p => 
      ['accepted', 'rejected', 'published'].includes(p.status)
    ).length;
    const accepted = papers.filter(p => 
      ['accepted', 'published'].includes(p.status)
    ).length;
    
    analytics.submissions.acceptanceRate = totalWithDecisions > 0 ? 
      Math.round((accepted / totalWithDecisions) * 100) : 0;

    // Top performing papers
    analytics.engagement.topPerformingPapers = papers
      .sort((a, b) => (b.metrics.views + b.metrics.downloads) - (a.metrics.views + a.metrics.downloads))
      .slice(0, 10)
      .map(paper => ({
        id: paper._id,
        title: paper.title,
        views: paper.metrics.views,
        downloads: paper.metrics.downloads,
        citations: paper.metrics.citations,
        status: paper.status,
        createdAt: paper.createdAt
      }));

    res.json({
      analytics,
      timeRange,
      dataPoints: papers.length
    });

  } catch (error) {
    console.error('Paper analytics error:', error);
    res.status(500).json({ 
      error: 'Failed to fetch paper analytics',
      message: 'An error occurred while fetching analytics data'
    });
  }
});

// Get review analytics
router.get('/reviews', auth, async (req, res) => {
  try {
    const { timeRange = '12months' } = req.query;
    const userId = req.user.userId;
    const isEditor = req.user.role === 'editor' || req.user.role === 'admin';

    // Calculate date range
    const now = new Date();
    let startDate;
    
    switch (timeRange) {
      case '1month':
        startDate = new Date(now.getFullYear(), now.getMonth() - 1, now.getDate());
        break;
      case '3months':
        startDate = new Date(now.getFullYear(), now.getMonth() - 3, now.getDate());
        break;
      case '6months':
        startDate = new Date(now.getFullYear(), now.getMonth() - 6, now.getDate());
        break;
      case '1year':
      default:
        startDate = new Date(now.getFullYear() - 1, now.getMonth(), now.getDate());
        break;
    }

    let query = {
      assignmentDate: { $gte: startDate }
    };

    // User can only see their own reviews unless they're an editor
    if (!isEditor) {
      query.reviewer = userId;
    }

    const reviews = await Review.find(query)
      .populate('paper', 'title researchArea')
      .populate('reviewer', 'firstName lastName affiliation')
      .sort({ assignmentDate: -1 });

    // Aggregate analytics
    const analytics = {
      overview: {
        totalReviews: reviews.length,
        completed: reviews.filter(r => r.status === 'completed').length,
        active: reviews.filter(r => r.status === 'in_progress').length,
        overdue: reviews.filter(r => r.status === 'late').length,
        averageCompletionTime: 0,
        averageRating: 0
      },
      performance: {
        byMonth: [],
        byResearchArea: {},
        ratingDistribution: {
          '1': 0, '2': 0, '3': 0, '4': 0, '5': 0
        },
        recommendationDistribution: {
          accept: 0,
          minor_revision: 0,
          major_revision: 0,
          reject: 0
        }
      },
      quality: {
        timeliness: 0,
        constructiveness: 0,
        expertise: 0
      }
    };

    // Calculate average completion time
    const completedReviews = reviews.filter(r => r.completionTimeInDays !== null);
    if (completedReviews.length > 0) {
      analytics.overview.averageCompletionTime = Math.round(
        completedReviews.reduce((sum, r) => sum + r.completionTimeInDays, 0) / completedReviews.length
      );
    }

    // Calculate average rating
    const reviewsWithRatings = reviews.filter(r => r.averageRating !== null);
    if (reviewsWithRatings.length > 0) {
      analytics.overview.averageRating = Math.round(
        reviewsWithRatings.reduce((sum, r) => sum + r.averageRating, 0) / reviewsWithRatings.length * 100
      ) / 100;
    }

    // Group reviews by month
    const reviewsByMonth = {};
    reviews.forEach(review => {
      const monthKey = review.assignmentDate.toISOString().substring(0, 7);
      if (!reviewsByMonth[monthKey]) {
        reviewsByMonth[monthKey] = 0;
      }
      reviewsByMonth[monthKey]++;
    });

    analytics.performance.byMonth = Object.entries(reviewsByMonth)
      .map(([month, count]) => ({ month, count }))
      .sort((a, b) => a.month.localeCompare(b.month));

    // Group by research area
    const areaStats = {};
    reviews.forEach(review => {
      const area = review.paper?.researchArea || 'Unknown';
      if (!areaStats[area]) {
        areaStats[area] = { total: 0, completed: 0, averageRating: 0 };
      }
      areaStats[area].total++;
      if (review.status === 'completed') {
        areaStats[area].completed++;
      }
    });

    analytics.performance.byResearchArea = areaStats;

    // Rating distribution
    reviews.forEach(review => {
      if (review.rating?.overall?.score) {
        const score = review.rating.overall.score.toString();
        if (analytics.performance.ratingDistribution.hasOwnProperty(score)) {
          analytics.performance.ratingDistribution[score]++;
        }
      }
    });

    // Recommendation distribution
    reviews.forEach(review => {
      if (review.recommendation?.decision) {
        const decision = review.recommendation.decision;
        if (analytics.performance.recommendationDistribution.hasOwnProperty(decision)) {
          analytics.performance.recommendationDistribution[decision]++;
        }
      }
    });

    res.json({
      analytics,
      timeRange,
      dataPoints: reviews.length
    });

  } catch (error) {
    console.error('Review analytics error:', error);
    res.status(500).json({ 
      error: 'Failed to fetch review analytics',
      message: 'An error occurred while fetching analytics data'
    });
  }
});

// Get platform-wide analytics (Editor/Admin only)
router.get('/platform', auth, requireRole(['editor', 'admin']), async (req, res) => {
  try {
    const { timeRange = '12months' } = req.query;

    // Calculate date range
    const now = new Date();
    let startDate;
    
    switch (timeRange) {
      case '1month':
        startDate = new Date(now.getFullYear(), now.getMonth() - 1, now.getDate());
        break;
      case '3months':
        startDate = new Date(now.getFullYear(), now.getMonth() - 3, now.getDate());
        break;
      case '6months':
        startDate = new Date(now.getFullYear(), now.getMonth() - 6, now.getDate());
        break;
      case '1year':
      default:
        startDate = new Date(now.getFullYear() - 1, now.getMonth(), now.getDate());
        break;
    }

    // Platform-wide statistics
    const platformStats = await Promise.all([
      // User growth
      User.countDocuments({ createdAt: { $gte: startDate } }),
      User.countDocuments({ role: 'reviewer', createdAt: { $gte: startDate } }),
      
      // Paper submissions
      Paper.countDocuments({ createdAt: { $gte: startDate } }),
      Paper.countDocuments({ 
        status: 'published', 
        submissionDate: { $gte: startDate }
      }),
      
      // Review activity
      Review.countDocuments({ assignmentDate: { $gte: startDate } }),
      Review.findOverdue(),
      
      // Conference activity
      Conference.countDocuments({ createdAt: { $gte: startDate } }),
      
      // Citation activity
      Citation.countDocuments({ addedDate: { $gte: startDate } })
    ]);

    const [
      newUsers,
      newReviewers,
      newPapers,
      publishedPapers,
      newReviews,
      overdueReviews,
      newConferences,
      newCitations
    ] = platformStats;

    const analytics = {
      growth: {
        users: newUsers,
        reviewers: newReviewers,
        papers: newPapers,
        conferences: newConferences
      },
      productivity: {
        publications: publishedPapers,
        reviews: newReviews,
        citations: newCitations
      },
      challenges: {
        overdueReviews: overdueReviews.length,
        completionRate: newReviews > 0 ? 
          Math.round(((newReviews - overdueReviews.length) / newReviews) * 100) : 100
      },
      trends: {
        submissionsOverTime: [],
        acceptanceRates: [],
        reviewerParticipation: []
      }
    };

    // Get monthly trends
    const monthlyStats = await Paper.aggregate([
      {
        $match: {
          createdAt: { $gte: startDate }
        }
      },
      {
        $group: {
          _id: {
            year: { $year: '$createdAt' },
            month: { $month: '$createdAt' }
          },
          submissions: { $sum: 1 },
          accepted: {
            $sum: {
              $cond: [
                { $in: ['$status', ['accepted', 'published']] },
                1,
                0
              ]
            }
          }
        }
      },
      {
        $sort: { '_id.year': 1, '_id.month': 1 }
      }
    ]);

    analytics.trends.submissionsOverTime = monthlyStats.map(stat => ({
      period: `${stat._id.year}-${stat._id.month.toString().padStart(2, '0')}`,
      submissions: stat.submissions,
      acceptances: stat.accepted,
      acceptanceRate: stat.submissions > 0 ? 
        Math.round((stat.accepted / stat.submissions) * 100) : 0
    }));

    res.json({
      analytics,
      timeRange,
      generatedAt: new Date()
    });

  } catch (error) {
    console.error('Platform analytics error:', error);
    res.status(500).json({ 
      error: 'Failed to fetch platform analytics',
      message: 'An error occurred while fetching analytics data'
    });
  }
});

// Get citation metrics
router.get('/citations', auth, async (req, res) => {
  try {
    const { timeRange = '12months' } = req.query;
    const userId = req.user.userId;
    const isEditor = req.user.role === 'editor' || req.user.role === 'admin';

    // Calculate date range
    const now = new Date();
    let startDate;
    
    switch (timeRange) {
      case '1month':
        startDate = new Date(now.getFullYear(), now.getMonth() - 1, now.getDate());
        break;
      case '3months':
        startDate = new Date(now.getFullYear(), now.getMonth() - 3, now.getDate());
        break;
      case '6months':
        startDate = new Date(now.getFullYear(), now.getMonth() - 6, now.getDate());
        break;
      case '1year':
      default:
        startDate = new Date(now.getFullYear() - 1, now.getMonth(), now.getDate());
        break;
    }

    let query = {
      addedDate: { $gte: startDate }
    };

    // User can only see their own citations unless they're an editor
    if (!isEditor) {
      query.addedBy = userId;
    }

    const citations = await Citation.find(query)
      .sort({ addedDate: -1 });

    // Aggregate analytics
    const analytics = {
      overview: {
        totalCitations: citations.length,
        verified: citations.filter(c => c.quality.isVerified).length,
        withDOI: citations.filter(c => c.identifiers.doi).length,
        openAccess: citations.filter(c => c.openAccess.isOpenAccess).length,
        totalMetrics: 0
      },
      distribution: {
        byType: {},
        byYear: {},
        byAuthor: {}
      },
      quality: {
        verifiedRate: 0,
        doiRate: 0,
        openAccessRate: 0,
        averageQualityScore: 0
      }
    };

    // Calculate totals
    analytics.overview.totalMetrics = citations.reduce(
      (sum, c) => sum + c.metrics.totalCitations, 0
    );

    // Group by type
    const typeStats = {};
    citations.forEach(citation => {
      const type = citation.type;
      if (!typeStats[type]) {
        typeStats[type] = 0;
      }
      typeStats[type]++;
    });

    analytics.distribution.byType = typeStats;

    // Group by year
    const yearStats = {};
    citations.forEach(citation => {
      const year = citation.publication.year;
      if (year) {
        if (!yearStats[year]) {
          yearStats[year] = 0;
        }
        yearStats[year]++;
      }
    });

    analytics.distribution.byYear = yearStats;

    // Calculate quality metrics
    analytics.quality.verifiedRate = citations.length > 0 ? 
      Math.round((analytics.overview.verified / citations.length) * 100) : 0;
    
    analytics.quality.doiRate = citations.length > 0 ? 
      Math.round((analytics.overview.withDOI / citations.length) * 100) : 0;
    
    analytics.quality.openAccessRate = citations.length > 0 ? 
      Math.round((analytics.overview.openAccess / citations.length) * 100) : 0;

    const citationsWithScores = citations.filter(c => c.quality.qualityScore > 0);
    if (citationsWithScores.length > 0) {
      analytics.quality.averageQualityScore = Math.round(
        citationsWithScores.reduce((sum, c) => sum + c.quality.qualityScore, 0) / 
        citationsWithScores.length * 100
      ) / 100;
    }

    res.json({
      analytics,
      timeRange,
      dataPoints: citations.length
    });

  } catch (error) {
    console.error('Citation analytics error:', error);
    res.status(500).json({ 
      error: 'Failed to fetch citation analytics',
      message: 'An error occurred while fetching analytics data'
    });
  }
});

// Export analytics data
router.get('/export', auth, requireRole(['editor', 'admin']), async (req, res) => {
  try {
    const { type = 'papers', format = 'json', timeRange = '12months' } = req.query;

    // This would generate CSV/Excel exports
    // For now, return JSON data
    res.json({
      message: 'Export functionality',
      type,
      format,
      timeRange,
      note: 'Full export functionality would be implemented here'
    });

  } catch (error) {
    console.error('Export analytics error:', error);
    res.status(500).json({ 
      error: 'Failed to export analytics',
      message: 'An error occurred while exporting analytics data'
    });
  }
});

module.exports = router;