"""
Educational Robotics Curriculum

Step-by-step tutorials and learning materials for educational robotics
"""

import json
import time
from typing import Dict, List, Optional, Any, Callable
from dataclasses import dataclass, asdict
from pathlib import Path


@dataclass
class LearningObjective:
    """Learning objective for a lesson"""
    id: str
    title: str
    description: str
    difficulty_level: str  # beginner, intermediate, advanced
    prerequisites: List[str]  # List of prerequisite lesson IDs
    estimated_duration: int  # Duration in minutes
    skills_covered: List[str]


@dataclass
class Lesson:
    """Individual lesson in the curriculum"""
    id: str
    title: str
    description: str
    objective: LearningObjective
    content: Dict[str, Any]
    activities: List[Dict[str, Any]]
    assessment_questions: List[Dict[str, str]]
    resources: List[str]
    hands_on_project: Optional[Dict[str, Any]] = None


@dataclass
class Curriculum:
    """Complete curriculum structure"""
    name: str
    description: str
    target_audience: str
    total_duration: int  # Total duration in hours
    lessons: List[Lesson]
    prerequisites: List[str]  # Overall curriculum prerequisites
    learning_outcomes: List[str]
    assessment_methods: List[str]


class EducationalRobot:
    """Educational robot model for curriculum examples"""
    
    def __init__(self, robot_type: str):
        self.robot_type = robot_type
        self.capabilities = self._get_capabilities()
        self.setup_instructions = self._get_setup_instructions()
        
    def _get_capabilities(self) -> Dict[str, Any]:
        """Get robot capabilities by type"""
        capabilities = {
            'ev3': {
                'sensors': ['Color Sensor', 'Ultrasonic Sensor', 'Touch Sensor', 'Gyro Sensor'],
                'actuators': ['Large Motors', 'Medium Motor'],
                'communication': ['Bluetooth', 'USB'],
                'programming': ['LabView', 'Python', 'Scratch'],
                'price_range': '$300-400',
                'age_range': '10-16'
            },
            'arduino': {
                'sensors': ['Ultrasonic', 'IR', 'IMU', 'Camera', 'Temperature'],
                'actuators': ['Servo Motors', 'DC Motors', 'LEDs', 'Buzzers'],
                'communication': ['WiFi', 'Bluetooth', 'Serial'],
                'programming': ['C/C++', 'Blockly', 'Python'],
                'price_range': '$50-200',
                'age_range': '12-18'
            },
            'raspberry_pi': {
                'sensors': ['Camera', 'IMU', 'Ultrasonic', 'Environmental'],
                'actuators': ['Servo Motors', 'Stepper Motors', 'LEDs'],
                'communication': ['WiFi', 'Bluetooth', 'SSH'],
                'programming': ['Python', 'Scratch', 'JavaScript'],
                'price_range': '$100-300',
                'age_range': '13-18'
            },
            'microbit': {
                'sensors': ['Accelerometer', 'Magnetometer', 'Light Sensor', 'Temperature'],
                'actuators': ['LED Matrix', 'Buzzer'],
                'communication': ['Bluetooth', 'Radio'],
                'programming': ['Scratch', 'Python', 'Blockly'],
                'price_range': '$15-25',
                'age_range': '8-14'
            }
        }
        return capabilities.get(robot_type, {})
        
    def _get_setup_instructions(self) -> Dict[str, str]:
        """Get setup instructions for robot"""
        return {
            'hardware_setup': f"1. Unpack your {self.robot_type} robot\n2. Charge/install batteries\n3. Connect sensors to designated ports\n4. Power on and verify system",
            'software_setup': f"1. Install {self.robot_type} programming environment\n2. Connect robot via USB/Bluetooth\n3. Test basic motor movement\n4. Verify sensor readings",
            'testing': f"1. Run basic movement test\n2. Test all sensors\n3. Verify communication\n4. Check power levels"
        }


class CurriculumManager:
    """Manager for educational robotics curriculum"""
    
    def __init__(self, output_dir: str = "/workspace/real_world/robotics/curriculum"):
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(exist_ok=True)
        
        self.curriculum_data = {}
        self.robot_models = {}
        
        # Initialize with default robot models
        for robot_type in ['ev3', 'arduino', 'raspberry_pi', 'microbit']:
            self.robot_models[robot_type] = EducationalRobot(robot_type)
            
    def create_foundation_curriculum(self) -> Curriculum:
        """Create foundational robotics curriculum"""
        lessons = []
        
        # Lesson 1: Introduction to Robotics
        lesson1 = Lesson(
            id="intro_robotics",
            title="Introduction to Robotics",
            description="Learn what robots are and how they work",
            objective=LearningObjective(
                id="obj_intro",
                title="Understand robot basics",
                description="Students will understand what robots are and their main components",
                difficulty_level="beginner",
                prerequisites=[],
                estimated_duration=45,
                skills_covered=["robot concepts", "components", "terminology"]
            ),
            content={
                "theory": [
                    "What is a robot?",
                    "Main robot components: sensors, actuators, controller",
                    "Types of robots: mobile, industrial, educational",
                    "How robots sense their environment",
                    "How robots move and act"
                ],
                "examples": [
                    "Vacuum robots (Roomba)",
                    "Assembly line robots",
                    "Educational robots (EV3, Arduino)",
                    "Autonomous vehicles"
                ],
                "key_concepts": [
                    "Autonomous vs remote control",
                    "Feedback loops",
                    "Sensors vs actuators",
                    "Robot intelligence levels"
                ]
            },
            activities=[
                {
                    "name": "Robot Hunt",
                    "description": "Find and identify robots in your environment",
                    "time": 15,
                    "materials": ["worksheet", "camera/phone"]
                },
                {
                    "name": "Component Matching",
                    "description": "Match robot components to their functions",
                    "time": 10,
                    "materials": ["component cards", "robot diagram"]
                },
                {
                    "name": "Think-Pair-Share",
                    "description": "Discuss what makes a good robot design",
                    "time": 15,
                    "materials": ["discussion prompts"]
                }
            ],
            assessment_questions=[
                {
                    "question": "What are the three main components of a robot?",
                    "answer": "Sensors, actuators, and controller"
                },
                {
                    "question": "Give an example of a sensor and what it measures.",
                    "answer": "Ultrasonic sensor measures distance, color sensor measures light"
                },
                {
                    "question": "What is the difference between autonomous and remote control?",
                    "answer": "Autonomous robots make decisions on their own, remote control needs human input"
                }
            ],
            resources=[
                "Robot Anatomy PDF",
                "Component Reference Sheet",
                "Famous Robots Video"
            ],
            hands_on_project={
                "name": "Robot Design Challenge",
                "description": "Design a robot for a specific task",
                "deliverables": ["Sketch", "Component list", "Problem statement"],
                "time": 20
            }
        )
        lessons.append(lesson1)
        
        # Lesson 2: Basic Programming Concepts
        lesson2 = Lesson(
            id="programming_basics",
            title="Programming Your Robot",
            description="Learn basic programming concepts and robot control",
            objective=LearningObjective(
                id="obj_programming",
                title="Program basic robot behaviors",
                description="Students will understand programming concepts and write simple robot programs",
                difficulty_level="beginner",
                prerequisites=["intro_robotics"],
                estimated_duration=60,
                skills_covered=["programming logic", "variables", "loops", "conditional statements"]
            ),
            content={
                "theory": [
                    "What is programming?",
                    "Programming languages for robots",
                    "Basic syntax and structure",
                    "Variables and data types",
                    "Control structures (if/else, loops)"
                ],
                "concepts": [
                    "Sequential execution",
                    "Conditional branching",
                    "Repetition and loops",
                    "Functions and procedures",
                    "Debugging techniques"
                ]
            },
            activities=[
                {
                    "name": "Pseudocode Practice",
                    "description": "Write pseudocode for everyday tasks",
                    "time": 15,
                    "materials": ["pseudocode worksheet"]
                },
                {
                    "name": "First Robot Program",
                    "description": "Program robot to move forward and stop",
                    "time": 25,
                    "materials": ["robot", "programming software"]
                },
                {
                    "name": "Debug Challenge",
                    "description": "Find and fix bugs in given programs",
                    "time": 15,
                    "materials": ["buggy code examples"]
                }
            ],
            assessment_questions=[
                {
                    "question": "What is a variable in programming?",
                    "answer": "A variable is a storage location that holds data that can change"
                },
                {
                    "question": "When would you use a loop in your program?",
                    "answer": "To repeat actions multiple times or continue an action while a condition is true"
                },
                {
                    "question": "What is the purpose of an if/else statement?",
                    "answer": "To make decisions in the program based on conditions"
                }
            ],
            resources=[
                "Programming Basics Slides",
                "Robot Programming Cheat Sheet",
                "Common Bugs Reference"
            ],
            hands_on_project={
                "name": "Robot Dance Program",
                "description": "Program robot to perform a sequence of movements",
                "deliverables": ["Working program", "Documentation", "Demo"],
                "time": 30
            }
        )
        lessons.append(lesson2)
        
        # Lesson 3: Sensors and Input
        lesson3 = Lesson(
            id="sensors_and_input",
            title="Understanding Robot Sensors",
            description="Learn about different sensors and how robots use them",
            objective=LearningObjective(
                id="obj_sensors",
                title="Use sensors for robot control",
                description="Students will understand different sensor types and implement sensor-based control",
                difficulty_level="beginner",
                prerequisites=["programming_basics"],
                estimated_duration=75,
                skills_covered=["sensor types", "data interpretation", "sensor integration"]
            ),
            content={
                "sensor_types": [
                    "Ultrasonic sensors - distance measurement",
                    "Color sensors - light and color detection",
                    "Touch sensors - physical contact detection",
                    "Gyroscope sensors - orientation and rotation",
                    "Accelerometer sensors - movement and acceleration"
                ],
                "concepts": [
                    "How sensors work",
                    "Reading sensor data",
                    "Converting analog to digital",
                    "Noise and calibration",
                    "Sensor fusion basics"
                ]
            },
            activities=[
                {
                    "name": "Sensor Exploration",
                    "description": "Test different sensors and record readings",
                    "time": 25,
                    "materials": ["robot with sensors", "data collection sheet"]
                },
                {
                    "name": "Distance Measurement Challenge",
                    "description": "Use ultrasonic sensor to measure distances accurately",
                    "time": 20,
                    "materials": ["ultrasonic sensor", "measuring tape", "calculator"]
                },
                {
                    "name": "Color Detection Game",
                    "description": "Program robot to detect and respond to different colors",
                    "time": 25,
                    "materials": ["color sensor", "colored objects"]
                }
            ],
            assessment_questions=[
                {
                    "question": "Name three different types of sensors and their purpose.",
                    "answer": "Ultrasonic (distance), Color (light/color), Touch (contact), Gyroscope (rotation)"
                },
                {
                    "question": "Why might sensor readings be inaccurate sometimes?",
                    "answer": "Noise, interference, calibration issues, environmental factors"
                },
                {
                    "question": "How can you improve sensor accuracy?",
                    "answer": "Calibration, averaging multiple readings, filtering noise"
                }
            ],
            resources=[
                "Sensor Types Reference",
                "Data Collection Template",
                "Calibration Procedures"
            ],
            hands_on_project={
                "name": "Smart Sensor Station",
                "description": "Create a station that uses multiple sensors to gather data",
                "deliverables": ["Working sensor array", "Data log", "Analysis report"],
                "time": 45
            }
        )
        lessons.append(lesson3)
        
        return Curriculum(
            name="Foundations of Educational Robotics",
            description="Introduction to robotics concepts, programming, and sensors",
            target_audience="Ages 10-16, no prior experience required",
            total_duration=3,  # 3 hours
            lessons=lessons,
            prerequisites=[],
            learning_outcomes=[
                "Understand basic robot components and concepts",
                "Write simple robot programs",
                "Use sensors for robot control",
                "Debug and troubleshoot robot programs",
                "Apply problem-solving to robotics challenges"
            ],
            assessment_methods=[
                "Hands-on demonstrations",
                "Written assessments",
                "Peer evaluation",
                "Project presentations",
                "Self-reflection journals"
            ]
        )
        
    def create_intermediate_curriculum(self) -> Curriculum:
        """Create intermediate level curriculum"""
        lessons = []
        
        # Lesson 4: Autonomous Navigation
        lesson4 = Lesson(
            id="autonomous_navigation",
            title="Autonomous Robot Navigation",
            description="Program robots to navigate without human control",
            objective=LearningObjective(
                id="obj_navigation",
                title="Navigate autonomously using sensors",
                description="Students will implement autonomous navigation using sensors and path planning",
                difficulty_level="intermediate",
                prerequisites=["sensors_and_input"],
                estimated_duration=90,
                skills_covered=["path planning", "sensor fusion", "decision making", "obstacle avoidance"]
            ),
            content={
                "navigation_concepts": [
                    "Dead reckoning vs. external references",
                    "Odometry and wheel encoders",
                    "Landmark-based navigation",
                    "SLAM (Simultaneous Localization and Mapping)",
                    "Obstacle detection and avoidance"
                ],
                "algorithms": [
                    "Wall-following algorithm",
                    "Bug algorithm",
                    "Right-hand rule",
                    "Potential fields",
                    "A* path planning (simplified)"
                ]
            },
            activities=[
                {
                    "name": "Wall Following Challenge",
                    "description": "Program robot to follow a wall autonomously",
                    "time": 30,
                    "materials": ["robot", "test track with walls"]
                },
                {
                    "name": "Obstacle Course",
                    "description": "Navigate through an obstacle course",
                    "time": 35,
                    "materials": ["obstacle course", "timer"]
                },
                {
                    "name": "Map Making Exercise",
                    "description": "Create a map while navigating",
                    "time": 25,
                    "materials": ["graph paper", "robot"]
                }
            ],
            assessment_questions=[
                {
                    "question": "What is the difference between dead reckoning and landmark-based navigation?",
                    "answer": "Dead reckoning uses internal sensors (odometry), landmark-based uses external references"
                },
                {
                    "question": "Describe the right-hand rule algorithm.",
                    "answer": "Always try to turn right, if not possible go straight, if not possible turn left, if stuck turn around"
                },
                {
                    "question": "Why might a robot get lost during navigation?",
                    "answer": "Wheel slip, encoder errors, unmodeled obstacles, sensor noise"
                }
            ],
            resources=[
                "Navigation Algorithms Guide",
                "Obstacle Course Layouts",
                "Mapping Templates"
            ],
            hands_on_project={
                "name": "Autonomous Maze Solver",
                "description": "Program robot to solve a maze completely autonomously",
                "deliverables": ["Working maze solver", "Algorithm explanation", "Performance analysis"],
                "time": 60
            }
        )
        lessons.append(lesson4)
        
        return Curriculum(
            name="Intermediate Robotics Programming",
            description="Advanced programming concepts and autonomous robot behaviors",
            target_audience="Ages 12-18, some programming experience required",
            total_duration=6,  # 6 hours
            lessons=lessons,
            prerequisites=["Foundations of Educational Robotics"],
            learning_outcomes=[
                "Implement autonomous navigation algorithms",
                "Combine multiple sensors for robust control",
                "Debug complex robot behaviors",
                "Design and test navigation strategies",
                "Optimize robot performance"
            ],
            assessment_methods=[
                "Complex problem solving",
                "Code review",
                "Performance optimization",
                "Team collaboration",
                "Technical presentations"
            ]
        )
        
    def create_advanced_curriculum(self) -> Curriculum:
        """Create advanced level curriculum"""
        lessons = []
        
        # Lesson 5: Computer Vision
        lesson5 = Lesson(
            id="computer_vision",
            title="Computer Vision for Robotics",
            description="Add vision capabilities to robots",
            objective=LearningObjective(
                id="obj_vision",
                title="Implement computer vision features",
                description="Students will integrate computer vision for advanced robot behaviors",
                difficulty_level="advanced",
                prerequisites=["autonomous_navigation"],
                estimated_duration=120,
                skills_covered=["image processing", "object detection", "visual servoing", "OpenCV basics"]
            ),
            content={
                "vision_concepts": [
                    "How cameras work",
                    "Image representation and color spaces",
                    "Basic image processing operations",
                    "Object detection and recognition",
                    "Feature detection and matching"
                ],
                "applications": [
                    "Line following with vision",
                    "Object recognition and manipulation",
                    "Visual servoing",
                    "Map building with vision",
                    "Human-robot interaction"
                ],
                "tools": [
                    "OpenCV library",
                    "Color filtering",
                    "Edge detection",
                    "Contour analysis",
                    "Template matching"
                ]
            },
            activities=[
                {
                    "name": "Vision Calibration",
                    "description": "Set up and calibrate camera for robot vision",
                    "time": 25,
                    "materials": ["camera", "calibration patterns", "calibration software"]
                },
                {
                    "name": "Color-Based Tracking",
                    "description": "Track objects of specific colors",
                    "time": 35,
                    "materials": ["colored objects", "vision software"]
                },
                {
                    "name": "Shape Recognition",
                    "description": "Detect and classify different shapes",
                    "time": 40,
                    "materials": ["shape templates", "test objects"]
                },
                {
                    "name": "Vision-Guored Navigation",
                    "description": "Navigate using visual landmarks",
                    "time": 20,
                    "materials": ["landmark course"]
                }
            ],
            assessment_questions=[
                {
                    "question": "How does a camera capture digital images?",
                    "answer": "Light hits sensors, converted to electrical signals, digitized into pixels"
                },
                {
                    "question": "What is the difference between RGB and HSV color spaces?",
                    "answer": "RGB combines red, green, blue; HSV uses hue, saturation, value for color perception"
                },
                {
                    "question": "Name three applications of computer vision in robotics.",
                    "answer": "Object recognition, navigation, quality inspection, human detection"
                }
            ],
            resources=[
                "OpenCV Tutorial Guide",
                "Color Spaces Reference",
                "Vision Algorithms Cheat Sheet",
                "Sample Image Datasets"
            ],
            hands_on_project={
                "name": "Vision-Guored Robot Assistant",
                "description": "Create a robot that uses vision to help with specific tasks",
                "deliverables": ["Working vision system", "Task implementation", "Video demonstration"],
                "time": 90
            }
        )
        lessons.append(lesson5)
        
        return Curriculum(
            name="Advanced Robotics and AI",
            description="AI techniques and advanced robotics applications",
            target_audience="Ages 14-18, strong programming background",
            total_duration=12,  # 12 hours
            lessons=lessons,
            prerequisites=["Intermediate Robotics Programming"],
            learning_outcomes=[
                "Implement computer vision systems",
                "Apply machine learning to robotics",
                "Design complex robotic behaviors",
                "Integrate multiple AI technologies",
                "Lead robotics projects"
            ],
            assessment_methods=[
                "Complex project development",
                "Research presentations",
                "Peer mentorship",
                "Industry project simulation",
                "Capstone project"
            ]
        )
        
    def generate_lesson_plans(self, curriculum: Curriculum, output_dir: Optional[str] = None):
        """Generate detailed lesson plans for curriculum"""
        if output_dir:
            lesson_dir = Path(output_dir)
        else:
            lesson_dir = self.output_dir / "lesson_plans"
            
        lesson_dir.mkdir(exist_ok=True)
        
        for lesson in curriculum.lessons:
            lesson_plan = self._create_detailed_lesson_plan(lesson)
            filename = lesson_dir / f"{lesson.id}_lesson_plan.md"
            
            with open(filename, 'w') as f:
                f.write(lesson_plan)
                
        print(f"Generated {len(curriculum.lessons)} lesson plans in {lesson_dir}")
        
    def _create_detailed_lesson_plan(self, lesson: Lesson) -> str:
        """Create detailed markdown lesson plan"""
        plan = f"""# {lesson.title}

## Lesson Overview
- **Duration**: {lesson.objective.estimated_duration} minutes
- **Difficulty**: {lesson.objective.difficulty_level.title()}
- **Prerequisites**: {', '.join(lesson.objective.prerequisites) if lesson.objective.prerequisites else 'None'}

## Learning Objectives
By the end of this lesson, students will be able to:
"""
        for objective in lesson.objective.skills_covered:
            plan += f"- {objective.title()}\n"
            
        plan += f"""
## Materials Needed
- Robot and programming environment
- Sensors as specified in activities
- Computers with appropriate software
- Worksheets and handouts
- Project materials as needed

## Lesson Structure

### Introduction (10 minutes)
1. **Hook**: {lesson.activities[0]['description'] if lesson.activities else 'Interactive demonstration'}
2. **Review**: Previous lesson concepts
3. **Preview**: Today's learning objectives

### Theory Section (20 minutes)
"""
        
        if 'theory' in lesson.content:
            for i, concept in enumerate(lesson.content['theory'], 1):
                plan += f"{i}. {concept}\n"
                
        plan += """
### Hands-On Activities (Variable time)
"""
        
        for i, activity in enumerate(lesson.activities, 1):
            plan += f"""
#### Activity {i}: {activity['name']} ({activity['time']} minutes)
- **Description**: {activity['description']}
- **Materials**: {', '.join(activity['materials'])}
- **Instructions**:
  1. [Step-by-step instructions]
  2. [Student task details]
  3. [Expected outcomes]
"""
            
        plan += """
### Assessment (10 minutes)
"""
        
        for i, question in enumerate(lesson.assessment_questions, 1):
            plan += f"""
**Question {i}**: {question['question']}
*Expected Answer*: {question['answer']}
"""
            
        if lesson.hands_on_project:
            plan += f"""
### Project Component
**{lesson.hands_on_project['name']}** ({lesson.hands_on_project['time']} minutes)

- **Description**: {lesson.hands_on_project['description']}
- **Deliverables**: {', '.join(lesson.hands_on_project['deliverables'])}
- **Grading Rubric**: [To be provided separately]

## Extension Activities
- Advanced challenges for fast finishers
- Cross-curricular connections
- Real-world applications research

## Resources
"""
        
        for resource in lesson.resources:
            plan += f"- {resource}\n"
            
        plan += """
## Differentiation Strategies
- **For struggling students**: Provide additional scaffolding and peer support
- **For advanced students**: Offer extension challenges and leadership roles
- **For different learning styles**: Include visual, auditory, and kinesthetic activities

## Safety Considerations
- [Robot operation safety]
- [Hardware handling procedures]
- [Workspace organization]

## Reflection Questions
1. What was the most challenging part of today's lesson?
2. How could you improve your robot's performance?
3. What real-world applications can you think of for this technology?

---
*Lesson plan generated by Educational Robotics Curriculum Framework*
"""
        
        return plan
        
    def generate_teacher_guide(self, curriculum: Curriculum, output_dir: Optional[str] = None):
        """Generate comprehensive teacher guide"""
        if output_dir:
            guide_dir = Path(output_dir)
        else:
            guide_dir = self.output_dir / "teacher_guides"
            
        guide_dir.mkdir(exist_ok=True)
        
        guide_content = f"""# Teacher Guide: {curriculum.name}

## Curriculum Overview

### Target Audience
{curriculum.target_audience}

### Total Duration
{curriculum.total_duration} hours

### Prerequisites
{', '.join(curriculum.prerequisites) if curriculum.prerequisites else 'None required'}

## Learning Outcomes
"""
        
        for outcome in curriculum.learning_outcomes:
            guide_content += f"- {outcome}\n"
            
        guide_content += f"""
## Assessment Methods
"""
        
        for method in curriculum.assessment_methods:
            guide_content += f"- {method}\n"
            
        guide_content += f"""
## Equipment and Software Requirements

### Hardware Requirements
- Educational robots (EV3, Arduino, Raspberry Pi, or micro:bit)
- Sensors: ultrasonic, color, touch, IMU (varies by robot type)
- Computers with appropriate programming environments
- Network connectivity for software updates

### Software Requirements
- Robot-specific programming environments
- Simulator software (optional but recommended)
- Documentation and reference materials

## Lesson-by-Lesson Guide

"""
        
        for lesson in curriculum.lessons:
            guide_content += f"""### {lesson.title}

**Duration**: {lesson.objective.estimated_duration} minutes
**Difficulty**: {lesson.objective.difficulty_level.title()}

#### Preparation Checklist
- [ ] Test all robots and sensors
- [ ] Prepare programming environment
- [ ] Print student materials
- [ ] Set up activity stations
- [ ] Review safety procedures

#### Common Student Challenges
1. **Programming Errors**: Provide debugging strategies and peer support
2. **Hardware Issues**: Have backup robots and troubleshooting guides ready
3. **Conceptual Understanding**: Use multiple examples and analogies

#### Extension Opportunities
- Connect to STEM careers
- Research current robotics applications
- Design original challenges
- Peer teaching opportunities

---
"""
            
        guide_content += """
## Assessment Rubrics

### Programming Skills
- **Excellent (4)**: Code is clean, well-documented, efficient, and includes error handling
- **Good (3)**: Code works correctly with minor issues, basic documentation
- **Satisfactory (2)**: Code works but may have bugs, minimal documentation
- **Needs Improvement (1)**: Code has significant issues or doesn't work

### Problem Solving
- **Excellent (4)**: Systematically approaches problems, considers multiple solutions
- **Good (3)**: Identifies problems and tries solutions effectively
- **Satisfactory (2)**: Shows some problem-solving ability
- **Needs Improvement (1)**: Difficulty identifying or solving problems

### Collaboration
- **Excellent (4)**: Excellent teamwork, helps others, shares ideas effectively
- **Good (3)**: Good collaboration skills, contributes to group work
- **Satisfactory (2)**: Participates in group activities
- **Needs Improvement (1)**: Limited participation or collaboration

## Troubleshooting Guide

### Common Hardware Issues
1. **Robot won't connect**: Check USB/Bluetooth, restart robot and computer
2. **Sensors not responding**: Verify connections, check power, calibrate if needed
3. **Motors not working**: Check motor connections, verify battery level
4. **Communication errors**: Ensure compatible software versions

### Common Software Issues
1. **Programs won't compile**: Check syntax, verify libraries installed
2. **Robots behave unexpectedly**: Review logic, add debugging output
3. **Performance issues**: Optimize code, check for infinite loops
4. **File corruption**: Save work frequently, maintain backups

## Professional Development Resources

### Recommended Training
- Robotics education workshops
- Programming pedagogy courses
- STEM education conferences
- Online tutorials and certifications

### Staying Current
- Follow robotics education blogs and newsletters
- Participate in teacher communities
- Attend industry conferences
- Collaborate with local tech companies

## Additional Resources

### Student Resources
- [Educational robotics websites]
- [Programming tutorials]
- [Project inspiration galleries]
- [Career information]

### Teacher Resources
- [Curriculum planning tools]
- [Assessment templates]
- [Professional learning communities]
- [Grant opportunities]

---
*Teacher guide generated by Educational Robotics Curriculum Framework*
"""
        
        filename = guide_dir / f"{curriculum.name.lower().replace(' ', '_')}_teacher_guide.md"
        with open(filename, 'w') as f:
            f.write(guide_content)
            
        print(f"Generated teacher guide: {filename}")
        
    def generate_all_materials(self, output_base_dir: Optional[str] = None):
        """Generate all curriculum materials"""
        if output_base_dir:
            base_dir = Path(output_base_dir)
        else:
            base_dir = self.output_dir
            
        base_dir.mkdir(exist_ok=True)
        
        # Create subdirectories
        (base_dir / "lesson_plans").mkdir(exist_ok=True)
        (base_dir / "teacher_guides").mkdir(exist_ok=True)
        (base_dir / "student_materials").mkdir(exist_ok=True)
        (base_dir / "assessments").mkdir(exist_ok=True)
        (base_dir / "resources").mkdir(exist_ok=True)
        
        # Generate foundation curriculum
        foundation_curriculum = self.create_foundation_curriculum()
        self.generate_lesson_plans(foundation_curriculum, str(base_dir / "lesson_plans"))
        self.generate_teacher_guide(foundation_curriculum, str(base_dir / "teacher_guides"))
        
        # Generate intermediate curriculum
        intermediate_curriculum = self.create_intermediate_curriculum()
        self.generate_lesson_plans(intermediate_curriculum, str(base_dir / "lesson_plans"))
        self.generate_teacher_guide(intermediate_curriculum, str(base_dir / "teacher_guides"))
        
        # Generate advanced curriculum
        advanced_curriculum = self.create_advanced_curriculum()
        self.generate_lesson_plans(advanced_curriculum, str(base_dir / "lesson_plans"))
        self.generate_teacher_guide(advanced_curriculum, str(base_dir / "teacher_guides"))
        
        # Generate overview document
        self._generate_curriculum_overview(base_dir)
        
        print(f"All curriculum materials generated in {base_dir}")
        
    def _generate_curriculum_overview(self, output_dir: Path):
        """Generate curriculum overview document"""
        overview = """# Educational Robotics Curriculum Overview

## Curriculum Pathways

### Level 1: Foundations (Beginner)
**Duration**: 3 hours | **Age Range**: 10-16 | **Prerequisites**: None

Focus: Basic concepts, programming, and sensors
- Introduction to Robotics
- Basic Programming Concepts  
- Sensors and Input

### Level 2: Intermediate
**Duration**: 6 hours | **Age Range**: 12-18 | **Prerequisites**: Level 1

Focus: Autonomous navigation and advanced programming
- Autonomous Navigation
- Complex Behaviors
- Project Development

### Level 3: Advanced
**Duration**: 12 hours | **Age Range**: 14-18 | **Prerequisites**: Level 2

Focus: AI, computer vision, and complex systems
- Computer Vision
- Machine Learning Applications
- Capstone Project

## Learning Progression

Each level builds upon previous knowledge while introducing new concepts:

1. **Conceptual Understanding**: Students learn what robots are and how they work
2. **Programming Skills**: Students learn to program robot behaviors
3. **Sensor Integration**: Students use sensors for robot control
4. **Autonomous Behavior**: Students create self-directed robot actions
5. **Advanced Applications**: Students implement AI and vision systems

## Assessment Strategy

### Formative Assessment
- Daily check-ins and exit tickets
- Peer code reviews
- Quick debugging challenges
- Progress demonstrations

### Summative Assessment
- Project-based assessments
- Competency demonstrations
- Portfolio development
- Peer evaluations

## Differentiation Approaches

### For Diverse Learners
- Multiple entry points for different skill levels
- Hands-on, visual, and textual materials
- Flexible grouping strategies
- Choice in project topics

### For Advanced Students
- Leadership opportunities
- Independent research projects
- Mentoring roles
- Competition participation

## Cross-Curricular Connections

### Mathematics
- Geometry and spatial reasoning
- Algebra and variables
- Statistics and data analysis
- Calculus concepts (advanced level)

### Science
- Physics of motion and forces
- Sensors and measurement
- Scientific method and experimentation
- Technology and engineering design

### Language Arts
- Technical documentation
- Presentation skills
- Research and reporting
- Creative problem description

## Implementation Guidelines

### Classroom Setup
- Flexible workspace arrangement
- Charging stations for robots
- Materials storage systems
- Demonstration area

### Safety Considerations
- Robot operation guidelines
- Electrical safety procedures
- Workspace organization
- Emergency procedures

### Professional Development
- Initial teacher training required
- Ongoing support resources
- Community of practice
- Technology updates

## Getting Started

### First Steps
1. Choose robot platform based on budget and age group
2. Review teacher guide and materials
3. Set up robots and programming environment
4. Begin with Level 1, Lesson 1

### Support Resources
- Technical support for hardware/software
- Online community of educators
- Regular curriculum updates
- Professional development opportunities

---

*This curriculum framework provides a comprehensive pathway for robotics education from beginner to advanced levels.*
"""
        
        with open(output_dir / "curriculum_overview.md", 'w') as f:
            f.write(overview)


if __name__ == "__main__":
    print("Generating Educational Robotics Curriculum Materials...")
    
    # Create curriculum manager
    curriculum_manager = CurriculumManager()
    
    # Generate all materials
    curriculum_manager.generate_all_materials()
    
    # Create individual curriculum documents
    foundation = curriculum_manager.create_foundation_curriculum()
    intermediate = curriculum_manager.create_intermediate_curriculum()
    advanced = curriculum_manager.create_advanced_curriculum()
    
    # Save curriculum data
    curricula_data = {
        "foundation": asdict(foundation),
        "intermediate": asdict(intermediate),
        "advanced": asdict(advanced)
    }
    
    output_path = Path("/workspace/real_world/robotics/curriculum/curricula_data.json")
    with open(output_path, 'w') as f:
        json.dump(curricula_data, f, indent=2)
        
    print(f"Curriculum data saved to {output_path}")
    print("Curriculum generation complete!")
