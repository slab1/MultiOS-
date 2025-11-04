#!/usr/bin/env python3
"""
Alert Manager - Manages performance alerts and notifications
Handles alert creation, acknowledgment, and notification delivery
"""

import sqlite3
import smtplib
import json
import threading
import time
from datetime import datetime, timedelta
from email.mime.text import MimeText
from email.mime.multipart import MimeMultipart
from typing import Dict, List, Any, Optional, Callable
import logging
import os

class AlertManager:
    def __init__(self, db_path: str = "data/monitor.db"):
        self.db_path = db_path
        self.alert_handlers = []
        self.notification_config = {
            'email': {
                'enabled': False,
                'smtp_server': '',
                'smtp_port': 587,
                'username': '',
                'password': '',
                'to_addresses': []
            },
            'webhook': {
                'enabled': False,
                'url': '',
                'headers': {}
            },
            'log': {
                'enabled': True,
                'log_file': 'logs/alerts.log'
            }
        }
        
        # Alert aggregation to prevent spam
        self.alert_aggregator = {}
        self.aggregation_window = 300  # 5 minutes
        
        # Active alerts tracking
        self.active_alerts = {}
        
        # Setup logging
        self.setup_logging()
        
        # Initialize database
        self.init_database()
    
    def setup_logging(self):
        """Setup logging configuration for alerts"""
        os.makedirs('logs', exist_ok=True)
        
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
            handlers=[
                logging.FileHandler(self.notification_config['log']['log_file']),
                logging.StreamHandler()
            ]
        )
        self.logger = logging.getLogger('AlertManager')
    
    def init_database(self):
        """Initialize alerts database table"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS alerts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME,
                alert_type TEXT,
                severity TEXT,
                message TEXT,
                metric_value REAL,
                threshold REAL,
                acknowledged BOOLEAN DEFAULT FALSE,
                acknowledged_by TEXT,
                acknowledged_at DATETIME,
                resolved BOOLEAN DEFAULT FALSE,
                resolved_at DATETIME,
                tags TEXT,
                data TEXT
            )
        ''')
        
        conn.commit()
        conn.close()
        
        self.logger.info("Alert database initialized")
    
    def configure_email_notifications(self, smtp_server: str, smtp_port: int, 
                                    username: str, password: str, to_addresses: List[str]):
        """Configure email notifications"""
        self.notification_config['email'].update({
            'enabled': True,
            'smtp_server': smtp_server,
            'smtp_port': smtp_port,
            'username': username,
            'password': password,
            'to_addresses': to_addresses
        })
        
        self.logger.info("Email notifications configured")
    
    def configure_webhook_notifications(self, webhook_url: str, headers: Dict[str, str] = None):
        """Configure webhook notifications"""
        self.notification_config['webhook'].update({
            'enabled': True,
            'url': webhook_url,
            'headers': headers or {}
        })
        
        self.logger.info("Webhook notifications configured")
    
    def add_alert_handler(self, handler: Callable[[Dict[str, Any]], None]):
        """Add custom alert handler"""
        self.alert_handlers.append(handler)
        self.logger.info("Custom alert handler added")
    
    def create_alert(self, alert_type: str, severity: str, message: str, 
                    metric_value: float, threshold: float, 
                    tags: List[str] = None, data: Dict[str, Any] = None) -> int:
        """Create a new alert"""
        try:
            # Check for aggregation
            alert_key = f"{alert_type}_{severity}"
            current_time = time.time()
            
            # If we have a recent similar alert, aggregate it
            if alert_key in self.alert_aggregator:
                last_alert_time = self.alert_aggregator[alert_key]
                if current_time - last_alert_time < self.aggregation_window:
                    self.logger.info(f"Alert aggregated: {alert_type} ({severity})")
                    return self._update_last_alert(alert_key, metric_value)
            
            # Store in database
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            cursor.execute('''
                INSERT INTO alerts (
                    timestamp, alert_type, severity, message, metric_value, 
                    threshold, tags, data
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ''', (
                datetime.now().isoformat(),
                alert_type,
                severity,
                message,
                metric_value,
                threshold,
                json.dumps(tags) if tags else None,
                json.dumps(data) if data else None
            ))
            
            alert_id = cursor.lastrowid
            conn.commit()
            conn.close()
            
            # Update aggregator
            self.alert_aggregator[alert_key] = current_time
            
            # Create alert object
            alert = {
                'id': alert_id,
                'timestamp': datetime.now().isoformat(),
                'alert_type': alert_type,
                'severity': severity,
                'message': message,
                'metric_value': metric_value,
                'threshold': threshold,
                'tags': tags or [],
                'data': data or {}
            }
            
            # Process alert
            self._process_alert(alert)
            
            self.logger.info(f"Alert created: {alert_type} ({severity}) - {message}")
            
            return alert_id
            
        except Exception as e:
            self.logger.error(f"Error creating alert: {e}")
            return -1
    
    def _update_last_alert(self, alert_key: str, metric_value: float):
        """Update the last occurrence of an aggregated alert"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            # Get the most recent alert of this type
            alert_type, severity = alert_key.split('_', 1)
            cursor.execute('''
                UPDATE alerts 
                SET timestamp = ?
                WHERE alert_type = ? AND severity = ? 
                AND id = (SELECT MAX(id) FROM alerts WHERE alert_type = ? AND severity = ?)
            ''', (datetime.now().isoformat(), alert_type, severity, alert_type, severity))
            
            conn.commit()
            conn.close()
            
            return 1
            
        except Exception as e:
            self.logger.error(f"Error updating aggregated alert: {e}")
            return -1
    
    def _process_alert(self, alert: Dict[str, Any]):
        """Process alert through various notification channels"""
        try:
            # Send notifications
            self._send_email_notification(alert)
            self._send_webhook_notification(alert)
            self._write_to_log(alert)
            
            # Call custom handlers
            for handler in self.alert_handlers:
                try:
                    handler(alert)
                except Exception as e:
                    self.logger.error(f"Error in custom alert handler: {e}")
            
            # Track active alerts
            self.active_alerts[alert['id']] = alert
            
        except Exception as e:
            self.logger.error(f"Error processing alert: {e}")
    
    def _send_email_notification(self, alert: Dict[str, Any]):
        """Send email notification"""
        email_config = self.notification_config['email']
        
        if not email_config['enabled'] or not email_config['to_addresses']:
            return
        
        try:
            msg = MimeMultipart()
            msg['From'] = email_config['username']
            msg['To'] = ', '.join(email_config['to_addresses'])
            msg['Subject'] = f"[{alert['severity'].upper()}] {alert['alert_type']} Alert"
            
            body = f"""
Performance Alert

Alert Type: {alert['alert_type']}
Severity: {alert['severity'].upper()}
Message: {alert['message']}
Metric Value: {alert['metric_value']}
Threshold: {alert['threshold']}
Timestamp: {alert['timestamp']}

Generated by Performance Monitoring Dashboard
            """
            
            msg.attach(MimeText(body, 'plain'))
            
            server = smtplib.SMTP(email_config['smtp_server'], email_config['smtp_port'])
            server.starttls()
            server.login(email_config['username'], email_config['password'])
            server.send_message(msg)
            server.quit()
            
            self.logger.info(f"Email notification sent for alert {alert['id']}")
            
        except Exception as e:
            self.logger.error(f"Error sending email notification: {e}")
    
    def _send_webhook_notification(self, alert: Dict[str, Any]):
        """Send webhook notification"""
        webhook_config = self.notification_config['webhook']
        
        if not webhook_config['enabled'] or not webhook_config['url']:
            return
        
        try:
            import requests
            
            headers = {'Content-Type': 'application/json'}
            headers.update(webhook_config['headers'])
            
            response = requests.post(
                webhook_config['url'],
                json=alert,
                headers=headers,
                timeout=10
            )
            
            if response.status_code == 200:
                self.logger.info(f"Webhook notification sent for alert {alert['id']}")
            else:
                self.logger.warning(f"Webhook notification failed: {response.status_code}")
            
        except Exception as e:
            self.logger.error(f"Error sending webhook notification: {e}")
    
    def _write_to_log(self, alert: Dict[str, Any]):
        """Write alert to log file"""
        if not self.notification_config['log']['enabled']:
            return
        
        log_message = f"ALERT [{alert['severity'].upper()}] {alert['alert_type']}: {alert['message']}"
        
        if alert['severity'] == 'critical':
            self.logger.critical(log_message)
        elif alert['severity'] == 'warning':
            self.logger.warning(log_message)
        else:
            self.logger.info(log_message)
    
    def acknowledge_alert(self, alert_id: int, acknowledged_by: str = None) -> bool:
        """Acknowledge an alert"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            cursor.execute('''
                UPDATE alerts 
                SET acknowledged = TRUE, acknowledged_by = ?, acknowledged_at = ?
                WHERE id = ?
            ''', (acknowledged_by or 'system', datetime.now().isoformat(), alert_id))
            
            if cursor.rowcount > 0:
                conn.commit()
                conn.close()
                
                # Remove from active alerts
                if alert_id in self.active_alerts:
                    del self.active_alerts[alert_id]
                
                self.logger.info(f"Alert {alert_id} acknowledged by {acknowledged_by or 'system'}")
                return True
            else:
                conn.close()
                return False
            
        except Exception as e:
            self.logger.error(f"Error acknowledging alert: {e}")
            return False
    
    def resolve_alert(self, alert_id: int) -> bool:
        """Mark an alert as resolved"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            cursor.execute('''
                UPDATE alerts 
                SET resolved = TRUE, resolved_at = ?
                WHERE id = ?
            ''', (datetime.now().isoformat(), alert_id))
            
            if cursor.rowcount > 0:
                conn.commit()
                conn.close()
                
                # Remove from active alerts
                if alert_id in self.active_alerts:
                    del self.active_alerts[alert_id]
                
                self.logger.info(f"Alert {alert_id} resolved")
                return True
            else:
                conn.close()
                return False
            
        except Exception as e:
            self.logger.error(f"Error resolving alert: {e}")
            return False
    
    def get_active_alerts(self, severity: str = None) -> List[Dict[str, Any]]:
        """Get all active (unacknowledged) alerts"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            if severity:
                cursor.execute('''
                    SELECT * FROM alerts 
                    WHERE acknowledged = FALSE AND resolved = FALSE AND severity = ?
                    ORDER BY timestamp DESC
                ''', (severity,))
            else:
                cursor.execute('''
                    SELECT * FROM alerts 
                    WHERE acknowledged = FALSE AND resolved = FALSE
                    ORDER BY timestamp DESC
                ''')
            
            columns = [description[0] for description in cursor.description]
            alerts = [dict(zip(columns, row)) for row in cursor.fetchall()]
            
            conn.close()
            
            # Parse JSON fields
            for alert in alerts:
                if alert['tags']:
                    alert['tags'] = json.loads(alert['tags'])
                if alert['data']:
                    alert['data'] = json.loads(alert['data'])
            
            return alerts
            
        except Exception as e:
            self.logger.error(f"Error getting active alerts: {e}")
            return []
    
    def get_alerts_history(self, hours: int = 24, severity: str = None, 
                          alert_type: str = None) -> List[Dict[str, Any]]:
        """Get historical alerts"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            start_time = datetime.now() - timedelta(hours=hours)
            
            query = '''
                SELECT * FROM alerts 
                WHERE timestamp > ?
            '''
            params = [start_time.isoformat()]
            
            if severity:
                query += ' AND severity = ?'
                params.append(severity)
            
            if alert_type:
                query += ' AND alert_type = ?'
                params.append(alert_type)
            
            query += ' ORDER BY timestamp DESC'
            
            cursor.execute(query, params)
            
            columns = [description[0] for description in cursor.description]
            alerts = [dict(zip(columns, row)) for row in cursor.fetchall()]
            
            conn.close()
            
            # Parse JSON fields
            for alert in alerts:
                if alert['tags']:
                    alert['tags'] = json.loads(alert['tags'])
                if alert['data']:
                    alert['data'] = json.loads(alert['data'])
            
            return alerts
            
        except Exception as e:
            self.logger.error(f"Error getting alerts history: {e}")
            return []
    
    def get_alert_statistics(self, hours: int = 24) -> Dict[str, Any]:
        """Get alert statistics"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            start_time = datetime.now() - timedelta(hours=hours)
            
            # Total alerts
            cursor.execute('''
                SELECT COUNT(*) FROM alerts WHERE timestamp > ?
            ''', (start_time.isoformat(),))
            total_alerts = cursor.fetchone()[0]
            
            # By severity
            cursor.execute('''
                SELECT severity, COUNT(*) FROM alerts 
                WHERE timestamp > ? 
                GROUP BY severity
            ''', (start_time.isoformat(),))
            severity_stats = dict(cursor.fetchall())
            
            # By type
            cursor.execute('''
                SELECT alert_type, COUNT(*) FROM alerts 
                WHERE timestamp > ? 
                GROUP BY alert_type
                ORDER BY COUNT(*) DESC
                LIMIT 10
            ''', (start_time.isoformat(),))
            type_stats = dict(cursor.fetchall())
            
            # Acknowledged vs unacknowledged
            cursor.execute('''
                SELECT 
                    COUNT(*) as total,
                    SUM(CASE WHEN acknowledged = TRUE THEN 1 ELSE 0 END) as acknowledged,
                    SUM(CASE WHEN acknowledged = FALSE THEN 1 ELSE 0 END) as unacknowledged
                FROM alerts 
                WHERE timestamp > ?
            ''', (start_time.isoformat(),))
            ack_stats = cursor.fetchone()
            
            conn.close()
            
            return {
                'total_alerts': total_alerts,
                'severity_distribution': severity_stats,
                'type_distribution': type_stats,
                'acknowledgment_stats': {
                    'acknowledged': ack_stats[1] if ack_stats else 0,
                    'unacknowledged': ack_stats[2] if ack_stats else 0
                },
                'period_hours': hours
            }
            
        except Exception as e:
            self.logger.error(f"Error getting alert statistics: {e}")
            return {}
    
    def create_threshold_alert(self, alert_type: str, current_value: float, 
                             threshold: float, severity: str = 'warning',
                             tags: List[str] = None) -> int:
        """Create threshold-based alert"""
        if severity not in ['info', 'warning', 'critical']:
            severity = 'warning'
        
        # Determine if threshold is breached
        breached = False
        if alert_type == 'cpu' and current_value > threshold:
            breached = True
        elif alert_type == 'memory' and current_value > threshold:
            breached = True
        elif alert_type == 'disk' and current_value > threshold:
            breached = True
        elif alert_type == 'network' and current_value > threshold:
            breached = True
        elif alert_type == 'load_average' and current_value > threshold:
            breached = True
        
        if not breached:
            return -1
        
        message = f"{alert_type} value {current_value:.2f} exceeded threshold {threshold:.2f}"
        
        return self.create_alert(
            alert_type=alert_type,
            severity=severity,
            message=message,
            metric_value=current_value,
            threshold=threshold,
            tags=tags or [alert_type]
        )
    
    def cleanup_old_alerts(self, days: int = 30) -> int:
        """Clean up old alerts from database"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            cutoff_time = datetime.now() - timedelta(days=days)
            
            cursor.execute('''
                DELETE FROM alerts 
                WHERE timestamp < ? AND (acknowledged = TRUE OR resolved = TRUE)
            ''', (cutoff_time.isoformat(),))
            
            deleted_count = cursor.rowcount
            conn.commit()
            conn.close()
            
            self.logger.info(f"Cleaned up {deleted_count} old alerts")
            
            return deleted_count
            
        except Exception as e:
            self.logger.error(f"Error cleaning up old alerts: {e}")
            return 0
    
    def test_notifications(self) -> bool:
        """Test all configured notification channels"""
        try:
            # Create test alert
            test_alert = {
                'id': 0,
                'timestamp': datetime.now().isoformat(),
                'alert_type': 'test',
                'severity': 'info',
                'message': 'Test notification from Performance Monitoring Dashboard',
                'metric_value': 0,
                'threshold': 0,
                'tags': ['test'],
                'data': {'test': True}
            }
            
            # Test email
            if self.notification_config['email']['enabled']:
                try:
                    self._send_email_notification(test_alert)
                    self.logger.info("Email notification test successful")
                except Exception as e:
                    self.logger.error(f"Email notification test failed: {e}")
            
            # Test webhook
            if self.notification_config['webhook']['enabled']:
                try:
                    self._send_webhook_notification(test_alert)
                    self.logger.info("Webhook notification test successful")
                except Exception as e:
                    self.logger.error(f"Webhook notification test failed: {e}")
            
            return True
            
        except Exception as e:
            self.logger.error(f"Error testing notifications: {e}")
            return False
    
    def export_alerts(self, output_file: str, hours: int = 24, format: str = 'json') -> bool:
        """Export alerts to file"""
        try:
            alerts = self.get_alerts_history(hours)
            
            if format.lower() == 'json':
                with open(output_file, 'w') as f:
                    json.dump({
                        'export_time': datetime.now().isoformat(),
                        'period_hours': hours,
                        'alerts': alerts
                    }, f, indent=2, default=str)
            
            elif format.lower() == 'csv':
                import csv
                with open(output_file, 'w', newline='') as f:
                    if alerts:
                        writer = csv.DictWriter(f, fieldnames=alerts[0].keys())
                        writer.writeheader()
                        for alert in alerts:
                            # Convert datetime objects to strings
                            row = {k: v.isoformat() if isinstance(v, datetime) else v 
                                  for k, v in alert.items()}
                            writer.writerow(row)
            
            else:
                self.logger.error(f"Unsupported export format: {format}")
                return False
            
            self.logger.info(f"Alerts exported to {output_file}")
            return True
            
        except Exception as e:
            self.logger.error(f"Error exporting alerts: {e}")
            return False
    
    def get_alert_trends(self, hours: int = 24) -> Dict[str, Any]:
        """Analyze alert trends over time"""
        try:
            alerts = self.get_alerts_history(hours)
            
            if not alerts:
                return {'trends': [], 'summary': {}}
            
            # Group alerts by hour
            hourly_counts = {}
            for alert in alerts:
                alert_time = datetime.fromisoformat(alert['timestamp'])
                hour_key = alert_time.strftime('%Y-%m-%d %H:00')
                
                if hour_key not in hourly_counts:
                    hourly_counts[hour_key] = {'total': 0, 'critical': 0, 'warning': 0, 'info': 0}
                
                hourly_counts[hour_key]['total'] += 1
                hourly_counts[hour_key][alert['severity']] += 1
            
            # Convert to sorted list
            trends = []
            for hour, counts in sorted(hourly_counts.items()):
                trends.append({
                    'hour': hour,
                    'total_alerts': counts['total'],
                    'critical_alerts': counts['critical'],
                    'warning_alerts': counts['warning'],
                    'info_alerts': counts['info']
                })
            
            # Calculate summary statistics
            total_alerts = len(alerts)
            avg_per_hour = total_alerts / (hours if hours > 0 else 1)
            
            severity_counts = {}
            for alert in alerts:
                severity = alert['severity']
                severity_counts[severity] = severity_counts.get(severity, 0) + 1
            
            return {
                'trends': trends,
                'summary': {
                    'total_alerts': total_alerts,
                    'average_per_hour': avg_per_hour,
                    'severity_distribution': severity_counts,
                    'peak_hour': max(trends, key=lambda x: x['total_alerts'])['hour'] if trends else None
                }
            }
            
        except Exception as e:
            self.logger.error(f"Error analyzing alert trends: {e}")
            return {'trends': [], 'summary': {}}

if __name__ == "__main__":
    # Example usage
    alert_manager = AlertManager()
    
    # Create test alerts
    alert_manager.create_threshold_alert('cpu', 85.5, 80.0, 'warning')
    alert_manager.create_threshold_alert('memory', 95.2, 90.0, 'critical')
    
    # Get active alerts
    active_alerts = alert_manager.get_active_alerts()
    print(f"Active alerts: {len(active_alerts)}")
    
    # Get statistics
    stats = alert_manager.get_alert_statistics()
    print(f"Alert statistics: {stats}")