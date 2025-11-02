#!/usr/bin/env python3
"""
Stress test reporting module
Generates comprehensive reports and analysis from stress test results
"""

import os
import sys
import json
import csv
import time
import base64
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Any, Optional
import matplotlib.pyplot as plt
import matplotlib.patches as patches
from matplotlib.backends.backend_pdf import PdfPages
import pandas as pd
import seaborn as sns
import numpy as np
from jinja2 import Template


class StressReportGenerator:
    """Advanced stress test reporting and analysis module"""
    
    def __init__(self, config):
        self.config = config
        self.output_dir = Path(config.output_dir)
        self.reports_dir = self.output_dir / "reports"
        self.charts_dir = self.output_dir / "charts"
        self.reports_dir.mkdir(parents=True, exist_ok=True)
        self.charts_dir.mkdir(parents=True, exist_ok=True)
        
        # Set matplotlib backend for headless operation
        plt.switch_backend('Agg')
        sns.set_style("whitegrid")
    
    def generate_html_report(self, report_data: Dict[str, Any]) -> str:
        """Generate comprehensive HTML report"""
        try:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            report_file = self.reports_dir / f"stress_test_report_{timestamp}.html"
            
            # Generate charts
            self._generate_performance_charts(report_data)
            self._generate_category_charts(report_data)
            self._generate_resource_usage_charts(report_data)
            
            # HTML report template
            html_template = """
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Comprehensive Stress Test Report</title>
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background-color: #f5f5f5;
        }
        .container {
            background-color: white;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
        }
        .header {
            text-align: center;
            border-bottom: 3px solid #4CAF50;
            padding-bottom: 20px;
            margin-bottom: 30px;
        }
        .header h1 {
            color: #2c3e50;
            margin-bottom: 10px;
        }
        .timestamp {
            color: #7f8c8d;
            font-size: 14px;
        }
        .summary {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }
        .summary-card {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 20px;
            border-radius: 10px;
            text-align: center;
            box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
        }
        .summary-card h3 {
            margin: 0 0 10px 0;
            font-size: 18px;
        }
        .summary-card .value {
            font-size: 32px;
            font-weight: bold;
            margin: 10px 0;
        }
        .section {
            margin-bottom: 40px;
        }
        .section h2 {
            color: #2c3e50;
            border-left: 4px solid #3498db;
            padding-left: 15px;
            margin-bottom: 20px;
        }
        .test-results {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
        }
        .test-card {
            border: 1px solid #ddd;
            border-radius: 8px;
            padding: 15px;
            background-color: #fafafa;
        }
        .test-card h4 {
            margin: 0 0 10px 0;
            color: #2c3e50;
        }
        .status {
            padding: 5px 10px;
            border-radius: 20px;
            color: white;
            font-size: 12px;
            font-weight: bold;
            display: inline-block;
            margin-bottom: 10px;
        }
        .status.PASS { background-color: #27ae60; }
        .status.FAIL { background-color: #e74c3c; }
        .status.PARTIAL { background-color: #f39c12; }
        .status.ERROR { background-color: #9b59b6; }
        .status.SKIPPED { background-color: #95a5a6; }
        .metrics {
            font-size: 14px;
            color: #555;
        }
        .chart-container {
            text-align: center;
            margin: 20px 0;
            padding: 15px;
            background-color: #fafafa;
            border-radius: 8px;
        }
        .chart-container img {
            max-width: 100%;
            height: auto;
            border-radius: 5px;
            box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
        }
        .recommendations {
            background-color: #e8f4fd;
            border-left: 4px solid #3498db;
            padding: 20px;
            margin: 20px 0;
            border-radius: 5px;
        }
        .recommendations h3 {
            color: #2980b9;
            margin-top: 0;
        }
        .recommendations ul {
            margin: 10px 0;
            padding-left: 20px;
        }
        .recommendations li {
            margin: 8px 0;
        }
        .system-info {
            background-color: #f8f9fa;
            border: 1px solid #dee2e6;
            border-radius: 8px;
            padding: 20px;
            margin: 20px 0;
        }
        .system-info h3 {
            color: #495057;
            margin-top: 0;
        }
        .info-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 15px;
            margin-top: 15px;
        }
        .info-item {
            background-color: white;
            padding: 10px;
            border-radius: 5px;
            border: 1px solid #e9ecef;
        }
        .info-item strong {
            color: #495057;
        }
        .footer {
            text-align: center;
            margin-top: 40px;
            padding-top: 20px;
            border-top: 1px solid #ddd;
            color: #7f8c8d;
            font-size: 14px;
        }
        .error-list {
            background-color: #fdf2f2;
            border: 1px solid #f8d7da;
            border-radius: 5px;
            padding: 15px;
            margin: 10px 0;
        }
        .error-list h4 {
            color: #721c24;
            margin-top: 0;
        }
        .error-item {
            background-color: white;
            padding: 8px;
            margin: 5px 0;
            border-radius: 3px;
            border-left: 3px solid #f8d7da;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Comprehensive Stress Test Report</h1>
            <div class="timestamp">Generated on: {{ timestamp }}</div>
            {% if report_data.system_info %}
            <div style="margin-top: 15px;">
                <strong>System:</strong> {{ report_data.system_info.get('system', {}).get('hostname', 'Unknown') }} | 
                <strong>Platform:</strong> {{ report_data.system_info.get('system', {}).get('platform', 'Unknown') }}
            </div>
            {% endif %}
        </div>

        <div class="summary">
            <div class="summary-card">
                <h3>Total Tests</h3>
                <div class="value">{{ report_data.summary.total_tests }}</div>
            </div>
            <div class="summary-card">
                <h3>Success Rate</h3>
                <div class="value">{{ "%.1f"|format(report_data.summary.success_rate) }}%</div>
            </div>
            <div class="summary-card">
                <h3>Duration</h3>
                <div class="value">{{ "%.1f"|format(report_data.summary.total_duration) }}s</div>
            </div>
            <div class="summary-card">
                <h3>Passed</h3>
                <div class="value">{{ report_data.summary.passed_tests }}</div>
            </div>
        </div>

        {% if report_data.summary.failed_tests > 0 or report_data.summary.error_tests > 0 %}
        <div class="error-list">
            <h4>Issues Summary</h4>
            {% if report_data.summary.failed_tests > 0 %}
            <div class="error-item"><strong>Failed Tests:</strong> {{ report_data.summary.failed_tests }}</div>
            {% endif %}
            {% if report_data.summary.error_tests > 0 %}
            <div class="error-item"><strong>Error Tests:</strong> {{ report_data.summary.error_tests }}</div>
            {% endif %}
            {% if report_data.summary.timeout_tests > 0 %}
            <div class="error-item"><strong>Timeout Tests:</strong> {{ report_data.summary.timeout_tests }}</div>
            {% endif %}
        </div>
        {% endif %}

        <div class="section">
            <h2>Performance Overview</h2>
            <div class="chart-container">
                <h4>Test Results by Category</h4>
                <img src="../charts/category_performance_{{ timestamp }}.png" alt="Category Performance">
            </div>
            <div class="chart-container">
                <h4>Test Results Distribution</h4>
                <img src="../charts/test_distribution_{{ timestamp }}.png" alt="Test Distribution">
            </div>
        </div>

        <div class="section">
            <h2>Resource Usage Analysis</h2>
            <div class="chart-container">
                <h4>Memory Usage During Tests</h4>
                <img src="../charts/memory_usage_{{ timestamp }}.png" alt="Memory Usage">
            </div>
            <div class="chart-container">
                <h4>CPU Usage During Tests</h4>
                <img src="../charts/cpu_usage_{{ timestamp }}.png" alt="CPU Usage">
            </div>
        </div>

        <div class="section">
            <h2>Detailed Test Results</h2>
            <div class="test-results">
                {% for result in report_data.test_results %}
                <div class="test-card">
                    <h4>{{ result.test_name }}</h4>
                    <span class="status {{ result.status }}">{{ result.status }}</span>
                    <div class="metrics">
                        <div><strong>Category:</strong> {{ result.category }}</div>
                        <div><strong>Duration:</strong> {{ "%.2f"|format(result.duration) }}s</div>
                        {% if result.errors %}
                        <div><strong>Errors:</strong> {{ result.errors|length }}</div>
                        {% endif %}
                        {% if result.warnings %}
                        <div><strong>Warnings:</strong> {{ result.warnings|length }}</div>
                        {% endif %}
                        {% if result.memory_usage.avg_percent %}
                        <div><strong>Avg Memory:</strong> {{ "%.1f"|format(result.memory_usage.avg_percent) }}%</div>
                        {% endif %}
                        {% if result.cpu_usage.avg_percent %}
                        <div><strong>Avg CPU:</strong> {{ "%.1f"|format(result.cpu_usage.avg_percent) }}%</div>
                        {% endif %}
                    </div>
                    {% if result.errors %}
                    <div style="margin-top: 10px; font-size: 12px; color: #e74c3c;">
                        <strong>Errors:</strong>
                        <ul>
                            {% for error in result.errors[:3] %}
                            <li>{{ error }}</li>
                            {% endfor %}
                        </ul>
                    </div>
                    {% endif %}
                </div>
                {% endfor %}
            </div>
        </div>

        {% if report_data.recommendations %}
        <div class="recommendations">
            <h3>Recommendations</h3>
            <ul>
                {% for recommendation in report_data.recommendations %}
                <li>{{ recommendation }}</li>
                {% endfor %}
            </ul>
        </div>
        {% endif %}

        {% if report_data.system_info %}
        <div class="system-info">
            <h3>System Information</h3>
            <div class="info-grid">
                {% if report_data.system_info.memory %}
                <div class="info-item">
                    <strong>Memory:</strong><br>
                    Total: {{ "%.1f"|format(report_data.system_info.memory.total_gb) }} GB<br>
                    Available: {{ "%.1f"|format(report_data.system_info.memory.available_gb) }} GB<br>
                    Usage: {{ "%.1f"|format(report_data.system_info.memory.percent) }}%
                </div>
                {% endif %}
                {% if report_data.system_info.cpu %}
                <div class="info-item">
                    <strong>CPU:</strong><br>
                    Cores: {{ report_data.system_info.cpu.count }}<br>
                    Frequency: {{ "%.0f"|format(report_data.system_info.cpu.frequency_mhz) }} MHz
                </div>
                {% endif %}
                {% if report_data.system_info.disk %}
                <div class="info-item">
                    <strong>Disk:</strong><br>
                    Total: {{ "%.1f"|format(report_data.system_info.disk.total_gb) }} GB<br>
                    Free: {{ "%.1f"|format(report_data.system_info.disk.free_gb) }} GB<br>
                    Usage: {{ "%.1f"|format(report_data.system_info.disk.percent) }}%
                </div>
                {% endif %}
                <div class="info-item">
                    <strong>Uptime:</strong><br>
                    {{ "%.1f"|format(report_data.system_info.uptime_seconds / 3600) }} hours
                </div>
            </div>
        </div>
        {% endif %}

        <div class="footer">
            <p>Generated by Comprehensive Stress Testing Suite | Report ID: {{ timestamp }}</p>
        </div>
    </div>
</body>
</html>
            """
            
            # Render HTML
            template = Template(html_template)
            html_content = template.render(
                timestamp=datetime.fromtimestamp(report_data["summary"]["start_time"]).strftime("%Y-%m-%d %H:%M:%S"),
                report_data=report_data
            )
            
            # Write HTML file
            with open(report_file, 'w', encoding='utf-8') as f:
                f.write(html_content)
            
            return str(report_file)
            
        except Exception as e:
            print(f"Error generating HTML report: {str(e)}")
            return ""
    
    def generate_json_report(self, report_data: Dict[str, Any]) -> str:
        """Generate detailed JSON report"""
        try:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            report_file = self.reports_dir / f"stress_test_report_{timestamp}.json"
            
            # Enhanced JSON report with additional analysis
            enhanced_report = {
                "report_metadata": {
                    "timestamp": datetime.now().isoformat(),
                    "report_version": "1.0",
                    "generator": "Comprehensive Stress Testing Suite"
                },
                "summary": report_data["summary"],
                "test_results": report_data["test_results"],
                "system_info": report_data.get("system_info", {}),
                "recommendations": report_data.get("recommendations", []),
                "analysis": self._generate_detailed_analysis(report_data),
                "performance_metrics": self._extract_performance_metrics(report_data),
                "resource_analysis": self._analyze_resource_usage(report_data)
            }
            
            # Write JSON file
            with open(report_file, 'w', encoding='utf-8') as f:
                json.dump(enhanced_report, f, indent=2, default=str)
            
            return str(report_file)
            
        except Exception as e:
            print(f"Error generating JSON report: {str(e)}")
            return ""
    
    def generate_csv_report(self, report_data: Dict[str, Any]) -> str:
        """Generate CSV report for spreadsheet analysis"""
        try:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            report_file = self.reports_dir / f"stress_test_report_{timestamp}.csv"
            
            # Prepare data for CSV
            csv_data = []
            for result in report_data["test_results"]:
                csv_row = {
                    "test_name": result["test_name"],
                    "category": result["category"],
                    "status": result["status"],
                    "duration_seconds": result["duration"],
                    "start_time": datetime.fromtimestamp(result["start_time"]).isoformat(),
                    "end_time": datetime.fromtimestamp(result["end_time"]).isoformat(),
                    "errors_count": len(result.get("errors", [])),
                    "warnings_count": len(result.get("warnings", [])),
                    "avg_memory_percent": result.get("memory_usage", {}).get("avg_percent", 0),
                    "max_memory_percent": result.get("memory_usage", {}).get("max_percent", 0),
                    "avg_cpu_percent": result.get("cpu_usage", {}).get("avg_percent", 0),
                    "max_cpu_percent": result.get("cpu_usage", {}).get("max_percent", 0),
                    "avg_disk_percent": result.get("disk_usage", {}).get("avg_percent", 0),
                    "max_disk_percent": result.get("disk_usage", {}).get("max_percent", 0)
                }
                csv_data.append(csv_row)
            
            # Write CSV file
            with open(report_file, 'w', newline='', encoding='utf-8') as f:
                if csv_data:
                    writer = csv.DictWriter(f, fieldnames=csv_data[0].keys())
                    writer.writeheader()
                    writer.writerows(csv_data)
            
            return str(report_file)
            
        except Exception as e:
            print(f"Error generating CSV report: {str(e)}")
            return ""
    
    def generate_pdf_report(self, report_data: Dict[str, Any]) -> str:
        """Generate PDF report with charts and analysis"""
        try:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            report_file = self.reports_dir / f"stress_test_report_{timestamp}.pdf"
            
            # Generate charts first
            self._generate_performance_charts(report_data)
            self._generate_category_charts(report_data)
            self._generate_resource_usage_charts(report_data)
            
            with PdfPages(str(report_file)) as pdf:
                # Cover page
                self._create_cover_page(pdf, report_data)
                
                # Summary page
                self._create_summary_page(pdf, report_data)
                
                # Test results page
                self._create_test_results_page(pdf, report_data)
                
                # Performance charts page
                self._create_charts_page(pdf, report_data)
                
                # Recommendations page
                self._create_recommendations_page(pdf, report_data)
            
            return str(report_file)
            
        except Exception as e:
            print(f"Error generating PDF report: {str(e)}")
            return ""
    
    def _generate_performance_charts(self, report_data: Dict[str, Any]):
        """Generate performance-related charts"""
        try:
            # Test duration chart
            fig, axes = plt.subplots(2, 2, figsize=(15, 12))
            fig.suptitle('Performance Analysis', fontsize=16, fontweight='bold')
            
            # Test duration by category
            categories = {}
            durations = {}
            for result in report_data["test_results"]:
                cat = result["category"]
                if cat not in categories:
                    categories[cat] = []
                    durations[cat] = []
                categories[cat].append(result["test_name"])
                durations[cat].append(result["duration"])
            
            # Duration by category
            ax1 = axes[0, 0]
            for i, (cat, durs) in enumerate(durations.items()):
                ax1.bar([f"{cat}_{j}" for j in range(len(durs))], durs, 
                       alpha=0.7, label=cat)
            ax1.set_title('Test Duration by Category')
            ax1.set_ylabel('Duration (seconds)')
            ax1.tick_params(axis='x', rotation=45)
            
            # Status distribution pie chart
            ax2 = axes[0, 1]
            status_counts = {}
            for result in report_data["test_results"]:
                status = result["status"]
                status_counts[status] = status_counts.get(status, 0) + 1
            
            if status_counts:
                colors = {'PASS': '#2ecc71', 'FAIL': '#e74c3c', 'PARTIAL': '#f39c12', 
                         'ERROR': '#9b59b6', 'SKIPPED': '#95a5a6'}
                ax2.pie(status_counts.values(), labels=status_counts.keys(), 
                       colors=[colors.get(status, '#34495e') for status in status_counts.keys()],
                       autopct='%1.1f%%', startangle=90)
            ax2.set_title('Test Status Distribution')
            
            # Memory usage over time (if available)
            ax3 = axes[1, 0]
            memory_data = []
            test_names = []
            for result in report_data["test_results"]:
                if result.get("memory_usage", {}).get("avg_percent"):
                    memory_data.append(result["memory_usage"]["avg_percent"])
                    test_names.append(result["test_name"][:20])
            
            if memory_data:
                ax3.plot(range(len(memory_data)), memory_data, 'b-', marker='o')
                ax3.set_title('Average Memory Usage by Test')
                ax3.set_ylabel('Memory Usage (%)')
                ax3.set_xlabel('Test Index')
                ax3.tick_params(axis='x', rotation=45)
            
            # CPU usage over time (if available)
            ax4 = axes[1, 1]
            cpu_data = []
            for result in report_data["test_results"]:
                if result.get("cpu_usage", {}).get("avg_percent"):
                    cpu_data.append(result["cpu_usage"]["avg_percent"])
            
            if cpu_data:
                ax4.plot(range(len(cpu_data)), cpu_data, 'r-', marker='s')
                ax4.set_title('Average CPU Usage by Test')
                ax4.set_ylabel('CPU Usage (%)')
                ax4.set_xlabel('Test Index')
                ax4.tick_params(axis='x', rotation=45)
            
            plt.tight_layout()
            
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            chart_file = self.charts_dir / f"performance_charts_{timestamp}.png"
            plt.savefig(chart_file, dpi=300, bbox_inches='tight')
            plt.close()
            
        except Exception as e:
            print(f"Error generating performance charts: {str(e)}")
    
    def _generate_category_charts(self, report_data: Dict[str, Any]):
        """Generate category-specific charts"""
        try:
            fig, axes = plt.subplots(2, 2, figsize=(15, 12))
            fig.suptitle('Category Analysis', fontsize=16, fontweight='bold')
            
            # Test count by category
            ax1 = axes[0, 0]
            category_counts = {}
            for result in report_data["test_results"]:
                cat = result["category"]
                category_counts[cat] = category_counts.get(cat, 0) + 1
            
            if category_counts:
                bars = ax1.bar(category_counts.keys(), category_counts.values(), 
                              color='skyblue', alpha=0.8)
                ax1.set_title('Test Count by Category')
                ax1.set_ylabel('Number of Tests')
                ax1.tick_params(axis='x', rotation=45)
                
                # Add value labels on bars
                for bar in bars:
                    height = bar.get_height()
                    ax1.text(bar.get_x() + bar.get_width()/2., height,
                            f'{int(height)}', ha='center', va='bottom')
            
            # Success rate by category
            ax2 = axes[0, 1]
            category_success = {}
            for result in report_data["test_results"]:
                cat = result["category"]
                if cat not in category_success:
                    category_success[cat] = {"total": 0, "passed": 0}
                category_success[cat]["total"] += 1
                if result["status"] == "PASS":
                    category_success[cat]["passed"] += 1
            
            if category_success:
                categories = list(category_success.keys())
                success_rates = [category_success[cat]["passed"]/category_success[cat]["total"]*100 
                               for cat in categories]
                
                bars = ax2.bar(categories, success_rates, color='lightgreen', alpha=0.8)
                ax2.set_title('Success Rate by Category')
                ax2.set_ylabel('Success Rate (%)')
                ax2.tick_params(axis='x', rotation=45)
                ax2.set_ylim(0, 100)
                
                # Add value labels on bars
                for bar, rate in zip(bars, success_rates):
                    height = bar.get_height()
                    ax2.text(bar.get_x() + bar.get_width()/2., height,
                            f'{rate:.1f}%', ha='center', va='bottom')
            
            # Average duration by category
            ax3 = axes[1, 0]
            category_duration = {}
            for result in report_data["test_results"]:
                cat = result["category"]
                if cat not in category_duration:
                    category_duration[cat] = []
                category_duration[cat].append(result["duration"])
            
            if category_duration:
                categories = list(category_duration.keys())
                avg_durations = [sum(durs)/len(durs) for durs in category_duration.values()]
                
                bars = ax3.bar(categories, avg_durations, color='orange', alpha=0.8)
                ax3.set_title('Average Duration by Category')
                ax3.set_ylabel('Average Duration (seconds)')
                ax3.tick_params(axis='x', rotation=45)
                
                # Add value labels on bars
                for bar, duration in zip(bars, avg_durations):
                    height = bar.get_height()
                    ax3.text(bar.get_x() + bar.get_width()/2., height,
                            f'{duration:.2f}s', ha='center', va='bottom')
            
            # Memory usage by category
            ax4 = axes[1, 1]
            category_memory = {}
            for result in report_data["test_results"]:
                cat = result["category"]
                if cat not in category_memory:
                    category_memory[cat] = []
                if result.get("memory_usage", {}).get("avg_percent"):
                    category_memory[cat].append(result["memory_usage"]["avg_percent"])
            
            if category_memory:
                categories = list(category_memory.keys())
                avg_memory = [sum(mem)/len(mem) for mem in category_memory.values()]
                
                bars = ax4.bar(categories, avg_memory, color='lightcoral', alpha=0.8)
                ax4.set_title('Average Memory Usage by Category')
                ax4.set_ylabel('Average Memory Usage (%)')
                ax4.tick_params(axis='x', rotation=45)
                
                # Add value labels on bars
                for bar, memory in zip(bars, avg_memory):
                    height = bar.get_height()
                    ax4.text(bar.get_x() + bar.get_width()/2., height,
                            f'{memory:.1f}%', ha='center', va='bottom')
            
            plt.tight_layout()
            
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            chart_file = self.charts_dir / f"category_performance_{timestamp}.png"
            plt.savefig(chart_file, dpi=300, bbox_inches='tight')
            plt.close()
            
            # Generate test distribution chart
            fig, ax = plt.subplots(1, 1, figsize=(10, 8))
            
            # Status distribution with detailed breakdown
            status_counts = {}
            for result in report_data["test_results"]:
                status = result["status"]
                status_counts[status] = status_counts.get(status, 0) + 1
            
            if status_counts:
                colors = ['#2ecc71', '#e74c3c', '#f39c12', '#9b59b6', '#95a5a6']
                status_list = list(status_counts.keys())
                color_map = {status: colors[i % len(colors)] for i, status in enumerate(status_list)}
                
                wedges, texts, autotexts = ax.pie(
                    status_counts.values(),
                    labels=status_counts.keys(),
                    colors=[color_map[status] for status in status_counts.keys()],
                    autopct='%1.1f%%',
                    startangle=90,
                    explode=[0.05] * len(status_counts)
                )
                
                ax.set_title('Test Results Distribution', fontsize=16, fontweight='bold')
                
                # Enhance text
                for autotext in autotexts:
                    autotext.set_color('white')
                    autotext.set_fontweight('bold')
            
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            chart_file = self.charts_dir / f"test_distribution_{timestamp}.png"
            plt.savefig(chart_file, dpi=300, bbox_inches='tight')
            plt.close()
            
        except Exception as e:
            print(f"Error generating category charts: {str(e)}")
    
    def _generate_resource_usage_charts(self, report_data: Dict[str, Any]):
        """Generate resource usage analysis charts"""
        try:
            fig, axes = plt.subplots(2, 2, figsize=(15, 12))
            fig.suptitle('Resource Usage Analysis', fontsize=16, fontweight='bold')
            
            # Memory usage timeline
            ax1 = axes[0, 0]
            memory_points = []
            test_names = []
            
            for result in report_data["test_results"]:
                if result.get("memory_usage", {}).get("avg_percent"):
                    memory_points.append(result["memory_usage"]["avg_percent"])
                    test_names.append(result["test_name"][:15] + "..." if len(result["test_name"]) > 15 else result["test_name"])
            
            if memory_points:
                ax1.plot(range(len(memory_points)), memory_points, 'b-', marker='o', linewidth=2, markersize=6)
                ax1.fill_between(range(len(memory_points)), memory_points, alpha=0.3)
                ax1.set_title('Memory Usage Timeline')
                ax1.set_ylabel('Memory Usage (%)')
                ax1.set_xlabel('Test Sequence')
                ax1.grid(True, alpha=0.3)
                
                # Rotate x-axis labels for better readability
                ax1.tick_params(axis='x', rotation=45)
            
            # CPU usage timeline
            ax2 = axes[0, 1]
            cpu_points = []
            
            for result in report_data["test_results"]:
                if result.get("cpu_usage", {}).get("avg_percent"):
                    cpu_points.append(result["cpu_usage"]["avg_percent"])
            
            if cpu_points:
                ax2.plot(range(len(cpu_points)), cpu_points, 'r-', marker='s', linewidth=2, markersize=6)
                ax2.fill_between(range(len(cpu_points)), cpu_points, alpha=0.3)
                ax2.set_title('CPU Usage Timeline')
                ax2.set_ylabel('CPU Usage (%)')
                ax2.set_xlabel('Test Sequence')
                ax2.grid(True, alpha=0.3)
            
            # Memory vs CPU correlation
            ax3 = axes[1, 0]
            memory_data = []
            cpu_data = []
            
            for result in report_data["test_results"]:
                if (result.get("memory_usage", {}).get("avg_percent") and 
                    result.get("cpu_usage", {}).get("avg_percent")):
                    memory_data.append(result["memory_usage"]["avg_percent"])
                    cpu_data.append(result["cpu_usage"]["avg_percent"])
            
            if memory_data and cpu_data:
                scatter = ax3.scatter(cpu_data, memory_data, alpha=0.6, s=50, c=range(len(memory_data)), cmap='viridis')
                ax3.set_title('Memory vs CPU Usage Correlation')
                ax3.set_xlabel('CPU Usage (%)')
                ax3.set_ylabel('Memory Usage (%)')
                ax3.grid(True, alpha=0.3)
                
                # Add correlation line
                if len(memory_data) > 1:
                    z = np.polyfit(cpu_data, memory_data, 1)
                    p = np.poly1d(z)
                    ax3.plot(cpu_data, p(cpu_data), "r--", alpha=0.8, linewidth=2)
            
            # Resource usage distribution
            ax4 = axes[1, 1]
            if memory_data and cpu_data:
                data_for_box = [memory_data, cpu_data]
                labels = ['Memory (%)', 'CPU (%)']
                
                box_plot = ax4.boxplot(data_for_box, labels=labels, patch_artist=True)
                colors = ['lightblue', 'lightcoral']
                
                for patch, color in zip(box_plot['boxes'], colors):
                    patch.set_facecolor(color)
                    patch.set_alpha(0.7)
                
                ax4.set_title('Resource Usage Distribution')
                ax4.set_ylabel('Usage (%)')
                ax4.grid(True, alpha=0.3)
            
            plt.tight_layout()
            
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            chart_file = self.charts_dir / f"resource_usage_{timestamp}.png"
            plt.savefig(chart_file, dpi=300, bbox_inches='tight')
            plt.close()
            
            # Individual resource charts
            if memory_points:
                fig, ax = plt.subplots(1, 1, figsize=(12, 6))
                
                ax.plot(range(len(memory_points)), memory_points, 'b-', marker='o', linewidth=2, markersize=4)
                ax.fill_between(range(len(memory_points)), memory_points, alpha=0.3)
                ax.set_title('Memory Usage During Tests', fontsize=14, fontweight='bold')
                ax.set_ylabel('Memory Usage (%)')
                ax.set_xlabel('Test Sequence')
                ax.grid(True, alpha=0.3)
                
                # Add threshold lines
                ax.axhline(y=80, color='orange', linestyle='--', alpha=0.7, label='Warning (80%)')
                ax.axhline(y=90, color='red', linestyle='--', alpha=0.7, label='Critical (90%)')
                ax.legend()
                
                timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
                chart_file = self.charts_dir / f"memory_usage_{timestamp}.png"
                plt.savefig(chart_file, dpi=300, bbox_inches='tight')
                plt.close()
            
            if cpu_points:
                fig, ax = plt.subplots(1, 1, figsize=(12, 6))
                
                ax.plot(range(len(cpu_points)), cpu_points, 'r-', marker='s', linewidth=2, markersize=4)
                ax.fill_between(range(len(cpu_points)), cpu_points, alpha=0.3)
                ax.set_title('CPU Usage During Tests', fontsize=14, fontweight='bold')
                ax.set_ylabel('CPU Usage (%)')
                ax.set_xlabel('Test Sequence')
                ax.grid(True, alpha=0.3)
                
                # Add threshold lines
                ax.axhline(y=70, color='orange', linestyle='--', alpha=0.7, label='Warning (70%)')
                ax.axhline(y=90, color='red', linestyle='--', alpha=0.7, label='Critical (90%)')
                ax.legend()
                
                timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
                chart_file = self.charts_dir / f"cpu_usage_{timestamp}.png"
                plt.savefig(chart_file, dpi=300, bbox_inches='tight')
                plt.close()
            
        except Exception as e:
            print(f"Error generating resource usage charts: {str(e)}")
    
    def _generate_detailed_analysis(self, report_data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate detailed analysis of test results"""
        analysis = {
            "overall_performance": {},
            "test_category_analysis": {},
            "resource_efficiency": {},
            "failure_analysis": {},
            "trends": {}
        }
        
        try:
            # Overall performance analysis
            total_tests = report_data["summary"]["total_tests"]
            success_rate = report_data["summary"]["success_rate"]
            total_duration = report_data["summary"]["total_duration"]
            
            analysis["overall_performance"] = {
                "test_throughput": total_tests / total_duration if total_duration > 0 else 0,
                "average_test_duration": total_duration / total_tests if total_tests > 0 else 0,
                "performance_grade": self._calculate_performance_grade(success_rate, total_duration),
                "stability_score": self._calculate_stability_score(report_data["test_results"])
            }
            
            # Test category analysis
            categories = {}
            for result in report_data["test_results"]:
                cat = result["category"]
                if cat not in categories:
                    categories[cat] = {"total": 0, "passed": 0, "total_duration": 0, "avg_memory": [], "avg_cpu": []}
                
                categories[cat]["total"] += 1
                categories[cat]["total_duration"] += result["duration"]
                
                if result["status"] == "PASS":
                    categories[cat]["passed"] += 1
                
                if result.get("memory_usage", {}).get("avg_percent"):
                    categories[cat]["avg_memory"].append(result["memory_usage"]["avg_percent"])
                
                if result.get("cpu_usage", {}).get("avg_percent"):
                    categories[cat]["avg_cpu"].append(result["cpu_usage"]["avg_percent"])
            
            # Calculate category metrics
            for cat, data in categories.items():
                data["success_rate"] = (data["passed"] / data["total"] * 100) if data["total"] > 0 else 0
                data["avg_duration"] = data["total_duration"] / data["total"] if data["total"] > 0 else 0
                data["avg_memory_usage"] = sum(data["avg_memory"]) / len(data["avg_memory"]) if data["avg_memory"] else 0
                data["avg_cpu_usage"] = sum(data["avg_cpu"]) / len(data["avg_cpu"]) if data["avg_cpu"] else 0
            
            analysis["test_category_analysis"] = categories
            
            # Resource efficiency analysis
            memory_efficiency = []
            cpu_efficiency = []
            
            for result in report_data["test_results"]:
                # Calculate efficiency as work done per resource used
                duration = result["duration"]
                if duration > 0:
                    work_per_second = 1.0 / duration  # Simplified work metric
                    
                    if result.get("memory_usage", {}).get("avg_percent"):
                        memory_eff = work_per_second / (result["memory_usage"]["avg_percent"] / 100)
                        memory_efficiency.append(memory_eff)
                    
                    if result.get("cpu_usage", {}).get("avg_percent"):
                        cpu_eff = work_per_second / (result["cpu_usage"]["avg_percent"] / 100)
                        cpu_efficiency.append(cpu_eff)
            
            analysis["resource_efficiency"] = {
                "memory_efficiency_score": sum(memory_efficiency) / len(memory_efficiency) if memory_efficiency else 0,
                "cpu_efficiency_score": sum(cpu_efficiency) / len(cpu_efficiency) if cpu_efficiency else 0,
                "overall_efficiency": (sum(memory_efficiency + cpu_efficiency) / 
                                     len(memory_efficiency + cpu_efficiency) if (memory_efficiency + cpu_efficiency) else 0)
            }
            
            # Failure analysis
            failures = [r for r in report_data["test_results"] if r["status"] in ["FAIL", "ERROR"]]
            
            analysis["failure_analysis"] = {
                "total_failures": len(failures),
                "failure_rate": len(failures) / total_tests * 100 if total_tests > 0 else 0,
                "failure_by_category": {},
                "common_error_patterns": self._analyze_error_patterns(failures)
            }
            
            # Failure by category
            for failure in failures:
                cat = failure["category"]
                analysis["failure_analysis"]["failure_by_category"][cat] = \
                    analysis["failure_analysis"]["failure_by_category"].get(cat, 0) + 1
            
            # Trends analysis
            analysis["trends"] = self._analyze_trends(report_data["test_results"])
            
        except Exception as e:
            print(f"Error generating detailed analysis: {str(e)}")
        
        return analysis
    
    def _extract_performance_metrics(self, report_data: Dict[str, Any]) -> Dict[str, Any]:
        """Extract key performance metrics"""
        metrics = {
            "response_times": [],
            "throughput": {},
            "resource_utilization": {},
            "scalability_indicators": {}
        }
        
        try:
            # Response times (test durations)
            durations = [r["duration"] for r in report_data["test_results"]]
            metrics["response_times"] = {
                "min": min(durations) if durations else 0,
                "max": max(durations) if durations else 0,
                "avg": sum(durations) / len(durations) if durations else 0,
                "p95": np.percentile(durations, 95) if durations else 0,
                "p99": np.percentile(durations, 99) if durations else 0
            }
            
            # Throughput by category
            category_throughput = {}
            for result in report_data["test_results"]:
                cat = result["category"]
                if cat not in category_throughput:
                    category_throughput[cat] = []
                category_throughput[cat].append(1.0 / result["duration"] if result["duration"] > 0 else 0)
            
            for cat, throughputs in category_throughput.items():
                metrics["throughput"][cat] = {
                    "avg_throughput": sum(throughputs) / len(throughputs),
                    "max_throughput": max(throughputs),
                    "min_throughput": min(throughputs)
                }
            
            # Resource utilization statistics
            memory_usage = []
            cpu_usage = []
            
            for result in report_data["test_results"]:
                if result.get("memory_usage", {}).get("avg_percent"):
                    memory_usage.append(result["memory_usage"]["avg_percent"])
                
                if result.get("cpu_usage", {}).get("avg_percent"):
                    cpu_usage.append(result["cpu_usage"]["avg_percent"])
            
            metrics["resource_utilization"] = {
                "memory": {
                    "avg": sum(memory_usage) / len(memory_usage) if memory_usage else 0,
                    "max": max(memory_usage) if memory_usage else 0,
                    "peak_sustained": np.percentile(memory_usage, 90) if memory_usage else 0
                },
                "cpu": {
                    "avg": sum(cpu_usage) / len(cpu_usage) if cpu_usage else 0,
                    "max": max(cpu_usage) if cpu_usage else 0,
                    "peak_sustained": np.percentile(cpu_usage, 90) if cpu_usage else 0
                }
            }
            
        except Exception as e:
            print(f"Error extracting performance metrics: {str(e)}")
        
        return metrics
    
    def _analyze_resource_usage(self, report_data: Dict[str, Any]) -> Dict[str, Any]:
        """Analyze resource usage patterns"""
        resource_analysis = {
            "memory_patterns": {},
            "cpu_patterns": {},
            "resource_correlation": {},
            "optimization_opportunities": []
        }
        
        try:
            # Memory patterns
            memory_data = []
            for result in report_data["test_results"]:
                if result.get("memory_usage", {}).get("avg_percent"):
                    memory_data.append({
                        "test": result["test_name"],
                        "category": result["category"],
                        "usage": result["memory_usage"]["avg_percent"],
                        "duration": result["duration"]
                    })
            
            if memory_data:
                memory_usages = [d["usage"] for d in memory_data]
                resource_analysis["memory_patterns"] = {
                    "baseline": np.percentile(memory_usages, 10),
                    "normal_range": (np.percentile(memory_usages, 25), np.percentile(memory_usages, 75)),
                    "stress_threshold": np.percentile(memory_usages, 90),
                    "critical_threshold": np.percentile(memory_usages, 95)
                }
            
            # CPU patterns
            cpu_data = []
            for result in report_data["test_results"]:
                if result.get("cpu_usage", {}).get("avg_percent"):
                    cpu_data.append({
                        "test": result["test_name"],
                        "category": result["category"],
                        "usage": result["cpu_usage"]["avg_percent"],
                        "duration": result["duration"]
                    })
            
            if cpu_data:
                cpu_usages = [d["usage"] for d in cpu_data]
                resource_analysis["cpu_patterns"] = {
                    "baseline": np.percentile(cpu_usages, 10),
                    "normal_range": (np.percentile(cpu_usages, 25), np.percentile(cpu_usages, 75)),
                    "stress_threshold": np.percentile(cpu_usages, 90),
                    "critical_threshold": np.percentile(cpu_usages, 95)
                }
            
            # Resource correlation
            paired_data = []
            for result in report_data["test_results"]:
                if (result.get("memory_usage", {}).get("avg_percent") and 
                    result.get("cpu_usage", {}).get("avg_percent")):
                    paired_data.append((result["memory_usage"]["avg_percent"], 
                                      result["cpu_usage"]["avg_percent"]))
            
            if len(paired_data) > 1:
                memory_vals = [d[0] for d in paired_data]
                cpu_vals = [d[1] for d in paired_data]
                correlation = np.corrcoef(memory_vals, cpu_vals)[0, 1]
                
                resource_analysis["resource_correlation"] = {
                    "memory_cpu_correlation": correlation,
                    "correlation_strength": "strong" if abs(correlation) > 0.7 else "moderate" if abs(correlation) > 0.4 else "weak"
                }
            
            # Optimization opportunities
            optimization_opportunities = []
            
            # High memory usage tests
            if memory_data:
                high_memory_tests = [d for d in memory_data if d["usage"] > 85]
                if high_memory_tests:
                    optimization_opportunities.append({
                        "type": "memory_optimization",
                        "description": f"High memory usage detected in {len(high_memory_tests)} tests",
                        "affected_tests": [d["test"] for d in high_memory_tests],
                        "recommendation": "Consider optimizing memory allocation patterns or increasing available memory"
                    })
            
            # High CPU usage tests
            if cpu_data:
                high_cpu_tests = [d for d in cpu_data if d["usage"] > 80]
                if high_cpu_tests:
                    optimization_opportunities.append({
                        "type": "cpu_optimization",
                        "description": f"High CPU usage detected in {len(high_cpu_tests)} tests",
                        "affected_tests": [d["test"] for d in high_cpu_tests],
                        "recommendation": "Consider optimizing CPU-intensive operations or adding more CPU cores"
                    })
            
            # Long-running tests
            long_tests = [r for r in report_data["test_results"] if r["duration"] > 30]
            if long_tests:
                optimization_opportunities.append({
                    "type": "performance_optimization",
                    "description": f"{len(long_tests)} tests took longer than 30 seconds",
                    "affected_tests": [r["test_name"] for r in long_tests],
                    "recommendation": "Review and optimize algorithms in long-running tests"
                })
            
            resource_analysis["optimization_opportunities"] = optimization_opportunities
            
        except Exception as e:
            print(f"Error analyzing resource usage: {str(e)}")
        
        return resource_analysis
    
    def _calculate_performance_grade(self, success_rate: float, duration: float) -> str:
        """Calculate overall performance grade"""
        if success_rate >= 95 and duration < 300:
            return "A+"
        elif success_rate >= 90 and duration < 600:
            return "A"
        elif success_rate >= 85:
            return "B"
        elif success_rate >= 75:
            return "C"
        elif success_rate >= 60:
            return "D"
        else:
            return "F"
    
    def _calculate_stability_score(self, test_results: List[Dict[str, Any]]) -> float:
        """Calculate system stability score"""
        if not test_results:
            return 0.0
        
        # Factors for stability calculation
        success_rate = sum(1 for r in test_results if r["status"] == "PASS") / len(test_results)
        
        # Check for consistent resource usage (low variance indicates stability)
        memory_variances = []
        cpu_variances = []
        
        for result in test_results:
            if result.get("memory_usage", {}).get("avg_percent"):
                # Simplified variance calculation
                memory_variances.append(result["memory_usage"]["avg_percent"])
            
            if result.get("cpu_usage", {}).get("avg_percent"):
                cpu_variances.append(result["cpu_usage"]["avg_percent"])
        
        memory_stability = 1.0 - (np.std(memory_variances) / 100) if memory_variances else 1.0
        cpu_stability = 1.0 - (np.std(cpu_variances) / 100) if cpu_variances else 1.0
        
        # Overall stability score (weighted average)
        stability_score = (success_rate * 0.5 + memory_stability * 0.25 + cpu_stability * 0.25)
        return max(0.0, min(1.0, stability_score))
    
    def _analyze_error_patterns(self, failures: List[Dict[str, Any]]) -> List[str]:
        """Analyze patterns in test failures"""
        patterns = []
        
        try:
            # Category-based failure patterns
            category_failures = {}
            for failure in failures:
                cat = failure["category"]
                category_failures[cat] = category_failures.get(cat, 0) + 1
            
            # Find categories with high failure rates
            if category_failures:
                max_failures = max(category_failures.values())
                for cat, count in category_failures.items():
                    if count == max_failures and count > 1:
                        patterns.append(f"Most failures occur in {cat} category ({count} failures)")
            
            # Error message patterns
            all_errors = []
            for failure in failures:
                all_errors.extend(failure.get("errors", []))
            
            if all_errors:
                # Simple pattern matching for common error types
                error_types = {
                    "Memory": sum(1 for error in all_errors if "memory" in error.lower()),
                    "Timeout": sum(1 for error in all_errors if "timeout" in error.lower()),
                    "Permission": sum(1 for error in all_errors if "permission" in error.lower()),
                    "Resource": sum(1 for error in all_errors if "resource" in error.lower())
                }
                
                for error_type, count in error_types.items():
                    if count > 0:
                        patterns.append(f"{error_type} related errors: {count} occurrences")
        
        except Exception as e:
            print(f"Error analyzing error patterns: {str(e)}")
        
        return patterns
    
    def _analyze_trends(self, test_results: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Analyze trends in test performance"""
        trends = {
            "performance_trend": "stable",
            "resource_trend": "stable",
            "reliability_trend": "stable"
        }
        
        try:
            if len(test_results) < 3:
                return trends
            
            # Sort by start time
            sorted_results = sorted(test_results, key=lambda x: x["start_time"])
            
            # Analyze duration trends
            durations = [r["duration"] for r in sorted_results]
            if len(durations) > 2:
                first_half = durations[:len(durations)//2]
                second_half = durations[len(durations)//2:]
                
                first_avg = sum(first_half) / len(first_half)
                second_avg = sum(second_half) / len(second_half)
                
                if second_avg > first_avg * 1.2:
                    trends["performance_trend"] = "degrading"
                elif second_avg < first_avg * 0.8:
                    trends["performance_trend"] = "improving"
            
            # Analyze resource trends
            memory_values = []
            cpu_values = []
            
            for result in sorted_results:
                if result.get("memory_usage", {}).get("avg_percent"):
                    memory_values.append(result["memory_usage"]["avg_percent"])
                if result.get("cpu_usage", {}).get("avg_percent"):
                    cpu_values.append(result["cpu_usage"]["avg_percent"])
            
            if len(memory_values) > 2:
                first_half_mem = memory_values[:len(memory_values)//2]
                second_half_mem = memory_values[len(memory_values)//2:]
                
                if sum(second_half_mem) / len(second_half_mem) > sum(first_half_mem) / len(first_half_mem) * 1.1:
                    trends["resource_trend"] = "increasing"
                elif sum(second_half_mem) / len(second_half_mem) < sum(first_half_mem) / len(first_half_mem) * 0.9:
                    trends["resource_trend"] = "decreasing"
            
            # Analyze reliability trends
            first_half_success = sum(1 for r in sorted_results[:len(sorted_results)//2] if r["status"] == "PASS")
            second_half_success = sum(1 for r in sorted_results[len(sorted_results)//2:] if r["status"] == "PASS")
            
            if len(sorted_results) > 2:
                if second_half_success < first_half_success:
                    trends["reliability_trend"] = "degrading"
                elif second_half_success > first_half_success:
                    trends["reliability_trend"] = "improving"
        
        except Exception as e:
            print(f"Error analyzing trends: {str(e)}")
        
        return trends
    
    def _create_cover_page(self, pdf, report_data: Dict[str, Any]):
        """Create PDF cover page"""
        fig, ax = plt.subplots(figsize=(8.5, 11))
        ax.set_xlim(0, 10)
        ax.set_ylim(0, 10)
        ax.axis('off')
        
        # Title
        ax.text(5, 8.5, 'Comprehensive Stress Test Report', 
                fontsize=24, fontweight='bold', ha='center', va='center')
        
        # Subtitle
        ax.text(5, 7.8, 'Advanced System Performance Analysis', 
                fontsize=16, ha='center', va='center', style='italic')
        
        # Report metadata
        summary = report_data["summary"]
        ax.text(5, 6.5, f'Test Duration: {summary["total_duration"]:.1f} seconds', 
                fontsize=12, ha='center', va='center')
        ax.text(5, 6.1, f'Total Tests: {summary["total_tests"]}', 
                fontsize=12, ha='center', va='center')
        ax.text(5, 5.7, f'Success Rate: {summary["success_rate"]:.1f}%', 
                fontsize=12, ha='center', va='center')
        ax.text(5, 5.3, f'Passed: {summary["passed_tests"]} | Failed: {summary["failed_tests"]}', 
                fontsize=12, ha='center', va='center')
        
        # System info
        if report_data.get("system_info"):
            system_info = report_data["system_info"]
            ax.text(5, 4.5, 'System Information', 
                    fontsize=16, fontweight='bold', ha='center', va='center')
            
            y_pos = 4.0
            if system_info.get("system"):
                ax.text(5, y_pos, f"Platform: {system_info['system'].get('platform', 'Unknown')}", 
                        fontsize=12, ha='center', va='center')
                y_pos -= 0.4
            
            if system_info.get("cpu"):
                ax.text(5, y_pos, f"CPU: {system_info['cpu'].get('count', 'Unknown')} cores", 
                        fontsize=12, ha='center', va='center')
                y_pos -= 0.4
            
            if system_info.get("memory"):
                ax.text(5, y_pos, f"Memory: {system_info['memory'].get('total_gb', 0):.1f} GB", 
                        fontsize=12, ha='center', va='center')
                y_pos -= 0.4
        
        # Generation timestamp
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        ax.text(5, 1.5, f'Generated: {timestamp}', 
                fontsize=10, ha='center', va='center', style='italic')
        
        pdf.savefig(fig, bbox_inches='tight')
        plt.close()
    
    def _create_summary_page(self, pdf, report_data: Dict[str, Any]):
        """Create PDF summary page"""
        fig, ax = plt.subplots(figsize=(8.5, 11))
        ax.set_xlim(0, 10)
        ax.set_ylim(0, 10)
        ax.axis('off')
        
        # Title
        ax.text(5, 9.5, 'Executive Summary', 
                fontsize=20, fontweight='bold', ha='center', va='center')
        
        # Summary metrics
        summary = report_data["summary"]
        y_pos = 8.5
        
        metrics = [
            ("Overall Performance", f"{summary['success_rate']:.1f}% success rate"),
            ("Test Coverage", f"{summary['total_tests']} tests executed"),
            ("System Stability", "High" if summary['success_rate'] > 90 else "Moderate" if summary['success_rate'] > 75 else "Low"),
            ("Duration", f"{summary['total_duration']:.1f} seconds"),
            ("Pass Rate", f"{summary['passed_tests']}/{summary['total_tests']}")
        ]
        
        for metric_name, metric_value in metrics:
            ax.text(1, y_pos, f"{metric_name}:", fontsize=12, fontweight='bold', ha='left', va='center')
            ax.text(6, y_pos, metric_value, fontsize=12, ha='left', va='center')
            y_pos -= 0.6
        
        # Test categories breakdown
        ax.text(5, 5.8, 'Test Categories', fontsize=16, fontweight='bold', ha='center', va='center')
        
        categories = {}
        for result in report_data["test_results"]:
            cat = result["category"]
            if cat not in categories:
                categories[cat] = {"total": 0, "passed": 0}
            categories[cat]["total"] += 1
            if result["status"] == "PASS":
                categories[cat]["passed"] += 1
        
        y_pos = 5.2
        for cat, data in categories.items():
            success_rate = (data["passed"] / data["total"] * 100) if data["total"] > 0 else 0
            ax.text(1, y_pos, f"{cat}:", fontsize=12, ha='left', va='center')
            ax.text(6, y_pos, f"{data['passed']}/{data['total']} ({success_rate:.1f}%)", 
                   fontsize=12, ha='left', va='center')
            y_pos -= 0.4
        
        # Recommendations
        if report_data.get("recommendations"):
            ax.text(5, 3.2, 'Key Recommendations', fontsize=16, fontweight='bold', ha='center', va='center')
            
            y_pos = 2.8
            for i, rec in enumerate(report_data["recommendations"][:5], 1):  # Top 5 recommendations
                ax.text(0.5, y_pos, f"{i}.", fontsize=10, ha='right', va='top')
                ax.text(0.7, y_pos, rec, fontsize=10, ha='left', va='top', wrap=True)
                y_pos -= 0.5
        
        pdf.savefig(fig, bbox_inches='tight')
        plt.close()
    
    def _create_test_results_page(self, pdf, report_data: Dict[str, Any]):
        """Create detailed test results page"""
        fig, ax = plt.subplots(figsize=(8.5, 11))
        ax.set_xlim(0, 10)
        ax.set_ylim(0, 10)
        ax.axis('off')
        
        # Title
        ax.text(5, 9.5, 'Detailed Test Results', 
                fontsize=20, fontweight='bold', ha='center', va='center')
        
        # Table headers
        y_pos = 8.8
        headers = ['Test Name', 'Status', 'Duration', 'Memory', 'CPU']
        x_positions = [0.5, 3.5, 4.8, 6.2, 7.5]
        
        for header, x_pos in zip(headers, x_positions):
            ax.text(x_pos, y_pos, header, fontsize=12, fontweight='bold', ha='left', va='center')
        
        # Draw header line
        ax.plot([0.5, 9], [8.6, 8.6], 'k-', linewidth=1)
        
        # Test results
        y_pos = 8.2
        for i, result in enumerate(report_data["test_results"][:15]):  # Limit to 15 tests per page
            # Truncate long test names
            test_name = result["test_name"][:25] + "..." if len(result["test_name"]) > 25 else result["test_name"]
            
            # Color code by status
            color = {'PASS': 'green', 'FAIL': 'red', 'PARTIAL': 'orange', 
                    'ERROR': 'purple', 'SKIPPED': 'gray'}.get(result['status'], 'black')
            
            ax.text(x_positions[0], y_pos, test_name, fontsize=8, ha='left', va='center')
            ax.text(x_positions[1], y_pos, result['status'], fontsize=8, ha='left', va='center', color=color, fontweight='bold')
            ax.text(x_positions[2], y_pos, f"{result['duration']:.2f}s", fontsize=8, ha='left', va='center')
            
            memory_usage = result.get("memory_usage", {}).get("avg_percent", 0)
            ax.text(x_positions[3], y_pos, f"{memory_usage:.1f}%", fontsize=8, ha='left', va='center')
            
            cpu_usage = result.get("cpu_usage", {}).get("avg_percent", 0)
            ax.text(x_positions[4], y_pos, f"{cpu_usage:.1f}%", fontsize=8, ha='left', va='center')
            
            y_pos -= 0.3
        
        if len(report_data["test_results"]) > 15:
            ax.text(5, y_pos-0.5, f"... and {len(report_data['test_results']) - 15} more tests", 
                   fontsize=10, ha='center', va='center', style='italic')
        
        pdf.savefig(fig, bbox_inches='tight')
        plt.close()
    
    def _create_charts_page(self, pdf, report_data: Dict[str, Any]):
        """Create charts page for PDF"""
        # Create a comprehensive chart page
        fig, axes = plt.subplots(2, 2, figsize=(8.5, 11))
        fig.suptitle('Performance Charts', fontsize=16, fontweight='bold')
        
        # Test status distribution
        ax1 = axes[0, 0]
        status_counts = {}
        for result in report_data["test_results"]:
            status = result["status"]
            status_counts[status] = status_counts.get(status, 0) + 1
        
        if status_counts:
            colors = ['#2ecc71', '#e74c3c', '#f39c12', '#9b59b6', '#95a5a6']
            ax1.pie(status_counts.values(), labels=status_counts.keys(), 
                   colors=[colors[i % len(colors)] for i in range(len(status_counts))],
                   autopct='%1.1f%%', startangle=90)
        ax1.set_title('Test Status Distribution')
        
        # Category performance
        ax2 = axes[0, 1]
        categories = {}
        for result in report_data["test_results"]:
            cat = result["category"]
            if cat not in categories:
                categories[cat] = {"total": 0, "passed": 0}
            categories[cat]["total"] += 1
            if result["status"] == "PASS":
                categories[cat]["passed"] += 1
        
        if categories:
            cat_names = list(categories.keys())
            success_rates = [categories[cat]["passed"]/categories[cat]["total"]*100 for cat in cat_names]
            bars = ax2.bar(cat_names, success_rates, color='skyblue', alpha=0.8)
            ax2.set_title('Success Rate by Category')
            ax2.set_ylabel('Success Rate (%)')
            ax2.tick_params(axis='x', rotation=45)
            
            # Add value labels
            for bar, rate in zip(bars, success_rates):
                height = bar.get_height()
                ax2.text(bar.get_x() + bar.get_width()/2., height,
                        f'{rate:.1f}%', ha='center', va='bottom', fontsize=8)
        
        # Memory usage timeline
        ax3 = axes[1, 0]
        memory_data = []
        for result in report_data["test_results"]:
            if result.get("memory_usage", {}).get("avg_percent"):
                memory_data.append(result["memory_usage"]["avg_percent"])
        
        if memory_data:
            ax3.plot(range(len(memory_data)), memory_data, 'b-', marker='o', markersize=4)
            ax3.set_title('Memory Usage Timeline')
            ax3.set_ylabel('Memory Usage (%)')
            ax3.set_xlabel('Test Index')
            ax3.grid(True, alpha=0.3)
        
        # Duration distribution
        ax4 = axes[1, 1]
        durations = [r["duration"] for r in report_data["test_results"]]
        if durations:
            ax4.hist(durations, bins=10, alpha=0.7, color='lightgreen', edgecolor='black')
            ax4.set_title('Test Duration Distribution')
            ax4.set_xlabel('Duration (seconds)')
            ax4.set_ylabel('Frequency')
            ax4.grid(True, alpha=0.3)
        
        plt.tight_layout()
        pdf.savefig(fig, bbox_inches='tight')
        plt.close()
    
    def _create_recommendations_page(self, pdf, report_data: Dict[str, Any]):
        """Create recommendations page for PDF"""
        fig, ax = plt.subplots(figsize=(8.5, 11))
        ax.set_xlim(0, 10)
        ax.set_ylim(0, 10)
        ax.axis('off')
        
        # Title
        ax.text(5, 9.5, 'Recommendations & Action Items', 
                fontsize=20, fontweight='bold', ha='center', va='center')
        
        if report_data.get("recommendations"):
            y_pos = 8.5
            
            for i, recommendation in enumerate(report_data["recommendations"], 1):
                # Recommendation box
                rect = patches.Rectangle((0.5, y_pos-0.3), 9, 0.6, 
                                       linewidth=1, edgecolor='lightblue', 
                                       facecolor='lightblue', alpha=0.3)
                ax.add_patch(rect)
                
                ax.text(5, y_pos, f"{i}. {recommendation}", 
                       fontsize=12, ha='center', va='center', wrap=True)
                
                y_pos -= 0.8
        else:
            ax.text(5, 5, 'No specific recommendations available', 
                   fontsize=14, ha='center', va='center', style='italic')
        
        # Additional insights
        ax.text(5, 2.5, 'Additional Insights', 
                fontsize=16, fontweight='bold', ha='center', va='center')
        
        insights = [
            "Monitor resource usage during peak loads",
            "Implement automated recovery mechanisms",
            "Consider scaling resources based on demand patterns",
            "Regular stress testing for continuous validation"
        ]
        
        y_pos = 2.0
        for insight in insights:
            ax.text(1, y_pos, f"• {insight}", fontsize=10, ha='left', va='center')
            y_pos -= 0.3
        
        pdf.savefig(fig, bbox_inches='tight')
        plt.close()