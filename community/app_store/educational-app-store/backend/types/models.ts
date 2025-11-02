// Core types for Educational App Store

export interface User {
  id: string;
  email: string;
  name: string;
  role: 'developer' | 'educator' | 'admin';
  institution?: string;
  bio?: string;
  avatar?: string;
  createdAt: Date;
  updatedAt: Date;
}

export interface App {
  id: string;
  title: string;
  description: string;
  shortDescription: string;
  developerId: string;
  categoryId: string;
  subcategoryId?: string;
  gradeLevels: string[];
  subjects: string[];
  tags: string[];
  price: number;
  currency: string;
  platform: 'web' | 'mobile' | 'desktop' | 'cross-platform';
  version: string;
  appSize?: number;
  screenshots: string[];
  icon: string;
  videoUrl?: string;
  websiteUrl?: string;
  downloadUrl?: string;
  status: 'draft' | 'pending' | 'approved' | 'rejected' | 'suspended';
  rating: number;
  reviewCount: number;
  downloadCount: number;
  featured: boolean;
  verified: boolean;
  educationalImpact: EducationalImpact;
  accessibility: AccessibilityFeatures;
  createdAt: Date;
  updatedAt: Date;
  approvedAt?: Date;
  approvedBy?: string;
  rejectionReason?: string;
}

export interface Category {
  id: string;
  name: string;
  description: string;
  icon: string;
  color: string;
  parentId?: string;
  sortOrder: number;
  active: boolean;
  createdAt: Date;
}

export interface Subcategory {
  id: string;
  categoryId: string;
  name: string;
  description: string;
  icon: string;
  sortOrder: number;
  active: boolean;
  createdAt: Date;
}

export interface Review {
  id: string;
  appId: string;
  userId: string;
  rating: number;
  title: string;
  content: string;
  helpful: number;
  notHelpful: number;
  verified: boolean;
  createdAt: Date;
  updatedAt: Date;
}

export interface AppSubmission {
  id: string;
  developerId: string;
  appData: Partial<App>;
  status: 'draft' | 'submitted' | 'under_review' | 'approved' | 'rejected';
  submissionNotes?: string;
  reviewNotes?: string;
  reviewerId?: string;
  submittedAt: Date;
  reviewedAt?: Date;
  createdAt: Date;
}

export interface EducationalImpact {
  learningOutcomes: string[];
  curriculumAlignment: CurriculumAlignment[];
  pedagogicalApproach: string[];
  assessmentFeatures: AssessmentType[];
  collaboration: boolean;
  adaptiveLearning: boolean;
}

export interface CurriculumAlignment {
  standard: string;
  gradeLevel: string;
  subject: string;
  description: string;
}

export interface AccessibilityFeatures {
  screenReader: boolean;
  keyboardNavigation: boolean;
  highContrast: boolean;
  textToSpeech: boolean;
  subtitles: boolean;
  dyslexicFriendly: boolean;
}

export type AssessmentType = 
  | 'quiz'
  | 'assignment'
  | 'project'
  | 'observation'
  | 'peer_review'
  | 'self_assessment'
  | 'automated';

export interface AppAnalytics {
  id: string;
  appId: string;
  date: Date;
  views: number;
  downloads: number;
  reviews: number;
  averageRating: number;
  bounceRate: number;
  sessionDuration: number;
  deviceTypes: DeviceTypeCount[];
  geographicData: GeographicData[];
}

export interface DeviceTypeCount {
  device: 'desktop' | 'mobile' | 'tablet';
  count: number;
  percentage: number;
}

export interface GeographicData {
  country: string;
  count: number;
  percentage: number;
}

export interface RecommendationData {
  appId: string;
  userId?: string;
  score: number;
  reason: 'similar_users' | 'category_match' | 'popular' | 'recent' | 'featured';
  metadata: Record<string, any>;
}

export interface SearchFilters {
  categories?: string[];
  subcategories?: string[];
  gradeLevels?: string[];
  subjects?: string[];
  price?: 'free' | 'paid' | 'freemium';
  platform?: string[];
  rating?: number;
  ageGroup?: string;
  language?: string[];
  accessibility?: string[];
  searchQuery?: string;
  sortBy?: 'rating' | 'popularity' | 'newest' | 'price_low' | 'price_high';
  sortOrder?: 'asc' | 'desc';
}

export interface DeveloperSubmission {
  id: string;
  developerId: string;
  appId?: string;
  title: string;
  description: string;
  version: string;
  changelog?: string;
  technicalRequirements: TechnicalRequirements;
  educationalContent: EducationalContent;
  mediaFiles: MediaFiles;
  complianceData: ComplianceData;
  status: 'draft' | 'submitted' | 'under_review' | 'needs_changes' | 'approved' | 'rejected';
  submittedAt?: Date;
  reviewedAt?: Date;
  reviewerNotes?: string;
  createdAt: Date;
  updatedAt: Date;
}

export interface TechnicalRequirements {
  minimumOS: string[];
  browserRequirements?: string[];
  internetRequired: boolean;
  storageRequired: number;
  ramRequired: number;
  processorRequirements?: string;
  gpuRequirements?: string;
}

export interface EducationalContent {
  targetAgeGroup: string;
  gradeLevels: string[];
  subjects: string[];
  learningObjectives: string[];
  curriculumStandards: string[];
  teachingMethod: string[];
  difficultyLevel: 'beginner' | 'intermediate' | 'advanced';
  estimatedDuration: string;
  collaboration: boolean;
  multilingual: boolean;
}

export interface MediaFiles {
  icon: File | null;
  screenshots: (File | string)[];
  video?: File | null;
  demoLink?: string;
}

export interface ComplianceData {
  coppaCompliant: boolean;
  dataProtection: boolean;
  accessibilityCompliant: boolean;
  accessibilityStandard: string;
  privacyPolicy: boolean;
  termsOfService: boolean;
  contentRating: string;
  parentalConsent: boolean;
}

export interface APIResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  limit: number;
  totalPages: number;
}

export interface AppStoreStats {
  totalApps: number;
  totalDownloads: number;
  totalReviews: number;
  averageRating: number;
  topCategories: { name: string; count: number }[];
  recentSubmissions: number;
  pendingReviews: number;
}