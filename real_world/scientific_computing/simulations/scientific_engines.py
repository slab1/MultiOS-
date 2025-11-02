"""
Scientific Simulation Engines for Educational Computing
======================================================

This module provides simulation engines for various scientific domains:
- Physics: mechanics, electromagnetism, thermodynamics
- Chemistry: reaction kinetics, molecular dynamics, quantum chemistry
- Biology: population dynamics, genetic algorithms, cellular automata

Author: Scientific Computing Education Team
"""

import numpy as np
import matplotlib.pyplot as plt
from typing import Dict, List, Tuple, Callable, Optional
from abc import ABC, abstractmethod


class PhysicsSimulation:
    """Physics simulation engine for educational purposes."""
    
    def __init__(self):
        self.gravity = 9.81  # m/s²
        self.planck_constant = 6.626e-34  # J⋅s
        self.boltzmann_constant = 1.381e-23  # J/K
    
    def newtonian_mechanics(self, mass: float, force: float, 
                          initial_position: float = 0.0, 
                          initial_velocity: float = 0.0,
                          dt: float = 0.01, total_time: float = 10.0) -> Dict:
        """
        Simulate Newtonian mechanics with constant force.
        
        Physics: F = ma, s = ut + ½at², v = u + at
        
        Args:
            mass: Object mass in kg
            force: Applied force in N
            initial_position: Initial position in m
            initial_velocity: Initial velocity in m/s
            dt: Time step in s
            total_time: Total simulation time in s
            
        Returns:
            Dictionary with time, position, velocity, and acceleration arrays
        """
        n_steps = int(total_time / dt) + 1
        time = np.linspace(0, total_time, n_steps)
        
        acceleration = force / mass
        position = np.zeros(n_steps)
        velocity = np.zeros(n_steps)
        
        position[0] = initial_position
        velocity[0] = initial_velocity
        
        # Integrate equations of motion
        for i in range(1, n_steps):
            # Euler integration
            velocity[i] = velocity[i-1] + acceleration * dt
            position[i] = position[i-1] + velocity[i-1] * dt
            
        return {
            'time': time,
            'position': position,
            'velocity': velocity,
            'acceleration': np.full(n_steps, acceleration)
        }
    
    def harmonic_oscillator(self, mass: float, spring_constant: float,
                          damping: float = 0.0, initial_position: float = 1.0,
                          initial_velocity: float = 0.0,
                          dt: float = 0.01, total_time: float = 20.0) -> Dict:
        """
        Simulate harmonic oscillator motion.
        
        Physics: F = -kx - c*v, m*dv/dt = F
        
        Args:
            mass: Mass in kg
            spring_constant: Spring constant in N/m
            damping: Damping coefficient in kg/s
            initial_position: Initial displacement in m
            initial_velocity: Initial velocity in m/s
            dt: Time step in s
            total_time: Total simulation time in s
            
        Returns:
            Dictionary with simulation results
        """
        omega = np.sqrt(spring_constant / mass)  # Natural frequency
        
        n_steps = int(total_time / dt) + 1
        time = np.linspace(0, total_time, n_steps)
        
        position = np.zeros(n_steps)
        velocity = np.zeros(n_steps)
        
        position[0] = initial_position
        velocity[0] = initial_velocity
        
        # Analytical solution for reference
        if damping == 0:
            # Undamped case
            position_analytical = initial_position * np.cos(omega * time)
            velocity_analytical = -initial_position * omega * np.sin(omega * time)
        else:
            # Damped case
            damping_ratio = damping / (2 * np.sqrt(mass * spring_constant))
            if damping_ratio < 1:  # Underdamped
                omega_d = omega * np.sqrt(1 - damping_ratio**2)
                envelope = np.exp(-damping_ratio * omega * time)
                position_analytical = (initial_position * np.cos(omega_d * time) +
                                     (initial_velocity / omega_d) * np.sin(omega_d * time))
                velocity_analytical = (-initial_position * omega_d * np.sin(omega_d * time) +
                                     (initial_velocity / omega_d) * np.cos(omega_d * time))
                position_analytical *= envelope
                velocity_analytical *= envelope
            else:  # Overdamped
                r1 = (-damping_ratio + np.sqrt(damping_ratio**2 - 1)) * omega
                r2 = (-damping_ratio - np.sqrt(damping_ratio**2 - 1)) * omega
                A = (initial_velocity - r2 * initial_position) / (r1 - r2)
                B = initial_position - A
                position_analytical = A * np.exp(r1 * time) + B * np.exp(r2 * time)
                velocity_analytical = A * r1 * np.exp(r1 * time) + B * r2 * np.exp(r2 * time)
        
        # Numerical integration
        for i in range(1, n_steps):
            # RK4 method for better accuracy
            def derivatives(state):
                x, v = state
                return np.array([v, -(spring_constant/mass) * x - (damping/mass) * v])
            
            # RK4 integration
            k1 = derivatives(np.array([position[i-1], velocity[i-1]])) * dt
            k2 = derivatives(np.array([position[i-1] + k1[0]/2, velocity[i-1] + k1[1]/2])) * dt
            k3 = derivatives(np.array([position[i-1] + k2[0]/2, velocity[i-1] + k2[1]/2])) * dt
            k4 = derivatives(np.array([position[i-1] + k3[0], velocity[i-1] + k3[1]])) * dt
            
            position[i] = position[i-1] + (k1[0] + 2*k2[0] + 2*k3[0] + k4[0]) / 6
            velocity[i] = velocity[i-1] + (k1[1] + 2*k2[1] + 2*k3[1] + k4[1]) / 6
        
        return {
            'time': time,
            'position': position,
            'velocity': velocity,
            'position_analytical': position_analytical,
            'velocity_analytical': velocity_analytical,
            'natural_frequency': omega,
            'damping_ratio': damping / (2 * np.sqrt(mass * spring_constant))
        }
    
    def electromagnetic_wave(self, frequency: float, amplitude: float,
                          electric_field_amplitude: float, 
                          dt: float = 0.01, total_time: float = 1.0) -> Dict:
        """
        Simulate electromagnetic wave propagation.
        
        Physics: E(z,t) = E₀ cos(ωt - kz), B(z,t) = E₀/c cos(ωt - kz)
        
        Args:
            frequency: Wave frequency in Hz
            amplitude: Wave amplitude
            electric_field_amplitude: E₀ in V/m
            dt: Time step in s
            total_time: Total simulation time in s
            
        Returns:
            Dictionary with E and B field data
        """
        c = 3e8  # Speed of light in m/s
        wavelength = c / frequency
        omega = 2 * np.pi * frequency
        k = 2 * np.pi / wavelength
        
        n_steps = int(total_time / dt) + 1
        time = np.linspace(0, total_time, n_steps)
        z = np.linspace(0, 5 * wavelength, 100)  # Spatial domain
        
        # Create mesh
        T, Z = np.meshgrid(time, z)
        
        # Calculate fields
        E_field = electric_field_amplitude * np.cos(omega * T - k * Z)
        B_field = (electric_field_amplitude / c) * np.cos(omega * T - k * Z)
        
        return {
            'time': time,
            'position': z,
            'electric_field': E_field,
            'magnetic_field': B_field,
            'wavelength': wavelength,
            'frequency': frequency,
            'c': c
        }
    
    def thermodynamics_ideal_gas(self, n_moles: float, initial_temp: float,
                               pressure: float = 101325,  # 1 atm in Pa
                               volume: float = 0.0224,    # 1 mole at STP in m³
                               cv: float = 20.8,          # CV,m for ideal gas
                               dt: float = 0.1, total_time: float = 100.0) -> Dict:
        """
        Simulate ideal gas thermodynamics.
        
        Physics: PV = nRT, ΔU = nCvΔT for ideal gas
        
        Args:
            n_moles: Number of moles
            initial_temp: Initial temperature in K
            pressure: Pressure in Pa
            volume: Volume in m³
            cv: Molar heat capacity at constant volume in J/(mol·K)
            dt: Time step in s
            total_time: Total simulation time in s
            
        Returns:
            Dictionary with thermodynamic properties
        """
        R = 8.314  # Gas constant in J/(mol·K)
        
        # Calculate initial temperature from ideal gas law
        T = np.full(int(total_time/dt) + 1, initial_temp)
        U = np.zeros(len(T))  # Internal energy
        
        # Heat addition curve (example: exponential heating)
        t = np.linspace(0, total_time, len(T))
        heat_rate = 100 * np.exp(-t/20)  # W, decreasing heat input
        
        # Energy balance
        for i in range(1, len(T)):
            # dU = dQ for constant volume process
            delta_U = heat_rate[i-1] * dt
            U[i] = U[i-1] + delta_U
            T[i] = T[0] + U[i] / (n_moles * cv)
        
        # Calculate pressure from ideal gas law
        P = n_moles * R * T / volume
        
        return {
            'time': t,
            'temperature': T,
            'pressure': P,
            'internal_energy': U,
            'heat_rate': heat_rate,
            'volume': volume,
            'moles': n_moles
        }


class ChemistrySimulation:
    """Chemistry simulation engine for educational purposes."""
    
    def __init__(self):
        self.avogadro_number = 6.022e23  # molecules/mol
        self.gas_constant = 8.314  # J/(mol·K)
    
    def reaction_kinetics_1st_order(self, rate_constant: float, initial_concentration: float,
                                  dt: float = 0.1, total_time: float = 10.0) -> Dict:
        """
        Simulate first-order reaction kinetics.
        
        Chemistry: A → B, d[A]/dt = -k[A]
        
        Args:
            rate_constant: First-order rate constant in s⁻¹
            initial_concentration: Initial concentration of A in mol/L
            dt: Time step in s
            total_time: Total simulation time in s
            
        Returns:
            Dictionary with concentration vs time data
        """
        n_steps = int(total_time / dt) + 1
        time = np.linspace(0, total_time, n_steps)
        
        concentration_A = np.zeros(n_steps)
        concentration_B = np.zeros(n_steps)
        
        concentration_A[0] = initial_concentration
        concentration_B[0] = 0.0
        
        # Analytical solution: [A] = [A]₀e^(-kt)
        concentration_A_analytical = initial_concentration * np.exp(-rate_constant * time)
        concentration_B_analytical = initial_concentration - concentration_A_analytical
        
        # Numerical integration (Euler)
        for i in range(1, n_steps):
            dA_dt = -rate_constant * concentration_A[i-1]
            concentration_A[i] = concentration_A[i-1] + dA_dt * dt
            concentration_B[i] = initial_concentration - concentration_A[i]
        
        return {
            'time': time,
            'concentration_A': concentration_A,
            'concentration_B': concentration_B,
            'concentration_A_analytical': concentration_A_analytical,
            'concentration_B_analytical': concentration_B_analytical,
            'rate_constant': rate_constant
        }
    
    def reaction_kinetics_2nd_order(self, rate_constant: float, initial_concentration_A: float,
                                  initial_concentration_B: float,
                                  dt: float = 0.1, total_time: float = 10.0) -> Dict:
        """
        Simulate second-order reaction kinetics.
        
        Chemistry: A + B → C, d[A]/dt = -k[A][B]
        
        Args:
            rate_constant: Second-order rate constant in L/(mol·s)
            initial_concentration_A: Initial concentration of A
            initial_concentration_B: Initial concentration of B
            dt: Time step in s
            total_time: Total simulation time in s
            
        Returns:
            Dictionary with concentration vs time data
        """
        n_steps = int(total_time / dt) + 1
        time = np.linspace(0, total_time, n_steps)
        
        concentration_A = np.zeros(n_steps)
        concentration_B = np.zeros(n_steps)
        concentration_C = np.zeros(n_steps)
        
        concentration_A[0] = initial_concentration_A
        concentration_B[0] = initial_concentration_B
        concentration_C[0] = 0.0
        
        # Numerical integration (RK4 for better accuracy)
        for i in range(1, n_steps):
            def derivatives(concentrations):
                A, B, C = concentrations
                dA_dt = -rate_constant * A * B
                dB_dt = -rate_constant * A * B
                dC_dt = rate_constant * A * B
                return np.array([dA_dt, dB_dt, dC_dt])
            
            # RK4 integration
            current_concs = np.array([concentration_A[i-1], concentration_B[i-1], concentration_C[i-1]])
            
            k1 = derivatives(current_concs) * dt
            k2 = derivatives(current_concs + k1/2) * dt
            k3 = derivatives(current_concs + k2/2) * dt
            k4 = derivatives(current_concs + k3) * dt
            
            new_concs = current_concs + (k1 + 2*k2 + 2*k3 + k4) / 6
            
            concentration_A[i] = max(0, new_concs[0])  # Ensure non-negative
            concentration_B[i] = max(0, new_concs[1])
            concentration_C[i] = max(0, new_concs[2])
        
        return {
            'time': time,
            'concentration_A': concentration_A,
            'concentration_B': concentration_B,
            'concentration_C': concentration_C,
            'rate_constant': rate_constant
        }
    
    def molecular_dynamics_simple(self, n_particles: int = 100, 
                                temperature: float = 300, box_size: float = 20.0,
                                dt: float = 0.001, total_time: float = 1.0) -> Dict:
        """
        Simple molecular dynamics simulation using Lennard-Jones potential.
        
        Physics: U(r) = 4ε[(σ/r)¹² - (σ/r)⁶], F = -∇U
        
        Args:
            n_particles: Number of particles
            temperature: Temperature in K
            box_size: Simulation box size in Å
            dt: Time step in fs (femtoseconds)
            total_time: Total simulation time in ps (picoseconds)
            
        Returns:
            Dictionary with trajectory and thermodynamic data
        """
        # LJ parameters (reduced units)
        epsilon = 1.0  # Depth of potential well
        sigma = 1.0    # Distance at which potential is zero
        mass = 1.0     # Particle mass
        
        # Convert simulation units
        n_steps = int(total_time * 1000 / dt)  # Convert ps to fs, then to steps
        dt_fs = dt
        total_steps = min(n_steps, 10000)  # Limit for performance
        
        # Initialize positions (simple cubic lattice)
        grid_size = int(np.ceil(n_particles ** (1/3)))
        positions = np.zeros((n_particles, 3))
        
        for i in range(n_particles):
            ix, iy, iz = np.unravel_index(i, (grid_size, grid_size, grid_size))
            spacing = box_size / grid_size
            positions[i] = np.array([ix * spacing, iy * spacing, iz * spacing]) + spacing/2
        
        # Initialize velocities (Maxwell-Boltzmann distribution)
        velocities = np.random.normal(0, np.sqrt(self.boltzmann_constant * temperature / mass),
                                    (n_particles, 3))
        
        # Remove center of mass motion
        velocities -= np.mean(velocities, axis=0)
        
        # Rescale to target temperature
        current_temp = np.mean(np.sum(velocities**2, axis=1)) * mass / (3 * self.boltzmann_constant)
        scale_factor = np.sqrt(temperature / current_temp)
        velocities *= scale_factor
        
        # Storage arrays
        trajectory = np.zeros((total_steps, n_particles, 3))
        temperatures = np.zeros(total_steps)
        energies = np.zeros(total_steps)
        
        for step in range(total_steps):
            forces = np.zeros((n_particles, 3))
            potential_energy = 0.0
            
            # Calculate forces using Lennard-Jones potential
            for i in range(n_particles):
                for j in range(i+1, n_particles):
                    r_vec = positions[j] - positions[i]
                    
                    # Periodic boundary conditions
                    r_vec -= box_size * np.round(r_vec / box_size)
                    
                    r = np.linalg.norm(r_vec)
                    
                    if r < 2.5 * sigma:  # Cutoff at 2.5σ
                        # Lennard-Jones force
                        force_mag = 24 * epsilon * (2 * (sigma/r)**12 - (sigma/r)**6) / r
                        force = force_mag * r_vec / r
                        
                        forces[i] += force
                        forces[j] -= force
                        
                        # Potential energy
                        potential_energy += 4 * epsilon * ((sigma/r)**12 - (sigma/r)**6)
            
            # Velocity Verlet integration
            velocities += forces * dt_fs / (2 * mass)
            positions += velocities * dt_fs
            
            # Periodic boundary conditions
            positions = np.mod(positions, box_size)
            
            # Calculate new forces
            forces_new = np.zeros((n_particles, 3))
            for i in range(n_particles):
                for j in range(i+1, n_particles):
                    r_vec = positions[j] - positions[i]
                    r_vec -= box_size * np.round(r_vec / box_size)
                    
                    r = np.linalg.norm(r_vec)
                    
                    if r < 2.5 * sigma:
                        force_mag = 24 * epsilon * (2 * (sigma/r)**12 - (sigma/r)**6) / r
                        force = force_mag * r_vec / r
                        
                        forces_new[i] += force
                        forces_new[j] -= force
            
            velocities += forces_new * dt_fs / (2 * mass)
            
            # Calculate kinetic energy and temperature
            kinetic_energy = 0.5 * mass * np.sum(velocities**2)
            current_temp = 2 * kinetic_energy / (3 * n_particles * self.boltzmann_constant)
            
            # Store data
            trajectory[step] = positions.copy()
            temperatures[step] = current_temp
            energies[step] = potential_energy + kinetic_energy
        
        return {
            'trajectory': trajectory,
            'temperatures': temperatures,
            'energies': energies,
            'time': np.arange(total_steps) * dt_fs,
            'box_size': box_size,
            'n_particles': n_particles
        }


class BiologySimulation:
    """Biology simulation engine for educational purposes."""
    
    def __init__(self):
        pass
    
    def population_dynamics_lv(self, r: float, K: float, initial_population: float,
                             dt: float = 0.1, total_time: float = 100.0) -> Dict:
        """
        Simulate population dynamics using Lotka-Volterra model.
        
        Biology: dN/dt = rN(1 - N/K) - predation_rate
        
        Args:
            r: Growth rate per time unit
            K: Carrying capacity
            initial_population: Initial population size
            dt: Time step
            total_time: Total simulation time
            
        Returns:
            Dictionary with population data
        """
        n_steps = int(total_time / dt) + 1
        time = np.linspace(0, total_time, n_steps)
        
        population = np.zeros(n_steps)
        population[0] = initial_population
        
        # Add environmental stochasticity
        noise_level = 0.05
        
        # Logistic growth with environmental variation
        for i in range(1, n_steps):
            noise = np.random.normal(0, noise_level * population[i-1])
            growth_rate = r * (1 - population[i-1] / K)
            dp_dt = growth_rate * population[i-1] + noise
            population[i] = max(0, population[i-1] + dp_dt * dt)
        
        return {
            'time': time,
            'population': population,
            'growth_rate': r,
            'carrying_capacity': K
        }
    
    def predator_prey_model(self, prey_growth: float, predation_rate: float,
                          predator_efficiency: float, predator_death: float,
                          initial_prey: float, initial_predators: float,
                          dt: float = 0.1, total_time: float = 100.0) -> Dict:
        """
        Simulate predator-prey dynamics using Lotka-Volterra equations.
        
        Biology: 
        dPrey/dt = αPrey - βPrey*Predator
        dPredator/dt = γPrey*Predator - δPredator
        
        Args:
            prey_growth: Prey growth rate (α)
            predation_rate: Predation rate (β)
            predator_efficiency: Predator reproduction efficiency (γ)
            predator_death: Predator death rate (δ)
            initial_prey: Initial prey population
            initial_predators: Initial predator population
            dt: Time step
            total_time: Total simulation time
            
        Returns:
            Dictionary with population dynamics
        """
        n_steps = int(total_time / dt) + 1
        time = np.linspace(0, total_time, n_steps)
        
        prey = np.zeros(n_steps)
        predators = np.zeros(n_steps)
        
        prey[0] = initial_prey
        predators[0] = initial_predators
        
        # RK4 integration for better accuracy
        for i in range(1, n_steps):
            def derivatives(state):
                P, R = state  # Prey, Predators
                dPrey_dt = prey_growth * P - predation_rate * P * R
                dPredator_dt = predator_efficiency * P * R - predator_death * R
                return np.array([dPrey_dt, dPredator_dt])
            
            # RK4 integration
            current_state = np.array([prey[i-1], predators[i-1]])
            
            k1 = derivatives(current_state) * dt
            k2 = derivatives(current_state + k1/2) * dt
            k3 = derivatives(current_state + k2/2) * dt
            k4 = derivatives(current_state + k3) * dt
            
            new_state = current_state + (k1 + 2*k2 + 2*k3 + k4) / 6
            
            prey[i] = max(0, new_state[0])
            predators[i] = max(0, new_state[1])
        
        return {
            'time': time,
            'prey': prey,
            'predators': predators,
            'prey_growth': prey_growth,
            'predation_rate': predation_rate
        }
    
    def genetic_algorithm(self, population_size: int = 100, gene_length: int = 20,
                        target_fitness: float = 1.0, mutation_rate: float = 0.01,
                        max_generations: int = 1000) -> Dict:
        """
        Simulate evolution using genetic algorithm.
        
        Biology: Natural selection through crossover, mutation, and selection
        
        Args:
            population_size: Number of individuals in population
            gene_length: Length of genetic code (binary)
            target_fitness: Fitness threshold for convergence
            mutation_rate: Probability of gene mutation
            max_generations: Maximum number of generations
            
        Returns:
            Dictionary with evolution data
        """
        # Initialize random population
        population = np.random.randint(0, 2, (population_size, gene_length))
        
        fitness_history = []
        best_individuals = []
        
        for generation in range(max_generations):
            # Calculate fitness (example: maximize number of 1s)
            fitness = np.sum(population, axis=1) / gene_length
            
            # Check for convergence
            max_fitness = np.max(fitness)
            if max_fitness >= target_fitness:
                break
            
            # Selection (tournament selection)
            new_population = np.zeros_like(population)
            
            for i in range(population_size):
                # Tournament selection
                tournament_size = 3
                tournament_indices = np.random.choice(population_size, tournament_size, replace=False)
                tournament_fitness = fitness[tournament_indices]
                winner_idx = tournament_indices[np.argmax(tournament_fitness)]
                new_population[i] = population[winner_idx].copy()
            
            # Crossover (single point)
            crossover_rate = 0.8
            for i in range(0, population_size-1, 2):
                if np.random.random() < crossover_rate:
                    crossover_point = np.random.randint(1, gene_length)
                    # Swap genes after crossover point
                    temp = new_population[i, crossover_point:].copy()
                    new_population[i, crossover_point:] = new_population[i+1, crossover_point:]
                    new_population[i+1, crossover_point:] = temp
            
            # Mutation
            for i in range(population_size):
                for j in range(gene_length):
                    if np.random.random() < mutation_rate:
                        new_population[i, j] = 1 - new_population[i, j]
            
            population = new_population.copy()
            
            # Record statistics
            fitness_history.append((np.mean(fitness), np.max(fitness), np.min(fitness)))
            best_idx = np.argmax(fitness)
            best_individuals.append(population[best_idx].copy())
        
        return {
            'final_population': population,
            'fitness_history': fitness_history,
            'generations': generation + 1,
            'best_fitness': np.max(fitness),
            'average_fitness': np.mean(fitness)
        }
    
    def cellular_automaton_conway(self, grid_size: int = 50, 
                                initial_alive_probability: float = 0.3,
                                steps: int = 100) -> Dict:
        """
        Simulate Conway's Game of Life cellular automaton.
        
        Biology: Cellular automata model of life simulation
        Rules:
        1. Live cell with <2 neighbors dies (underpopulation)
        2. Live cell with 2-3 neighbors survives
        3. Live cell with >3 neighbors dies (overpopulation)
        4. Dead cell with exactly 3 neighbors becomes alive (reproduction)
        
        Args:
            grid_size: Size of the grid
            initial_alive_probability: Initial probability of cell being alive
            steps: Number of simulation steps
            
        Returns:
            Dictionary with simulation results
        """
        # Initialize grid
        grid = np.random.random((grid_size, grid_size)) < initial_alive_probability
        grid = grid.astype(int)
        
        history = [grid.copy()]
        
        for step in range(steps):
            new_grid = grid.copy()
            
            for i in range(grid_size):
                for j in range(grid_size):
                    # Count neighbors (including wrapping)
                    neighbors = 0
                    for di in [-1, 0, 1]:
                        for dj in [-1, 0, 1]:
                            if di == 0 and dj == 0:
                                continue
                            ni, nj = (i + di) % grid_size, (j + dj) % grid_size
                            neighbors += grid[ni, nj]
                    
                    # Apply Game of Life rules
                    if grid[i, j] == 1:  # Cell is alive
                        if neighbors < 2 or neighbors > 3:
                            new_grid[i, j] = 0  # Dies
                    else:  # Cell is dead
                        if neighbors == 3:
                            new_grid[i, j] = 1  # Becomes alive
            
            grid = new_grid.copy()
            history.append(grid.copy())
        
        return {
            'grid_history': np.array(history),
            'initial_grid': history[0],
            'final_grid': history[-1],
            'steps': steps,
            'grid_size': grid_size
        }


# Demonstration and testing functions
def run_physics_simulations():
    """Demonstrate physics simulation capabilities."""
    print("Physics Simulation Engine - Educational Examples")
    print("=" * 50)
    
    physics = PhysicsSimulation()
    
    # Example 1: Harmonic Oscillator
    print("\n1. Harmonic Oscillator:")
    result = physics.harmonic_oscillator(
        mass=1.0, spring_constant=10.0, damping=0.1,
        initial_position=1.0, total_time=10.0
    )
    print(f"Natural frequency: {result['natural_frequency']:.3f} rad/s")
    print(f"Damping ratio: {result['damping_ratio']:.3f}")
    
    # Example 2: Electromagnetic Wave
    print("\n2. Electromagnetic Wave:")
    em_result = physics.electromagnetic_wave(
        frequency=1e9,  # 1 GHz
        electric_field_amplitude=100  # V/m
    )
    print(f"Wavelength: {em_result['wavelength']:.3f} m")
    print(f"Frequency: {em_result['frequency']:.3e} Hz")


def run_chemistry_simulations():
    """Demonstrate chemistry simulation capabilities."""
    print("\n\nChemistry Simulation Engine - Educational Examples")
    print("=" * 52)
    
    chemistry = ChemistrySimulation()
    
    # Example 1: First-order kinetics
    print("\n1. First-order Reaction Kinetics:")
    kinetics_result = chemistry.reaction_kinetics_1st_order(
        rate_constant=0.1, initial_concentration=1.0, total_time=50.0
    )
    print(f"Rate constant: {kinetics_result['rate_constant']:.3f} s⁻¹")
    print(f"Final concentration A: {kinetics_result['concentration_A'][-1]:.3f}")
    
    # Example 2: Molecular Dynamics (simplified)
    print("\n2. Molecular Dynamics (100 particles):")
    md_result = chemistry.molecular_dynamics_simple(
        n_particles=100, temperature=300, total_time=0.1  # Short simulation
    )
    print(f"Average temperature: {np.mean(md_result['temperatures']):.1f} K")
    print(f"Final energy: {md_result['energies'][-1]:.3f}")


def run_biology_simulations():
    """Demonstrate biology simulation capabilities."""
    print("\n\nBiology Simulation Engine - Educational Examples")
    print("=" * 51)
    
    biology = BiologySimulation()
    
    # Example 1: Predator-Prey Model
    print("\n1. Predator-Prey Dynamics:")
    pred_prey_result = biology.predator_prey_model(
        prey_growth=1.0, predation_rate=0.1,
        predator_efficiency=0.1, predator_death=0.5,
        initial_prey=10.0, initial_predators=1.0,
        total_time=100.0
    )
    print(f"Final prey population: {pred_prey_result['prey'][-1]:.3f}")
    print(f"Final predator population: {pred_prey_result['predators'][-1]:.3f}")
    
    # Example 2: Genetic Algorithm
    print("\n2. Genetic Algorithm Evolution:")
    ga_result = biology.genetic_algorithm(
        population_size=50, gene_length=20, target_fitness=0.9
    )
    print(f"Generations to converge: {ga_result['generations']}")
    print(f"Best fitness achieved: {ga_result['best_fitness']:.3f}")
    
    # Example 3: Conway's Game of Life
    print("\n3. Conway's Game of Life:")
    life_result = biology.cellular_automaton_conway(
        grid_size=30, initial_alive_probability=0.3, steps=50
    )
    print(f"Grid size: {life_result['grid_size']}x{life_result['grid_size']}")
    print(f"Steps simulated: {life_result['steps']}")
    
    # Count living cells
    initial_live = np.sum(life_result['initial_grid'])
    final_live = np.sum(life_result['final_grid'])
    print(f"Initial living cells: {initial_live}")
    print(f"Final living cells: {final_live}")


if __name__ == "__main__":
    run_physics_simulations()
    run_chemistry_simulations()
    run_biology_simulations()