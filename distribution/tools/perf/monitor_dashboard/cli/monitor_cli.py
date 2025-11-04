#!/usr/bin/env python3
"""
Performance Monitor CLI - Command-line interface for system monitoring
Provides real-time system performance monitoring from the terminal
"""

import click
import time
import json
import sys
import os
from datetime import datetime
from typing import Dict, Any
from system_monitor import SystemMonitor
from alert_manager import AlertManager
from report_generator import ReportGenerator
import threading

# Color codes for terminal output
class Colors:
    HEADER = '\033[95m'
    BLUE = '\033[94m'
    CYAN = '\033[96m'
    GREEN = '\033[92m'
    WARNING = '\033[93m'
    RED = '\033[91m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'
    UNDERLINE = '\033[4m'

def colorize(text: str, color: str) -> str:
    """Add color to text if terminal supports it"""
    if sys.stdout.isatty():
        return f"{color}{text}{Colors.ENDC}"
    return text

def format_percentage(value: float) -> str:
    """Format percentage with color coding"""
    if value >= 95:
        return colorize(f"{value:.1f}%", Colors.RED)
    elif value >= 80:
        return colorize(f"{value:.1f}%", Colors.WARNING)
    else:
        return colorize(f"{value:.1f}%", Colors.GREEN)

def format_bytes(bytes_value: int) -> str:
    """Format bytes into human readable format"""
    for unit in ['B', 'KB', 'MB', 'GB', 'TB']:
        if bytes_value < 1024.0:
            return f"{bytes_value:.1f} {unit}"
        bytes_value /= 1024.0
    return f"{bytes_value:.1f} PB"

def clear_screen():
    """Clear terminal screen"""
    os.system('cls' if os.name == 'nt' else 'clear')

@click.group()
@click.pass_context
def cli(ctx):
    """Performance Monitoring Dashboard CLI"""
    ctx.ensure_object(dict)
    monitor = SystemMonitor()
    ctx.obj['monitor'] = monitor

@cli.command()
@click.option('--interval', '-i', default=2, help='Update interval in seconds')
@click.option('--duration', '-d', help='Duration to run (e.g., 5m, 1h)')
@click.option('--alerts', is_flag=True, help='Show alert status')
@click.pass_context
def monitor(ctx, interval: int, duration: str, alerts: bool):
    """Start real-time monitoring"""
    monitor = ctx.obj['monitor']
    
    click.echo(colorize("Performance Monitoring Dashboard - CLI", Colors.HEADER))
    click.echo(colorize("=" * 50, Colors.BLUE))
    click.echo(f"Update interval: {interval}s")
    if duration:
        click.echo(f"Duration: {duration}")
    if alerts:
        click.echo("Alert monitoring: enabled")
    click.echo("\nPress Ctrl+C to stop\n")
    
    # Calculate duration in seconds
    duration_seconds = None
    if duration:
        if duration.endswith('m'):
            duration_seconds = int(duration[:-1]) * 60
        elif duration.endswith('h'):
            duration_seconds = int(duration[:-1]) * 3600
        else:
            try:
                duration_seconds = int(duration)
            except ValueError:
                click.echo(colorize("Invalid duration format", Colors.RED))
                return
    
    start_time = time.time()
    
    try:
        while True:
            clear_screen()
            
            # Header
            click.echo(colorize("Performance Monitoring Dashboard - CLI", Colors.HEADER))
            click.echo(colorize("=" * 50, Colors.BLUE))
            click.echo(f"Time: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
            
            # Get current metrics
            metrics = monitor.get_current_metrics()
            
            # System Overview
            click.echo(colorize("\n[System Overview]", Colors.CYAN))
            
            cpu = metrics.get('cpu', {})
            memory = metrics.get('memory', {})
            disk = metrics.get('disk', {})
            kernel = metrics.get('kernel', {})
            
            click.echo(f"CPU Usage: {format_percentage(cpu.get('cpu_percent', 0))}")
            click.echo(f"CPU Cores: {cpu.get('cpu_count', 0)} physical, {cpu.get('cpu_count_logical', 0)} logical")
            click.echo(f"CPU Frequency: {cpu.get('cpu_freq', {}).get('current', 0):.0f} MHz")
            
            load_avg = cpu.get('load_avg', {})
            if load_avg:
                click.echo(f"Load Average: {load_avg.get('1min', 0):.2f} / {load_avg.get('5min', 0):.2f} / {load_avg.get('15min', 0):.2f}")
            
            click.echo(f"Memory Usage: {format_percentage(memory.get('virtual_memory', {}).get('percent', 0))}")
            vm = memory.get('virtual_memory', {})
            click.echo(f"Memory: {format_bytes(vm.get('used', 0))} / {format_bytes(vm.get('total', 0))}")
            
            disk_usage = disk.get('disk_usage', {})
            if disk_usage:
                for mount, usage in disk_usage.items():
                    click.echo(f"Disk {mount}: {format_percentage(usage.get('percent', 0))} ({format_bytes(usage.get('used', 0))} / {format_bytes(usage.get('total', 0))})")
            
            # Network
            network = metrics.get('network', {})
            net_io = network.get('network_io', {})
            if net_io:
                click.echo(f"Network: ↓{net_io.get('download_rate_mbps', 0):.1f} MB/s ↑{net_io.get('upload_rate_mbps', 0):.1f} MB/s")
            
            # Uptime
            if kernel:
                uptime = kernel.get('uptime_formatted', 'N/A')
                click.echo(f"Uptime: {uptime}")
                click.echo(f"Users: {kernel.get('users', 0)}")
            
            # Top Processes
            click.echo(colorize("\n[Top Processes by CPU]", Colors.CYAN))
            processes = metrics.get('processes', {})
            top_processes = processes.get('top_cpu_processes', [])[:5]
            
            if top_processes:
                click.echo(f"{'PID':<8} {'CPU%':<8} {'MEM%':<8} {'NAME':<20}")
                click.echo("-" * 50)
                for proc in top_processes:
                    cpu_percent = proc.get('cpu_percent', 0)
                    mem_percent = proc.get('memory_percent', 0)
                    name = proc.get('name', '')[:19]
                    
                    cpu_str = colorize(f"{cpu_percent:>6.1f}", Colors.GREEN if cpu_percent < 50 else Colors.WARNING if cpu_percent < 80 else Colors.RED)
                    mem_str = colorize(f"{mem_percent:>6.1f}", Colors.GREEN if mem_percent < 10 else Colors.WARNING if mem_percent < 30 else Colors.RED)
                    
                    click.echo(f"{proc.get('pid', 0):<8} {cpu_str:<8} {mem_str:<8} {name:<20}")
            
            # Process Summary
            click.echo(colorize("\n[Process Summary]", Colors.CYAN))
            click.echo(f"Total Processes: {processes.get('total_processes', 0)}")
            click.echo(f"Running: {processes.get('running_processes', 0)}")
            click.echo(f"Sleeping: {processes.get('sleeping_processes', 0)}")
            click.echo(f"Zombie: {processes.get('zombie_processes', 0)}")
            
            # Alert Status
            if alerts:
                click.echo(colorize("\n[Recent Alerts]", Colors.CYAN))
                recent_alerts = monitor.get_recent_alerts(1)  # Last hour
                if recent_alerts:
                    for alert in recent_alerts[-5:]:  # Last 5 alerts
                        severity_color = Colors.RED if alert.get('severity') == 'critical' else Colors.WARNING
                        click.echo(colorize(f"{alert.get('severity', '').upper()}: {alert.get('message', '')}", severity_color))
                else:
                    click.echo("No recent alerts")
            
            # Check if duration exceeded
            if duration_seconds and (time.time() - start_time) >= duration_seconds:
                break
            
            # Wait for next update
            time.sleep(interval)
    
    except KeyboardInterrupt:
        click.echo(colorize("\n\nMonitoring stopped by user", Colors.WARNING))

@cli.command()
@click.option('--output', '-o', help='Output file path')
@click.option('--format', 'output_format', default='json', type=click.Choice(['json', 'csv']), help='Output format')
@click.option('--hours', '-h', default=24, help='Hours of history to include')
@click.pass_context
def export(ctx, output: str, output_format: str, hours: int):
    """Export performance data"""
    monitor = ctx.obj['monitor']
    
    if not output:
        timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
        output = f'metrics_export_{timestamp}.{output_format}'
    
    click.echo(colorize("Exporting performance data...", Colors.BLUE))
    
    try:
        monitor.export_metrics(output, hours)
        click.echo(colorize(f"Data exported to {output}", Colors.GREEN))
    except Exception as e:
        click.echo(colorize(f"Export failed: {e}", Colors.RED))

@cli.command()
@click.option('--output', '-o', help='Report output file')
@click.option('--format', 'report_format', default='pdf', type=click.Choice(['pdf', 'html', 'csv']), help='Report format')
@click.option('--hours', '-h', default=24, help='Hours of history to include')
@click.pass_context
def report(ctx, output: str, report_format: str, hours: int):
    """Generate performance report"""
    monitor = ctx.obj['monitor']
    generator = ReportGenerator(monitor.db_path)
    
    if not output:
        timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
        output = f'report_{timestamp}.{report_format}'
    
    click.echo(colorize("Generating performance report...", Colors.BLUE))
    
    try:
        if report_format == 'pdf':
            generator.generate_pdf_report(hours, output)
        elif report_format == 'html':
            generator.generate_html_report(hours, output)
        elif report_format == 'csv':
            generator.generate_csv_report(hours, output)
        
        click.echo(colorize(f"Report generated: {output}", Colors.GREEN))
    except Exception as e:
        click.echo(colorize(f"Report generation failed: {e}", Colors.RED))

@cli.command()
@click.option('--hours', '-h', default=24, help='Hours of history to show')
@click.pass_context
def alerts(ctx, hours: int):
    """Show alert history and statistics"""
    monitor = ctx.obj['monitor']
    
    click.echo(colorize("Alert Management", Colors.HEADER))
    click.echo("=" * 30)
    
    # Get recent alerts
    alerts = monitor.get_recent_alerts(hours)
    
    click.echo(f"\nAlerts in last {hours} hours: {len(alerts)}")
    
    if alerts:
        # Count by severity
        critical_count = sum(1 for alert in alerts if alert.get('severity') == 'critical')
        warning_count = sum(1 for alert in alerts if alert.get('severity') == 'warning')
        
        click.echo(f"Critical: {critical_count}")
        click.echo(f"Warning: {warning_count}")
        
        # Show recent alerts
        click.echo(colorize("\nRecent Alerts:", Colors.CYAN))
        for alert in alerts[-10:]:  # Last 10
            severity = alert.get('severity', '').upper()
            message = alert.get('message', '')
            timestamp = alert.get('timestamp', '')
            
            severity_color = Colors.RED if severity == 'CRITICAL' else Colors.WARNING
            
            click.echo(colorize(f"[{severity}] {message}", severity_color))
            click.echo(f"    {timestamp}")
            click.echo()
    else:
        click.echo("No alerts found")

@cli.command()
@click.option('--metric', help='Custom metric name')
@click.option('--value', type=float, help='Metric value')
@click.option('--threshold', type=float, help='Alert threshold')
@click.option('--severity', default='warning', type=click.Choice(['info', 'warning', 'critical']), help='Alert severity')
def alert(metric: str, value: float, threshold: float, severity: str):
    """Create a custom alert"""
    if not metric or value is None or threshold is None:
        click.echo(colorize("Metric, value, and threshold are required", Colors.RED))
        return
    
    monitor = SystemMonitor()
    alert_manager = AlertManager(monitor.db_path)
    
    alert_id = alert_manager.create_threshold_alert(
        alert_type=metric,
        current_value=value,
        threshold=threshold,
        severity=severity
    )
    
    if alert_id > 0:
        click.echo(colorize(f"Alert created with ID: {alert_id}", Colors.GREEN))
    else:
        click.echo(colorize("No alert created (threshold not breached)", Colors.WARNING))

@cli.command()
@click.pass_context
def status(ctx):
    """Show current system status"""
    monitor = ctx.obj['monitor']
    
    click.echo(colorize("System Status", Colors.HEADER))
    click.echo("=" * 20)
    
    try:
        metrics = monitor.get_current_metrics()
        
        # Quick status overview
        cpu_percent = metrics.get('cpu', {}).get('cpu_percent', 0)
        memory_percent = metrics.get('memory', {}).get('virtual_memory', {}).get('percent', 0)
        
        cpu_status = "OK" if cpu_percent < 80 else "HIGH" if cpu_percent < 95 else "CRITICAL"
        memory_status = "OK" if memory_percent < 85 else "HIGH" if memory_percent < 95 else "CRITICAL"
        
        cpu_color = Colors.GREEN if cpu_percent < 80 else Colors.WARNING if cpu_percent < 95 else Colors.RED
        memory_color = Colors.GREEN if memory_percent < 85 else Colors.WARNING if memory_percent < 95 else Colors.RED
        
        click.echo(f"CPU Usage: {colorize(cpu_status, cpu_color)} ({cpu_percent:.1f}%)")
        click.echo(f"Memory Usage: {colorize(memory_status, memory_color)} ({memory_percent:.1f}%)")
        
        # Recent alerts
        recent_alerts = monitor.get_recent_alerts(1)  # Last hour
        active_alerts = [alert for alert in recent_alerts if not alert.get('acknowledged', False)]
        
        if active_alerts:
            click.echo(f"\nActive Alerts: {len(active_alerts)}")
            for alert in active_alerts[-5:]:  # Show last 5
                severity = alert.get('severity', '').upper()
                message = alert.get('message', '')
                click.echo(colorize(f"  [{severity}] {message}", Colors.WARNING if severity == 'WARNING' else Colors.RED))
        else:
            click.echo("\nActive Alerts: 0")
        
        click.echo(colorize("\nSystem Status: HEALTHY", Colors.GREEN))
        
    except Exception as e:
        click.echo(colorize(f"Error getting status: {e}", Colors.RED))

@cli.command()
@click.option('--host', default='localhost', help='Web dashboard host')
@click.option('--port', default=5000, help='Web dashboard port')
@click.pass_context
def web(ctx, host: str, port: int):
    """Start the web dashboard"""
    click.echo(colorize("Starting web dashboard...", Colors.BLUE))
    click.echo(colorize(f"Dashboard will be available at http://{host}:{port}", Colors.GREEN))
    click.echo(colorize("Press Ctrl+C to stop", Colors.WARNING))
    
    # Import and start the web dashboard
    try:
        from web_dashboard import app, socketio
        socketio.run(app, debug=False, host=host, port=port)
    except Exception as e:
        click.echo(colorize(f"Failed to start web dashboard: {e}", Colors.RED))

@cli.command()
@click.option('--duration', default=10, help='Test duration in seconds')
@click.pass_context
def test(ctx, duration: int):
    """Run system performance test"""
    monitor = ctx.obj['monitor']
    
    click.echo(colorize("Running performance test...", Colors.BLUE))
    click.echo(f"Test duration: {duration} seconds")
    
    start_time = time.time()
    metrics_history = []
    
    try:
        while time.time() - start_time < duration:
            metrics = monitor.get_current_metrics()
            metrics_history.append(metrics)
            time.sleep(1)
        
        # Analyze results
        if metrics_history:
            cpu_values = [m.get('cpu', {}).get('cpu_percent', 0) for m in metrics_history]
            memory_values = [m.get('memory', {}).get('virtual_memory', {}).get('percent', 0) for m in metrics_history]
            
            click.echo(colorize("\nTest Results:", Colors.CYAN))
            click.echo(f"Average CPU: {sum(cpu_values)/len(cpu_values):.1f}%")
            click.echo(f"Max CPU: {max(cpu_values):.1f}%")
            click.echo(f"Average Memory: {sum(memory_values)/len(memory_values):.1f}%")
            click.echo(f"Max Memory: {max(memory_values):.1f}%")
            
            click.echo(colorize("\nTest completed successfully", Colors.GREEN))
        
    except Exception as e:
        click.echo(colorize(f"Test failed: {e}", Colors.RED))

if __name__ == '__main__':
    cli()