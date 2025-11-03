"""
Examples Package for OS Research API

This package contains comprehensive examples demonstrating the usage
of the OS Research API for various types of OS experiments and analysis tasks.

Examples included:
- Basic experiment setup and configuration
- Data collection, analysis, and reporting
- Publication-quality visualization
- Comprehensive report generation
- OS instrumentation workflow
- Complete end-to-end research pipeline

Usage:
    from research_api.examples import run_all_examples
    
    # Run all examples
    results = run_all_examples()

Run individual examples:
    from research_api.examples import complete_usage_examples
    
    # Run specific example
    complete_usage_examples.example_1_basic_experiment_setup()
"""

from .complete_usage_examples import (
    generate_sample_data,
    example_1_basic_experiment_setup,
    example_2_data_collection_and_analysis,
    example_3_publication_visualization,
    example_4_comprehensive_report_generation,
    example_5_os_instrumentation_workflow,
    example_6_complete_research_pipeline,
    run_all_examples
)

__all__ = [
    'generate_sample_data',
    'example_1_basic_experiment_setup',
    'example_2_data_collection_and_analysis',
    'example_3_publication_visualization',
    'example_4_comprehensive_report_generation',
    'example_5_os_instrumentation_workflow',
    'example_6_complete_research_pipeline',
    'run_all_examples'
]

__version__ = "1.0.0"
__author__ = "OS Research API Development Team"