#!/usr/bin/env python3
"""
MultiOS Test Generation Framework CLI

Command-line interface for the intelligent test case generation system.
Provides easy access to all framework capabilities for testing MultiOS edge cases.
"""

import argparse
import asyncio
import json
import sys
import os
from pathlib import Path
from typing import Dict, List, Any, Optional

# Add the framework to the path
sys.path.insert(0, str(Path(__file__).parent))

from testgen import (
    TestGenFramework, TestConfig, TestType,
    get_available_components, get_available_test_types,
    validate_component, validate_test_type
)

class MultiOSTestGenCLI:
    """Command-line interface for MultiOS test generation"""
    
    def __init__(self):
        self.framework = TestGenFramework()
        
    def main(self, args=None):
        """Main entry point"""
        parser = self.create_parser()
        parsed_args = parser.parse_args(args)
        
        try:
            return asyncio.run(self.execute_command(parsed_args))
        except KeyboardInterrupt:
            print("\nOperation cancelled by user")
            return 1
        except Exception as e:
            print(f"Error: {e}")
            return 1
    
    def create_parser(self):
        """Create command line argument parser"""
        parser = argparse.ArgumentParser(
            description="MultiOS Test Generation Framework",
            formatter_class=argparse.RawDescriptionHelpFormatter,
            epilog="""
Examples:
  # Generate comprehensive tests for filesystem
  %(prog)s comprehensive filesystem --iterations 1000
  
  # Generate edge case tests for memory
  %(prog)s edge-case memory --output-dir ./memory_tests
  
  # Generate fuzz tests for network component
  %(prog)s fuzz network --iterations 500
  
  # Generate property-based tests for API
  %(prog)s property api --seed 42
  
  # List available components
  %(prog)s list components
  
  # List available test types
  %(prog)s list test-types
            """
        )
        
        # Global options
        parser.add_argument(
            '--workspace', '-w',
            type=str,
            default='/workspace',
            help='Workspace directory path'
        )
        
        parser.add_argument(
            '--verbose', '-v',
            action='store_true',
            help='Verbose output'
        )
        
        parser.add_argument(
            '--version',
            action='version',
            version='%(prog)s 1.0.0'
        )
        
        # Subparsers
        subparsers = parser.add_subparsers(dest='command', help='Available commands')
        
        # List command
        list_parser = subparsers.add_parser('list', help='List available options')
        list_parser.add_argument('what', choices=['components', 'test-types'])
        
        # Comprehensive test generation
        comprehensive_parser = subparsers.add_parser('comprehensive', help='Generate comprehensive test suite')
        self.add_common_arguments(comprehensive_parser)
        
        # Edge case generation
        edge_parser = subparsers.add_parser('edge-case', help='Generate edge case tests')
        self.add_common_arguments(edge_parser)
        
        # Fuzz testing
        fuzz_parser = subparsers.add_parser('fuzz', help='Generate fuzz tests')
        self.add_common_arguments(fuzz_parser)
        
        # Property-based testing
        prop_parser = subparsers.add_parser('property', help='Generate property-based tests')
        self.add_common_arguments(prop_parser)
        
        # State space testing
        state_parser = subparsers.add_parser('state', help='Generate state space tests')
        self.add_common_arguments(state_parser)
        
        # Memory safety testing
        mem_parser = subparsers.add_parser('memory', help='Generate memory safety tests')
        self.add_common_arguments(mem_parser)
        
        # API compliance testing
        api_parser = subparsers.add_parser('api', help='Generate API compliance tests')
        self.add_common_arguments(api_parser)
        
        # Performance testing
        perf_parser = subparsers.add_parser('performance', help='Generate performance tests')
        self.add_common_arguments(perf_parser)
        
        # Coverage analysis
        coverage_parser = subparsers.add_parser('coverage', help='Analyze test coverage')
        coverage_parser.add_argument(
            'test_file',
            type=str,
            help='JSON file containing test results to analyze'
        )
        coverage_parser.add_argument(
            '--output', '-o',
            type=str,
            help='Output file for coverage report'
        )
        
        # Batch generation
        batch_parser = subparsers.add_parser('batch', help='Generate tests for multiple components')
        batch_parser.add_argument(
            'components',
            nargs='+',
            choices=get_available_components(),
            help='Components to generate tests for'
        )
        batch_parser.add_argument(
            '--test-type', '-t',
            type=str,
            choices=get_available_test_types(),
            default='comprehensive',
            help='Test type to generate'
        )
        batch_parser.add_argument(
            '--iterations', '-i',
            type=int,
            default=100,
            help='Number of iterations per component'
        )
        batch_parser.add_argument(
            '--parallel',
            action='store_true',
            help='Generate tests in parallel'
        )
        
        # Export command
        export_parser = subparsers.add_parser('export', help='Export test cases')
        export_parser.add_argument('test_id', type=str, help='Test ID to export')
        export_parser.add_argument(
            '--format', '-f',
            type=str,
            choices=['json', 'python', 'xml'],
            default='json',
            help='Export format'
        )
        export_parser.add_argument(
            '--output', '-o',
            type=str,
            help='Output file (default: stdout)'
        )
        
        # Benchmark command
        benchmark_parser = subparsers.add_parser('benchmark', help='Generate benchmark suite')
        benchmark_parser.add_argument(
            'component',
            type=str,
            choices=get_available_components(),
            help='Component to benchmark'
        )
        benchmark_parser.add_argument(
            '--output', '-o',
            type=str,
            help='Output file for benchmark configuration'
        )
        
        return parser
    
    def add_common_arguments(self, parser):
        """Add common arguments to test generation parsers"""
        parser.add_argument(
            'component',
            type=str,
            choices=get_available_components(),
            help='Component to generate tests for'
        )
        parser.add_argument(
            '--iterations', '-i',
            type=int,
            default=1000,
            help='Number of test iterations'
        )
        parser.add_argument(
            '--timeout', '-t',
            type=int,
            default=3600,
            help='Timeout in seconds'
        )
        parser.add_argument(
            '--seed', '-s',
            type=int,
            help='Random seed for reproducibility'
        )
        parser.add_argument(
            '--output-dir', '-o',
            type=str,
            default='testgen_output',
            help='Output directory for test results'
        )
        parser.add_argument(
            '--parameters', '-p',
            type=str,
            help='JSON string of additional parameters'
        )
        parser.add_argument(
            '--format', '-f',
            type=str,
            choices=['json', 'html', 'xml'],
            default='json',
            help='Output format'
        )
    
    async def execute_command(self, args):
        """Execute the requested command"""
        if args.command == 'list':
            return self.handle_list_command(args)
        elif args.command in ['comprehensive', 'edge-case', 'fuzz', 'property', 
                             'state', 'memory', 'api', 'performance']:
            return await self.handle_test_generation(args)
        elif args.command == 'coverage':
            return self.handle_coverage_analysis(args)
        elif args.command == 'batch':
            return await self.handle_batch_generation(args)
        elif args.command == 'export':
            return self.handle_export(args)
        elif args.command == 'benchmark':
            return self.handle_benchmark_generation(args)
        else:
            print("Please specify a command. Use --help for available commands.")
            return 1
    
    def handle_list_command(self, args):
        """Handle list command"""
        if args.what == 'components':
            print("Available components:")
            for component in get_available_components():
                print(f"  - {component}")
        elif args.what == 'test-types':
            print("Available test types:")
            for test_type in get_available_test_types():
                print(f"  - {test_type}")
        return 0
    
    async def handle_test_generation(self, args):
        """Handle test generation commands"""
        # Map command to test type
        test_type_mapping = {
            'comprehensive': TestType.COMPREHENSIVE,
            'edge-case': TestType.EDGE_CASE,
            'fuzz': TestType.FUZZ_TEST,
            'property': TestType.PROPERTY_BASED,
            'state': TestType.STATE_SPACE,
            'memory': TestType.MEMORY_SAFETY,
            'api': TestType.API_COMPLIANCE,
            'performance': TestType.PERFORMANCE
        }
        
        # Parse parameters
        parameters = {}
        if hasattr(args, 'parameters') and args.parameters:
            try:
                parameters = json.loads(args.parameters)
            except json.JSONDecodeError as e:
                print(f"Error parsing parameters: {e}")
                return 1
        
        # Create test configuration
        config = TestConfig(
            test_type=test_type_mapping[args.command],
            component=args.component,
            iterations=args.iterations,
            timeout=args.timeout,
            seed=args.seed,
            output_dir=args.output_dir,
            parameters=parameters
        )
        
        print(f"Generating {args.command} tests for {args.component}...")
        print(f"Iterations: {args.iterations}")
        print(f"Timeout: {args.timeout}s")
        if args.seed:
            print(f"Seed: {args.seed}")
        
        # Generate tests
        result = await self.framework.generate_test_cases(config)
        
        # Display results
        print(f"\nTest generation completed:")
        print(f"  Test ID: {result.test_id}")
        print(f"  Generated: {result.generated_count} test cases")
        print(f"  Duration: {result.execution_time:.2f}s")
        print(f"  Status: {result.status}")
        
        if result.errors:
            print(f"  Errors: {len(result.errors)}")
            for error in result.errors[:3]:  # Show first 3 errors
                print(f"    - {error}")
        
        # Save results in requested format
        output_file = f"{args.output_dir}/{result.test_id}.{args.format}"
        if args.format == 'json':
            with open(output_file, 'w') as f:
                json.dump(result.test_cases, f, indent=2)
        elif args.format == 'html':
            from testgen.utils.test_utils import TestReporter
            reporter = TestReporter()
            reporter.generate_html_report(result.test_cases, output_file)
        elif args.format == 'xml':
            from testgen.utils.test_utils import TestReporter
            reporter = TestReporter()
            reporter.generate_xml_report(result.test_cases, output_file)
        
        print(f"  Results saved to: {output_file}")
        
        return 0 if result.status == "success" else 1
    
    def handle_coverage_analysis(self, args):
        """Handle coverage analysis command"""
        print(f"Analyzing coverage from {args.test_file}...")
        
        # Load test results
        try:
            with open(args.test_file, 'r') as f:
                test_data = json.load(f)
        except Exception as e:
            print(f"Error loading test file: {e}")
            return 1
        
        # Analyze coverage
        component = args.component if hasattr(args, 'component') else "unknown"
        coverage_report = asyncio.run(
            self.framework.coverage_analyzer.analyze_coverage(
                test_data.get('test_cases', []), component
            )
        )
        
        # Display results
        print("\nCoverage Analysis:")
        print(f"  Component: {coverage_report.get('component', 'N/A')}")
        print(f"  Test Cases: {coverage_report.get('test_case_count', 0)}")
        print(f"  Effectiveness Score: {coverage_report.get('effectiveness_score', 0):.1f}%")
        
        metrics = coverage_report.get('coverage_metrics', {})
        if metrics:
            print("\n  Coverage Metrics:")
            for metric_name, metric in metrics.items():
                if hasattr(metric, 'coverage_percentage'):
                    coverage = metric.coverage_percentage
                else:
                    coverage = metric.get('coverage_percentage', 0)
                print(f"    {metric_name}: {coverage:.1f}%")
        
        # Save report
        output_file = args.output or f"coverage_report_{component}.json"
        with open(output_file, 'w') as f:
            json.dump(coverage_report, f, indent=2)
        
        print(f"\nCoverage report saved to: {output_file}")
        
        return 0
    
    async def handle_batch_generation(self, args):
        """Handle batch test generation"""
        print(f"Generating {args.test_type} tests for {len(args.components)} components...")
        
        test_type = TestType(args.test_type)
        configs = []
        
        for component in args.components:
            config = TestConfig(
                test_type=test_type,
                component=component,
                iterations=args.iterations,
                timeout=3600,
                output_dir=args.output_dir
            )
            configs.append((component, config))
        
        if args.parallel:
            # Generate in parallel
            tasks = [self.framework.generate_test_cases(config) for _, config in configs]
            results = await asyncio.gather(*tasks)
            
            for (component, _), result in zip(configs, results):
                print(f"\n{component}: {result.generated_count} tests generated")
        else:
            # Generate sequentially
            results = []
            for component, config in configs:
                result = await self.framework.generate_test_cases(config)
                results.append(result)
                print(f"{component}: {result.generated_count} tests generated")
        
        print(f"\nBatch generation completed!")
        return 0
    
    def handle_export(self, args):
        """Handle export command"""
        result = self.framework.get_test_results(args.test_id)
        
        if not result:
            print(f"Test ID {args.test_id} not found")
            return 1
        
        exported_data = self.framework.export_test_cases(args.test_id, args.format)
        
        if args.output:
            with open(args.output, 'w') as f:
                f.write(exported_data)
            print(f"Test cases exported to: {args.output}")
        else:
            print(exported_data)
        
        return 0
    
    def handle_benchmark_generation(self, args):
        """Handle benchmark generation"""
        print(f"Generating benchmark suite for {args.component}...")
        
        import asyncio
        benchmark_config = asyncio.run(
            self.framework.performance_gen.generate_performance_benchmark(
                args.component
            )
        )
        
        output_file = args.output or f"benchmark_{args.component}.json"
        with open(output_file, 'w') as f:
            json.dump(benchmark_config, f, indent=2)
        
        print(f"Benchmark suite generated: {output_file}")
        print(f"  Test Suite: {benchmark_config['benchmark_name']}")
        
        test_suite = benchmark_config.get('test_suite', {})
        if test_suite:
            print("  Available Tests:")
            for test_type, tests in test_suite.items():
                if isinstance(tests, list):
                    print(f"    {test_type}: {len(tests)} tests")
                else:
                    print(f"    {test_type}: configured")
        
        return 0

def main():
    """Entry point for CLI"""
    cli = MultiOSTestGenCLI()
    exit_code = cli.main()
    sys.exit(exit_code)

if __name__ == '__main__':
    main()