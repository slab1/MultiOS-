"""
Core Numerical Computing Framework for Scientific Computing Education
=====================================================================

This module provides a comprehensive framework for numerical computing including:
- Linear algebra operations
- Numerical optimization
- FFT and signal processing
- Differential equation solvers
- Statistical computations

Author: Scientific Computing Education Team
"""

import numpy as np
import math
from typing import Union, Callable, Tuple, Optional
from abc import ABC, abstractmethod


class MatrixOperations:
    """Comprehensive linear algebra operations for educational purposes."""
    
    @staticmethod
    def matrix_multiply(A: np.ndarray, B: np.ndarray) -> np.ndarray:
        """
        Perform matrix multiplication with detailed educational logging.
        
        Args:
            A: First matrix (m x n)
            B: Second matrix (n x p)
            
        Returns:
            Result matrix (m x p)
            
        Raises:
            ValueError: If matrix dimensions are incompatible
            
        Educational Note:
            Matrix multiplication requires that the number of columns in A
            equals the number of rows in B.
        """
        if A.shape[1] != B.shape[0]:
            raise ValueError(f"Incompatible matrix dimensions: {A.shape} and {B.shape}")
        
        # Ensure matrices are float type for consistent calculations
        A = A.astype(float)
        B = B.astype(float)
            
        m, n = A.shape
        n2, p = B.shape
        
        # Initialize result matrix
        C = np.zeros((m, p))
        
        # Standard matrix multiplication
        for i in range(m):
            for j in range(p):
                for k in range(n):
                    C[i, j] += A[i, k] * B[k, j]
                    
        return C
    
    @staticmethod
    def lu_decomposition(A: np.ndarray) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
        """
        Compute LU decomposition with partial pivoting.
        
        Args:
            A: Square matrix to decompose
            
        Returns:
            Tuple of (L, U, P) where P is the permutation matrix
            
        Educational Note:
            LU decomposition factors a matrix into lower (L) and upper (U)
            triangular matrices. Partial pivoting reorders rows to improve
            numerical stability.
        """
        n = A.shape[0]
        if A.shape[1] != n:
            raise ValueError("Matrix must be square for LU decomposition")
            
        # Initialize matrices
        L = np.eye(n)
        U = np.zeros((n, n))
        P = np.eye(n)  # Permutation matrix
        
        # Make a copy to work with
        A_copy = A.copy()
        
        for k in range(n):
            # Find pivot
            pivot_row = np.argmax(np.abs(A_copy[k:, k])) + k
            
            # Swap rows in A_copy and P
            if pivot_row != k:
                A_copy[[k, pivot_row]] = A_copy[[pivot_row, k]]
                P[[k, pivot_row]] = P[[pivot_row, k]]
                
            # Check for singular matrix
            if abs(A_copy[k, k]) < 1e-12:
                raise ValueError("Matrix is singular or nearly singular")
                
            # Gaussian elimination
            for i in range(k+1, n):
                factor = A_copy[i, k] / A_copy[k, k]
                L[i, k] = factor
                A_copy[i, k:] -= factor * A_copy[k, k:]
                
        U = A_copy
        return L, U, P
    
    @staticmethod
    def solve_linear_system(A: np.ndarray, b: np.ndarray) -> np.ndarray:
        """
        Solve linear system Ax = b using LU decomposition.
        
        Args:
            A: Coefficient matrix
            b: Right-hand side vector
            
        Returns:
            Solution vector x
        """
        if A.shape[0] != A.shape[1]:
            raise ValueError("Coefficient matrix must be square")
        if A.shape[0] != len(b):
            raise ValueError("Matrix and vector dimensions incompatible")
            
        # Perform LU decomposition
        L, U, P = MatrixOperations.lu_decomposition(A)
        
        # Solve Ly = Pb
        y = np.zeros(len(b))
        for i in range(len(b)):
            y[i] = b[i]
            for j in range(i):
                y[i] -= L[i, j] * y[j]
                
        # Solve Ux = y
        x = np.zeros(len(b))
        for i in range(len(b)-1, -1, -1):
            x[i] = y[i]
            for j in range(i+1, len(b)):
                x[i] -= U[i, j] * x[j]
            x[i] /= U[i, i]
            
        return x
    
    @staticmethod
    def eigen_decomposition_simple(A: np.ndarray, max_iterations: int = 1000, 
                                 tolerance: float = 1e-10) -> Tuple[np.ndarray, np.ndarray]:
        """
        Compute eigenvalues and eigenvectors using power iteration.
        
        Args:
            A: Square symmetric matrix
            max_iterations: Maximum number of iterations
            tolerance: Convergence tolerance
            
        Returns:
            Tuple of (eigenvalues, eigenvectors)
            
        Educational Note:
            Power iteration finds the dominant eigenvalue by repeatedly
            multiplying a vector by the matrix and normalizing.
        """
        n = A.shape[0]
        if not np.allclose(A, A.T):
            raise ValueError("Matrix must be symmetric for this algorithm")
            
        eigenvalues = np.zeros(n)
        eigenvectors = np.zeros((n, n))
        
        for k in range(n):
            # Start with random vector
            v = np.random.rand(n)
            v /= np.linalg.norm(v)
            
            for i in range(max_iterations):
                v_new = np.dot(A, v)
                v_new /= np.linalg.norm(v_new)
                
                if np.linalg.norm(v_new - v) < tolerance:
                    break
                v = v_new
                
            eigenvalues[k] = np.dot(v, np.dot(A, v))
            eigenvectors[:, k] = v
            
            # Deflate the matrix
            A = A - eigenvalues[k] * np.outer(v, v)
            
        return eigenvalues, eigenvectors


class Optimization:
    """Numerical optimization algorithms for educational purposes."""
    
    @staticmethod
    def gradient_descent(func: Callable, grad_func: Callable, 
                        initial_point: np.ndarray, learning_rate: float = 0.01,
                        max_iterations: int = 1000, tolerance: float = 1e-6) -> Tuple[np.ndarray, list]:
        """
        Gradient descent optimization algorithm.
        
        Args:
            func: Objective function to minimize
            grad_func: Gradient function
            initial_point: Starting point
            learning_rate: Step size
            max_iterations: Maximum iterations
            tolerance: Convergence tolerance
            
        Returns:
            Tuple of (optimal_point, convergence_history)
        """
        x = initial_point.copy()
        history = []
        
        for i in range(max_iterations):
            grad = grad_func(x)
            
            # Check for convergence
            if np.linalg.norm(grad) < tolerance:
                break
                
            # Update parameters
            x = x - learning_rate * grad
            
            # Record history
            history.append({
                'iteration': i,
                'point': x.copy(),
                'value': func(x),
                'gradient_norm': np.linalg.norm(grad)
            })
            
        return x, history
    
    @staticmethod
    def newton_method(func: Callable, grad_func: Callable, hess_func: Callable,
                     initial_point: np.ndarray, max_iterations: int = 100,
                     tolerance: float = 1e-6) -> Tuple[np.ndarray, list]:
        """
        Newton's method for optimization.
        
        Args:
            func: Objective function
            grad_func: Gradient function
            hess_func: Hessian function
            initial_point: Starting point
            max_iterations: Maximum iterations
            tolerance: Convergence tolerance
            
        Returns:
            Tuple of (optimal_point, convergence_history)
        """
        x = initial_point.copy()
        history = []
        
        for i in range(max_iterations):
            grad = grad_func(x)
            hess = hess_func(x)
            
            # Check for convergence
            if np.linalg.norm(grad) < tolerance:
                break
                
            try:
                # Solve H * delta = -grad
                delta = np.linalg.solve(hess, -grad)
                x = x + delta
            except np.linalg.LinAlgError:
                # Use gradient descent if Hessian is singular
                x = x - 0.01 * grad
                
            history.append({
                'iteration': i,
                'point': x.copy(),
                'value': func(x),
                'gradient_norm': np.linalg.norm(grad)
            })
            
        return x, history
    
    @staticmethod
    def simulated_annealing(func: Callable, bounds: list, 
                          initial_temp: float = 1000, cooling_rate: float = 0.95,
                          min_temp: float = 0.01, max_iterations: int = 10000) -> np.ndarray:
        """
        Simulated annealing optimization.
        
        Args:
            func: Objective function to minimize
            bounds: List of (min, max) bounds for each variable
            initial_temp: Starting temperature
            cooling_rate: Temperature reduction factor
            min_temp: Minimum temperature
            max_iterations: Maximum iterations per temperature
            
        Returns:
            Approximate optimal point
        """
        # Initialize
        current_point = np.array([np.random.uniform(low, high) for low, high in bounds])
        current_value = func(current_point)
        best_point = current_point.copy()
        best_value = current_value
        
        temperature = initial_temp
        
        while temperature > min_temp:
            for _ in range(max_iterations):
                # Generate neighbor solution
                neighbor = current_point.copy()
                for i in range(len(bounds)):
                    noise = np.random.normal(0, temperature / 10)
                    neighbor[i] += noise
                    # Keep within bounds
                    neighbor[i] = np.clip(neighbor[i], bounds[i][0], bounds[i][1])
                    
                neighbor_value = func(neighbor)
                
                # Accept or reject move
                if neighbor_value < current_value:
                    # Always accept improvement
                    current_point = neighbor
                    current_value = neighbor_value
                    
                    if neighbor_value < best_value:
                        best_point = neighbor.copy()
                        best_value = neighbor_value
                else:
                    # Accept with probability
                    delta = neighbor_value - current_value
                    probability = np.exp(-delta / temperature)
                    
                    if np.random.random() < probability:
                        current_point = neighbor
                        current_value = neighbor_value
                        
            # Cool down
            temperature *= cooling_rate
            
        return best_point


class FFTOperations:
    """Fast Fourier Transform and signal processing operations."""
    
    @staticmethod
    def fft_1d(signal: np.ndarray) -> np.ndarray:
        """
        Compute 1D Fast Fourier Transform.
        
        Args:
            signal: Input time-domain signal
            
        Returns:
            Complex frequency-domain representation
        """
        n = len(signal)
        
        # Base case
        if n <= 1:
            return signal
            
        # Check if n is power of 2
        if n & (n - 1) != 0:
            # Pad to next power of 2
            next_pow2 = 1 << (n - 1).bit_length()
            signal = np.pad(signal, (0, next_pow2 - n))
            n = len(signal)
            
        # Recursive FFT
        even = FFTOperations.fft_1d(signal[::2])
        odd = FFTOperations.fft_1d(signal[1::2])
        
        # Combine
        result = np.zeros(n, dtype=complex)
        for k in range(n // 2):
            t = np.exp(-2j * np.pi * k / n) * odd[k]
            result[k] = even[k] + t
            result[k + n // 2] = even[k] - t
            
        return result
    
    @staticmethod
    def ifft_1d(frequency_signal: np.ndarray) -> np.ndarray:
        """
        Compute inverse 1D Fast Fourier Transform.
        
        Args:
            frequency_signal: Input frequency-domain signal
            
        Returns:
            Real time-domain signal
        """
        n = len(frequency_signal)
        
        # Conjugate input
        conjugated = np.conjugate(frequency_signal)
        
        # Forward FFT
        result_conjugate = FFTOperations.fft_1d(conjugated)
        
        # Conjugate and scale result
        result = np.conjugate(result_conjugate) / n
        
        return np.real(result)
    
    @staticmethod
    def fft_2d(image: np.ndarray) -> np.ndarray:
        """
        Compute 2D Fast Fourier Transform.
        
        Args:
            image: 2D input array
            
        Returns:
            2D frequency-domain representation
        """
        rows, cols = image.shape
        
        # Apply FFT to each row
        temp = np.zeros_like(image, dtype=complex)
        for i in range(rows):
            temp[i, :] = FFTOperations.fft_1d(image[i, :])
            
        # Apply FFT to each column
        result = np.zeros_like(temp, dtype=complex)
        for j in range(cols):
            result[:, j] = FFTOperations.fft_1d(temp[:, j])
            
        return result
    
    @staticmethod
    def filter_frequency_domain(data: np.ndarray, cutoff_freq: float, 
                              filter_type: str = 'lowpass') -> np.ndarray:
        """
        Apply frequency domain filtering.
        
        Args:
            data: Input signal
            cutoff_freq: Cutoff frequency
            filter_type: Type of filter ('lowpass', 'highpass', 'bandpass')
            
        Returns:
            Filtered signal
        """
        # Compute FFT
        fft_data = FFTOperations.fft_1d(data)
        frequencies = np.fft.fftfreq(len(data))
        
        # Create filter
        filter_mask = np.ones(len(frequencies))
        
        if filter_type == 'lowpass':
            filter_mask[np.abs(frequencies) > cutoff_freq] = 0
        elif filter_type == 'highpass':
            filter_mask[np.abs(frequencies) < cutoff_freq] = 0
        elif filter_type == 'bandpass':
            filter_mask[np.abs(frequencies) < cutoff_freq[0]] = 0
            filter_mask[np.abs(frequencies) > cutoff_freq[1]] = 0
            
        # Apply filter
        filtered_fft = fft_data * filter_mask
        
        # Convert back to time domain
        return FFTOperations.ifft_1d(filtered_fft)


class DifferentialEquations:
    """Solvers for ordinary differential equations."""
    
    @staticmethod
    def euler_method(f: Callable, y0: float, t0: float, t_end: float, h: float) -> Tuple[np.ndarray, np.ndarray]:
        """
        Euler's method for solving ODEs.
        
        dy/dt = f(y, t), y(t0) = y0
        
        Args:
            f: Right-hand side function f(y, t)
            y0: Initial condition
            t0: Initial time
            t_end: Final time
            h: Step size
            
        Returns:
            Tuple of (time_points, solution_values)
        """
        n_steps = int((t_end - t0) / h) + 1
        t = np.linspace(t0, t_end, n_steps)
        y = np.zeros(n_steps)
        y[0] = y0
        
        for i in range(1, n_steps):
            y[i] = y[i-1] + h * f(y[i-1], t[i-1])
            
        return t, y
    
    @staticmethod
    def runge_kutta_4(f: Callable, y0: float, t0: float, t_end: float, h: float) -> Tuple[np.ndarray, np.ndarray]:
        """
        Fourth-order Runge-Kutta method for solving ODEs.
        
        Args:
            f: Right-hand side function f(y, t)
            y0: Initial condition
            t0: Initial time
            t_end: Final time
            h: Step size
            
        Returns:
            Tuple of (time_points, solution_values)
        """
        n_steps = int((t_end - t0) / h) + 1
        t = np.linspace(t0, t_end, n_steps)
        y = np.zeros(n_steps)
        y[0] = y0
        
        for i in range(1, n_steps):
            k1 = h * f(y[i-1], t[i-1])
            k2 = h * f(y[i-1] + k1/2, t[i-1] + h/2)
            k3 = h * f(y[i-1] + k2/2, t[i-1] + h/2)
            k4 = h * f(y[i-1] + k3, t[i-1] + h)
            
            y[i] = y[i-1] + (k1 + 2*k2 + 2*k3 + k4) / 6
            
        return t, y
    
    @staticmethod
    def solve_system_ode(f_system: Callable, y0: np.ndarray, t0: float, 
                        t_end: float, h: float) -> Tuple[np.ndarray, np.ndarray]:
        """
        Solve system of ODEs using RK4.
        
        Args:
            f_system: System of RHS functions f(y, t)
            y0: Initial conditions vector
            t0: Initial time
            t_end: Final time
            h: Step size
            
        Returns:
            Tuple of (time_points, solution_matrix)
        """
        n_steps = int((t_end - t0) / h) + 1
        t = np.linspace(t0, t_end, n_steps)
        n_vars = len(y0)
        y = np.zeros((n_steps, n_vars))
        y[0] = y0
        
        for i in range(1, n_steps):
            # RK4 for system
            k1 = h * f_system(y[i-1], t[i-1])
            k2 = h * f_system(y[i-1] + k1/2, t[i-1] + h/2)
            k3 = h * f_system(y[i-1] + k2/2, t[i-1] + h/2)
            k4 = h * f_system(y[i-1] + k3, t[i-1] + h)
            
            y[i] = y[i-1] + (k1 + 2*k2 + 2*k3 + k4) / 6
            
        return t, y


class StatisticalComputations:
    """Statistical and probability operations for scientific computing."""
    
    @staticmethod
    def linear_regression(x: np.ndarray, y: np.ndarray) -> Tuple[float, float, float]:
        """
        Perform linear regression y = ax + b.
        
        Args:
            x: Independent variable data
            y: Dependent variable data
            
        Returns:
            Tuple of (slope, intercept, r_squared)
        """
        n = len(x)
        if len(y) != n:
            raise ValueError("x and y must have same length")
            
        # Calculate statistics
        x_mean = np.mean(x)
        y_mean = np.mean(y)
        
        # Calculate slope and intercept
        numerator = np.sum((x - x_mean) * (y - y_mean))
        denominator = np.sum((x - x_mean) ** 2)
        
        if denominator == 0:
            raise ValueError("x has no variance")
            
        slope = numerator / denominator
        intercept = y_mean - slope * x_mean
        
        # Calculate R-squared
        y_pred = slope * x + intercept
        ss_res = np.sum((y - y_pred) ** 2)
        ss_tot = np.sum((y - y_mean) ** 2)
        
        r_squared = 1 - (ss_res / ss_tot) if ss_tot != 0 else 0
        
        return slope, intercept, r_squared
    
    @staticmethod
    def principle_component_analysis(data: np.ndarray, n_components: int) -> Tuple[np.ndarray, np.ndarray]:
        """
        Perform Principal Component Analysis.
        
        Args:
            data: Input data matrix (samples x features)
            n_components: Number of principal components to extract
            
        Returns:
            Tuple of (transformed_data, components)
        """
        # Center the data
        data_centered = data - np.mean(data, axis=0)
        
        # Compute covariance matrix
        cov_matrix = np.cov(data_centered, rowvar=False)
        
        # Compute eigenvalues and eigenvectors
        eigenvalues, eigenvectors = MatrixOperations.eigen_decomposition_simple(cov_matrix)
        
        # Sort by eigenvalues (descending)
        indices = np.argsort(eigenvalues)[::-1]
        eigenvalues = eigenvalues[indices]
        eigenvectors = eigenvectors[:, indices]
        
        # Select top components
        components = eigenvectors[:, :n_components]
        transformed_data = np.dot(data_centered, components)
        
        return transformed_data, components
    
    @staticmethod
    def monte_carlo_integration(func: Callable, bounds: list, n_samples: int = 10000) -> float:
        """
        Monte Carlo integration for high-dimensional functions.
        
        Args:
            func: Function to integrate
            bounds: List of (min, max) bounds for each dimension
            n_samples: Number of Monte Carlo samples
            
        Returns:
            Approximate integral value
        """
        n_dim = len(bounds)
        
        # Generate random samples
        samples = np.random.uniform(
            low=[b[0] for b in bounds],
            high=[b[1] for b in bounds],
            size=(n_samples, n_dim)
        )
        
        # Calculate function values
        values = np.array([func(*sample) for sample in samples])
        
        # Calculate volume
        volume = np.prod([b[1] - b[0] for b in bounds])
        
        # Monte Carlo estimate
        integral = volume * np.mean(values)
        
        return integral
    
    @staticmethod
    def bayesian_update(prior: np.ndarray, likelihood: np.ndarray, evidence: float = 1.0) -> np.ndarray:
        """
        Perform Bayesian update of probability distribution.
        
        Args:
            prior: Prior probability distribution
            likelihood: Likelihood function values
            evidence: Evidence (normalizing constant)
            
        Returns:
            Posterior probability distribution
        """
        posterior = prior * likelihood / evidence
        return posterior / np.sum(posterior)  # Normalize


# Educational examples and test functions
def educational_examples():
    """Demonstration of core numerical framework capabilities."""
    print("Scientific Computing Framework - Educational Examples")
    print("=" * 55)
    
    # Example 1: Matrix Operations
    print("\n1. Matrix Operations:")
    A = np.array([[3, 1], [1, 2]], dtype=float)
    b = np.array([9, 8], dtype=float)
    
    print(f"Matrix A:\n{A}")
    print(f"Vector b: {b}")
    
    solution = MatrixOperations.solve_linear_system(A, b)
    print(f"Solution x: {solution}")
    
    # Example 2: Optimization
    print("\n2. Optimization:")
    def quadratic_function(x):
        return x[0]**2 + 2*x[1]**2 + 0.5*x[0]*x[1] - 3*x[0] - 4*x[1] + 6
    
    def quadratic_gradient(x):
        return np.array([2*x[0] + 0.5*x[1] - 3, 4*x[1] + 0.5*x[0] - 4])
    
    initial_point = np.array([0.0, 0.0])
    optimum, history = Optimization.gradient_descent(
        quadratic_function, quadratic_gradient, initial_point, learning_rate=0.1
    )
    
    print(f"Initial point: {initial_point}")
    print(f"Optimum found: {optimum}")
    print(f"Function value at optimum: {quadratic_function(optimum):.6f}")
    
    # Example 3: FFT
    print("\n3. Signal Processing (FFT):")
    t = np.linspace(0, 1, 1000)
    signal = np.sin(2*np.pi*5*t) + 0.5*np.sin(2*np.pi*15*t)
    
    fft_result = FFTOperations.fft_1d(signal)
    print(f"Original signal length: {len(signal)}")
    print(f"FFT result length: {len(fft_result)}")
    
    # Example 4: Differential Equations
    print("\n4. Differential Equations:")
    def pendulum_equation(y, t):
        theta, omega = y
        dtheta_dt = omega
        domega_dt = -9.81 * np.sin(theta)
        return np.array([dtheta_dt, domega_dt])
    
    y0 = np.array([0.5, 0.0])  # Initial angle and angular velocity
    t, y = DifferentialEquations.solve_system_ode(pendulum_equation, y0, 0, 10, 0.01)
    
    print(f"Solved pendulum motion for {len(t)} time points")
    print(f"Initial conditions: θ₀={y0[0]:.3f}, ω₀={y0[1]:.3f}")
    print(f"Final state: θ={y[-1,0]:.3f}, ω={y[-1,1]:.3f}")


if __name__ == "__main__":
    educational_examples()