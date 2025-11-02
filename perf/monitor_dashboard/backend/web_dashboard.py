#!/usr/bin/env python3
"""
Web Dashboard - Flask-based web interface for system monitoring
Provides REST API and WebSocket support for real-time updates
"""

from flask import Flask, render_template, jsonify, request, send_file
from flask_socketio import SocketIO, emit
from flask_cors import CORS
import threading
import time
import json
import os
import csv
from datetime import datetime, timedelta
import logging
from system_monitor import SystemMonitor
from report_generator import ReportGenerator
from alert_manager import AlertManager

app = Flask(__name__)
app.config['SECRET_KEY'] = 'monitor_dashboard_secret_key'
socketio = SocketIO(app, cors_allowed_origins="*")
CORS(app)

# Initialize components
monitor = SystemMonitor()
report_generator = ReportGenerator(monitor.db_path)
alert_manager = AlertManager(monitor.db_path)

# Global variables
current_metrics = {}
is_monitoring = False

# Setup logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# WebSocket event handlers
@socketio.on('connect')
def handle_connect():
    logger.info('Client connected to WebSocket')
    emit('connected', {'status': 'connected'})

@socketio.on('disconnect')
def handle_disconnect():
    logger.info('Client disconnected from WebSocket')

@socketio.on('request_metrics')
def handle_metrics_request():
    """Handle client request for current metrics"""
    try:
        metrics = monitor.get_current_metrics()
        emit('metrics_update', metrics)
    except Exception as e:
        logger.error(f"Error sending metrics: {e}")
        emit('error', {'message': str(e)})

# HTTP API routes

@app.route('/')
def dashboard():
    """Main dashboard page"""
    return render_template('dashboard.html')

@app.route('/api/health')
def health_check():
    """Health check endpoint"""
    return jsonify({
        'status': 'healthy',
        'timestamp': datetime.now().isoformat(),
        'monitoring': is_monitoring
    })

@app.route('/api/metrics/current')
def get_current_metrics():
    """Get current system metrics"""
    try:
        metrics = monitor.get_current_metrics()
        return jsonify({
            'success': True,
            'data': metrics,
            'timestamp': datetime.now().isoformat()
        })
    except Exception as e:
        logger.error(f"Error getting current metrics: {e}")
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500

@app.route('/api/metrics/history/<metric_type>')
def get_historical_metrics(metric_type):
    """Get historical metrics data"""
    try:
        hours = request.args.get('hours', 24, type=int)
        data = monitor.get_historical_data(metric_type, hours)
        return jsonify({
            'success': True,
            'data': data,
            'metric_type': metric_type,
            'period_hours': hours
        })
    except Exception as e:
        logger.error(f"Error getting historical metrics: {e}")
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500

@app.route('/api/processes')
def get_processes():
    """Get detailed process information"""
    try:
        sort_by = request.args.get('sort', 'cpu')  # cpu, memory, name
        limit = request.args.get('limit', 50, type=int)
        
        metrics = monitor.get_current_metrics()
        processes = metrics.get('processes', {})
        
        if sort_by == 'cpu':
            process_list = processes.get('top_cpu_processes', [])
        elif sort_by == 'memory':
            process_list = processes.get('top_memory_processes', [])
        else:
            process_list = processes.get('top_cpu_processes', [])
        
        # Limit results
        limited_processes = process_list[:limit]
        
        return jsonify({
            'success': True,
            'data': limited_processes,
            'total_processes': processes.get('total_processes', 0),
            'running_processes': processes.get('running_processes', 0),
            'sleeping_processes': processes.get('sleeping_processes', 0),
            'zombie_processes': processes.get('zombie_processes', 0)
        })
    except Exception as e:
        logger.error(f"Error getting processes: {e}")
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500

@app.route('/api/alerts')
def get_alerts():
    """Get alert information"""
    try:
        hours = request.args.get('hours', 24, type=int)
        limit = request.args.get('limit', 100, type=int)
        
        alerts = monitor.get_recent_alerts(hours)
        limited_alerts = alerts[:limit]
        
        return jsonify({
            'success': True,
            'data': limited_alerts,
            'total_alerts': len(alerts)
        })
    except Exception as e:
        logger.error(f"Error getting alerts: {e}")
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500

@app.route('/api/alerts/<int:alert_id>/acknowledge', methods=['POST'])
def acknowledge_alert(alert_id):
    """Acknowledge an alert"""
    try:
        conn = sqlite3.connect(monitor.db_path)
        cursor = conn.cursor()
        cursor.execute('UPDATE alerts SET acknowledged = TRUE WHERE id = ?', (alert_id,))
        conn.commit()
        conn.close()
        
        return jsonify({
            'success': True,
            'message': 'Alert acknowledged'
        })
    except Exception as e:
        logger.error(f"Error acknowledging alert: {e}")
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500

@app.route('/api/custom_metrics', methods=['GET', 'POST'])
def manage_custom_metrics():
    """Manage custom metrics"""
    try:
        if request.method == 'GET':
            # Return list of custom metrics
            return jsonify({
                'success': True,
                'data': list(monitor.custom_metrics.keys())
            })
        
        elif request.method == 'POST':
            # Add custom metric
            data = request.json
            name = data.get('name')
            metric_func = data.get('function')
            
            if not name or not metric_func:
                return jsonify({
                    'success': False,
                    'error': 'Name and function are required'
                }), 400
            
            # For security, we'll need to implement proper function evaluation
            # This is a simplified version
            monitor.add_custom_metric(name, eval(metric_func))
            
            return jsonify({
                'success': True,
                'message': f'Custom metric {name} added'
            })
    
    except Exception as e:
        logger.error(f"Error managing custom metrics: {e}")
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500

@app.route('/api/config', methods=['GET', 'POST'])
def manage_config():
    """Get or update configuration"""
    try:
        if request.method == 'GET':
            # Return current configuration
            config = {
                'thresholds': monitor.thresholds,
                'monitoring_interval': getattr(monitor, 'monitoring_interval', 5),
                'history_size': monitor.history_size
            }
            return jsonify({
                'success': True,
                'data': config
            })
        
        elif request.method == 'POST':
            # Update configuration
            data = request.json
            
            if 'thresholds' in data:
                monitor.thresholds.update(data['thresholds'])
            
            if 'monitoring_interval' in data:
                monitor.monitoring_interval = data['monitoring_interval']
            
            if 'history_size' in data:
                monitor.history_size = data['history_size']
            
            return jsonify({
                'success': True,
                'message': 'Configuration updated'
            })
    
    except Exception as e:
        logger.error(f"Error managing config: {e}")
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500

@app.route('/api/export/metrics')
def export_metrics():
    """Export metrics to JSON file"""
    try:
        hours = request.args.get('hours', 24, type=int)
        timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
        filename = f'metrics_export_{timestamp}.json'
        filepath = os.path.join('reports', filename)
        
        monitor.export_metrics(filepath, hours)
        
        return send_file(
            filepath,
            as_attachment=True,
            download_name=filename,
            mimetype='application/json'
        )
    except Exception as e:
        logger.error(f"Error exporting metrics: {e}")
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500

@app.route('/api/export/report')
def export_report():
    """Export comprehensive performance report"""
    try:
        hours = request.args.get('hours', 24, type=int)
        format_type = request.args.get('format', 'pdf')  # pdf, html, csv
        timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
        
        if format_type == 'pdf':
            filename = f'performance_report_{timestamp}.pdf'
            filepath = os.path.join('reports', filename)
            report_generator.generate_pdf_report(hours, filepath)
        elif format_type == 'html':
            filename = f'performance_report_{timestamp}.html'
            filepath = os.path.join('reports', filename)
            report_generator.generate_html_report(hours, filepath)
        elif format_type == 'csv':
            filename = f'performance_report_{timestamp}.csv'
            filepath = os.path.join('reports', filename)
            report_generator.generate_csv_report(hours, filepath)
        else:
            return jsonify({
                'success': False,
                'error': 'Invalid format. Use pdf, html, or csv'
            }), 400
        
        return send_file(
            filepath,
            as_attachment=True,
            download_name=filename
        )
    except Exception as e:
        logger.error(f"Error exporting report: {e}")
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500

@app.route('/api/system/info')
def get_system_info():
    """Get detailed system information"""
    try:
        import platform
        import socket
        
        info = {
            'hostname': socket.gethostname(),
            'platform': platform.platform(),
            'architecture': platform.architecture(),
            'processor': platform.processor(),
            'python_version': platform.python_version(),
            'boot_time': monitor.get_current_metrics().get('kernel', {}).get('boot_time'),
            'uptime_seconds': monitor.get_current_metrics().get('kernel', {}).get('uptime_seconds'),
            'uptime_formatted': monitor.get_current_metrics().get('kernel', {}).get('uptime_formatted')
        }
        
        return jsonify({
            'success': True,
            'data': info
        })
    except Exception as e:
        logger.error(f"Error getting system info: {e}")
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500

# Monitoring control endpoints

@app.route('/api/monitoring/start', methods=['POST'])
def start_monitoring():
    """Start monitoring"""
    global is_monitoring
    
    try:
        interval = request.json.get('interval', 5) if request.json else 5
        
        if not is_monitoring:
            monitor.start_monitoring(interval)
            is_monitoring = True
            
            # Start background thread for WebSocket updates
            threading.Thread(target=broadcast_metrics_loop, daemon=True).start()
        
        return jsonify({
            'success': True,
            'message': 'Monitoring started',
            'interval': interval
        })
    except Exception as e:
        logger.error(f"Error starting monitoring: {e}")
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500

@app.route('/api/monitoring/stop', methods=['POST'])
def stop_monitoring():
    """Stop monitoring"""
    global is_monitoring
    
    try:
        if is_monitoring:
            monitor.stop_monitoring()
            is_monitoring = False
        
        return jsonify({
            'success': True,
            'message': 'Monitoring stopped'
        })
    except Exception as e:
        logger.error(f"Error stopping monitoring: {e}")
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500

def broadcast_metrics_loop():
    """Background thread to broadcast metrics to WebSocket clients"""
    while is_monitoring:
        try:
            metrics = monitor.get_current_metrics()
            socketio.emit('metrics_update', metrics)
            time.sleep(1)  # Update every second
        except Exception as e:
            logger.error(f"Error broadcasting metrics: {e}")
            time.sleep(5)

# Error handlers

@app.errorhandler(404)
def not_found(error):
    return jsonify({
        'success': False,
        'error': 'Endpoint not found'
    }), 404

@app.errorhandler(500)
def internal_error(error):
    return jsonify({
        'success': False,
        'error': 'Internal server error'
    }), 500

if __name__ == '__main__':
    # Create necessary directories
    os.makedirs('templates', exist_ok=True)
    os.makedirs('static', exist_ok=True)
    os.makedirs('reports', exist_ok=True)
    os.makedirs('data', exist_ok=True)
    os.makedirs('logs', exist_ok=True)
    
    logger.info("Starting Performance Monitoring Dashboard...")
    logger.info("Dashboard available at http://localhost:5000")
    
    # Run the application
    socketio.run(app, debug=False, host='0.0.0.0', port=5000)