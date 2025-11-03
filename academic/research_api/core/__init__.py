"""
MultiOS Research API - Core Framework

Comprehensive research framework for operating system experimentation and testing.
Provides modular APIs for benchmarking, analysis, instrumentation, and automated testing.
"""

from .experiment import Experiment, ExperimentResult, ExperimentRunner
from .config import ResearchConfig
from .environment import EnvironmentManager

__version__ = "1.0.0"
__author__ = "MultiOS Research Team"

__all__ = [
    "Experiment",
    "ExperimentResult", 
    "ExperimentRunner",
    "ResearchConfig",
    "EnvironmentManager"
]