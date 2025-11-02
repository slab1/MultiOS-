#!/usr/bin/env python3
"""
Report Generator - Generates comprehensive performance reports
Supports PDF, HTML, and CSV export formats
"""

import sqlite3
import json
import csv
import os
from datetime import datetime, timedelta
from typing import Dict, List, Any, Optional
import matplotlib.pyplot as plt
import matplotlib.dates as mdates
from matplotlib.backends.backend_pdf import PdfPages
import pandas as pd
import numpy as np
from io import BytesIO
import base64

class ReportGenerator:
    def __init__(self, db_path: str = "data/monitor.db"):
        self.db_path = db_path
        
    def generate_comprehensive_report(self, hours: int = 24) -> Dict[str, Any]:
        """Generate comprehensive performance analysis"""
        try:
            # Get historical data
            system_data = self._get_system_metrics_history(hours)
            process_data = self._get_process_metrics_history(hours)
            alert_data = self._get_alerts_history(hours)
            
            # Generate analysis
            report = {
                'period_hours': hours,
                'generated_at': datetime.now().isoformat(),
                'system_analysis': self._analyze_system_performance(system_data),
                'process_analysis': self._analyze_process_performance(process_data),
                'alert_analysis': self._analyze_alerts(alert_data),
                'recommendations': self._generate_recommendations(system_data, process_data, alert_data)
            }
            
            return report
            
        except Exception as e:
            print(f"Error generating comprehensive report: {e}")
            return {}
    
    def generate_pdf_report(self, hours: int = 24, output_path: str = None):
        """Generate PDF performance report"""
        if output_path is None:
            timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
            output_path = f'reports/performance_report_{timestamp}.pdf'
        
        os.makedirs(os.path.dirname(output_path), exist_ok=True)
        
        try:
            report = self.generate_comprehensive_report(hours)
            
            with PdfPages(output_path) as pdf:
                # Title page
                self._create_title_page(pdf, hours)
                
                # System performance charts
                self._create_system_charts(pdf, hours)
                
                # Process analysis
                self._create_process_analysis(pdf, hours)
                
                # Alerts summary
                self._create_alerts_summary(pdf, hours)
                
                # Recommendations
                self._create_recommendations_page(pdf, report['recommendations'])
            
            print(f"PDF report generated: {output_path}")
            
        except Exception as e:
            print(f"Error generating PDF report: {e}")
    
    def generate_html_report(self, hours: int = 24, output_path: str = None):
        """Generate HTML performance report"""
        if output_path is None:
            timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
            output_path = f'reports/performance_report_{timestamp}.html'
        
        os.makedirs(os.path.dirname(output_path), exist_ok=True)
        
        try:
            report = self.generate_comprehensive_report(hours)
            html_content = self._create_html_report(report)
            
            with open(output_path, 'w') as f:
                f.write(html_content)
            
            print(f"HTML report generated: {output_path}")
            
        except Exception as e:
            print(f"Error generating HTML report: {e}")
    
    def generate_csv_report(self, hours: int = 24, output_path: str = None):
        """Generate CSV performance report"""
        if output_path is None:
            timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
            output_path = f'reports/performance_report_{timestamp}.csv'
        
        os.makedirs(os.path.dirname(output_path), exist_ok=True)
        
        try:
            # Get data
            system_data = self._get_system_metrics_history(hours)
            process_data = self._get_process_metrics_history(hours)
            alert_data = self._get_alerts_history(hours)
            
            with open(output_path, 'w', newline='') as csvfile:
                writer = csv.writer(csvfile)
                
                # Write summary
                writer.writerow(['Performance Report'])
                writer.writerow([f'Period: {hours} hours'])
                writer.writerow([f'Generated: {datetime.now().isoformat()}'])
                writer.writerow([])
                
                # System metrics summary
                if system_data:
                    writer.writerow(['System Metrics Summary'])
                    avg_cpu = np.mean([row['cpu_percent'] for row in system_data if row['cpu_percent']])
                    max_cpu = max([row['cpu_percent'] for row in system_data if row['cpu_percent']])
                    avg_memory = np.mean([json.loads(row['memory_percent'] or '0') if row['memory_percent'] else 0 for row in system_data])
                    
                    writer.writerow([f'Average CPU Usage: {avg_cpu:.1f}%'])
                    writer.writerow([f'Maximum CPU Usage: {max_cpu:.1f}%'])
                    writer.writerow([f'Average Memory Usage: {avg_memory:.1f}%'])
                    writer.writerow([])
                
                # Alerts summary
                writer.writerow(['Alerts Summary'])
                critical_alerts = [a for a in alert_data if a['severity'] == 'critical']
                warning_alerts = [a for a in alert_data if a['severity'] == 'warning']
                
                writer.writerow([f'Total Alerts: {len(alert_data)}'])
                writer.writerow([f'Critical Alerts: {len(critical_alerts)}'])
                writer.writerow([f'Warning Alerts: {len(warning_alerts)}'])
                writer.writerow([])
                
                # Detailed data
                writer.writerow(['Detailed System Metrics'])
                writer.writerow(['Timestamp', 'CPU %', 'Memory %', 'Load Avg'])
                for row in system_data:
                    writer.writerow([
                        row['timestamp'],
                        row['cpu_percent'] or 0,
                        json.loads(row['memory_percent'] or '0') if row['memory_percent'] else 0,
                        json.loads(row['load_avg'] or '{}').get('1min', 0) if row['load_avg'] else 0
                    ])
            
            print(f"CSV report generated: {output_path}")
            
        except Exception as e:
            print(f"Error generating CSV report: {e}")
    
    def _get_system_metrics_history(self, hours: int) -> List[Dict]:
        """Get historical system metrics"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            start_time = datetime.now() - timedelta(hours=hours)
            cursor.execute('''
                SELECT * FROM system_metrics 
                WHERE timestamp > ? 
                ORDER BY timestamp
            ''', (start_time.isoformat(),))
            
            columns = [description[0] for description in cursor.description]
            data = [dict(zip(columns, row)) for row in cursor.fetchall()]
            
            conn.close()
            return data
            
        except Exception as e:
            print(f"Error getting system metrics history: {e}")
            return []
    
    def _get_process_metrics_history(self, hours: int) -> List[Dict]:
        """Get historical process metrics"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            start_time = datetime.now() - timedelta(hours=hours)
            cursor.execute('''
                SELECT * FROM process_metrics 
                WHERE timestamp > ? 
                ORDER BY timestamp
            ''', (start_time.isoformat(),))
            
            columns = [description[0] for description in cursor.description]
            data = [dict(zip(columns, row)) for row in cursor.fetchall()]
            
            conn.close()
            return data
            
        except Exception as e:
            print(f"Error getting process metrics history: {e}")
            return []
    
    def _get_alerts_history(self, hours: int) -> List[Dict]:
        """Get historical alerts"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            start_time = datetime.now() - timedelta(hours=hours)
            cursor.execute('''
                SELECT * FROM alerts 
                WHERE timestamp > ? 
                ORDER BY timestamp
            ''', (start_time.isoformat(),))
            
            columns = [description[0] for description in cursor.description]
            data = [dict(zip(columns, row)) for row in cursor.fetchall()]
            
            conn.close()
            return data
            
        except Exception as e:
            print(f"Error getting alerts history: {e}")
            return []
    
    def _analyze_system_performance(self, system_data: List[Dict]) -> Dict[str, Any]:
        """Analyze system performance from historical data"""
        if not system_data:
            return {}
        
        try:
            # CPU analysis
            cpu_values = [row['cpu_percent'] for row in system_data if row['cpu_percent']]
            cpu_analysis = {
                'average': np.mean(cpu_values) if cpu_values else 0,
                'maximum': max(cpu_values) if cpu_values else 0,
                'minimum': min(cpu_values) if cpu_values else 0,
                'percentiles': {
                    '50': np.percentile(cpu_values, 50) if cpu_values else 0,
                    '90': np.percentile(cpu_values, 90) if cpu_values else 0,
                    '95': np.percentile(cpu_values, 95) if cpu_values else 0,
                    '99': np.percentile(cpu_values, 99) if cpu_values else 0
                }
            }
            
            # Memory analysis
            memory_values = []
            for row in system_data:
                if row['memory_percent']:
                    try:
                        memory_val = float(row['memory_percent'])
                        memory_values.append(memory_val)
                    except (ValueError, TypeError):
                        continue
            
            memory_analysis = {
                'average': np.mean(memory_values) if memory_values else 0,
                'maximum': max(memory_values) if memory_values else 0,
                'minimum': min(memory_values) if memory_values else 0,
                'percentiles': {
                    '50': np.percentile(memory_values, 50) if memory_values else 0,
                    '90': np.percentile(memory_values, 90) if memory_values else 0,
                    '95': np.percentile(memory_values, 95) if memory_values else 0,
                    '99': np.percentile(memory_values, 99) if memory_values else 0
                }
            }
            
            # Load average analysis
            load_values = []
            for row in system_data:
                if row['load_avg']:
                    try:
                        load_data = json.loads(row['load_avg'])
                        load_values.append(load_data.get('1min', 0))
                    except (ValueError, TypeError):
                        continue
            
            load_analysis = {
                'average': np.mean(load_values) if load_values else 0,
                'maximum': max(load_values) if load_values else 0,
                'minimum': min(load_values) if load_values else 0
            }
            
            return {
                'cpu': cpu_analysis,
                'memory': memory_analysis,
                'load_average': load_analysis,
                'data_points': len(system_data)
            }
            
        except Exception as e:
            print(f"Error analyzing system performance: {e}")
            return {}
    
    def _analyze_process_performance(self, process_data: List[Dict]) -> Dict[str, Any]:
        """Analyze process performance from historical data"""
        if not process_data:
            return {}
        
        try:
            # Top CPU consuming processes
            cpu_processes = {}
            for row in process_data:
                name = row['name']
                cpu_percent = row['cpu_percent'] or 0
                if name not in cpu_processes:
                    cpu_processes[name] = []
                cpu_processes[name].append(cpu_percent)
            
            avg_cpu_per_process = {name: np.mean(values) for name, values in cpu_processes.items()}
            top_cpu_processes = sorted(avg_cpu_per_process.items(), key=lambda x: x[1], reverse=True)[:10]
            
            # Top memory consuming processes
            memory_processes = {}
            for row in process_data:
                name = row['name']
                memory_percent = row['memory_percent'] or 0
                if name not in memory_processes:
                    memory_processes[name] = []
                memory_processes[name].append(memory_percent)
            
            avg_memory_per_process = {name: np.mean(values) for name, values in memory_processes.items()}
            top_memory_processes = sorted(avg_memory_per_process.items(), key=lambda x: x[1], reverse=True)[:10]
            
            return {
                'top_cpu_processes': top_cpu_processes,
                'top_memory_processes': top_memory_processes,
                'total_processes': len(process_data),
                'unique_processes': len(set(row['name'] for row in process_data))
            }
            
        except Exception as e:
            print(f"Error analyzing process performance: {e}")
            return {}
    
    def _analyze_alerts(self, alert_data: List[Dict]) -> Dict[str, Any]:
        """Analyze alert patterns"""
        if not alert_data:
            return {}
        
        try:
            alert_counts = {}
            for alert in alert_data:
                alert_type = alert['alert_type']
                severity = alert['severity']
                key = f"{alert_type}_{severity}"
                alert_counts[key] = alert_counts.get(key, 0) + 1
            
            # Group by severity
            critical_count = sum(1 for alert in alert_data if alert['severity'] == 'critical')
            warning_count = sum(1 for alert in alert_data if alert['severity'] == 'warning')
            
            # Most frequent alert types
            type_counts = {}
            for alert in alert_data:
                alert_type = alert['alert_type']
                type_counts[alert_type] = type_counts.get(alert_type, 0) + 1
            
            top_alert_types = sorted(type_counts.items(), key=lambda x: x[1], reverse=True)
            
            return {
                'total_alerts': len(alert_data),
                'critical_alerts': critical_count,
                'warning_alerts': warning_count,
                'alert_types': top_alert_types,
                'alert_counts': alert_counts
            }
            
        except Exception as e:
            print(f"Error analyzing alerts: {e}")
            return {}
    
    def _generate_recommendations(self, system_data: List[Dict], 
                                process_data: List[Dict], 
                                alert_data: List[Dict]) -> List[str]:
        """Generate performance recommendations"""
        recommendations = []
        
        try:
            # CPU recommendations
            if system_data:
                cpu_values = [row['cpu_percent'] for row in system_data if row['cpu_percent']]
                if cpu_values:
                    avg_cpu = np.mean(cpu_values)
                    max_cpu = max(cpu_values)
                    
                    if max_cpu > 95:
                        recommendations.append("CPU usage frequently exceeds 95% - consider adding more CPU cores or optimizing resource-intensive processes")
                    elif avg_cpu > 80:
                        recommendations.append("Average CPU usage is high (80%+) - consider reviewing running processes for optimization")
                    elif avg_cpu < 20:
                        recommendations.append("CPU usage is consistently low - the system may be over-provisioned")
            
            # Memory recommendations
            if system_data:
                memory_values = []
                for row in system_data:
                    if row['memory_percent']:
                        try:
                            memory_val = float(row['memory_percent'])
                            memory_values.append(memory_val)
                        except (ValueError, TypeError):
                            continue
                
                if memory_values:
                    avg_memory = np.mean(memory_values)
                    max_memory = max(memory_values)
                    
                    if max_memory > 95:
                        recommendations.append("Memory usage frequently exceeds 95% - consider adding more RAM or identifying memory leaks")
                    elif avg_memory > 85:
                        recommendations.append("Average memory usage is high (85%+) - monitor for memory leaks and consider optimization")
            
            # Process recommendations
            if process_data:
                # Find processes with high average CPU
                cpu_processes = {}
                for row in process_data:
                    name = row['name']
                    cpu_percent = row['cpu_percent'] or 0
                    if name not in cpu_processes:
                        cpu_processes[name] = []
                    cpu_processes[name].append(cpu_percent)
                
                high_cpu_processes = []
                for name, values in cpu_processes.items():
                    avg_cpu = np.mean(values)
                    if avg_cpu > 50:
                        high_cpu_processes.append((name, avg_cpu))
                
                if high_cpu_processes:
                    top_proc = max(high_cpu_processes, key=lambda x: x[1])
                    recommendations.append(f"Process '{top_proc[0]}' shows high average CPU usage ({top_proc[1]:.1f}%) - review and optimize if possible")
            
            # Alert-based recommendations
            if alert_data:
                alert_types = {}
                for alert in alert_data:
                    alert_type = alert['alert_type']
                    alert_types[alert_type] = alert_types.get(alert_type, 0) + 1
                
                for alert_type, count in alert_types.items():
                    if count > 10:
                        recommendations.append(f"Recurring {alert_type} alerts ({count} occurrences) - investigate root cause and implement fixes")
            
            # General recommendations
            if not recommendations:
                recommendations.append("System performance appears stable with no major issues detected")
            
        except Exception as e:
            print(f"Error generating recommendations: {e}")
            recommendations.append("Error generating recommendations - check system logs")
        
        return recommendations
    
    def _create_title_page(self, pdf, hours: int):
        """Create title page for PDF report"""
        fig, ax = plt.subplots(figsize=(8.5, 11))
        ax.axis('off')
        
        # Title
        ax.text(0.5, 0.9, 'Performance Monitoring Report', 
                fontsize=24, fontweight='bold', ha='center', va='center')
        
        # Period info
        ax.text(0.5, 0.8, f'Analysis Period: {hours} Hours', 
                fontsize=16, ha='center', va='center')
        
        # Generated info
        ax.text(0.5, 0.75, f'Generated: {datetime.now().strftime("%Y-%m-%d %H:%M:%S")}', 
                fontsize=12, ha='center', va='center')
        
        # Summary stats
        report = self.generate_comprehensive_report(hours)
        system_analysis = report.get('system_analysis', {})
        
        y_pos = 0.6
        if system_analysis:
            ax.text(0.5, y_pos, 'Key Metrics Summary', 
                    fontsize=16, fontweight='bold', ha='center', va='center')
            
            y_pos -= 0.1
            cpu_analysis = system_analysis.get('cpu', {})
            if cpu_analysis:
                ax.text(0.3, y_pos, f'CPU Usage: {cpu_analysis.get("average", 0):.1f}% avg, {cpu_analysis.get("maximum", 0):.1f}% max', 
                        fontsize=12, ha='left', va='center')
                y_pos -= 0.07
            
            memory_analysis = system_analysis.get('memory', {})
            if memory_analysis:
                ax.text(0.3, y_pos, f'Memory Usage: {memory_analysis.get("average", 0):.1f}% avg, {memory_analysis.get("maximum", 0):.1f}% max', 
                        fontsize=12, ha='left', va='center')
                y_pos -= 0.07
            
            load_analysis = system_analysis.get('load_average', {})
            if load_analysis:
                ax.text(0.3, y_pos, f'Load Average: {load_analysis.get("average", 0):.2f} avg, {load_analysis.get("maximum", 0):.2f} max', 
                        fontsize=12, ha='left', va='center')
                y_pos -= 0.07
        
        pdf.savefig(fig, bbox_inches='tight')
        plt.close()
    
    def _create_system_charts(self, pdf, hours: int):
        """Create system performance charts"""
        try:
            system_data = self._get_system_metrics_history(hours)
            if not system_data:
                return
            
            # Prepare data
            timestamps = []
            cpu_values = []
            memory_values = []
            load_values = []
            
            for row in system_data:
                try:
                    timestamp = datetime.fromisoformat(row['timestamp'])
                    timestamps.append(timestamp)
                    cpu_values.append(row['cpu_percent'] or 0)
                    
                    if row['memory_percent']:
                        memory_values.append(float(row['memory_percent']))
                    else:
                        memory_values.append(0)
                    
                    if row['load_avg']:
                        load_data = json.loads(row['load_avg'])
                        load_values.append(load_data.get('1min', 0))
                    else:
                        load_values.append(0)
                except (ValueError, TypeError):
                    continue
            
            # Create CPU chart
            fig, axes = plt.subplots(2, 2, figsize=(11, 8.5))
            
            # CPU usage over time
            if timestamps and cpu_values:
                axes[0, 0].plot(timestamps, cpu_values, color='red', alpha=0.7)
                axes[0, 0].set_title('CPU Usage Over Time')
                axes[0, 0].set_ylabel('CPU Usage (%)')
                axes[0, 0].grid(True, alpha=0.3)
                axes[0, 0].xaxis.set_major_formatter(mdates.DateFormatter('%H:%M'))
                plt.setp(axes[0, 0].xaxis.get_majorticklabels(), rotation=45)
            
            # Memory usage over time
            if timestamps and memory_values:
                axes[0, 1].plot(timestamps, memory_values, color='blue', alpha=0.7)
                axes[0, 1].set_title('Memory Usage Over Time')
                axes[0, 1].set_ylabel('Memory Usage (%)')
                axes[0, 1].grid(True, alpha=0.3)
                axes[0, 1].xaxis.set_major_formatter(mdates.DateFormatter('%H:%M'))
                plt.setp(axes[0, 1].xaxis.get_majorticklabels(), rotation=45)
            
            # Load average over time
            if timestamps and load_values:
                axes[1, 0].plot(timestamps, load_values, color='green', alpha=0.7)
                axes[1, 0].set_title('Load Average Over Time')
                axes[1, 0].set_ylabel('Load Average (1 min)')
                axes[1, 0].grid(True, alpha=0.3)
                axes[1, 0].xaxis.set_major_formatter(mdates.DateFormatter('%H:%M'))
                plt.setp(axes[1, 0].xaxis.get_majorticklabels(), rotation=45)
            
            # CPU distribution
            if cpu_values:
                axes[1, 1].hist(cpu_values, bins=20, color='orange', alpha=0.7, edgecolor='black')
                axes[1, 1].set_title('CPU Usage Distribution')
                axes[1, 1].set_xlabel('CPU Usage (%)')
                axes[1, 1].set_ylabel('Frequency')
                axes[1, 1].grid(True, alpha=0.3)
            
            plt.tight_layout()
            pdf.savefig(fig, bbox_inches='tight')
            plt.close()
            
        except Exception as e:
            print(f"Error creating system charts: {e}")
    
    def _create_process_analysis(self, pdf, hours: int):
        """Create process analysis page"""
        try:
            process_data = self._get_process_metrics_history(hours)
            if not process_data:
                return
            
            # Analyze processes
            cpu_processes = {}
            memory_processes = {}
            
            for row in process_data:
                name = row['name']
                cpu_percent = row['cpu_percent'] or 0
                memory_percent = row['memory_percent'] or 0
                
                if name not in cpu_processes:
                    cpu_processes[name] = []
                    memory_processes[name] = []
                
                cpu_processes[name].append(cpu_percent)
                memory_processes[name].append(memory_percent)
            
            # Calculate averages and get top processes
            avg_cpu_per_process = {name: np.mean(values) for name, values in cpu_processes.items()}
            avg_memory_per_process = {name: np.mean(values) for name, values in memory_processes.items()}
            
            top_cpu = sorted(avg_cpu_per_process.items(), key=lambda x: x[1], reverse=True)[:10]
            top_memory = sorted(avg_memory_per_process.items(), key=lambda x: x[1], reverse=True)[:10]
            
            # Create charts
            fig, axes = plt.subplots(1, 2, figsize=(11, 8.5))
            
            # Top CPU processes
            if top_cpu:
                processes, cpu_values = zip(*top_cpu)
                axes[0].barh(range(len(processes)), cpu_values, color='red', alpha=0.7)
                axes[0].set_yticks(range(len(processes)))
                axes[0].set_yticklabels(processes)
                axes[0].set_title('Top 10 CPU Consuming Processes')
                axes[0].set_xlabel('Average CPU Usage (%)')
                axes[0].grid(True, alpha=0.3)
            
            # Top memory processes
            if top_memory:
                processes, memory_values = zip(*top_memory)
                axes[1].barh(range(len(processes)), memory_values, color='blue', alpha=0.7)
                axes[1].set_yticks(range(len(processes)))
                axes[1].set_yticklabels(processes)
                axes[1].set_title('Top 10 Memory Consuming Processes')
                axes[1].set_xlabel('Average Memory Usage (%)')
                axes[1].grid(True, alpha=0.3)
            
            plt.tight_layout()
            pdf.savefig(fig, bbox_inches='tight')
            plt.close()
            
        except Exception as e:
            print(f"Error creating process analysis: {e}")
    
    def _create_alerts_summary(self, pdf, hours: int):
        """Create alerts summary page"""
        try:
            alert_data = self._get_alerts_history(hours)
            if not alert_data:
                return
            
            # Analyze alerts
            alert_types = {}
            severity_counts = {'critical': 0, 'warning': 0}
            
            for alert in alert_data:
                alert_type = alert['alert_type']
                severity = alert['severity']
                
                if alert_type not in alert_types:
                    alert_types[alert_type] = {'critical': 0, 'warning': 0}
                
                alert_types[alert_type][severity] += 1
                severity_counts[severity] += 1
            
            # Create charts
            fig, axes = plt.subplots(1, 2, figsize=(11, 8.5))
            
            # Alert distribution by type
            if alert_types:
                types = list(alert_types.keys())
                critical_counts = [alert_types[t]['critical'] for t in types]
                warning_counts = [alert_types[t]['warning'] for t in types]
                
                x = np.arange(len(types))
                width = 0.35
                
                axes[0].bar(x - width/2, critical_counts, width, label='Critical', color='red', alpha=0.7)
                axes[0].bar(x + width/2, warning_counts, width, label='Warning', color='orange', alpha=0.7)
                
                axes[0].set_xlabel('Alert Type')
                axes[0].set_ylabel('Count')
                axes[0].set_title('Alert Distribution by Type')
                axes[0].set_xticks(x)
                axes[0].set_xticklabels(types, rotation=45)
                axes[0].legend()
                axes[0].grid(True, alpha=0.3)
            
            # Severity distribution pie chart
            if severity_counts['critical'] > 0 or severity_counts['warning'] > 0:
                labels = []
                sizes = []
                colors = []
                
                if severity_counts['critical'] > 0:
                    labels.append('Critical')
                    sizes.append(severity_counts['critical'])
                    colors.append('red')
                
                if severity_counts['warning'] > 0:
                    labels.append('Warning')
                    sizes.append(severity_counts['warning'])
                    colors.append('orange')
                
                axes[1].pie(sizes, labels=labels, colors=colors, autopct='%1.1f%%', startangle=90)
                axes[1].set_title('Alert Severity Distribution')
            
            plt.tight_layout()
            pdf.savefig(fig, bbox_inches='tight')
            plt.close()
            
        except Exception as e:
            print(f"Error creating alerts summary: {e}")
    
    def _create_recommendations_page(self, pdf, recommendations: List[str]):
        """Create recommendations page"""
        try:
            fig, ax = plt.subplots(figsize=(8.5, 11))
            ax.axis('off')
            
            ax.text(0.5, 0.95, 'Performance Recommendations', 
                    fontsize=20, fontweight='bold', ha='center', va='center')
            
            y_pos = 0.85
            for i, recommendation in enumerate(recommendations, 1):
                ax.text(0.1, y_pos, f"{i}. {recommendation}", 
                        fontsize=12, ha='left', va='top', wrap=True)
                y_pos -= 0.1
            
            # Add footer
            ax.text(0.5, 0.05, 'Generated by Performance Monitoring Dashboard', 
                    fontsize=10, style='italic', ha='center', va='center')
            
            pdf.savefig(fig, bbox_inches='tight')
            plt.close()
            
        except Exception as e:
            print(f"Error creating recommendations page: {e}")
    
    def _create_html_report(self, report: Dict[str, Any]) -> str:
        """Create HTML report content"""
        try:
            html_content = f"""
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Performance Monitoring Report</title>
                <style>
                    body {{
                        font-family: Arial, sans-serif;
                        margin: 0;
                        padding: 20px;
                        background-color: #f5f5f5;
                    }}
                    .container {{
                        max-width: 1200px;
                        margin: 0 auto;
                        background-color: white;
                        padding: 30px;
                        border-radius: 10px;
                        box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                    }}
                    h1, h2, h3 {{
                        color: #333;
                    }}
                    h1 {{
                        text-align: center;
                        border-bottom: 2px solid #007bff;
                        padding-bottom: 10px;
                    }}
                    .metric-grid {{
                        display: grid;
                        grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
                        gap: 20px;
                        margin: 20px 0;
                    }}
                    .metric-card {{
                        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                        color: white;
                        padding: 20px;
                        border-radius: 10px;
                        text-align: center;
                    }}
                    .metric-value {{
                        font-size: 2em;
                        font-weight: bold;
                        margin: 10px 0;
                    }}
                    .recommendations {{
                        background-color: #e8f4f8;
                        border-left: 4px solid #007bff;
                        padding: 20px;
                        margin: 20px 0;
                    }}
                    .recommendation-item {{
                        margin: 10px 0;
                        padding: 10px;
                        background-color: white;
                        border-radius: 5px;
                    }}
                    .section {{
                        margin: 30px 0;
                        padding: 20px;
                        border: 1px solid #ddd;
                        border-radius: 5px;
                    }}
                </style>
            </head>
            <body>
                <div class="container">
                    <h1>Performance Monitoring Report</h1>
                    <p><strong>Period:</strong> {report.get('period_hours', 'N/A')} hours</p>
                    <p><strong>Generated:</strong> {report.get('generated_at', 'N/A')}</p>
                    
            """
            
            # System analysis section
            system_analysis = report.get('system_analysis', {})
            if system_analysis:
                html_content += """
                    <div class="section">
                        <h2>System Performance Analysis</h2>
                        <div class="metric-grid">
                """
                
                cpu_analysis = system_analysis.get('cpu', {})
                if cpu_analysis:
                    html_content += f"""
                        <div class="metric-card">
                            <h3>CPU Usage</h3>
                            <div class="metric-value">{cpu_analysis.get('average', 0):.1f}%</div>
                            <p>Average</p>
                        </div>
                    """
                
                memory_analysis = system_analysis.get('memory', {})
                if memory_analysis:
                    html_content += f"""
                        <div class="metric-card">
                            <h3>Memory Usage</h3>
                            <div class="metric-value">{memory_analysis.get('average', 0):.1f}%</div>
                            <p>Average</p>
                        </div>
                    """
                
                load_analysis = system_analysis.get('load_average', {})
                if load_analysis:
                    html_content += f"""
                        <div class="metric-card">
                            <h3>Load Average</h3>
                            <div class="metric-value">{load_analysis.get('average', 0):.2f}</div>
                            <p>1-minute average</p>
                        </div>
                    """
                
                html_content += """
                        </div>
                    </div>
                """
            
            # Recommendations section
            recommendations = report.get('recommendations', [])
            if recommendations:
                html_content += f"""
                    <div class="recommendations">
                        <h2>Performance Recommendations</h2>
                """
                
                for i, recommendation in enumerate(recommendations, 1):
                    html_content += f"""
                        <div class="recommendation-item">
                            <strong>{i}.</strong> {recommendation}
                        </div>
                    """
                
                html_content += """
                    </div>
                """
            
            html_content += """
                </div>
            </body>
            </html>
            """
            
            return html_content
            
        except Exception as e:
            print(f"Error creating HTML report: {e}")
            return f"<html><body><h1>Error generating report: {e}</h1></body></html>"

if __name__ == "__main__":
    # Example usage
    generator = ReportGenerator()
    
    # Generate reports in different formats
    generator.generate_pdf_report(24, 'sample_report.pdf')
    generator.generate_html_report(24, 'sample_report.html')
    generator.generate_csv_report(24, 'sample_report.csv')