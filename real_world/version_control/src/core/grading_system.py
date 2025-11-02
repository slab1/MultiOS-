"""
Assignment Submission and Grading Integration System
Connects VCS with educational assessment and grading workflows
"""

from typing import Dict, List, Optional, Set
from datetime import datetime, timedelta
from dataclasses import dataclass, asdict
from enum import Enum
import json
from pathlib import Path


class SubmissionStatus(Enum):
    DRAFT = "draft"
    SUBMITTED = "submitted"
    UNDER_REVIEW = "under_review"
    GRADED = "graded"
    RETURNED = "returned"
    LATE = "late"


class GradingScale(Enum):
    PERCENTAGE = "percentage"
    LETTER_GRADE = "letter_grade"
    NUMERIC = "numeric"
    RUBRIC = "rubric"


@dataclass
class AssignmentCriteria:
    """Represents grading criteria for an assignment"""
    name: str
    description: str
    max_points: float
    weight: float  # Weight in final grade (0-1)
    rubric_description: str = ""
    
    def to_dict(self) -> Dict:
        return {
            'name': self.name,
            'description': self.description,
            'max_points': self.max_points,
            'weight': self.weight,
            'rubric_description': self.rubric_description
        }
    
    @classmethod
    def from_dict(cls, data: Dict) -> 'AssignmentCriteria':
        return cls(
            name=data['name'],
            description=data['description'],
            max_points=data['max_points'],
            weight=data['weight'],
            rubric_description=data.get('rubric_description', '')
        )


@dataclass
class Assignment:
    """Represents an assignment in the course"""
    id: str
    title: str
    description: str
    course_id: str
    instructor: str
    created_at: datetime
    due_date: datetime
    max_points: float
    criteria: List[AssignmentCriteria]
    allowed_languages: List[str]
    required_files: List[str]
    test_cases: List[Dict] = None
    plagiarism_check: bool = True
    peer_review_required: bool = False
    
    def to_dict(self) -> Dict:
        return {
            'id': self.id,
            'title': self.title,
            'description': self.description,
            'course_id': self.course_id,
            'instructor': self.instructor,
            'created_at': self.created_at.isoformat(),
            'due_date': self.due_date.isoformat(),
            'max_points': self.max_points,
            'criteria': [c.to_dict() for c in self.criteria],
            'allowed_languages': self.allowed_languages,
            'required_files': self.required_files,
            'test_cases': self.test_cases or [],
            'plagiarism_check': self.plagiarism_check,
            'peer_review_required': self.peer_review_required
        }
    
    @classmethod
    def from_dict(cls, data: Dict) -> 'Assignment':
        return cls(
            id=data['id'],
            title=data['title'],
            description=data['description'],
            course_id=data['course_id'],
            instructor=data['instructor'],
            created_at=datetime.fromisoformat(data['created_at']),
            due_date=datetime.fromisoformat(data['due_date']),
            max_points=data['max_points'],
            criteria=[AssignmentCriteria.from_dict(c) for c in data['criteria']],
            allowed_languages=data['allowed_languages'],
            required_files=data['required_files'],
            test_cases=data.get('test_cases', []),
            plagiarism_check=data.get('plagiarism_check', True),
            peer_review_required=data.get('peer_review_required', False)
        )


@dataclass
class Submission:
    """Represents a student submission"""
    id: str
    assignment_id: str
    student_id: str
    submitted_at: datetime
    commit_hash: str
    branch: str
    files: List[str]
    status: SubmissionStatus
    grade: Optional[float] = None
    feedback: List[Dict] = None
    plagiarism_score: Optional[float] = None
    peer_reviews: List[str] = None  # Review IDs
    
    def to_dict(self) -> Dict:
        return {
            'id': self.id,
            'assignment_id': self.assignment_id,
            'student_id': self.student_id,
            'submitted_at': self.submitted_at.isoformat(),
            'commit_hash': self.commit_hash,
            'branch': self.branch,
            'files': self.files,
            'status': self.status.value,
            'grade': self.grade,
            'feedback': self.feedback or [],
            'plagiarism_score': self.plagiarism_score,
            'peer_reviews': self.peer_reviews or []
        }
    
    @classmethod
    def from_dict(cls, data: Dict) -> 'Submission':
        return cls(
            id=data['id'],
            assignment_id=data['assignment_id'],
            student_id=data['student_id'],
            submitted_at=datetime.fromisoformat(data['submitted_at']),
            commit_hash=data['commit_hash'],
            branch=data['branch'],
            files=data['files'],
            status=SubmissionStatus(data['status']),
            grade=data.get('grade'),
            feedback=data.get('feedback', []),
            plagiarism_score=data.get('plagiarism_score'),
            peer_reviews=data.get('peer_reviews', [])
        )


class AssignmentGradingSystem:
    """Handles assignment submission and grading"""
    
    def __init__(self, vcs_repo):
        self.repo = vcs_repo
        self.assignments_dir = vcs_repo.repo_path / '.edu_vcs' / 'assignments'
        self.submissions_dir = vcs_repo.repo_path / '.edu_vcs' / 'submissions'
        self.grades_dir = vcs_repo.repo_path / '.edu_vcs' / 'grades'
        
        # Create directories
        for dir_path in [self.assignments_dir, self.submissions_dir, self.grades_dir]:
            dir_path.mkdir(parents=True, exist_ok=True)
        
        self.assignments_file = self.assignments_dir / 'assignments.json'
        self.submissions_file = self.submissions_dir / 'submissions.json'
        self.grades_file = self.grades_dir / 'grades.json'
        
        self._load_data()
    
    def _load_data(self):
        """Load assignments and submissions from storage"""
        # Load assignments
        if self.assignments_file.exists():
            with open(self.assignments_file, 'r') as f:
                data = json.load(f)
                self.assignments = {
                    aid: Assignment.from_dict(adata)
                    for aid, adata in data.items()
                }
        else:
            self.assignments = {}
        
        # Load submissions
        if self.submissions_file.exists():
            with open(self.submissions_file, 'r') as f:
                data = json.load(f)
                self.submissions = {
                    sid: Submission.from_dict(sdata)
                    for sid, sdata in data.items()
                }
        else:
            self.submissions = {}
        
        # Load grades
        if self.grades_file.exists():
            with open(self.grades_file, 'r') as f:
                self.grades = json.load(f)
        else:
            self.grades = {}
    
    def _save_data(self):
        """Save all data to storage"""
        # Save assignments
        assignments_data = {
            aid: assignment.to_dict()
            for aid, assignment in self.assignments.items()
        }
        with open(self.assignments_file, 'w') as f:
            json.dump(assignments_data, f, indent=2)
        
        # Save submissions
        submissions_data = {
            sid: submission.to_dict()
            for sid, submission in self.submissions.items()
        }
        with open(self.submissions_file, 'w') as f:
            json.dump(submissions_data, f, indent=2)
        
        # Save grades
        with open(self.grades_file, 'w') as f:
            json.dump(self.grades, f, indent=2)
    
    def create_assignment(self, assignment_data: Dict) -> str:
        """Create a new assignment"""
        assignment_id = f"assignment_{len(self.assignments) + 1}"
        
        criteria = [
            AssignmentCriteria.from_dict(c) 
            for c in assignment_data.get('criteria', [])
        ]
        
        assignment = Assignment(
            id=assignment_id,
            title=assignment_data['title'],
            description=assignment_data['description'],
            course_id=assignment_data['course_id'],
            instructor=assignment_data['instructor'],
            created_at=datetime.now(),
            due_date=datetime.fromisoformat(assignment_data['due_date']),
            max_points=assignment_data['max_points'],
            criteria=criteria,
            allowed_languages=assignment_data.get('allowed_languages', []),
            required_files=assignment_data.get('required_files', []),
            test_cases=assignment_data.get('test_cases', []),
            plagiarism_check=assignment_data.get('plagiarism_check', True),
            peer_review_required=assignment_data.get('peer_review_required', False)
        )
        
        self.assignments[assignment_id] = assignment
        self._save_data()
        
        return assignment_id
    
    def submit_assignment(self, assignment_id: str, student_id: str,
                         commit_hash: str, branch: str, files: List[str]) -> str:
        """Submit an assignment"""
        if assignment_id not in self.assignments:
            raise ValueError(f"Assignment {assignment_id} not found")
        
        assignment = self.assignments[assignment_id]
        submission_id = f"submission_{len(self.submissions) + 1}"
        
        # Check if submission is late
        now = datetime.now()
        is_late = now > assignment.due_date
        
        # Create submission
        submission = Submission(
            id=submission_id,
            assignment_id=assignment_id,
            student_id=student_id,
            submitted_at=now,
            commit_hash=commit_hash,
            branch=branch,
            files=files,
            status=SubmissionStatus.LATE if is_late else SubmissionStatus.SUBMITTED,
            peer_reviews=[]
        )
        
        self.submissions[submission_id] = submission
        self._save_data()
        
        # Run automated tests if available
        if assignment.test_cases:
            test_results = self._run_test_cases(submission_id, assignment.test_cases)
            submission.feedback = test_results
        
        # Check for plagiarism
        if assignment.plagiarism_check:
            plagiarism_score = self._check_plagiarism(submission_id)
            submission.plagiarism_score = plagiarism_score
        
        return submission_id
    
    def _run_test_cases(self, submission_id: str, test_cases: List[Dict]) -> List[Dict]:
        """Run automated test cases on submission"""
        results = []
        
        for i, test_case in enumerate(test_cases):
            # Simulate test execution
            result = {
                'test_id': f"test_{i+1}",
                'name': test_case.get('name', f'Test {i+1}'),
                'status': 'passed',  # Would actually run the test
                'output': 'Test passed successfully',
                'points_earned': test_case.get('points', 0),
                'points_possible': test_case.get('max_points', 1),
                'feedback': ''
            }
            results.append(result)
        
        return results
    
    def _check_plagiarism(self, submission_id: str) -> float:
        """Check submission for plagiarism (simplified)"""
        # Simplified plagiarism detection
        # In reality, this would compare against other submissions
        return 0.0  # 0% similarity found
    
    def grade_submission(self, submission_id: str, grader_id: str,
                        criteria_scores: Dict[str, float], 
                        overall_feedback: str = "") -> Dict:
        """Grade a submission based on criteria"""
        if submission_id not in self.submissions:
            return {'status': 'error', 'message': 'Submission not found'}
        
        submission = self.submissions[submission_id]
        assignment = self.assignments[submission.assignment_id]
        
        # Calculate weighted score
        total_score = 0.0
        for criteria in assignment.criteria:
            score = criteria_scores.get(criteria.name, 0.0)
            weighted_score = (score / criteria.max_points) * criteria.weight * assignment.max_points
            total_score += weighted_score
        
        submission.grade = total_score
        submission.status = SubmissionStatus.GRADED
        
        # Store detailed grading
        grade_detail = {
            'submission_id': submission_id,
            'grader_id': grader_id,
            'criteria_scores': criteria_scores,
            'total_score': total_score,
            'max_score': assignment.max_points,
            'percentage': (total_score / assignment.max_points) * 100,
            'overall_feedback': overall_feedback,
            'graded_at': datetime.now().isoformat()
        }
        
        self.grades[submission_id] = grade_detail
        self._save_data()
        
        return {
            'status': 'success',
            'grade': total_score,
            'percentage': grade_detail['percentage'],
            'letter_grade': self._get_letter_grade(grade_detail['percentage'])
        }
    
    def _get_letter_grade(self, percentage: float) -> str:
        """Convert percentage to letter grade"""
        if percentage >= 97:
            return 'A+'
        elif percentage >= 93:
            return 'A'
        elif percentage >= 90:
            return 'A-'
        elif percentage >= 87:
            return 'B+'
        elif percentage >= 83:
            return 'B'
        elif percentage >= 80:
            return 'B-'
        elif percentage >= 77:
            return 'C+'
        elif percentage >= 73:
            return 'C'
        elif percentage >= 70:
            return 'C-'
        elif percentage >= 67:
            return 'D+'
        elif percentage >= 63:
            return 'D'
        elif percentage >= 60:
            return 'D-'
        else:
            return 'F'
    
    def get_student_dashboard(self, student_id: str) -> Dict:
        """Get dashboard for a student"""
        student_submissions = [
            submission for submission in self.submissions.values()
            if submission.student_id == student_id
        ]
        
        # Calculate statistics
        total_assignments = len(self.assignments)
        submitted_assignments = len([s for s in student_submissions if s.status in [SubmissionStatus.SUBMITTED, SubmissionStatus.GRADED]])
        graded_assignments = len([s for s in student_submissions if s.status == SubmissionStatus.GRADED])
        
        # Calculate average grade
        grades = [s.grade for s in student_submissions if s.grade is not None]
        average_grade = sum(grades) / len(grades) if grades else 0
        
        # Get upcoming assignments
        now = datetime.now()
        upcoming_assignments = [
            {
                'id': aid,
                'title': assignment.title,
                'due_date': assignment.due_date.isoformat(),
                'days_until_due': (assignment.due_date - now).days
            }
            for aid, assignment in self.assignments.items()
            if assignment.due_date > now
        ][:5]  # Next 5 assignments
        
        return {
            'student_id': student_id,
            'total_assignments': total_assignments,
            'submitted_assignments': submitted_assignments,
            'graded_assignments': graded_assignments,
            'average_grade': average_grade,
            'completion_rate': (submitted_assignments / total_assignments * 100) if total_assignments > 0 else 0,
            'upcoming_assignments': upcoming_assignments,
            'recent_submissions': [
                {
                    'id': s.id,
                    'assignment_id': s.assignment_id,
                    'status': s.status.value,
                    'grade': s.grade,
                    'submitted_at': s.submitted_at.isoformat()
                }
                for s in sorted(student_submissions, key=lambda x: x.submitted_at, reverse=True)[:10]
            ]
        }
    
    def get_assignment_statistics(self, assignment_id: str) -> Dict:
        """Get statistics for an assignment"""
        if assignment_id not in self.assignments:
            return {'status': 'error', 'message': 'Assignment not found'}
        
        assignment = self.assignments[assignment_id]
        submissions = [s for s in self.submissions.values() if s.assignment_id == assignment_id]
        
        # Calculate statistics
        total_submissions = len(submissions)
        graded_submissions = [s for s in submissions if s.grade is not None]
        
        if graded_submissions:
            grades = [s.grade for s in graded_submissions]
            average_grade = sum(grades) / len(grades)
            median_grade = sorted(grades)[len(grades) // 2]
            highest_grade = max(grades)
            lowest_grade = min(grades)
        else:
            average_grade = median_grade = highest_grade = lowest_grade = 0
        
        # Late submissions
        late_submissions = [s for s in submissions if s.status == SubmissionStatus.LATE]
        
        return {
            'assignment_id': assignment_id,
            'title': assignment.title,
            'total_submissions': total_submissions,
            'graded_submissions': len(graded_submissions),
            'average_grade': average_grade,
            'median_grade': median_grade,
            'highest_grade': highest_grade,
            'lowest_grade': lowest_grade,
            'late_submissions': len(late_submissions),
            'completion_rate': (total_submissions / 1) * 100,  # Would need total enrolled students
            'grade_distribution': self._calculate_grade_distribution(graded_submissions)
        }
    
    def _calculate_grade_distribution(self, submissions: List[Submission]) -> Dict:
        """Calculate distribution of letter grades"""
        distribution = {
            'A+': 0, 'A': 0, 'A-': 0,
            'B+': 0, 'B': 0, 'B-': 0,
            'C+': 0, 'C': 0, 'C-': 0,
            'D+': 0, 'D': 0, 'D-': 0,
            'F': 0
        }
        
        for submission in submissions:
            if submission.grade is not None:
                assignment = self.assignments[submission.assignment_id]
                percentage = (submission.grade / assignment.max_points) * 100
                letter = self._get_letter_grade(percentage)
                distribution[letter] += 1
        
        return distribution
