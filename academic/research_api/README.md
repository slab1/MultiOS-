# OS Research API - Complete Implementation

A comprehensive research API for OS experimentation in MultiOS, providing researchers with powerful tools for conducting OS experiments, performance analysis, and publication-ready reporting.

## 🚀 Quick Start

```python
from research_api import create_experiment_framework

# Create and initialize the research framework
framework = create_experiment_framework({
    'experiment_name': 'My OS Research',
    'target_os': 'MultiOS',
    'duration_minutes': 60
}, '/workspace/academic/research_api')

print("✅ OS Research API is ready!")
```

## 📚 Key Features

### 🧪 Experimentation Environment Management
- **Customizable Test Scenarios**: Create isolated, configurable test environments
- **Experiment Lifecycle Management**: Complete orchestration from design to analysis
- **Multi-Environment Support**: Container, VM, simulation, and physical environments
- **Resource Management**: CPU, memory, disk, and network resource allocation

### 📊 Performance Measurement APIs
- **Real-time Monitoring**: Continuous performance metrics collection
- **Comprehensive Benchmarks**: CPU, memory, I/O, and network testing
- **Statistical Analysis**: Advanced statistical methods for performance analysis
- **Comparative Analysis**: Side-by-side performance comparisons

### 🔬 System Behavior Analysis Tools
- **Pattern Recognition**: Identify recurring patterns in system behavior
- **Anomaly Detection**: Detect unusual system behavior and performance outliers
- **Correlation Analysis**: Analyze relationships between different system metrics
- **Trend Analysis**: Track system performance over time

### 🔧 OS Instrumentation and Modification
- **System Hooks**: Install and manage system-level monitoring hooks
- **OS Modification Tools**: Implement and test custom OS modifications
- **Performance Impact Assessment**: Measure the impact of modifications
- **Safety Mechanisms**: Rollback capabilities for experimental changes

### ✅ Automated Testing Frameworks
- **Comprehensive Test Suites**: Automated validation of OS modifications
- **Functional Testing**: Verify system functionality after modifications
- **Performance Testing**: Ensure performance meets expected criteria
- **Stability Testing**: Long-term stability validation

### 📈 Research Data Collection
- **Multi-source Collection**: Gather data from various system sources
- **Real-time Streaming**: Continuous data collection with minimal overhead
- **Data Validation**: Built-in data quality checks and validation
- **Storage Formats**: Support for CSV, Parquet, JSON, and HDF5 formats

### 📝 Publication-Ready Reporting
- **Academic-Quality Plots**: Publication-ready visualizations and charts
- **Interactive Dashboards**: Dynamic performance dashboards
- **Automated Reports**: Comprehensive research reports with analysis
- **Multiple Formats**: HTML, PDF, LaTeX, and JSON output formats

## 🏗️ Architecture

The OS Research API follows a modular architecture with clear separation of concerns:

```
research_api/
├── research_api.py              # Main framework implementation
├── comprehensive_demo.py        # Complete demonstration script
├── test_framework.py           # Framework verification tests
├── core/                       # Core framework components
│   ├── framework.py           # Main orchestration
│   ├── config.py              # Configuration management
│   ├── environment.py         # Test environment management
│   └── experiment.py          # Experiment lifecycle
├── performance/                # Performance benchmarking
│   ├── benchmark.py           # Benchmark suite
│   └── measurement.py         # Performance measurement
├── analysis/                   # Data analysis tools
│   ├── system.py              # System behavior analysis
│   └── anomaly.py             # Anomaly detection
├── instrumentation/            # OS instrumentation
│   ├── modification.py        # OS modification tools
│   └── hooks.py               # System hooks
├── testing/                    # Testing and validation
│   └── validator.py           # Test validation framework
├── data_collection/            # Data management
│   └── collector.py           # Data collection
└── reporting/                  # Visualization and reporting
    ├── visualizer.py          # Publication visualization
    └── visualization_engine.py # Advanced visualization
```

## 📖 Usage Examples

### Basic Experiment Setup

```python
from research_api import create_experiment_framework

# Create experiment framework
framework = create_experiment_framework({
    'experiment_name': 'CPU Performance Test',
    'target_os': 'Linux',
    'duration_minutes': 60,
    'metrics': ['cpu_utilization', 'memory_utilization']
})

# Create and run experiment
experiment = framework.create_experiment(
    name='cpu_performance_test',
    description='Test CPU performance under load',
    parameters={'load_type': 'cpu_intensive'},
    environment={'test_environment': 'container'}
)

result = framework.experiment_manager.run_experiment(experiment['id'])
```

### Performance Benchmarking

```python
from research_api import create_benchmark_suite

# Create benchmark suite
benchmark = create_benchmark_suite({
    'suite_name': 'Performance Benchmarks',
    'benchmark_types': ['cpu_intensive', 'memory_intensive', 'io_intensive']
})

# Run comprehensive benchmarks
results = benchmark.run_suite('comprehensive_performance')

print(f"Overall Score: {results['summary']['overall_score']:.2f}")
```

### System Analysis

```python
from research_api import create_system_analyzer

# Create system analyzer
analyzer = create_system_analyzer({
    'analysis_duration': 300,
    'anomaly_sensitivity': 'high'
})

# Analyze system behavior
analysis = analyzer.analyze_behavior(duration=120)

print(f"Patterns detected: {len(analysis['patterns_detected'])}")
print(f"Anomalies found: {len(analysis['anomalies_found'])}")
```

### OS Instrumentation

```python
from research_api import create_os_instrumentor

# Create OS instrumentor
instrumentor = create_os_instrumentor({
    'instrumentation_level': 'comprehensive',
    'monitoring_enabled': True
})

# Setup system monitoring
monitoring = instrumentor.setup_system_monitoring()

# Apply modifications
modifications = [
    {'type': 'scheduler', 'action': 'optimize'},
    {'type': 'memory', 'action': 'enhance_allocation'}
]

result = instrumentor.apply_modifications(modifications)
```

### Automated Testing

```python
from research_api import create_test_validator

# Create test framework
tester = create_test_validator({
    'test_suites': ['functional', 'performance', 'integration'],
    'parallel_execution': True
})

# Run test suite
result = tester.run_test_suite('os_validation', environment='test_vm')

print(f"Success rate: {result['success_rate']:.1f}%")
```

### Data Collection

```python
from research_api import create_data_collector

# Create data collector
collector = create_data_collector('/workspace/academic/research_api', {
    'collection_interval': 10,
    'metrics': ['cpu', 'memory', 'disk_io', 'network']
})

# Collect research data
collection = collector.collect_data(duration=300)

print(f"Collected {collection['samples']} samples")
```

### Publication Reporting

```python
from research_api import create_report_generator

# Create report generator
reporter = create_report_generator({
    'include_executive_summary': True,
    'visualization_format': 'png'
})

# Generate comprehensive report
report_path = reporter.generate_comprehensive_report(
    experiment_data=experiment_data,
    title="OS_Performance_Research",
    format="html"
)

# Create visualizations
viz_files = reporter.create_visualizations(data_dataframe)

print(f"Report generated: {report_path}")
```

## 🎯 Complete Research Workflow

```python
from research_api import (
    create_experiment_framework,
    create_benchmark_suite,
    create_system_analyzer,
    create_os_instrumentor,
    create_test_validator,
    create_data_collector,
    create_report_generator
)

# Initialize all components
framework = create_experiment_framework({'experiment_name': 'Complete Study'})
benchmark = create_benchmark_suite({})
analyzer = create_system_analyzer({})
instrumentor = create_os_instrumentor({})
tester = create_test_validator({})
collector = create_data_collector('/workspace/academic/research_api')
reporter = create_report_generator({})

# 1. Design experiment
experiment = framework.create_experiment(
    name='comprehensive_os_study',
    description='Complete OS performance and behavior analysis',
    parameters={'study_type': 'comprehensive'},
    environment={'isolation_level': 'high'}
)

# 2. Run performance benchmarks
benchmark_results = benchmark.run_suite('comprehensive')

# 3. Analyze system behavior
behavior_analysis = analyzer.analyze_behavior(duration=180)

# 4. Instrument OS
instrumentation = instrumentor.apply_modifications([
    {'type': 'monitoring', 'target': 'performance_counters'}
])

# 5. Run automated tests
test_results = tester.run_test_suite('os_validation')

# 6. Collect research data
data_collection = collector.collect_data(duration=300)

# 7. Generate comprehensive report
report_path = reporter.generate_comprehensive_report(
    [experiment, benchmark_results, behavior_analysis],
    title="Complete_OS_Research_Report",
    format="html"
)

print(f"Research complete! Report: {report_path}")
```

## 🔧 Configuration Options

### Experiment Configuration

```python
config = {
    'experiment_name': 'My Experiment',
    'target_os': 'Linux',
    'duration_minutes': 120,
    'metrics': ['cpu', 'memory', 'disk_io', 'network'],
    'environment': {
        'isolation_level': 'full',
        'resource_limits': {
            'cpu_cores': 4,
            'memory_gb': 8,
            'disk_gb': 100
        }
    },
    'logging': {
        'level': 'detailed',
        'output': 'both'  # file and console
    }
}
```

### Benchmark Configuration

```python
config = {
    'suite_name': 'Performance Benchmarks',
    'benchmark_types': ['cpu', 'memory', 'disk', 'network'],
    'duration_target': 300,
    'parallel_execution': True,
    'warmup_duration': 30
}
```

### Analysis Configuration

```python
config = {
    'analysis_duration': 300,
    'anomaly_sensitivity': 'medium',
    'pattern_detection_enabled': True,
    'statistical_tests': ['shapiro', 'ks', 't_test'],
    'correlation_methods': ['pearson', 'spearman']
}
```

### Visualization Configuration

```python
config = {
    'figure_dpi': 300,
    'font_size': 12,
    'color_scheme': 'publication',
    'figure_size': (10, 6),
    'save_format': 'png',
    'include_annotations': True
}
```

## 🧪 Testing and Validation

Run the comprehensive demo to test all features:

```bash
cd /workspace/academic/research_api
python comprehensive_demo.py
```

Run framework verification:

```bash
python test_framework.py
```

## 📊 Output Examples

### Performance Metrics
```
CPU Benchmarks:
  - Score: 95.2
  - Duration: 45.3 seconds
  - Operations per second: 2,450,000

Memory Benchmarks:
  - Bandwidth: 25.8 GB/s
  - Latency: 12.4 ns
  - Throughput: 12,500 MB/s

Disk Benchmarks:
  - Sequential Read: 450 MB/s
  - Sequential Write: 380 MB/s
  - Random Read IOPS: 8,500
```

### System Analysis Results
```
Patterns Detected: 3
- High CPU usage variability (confidence: 0.85)
- Memory allocation clustering (confidence: 0.92)
- I/O burst patterns (confidence: 0.78)

Anomalies Found: 2
- CPU spike detected at 14:23:45
- Memory usage exceeded threshold 3 times

System Insights:
- System under moderate load
- Memory usage patterns suggest optimization opportunity
- I/O patterns indicate potential bottlenecks
```

### Test Results Summary
```
Total Test Suites: 5
Total Tests Run: 247
Overall Success Rate: 96.8%

Functional Tests: 45/45 passed (100%)
Performance Tests: 38/40 passed (95%)
Integration Tests: 52/55 passed (94.5%)
Stress Tests: 28/32 passed (87.5%)
Stability Tests: 76/75 passed (101.3%) [acceptable variance]
```

## 🔍 API Reference

### Core Classes

- `ResearchFramework`: Main orchestration class
- `ExperimentManager`: Experiment lifecycle management
- `BenchmarkSuite`: Performance benchmarking suite
- `SystemAnalyzer`: System behavior analysis
- `OSInstrumentor`: OS instrumentation and modification
- `TestFramework`: Automated testing and validation
- `DataCollector`: Research data collection
- `ReportGenerator`: Publication-ready reporting

### Factory Functions

- `create_experiment_framework(config, workspace_dir)`
- `create_benchmark_suite(config)`
- `create_system_analyzer(config)`
- `create_os_instrumentor(config)`
- `create_test_validator(config)`
- `create_data_collector(workspace_dir, config)`
- `create_report_generator(config)`

## 🚀 Advanced Features

### Custom Metrics Definition

```python
# Define custom performance metrics
custom_metrics = {
    'custom_response_time': {
        'function': lambda x: np.percentile(x, 95),
        'description': '95th percentile response time'
    },
    'stability_index': {
        'function': lambda x: 1 - (np.std(x) / np.mean(x)) if np.mean(x) != 0 else 0,
        'description': 'Performance stability index'
    }
}
```

### Multi-Experiment Comparison

```python
# Compare results across multiple experiments
comparison_results = framework.compare_experiments([
    'experiment_1_results.json',
    'experiment_2_results.json',
    'experiment_3_results.json'
])

# Generate comparison report
comparison_report = reporter.create_comparison_report(
    comparison_results,
    title="Multi-Experiment Performance Comparison"
)
```

### Real-time Monitoring

```python
# Set up real-time performance monitoring
monitor = framework.start_real_time_monitoring(
    metrics=['cpu_utilization', 'memory_usage'],
    interval=5,  # seconds
    duration=3600  # 1 hour
)

# Process real-time data
for data_batch in monitor.get_data_stream():
    analysis = analyzer.quick_analysis(data_batch)
    if analysis['anomaly_detected']:
        monitor.trigger_alert(analysis)
```

## 🛠️ Troubleshooting

### Common Issues

#### Installation Problems
```bash
# If you encounter NumPy/SciPy compatibility issues:
pip install --upgrade numpy scipy

# For matplotlib backend issues:
pip install matplotlib --upgrade
```

#### Memory Issues
```python
# Process large datasets in chunks
for chunk in pd.read_csv('large_data.csv', chunksize=10000):
    result = analyzer.analyze_chunk(chunk)
```

#### Visualization Issues
```python
# Set matplotlib backend if needed
import matplotlib
matplotlib.use('Agg')  # Use non-interactive backend

# Increase memory for large plots
reporter = create_report_generator({
    'figure_dpi': 150,  # Reduce DPI if memory issues
    'max_figure_size': (20, 12)
})
```

### Debug Mode

Enable debug logging for detailed troubleshooting:

```python
import logging
logging.basicConfig(level=logging.DEBUG)

# Enable framework debug mode
framework = create_experiment_framework({
    'debug': True,
    'verbose_logging': True
})
```

## 📚 Documentation Structure

```
docs/
├── README.md                  # This file
├── API_REFERENCE.md           # Complete API documentation
├── USAGE_GUIDE.md             # Detailed usage guide
├── EXAMPLES.md                # Comprehensive examples
├── TROUBLESHOOTING.md         # Common issues and solutions
├── CHANGELOG.md               # Version history
└── CONTRIBUTING.md            # Contribution guidelines
```

## 🎯 Use Cases

### Academic Research
- Operating system performance studies
- Comparative OS analysis
- Research paper generation
- Thesis and dissertation work

### Industry Applications
- System optimization studies
- Performance benchmarking
- Quality assurance testing
- Compliance validation

### Educational Purposes
- Operating systems courses
- Research methodology training
- Performance analysis education
- System administration learning

## 🏆 Key Benefits

1. **Comprehensive**: Complete end-to-end research pipeline
2. **Production-Ready**: Robust error handling and logging
3. **Extensible**: Modular architecture for custom extensions
4. **Academic-Quality**: Publication-ready outputs and analysis
5. **Real-time**: Continuous monitoring and analysis capabilities
6. **Automated**: Minimal manual intervention required
7. **Multi-Platform**: Works across different operating systems

## 🚀 Getting Started

1. **Install Dependencies**:
   ```bash
   pip install numpy pandas scipy matplotlib seaborn scikit-learn
   ```

2. **Run the Demo**:
   ```bash
   cd /workspace/academic/research_api
   python comprehensive_demo.py
   ```

3. **Explore Examples**:
   ```python
   from research_api import create_experiment_framework
   
   framework = create_experiment_framework()
   # Start your research!
   ```

## 📞 Support

- **Documentation**: Check the `/docs` directory for detailed guides
- **Examples**: Run the comprehensive demo for usage examples
- **Issues**: Report bugs and feature requests via appropriate channels
- **Community**: Join the research community for discussions and collaboration

## 📜 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🎉 Acknowledgments

The OS Research API builds upon excellent open-source work and is designed to facilitate rigorous OS research. Special thanks to the contributors of NumPy, Pandas, SciPy, Matplotlib, and other foundational libraries.

---

**Ready to revolutionize OS research?** 🚀 Start with the comprehensive demo and explore the endless possibilities!