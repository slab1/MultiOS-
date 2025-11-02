#!/usr/bin/env python3
"""
MultiOS Test Generation Framework - Demo Script

Demonstrates the capabilities of the test generation framework
by creating sample test suites for different components.
"""

import asyncio
import sys
import json
from pathlib import Path

# Add framework to path
sys.path.insert(0, str(Path(__file__).parent))

from testing.testgen import (
    create_framework, TestConfig, TestType,
    get_available_components, get_available_test_types
)

class TestGenDemo:
    """Demo class for testing framework capabilities"""
    
    def __init__(self):
        self.framework = create_framework()
        
    async def demo_comprehensive_testing(self):
        """Demonstrate comprehensive test generation"""
        print("\n" + "="*60)
        print("COMPREHENSIVE TEST GENERATION DEMO")
        print("="*60)
        
        components = ["filesystem", "memory", "api"]
        
        for component in components:
            print(f"\nGenerating comprehensive tests for {component}...")
            
            config = TestConfig(
                test_type=TestType.COMPREHENSIVE,
                component=component,
                iterations=100,  # Small number for demo
                timeout=300
            )
            
            result = await self.framework.generate_test_cases(config)
            
            print(f"✓ Generated {result.generated_count} test cases")
            print(f"  Duration: {result.execution_time:.2f}s")
            print(f"  Status: {result.status}")
            
            if result.test_cases:
                sample_case = result.test_cases[0]
                print(f"  Sample test: {sample_case.get('name', 'N/A')}")
    
    async def demo_edge_case_generation(self):
        """Demonstrate edge case generation"""
        print("\n" + "="*60)
        print("EDGE CASE GENERATION DEMO")
        print("="*60)
        
        config = TestConfig(
            test_type=TestType.EDGE_CASE,
            component="filesystem",
            iterations=50
        )
        
        result = await self.framework.generate_test_cases(config)
        
        print(f"Generated {result.generated_count} edge case tests")
        print(f"Coverage metrics: {result.coverage_report}")
        
        # Show different types of edge cases
        test_types = {}
        for test_case in result.test_cases:
            test_type = test_case.get("type", "unknown")
            test_types[test_type] = test_types.get(test_type, 0) + 1
        
        print("\nEdge case distribution:")
        for test_type, count in test_types.items():
            print(f"  {test_type}: {count} tests")
    
    async def demo_fuzz_testing(self):
        """Demonstrate fuzz testing"""
        print("\n" + "="*60)
        print("FUZZ TESTING DEMO")
        print("="*60)
        
        config = TestConfig(
            test_type=TestType.FUZZ_TEST,
            component="network",
            iterations=75
        )
        
        result = await self.framework.generate_test_cases(config)
        
        print(f"Generated {result.generated_count} fuzz tests")
        
        # Analyze fuzz test types
        input_types = {}
        for test_case in result.test_cases:
            input_data = test_case.get("input_data", {})
            fuzz_type = input_data.get("fuzz_type", "unknown")
            input_types[fuzz_type] = input_types.get(fuzz_type, 0) + 1
        
        print("\nFuzz test types:")
        for fuzz_type, count in input_types.items():
            print(f"  {fuzz_type}: {count} tests")
    
    async def demo_property_based_testing(self):
        """Demonstrate property-based testing"""
        print("\n" + "="*60)
        print("PROPERTY-BASED TESTING DEMO")
        print("="*60)
        
        config = TestConfig(
            test_type=TestType.PROPERTY_BASED,
            component="memory",
            iterations=60
        )
        
        result = await self.framework.generate_test_cases(config)
        
        print(f"Generated {result.generated_count} property-based tests")
        
        # Show property types
        property_names = set()
        for test_case in result.test_cases:
            prop_name = test_case.get("property_name", "unknown")
            property_names.add(prop_name)
        
        print(f"\nProperties tested: {', '.join(property_names)}")
    
    async def demo_coverage_analysis(self):
        """Demonstrate coverage analysis"""
        print("\n" + "="*60)
        print("COVERAGE ANALYSIS DEMO")
        print("="*60)
        
        # Generate some test cases first
        config = TestConfig(
            test_type=TestType.COMPREHENSIVE,
            component="api",
            iterations=80
        )
        
        result = await self.framework.generate_test_cases(config)
        
        print(f"Analyzing coverage for {result.generated_count} tests...")
        
        # Analyze coverage
        coverage_report = await self.framework.coverage_analyzer.analyze_coverage(
            result.test_cases, "api"
        )
        
        print(f"Effectiveness Score: {coverage_report.get('effectiveness_score', 0):.1f}%")
        
        metrics = coverage_report.get('coverage_metrics', {})
        print("\nCoverage Metrics:")
        for metric_name, metric in metrics.items():
            if hasattr(metric, 'coverage_percentage'):
                coverage = metric.coverage_percentage
            else:
                coverage = metric.get('coverage_percentage', 0)
            print(f"  {metric_name}: {coverage:.1f}%")
    
    async def demo_batch_generation(self):
        """Demonstrate batch test generation"""
        print("\n" + "="*60)
        print("BATCH GENERATION DEMO")
        print("="*60)
        
        components = ["filesystem", "memory", "process"]
        test_type = TestType.EDGE_CASE
        
        print(f"Generating {test_type.value} tests for {len(components)} components...")
        
        results = []
        for component in components:
            config = TestConfig(
                test_type=test_type,
                component=component,
                iterations=40
            )
            
            result = await self.framework.generate_test_cases(config)
            results.append(result)
            
            print(f"  {component}: {result.generated_count} tests")
        
        total_tests = sum(r.generated_count for r in results)
        total_time = sum(r.execution_time for r in results)
        
        print(f"\nBatch Summary:")
        print(f"  Total tests generated: {total_tests}")
        print(f"  Total generation time: {total_time:.2f}s")
        print(f"  Average time per component: {total_time/len(components):.2f}s")
    
    async def demo_test_export(self):
        """Demonstrate test export capabilities"""
        print("\n" + "="*60)
        print("TEST EXPORT DEMO")
        print("="*60)
        
        # Generate a small test suite
        config = TestConfig(
            test_type=TestType.COMPREHENSIVE,
            component="filesystem",
            iterations=20
        )
        
        result = await self.framework.generate_test_cases(config)
        
        print(f"Exporting {result.test_id} with {result.generated_count} tests...")
        
        # Export in different formats
        formats = ["json", "python", "xml"]
        
        for format_type in formats:
            try:
                exported = self.framework.export_test_cases(result.test_id, format_type)
                print(f"  {format_type.upper()}: {len(exported)} characters")
                
                # Save to file for inspection
                output_file = f"/workspace/testing/testgen/demo_export_{format_type}.{format_type}"
                with open(output_file, 'w') as f:
                    f.write(exported)
                print(f"    Saved to: {output_file}")
            except Exception as e:
                print(f"  {format_type.upper()}: Error - {e}")
    
    def show_framework_info(self):
        """Show framework information"""
        print("\n" + "="*60)
        print("FRAMEWORK INFORMATION")
        print("="*60)
        
        print("\nAvailable Components:")
        for component in get_available_components():
            print(f"  - {component}")
        
        print("\nAvailable Test Types:")
        for test_type in get_available_test_types():
            print(f"  - {test_type}")
        
        print(f"\nFramework Version: 1.0.0")
        print(f"Total Generators: 7")
        print(f"Analysis Tools: 1")
        print(f"Utility Modules: 1")

async def run_all_demos():
    """Run all demonstration functions"""
    demo = TestGenDemo()
    
    # Show framework info
    demo.show_framework_info()
    
    # Run individual demos
    await demo.demo_comprehensive_testing()
    await demo.demo_edge_case_generation()
    await demo.demo_fuzz_testing()
    await demo.demo_property_based_testing()
    await demo.demo_coverage_analysis()
    await demo.demo_batch_generation()
    await demo.demo_test_export()
    
    print("\n" + "="*60)
    print("ALL DEMOS COMPLETED SUCCESSFULLY!")
    print("="*60)
    print("\nNext steps:")
    print("1. Try the CLI: python testing/testgen_cli.py --help")
    print("2. Generate tests: python testing/testgen_cli.py comprehensive filesystem")
    print("3. Read the docs: cat testing/testgen/README.md")

if __name__ == "__main__":
    print("MultiOS Test Generation Framework - Demo")
    print("This demo showcases the framework's capabilities.")
    
    try:
        asyncio.run(run_all_demos())
    except KeyboardInterrupt:
        print("\nDemo interrupted by user")
    except Exception as e:
        print(f"\nDemo failed with error: {e}")
        import traceback
        traceback.print_exc()