"""
Core Curriculum Management System for MultiOS Academic Integration

This module provides the foundation for managing academic curricula, courses, and standards alignment.
"""

from typing import Dict, List, Optional, Any
from dataclasses import dataclass, asdict
from datetime import datetime, timedelta
from enum import Enum
import json
import uuid
import os


class CurriculumLevel(Enum):
    """Curriculum levels for different educational stages"""
    ELEMENTARY = "elementary"
    MIDDLE_SCHOOL = "middle_school"
    HIGH_SCHOOL = "high_school"
    UNDERGRADUATE = "undergraduate"
    GRADUATE = "graduate"
    PROFESSIONAL = "professional"


class LearningOutcomeType(Enum):
    """Types of learning outcomes based on Bloom's Taxonomy"""
    REMEMBER = "remember"
    UNDERSTAND = "understand"
    APPLY = "apply"
    ANALYZE = "analyze"
    EVALUATE = "evaluate"
    CREATE = "create"


class DifficultyLevel(Enum):
    """Difficulty levels for curriculum content"""
    BEGINNER = "beginner"
    INTERMEDIATE = "intermediate"
    ADVANCED = "advanced"
    EXPERT = "expert"


@dataclass
class LearningOutcome:
    """Represents a specific learning outcome with Bloom's taxonomy alignment"""
    id: str
    description: str
    outcome_type: LearningOutcomeType
    difficulty: DifficultyLevel
    standards_alignment: List[str]
    prerequisites: List[str]
    assessment_methods: List[str]
    evidence: Optional[str] = None
    created_at: datetime = None
    
    def __post_init__(self):
        if self.created_at is None:
            self.created_at = datetime.now()


@dataclass
class CurriculumUnit:
    """Represents a unit within a course"""
    id: str
    title: str
    description: str
    learning_outcomes: List[LearningOutcome]
    estimated_duration: timedelta
    prerequisites: List[str]
    resources: List[Dict[str, Any]]
    activities: List[Dict[str, Any]]
    assessments: List[Dict[str, Any]]
    standards_alignment: List[str]
    created_at: datetime = None
    
    def __post_init__(self):
        if self.created_at is None:
            self.created_at = datetime.now()


@dataclass
class Course:
    """Represents a complete course with curriculum units"""
    id: str
    title: str
    description: str
    code: str
    credits: int
    level: CurriculumLevel
    units: List[CurriculumUnit]
    prerequisites: List[str]
    learning_outcomes: List[LearningOutcome]
    resources: List[Dict[str, Any]]
    assessment_strategy: Dict[str, Any]
    standards_alignment: List[str]
    tags: List[str]
    instructor_requirements: List[str]
    facilities_required: List[str]
    created_at: datetime = None
    updated_at: datetime = None
    is_active: bool = True
    
    def __post_init__(self):
        if self.created_at is None:
            self.created_at = datetime.now()
        if self.updated_at is None:
            self.updated_at = datetime.now()


class CurriculumManager:
    """Core curriculum management system"""
    
    def __init__(self, config_path: str = None):
        """Initialize curriculum manager with CS standards"""
        self.courses: Dict[str, Course] = {}
        self.learning_outcomes: Dict[str, LearningOutcome] = {}
        self.units: Dict[str, CurriculumUnit] = {}
        
        # Load CS standards if config path provided
        self.cs_standards = {}
        if config_path and os.path.exists(config_path):
            with open(config_path, 'r') as f:
                self.cs_standards = json.load(f)
    
    def create_learning_outcome(
        self,
        description: str,
        outcome_type: LearningOutcomeType,
        difficulty: DifficultyLevel,
        standards_alignment: List[str],
        prerequisites: List[str] = None,
        assessment_methods: List[str] = None
    ) -> LearningOutcome:
        """Create a new learning outcome"""
        if prerequisites is None:
            prerequisites = []
        if assessment_methods is None:
            assessment_methods = []
            
        outcome = LearningOutcome(
            id=str(uuid.uuid4()),
            description=description,
            outcome_type=outcome_type,
            difficulty=difficulty,
            standards_alignment=standards_alignment,
            prerequisites=prerequisites,
            assessment_methods=assessment_methods
        )
        
        self.learning_outcomes[outcome.id] = outcome
        return outcome
    
    def create_unit(
        self,
        title: str,
        description: str,
        learning_outcomes: List[str],
        estimated_duration: timedelta,
        prerequisites: List[str] = None,
        resources: List[Dict[str, Any]] = None,
        activities: List[Dict[str, Any]] = None,
        assessments: List[Dict[str, Any]] = None,
        standards_alignment: List[str] = None
    ) -> CurriculumUnit:
        """Create a new curriculum unit"""
        if prerequisites is None:
            prerequisites = []
        if resources is None:
            resources = []
        if activities is None:
            activities = []
        if assessments is None:
            assessments = []
        if standards_alignment is None:
            standards_alignment = []
        
        # Get learning outcome objects
        outcome_objects = [self.learning_outcomes[outcome_id] for outcome_id in learning_outcomes]
        
        unit = CurriculumUnit(
            id=str(uuid.uuid4()),
            title=title,
            description=description,
            learning_outcomes=outcome_objects,
            estimated_duration=estimated_duration,
            prerequisites=prerequisites,
            resources=resources,
            activities=activities,
            assessments=assessments,
            standards_alignment=standards_alignment
        )
        
        self.units[unit.id] = unit
        return unit
    
    def create_course(
        self,
        title: str,
        description: str,
        code: str,
        credits: int,
        level: CurriculumLevel,
        unit_ids: List[str],
        prerequisites: List[str] = None,
        learning_outcomes: List[str] = None,
        resources: List[Dict[str, Any]] = None,
        assessment_strategy: Dict[str, Any] = None,
        standards_alignment: List[str] = None,
        tags: List[str] = None,
        instructor_requirements: List[str] = None,
        facilities_required: List[str] = None
    ) -> Course:
        """Create a new course"""
        if prerequisites is None:
            prerequisites = []
        if learning_outcomes is None:
            learning_outcomes = []
        if resources is None:
            resources = []
        if assessment_strategy is None:
            assessment_strategy = {
                "midterm": 0.3,
                "final": 0.4,
                "assignments": 0.2,
                "participation": 0.1
            }
        if standards_alignment is None:
            standards_alignment = []
        if tags is None:
            tags = []
        if instructor_requirements is None:
            instructor_requirements = []
        if facilities_required is None:
            facilities_required = []
        
        # Get unit objects
        unit_objects = [self.units[unit_id] for unit_id in unit_ids]
        outcome_objects = [self.learning_outcomes[outcome_id] for outcome_id in learning_outcomes]
        
        course = Course(
            id=str(uuid.uuid4()),
            title=title,
            description=description,
            code=code,
            credits=credits,
            level=level,
            units=unit_objects,
            prerequisites=prerequisites,
            learning_outcomes=outcome_objects,
            resources=resources,
            assessment_strategy=assessment_strategy,
            standards_alignment=standards_alignment,
            tags=tags,
            instructor_requirements=instructor_requirements,
            facilities_required=facilities_required
        )
        
        self.courses[course.id] = course
        return course
    
    def get_course(self, course_id: str) -> Optional[Course]:
        """Retrieve a course by ID"""
        return self.courses.get(course_id)
    
    def get_courses_by_level(self, level: CurriculumLevel) -> List[Course]:
        """Get all courses at a specific level"""
        return [course for course in self.courses.values() if course.level == level]
    
    def get_courses_by_standards_alignment(self, standard: str) -> List[Course]:
        """Get courses aligned with specific standards"""
        return [course for standard in course.standards_alignment for course in self.courses 
                if standard in course.standards_alignment]
    
    def update_course(self, course_id: str, **kwargs) -> Optional[Course]:
        """Update an existing course"""
        if course_id not in self.courses:
            return None
        
        course = self.courses[course_id]
        
        # Update fields
        for key, value in kwargs.items():
            if hasattr(course, key):
                setattr(course, key, value)
        
        course.updated_at = datetime.now()
        return course
    
    def delete_course(self, course_id: str) -> bool:
        """Delete a course"""
        if course_id in self.courses:
            del self.courses[course_id]
            return True
        return False
    
    def export_course(self, course_id: str, format: str = "json") -> Dict[str, Any]:
        """Export course data in specified format"""
        course = self.get_course(course_id)
        if not course:
            return {}
        
        course_dict = asdict(course)
        
        # Convert datetime objects to strings
        for unit in course_dict['units']:
            if unit['created_at']:
                unit['created_at'] = unit['created_at'].isoformat()
        
        if format == "json":
            return course_dict
        elif format == "xml":
            # Simple XML conversion
            return self._dict_to_xml(course_dict, "course")
        else:
            return course_dict
    
    def _dict_to_xml(self, data: Dict[str, Any], root: str) -> Dict[str, Any]:
        """Simple dictionary to XML conversion"""
        xml_parts = []
        xml_parts.append(f'<{root}>')
        
        for key, value in data.items():
            if isinstance(value, list):
                xml_parts.append(f'  <{key}>')
                for item in value:
                    if isinstance(item, dict):
                        xml_parts.append(f'    <item>')
                        for sub_key, sub_value in item.items():
                            xml_parts.append(f'      <{sub_key}>{sub_value}</{sub_key}>')
                        xml_parts.append(f'    </item>')
                    else:
                        xml_parts.append(f'    <item>{item}</item>')
                xml_parts.append(f'  </{key}>')
            else:
                xml_parts.append(f'  <{key}>{value}</{key}>')
        
        xml_parts.append(f'</{root}>')
        
        return {"xml": '\n'.join(xml_parts)}
    
    def get_learning_outcome_by_id(self, outcome_id: str) -> Optional[LearningOutcome]:
        """Get a specific learning outcome"""
        return self.learning_outcomes.get(outcome_id)
    
    def get_units_by_difficulty(self, difficulty: DifficultyLevel) -> List[CurriculumUnit]:
        """Get units filtered by difficulty level"""
        units = []
        for unit in self.units.values():
            # Check if any learning outcomes match the difficulty
            if any(outcome.difficulty == difficulty for outcome in unit.learning_outcomes):
                units.append(unit)
        return units
    
    def analyze_curriculum_coverage(self, standard: str) -> Dict[str, Any]:
        """Analyze curriculum coverage of specific standards"""
        coverage = {
            "standard": standard,
            "courses_aligned": 0,
            "units_aligned": 0,
            "outcomes_aligned": 0,
            "coverage_percentage": 0.0,
            "gaps": [],
            "strengths": []
        }
        
        # Count alignments
        for course in self.courses.values():
            if standard in course.standards_alignment:
                coverage["courses_aligned"] += 1
                coverage["units_aligned"] += len(course.units)
                coverage["outcomes_aligned"] += len(course.learning_outcomes)
        
        # Calculate coverage percentage (simplified)
        total_courses = len(self.courses)
        if total_courses > 0:
            coverage["coverage_percentage"] = (coverage["courses_aligned"] / total_courses) * 100
        
        return coverage
    
    def generate_curriculum_report(self) -> Dict[str, Any]:
        """Generate comprehensive curriculum analysis report"""
        report = {
            "summary": {
                "total_courses": len(self.courses),
                "total_units": len(self.units),
                "total_learning_outcomes": len(self.learning_outcomes),
                "generated_at": datetime.now().isoformat()
            },
            "by_level": {},
            "by_difficulty": {},
            "standards_coverage": {},
            "prerequisites_analysis": {},
            "assessment_coverage": {}
        }
        
        # Analysis by level
        for level in CurriculumLevel:
            level_courses = self.get_courses_by_level(level)
            if level_courses:
                report["by_level"][level.value] = {
                    "count": len(level_courses),
                    "average_credits": sum(course.credits for course in level_courses) / len(level_courses),
                    "courses": [course.title for course in level_courses]
                }
        
        # Standards coverage analysis
        all_standards = set()
        for course in self.courses.values():
            all_standards.update(course.standards_alignment)
        
        for standard in all_standards:
            report["standards_coverage"][standard] = self.analyze_curriculum_coverage(standard)
        
        return report


# Example usage and testing
if __name__ == "__main__":
    # Initialize curriculum manager
    cm = CurriculumManager("../config/cs_standards.json")
    
    # Create sample learning outcomes
    outcome1 = cm.create_learning_outcome(
        description="Students can analyze algorithmic complexity using Big O notation",
        outcome_type=LearningOutcomeType.ANALYZE,
        difficulty=DifficultyLevel.INTERMEDIATE,
        standards_alignment=["Algorithms and complexity"],
        assessment_methods=["Written analysis", "Algorithm implementation"]
    )
    
    outcome2 = cm.create_learning_outcome(
        description="Students can implement basic process scheduling algorithms",
        outcome_type=LearningOutcomeType.CREATE,
        difficulty=DifficultyLevel.ADVANCED,
        standards_alignment=["Operating systems"],
        prerequisites=[outcome1.id],
        assessment_methods=["Programming project", "System simulation"]
    )
    
    # Create sample units
    unit1 = cm.create_unit(
        title="Introduction to Operating Systems",
        description="Basic concepts and principles of operating systems",
        learning_outcomes=[outcome1.id],
        estimated_duration=timedelta(weeks=2),
        resources=[
            {"type": "textbook", "title": "Operating System Concepts", "required": True},
            {"type": "video", "title": "OS Overview", "required": False}
        ],
        activities=[
            {"type": "lecture", "duration": "50 minutes"},
            {"type": "lab", "duration": "2 hours"}
        ]
    )
    
    # Create sample course
    course = cm.create_course(
        title="Introduction to Operating Systems",
        description="Fundamental concepts and principles of modern operating systems",
        code="CS 301",
        credits=3,
        level=CurriculumLevel.UNDERGRADUATE,
        unit_ids=[unit1.id],
        learning_outcomes=[outcome1.id, outcome2.id],
        standards_alignment=["Operating systems", "Computing systems"],
        tags=["operating-systems", "systems", "undergraduate"]
    )
    
    print("Curriculum Manager initialized successfully!")
    print(f"Created course: {course.title}")
    print(f"Course ID: {course.id}")
    print(f"Credits: {course.credits}")
    print(f"Level: {course.level.value}")
    
    # Generate report
    report = cm.generate_curriculum_report()
    print("\nCurriculum Report Summary:")
    print(json.dumps(report["summary"], indent=2))