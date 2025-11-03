"""
Student Progress Tracking and Assessment System for MultiOS Academic Platform

This module provides comprehensive tools for tracking student progress, managing assessments,
and generating analytics and reports.
"""

from typing import Dict, List, Optional, Any, Union
from dataclasses import dataclass, field, asdict
from datetime import datetime, timedelta
from enum import Enum
import json
import statistics
import uuid
from collections import defaultdict, Counter


class AssessmentType(Enum):
    """Types of assessments"""
    QUIZ = "quiz"
    ASSIGNMENT = "assignment"
    EXAM = "exam"
    PROJECT = "project"
    PARTICIPATION = "participation"
    PORTFOLIO = "portfolio"
    PEER_REVIEW = "peer_review"
    SELF_ASSESSMENT = "self_assessment"


class GradingScale(Enum):
    """Grading scales"""
    LETTER = "letter"  # A, B, C, D, F
    PERCENTAGE = "percentage"  # 0-100
    POINTS = "points"  # Raw points
    RUBRIC = "rubric"  # Rubric-based scoring


class ProgressStatus(Enum):
    """Student progress statuses"""
    NOT_STARTED = "not_started"
    IN_PROGRESS = "in_progress"
    COMPLETED = "completed"
    OVERDUE = "overdue"
    FAILED = "failed"
    WITHDRAWN = "withdrawn"


class CompetencyLevel(Enum):
    """Student competency levels"""
    BEGINNER = "beginner"
    DEVELOPING = "developing"
    PROFICIENT = "proficient"
    ADVANCED = "advanced"
    EXPERT = "expert"


@dataclass
class AssessmentCriteria:
    """Individual assessment criteria"""
    id: str
    name: str
    description: str
    weight: float  # Weight in overall assessment (0-1)
    max_points: float
    rubric_levels: List[Dict[str, Any]] = field(default_factory=list)
    
    def __post_init__(self):
        if self.id is None:
            self.id = str(uuid.uuid4())


@dataclass
class Assessment:
    """Assessment definition"""
    id: str
    title: str
    description: str
    assessment_type: AssessmentType
    course_id: str
    unit_id: Optional[str]
    total_points: float
    weight: float  # Weight in course grade
    due_date: Optional[datetime]
    time_limit: Optional[timedelta] = None
    attempts_allowed: int = 1
    criteria: List[AssessmentCriteria] = field(default_factory=list)
    learning_outcomes: List[str] = field(default_factory=list)
    resources: List[Dict[str, Any]] = field(default_factory=list)
    instructions: str = ""
    rubric: Optional[Dict[str, Any]] = None
    created_at: datetime = field(default_factory=datetime.now)
    updated_at: datetime = field(default_factory=datetime.now)
    is_active: bool = True
    
    def calculate_max_score(self) -> float:
        """Calculate maximum possible score"""
        if self.criteria:
            return sum(criteria.max_points * criteria.weight for criteria in self.criteria)
        return self.total_points


@dataclass
class StudentSubmission:
    """Student submission for an assessment"""
    id: str
    assessment_id: str
    student_id: str
    submitted_at: Optional[datetime]
    content: Dict[str, Any]  # Submission content (files, text, etc.)
    attempt_number: int = 1
    time_spent: Optional[timedelta] = None
    status: ProgressStatus = ProgressStatus.NOT_STARTED
    created_at: datetime = field(default_factory=datetime.now)
    
    def __post_init__(self):
        if self.id is None:
            self.id = str(uuid.uuid4())


@dataclass
class AssessmentScore:
    """Score for a specific assessment"""
    id: str
    submission_id: str
    assessment_id: str
    student_id: str
    score: float
    max_score: float
    percentage: float
    letter_grade: Optional[str] = None
    criteria_scores: Dict[str, float] = field(default_factory=dict)
    feedback: str = ""
    graded_by: Optional[str] = None
    graded_at: Optional[datetime] = None
    attempt_number: int = 1
    created_at: datetime = field(default_factory=datetime.now)
    
    def __post_init__(self):
        if self.id is None:
            self.id = str(uuid.uuid4())
        if self.graded_at is None and self.score > 0:
            self.graded_at = datetime.now()
    
    @property
    def is_passing(self) -> bool:
        """Check if score represents passing grade"""
        return self.percentage >= 60.0  # Default passing threshold


@dataclass
class LearningOutcomeProgress:
    """Track student progress on specific learning outcomes"""
    student_id: str
    learning_outcome_id: str
    competency_level: CompetencyLevel
    evidence_count: int = 0
    last_assessment_date: Optional[datetime] = None
    average_score: float = 0.0
    trend: str = "stable"  # improving, declining, stable
    notes: str = ""
    
    def update_progress(self, new_score: float, assessment_date: datetime):
        """Update progress based on new assessment"""
        self.evidence_count += 1
        self.last_assessment_date = assessment_date
        
        # Update average score
        if self.average_score == 0.0:
            self.average_score = new_score
        else:
            self.average_score = (self.average_score + new_score) / 2
        
        # Update competency level based on average
        if self.average_score >= 90:
            self.competency_level = CompetencyLevel.EXPERT
        elif self.average_score >= 80:
            self.competency_level = CompetencyLevel.ADVANCED
        elif self.average_score >= 70:
            self.competency_level = CompetencyLevel.PROFICIENT
        elif self.average_score >= 60:
            self.competency_level = CompetencyLevel.DEVELOPING
        else:
            self.competency_level = CompetencyLevel.BEGINNER


@dataclass
class StudentProgress:
    """Overall student progress in a course"""
    student_id: str
    course_id: str
    enrollment_date: datetime
    current_grade: float = 0.0
    completed_assessments: int = 0
    total_assessments: int = 0
    attendance_rate: float = 100.0
    participation_score: float = 0.0
    overall_status: ProgressStatus = ProgressStatus.NOT_STARTED
    learning_outcome_progress: List[LearningOutcomeProgress] = field(default_factory=list)
    milestones: List[Dict[str, Any]] = field(default_factory=list)
    last_activity: Optional[datetime] = None
    created_at: datetime = field(default_factory=datetime.now)
    updated_at: datetime = field(default_factory=datetime.now)
    
    def __post_init__(self):
        if self.last_activity is None:
            self.last_activity = self.enrollment_date
    
    @property
    def completion_percentage(self) -> float:
        """Calculate course completion percentage"""
        if self.total_assessments == 0:
            return 0.0
        return (self.completed_assessments / self.total_assessments) * 100
    
    @property
    def is_on_track(self) -> bool:
        """Check if student is on track to pass"""
        return (self.completion_percentage >= 70 and 
                self.current_grade >= 60 and 
                self.overall_status in [ProgressStatus.NOT_STARTED, ProgressStatus.IN_PROGRESS])


class AssessmentManager:
    """Manages assessments and scoring"""
    
    def __init__(self):
        self.assessments: Dict[str, Assessment] = {}
        self.submissions: Dict[str, StudentSubmission] = {}
        self.scores: Dict[str, AssessmentScore] = {}
        self.rubrics: Dict[str, Dict[str, Any]] = {}
    
    def create_assessment(
        self,
        title: str,
        description: str,
        assessment_type: AssessmentType,
        course_id: str,
        total_points: float,
        weight: float,
        due_date: Optional[datetime] = None,
        unit_id: Optional[str] = None,
        criteria: Optional[List[Dict[str, Any]]] = None,
        learning_outcomes: Optional[List[str]] = None,
        instructions: str = ""
    ) -> Assessment:
        """Create a new assessment"""
        
        if criteria is None:
            criteria = []
        if learning_outcomes is None:
            learning_outcomes = []
        
        # Create criteria objects
        criteria_objects = []
        for crit_data in criteria:
            criteria_objects.append(AssessmentCriteria(
                name=crit_data["name"],
                description=crit_data["description"],
                weight=crit_data.get("weight", 1.0),
                max_points=crit_data["max_points"],
                rubric_levels=crit_data.get("rubric_levels", [])
            ))
        
        assessment = Assessment(
            id=str(uuid.uuid4()),
            title=title,
            description=description,
            assessment_type=assessment_type,
            course_id=course_id,
            unit_id=unit_id,
            total_points=total_points,
            weight=weight,
            due_date=due_date,
            criteria=criteria_objects,
            learning_outcomes=learning_outcomes,
            instructions=instructions
        )
        
        self.assessments[assessment.id] = assessment
        return assessment
    
    def create_submission(
        self,
        assessment_id: str,
        student_id: str,
        content: Dict[str, Any],
        attempt_number: int = 1
    ) -> StudentSubmission:
        """Create student submission"""
        submission = StudentSubmission(
            assessment_id=assessment_id,
            student_id=student_id,
            submitted_at=datetime.now(),
            content=content,
            attempt_number=attempt_number,
            status=ProgressStatus.COMPLETED
        )
        
        self.submissions[submission.id] = submission
        return submission
    
    def grade_submission(
        self,
        submission_id: str,
        assessment_id: str,
        student_id: str,
        score: float,
        max_score: Optional[float] = None,
        criteria_scores: Optional[Dict[str, float]] = None,
        feedback: str = "",
        graded_by: str = ""
    ) -> AssessmentScore:
        """Grade student submission"""
        
        if max_score is None:
            max_score = self.assessments[assessment_id].calculate_max_score()
        
        if criteria_scores is None:
            criteria_scores = {}
        
        # Calculate percentage
        percentage = (score / max_score * 100) if max_score > 0 else 0.0
        
        # Determine letter grade
        letter_grade = self._calculate_letter_grade(percentage)
        
        score_record = AssessmentScore(
            submission_id=submission_id,
            assessment_id=assessment_id,
            student_id=student_id,
            score=score,
            max_score=max_score,
            percentage=percentage,
            letter_grade=letter_grade,
            criteria_scores=criteria_scores,
            feedback=feedback,
            graded_by=graded_by
        )
        
        self.scores[score_record.id] = score_record
        return score_record
    
    def _calculate_letter_grade(self, percentage: float) -> str:
        """Calculate letter grade from percentage"""
        if percentage >= 97:
            return "A+"
        elif percentage >= 93:
            return "A"
        elif percentage >= 90:
            return "A-"
        elif percentage >= 87:
            return "B+"
        elif percentage >= 83:
            return "B"
        elif percentage >= 80:
            return "B-"
        elif percentage >= 77:
            return "C+"
        elif percentage >= 73:
            return "C"
        elif percentage >= 70:
            return "C-"
        elif percentage >= 67:
            return "D+"
        elif percentage >= 65:
            return "D"
        elif percentage >= 60:
            return "D-"
        else:
            return "F"
    
    def get_assessment_scores(self, assessment_id: str) -> List[AssessmentScore]:
        """Get all scores for a specific assessment"""
        return [score for score in self.scores.values() 
                if score.assessment_id == assessment_id]
    
    def get_student_scores(self, student_id: str, course_id: str = None) -> List[AssessmentScore]:
        """Get all scores for a student"""
        scores = [score for score in self.scores.values() 
                 if score.student_id == student_id]
        
        if course_id:
            # Filter by course
            course_scores = []
            for score in scores:
                assessment = self.assessments.get(score.assessment_id)
                if assessment and assessment.course_id == course_id:
                    course_scores.append(score)
            return course_scores
        
        return scores


class ProgressTracker:
    """Tracks student progress and generates analytics"""
    
    def __init__(self, assessment_manager: AssessmentManager):
        self.assessment_manager = assessment_manager
        self.student_progress: Dict[str, StudentProgress] = {}
    
    def create_student_progress(
        self,
        student_id: str,
        course_id: str,
        enrollment_date: datetime,
        total_assessments: int = 0
    ) -> StudentProgress:
        """Create student progress record"""
        progress = StudentProgress(
            student_id=student_id,
            course_id=course_id,
            enrollment_date=enrollment_date,
            total_assessments=total_assessments
        )
        
        key = f"{student_id}_{course_id}"
        self.student_progress[key] = progress
        return progress
    
    def update_student_progress(self, student_id: str, course_id: str, 
                               assessment_id: str, score: AssessmentScore):
        """Update student progress based on new score"""
        key = f"{student_id}_{course_id}"
        
        if key not in self.student_progress:
            self.create_student_progress(student_id, course_id, datetime.now())
        
        progress = self.student_progress[key]
        
        # Update basic metrics
        progress.completed_assessments += 1
        progress.last_activity = datetime.now()
        
        # Update current grade (weighted average)
        assessment = self.assessment_manager.assessments[assessment_id]
        current_total = progress.current_grade * (progress.completed_assessments - 1)
        new_total = current_total + (score.percentage * assessment.weight)
        progress.current_grade = new_total / progress.completed_assessments
        
        # Update status
        if progress.completed_assessments >= progress.total_assessments:
            progress.overall_status = ProgressStatus.COMPLETED
        elif score.percentage < 60:
            progress.overall_status = ProgressStatus.FAILED
        else:
            progress.overall_status = ProgressStatus.IN_PROGRESS
        
        # Update learning outcome progress
        self._update_learning_outcome_progress(student_id, assessment, score)
        
        progress.updated_at = datetime.now()
    
    def _update_learning_outcome_progress(self, student_id: str, 
                                        assessment: Assessment, score: AssessmentScore):
        """Update progress on learning outcomes"""
        for outcome_id in assessment.learning_outcomes:
            outcome_key = f"{student_id}_{outcome_id}"
            
            # Find existing progress or create new
            outcome_progress = None
            for progress in self.get_student_progress(student_id).learning_outcome_progress:
                if progress.learning_outcome_id == outcome_id:
                    outcome_progress = progress
                    break
            
            if outcome_progress:
                outcome_progress.update_progress(score.percentage, datetime.now())
            else:
                new_progress = LearningOutcomeProgress(
                    student_id=student_id,
                    learning_outcome_id=outcome_id,
                    competency_level=CompetencyLevel.BEGINNER
                )
                new_progress.update_progress(score.percentage, datetime.now())
                self.get_student_progress(student_id).learning_outcome_progress.append(new_progress)
    
    def get_student_progress(self, student_id: str, course_id: str = None) -> StudentProgress:
        """Get student progress record"""
        if course_id:
            key = f"{student_id}_{course_id}"
            return self.student_progress.get(key)
        
        # Return any progress record for student
        for progress in self.student_progress.values():
            if progress.student_id == student_id:
                return progress
        
        return None
    
    def get_course_progress_summary(self, course_id: str) -> Dict[str, Any]:
        """Get progress summary for entire course"""
        course_students = [
            progress for progress in self.student_progress.values()
            if progress.course_id == course_id
        ]
        
        if not course_students:
            return {
                "course_id": course_id,
                "total_students": 0,
                "summary": "No students enrolled"
            }
        
        # Calculate statistics
        grades = [progress.current_grade for progress in course_students]
        completion_rates = [progress.completion_percentage for progress in course_students]
        
        return {
            "course_id": course_id,
            "total_students": len(course_students),
            "average_grade": statistics.mean(grades) if grades else 0.0,
            "median_grade": statistics.median(grades) if grades else 0.0,
            "grade_distribution": self._calculate_grade_distribution(grades),
            "average_completion_rate": statistics.mean(completion_rates) if completion_rates else 0.0,
            "on_track_count": sum(1 for p in course_students if p.is_on_track),
            "completion_status": Counter(p.overall_status for p in course_students),
            "learning_outcome_achievement": self._analyze_learning_outcomes(course_id),
            "generated_at": datetime.now().isoformat()
        }
    
    def _calculate_grade_distribution(self, grades: List[float]) -> Dict[str, int]:
        """Calculate grade distribution"""
        distribution = {
            "A (90-100)": 0,
            "B (80-89)": 0,
            "C (70-79)": 0,
            "D (60-69)": 0,
            "F (0-59)": 0
        }
        
        for grade in grades:
            if grade >= 90:
                distribution["A (90-100)"] += 1
            elif grade >= 80:
                distribution["B (80-89)"] += 1
            elif grade >= 70:
                distribution["C (70-79)"] += 1
            elif grade >= 60:
                distribution["D (60-69)"] += 1
            else:
                distribution["F (0-59)"] += 1
        
        return distribution
    
    def _analyze_learning_outcomes(self, course_id: str) -> Dict[str, Any]:
        """Analyze learning outcome achievement"""
        # Get all assessments for course
        course_assessments = [
            assessment for assessment in self.assessment_manager.assessments.values()
            if assessment.course_id == course_id
        ]
        
        # Collect outcome data
        outcome_data = defaultdict(list)
        for assessment in course_assessments:
            scores = self.assessment_manager.get_assessment_scores(assessment.id)
            for outcome_id in assessment.learning_outcomes:
                outcome_scores = [score.percentage for score in scores]
                outcome_data[outcome_id].extend(outcome_scores)
        
        # Calculate statistics for each outcome
        outcome_stats = {}
        for outcome_id, scores in outcome_data.items():
            if scores:
                outcome_stats[outcome_id] = {
                    "average_score": statistics.mean(scores),
                    "median_score": statistics.median(scores),
                    "attempts": len(scores),
                    "proficiency_rate": len([s for s in scores if s >= 70]) / len(scores) * 100
                }
        
        return outcome_stats
    
    def generate_student_report(self, student_id: str, course_id: str) -> Dict[str, Any]:
        """Generate comprehensive student report"""
        progress = self.get_student_progress(student_id, course_id)
        if not progress:
            return {"error": "Student progress not found"}
        
        scores = self.assessment_manager.get_student_scores(student_id, course_id)
        
        # Calculate performance metrics
        recent_scores = [score for score in scores if 
                        score.graded_at and 
                        (datetime.now() - score.graded_at).days <= 30]
        
        performance_trend = "stable"
        if len(recent_scores) >= 2:
            if recent_scores[-1].percentage > recent_scores[0].percentage:
                performance_trend = "improving"
            elif recent_scores[-1].percentage < recent_scores[0].percentage:
                performance_trend = "declining"
        
        return {
            "student_id": student_id,
            "course_id": course_id,
            "progress_summary": {
                "current_grade": progress.current_grade,
                "completion_percentage": progress.completion_percentage,
                "completed_assessments": progress.completed_assessments,
                "total_assessments": progress.total_assessments,
                "overall_status": progress.overall_status.value,
                "enrollment_date": progress.enrollment_date.isoformat(),
                "last_activity": progress.last_activity.isoformat() if progress.last_activity else None,
                "is_on_track": progress.is_on_track
            },
            "performance_metrics": {
                "average_score": statistics.mean([s.percentage for s in scores]) if scores else 0.0,
                "highest_score": max([s.percentage for s in scores]) if scores else 0.0,
                "lowest_score": min([s.percentage for s in scores]) if scores else 0.0,
                "recent_performance_trend": performance_trend,
                "assessment_counts": Counter(s.assessment_id for s in scores)
            },
            "learning_outcome_progress": [
                {
                    "outcome_id": lp.learning_outcome_id,
                    "competency_level": lp.competency_level.value,
                    "average_score": lp.average_score,
                    "evidence_count": lp.evidence_count,
                    "trend": lp.trend
                } for lp in progress.learning_outcome_progress
            ],
            "recommendations": self._generate_recommendations(progress, scores),
            "generated_at": datetime.now().isoformat()
        }
    
    def _generate_recommendations(self, progress: StudentProgress, 
                                scores: List[AssessmentScore]) -> List[str]:
        """Generate personalized recommendations for student"""
        recommendations = []
        
        if progress.current_grade < 70:
            recommendations.append("Consider seeking additional help or tutoring")
        
        if progress.completion_percentage < 50:
            recommendations.append("Focus on completing pending assignments")
        
        if len([s for s in scores if s.percentage < 60]) > 0:
            recommendations.append("Review topics where you scored below 60%")
        
        if progress.participation_score < 70:
            recommendations.append("Increase class participation and engagement")
        
        if len(scores) > 0:
            recent_avg = statistics.mean([s.percentage for s in scores[-3:]])
            if recent_avg > 85:
                recommendations.append("Consider taking on additional challenge work")
        
        if not recommendations:
            recommendations.append("Keep up the good work!")
        
        return recommendations
    
    def export_progress_data(self, course_id: str, format: str = "csv") -> str:
        """Export progress data for external analysis"""
        course_progress = [
            progress for progress in self.student_progress.values()
            if progress.course_id == course_id
        ]
        
        if format == "csv":
            lines = ["Student ID,Current Grade,Completion %,Status,Last Activity"]
            for progress in course_progress:
                last_activity = progress.last_activity.isoformat() if progress.last_activity else ""
                line = f"{progress.student_id},{progress.current_grade:.2f},{progress.completion_percentage:.1f},{progress.overall_status.value},{last_activity}"
                lines.append(line)
            return "\n".join(lines)
        
        elif format == "json":
            return json.dumps([asdict(p) for p in course_progress], indent=2)
        
        return ""


# Example usage
if __name__ == "__main__":
    # Initialize assessment manager and progress tracker
    assessment_mgr = AssessmentManager()
    progress_tracker = ProgressTracker(assessment_mgr)
    
    # Create sample assessment
    assessment = assessment_mgr.create_assessment(
        title="Operating Systems Fundamentals Quiz",
        description="Test understanding of OS concepts",
        assessment_type=AssessmentType.QUIZ,
        course_id="CS301",
        total_points=100.0,
        weight=0.1,
        due_date=datetime.now() + timedelta(days=7)
    )
    
    # Create student submission
    submission = assessment_mgr.create_submission(
        assessment_id=assessment.id,
        student_id="student123",
        content={"answers": {"q1": "A", "q2": "B", "q3": "C"}}
    )
    
    # Grade submission
    score = assessment_mgr.grade_submission(
        submission_id=submission.id,
        assessment_id=assessment.id,
        student_id="student123",
        score=85.0,
        feedback="Good understanding of basic concepts"
    )
    
    # Update progress
    progress_tracker.update_student_progress(
        student_id="student123",
        course_id="CS301",
        assessment_id=assessment.id,
        score=score
    )
    
    # Generate report
    report = progress_tracker.generate_student_report("student123", "CS301")
    print("Student Report:")
    print(json.dumps(report, indent=2, default=str))
    
    # Get course summary
    course_summary = progress_tracker.get_course_progress_summary("CS301")
    print("\nCourse Summary:")
    print(json.dumps(course_summary, indent=2, default=str))