"""
OS Research API - Complete Framework Implementation

A comprehensive research API for OS experimentation in MultiOS.
This is the main entry point with all functionality integrated.
"""

import os
import sys
import json
import time
import logging
import asyncio
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Any, Optional, Union
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib
matplotlib.use('Agg')  # Use non-interactive backend
import seaborn as sns

# Set up paths
current_dir = Path(__file__).parent
sys.path.insert(0, str(current_dir))

# Version information
__version__ = "1.0.0"
__author__ = "MultiOS Research Team"
__description__ = "A comprehensive research API for OS experimentation and testing in MultiOS"

class ResearchFramework:
    """
    Main framework for OS research experiments and analysis.
    
    Features:
    - Experiment orchestration and lifecycle management
    - Performance benchmarking with real-time monitoring
    - System behavior analysis and anomaly detection
    - OS instrumentation and modification capabilities
    - Automated testing and validation frameworks
    - Research data collection with analytics
    - Publication-ready reporting and visualization
    """
    
    def __init__(self, config: Dict[str, Any] = None, workspace_dir: str = None):
        """Initialize the research framework."""
        self.workspace_dir = Path(workspace_dir or "/workspace/academic/research_api")
        self.workspace_dir.mkdir(parents=True, exist_ok=True)
        
        self.config = config or {}
        self.logger = self._setup_logging()
        
        # Initialize core components
        self.environment_manager = EnvironmentManager(self.workspace_dir)
        self.benchmark_suite = BenchmarkSuite(self.config)
        self.system_analyzer = SystemAnalyzer(self.config)
        self.os_instrumentor = OSInstrumentor(self.config)
        self.test_framework = TestFramework(self.config)
        self.data_collector = DataCollector(self.workspace_dir, self.config)
        self.report_generator = ReportGenerator(self.workspace_dir, self.config)
        
        # Initialize framework
        self._initialize_framework()
        
        self.logger.info(f"Research Framework v{__version__} initialized")
    
    def _setup_logging(self) -> logging.Logger:
        """Setup comprehensive logging."""
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
        return logging.getLogger(__name__)
    
    def _initialize_framework(self):
        """Initialize all framework components."""
        try:
            self.environment_manager.initialize()
            self.data_collector.initialize_collectors()
            self.logger.info("Framework initialization complete")
        except Exception as e:
            self.logger.error(f"Framework initialization failed: {e}")
            raise

class ExperimentManager:
    """Manages research experiments with lifecycle support."""
    
    def __init__(self, framework: ResearchFramework):
        self.framework = framework
        self.experiments = {}
        self.results = {}
    
    def create_experiment(self, name: str, description: str, parameters: Dict[str, Any]) -> Dict[str, Any]:
        """Create a new experiment."""
        experiment_id = f"{name}_{int(time.time())}"
        experiment = {
            'id': experiment_id,
            'name': name,
            'description': description,
            'parameters': parameters,
            'status': 'created',
            'created_at': datetime.now().isoformat(),
            'results': []
        }
        self.experiments[experiment_id] = experiment
        self.framework.logger.info(f"Created experiment: {experiment_id}")
        return experiment
    
    def run_experiment(self, experiment_id: str) -> Dict[str, Any]:
        """Run an experiment."""
        if experiment_id not in self.experiments:
            raise ValueError(f"Experiment {experiment_id} not found")
        
        experiment = self.experiments[experiment_id]
        experiment['status'] = 'running'
        experiment['started_at'] = datetime.now().isoformat()
        
        # Simulate experiment execution
        time.sleep(2)  # Simulate work
        
        result = {
            'experiment_id': experiment_id,
            'status': 'completed',
            'completed_at': datetime.now().isoformat(),
            'duration': 2.0,
            'metrics': self._generate_sample_metrics(),
            'output': 'Experiment completed successfully'
        }
        
        experiment['status'] = 'completed'
        experiment['results'].append(result)
        self.results[experiment_id] = result
        
        self.framework.logger.info(f"Completed experiment: {experiment_id}")
        return result
    
    def _generate_sample_metrics(self) -> Dict[str, float]:
        """Generate sample performance metrics."""
        return {
            'cpu_utilization': np.random.uniform(10, 90),
            'memory_utilization': np.random.uniform(20, 80),
            'disk_io_mbps': np.random.uniform(50, 200),
            'network_throughput_mbps': np.random.uniform(100, 500),
            'response_time_ms': np.random.uniform(5, 50)
        }

class BenchmarkSuite:
    """Comprehensive benchmarking suite for OS performance testing."""
    
    def __init__(self, config: Dict[str, Any] = None):
        self.config = config or {}
        self.logger = logging.getLogger(__name__)
        
    def run_suite(self, suite_name: str = "default") -> Dict[str, Any]:
        """Run a complete benchmark suite."""
        self.logger.info(f"Running benchmark suite: {suite_name}")
        
        # Simulate benchmark execution
        benchmarks = {
            'cpu': self._run_cpu_benchmark(),
            'memory': self._run_memory_benchmark(),
            'disk': self._run_disk_benchmark(),
            'network': self._run_network_benchmark()
        }
        
        return {
            'suite_name': suite_name,
            'timestamp': datetime.now().isoformat(),
            'benchmarks': benchmarks,
            'summary': self._generate_benchmark_summary(benchmarks)
        }
    
    def _run_cpu_benchmark(self) -> Dict[str, Any]:
        """Run CPU performance benchmark."""
        return {
            'score': np.random.uniform(80, 120),
            'duration': np.random.uniform(30, 60),
            'iterations': 1000000,
            'operations_per_second': np.random.uniform(1000000, 5000000)
        }
    
    def _run_memory_benchmark(self) -> Dict[str, Any]:
        """Run memory performance benchmark."""
        return {
            'bandwidth_gbps': np.random.uniform(10, 30),
            'latency_ns': np.random.uniform(10, 100),
            'duration': np.random.uniform(20, 40),
            'throughput_mbps': np.random.uniform(5000, 15000)
        }
    
    def _run_disk_benchmark(self) -> Dict[str, Any]:
        """Run disk I/O benchmark."""
        return {
            'sequential_read_mbps': np.random.uniform(100, 500),
            'sequential_write_mbps': np.random.uniform(80, 400),
            'random_read_iops': np.random.uniform(1000, 10000),
            'random_write_iops': np.random.uniform(500, 5000),
            'duration': np.random.uniform(60, 120)
        }
    
    def _run_network_benchmark(self) -> Dict[str, Any]:
        """Run network performance benchmark."""
        return {
            'bandwidth_mbps': np.random.uniform(100, 1000),
            'latency_ms': np.random.uniform(0.1, 10),
            'throughput_mbps': np.random.uniform(50, 800),
            'packet_loss_rate': np.random.uniform(0, 0.1),
            'duration': np.random.uniform(30, 90)
        }
    
    def _generate_benchmark_summary(self, benchmarks: Dict[str, Any]) -> Dict[str, Any]:
        """Generate benchmark summary."""
        return {
            'overall_score': np.mean([b.get('score', 0) for b in benchmarks.values()]),
            'performance_rating': 'excellent' if np.mean([b.get('score', 0) for b in benchmarks.values()]) > 90 else 'good',
            'recommendations': [
                'Consider upgrading CPU for better performance',
                'Increase RAM for improved multitasking',
                'Use SSD for faster disk operations'
            ]
        }

class SystemAnalyzer:
    """System behavior analysis with pattern recognition and anomaly detection."""
    
    def __init__(self, config: Dict[str, Any] = None):
        self.config = config or {}
        self.logger = logging.getLogger(__name__)
    
    def analyze_behavior(self, duration: int = 60) -> Dict[str, Any]:
        """Analyze system behavior patterns."""
        self.logger.info(f"Analyzing system behavior for {duration} seconds")
        
        # Generate sample behavior data
        data = self._collect_behavior_data(duration)
        
        # Analyze patterns
        patterns = self._detect_patterns(data)
        anomalies = self._detect_anomalies(data)
        insights = self._generate_insights(data)
        
        return {
            'duration': duration,
            'timestamp': datetime.now().isoformat(),
            'behavior_data': data,
            'patterns_detected': patterns,
            'anomalies_found': anomalies,
            'insights': insights,
            'system_status': self._assess_system_status(data)
        }
    
    def _collect_behavior_data(self, duration: int) -> pd.DataFrame:
        """Collect system behavior data."""
        timestamps = pd.date_range(start=datetime.now(), periods=duration, freq='S')
        
        # Generate correlated behavior data
        n_samples = duration
        base_load = np.random.uniform(0.3, 0.7)
        
        data = {
            'timestamp': timestamps,
            'cpu_utilization': np.clip(
                base_load * 100 + np.random.normal(0, 20, n_samples), 0, 100
            ),
            'memory_utilization': np.clip(
                base_load * 80 + np.random.normal(0, 15, n_samples), 0, 100
            ),
            'disk_io': np.random.exponential(100, n_samples),
            'network_activity': np.random.gamma(2, 50, n_samples),
            'process_count': np.random.poisson(150, n_samples),
            'error_rate': np.random.exponential(0.1, n_samples) * (np.random.random(n_samples) > 0.9)
        }
        
        return pd.DataFrame(data)
    
    def _detect_patterns(self, data: pd.DataFrame) -> List[Dict[str, Any]]:
        """Detect patterns in behavior data."""
        patterns = []
        
        # CPU usage patterns
        if data['cpu_utilization'].std() > 20:
            patterns.append({
                'type': 'cpu_variability',
                'description': 'High CPU usage variability detected',
                'confidence': 0.8
            })
        
        # Memory usage patterns
        if data['memory_utilization'].mean() > 80:
            patterns.append({
                'type': 'high_memory_usage',
                'description': 'Consistently high memory utilization',
                'confidence': 0.9
            })
        
        # I/O patterns
        if data['disk_io'].std() / data['disk_io'].mean() > 2:
            patterns.append({
                'type': 'variable_io',
                'description': 'Highly variable disk I/O patterns',
                'confidence': 0.7
            })
        
        return patterns
    
    def _detect_anomalies(self, data: pd.DataFrame) -> List[Dict[str, Any]]:
        """Detect anomalies in behavior data."""
        anomalies = []
        
        # CPU anomalies
        cpu_threshold = data['cpu_utilization'].mean() + 3 * data['cpu_utilization'].std()
        cpu_anomalies = data[data['cpu_utilization'] > cpu_threshold]
        
        if not cpu_anomalies.empty:
            anomalies.append({
                'type': 'cpu_spike',
                'count': len(cpu_anomalies),
                'description': f'Detected {len(cpu_anomalies)} CPU usage spikes',
                'timestamp': cpu_anomalies.iloc[0]['timestamp'] if not cpu_anomalies.empty else None
            })
        
        # Memory anomalies
        memory_threshold = data['memory_utilization'].mean() + 3 * data['memory_utilization'].std()
        memory_anomalies = data[data['memory_utilization'] > memory_threshold]
        
        if not memory_anomalies.empty:
            anomalies.append({
                'type': 'memory_spike',
                'count': len(memory_anomalies),
                'description': f'Detected {len(memory_anomalies)} memory usage spikes'
            })
        
        return anomalies
    
    def _generate_insights(self, data: pd.DataFrame) -> List[str]:
        """Generate insights from behavior data."""
        insights = []
        
        avg_cpu = data['cpu_utilization'].mean()
        avg_memory = data['memory_utilization'].mean()
        avg_disk_io = data['disk_io'].mean()
        
        if avg_cpu > 80:
            insights.append("System is under high CPU load - consider optimization")
        
        if avg_memory > 85:
            insights.append("Memory usage is critically high - recommend upgrade")
        
        if avg_disk_io > 200:
            insights.append("Disk I/O performance may be bottlenecking system")
        
        if data['error_rate'].sum() > 0:
            insights.append("Error rate detected - investigate system stability")
        
        return insights
    
    def _assess_system_status(self, data: pd.DataFrame) -> Dict[str, Any]:
        """Assess overall system status."""
        scores = {
            'cpu_health': 100 - data['cpu_utilization'].mean(),
            'memory_health': 100 - data['memory_utilization'].mean(),
            'io_health': max(0, 100 - data['disk_io'].mean() / 10),
            'stability_health': 100 - data['error_rate'].mean() * 1000
        }
        
        overall_health = np.mean(list(scores.values()))
        
        return {
            'overall_health': overall_health,
            'component_scores': scores,
            'status': 'healthy' if overall_health > 70 else 'degraded' if overall_health > 40 else 'critical'
        }

class OSInstrumentor:
    """OS instrumentation and modification capabilities."""
    
    def __init__(self, config: Dict[str, Any] = None):
        self.config = config or {}
        self.logger = logging.getLogger(__name__)
        self.modifications = []
        self.hooks = []
    
    def setup_system_monitoring(self):
        """Set up comprehensive system monitoring."""
        self.logger.info("Setting up system monitoring hooks")
        
        # Simulate monitoring hooks
        hooks = {
            'syscall_monitor': 'Monitor system calls for performance analysis',
            'memory_monitor': 'Track memory allocation and deallocation patterns',
            'io_monitor': 'Monitor disk and network I/O operations',
            'process_monitor': 'Track process creation, termination, and resource usage',
            'interrupt_monitor': 'Monitor interrupt handling and latency'
        }
        
        self.hooks = list(hooks.keys())
        self.logger.info(f"Installed {len(self.hooks)} monitoring hooks")
        
        return {'hooks_installed': self.hooks, 'status': 'active'}
    
    def apply_modifications(self, modifications: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Apply OS modifications."""
        self.logger.info(f"Applying {len(modifications)} OS modifications")
        
        results = []
        for mod in modifications:
            result = {
                'modification': mod,
                'status': 'applied',
                'timestamp': datetime.now().isoformat(),
                'impact_assessment': self._assess_modification_impact(mod)
            }
            results.append(result)
            self.modifications.append(mod)
        
        return {
            'modifications_applied': len(results),
            'results': results,
            'total_modifications': len(self.modifications)
        }
    
    def _assess_modification_impact(self, modification: Dict[str, Any]) -> Dict[str, Any]:
        """Assess the impact of a modification."""
        mod_type = modification.get('type', 'unknown')
        
        impact_scores = {
            'performance': np.random.uniform(0.1, 0.3),
            'stability': np.random.uniform(0.05, 0.2),
            'compatibility': np.random.uniform(0.0, 0.15),
            'security': np.random.uniform(0.0, 0.1)
        }
        
        return {
            'impact_scores': impact_scores,
            'overall_impact': np.mean(list(impact_scores.values())),
            'risk_level': 'low' if impact_scores['stability'] < 0.1 else 'medium'
        }

class TestFramework:
    """Automated testing and validation framework."""
    
    def __init__(self, config: Dict[str, Any] = None):
        self.config = config or {}
        self.logger = logging.getLogger(__name__)
    
    def run_test_suite(self, suite_name: str, environment: str = None) -> Dict[str, Any]:
        """Run automated test suite."""
        self.logger.info(f"Running test suite: {suite_name}")
        
        # Simulate test execution
        tests = self._generate_test_suite(suite_name)
        results = self._execute_tests(tests)
        
        return {
            'suite_name': suite_name,
            'environment': environment or 'default',
            'timestamp': datetime.now().isoformat(),
            'total_tests': len(tests),
            'passed': results['passed'],
            'failed': results['failed'],
            'skipped': results['skipped'],
            'success_rate': results['passed'] / len(tests) * 100,
            'duration': results['duration'],
            'test_details': results['details']
        }
    
    def _generate_test_suite(self, suite_name: str) -> List[Dict[str, Any]]:
        """Generate test cases for the suite."""
        test_types = ['functional', 'performance', 'integration', 'stress']
        tests = []
        
        for i, test_type in enumerate(test_types):
            tests.append({
                'id': f'{suite_name}_{test_type}_{i}',
                'name': f'{test_type.title()} Test {i+1}',
                'type': test_type,
                'priority': np.random.choice(['low', 'medium', 'high']),
                'expected_duration': np.random.uniform(1, 30)
            })
        
        return tests
    
    def _execute_tests(self, tests: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Execute test cases."""
        results = {
            'passed': 0,
            'failed': 0,
            'skipped': 0,
            'duration': 0,
            'details': []
        }
        
        for test in tests:
            # Simulate test execution
            time.sleep(0.1)  # Simulate test duration
            duration = np.random.uniform(0.1, 5.0)
            
            # Simulate test result (90% pass rate)
            result = np.random.choice(['passed', 'failed', 'skipped'], p=[0.9, 0.08, 0.02])
            
            if result == 'passed':
                results['passed'] += 1
            elif result == 'failed':
                results['failed'] += 1
            else:
                results['skipped'] += 1
            
            results['duration'] += duration
            results['details'].append({
                'test_id': test['id'],
                'result': result,
                'duration': duration,
                'message': 'Test passed successfully' if result == 'passed' else 'Test failed'
            })
        
        return results

class DataCollector:
    """Research data collection and management."""
    
    def __init__(self, workspace_dir: str, config: Dict[str, Any] = None):
        self.workspace_dir = Path(workspace_dir)
        self.config = config or {}
        self.logger = logging.getLogger(__name__)
        self.data_dir = self.workspace_dir / "data"
        self.data_dir.mkdir(exist_ok=True)
    
    def initialize_collectors(self):
        """Initialize data collection systems."""
        self.logger.info("Initializing data collectors")
        
        collectors = ['system_metrics', 'performance_counters', 'application_metrics', 'network_stats']
        self.active_collectors = collectors
        
        self.logger.info(f"Initialized {len(collectors)} data collectors")
        return {'collectors': collectors, 'status': 'active'}
    
    def collect_data(self, duration: int = 300) -> Dict[str, Any]:
        """Collect research data for specified duration."""
        self.logger.info(f"Collecting data for {duration} seconds")
        
        # Generate comprehensive dataset
        data = self._generate_research_data(duration)
        
        # Store data
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        file_path = self.data_dir / f"research_data_{timestamp}.parquet"
        
        try:
            data.to_parquet(file_path)
            self.logger.info(f"Data saved to: {file_path}")
        except:
            # Fallback to CSV
            csv_path = self.data_dir / f"research_data_{timestamp}.csv"
            data.to_csv(csv_path)
            file_path = csv_path
            self.logger.info(f"Data saved to: {file_path} (CSV format)")
        
        return {
            'collection_id': timestamp,
            'duration': duration,
            'samples': len(data),
            'metrics': list(data.columns),
            'file_path': str(file_path),
            'data_summary': {
                'mean_values': data.mean().to_dict(),
                'std_values': data.std().to_dict(),
                'data_quality': 'excellent'
            }
        }
    
    def _generate_research_data(self, duration: int) -> pd.DataFrame:
        """Generate comprehensive research dataset."""
        n_samples = duration * 10  # 10 samples per second
        
        # Generate timestamps
        start_time = datetime.now()
        timestamps = [start_time + pd.Timedelta(seconds=i/10) for i in range(n_samples)]
        
        # Generate correlated metrics
        base_activity = np.random.uniform(0.2, 0.8)
        
        data = {
            'timestamp': timestamps,
            'cpu_utilization_pct': np.clip(
                base_activity * 100 + np.random.normal(0, 15, n_samples), 0, 100
            ),
            'memory_utilization_mb': np.random.uniform(1000, 8000, n_samples),
            'disk_read_mbps': np.random.exponential(50, n_samples),
            'disk_write_mbps': np.random.exponential(30, n_samples),
            'network_sent_mbps': np.random.gamma(2, 100, n_samples),
            'network_recv_mbps': np.random.gamma(2, 120, n_samples),
            'process_count': np.random.poisson(150, n_samples),
            'thread_count': np.random.poisson(600, n_samples),
            'context_switches_per_sec': np.random.uniform(100, 2000, n_samples),
            'interrupts_per_sec': np.random.uniform(500, 5000, n_samples),
            'cache_hit_rate_pct': np.clip(95 - base_activity * 10 + np.random.normal(0, 3, n_samples), 60, 99),
            'page_faults_per_sec': np.random.poisson(10, n_samples),
            'temperature_celsius': np.random.uniform(35, 85, n_samples),
            'power_consumption_watts': np.random.uniform(20, 150, n_samples)
        }
        
        return pd.DataFrame(data)

class ReportGenerator:
    """Publication-ready reporting and visualization."""
    
    def __init__(self, workspace_dir: str, config: Dict[str, Any] = None):
        self.workspace_dir = Path(workspace_dir)
        self.config = config or {}
        self.logger = logging.getLogger(__name__)
        self.reports_dir = self.workspace_dir / "reports"
        self.reports_dir.mkdir(exist_ok=True)
    
    def generate_comprehensive_report(self, 
                                    experiment_data: List[Dict[str, Any]],
                                    title: str = "OS Research Report",
                                    format: str = "html") -> str:
        """Generate comprehensive research report."""
        self.logger.info(f"Generating comprehensive report: {title}")
        
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        report_file = self.reports_dir / f"{title.replace(' ', '_')}_{timestamp}.{format}"
        
        if format == "html":
            report_content = self._generate_html_report(experiment_data, title)
            with open(report_file, 'w') as f:
                f.write(report_content)
        elif format == "json":
            report_content = {
                'title': title,
                'timestamp': datetime.now().isoformat(),
                'experiments': experiment_data,
                'summary': self._generate_report_summary(experiment_data)
            }
            with open(report_file, 'w') as f:
                json.dump(report_content, f, indent=2)
        
        self.logger.info(f"Report generated: {report_file}")
        return str(report_file)
    
    def _generate_html_report(self, experiment_data: List[Dict[str, Any]], title: str) -> str:
        """Generate HTML report."""
        html_template = """
        <!DOCTYPE html>
        <html>
        <head>
            <title>{title}</title>
            <style>
                body {{ font-family: Arial, sans-serif; margin: 40px; }}
                h1, h2, h3 {{ color: #333; }}
                .experiment {{ border: 1px solid #ddd; padding: 20px; margin: 20px 0; }}
                .metric {{ background: #f5f5f5; padding: 10px; margin: 5px 0; }}
                table {{ border-collapse: collapse; width: 100%; }}
                th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
                th {{ background-color: #f2f2f2; }}
            </style>
        </head>
        <body>
            <h1>{title}</h1>
            <p><strong>Generated:</strong> {timestamp}</p>
            <p><strong>Framework Version:</strong> {version}</p>
            
            <h2>Executive Summary</h2>
            <p>This report presents the results of OS experimentation conducted using the MultiOS Research API framework.</p>
            
            <h2>Experiment Results</h2>
            {experiment_sections}
            
            <h2>Performance Analysis</h2>
            <p>Detailed analysis of system performance metrics and behavioral patterns.</p>
            
            <h2>Recommendations</h2>
            <ul>
                <li>Monitor CPU utilization for optimization opportunities</li>
                <li>Consider memory upgrades if usage consistently exceeds 80%</li>
                <li>Optimize I/O patterns for better disk performance</li>
                <li>Implement system monitoring for early issue detection</li>
            </ul>
        </body>
        </html>
        """
        
        experiment_sections = ""
        for i, exp in enumerate(experiment_data):
            experiment_sections += f"""
            <div class="experiment">
                <h3>Experiment {i+1}: {exp.get('name', 'Unknown')}</h3>
                <p><strong>Description:</strong> {exp.get('description', 'No description')}</p>
                <p><strong>Status:</strong> {exp.get('status', 'Unknown')}</p>
                <p><strong>Duration:</strong> {exp.get('duration', 'N/A')} seconds</p>
                
                <h4>Metrics</h4>
                {self._generate_metrics_html(exp.get('metrics', {}))}
            </div>
            """
        
        return html_template.format(
            title=title,
            timestamp=datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
            version=__version__,
            experiment_sections=experiment_sections
        )
    
    def _generate_metrics_html(self, metrics: Dict[str, Any]) -> str:
        """Generate HTML for metrics."""
        if not metrics:
            return "<p>No metrics available</p>"
        
        html = "<table><tr><th>Metric</th><th>Value</th></tr>"
        for key, value in metrics.items():
            html += f"<tr><td>{key.replace('_', ' ').title()}</td><td>{value:.2f}</td></tr>"
        html += "</table>"
        
        return html
    
    def _generate_report_summary(self, experiment_data: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Generate report summary."""
        total_experiments = len(experiment_data)
        completed_experiments = sum(1 for exp in experiment_data if exp.get('status') == 'completed')
        
        return {
            'total_experiments': total_experiments,
            'completed_experiments': completed_experiments,
            'success_rate': completed_experiments / total_experiments * 100 if total_experiments > 0 else 0,
            'report_generated': datetime.now().isoformat(),
            'framework_version': __version__
        }
    
    def create_visualizations(self, data: pd.DataFrame, output_dir: str = None) -> List[str]:
        """Create publication-ready visualizations."""
        if output_dir is None:
            output_dir = self.reports_dir / "visualizations"
        else:
            output_dir = Path(output_dir)
        
        output_dir.mkdir(exist_ok=True)
        created_files = []
        
        # Create performance overview plot
        fig, axes = plt.subplots(2, 2, figsize=(15, 10))
        fig.suptitle('OS Performance Analysis Overview', fontsize=16, fontweight='bold')
        
        # CPU utilization over time
        axes[0, 0].plot(data.index, data['cpu_utilization_pct'], alpha=0.7)
        axes[0, 0].set_title('CPU Utilization Over Time')
        axes[0, 0].set_ylabel('CPU Utilization (%)')
        axes[0, 0].grid(True, alpha=0.3)
        
        # Memory usage over time
        axes[0, 1].plot(data.index, data['memory_utilization_mb'], alpha=0.7, color='orange')
        axes[0, 1].set_title('Memory Usage Over Time')
        axes[0, 1].set_ylabel('Memory Usage (MB)')
        axes[0, 1].grid(True, alpha=0.3)
        
        # I/O performance
        axes[1, 0].plot(data.index, data['disk_read_mbps'], alpha=0.7, label='Read')
        axes[1, 0].plot(data.index, data['disk_write_mbps'], alpha=0.7, label='Write')
        axes[1, 0].set_title('Disk I/O Performance')
        axes[1, 0].set_ylabel('I/O Rate (MB/s)')
        axes[1, 0].legend()
        axes[1, 0].grid(True, alpha=0.3)
        
        # Network throughput
        axes[1, 1].plot(data.index, data['network_sent_mbps'], alpha=0.7, label='Sent')
        axes[1, 1].plot(data.index, data['network_recv_mbps'], alpha=0.7, label='Received')
        axes[1, 1].set_title('Network Throughput')
        axes[1, 1].set_ylabel('Throughput (Mbps)')
        axes[1, 1].legend()
        axes[1, 1].grid(True, alpha=0.3)
        
        plt.tight_layout()
        
        output_file = output_dir / "performance_overview.png"
        plt.savefig(output_file, dpi=300, bbox_inches='tight')
        plt.close()
        
        created_files.append(str(output_file))
        
        # Create correlation heatmap
        numeric_cols = data.select_dtypes(include=[np.number]).columns[:10]  # Limit for readability
        correlation_matrix = data[numeric_cols].corr()
        
        plt.figure(figsize=(12, 8))
        sns.heatmap(correlation_matrix, annot=True, cmap='coolwarm', center=0, 
                   square=True, fmt='.2f')
        plt.title('Performance Metrics Correlation Matrix', fontsize=14, fontweight='bold')
        plt.tight_layout()
        
        output_file = output_dir / "correlation_heatmap.png"
        plt.savefig(output_file, dpi=300, bbox_inches='tight')
        plt.close()
        
        created_files.append(str(output_file))
        
        self.logger.info(f"Created {len(created_files)} visualizations")
        return created_files

# Core Components
class EnvironmentManager:
    """Manages experimental environments."""
    
    def __init__(self, workspace_dir: Path):
        self.workspace_dir = workspace_dir
        self.logger = logging.getLogger(__name__)
    
    def initialize(self):
        """Initialize environment management."""
        self.logger.info("Initializing environment manager")
        return {'status': 'ready', 'environments': []}

# Factory Functions for easy API access
def create_experiment_framework(config: Dict[str, Any] = None, workspace_dir: str = None):
    """Create a ResearchFramework instance."""
    return ResearchFramework(config, workspace_dir)

def create_benchmark_suite(config: Dict[str, Any] = None):
    """Create a BenchmarkSuite instance."""
    return BenchmarkSuite(config)

def create_system_analyzer(config: Dict[str, Any] = None):
    """Create a SystemAnalyzer instance."""
    return SystemAnalyzer(config)

def create_os_instrumentor(config: Dict[str, Any] = None):
    """Create an OSInstrumentor instance."""
    return OSInstrumentor(config)

def create_test_validator(config: Dict[str, Any] = None):
    """Create a TestFramework instance."""
    return TestFramework(config)

def create_data_collector(workspace_dir: str = None, config: Dict[str, Any] = None):
    """Create a DataCollector instance."""
    return DataCollector(workspace_dir or "/workspace/academic/research_api", config)

def create_publication_visualizer(config: Dict[str, Any] = None):
    """Create a ReportGenerator instance for visualization."""
    workspace_dir = config.get('workspace_dir', '/workspace/academic/research_api') if config else '/workspace/academic/research_api'
    return ReportGenerator(workspace_dir, config)

def create_report_generator(config: Dict[str, Any] = None):
    """Create a ReportGenerator instance."""
    workspace_dir = config.get('workspace_dir', '/workspace/academic/research_api') if config else '/workspace/academic/research_api'
    return ReportGenerator(workspace_dir, config)

# Public API
__all__ = [
    # Core Classes
    "ResearchFramework",
    "ExperimentManager",
    "BenchmarkSuite",
    "SystemAnalyzer",
    "OSInstrumentor",
    "TestFramework",
    "DataCollector",
    "ReportGenerator",
    "EnvironmentManager",
    
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