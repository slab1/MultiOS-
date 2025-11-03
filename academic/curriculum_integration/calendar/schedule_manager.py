"""
Academic Calendar and Scheduling Integration for MultiOS Academic Platform

This module provides comprehensive tools for managing academic calendars, course scheduling,
exam scheduling, and integration with external calendar systems.
"""

from typing import Dict, List, Optional, Any, Union
from dataclasses import dataclass, asdict, field
from datetime import datetime, timedelta, date, time
from enum import Enum
import json
import uuid
import calendar
from collections import defaultdict


class EventType(Enum):
    """Types of academic events"""
    CLASS = "class"
    LAB = "lab"
    EXAM = "exam"
    ASSIGNMENT_DUE = "assignment_due"
    PROJECT_DUE = "project_due"
    HOLIDAY = "holiday"
    BREAK = "break"
    REGISTRATION = "registration"
    GRADUATION = "graduation"
    FACULTY_MEETING = "faculty_meeting"
    CONFERENCE = "conference"
    WORKSHOP = "workshop"


class ScheduleFrequency(Enum):
    """Schedule frequency options"""
    ONCE = "once"
    WEEKLY = "weekly"
    BIWEEKLY = "biweekly"
    MONTHLY = "monthly"
    CUSTOM = "custom"


class RoomType(Enum):
    """Types of rooms"""
    LECTURE_HALL = "lecture_hall"
    COMPUTER_LAB = "computer_lab"
    REGULAR_CLASSROOM = "regular_classroom"
    AUDITORIUM = "auditorium"
    CONFERENCE_ROOM = "conference_room"
    OUTDOOR = "outdoor"


class TimeSlot(Enum):
    """Standard time slots"""
    EARLY_MORNING = (7, 0)  # 7:00 AM
    MORNING = (9, 0)        # 9:00 AM
    LATE_MORNING = (10, 30) # 10:30 AM
    NOON = (12, 0)          # 12:00 PM
    AFTERNOON = (13, 30)    # 1:30 PM
    LATE_AFTERNOON = (15, 0) # 3:00 PM
    EVENING = (18, 0)       # 6:00 PM
    NIGHT = (20, 0)         # 8:00 PM


@dataclass
class TimeRange:
    """Represents a time range"""
    start_time: datetime
    end_time: datetime
    timezone: str = "UTC"
    
    @property
    def duration(self) -> timedelta:
        """Get duration of time range"""
        return self.end_time - self.start_time
    
    def overlaps_with(self, other: 'TimeRange') -> bool:
        """Check if this time range overlaps with another"""
        return (self.start_time < other.end_time and 
                other.start_time < self.end_time)
    
    def is_within(self, other: 'TimeRange') -> bool:
        """Check if this time range is within another"""
        return (other.start_time <= self.start_time and 
                self.end_time <= other.end_time)


@dataclass
class Room:
    """Represents a physical room"""
    id: str
    name: str
    building: str
    capacity: int
    room_type: RoomType
    features: List[str] = field(default_factory=list)  # projector, whiteboard, computers, etc.
    accessibility_features: List[str] = field(default_factory=list)
    booking_priority: int = 0
    is_available: bool = True
    maintenance_schedule: List[Dict[str, Any]] = field(default_factory=list)
    
    def __post_init__(self):
        if self.id is None:
            self.id = str(uuid.uuid4())
    
    @property
    def is_accessible(self) -> bool:
        """Check if room has accessibility features"""
        required_features = ["wheelchair_accessible", "hearing_assistance"]
        return any(feature in self.accessibility_features for feature in required_features)


@dataclass
class Instructor:
    """Represents an instructor"""
    id: str
    name: str
    email: str
    department: str
    office_hours: List[Dict[str, Any]] = field(default_factory=list)
    max_teaching_load: int = 4  # courses per semester
    preferred_times: List[TimeSlot] = field(default_factory=list)
    unavailable_times: List[TimeRange] = field(default_factory=list)
    specializations: List[str] = field(default_factory=list)
    
    def __post_init__(self):
        if self.id is None:
            self.id = str(uuid.uuid4())
    
    def is_available(self, time_range: TimeRange) -> bool:
        """Check if instructor is available during time range"""
        # Check if time overlaps with unavailable times
        for unavailable in self.unavailable_times:
            if time_range.overlaps_with(unavailable):
                return False
        return True


@dataclass
class Course:
    """Course with scheduling information"""
    id: str
    code: str
    title: str
    instructor_id: str
    credits: int
    enrolled_students: int = 0
    max_capacity: int = 30
    prerequisites: List[str] = field(default_factory=list)
    required_room_type: RoomType = RoomType.REGULAR_CLASSROOM
    special_requirements: List[str] = field(default_factory=list)
    
    def __post_init__(self):
        if self.id is None:
            self.id = str(uuid.uuid4())


@dataclass
class AcademicEvent:
    """Represents an academic event or class meeting"""
    id: str
    title: str
    event_type: EventType
    course_id: Optional[str]
    instructor_id: str
    room_id: Optional[str]
    time_range: TimeRange
    frequency: ScheduleFrequency = ScheduleFrequency.WEEKLY
    end_date: Optional[datetime] = None
    capacity: int = 0
    attendees: List[str] = field(default_factory=list)
    description: str = ""
    resources: List[Dict[str, Any]] = field(default_factory=list)
    recurrence_pattern: Optional[str] = None  # e.g., "every Monday and Wednesday"
    created_at: datetime = field(default_factory=datetime.now)
    
    def __post_init__(self):
        if self.id is None:
            self.id = str(uuid.uuid4())
    
    @property
    def duration_minutes(self) -> int:
        """Get duration in minutes"""
        return int(self.time_range.duration.total_seconds() / 60)


@dataclass
class Semester:
    """Academic semester definition"""
    id: str
    name: str  # "Fall 2025", "Spring 2026", etc.
    start_date: date
    end_date: date
    registration_deadline: date
    add_drop_deadline: date
    final_exam_period_start: date
    final_exam_period_end: date
    holidays: List[Dict[str, Any]] = field(default_factory=list)
    breaks: List[Dict[str, Any]] = field(default_factory=list)
    
    def __post_init__(self):
        if self.id is None:
            self.id = str(uuid.uuid4())
    
    @property
    def duration_days(self) -> int:
        """Get semester duration in days"""
        return (self.end_date - self.start_date).days + 1
    
    @property
    def total_weeks(self) -> int:
        """Get total number of weeks"""
        return self.duration_days // 7
    
    def is_class_day(self, day: date) -> bool:
        """Check if a given day is a regular class day"""
        # Check if it's during semester
        if not (self.start_date <= day <= self.end_date):
            return False
        
        # Check if it's not a holiday or break
        for holiday in self.holidays:
            holiday_date = datetime.strptime(holiday["date"], "%Y-%m-%d").date()
            if holiday_date == day:
                return False
        
        for break_period in self.breaks:
            break_start = datetime.strptime(break_period["start_date"], "%Y-%m-%d").date()
            break_end = datetime.strptime(break_period["end_date"], "%Y-%m-%d").date()
            if break_start <= day <= break_end:
                return False
        
        return True


@dataclass
class ExamSchedule:
    """Exam schedule for a course"""
    id: str
    course_id: str
    exam_type: str  # "midterm", "final", "quiz"
    scheduled_events: List[AcademicEvent] = field(default_factory=list)
    room_requirements: List[str] = field(default_factory=list)
    proctor_requirements: int = 1
    
    def __post_init__(self):
        if self.id is None:
            self.id = str(uuid.uuid4())


class CalendarManager:
    """Main calendar and scheduling management system"""
    
    def __init__(self):
        self.rooms: Dict[str, Room] = {}
        self.instructors: Dict[str, Instructor] = {}
        self.courses: Dict[str, Course] = {}
        self.events: Dict[str, AcademicEvent] = {}
        self.semesters: Dict[str, Semester] = {}
        self.exam_schedules: Dict[str, ExamSchedule] = {}
        self.conflicts: List[Dict[str, Any]] = []
    
    # Room Management
    def add_room(self, name: str, building: str, capacity: int, 
                room_type: RoomType, features: List[str] = None) -> Room:
        """Add a new room"""
        if features is None:
            features = []
        
        room = Room(
            id=str(uuid.uuid4()),
            name=name,
            building=building,
            capacity=capacity,
            room_type=room_type,
            features=features
        )
        
        self.rooms[room.id] = room
        return room
    
    def get_available_rooms(self, time_range: TimeRange, 
                           capacity_needed: int = 0,
                           room_type: RoomType = None,
                           required_features: List[str] = None) -> List[Room]:
        """Get available rooms for a specific time"""
        
        if required_features is None:
            required_features = []
        
        available_rooms = []
        
        for room in self.rooms.values():
            # Check capacity
            if capacity_needed > 0 and room.capacity < capacity_needed:
                continue
            
            # Check room type
            if room_type and room.room_type != room_type:
                continue
            
            # Check features
            if required_features and not all(feature in room.features for feature in required_features):
                continue
            
            # Check availability
            if not self._is_room_available(room.id, time_range):
                continue
            
            available_rooms.append(room)
        
        # Sort by capacity (closest match first) and priority
        available_rooms.sort(key=lambda r: (abs(r.capacity - capacity_needed) if capacity_needed > 0 else 0, -r.booking_priority))
        
        return available_rooms
    
    def _is_room_available(self, room_id: str, time_range: TimeRange) -> bool:
        """Check if room is available during time range"""
        for event in self.events.values():
            if event.room_id == room_id and event.time_range.overlaps_with(time_range):
                return False
        return True
    
    # Instructor Management
    def add_instructor(self, name: str, email: str, department: str,
                      max_teaching_load: int = 4) -> Instructor:
        """Add a new instructor"""
        instructor = Instructor(
            id=str(uuid.uuid4()),
            name=name,
            email=email,
            department=department,
            max_teaching_load=max_teaching_load
        )
        
        self.instructors[instructor.id] = instructor
        return instructor
    
    def get_instructor_schedule(self, instructor_id: str, start_date: date, 
                               end_date: date) -> List[AcademicEvent]:
        """Get instructor's schedule for a date range"""
        schedule = []
        
        for event in self.events.values():
            if event.instructor_id == instructor_id:
                event_date = event.time_range.start_time.date()
                if start_date <= event_date <= end_date:
                    schedule.append(event)
        
        # Sort by date and time
        schedule.sort(key=lambda e: e.time_range.start_time)
        return schedule
    
    def get_available_instructors(self, time_range: TimeRange) -> List[Instructor]:
        """Get available instructors for a specific time"""
        available = []
        
        for instructor in self.instructors.values():
            if instructor.is_available(time_range):
                # Check current teaching load
                current_events = self.get_instructor_schedule(
                    instructor.id,
                    time_range.start_time.date(),
                    time_range.start_time.date()
                )
                
                if len(current_events) < instructor.max_teaching_load:
                    available.append(instructor)
        
        return available
    
    # Course Management
    def add_course(self, code: str, title: str, instructor_id: str, 
                  credits: int, max_capacity: int = 30) -> Course:
        """Add a new course"""
        course = Course(
            id=str(uuid.uuid4()),
            code=code,
            title=title,
            instructor_id=instructor_id,
            credits=credits,
            max_capacity=max_capacity
        )
        
        self.courses[course.id] = course
        return course
    
    # Event Management
    def create_class_meeting(self, course_id: str, title: str, 
                           instructor_id: str, room_id: str,
                           start_time: datetime, end_time: datetime,
                           frequency: ScheduleFrequency = ScheduleFrequency.WEEKLY,
                           end_date: Optional[datetime] = None) -> AcademicEvent:
        """Create a class meeting event"""
        
        course = self.courses.get(course_id)
        if not course:
            raise ValueError("Course not found")
        
        time_range = TimeRange(start_time, end_time)
        
        event = AcademicEvent(
            id=str(uuid.uuid4()),
            title=title,
            event_type=EventType.CLASS,
            course_id=course_id,
            instructor_id=instructor_id,
            room_id=room_id,
            time_range=time_range,
            frequency=frequency,
            end_date=end_date,
            capacity=course.max_capacity
        )
        
        self.events[event.id] = event
        return event
    
    def create_exam(self, course_id: str, exam_type: str,
                   start_time: datetime, duration_minutes: int,
                   room_id: Optional[str] = None,
                   proctor_id: Optional[str] = None) -> ExamSchedule:
        """Create an exam schedule"""
        
        end_time = start_time + timedelta(minutes=duration_minutes)
        time_range = TimeRange(start_time, end_time)
        
        # Create exam event
        event = AcademicEvent(
            title=f"{exam_type.title()} Exam",
            event_type=EventType.EXAM,
            course_id=course_id,
            instructor_id=proctor_id or "",
            room_id=room_id,
            time_range=time_range
        )
        
        exam_schedule = ExamSchedule(
            course_id=course_id,
            exam_type=exam_type
        )
        exam_schedule.scheduled_events.append(event)
        
        self.exam_schedules[exam_schedule.id] = exam_schedule
        return exam_schedule
    
    def schedule_course(self, course_id: str, semester_id: str,
                       meeting_days: List[str],  # ["Monday", "Wednesday", "Friday"]
                       start_time: time, end_time: time,
                       room_id: Optional[str] = None) -> List[AcademicEvent]:
        """Schedule all class meetings for a course in a semester"""
        
        course = self.courses.get(course_id)
        semester = self.semesters.get(semester_id)
        
        if not course or not semester:
            raise ValueError("Course or semester not found")
        
        events = []
        
        # Generate class meetings for each class day
        current_date = semester.start_date
        while current_date <= semester.end_date:
            # Check if this is a class day and matches meeting days
            day_name = calendar.day_name[current_date.weekday()]
            
            if (day_name in meeting_days and 
                semester.is_class_day(current_date)):
                
                # Find available room if not specified
                if not room_id:
                    class_duration = datetime.combine(current_date, end_time) - datetime.combine(current_date, start_time)
                    start_datetime = datetime.combine(current_date, start_time)
                    end_datetime = start_datetime + class_duration
                    
                    time_range = TimeRange(start_datetime, end_datetime)
                    
                    available_rooms = self.get_available_rooms(
                        time_range,
                        capacity_needed=course.max_capacity,
                        room_type=course.required_room_type
                    )
                    
                    if not available_rooms:
                        raise ValueError(f"No available room for {course.code} on {current_date}")
                    
                    room_id = available_rooms[0].id
                
                # Create class meeting
                start_datetime = datetime.combine(current_date, start_time)
                end_datetime = datetime.combine(current_date, end_time)
                
                event = self.create_class_meeting(
                    course_id=course_id,
                    title=f"{course.code} - {course.title}",
                    instructor_id=course.instructor_id,
                    room_id=room_id,
                    start_time=start_datetime,
                    end_time=end_datetime
                )
                
                events.append(event)
                
                # Reset room_id for next iteration if it was auto-assigned
                if room_id not in [r.id for r in self.rooms.values()]:
                    room_id = None
            
            current_date += timedelta(days=1)
        
        return events
    
    # Conflict Detection
    def detect_conflicts(self) -> List[Dict[str, Any]]:
        """Detect scheduling conflicts"""
        conflicts = []
        
        # Room conflicts
        room_usage = defaultdict(list)
        for event in self.events.values():
            if event.room_id:
                room_usage[event.room_id].append(event)
        
        for room_id, events in room_usage.items():
            events.sort(key=lambda e: e.time_range.start_time)
            for i in range(len(events) - 1):
                if events[i].time_range.overlaps_with(events[i + 1].time_range):
                    conflicts.append({
                        "type": "room_conflict",
                        "room_id": room_id,
                        "event1_id": events[i].id,
                        "event2_id": events[i + 1].id,
                        "time": events[i].time_range.start_time,
                        "description": f"Room double-booked for events {events[i].id} and {events[i + 1].id}"
                    })
        
        # Instructor conflicts
        instructor_schedule = defaultdict(list)
        for event in self.events.values():
            if event.instructor_id:
                instructor_schedule[event.instructor_id].append(event)
        
        for instructor_id, events in instructor_schedule.items():
            events.sort(key=lambda e: e.time_range.start_time)
            for i in range(len(events) - 1):
                if events[i].time_range.overlaps_with(events[i + 1].time_range):
                    conflicts.append({
                        "type": "instructor_conflict",
                        "instructor_id": instructor_id,
                        "event1_id": events[i].id,
                        "event2_id": events[i + 1].id,
                        "time": events[i].time_range.start_time,
                        "description": f"Instructor double-booked for events {events[i].id} and {events[i + 1].id}"
                    })
        
        self.conflicts = conflicts
        return conflicts
    
    # Calendar Export and Integration
    def export_calendar(self, format: str = "json", 
                       start_date: Optional[date] = None,
                       end_date: Optional[date] = None) -> str:
        """Export calendar in various formats"""
        
        if start_date is None:
            start_date = date.today()
        if end_date is None:
            end_date = start_date + timedelta(days=365)
        
        filtered_events = []
        for event in self.events.values():
            event_date = event.time_range.start_time.date()
            if start_date <= event_date <= end_date:
                filtered_events.append(event)
        
        if format == "json":
            return json.dumps([asdict(event) for event in filtered_events], indent=2, default=str)
        
        elif format == "ical":
            return self._generate_icalendar(filtered_events)
        
        elif format == "csv":
            return self._generate_csv(filtered_events)
        
        else:
            raise ValueError(f"Unsupported format: {format}")
    
    def _generate_icalendar(self, events: List[AcademicEvent]) -> str:
        """Generate iCalendar format"""
        lines = [
            "BEGIN:VCALENDAR",
            "VERSION:2.0",
            "PRODID:-//MultiOS Academic Platform//Calendar//EN"
        ]
        
        for event in events:
            event_data = asdict(event)
            start_time = event.time_range.start_time.strftime("%Y%m%dT%H%M%S")
            end_time = event.time_range.end_time.strftime("%Y%m%dT%H%M%S")
            
            lines.extend([
                "BEGIN:VEVENT",
                f"UID:{event.id}",
                f"DTSTART:{start_time}",
                f"DTEND:{end_time}",
                f"SUMMARY:{event.title}",
                f"DESCRIPTION:{event.description}",
                "END:VEVENT"
            ])
        
        lines.append("END:VCALENDAR")
        return "\n".join(lines)
    
    def _generate_csv(self, events: List[AcademicEvent]) -> str:
        """Generate CSV format"""
        lines = ["Title,Date,Start Time,End Time,Type,Course,Instructor,Room"]
        
        for event in events:
            course_title = ""
            instructor_name = ""
            room_name = ""
            
            if event.course_id and event.course_id in self.courses:
                course_title = self.courses[event.course_id].title
            
            if event.instructor_id and event.instructor_id in self.instructors:
                instructor_name = self.instructors[event.instructor_id].name
            
            if event.room_id and event.room_id in self.rooms:
                room_name = self.rooms[event.room_id].name
            
            date_str = event.time_range.start_time.strftime("%Y-%m-%d")
            start_time_str = event.time_range.start_time.strftime("%H:%M")
            end_time_str = event.time_range.end_time.strftime("%H:%M")
            
            line = f'"{event.title}",{date_str},{start_time_str},{end_time_str},{event.event_type.value},"{course_title}","{instructor_name}","{room_name}"'
            lines.append(line)
        
        return "\n".join(lines)
    
    # Semester Management
    def create_semester(self, name: str, start_date: date, end_date: date,
                       holidays: List[Dict[str, Any]] = None,
                       breaks: List[Dict[str, Any]] = None) -> Semester:
        """Create a new semester"""
        
        if holidays is None:
            holidays = []
        if breaks is None:
            breaks = []
        
        # Calculate important dates
        registration_deadline = start_date - timedelta(days=30)
        add_drop_deadline = start_date + timedelta(days=14)
        final_exam_period_start = end_date - timedelta(days=7)
        final_exam_period_end = end_date
        
        semester = Semester(
            id=str(uuid.uuid4()),
            name=name,
            start_date=start_date,
            end_date=end_date,
            registration_deadline=registration_deadline,
            add_drop_deadline=add_drop_deadline,
            final_exam_period_start=final_exam_period_start,
            final_exam_period_end=final_exam_period_end,
            holidays=holidays,
            breaks=breaks
        )
        
        self.semesters[semester.id] = semester
        return semester
    
    # Analytics and Reporting
    def get_room_utilization(self, start_date: date, end_date: date) -> Dict[str, Any]:
        """Get room utilization statistics"""
        utilization = {}
        
        for room_id, room in self.rooms.items():
            room_events = []
            
            for event in self.events.values():
                if event.room_id == room_id:
                    event_date = event.time_range.start_time.date()
                    if start_date <= event_date <= end_date:
                        room_events.append(event)
            
            # Calculate utilization
            total_time = 0
            for event in room_events:
                total_time += event.time_range.duration.total_seconds()
            
            # Total available time (assuming 8 AM to 10 PM weekdays)
            days_count = (end_date - start_date).days + 1
            available_hours_per_day = 14  # 8 AM to 10 PM
            total_available_seconds = days_count * available_hours_per_day * 3600
            
            utilization_percentage = (total_time / total_available_seconds * 100) if total_available_seconds > 0 else 0
            
            utilization[room_id] = {
                "room_name": room.name,
                "total_events": len(room_events),
                "total_hours": total_time / 3600,
                "utilization_percentage": round(utilization_percentage, 2),
                "average_daily_events": len(room_events) / days_count if days_count > 0 else 0
            }
        
        return utilization
    
    def get_instructor_workload(self, instructor_id: str, 
                              start_date: date, end_date: date) -> Dict[str, Any]:
        """Get instructor workload statistics"""
        
        instructor = self.instructors.get(instructor_id)
        if not instructor:
            return {}
        
        events = self.get_instructor_schedule(instructor_id, start_date, end_date)
        
        total_hours = sum(event.time_range.duration.total_seconds() for event in events) / 3600
        total_events = len(events)
        
        # Calculate class hours vs office hours
        class_hours = sum(event.time_range.duration.total_seconds() for event in events 
                         if event.event_type == EventType.CLASS) / 3600
        
        return {
            "instructor_name": instructor.name,
            "total_hours": round(total_hours, 2),
            "total_events": total_events,
            "class_hours": round(class_hours, 2),
            "office_hours": round(total_hours - class_hours, 2),
            "average_hours_per_day": total_hours / ((end_date - start_date).days + 1) if start_date <= end_date else 0
        }
    
    def generate_schedule_report(self, semester_id: str) -> Dict[str, Any]:
        """Generate comprehensive schedule report for semester"""
        
        semester = self.semesters.get(semester_id)
        if not semester:
            return {"error": "Semester not found"}
        
        # Get all events in semester
        semester_events = []
        for event in self.events.values():
            event_date = event.time_range.start_time.date()
            if semester.start_date <= event_date <= semester.end_date:
                semester_events.append(event)
        
        # Calculate statistics
        room_utilization = self.get_room_utilization(semester.start_date, semester.end_date)
        conflicts = self.detect_conflicts()
        
        # Event type distribution
        event_types = defaultdict(int)
        for event in semester_events:
            event_types[event.event_type.value] += 1
        
        # Daily class load
        daily_load = defaultdict(int)
        for event in semester_events:
            day = event.time_range.start_time.strftime("%A")
            daily_load[day] += 1
        
        return {
            "semester": asdict(semester),
            "summary": {
                "total_events": len(semester_events),
                "total_courses": len(self.courses),
                "total_instructors": len(self.instructors),
                "total_rooms": len(self.rooms),
                "conflict_count": len(conflicts)
            },
            "event_distribution": dict(event_types),
            "daily_load": dict(daily_load),
            "room_utilization": room_utilization,
            "conflicts": conflicts,
            "recommendations": self._generate_schedule_recommendations(conflicts, room_utilization)
        }
    
    def _generate_schedule_recommendations(self, conflicts: List[Dict[str, Any]], 
                                         room_utilization: Dict[str, Any]) -> List[str]:
        """Generate scheduling recommendations"""
        recommendations = []
        
        # Conflict-based recommendations
        room_conflicts = [c for c in conflicts if c["type"] == "room_conflict"]
        if room_conflicts:
            recommendations.append(f"Resolve {len(room_conflicts)} room conflicts by adjusting schedules")
        
        # Utilization-based recommendations
        low_utilization_rooms = [r for r in room_utilization.values() 
                               if r["utilization_percentage"] < 30]
        high_utilization_rooms = [r for r in room_utilization.values() 
                                if r["utilization_percentage"] > 90]
        
        if low_utilization_rooms:
            recommendations.append("Consider using underutilized rooms more efficiently")
        
        if high_utilization_rooms:
            recommendations.append("High-utilization rooms may need backup options or capacity expansion")
        
        # General recommendations
        recommendations.extend([
            "Review scheduling policies for optimal resource allocation",
            "Consider instructor preferences when creating schedules",
            "Implement automated scheduling tools for complex course combinations"
        ])
        
        return recommendations


# Example usage
if __name__ == "__main__":
    # Initialize calendar manager
    calendar_mgr = CalendarManager()
    
    # Create semester
    fall_2025 = calendar_mgr.create_semester(
        name="Fall 2025",
        start_date=date(2025, 8, 25),
        end_date=date(2025, 12, 15),
        holidays=[
            {"name": "Labor Day", "date": "2025-09-01"},
            {"name": "Thanksgiving", "date": "2025-11-27"},
            {"name": "Christmas", "date": "2025-12-25"}
        ],
        breaks=[
            {"name": "Fall Break", "start_date": "2025-10-13", "end_date": "2025-10-14"},
            {"name": "Thanksgiving Break", "start_date": "2025-11-26", "end_date": "2025-11-30"}
        ]
    )
    
    # Add rooms
    room101 = calendar_mgr.add_room(
        name="Computer Lab 101",
        building="Engineering Building",
        capacity=30,
        room_type=RoomType.COMPUTER_LAB,
        features=["projector", "whiteboard", "computers", "internet"]
    )
    
    room201 = calendar_mgr.add_room(
        name="Lecture Hall 201",
        building="Science Building",
        capacity=100,
        room_type=RoomType.LECTURE_HALL,
        features=["projector", "microphone", "recording", "accessibility"]
    )
    
    # Add instructors
    prof_smith = calendar_mgr.add_instructor(
        name="Dr. Jane Smith",
        email="j.smith@university.edu",
        department="Computer Science"
    )
    
    prof_jones = calendar_mgr.add_instructor(
        name="Dr. Bob Jones",
        email="b.jones@university.edu",
        department="Computer Science"
    )
    
    # Add courses
    cs301 = calendar_mgr.add_course(
        code="CS 301",
        title="Introduction to Operating Systems",
        instructor_id=prof_smith.id,
        credits=3,
        max_capacity=30
    )
    
    cs401 = calendar_mgr.add_course(
        code="CS 401",
        title="Advanced Operating Systems",
        instructor_id=prof_jones.id,
        credits=4,
        max_capacity=25
    )
    
    # Schedule courses
    try:
        cs301_events = calendar_mgr.schedule_course(
            course_id=cs301.id,
            semester_id=fall_2025.id,
            meeting_days=["Monday", "Wednesday", "Friday"],
            start_time=time(10, 0),  # 10:00 AM
            end_time=time(11, 15)    # 11:15 AM
        )
        
        print(f"Scheduled {len(cs301_events)} class meetings for CS 301")
        
        cs401_events = calendar_mgr.schedule_course(
            course_id=cs401.id,
            semester_id=fall_2025.id,
            meeting_days=["Tuesday", "Thursday"],
            start_time=time(14, 0),  # 2:00 PM
            end_time=time(15, 45)    # 3:45 PM
        )
        
        print(f"Scheduled {len(cs401_events)} class meetings for CS 401")
        
    except Exception as e:
        print(f"Scheduling error: {e}")
    
    # Create final exams
    final_exam = calendar_mgr.create_exam(
        course_id=cs301.id,
        exam_type="final",
        start_time=datetime(2025, 12, 15, 9, 0),  # 9:00 AM on Dec 15
        duration_minutes=180,  # 3 hours
        room_id=room201.id
    )
    
    print(f"Created final exam schedule with {len(final_exam.scheduled_events)} events")
    
    # Detect conflicts
    conflicts = calendar_mgr.detect_conflicts()
    print(f"Found {len(conflicts)} scheduling conflicts")
    
    # Generate reports
    room_utilization = calendar_mgr.get_room_utilization(
        fall_2025.start_date,
        fall_2025.end_date
    )
    print("\nRoom Utilization:")
    for room_id, stats in room_utilization.items():
        print(f"  {stats['room_name']}: {stats['utilization_percentage']:.1f}% utilization")
    
    workload = calendar_mgr.get_instructor_workload(
        prof_smith.id,
        fall_2025.start_date,
        fall_2025.end_date
    )
    print(f"\n{workload['instructor_name']} Workload:")
    print(f"  Total hours: {workload['total_hours']}")
    print(f"  Total events: {workload['total_events']}")
    
    # Export calendar
    calendar_export = calendar_mgr.export_calendar("ical")
    print(f"\nExported calendar with {len(calendar_export)} characters")
    
    # Generate comprehensive report
    schedule_report = calendar_mgr.generate_schedule_report(fall_2025.id)
    print(f"\nSchedule Report Summary:")
    print(f"  Total events: {schedule_report['summary']['total_events']}")
    print(f"  Total courses: {schedule_report['summary']['total_courses']}")
    print(f"  Conflicts: {schedule_report['summary']['conflict_count']}")
    
    if schedule_report['recommendations']:
        print("  Recommendations:")
        for rec in schedule_report['recommendations']:
            print(f"    - {rec}")