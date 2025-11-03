#!/usr/bin/env python3
"""
MultiOS Educational Analytics Module
Comprehensive analytics for educational lab environments and student activity
"""

import asyncio
import json
import logging
import time
import sqlite3
import threading
from datetime import datetime, timedelta
from typing import Dict, List, Any, Optional, Tuple
from dataclasses import dataclass, asdict
from pathlib import Path
from collections import defaultdict, deque
import psutil
import yaml

@dataclass
class LabSession:
    """Lab session tracking"""
    session_id: str
    student_id: str
    username: str
    lab_id: str
    start_time: float
    end_time: Optional[float]
    duration: Optional[float]
    ip_address: str
    terminal: str
    resources_used: Dict[str, Any]
    activities: List[Dict[str, Any]]
    status: str  # active, completed, abandoned

@dataclass
class StudentActivity:
    """Student activity tracking"""
    student_id: str
    timestamp: float
    activity_type: str  # login, logout, application_use, file_access, network_activity
    details: Dict[str, Any]
    lab_context: str
    session_id: str

@dataclass
class CourseResource:
    """Course resource utilization"""
    course_id: str
    course_name: str
    lab_id: str
    student_count: int
    cpu_usage: float
    memory_usage: float
    storage_usage: float
    network_usage: float
    session_duration_avg: float
    completion_rate: float

@dataclass
class EducationalKPI:
    """Educational KPI metric"""
    name: str
    value: float
    target: float
    unit: str
    category: str
    trend: str  # improving, declining, stable
    last_updated: float

class LabSessionManager:
    """Manages lab session tracking and lifecycle"""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.active_sessions = {}
        self.session_history = deque(maxlen=10000)
        self.session_id_counter = 0
        
    def start_session(self, username: str, student_id: str, lab_id: str, 
                     ip_address: str, terminal: str) -> str:
        """Start a new lab session"""
        session_id = f"session_{int(time.time())}_{self.session_id_counter}"
        self.session_id_counter += 1
        
        session = LabSession(
            session_id=session_id,
            student_id=student_id,
            username=username,
            lab_id=lab_id,
            start_time=time.time(),
            end_time=None,
            duration=None,
            ip_address=ip_address,
            terminal=terminal,
            resources_used={},
            activities=[],
            status='active'
        )
        
        self.active_sessions[session_id] = session
        logging.info(f"Started lab session: {session_id} for {username}")
        
        return session_id
    
    def end_session(self, session_id: str, status: str = 'completed'):
        """End a lab session"""
        if session_id in self.active_sessions:
            session = self.active_sessions[session_id]
            session.end_time = time.time()
            session.duration = session.end_time - session.start_time
            session.status = status
            
            # Move to history
            self.session_history.append(session)
            del self.active_sessions[session_id]
            
            logging.info(f"Ended lab session: {session_id} - {status}")
    
    def record_activity(self, session_id: str, activity_type: str, details: Dict[str, Any]):
        """Record student activity"""
        if session_id in self.active_sessions:
            session = self.active_sessions[session_id]
            
            activity = {
                'timestamp': time.time(),
                'type': activity_type,
                'details': details
            }
            
            session.activities.append(activity)
    
    def update_resources(self, session_id: str, resources: Dict[str, Any]):
        """Update resource usage for session"""
        if session_id in self.active_sessions:
            session = self.active_sessions[session_id]
            session.resources_used.update(resources)
    
    def get_active_sessions(self) -> List[LabSession]:
        """Get all active sessions"""
        return list(self.active_sessions.values())
    
    def get_session_by_id(self, session_id: str) -> Optional[LabSession]:
        """Get session by ID"""
        if session_id in self.active_sessions:
            return self.active_sessions[session_id]
        
        # Check session history
        for session in self.session_history:
            if session.session_id == session_id:
                return session
        
        return None
    
    def get_sessions_by_student(self, student_id: str) -> List[LabSession]:
        """Get all sessions for a student"""
        sessions = []
        
        # Check active sessions
        for session in self.active_sessions.values():
            if session.student_id == student_id:
                sessions.append(session)
        
        # Check session history
        for session in self.session_history:
            if session.student_id == student_id:
                sessions.append(session)
        
        return sessions
    
    def get_sessions_by_lab(self, lab_id: str) -> List[LabSession]:
        """Get all sessions for a lab"""
        sessions = []
        
        # Check active sessions
        for session in self.active_sessions.values():
            if session.lab_id == lab_id:
                sessions.append(session)
        
        # Check session history
        for session in self.session_history:
            if session.lab_id == lab_id:
                sessions.append(session)
        
        return sessions
    
    def get_session_statistics(self, hours: int = 24) -> Dict[str, Any]:
        """Get session statistics for specified period"""
        cutoff_time = time.time() - (hours * 3600)
        
        # Get sessions in period
        sessions_in_period = []
        
        for session in list(self.active_sessions.values()):
            if session.start_time > cutoff_time:
                sessions_in_period.append(session)
        
        for session in self.session_history:
            if session.start_time > cutoff_time:
                sessions_in_period.append(session)
        
        # Calculate statistics
        total_sessions = len(sessions_in_period)
        active_sessions = len([s for s in sessions_in_period if s.status == 'active'])
        completed_sessions = len([s for s in sessions_in_period if s.status == 'completed'])
        abandoned_sessions = len([s for s in sessions_in_period if s.status == 'abandoned'])
        
        # Calculate average session duration
        completed_durations = [s.duration for s in sessions_in_period if s.duration]
        avg_duration = sum(completed_durations) / len(completed_durations) if completed_durations else 0
        
        # Calculate peak concurrent sessions
        peak_concurrent = 0
        for session in sessions_in_period:
            concurrent = len([s for s in sessions_in_period 
                             if s.start_time <= session.start_time and 
                             (s.end_time is None or s.end_time >= session.start_time)])
            peak_concurrent = max(peak_concurrent, concurrent)
        
        return {
            'total_sessions': total_sessions,
            'active_sessions': active_sessions,
            'completed_sessions': completed_sessions,
            'abandoned_sessions': abandoned_sessions,
            'completion_rate': completed_sessions / total_sessions if total_sessions > 0 else 0,
            'abandonment_rate': abandoned_sessions / total_sessions if total_sessions > 0 else 0,
            'average_duration_minutes': avg_duration / 60,
            'peak_concurrent_sessions': peak_concurrent,
            'unique_students': len(set(s.student_id for s in sessions_in_period)),
            'unique_labs': len(set(s.lab_id for s in sessions_in_period))
        }

class StudentActivityTracker:
    """Tracks detailed student activity"""
    
    def __init__(self, session_manager: LabSessionManager):
        self.session_manager = session_manager
        self.activity_history = deque(maxlen=50000)
        self.learning_patterns = {}
        
    def track_activity(self, student_id: str, activity_type: str, details: Dict[str, Any],
                      lab_context: str = None, session_id: str = None):
        """Track student activity"""
        activity = StudentActivity(
            student_id=student_id,
            timestamp=time.time(),
            activity_type=activity_type,
            details=details,
            lab_context=lab_context or 'unknown',
            session_id=session_id or 'unknown'
        )
        
        self.activity_history.append(activity)
        
        # Update learning patterns
        self._update_learning_patterns(student_id, activity)
        
        logging.debug(f"Tracked activity: {student_id} - {activity_type}")
    
    def _update_learning_patterns(self, student_id: str, activity: StudentActivity):
        """Update learning patterns for student"""
        if student_id not in self.learning_patterns:
            self.learning_patterns[student_id] = {
                'total_activities': 0,
                'activity_types': defaultdict(int),
                'total_study_time': 0,
                'last_activity': activity.timestamp,
                'average_session_length': 0,
                'preferred_lab_times': defaultdict(int),
                'learning_velocity': 0,
                'engagement_score': 0
            }
        
        pattern = self.learning_patterns[student_id]
        pattern['total_activities'] += 1
        pattern['activity_types'][activity.activity_type] += 1
        pattern['last_activity'] = activity.timestamp
        
        # Update preferred lab times (hour of day)
        hour = datetime.fromtimestamp(activity.timestamp).hour
        pattern['preferred_lab_times'][hour] += 1
        
        # Calculate learning velocity (activities per hour of study time)
        if pattern['total_study_time'] > 0:
            pattern['learning_velocity'] = pattern['total_activities'] / (pattern['total_study_time'] / 3600)
        
        # Calculate engagement score
        self._calculate_engagement_score(student_id)
    
    def _calculate_engagement_score(self, student_id: str):
        """Calculate student engagement score"""
        if student_id not in self.learning_patterns:
            return
        
        pattern = self.learning_patterns[student_id]
        
        # Factors for engagement score
        activity_score = min(pattern['total_activities'] / 100, 1.0)  # Max at 100 activities
        variety_score = len(pattern['activity_types']) / 10  # Max variety at 10 types
        recency_score = min((time.time() - pattern['last_activity']) / (7 * 24 * 3600), 1.0)  # Recency in last week
        
        # Calculate session completion rate from sessions
        sessions = self.session_manager.get_sessions_by_student(student_id)
        if sessions:
            completed_sessions = len([s for s in sessions if s.status == 'completed'])
            completion_score = completed_sessions / len(sessions)
        else:
            completion_score = 0
        
        # Calculate engagement score (0-100)
        engagement_score = (
            activity_score * 0.3 +
            variety_score * 0.2 +
            (1 - recency_score) * 0.2 +
            completion_score * 0.3
        ) * 100
        
        pattern['engagement_score'] = engagement_score
    
    def get_student_activity_summary(self, student_id: str, days: int = 7) -> Dict[str, Any]:
        """Get activity summary for a student"""
        cutoff_time = time.time() - (days * 24 * 3600)
        
        # Get recent activities
        recent_activities = [
            activity for activity in self.activity_history
            if activity.student_id == student_id and activity.timestamp > cutoff_time
        ]
        
        if not recent_activities:
            return {
                'student_id': student_id,
                'period_days': days,
                'total_activities': 0,
                'activity_breakdown': {},
                'engagement_score': 0,
                'learning_pattern': 'insufficient_data'
            }
        
        # Activity breakdown
        activity_breakdown = defaultdict(int)
        for activity in recent_activities:
            activity_breakdown[activity.activity_type] += 1
        
        # Learning pattern
        pattern = self.learning_patterns.get(student_id, {})
        
        # Determine learning pattern
        if pattern.get('engagement_score', 0) > 80:
            learning_pattern = 'highly_engaged'
        elif pattern.get('engagement_score', 0) > 60:
            learning_pattern = 'engaged'
        elif pattern.get('engagement_score', 0) > 40:
            learning_pattern = 'moderately_engaged'
        else:
            learning_pattern = 'low_engagement'
        
        return {
            'student_id': student_id,
            'period_days': days,
            'total_activities': len(recent_activities),
            'activity_breakdown': dict(activity_breakdown),
            'engagement_score': pattern.get('engagement_score', 0),
            'learning_pattern': learning_pattern,
            'total_study_time_minutes': pattern.get('total_study_time', 0) / 60,
            'average_session_length_minutes': pattern.get('average_session_length', 0) / 60,
            'preferred_lab_hours': dict(pattern.get('preferred_lab_times', {})),
            'learning_velocity': pattern.get('learning_velocity', 0)
        }
    
    def get_activity_trends(self, days: int = 30) -> Dict[str, Any]:
        """Get activity trends for all students"""
        cutoff_time = time.time() - (days * 24 * 3600)
        
        # Group activities by day
        daily_activities = defaultdict(int)
        daily_active_students = defaultdict(set)
        activity_types_by_day = defaultdict(lambda: defaultdict(int))
        
        for activity in self.activity_history:
            if activity.timestamp > cutoff_time:
                day = datetime.fromtimestamp(activity.timestamp).strftime('%Y-%m-%d')
                daily_activities[day] += 1
                daily_active_students[day].add(activity.student_id)
                activity_types_by_day[day][activity.activity_type] += 1
        
        # Calculate trends
        sorted_days = sorted(daily_activities.keys())
        
        if len(sorted_days) >= 7:
            recent_week = sorted_days[-7:]
            previous_week = sorted_days[-14:-7] if len(sorted_days) >= 14 else []
            
            recent_avg = sum(daily_activities[day] for day in recent_week) / len(recent_week)
            previous_avg = sum(daily_activities[day] for day in previous_week) / len(previous_week) if previous_week else recent_avg
            
            activity_trend = 'increasing' if recent_avg > previous_avg * 1.1 else 'decreasing' if recent_avg < previous_avg * 0.9 else 'stable'
        else:
            activity_trend = 'insufficient_data'
        
        return {
            'period_days': days,
            'total_activities': sum(daily_activities.values()),
            'daily_activities': dict(daily_activities),
            'daily_active_students': {day: len(students) for day, students in daily_active_students.items()},
            'activity_trend': activity_trend,
            'most_active_day': max(daily_activities.items(), key=lambda x: x[1])[0] if daily_activities else None,
            'peak_activity_hour': self._get_peak_activity_hour(),
            'activity_types_distribution': self._get_activity_types_distribution(days)
        }
    
    def _get_peak_activity_hour(self) -> int:
        """Get the hour with peak activity"""
        hourly_activities = defaultdict(int)
        
        for activity in self.activity_history:
            hour = datetime.fromtimestamp(activity.timestamp).hour
            hourly_activities[hour] += 1
        
        if hourly_activities:
            return max(hourly_activities.items(), key=lambda x: x[1])[0]
        
        return 12  # Default to noon
    
    def _get_activity_types_distribution(self, days: int) -> Dict[str, int]:
        """Get activity types distribution"""
        cutoff_time = time.time() - (days * 24 * 3600)
        distribution = defaultdict(int)
        
        for activity in self.activity_history:
            if activity.timestamp > cutoff_time:
                distribution[activity.activity_type] += 1
        
        return dict(distribution)

class CourseResourceAnalyzer:
    """Analyzes resource utilization by course"""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.course_data = {}
        self.resource_history = deque(maxlen=1000)
        
    def update_course_resources(self, course_id: str, course_name: str, lab_id: str,
                               student_count: int, metrics: Dict[str, float]):
        """Update resource usage for a course"""
        resource_data = CourseResource(
            course_id=course_id,
            course_name=course_name,
            lab_id=lab_id,
            student_count=student_count,
            cpu_usage=metrics.get('cpu_usage', 0),
            memory_usage=metrics.get('memory_usage', 0),
            storage_usage=metrics.get('storage_usage', 0),
            network_usage=metrics.get('network_usage', 0),
            session_duration_avg=metrics.get('session_duration_avg', 0),
            completion_rate=metrics.get('completion_rate', 0)
        )
        
        self.course_data[course_id] = resource_data
        self.resource_history.append({
            'timestamp': time.time(),
            'course_id': course_id,
            **asdict(resource_data)
        })
    
    def get_course_utilization(self, course_id: str) -> Optional[CourseResource]:
        """Get current utilization for a course"""
        return self.course_data.get(course_id)
    
    def get_all_courses(self) -> List[CourseResource]:
        """Get all course resource data"""
        return list(self.course_data.values())
    
    def get_resource_trends(self, course_id: str, hours: int = 24) -> Dict[str, Any]:
        """Get resource trends for a course"""
        cutoff_time = time.time() - (hours * 3600)
        
        course_history = [
            data for data in self.resource_history
            if data['course_id'] == course_id and data['timestamp'] > cutoff_time
        ]
        
        if not course_history:
            return {'error': 'No data available for the specified period'}
        
        # Calculate trends
        cpu_values = [data['cpu_usage'] for data in course_history]
        memory_values = [data['memory_usage'] for data in course_history]
        storage_values = [data['storage_usage'] for data in course_history]
        student_counts = [data['student_count'] for data in course_history]
        
        return {
            'course_id': course_id,
            'period_hours': hours,
            'cpu_usage': {
                'average': sum(cpu_values) / len(cpu_values),
                'peak': max(cpu_values),
                'trend': self._calculate_trend(cpu_values)
            },
            'memory_usage': {
                'average': sum(memory_values) / len(memory_values),
                'peak': max(memory_values),
                'trend': self._calculate_trend(memory_values)
            },
            'storage_usage': {
                'average': sum(storage_values) / len(storage_values),
                'peak': max(storage_values),
                'trend': self._calculate_trend(storage_values)
            },
            'student_count': {
                'average': sum(student_counts) / len(student_counts),
                'peak': max(student_counts),
                'trend': self._calculate_trend(student_counts)
            },
            'data_points': len(course_history)
        }
    
    def _calculate_trend(self, values: List[float]) -> str:
        """Calculate trend direction for a list of values"""
        if len(values) < 3:
            return 'insufficient_data'
        
        # Simple linear trend calculation
        n = len(values)
        x = list(range(n))
        
        # Calculate slope
        x_mean = sum(x) / n
        y_mean = sum(values) / n
        
        numerator = sum((x[i] - x_mean) * (values[i] - y_mean) for i in range(n))
        denominator = sum((x[i] - x_mean) ** 2 for i in range(n))
        
        if denominator == 0:
            return 'stable'
        
        slope = numerator / denominator
        
        # Determine trend
        if slope > 0.1:
            return 'increasing'
        elif slope < -0.1:
            return 'decreasing'
        else:
            return 'stable'
    
    def get_resource_efficiency_score(self, course_id: str) -> Dict[str, Any]:
        """Calculate resource efficiency score for a course"""
        course_resource = self.course_data.get(course_id)
        
        if not course_resource:
            return {'error': 'Course not found'}
        
        # Efficiency factors
        cpu_efficiency = 100 - abs(course_resource.cpu_usage - 70)  # Optimal around 70%
        memory_efficiency = 100 - abs(course_resource.memory_usage - 80)  # Optimal around 80%
        completion_bonus = course_resource.completion_rate * 20  # Up to 20 points for completion
        
        # Overall efficiency score
        efficiency_score = max(0, (cpu_efficiency + memory_efficiency) / 2 + completion_bonus)
        
        return {
            'course_id': course_id,
            'overall_score': efficiency_score,
            'cpu_efficiency': cpu_efficiency,
            'memory_efficiency': memory_efficiency,
            'completion_rate': course_resource.completion_rate,
            'completion_bonus': completion_bonus,
            'recommendations': self._get_efficiency_recommendations(course_resource)
        }
    
    def _get_efficiency_recommendations(self, course_resource: CourseResource) -> List[str]:
        """Get recommendations for improving resource efficiency"""
        recommendations = []
        
        if course_resource.cpu_usage < 50:
            recommendations.append("Consider increasing student capacity or adding more compute-intensive activities")
        elif course_resource.cpu_usage > 85:
            recommendations.append("Consider reducing student load or optimizing resource allocation")
        
        if course_resource.memory_usage < 60:
            recommendations.append("Memory usage is low - consider increasing student capacity")
        elif course_resource.memory_usage > 90:
            recommendations.append("Memory usage is high - consider adding more RAM or reducing student load")
        
        if course_resource.completion_rate < 0.8:
            recommendations.append("Low completion rate - review course difficulty and provide additional support")
        
        if not recommendations:
            recommendations.append("Resource usage is optimal")
        
        return recommendations

class EducationalKPIAnalyzer:
    """Analyzes and calculates educational KPIs"""
    
    def __init__(self, session_manager: LabSessionManager, 
                 activity_tracker: StudentActivityTracker,
                 resource_analyzer: CourseResourceAnalyzer):
        self.session_manager = session_manager
        self.activity_tracker = activity_tracker
        self.resource_analyzer = resource_analyzer
        self.kpis = {}
        
    def calculate_all_kpis(self) -> List[EducationalKPI]:
        """Calculate all educational KPIs"""
        kpis = []
        
        # Lab utilization rate
        utilization_kpi = self._calculate_lab_utilization()
        if utilization_kpi:
            kpis.append(utilization_kpi)
        
        # Student engagement score
        engagement_kpi = self._calculate_student_engagement()
        if engagement_kpi:
            kpis.append(engagement_kpi)
        
        # Resource efficiency
        efficiency_kpi = self._calculate_resource_efficiency()
        if efficiency_kpi:
            kpis.append(efficiency_kpi)
        
        # Learning outcomes
        outcomes_kpi = self._calculate_learning_outcomes()
        if outcomes_kpi:
            kpis.append(outcomes_kpi)
        
        # System reliability
        reliability_kpi = self._calculate_system_reliability()
        if reliability_kpi:
            kpis.append(reliability_kpi)
        
        # Student satisfaction (simulated)
        satisfaction_kpi = self._calculate_student_satisfaction()
        if satisfaction_kpi:
            kpis.append(satisfaction_kpi)
        
        return kpis
    
    def _calculate_lab_utilization(self) -> Optional[EducationalKPI]:
        """Calculate lab utilization rate"""
        session_stats = self.session_manager.get_session_statistics(24)
        
        if session_stats['total_sessions'] == 0:
            return None
        
        # Get lab capacity (simulated)
        total_lab_capacity = 60  # Total seats across all labs
        utilization_rate = (session_stats['peak_concurrent_sessions'] / total_lab_capacity) * 100
        
        return EducationalKPI(
            name='Lab Utilization Rate',
            value=utilization_rate,
            target=75.0,  # Target 75% utilization
            unit='%',
            category='utilization',
            trend=self._calculate_trend('lab_utilization', utilization_rate),
            last_updated=time.time()
        )
    
    def _calculate_student_engagement(self) -> Optional[EducationalKPI]:
        """Calculate average student engagement score"""
        engagement_scores = []
        
        # Get all students with activity
        students = set()
        for activity in self.activity_tracker.activity_history:
            students.add(activity.student_id)
        
        if not students:
            return None
        
        for student_id in students:
            summary = self.activity_tracker.get_student_activity_summary(student_id)
            if summary['total_activities'] > 0:
                engagement_scores.append(summary['engagement_score'])
        
        if not engagement_scores:
            return None
        
        avg_engagement = sum(engagement_scores) / len(engagement_scores)
        
        return EducationalKPI(
            name='Student Engagement Score',
            value=avg_engagement,
            target=80.0,
            unit='score',
            category='engagement',
            trend=self._calculate_trend('student_engagement', avg_engagement),
            last_updated=time.time()
        )
    
    def _calculate_resource_efficiency(self) -> Optional[EducationalKPI]:
        """Calculate resource efficiency score"""
        efficiency_scores = []
        
        for course_id in self.resource_analyzer.course_data.keys():
            efficiency_data = self.resource_analyzer.get_resource_efficiency_score(course_id)
            if 'overall_score' in efficiency_data:
                efficiency_scores.append(efficiency_data['overall_score'])
        
        if not efficiency_scores:
            return None
        
        avg_efficiency = sum(efficiency_scores) / len(efficiency_scores)
        
        return EducationalKPI(
            name='Resource Efficiency',
            value=avg_efficiency,
            target=85.0,
            unit='score',
            category='efficiency',
            trend=self._calculate_trend('resource_efficiency', avg_efficiency),
            last_updated=time.time()
        )
    
    def _calculate_learning_outcomes(self) -> Optional[EducationalKPI]:
        """Calculate learning outcomes score"""
        session_stats = self.session_manager.get_session_statistics(168)  # Last week
        
        if session_stats['total_sessions'] == 0:
            return None
        
        # Learning outcomes based on completion rate and session quality
        completion_score = session_stats['completion_rate'] * 100
        quality_score = min(session_stats['average_duration_minutes'] / 120 * 100, 100)  # Quality if avg duration > 2 hours
        
        learning_outcomes = (completion_score * 0.7 + quality_score * 0.3)
        
        return EducationalKPI(
            name='Learning Outcomes Score',
            value=learning_outcomes,
            target=85.0,
            unit='score',
            category='outcomes',
            trend=self._calculate_trend('learning_outcomes', learning_outcomes),
            last_updated=time.time()
        )
    
    def _calculate_system_reliability(self) -> Optional[EducationalKPI]:
        """Calculate system reliability score"""
        # Simulate system reliability based on error rates
        import random
        error_rate = random.uniform(0.1, 2.0)  # 0.1% to 2% error rate
        
        reliability_score = 100 - error_rate
        
        return EducationalKPI(
            name='System Reliability',
            value=reliability_score,
            target=99.5,
            unit='%',
            category='reliability',
            trend=self._calculate_trend('system_reliability', reliability_score),
            last_updated=time.time()
        )
    
    def _calculate_student_satisfaction(self) -> Optional[EducationalKPI]:
        """Calculate student satisfaction score (simulated)"""
        # This would be based on actual survey data
        import random
        satisfaction_score = random.uniform(3.8, 4.5)  # Out of 5.0
        
        # Convert to percentage
        satisfaction_percent = (satisfaction_score / 5.0) * 100
        
        return EducationalKPI(
            name='Student Satisfaction',
            value=satisfaction_percent,
            target=85.0,
            unit='%',
            category='satisfaction',
            trend=self._calculate_trend('student_satisfaction', satisfaction_percent),
            last_updated=time.time()
        )
    
    def _calculate_trend(self, kpi_name: str, current_value: float) -> str:
        """Calculate KPI trend (simplified)"""
        # This would compare with historical data
        # For now, simulate based on current value vs target
        if current_value > 100:  # Above target
            return 'improving'
        elif current_value < 90:  # Below target
            return 'declining'
        else:
            return 'stable'
    
    def get_kpi_dashboard(self) -> Dict[str, Any]:
        """Get KPI dashboard data"""
        kpis = self.calculate_all_kpis()
        
        dashboard_data = {
            'timestamp': time.time(),
            'kpis': [asdict(kpi) for kpi in kpis],
            'summary': {
                'total_kpis': len(kpis),
                'kpis_on_target': len([kpi for kpi in kpis if kpi.value >= kpi.target]),
                'kpis_improving': len([kpi for kpi in kpis if kpi.trend == 'improving']),
                'kpis_declining': len([kpi for kpi in kpis if kpi.trend == 'declining'])
            }
        }
        
        return dashboard_data

class EducationalAnalyticsEngine:
    """Main educational analytics engine"""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.session_manager = LabSessionManager(config)
        self.activity_tracker = StudentActivityTracker(self.session_manager)
        self.resource_analyzer = CourseResourceAnalyzer(config)
        self.kpi_analyzer = EducationalKPIAnalyzer(
            self.session_manager, self.activity_tracker, self.resource_analyzer
        )
        
        self.is_running = False
        self.update_interval = config.get('update_interval', 300)  # 5 minutes
        
    def start(self):
        """Start the analytics engine"""
        self.is_running = True
        
        # Start background updates
        self.update_thread = threading.Thread(target=self._update_loop)
        self.update_thread.daemon = True
        self.update_thread.start()
        
        # Start activity simulation
        self.simulate_activities()
        
        logging.info("Educational analytics engine started")
    
    def stop(self):
        """Stop the analytics engine"""
        self.is_running = False
        logging.info("Educational analytics engine stopped")
    
    def _update_loop(self):
        """Background update loop"""
        while self.is_running:
            try:
                self._update_course_resources()
                time.sleep(self.update_interval)
            except Exception as e:
                logging.error(f"Error in analytics update loop: {e}")
                time.sleep(10)
    
    def _update_course_resources(self):
        """Update course resource utilization"""
        # Get current system metrics
        cpu_usage = psutil.cpu_percent(interval=1)
        memory = psutil.virtual_memory()
        memory_usage = memory.percent
        
        disk = psutil.disk_usage('/')
        disk_usage = (disk.used / disk.total) * 100
        
        # Simulate course data
        courses = [
            ('CS101', 'Computer Science Fundamentals', 'lab_1'),
            ('CS201', 'Data Structures', 'lab_2'),
            ('CS301', 'Operating Systems', 'lab_3')
        ]
        
        import random
        for course_id, course_name, lab_id in courses:
            student_count = random.randint(25, 50)
            
            metrics = {
                'cpu_usage': cpu_usage + random.uniform(-10, 10),
                'memory_usage': memory_usage + random.uniform(-5, 5),
                'storage_usage': disk_usage + random.uniform(-2, 2),
                'network_usage': random.uniform(10, 30),
                'session_duration_avg': random.uniform(90, 180),  # minutes
                'completion_rate': random.uniform(0.8, 0.95)
            }
            
            self.resource_analyzer.update_course_resources(
                course_id, course_name, lab_id, student_count, metrics
            )
    
    def simulate_activities(self):
        """Simulate student activities for demonstration"""
        def simulate_student_activity():
            import random
            
            while self.is_running:
                # Simulate student login
                student_id = f"student_{random.randint(1, 100)}"
                session_id = self.session_manager.start_session(
                    username=f"user{student_id.split('_')[1]}",
                    student_id=student_id,
                    lab_id=f"lab_{random.randint(1, 3)}",
                    ip_address=f"192.168.1.{random.randint(100, 200)}",
                    terminal=f"pts/{random.randint(0, 9)}"
                )
                
                # Track activities during session
                activities = ['login', 'file_access', 'application_use', 'network_activity', 'logout']
                
                for activity in activities:
                    time.sleep(random.randint(5, 30))  # Random intervals
                    
                    self.activity_tracker.track_activity(
                        student_id=student_id,
                        activity_type=activity,
                        details={'resource': f'lab_resource_{random.randint(1, 10)}'},
                        session_id=session_id
                    )
                
                # End session
                time.sleep(random.randint(10, 60))
                self.session_manager.end_session(session_id, random.choice(['completed', 'abandoned']))
        
        # Start simulation in background
        simulation_thread = threading.Thread(target=simulate_student_activity)
        simulation_thread.daemon = True
        simulation_thread.start()
    
    def get_comprehensive_report(self, hours: int = 24) -> Dict[str, Any]:
        """Get comprehensive educational analytics report"""
        report = {
            'timestamp': time.time(),
            'period_hours': hours,
            'session_statistics': self.session_manager.get_session_statistics(hours),
            'activity_trends': self.activity_tracker.get_activity_trends(hours),
            'course_resources': self.resource_analyzer.get_all_courses(),
            'kpi_dashboard': self.kpi_analyzer.get_kpi_dashboard(),
            'top_performing_courses': self._get_top_performing_courses(),
            'resource_efficiency_by_course': self._get_resource_efficiency_by_course(),
            'student_engagement_insights': self._get_student_engagement_insights(),
            'recommendations': self._generate_recommendations()
        }
        
        return report
    
    def _get_top_performing_courses(self) -> List[Dict[str, Any]]:
        """Get top performing courses by various metrics"""
        courses = self.resource_analyzer.get_all_courses()
        
        # Sort by completion rate and efficiency
        courses_with_scores = []
        for course in courses:
            efficiency_data = self.resource_analyzer.get_resource_efficiency_score(course.course_id)
            if 'overall_score' in efficiency_data:
                score = (course.completion_rate * 100 + efficiency_data['overall_score']) / 2
                courses_with_scores.append({
                    'course_id': course.course_id,
                    'course_name': course.course_name,
                    'completion_rate': course.completion_rate,
                    'efficiency_score': efficiency_data['overall_score'],
                    'combined_score': score,
                    'student_count': course.student_count
                })
        
        # Sort by combined score
        courses_with_scores.sort(key=lambda x: x['combined_score'], reverse=True)
        
        return courses_with_scores[:5]
    
    def _get_resource_efficiency_by_course(self) -> Dict[str, Dict[str, Any]]:
        """Get resource efficiency data for all courses"""
        efficiency_data = {}
        
        for course_id in self.resource_analyzer.course_data.keys():
            course_efficiency = self.resource_analyzer.get_resource_efficiency_score(course_id)
            efficiency_data[course_id] = course_efficiency
        
        return efficiency_data
    
    def _get_student_engagement_insights(self) -> Dict[str, Any]:
        """Get insights about student engagement"""
        # Get engagement scores for all students
        engagement_scores = []
        
        students = set()
        for activity in self.activity_tracker.activity_history:
            students.add(activity.student_id)
        
        for student_id in students:
            summary = self.activity_tracker.get_student_activity_summary(student_id, 7)
            if summary['total_activities'] > 0:
                engagement_scores.append(summary['engagement_score'])
        
        if not engagement_scores:
            return {'error': 'No engagement data available'}
        
        return {
            'average_engagement': sum(engagement_scores) / len(engagement_scores),
            'engagement_distribution': {
                'high': len([s for s in engagement_scores if s > 80]),
                'medium': len([s for s in engagement_scores if 60 < s <= 80]),
                'low': len([s for s in engagement_scores if s <= 60])
            },
            'engagement_trend': self._calculate_engagement_trend(engagement_scores)
        }
    
    def _calculate_engagement_trend(self, scores: List[float]) -> str:
        """Calculate overall engagement trend"""
        if len(scores) < 10:
            return 'insufficient_data'
        
        # Simple trend calculation
        recent_scores = scores[-10:]
        earlier_scores = scores[-20:-10] if len(scores) >= 20 else scores[:10]
        
        recent_avg = sum(recent_scores) / len(recent_scores)
        earlier_avg = sum(earlier_scores) / len(earlier_scores)
        
        if recent_avg > earlier_avg * 1.05:
            return 'improving'
        elif recent_avg < earlier_avg * 0.95:
            return 'declining'
        else:
            return 'stable'
    
    def _generate_recommendations(self) -> List[str]:
        """Generate actionable recommendations"""
        recommendations = []
        
        # Get session statistics
        session_stats = self.session_manager.get_session_statistics(24)
        
        # Check abandonment rate
        if session_stats['abandonment_rate'] > 0.2:
            recommendations.append(
                f"High abandonment rate ({session_stats['abandonment_rate']:.1%}). "
                "Consider reviewing course difficulty and providing additional support."
            )
        
        # Check lab utilization
        if session_stats['peak_concurrent_sessions'] < 30:
            recommendations.append(
                "Low lab utilization. Consider marketing more courses or adjusting lab schedules."
            )
        
        # Check average session duration
        if session_stats['average_duration_minutes'] < 60:
            recommendations.append(
                f"Short average session duration ({session_stats['average_duration_minutes']:.1f} min). "
                "Consider adding more engaging activities."
            )
        
        # Get resource efficiency data
        for course_id in self.resource_analyzer.course_data.keys():
            efficiency_data = self.resource_analyzer.get_resource_efficiency_score(course_id)
            if efficiency_data.get('overall_score', 0) < 70:
                course_resource = self.resource_analyzer.get_course_utilization(course_id)
                if course_resource:
                    recommendations.extend([
                        f"Low efficiency score for {course_resource.course_name}. "
                        "Consider optimizing resource allocation."
                    ])
        
        if not recommendations:
            recommendations.append("All metrics are performing well. Continue monitoring.")
        
        return recommendations

def main():
    """Main function to run educational analytics"""
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )
    
    # Load configuration
    config_file = Path(__file__).parent.parent / 'config' / 'monitoring.yaml'
    
    if config_file.exists():
        with open(config_file, 'r') as f:
            config = yaml.safe_load(f)
    else:
        config = {
            'educational_analytics': {
                'enabled': True,
                'update_interval': 300,
                'anonymization': True,
                'privacy_mode': 'strict'
            }
        }
    
    # Create and start analytics engine
    engine = EducationalAnalyticsEngine(config.get('educational_analytics', {}))
    
    try:
        # Start analytics
        engine.start()
        
        # Generate periodic reports
        while True:
            time.sleep(600)  # Generate report every 10 minutes
            
            report = engine.get_comprehensive_report(24)
            logging.info(f"Generated educational analytics report with {len(report['kpi_dashboard']['kpis'])} KPIs")
            
    except KeyboardInterrupt:
        logging.info("Shutting down educational analytics engine...")
        engine.stop()

if __name__ == '__main__':
    main()