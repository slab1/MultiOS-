"""
PID Controller Implementation

Provides PID control algorithms for robot motion control
"""

import time
import numpy as np
from typing import List, Tuple, Optional
from dataclasses import dataclass


@dataclass
class PIDConfig:
    """PID controller configuration"""
    kp: float = 1.0      # Proportional gain
    ki: float = 0.1      # Integral gain  
    kd: float = 0.05     # Derivative gain
    output_min: float = -1.0  # Minimum output
    output_max: float = 1.0   # Maximum output
    integral_limit: float = 100.0  # Integral windup limit


class PIDController:
    """PID Controller for robot motion control"""
    
    def __init__(self, config: Optional[PIDConfig] = None):
        self.config = config or PIDConfig()
        
        # State variables
        self.last_error = 0.0
        self.integral = 0.0
        self.last_time = None
        self.last_output = 0.0
        
        # For tracking and analysis
        self.history = {
            'time': [],
            'error': [],
            'output': [],
            'setpoint': [],
            'process_variable': []
        }
        
    def compute(self, setpoint: float, process_variable: float, dt: Optional[float] = None) -> float:
        """
        Compute PID control output
        
        Args:
            setpoint: Desired target value
            process_variable: Current measured value
            dt: Time step (computed if not provided)
            
        Returns:
            Control output
        """
        current_time = time.time()
        
        # Calculate time step
        if dt is None:
            if self.last_time is None:
                dt = 0.02  # Default 50 Hz
            else:
                dt = current_time - self.last_time
                
        # Avoid division by zero and limit dt
        dt = max(1e-6, min(dt, 0.1))
        
        # Calculate error
        error = setpoint - process_variable
        
        # Proportional term
        proportional = self.config.kp * error
        
        # Integral term with anti-windup
        self.integral += error * dt
        if abs(self.integral) > self.config.integral_limit:
            self.integral = np.sign(self.integral) * self.config.integral_limit
            
        integral_term = self.config.ki * self.integral
        
        # Derivative term
        derivative = (error - self.last_error) / dt
        derivative_term = self.config.kd * derivative
        
        # Calculate output
        output = proportional + integral_term + derivative_term
        
        # Apply output limits
        output = max(self.config.output_min, 
                    min(self.config.output_max, output))
        
        # Update state
        self.last_error = error
        self.last_time = current_time
        self.last_output = output
        
        # Record history
        self._record_history(current_time, error, output, setpoint, process_variable)
        
        return output
        
    def _record_history(self, time: float, error: float, output: float, 
                       setpoint: float, process_variable: float):
        """Record control history for analysis"""
        max_history = 1000  # Keep last 1000 points
        
        self.history['time'].append(time)
        self.history['error'].append(error)
        self.history['output'].append(output)
        self.history['setpoint'].append(setpoint)
        self.history['process_variable'].append(process_variable)
        
        # Trim history if it gets too long
        if len(self.history['time']) > max_history:
            for key in self.history:
                self.history[key] = self.history[key][-max_history:]
                
    def reset(self):
        """Reset controller state"""
        self.last_error = 0.0
        self.integral = 0.0
        self.last_time = None
        self.last_output = 0.0
        
    def tune(self, kp: Optional[float] = None, ki: Optional[float] = None, 
             kd: Optional[float] = None):
        """Tune PID parameters"""
        if kp is not None:
            self.config.kp = kp
        if ki is not None:
            self.config.ki = ki
        if kd is not None:
            self.config.kd = kd
            
    def get_config(self) -> PIDConfig:
        """Get current configuration"""
        return self.config
        
    def get_performance_metrics(self) -> dict:
        """Calculate performance metrics"""
        if len(self.history['error']) < 2:
            return {'settling_time': 0, 'overshoot': 0, 'steady_state_error': 0}
            
        errors = np.array(self.history['error'])
        times = np.array(self.history['time'])
        
        # Steady-state error (last 10% of data)
        steady_state_portion = int(len(errors) * 0.9)
        steady_state_error = np.mean(np.abs(errors[steady_state_portion:]))
        
        # Overshoot calculation
        max_error = np.max(errors)
        initial_error = errors[0] if len(errors) > 0 else 0
        overshoot = abs(max_error) / abs(initial_error) if initial_error != 0 else 0
        
        # Settling time (time to reach 2% of target)
        target_error = 0.02 * abs(errors[0]) if len(errors) > 0 else 0
        settled_indices = np.where(np.abs(errors) < target_error)[0]
        
        if len(settled_indices) > 0:
            # Find when it first stays within 2% for good
            for i in range(len(settled_indices) - 10):
                if all(abs(errors[settled_indices[i:i+10]]) < target_error):
                    settling_time = times[settled_indices[i]] - times[0]
                    break
            else:
                settling_time = times[-1] - times[0] if len(times) > 0 else 0
        else:
            settling_time = 0
            
        return {
            'settling_time': settling_time,
            'overshoot': overshoot,
            'steady_state_error': steady_state_error,
            'rms_error': np.sqrt(np.mean(errors**2))
        }
        
    def plot_response(self, save_path: Optional[str] = None):
        """Plot PID controller response"""
        try:
            import matplotlib.pyplot as plt
            
            if not self.history['time']:
                print("No history data to plot")
                return
                
            fig, (ax1, ax2, ax3) = plt.subplots(3, 1, figsize=(12, 8))
            
            times = np.array(self.history['time']) - self.history['time'][0]
            
            # Error plot
            ax1.plot(times, self.history['error'], 'r-', label='Error')
            ax1.set_ylabel('Error')
            ax1.set_title('PID Controller Response')
            ax1.grid(True)
            ax1.legend()
            
            # Setpoint vs Process Variable
            ax2.plot(times, self.history['setpoint'], 'g--', label='Setpoint')
            ax2.plot(times, self.history['process_variable'], 'b-', label='Process Variable')
            ax2.set_ylabel('Value')
            ax2.legend()
            ax2.grid(True)
            
            # Control Output
            ax3.plot(times, self.history['output'], 'm-', label='Control Output')
            ax3.set_ylabel('Output')
            ax3.set_xlabel('Time (s)')
            ax3.legend()
            ax3.grid(True)
            
            plt.tight_layout()
            
            if save_path:
                plt.savefig(save_path)
                print(f"Plot saved to {save_path}")
            else:
                plt.show()
                
        except ImportError:
            print("matplotlib not available - cannot plot response")
        except Exception as e:
            print(f"Error plotting response: {e}")


class MultiPIDController:
    """Multiple PID controllers for multi-axis control"""
    
    def __init__(self, configs: Optional[dict] = None):
        self.controllers = {}
        
        # Default controller configurations
        default_configs = {
            'position_x': PIDConfig(kp=2.0, ki=0.1, kd=0.1),
            'position_y': PIDConfig(kp=2.0, ki=0.1, kd=0.1),
            'orientation': PIDConfig(kp=3.0, ki=0.2, kd=0.15),
            'speed': PIDConfig(kp=1.5, ki=0.05, kd=0.05)
        }
        
        final_configs = {**default_configs, **(configs or {})}
        
        for axis, config in final_configs.items():
            self.controllers[axis] = PIDController(config)
            
    def compute_all(self, setpoints: dict, process_variables: dict, dt: Optional[float] = None) -> dict:
        """Compute all PID control outputs"""
        outputs = {}
        
        for axis in self.controllers:
            if axis in setpoints and axis in process_variables:
                output = self.controllers[axis].compute(
                    setpoints[axis], 
                    process_variables[axis], 
                    dt
                )
                outputs[axis] = output
                
        return outputs
        
    def reset_all(self):
        """Reset all controllers"""
        for controller in self.controllers.values():
            controller.reset()
            
    def get_performance_summary(self) -> dict:
        """Get performance summary for all controllers"""
        summary = {}
        for axis, controller in self.controllers.items():
            summary[axis] = controller.get_performance_metrics()
        return summary


class PIDTuner:
    """Auto-tuning for PID controllers"""
    
    @staticmethod
    def ziegler_nichols_step_response(controller: PIDController, 
                                    process_func, 
                                    step_size: float = 0.1) -> PIDConfig:
        """
        Ziegler-Nichols tuning method using step response
        
        Args:
            controller: PID controller to tune
            process_func: Function that simulates the process
            step_size: Size of step input
            
        Returns:
            Tuned PID configuration
        """
        print("Starting Ziegler-Nichols step response tuning...")
        
        # Step test
        controller.reset()
        process_variable = 0.0
        setpoint = step_size
        
        # Run step test
        history = {'time': [], 'output': [], 'response': []}
        
        start_time = time.time()
        while time.time() - start_time < 30:  # 30 second test
            output = controller.compute(setpoint, process_variable)
            process_variable = process_func(output, process_variable)
            
            current_time = time.time() - start_time
            history['time'].append(current_time)
            history['output'].append(output)
            history['response'].append(process_variable)
            
            time.sleep(0.1)
            
        # Analyze response
        response_array = np.array(history['response'])
        time_array = np.array(history['time'])
        
        # Find key parameters
        initial_value = response_array[0] if len(response_array) > 0 else 0
        final_value = response_array[-1] if len(response_array) > 0 else step_size
        
        # Time constant (63% of final value)
        target_value = initial_value + 0.63 * (final_value - initial_value)
        time_constant_index = np.where(response_array >= target_value)[0]
        time_constant = time_array[time_constant_index[0]] if len(time_constant_index) > 0 else 1.0
        
        # Rise time (10% to 90%)
        ten_percent = initial_value + 0.1 * (final_value - initial_value)
        ninety_percent = initial_value + 0.9 * (final_value - initial_value)
        
        ten_percent_idx = np.where(response_array >= ten_percent)[0]
        ninety_percent_idx = np.where(response_array >= ninety_percent)[0]
        
        if len(ten_percent_idx) > 0 and len(ninety_percent_idx) > 0:
            rise_time = time_array[ninety_percent_idx[0]] - time_array[ten_percent_idx[0]]
        else:
            rise_time = time_constant
            
        # Calculate PID parameters using Ziegler-Nichols rules
        k_process = abs(final_value - initial_value) / step_size if step_size != 0 else 1.0
        
        kp = 1.2 * (1.0 / (k_process * time_constant)) if time_constant > 0 else 1.0
        ki = kp * 2.0 / rise_time if rise_time > 0 else 0.1
        kd = kp * rise_time * 0.5 if rise_time > 0 else 0.05
        
        tuned_config = PIDConfig(
            kp=kp,
            ki=ki, 
            kd=kd,
            output_min=controller.config.output_min,
            output_max=controller.config.output_max
        )
        
        print(f"Ziegler-Nichols tuning complete:")
        print(f"  Kp: {kp:.3f}")
        print(f"  Ki: {ki:.3f}")
        print(f"  Kd: {kd:.3f}")
        
        return tuned_config
        
    @staticmethod
    def simple_tune(controller: PIDController, 
                   initial_response: float,
                   overshoot_target: float = 0.2) -> PIDConfig:
        """
        Simple tuning method based on initial response
        
        Args:
            controller: Controller to tune
            initial_response: Initial process response value
            overshoot_target: Target overshoot ratio (0.0 to 1.0)
            
        Returns:
            Tuned PID configuration
        """
        # Simple heuristic tuning
        base_kp = 2.0 / abs(initial_response) if initial_response != 0 else 2.0
        
        # Adjust based on overshoot target
        if overshoot_target < 0.1:
            # Low overshoot - increase derivative gain
            kd_factor = 0.15
            ki_factor = 0.05
        elif overshoot_target > 0.3:
            # High overshoot - decrease proportional gain
            kd_factor = 0.05
            ki_factor = 0.1
            base_kp *= 0.7
        else:
            # Moderate overshoot
            kd_factor = 0.1
            ki_factor = 0.08
            
        tuned_config = PIDConfig(
            kp=base_kp,
            ki=base_kp * ki_factor,
            kd=base_kp * kd_factor
        )
        
        return tuned_config


def process_simulation(input_signal: float, current_state: float, 
                      time_constant: float = 2.0, gain: float = 1.0) -> float:
    """Simple first-order process simulation for tuning"""
    dt = 0.1
    rate = gain * input_signal / time_constant
    new_state = current_state + rate * dt
    
    # Add some damping
    new_state *= (1 - dt / time_constant)
    
    return new_state


if __name__ == "__main__":
    # Example usage
    config = PIDConfig(kp=2.0, ki=0.1, kd=0.05)
    pid = PIDController(config)
    
    # Simulate control response
    setpoint = 1.0
    process_variable = 0.0
    
    print("Simulating PID controller response...")
    
    for i in range(100):
        dt = 0.02
        output = pid.compute(setpoint, process_variable, dt)
        process_variable = process_simulation(output, process_variable)
        
        if i % 20 == 0:
            print(f"Step {i}: PV={process_variable:.3f}, Output={output:.3f}")
            
    metrics = pid.get_performance_metrics()
    print(f"Performance: {metrics}")
    
    # Example of multi-axis control
    print("\nMulti-axis PID example:")
    multi_pid = MultiPIDController({
        'position_x': PIDConfig(kp=2.0),
        'position_y': PIDConfig(kp=2.0),
        'orientation': PIDConfig(kp=3.0)
    })
    
    setpoints = {'position_x': 1.0, 'position_y': 0.5, 'orientation': 0.0}
    process_vars = {'position_x': 0.0, 'position_y': 0.0, 'orientation': 0.0}
    
    outputs = multi_pid.compute_all(setpoints, process_vars)
    print(f"Multi-axis outputs: {outputs}")
