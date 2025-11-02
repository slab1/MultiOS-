"""
Scientific Computing Curriculum - Educational Materials
====================================================

This module provides a comprehensive curriculum for learning scientific computing:
- Structured tutorials covering all major topics
- Progressive exercises from basic to advanced
- Real-world project examples
- Assessment and evaluation frameworks

Author: Scientific Computing Education Team
"""

import numpy as np
import matplotlib.pyplot as plt
from typing import Dict, List, Tuple, Optional, Callable
from dataclasses import dataclass
from abc import ABC, abstractmethod


@dataclass
class LearningObjective:
    """Learning objective for curriculum modules."""
    topic: str
    description: str
    difficulty_level: str  # 'beginner', 'intermediate', 'advanced'
    prerequisites: List[str]
    estimated_time: str


@dataclass
class Exercise:
    """Educational exercise definition."""
    title: str
    description: str
    difficulty: str
    starter_code: str
    solution: str
    hints: List[str]
    learning_points: List[str]


class CurriculumModule:
    """Base class for curriculum modules."""
    
    def __init__(self, title: str, description: str, objectives: List[LearningObjective]):
        self.title = title
        self.description = description
        self.objectives = objectives
        self.exercises = []
        self.projects = []
    
    def add_exercise(self, exercise: Exercise):
        """Add an exercise to the module."""
        self.exercises.append(exercise)
    
    def add_project(self, project: 'Project'):
        """Add a project to the module."""
        self.projects.append(project)


class Tutorial:
    """Interactive tutorial framework."""
    
    def __init__(self, title: str, objectives: List[str]):
        self.title = title
        self.objectives = objectives
        self.steps = []
        self.current_step = 0
    
    def add_step(self, title: str, content: str, code_example: str = "", 
                exercise: Optional[Exercise] = None):
        """Add a tutorial step."""
        self.steps.append({
            'title': title,
            'content': content,
            'code_example': code_example,
            'exercise': exercise
        })
    
    def run_interactive(self):
        """Run tutorial in interactive mode."""
        print(f"\n=== {self.title} ===")
        print(f"Objectives: {', '.join(self.objectives)}\n")
        
        for i, step in enumerate(self.steps):
            print(f"\nStep {i+1}: {step['title']}")
            print("-" * (len(step['title']) + 8))
            print(step['content'])
            
            if step['code_example']:
                print("\nCode Example:")
                print(step['code_example'])
            
            if step['exercise']:
                print(f"\nExercise: {step['exercise'].title}")
                print(step['exercise'].description)
                
                # Prompt user to try the exercise
                response = input("\nWould you like to try the exercise? (y/n): ")
                if response.lower() == 'y':
                    self._run_exercise(step['exercise'])
    
    def _run_exercise(self, exercise: Exercise):
        """Run an individual exercise."""
        print(f"\nExercise: {exercise.title}")
        print(f"Difficulty: {exercise.difficulty}")
        print(f"Description: {exercise.description}")
        
        print("\nStarter Code:")
        print(exercise.starter_code)
        
        # Simple exercise execution (in a real implementation, this would be more sophisticated)
        print("\nHints:")
        for hint in exercise.hints:
            print(f"- {hint}")
        
        print("\nLearning Points:")
        for point in exercise.learning_points:
            print(f"- {point}")


# Curriculum Modules
class BasicProgrammingModule(CurriculumModule):
    """Introduction to programming for scientific computing."""
    
    def __init__(self):
        super().__init__(
            "Basic Programming for Scientific Computing",
            "Learn fundamental programming concepts essential for scientific computing",
            [
                LearningObjective(
                    "Python Basics",
                    "Understand Python syntax, data types, and control structures",
                    "beginner",
                    [],
                    "2-3 hours"
                ),
                LearningObjective(
                    "NumPy Fundamentals",
                    "Master NumPy arrays and basic operations",
                    "beginner",
                    ["Python Basics"],
                    "3-4 hours"
                ),
                LearningObjective(
                    "Data Visualization",
                    "Create basic plots and visualizations",
                    "beginner",
                    ["NumPy Fundamentals"],
                    "2-3 hours"
                )
            ]
        )
        
        # Add exercises
        self._create_exercises()
    
    def _create_exercises(self):
        """Create exercises for basic programming."""
        
        # Exercise 1: Array Operations
        exercise1 = Exercise(
            "Array Manipulation",
            "Create and manipulate NumPy arrays for scientific data",
            "beginner",
            """
import numpy as np

# Create a 1D array of temperatures in Celsius
temperatures_celsius = np.array([20, 25, 30, 35, 40])

# TODO: Convert to Fahrenheit (F = C * 9/5 + 32)
# Your code here

# TODO: Calculate mean, min, max temperatures
# Your code here

# TODO: Find temperatures above 30°C
# Your code here

print("Temperatures in Celsius:", temperatures_celsius)
print("Temperatures in Fahrenheit:", temperatures_celsius)  # Replace with your conversion
""",
            """
import numpy as np

# Create a 1D array of temperatures in Celsius
temperatures_celsius = np.array([20, 25, 30, 35, 40])

# Convert to Fahrenheit
temperatures_fahrenheit = temperatures_celsius * 9/5 + 32

# Calculate statistics
mean_temp = np.mean(temperatures_celsius)
min_temp = np.min(temperatures_celsius)
max_temp = np.max(temperatures_celsius)

# Find temperatures above 30°C
hot_temperatures = temperatures_celsius[temperatures_celsius > 30]

print("Temperatures in Celsius:", temperatures_celsius)
print("Temperatures in Fahrenheit:", temperatures_fahrenheit)
print(f"Mean temperature: {mean_temp:.1f}°C")
print(f"Min temperature: {min_temp}°C")
print(f"Max temperature: {max_temp}°C")
print(f"Temperatures above 30°C: {hot_temperatures}")
""",
            [
                "Use broadcasting for the temperature conversion",
                "Use NumPy's built-in statistical functions",
                "Use boolean indexing to filter arrays"
            ],
            [
                "Arrays can be used for efficient numerical computations",
                "NumPy provides vectorized operations for better performance",
                "Boolean indexing allows for flexible data filtering"
            ]
        )
        
        self.add_exercise(exercise1)


class NumericalComputingModule(CurriculumModule):
    """Numerical computing fundamentals."""
    
    def __init__(self):
        super().__init__(
            "Numerical Computing Fundamentals",
            "Master core numerical methods and algorithms",
            [
                LearningObjective(
                    "Linear Algebra",
                    "Understand matrix operations and solve linear systems",
                    "intermediate",
                    ["NumPy Fundamentals"],
                    "4-5 hours"
                ),
                LearningObjective(
                    "Optimization",
                    "Learn gradient descent and optimization algorithms",
                    "intermediate",
                    ["Linear Algebra", "Calculus"],
                    "5-6 hours"
                ),
                LearningObjective(
                    "FFT and Signal Processing",
                    "Implement and use FFT for frequency analysis",
                    "intermediate",
                    ["Complex Numbers", "Trigonometry"],
                    "3-4 hours"
                )
            ]
        )
        
        self._create_exercises()
    
    def _create_exercises(self):
        """Create exercises for numerical computing."""
        
        # Exercise 1: System of Equations
        exercise1 = Exercise(
            "Solving Linear Systems",
            "Solve a system of linear equations using matrix methods",
            "intermediate",
            """
import numpy as np
from core.numerical_framework import MatrixOperations

# Define the system Ax = b
A = np.array([[3, 2, -1],
              [2, -2, 4],
              [-1, 0.5, -1]])

b = np.array([10, -5, 9])

# TODO: Solve using your matrix operations
# Your code here

# TODO: Verify the solution by computing Ax
# Your code here

print("Matrix A:")
print(A)
print("Vector b:", b)
""",
            """
import numpy as np
from core.numerical_framework import MatrixOperations

# Define the system Ax = b
A = np.array([[3, 2, -1],
              [2, -2, 4],
              [-1, 0.5, -1]])

b = np.array([10, -5, 9])

# Solve using LU decomposition
solution = MatrixOperations.solve_linear_system(A, b)

# Verify solution
verification = np.dot(A, solution)

print("Matrix A:")
print(A)
print("Vector b:", b)
print("Solution x:", solution)
print("Verification (Ax):", verification)
print("Matches b?", np.allclose(verification, b))
""",
            [
                "Use LU decomposition for solving linear systems",
                "Always verify your solution by computing Ax",
                "Check if your result matches the original vector b"
            ],
            [
                "LU decomposition factorizes a matrix into lower and upper triangular matrices",
                "This makes solving much faster than Gaussian elimination",
                "Numerical stability is important for real-world applications"
            ]
        )
        
        self.add_exercise(exercise1)
        
        # Exercise 2: Optimization
        exercise2 = Exercise(
            "Gradient Descent",
            "Implement gradient descent to find the minimum of a function",
            "intermediate",
            """
import numpy as np
from core.numerical_framework import Optimization

# Define a simple quadratic function: f(x,y) = x² + y² + 2xy - 3x - 4y + 6
def quadratic_function(point):
    x, y = point
    return x**2 + y**2 + 2*x*y - 3*x - 4*y + 6

# Define the gradient: ∇f = [2x + 2y - 3, 2y + 2x - 4]
def quadratic_gradient(point):
    x, y = point
    return np.array([2*x + 2*y - 3, 2*y + 2*x - 4])

# TODO: Run gradient descent optimization
# Your code here

# TODO: Print the optimal point and function value
# Your code here
""",
            """
import numpy as np
from core.numerical_framework import Optimization

# Define a simple quadratic function: f(x,y) = x² + y² + 2xy - 3x - 4y + 6
def quadratic_function(point):
    x, y = point
    return x**2 + y**2 + 2*x*y - 3*x - 4*y + 6

# Define the gradient: ∇f = [2x + 2y - 3, 2y + 2x - 4]
def quadratic_gradient(point):
    x, y = point
    return np.array([2*x + 2*y - 3, 2*y + 2*x - 4])

# Run gradient descent optimization
initial_point = np.array([0.0, 0.0])
optimum, history = Optimization.gradient_descent(
    quadratic_function, quadratic_gradient, initial_point, 
    learning_rate=0.1, max_iterations=1000
)

# Print results
print("Starting point:", initial_point)
print("Optimal point:", optimum)
print("Optimal function value:", quadratic_function(optimum))
print("Converged in", len(history), "iterations")
""",
            [
                "Gradient descent updates parameters along the negative gradient",
                "The learning rate controls step size - too large may oscillate, too small may be slow",
                "Check for convergence by monitoring gradient norm or function value"
            ],
            [
                "Gradient descent is fundamental to machine learning",
                "Understanding the relationship between gradient and function shape is crucial",
                "Different learning rates lead to different convergence behaviors"
            ]
        )
        
        self.add_exercise(exercise2)


class SimulationModule(CurriculumModule):
    """Scientific simulation methods."""
    
    def __init__(self):
        super().__init__(
            "Scientific Simulation Methods",
            "Learn to simulate physical and biological systems",
            [
                LearningObjective(
                    "Physics Simulations",
                    "Simulate mechanical systems, waves, and electromagnetic phenomena",
                    "intermediate",
                    ["Newtonian Mechanics", "Differential Equations"],
                    "6-8 hours"
                ),
                LearningObjective(
                    "Chemical Kinetics",
                    "Model reaction rates and molecular dynamics",
                    "intermediate",
                    ["Chemistry Fundamentals", "Numerical Integration"],
                    "4-5 hours"
                ),
                LearningObjective(
                    "Biological Systems",
                    "Simulate population dynamics and evolution",
                    "intermediate",
                    ["Biology Basics", "Stochastic Processes"],
                    "5-6 hours"
                )
            ]
        )
        
        self._create_exercises()
    
    def _create_exercises(self):
        """Create exercises for simulations."""
        
        # Exercise 1: Pendulum Simulation
        exercise1 = Exercise(
            "Simple Pendulum",
            "Simulate the motion of a pendulum using differential equations",
            "intermediate",
            """
import numpy as np
import matplotlib.pyplot as plt
from core.numerical_framework import DifferentialEquations

# Define the pendulum ODE system:
# dθ/dt = ω
# dω/dt = -g/L * sin(θ)

def pendulum_ode(state, t):
    theta, omega = state
    g = 9.81  # m/s²
    L = 1.0   # m
    
    dtheta_dt = omega
    domega_dt = -(g/L) * np.sin(theta)
    return np.array([dtheta_dt, domega_dt])

# TODO: Set initial conditions (θ₀, ω₀)
# Your code here

# TODO: Solve using RK4 method
# Your code here

# TODO: Plot the results
# Your code here

print("Pendulum simulation completed!")
""",
            """
import numpy as np
import matplotlib.pyplot as plt
from core.numerical_framework import DifferentialEquations

# Define the pendulum ODE system
def pendulum_ode(state, t):
    theta, omega = state
    g = 9.81  # m/s²
    L = 1.0   # m
    
    dtheta_dt = omega
    domega_dt = -(g/L) * np.sin(theta)
    return np.array([dtheta_dt, domega_dt])

# Set initial conditions
initial_theta = np.pi/4  # 45 degrees
initial_omega = 0.0      # No initial angular velocity
initial_state = np.array([initial_theta, initial_omega])

# Solve using RK4 method
t, y = DifferentialEquations.solve_system_ode(
    pendulum_ode, initial_state, 0, 10, 0.01
)

# Plot results
plt.figure(figsize=(12, 4))

plt.subplot(1, 2, 1)
plt.plot(t, y[:, 0])
plt.xlabel('Time (s)')
plt.ylabel('Angle (rad)')
plt.title('Pendulum Angle vs Time')
plt.grid(True)

plt.subplot(1, 2, 2)
plt.plot(t, y[:, 1])
plt.xlabel('Time (s)')
plt.ylabel('Angular Velocity (rad/s)')
plt.title('Angular Velocity vs Time')
plt.grid(True)

plt.tight_layout()
plt.show()

print(f"Simulation completed!")
print(f"Initial angle: {initial_theta:.3f} rad ({np.degrees(initial_theta):.1f}°)")
print(f"Final angle: {y[-1, 0]:.3f} rad ({np.degrees(y[-1, 0]):.1f}°)")
""",
            [
                "The pendulum equation is a second-order ODE converted to a system of first-order ODEs",
                "RK4 provides better accuracy than Euler's method",
                "Small time steps are needed for accuracy with stiff problems"
            ],
            [
                "Many physical systems can be modeled as ODEs",
                "Numerical methods allow us to solve problems without analytical solutions",
                "Visualization helps understand the behavior of dynamical systems"
            ]
        )
        
        self.add_exercise(exercise1)


class MachineLearningModule(CurriculumModule):
    """Machine learning algorithms and applications."""
    
    def __init__(self):
        super().__init__(
            "Machine Learning from Scratch",
            "Implement and understand ML algorithms using mathematical foundations",
            [
                LearningObjective(
                    "Linear Models",
                    "Implement linear and logistic regression from scratch",
                    "intermediate",
                    ["Linear Algebra", "Statistics"],
                    "5-6 hours"
                ),
                LearningObjective(
                    "Classification Algorithms",
                    "Build decision trees and k-nearest neighbors",
                    "intermediate",
                    ["Linear Models"],
                    "4-5 hours"
                ),
                LearningObjective(
                    "Unsupervised Learning",
                    "Understand clustering and dimensionality reduction",
                    "intermediate",
                    ["Linear Models", "Statistics"],
                    "6-7 hours"
                )
            ]
        )
        
        self._create_exercises()
    
    def _create_exercises(self):
        """Create exercises for machine learning."""
        
        # Exercise 1: Linear Regression
        exercise1 = Exercise(
            "Linear Regression from Scratch",
            "Implement linear regression using gradient descent",
            "intermediate",
            """
import numpy as np
from ml_algorithms.from_scratch import LinearRegression
from data_analysis.visualization_tools import DataPreprocessing

# Generate sample data
np.random.seed(42)
n_samples = 100
X = np.random.randn(n_samples, 1)
y = 2 * X.ravel() + 1 + 0.1 * np.random.randn(n_samples)

# TODO: Split data into training and testing sets
# Your code here

# TODO: Standardize features
# Your code here

# TODO: Train linear regression model
# Your code here

# TODO: Make predictions and evaluate
# Your code here

print("Linear regression from scratch completed!")
""",
            """
import numpy as np
from ml_algorithms.from_scratch import LinearRegression
from data_analysis.visualization_tools import DataPreprocessing

# Generate sample data
np.random.seed(42)
n_samples = 100
X = np.random.randn(n_samples, 1)
y = 2 * X.ravel() + 1 + 0.1 * np.random.randn(n_samples)

# Split data
X_train, X_test, y_train, y_test = DataPreprocessing.train_test_split(
    X, y, test_size=0.2, random_state=42
)

# Standardize features
X_train_std, mean, std = DataPreprocessing.standardize(X_train)
X_test_std = (X_test - mean) / std

# Train linear regression model
model = LinearRegression(learning_rate=0.01, max_iterations=1000)
model.fit(X_train_std, y_train)

# Make predictions
y_pred_train = model.predict(X_train_std)
y_pred_test = model.predict(X_test_std)

# Evaluate
train_r2 = model.r_squared(X_train_std, y_train)
test_r2 = model.r_squared(X_test_std, y_test)

print(f"Training R²: {train_r2:.4f}")
print(f"Testing R²: {test_r2:.4f}")
print(f"Model weights: {model.weights[1]:.4f} (slope)")
print(f"Model bias: {model.weights[0]:.4f} (intercept)")
""",
            [
                "Feature standardization helps gradient descent converge faster",
                "Always split data to evaluate generalization",
                "R² measures how well the model explains the variance in the data"
            ],
            [
                "Linear regression finds the best line through the data",
                "Gradient descent iteratively improves the model parameters",
                "Feature scaling is crucial for optimization algorithms"
            ]
        )
        
        self.add_exercise(exercise1)


class HPCModule(CurriculumModule):
    """High-performance computing concepts."""
    
    def __init__(self):
        super().__init__(
            "High-Performance Computing",
            "Learn parallel computing and performance optimization",
            [
                LearningObjective(
                    "Parallel Computing",
                    "Understand parallel processing concepts and implementations",
                    "advanced",
                    ["Python Programming", "Algorithm Analysis"],
                    "8-10 hours"
                ),
                LearningObjective(
                    "Performance Optimization",
                    "Profile and optimize code for better performance",
                    "advanced",
                    ["Parallel Computing"],
                    "6-8 hours"
                ),
                LearningObjective(
                    "Distributed Computing",
                    "Learn distributed computing patterns and architectures",
                    "advanced",
                    ["Parallel Computing"],
                    "10-12 hours"
                )
            ]
        )
        
        self._create_exercises()
    
    def _create_exercises(self):
        """Create exercises for HPC."""
        
        # Exercise 1: Vectorization
        exercise1 = Exercise(
            "Vectorization vs Loops",
            "Compare performance of vectorized vs loop-based computations",
            "advanced",
            """
import numpy as np
import time
from hpc.examples import HPCPatterns

# Generate large arrays for testing
size = 1000000
x = np.random.randn(size)
y = np.random.randn(size)

# TODO: Time loop-based computation
# Your code here

# TODO: Time vectorized computation
# Your code here

# TODO: Compare results
# Your code here

print("Performance comparison completed!")
""",
            """
import numpy as np
import time
from hpc.examples import HPCPatterns

# Generate large arrays
size = 1000000
x = np.random.randn(size)
y = np.random.randn(size)

# Time loop-based computation
start_time = time.time()
result_loop = np.zeros(size)
for i in range(size):
    result_loop[i] = np.sqrt(x[i]**2 + y[i]**2)
loop_time = time.time() - start_time

# Time vectorized computation
start_time = time.time()
result_vectorized = np.sqrt(x**2 + y**2)
vectorized_time = time.time() - start_time

# Compare results
speedup = loop_time / vectorized_time
max_diff = np.max(np.abs(result_loop - result_vectorized))

print(f"Loop time: {loop_time:.4f} seconds")
print(f"Vectorized time: {vectorized_time:.4f} seconds")
print(f"Speedup: {speedup:.2f}x")
print(f"Results match: {max_diff < 1e-10}")
""",
            [
                "Vectorization exploits SIMD instructions in modern CPUs",
                "Loops in Python are slow due to interpreted nature",
                "NumPy operations are implemented in optimized C/Fortran"
            ],
            [
                "Vectorization is key to performance in scientific computing",
                "Understanding when to vectorize vs parallelize is important",
                "Always verify correctness when optimizing performance"
            ]
        )
        
        self.add_exercise(exercise1)


# Project Definitions
class Project:
    """Real-world project for applying scientific computing skills."""
    
    def __init__(self, title: str, description: str, difficulty: str, 
                 skills_required: List[str], estimated_time: str):
        self.title = title
        self.description = description
        self.difficulty = difficulty
        self.skills_required = skills_required
        self.estimated_time = estimated_time
        self.milestones = []
    
    def add_milestone(self, title: str, description: str, deliverables: List[str]):
        """Add a project milestone."""
        self.milestones.append({
            'title': title,
            'description': description,
            'deliverables': deliverables
        })


def create_comprehensive_curriculum() -> List[CurriculumModule]:
    """Create the complete scientific computing curriculum."""
    
    modules = []
    
    # Module 1: Basic Programming
    basic_module = BasicProgrammingModule()
    
    # Module 2: Numerical Computing
    numerical_module = NumericalComputingModule()
    
    # Module 3: Scientific Simulations
    simulation_module = SimulationModule()
    
    # Module 4: Machine Learning
    ml_module = MachineLearningModule()
    
    # Module 5: High-Performance Computing
    hpc_module = HPCModule()
    
    # Module 6: Data Analysis and Visualization
    data_module = CurriculumModule(
        "Data Analysis and Visualization",
        "Learn to analyze and visualize scientific data",
        [
            LearningObjective(
                "Statistical Analysis",
                "Perform descriptive and inferential statistics",
                "intermediate",
                ["Python Basics", "NumPy"],
                "4-5 hours"
            ),
            LearningObjective(
                "Data Visualization",
                "Create effective scientific visualizations",
                "intermediate",
                ["Statistical Analysis"],
                "3-4 hours"
            ),
            LearningObjective(
                "Time Series Analysis",
                "Analyze temporal patterns in data",
                "intermediate",
                ["Statistical Analysis"],
                "4-5 hours"
            )
        ]
    )
    
    # Module 7: Bioinformatics
    bio_module = CurriculumModule(
        "Computational Biology and Bioinformatics",
        "Apply computing to biological problems",
        [
            LearningObjective(
                "Sequence Analysis",
                "Analyze DNA, RNA, and protein sequences",
                "intermediate",
                ["String Processing", "Statistics"],
                "5-6 hours"
            ),
            LearningObjective(
                "Phylogenetics",
                "Construct and analyze evolutionary trees",
                "advanced",
                ["Sequence Analysis", "Graph Theory"],
                "6-7 hours"
            ),
            LearningObjective(
                "Molecular Evolution",
                "Model evolutionary processes",
                "advanced",
                ["Phylogenetics", "Probability"],
                "6-8 hours"
            )
        ]
    )
    
    modules = [
        basic_module,
        numerical_module,
        data_module,
        simulation_module,
        ml_module,
        hpc_module,
        bio_module
    ]
    
    return modules


def create_sample_projects() -> List[Project]:
    """Create sample real-world projects."""
    
    projects = []
    
    # Project 1: Climate Data Analysis
    climate_project = Project(
        "Climate Change Data Analysis",
        "Analyze global temperature trends and create predictive models",
        "intermediate",
        ["Data Analysis", "Statistics", "Visualization"],
        "15-20 hours"
    )
    
    climate_project.add_milestone(
        "Data Collection and Preprocessing",
        "Gather and clean climate datasets",
        ["Download temperature datasets", "Handle missing values", "Standardize data formats"]
    )
    
    climate_project.add_milestone(
        "Exploratory Data Analysis",
        "Discover patterns in climate data",
        ["Calculate descriptive statistics", "Identify trends and anomalies", "Create visualizations"]
    )
    
    climate_project.add_milestone(
        "Modeling and Prediction",
        "Build models to predict future temperatures",
        ["Implement time series models", "Train machine learning models", "Validate predictions"]
    )
    
    projects.append(climate_project)
    
    # Project 2: Drug Discovery Simulation
    drug_project = Project(
        "Molecular Drug Discovery Simulation",
        "Simulate drug-target interactions using computational chemistry",
        "advanced",
        ["Chemistry", "Numerical Computing", "Machine Learning"],
        "25-30 hours"
    )
    
    drug_project.add_milestone(
        "Molecular Modeling",
        "Create and analyze molecular structures",
        ["Generate molecular conformations", "Calculate molecular properties", "Implement force fields"]
    )
    
    drug_project.add_milestone(
        "Binding Affinity Prediction",
        "Predict how strongly drugs bind to targets",
        ["Implement docking algorithms", "Calculate binding energies", "Use machine learning for prediction"]
    )
    
    projects.append(drug_project)
    
    # Project 3: Epidemiology Modeling
    epi_project = Project(
        "Epidemic Spread Modeling",
        "Model the spread of infectious diseases in populations",
        "intermediate",
        ["Differential Equations", "Statistics", "Simulation"],
        "20-25 hours"
    )
    
    epi_project.add_milestone(
        "Disease Transmission Models",
        "Implement SIR and SEIR models",
        ["Code epidemiological models", "Simulate disease spread", "Analyze parameters"]
    )
    
    epi_project.add_milestone(
        "Intervention Strategies",
        "Model the impact of public health interventions",
        ["Simulate vaccination programs", "Model social distancing", "Analyze effectiveness"]
    )
    
    projects.append(epi_project)
    
    # Project 4: Genomic Sequence Analysis
    genome_project = Project(
        "Comparative Genomics Analysis",
        "Compare genomes across different species",
        "advanced",
        ["Bioinformatics", "Statistics", "Machine Learning"],
        "30-35 hours"
    )
    
    genome_project.add_milestone(
        "Sequence Alignment",
        "Align genomic sequences from multiple species",
        ["Implement alignment algorithms", "Handle large genomic datasets", "Visualize alignments"]
    )
    
    genome_project.add_milestone(
        "Phylogenetic Analysis",
        "Construct evolutionary relationships",
        ["Build phylogenetic trees", "Analyze evolutionary distances", "Interpret biological significance"]
    )
    
    projects.append(genome_project)
    
    return projects


def display_curriculum_overview():
    """Display an overview of the complete curriculum."""
    print("Scientific Computing Education - Complete Curriculum")
    print("=" * 60)
    
    modules = create_comprehensive_curriculum()
    
    print(f"\nTotal Modules: {len(modules)}")
    print(f"Estimated Total Time: 80-100 hours")
    
    for i, module in enumerate(modules, 1):
        print(f"\n{i}. {module.title}")
        print(f"   {module.description}")
        print(f"   Learning Objectives: {len(module.objectives)}")
        print(f"   Exercises: {len(module.exercises)}")
        
        for obj in module.objectives:
            print(f"   - {obj.topic} ({obj.difficulty_level}, {obj.estimated_time})")
    
    print("\n" + "="*60)
    print("Real-World Projects")
    print("="*60)
    
    projects = create_sample_projects()
    
    for i, project in enumerate(projects, 1):
        print(f"\n{i}. {project.title}")
        print(f"   {project.description}")
        print(f"   Difficulty: {project.difficulty}")
        print(f"   Time: {project.estimated_time}")
        print(f"   Skills Required: {', '.join(project.skills_required)}")
        print(f"   Milestones: {len(project.milestones)}")
        
        for milestone in project.milestones:
            print(f"   - {milestone['title']}: {milestone['description']}")


def run_sample_tutorial():
    """Run a sample interactive tutorial."""
    tutorial = Tutorial(
        "Introduction to NumPy Arrays",
        [
            "Create NumPy arrays from Python lists",
            "Perform basic array operations",
            "Understand broadcasting and vectorization"
        ]
    )
    
    tutorial.add_step(
        "Creating Arrays",
        "NumPy arrays are similar to Python lists but offer much better performance for numerical operations. Let's start by creating arrays from Python lists.",
        """
import numpy as np

# Create array from Python list
my_list = [1, 2, 3, 4, 5]
my_array = np.array(my_list)
print("Array from list:", my_array)

# Create array with specific data type
float_array = np.array([1, 2, 3], dtype=float)
print("Float array:", float_array)

# Create commonly used arrays
zeros = np.zeros(5)
ones = np.ones(5)
range_array = np.arange(0, 10, 2)
print("Zeros:", zeros)
print("Ones:", ones)
print("Range:", range_array)
""",
        None
    )
    
    tutorial.add_step(
        "Array Operations",
        "NumPy allows you to perform operations on entire arrays at once, which is much faster than using Python loops.",
        """
# Element-wise operations
a = np.array([1, 2, 3])
b = np.array([4, 5, 6])

print("a + b =", a + b)
print("a * b =", a * b)
print("a**2 =", a**2)

# Statistical operations
print("Sum of a:", np.sum(a))
print("Mean of a:", np.mean(a))
print("Standard deviation:", np.std(a))
""",
        None
    )
    
    tutorial.add_step(
        "Broadcasting",
        "Broadcasting allows NumPy to automatically expand arrays to compatible shapes for operations.",
        """
# Broadcasting example
a = np.array([[1, 2, 3], [4, 5, 6]])
b = np.array([10, 20, 30])

print("Matrix a:")
print(a)
print("\nVector b:", b)
print("\na + b (broadcasting):")
print(a + b)
""",
        None
    )
    
    # Run the tutorial
    tutorial.run_interactive()


if __name__ == "__main__":
    # Display curriculum overview
    display_curriculum_overview()
    
    print("\n" + "="*60)
    print("Sample Interactive Tutorial")
    print("="*60)
    
    # Run sample tutorial
    run_sample_tutorial()
    
    print("\n" + "="*60)
    print("Curriculum Complete!")
    print("="*60)
    print("\nTo get started:")
    print("1. Begin with the Basic Programming module")
    print("2. Progress through modules in order")
    print("3. Complete exercises as you learn")
    print("4. Apply knowledge through real-world projects")
    print("5. Use interactive tutorials for hands-on learning")