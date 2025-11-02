"""
Code Review and Approval Workflow System
Educational-focused code review with peer learning and assessment
"""

from typing import Dict, List, Optional, Set
from datetime import datetime
from dataclasses import dataclass, asdict
from enum import Enum
import json
from pathlib import Path


class ReviewStatus(Enum):
    PENDING = "pending"
    IN_REVIEW = "in_review"
    APPROVED = "approved"
    CHANGES_REQUESTED = "changes_requested"
    REJECTED = "rejected"


class ReviewPriority(Enum):
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"


@dataclass
class ReviewComment:
    """Represents a review comment"""
    id: str
    file_path: str
    line_number: Optional[int]
    author: str
    content: str
    timestamp: datetime
    resolved: bool = False
    parent_comment_id: Optional[str] = None
    comment_type: str = "general"  # suggestion, issue, question, praise
    
    def to_dict(self) -> Dict:
        return {
            'id': self.id,
            'file_path': self.file_path,
            'line_number': self.line_number,
            'author': self.author,
            'content': self.content,
            'timestamp': self.timestamp.isoformat(),
            'resolved': self.resolved,
            'parent_comment_id': self.parent_comment_id,
            'comment_type': self.comment_type
        }
    
    @classmethod
    def from_dict(cls, data: Dict) -> 'ReviewComment':
        return cls(
            id=data['id'],
            file_path=data['file_path'],
            line_number=data['line_number'],
            author=data['author'],
            content=data['content'],
            timestamp=datetime.fromisoformat(data['timestamp']),
            resolved=data.get('resolved', False),
            parent_comment_id=data.get('parent_comment_id'),
            comment_type=data.get('comment_type', 'general')
        )


@dataclass
class ReviewRequest:
    """Represents a code review request"""
    id: str
    pull_request_id: str
    author: str
    title: str
    description: str
    target_branch: str
    source_branch: str
    created_at: datetime
    reviewers: List[str]
    status: ReviewStatus
    priority: ReviewPriority
    comments: List[ReviewComment]
    educational_focus: str = ""  # What learning objective this review serves
    
    def to_dict(self) -> Dict:
        return {
            'id': self.id,
            'pull_request_id': self.pull_request_id,
            'author': self.author,
            'title': self.title,
            'description': self.description,
            'target_branch': self.target_branch,
            'source_branch': self.source_branch,
            'created_at': self.created_at.isoformat(),
            'reviewers': self.reviewers,
            'status': self.status.value,
            'priority': self.priority.value,
            'comments': [comment.to_dict() for comment in self.comments],
            'educational_focus': self.educational_focus
        }
    
    @classmethod
    def from_dict(cls, data: Dict) -> 'ReviewRequest':
        return cls(
            id=data['id'],
            pull_request_id=data['pull_request_id'],
            author=data['author'],
            title=data['title'],
            description=data['description'],
            target_branch=data['target_branch'],
            source_branch=data['source_branch'],
            created_at=datetime.fromisoformat(data['created_at']),
            reviewers=data['reviewers'],
            status=ReviewStatus(data['status']),
            priority=ReviewPriority(data['priority']),
            comments=[ReviewComment.from_dict(comment_data) for comment_data in data['comments']],
            educational_focus=data.get('educational_focus', '')
        )


class EducationalCodeReview:
    """Educational-focused code review system"""
    
    def __init__(self, vcs_repo):
        self.repo = vcs_repo
        self.reviews_dir = vcs_repo.repo_path / '.edu_vcs' / 'reviews'
        self.reviews_dir.mkdir(parents=True, exist_ok=True)
        self.reviews_file = self.reviews_dir / 'reviews.json'
        
        # Learning objectives and rubrics
        self.review_rubrics = {
            'code_quality': {
                'readability': {'weight': 0.3, 'description': 'Code is easy to read and understand'},
                'structure': {'weight': 0.2, 'description': 'Good code organization and structure'},
                'naming': {'weight': 0.2, 'description': 'Meaningful variable and function names'},
                'comments': {'weight': 0.15, 'description': 'Appropriate comments and documentation'},
                'consistency': {'weight': 0.15, 'description': 'Consistent coding style'}
            },
            'functionality': {
                'correctness': {'weight': 0.4, 'description': 'Code works as intended'},
                'efficiency': {'weight': 0.2, 'description': 'Appropriate time and space complexity'},
                'robustness': {'weight': 0.2, 'description': 'Handles edge cases properly'},
                'testing': {'weight': 0.2, 'description': 'Includes appropriate tests'}
            },
            'educational': {
                'learning_objective': {'weight': 0.4, 'description': 'Demonstrates learning objective'},
                'creativity': {'weight': 0.3, 'description': 'Shows creative problem solving'},
                'best_practices': {'weight': 0.3, 'description': 'Follows coding best practices'}
            }
        }
        
        self._load_reviews()
    
    def _load_reviews(self):
        """Load existing reviews from storage"""
        if self.reviews_file.exists():
            with open(self.reviews_file, 'r') as f:
                data = json.load(f)
                self.reviews = {
                    rid: ReviewRequest.from_dict(rdata) 
                    for rid, rdata in data.items()
                }
        else:
            self.reviews = {}
    
    def _save_reviews(self):
        """Save reviews to storage"""
        data = {
            rid: review.to_dict() 
            for rid, review in self.reviews.items()
        }
        with open(self.reviews_file, 'w') as f:
            json.dump(data, f, indent=2)
    
    def create_review_request(self, pull_request_id: str, author: str, 
                            title: str, description: str, target_branch: str,
                            source_branch: str, reviewers: List[str],
                            priority: ReviewPriority = ReviewPriority.MEDIUM,
                            educational_focus: str = "") -> str:
        """Create a new review request"""
        review_id = f"review_{len(self.reviews) + 1}"
        
        review = ReviewRequest(
            id=review_id,
            pull_request_id=pull_request_id,
            author=author,
            title=title,
            description=description,
            target_branch=target_branch,
            source_branch=source_branch,
            created_at=datetime.now(),
            reviewers=reviewers,
            status=ReviewStatus.PENDING,
            priority=priority,
            comments=[],
            educational_focus=educational_focus
        )
        
        self.reviews[review_id] = review
        self._save_reviews()
        
        return review_id
    
    def add_comment(self, review_id: str, file_path: str, author: str,
                   content: str, line_number: Optional[int] = None,
                   comment_type: str = "general",
                   parent_comment_id: Optional[str] = None) -> str:
        """Add a comment to a review"""
        if review_id not in self.reviews:
            raise ValueError(f"Review {review_id} not found")
        
        review = self.reviews[review_id]
        comment_id = f"comment_{len(review.comments) + 1}"
        
        comment = ReviewComment(
            id=comment_id,
            file_path=file_path,
            line_number=line_number,
            author=author,
            content=content,
            timestamp=datetime.now(),
            comment_type=comment_type,
            parent_comment_id=parent_comment_id
        )
        
        review.comments.append(comment)
        self._save_reviews()
        
        return comment_id
    
    def resolve_comment(self, review_id: str, comment_id: str, resolver: str) -> Dict:
        """Mark a comment as resolved"""
        if review_id not in self.reviews:
            return {'status': 'error', 'message': 'Review not found'}
        
        review = self.reviews[review_id]
        comment = next((c for c in review.comments if c.id == comment_id), None)
        
        if not comment:
            return {'status': 'error', 'message': 'Comment not found'}
        
        comment.resolved = True
        self._save_reviews()
        
        return {
            'status': 'success',
            'message': f'Comment {comment_id} resolved by {resolver}'
        }
    
    def update_review_status(self, review_id: str, status: ReviewStatus,
                           updater: str) -> Dict:
        """Update the status of a review"""
        if review_id not in self.reviews:
            return {'status': 'error', 'message': 'Review not found'}
        
        review = self.reviews[review_id]
        review.status = status
        self._save_reviews()
        
        return {
            'status': 'success',
            'message': f'Review {review_id} status updated to {status.value} by {updater}'
        }
    
    def get_review_summary(self, review_id: str) -> Dict:
        """Get a summary of review statistics and feedback"""
        if review_id not in self.reviews:
            return {'status': 'error', 'message': 'Review not found'}
        
        review = self.reviews[review_id]
        
        # Count comments by type
        comment_stats = {}
        for comment in review.comments:
            comment_stats[comment.comment_type] = comment_stats.get(comment.comment_type, 0) + 1
        
        # Calculate unresolved issues
        unresolved = sum(1 for c in review.comments if not c.resolved)
        
        # Generate educational feedback
        feedback = self._generate_educational_feedback(review)
        
        return {
            'review_id': review_id,
            'status': review.status.value,
            'total_comments': len(review.comments),
            'resolved_comments': len(review.comments) - unresolved,
            'unresolved_comments': unresolved,
            'comment_breakdown': comment_stats,
            'reviewers_needed': len(review.reviewers),
            'educational_feedback': feedback,
            'created_at': review.created_at.isoformat()
        }
    
    def _generate_educational_feedback(self, review: ReviewRequest) -> Dict:
        """Generate educational feedback based on review content"""
        feedback = {
            'learning_points': [],
            'strengths': [],
            'areas_for_improvement': [],
            'next_steps': []
        }
        
        # Analyze comments for educational insights
        suggestions = [c for c in review.comments if c.comment_type == 'suggestion']
        issues = [c for c in review.comments if c.comment_type == 'issue']
        questions = [c for c in review.comments if c.comment_type == 'question']
        praise = [c for c in review.comments if c.comment_type == 'praise']
        
        if praise:
            feedback['strengths'].append("Reviewers appreciated your implementation approach")
        
        if suggestions:
            feedback['areas_for_improvement'].append("Consider implementing the suggested improvements")
        
        if issues:
            feedback['learning_points'].append("Address the identified issues to strengthen your code")
        
        if questions:
            feedback['learning_points'].append("Review and clarify areas that raised questions")
        
        # Generic next steps
        feedback['next_steps'] = [
            "Address all unresolved comments",
            "Run tests to ensure changes work correctly",
            "Consider refactoring based on feedback",
            "Document any architectural decisions"
        ]
        
        return feedback
    
    def get_available_reviewers(self, target_file: str) -> List[str]:
        """Get list of potential reviewers based on expertise"""
        # In a real system, this would query user expertise and availability
        # For now, return a simple list
        expertise_map = {
            'python': ['alice', 'bob', 'charlie'],
            'javascript': ['alice', 'diana', 'eve'],
            'database': ['bob', 'frank'],
            'testing': ['charlie', 'grace']
        }
        
        file_ext = target_file.split('.')[-1]
        language = file_ext if file_ext in expertise_map else 'general'
        
        return expertise_map.get(language, ['instructor', 'ta'])
    
    def generate_review_rubric(self, focus_area: str) -> Dict:
        """Generate a review rubric for a specific focus area"""
        return self.review_rubrics.get(focus_area, {})
    
    def create_pull_request(self, source_branch: str, target_branch: str,
                          title: str, description: str, author: str) -> str:
        """Create a pull request (simplified version)"""
        pr_id = f"pr_{len([r for r in self.reviews.values() if r.pull_request_id.startswith('pr_')]) + 1}"
        
        # Create an initial review request for the PR
        review_id = self.create_review_request(
            pull_request_id=pr_id,
            author=author,
            title=title,
            description=description,
            target_branch=target_branch,
            source_branch=source_branch,
            reviewers=[],  # Will be assigned later
            priority=ReviewPriority.MEDIUM,
            educational_focus="Code review for collaborative learning"
        )
        
        return pr_id
    
    def merge_pull_request(self, pr_id: str, merger: str) -> Dict:
        """Merge a pull request after review approval"""
        # Find the review for this PR
        review = next(
            (r for r in self.reviews.values() if r.pull_request_id == pr_id),
            None
        )
        
        if not review:
            return {'status': 'error', 'message': 'Pull request not found'}
        
        if review.status != ReviewStatus.APPROVED:
            return {'status': 'error', 'message': 'Pull request not approved'}
        
        # Update status to merged
        review.status = ReviewStatus.APPROVED  # Or a new MERGED status
        self._save_reviews()
        
        return {
            'status': 'success',
            'message': f'Pull request {pr_id} merged by {merger}',
            'merged_at': datetime.now().isoformat()
        }
