"""
Resource Allocation and Scheduling Tools
"""

import json
import logging
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any
from pathlib import Path

from ..core.models import ResourceSchedule

class ResourceScheduler:
    """Manager for resource allocation and scheduling"""
    
    def __init__(self):
        self.schedules = {}
        self.resources = {}
        self.logger = logging.getLogger(__name__)
        
        self._setup_directories()
        self._initialize_resources()
    
    def _setup_directories(self) -> None:
        """Create scheduler directories"""
        directories = [
            "/var/lib/multios-enterprise/scheduler",
            "/var/lib/multios-enterprise/scheduler/schedules",
            "/var/lib/multios-enterprise/scheduler/resources"
        ]
        
        for directory in directories:
            Path(directory).mkdir(parents=True, exist_ok=True)
    
    def _initialize_resources(self) -> None:
        """Initialize available resources"""
        # Lab resources
        for i in range(1, 11):  # 10 labs
            lab_id = f"LAP-{i:03d}"
            self.resources[lab_id] = {
                'resource_id': lab_id,
                'resource_type': 'lab',
                'name': f'Computer Lab {i}',
                'capacity': 30,
                'location': f'Building A, Floor {(i-1)//3 + 1}',
                'available_hours': '08:00-18:00',
                'equipment': ['computers', 'projector', 'whiteboard']
            }
        
        # System resources
        for i in range(1, 101):  # 100 systems
            system_id = f"SYS-{i:03d}"
            self.resources[system_id] = {
                'resource_id': system_id,
                'resource_type': 'system',
                'name': f'Workstation {i}',
                'specs': 'Intel i5, 8GB RAM, 500GB SSD',
                'location': f'Lab {((i-1)//10) + 1}',
                'available_hours': '08:00-18:00'
            }
        
        # Software resources
        software_list = [
            'MATLAB', 'AutoCAD', 'Adobe Creative Suite', 'SolidWorks',
            'LabVIEW', 'Visual Studio', 'VMware Workstation'
        ]
        
        for software in software_list:
            resource_id = f"SOFT-{software.replace(' ', '-').upper()}"
            self.resources[resource_id] = {
                'resource_id': resource_id,
                'resource_type': 'software',
                'name': software,
                'license_count': 50,
                'usage_tracking': True
            }
    
    def schedule_resource(self, resource_type: str, resource_id: str, start_time: datetime,
                         end_time: datetime, user_id: str, purpose: str,
                         priority: int = 1, reservation_notes: Optional[str] = None) -> Optional[str]:
        """Schedule a resource reservation"""
        try:
            # Check resource availability
            if not self._is_resource_available(resource_id, start_time, end_time):
                self.logger.error(f"Resource {resource_id} not available for requested time")
                return None
            
            # Create schedule entry
            schedule_id = f"SCH-{datetime.now().strftime('%Y%m%d_%H%M%S')}-{len(self.schedules)}"
            
            resource_schedule = ResourceSchedule(
                schedule_id=schedule_id,
                resource_type=resource_type,
                resource_id=resource_id,
                start_time=start_time,
                end_time=end_time,
                user_id=user_id,
                purpose=purpose,
                status='scheduled',
                priority=priority,
                reservation_notes=reservation_notes
            )
            
            # Store schedule
            self.schedules[schedule_id] = resource_schedule
            
            # Save to storage
            self._save_schedule(resource_schedule)
            
            self.logger.info(f"Scheduled resource {resource_id} for user {user_id}")
            return schedule_id
            
        except Exception as e:
            self.logger.error(f"Failed to schedule resource: {e}")
            return None
    
    def schedule_lab_session(self, template_id: str, site_id: str, 
                           start_time: datetime, duration_minutes: int,
                           instructor_id: str, student_count: int) -> Optional[str]:
        """Schedule a lab session"""
        try:
            end_time = start_time + timedelta(minutes=duration_minutes)
            
            # Find available lab with sufficient capacity
            available_lab = self._find_available_lab(student_count, start_time, end_time)
            if not available_lab:
                self.logger.error("No available lab found for requested capacity and time")
                return None
            
            # Schedule lab and required systems
            lab_schedule_id = self.schedule_resource(
                'lab', available_lab['resource_id'], start_time, end_time,
                instructor_id, f"Lab session with template {template_id}"
            )
            
            if not lab_schedule_id:
                return None
            
            # Schedule individual systems
            system_schedules = []
            for i in range(student_count):
                system_id = f"SYS-{(i%100)+1:03d}"  # Distribute across systems
                system_schedule_id = self.schedule_resource(
                    'system', system_id, start_time, end_time,
                    instructor_id, f"Student workstation for lab session"
                )
                if system_schedule_id:
                    system_schedules.append(system_schedule_id)
            
            # Create consolidated session record
            session_record = {
                'session_id': lab_schedule_id,
                'template_id': template_id,
                'lab_schedule_id': lab_schedule_id,
                'system_schedule_ids': system_schedules,
                'site_id': site_id,
                'start_time': start_time.isoformat(),
                'end_time': end_time.isoformat(),
                'instructor_id': instructor_id,
                'student_count': student_count,
                'status': 'scheduled'
            }
            
            self._save_lab_session(session_record)
            
            self.logger.info(f"Scheduled lab session for {student_count} students")
            return lab_schedule_id
            
        except Exception as e:
            self.logger.error(f"Failed to schedule lab session: {e}")
            return None
    
    def get_resource_schedule(self, resource_id: str, start_date: datetime, 
                            end_date: datetime) -> List[ResourceSchedule]:
        """Get schedule for a specific resource"""
        schedules = []
        
        for schedule in self.schedules.values():
            if (schedule.resource_id == resource_id and
                schedule.start_time >= start_date and
                schedule.end_time <= end_date):
                schedules.append(schedule)
        
        return schedules
    
    def get_user_schedule(self, user_id: str, start_date: datetime, 
                         end_date: datetime) -> List[ResourceSchedule]:
        """Get schedule for a specific user"""
        schedules = []
        
        for schedule in self.schedules.values():
            if (schedule.user_id == user_id and
                schedule.start_time >= start_date and
                schedule.end_time <= end_date):
                schedules.append(schedule)
        
        return schedules
    
    def cancel_schedule(self, schedule_id: str) -> bool:
        """Cancel a scheduled reservation"""
        try:
            if schedule_id in self.schedules:
                schedule = self.schedules[schedule_id]
                schedule.status = 'cancelled'
                
                # Update storage
                self._save_schedule(schedule)
                
                self.logger.info(f"Cancelled schedule {schedule_id}")
                return True
            return False
            
        except Exception as e:
            self.logger.error(f"Failed to cancel schedule {schedule_id}: {e}")
            return False
    
    def _is_resource_available(self, resource_id: str, start_time: datetime, 
                              end_time: datetime) -> bool:
        """Check if a resource is available for the requested time"""
        for schedule in self.schedules.values():
            if (schedule.resource_id == resource_id and 
                schedule.status != 'cancelled'):
                
                # Check for time overlap
                if (schedule.start_time < end_time and schedule.end_time > start_time):
                    return False
        
        return True
    
    def _find_available_lab(self, required_capacity: int, start_time: datetime, 
                           end_time: datetime) -> Optional[Dict[str, Any]]:
        """Find an available lab with sufficient capacity"""
        labs = [r for r in self.resources.values() if r['resource_type'] == 'lab']
        
        # Sort by capacity (smallest sufficient first)
        suitable_labs = [lab for lab in labs if lab['capacity'] >= required_capacity]
        suitable_labs.sort(key=lambda x: x['capacity'])
        
        for lab in suitable_labs:
            if self._is_resource_available(lab['resource_id'], start_time, end_time):
                return lab
        
        return None
    
    def _save_schedule(self, resource_schedule: ResourceSchedule) -> None:
        """Save schedule to storage"""
        try:
            schedule_file = Path("/var/lib/multios-enterprise/scheduler/schedules") / f"{resource_schedule.schedule_id}.json"
            
            schedule_data = {
                'schedule_id': resource_schedule.schedule_id,
                'resource_type': resource_schedule.resource_type,
                'resource_id': resource_schedule.resource_id,
                'start_time': resource_schedule.start_time.isoformat(),
                'end_time': resource_schedule.end_time.isoformat(),
                'user_id': resource_schedule.user_id,
                'purpose': resource_schedule.purpose,
                'status': resource_schedule.status,
                'priority': resource_schedule.priority,
                'reservation_notes': resource_schedule.reservation_notes
            }
            
            with open(schedule_file, 'w') as f:
                json.dump(schedule_data, f, indent=2)
                
        except Exception as e:
            self.logger.error(f"Failed to save schedule: {e}")
    
    def _save_lab_session(self, session: Dict[str, Any]) -> None:
        """Save lab session record"""
        try:
            session_file = Path("/var/lib/multios-enterprise/scheduler/sessions") / f"{session['session_id']}.json"
            
            with open(session_file, 'w') as f:
                json.dump(session, f, indent=2)
                
        except Exception as e:
            self.logger.error(f"Failed to save lab session: {e}")
    
    def schedule_maintenance(self, maintenance_data: Dict[str, Any]) -> bool:
        """Schedule system maintenance"""
        try:
            maintenance_id = f"MAINT-{datetime.now().strftime('%Y%m%d_%H%M%S')}"
            
            maintenance_record = {
                'maintenance_id': maintenance_id,
                'system_ids': maintenance_data['system_ids'],
                'start_time': maintenance_data['start_time'],
                'duration_minutes': maintenance_data['duration_minutes'],
                'status': 'scheduled',
                'maintenance_type': maintenance_data.get('maintenance_type', 'routine'),
                'description': maintenance_data.get('description', '')
            }
            
            # Schedule each system
            for system_id in maintenance_data['system_ids']:
                start_time = datetime.fromisoformat(maintenance_data['start_time'])
                end_time = start_time + timedelta(minutes=maintenance_data['duration_minutes'])
                
                self.schedule_resource(
                    'system', system_id, start_time, end_time,
                    'system_admin', 'Scheduled maintenance'
                )
            
            self.logger.info(f"Scheduled maintenance for {len(maintenance_data['system_ids'])} systems")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to schedule maintenance: {e}")
            return False
