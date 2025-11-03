"""
Course Templates for Operating Systems Courses in MultiOS Academic Platform

This module provides comprehensive, standards-aligned course templates for operating systems
education across different levels and specializations.
"""

from typing import Dict, List, Optional, Any
from dataclasses import dataclass, asdict
from datetime import datetime, timedelta
from enum import Enum

from ..core.curriculum_manager import (
    CurriculumManager, Course, CurriculumUnit, LearningOutcome, 
    LearningOutcomeType, DifficultyLevel, CurriculumLevel
)


class OSTemplateType(Enum):
    """Types of OS course templates"""
    INTRODUCTION = "introduction"
    INTERMEDIATE = "intermediate"
    ADVANCED = "advanced"
    SPECIALIZED = "specialized"
    GRADUATE = "graduate"


class CourseTrack(Enum):
    """Course specialization tracks"""
    GENERAL = "general"
    SYSTEMS = "systems"
    DISTRIBUTED = "distributed"
    REAL_TIME = "real_time"
    EMBEDDED = "embedded"
    MOBILE = "mobile"


@dataclass
class OSTemplate:
    """Template for operating systems courses"""
    id: str
    name: str
    description: str
    level: CurriculumLevel
    credits: int
    prerequisites: List[str]
    learning_outcomes: List[str]
    units: List[str]
    assessment_strategy: Dict[str, float]
    resources: List[Dict[str, Any]]
    software_requirements: List[str]
    recommended_duration: timedelta
    difficulty_progression: List[DifficultyLevel]
    tags: List[str]
    track: CourseTrack
    
    def create_course(self, curriculum_manager: CurriculumManager, 
                     instructor_id: str = "", semester: str = "") -> Course:
        """Create a course instance from template"""
        
        # Get learning outcomes
        outcome_objects = [curriculum_manager.learning_outcomes[outcome_id] 
                          for outcome_id in self.learning_outcomes 
                          if outcome_id in curriculum_manager.learning_outcomes]
        
        # Get units
        unit_objects = [curriculum_manager.units[unit_id] 
                       for unit_id in self.units 
                       if unit_id in curriculum_manager.units]
        
        return Course(
            id="",  # Will be set by curriculum manager
            title=self.name,
            description=self.description,
            code=f"CS{self.credits}0",  # Default code
            credits=self.credits,
            level=self.level,
            units=unit_objects,
            prerequisites=self.prerequisites,
            learning_outcomes=outcome_objects,
            resources=self.resources,
            assessment_strategy=self.assessment_strategy,
            standards_alignment=["Operating systems", "Computer systems"],
            tags=self.tags,
            instructor_requirements=[instructor_id] if instructor_id else [],
            facilities_required=self.software_requirements
        )


class OperatingSystemsTemplateManager:
    """Manager for OS course templates"""
    
    def __init__(self, curriculum_manager: CurriculumManager):
        self.curriculum_manager = curriculum_manager
        self.templates: Dict[str, OSTemplate] = {}
        self._initialize_templates()
    
    def _initialize_templates(self):
        """Initialize all OS course templates"""
        self._create_introduction_template()
        self._create_intermediate_template()
        self._create_advanced_template()
        self._create_distributed_systems_template()
        self._create_real_time_systems_template()
        self._create_embedded_systems_template()
        self._create_mobile_os_template()
        self._create_graduate_template()
    
    def _create_learning_outcome(self, description: str, outcome_type: LearningOutcomeType,
                               difficulty: DifficultyLevel, standards: List[str]) -> str:
        """Helper to create learning outcome and return ID"""
        outcome = self.curriculum_manager.create_learning_outcome(
            description=description,
            outcome_type=outcome_type,
            difficulty=difficulty,
            standards_alignment=standards,
            assessment_methods=["Project", "Exam", "Assignment"]
        )
        return outcome.id
    
    def _create_unit(self, title: str, description: str, outcome_ids: List[str],
                    duration_days: int, resources: List[Dict[str, Any]]) -> str:
        """Helper to create unit and return ID"""
        unit = self.curriculum_manager.create_unit(
            title=title,
            description=description,
            learning_outcomes=outcome_ids,
            estimated_duration=timedelta(days=duration_days),
            resources=resources
        )
        return unit.id
    
    def _create_introduction_template(self):
        """Create Introduction to Operating Systems template"""
        
        # Create learning outcomes
        outcome_ids = []
        outcome_ids.append(self._create_learning_outcome(
            "Students can explain fundamental operating system concepts including processes, memory, and file systems",
            LearningOutcomeType.UNDERSTAND,
            DifficultyLevel.BEGINNER,
            ["Operating systems", "Computer systems"]
        ))
        outcome_ids.append(self._create_learning_outcome(
            "Students can use command-line interfaces and basic system administration tasks",
            LearningOutcomeType.APPLY,
            DifficultyLevel.BEGINNER,
            ["Operating systems"]
        ))
        outcome_ids.append(self._create_learning_outcome(
            "Students can analyze the relationship between hardware and operating system components",
            LearningOutcomeType.ANALYZE,
            DifficultyLevel.INTERMEDIATE,
            ["Computer systems", "Operating systems"]
        ))
        
        # Create units
        unit_ids = []
        unit_ids.append(self._create_unit(
            "Introduction to Operating Systems",
            "Overview of OS concepts, history, and basic components",
            [outcome_ids[0]],
            14,
            [
                {"type": "textbook", "title": "Operating System Concepts (Chapters 1-3)", "required": True},
                {"type": "video", "title": "OS Overview Lectures", "required": False},
                {"type": "lab", "title": "Basic System Commands", "required": True}
            ]
        ))
        
        unit_ids.append(self._create_unit(
            "Process Management",
            "Understanding processes, threads, and scheduling algorithms",
            [outcome_ids[0], outcome_ids[1]],
            21,
            [
                {"type": "textbook", "title": "Operating System Concepts (Chapter 4)", "required": True},
                {"type": "simulation", "title": "Process Scheduler Simulator", "required": True},
                {"type": "assignment", "title": "Process Creation and Management", "required": True}
            ]
        ))
        
        unit_ids.append(self._create_unit(
            "Memory Management",
            "Virtual memory, paging, and memory allocation strategies",
            [outcome_ids[2]],
            21,
            [
                {"type": "textbook", "title": "Operating System Concepts (Chapter 8)", "required": True},
                {"type": "lab", "title": "Memory Management Lab", "required": True},
                {"type": "project", "title": "Memory Allocator Implementation", "required": False}
            ]
        ))
        
        # Create template
        self.templates["intro_os"] = OSTemplate(
            id="intro_os",
            name="Introduction to Operating Systems",
            description="Fundamental concepts and principles of modern operating systems for undergraduate students",
            level=CurriculumLevel.UNDERGRADUATE,
            credits=3,
            prerequisites=["CS101: Introduction to Programming", "CS201: Data Structures"],
            learning_outcomes=outcome_ids,
            units=unit_ids,
            assessment_strategy={
                "exams": 0.4,
                "assignments": 0.3,
                "labs": 0.2,
                "participation": 0.1
            },
            resources=[
                {"type": "textbook", "title": "Operating System Concepts by Silberschatz, Galvin, Gagne", "required": True},
                {"type": "vm_image", "title": "Linux VM for Lab Exercises", "required": True},
                {"type": "software", "title": "VirtualBox/VMware", "required": True}
            ],
            software_requirements=["Linux environment", "C compiler", "Text editor"],
            recommended_duration=timedelta(weeks=16),
            difficulty_progression=[DifficultyLevel.BEGINNER, DifficultyLevel.INTERMEDIATE],
            tags=["operating-systems", "undergraduate", "systems"],
            track=CourseTrack.GENERAL
        )
    
    def _create_intermediate_template(self):
        """Create Intermediate Operating Systems template"""
        
        # Create learning outcomes
        outcome_ids = []
        outcome_ids.append(self._create_learning_outcome(
            "Students can implement basic operating system components including system calls and process management",
            LearningOutcomeType.CREATE,
            DifficultyLevel.INTERMEDIATE,
            ["Operating systems", "Systems programming"]
        ))
        outcome_ids.append(self._create_learning_outcome(
            "Students can analyze and compare different scheduling algorithms and their performance characteristics",
            LearningOutcomeType.EVALUATE,
            DifficultyLevel.INTERMEDIATE,
            ["Algorithms", "Operating systems"]
        ))
        outcome_ids.append(self._create_learning_outcome(
            "Students can design and implement file system interfaces and basic file operations",
            LearningOutcomeType.CREATE,
            DifficultyLevel.INTERMEDIATE,
            ["Operating systems"]
        ))
        
        # Create units
        unit_ids = []
        unit_ids.append(self._create_unit(
            "System Programming and APIs",
            "System calls, POSIX APIs, and programming interfaces",
            [outcome_ids[0]],
            14,
            [
                {"type": "textbook", "title": "Advanced Programming in UNIX Environment", "required": True},
                {"type": "lab", "title": "System Call Programming", "required": True},
                {"type": "project", "title": "Shell Implementation", "required": True}
            ]
        ))
        
        unit_ids.append(self._create_unit(
            "Synchronization and Concurrency",
            "Thread synchronization, locks, semaphores, and deadlock handling",
            [outcome_ids[1], outcome_ids[2]],
            21,
            [
                {"type": "textbook", "title": "Operating System Concepts (Chapter 6)", "required": True},
                {"type": "simulation", "title": "Synchronization Simulator", "required": True},
                {"type": "project", "title": "Producer-Consumer Problem", "required": True}
            ]
        ))
        
        unit_ids.append(self._create_unit(
            "File Systems and Storage",
            "File system design, I/O management, and storage optimization",
            [outcome_ids[2]],
            21,
            [
                {"type": "textbook", "title": "Operating System Concepts (Chapter 11)", "required": True},
                {"type": "lab", "title": "File System Implementation", "required": True},
                {"type": "assignment", "title": "I/O Performance Analysis", "required": True}
            ]
        ))
        
        self.templates["intermediate_os"] = OSTemplate(
            id="intermediate_os",
            name="Intermediate Operating Systems",
            description="In-depth study of OS implementation and system programming",
            level=CurriculumLevel.UNDERGRADUATE,
            credits=4,
            prerequisites=["CS301: Introduction to Operating Systems", "CS401: Systems Programming"],
            learning_outcomes=outcome_ids,
            units=unit_ids,
            assessment_strategy={
                "programming_projects": 0.4,
                "exams": 0.3,
                "labs": 0.2,
                "code_reviews": 0.1
            },
            resources=[
                {"type": "textbook", "title": "The Design and Implementation of the 4.4BSD Operating System", "required": True},
                {"type": "source_code", "title": "Linux Kernel Source", "required": False},
                {"type": "software", "title": "QEMU/GDB Debugger", "required": True}
            ],
            software_requirements=["Linux development environment", "GCC compiler", "GDB debugger"],
            recommended_duration=timedelta(weeks=16),
            difficulty_progression=[DifficultyLevel.INTERMEDIATE, DifficultyLevel.ADVANCED],
            tags=["operating-systems", "systems-programming", "undergraduate"],
            track=CourseTrack.SYSTEMS
        )
    
    def _create_advanced_template(self):
        """Create Advanced Operating Systems template"""
        
        # Create learning outcomes
        outcome_ids = []
        outcome_ids.append(self._create_learning_outcome(
            "Students can analyze and modify operating system kernel components",
            LearningOutcomeType.ANALYZE,
            DifficultyLevel.ADVANCED,
            ["Operating systems", "Systems programming"]
        ))
        outcome_ids.append(self._create_learning_outcome(
            "Students can design and implement device drivers and kernel modules",
            LearningOutcomeType.CREATE,
            DifficultyLevel.ADVANCED,
            ["Operating systems", "Computer architecture"]
        ))
        outcome_ids.append(self._create_learning_outcome(
            "Students can evaluate and optimize operating system performance",
            LearningOutcomeType.EVALUATE,
            DifficultyLevel.ADVANCED,
            ["Operating systems", "Performance analysis"]
        ))
        
        # Create units
        unit_ids = []
        unit_ids.append(self._create_unit(
            "Kernel Architecture and Design",
            "Internal kernel design, data structures, and performance considerations",
            [outcome_ids[0]],
            21,
            [
                {"type": "textbook", "title": "Understanding the Linux Kernel", "required": True},
                {"type": "source_code", "title": "Linux Kernel Source Analysis", "required": True},
                {"type": "project", "title": "Kernel Module Development", "required": True}
            ]
        ))
        
        unit_ids.append(self._create_unit(
            "Device Drivers and Hardware Interface",
            "Character devices, block devices, and hardware abstraction layers",
            [outcome_ids[1]],
            28,
            [
                {"type": "textbook", "title": "Linux Device Drivers", "required": True},
                {"type": "hardware", "title": "Raspberry Pi/Arduino Kit", "required": True},
                {"type": "project", "title": "Custom Device Driver", "required": True}
            ]
        ))
        
        unit_ids.append(self._create_unit(
            "Performance Optimization and Analysis",
            "Profiling, benchmarking, and optimization techniques for OS components",
            [outcome_ids[2]],
            21,
            [
                {"type": "textbook", "title": "Systems Performance", "required": True},
                {"type": "software", "title": "Performance Analysis Tools", "required": True},
                {"type": "project", "title": "OS Performance Tuning", "required": True}
            ]
        ))
        
        self.templates["advanced_os"] = OSTemplate(
            id="advanced_os",
            name="Advanced Operating Systems",
            description="Advanced topics in OS design, implementation, and optimization",
            level=CurriculumLevel.UNDERGRADUATE,
            credits=4,
            prerequisites=["CS401: Intermediate Operating Systems", "CS402: Computer Architecture"],
            learning_outcomes=outcome_ids,
            units=unit_ids,
            assessment_strategy={
                "research_project": 0.4,
                "kernel_implementation": 0.3,
                "performance_analysis": 0.2,
                "presentation": 0.1
            },
            resources=[
                {"type": "textbook", "title": "Linux Kernel Development by Robert Love", "required": True},
                {"type": "hardware", "title": "Embedded Development Board", "required": True},
                {"type": "software", "title": "Kernel Build Environment", "required": True}
            ],
            software_requirements=["Custom kernel build environment", "Hardware development tools"],
            recommended_duration=timedelta(weeks=16),
            difficulty_progression=[DifficultyLevel.ADVANCED],
            tags=["operating-systems", "kernel-programming", "advanced"],
            track=CourseTrack.SYSTEMS
        )
    
    def _create_distributed_systems_template(self):
        """Create Distributed Operating Systems template"""
        
        # Create learning outcomes
        outcome_ids = []
        outcome_ids.append(self._create_learning_outcome(
            "Students can design and implement distributed algorithms for process coordination",
            LearningOutcomeType.CREATE,
            DifficultyLevel.ADVANCED,
            ["Distributed systems", "Algorithms"]
        ))
        outcome_ids.append(self._create_learning_outcome(
            "Students can analyze fault tolerance and consistency models in distributed systems",
            LearningOutcomeType.ANALYZE,
            DifficultyLevel.ADVANCED,
            ["Distributed systems", "Operating systems"]
        ))
        outcome_ids.append(self._create_learning_outcome(
            "Students can implement distributed file systems and middleware services",
            LearningOutcomeType.CREATE,
            DifficultyLevel.ADVANCED,
            ["Distributed systems"]
        ))
        
        # Create units
        unit_ids = []
        unit_ids.append(self._create_unit(
            "Distributed System Fundamentals",
            "Network protocols, communication, and distributed coordination",
            [outcome_ids[0]],
            21,
            [
                {"type": "textbook", "title": "Distributed Systems: Principles and Paradigms", "required": True},
                {"type": "simulation", "title": "Network Simulator", "required": True},
                {"type": "project", "title": "Distributed Mutual Exclusion", "required": True}
            ]
        ))
        
        unit_ids.append(self._create_unit(
            "Consistency and Replication",
            "CAP theorem, eventual consistency, and replication protocols",
            [outcome_ids[1]],
            21,
            [
                {"type": "textbook", "title": "Designing Data-Intensive Applications", "required": True},
                {"type": "assignment", "title": "Consistency Protocol Implementation", "required": True},
                {"type": "project", "title": "Distributed Key-Value Store", "required": True}
            ]
        ))
        
        unit_ids.append(self._create_unit(
            "Distributed File Systems and Services",
            "NFS, AFS, cloud storage, and distributed computing frameworks",
            [outcome_ids[2]],
            28,
            [
                {"type": "textbook", "title": "The Google File System", "required": True},
                {"type": "software", "title": "Hadoop/Spark Environment", "required": True},
                {"type": "project", "title": "Distributed File System", "required": True}
            ]
        ))
        
        self.templates["distributed_os"] = OSTemplate(
            id="distributed_os",
            name="Distributed Operating Systems",
            description="Design and implementation of distributed operating systems",
            level=CurriculumLevel.UNDERGRADUATE,
            credits=4,
            prerequisites=["CS401: Intermediate Operating Systems", "CS405: Computer Networks"],
            learning_outcomes=outcome_ids,
            units=unit_ids,
            assessment_strategy={
                "distributed_projects": 0.5,
                "protocol_implementation": 0.3,
                "research_paper": 0.2
            },
            resources=[
                {"type": "textbook", "title": "Distributed Systems by Tanenbaum & Steen", "required": True},
                {"type": "cloud", "title": "Cloud Computing Credits", "required": True},
                {"type": "software", "title": "Docker/Kubernetes", "required": True}
            ],
            software_requirements=["Distributed system simulator", "Cloud platform access"],
            recommended_duration=timedelta(weeks=16),
            difficulty_progression=[DifficultyLevel.ADVANCED],
            tags=["distributed-systems", "advanced", "networking"],
            track=CourseTrack.DISTRIBUTED
        )
    
    def _create_real_time_systems_template(self):
        """Create Real-Time Systems template"""
        
        # Create learning outcomes
        outcome_ids = []
        outcome_ids.append(self._create_learning_outcome(
            "Students can analyze real-time scheduling algorithms and their guarantees",
            LearningOutcomeType.ANALYZE,
            DifficultyLevel.ADVANCED,
            ["Real-time systems", "Algorithms"]
        ))
        outcome_ids.append(self._create_learning_outcome(
            "Students can design and implement real-time operating systems",
            LearningOutcomeType.CREATE,
            DifficultyLevel.EXPERT,
            ["Real-time systems", "Operating systems"]
        ))
        outcome_ids.append(self._create_learning_outcome(
            "Students can evaluate timing constraints and system performance in real-time applications",
            LearningOutcomeType.EVALUATE,
            DifficultyLevel.ADVANCED,
            ["Real-time systems"]
        ))
        
        # Create units
        unit_ids = []
        unit_ids.append(self._create_unit(
            "Real-Time Scheduling Theory",
            "Priority-based scheduling, rate monotonic analysis, and Earliest Deadline First",
            [outcome_ids[0]],
            21,
            [
                {"type": "textbook", "title": "Real-Time Systems by Jane Liu", "required": True},
                {"type": "simulation", "title": "Real-Time Scheduler Simulator", "required": True},
                {"type": "assignment", "title": "Scheduling Algorithm Analysis", "required": True}
            ]
        ))
        
        unit_ids.append(self._create_unit(
            "Real-Time Operating Systems",
            "RTOS architecture, interrupt handling, and timing mechanisms",
            [outcome_ids[1]],
            28,
            [
                {"type": "textbook", "title": "Computer Systems: A Programmer's Perspective", "required": True},
                {"type": "hardware", "title": "Real-Time Development Board", "required": True},
                {"type": "project", "title": "RTOS Implementation", "required": True}
            ]
        ))
        
        unit_ids.append(self._create_unit(
            "Real-Time Applications",
            "Industrial control, automotive systems, and multimedia processing",
            [outcome_ids[2]],
            21,
            [
                {"type": "case_study", "title": "Automotive Safety Systems", "required": True},
                {"type": "project", "title": "Real-Time Control System", "required": True},
                {"type": "hardware", "title": "Sensor/Actuator Kit", "required": True}
            ]
        ))
        
        self.templates["realtime_os"] = OSTemplate(
            id="realtime_os",
            name="Real-Time Operating Systems",
            description="Design and implementation of real-time operating systems",
            level=CurriculumLevel.GRADUATE,
            credits=3,
            prerequisites=["CS401: Intermediate Operating Systems", "CS450: Embedded Systems"],
            learning_outcomes=outcome_ids,
            units=unit_ids,
            assessment_strategy={
                "rtos_project": 0.4,
                "timing_analysis": 0.3,
                "research_project": 0.3
            },
            resources=[
                {"type": "textbook", "title": "Real-Time Systems by Hermann Kopetz", "required": True},
                {"type": "hardware", "title": "Real-Time Development Platform", "required": True},
                {"type": "software", "title": "RTOS Simulator", "required": True}
            ],
            software_requirements=["Real-time development tools", "Hardware timing analyzers"],
            recommended_duration=timedelta(weeks=16),
            difficulty_progression=[DifficultyLevel.ADVANCED, DifficultyLevel.EXPERT],
            tags=["real-time-systems", "graduate", "embedded"],
            track=CourseTrack.REAL_TIME
        )
    
    def _create_embedded_systems_template(self):
        """Create Embedded Systems OS template"""
        
        # Create learning outcomes
        outcome_ids = []
        outcome_ids.append(self._create_learning_outcome(
            "Students can optimize operating systems for resource-constrained embedded environments",
            LearningOutcomeType.OPTIMIZE,  # Custom type
            DifficultyLevel.ADVANCED,
            ["Embedded systems", "Operating systems"]
        ))
        outcome_ids.append(self._create_learning_outcome(
            "Students can design and implement low-power management strategies",
            LearningOutcomeType.CREATE,
            DifficultyLevel.ADVANCED,
            ["Embedded systems", "Power management"]
        ))
        
        # Create units
        unit_ids = []
        unit_ids.append(self._create_unit(
            "Embedded OS Architecture",
            "Microkernel vs monolithic, footprint optimization, and resource management",
            [outcome_ids[0]],
            21,
            [
                {"type": "textbook", "title": "Building Embedded Linux Systems", "required": True},
                {"type": "hardware", "title": "ARM Development Board", "required": True},
                {"type": "project", "title": "Minimal OS Implementation", "required": True}
            ]
        ))
        
        unit_ids.append(self._create_unit(
            "Power Management and Optimization",
            "Dynamic frequency scaling, sleep states, and energy-efficient scheduling",
            [outcome_ids[1]],
            21,
            [
                {"type": "textbook", "title": "Low Power Design", "required": True},
                {"type": "hardware", "title": "Power Measurement Tools", "required": True},
                {"type": "project", "title": "Battery-Optimized System", "required": True}
            ]
        ))
        
        self.templates["embedded_os"] = OSTemplate(
            id="embedded_os",
            name="Embedded Operating Systems",
            description="Operating systems for resource-constrained embedded environments",
            level=CurriculumLevel.UNDERGRADUATE,
            credits=3,
            prerequisites=["CS401: Intermediate Operating Systems", "CS450: Embedded Systems"],
            learning_outcomes=outcome_ids,
            units=unit_ids,
            assessment_strategy={
                "embedded_project": 0.5,
                "optimization_analysis": 0.3,
                "hardware_integration": 0.2
            },
            resources=[
                {"type": "textbook", "title": "Embedded Linux Primer", "required": True},
                {"type": "hardware", "title": "Embedded Development Kit", "required": True},
                {"type": "software", "title": "Cross-Compilation Tools", "required": True}
            ],
            software_requirements=["Cross-compilation toolchain", "Hardware debugger"],
            recommended_duration=timedelta(weeks=16),
            difficulty_progression=[DifficultyLevel.ADVANCED],
            tags=["embedded-systems", "optimization", "resource-constrained"],
            track=CourseTrack.EMBEDDED
        )
    
    def _create_mobile_os_template(self):
        """Create Mobile OS template"""
        
        # Create learning outcomes
        outcome_ids = []
        outcome_ids.append(self._create_learning_outcome(
            "Students can analyze mobile operating system architecture and design principles",
            LearningOutcomeType.ANALYZE,
            DifficultyLevel.INTERMEDIATE,
            ["Mobile systems", "Operating systems"]
        ))
        outcome_ids.append(self._create_learning_outcome(
            "Students can develop and optimize applications for mobile platforms",
            LearningOutcomeType.CREATE,
            DifficultyLevel.INTERMEDIATE,
            ["Mobile development", "Platform optimization"]
        ))
        
        # Create units
        unit_ids = []
        unit_ids.append(self._create_unit(
            "Mobile OS Architecture",
            "Android and iOS system architecture, runtime environments, and security models",
            [outcome_ids[0]],
            21,
            [
                {"type": "textbook", "title": "Android Internals", "required": True},
                {"type": "software", "title": "Android SDK/iOS SDK", "required": True},
                {"type": "project", "title": "System Service Implementation", "required": True}
            ]
        ))
        
        unit_ids.append(self._create_unit(
            "Mobile App Development and Optimization",
            "Performance optimization, battery management, and user experience design",
            [outcome_ids[1]],
            21,
            [
                {"type": "tutorial", "title": "Mobile Development Best Practices", "required": True},
                {"type": "tools", "title": "Profiling and Debugging Tools", "required": True},
                {"type": "project", "title": "Performance-Optimized App", "required": True}
            ]
        ))
        
        self.templates["mobile_os"] = OSTemplate(
            id="mobile_os",
            name="Mobile Operating Systems",
            description="Design and development for mobile operating systems",
            level=CurriculumLevel.UNDERGRADUATE,
            credits=3,
            prerequisites=["CS201: Data Structures", "CS301: Introduction to Operating Systems"],
            learning_outcomes=outcome_ids,
            units=unit_ids,
            assessment_strategy={
                "app_development": 0.4,
                "system_analysis": 0.3,
                "optimization_project": 0.3
            },
            resources=[
                {"type": "software", "title": "Android Studio/Xcode", "required": True},
                {"type": "device", "title": "Mobile Development Device", "required": True},
                {"type": "cloud", "title": "Mobile App Store Credits", "required": False}
            ],
            software_requirements=["Mobile development environment", "Device testing capability"],
            recommended_duration=timedelta(weeks=16),
            difficulty_progression=[DifficultyLevel.INTERMEDIATE, DifficultyLevel.ADVANCED],
            tags=["mobile-systems", "android", "ios", "app-development"],
            track=CourseTrack.MOBILE
        )
    
    def _create_graduate_template(self):
        """Create Graduate OS template"""
        
        # Create learning outcomes
        outcome_ids = []
        outcome_ids.append(self._create_learning_outcome(
            "Students can conduct original research in operating systems and present findings",
            LearningOutcomeType.CREATE,
            DifficultyLevel.EXPERT,
            ["Operating systems", "Research methods"]
        ))
        outcome_ids.append(self._create_learning_outcome(
            "Students can evaluate and design next-generation operating system architectures",
            LearningOutcomeType.EVALUATE,
            DifficultyLevel.EXPERT,
            ["Operating systems", "System design"]
        ))
        outcome_ids.append(self._create_learning_outcome(
            "Students can analyze and solve complex problems in operating systems research",
            LearningOutcomeType.ANALYZE,
            DifficultyLevel.EXPERT,
            ["Operating systems", "Problem solving"]
        ))
        
        # Create units
        unit_ids = []
        unit_ids.append(self._create_unit(
            "Advanced OS Research Topics",
            "Current research frontiers, emerging technologies, and research methodology",
            [outcome_ids[0]],
            21,
            [
                {"type": "research_papers", "title": "Recent OS Research Papers", "required": True},
                {"type": "seminar", "title": "Research Paper Presentations", "required": True},
                {"type": "project", "title": "Research Proposal", "required": True}
            ]
        ))
        
        unit_ids.append(self._create_unit(
            "Next-Generation OS Architectures",
            "Virtualization, cloud computing, and emerging paradigms",
            [outcome_ids[1]],
            28,
            [
                {"type": "textbook", "title": "Virtualization: Concepts and Applications", "required": True},
                {"type": "software", "title": "Virtualization Platform", "required": True},
                {"type": "project", "title": "OS Architecture Design", "required": True}
            ]
        ))
        
        unit_ids.append(self._create_unit(
            "Independent Research Project",
            "Student-driven research project with faculty mentorship",
            [outcome_ids[2]],
            42,
            [
                {"type": "mentorship", "title": "Faculty Research Mentorship", "required": True},
                {"type": "resources", "title": "Research Computing Resources", "required": True},
                {"type": "publication", "title": "Conference Paper Submission", "required": False}
            ]
        ))
        
        self.templates["graduate_os"] = OSTemplate(
            id="graduate_os",
            name="Advanced Operating Systems (Graduate)",
            description="Graduate-level operating systems research and advanced topics",
            level=CurriculumLevel.GRADUATE,
            credits=6,
            prerequisites=["CS401: Intermediate Operating Systems", "Graduate standing"],
            learning_outcomes=outcome_ids,
            units=unit_ids,
            assessment_strategy={
                "research_project": 0.5,
                "paper_analysis": 0.2,
                "presentation": 0.2,
                "participation": 0.1
            },
            resources=[
                {"type": "research_papers", "title": "ACM/IEEE Digital Library Access", "required": True},
                {"type": "computing", "title": "High-Performance Computing Access", "required": True},
                {"type": "conference", "title": "Conference Attendance Support", "required": False}
            ],
            software_requirements=["Advanced development tools", "Research computing resources"],
            recommended_duration=timedelta(weeks=16),
            difficulty_progression=[DifficultyLevel.EXPERT],
            tags=["graduate", "research", "advanced", "next-generation"],
            track=CourseTrack.GENERAL
        )
    
    def get_template(self, template_id: str) -> Optional[OSTemplate]:
        """Get a specific template"""
        return self.templates.get(template_id)
    
    def get_templates_by_level(self, level: CurriculumLevel) -> List[OSTemplate]:
        """Get templates for a specific level"""
        return [template for template in self.templates.values() 
                if template.level == level]
    
    def get_templates_by_track(self, track: CourseTrack) -> List[OSTemplate]:
        """Get templates for a specific track"""
        return [template for template in self.templates.values() 
                if template.track == track]
    
    def create_course_from_template(self, template_id: str, 
                                   instructor_id: str = "", 
                                   semester: str = "",
                                   custom_code: str = "") -> Optional[Course]:
        """Create a course instance from template"""
        
        template = self.get_template(template_id)
        if not template:
            return None
        
        course = template.create_course(self.curriculum_manager, instructor_id, semester)
        
        # Apply customizations
        if custom_code:
            course.code = custom_code
        
        return course
    
    def suggest_prerequisites(self, template_id: str) -> Dict[str, List[str]]:
        """Suggest additional courses for students based on template"""
        
        template = self.get_template(template_id)
        if not template:
            return {}
        
        suggestions = {
            "recommended": [],
            "highly_recommended": [],
            "optional": []
        }
        
        # Analyze track and level to suggest relevant courses
        if template.track == CourseTrack.DISTRIBUTED:
            suggestions["highly_recommended"].extend([
                "CS405: Computer Networks",
                "CS410: Database Systems"
            ])
        elif template.track == CourseTrack.REAL_TIME:
            suggestions["highly_recommended"].extend([
                "CS450: Embedded Systems",
                "CS460: Control Systems"
            ])
        elif template.track == CourseTrack.EMBEDDED:
            suggestions["highly_recommended"].extend([
                "CS450: Embedded Systems",
                "CS455: Digital Signal Processing"
            ])
        elif template.track == CourseTrack.MOBILE:
            suggestions["recommended"].extend([
                "CS310: Human-Computer Interaction",
                "CS420: Software Engineering"
            ])
        
        # Add general recommendations based on level
        if template.level == CurriculumLevel.GRADUATE:
            suggestions["highly_recommended"].append("CS500: Research Methods")
            suggestions["recommended"].append("CS510: Advanced Algorithms")
        
        return suggestions
    
    def generate_learning_path(self, student_goals: List[str], 
                             current_level: CurriculumLevel) -> List[OSTemplate]:
        """Generate a recommended learning path based on goals and current level"""
        
        path = []
        
        # Map goals to relevant tracks
        goal_track_mapping = {
            "systems_programming": CourseTrack.SYSTEMS,
            "distributed_systems": CourseTrack.DISTRIBUTED,
            "real_time_systems": CourseTrack.REAL_TIME,
            "embedded_systems": CourseTrack.EMBEDDED,
            "mobile_development": CourseTrack.MOBILE,
            "research": CourseTrack.GENERAL
        }
        
        # Determine starting level
        target_tracks = []
        for goal in student_goals:
            track = goal_track_mapping.get(goal.lower())
            if track:
                target_tracks.append(track)
        
        # Build path based on current level and goals
        if current_level == CurriculumLevel.UNDERGRADUATE:
            if CourseTrack.SYSTEMS in target_tracks or not target_tracks:
                path.extend(self.get_templates_by_track(CourseTrack.GENERAL))
                path.extend(self.get_templates_by_track(CourseTrack.SYSTEMS))
            else:
                path.extend([t for t in self.get_templates_by_level(current_level) 
                           if t.track in target_tracks])
        
        elif current_level == CurriculumLevel.GRADUATE:
            path.extend(self.get_templates_by_level(CurrentLevel.GRADUATE))
        
        return path
    
    def get_template_statistics(self) -> Dict[str, Any]:
        """Get statistics about available templates"""
        
        level_counts = {}
        track_counts = {}
        difficulty_distribution = {}
        
        for template in self.templates.values():
            # Count by level
            level = template.level.value
            level_counts[level] = level_counts.get(level, 0) + 1
            
            # Count by track
            track = template.track.value
            track_counts[track] = track_counts.get(track, 0) + 1
            
            # Count by difficulty
            for difficulty in template.difficulty_progression:
                diff_name = difficulty.value
                difficulty_distribution[diff_name] = difficulty_distribution.get(diff_name, 0) + 1
        
        return {
            "total_templates": len(self.templates),
            "by_level": level_counts,
            "by_track": track_counts,
            "difficulty_distribution": difficulty_distribution,
            "average_credits": sum(t.credits for t in self.templates.values()) / len(self.templates),
            "total_software_requirements": len(set(
                req for template in self.templates.values() 
                for req in template.software_requirements
            ))
        }


# Example usage
if __name__ == "__main__":
    # Initialize curriculum manager
    from ..core.curriculum_manager import CurriculumManager
    
    curriculum_mgr = CurriculumManager("../config/cs_standards.json")
    template_mgr = OperatingSystemsTemplateManager(curriculum_mgr)
    
    print("Operating Systems Course Templates Initialized!")
    
    # Display template statistics
    stats = template_mgr.get_template_statistics()
    print(f"\nTemplate Statistics:")
    print(json.dumps(stats, indent=2, default=str))
    
    # Create a course from template
    course = template_mgr.create_course_from_template(
        template_id="intro_os",
        instructor_id="prof_smith",
        semester="Fall 2025",
        custom_code="CS 301"
    )
    
    if course:
        print(f"\nCreated course: {course.title}")
        print(f"Code: {course.code}")
        print(f"Credits: {course.credits}")
        print(f"Units: {len(course.units)}")
    
    # Get suggestions for prerequisites
    suggestions = template_mgr.suggest_prerequisites("distributed_os")
    print(f"\nPrerequisite suggestions for Distributed OS:")
    print(json.dumps(suggestions, indent=2, default=str))
    
    # Generate learning path
    path = template_mgr.generate_learning_path(
        student_goals=["distributed_systems", "research"],
        current_level=CurriculumLevel.UNDERGRADUATE
    )
    print(f"\nRecommended learning path ({len(path)} courses):")
    for template in path:
        print(f"- {template.name} ({template.credits} credits)")
