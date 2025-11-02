# Scientific Computing Workloads for Education

A comprehensive educational framework for learning scientific computing through hands-on implementation of algorithms and real-world applications.

## Overview

This project provides a complete learning environment for scientific computing education, featuring:

- **Core Numerical Computing Framework** - Linear algebra, optimization, FFT, differential equations
- **Scientific Simulation Engines** - Physics, chemistry, and biology simulations
- **Data Analysis and Visualization Tools** - Statistical analysis, time series, signal processing
- **High-Performance Computing Examples** - Parallel computing, optimization techniques
- **Machine Learning from Scratch** - Educational implementations of ML algorithms
- **Computational Biology & Bioinformatics** - Sequence analysis, phylogenetics, molecular evolution
- **Comprehensive Curriculum** - Structured tutorials, exercises, and projects

## Architecture

```
scientific_computing/
├── core/                          # Core numerical computing framework
│   └── numerical_framework.py     # Linear algebra, optimization, FFT, ODEs
├── simulations/                   # Scientific simulation engines
│   └── scientific_engines.py      # Physics, chemistry, biology simulations
├── data_analysis/                 # Data analysis and visualization
│   └── visualization_tools.py     # Statistical analysis, plotting tools
├── hpc/                          # High-performance computing
│   └── examples.py               # Parallel computing, optimization
├── ml_algorithms/                # Machine learning from scratch
│   └── from_scratch.py          # Educational ML implementations
├── bioinformatics/               # Computational biology tools
│   └── tools.py                 # Sequence analysis, phylogenetics
├── curriculum/                   # Educational materials
│   └── curriculum_framework.py   # Tutorials, exercises, projects
└── main.py                       # Main integration script
```

## Quick Start

### Basic Usage

Run a quick demonstration:
```bash
python main.py
```

Run specific component demonstrations:
```bash
python main.py --demo numerical          # Numerical computing
python main.py --demo simulations        # Scientific simulations
python main.py --demo machine_learning   # ML algorithms
python main.py --demo bioinformatics     # Bioinformatics tools
```

List all available components:
```bash
python main.py --list-components
```

### Educational Examples

Each module contains educational examples that can be run directly:

```python
# Numerical computing
from core.numerical_framework import *
educational_examples()

# Scientific simulations
from simulations.scientific_engines import *
run_physics_simulations()
run_chemistry_simulations()
run_biology_simulations()

# Data analysis
from data_analysis.visualization_tools import *
demo_data_analysis()

# Machine learning
from ml_algorithms.from_scratch import *
demo_ml_algorithms()

# Bioinformatics
from bioinformatics.tools import *
demo_bioinformatics_tools()
```

## Core Components

### 1. Numerical Computing Framework (`core/numerical_framework.py`)

**Features:**
- Matrix operations with LU decomposition
- Numerical optimization (gradient descent, Newton's method, simulated annealing)
- Fast Fourier Transform (FFT) implementations
- Ordinary differential equation solvers (Euler, Runge-Kutta 4)
- Statistical computations and Monte Carlo integration

**Example Usage:**
```python
from core.numerical_framework import MatrixOperations, Optimization, FFTOperations

# Solve linear system
A = np.array([[3, 1], [1, 2]])
b = np.array([9, 8])
solution = MatrixOperations.solve_linear_system(A, b)

# Optimize function
def func(x): return x[0]**2 + x[1]**2
def grad(x): return np.array([2*x[0], 2*x[1]])
optimum, history = Optimization.gradient_descent(func, grad, np.array([1.0, 1.0]))
```

### 2. Scientific Simulation Engines (`simulations/scientific_engines.py`)

**Physics Simulations:**
- Newtonian mechanics with constant force
- Harmonic oscillators (damped/undamped)
- Electromagnetic wave propagation
- Thermodynamics of ideal gases

**Chemistry Simulations:**
- First and second-order reaction kinetics
- Simple molecular dynamics (Lennard-Jones potential)

**Biology Simulations:**
- Population dynamics (Lotka-Volterra)
- Predator-prey models
- Genetic algorithms
- Conway's Game of Life

**Example Usage:**
```python
from simulations.scientific_engines import PhysicsSimulation, ChemistrySimulation, BiologySimulation

# Physics: Harmonic oscillator
physics = PhysicsSimulation()
result = physics.harmonic_oscillator(mass=1.0, spring_constant=10.0)

# Chemistry: First-order kinetics
chemistry = ChemistrySimulation()
result = chemistry.reaction_kinetics_1st_order(rate_constant=0.1, initial_concentration=1.0)

# Biology: Predator-prey model
biology = BiologySimulation()
result = biology.predator_prey_model(prey_growth=1.0, predation_rate=0.1)
```

### 3. Data Analysis and Visualization (`data_analysis/visualization_tools.py`)

**Statistical Analysis:**
- Descriptive statistics with outlier detection
- Correlation analysis (Pearson, Spearman, Kendall)
- Hypothesis testing (t-test, chi-square, ANOVA)
- Time series analysis and forecasting

**Signal Processing:**
- Low/high-pass filters
- Peak detection
- Power spectral density estimation

**Visualization Tools:**
- Distribution plots (histograms, box plots, Q-Q plots)
- Time series plots
- Correlation matrices
- 3D scatter plots

**Example Usage:**
```python
from data_analysis.visualization_tools import DataAnalysis, TimeSeriesAnalysis, SignalProcessing

# Statistical analysis
stats = DataAnalysis.descriptive_statistics(data)
correlation = DataAnalysis.correlation_analysis(x, y)

# Time series analysis
trend = TimeSeriesAnalysis.trend_analysis(time_series_data)
seasonal = TimeSeriesAnalysis.seasonal_decomposition(data, period=12)

# Signal processing
filtered = SignalProcessing.filter_lowpass(noisy_signal, cutoff_freq=10, sampling_rate=100)
peaks = SignalProcessing.detect_peaks(signal_data)
```

### 4. High-Performance Computing (`hpc/examples.py`)

**Parallel Computing:**
- Matrix multiplication (sequential vs parallel)
- Reduction operations (sum, max, min)
- Domain decomposition techniques

**Performance Optimization:**
- Vectorization vs loops
- Cache-aware algorithms
- Memory layout optimization

**Distributed Computing:**
- Master-worker patterns
- Map-reduce implementations
- Fault-tolerant computation

**Example Usage:**
```python
from hpc.examples import HPCPatterns, ParallelNumericalMethods, PerformanceBenchmarking

# Vectorization benchmark
hpc = HPCPatterns()
result = hpc.vectorized_vs_loop(size=1000000)

# Parallel matrix multiplication
C = hpc.parallel_matrix_multiplication(A, B, use_multiprocessing=True)

# Performance benchmarking
benchmark = PerformanceBenchmarking()
perf_result = benchmark.benchmark_function(np.linalg.norm, large_array)
```

### 5. Machine Learning from Scratch (`ml_algorithms/from_scratch.py`)

**Supervised Learning:**
- Linear Regression with gradient descent
- Logistic Regression for classification
- Decision Tree Classifier
- K-Nearest Neighbors

**Unsupervised Learning:**
- K-Means Clustering
- Principal Component Analysis (PCA)

**Evaluation Metrics:**
- Accuracy, precision, recall, F1-score
- Cross-validation
- Confusion matrices

**Example Usage:**
```python
from ml_algorithms.from_scratch import LinearRegression, LogisticRegression, KMeansClustering

# Linear regression
model = LinearRegression(learning_rate=0.01, max_iterations=1000)
model.fit(X_train, y_train)
predictions = model.predict(X_test)

# Logistic regression
clf = LogisticRegression(learning_rate=0.1, max_iterations=1000)
clf.fit(X_train, y_train)
predictions = clf.predict(X_test)
probabilities = clf.predict_proba(X_test)

# K-means clustering
kmeans = KMeansClustering(k=3, random_state=42)
labels = kmeans.fit_predict(data)
```

### 6. Computational Biology & Bioinformatics (`bioinformatics/tools.py`)

**Sequence Analysis:**
- DNA/RNA sequence manipulation
- GC content calculation
- Reverse complement generation
- Open Reading Frame (ORF) detection
- Codon translation

**Sequence Alignment:**
- Global alignment (Needleman-Wunsch)
- Local alignment (Smith-Waterman)
- Scoring matrices and gap penalties

**Phylogenetics:**
- Distance matrix calculation
- Neighbor Joining tree construction
- Evolutionary distance models (Jukes-Cantor, Kimura)

**Molecular Evolution:**
- Substitution rate models
- Molecular clock simulation
- Sequence evolution with mutations

**Genome Analysis:**
- K-mer frequency analysis
- Repeat identification
- GC skew analysis
- Origin of replication detection

**Example Usage:**
```python
from bioinformatics.tools import SequenceAnalysis, SequenceAlignment, Phylogenetics, MolecularEvolution

# Sequence analysis
gc_content = SequenceAnalysis.gc_content("ATGCGTACGT")
protein = SequenceAnalysis.translate("AUGGCCAUGGCGCCCAGAACUGAGAUCAAUAGUACCCGUAUUAACGGGUGA")

# Sequence alignment
alignment = SequenceAlignment.needleman_wunsch(seq1, seq2)
print(f"Alignment score: {alignment['score']}")

# Phylogenetics
distance_matrix = Phylogenetics.calculate_distance_matrix(sequences)
tree = Phylogenetics.neighbor_joining(distance_matrix, labels)

# Molecular evolution
distance = MolecularEvolution.jukes_cantor_distance(seq1, seq2)
evolved_sequences = MolecularEvolution.generate_sequences_with_substitution(
    n_sequences=5, sequence_length=100, substitution_rate=0.01
)
```

### 7. Educational Curriculum (`curriculum/curriculum_framework.py`)

**Structured Learning Modules:**
- Basic Programming for Scientific Computing
- Numerical Computing Fundamentals
- Scientific Simulation Methods
- Machine Learning from Scratch
- High-Performance Computing
- Data Analysis and Visualization
- Computational Biology and Bioinformatics

**Features:**
- Learning objectives for each module
- Progressive exercises from beginner to advanced
- Interactive tutorials
- Real-world projects
- Assessment frameworks

**Example Usage:**
```python
from curriculum.curriculum_framework import create_comprehensive_curriculum, create_sample_projects

# Get complete curriculum
modules = create_comprehensive_curriculum()
for module in modules:
    print(f"Module: {module.title}")
    print(f"Exercises: {len(module.exercises)}")

# Get sample projects
projects = create_sample_projects()
for project in projects:
    print(f"Project: {project.title}")
    print(f"Estimated time: {project.estimated_time}")

# Run interactive tutorial
run_sample_tutorial()
```

## Educational Philosophy

This framework follows several key educational principles:

1. **Implementation from First Principles**: Algorithms are implemented from scratch to deepen understanding
2. **Progressive Difficulty**: Exercises progress from basic to advanced concepts
3. **Real-World Applications**: Examples based on actual scientific computing problems
4. **Hands-On Learning**: Interactive tutorials and practical exercises
5. **Cross-Disciplinary**: Covers physics, chemistry, biology, and computer science

## Performance Considerations

The framework demonstrates several performance optimization techniques:

- **Vectorization**: NumPy operations for better performance than Python loops
- **Parallel Computing**: Multi-processing for computationally intensive tasks
- **Memory Optimization**: Cache-aware algorithms and memory layout considerations
- **Numerical Stability**: Proper handling of numerical edge cases

## Use Cases

### For Students:
- Learn scientific computing concepts through implementation
- Understand mathematical foundations of algorithms
- Practice with real-world datasets and problems
- Build portfolio of scientific computing projects

### For Educators:
- Ready-to-use curriculum materials
- Progressive exercises and assessments
- Customizable content for different courses
- Integration with existing curricula

### For Researchers:
- Reference implementations of algorithms
- Educational tools for explaining methods
- Foundation for more advanced implementations
- Learning resource for new computational methods

## Dependencies

The framework requires:
- Python 3.7+
- NumPy (for numerical computations)
- Matplotlib (for visualization)
- SciPy (for statistical functions)

Optional dependencies for enhanced functionality:
- psutil (for memory profiling)
- Jupyter notebooks (for interactive tutorials)

## Installation

1. Clone or download the repository
2. Install dependencies:
```bash
pip install numpy matplotlib scipy psutil
```
3. Run the main script to test the installation:
```bash
python main.py
```

## Testing

Run the comprehensive test suite:
```bash
python main.py --test
```

Run specific component tests:
```bash
python main.py --demo numerical
python main.py --demo machine_learning
```

## Extending the Framework

### Adding New Algorithms

1. Implement the algorithm in the appropriate module
2. Add educational examples and documentation
3. Create exercises for the curriculum
4. Update the main integration script

### Adding New Domains

1. Create a new module in the appropriate directory
2. Implement domain-specific algorithms
3. Add to the curriculum framework
4. Create real-world project examples

### Customizing Curriculum

1. Modify `curriculum/curriculum_framework.py`
2. Add new learning objectives
3. Create domain-specific exercises
4. Design assessment criteria

## Contributing

Contributions are welcome! Areas for improvement:

- Additional algorithms and methods
- More real-world datasets
- Enhanced visualization capabilities
- Performance optimizations
- Additional domain applications
- Educational content improvements

## License

This educational framework is provided for learning purposes. Feel free to use, modify, and distribute for educational and research purposes.

## Acknowledgments

This framework was developed for educational purposes to provide hands-on experience with scientific computing concepts and algorithms. It draws inspiration from various scientific computing libraries and educational resources while implementing algorithms from first principles for better understanding.

## Support

For questions about using the framework:
1. Check the examples and documentation in each module
2. Run the main script with different demo options
3. Examine the curriculum framework for structured learning paths
4. Modify and experiment with the code examples

---

**Happy Computing!** 🚀

This framework provides a comprehensive introduction to scientific computing through hands-on implementation and real-world applications. Start with the basics and progress through the curriculum to build strong foundations in computational science.