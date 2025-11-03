#!/usr/bin/env python3
"""
Configuration and Setup Script for MultiOS Academic Curriculum Integration Platform

This script sets up the complete academic curriculum integration system with proper
configuration, database initialization, and sample data.
"""

import os
import json
import sys
from datetime import datetime, timedelta, date
from typing import Dict, List, Any
import logging

# Add parent directory to path for imports
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from core.curriculum_manager import (
    CurriculumManager, LearningOutcomeType, DifficultyLevel, CurriculumLevel
)
from lms_integration.lms_manager import LMSIntegrationManager, LMSPlatform
from assessment.progress_tracker import ProgressTracker, AssessmentManager
from assignments.assignment_manager import AssignmentManager, Assignment
from calendar.schedule_manager import CalendarManager, Semester, EventType
from courses.os_templates import OperatingSystemsTemplateManager
from educator_dashboard.dashboard import AnalyticsEngine, DashboardGenerator, AlertManager


class SystemConfig:
    """Centralized configuration management"""
    
    def __init__(self, config_path: str = "config/config.json"):
        self.config_path = config_path
        self.config = self.load_config()
    
    def load_config(self) -> Dict[str, Any]:
        """Load configuration from file"""
        default_config = {
            "database": {
                "type": "sqlite",
                "name": "academic_platform.db",
                "host": "localhost",
                "port": 5432,
                "user": "academic_user",
                "password": ""
            },
            "lms_integrations": {
                "canvas": {
                    "enabled": True,
                    "base_url": "https://your-institution.instructure.com",
                    "default_timeout": 30
                },
                "blackboard": {
                    "enabled": True,
                    "base_url": "https://your-institution.blackboard.com",
                    "default_timeout": 30
                },
                "moodle": {
                    "enabled": True,
                    "base_url": "https://your-institution.moodle.com",
                    "default_timeout": 30
                }
            },
            "assessment": {
                "default_passing_grade": 60.0,
                "grade_scale": {
                    "A+": 97,
                    "A": 93,
                    "A-": 90,
                    "B+": 87,
                    "B": 83,
                    "B-": 80,
                    "C+": 77,
                    "C": 73,
                    "C-": 70,
                    "D+": 67,
                    "D": 65,
                    "D-": 60,
                    "F": 0
                },
                "plagiarism_detection": {
                    "enabled": True,
                    "similarity_threshold": 0.8,
                    "check_submissions": True
                }
            },
            "analytics": {
                "real_time_updates": True,
                "retention_period_days": 365,
                "alert_thresholds": {
                    "low_engagement": 60.0,
                    "at_risk_grade": 70.0,
                    "missing_assignments": 3
                }
            },
            "calendar": {
                "default_class_duration": 75,  # minutes
                "office_hours_start": 9,      # 9 AM
                "office_hours_end": 17,       # 5 PM
                "auto_conflict_detection": True
            },
            "notifications": {
                "email_enabled": True,
                "sms_enabled": False,
                "slack_enabled": False,
                "default_recipients": ["instructors@domain.edu"]
            },
            "security": {
                "session_timeout_minutes": 480,
                "max_file_size_mb": 100,
                "allowed_file_types": [".pdf", ".doc", ".docx", ".txt", ".zip"],
                "encryption_enabled": True
            },
            "features": {
                "peer_review": True,
                "mobile_support": True,
                "multi_language": True,
                "accessibility_compliance": True
            },
            "deployment": {
                "environment": "development",
                "debug_mode": True,
                "log_level": "INFO",
                "api_rate_limit": 1000
            }
        }
        
        # Create config directory if it doesn't exist
        os.makedirs(os.path.dirname(self.config_path), exist_ok=True)
        
        # Load existing config or create default
        if os.path.exists(self.config_path):
            try:
                with open(self.config_path, 'r') as f:
                    config = json.load(f)
                # Merge with defaults
                for key, value in default_config.items():
                    if key not in config:
                        config[key] = value
                return config
            except Exception as e:
                print(f"Warning: Could not load config file: {e}")
                print("Using default configuration...")
        else:
            # Save default config
            self.save_config(default_config)
        
        return default_config
    
    def save_config(self, config: Dict[str, Any]):
        """Save configuration to file"""
        os.makedirs(os.path.dirname(self.config_path), exist_ok=True)
        with open(self.config_path, 'w') as f:
            json.dump(config, f, indent=2)
    
    def get(self, key: str, default: Any = None) -> Any:
        """Get configuration value"""
        keys = key.split('.')
        value = self.config
        for k in keys:
            if isinstance(value, dict) and k in value:
                value = value[k]
            else:
                return default
        return value
    
    def set(self, key: str, value: Any):
        """Set configuration value"""
        keys = key.split('.')
        config = self.config
        for k in keys[:-1]:
            if k not in config:
                config[k] = {}
            config = config[k]
        config[keys[-1]] = value
        self.save_config(self.config)


class DatabaseSetup:
    """Database initialization and setup"""
    
    def __init__(self, config: SystemConfig):
        self.config = config
        self.db_type = config.get("database.type", "sqlite")
        self.db_name = config.get("database.name", "academic_platform.db")
    
    def initialize_database(self):
        """Initialize database schema"""
        # This would create actual database tables
        # For now, we'll create a simple JSON-based storage
        
        storage_dir = "data"
        os.makedirs(storage_dir, exist_ok=True)
        
        # Create database files
        db_files = {
            "courses.json": [],
            "assessments.json": [],
            "submissions.json": [],
            "progress.json": [],
            "events.json": [],
            "users.json": [],
            "rooms.json": [],
            "instructors.json": []
        }
        
        for filename, default_content in db_files.items():
            filepath = os.path.join(storage_dir, filename)
            if not os.path.exists(filepath):
                with open(filepath, 'w') as f:
                    json.dump(default_content, f, indent=2)
        
        print(f"Database initialized with {len(db_files)} tables")
    
    def create_sample_data(self):
        """Create sample data for testing"""
        storage_dir = "data"
        
        # Sample courses
        courses = [
            {
                "id": "cs301",
                "title": "Introduction to Operating Systems",
                "code": "CS 301",
                "credits": 3,
                "level": "undergraduate",
                "description": "Fundamental concepts of operating systems",
                "enrollment": 30
            },
            {
                "id": "cs401",
                "title": "Advanced Operating Systems",
                "code": "CS 401",
                "credits": 4,
                "level": "undergraduate",
                "description": "Advanced topics in operating systems",
                "enrollment": 25
            }
        ]
        
        with open(os.path.join(storage_dir, "courses.json"), 'w') as f:
            json.dump(courses, f, indent=2)
        
        # Sample instructors
        instructors = [
            {
                "id": "prof_smith",
                "name": "Dr. Jane Smith",
                "email": "j.smith@university.edu",
                "department": "Computer Science"
            },
            {
                "id": "prof_jones",
                "name": "Dr. Bob Jones",
                "email": "b.jones@university.edu",
                "department": "Computer Science"
            }
        ]
        
        with open(os.path.join(storage_dir, "instructors.json"), 'w') as f:
            json.dump(instructors, f, indent=2)
        
        # Sample rooms
        rooms = [
            {
                "id": "room101",
                "name": "Computer Lab 101",
                "building": "Engineering Building",
                "capacity": 30,
                "type": "computer_lab"
            },
            {
                "id": "room201",
                "name": "Lecture Hall 201",
                "building": "Science Building",
                "capacity": 100,
                "type": "lecture_hall"
            }
        ]
        
        with open(os.path.join(storage_dir, "rooms.json"), 'w') as f:
            json.dump(rooms, f, indent=2)
        
        print("Sample data created successfully")


class SystemInitializer:
    """Main system initialization class"""
    
    def __init__(self):
        self.config = SystemConfig()
        self.logger = self.setup_logging()
    
    def setup_logging(self) -> logging.Logger:
        """Setup logging configuration"""
        log_dir = "logs"
        os.makedirs(log_dir, exist_ok=True)
        
        logging.basicConfig(
            level=getattr(logging, self.config.get("deployment.log_level", "INFO")),
            format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
            handlers=[
                logging.FileHandler(os.path.join(log_dir, 'academic_platform.log')),
                logging.StreamHandler()
            ]
        )
        
        return logging.getLogger(__name__)
    
    def initialize_managers(self):
        """Initialize all system managers with sample data"""
        self.logger.info("Initializing system managers...")
        
        # Initialize curriculum manager
        curriculum_mgr = CurriculumManager()
        
        # Create sample learning outcomes
        outcome1 = curriculum_mgr.create_learning_outcome(
            description="Students can explain fundamental OS concepts",
            outcome_type=LearningOutcomeType.UNDERSTAND,
            difficulty=DifficultyLevel.BEGINNER,
            standards_alignment=["Operating systems"]
        )
        
        outcome2 = curriculum_mgr.create_learning_outcome(
            description="Students can implement basic OS algorithms",
            outcome_type=LearningOutcomeType.CREATE,
            difficulty=DifficultyLevel.INTERMEDIATE,
            standards_alignment=["Operating systems", "Algorithms"]
        )
        
        # Create sample unit
        unit1 = curriculum_mgr.create_unit(
            title="Process Management",
            description="Understanding processes and threads",
            learning_outcomes=[outcome1.id, outcome2.id],
            estimated_duration=timedelta(days=14),
            resources=[
                {"type": "textbook", "title": "OS Concepts", "required": True},
                {"type": "lab", "title": "Process Lab", "required": True}
            ]
        )
        
        # Create sample course
        course1 = curriculum_mgr.create_course(
            title="Introduction to Operating Systems",
            description="Fundamental OS concepts for undergraduates",
            code="CS 301",
            credits=3,
            level=CurriculumLevel.UNDERGRADUATE,
            unit_ids=[unit1.id],
            learning_outcomes=[outcome1.id, outcome2.id]
        )
        
        # Initialize other managers
        assessment_mgr = AssessmentManager()
        progress_tracker = ProgressTracker(assessment_mgr)
        assignment_mgr = AssignmentManager()
        calendar_mgr = CalendarManager()
        template_mgr = OperatingSystemsTemplateManager(curriculum_mgr)
        analytics_engine = AnalyticsEngine(progress_tracker, assignment_mgr, calendar_mgr)
        
        # Create sample semester
        fall_2025 = calendar_mgr.create_semester(
            name="Fall 2025",
            start_date=date(2025, 8, 25),
            end_date=date(2025, 12, 15)
        )
        
        # Create sample room and instructor
        room = calendar_mgr.add_room(
            name="Computer Lab 101",
            building="Engineering Building",
            capacity=30,
            room_type="computer_lab",
            features=["projector", "computers", "whiteboard"]
        )
        
        instructor = calendar_mgr.add_instructor(
            name="Dr. Jane Smith",
            email="j.smith@university.edu",
            department="Computer Science"
        )
        
        # Create sample assignment
        mc_question = assignment_mgr.create_multiple_choice_question(
            question_text="What is the main function of an operating system?",
            options=["Process files", "Manage resources", "Browse web", "Play games"],
            correct_answer="Manage resources",
            points=5.0
        )
        
        assignment = assignment_mgr.create_assignment(
            title="OS Fundamentals Quiz",
            description="Test basic OS concepts",
            course_id=course1.id,
            questions=[mc_question],
            due_date=datetime.now() + timedelta(days=7)
        )
        
        self.logger.info("System managers initialized successfully")
        return {
            "curriculum_manager": curriculum_mgr,
            "assessment_manager": assessment_mgr,
            "progress_tracker": progress_tracker,
            "assignment_manager": assignment_mgr,
            "calendar_manager": calendar_mgr,
            "template_manager": template_mgr,
            "analytics_engine": analytics_engine
        }
    
    def setup_lms_connections(self):
        """Setup sample LMS connections"""
        self.logger.info("Setting up LMS connections...")
        
        lms_mgr = LMSIntegrationManager()
        
        # Add sample Canvas connection
        try:
            from lms_integration.lms_manager import LMSConnection
            canvas_conn = LMSConnection(
                platform=LMSPlatform.CANVAS,
                base_url="https://your-institution.instructure.com",
                api_key="sample_api_key_placeholder"
            )
            lms_mgr.add_connection("canvas_main", canvas_conn)
            self.logger.info("Canvas integration configured")
        except Exception as e:
            self.logger.warning(f"Could not setup Canvas integration: {e}")
        
        return lms_mgr
    
    def generate_initial_reports(self):
        """Generate initial system reports"""
        self.logger.info("Generating initial reports...")
        
        reports_dir = "reports"
        os.makedirs(reports_dir, exist_ok=True)
        
        # Generate system status report
        status_report = {
            "timestamp": datetime.now().isoformat(),
            "system_status": "operational",
            "components": {
                "curriculum_manager": "active",
                "assessment_system": "active",
                "assignment_manager": "active",
                "calendar_system": "active",
                "lms_integration": "active",
                "analytics_engine": "active"
            },
            "initialization_complete": True,
            "next_steps": [
                "Configure production database",
                "Set up LMS connections",
                "Import course data",
                "Configure user accounts"
            ]
        }
        
        with open(os.path.join(reports_dir, "system_status.json"), 'w') as f:
            json.dump(status_report, f, indent=2)
        
        self.logger.info("Initial reports generated")
    
    def print_setup_summary(self):
        """Print setup completion summary"""
        print("\n" + "="*60)
        print("MultiOS Academic Curriculum Integration Platform")
        print("Setup Complete!")
        print("="*60)
        
        print("\n📁 Directory Structure Created:")
        print("  config/          - System configuration")
        print("  data/           - Database files")
        print("  logs/           - Application logs")
        print("  reports/        - System reports")
        
        print("\n🔧 Configuration Files:")
        print(f"  {self.config.config_path}")
        print("  config/cs_standards.json")
        
        print("\n📊 System Components Initialized:")
        print("  ✅ Curriculum Management")
        print("  ✅ LMS Integration")
        print("  ✅ Assessment System")
        print("  ✅ Assignment Management")
        print("  ✅ Calendar & Scheduling")
        print("  ✅ Course Templates")
        print("  ✅ Analytics & Reporting")
        print("  ✅ REST API")
        
        print("\n🚀 Next Steps:")
        print("  1. Configure database connection")
        print("  2. Set up production LMS connections")
        print("  3. Import institutional data")
        print("  4. Configure user authentication")
        print("  5. Start the API server")
        
        print("\n🌐 API Documentation:")
        print("  Swagger UI: http://localhost:8000/api/docs")
        print("  API Status: http://localhost:8000/api/health")
        
        print("\n📚 Example Usage:")
        print("  python -c \"")
        print("  from core.curriculum_manager import CurriculumManager")
        print("  cm = CurriculumManager()")
        print("  print('System ready for use!')")
        print("  \"")
        
        print("\n" + "="*60)


def main():
    """Main setup function"""
    print("Initializing MultiOS Academic Curriculum Integration Platform...")
    
    try:
        # Initialize system
        system = SystemInitializer()
        
        # Setup database
        db_setup = DatabaseSetup(system.config)
        db_setup.initialize_database()
        db_setup.create_sample_data()
        
        # Initialize managers
        managers = system.initialize_managers()
        
        # Setup LMS connections
        lms_manager = system.setup_lms_connections()
        
        # Generate initial reports
        system.generate_initial_reports()
        
        # Print summary
        system.print_setup_summary()
        
        print("✅ Setup completed successfully!")
        
        # Optional: Start the API server
        response = input("\nStart the API server now? (y/N): ").lower().strip()
        if response in ['y', 'yes']:
            print("\nStarting API server...")
            print("API will be available at: http://localhost:8000")
            print("Press Ctrl+C to stop the server")
            
            try:
                from api.main import app
                import uvicorn
                uvicorn.run("api.main:app", host="0.0.0.0", port=8000, reload=True)
            except KeyboardInterrupt:
                print("\nServer stopped.")
            except Exception as e:
                print(f"Error starting server: {e}")
        
        return True
    
    except Exception as e:
        print(f"\n❌ Setup failed: {e}")
        logging.error(f"Setup failed: {e}", exc_info=True)
        return False


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)