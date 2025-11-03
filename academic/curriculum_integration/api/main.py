"""
REST API Endpoints for MultiOS Academic Curriculum Integration Platform

This module provides comprehensive REST API endpoints for all academic curriculum
integration components including curriculum management, LMS integration, assessment,
assignments, scheduling, and analytics.
"""

from fastapi import FastAPI, HTTPException, Depends, Query, Body
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from pydantic import BaseModel, Field
from typing import Dict, List, Optional, Any, Union
from datetime import datetime, timedelta
import uvicorn
import json
import os

# Import our modules
from ..core.curriculum_manager import (
    CurriculumManager, LearningOutcome, LearningOutcomeType, 
    DifficultyLevel, CurriculumLevel, Course, CurriculumUnit
)
from ..lms_integration.lms_manager import (
    LMSIntegrationManager, LMSPlatform, LMSConnection
)
from ..assessment.progress_tracker import (
    ProgressTracker, AssessmentManager, Assessment, AssessmentScore
)
from ..assignments.assignment_manager import (
    AssignmentManager, Assignment, MultipleChoiceQuestion, CodeQuestion, EssayQuestion
)
from ..calendar.schedule_manager import (
    CalendarManager, Semester, AcademicEvent, EventType, ScheduleFrequency
)
from ..courses.os_templates import OperatingSystemsTemplateManager, OSTemplateType
from ..educator_dashboard.dashboard import (
    AnalyticsEngine, DashboardGenerator, AlertManager, 
    AnalyticsPeriod, MetricType, AlertSeverity
)


# FastAPI app initialization
app = FastAPI(
    title="MultiOS Academic Curriculum Integration API",
    description="Comprehensive API for academic curriculum management, assessment, and analytics",
    version="1.0.0",
    docs_url="/api/docs",
    redoc_url="/api/redoc"
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Global managers (in production, these would be properly initialized with dependencies)
curriculum_manager = CurriculumManager()
lms_manager = LMSIntegrationManager()
assessment_manager = AssessmentManager()
progress_tracker = ProgressTracker(assessment_manager)
assignment_manager = AssignmentManager()
calendar_manager = CalendarManager()
template_manager = OperatingSystemsTemplateManager(curriculum_manager)
analytics_engine = AnalyticsEngine(progress_tracker, assignment_manager, calendar_manager)
dashboard_generator = DashboardGenerator(analytics_engine)
alert_manager = AlertManager()


# Pydantic models for API requests/responses

class LearningOutcomeCreate(BaseModel):
    description: str
    outcome_type: LearningOutcomeType
    difficulty: DifficultyLevel
    standards_alignment: List[str] = []
    prerequisites: List[str] = []
    assessment_methods: List[str] = []


class UnitCreate(BaseModel):
    title: str
    description: str
    learning_outcomes: List[str]
    estimated_duration_days: int
    prerequisites: List[str] = []
    resources: List[Dict[str, Any]] = []
    activities: List[Dict[str, Any]] = []
    assessments: List[Dict[str, Any]] = []
    standards_alignment: List[str] = []


class CourseCreate(BaseModel):
    title: str
    description: str
    code: str
    credits: int
    level: CurriculumLevel
    unit_ids: List[str]
    prerequisites: List[str] = []
    learning_outcomes: List[str] = []
    resources: List[Dict[str, Any]] = []
    assessment_strategy: Dict[str, float] = {}
    standards_alignment: List[str] = []
    tags: List[str] = []
    instructor_requirements: List[str] = []
    facilities_required: List[str] = []


class LMSConnectionCreate(BaseModel):
    platform: LMSPlatform
    base_url: str
    api_key: Optional[str] = None
    client_id: Optional[str] = None
    client_secret: Optional[str] = None


class AssessmentCreate(BaseModel):
    title: str
    description: str
    assessment_type: str
    course_id: str
    total_points: float
    weight: float
    due_date: Optional[datetime] = None
    unit_id: Optional[str] = None
    instructions: str = ""


class AssignmentCreate(BaseModel):
    title: str
    description: str
    course_id: str
    questions: List[Dict[str, Any]]
    due_date: datetime
    total_points: Optional[float] = None
    grading_method: str = "MANUAL"
    late_penalty: float = 0.0
    max_attempts: int = 1


class SemesterCreate(BaseModel):
    name: str
    start_date: datetime
    end_date: datetime
    holidays: List[Dict[str, Any]] = []
    breaks: List[Dict[str, Any]] = []


class AnalyticsRequest(BaseModel):
    course_id: Optional[str] = None
    instructor_id: Optional[str] = None
    period: AnalyticsPeriod = AnalyticsPeriod.MONTH
    custom_start: Optional[datetime] = None
    custom_end: Optional[datetime] = None


# Core Curriculum Management Endpoints

@app.get("/api/curriculum/courses")
async def get_courses(
    level: Optional[CurriculumLevel] = None,
    limit: int = Query(50, ge=1, le=200),
    offset: int = Query(0, ge=0)
):
    """Get all courses with optional filtering"""
    try:
        courses = list(curriculum_manager.courses.values())
        
        if level:
            courses = [course for course in courses if course.level == level]
        
        # Apply pagination
        total = len(courses)
        courses = courses[offset:offset + limit]
        
        return {
            "courses": [course.__dict__ for course in courses],
            "total": total,
            "limit": limit,
            "offset": offset
        }
    
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/curriculum/courses")
async def create_course(course_data: CourseCreate):
    """Create a new course"""
    try:
        # Calculate duration from total points if not provided
        total_points = course_data.total_points if hasattr(course_data, 'total_points') else None
        
        course = curriculum_manager.create_course(
            title=course_data.title,
            description=course_data.description,
            code=course_data.code,
            credits=course_data.credits,
            level=course_data.level,
            unit_ids=course_data.unit_ids,
            prerequisites=course_data.prerequisites,
            learning_outcomes=course_data.learning_outcomes,
            resources=course_data.resources,
            assessment_strategy=course_data.assessment_strategy,
            standards_alignment=course_data.standards_alignment,
            tags=course_data.tags,
            instructor_requirements=course_data.instructor_requirements,
            facilities_required=course_data.facilities_required
        )
        
        return {"course": course.__dict__}
    
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


@app.get("/api/curriculum/courses/{course_id}")
async def get_course(course_id: str):
    """Get a specific course"""
    try:
        course = curriculum_manager.get_course(course_id)
        if not course:
            raise HTTPException(status_code=404, detail="Course not found")
        
        return {"course": course.__dict__}
    
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.put("/api/curriculum/courses/{course_id}")
async def update_course(course_id: str, course_data: CourseCreate):
    """Update an existing course"""
    try:
        updated_course = curriculum_manager.update_course(
            course_id,
            title=course_data.title,
            description=course_data.description,
            code=course_data.code,
            credits=course_data.credits,
            level=course_data.level,
            resources=course_data.resources,
            assessment_strategy=course_data.assessment_strategy,
            tags=course_data.tags,
            facilities_required=course_data.facilities_required
        )
        
        if not updated_course:
            raise HTTPException(status_code=404, detail="Course not found")
        
        return {"course": updated_course.__dict__}
    
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


@app.delete("/api/curriculum/courses/{course_id}")
async def delete_course(course_id: str):
    """Delete a course"""
    try:
        success = curriculum_manager.delete_course(course_id)
        if not success:
            raise HTTPException(status_code=404, detail="Course not found")
        
        return {"message": "Course deleted successfully"}
    
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/api/curriculum/learning-outcomes")
async def get_learning_outcomes():
    """Get all learning outcomes"""
    try:
        outcomes = list(curriculum_manager.learning_outcomes.values())
        return {"learning_outcomes": [outcome.__dict__ for outcome in outcomes]}
    
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/curriculum/learning-outcomes")
async def create_learning_outcome(outcome_data: LearningOutcomeCreate):
    """Create a new learning outcome"""
    try:
        outcome = curriculum_manager.create_learning_outcome(
            description=outcome_data.description,
            outcome_type=outcome_data.outcome_type,
            difficulty=outcome_data.difficulty,
            standards_alignment=outcome_data.standards_alignment,
            prerequisites=outcome_data.prerequisites,
            assessment_methods=outcome_data.assessment_methods
        )
        
        return {"learning_outcome": outcome.__dict__}
    
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


# LMS Integration Endpoints

@app.post("/api/lms/connections")
async def create_lms_connection(connection_data: LMSConnectionCreate):
    """Create LMS connection"""
    try:
        connection = LMSConnection(
            platform=connection_data.platform,
            base_url=connection_data.base_url,
            api_key=connection_data.api_key,
            client_id=connection_data.client_id,
            client_secret=connection_data.client_secret
        )
        
        # Generate connection ID
        connection_id = f"{connection_data.platform.value}_{len(lms_manager.connections)}"
        lms_manager.add_connection(connection_id, connection)
        
        return {
            "connection_id": connection_id,
            "message": "LMS connection created successfully"
        }
    
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


@app.get("/api/lms/status")
async def get_lms_status():
    """Get status of all LMS integrations"""
    try:
        status = lms_manager.get_integration_status()
        return status
    
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/lms/sync-course")
async def sync_course_to_lms(
    local_course_id: str,
    connection_id: str,
    lms_course_id: Optional[str] = None
):
    """Sync course to LMS"""
    try:
        # Get local course
        local_course = curriculum_manager.get_course(local_course_id)
        if not local_course:
            raise HTTPException(status_code=404, detail="Course not found")
        
        # Sync to LMS
        result = lms_manager.sync_course_to_lms(
            local_course.__dict__,
            connection_id,
            lms_course_id
        )
        
        return {"sync_result": result}
    
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


# Assessment and Progress Endpoints

@app.get("/api/assessment/courses/{course_id}/scores")
async def get_course_scores(course_id: str):
    """Get all scores for a course"""
    try:
        scores = assessment_manager.get_assessment_scores(course_id)
        return {"scores": [score.__dict__ for score in scores]}
    
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/api/assessment/students/{student_id}/progress")
async def get_student_progress(
    student_id: str,
    course_id: Optional[str] = None
):
    """Get student progress"""
    try:
        progress = progress_tracker.get_student_progress(student_id, course_id)
        if not progress:
            raise HTTPException(status_code=404, detail="Student progress not found")
        
        return {"progress": progress.__dict__}
    
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/api/assessment/courses/{course_id}/progress-summary")
async def get_course_progress_summary(course_id: str):
    """Get progress summary for a course"""
    try:
        summary = progress_tracker.get_course_progress_summary(course_id)
        return summary
    
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


# Assignment Management Endpoints

@app.get("/api/assignments/courses/{course_id}")
async def get_course_assignments(course_id: str):
    """Get all assignments for a course"""
    try:
        assignments = [assignment for assignment in assignment_manager.assignments.values()
                      if assignment.course_id == course_id]
        
        return {"assignments": [assignment.__dict__ for assignment in assignments]}
    
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/assignments")
async def create_assignment(assignment_data: AssignmentCreate):
    """Create a new assignment"""
    try:
        # Create questions from data
        questions = []
        for q_data in assignment_data.questions:
            if q_data["type"] == "multiple_choice":
                question = assignment_manager.create_multiple_choice_question(
                    question_text=q_data["question_text"],
                    options=q_data["options"],
                    correct_answer=q_data["correct_answer"],
                    points=q_data["points"]
                )
            elif q_data["type"] == "code":
                question = assignment_manager.create_code_question(
                    question_text=q_data["question_text"],
                    programming_language=q_data.get("programming_language", "python"),
                    points=q_data["points"]
                )
            elif q_data["type"] == "essay":
                question = assignment_manager.create_essay_question(
                    question_text=q_data["question_text"],
                    points=q_data["points"]
                )
            else:
                continue  # Skip unsupported types
            
            questions.append(question)
        
        assignment = assignment_manager.create_assignment(
            title=assignment_data.title,
            description=assignment_data.description,
            course_id=assignment_data.course_id,
            questions=questions,
            due_date=assignment_data.due_date,
            total_points=assignment_data.total_points,
            grading_method=assignment_data.grading_method,
            late_penalty=assignment_data.late_penalty,
            max_attempts=assignment_data.max_attempts
        )
        
        return {"assignment": assignment.__dict__}
    
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


@app.post("/api/assignments/{assignment_id}/submit")
async def submit_assignment(
    assignment_id: str,
    student_id: str,
    answers: Dict[str, Any],
    files: Optional[List[Dict[str, Any]]] = None
):
    """Submit assignment"""
    try:
        submission = assignment_manager.submit_assignment(
            assignment_id=assignment_id,
            student_id=student_id,
            answers=answers,
            files=files or []
        )
        
        return {"submission": submission.__dict__}
    
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


# Scheduling and Calendar Endpoints

@app.get("/api/schedule/semesters")
async def get_semesters():
    """Get all semesters"""
    try:
        semesters = list(calendar_manager.semesters.values())
        return {"semesters": [semester.__dict__ for semester in semesters]}
    
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/schedule/semesters")
async def create_semester(semester_data: SemesterCreate):
    """Create a new semester"""
    try:
        semester = calendar_manager.create_semester(
            name=semester_data.name,
            start_date=semester_data.start_date.date(),
            end_date=semester_data.end_date.date(),
            holidays=semester_data.holidays,
            breaks=semester_data.breaks
        )
        
        return {"semester": semester.__dict__}
    
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


@app.get("/api/schedule/courses/{course_id}/events")
async def get_course_events(course_id: str):
    """Get all events for a course"""
    try:
        events = [event for event in calendar_manager.events.values()
                 if event.course_id == course_id]
        
        return {"events": [event.__dict__ for event in events]}
    
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/schedule/courses/{course_id}/schedule")
async def schedule_course(
    course_id: str,
    semester_id: str,
    meeting_days: List[str],
    start_time: str,  # Format: "HH:MM"
    end_time: str,    # Format: "HH:MM"
    room_id: Optional[str] = None
):
    """Schedule a course"""
    try:
        from datetime import datetime, time as dt_time
        
        # Parse time strings
        start_dt = dt_time.fromisoformat(start_time)
        end_dt = dt_time.fromisoformat(end_time)
        
        events = calendar_manager.schedule_course(
            course_id=course_id,
            semester_id=semester_id,
            meeting_days=meeting_days,
            start_time=start_dt,
            end_time=end_dt,
            room_id=room_id
        )
        
        return {
            "scheduled_events": [event.__dict__ for event in events],
            "message": f"Scheduled {len(events)} class meetings"
        }
    
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


@app.get("/api/schedule/conflicts")
async def get_scheduling_conflicts():
    """Get all scheduling conflicts"""
    try:
        conflicts = calendar_manager.detect_conflicts()
        return {"conflicts": conflicts}
    
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/api/schedule/reports/{semester_id}")
async def get_schedule_report(semester_id: str):
    """Get schedule report for semester"""
    try:
        report = calendar_manager.generate_schedule_report(semester_id)
        return report
    
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


# Course Templates Endpoints

@app.get("/api/templates/os-courses")
async def get_os_templates():
    """Get all operating systems course templates"""
    try:
        templates = list(template_manager.templates.values())
        return {"templates": [template.__dict__ for template in templates]}
    
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/api/templates/os-courses/{template_id}")
async def get_os_template(template_id: str):
    """Get specific OS course template"""
    try:
        template = template_manager.get_template(template_id)
        if not template:
            raise HTTPException(status_code=404, detail="Template not found")
        
        return {"template": template.__dict__}
    
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/templates/os-courses/{template_id}/create-course")
async def create_course_from_template(
    template_id: str,
    instructor_id: str = "",
    semester: str = "",
    custom_code: str = ""
):
    """Create course from template"""
    try:
        course = template_manager.create_course_from_template(
            template_id=template_id,
            instructor_id=instructor_id,
            semester=semester,
            custom_code=custom_code
        )
        
        if not course:
            raise HTTPException(status_code=404, detail="Template not found")
        
        return {"course": course.__dict__}
    
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


@app.get("/api/templates/os-courses/suggestions/{template_id}")
async def get_prerequisite_suggestions(template_id: str):
    """Get prerequisite suggestions for template"""
    try:
        suggestions = template_manager.suggest_prerequisites(template_id)
        return suggestions
    
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


# Analytics and Dashboard Endpoints

@app.post("/api/analytics/course-dashboard")
async def generate_course_dashboard(analytics_request: AnalyticsRequest):
    """Generate course analytics dashboard"""
    try:
        if not analytics_request.course_id or not analytics_request.instructor_id:
            raise HTTPException(status_code=400, detail="course_id and instructor_id are required")
        
        dashboard = dashboard_generator.generate_course_dashboard(
            course_id=analytics_request.course_id,
            instructor_id=analytics_request.instructor_id,
            period=analytics_request.period,
            custom_start=analytics_request.custom_start,
            custom_end=analytics_request.custom_end
        )
        
        return dashboard.__dict__
    
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/analytics/instructor-dashboard")
async def generate_instructor_dashboard(analytics_request: AnalyticsRequest):
    """Generate instructor analytics dashboard"""
    try:
        if not analytics_request.instructor_id:
            raise HTTPException(status_code=400, detail="instructor_id is required")
        
        dashboard = dashboard_generator.generate_instructor_dashboard(
            instructor_id=analytics_request.instructor_id,
            period=analytics_request.period,
            custom_start=analytics_request.custom_start,
            custom_end=analytics_request.custom_end
        )
        
        return dashboard.__dict__
    
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/analytics/department-dashboard")
async def generate_department_dashboard(
    department_name: str = "Computer Science",
    period: AnalyticsPeriod = AnalyticsPeriod.SEMESTER
):
    """Generate department analytics dashboard"""
    try:
        dashboard = dashboard_generator.generate_department_dashboard(
            department_name=department_name,
            period=period
        )
        
        return dashboard.__dict__
    
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/api/analytics/alerts")
async def get_active_alerts(
    severity: Optional[AlertSeverity] = None
):
    """Get active alerts"""
    try:
        alerts = alert_manager.get_active_alerts(severity)
        return {"alerts": [alert.__dict__ for alert in alerts]}
    
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/analytics/alerts/{alert_id}/acknowledge")
async def acknowledge_alert(alert_id: str):
    """Acknowledge an alert"""
    try:
        success = alert_manager.acknowledge_alert(alert_id)
        if not success:
            raise HTTPException(status_code=404, detail="Alert not found")
        
        return {"message": "Alert acknowledged"}
    
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/api/analytics/alerts/summary")
async def get_alert_summary():
    """Get alert summary"""
    try:
        summary = alert_manager.get_alert_summary()
        return summary
    
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


# Report Generation Endpoints

@app.get("/api/reports/student/{student_id}/progress")
async def generate_student_progress_report(
    student_id: str,
    course_id: Optional[str] = None,
    format: str = Query("json", regex="^(json|csv|pdf)$")
):
    """Generate student progress report"""
    try:
        from ..educator_dashboard.dashboard import ReportGenerator
        
        report_generator = ReportGenerator(analytics_engine, dashboard_generator)
        
        if format == "json":
            report = report_generator.generate_progress_report(
                student_id, course_id or "", "json"
            )
            return JSONResponse(content=json.loads(report))
        else:
            report = report_generator.generate_progress_report(
                student_id, course_id or "", format
            )
            return {"report": report}
    
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/api/reports/course/{course_id}/performance")
async def generate_course_performance_report(
    course_id: str,
    instructor_id: str,
    period: AnalyticsPeriod = AnalyticsPeriod.MONTH,
    format: str = Query("json", regex="^(json|csv|pdf)$")
):
    """Generate course performance report"""
    try:
        from ..educator_dashboard.dashboard import ReportGenerator
        
        report_generator = ReportGenerator(analytics_engine, dashboard_generator)
        
        if format == "json":
            report = report_generator.generate_performance_report(
                course_id, instructor_id, period
            )
            return JSONResponse(content=json.loads(report))
        else:
            report = report_generator.generate_performance_report(
                course_id, instructor_id, period
            )
            return {"report": report}
    
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


# Health Check and System Status

@app.get("/api/health")
async def health_check():
    """System health check"""
    try:
        return {
            "status": "healthy",
            "timestamp": datetime.now().isoformat(),
            "services": {
                "curriculum_manager": "active",
                "lms_integration": "active",
                "assessment_system": "active",
                "assignment_manager": "active",
                "calendar_system": "active",
                "analytics_engine": "active"
            },
            "version": "1.0.0"
        }
    
    except Exception as e:
        return JSONResponse(
            status_code=500,
            content={
                "status": "unhealthy",
                "timestamp": datetime.now().isoformat(),
                "error": str(e)
            }
        )


@app.get("/api/system/status")
async def system_status():
    """Get comprehensive system status"""
    try:
        return {
            "system_info": {
                "platform": "MultiOS Academic Integration",
                "version": "1.0.0",
                "components": len([
                    curriculum_manager.courses,
                    lms_manager.connections,
                    assessment_manager.assessments,
                    assignment_manager.assignments,
                    calendar_manager.semesters,
                    template_manager.templates
                ])
            },
            "curriculum": {
                "total_courses": len(curriculum_manager.courses),
                "total_learning_outcomes": len(curriculum_manager.learning_outcomes),
                "total_units": len(curriculum_manager.units)
            },
            "lms_integration": lms_manager.get_integration_status(),
            "assessment": {
                "total_assessments": len(assessment_manager.assessments),
                "total_submissions": len(assessment_manager.submissions),
                "total_scores": len(assessment_manager.scores)
            },
            "assignments": {
                "total_assignments": len(assignment_manager.assignments),
                "total_submissions": len(assignment_manager.submissions),
                "peer_reviews": len(assignment_manager.peer_reviews)
            },
            "schedule": {
                "total_semesters": len(calendar_manager.semesters),
                "total_events": len(calendar_manager.events),
                "total_rooms": len(calendar_manager.rooms)
            },
            "templates": template_manager.get_template_statistics(),
            "analytics": {
                "active_alerts": len(alert_manager.get_active_alerts()),
                "alerts_summary": alert_manager.get_alert_summary()
            }
        }
    
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


# Root endpoint
@app.get("/")
async def root():
    """API root endpoint"""
    return {
        "message": "MultiOS Academic Curriculum Integration API",
        "version": "1.0.0",
        "docs": "/api/docs",
        "endpoints": {
            "curriculum": "/api/curriculum",
            "lms": "/api/lms",
            "assessment": "/api/assessment",
            "assignments": "/api/assignments",
            "schedule": "/api/schedule",
            "templates": "/api/templates",
            "analytics": "/api/analytics",
            "reports": "/api/reports"
        }
    }


if __name__ == "__main__":
    # Run the API server
    uvicorn.run(
        "api:app",
        host="0.0.0.0",
        port=8000,
        reload=True,
        log_level="info"
    )