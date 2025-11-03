"""
Core Research Framework

Main orchestration class for the MultiOS Research API.
Coordinates all components and provides unified interface for research operations.
"""

import os
import json
import logging
from typing import Dict, List, Any, Optional, Union
from pathlib import Path
from datetime import datetime
import asyncio
import yaml
import pandas as pd
import numpy as np

from .config import ResearchConfig
from .environment import EnvironmentManager
from .experiment import Experiment, ExperimentRunner
from ..benchmarking.benchmark import BenchmarkSuite
from ..analysis.system import SystemAnalyzer
from ..instrumentation.hooks import OSInstrumentor
from ..testing.validator import TestFramework
from ..data_collection.collector import DataCollector
from ..reporting.visualization_engine import ReportGenerator


class ResearchFramework:
    """
    Main framework for coordinating OS research experiments and operations.
    
    Provides unified interface for:
    - Experiment orchestration
    - Performance benchmarking
    - System analysis
    - Data collection and visualization
    - Research automation
    """
    
    def __init__(self, 
                 config_path: Optional[str] = None,
                 workspace_dir: str = "/workspace/academic/research_api"):
        """
        Initialize the research framework.
        
        Args:
            config_path: Path to configuration file
            workspace_dir: Base directory for research workspace
        """
        self.workspace_dir = Path(workspace_dir)
        self.workspace_dir.mkdir(parents=True, exist_ok=True)
        
        # Initialize configuration
        self.config = ResearchConfig(config_path)
        
        # Initialize core components
        self.environment_manager = EnvironmentManager(self.workspace_dir)
        self.benchmark_suite = BenchmarkSuite(self.config)
        self.system_analyzer = SystemAnalyzer(self.config)
        self.os_instrumentor = OSInstrumentor(self.config)
        self.test_framework = TestFramework(self.config)
        self.data_collector = DataCollector(self.workspace_dir, self.config)
        self.report_generator = ReportGenerator(self.workspace_dir, self.config)
        
        # Component registry
        self._components = {}
        self._experiments = {}
        
        # Setup logging
        self._setup_logging()
        
        # Initialize framework
        self._initialize_framework()
    
    def _setup_logging(self):
        """Setup comprehensive logging for research activities."""
        log_dir = self.workspace_dir / "logs"
        log_dir.mkdir(exist_ok=True)
        
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
            handlers=[
                logging.FileHandler(log_dir / "research_framework.log"),
                logging.StreamHandler()
            ]
        )
        
        self.logger = logging.getLogger(__name__)
        self.logger.info("Research framework initialized")
    
    def _initialize_framework(self):
        """Initialize all framework components."""
        try:
            # Initialize environment
            self.environment_manager.initialize()
            
            # Setup instrumentation
            self.os_instrumentor.setup_system_monitoring()
            
            # Initialize data collection
            self.data_collector.initialize_collectors()
            
            self.logger.info("Framework initialization complete")
            
        except Exception as e:
            self.logger.error(f"Framework initialization failed: {e}")
            raise
    
    def register_component(self, name: str, component: Any):
        """
        Register a custom component with the framework.
        
        Args:
            name: Component name
            component: Component instance
        """
        self._components[name] = component
        self.logger.info(f"Registered component: {name}")
    
    def get_component(self, name: str) -> Optional[Any]:
        """
        Retrieve a registered component.
        
        Args:
            name: Component name
            
        Returns:
            Component instance or None
        """
        return self._components.get(name)
    
    def create_experiment(self, 
                         name: str,
                         description: str,
                         parameters: Dict[str, Any],
                         environment: Optional[Dict[str, Any]] = None) -> Experiment:
        """
        Create a new research experiment.
        
        Args:
            name: Experiment name
            description: Experiment description
            parameters: Experiment parameters
            environment: Environment configuration
            
        Returns:
            Experiment instance
        """
        experiment = Experiment(
            name=name,
            description=description,
            parameters=parameters,
            environment=environment or {},
            framework=self
        )
        
        self._experiments[name] = experiment
        self.logger.info(f"Created experiment: {name}")
        
        return experiment
    
    def get_experiment(self, name: str) -> Optional[Experiment]:
        """
        Retrieve an existing experiment.
        
        Args:
            name: Experiment name
            
        Returns:
            Experiment instance or None
        """
        return self._experiments.get(name)
    
    async def run_experiment(self, 
                           experiment_name: str,
                           async_mode: bool = True) -> Dict[str, Any]:
        """
        Run a research experiment.
        
        Args:
            experiment_name: Name of experiment to run
            async_mode: Run experiment asynchronously
            
        Returns:
            Experiment results
        """
        experiment = self.get_experiment(experiment_name)
        if not experiment:
            raise ValueError(f"Experiment {experiment_name} not found")
        
        self.logger.info(f"Starting experiment: {experiment_name}")
        
        if async_mode:
            runner = ExperimentRunner(experiment, self.config)
            results = await runner.run_async()
        else:
            runner = ExperimentRunner(experiment, self.config)
            results = runner.run_sync()
        
        # Store results
        results_file = self.workspace_dir / "results" / f"{experiment_name}_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"
        results_file.parent.mkdir(exist_ok=True)
        
        with open(results_file, 'w') as f:
            json.dump(results, f, indent=2, default=str)
        
        self.logger.info(f"Experiment {experiment_name} completed")
        return results
    
    def run_benchmark_suite(self, 
                          suite_name: str,
                          metrics: Optional[List[str]] = None) -> Dict[str, Any]:
        """
        Run performance benchmark suite.
        
        Args:
            suite_name: Name of benchmark suite
            metrics: Specific metrics to collect
            
        Returns:
            Benchmark results
        """
        self.logger.info(f"Running benchmark suite: {suite_name}")
        
        if metrics:
            self.benchmark_suite.configure_metrics(metrics)
        
        results = self.benchmark_suite.run_suite(suite_name)
        
        # Analyze results
        analysis = self.system_analyzer.analyze_benchmark_results(results)
        
        # Visualize results
        charts = self.report_generator.create_benchmark_charts(results)
        
        return {
            'results': results,
            'analysis': analysis,
            'charts': charts,
            'suite_name': suite_name,
            'timestamp': datetime.now().isoformat()
        }
    
    def analyze_system_behavior(self, 
                              duration: int = 60,
                              metrics: Optional[List[str]] = None) -> Dict[str, Any]:
        """
        Analyze system behavior patterns.
        
        Args:
            duration: Analysis duration in seconds
            metrics: Metrics to analyze
            
        Returns:
            Analysis results
        """
        self.logger.info(f"Starting system behavior analysis for {duration}s")
        
        # Collect system data
        behavior_data = self.system_analyzer.collect_behavior_data(duration, metrics)
        
        # Detect patterns and anomalies
        patterns = self.system_analyzer.detect_patterns(behavior_data)
        anomalies = self.system_analyzer.detect_anomalies(behavior_data)
        
        # Generate insights
        insights = self.system_analyzer.generate_insights(behavior_data)
        
        # Create visualizations
        charts = self.report_generator.create_behavior_charts(behavior_data)
        
        return {
            'behavior_data': behavior_data,
            'patterns': patterns,
            'anomalies': anomalies,
            'insights': insights,
            'charts': charts,
            'analysis_duration': duration
        }
    
    def instrument_os(self, 
                     modifications: List[Dict[str, Any]],
                     monitor_events: bool = True) -> Dict[str, Any]:
        """
        Instrument OS with custom modifications and monitoring.
        
        Args:
            modifications: List of OS modifications to apply
            monitor_events: Enable event monitoring
            
        Returns:
            Instrumentation results
        """
        self.logger.info("Starting OS instrumentation")
        
        # Apply modifications
        mod_results = self.os_instrumentor.apply_modifications(modifications)
        
        # Setup monitoring if requested
        if monitor_events:
            self.os_instrumentor.setup_event_monitoring()
        
        # Collect baseline metrics
        baseline = self.system_analyzer.collect_baseline_metrics()
        
        return {
            'modifications_applied': mod_results,
            'baseline_metrics': baseline,
            'instrumentation_timestamp': datetime.now().isoformat()
        }
    
    def run_automated_tests(self, 
                          test_suite: str,
                          environment: Optional[str] = None) -> Dict[str, Any]:
        """
        Run automated test suite.
        
        Args:
            test_suite: Name of test suite to run
            environment: Target environment
            
        Returns:
            Test results
        """
        self.logger.info(f"Running automated tests: {test_suite}")
        
        test_results = self.test_framework.run_test_suite(test_suite, environment)
        
        # Analyze test results
        analysis = self.test_framework.analyze_results(test_results)
        
        # Generate test report
        report = self.test_framework.generate_report(test_results, analysis)
        
        return {
            'test_results': test_results,
            'analysis': analysis,
            'report': report,
            'test_suite': test_suite,
            'environment': environment
        }
    
    def collect_research_data(self,
                            duration: int = 300,
                            metrics: Optional[List[str]] = None,
                            storage_format: str = "parquet") -> Dict[str, Any]:
        """
        Collect comprehensive research data.
        
        Args:
            duration: Data collection duration
            metrics: Metrics to collect
            storage_format: Storage format (parquet, csv, json)
            
        Returns:
            Collection results
        """
        self.logger.info(f"Starting research data collection for {duration}s")
        
        # Configure data collection
        if metrics:
            self.data_collector.configure_metrics(metrics)
        
        # Start data collection
        collection_id = self.data_collector.start_collection(duration)
        
        # Collect system metrics
        system_data = self.data_collector.collect_system_metrics(duration)
        
        # Collect performance data
        performance_data = self.data_collector.collect_performance_data(duration)
        
        # Collect behavior data
        behavior_data = self.data_collector.collect_behavior_data(duration)
        
        # Store data
        storage_results = self.data_collector.store_data(
            collection_id, 
            system_data, 
            performance_data, 
            behavior_data,
            storage_format
        )
        
        return {
            'collection_id': collection_id,
            'system_data': system_data,
            'performance_data': performance_data,
            'behavior_data': behavior_data,
            'storage_results': storage_results,
            'duration': duration
        }
    
    def generate_research_report(self,
                               experiment_ids: List[str],
                               report_type: str = "comprehensive",
                               output_format: str = "html") -> str:
        """
        Generate comprehensive research report.
        
        Args:
            experiment_ids: List of experiment IDs to include
            report_type: Type of report (comprehensive, summary, publication)
            output_format: Output format (html, pdf, latex)
            
        Returns:
            Path to generated report
        """
        self.logger.info(f"Generating research report: {report_type}")
        
        # Collect experiment data
        experiment_data = []
        for exp_id in experiment_ids:
            exp_data = self._load_experiment_data(exp_id)
            experiment_data.append(exp_data)
        
        # Generate report
        report_path = self.report_generator.generate_report(
            experiment_data, 
            report_type, 
            output_format
        )
        
        return report_path
    
    def _load_experiment_data(self, experiment_id: str) -> Dict[str, Any]:
        """Load experiment data from storage."""
        results_dir = self.workspace_dir / "results"
        # Implementation for loading experiment data
        # This would load from JSON files or database
        pass
    
    def get_framework_status(self) -> Dict[str, Any]:
        """
        Get current framework status and health.
        
        Returns:
            Framework status information
        """
        return {
            'framework_version': __version__,
            'initialized_components': list(self._components.keys()),
            'active_experiments': list(self._experiments.keys()),
            'workspace_directory': str(self.workspace_dir),
            'configuration': self.config.get_config_dict(),
            'system_status': self.system_analyzer.get_system_status(),
            'data_collection_status': self.data_collector.get_status()
        }
    
    def export_research_data(self,
                           experiment_ids: List[str],
                           export_format: str = "csv",
                           include_analysis: bool = True) -> str:
        """
        Export research data in various formats.
        
        Args:
            experiment_ids: List of experiment IDs
            export_format: Export format (csv, json, xlsx)
            include_analysis: Include analysis results
            
        Returns:
            Path to exported data file
        """
        export_dir = self.workspace_dir / "exports"
        export_dir.mkdir(exist_ok=True)
        
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        export_file = export_dir / f"research_data_{timestamp}.{export_format}"
        
        # Collect data
        data = {
            'experiments': [],
            'metadata': {
                'export_timestamp': datetime.now().isoformat(),
                'export_format': export_format,
                'include_analysis': include_analysis
            }
        }
        
        for exp_id in experiment_ids:
            exp_data = self._load_experiment_data(exp_id)
            data['experiments'].append(exp_data)
        
        # Export data
        if export_format == "csv":
            # Convert to DataFrame and export
            df = pd.DataFrame(data['experiments'])
            df.to_csv(export_file, index=False)
        elif export_format == "json":
            with open(export_file, 'w') as f:
                json.dump(data, f, indent=2, default=str)
        elif export_format == "xlsx":
            df = pd.DataFrame(data['experiments'])
            df.to_excel(export_file, index=False)
        
        self.logger.info(f"Exported research data to: {export_file}")
        return str(export_file)
    
    def shutdown(self):
        """Gracefully shutdown the research framework."""
        self.logger.info("Shutting down research framework")
        
        # Stop data collection
        if hasattr(self, 'data_collector'):
            self.data_collector.stop_collection()
        
        # Stop instrumentation
        if hasattr(self, 'os_instrumentor'):
            self.os_instrumentor.cleanup()
        
        # Save configuration
        if self.config:
            self.config.save_config()
        
        self.logger.info("Research framework shutdown complete")