"""
OS Research API Framework

A comprehensive research API for OS experimentation and testing in MultiOS,
providing researchers with powerful tools for conducting OS experiments,
performance analysis, and publication-ready reporting.

Main Components:
- Core Framework: Experiment orchestration and lifecycle management
- Performance Benchmarking: Comprehensive benchmarking and measurement tools
- System Analysis: Behavior analysis and anomaly detection
- OS Instrumentation: System modification and instrumentation capabilities
- Testing & Validation: Automated testing and validation frameworks
- Data Management: Data collection, analysis, and storage
- Visualization & Reporting: Publication-ready visualization and reporting

Key Features:
- Customizable test environments
- Performance benchmarking and measurement APIs
- System behavior analysis tools
- Custom OS modification and instrumentation capabilities
- Automated testing and validation frameworks
- Research data collection and analysis tools
- Publication-ready reporting and visualization
"""

# Import the complete framework implementation
from .research_api import (
    ResearchFramework,
    ExperimentManager,
    BenchmarkSuite,
    SystemAnalyzer,
    OSInstrumentor,
    TestFramework,
    DataCollector,
    ReportGenerator,
    EnvironmentManager,
    create_experiment_framework,
    create_benchmark_suite,
    create_system_analyzer,
    create_os_instrumentor,
    create_test_validator,
    create_data_collector,
    create_publication_visualizer,
    create_report_generator,
    __version__,
    __author__,
    __description__
)

# Public API - All main classes and factory functions
__all__ = [
    # Core Classes
    "ResearchFramework",
    "ExperimentManager",
    "EnvironmentManager",
    
    # Performance Classes
    "BenchmarkSuite",
    
    # Analysis Classes
    "SystemAnalyzer",
    
    # Instrumentation Classes
    "OSInstrumentor",
    
    # Testing Classes
    "TestFramework",
    
    # Data Classes
    "DataCollector",
    
    # Visualization Classes
    "ReportGenerator",
    
    # Factory Functions
    "create_experiment_framework",
    "create_benchmark_suite",
    "create_system_analyzer",
    "create_os_instrumentor",
    "create_test_validator",
    "create_data_collector",
    "create_publication_visualizer",
    "create_report_generator",
    
    # Metadata
    "__version__",
    "__author__",
    "__description__"
]