"""
Path Planning and Trajectory Generation

Provides path planning algorithms and trajectory generation for robot navigation
"""

import numpy as np
import matplotlib.pyplot as plt
from typing import List, Tuple, Optional, Dict, Any
from dataclasses import dataclass
from abc import ABC, abstractmethod
import heapq
import math


@dataclass
class Point:
    """2D Point representation"""
    x: float
    y: float
    
    def __add__(self, other):
        return Point(self.x + other.x, self.y + other.y)
        
    def __sub__(self, other):
        return Point(self.x - other.x, self.y - other.y)
        
    def __mul__(self, scalar):
        return Point(self.x * scalar, self.y * scalar)
        
    def distance_to(self, other) -> float:
        return math.sqrt((self.x - other.x)**2 + (self.y - other.y)**2)
        
    def __repr__(self):
        return f"Point({self.x:.2f}, {self.y:.2f})"


@dataclass
class Path:
    """Path representation with waypoints"""
    points: List[Point]
    total_length: float = 0.0
    
    def __post_init__(self):
        self._compute_length()
        
    def _compute_length(self):
        """Calculate total path length"""
        self.total_length = 0.0
        for i in range(len(self.points) - 1):
            self.total_length += self.points[i].distance_to(self.points[i + 1])
            
    def get_point_at_distance(self, distance: float) -> Point:
        """Get point at specified distance along path"""
        if not self.points or distance <= 0:
            return self.points[0] if self.points else Point(0, 0)
        if distance >= self.total_length:
            return self.points[-1] if self.points else Point(0, 0)
            
        remaining = distance
        for i in range(len(self.points) - 1):
            segment = self.points[i].distance_to(self.points[i + 1])
            if remaining <= segment:
                # Interpolate within segment
                ratio = remaining / segment
                return Point(
                    self.points[i].x + ratio * (self.points[i + 1].x - self.points[i].x),
                    self.points[i].y + ratio * (self.points[i + 1].y - self.points[i].y)
                )
            remaining -= segment
            
        return self.points[-1]
        
    def get_direction_at_distance(self, distance: float) -> float:
        """Get direction angle at specified distance along path"""
        if len(self.points) < 2:
            return 0.0
            
        if distance <= 0:
            start = self.points[0]
            end = self.points[1]
        elif distance >= self.total_length:
            start = self.points[-2]
            end = self.points[-1]
        else:
            remaining = distance
            for i in range(len(self.points) - 1):
                segment = self.points[i].distance_to(self.points[i + 1])
                if remaining <= segment:
                    start = self.points[i]
                    end = self.points[i + 1]
                    break
                remaining -= segment
                
        dx = end.x - start.x
        dy = end.y - start.y
        return math.atan2(dy, dx)


@dataclass
class TrajectoryPoint:
    """Trajectory point with time information"""
    x: float
    y: float
    theta: float  # Orientation
    vx: float     # Linear velocity x-component
    vy: float     # Linear velocity y-component
    omega: float  # Angular velocity
    time: float   # Timestamp


class BasePathPlanner(ABC):
    """Abstract base class for path planners"""
    
    @abstractmethod
    def plan(self, start: Point, goal: Point, obstacles: List[Point]) -> Path:
        """Plan a path from start to goal avoiding obstacles"""
        pass


class AStarPlanner(BasePathPlanner):
    """A* path planning algorithm"""
    
    def __init__(self, grid_resolution: float = 0.1):
        self.grid_resolution = grid_resolution
        
    def plan(self, start: Point, goal: Point, obstacles: List[Point]) -> Path:
        """Plan path using A* algorithm"""
        # Convert continuous coordinates to grid coordinates
        grid_start = self._to_grid(start)
        grid_goal = self._to_grid(goal)
        
        # Create obstacle grid
        obstacle_grid = self._create_obstacle_grid(obstacles, start, goal)
        
        # Run A* algorithm
        path_grid = self._astar(grid_start, grid_goal, obstacle_grid)
        
        # Convert back to continuous coordinates
        path_points = [self._from_grid(point) for point in path_grid]
        
        return Path(path_points)
        
    def _to_grid(self, point: Point) -> Tuple[int, int]:
        """Convert continuous point to grid coordinates"""
        return (int(point.x / self.grid_resolution),
                int(point.y / self.grid_resolution))
                
    def _from_grid(self, grid_point: Tuple[int, int]) -> Point:
        """Convert grid coordinates to continuous point"""
        return Point(grid_point[0] * self.grid_resolution,
                    grid_point[1] * self.grid_resolution)
                    
    def _create_obstacle_grid(self, obstacles: List[Point], start: Point, goal: Point) -> set:
        """Create grid representation of obstacles"""
        obstacle_grid = set()
        
        # Determine grid bounds
        min_x = min(start.x, goal.x) - 1.0
        max_x = max(start.x, goal.x) + 1.0
        min_y = min(start.y, goal.y) - 1.0
        max_y = max(start.y, goal.y) + 1.0
        
        # Add obstacles to grid
        obstacle_radius = 0.2  # Safety margin around obstacles
        
        for obs in obstacles:
            obs_grid = self._to_grid(obs)
            for dx in range(-2, 3):  # 5x5 grid around obstacle
                for dy in range(-2, 3):
                    if dx*dx + dy*dy <= 4:  # Circle approximation
                        grid_point = (obs_grid[0] + dx, obs_grid[1] + dy)
                        obstacle_grid.add(grid_point)
                        
        return obstacle_grid
        
    def _astar(self, start: Tuple[int, int], goal: Tuple[int, int], 
               obstacle_grid: set) -> List[Tuple[int, int]]:
        """A* algorithm implementation"""
        open_set = [(0, start)]
        came_from = {}
        g_score = {start: 0}
        f_score = {start: self._heuristic(start, goal)}
        
        while open_set:
            _, current = heapq.heappop(open_set)
            
            if current == goal:
                return self._reconstruct_path(came_from, current)
                
            for neighbor in self._get_neighbors(current, obstacle_grid):
                tentative_g_score = g_score[current] + self._distance(current, neighbor)
                
                if neighbor not in g_score or tentative_g_score < g_score[neighbor]:
                    came_from[neighbor] = current
                    g_score[neighbor] = tentative_g_score
                    f_score[neighbor] = tentative_g_score + self._heuristic(neighbor, goal)
                    heapq.heappush(open_set, (f_score[neighbor], neighbor))
                    
        return [start]  # Return start if no path found
        
    def _get_neighbors(self, point: Tuple[int, int], obstacle_grid: set) -> List[Tuple[int, int]]:
        """Get valid neighbors for grid point"""
        neighbors = []
        for dx, dy in [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (-1, 1), (1, -1), (1, 1)]:
            neighbor = (point[0] + dx, point[1] + dy)
            if neighbor not in obstacle_grid:
                neighbors.append(neighbor)
        return neighbors
        
    def _heuristic(self, a: Tuple[int, int], b: Tuple[int, int]) -> float:
        """Heuristic function (Euclidean distance)"""
        return math.sqrt((a[0] - b[0])**2 + (a[1] - b[1])**2)
        
    def _distance(self, a: Tuple[int, int], b: Tuple[int, int]) -> float:
        """Distance between two grid points"""
        dx = abs(a[0] - b[0])
        dy = abs(a[1] - b[1])
        return math.sqrt(dx*dx + dy*dy)
        
    def _reconstruct_path(self, came_from: Dict, current: Tuple[int, int]) -> List[Tuple[int, int]]:
        """Reconstruct path from came_from dictionary"""
        path = [current]
        while current in came_from:
            current = came_from[current]
            path.append(current)
        return path[::-1]


class RRTPlanner(BasePathPlanner):
    """Rapidly-exploring Random Tree (RRT) path planning"""
    
    def __init__(self, max_iterations: int = 1000, step_size: float = 0.1):
        self.max_iterations = max_iterations
        self.step_size = step_size
        
    def plan(self, start: Point, goal: Point, obstacles: List[Point]) -> Path:
        """Plan path using RRT algorithm"""
        nodes = [start]  # Tree nodes
        edges = []       # Tree edges (parent, child)
        
        for _ in range(self.max_iterations):
            # Generate random point
            if np.random.random() < 0.1:  # 10% chance to sample goal
                random_point = goal
            else:
                random_point = Point(
                    np.random.uniform(-5, 5),
                    np.random.uniform(-5, 5)
                )
                
            # Find nearest node
            nearest_node = self._find_nearest_node(random_point, nodes)
            
            # Extend towards random point
            new_node = self._extend_towards(nearest_node, random_point)
            
            # Check if new node is valid (no collision)
            if self._is_valid_point(new_node, obstacles):
                nodes.append(new_node)
                edges.append((nearest_node, new_node))
                
                # Check if we reached the goal
                if new_node.distance_to(goal) < self.step_size * 2:
                    nodes.append(goal)
                    edges.append((new_node, goal))
                    break
                    
        # Extract path from tree
        path = self._extract_path(nodes, edges, start, goal)
        return Path(path)
        
    def _find_nearest_node(self, point: Point, nodes: List[Point]) -> Point:
        """Find nearest node to given point"""
        min_distance = float('inf')
        nearest = nodes[0]
        
        for node in nodes:
            distance = point.distance_to(node)
            if distance < min_distance:
                min_distance = distance
                nearest = node
                
        return nearest
        
    def _extend_towards(self, from_point: Point, to_point: Point) -> Point:
        """Extend tree towards target point"""
        direction = to_point - from_point
        distance = direction.distance_to(Point(0, 0))
        
        if distance < self.step_size:
            return to_point
            
        # Normalize direction and scale by step size
        unit_direction = Point(direction.x / distance, direction.y / distance)
        return from_point + unit_direction * self.step_size
        
    def _is_valid_point(self, point: Point, obstacles: List[Point]) -> bool:
        """Check if point is valid (no collision with obstacles)"""
        for obstacle in obstacles:
            if point.distance_to(obstacle) < 0.3:  # Safety margin
                return False
        return True
        
    def _extract_path(self, nodes: List[Point], edges: List[Tuple[Point, Point]], 
                     start: Point, goal: Point) -> List[Point]:
        """Extract path from RRT tree"""
        # Find path by working backwards from goal
        path = [goal]
        current = goal
        
        while current != start:
            # Find parent of current node
            parent = None
            for parent_node, child_node in edges:
                if child_node == current:
                    parent = parent_node
                    break
                    
            if parent is None:
                break
                
            path.append(parent)
            current = parent
            
        return path[::-1]  # Reverse to get start-to-goal order


class TrajectoryGenerator:
    """Trajectory generation for smooth robot motion"""
    
    def __init__(self, max_velocity: float = 1.0, max_acceleration: float = 2.0):
        self.max_velocity = max_velocity
        self.max_acceleration = max_acceleration
        
    def generate_trapezoidal_trajectory(self, path: Path, total_time: float) -> List[TrajectoryPoint]:
        """Generate trapezoidal velocity profile trajectory"""
        if not path.points or total_time <= 0:
            return []
            
        trajectory = []
        
        # Calculate distances between consecutive points
        segment_lengths = []
        for i in range(len(path.points) - 1):
            length = path.points[i].distance_to(path.points[i + 1])
            segment_lengths.append(length)
            
        total_distance = sum(segment_lengths)
        
        if total_distance == 0:
            # Stationary trajectory
            point = path.points[0]
            trajectory.append(TrajectoryPoint(
                point.x, point.y, 0.0, 0.0, 0.0, 0.0, 0.0
            ))
            return trajectory
            
        # Time allocation for trapezoidal profile
        time_accel = self.max_velocity / self.max_acceleration
        distance_accel = 0.5 * self.max_acceleration * time_accel * time_accel
        
        if total_distance < 2 * distance_accel:
            # Triangle profile (no constant velocity phase)
            time_accel = math.sqrt(total_distance / self.max_acceleration)
            time_total = 2 * time_accel
            time_constant = 0
        else:
            # Trapezoidal profile
            time_constant = (total_distance - 2 * distance_accel) / self.max_velocity
            time_total = time_accel + time_constant + time_accel
            
        # Scale time to match desired total time
        time_scale = total_time / time_total if time_total > 0 else 1.0
        time_accel *= time_scale
        time_constant *= time_scale
        
        # Generate trajectory points
        current_time = 0.0
        current_distance = 0.0
        
        for i, segment_length in enumerate(segment_lengths):
            segment_start = current_distance
            segment_end = current_distance + segment_length
            
            # Determine motion phase for this segment
            t_start = current_time
            t_end = current_time + (segment_length / total_distance) * total_time
            
            # Generate points for this segment
            num_points = max(2, int((t_end - t_start) / 0.02))  # 50 Hz
            for j in range(num_points + 1):
                t = t_start + (j / num_points) * (t_end - t_start)
                s = t_start + (j / num_points) * (t_end - t_start)
                
                # Calculate velocity profile
                if s <= time_accel:
                    # Acceleration phase
                    velocity = self.max_acceleration * s
                    acceleration = self.max_acceleration
                elif s <= time_accel + time_constant:
                    # Constant velocity phase
                    velocity = self.max_velocity
                    acceleration = 0
                else:
                    # Deceleration phase
                    time_left = time_total - s
                    velocity = self.max_acceleration * time_left
                    acceleration = -self.max_acceleration
                    
                # Get position along path
                distance = segment_start + (segment_length * j / num_points)
                point = path.get_point_at_distance(distance)
                direction = path.get_direction_at_distance(distance)
                
                # Calculate velocities in world coordinates
                vx = velocity * math.cos(direction)
                vy = velocity * math.sin(direction)
                
                trajectory.append(TrajectoryPoint(
                    point.x, point.y, direction, vx, vy, 0.0, t
                ))
                
            current_time = t_end
            current_distance = segment_end
            
        return trajectory
        
    def generate_smooth_trajectory(self, waypoints: List[Point], 
                                 total_time: float) -> List[TrajectoryPoint]:
        """Generate smooth trajectory using polynomial interpolation"""
        if len(waypoints) < 2 or total_time <= 0:
            return []
            
        trajectory = []
        
        # Time allocation for each segment
        segment_times = []
        total_path_length = 0
        
        for i in range(len(waypoints) - 1):
            length = waypoints[i].distance_to(waypoints[i + 1])
            total_path_length += length
            segment_times.append(length)
            
        # Normalize times to total time
        total_time_segments = sum(segment_times)
        if total_time_segments > 0:
            segment_times = [t * total_time / total_time_segments for t in segment_times]
            
        current_time = 0.0
        
        for i in range(len(waypoints) - 1):
            start_point = waypoints[i]
            end_point = waypoints[i + 1]
            segment_time = segment_times[i]
            
            # Generate points for this segment using quintic polynomials
            num_points = max(10, int(segment_time / 0.01))  # At least 10 points or 100Hz
            
            for j in range(num_points + 1):
                t = j / num_points * segment_time
                tau = t / segment_time  # Normalized time [0, 1]
                
                # Quintic polynomial for smooth acceleration/deceleration
                # s(tau) = 10*tau^3 - 15*tau^4 + 6*tau^5
                s = 10 * tau**3 - 15 * tau**4 + 6 * tau**5
                ds_dt = (30 * tau**2 - 60 * tau**3 + 30 * tau**4) / segment_time
                d2s_dt2 = (60 * tau - 180 * tau**2 + 120 * tau**3) / (segment_time**2)
                
                # Interpolate position
                x = start_point.x + s * (end_point.x - start_point.x)
                y = start_point.y + s * (end_point.y - start_point.y)
                
                # Calculate direction
                dx_dt = ds_dt * (end_point.x - start_point.x)
                dy_dt = ds_dt * (end_point.y - start_point.y)
                theta = math.atan2(dy_dt, dx_dt)
                
                # Calculate velocities
                vx = dx_dt
                vy = dy_dt
                
                # Calculate accelerations (for potential use)
                ax = d2s_dt2 * (end_point.x - start_point.x)
                ay = d2s_dt2 * (end_point.y - start_point.y)
                
                trajectory.append(TrajectoryPoint(
                    x, y, theta, vx, vy, 0.0, current_time + t
                ))
                
            current_time += segment_time
            
        return trajectory


class PathPlanner:
    """Main path planning interface combining different planners"""
    
    def __init__(self, planner_type: str = "astar", **kwargs):
        if planner_type.lower() == "astar":
            self.planner = AStarPlanner(**kwargs)
        elif planner_type.lower() == "rrt":
            self.planner = RRTPlanner(**kwargs)
        else:
            raise ValueError(f"Unknown planner type: {planner_type}")
            
        self.trajectory_generator = TrajectoryGenerator()
        
    def navigate_to(self, start: Tuple[float, float], goal: Tuple[float, float],
                   obstacles: Optional[List[Tuple[float, float]]] = None,
                   planner_type: Optional[str] = None,
                   total_time: Optional[float] = None) -> List[TrajectoryPoint]:
        """
        Plan and execute navigation to goal
        
        Args:
            start: (x, y) start position
            goal: (x, y) goal position  
            obstacles: List of (x, y) obstacle positions
            planner_type: Override planner type ("astar" or "rrt")
            total_time: Total trajectory time
            
        Returns:
            List of trajectory points
        """
        # Create Point objects
        start_point = Point(start[0], start[1])
        goal_point = Point(goal[0], goal[1])
        
        obstacle_points = []
        if obstacles:
            obstacle_points = [Point(obs[0], obs[1]) for obs in obstacles]
            
        # Select planner
        if planner_type:
            if planner_type.lower() == "astar":
                planner = AStarPlanner()
            elif planner_type.lower() == "rrt":
                planner = RRTPlanner()
            else:
                raise ValueError(f"Unknown planner type: {planner_type}")
        else:
            planner = self.planner
            
        # Plan path
        path = planner.plan(start_point, goal_point, obstacle_points)
        
        # Calculate total time if not provided
        if total_time is None:
            # Assume average speed of 0.5 m/s
            total_time = max(1.0, path.total_length / 0.5)
            
        # Generate trajectory
        trajectory = self.trajectory_generator.generate_trapezoidal_trajectory(
            path, total_time
        )
        
        return trajectory
        
    def plot_path(self, start: Tuple[float, float], goal: Tuple[float, float],
                  obstacles: Optional[List[Tuple[float, float]]] = None,
                  trajectory: Optional[List[TrajectoryPoint]] = None,
                  save_path: Optional[str] = None):
        """Plot path and trajectory"""
        try:
            import matplotlib.pyplot as plt
            
            fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(15, 6))
            
            # Plot 1: Path and obstacles
            if obstacles:
                obs_x = [obs[0] for obs in obstacles]
                obs_y = [obs[1] for obs in obstacles]
                ax1.scatter(obs_x, obs_y, c='red', s=50, marker='s', label='Obstacles')
                
            ax1.scatter(start[0], start[1], c='green', s=100, marker='o', label='Start')
            ax1.scatter(goal[0], goal[1], c='blue', s=100, marker='*', label='Goal')
            
            ax1.set_xlabel('X (m)')
            ax1.set_ylabel('Y (m)')
            ax1.set_title('Path Planning')
            ax1.legend()
            ax1.grid(True)
            ax1.set_aspect('equal')
            
            # Plot 2: Trajectory (if provided)
            if trajectory:
                times = [t.time for t in trajectory]
                x_positions = [t.x for t in trajectory]
                y_positions = [t.y for t in trajectory]
                velocities = [math.sqrt(t.vx**2 + t.vy**2) for t in trajectory]
                
                ax2.plot(times, velocities, 'b-', label='Velocity')
                ax2.set_xlabel('Time (s)')
                ax2.set_ylabel('Velocity (m/s)')
                ax2.set_title('Velocity Profile')
                ax2.grid(True)
                ax2.legend()
                
                # Add position trajectory as inset
                inset = fig.add_axes([0.65, 0.65, 0.25, 0.25])
                inset.plot(x_positions, y_positions, 'r-', linewidth=2)
                inset.scatter(start[0], start[1], c='green', s=50, marker='o')
                inset.scatter(goal[0], goal[1], c='blue', s=50, marker='*')
                inset.set_aspect('equal')
                inset.grid(True)
                inset.set_title('Position Trajectory')
                
            plt.tight_layout()
            
            if save_path:
                plt.savefig(save_path)
                print(f"Path plot saved to {save_path}")
            else:
                plt.show()
                
        except ImportError:
            print("matplotlib not available - cannot plot path")
        except Exception as e:
            print(f"Error plotting path: {e}")


# Example usage and testing
if __name__ == "__main__":
    # Test A* planning
    print("Testing A* path planning...")
    start = Point(0, 0)
    goal = Point(4, 4)
    obstacles = [Point(2, 2), Point(2, 3), Point(3, 2)]
    
    astar = AStarPlanner()
    path = astar.plan(start, goal, obstacles)
    print(f"A* path: {len(path.points)} points, length: {path.total_length:.2f}m")
    
    # Test RRT planning
    print("\nTesting RRT path planning...")
    rrt = RRTPlanner(max_iterations=500)
    rrt_path = rrt.plan(start, goal, obstacles)
    print(f"RRT path: {len(rrt_path.points)} points, length: {rrt_path.total_length:.2f}m")
    
    # Test trajectory generation
    print("\nTesting trajectory generation...")
    planner = PathPlanner("astar")
    trajectory = planner.navigate_to(
        (0, 0), (4, 4),
        obstacles=[(2, 2), (2, 3), (3, 2)],
        total_time=10.0
    )
    print(f"Generated {len(trajectory)} trajectory points")
    
    # Test path plotting
    print("\nTesting path plotting...")
    planner.plot_path((0, 0), (4, 4), 
                     obstacles=[(2, 2), (2, 3), (3, 2)],
                     trajectory=trajectory,
                     save_path="/workspace/real_world/robotics/docs/path_planning_example.png")
