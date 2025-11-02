#!/usr/bin/env python3
"""
Scientific Computing Framework - Test and Demonstration Script
=============================================================

This script tests all major components of the scientific computing framework
to ensure everything is working correctly.

Usage:
    python test_framework.py
    python test_framework.py --quick    # Run quick tests only
    python test_framework.py --verbose # Detailed output

Author: Scientific Computing Education Team
"""

import sys
import os
import time
import traceback
import numpy as np

# Add current directory to path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

def test_numerical_framework():
    """Test numerical computing framework."""
    print("\nTesting Numerical Computing Framework...")
    
    try:
        from core.numerical_framework import (
            MatrixOperations, Optimization, FFTOperations, 
            DifferentialEquations, StatisticalComputations
        )
        
        # Test matrix operations
        A = np.array([[3, 1], [1, 2]], dtype=float)
        b = np.array([9, 8], dtype=float)
        solution = MatrixOperations.solve_linear_system(A, b)
        assert len(solution) == 2, "Matrix solution incorrect"
        
        # Test optimization
        def test_func(x): return x[0]**2 + x[1]**2
        def test_grad(x): return np.array([2*x[0], 2*x[1]])
        optimum, history = Optimization.gradient_descent(
            test_func, test_grad, np.array([1.0, 1.0]), 
            learning_rate=0.1, max_iterations=10
        )
        assert len(history) <= 10, "Optimization failed"
        
        # Test FFT
        signal = np.sin(2*np.pi*np.arange(100)/10)
        fft_result = FFTOperations.fft_1d(signal)
        assert len(fft_result) >= 100, "FFT failed"
        
        print("✓ Numerical framework tests passed")
        return True
        
    except Exception as e:
        print(f"✗ Numerical framework test failed: {e}")
        traceback.print_exc()
        return False


def test_simulations():
    """Test scientific simulation engines."""
    print("\nTesting Scientific Simulation Engines...")
    
    try:
        from simulations.scientific_engines import (
            PhysicsSimulation, ChemistrySimulation, BiologySimulation
        )
        
        # Test physics simulation
        physics = PhysicsSimulation()
        result = physics.newtonian_mechanics(
            mass=1.0, force=10.0, total_time=1.0, dt=0.1
        )
        assert 'position' in result, "Physics simulation failed"
        assert len(result['position']) > 0, "Physics simulation empty"
        
        # Test chemistry simulation
        chemistry = ChemistrySimulation()
        result = chemistry.reaction_kinetics_1st_order(
            rate_constant=0.1, initial_concentration=1.0, total_time=5.0
        )
        assert 'concentration_A' in result, "Chemistry simulation failed"
        
        # Test biology simulation
        biology = BiologySimulation()
        result = biology.predator_prey_model(
            prey_growth=1.0, predation_rate=0.1,
            predator_efficiency=0.1, predator_death=0.5,
            initial_prey=10.0, initial_predators=1.0,
            total_time=5.0
        )
        assert 'prey' in result, "Biology simulation failed"
        
        print("✓ Simulation engines tests passed")
        return True
        
    except Exception as e:
        print(f"✗ Simulation engines test failed: {e}")
        traceback.print_exc()
        return False


def test_data_analysis():
    """Test data analysis and visualization tools."""
    print("\nTesting Data Analysis and Visualization Tools...")
    
    try:
        from data_analysis.visualization_tools import (
            DataAnalysis, TimeSeriesAnalysis, SignalProcessing
        )
        
        # Test statistical analysis
        data = np.random.normal(100, 15, 1000)
        stats = DataAnalysis.descriptive_statistics(data, print_output=False)
        assert 'mean' in stats, "Statistical analysis failed"
        
        # Test time series analysis
        ts_data = np.random.randn(100) + np.sin(2*np.pi*np.arange(100)/20)
        trend = TimeSeriesAnalysis.trend_analysis(ts_data)
        assert 'slope' in trend, "Time series analysis failed"
        
        # Test signal processing
        signal = np.sin(2*np.pi*np.arange(100)/10) + 0.1*np.random.randn(100)
        filtered = SignalProcessing.filter_lowpass(signal, cutoff_freq=1.0, sampling_rate=10.0)
        assert len(filtered) == len(signal), "Signal processing failed"
        
        print("✓ Data analysis tests passed")
        return True
        
    except Exception as e:
        print(f"✗ Data analysis test failed: {e}")
        traceback.print_exc()
        return False


def test_hpc_examples():
    """Test high-performance computing examples."""
    print("\nTesting High-Performance Computing Examples...")
    
    try:
        from hpc.examples import HPCPatterns, ParallelNumericalMethods
        
        # Test HPC patterns
        hpc = HPCPatterns()
        result = hpc.vectorized_vs_loop(size=10000)
        assert 'speedup' in result, "HPC benchmark failed"
        
        # Test parallel numerical integration
        def test_func(x): return x**2 + 1
        result = ParallelNumericalMethods.parallel_numerical_integration(
            test_func, 0, 1, 1000, num_processes=1
        )
        assert isinstance(result, float), "Parallel integration failed"
        
        print("✓ HPC examples tests passed")
        return True
        
    except Exception as e:
        print(f"✗ HPC examples test failed: {e}")
        traceback.print_exc()
        return False


def test_ml_algorithms():
    """Test machine learning algorithms from scratch."""
    print("\nTesting Machine Learning Algorithms...")
    
    try:
        from ml_algorithms.from_scratch import (
            LinearRegression, LogisticRegression, KMeansClustering, 
            PrincipalComponentAnalysis, ModelEvaluation
        )
        
        # Generate test data
        np.random.seed(42)
        X_reg = np.random.randn(100, 2)
        y_reg = 2 * X_reg[:, 0] + 0.5 * X_reg[:, 1] + 0.1 * np.random.randn(100)
        
        # Test linear regression
        lr = LinearRegression(learning_rate=0.01, max_iterations=100)
        lr.fit(X_reg, y_reg)
        predictions = lr.predict(X_reg)
        assert len(predictions) == len(y_reg), "Linear regression failed"
        
        # Test logistic regression
        X_clf = np.random.randn(100, 3)
        y_clf = ((X_clf[:, 0] + X_clf[:, 1] + X_clf[:, 2]) > 0).astype(int)
        
        log_reg = LogisticRegression(learning_rate=0.1, max_iterations=100)
        log_reg.fit(X_clf, y_clf)
        clf_predictions = log_reg.predict(X_clf)
        assert len(clf_predictions) == len(y_clf), "Logistic regression failed"
        
        # Test K-means
        X_cluster = np.random.randn(100, 2)
        kmeans = KMeansClustering(k=3, random_state=42)
        labels = kmeans.fit_predict(X_cluster)
        assert len(labels) == len(X_cluster), "K-means failed"
        
        # Test PCA
        pca = PrincipalComponentAnalysis(n_components=2)
        X_pca = pca.fit_transform(X_clf)
        assert X_pca.shape[1] == 2, "PCA failed"
        
        # Test model evaluation
        accuracy = ModelEvaluation.accuracy_score(y_clf, clf_predictions)
        assert 0 <= accuracy <= 1, "Model evaluation failed"
        
        print("✓ Machine learning tests passed")
        return True
        
    except Exception as e:
        print(f"✗ Machine learning test failed: {e}")
        traceback.print_exc()
        return False


def test_bioinformatics():
    """Test bioinformatics tools."""
    print("\nTesting Bioinformatics Tools...")
    
    try:
        from bioinformatics.tools import (
            SequenceAnalysis, SequenceAlignment, Phylogenetics, 
            MolecularEvolution, GenomeAnalysis
        )
        
        # Test sequence analysis
        dna_seq = "ATGCGTACGTTAGCTAGCTAGCTAGCGATCGATCG"
        gc_content = SequenceAnalysis.gc_content(dna_seq)
        assert 0 <= gc_content <= 1, "GC content calculation failed"
        
        rev_comp = SequenceAnalysis.reverse_complement(dna_seq)
        assert len(rev_comp) == len(dna_seq), "Reverse complement failed"
        
        # Test sequence alignment
        seq1 = "ACGTACGT"
        seq2 = "ACGTCGT"
        alignment = SequenceAlignment.needleman_wunsch(seq1, seq2)
        assert 'score' in alignment, "Sequence alignment failed"
        
        # Test phylogenetics
        sequences = ["ACGTACGT", "ACGTACGA", "ACGTACGG", "ACGTACTT"]
        distance_matrix = Phylogenetics.calculate_distance_matrix(sequences)
        assert distance_matrix.shape == (4, 4), "Phylogenetics failed"
        
        # Test molecular evolution
        ancestral = "ATG" * 10
        evolved = MolecularEvolution.generate_sequences_with_substitution(
            n_sequences=2, sequence_length=30, substitution_rate=0.01,
            ancestral_sequence=ancestral
        )
        assert len(evolved) == 2, "Molecular evolution failed"
        
        # Test genome analysis
        genome = "ATGC" * 100
        kmer_freq = GenomeAnalysis.kmer_frequency(genome, k=4)
        assert len(kmer_freq) > 0, "Genome analysis failed"
        
        print("✓ Bioinformatics tests passed")
        return True
        
    except Exception as e:
        print(f"✗ Bioinformatics test failed: {e}")
        traceback.print_exc()
        return False


def test_curriculum():
    """Test curriculum framework."""
    print("\nTesting Curriculum Framework...")
    
    try:
        from curriculum.curriculum_framework import (
            create_comprehensive_curriculum, create_sample_projects,
            LearningObjective, Exercise
        )
        
        # Test curriculum creation
        modules = create_comprehensive_curriculum()
        assert len(modules) > 0, "No curriculum modules created"
        
        # Test project creation
        projects = create_sample_projects()
        assert len(projects) > 0, "No projects created"
        
        # Test exercise creation
        exercise = Exercise(
            title="Test Exercise",
            description="A test exercise",
            difficulty="beginner",
            starter_code="print('Hello')",
            solution="print('Hello')",
            hints=["Use print()"],
            learning_points=["Basic Python"]
        )
        assert exercise.title == "Test Exercise", "Exercise creation failed"
        
        print("✓ Curriculum framework tests passed")
        return True
        
    except Exception as e:
        print(f"✗ Curriculum framework test failed: {e}")
        traceback.print_exc()
        return False


def run_all_tests():
    """Run all component tests."""
    print("=" * 60)
    print("SCIENTIFIC COMPUTING FRAMEWORK - COMPREHENSIVE TESTS")
    print("=" * 60)
    
    start_time = time.time()
    
    tests = [
        ("Numerical Framework", test_numerical_framework),
        ("Simulation Engines", test_simulations),
        ("Data Analysis Tools", test_data_analysis),
        ("HPC Examples", test_hpc_examples),
        ("Machine Learning", test_ml_algorithms),
        ("Bioinformatics", test_bioinformatics),
        ("Curriculum", test_curriculum)
    ]
    
    results = []
    
    for test_name, test_func in tests:
        print(f"\nRunning {test_name} tests...")
        try:
            result = test_func()
            results.append((test_name, result))
        except Exception as e:
            print(f"✗ {test_name} tests failed with exception: {e}")
            results.append((test_name, False))
    
    # Summary
    total_time = time.time() - start_time
    passed = sum(1 for _, result in results if result)
    total = len(results)
    
    print("\n" + "=" * 60)
    print("TEST SUMMARY")
    print("=" * 60)
    
    for test_name, result in results:
        status = "PASS" if result else "FAIL"
        print(f"{test_name:.<40} {status}")
    
    print("-" * 60)
    print(f"Total Tests: {total}")
    print(f"Passed: {passed}")
    print(f"Failed: {total - passed}")
    print(f"Success Rate: {passed/total*100:.1f}%")
    print(f"Total Time: {total_time:.2f} seconds")
    
    if passed == total:
        print("\n🎉 All tests passed! Framework is ready to use.")
        return True
    else:
        print(f"\n⚠️  {total - passed} test(s) failed. Please check the errors above.")
        return False


def run_quick_test():
    """Run a quick test of key components."""
    print("=" * 60)
    print("SCIENTIFIC COMPUTING FRAMEWORK - QUICK TEST")
    print("=" * 60)
    
    try:
        print("\n1. Testing NumPy import...")
        import numpy as np
        print("✓ NumPy available")
        
        print("\n2. Testing basic matrix operations...")
        A = np.array([[1, 2], [3, 4]])
        det = np.linalg.det(A)
        print(f"✓ Matrix determinant: {det}")
        
        print("\n3. Testing core framework import...")
        from core.numerical_framework import MatrixOperations
        solution = MatrixOperations.solve_linear_system(A.astype(float), np.array([5.0, 6.0]))
        print(f"✓ Linear system solved: {solution}")
        
        print("\n4. Testing simulation engines...")
        from simulations.scientific_engines import PhysicsSimulation
        physics = PhysicsSimulation()
        result = physics.newtonian_mechanics(mass=1.0, force=1.0, total_time=0.1)
        print(f"✓ Physics simulation: {len(result['position'])} data points")
        
        print("\n5. Testing ML algorithms...")
        from ml_algorithms.from_scratch import LinearRegression
        X = np.random.randn(10, 2)
        y = np.random.randn(10)
        lr = LinearRegression(max_iterations=10)
        lr.fit(X, y)
        pred = lr.predict(X)
        print(f"✓ Linear regression: {len(pred)} predictions")
        
        print("\n" + "=" * 60)
        print("✓ Quick test completed successfully!")
        print("Framework is ready for use.")
        print("=" * 60)
        return True
        
    except Exception as e:
        print(f"\n✗ Quick test failed: {e}")
        traceback.print_exc()
        return False


def main():
    """Main test function."""
    import argparse
    
    parser = argparse.ArgumentParser(description="Test the scientific computing framework")
    parser.add_argument('--quick', action='store_true', help='Run quick test only')
    parser.add_argument('--verbose', action='store_true', help='Verbose output')
    
    args = parser.parse_args()
    
    if args.quick:
        success = run_quick_test()
    else:
        success = run_all_tests()
    
    return 0 if success else 1


if __name__ == "__main__":
    exit_code = main()
    sys.exit(exit_code)