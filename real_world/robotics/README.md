# Educational Robotics Integration Framework for MultiOS

A comprehensive educational robotics framework designed for MultiOS that provides hardware abstraction, motion control, sensor fusion, computer vision, and communication protocols for popular educational robots.

## Features

- **Hardware Abstraction Layer**: Support for popular educational robots (LEGO EV3, Arduino, Raspberry Pi robots, micro:bit)
- **Motion Control**: PID controllers, path planning algorithms, and trajectory generation
- **Sensor Fusion**: Integration of multiple sensors (ultrasonic, infrared, IMU, camera)
- **Computer Vision**: Object detection, line following, and visual servoing
- **Communication Protocols**: WiFi, Bluetooth, and serial communication
- **Simulation Environment**: Safe testing and experimentation without physical hardware
- **Educational Curriculum**: Step-by-step tutorials and guided learning experiences
- **Real-world Demos**: Line following, obstacle avoidance, maze solving

## Quick Start

```python
from robotics import Robot, EV3Hardware, PathPlanner

# Initialize a LEGO EV3 robot
robot = Robot(EV3Hardware())

# Start line following demo
robot.start_demo('line_following')

# Navigate to a target position
planner = PathPlanner(robot)
planner.navigate_to(2.0, 1.5)  # Navigate to (2.0, 1.5) meters
```

## Installation

1. Install dependencies:
```bash
pip install numpy opencv-python scipy matplotlib pyserial
```

2. Install robotics framework:
```bash
cd /workspace/real_world/robotics
python setup.py install
```

3. Run examples:
```bash
python examples/basic_movement.py
```

## Documentation

- [Hardware Abstraction](docs/hardware_abstraction.md)
- [Motion Control](docs/motion_control.md)
- [Computer Vision](docs/computer_vision.md)
- [Communication](docs/communication.md)
- [Simulation](docs/simulation.md)
- [Curriculum](curriculum/README.md)

## Hardware Support

- LEGO Mindstorms EV3/NXT
- Arduino-based robots
- Raspberry Pi robots
- micro:bit robots
- Custom robot platforms

## License

MIT License - see LICENSE file for details