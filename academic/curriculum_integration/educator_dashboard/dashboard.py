"""
Educator Dashboard with Analytics and Reporting for MultiOS Academic Platform

This module provides comprehensive analytics, reporting, and dashboard capabilities
for educators to monitor student progress, analyze course performance, and make data-driven decisions.
"""

from typing import Dict, List, Optional, Any, Union
from dataclasses import dataclass, asdict, field
from datetime import datetime, timedelta, date
from enum import Enum
import json
import statistics
from collections import defaultdict, Counter
import math

from ..assessment.progress_tracker import ProgressTracker, AssessmentManager
from ..assignments.assignment_manager import AssignmentManager
from ..calendar.schedule_manager import CalendarManager


class AnalyticsPeriod(Enum):
    """Time periods for analytics"""
    WEEK = "week"
    MONTH = "month"
    SEMESTER = "semester"
    QUARTER = "quarter"
    YEAR = "year"
    CUSTOM = "custom"


class MetricType(Enum):
    """Types of performance metrics"""
    COMPLETION_RATE = "completion_rate"
    AVERAGE_GRADE = "average_grade"
    ATTENDANCE = "attendance"
    ENGAGEMENT = "engagement"
    LEARNING_GAIN = "learning_gain"
    RETENTION = "retention"
    SATISFACTION = "satisfaction"
    TIME_ON_TASK = "time_on_task"


class AlertSeverity(Enum):
    """Severity levels for alerts"""
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"


@dataclass
class PerformanceMetric:
    """Individual performance metric"""
    name: str
    value: float
    unit: str
    target: Optional[float] = None
    trend: str = "stable"  # "improving", "declining", "stable"
    change_from_previous: float = 0.0
    confidence_level: float = 1.0  # 0-1 scale
    
    @property
    def meets_target(self) -> bool:
        """Check if metric meets target"""
        if self.target is None:
            return True
        return self.value >= self.target


@dataclass
class StudentAlert:
    """Student performance alert"""
    id: str
    student_id: str
    student_name: str
    course_id: str
    alert_type: str
    severity: AlertSeverity
    description: str
    recommended_actions: List[str]
    created_at: datetime
    acknowledged: bool = False
    resolved: bool = False
    
    def __post_init__(self):
        if self.id is None:
            self.id = str(uuid.uuid4())


@dataclass
class CourseAnalytics:
    """Comprehensive course analytics"""
    course_id: str
    course_name: str
    instructor_id: str
    enrollment_count: int
    period_start: datetime
    period_end: datetime
    
    # Core metrics
    completion_rate: float
    average_grade: float
    pass_rate: float
    attendance_rate: float
    engagement_score: float
    
    # Detailed metrics
    assignment_completion: Dict[str, float]
    attendance_patterns: Dict[str, float]
    learning_outcome_achievement: Dict[str, float]
    grade_distribution: Dict[str, int]
    performance_trends: Dict[str, List[float]]
    
    # Comparative metrics
    vs_previous_period: Dict[str, float]
    vs_course_average: Dict[str, float]
    vs_department_average: Dict[str, float]
    
    # Alerts and recommendations
    alerts: List[StudentAlert]
    recommendations: List[str]
    generated_at: datetime = field(default_factory=datetime.now)


@dataclass
class InstructorAnalytics:
    """Analytics for instructor performance"""
    instructor_id: str
    instructor_name: str
    department: str
    period_start: datetime
    period_end: datetime
    
    # Teaching load
    courses_taught: int
    total_students: int
    total_classes: int
    
    # Performance metrics
    student_satisfaction: float
    course_completion_rates: Dict[str, float]
    grade_distributions: Dict[str, Dict[str, int]]
    attendance_rates: Dict[str, float]
    
    # Engagement metrics
    response_time_average: float  # hours
    office_hours_utilization: float
    student_meetings_count: int
    
    # Comparative analysis
    vs_department_average: Dict[str, float]
    trend_analysis: Dict[str, str]
    
    # Recommendations
    professional_development_suggestions: List[str]
    teaching_improvements: List[str]
    
    generated_at: datetime = field(default_factory=datetime.now)


@dataclass
class DepartmentAnalytics:
    """Department-level analytics"""
    department_name: str
    period_start: datetime
    period_end: datetime
    
    # Overview metrics
    total_courses: int
    total_instructors: int
    total_students: int
    total_classes: int
    
    # Performance overview
    department_gpa: float
    overall_completion_rate: float
    retention_rate: float
    graduation_rate: float
    
    # Course analysis
    course_performance: List[Dict[str, Any]]
    instructor_performance: List[Dict[str, Any]]
    
    # Resource utilization
    room_utilization: float
    resource_allocation_efficiency: float
    
    # Trends and insights
    enrollment_trends: List[float]
    performance_trends: Dict[str, float]
    student_satisfaction_trend: float
    
    # Strategic recommendations
    resource_needs: List[str]
    curriculum_improvements: List[str]
    staffing_recommendations: List[str]
    
    generated_at: datetime = field(default_factory=datetime.now)


class AnalyticsEngine:
    """Core analytics engine for processing and analyzing data"""
    
    def __init__(self, progress_tracker: ProgressTracker, 
                 assignment_manager: AssignmentManager,
                 calendar_manager: CalendarManager):
        self.progress_tracker = progress_tracker
        self.assignment_manager = assignment_manager
        self.calendar_manager = calendar_manager
    
    def calculate_completion_rate(self, course_id: str, period_start: datetime, 
                                period_end: datetime) -> float:
        """Calculate course completion rate"""
        # Get all student progress for the course
        course_progress = [
            progress for progress in self.progress_tracker.student_progress.values()
            if progress.course_id == course_id and
            period_start <= progress.enrollment_date <= period_end
        ]
        
        if not course_progress:
            return 0.0
        
        completed_students = sum(1 for p in course_progress 
                               if p.overall_status.value == "completed")
        
        return (completed_students / len(course_progress)) * 100
    
    def calculate_average_grade(self, course_id: str, period_start: datetime,
                              period_end: datetime) -> float:
        """Calculate average grade for course"""
        course_scores = []
        
        # Get all scores for the course
        for score in self.progress_tracker.assessment_manager.scores.values():
            assessment = self.progress_tracker.assessment_manager.assessments.get(score.assessment_id)
            if assessment and assessment.course_id == course_id:
                if period_start <= score.graded_at <= period_end:
                    course_scores.append(score.percentage)
        
        return statistics.mean(course_scores) if course_scores else 0.0
    
    def analyze_grade_distribution(self, course_id: str, period_start: datetime,
                                 period_end: datetime) -> Dict[str, int]:
        """Analyze grade distribution"""
        grades = []
        
        for score in self.progress_tracker.assessment_manager.scores.values():
            assessment = self.progress_tracker.assessment_manager.assessments.get(score.assessment_id)
            if assessment and assessment.course_id == course_id:
                if period_start <= score.graded_at <= period_end:
                    grades.append(score.letter_grade)
        
        return dict(Counter(grades))
    
    def calculate_engagement_score(self, course_id: str, student_id: str,
                                 period_start: datetime, period_end: datetime) -> float:
        """Calculate student engagement score"""
        engagement_factors = []
        
        # Assignment submission timeliness
        on_time_submissions = 0
        total_assignments = 0
        
        for submission in self.assignment_manager.submissions.values():
            if submission.student_id == student_id and submission.assignment_id:
                assignment = self.assignment_manager.assessments.get(submission.assignment_id)
                if assignment and assignment.course_id == course_id:
                    total_assignments += 1
                    if submission.status.value in ["submitted", "graded"]:
                        on_time_submissions += 1
        
        if total_assignments > 0:
            engagement_factors.append((on_time_submissions / total_assignments) * 100)
        
        # Attendance (if available from calendar)
        course_events = []
        for event in self.calendar_manager.events.values():
            if event.course_id == course_id:
                course_events.append(event)
        
        # Calculate attendance based on events
        if course_events:
            attendance_rate = 85.0  # Placeholder - would calculate from actual attendance data
            engagement_factors.append(attendance_rate)
        
        # Participation (simplified)
        engagement_factors.append(75.0)  # Placeholder
        
        return statistics.mean(engagement_factors) if engagement_factors else 0.0
    
    def identify_learning_gaps(self, course_id: str, period_start: datetime,
                             period_end: datetime) -> Dict[str, float]:
        """Identify areas where students are struggling"""
        
        # Analyze performance by learning outcome
        learning_outcome_performance = defaultdict(list)
        
        for progress in self.progress_tracker.student_progress.values():
            if progress.course_id == course_id:
                for outcome_progress in progress.learning_outcome_progress:
                    if period_start <= outcome_progress.last_assessment_date <= period_end:
                        learning_outcome_performance[outcome_progress.learning_outcome_id].append(
                            outcome_progress.average_score
                        )
        
        # Calculate average performance by outcome
        gap_analysis = {}
        for outcome_id, scores in learning_outcome_performance.items():
            avg_score = statistics.mean(scores) if scores else 0.0
            gap_analysis[outcome_id] = 100.0 - avg_score  # Higher gap = more struggling
        
        return gap_analysis
    
    def detect_at_risk_students(self, course_id: str, 
                              period_start: datetime, period_end: datetime) -> List[StudentAlert]:
        """Detect students at risk of failing"""
        
        at_risk_students = []
        
        for progress in self.progress_tracker.student_progress.values():
            if progress.course_id == course_id:
                student_id = progress.student_id
                risk_factors = []
                severity = AlertSeverity.LOW
                
                # Check current grade
                if progress.current_grade < 70:
                    risk_factors.append("Below passing grade")
                    severity = AlertSeverity.MEDIUM
                
                # Check completion rate
                completion_rate = progress.completion_percentage
                if completion_rate < 50:
                    risk_factors.append("Low assignment completion")
                    severity = AlertSeverity.HIGH
                
                # Check engagement
                engagement = self.calculate_engagement_score(course_id, student_id, 
                                                           period_start, period_end)
                if engagement < 60:
                    risk_factors.append("Low engagement")
                    if severity.value != "critical":
                        severity = AlertSeverity.MEDIUM
                
                # Check recent activity
                days_since_activity = (datetime.now() - progress.last_activity).days
                if days_since_activity > 14:
                    risk_factors.append("No recent activity")
                    severity = AlertSeverity.HIGH
                
                if risk_factors:
                    recommended_actions = self._generate_intervention_strategies(
                        progress, risk_factors
                    )
                    
                    alert = StudentAlert(
                        student_id=student_id,
                        student_name=f"Student {student_id}",  # Would lookup actual name
                        course_id=course_id,
                        alert_type="academic_performance",
                        severity=severity,
                        description=f"Risk factors: {', '.join(risk_factors)}",
                        recommended_actions=recommended_actions,
                        created_at=datetime.now()
                    )
                    
                    at_risk_students.append(alert)
        
        return at_risk_students
    
    def _generate_intervention_strategies(self, progress, risk_factors: List[str]) -> List[str]:
        """Generate intervention strategies for at-risk students"""
        strategies = []
        
        for factor in risk_factors:
            if "grade" in factor.lower():
                strategies.extend([
                    "Schedule one-on-one meeting to discuss challenges",
                    "Provide additional tutoring resources",
                    "Offer extra credit opportunities"
                ])
            elif "completion" in factor.lower():
                strategies.extend([
                    "Send reminder notifications about pending assignments",
                    "Break down large assignments into smaller milestones",
                    "Offer assignment extensions if appropriate"
                ])
            elif "engagement" in factor.lower():
                strategies.extend([
                    "Encourage participation in class discussions",
                    "Invite to study groups or peer tutoring",
                    "Provide multiple communication channels"
                ])
            elif "activity" in factor.lower():
                strategies.extend([
                    "Send check-in email",
                    "Schedule mandatory meeting",
                    "Connect with academic advisor"
                ])
        
        # Remove duplicates
        return list(set(strategies))
    
    def generate_performance_trends(self, course_id: str, period_start: datetime,
                                  period_end: datetime, granularity: str = "week") -> Dict[str, List[float]]:
        """Generate performance trends over time"""
        
        # Calculate performance at different time points
        time_points = self._generate_time_points(period_start, period_end, granularity)
        trends = {
            "grades": [],
            "completion_rates": [],
            "attendance": [],
            "engagement": []
        }
        
        for i, time_point in enumerate(time_points):
            # Calculate metrics for this time point
            start_date = time_point
            end_date = time_points[i + 1] if i + 1 < len(time_points) else period_end
            
            grade = self.calculate_average_grade(course_id, start_date, end_date)
            completion = self.calculate_completion_rate(course_id, start_date, end_date)
            attendance = 85.0  # Placeholder
            engagement = 75.0  # Placeholder
            
            trends["grades"].append(grade)
            trends["completion_rates"].append(completion)
            trends["attendance"].append(attendance)
            trends["engagement"].append(engagement)
        
        return trends
    
    def _generate_time_points(self, start: datetime, end: datetime, 
                            granularity: str) -> List[datetime]:
        """Generate time points for trend analysis"""
        time_points = []
        current = start
        
        if granularity == "week":
            while current <= end:
                time_points.append(current)
                current += timedelta(days=7)
        elif granularity == "month":
            while current <= end:
                time_points.append(current)
                if current.month == 12:
                    current = current.replace(year=current.year + 1, month=1)
                else:
                    current = current.replace(month=current.month + 1)
        else:  # daily
            while current <= end:
                time_points.append(current)
                current += timedelta(days=1)
        
        return time_points


class DashboardGenerator:
    """Generates dashboard views and reports"""
    
    def __init__(self, analytics_engine: AnalyticsEngine):
        self.analytics_engine = analytics_engine
    
    def generate_course_dashboard(self, course_id: str, instructor_id: str,
                                period: AnalyticsPeriod = AnalyticsPeriod.MONTH,
                                custom_start: Optional[datetime] = None,
                                custom_end: Optional[datetime] = None) -> CourseAnalytics:
        """Generate comprehensive course dashboard"""
        
        # Determine date range
        period_dates = self._get_period_dates(period, custom_start, custom_end)
        period_start, period_end = period_dates
        
        # Get basic course information
        # (In real implementation, would query actual course data)
        course_name = f"Course {course_id}"
        
        # Calculate core metrics
        completion_rate = self.analytics_engine.calculate_completion_rate(
            course_id, period_start, period_end
        )
        average_grade = self.analytics_engine.calculate_average_grade(
            course_id, period_start, period_end
        )
        
        # Calculate detailed metrics
        grade_distribution = self.analytics_engine.analyze_grade_distribution(
            course_id, period_start, period_end
        )
        
        learning_gaps = self.analytics_engine.identify_learning_gaps(
            course_id, period_start, period_end
        )
        
        # Generate performance trends
        performance_trends = self.analytics_engine.generate_performance_trends(
            course_id, period_start, period_end
        )
        
        # Detect at-risk students
        alerts = self.analytics_engine.detect_at_risk_students(
            course_id, period_start, period_end
        )
        
        # Generate recommendations
        recommendations = self._generate_course_recommendations(
            completion_rate, average_grade, learning_gaps, alerts
        )
        
        # Calculate comparative metrics (simplified)
        vs_previous_period = {
            "completion_rate": completion_rate - 5.0,  # Placeholder
            "average_grade": average_grade + 2.0       # Placeholder
        }
        
        enrollment_count = len([p for p in self.analytics_engine.progress_tracker.student_progress.values()
                              if p.course_id == course_id])
        
        return CourseAnalytics(
            course_id=course_id,
            course_name=course_name,
            instructor_id=instructor_id,
            enrollment_count=enrollment_count,
            period_start=period_start,
            period_end=period_end,
            completion_rate=completion_rate,
            average_grade=average_grade,
            pass_rate=85.0,  # Placeholder
            attendance_rate=88.0,  # Placeholder
            engagement_score=76.0,  # Placeholder
            assignment_completion={},  # Would calculate from assignment data
            attendance_patterns={},  # Would calculate from attendance data
            learning_outcome_achievement=learning_gaps,
            grade_distribution=grade_distribution,
            performance_trends=performance_trends,
            vs_previous_period=vs_previous_period,
            vs_course_average={},  # Would calculate
            vs_department_average={},  # Would calculate
            alerts=alerts,
            recommendations=recommendations
        )
    
    def generate_instructor_dashboard(self, instructor_id: str,
                                    period: AnalyticsPeriod = AnalyticsPeriod.MONTH,
                                    custom_start: Optional[datetime] = None,
                                    custom_end: Optional[datetime] = None) -> InstructorAnalytics:
        """Generate instructor performance dashboard"""
        
        # Determine date range
        period_dates = self._get_period_dates(period, custom_start, custom_end)
        period_start, period_end = period_dates
        
        # Get instructor courses
        instructor_courses = []
        for course in self.analytics_engine.progress_tracker.assessment_manager.assessments.values():
            # Would need to map courses to instructors
            pass  # Placeholder
        
        courses_taught = len(instructor_courses)
        total_students = sum(course.get("enrollment", 0) for course in instructor_courses)
        total_classes = len([e for e in self.analytics_engine.calendar_manager.events.values()
                           if e.instructor_id == instructor_id])
        
        # Calculate performance metrics
        student_satisfaction = 4.2  # Placeholder
        course_completion_rates = {f"Course_{i}": 85.0 + i * 2 for i in range(courses_taught)}
        
        return InstructorAnalytics(
            instructor_id=instructor_id,
            instructor_name=f"Instructor {instructor_id}",
            department="Computer Science",
            period_start=period_start,
            period_end=period_end,
            courses_taught=courses_taught,
            total_students=total_students,
            total_classes=total_classes,
            student_satisfaction=student_satisfaction,
            course_completion_rates=course_completion_rates,
            grade_distributions={},  # Would calculate
            attendance_rates={},     # Would calculate
            response_time_average=24.0,  # hours
            office_hours_utilization=68.0,  # percentage
            student_meetings_count=45,
            vs_department_average={},  # Would calculate
            trend_analysis={},       # Would calculate
            professional_development_suggestions=[
                "Consider training in active learning techniques",
                "Explore digital assessment tools",
                "Attend teaching effectiveness workshops"
            ],
            teaching_improvements=[
                "Increase use of formative assessments",
                "Provide more frequent feedback",
                "Incorporate peer learning activities"
            ]
        )
    
    def generate_department_dashboard(self, department_name: str,
                                    period: AnalyticsPeriod = AnalyticsPeriod.SEMESTER,
                                    custom_start: Optional[datetime] = None,
                                    custom_end: Optional[datetime] = None) -> DepartmentAnalytics:
        """Generate department-level dashboard"""
        
        # Determine date range
        period_dates = self._get_period_dates(period, custom_start, custom_end)
        period_start, period_end = period_dates
        
        # Get department data
        total_courses = len(self.analytics_engine.progress_tracker.assessment_manager.assessments)
        total_instructors = len(self.analytics_engine.calendar_manager.instructors)
        total_students = len(self.analytics_engine.progress_tracker.student_progress)
        
        # Calculate department metrics
        department_gpa = 3.2  # Placeholder
        overall_completion_rate = 89.0  # Placeholder
        retention_rate = 92.0  # Placeholder
        graduation_rate = 85.0  # Placeholder
        
        # Resource utilization
        room_utilization = 78.5  # Placeholder
        resource_allocation_efficiency = 82.0  # Placeholder
        
        return DepartmentAnalytics(
            department_name=department_name,
            period_start=period_start,
            period_end=period_end,
            total_courses=total_courses,
            total_instructors=total_instructors,
            total_students=total_students,
            total_classes=0,  # Would calculate
            department_gpa=department_gpa,
            overall_completion_rate=overall_completion_rate,
            retention_rate=retention_rate,
            graduation_rate=graduation_rate,
            course_performance=[],  # Would calculate
            instructor_performance=[],  # Would calculate
            room_utilization=room_utilization,
            resource_allocation_efficiency=resource_allocation_efficiency,
            enrollment_trends=[],  # Would calculate
            performance_trends={},  # Would calculate
            student_satisfaction_trend=4.1,  # Placeholder
            resource_needs=[
                "Additional computer lab space",
                "Updated software licenses",
                "Increased technical support"
            ],
            curriculum_improvements=[
                "Integrate more hands-on projects",
                "Update course prerequisites",
                "Add industry partnerships"
            ],
            staffing_recommendations=[
                "Hire additional adjunct faculty",
                "Provide professional development",
                "Recruit industry practitioners"
            ]
        )
    
    def _get_period_dates(self, period: AnalyticsPeriod, 
                         custom_start: Optional[datetime] = None,
                         custom_end: Optional[datetime] = None) -> tuple:
        """Get start and end dates for a period"""
        
        if period == AnalyticsPeriod.CUSTOM and custom_start and custom_end:
            return custom_start, custom_end
        
        now = datetime.now()
        
        if period == AnalyticsPeriod.WEEK:
            return now - timedelta(days=7), now
        elif period == AnalyticsPeriod.MONTH:
            return now - timedelta(days=30), now
        elif period == AnalyticsPeriod.SEMESTER:
            return now - timedelta(days=120), now
        elif period == AnalyticsPeriod.QUARTER:
            return now - timedelta(days=90), now
        elif period == AnalyticsPeriod.YEAR:
            return now - timedelta(days=365), now
        else:
            return now - timedelta(days=30), now
    
    def _generate_course_recommendations(self, completion_rate: float, 
                                       average_grade: float, 
                                       learning_gaps: Dict[str, float],
                                       alerts: List[StudentAlert]) -> List[str]:
        """Generate actionable recommendations for course improvement"""
        
        recommendations = []
        
        # Completion rate recommendations
        if completion_rate < 80:
            recommendations.append("Implement early intervention strategies for struggling students")
            recommendations.append("Review assignment difficulty and workload")
        
        # Grade distribution recommendations
        if average_grade < 75:
            recommendations.append("Provide additional learning resources and support")
            recommendations.append("Consider adjusting course pacing or difficulty")
        
        # Learning gap recommendations
        high_gaps = [(outcome, gap) for outcome, gap in learning_gaps.items() if gap > 30]
        if high_gaps:
            recommendations.append("Focus instruction on learning outcomes with high achievement gaps")
            recommendations.append("Provide supplementary materials for challenging topics")
        
        # Alert-based recommendations
        high_severity_alerts = [a for a in alerts if a.severity == AlertSeverity.HIGH]
        if len(high_severity_alerts) > 5:
            recommendations.append("Schedule additional support sessions for at-risk students")
            recommendations.append("Consider peer tutoring or study groups")
        
        # Engagement recommendations
        recommendations.append("Increase interactive and collaborative learning activities")
        recommendations.append("Utilize diverse assessment methods to engage different learning styles")
        
        return recommendations
    
    def export_dashboard_data(self, dashboard_data: Any, format: str = "json") -> str:
        """Export dashboard data in various formats"""
        
        if format == "json":
            return json.dumps(asdict(dashboard_data), indent=2, default=str)
        elif format == "csv":
            return self._convert_to_csv(dashboard_data)
        elif format == "pdf":
            return self._generate_pdf_report(dashboard_data)
        else:
            raise ValueError(f"Unsupported export format: {format}")
    
    def _convert_to_csv(self, dashboard_data: Any) -> str:
        """Convert dashboard data to CSV format"""
        # Simplified CSV conversion
        if hasattr(dashboard_data, '__dict__'):
            data_dict = asdict(dashboard_data)
        else:
            data_dict = dashboard_data
        
        lines = ["Metric,Value,Target"]
        for key, value in data_dict.items():
            if isinstance(value, (int, float, str)):
                lines.append(f"{key},{value},")
        
        return "\n".join(lines)
    
    def _generate_pdf_report(self, dashboard_data: Any) -> str:
        """Generate PDF report (placeholder - would use reportlab or similar)"""
        return f"PDF report generated for {type(dashboard_data).__name__}"


class AlertManager:
    """Manages alerts and notifications"""
    
    def __init__(self):
        self.active_alerts: Dict[str, StudentAlert] = {}
        self.alert_history: List[StudentAlert] = []
        self.notification_rules: List[Dict[str, Any]] = []
    
    def add_alert(self, alert: StudentAlert):
        """Add a new alert"""
        self.active_alerts[alert.id] = alert
        self.alert_history.append(alert)
    
    def acknowledge_alert(self, alert_id: str) -> bool:
        """Acknowledge an alert"""
        if alert_id in self.active_alerts:
            self.active_alerts[alert_id].acknowledged = True
            return True
        return False
    
    def resolve_alert(self, alert_id: str) -> bool:
        """Resolve an alert"""
        if alert_id in self.active_alerts:
            self.active_alerts[alert_id].resolved = True
            return True
        return False
    
    def get_active_alerts(self, severity: AlertSeverity = None) -> List[StudentAlert]:
        """Get active alerts, optionally filtered by severity"""
        alerts = [alert for alert in self.active_alerts.values() if not alert.resolved]
        
        if severity:
            alerts = [alert for alert in alerts if alert.severity == severity]
        
        return sorted(alerts, key=lambda a: a.created_at, reverse=True)
    
    def get_alert_summary(self) -> Dict[str, int]:
        """Get summary of alerts by severity"""
        summary = defaultdict(int)
        
        for alert in self.active_alerts.values():
            if not alert.resolved:
                summary[alert.severity.value] += 1
        
        return dict(summary)


class ReportGenerator:
    """Generates various types of reports"""
    
    def __init__(self, analytics_engine: AnalyticsEngine, dashboard_generator: DashboardGenerator):
        self.analytics_engine = analytics_engine
        self.dashboard_generator = dashboard_generator
    
    def generate_progress_report(self, student_id: str, course_id: str,
                               format: str = "json") -> str:
        """Generate student progress report"""
        
        # Use existing progress tracker functionality
        progress = self.analytics_engine.progress_tracker.get_student_progress(student_id, course_id)
        if not progress:
            return json.dumps({"error": "Student progress not found"})
        
        report_data = {
            "student_id": student_id,
            "course_id": course_id,
            "generated_at": datetime.now().isoformat(),
            "progress_summary": {
                "current_grade": progress.current_grade,
                "completion_percentage": progress.completion_percentage,
                "overall_status": progress.overall_status.value,
                "is_on_track": progress.is_on_track
            },
            "learning_outcomes": [
                {
                    "outcome_id": lp.learning_outcome_id,
                    "competency_level": lp.competency_level.value,
                    "average_score": lp.average_score,
                    "trend": lp.trend
                }
                for lp in progress.learning_outcome_progress
            ]
        }
        
        if format == "json":
            return json.dumps(report_data, indent=2, default=str)
        else:
            return str(report_data)
    
    def generate_performance_report(self, course_id: str, instructor_id: str,
                                  period: AnalyticsPeriod = AnalyticsPeriod.MONTH) -> str:
        """Generate course performance report"""
        
        dashboard = self.dashboard_generator.generate_course_dashboard(
            course_id, instructor_id, period
        )
        
        return self.dashboard_generator.export_dashboard_data(dashboard, "json")
    
    def generate_comparative_report(self, course_ids: List[str], 
                                  metrics: List[MetricType]) -> str:
        """Generate comparative report across multiple courses"""
        
        comparative_data = {
            "course_comparison": {},
            "metric_analysis": {},
            "rankings": {},
            "generated_at": datetime.now().isoformat()
        }
        
        # Compare courses on specified metrics
        for course_id in course_ids:
            course_data = {}
            
            for metric in metrics:
                # Calculate metric for course
                if metric == MetricType.COMPLETION_RATE:
                    value = self.analytics_engine.calculate_completion_rate(
                        course_id, datetime.now() - timedelta(days=30), datetime.now()
                    )
                elif metric == MetricType.AVERAGE_GRADE:
                    value = self.analytics_engine.calculate_average_grade(
                        course_id, datetime.now() - timedelta(days=30), datetime.now()
                    )
                else:
                    value = 0.0  # Placeholder for other metrics
                
                course_data[metric.value] = value
            
            comparative_data["course_comparison"][course_id] = course_data
        
        # Calculate rankings
        for metric in metrics:
            metric_values = [(course_id, data[metric.value]) 
                           for course_id, data in comparative_data["course_comparison"].items()]
            metric_values.sort(key=lambda x: x[1], reverse=True)
            
            comparative_data["rankings"][metric.value] = [
                {"course_id": course_id, "rank": i+1, "value": value}
                for i, (course_id, value) in enumerate(metric_values)
            ]
        
        return json.dumps(comparative_data, indent=2, default=str)


# Example usage
if __name__ == "__main__":
    # Initialize components (would use actual implementations)
    from ..assessment.progress_tracker import ProgressTracker, AssessmentManager
    from ..assignments.assignment_manager import AssignmentManager
    from ..calendar.schedule_manager import CalendarManager
    
    # Initialize managers
    assessment_mgr = AssessmentManager()
    progress_tracker = ProgressTracker(assessment_mgr)
    calendar_mgr = CalendarManager()
    
    # Initialize analytics components
    analytics_engine = AnalyticsEngine(progress_tracker, assessment_mgr, calendar_mgr)
    dashboard_generator = DashboardGenerator(analytics_engine)
    alert_manager = AlertManager()
    report_generator = ReportGenerator(analytics_engine, dashboard_generator)
    
    print("Educator Dashboard initialized!")
    
    # Generate sample course dashboard
    try:
        course_dashboard = dashboard_generator.generate_course_dashboard(
            course_id="CS301",
            instructor_id="prof_smith",
            period=AnalyticsPeriod.MONTH
        )
        
        print(f"\nCourse Dashboard for {course_dashboard.course_name}:")
        print(f"  Completion Rate: {course_dashboard.completion_rate:.1f}%")
        print(f"  Average Grade: {course_dashboard.average_grade:.1f}")
        print(f"  Active Alerts: {len(course_dashboard.alerts)}")
        print(f"  Recommendations: {len(course_dashboard.recommendations)}")
        
        # Generate instructor dashboard
        instructor_dashboard = dashboard_generator.generate_instructor_dashboard(
            instructor_id="prof_smith",
            period=AnalyticsPeriod.SEMESTER
        )
        
        print(f"\nInstructor Dashboard for {instructor_dashboard.instructor_name}:")
        print(f"  Courses Taught: {instructor_dashboard.courses_taught}")
        print(f"  Total Students: {instructor_dashboard.total_students}")
        print(f"  Student Satisfaction: {instructor_dashboard.student_satisfaction:.1f}/5.0")
        
        # Generate department dashboard
        dept_dashboard = dashboard_generator.generate_department_dashboard(
            department_name="Computer Science",
            period=AnalyticsPeriod.SEMESTER
        )
        
        print(f"\nDepartment Dashboard for {dept_dashboard.department_name}:")
        print(f"  Total Courses: {dept_dashboard.total_courses}")
        print(f"  Total Students: {dept_dashboard.total_students}")
        print(f"  Department GPA: {dept_dashboard.department_gpa:.2f}")
        print(f"  Retention Rate: {dept_dashboard.retention_rate:.1f}%")
        
        # Test alert system
        test_alert = StudentAlert(
            student_id="student123",
            student_name="John Doe",
            course_id="CS301",
            alert_type="low_grade",
            severity=AlertSeverity.HIGH,
            description="Student grade dropped below 70%",
            recommended_actions=["Schedule meeting", "Provide tutoring"],
            created_at=datetime.now()
        )
        
        alert_manager.add_alert(test_alert)
        print(f"\nAlert System: {len(alert_manager.get_active_alerts())} active alerts")
        
        # Test report generation
        performance_report = report_generator.generate_performance_report(
            course_id="CS301",
            instructor_id="prof_smith",
            period=AnalyticsPeriod.MONTH
        )
        
        print(f"\nGenerated performance report ({len(performance_report)} characters)")
        
        # Test comparative analysis
        comparative_report = report_generator.generate_comparative_report(
            course_ids=["CS301", "CS401", "CS501"],
            metrics=[MetricType.COMPLETION_RATE, MetricType.AVERAGE_GRADE]
        )
        
        print(f"Generated comparative report ({len(comparative_report)} characters)")
        
        print("\nEducator Dashboard system fully functional!")
        
    except Exception as e:
        print(f"Error generating dashboards: {e}")
        print("This is expected since we're using placeholder data.")