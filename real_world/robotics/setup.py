#!/usr/bin/env python3
"""
Educational Robotics Integration Framework Setup
Comprehensive setup and configuration for MultiOS robotics
"""

import os
import sys
import subprocess
import json
from pathlib import Path

class RoboticsFrameworkSetup:
    def __init__(self):
        self.root_dir = Path(__file__).parent
        self.config_file = self.root_dir / "config" / "robotics_config.json"
        self.requirements_file = self.root_dir / "requirements.txt"
        
    def create_directory_structure(self):
        """Create complete directory structure"""
        directories = [
            "core", "hardware", "control", "vision", "communication", 
            "simulation", "demos", "curriculum", "examples", "tests", 
            "docs", "assets", "config", "logs", "data"
        ]
        
        for dir_name in directories:
            (self.root_dir / dir_name).mkdir(exist_ok=True)
            (self.root_dir / dir_name / "__init__.py").touch()
            
        print("✓ Directory structure created")
        
    def create_configuration(self):
        """Create default configuration files"""
        config = {
            "robot_config": {
                "default_robot": "ev3",
                "hardware_modules": {
                    "ev3": {
                        "motors": {
                            "left_motor": "port_A",
                            "right_motor": "port_D"
                        },
                        "sensors": {
                            "color_sensor": "port_1",
                            "ultrasonic_sensor": "port_4"
                        }
                    },
                    "arduino": {
                        "motors": {
                            "left_motor": [5, 6],
                            "right_motor": [9, 10]
                        },
                        "sensors": {
                            "ultrasonic": [7, 8],
                            "line_sensor": [0, 1, 2, 3, 4]
                        }
                    },
                    "raspberry_pi": {
                        "motors": {
                            "left_motor": [18, 19],
                            "right_motor": [20, 21]
                        },
                        "sensors": {
                            "camera": "/dev/video0",
                            "imu": "/dev/i2c-1"
                        }
                    }
                }
            },
            "control_parameters": {
                "pid_controller": {
                    "kp": 1.0,
                    "ki": 0.1,
                    "kd": 0.05
                },
                "path_planning": {
                    "max_speed": 0.5,
                    "acceleration": 0.2,
                    "turn_rate": 1.0
                }
            },
            "simulation": {
                "physics_engine": "pygame",
                "environment": {
                    "width": 10.0,
                    "height": 8.0,
                    "obstacles": []
                }
            },
            "communication": {
                "serial": {
                    "baudrate": 115200,
                    "timeout": 1.0
                },
                "wifi": {
                    "port": 8080,
                    "protocol": "tcp"
                }
            }
        }
        
        self.config_file.parent.mkdir(exist_ok=True)
        with open(self.config_file, 'w') as f:
            json.dump(config, f, indent=4)
            
        print("✓ Configuration file created")
        
    def create_requirements(self):
        """Create requirements.txt file"""
        requirements = [
            "numpy>=1.21.0",
            "opencv-python>=4.5.0",
            "scipy>=1.7.0", 
            "matplotlib>=3.4.0",
            "pyserial>=3.5",
            "requests>=2.25.0",
            "websockets>=10.0",
            "pygame>=2.0.0",
            "scikit-image>=0.18.0",
            "pandas>=1.3.0",
            "pathlib2>=2.3.0",
            "pytest>=6.0.0"
        ]
        
        with open(self.requirements_file, 'w') as f:
            f.write('\n'.join(requirements))
            
        print("✓ Requirements file created")
        
    def create_setup_script(self):
        """Create Python setup script"""
        setup_content = '''#!/usr/bin/env python3
from setuptools import setup, find_packages

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="educational-robotics",
    version="1.0.0",
    author="MultiOS Team",
    author_email="team@multios.org",
    description="Educational Robotics Integration Framework for MultiOS",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/multios/robotics",
    packages=find_packages(),
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Education",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
    ],
    python_requires=">=3.8",
    install_requires=[
        "numpy>=1.21.0",
        "opencv-python>=4.5.0",
        "scipy>=1.7.0",
        "matplotlib>=3.4.0",
        "pyserial>=3.5",
        "requests>=2.25.0",
        "websockets>=10.0",
        "pygame>=2.0.0",
        "scikit-image>=0.18.0",
        "pandas>=1.3.0",
        "pathlib2>=2.3.0",
    ],
    extras_require={
        "dev": [
            "pytest>=6.0.0",
            "black>=21.0.0",
            "flake8>=3.8.0",
            "mypy>=0.812",
        ],
    },
    entry_points={
        "console_scripts": [
            "robotics-demo=robotics.demos.main:main",
            "robotics-sim=robotics.simulation.main:main",
            "robotics-test=robotics.tests.main:main",
        ],
    },
)
'''
        
        with open(self.root_dir / "setup.py", 'w') as f:
            f.write(setup_content)
            
        print("✓ Setup script created")
        
    def create_main_package(self):
        """Create main __init__.py with framework entry points"""
        init_content = '''"""
Educational Robotics Integration Framework for MultiOS

This framework provides comprehensive robotics capabilities including:
- Hardware abstraction for popular educational robots
- Motion control and path planning algorithms
- Sensor fusion and computer vision
- Communication protocols and networking
- Simulation environment for safe experimentation
- Educational demos and curriculum
"""

__version__ = "1.0.0"
__author__ = "MultiOS Team"
__email__ = "team@multios.org"

# Core imports
from .core.robot import Robot, RobotState
from .core.base_hardware import BaseHardware
from .core.base_sensor import BaseSensor
from .core.base_actuator import BaseActuator

# Hardware implementations
from .hardware.ev3_hardware import EV3Hardware
from .hardware.arduino_hardware import ArduinoHardware
from .hardware.raspberry_pi_hardware import RaspberryPiHardware

# Motion control
from .control.pid_controller import PIDController
from .control.path_planner import PathPlanner
from .control.trajectory import TrajectoryGenerator

# Vision and sensors
from .vision.camera import Camera
from .vision.line_detector import LineDetector
from .vision.object_detector import ObjectDetector

# Communication
from .communication.serial_comm import SerialCommunication
from .communication.wifi_comm import WiFiCommunication

# Simulation
from .simulation.environment import SimulationEnvironment
from .simulation.physics import PhysicsEngine

# Demos
from .demos.line_following import LineFollowingDemo
from .demos.obstacle_avoidance import ObstacleAvoidanceDemo
from .demos.maze_solving import MazeSolvingDemo

# Quick start functions
def create_robot(robot_type="ev3", **kwargs):
    """
    Create a robot instance of the specified type
    
    Args:
        robot_type: Type of robot ("ev3", "arduino", "raspberry_pi")
        **kwargs: Additional configuration parameters
    
    Returns:
        Robot instance
    """
    if robot_type.lower() == "ev3":
        return Robot(EV3Hardware(**kwargs))
    elif robot_type.lower() == "arduino":
        return Robot(ArduinoHardware(**kwargs))
    elif robot_type.lower() == "raspberry_pi":
        return Robot(RaspberryPiHardware(**kwargs))
    else:
        raise ValueError(f"Unsupported robot type: {robot_type}")

def start_demo(demo_name, robot=None, **kwargs):
    """
    Start an educational demo
    
    Args:
        demo_name: Name of the demo ("line_following", "obstacle_avoidance", "maze_solving")
        robot: Robot instance (will create default if not provided)
        **kwargs: Demo-specific parameters
    """
    if robot is None:
        robot = create_robot()
        
    if demo_name.lower() == "line_following":
        demo = LineFollowingDemo(robot, **kwargs)
    elif demo_name.lower() == "obstacle_avoidance":
        demo = ObstacleAvoidanceDemo(robot, **kwargs)
    elif demo_name.lower() == "maze_solving":
        demo = MazeSolvingDemo(robot, **kwargs)
    else:
        raise ValueError(f"Unknown demo: {demo_name}")
        
    demo.start()
    return demo

__all__ = [
    "Robot", "RobotState",
    "BaseHardware", "BaseSensor", "BaseActuator",
    "EV3Hardware", "ArduinoHardware", "RaspberryPiHardware",
    "PIDController", "PathPlanner", "TrajectoryGenerator",
    "Camera", "LineDetector", "ObjectDetector",
    "SerialCommunication", "WiFiCommunication",
    "SimulationEnvironment", "PhysicsEngine",
    "LineFollowingDemo", "ObstacleAvoidanceDemo", "MazeSolvingDemo",
    "create_robot", "start_demo"
]
'''
        
        with open(self.root_dir / "robotics" / "__init__.py", 'w') as f:
            f.write(init_content)
            
        print("✓ Main package created")
        
    def install_dependencies(self):
        """Install required dependencies"""
        print("Installing dependencies...")
        try:
            subprocess.run([
                sys.executable, "-m", "pip", "install", "-r", 
                str(self.requirements_file)
            ], check=True, capture_output=True)
            print("✓ Dependencies installed successfully")
        except subprocess.CalledProcessError as e:
            print(f"✗ Failed to install dependencies: {e}")
            
    def run_tests(self):
        """Run basic framework tests"""
        print("Running tests...")
        test_dir = self.root_dir / "tests"
        if test_dir.exists():
            try:
                subprocess.run([
                    sys.executable, "-m", "pytest", str(test_dir), "-v"
                ], check=True, capture_output=True)
                print("✓ Tests passed")
            except subprocess.CalledProcessError as e:
                print(f"✗ Tests failed: {e}")
                
    def setup(self):
        """Run complete setup process"""
        print("Setting up Educational Robotics Framework...")
        print("=" * 50)
        
        self.create_directory_structure()
        self.create_configuration()
        self.create_requirements()
        self.create_setup_script()
        self.create_main_package()
        
        print("=" * 50)
        print("Setup complete! You can now:")
        print("1. Install the package: pip install -e .")
        print("2. Run examples: python examples/basic_movement.py")
        print("3. Start demos: python -c 'from robotics import start_demo; start_demo(\"line_following\")'")

if __name__ == "__main__":
    setup = RoboticsFrameworkSetup()
    setup.setup()