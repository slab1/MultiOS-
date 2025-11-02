"""
Community Portal Module
======================

Provides community package sharing, discovery, rating, and review system
for the educational package manager.
"""

import os
import json
import logging
import hashlib
import requests
from datetime import datetime, timedelta
from pathlib import Path
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass, asdict
from enum import Enum
import re
import base64

from package_manager import PackageMetadata, PackageType, CompatibilityLevel

logger = logging.getLogger(__name__)


class PackageStatus(Enum):
    """Package publication status"""
    DRAFT = "draft"
    PENDING_REVIEW = "pending_review"
    PUBLISHED = "published"
    REJECTED = "rejected"
    DEPRECATED = "deprecated"


class RatingScore(Enum):
    """Package rating scores"""
    EXCELLENT = 5
    GOOD = 4
    AVERAGE = 3
    POOR = 2
    VERY_POOR = 1


@dataclass
class PackageReview:
    """User review for a package"""
    package_name: str
    user_id: str
    username: str
    rating: int
    title: str
    content: str
    pros: List[str]
    cons: List[str]
    created_at: str
    updated_at: str
    helpful_votes: int = 0
    verified: bool = False
    
    def __post_init__(self):
        if self.updated_at is None:
            self.updated_at = self.created_at


@dataclass
class CommunityPackage:
    """Community package with additional metadata"""
    metadata: PackageMetadata
    status: PackageStatus
    download_count: int
    rating_average: float
    rating_count: int
    reviews: List[PackageReview]
    download_url: str
    created_at: str
    updated_at: str
    published_at: Optional[str] = None
    featured: bool = False
    moderation_notes: str = ""
    developer_notes: str = ""
    tags: List[str] = None
    
    def __post_init__(self):
        if self.tags is None:
            self.tags = []


@dataclass
class SearchFilters:
    """Search filters for package discovery"""
    query: str = ""
    package_type: Optional[PackageType] = None
    compatibility: Optional[CompatibilityLevel] = None
    subjects: List[str] = None
    grade_levels: List[str] = None
    min_rating: float = 0.0
    featured_only: bool = False
    sort_by: str = "rating"  # rating, downloads, date, name
    sort_order: str = "desc"  # asc, desc
    limit: int = 50
    offset: int = 0
    
    def __post_init__(self):
        if self.subjects is None:
            self.subjects = []
        if self.grade_levels is None:
            self.grade_levels = []


class CommunityPortal:
    """Community portal for package sharing and discovery"""
    
    def __init__(self, package_manager):
        self.pm = package_manager
        self.community_dir = Path("/workspace/community/package_manager/packages/community")
        self.reviews_dir = Path("/workspace/community/package_manager/packages/reviews")
        self.assets_dir = Path("/workspace/community/package_manager/packages/assets")
        
        # Ensure directories exist
        for directory in [self.community_dir, self.reviews_dir, self.assets_dir]:
            directory.mkdir(parents=True, exist_ok=True)
        
        # Community portal API endpoints (would be configured)
        self.api_base_url = os.environ.get('MULTIOS_COMMUNITY_API', 'https://community.multios.edu/api')
        self.api_key = os.environ.get('MULTIOS_API_KEY')
        
    def publish_package(self, metadata: PackageMetadata, package_file: str, 
                       publish_notes: str = "", is_featured: bool = False) -> bool:
        """Publish package to community portal"""
        logger.info(f"Publishing package to community: {metadata.name}")
        
        try:
            # Validate package before publishing
            validation_result = self.pm.validator.validate_package(
                os.path.dirname(package_file), metadata
            )
            
            if not validation_result.passed:
                logger.error(f"Package validation failed before publishing: {validation_result.issues}")
                return False
            
            # Create community package record
            community_package = CommunityPackage(
                metadata=metadata,
                status=PackageStatus.PENDING_REVIEW,
                download_count=0,
                rating_average=0.0,
                rating_count=0,
                reviews=[],
                download_url=self._generate_download_url(metadata),
                created_at=datetime.now().isoformat(),
                updated_at=datetime.now().isoformat(),
                featured=is_featured,
                moderation_notes=publish_notes
            )
            
            # Upload package to community repository
            if not self._upload_package_file(package_file, metadata):
                logger.error("Failed to upload package file to community repository")
                return False
            
            # Store community metadata
            if not self._store_community_package(community_package):
                logger.error("Failed to store community package metadata")
                return False
            
            # Notify moderators for review
            self._notify_moderators(community_package)
            
            logger.info(f"Package {metadata.name} submitted for review")
            return True
            
        except Exception as e:
            logger.error(f"Error publishing package: {e}")
            return False
    
    def search_packages(self, filters: SearchFilters) -> List[CommunityPackage]:
        """Search for packages with filters"""
        logger.info(f"Searching packages with filters: {filters.query}")
        
        try:
            # Load all community packages
            all_packages = self._load_all_community_packages()
            
            # Apply filters
            filtered_packages = self._apply_filters(all_packages, filters)
            
            # Sort results
            sorted_packages = self._sort_packages(filtered_packages, filters.sort_by, filters.sort_order)
            
            # Apply pagination
            start_index = filters.offset
            end_index = start_index + filters.limit
            paginated_packages = sorted_packages[start_index:end_index]
            
            logger.info(f"Found {len(paginated_packages)} packages matching criteria")
            return paginated_packages
            
        except Exception as e:
            logger.error(f"Error searching packages: {e}")
            return []
    
    def get_package(self, package_name: str, version: str = None) -> Optional[CommunityPackage]:
        """Get detailed information about a package"""
        try:
            # Try to get specific version
            package_file = self.community_dir / f"{package_name}_{version}.json" if version else None
            
            if package_file and package_file.exists():
                with open(package_file, 'r') as f:
                    data = json.load(f)
                return self._dict_to_community_package(data)
            
            # If version not specified or not found, get latest published version
            all_packages = self._load_all_community_packages()
            for package in all_packages:
                if package.metadata.name == package_name and package.status == PackageStatus.PUBLISHED:
                    if version is None or package.metadata.version == version:
                        return package
            
            logger.warning(f"Package not found: {package_name}")
            return None
            
        except Exception as e:
            logger.error(f"Error getting package {package_name}: {e}")
            return None
    
    def add_review(self, package_name: str, user_id: str, username: str, 
                  rating: int, title: str, content: str, 
                  pros: List[str] = None, cons: List[str] = None) -> bool:
        """Add a review for a package"""
        logger.info(f"Adding review for package: {package_name}")
        
        try:
            # Validate review
            if not self._validate_review(rating, title, content):
                logger.error("Invalid review data")
                return False
            
            # Get package
            package = self.get_package(package_name)
            if not package:
                logger.error(f"Package not found for review: {package_name}")
                return False
            
            # Create review
            review = PackageReview(
                package_name=package_name,
                user_id=user_id,
                username=username,
                rating=rating,
                title=title,
                content=content,
                pros=pros or [],
                cons=cons or [],
                created_at=datetime.now().isoformat(),
                updated_at=datetime.now().isoformat()
            )
            
            # Add review to package
            package.reviews.append(review)
            
            # Update package rating
            self._update_package_rating(package)
            
            # Save updated package
            if not self._store_community_package(package):
                return False
            
            # Store individual review file for moderation
            review_file = self.reviews_dir / f"{package_name}_{user_id}_{datetime.now().strftime('%Y%m%d')}.json"
            with open(review_file, 'w') as f:
                json.dump(asdict(review), f, indent=2)
            
            logger.info(f"Review added successfully for {package_name}")
            return True
            
        except Exception as e:
            logger.error(f"Error adding review: {e}")
            return False
    
    def get_reviews(self, package_name: str, limit: int = 20, 
                   offset: int = 0) -> List[PackageReview]:
        """Get reviews for a package"""
        try:
            package = self.get_package(package_name)
            if not package:
                return []
            
            reviews = package.reviews
            # Sort by helpful votes and date
            reviews.sort(key=lambda r: (r.helpful_votes, r.created_at), reverse=True)
            
            return reviews[offset:offset + limit]
            
        except Exception as e:
            logger.error(f"Error getting reviews for {package_name}: {e}")
            return []
    
    def report_package(self, package_name: str, user_id: str, reason: str, 
                      description: str = "") -> bool:
        """Report a package for inappropriate content"""
        logger.info(f"Reporting package: {package_name}")
        
        try:
            report = {
                'package_name': package_name,
                'reporter_id': user_id,
                'reason': reason,
                'description': description,
                'timestamp': datetime.now().isoformat(),
                'status': 'pending'
            }
            
            report_file = self.community_dir / f"reports_{package_name}_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"
            with open(report_file, 'w') as f:
                json.dump(report, f, indent=2)
            
            # Notify moderators
            self._notify_report(package_name, report)
            
            return True
            
        except Exception as e:
            logger.error(f"Error reporting package: {e}")
            return False
    
    def get_trending_packages(self, days: int = 7, limit: int = 10) -> List[CommunityPackage]:
        """Get trending packages based on recent downloads and ratings"""
        try:
            all_packages = self._load_all_community_packages()
            published_packages = [p for p in all_packages if p.status == PackageStatus.PUBLISHED]
            
            # Calculate trending score for each package
            for package in published_packages:
                package.trending_score = self._calculate_trending_score(package, days)
            
            # Sort by trending score
            published_packages.sort(key=lambda p: getattr(p, 'trending_score', 0), reverse=True)
            
            return published_packages[:limit]
            
        except Exception as e:
            logger.error(f"Error getting trending packages: {e}")
            return []
    
    def get_featured_packages(self, limit: int = 6) -> List[CommunityPackage]:
        """Get featured packages"""
        try:
            all_packages = self._load_all_community_packages()
            featured_packages = [p for p in all_packages 
                               if p.status == PackageStatus.PUBLISHED and p.featured]
            # Sort by rating
            featured_packages.sort(key=lambda p: p.rating_average, reverse=True)
            return featured_packages[:limit]
            
        except Exception as e:
            logger.error(f"Error getting featured packages: {e}")
            return []
    
    def get_package_statistics(self, package_name: str) -> Dict[str, Any]:
        """Get detailed statistics for a package"""
        try:
            package = self.get_package(package_name)
            if not package:
                return {}
            
            stats = {
                'total_downloads': package.download_count,
                'rating_average': package.rating_average,
                'rating_count': package.rating_count,
                'review_count': len(package.reviews),
                'views_last_30_days': 0,  # Would track this separately
                'unique_users': 0,  # Would track unique user downloads
                'rating_distribution': self._get_rating_distribution(package.reviews),
                'review_sentiment': self._analyze_review_sentiment(package.reviews),
                'download_trend': self._get_download_trend(package),
                'featured': package.featured
            }
            
            return stats
            
        except Exception as e:
            logger.error(f"Error getting package statistics: {e}")
            return {}
    
    def _upload_package_file(self, package_file: str, metadata: PackageMetadata) -> bool:
        """Upload package file to community repository"""
        try:
            # Calculate file hash
            with open(package_file, 'rb') as f:
                file_hash = hashlib.sha256(f.read()).hexdigest()
            
            # Upload to community storage (would implement actual upload)
            upload_url = f"{self.api_base_url}/packages/upload"
            headers = {'Authorization': f'Bearer {self.api_key}'} if self.api_key else {}
            
            # For demonstration, we'll just copy to local community directory
            # In reality, this would upload to cloud storage
            community_file = self.assets_dir / f"{metadata.name}_{metadata.version}_{file_hash[:8]}.edu"
            with open(package_file, 'rb') as src, open(community_file, 'wb') as dst:
                dst.write(src.read())
            
            logger.info(f"Package uploaded to community repository: {community_file}")
            return True
            
        except Exception as e:
            logger.error(f"Error uploading package file: {e}")
            return False
    
    def _store_community_package(self, community_package: CommunityPackage) -> bool:
        """Store community package metadata"""
        try:
            package_file = self.community_dir / f"{community_package.metadata.name}_{community_package.metadata.version}.json"
            
            # Convert to dict for storage
            data = {
                'metadata': asdict(community_package.metadata),
                'status': community_package.status.value,
                'download_count': community_package.download_count,
                'rating_average': community_package.rating_average,
                'rating_count': community_package.rating_count,
                'reviews': [asdict(review) for review in community_package.reviews],
                'download_url': community_package.download_url,
                'created_at': community_package.created_at,
                'updated_at': community_package.updated_at,
                'published_at': community_package.published_at,
                'featured': community_package.featured,
                'moderation_notes': community_package.moderation_notes,
                'developer_notes': community_package.developer_notes,
                'tags': community_package.tags
            }
            
            # Convert enum values to strings
            data['metadata']['type'] = data['metadata']['type'].value if hasattr(data['metadata']['type'], 'value') else data['metadata']['type']
            data['metadata']['compatibility'] = data['metadata']['compatibility'].value if hasattr(data['metadata']['compatibility'], 'value') else data['metadata']['compatibility']
            
            with open(package_file, 'w') as f:
                json.dump(data, f, indent=2)
            
            return True
            
        except Exception as e:
            logger.error(f"Error storing community package: {e}")
            return False
    
    def _load_all_community_packages(self) -> List[CommunityPackage]:
        """Load all community packages"""
        packages = []
        
        try:
            for package_file in self.community_dir.glob("*.json"):
                if package_file.name.startswith("reports_"):
                    continue
                    
                with open(package_file, 'r') as f:
                    data = json.load(f)
                    packages.append(self._dict_to_community_package(data))
        except Exception as e:
            logger.error(f"Error loading community packages: {e}")
        
        return packages
    
    def _dict_to_community_package(self, data: Dict[str, Any]) -> CommunityPackage:
        """Convert dictionary to CommunityPackage object"""
        # Convert metadata
        metadata_dict = data['metadata']
        metadata_dict['type'] = PackageType(metadata_dict['type'])
        metadata_dict['compatibility'] = CompatibilityLevel(metadata_dict['compatibility'])
        metadata = PackageMetadata(**metadata_dict)
        
        # Convert reviews
        reviews = []
        for review_dict in data.get('reviews', []):
            reviews.append(PackageReview(**review_dict))
        
        return CommunityPackage(
            metadata=metadata,
            status=PackageStatus(data['status']),
            download_count=data['download_count'],
            rating_average=data['rating_average'],
            rating_count=data['rating_count'],
            reviews=reviews,
            download_url=data['download_url'],
            created_at=data['created_at'],
            updated_at=data['updated_at'],
            published_at=data.get('published_at'),
            featured=data.get('featured', False),
            moderation_notes=data.get('moderation_notes', ''),
            developer_notes=data.get('developer_notes', ''),
            tags=data.get('tags', [])
        )
    
    def _apply_filters(self, packages: List[CommunityPackage], 
                      filters: SearchFilters) -> List[CommunityPackage]:
        """Apply search filters to packages"""
        filtered = packages
        
        # Filter by status (only published packages)
        filtered = [p for p in filtered if p.status == PackageStatus.PUBLISHED]
        
        # Filter by search query
        if filters.query:
            query_lower = filters.query.lower()
            filtered = [p for p in filtered if self._package_matches_query(p, query_lower)]
        
        # Filter by package type
        if filters.package_type:
            filtered = [p for p in filtered if p.metadata.type == filters.package_type]
        
        # Filter by compatibility level
        if filters.compatibility:
            filtered = [p for p in filtered if p.metadata.compatibility == filters.compatibility]
        
        # Filter by subjects
        if filters.subjects:
            filtered = [p for p in filtered if any(s in p.metadata.subjects for s in filters.subjects)]
        
        # Filter by grade levels
        if filters.grade_levels:
            filtered = [p for p in filtered if any(g in p.metadata.grade_levels for g in filters.grade_levels)]
        
        # Filter by minimum rating
        if filters.min_rating > 0:
            filtered = [p for p in filtered if p.rating_average >= filters.min_rating]
        
        # Filter featured only
        if filters.featured_only:
            filtered = [p for p in filtered if p.featured]
        
        return filtered
    
    def _sort_packages(self, packages: List[CommunityPackage], 
                      sort_by: str, sort_order: str) -> List[CommunityPackage]:
        """Sort packages by specified criteria"""
        reverse = sort_order == 'desc'
        
        if sort_by == 'rating':
            packages.sort(key=lambda p: p.rating_average, reverse=reverse)
        elif sort_by == 'downloads':
            packages.sort(key=lambda p: p.download_count, reverse=reverse)
        elif sort_by == 'date':
            packages.sort(key=lambda p: p.published_at or p.created_at, reverse=reverse)
        elif sort_by == 'name':
            packages.sort(key=lambda p: p.metadata.name.lower(), reverse=reverse)
        
        return packages
    
    def _package_matches_query(self, package: CommunityPackage, query: str) -> bool:
        """Check if package matches search query"""
        # Check in metadata fields
        searchable_text = " ".join([
            package.metadata.name.lower(),
            package.metadata.description.lower(),
            " ".join([tag.lower() for tag in package.metadata.tags or []]),
            " ".join([subject.lower() for subject in package.metadata.subjects]),
            " ".join([level.lower() for level in package.metadata.grade_levels])
        ])
        
        return query in searchable_text
    
    def _generate_download_url(self, metadata: PackageMetadata) -> str:
        """Generate download URL for package"""
        package_hash = hashlib.md5(f"{metadata.name}_{metadata.version}".encode()).hexdigest()[:8]
        return f"https://packages.multios.edu/download/{metadata.name}/{metadata.version}/{package_hash}"
    
    def _validate_review(self, rating: int, title: str, content: str) -> bool:
        """Validate review data"""
        if not (1 <= rating <= 5):
            return False
        
        if not title or len(title) < 3 or len(title) > 100:
            return False
        
        if not content or len(content) < 10 or len(content) > 1000:
            return False
        
        return True
    
    def _update_package_rating(self, package: CommunityPackage):
        """Update package rating statistics"""
        if package.reviews:
            total_rating = sum(review.rating for review in package.reviews)
            package.rating_average = total_rating / len(package.reviews)
            package.rating_count = len(package.reviews)
        else:
            package.rating_average = 0.0
            package.rating_count = 0
        
        package.updated_at = datetime.now().isoformat()
    
    def _notify_moderators(self, community_package: CommunityPackage):
        """Notify moderators about new package submission"""
        # In a real implementation, this would send notifications
        logger.info(f"Notifying moderators about new package: {community_package.metadata.name}")
    
    def _notify_report(self, package_name: str, report: Dict[str, Any]):
        """Notify moderators about package report"""
        logger.info(f"Notifying moderators about package report: {package_name}")
    
    def _calculate_trending_score(self, package: CommunityPackage, days: int) -> float:
        """Calculate trending score based on recent activity"""
        # Simplified trending calculation
        # In practice, would consider download velocity, rating changes, etc.
        base_score = package.download_count * 0.3 + package.rating_average * package.rating_count * 0.7
        return base_score
    
    def _get_rating_distribution(self, reviews: List[PackageReview]) -> Dict[int, int]:
        """Get distribution of ratings"""
        distribution = {1: 0, 2: 0, 3: 0, 4: 0, 5: 0}
        for review in reviews:
            distribution[review.rating] += 1
        return distribution
    
    def _analyze_review_sentiment(self, reviews: List[PackageReview]) -> Dict[str, float]:
        """Analyze sentiment of reviews"""
        # Simplified sentiment analysis
        # In practice, would use NLP libraries
        positive_words = ['good', 'great', 'excellent', 'amazing', 'helpful', 'useful']
        negative_words = ['bad', 'terrible', 'awful', 'useless', 'confusing', 'difficult']
        
        sentiment_scores = {'positive': 0.0, 'negative': 0.0, 'neutral': 1.0}
        
        for review in reviews:
            text = (review.content + " " + " ".join(review.pros) + " " + " ".join(review.cons)).lower()
            
            positive_count = sum(1 for word in positive_words if word in text)
            negative_count = sum(1 for word in negative_words if word in text)
            
            if positive_count > negative_count:
                sentiment_scores['positive'] += 1
                sentiment_scores['neutral'] -= 0.5
            elif negative_count > positive_count:
                sentiment_scores['negative'] += 1
                sentiment_scores['neutral'] -= 0.5
        
        total = sum(sentiment_scores.values())
        if total > 0:
            sentiment_scores = {k: v/total for k, v in sentiment_scores.items()}
        
        return sentiment_scores
    
    def _get_download_trend(self, package: CommunityPackage) -> Dict[str, int]:
        """Get download trend data"""
        # Simplified trend data
        # In practice, would track daily downloads
        return {
            'today': 0,
            'this_week': 0,
            'this_month': 0,
            'total': package.download_count
        }