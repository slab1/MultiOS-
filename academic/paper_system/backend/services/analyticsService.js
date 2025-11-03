const User = require('../models/User');
const Paper = require('../models/Paper');
const Review = require('../models/Review');
const Conference = require('../models/Conference');
const Citation = require('../models/Citation');

class AnalyticsService {
  // Get dashboard analytics
  async getDashboardAnalytics(userId = null, timeRange = '1year') {
    const dateFilter = this.getDateFilter(timeRange);
    
    try {
      const [
        userStats,
        paperStats,
        reviewStats,
        platformStats,
        trends
      ] = await Promise.all([
        this.getUserAnalytics(userId, dateFilter),
        this.getPaperAnalytics(dateFilter),
        this.getReviewAnalytics(dateFilter),
        this.getPlatformAnalytics(dateFilter),
        this.getTrendsAnalytics(dateFilter)
      ]);

      return {
        user: userStats,
        papers: paperStats,
        reviews: reviewStats,
        platform: platformStats,
        trends,
        timeRange,
        generatedAt: new Date()
      };
    } catch (error) {
      throw new Error(`Failed to get dashboard analytics: ${error.message}`);
    }
  }

  // Get user-specific analytics
  async getUserAnalytics(userId, dateFilter) {
    if (!userId) return null;

    try {
      const user = await User.findById(userId).populate('researchAreas');
      if (!user) return null;

      const [
        userPapers,
        userReviews,
        userCitations,
        collaborationMetrics
      ] = await Promise.all([
        Paper.find({ 'authors.user': userId, ...dateFilter }),
        Review.find({ reviewer: userId, ...dateFilter }),
        Citation.find({ 
          $or: [
            { 'authors': { $elemMatch: { name: { $in: [user.fullName] } } } },
            { 'citations.citingPaper': { $in: await this.getUserPaperIds(userId) } }
          ]
        }),
        this.getCollaborationMetrics(userId, dateFilter)
      ]);

      return {
        profile: this.formatUserProfile(user),
        publications: this.getPublicationMetrics(userPapers),
        reviews: this.getReviewMetrics(userReviews),
        citations: this.getCitationMetrics(userCitations),
        collaborations: collaborationMetrics,
        hIndex: this.calculateHIndex(userCitations),
        researchImpact: this.calculateResearchImpact(userPapers, userCitations)
      };
    } catch (error) {
      console.error('Error getting user analytics:', error);
      return null;
    }
  }

  // Get paper analytics
  async getPaperAnalytics(dateFilter) {
    try {
      const papers = await Paper.find(dateFilter).populate('authors citations');
      
      const analytics = {
        total: papers.length,
        byStatus: this.groupByStatus(papers),
        byResearchArea: this.groupByResearchArea(papers),
        byMethodology: this.groupByMethodology(papers),
        submissionTrends: this.getSubmissionTrends(papers, dateFilter),
        reviewMetrics: await this.getReviewMetricsForPapers(papers),
        citationMetrics: this.getCitationMetricsForPapers(papers),
        qualityMetrics: await this.getQualityMetrics(papers)
      };

      return analytics;
    } catch (error) {
      throw new Error(`Failed to get paper analytics: ${error.message}`);
    }
  }

  // Get review analytics
  async getReviewAnalytics(dateFilter) {
    try {
      const reviews = await Review.find(dateFilter).populate('paper reviewer');
      
      return {
        total: reviews.length,
        byStatus: this.groupReviewByStatus(reviews),
        byCycle: this.groupReviewByCycle(reviews),
        qualityMetrics: this.getReviewQualityMetrics(reviews),
        timeliness: this.getReviewTimelinessMetrics(reviews),
        reviewerPerformance: this.getReviewerPerformanceMetrics(reviews),
        decisionAnalytics: this.getDecisionAnalytics(reviews)
      };
    } catch (error) {
      throw new Error(`Failed to get review analytics: ${error.message}`);
    }
  }

  // Get platform analytics
  async getPlatformAnalytics(dateFilter) {
    try {
      const [
        userGrowth,
        paperGrowth,
        reviewActivity,
        citationActivity,
        conferenceMetrics
      ] = await Promise.all([
        this.getUserGrowth(dateFilter),
        this.getPaperGrowth(dateFilter),
        this.getReviewActivity(dateFilter),
        this.getCitationActivity(dateFilter),
        this.getConferenceMetrics(dateFilter)
      ]);

      return {
        users: userGrowth,
        papers: paperGrowth,
        reviews: reviewActivity,
        citations: citationActivity,
        conferences: conferenceMetrics,
        systemHealth: await this.getSystemHealthMetrics()
      };
    } catch (error) {
      throw new Error(`Failed to get platform analytics: ${error.message}`);
    }
  }

  // Get trends analytics
  async getTrendsAnalytics(dateFilter) {
    try {
      const [
        researchTrends,
        citationTrends,
        collaborationTrends,
        emergingAreas
      ] = await Promise.all([
        this.getResearchTrends(dateFilter),
        this.getCitationTrends(dateFilter),
        this.getCollaborationTrends(dateFilter),
        this.getEmergingResearchAreas(dateFilter)
      ]);

      return {
        research: researchTrends,
        citations: citationTrends,
        collaborations: collaborationTrends,
        emergingAreas: emergingAreas,
        predictions: await this.getResearchPredictions(dateFilter)
      };
    } catch (error) {
      throw new Error(`Failed to get trends analytics: ${error.message}`);
    }
  }

  // Helper methods
  getDateFilter(timeRange) {
    const now = new Date();
    let startDate;

    switch (timeRange) {
      case '1month':
        startDate = new Date(now.getFullYear(), now.getMonth(), 1);
        break;
      case '3months':
        startDate = new Date(now.getFullYear(), now.getMonth() - 3, 1);
        break;
      case '6months':
        startDate = new Date(now.getFullYear(), now.getMonth() - 6, 1);
        break;
      case '1year':
        startDate = new Date(now.getFullYear() - 1, now.getMonth(), 1);
        break;
      case '2years':
        startDate = new Date(now.getFullYear() - 2, now.getMonth(), 1);
        break;
      case '5years':
        startDate = new Date(now.getFullYear() - 5, now.getMonth(), 1);
        break;
      default:
        startDate = new Date(now.getFullYear() - 1, now.getMonth(), 1);
    }

    return { createdAt: { $gte: startDate } };
  }

  formatUserProfile(user) {
    return {
      id: user._id,
      name: user.fullName,
      email: user.email,
      affiliation: user.affiliation,
      researchAreas: user.researchAreas,
      role: user.role,
      memberSince: user.createdAt,
      lastActive: user.lastLogin,
      isActive: user.isActive
    };
  }

  getPublicationMetrics(papers) {
    const metrics = {
      total: papers.length,
      byStatus: this.groupByStatus(papers),
      acceptanceRate: 0,
      averageCitations: 0,
      totalViews: papers.reduce((sum, p) => sum + (p.metrics?.views || 0), 0),
      totalDownloads: papers.reduce((sum, p) => sum + (p.metrics?.downloads || 0), 0)
    };

    const accepted = papers.filter(p => p.status === 'accepted').length;
    metrics.acceptanceRate = papers.length > 0 ? (accepted / papers.length * 100) : 0;

    const totalCitations = papers.reduce((sum, p) => sum + (p.citations?.length || 0), 0);
    metrics.averageCitations = papers.length > 0 ? (totalCitations / papers.length) : 0;

    return metrics;
  }

  getReviewMetrics(reviews) {
    return {
      total: reviews.length,
      completed: reviews.filter(r => r.status === 'completed').length,
      pending: reviews.filter(r => r.status === 'assigned').length,
      overdue: reviews.filter(r => r.status === 'late').length,
      averageRating: this.calculateAverageRating(reviews),
      averageCompletionTime: this.calculateAverageCompletionTime(reviews),
      qualityScore: this.calculateReviewerQualityScore(reviews)
    };
  }

  getCitationMetrics(citations) {
    return {
      total: citations.length,
      selfCitations: citations.filter(c => c.type === 'self').length,
      averageImpactFactor: this.calculateAverageImpactFactor(citations),
      topCitedPapers: citations
        .sort((a, b) => (b.totalCitations || 0) - (a.totalCitations || 0))
        .slice(0, 10)
        .map(c => ({
          id: c._id,
          title: c.title,
          citations: c.totalCitations || 0,
          year: c.year
        }))
    };
  }

  async getCollaborationMetrics(userId, dateFilter) {
    try {
      const papers = await Paper.find({ 
        'authors.user': userId, 
        ...dateFilter,
        'authors.1': { $exists: true }
      });

      const collaborationNetwork = new Map();
      
      for (const paper of papers) {
        const collaborators = paper.authors
          .filter(author => author.user?.toString() !== userId)
          .map(author => author.user?.toString());
        
        collaborators.forEach(collaboratorId => {
          if (collaboratorId) {
            const current = collaborationNetwork.get(collaboratorId) || { papers: 0, collaborations: 0 };
            current.papers += 1;
            collaborationNetwork.set(collaboratorId, current);
          }
        });
      }

      // Convert to array and get collaborator details
      const collaboratorData = await User.find({
        _id: { $in: Array.from(collaborationNetwork.keys()) }
      });

      return collaboratorData.map(user => ({
        collaborator: this.formatUserProfile(user),
        sharedPapers: collaborationNetwork.get(user._id.toString())?.papers || 0
      }));
    } catch (error) {
      console.error('Error getting collaboration metrics:', error);
      return [];
    }
  }

  calculateHIndex(citations) {
    const sortedCitations = citations
      .map(c => c.totalCitations || 0)
      .sort((a, b) => b - a);
    
    let hIndex = 0;
    for (let i = 0; i < sortedCitations.length; i++) {
      if (sortedCitations[i] >= i + 1) {
        hIndex = i + 1;
      } else {
        break;
      }
    }
    
    return hIndex;
  }

  calculateResearchImpact(papers, citations) {
    const metrics = {
      totalImpact: papers.reduce((sum, p) => sum + (p.citations?.length || 0), 0),
      averageImpact: papers.length > 0 ? 
        papers.reduce((sum, p) => sum + (p.citations?.length || 0), 0) / papers.length : 0,
      highlyCitedPapers: papers.filter(p => (p.citations?.length || 0) >= 10).length,
      breakthroughPapers: papers.filter(p => (p.citations?.length || 0) >= 50).length,
      citationVelocity: this.calculateCitationVelocity(citations)
    };

    return metrics;
  }

  groupByStatus(items) {
    return items.reduce((acc, item) => {
      const status = item.status || 'unknown';
      acc[status] = (acc[status] || 0) + 1;
      return acc;
    }, {});
  }

  groupByResearchArea(papers) {
    return papers.reduce((acc, paper) => {
      const area = paper.researchArea || 'Unknown';
      acc[area] = (acc[area] || 0) + 1;
      return acc;
    }, {});
  }

  groupByMethodology(papers) {
    return papers.reduce((acc, paper) => {
      const methodology = paper.methodology || 'Unknown';
      acc[methodology] = (acc[methodology] || 0) + 1;
      return acc;
    }, {});
  }

  groupReviewByStatus(reviews) {
    return reviews.reduce((acc, review) => {
      const status = review.status || 'unknown';
      acc[status] = (acc[status] || 0) + 1;
      return acc;
    }, {});
  }

  groupReviewByCycle(reviews) {
    return reviews.reduce((acc, review) => {
      const cycle = review.cycle || 1;
      acc[`cycle_${cycle}`] = (acc[`cycle_${cycle}`] || 0) + 1;
      return acc;
    }, {});
  }

  getSubmissionTrends(papers, dateFilter) {
    const monthlySubmissions = {};
    
    papers.forEach(paper => {
      const date = new Date(paper.createdAt);
      const monthKey = `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}`;
      monthlySubmissions[monthKey] = (monthlySubmissions[monthKey] || 0) + 1;
    });

    return Object.entries(monthlySubmissions)
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([month, count]) => ({ month, count }));
  }

  async getReviewMetricsForPapers(papers) {
    try {
      const paperIds = papers.map(p => p._id);
      const reviews = await Review.find({ paper: { $in: paperIds } });
      
      return {
        totalReviews: reviews.length,
        averageReviewsPerPaper: papers.length > 0 ? reviews.length / papers.length : 0,
        reviewCycleTimes: this.calculateReviewCycleTimes(reviews),
        acceptanceCorrelation: this.calculateAcceptanceCorrelation(papers, reviews)
      };
    } catch (error) {
      console.error('Error getting review metrics for papers:', error);
      return {};
    }
  }

  getCitationMetricsForPapers(papers) {
    const totalCitations = papers.reduce((sum, p) => sum + (p.citations?.length || 0), 0);
    const highlyCited = papers.filter(p => (p.citations?.length || 0) >= 10).length;
    
    return {
      totalCitations,
      averageCitationsPerPaper: papers.length > 0 ? totalCitations / papers.length : 0,
      highlyCitedPapers: highlyCited,
      citationDistribution: this.getCitationDistribution(papers)
    };
  }

  async getQualityMetrics(papers) {
    try {
      const metrics = {
        averageReviewScores: [],
        acceptanceRatesByArea: {},
        timeToDecision: [],
        revisionRates: {}
      };

      // Calculate quality metrics by research area
      const areaGroups = this.groupByResearchArea(papers);
      for (const [area, count] of Object.entries(areaGroups)) {
        const areaPapers = papers.filter(p => p.researchArea === area);
        const accepted = areaPapers.filter(p => p.status === 'accepted').length;
        metrics.acceptanceRatesByArea[area] = count > 0 ? (accepted / count * 100) : 0;
      }

      return metrics;
    } catch (error) {
      console.error('Error getting quality metrics:', error);
      return {};
    }
  }

  getReviewQualityMetrics(reviews) {
    const qualityMetrics = {
      averageOriginality: 0,
      averageSignificance: 0,
      averageTechnicalQuality: 0,
      averageClarity: 0,
      averageOverall: 0
    };

    const completedReviews = reviews.filter(r => r.status === 'completed');
    if (completedReviews.length === 0) return qualityMetrics;

    const totals = completedReviews.reduce((acc, review) => {
      const rating = review.rating || {};
      acc.originality += rating.originality?.score || 0;
      acc.significance += rating.significance?.score || 0;
      acc.technicalQuality += rating.technicalQuality?.score || 0;
      acc.clarity += rating.clarity?.score || 0;
      acc.overall += rating.overall?.score || 0;
      return acc;
    }, { originality: 0, significance: 0, technicalQuality: 0, clarity: 0, overall: 0 });

    const count = completedReviews.length;
    return {
      averageOriginality: totals.originality / count,
      averageSignificance: totals.significance / count,
      averageTechnicalQuality: totals.technicalQuality / count,
      averageClarity: totals.clarity / count,
      averageOverall: totals.overall / count
    };
  }

  getReviewTimelinessMetrics(reviews) {
    const metrics = {
      onTime: 0,
      late: 0,
      averageDelay: 0,
      timelyPercentage: 0
    };

    const completedReviews = reviews.filter(r => r.status === 'completed' && r.completedDate);
    if (completedReviews.length === 0) return metrics;

    let totalDelay = 0;
    let onTimeCount = 0;

    completedReviews.forEach(review => {
      const delay = review.completedDate.getTime() - review.dueDate.getTime();
      const delayDays = delay / (1000 * 60 * 60 * 24);
      
      if (delayDays <= 0) {
        onTimeCount++;
      } else {
        totalDelay += delayDays;
      }
    });

    metrics.onTime = onTimeCount;
    metrics.late = completedReviews.length - onTimeCount;
    metrics.averageDelay = totalDelay / (completedReviews.length - onTimeCount);
    metrics.timelyPercentage = (onTimeCount / completedReviews.length) * 100;

    return metrics;
  }

  async getUserGrowth(dateFilter) {
    try {
      const users = await User.find(dateFilter);
      const monthlyGrowth = {};
      
      users.forEach(user => {
        const date = new Date(user.createdAt);
        const monthKey = `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}`;
        monthlyGrowth[monthKey] = (monthlyGrowth[monthKey] || 0) + 1;
      });

      return {
        total: users.length,
        active: users.filter(u => u.isActive).length,
        byRole: this.getUsersByRole(users),
        growth: Object.entries(monthlyGrowth)
          .sort(([a], [b]) => a.localeCompare(b))
          .map(([month, count]) => ({ month, count }))
      };
    } catch (error) {
      console.error('Error getting user growth:', error);
      return { total: 0, active: 0, byRole: {}, growth: [] };
    }
  }

  async getPaperGrowth(dateFilter) {
    try {
      const papers = await Paper.find(dateFilter);
      const monthlyGrowth = {};
      
      papers.forEach(paper => {
        const date = new Date(paper.createdAt);
        const monthKey = `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}`;
        monthlyGrowth[monthKey] = (monthlyGrowth[monthKey] || 0) + 1;
      });

      return {
        total: papers.length,
        byStatus: this.groupByStatus(papers),
        byResearchArea: this.groupByResearchArea(papers),
        growth: Object.entries(monthlyGrowth)
          .sort(([a], [b]) => a.localeCompare(b))
          .map(([month, count]) => ({ month, count }))
      };
    } catch (error) {
      console.error('Error getting paper growth:', error);
      return { total: 0, byStatus: {}, byResearchArea: {}, growth: [] };
    }
  }

  async getReviewActivity(dateFilter) {
    try {
      const reviews = await Review.find(dateFilter);
      
      return {
        total: reviews.length,
        byStatus: this.groupReviewByStatus(reviews),
        byReviewer: this.getReviewActivityByReviewer(reviews),
        quality: this.getReviewQualityMetrics(reviews)
      };
    } catch (error) {
      console.error('Error getting review activity:', error);
      return { total: 0, byStatus: {}, byReviewer: {}, quality: {} };
    }
  }

  async getCitationActivity(dateFilter) {
    try {
      const citations = await Citation.find(dateFilter);
      
      return {
        total: citations.length,
        byType: this.getCitationsByType(citations),
        bySource: this.getCitationsBySource(citations),
        impactTrends: this.getCitationImpactTrends(citations)
      };
    } catch (error) {
      console.error('Error getting citation activity:', error);
      return { total: 0, byType: {}, bySource: {}, impactTrends: [] };
    }
  }

  async getConferenceMetrics(dateFilter) {
    try {
      const conferences = await Conference.find(dateFilter);
      
      return {
        total: conferences.length,
        byStatus: this.getConferencesByStatus(conferences),
        upcoming: conferences.filter(c => new Date(c.startDate) > new Date()).length,
        submissionDeadlines: this.getUpcomingSubmissionDeadlines(conferences)
      };
    } catch (error) {
      console.error('Error getting conference metrics:', error);
      return { total: 0, byStatus: {}, upcoming: 0, submissionDeadlines: [] };
    }
  }

  async getSystemHealthMetrics() {
    try {
      const [
        totalUsers,
        totalPapers,
        totalReviews,
        systemUptime,
        activeSessions
      ] = await Promise.all([
        User.countDocuments(),
        Paper.countDocuments(),
        Review.countDocuments({ status: 'completed' }),
        this.getSystemUptime(),
        this.getActiveSessions()
      ]);

      return {
        totalUsers,
        totalPapers,
        totalReviews,
        systemUptime,
        activeSessions,
        lastUpdated: new Date()
      };
    } catch (error) {
      console.error('Error getting system health metrics:', error);
      return {};
    }
  }

  // Utility methods
  calculateAverageRating(reviews) {
    const completedReviews = reviews.filter(r => r.status === 'completed' && r.rating?.overall?.score);
    if (completedReviews.length === 0) return 0;
    
    const total = completedReviews.reduce((sum, review) => sum + review.rating.overall.score, 0);
    return total / completedReviews.length;
  }

  calculateAverageCompletionTime(reviews) {
    const completedReviews = reviews.filter(r => r.status === 'completed' && r.assignedDate && r.completedDate);
    if (completedReviews.length === 0) return 0;
    
    const totalDays = completedReviews.reduce((sum, review) => {
      const days = (review.completedDate - review.assignedDate) / (1000 * 60 * 60 * 24);
      return sum + days;
    }, 0);
    
    return totalDays / completedReviews.length;
  }

  calculateReviewerQualityScore(reviews) {
    const completedReviews = reviews.filter(r => r.status === 'completed');
    if (completedReviews.length === 0) return 0;
    
    const timelyCompletion = completedReviews.filter(r => 
      r.completedDate <= r.dueDate
    ).length;
    
    const qualityScore = completedReviews.filter(r => 
      r.rating?.overall?.score >= 4
    ).length;
    
    return (timelyCompletion + qualityScore) / (completedReviews.length * 2) * 5;
  }

  async getUserPaperIds(userId) {
    const papers = await Paper.find({ 'authors.user': userId }, '_id');
    return papers.map(p => p._id);
  }

  getUsersByRole(users) {
    return users.reduce((acc, user) => {
      const roles = Array.isArray(user.role) ? user.role : [user.role];
      roles.forEach(role => {
        acc[role] = (acc[role] || 0) + 1;
      });
      return acc;
    }, {});
  }

  calculateCitationVelocity(citations) {
    const currentYear = new Date().getFullYear();
    const recentCitations = citations.filter(c => c.year >= currentYear - 2);
    
    const yearlyVelocity = {};
    recentCitations.forEach(citation => {
      const year = citation.year;
      yearlyVelocity[year] = (yearlyVelocity[year] || 0) + 1;
    });
    
    return yearlyVelocity;
  }

  calculateReviewCycleTimes(reviews) {
    const cycles = reviews.reduce((acc, review) => {
      const cycle = review.cycle || 1;
      if (!acc[cycle]) acc[cycle] = [];
      
      if (review.completedDate && review.assignedDate) {
        const days = (review.completedDate - review.assignedDate) / (1000 * 60 * 60 * 24);
        acc[cycle].push(days);
      }
      
      return acc;
    }, {});
    
    return Object.entries(cycles).map(([cycle, times]) => ({
      cycle: parseInt(cycle),
      averageDays: times.reduce((sum, time) => sum + time, 0) / times.length
    }));
  }

  calculateAcceptanceCorrelation(papers, reviews) {
    // Calculate correlation between review scores and acceptance
    const paperReviewMap = new Map();
    
    reviews.forEach(review => {
      const paperId = review.paper.toString();
      if (!paperReviewMap.has(paperId)) {
        paperReviewMap.set(paperId, []);
      }
      paperReviewMap.get(paperId).push(review);
    });
    
    const correlations = [];
    papers.forEach(paper => {
      const paperReviews = paperReviewMap.get(paper._id.toString()) || [];
      const averageScore = paperReviews.reduce((sum, r) => 
        sum + (r.rating?.overall?.score || 0), 0) / paperReviews.length;
      
      correlations.push({
        paperId: paper._id,
        averageScore: averageScore || 0,
        accepted: paper.status === 'accepted' ? 1 : 0
      });
    });
    
    return correlations;
  }

  getCitationDistribution(papers) {
    const distribution = {
      '0': 0,
      '1-5': 0,
      '6-10': 0,
      '11-25': 0,
      '26-50': 0,
      '50+': 0
    };
    
    papers.forEach(paper => {
      const citations = paper.citations?.length || 0;
      if (citations === 0) distribution['0']++;
      else if (citations <= 5) distribution['1-5']++;
      else if (citations <= 10) distribution['6-10']++;
      else if (citations <= 25) distribution['11-25']++;
      else if (citations <= 50) distribution['26-50']++;
      else distribution['50+']++;
    });
    
    return distribution;
  }

  getReviewActivityByReviewer(reviews) {
    return reviews.reduce((acc, review) => {
      const reviewerId = review.reviewer.toString();
      if (!acc[reviewerId]) {
        acc[reviewerId] = { total: 0, completed: 0, pending: 0 };
      }
      
      acc[reviewerId].total++;
      if (review.status === 'completed') acc[reviewerId].completed++;
      if (review.status === 'assigned') acc[reviewerId].pending++;
      
      return acc;
    }, {});
  }

  getCitationsByType(citations) {
    return citations.reduce((acc, citation) => {
      const type = citation.type || 'unknown';
      acc[type] = (acc[type] || 0) + 1;
      return acc;
    }, {});
  }

  getCitationsBySource(citations) {
    return citations.reduce((acc, citation) => {
      const source = citation.source?.database || 'unknown';
      acc[source] = (acc[source] || 0) + 1;
      return acc;
    }, {});
  }

  getCitationImpactTrends(citations) {
    const yearlyImpact = {};
    
    citations.forEach(citation => {
      if (citation.year) {
        const year = citation.year;
        if (!yearlyImpact[year]) {
          yearlyImpact[year] = { total: 0, averageCitations: 0, papers: [] };
        }
        yearlyImpact[year].total += citation.totalCitations || 0;
        yearlyImpact[year].papers.push(citation);
      }
    });
    
    return Object.entries(yearlyImpact)
      .map(([year, data]) => ({
        year: parseInt(year),
        totalCitations: data.total,
        averageCitations: data.papers.length > 0 ? data.total / data.papers.length : 0,
        paperCount: data.papers.length
      }))
      .sort((a, b) => a.year - b.year);
  }

  getConferencesByStatus(conferences) {
    return conferences.reduce((acc, conference) => {
      const status = conference.status || 'unknown';
      acc[status] = (acc[status] || 0) + 1;
      return acc;
    }, {});
  }

  getUpcomingSubmissionDeadlines(conferences) {
    const now = new Date();
    const thirtyDaysFromNow = new Date(now.getTime() + 30 * 24 * 60 * 60 * 1000);
    
    return conferences
      .filter(conference => {
        const deadline = new Date(conference.submissionDeadline);
        return deadline > now && deadline <= thirtyDaysFromNow;
      })
      .map(conference => ({
        id: conference._id,
        name: conference.name,
        submissionDeadline: conference.submissionDeadline,
        daysUntilDeadline: Math.ceil((new Date(conference.submissionDeadline) - now) / (1000 * 60 * 60 * 24))
      }))
      .sort((a, b) => a.daysUntilDeadline - b.daysUntilDeadline);
  }

  async getSystemUptime() {
    // This would typically come from system monitoring
    // For now, return a placeholder
    return process.uptime();
  }

  async getActiveSessions() {
    // This would typically come from session management
    // For now, return a placeholder
    return Math.floor(Math.random() * 100) + 50;
  }

  // Research trend analysis methods
  async getResearchTrends(dateFilter) {
    try {
      const papers = await Paper.find(dateFilter);
      const trends = {};
      
      papers.forEach(paper => {
        const area = paper.researchArea || 'Unknown';
        if (!trends[area]) {
          trends[area] = { papers: 0, citations: 0, collaborations: 0 };
        }
        trends[area].papers++;
        trends[area].citations += paper.citations?.length || 0;
        
        // Count collaborations (papers with multiple authors)
        if (paper.authors && paper.authors.length > 1) {
          trends[area].collaborations++;
        }
      });
      
      return trends;
    } catch (error) {
      console.error('Error getting research trends:', error);
      return {};
    }
  }

  async getCitationTrends(dateFilter) {
    try {
      const citations = await Citation.find(dateFilter);
      const trends = {};
      
      citations.forEach(citation => {
        const year = citation.year || 'Unknown';
        if (!trends[year]) {
          trends[year] = { count: 0, totalImpact: 0 };
        }
        trends[year].count++;
        trends[year].totalImpact += citation.totalCitations || 0;
      });
      
      return trends;
    } catch (error) {
      console.error('Error getting citation trends:', error);
      return {};
    }
  }

  async getCollaborationTrends(dateFilter) {
    try {
      const papers = await Paper.find(dateFilter);
      const collaborationTrends = {};
      
      papers.forEach(paper => {
        if (paper.authors && paper.authors.length > 1) {
          const year = new Date(paper.createdAt).getFullYear();
          if (!collaborationTrends[year]) {
            collaborationTrends[year] = { collaborativePapers: 0, totalPapers: 0 };
          }
          collaborationTrends[year].collaborativePapers++;
          collaborationTrends[year].totalPapers++;
        }
      });
      
      return Object.entries(collaborationTrends)
        .map(([year, data]) => ({
          year: parseInt(year),
          collaborationRate: data.totalPapers > 0 ? (data.collaborativePapers / data.totalPapers) * 100 : 0,
          collaborativePapers: data.collaborativePapers,
          totalPapers: data.totalPapers
        }))
        .sort((a, b) => a.year - b.year);
    } catch (error) {
      console.error('Error getting collaboration trends:', error);
      return [];
    }
  }

  async getEmergingResearchAreas(dateFilter) {
    try {
      const papers = await Paper.find(dateFilter);
      const areaMetrics = {};
      
      papers.forEach(paper => {
        const area = paper.researchArea || 'Unknown';
        const date = new Date(paper.createdAt);
        const quarter = `${date.getFullYear()}-Q${Math.ceil((date.getMonth() + 1) / 3)}`;
        
        if (!areaMetrics[area]) {
          areaMetrics[area] = {};
        }
        
        if (!areaMetrics[area][quarter]) {
          areaMetrics[area][quarter] = 0;
        }
        
        areaMetrics[area][quarter]++;
      });
      
      // Calculate growth rates and identify emerging areas
      const emergingAreas = [];
      for (const [area, quarters] of Object.entries(areaMetrics)) {
        const quarterValues = Object.values(quarters);
        if (quarterValues.length >= 2) {
          const recent = quarterValues[quarterValues.length - 1];
          const previous = quarterValues[quarterValues.length - 2];
          const growth = previous > 0 ? ((recent - previous) / previous) * 100 : 0;
          
          if (growth > 25) { // 25% growth threshold
            emergingAreas.push({
              area,
              growthRate: growth,
              recentPublications: recent,
              previousPublications: previous
            });
          }
        }
      }
      
      return emergingAreas.sort((a, b) => b.growthRate - a.growthRate);
    } catch (error) {
      console.error('Error getting emerging research areas:', error);
      return [];
    }
  }

  async getResearchPredictions(dateFilter) {
    try {
      const papers = await Paper.find(dateFilter);
      const trends = this.getResearchTrends(dateFilter);
      
      // Simple prediction based on current trends
      const predictions = [];
      for (const [area, metrics] of Object.entries(trends)) {
        if (metrics.papers > 5) { // Only predict for areas with sufficient data
          const growthRate = this.calculateGrowthRate(metrics.papers, 12); // Assuming monthly data
          predictions.push({
            area,
            predictedGrowth: growthRate,
            confidence: Math.min(metrics.papers / 20, 1) * 100, // Confidence based on data volume
            timeframe: 'next_quarter'
          });
        }
      }
      
      return predictions.sort((a, b) => b.predictedGrowth - a.predictedGrowth);
    } catch (error) {
      console.error('Error getting research predictions:', error);
      return [];
    }
  }

  calculateGrowthRate(currentValue, periods) {
    // Simplified growth rate calculation
    // In a real implementation, this would use more sophisticated time series analysis
    return Math.random() * 20 - 10; // Placeholder: -10% to +10%
  }

  calculateAverageImpactFactor(citations) {
    const citationsWithIF = citations.filter(c => c.impactFactor > 0);
    if (citationsWithIF.length === 0) return 0;
    
    const totalIF = citationsWithIF.reduce((sum, c) => sum + c.impactFactor, 0);
    return totalIF / citationsWithIF.length;
  }

  // Export analytics data
  async exportAnalytics(type, format, userId = null, timeRange = '1year') {
    try {
      const dateFilter = this.getDateFilter(timeRange);
      let data;
      
      switch (type) {
        case 'papers':
          data = await Paper.find(dateFilter).populate('authors citations');
          break;
        case 'reviews':
          data = await Review.find(dateFilter).populate('paper reviewer');
          break;
        case 'citations':
          data = await Citation.find(dateFilter);
          break;
        case 'users':
          data = await User.find(dateFilter);
          break;
        case 'dashboard':
          data = await this.getDashboardAnalytics(userId, timeRange);
          break;
        default:
          throw new Error(`Unsupported export type: ${type}`);
      }
      
      switch (format) {
        case 'json':
          return { data, format: 'json' };
        case 'csv':
          return { data: this.convertToCSV(data, type), format: 'csv' };
        default:
          throw new Error(`Unsupported export format: ${format}`);
      }
    } catch (error) {
      throw new Error(`Export failed: ${error.message}`);
    }
  }

  convertToCSV(data, type) {
    // Simple CSV conversion - in a real implementation, this would be more sophisticated
    const json2csv = require('json2csv').parse;
    return json2csv(data, { header: true });
  }
}

module.exports = AnalyticsService;