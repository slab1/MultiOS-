"""
High-Performance Computing Examples for Scientific Computing Education
====================================================================

This module provides HPC examples demonstrating:
- Parallel computing patterns
- Vectorization techniques
- Memory optimization strategies
- GPU computing concepts
- Distributed computing models

Author: Scientific Computing Education Team
"""

import numpy as np
import multiprocessing as mp
from concurrent.futures import ProcessPoolExecutor, ThreadPoolExecutor
from typing import Callable, Tuple, List, Union
import time
import threading
from queue import Queue
import sys


class HPCPatterns:
    """High-Performance Computing patterns and implementations."""
    
    def __init__(self, num_threads: int = None):
        """Initialize HPC patterns with thread configuration."""
        self.num_threads = num_threads or mp.cpu_count()
        self.lock = threading.Lock()
    
    def vectorized_vs_loop(self, size: int = 1000000) -> Dict:
        """
        Compare vectorized operations vs loops for performance.
        
        Args:
            size: Size of arrays to process
            
        Returns:
            Dictionary with timing results
        """
        print(f"Vectorization Benchmark - Array Size: {size:,}")
        print("=" * 50)
        
        # Generate test data
        x = np.random.randn(size)
        y = np.random.randn(size)
        
        # Method 1: Loop-based computation
        start_time = time.time()
        result_loop = np.zeros(size)
        for i in range(size):
            result_loop[i] = np.sqrt(x[i]**2 + y[i]**2)
        loop_time = time.time() - start_time
        
        # Method 2: Vectorized computation
        start_time = time.time()
        result_vectorized = np.sqrt(x**2 + y**2)
        vectorized_time = time.time() - start_time
        
        # Verify results are equivalent
        max_diff = np.max(np.abs(result_loop - result_vectorized))
        speedup = loop_time / vectorized_time
        
        result = {
            'size': size,
            'loop_time': loop_time,
            'vectorized_time': vectorized_time,
            'speedup': speedup,
            'max_difference': max_diff,
            'correct': max_diff < 1e-10
        }
        
        print(f"Loop time:        {loop_time:.4f} seconds")
        print(f"Vectorized time:  {vectorized_time:.4f} seconds")
        print(f"Speedup:          {speedup:.2f}x")
        print(f"Max difference:   {max_diff:.2e}")
        print(f"Results match:    {result['correct']}")
        
        return result
    
    def parallel_matrix_multiplication(self, A: np.ndarray, B: np.ndarray,
                                     use_multiprocessing: bool = False) -> np.ndarray:
        """
        Parallel matrix multiplication implementation.
        
        Args:
            A: First matrix (m x k)
            B: Second matrix (k x n)
            use_multiprocessing: Whether to use multiprocessing
            
        Returns:
            Result matrix (m x n)
        """
        m, k = A.shape
        k2, n = B.shape
        
        if k != k2:
            raise ValueError(f"Incompatible matrix dimensions: {A.shape} and {B.shape}")
        
        if not use_multiprocessing:
            # Sequential implementation
            C = np.zeros((m, n))
            for i in range(m):
                for j in range(n):
                    for l in range(k):
                        C[i, j] += A[i, l] * B[l, j]
            return C
        
        # Parallel implementation
        def compute_row(i):
            row_result = np.zeros(n)
            for j in range(n):
                for l in range(k):
                    row_result[j] += A[i, l] * B[l, j]
            return row_result
        
        # Use multiprocessing
        with ProcessPoolExecutor(max_workers=self.num_threads) as executor:
            results = list(executor.map(compute_row, range(m)))
        
        return np.array(results)
    
    def parallel_reduction(self, data: np.ndarray, operation: str = 'sum') -> Union[float, int]:
        """
        Parallel reduction operation (sum, max, min, etc.).
        
        Args:
            data: Input data array
            operation: Reduction operation ('sum', 'max', 'min', 'mean')
            
        Returns:
            Reduced value
        """
        chunk_size = len(data) // self.num_threads
        
        def reduce_chunk(chunk):
            if operation == 'sum':
                return np.sum(chunk)
            elif operation == 'max':
                return np.max(chunk)
            elif operation == 'min':
                return np.min(chunk)
            elif operation == 'mean':
                return np.mean(chunk)
            else:
                raise ValueError(f"Unsupported operation: {operation}")
        
        # Divide data into chunks
        chunks = []
        for i in range(0, len(data), chunk_size):
            chunk = data[i:i + chunk_size]
            if len(chunk) > 0:
                chunks.append(chunk)
        
        # Process chunks in parallel
        with ThreadPoolExecutor(max_workers=self.num_threads) as executor:
            chunk_results = list(executor.map(reduce_chunk, chunks))
        
        # Combine chunk results
        if operation == 'sum':
            return sum(chunk_results)
        elif operation == 'mean':
            return sum(chunk_results) / len(chunk_results)
        elif operation in ['max', 'min']:
            from functools import reduce
            return reduce(operation, chunk_results)
    
    def cache_aware_loop_optimization(self, A: np.ndarray, B: np.ndarray) -> np.ndarray:
        """
        Cache-aware matrix multiplication for better memory locality.
        
        This implements blocked matrix multiplication to improve cache performance.
        
        Args:
            A: First matrix (m x k)
            B: Second matrix (k x n)
            
        Returns:
            Result matrix (m x n)
        """
        m, k = A.shape
        k2, n = B.shape
        
        if k != k2:
            raise ValueError(f"Incompatible matrix dimensions: {A.shape} and {B.shape}")
        
        # Block size optimized for cache (typically 32-64)
        block_size = 32
        
        C = np.zeros((m, n))
        
        # Blocked matrix multiplication
        for i in range(0, m, block_size):
            for j in range(0, n, block_size):
                for l in range(0, k, block_size):
                    # Process blocks
                    i_end = min(i + block_size, m)
                    j_end = min(j + block_size, n)
                    l_end = min(l + block_size, k)
                    
                    # Multiply blocks
                    for ii in range(i, i_end):
                        for jj in range(j, j_end):
                            for ll in range(l, l_end):
                                C[ii, jj] += A[ii, ll] * B[ll, jj]
        
        return C
    
    def memory_layout_optimization(self, A: np.ndarray) -> np.ndarray:
        """
        Demonstrate memory layout optimization (C vs Fortran ordering).
        
        Args:
            A: Input array
            
        Returns:
            Optimized array with different memory layout
        """
        print("Memory Layout Optimization Demo")
        print("=" * 35)
        
        # Get original statistics
        original = A.copy()
        print(f"Original array shape: {A.shape}")
        print(f"Original strides: {A.strides}")
        print(f"Original flags: {A.flags}")
        
        # Create Fortran-ordered copy
        fortran_order = A.copy(order='F')
        print(f"\nFortran-order array shape: {fortran_order.shape}")
        print(f"Fortran-order strides: {fortran_order.strides}")
        print(f"Fortran-order flags: {fortran_order.flags}")
        
        # Demonstrate access patterns
        print("\nAccess Pattern Performance:")
        
        # Row-wise access (good for C-order)
        start_time = time.time()
        for i in range(100):
            for j in range(A.shape[1]):
                _ = A[i, j]
        c_order_time = time.time() - start_time
        
        # Column-wise access (good for Fortran-order)
        start_time = time.time()
        for i in range(100):
            for j in range(A.shape[0]):
                _ = A[j, i]
        f_order_time = time.time() - start_time
        
        print(f"Row-wise access (C-order): {c_order_time:.4f}s")
        print(f"Column-wise access (F-order): {f_order_time:.4f}s")
        
        return fortran_order


class ParallelNumericalMethods:
    """Parallel implementations of numerical methods."""
    
    @staticmethod
    def parallel_numerical_integration(func: Callable, a: float, b: float, 
                                     n_subintervals: int = 100000,
                                     num_processes: int = None) -> float:
        """
        Parallel numerical integration using trapezoidal rule.
        
        Args:
            func: Function to integrate
            a: Lower limit
            b: Upper limit
            n_subintervals: Number of subintervals
            num_processes: Number of parallel processes
            
        Returns:
            Approximate integral value
        """
        if num_processes is None:
            num_processes = mp.cpu_count()
        
        dx = (b - a) / n_subintervals
        
        def integrate_chunk(start_idx: int, end_idx: int) -> float:
            """Integrate function over a chunk of subintervals."""
            result = 0.0
            for i in range(start_idx, end_idx):
                x1 = a + i * dx
                x2 = a + (i + 1) * dx
                result += 0.5 * dx * (func(x1) + func(x2))
            return result
        
        # Divide work among processes
        chunk_size = n_subintervals // num_processes
        ranges = [(i * chunk_size, min((i + 1) * chunk_size, n_subintervals)) 
                 for i in range(num_processes)]
        
        with ProcessPoolExecutor(max_workers=num_processes) as executor:
            results = list(executor.map(lambda r: integrate_chunk(*r), ranges))
        
        return sum(results)
    
    @staticmethod
    def parallel_euler_ode(self, f: Callable, y0: float, t0: float, 
                          t_end: float, h: float, num_processes: int = None) -> Tuple[np.ndarray, np.ndarray]:
        """
        Parallel ODE solving using domain decomposition.
        
        Args:
            f: ODE function f(t, y)
            y0: Initial condition
            t0: Initial time
            t_end: Final time
            h: Step size
            num_processes: Number of parallel processes
            
        Returns:
            Tuple of (time_points, solution)
        """
        if num_processes is None:
            num_processes = mp.cpu_count()
        
        # Create time grid
        time_points = np.arange(t0, t_end + h, h)
        
        # Divide time domain among processes
        segment_length = len(time_points) // num_processes
        segments = [(time_points[i*segment_length:(i+1)*segment_length], y0) 
                   for i in range(num_processes)]
        
        def solve_segment(segment_info):
            """Solve ODE on a time segment."""
            segment_times, y_start = segment_info
            segment_solution = np.zeros(len(segment_times))
            segment_solution[0] = y_start
            
            for i in range(1, len(segment_times)):
                dt = segment_times[i] - segment_times[i-1]
                segment_solution[i] = segment_solution[i-1] + dt * f(segment_times[i-1], segment_solution[i-1])
            
            return segment_times, segment_solution
        
        # Solve segments in parallel
        with ProcessPoolExecutor(max_workers=num_processes) as executor:
            segment_results = list(executor.map(solve_segment, segments))
        
        # Combine results
        all_times = np.concatenate([result[0] for result in segment_results])
        all_solutions = np.concatenate([result[1] for result in segment_results])
        
        return all_times, all_solutions
    
    @staticmethod
    def parallel_monte_carlo(func: Callable, bounds: List[Tuple[float, float]], 
                           n_samples: int = 1000000,
                           num_processes: int = None) -> float:
        """
        Parallel Monte Carlo integration.
        
        Args:
            func: Function to integrate
            bounds: List of (min, max) bounds for each dimension
            n_samples: Number of samples
            num_processes: Number of parallel processes
            
        Returns:
            Monte Carlo estimate of the integral
        """
        if num_processes is None:
            num_processes = mp.cpu_count()
        
        samples_per_process = n_samples // num_processes
        
        def monte_carlo_chunk(n_samples_chunk: int) -> float:
            """Perform Monte Carlo integration on a chunk of samples."""
            dimensions = len(bounds)
            
            # Generate random samples
            samples = np.random.uniform(
                low=[b[0] for b in bounds],
                high=[b[1] for b in bounds],
                size=(n_samples_chunk, dimensions)
            )
            
            # Calculate function values
            values = np.array([func(*sample) for sample in samples])
            
            # Calculate volume
            volume = np.prod([b[1] - b[0] for b in bounds])
            
            # Monte Carlo estimate for this chunk
            return volume * np.mean(values)
        
        # Process chunks in parallel
        with ProcessPoolExecutor(max_workers=num_processes) as executor:
            chunk_estimates = list(executor.map(monte_carlo_chunk, 
                                              [samples_per_process] * num_processes))
        
        # Combine estimates
        return np.mean(chunk_estimates)


class PerformanceBenchmarking:
    """Tools for benchmarking and performance analysis."""
    
    def __init__(self):
        self.results = {}
    
    def benchmark_function(self, func: Callable, *args, **kwargs) -> Dict:
        """
        Benchmark a function with multiple runs.
        
        Args:
            func: Function to benchmark
            *args: Arguments for the function
            **kwargs: Keyword arguments for the function
            
        Returns:
            Dictionary with timing statistics
        """
        # Warm up
        for _ in range(3):
            _ = func(*args, **kwargs)
        
        # Measure execution time
        n_runs = 10
        times = []
        
        for _ in range(n_runs):
            start_time = time.time()
            result = func(*args, **kwargs)
            end_time = time.time()
            times.append(end_time - start_time)
        
        times = np.array(times)
        
        result_dict = {
            'function_name': func.__name__,
            'mean_time': np.mean(times),
            'median_time': np.median(times),
            'std_time': np.std(times),
            'min_time': np.min(times),
            'max_time': np.max(times),
            'result': result
        }
        
        print(f"Benchmark: {func.__name__}")
        print(f"Mean time:     {result_dict['mean_time']:.6f}s")
        print(f"Median time:   {result_dict['median_time']:.6f}s")
        print(f"Std deviation: {result_dict['std_time']:.6f}s")
        print(f"Min time:      {result_dict['min_time']:.6f}s")
        print(f"Max time:      {result_dict['max_time']:.6f}s")
        
        return result_dict
    
    def memory_usage(self, func: Callable, *args, **kwargs) -> Dict:
        """
        Measure memory usage of a function.
        
        Args:
            func: Function to measure
            *args: Arguments for the function
            **kwargs: Keyword arguments for the function
            
        Returns:
            Dictionary with memory usage statistics
        """
        import psutil
        import os
        
        process = psutil.Process(os.getpid())
        
        # Memory before
        memory_before = process.memory_info().rss / 1024 / 1024  # MB
        
        # Execute function
        result = func(*args, **kwargs)
        
        # Memory after
        memory_after = process.memory_info().rss / 1024 / 1024  # MB
        
        memory_used = memory_after - memory_before
        
        result_dict = {
            'memory_before_mb': memory_before,
            'memory_after_mb': memory_after,
            'memory_used_mb': memory_used,
            'result': result
        }
        
        print(f"Memory Usage: {func.__name__}")
        print(f"Before: {memory_before:.2f} MB")
        print(f"After:  {memory_after:.2f} MB")
        print(f"Used:   {memory_used:.2f} MB")
        
        return result_dict
    
    def scalability_analysis(self, func: Callable, sizes: List[int], 
                           *args, **kwargs) -> Dict:
        """
        Analyze how function performance scales with input size.
        
        Args:
            func: Function to analyze
            sizes: List of input sizes to test
            *args: Arguments for the function
            **kwargs: Keyword arguments for the function
            
        Returns:
            Dictionary with scalability analysis
        """
        results = []
        
        for size in sizes:
            # Modify args to include size (assuming first arg is data size)
            modified_args = (size,) + args[1:] if args else (size,)
            
            benchmark_result = self.benchmark_function(func, *modified_args, **kwargs)
            
            results.append({
                'size': size,
                'time': benchmark_result['mean_time'],
                'std': benchmark_result['std_time']
            })
        
        # Calculate scaling exponent (assuming power law: time ∝ size^α)
        sizes_array = np.array([r['size'] for r in results])
        times_array = np.array([r['time'] for r in results])
        
        # Fit power law: log(time) = log(a) + α*log(size)
        log_sizes = np.log(sizes_array)
        log_times = np.log(times_array)
        
        # Linear regression in log space
        alpha = np.sum((log_sizes - np.mean(log_sizes)) * 
                      (log_times - np.mean(log_times))) / \
                np.sum((log_sizes - np.mean(log_sizes))**2)
        
        scaling_result = {
            'results': results,
            'scaling_exponent': alpha,
            'interpretation': self._interpret_scaling(alpha)
        }
        
        print("\nScalability Analysis")
        print("=" * 20)
        print(f"Scaling exponent (α): {alpha:.3f}")
        print(f"Interpretation: {scaling_result['interpretation']}")
        
        return scaling_result
    
    def _interpret_scaling(self, alpha: float) -> str:
        """Interpret scaling exponent."""
        if alpha < 1.1:
            return "Linear scaling (O(n))"
        elif alpha < 1.9:
            return "Nearly linear (better than O(n log n))"
        elif alpha < 2.1:
            return "Linearithmic scaling (O(n log n))"
        elif alpha < 2.9:
            return "Quadratic scaling (O(n²))"
        else:
            return "Worse than quadratic (O(n²+))"


class DistributedComputing:
    """Simple distributed computing concepts and implementations."""
    
    @staticmethod
    def master_worker_pattern(data_chunks: List, worker_func: Callable,
                            num_workers: int = None) -> List:
        """
        Master-worker distributed computing pattern.
        
        Args:
            data_chunks: List of data chunks to process
            worker_func: Function to apply to each chunk
            num_workers: Number of worker processes
            
        Returns:
            List of results from all workers
        """
        if num_workers is None:
            num_workers = mp.cpu_count()
        
        print(f"Master-Worker Pattern: {len(data_chunks)} chunks, {num_workers} workers")
        
        with ProcessPoolExecutor(max_workers=num_workers) as executor:
            results = list(executor.map(worker_func, data_chunks))
        
        return results
    
    @staticmethod
    def map_reduce_implementation(mapper_func: Callable, reducer_func: Callable,
                                data: List, num_workers: int = None) -> any:
        """
        Simple Map-Reduce implementation.
        
        Args:
            mapper_func: Function to map over data
            reducer_func: Function to reduce mapped results
            data: Input data list
            num_workers: Number of parallel workers
            
        Returns:
            Reduced result
        """
        if num_workers is None:
            num_workers = mp.cpu_count()
        
        # Map phase
        with ProcessPoolExecutor(max_workers=num_workers) as executor:
            mapped_results = list(executor.map(mapper_func, data))
        
        # Shuffle and group (simplified)
        grouped_results = {}
        for key, value in mapped_results:
            if key not in grouped_results:
                grouped_results[key] = []
            grouped_results[key].append(value)
        
        # Reduce phase
        reduced_results = {}
        for key, values in grouped_results.items():
            reduced_results[key] = reducer_func(values)
        
        return reduced_results
    
    @staticmethod
    def fault_tolerant_computation(data_chunks: List, worker_func: Callable,
                                 max_retries: int = 3) -> List:
        """
        Fault-tolerant computation with retry logic.
        
        Args:
            data_chunks: List of data chunks
            worker_func: Function to apply to each chunk
            max_retries: Maximum number of retry attempts
            
        Returns:
            List of results (may contain None for failed chunks)
        """
        results = [None] * len(data_chunks)
        
        for i, chunk in enumerate(data_chunks):
            for attempt in range(max_retries):
                try:
                    result = worker_func(chunk)
                    results[i] = result
                    break  # Success, move to next chunk
                except Exception as e:
                    print(f"Attempt {attempt + 1} failed for chunk {i}: {e}")
                    if attempt == max_retries - 1:
                        print(f"All attempts failed for chunk {i}")
                        results[i] = None  # Mark as failed
        
        return results


def demo_hpc_examples():
    """Demonstrate HPC examples and benchmarks."""
    print("High-Performance Computing Examples")
    print("=" * 35)
    
    hpc = HPCPatterns()
    benchmark = PerformanceBenchmarking()
    
    # Example 1: Vectorization vs Loops
    print("\n1. Vectorization Benchmark:")
    vectorize_result = hpc.vectorized_vs_loop(1000000)
    
    # Example 2: Parallel Matrix Multiplication
    print("\n2. Parallel Matrix Multiplication:")
    size = 100
    A = np.random.randn(size, size)
    B = np.random.randn(size, size)
    
    print(f"Matrix size: {size}x{size}")
    
    # Sequential
    start_time = time.time()
    C_seq = hpc.parallel_matrix_multiplication(A, B, use_multiprocessing=False)
    seq_time = time.time() - start_time
    
    # Parallel
    start_time = time.time()
    C_par = hpc.parallel_matrix_multiplication(A, B, use_multiprocessing=True)
    par_time = time.time() - start_time
    
    print(f"Sequential time: {seq_time:.4f}s")
    print(f"Parallel time:   {par_time:.4f}s")
    print(f"Speedup:         {seq_time/par_time:.2f}x")
    print(f"Results match:   {np.allclose(C_seq, C_par)}")
    
    # Example 3: Memory Layout Optimization
    print("\n3. Memory Layout Optimization:")
    large_array = np.random.randn(1000, 1000)
    hpc.memory_layout_optimization(large_array)
    
    # Example 4: Numerical Integration
    print("\n4. Parallel Numerical Integration:")
    def test_function(x):
        return np.sin(x) * np.exp(-x)
    
    # Sequential integration
    start_time = time.time()
    result_seq = ParallelNumericalMethods.parallel_numerical_integration(
        test_function, 0, 10, 100000, num_processes=1)
    seq_time = time.time() - start_time
    
    # Parallel integration
    start_time = time.time()
    result_par = ParallelNumericalMethods.parallel_numerical_integration(
        test_function, 0, 10, 100000, num_processes=4)
    par_time = time.time() - start_time
    
    print(f"Sequential result: {result_seq:.6f} ({seq_time:.4f}s)")
    print(f"Parallel result:   {result_par:.6f} ({par_time:.4f}s)")
    print(f"Speedup:           {seq_time/par_time:.2f}x")
    
    # Example 5: Scalability Analysis
    print("\n5. Scalability Analysis:")
    def matrix_operation_test(n):
        A = np.random.randn(n, n)
        B = np.random.randn(n, n)
        return np.dot(A, B)
    
    sizes = [50, 100, 200, 400, 800]
    scalability_result = benchmark.scalability_analysis(
        matrix_operation_test, sizes, 50)
    
    # Example 6: Master-Worker Pattern
    print("\n6. Master-Worker Pattern:")
    data_chunks = [np.random.randn(1000) for _ in range(10)]
    
    def simple_analysis(chunk):
        return {
            'mean': np.mean(chunk),
            'std': np.std(chunk),
            'min': np.min(chunk),
            'max': np.max(chunk)
        }
    
    results = DistributedComputing.master_worker_pattern(data_chunks, simple_analysis)
    
    print(f"Processed {len(data_chunks)} chunks")
    print(f"First chunk result: {results[0]}")
    
    # Example 7: Map-Reduce
    print("\n7. Map-Reduce Pattern:")
    data = list(range(20))
    
    def mapper(x):
        return (x % 3, x)  # Group by x % 3
    
    def reducer(values):
        return sum(values)  # Sum all values in each group
    
    map_reduce_result = DistributedComputing.map_reduce_implementation(
        mapper, reducer, data)
    
    print(f"Map-Reduce result: {map_reduce_result}")


if __name__ == "__main__":
    demo_hpc_examples()