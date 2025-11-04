#!/usr/bin/env python3
"""
MultiOS Alerting System
Comprehensive alerting infrastructure with rules, escalation, and notifications
"""

import asyncio
import json
import logging
import time
import smtplib
import sqlite3
import threading
import yaml
from datetime import datetime, timedelta
from email.mime.text import MIMEText
from email.mime.multipart import MIMEMultipart
from typing import Dict, List, Any, Optional, Callable, Union
from dataclasses import dataclass, asdict
from pathlib import Path
from collections import defaultdict, deque
import requests
import hashlib

@dataclass
class AlertRule:
    """Alert rule configuration"""
    id: str
    name: str
    description: str
    enabled: bool
    severity: str  # critical, warning, info
    metric_name: str
    operator: str  # >, <, >=, <=, ==, !=
    threshold: float
    duration: int  # seconds to maintain condition
    aggregation: str  # avg, min, max, sum, count
    labels: Dict[str, str] = None
    annotations: Dict[str, str] = None
    group_key: str = None
    notification_channels: List[str] = None

@dataclass
class Alert:
    """Alert instance"""
    id: str
    rule_id: str
    rule_name: str
    timestamp: float
    severity: str
    status: str  # firing, pending, resolved
    labels: Dict[str, str]
    annotations: Dict[str, str]
    values: Dict[str, float]
    generator_url: str = None
    fingerprint: str = None

@dataclass
class Notification:
    """Notification configuration"""
    id: str
    name: str
    type: str  # email, slack, webhook, sms, pagerduty
    config: Dict[str, Any]
    enabled: bool
    rate_limiting: Dict[str, Any] = None

class AlertRuleEngine:
    """Engine for evaluating alert rules against metrics"""
    
    def __init__(self):
        self.rules = {}
        self.active_alerts = {}
        self.pending_alerts = {}
        self.rule_evaluators = {}
        self.evaluation_lock = threading.Lock()
        
    def add_rule(self, rule: AlertRule):
        """Add an alert rule"""
        self.rules[rule.id] = rule
        logging.info(f"Added alert rule: {rule.id} - {rule.name}")
    
    def remove_rule(self, rule_id: str):
        """Remove an alert rule"""
        if rule_id in self.rules:
            del self.rules[rule_id]
            logging.info(f"Removed alert rule: {rule_id}")
    
    def update_rule(self, rule: AlertRule):
        """Update an existing rule"""
        self.rules[rule.id] = rule
        logging.info(f"Updated alert rule: {rule.id} - {rule.name}")
    
    def evaluate_rules(self, metrics_data: Dict[str, Any], timestamp: float = None):
        """Evaluate all rules against current metrics"""
        if timestamp is None:
            timestamp = time.time()
        
        with self.evaluation_lock:
            for rule_id, rule in self.rules.items():
                if not rule.enabled:
                    continue
                
                try:
                    self._evaluate_rule(rule, metrics_data, timestamp)
                except Exception as e:
                    logging.error(f"Error evaluating rule {rule_id}: {e}")
    
    def _evaluate_rule(self, rule: AlertRule, metrics_data: Dict[str, Any], timestamp: float):
        """Evaluate a single rule"""
        # Extract metric value
        metric_value = self._extract_metric_value(rule.metric_name, metrics_data)
        
        if metric_value is None:
            return  # Metric not available
        
        # Apply aggregation if needed
        if rule.aggregation != 'value':
            metric_value = self._apply_aggregation(rule.aggregation, rule.metric_name, metrics_data)
        
        # Check if condition is met
        condition_met = self._check_condition(metric_value, rule.operator, rule.threshold)
        
        # Handle pending alerts
        if condition_met:
            if rule.id not in self.pending_alerts:
                self.pending_alerts[rule.id] = {
                    'first_seen': timestamp,
                    'value': metric_value
                }
            else:
                # Check duration
                time_in_state = timestamp - self.pending_alerts[rule.id]['first_seen']
                if time_in_state >= rule.duration:
                    self._trigger_alert(rule, metrics_data, timestamp, metric_value)
        else:
            # Clear pending alert
            if rule.id in self.pending_alerts:
                del self.pending_alerts[rule.id]
                
                # Resolve alert if it was active
                if rule.id in self.active_alerts:
                    self._resolve_alert(rule, timestamp)
    
    def _extract_metric_value(self, metric_name: str, metrics_data: Dict[str, Any]) -> Optional[float]:
        """Extract metric value from metrics data"""
        # Navigate through nested structure
        parts = metric_name.split('.')
        current = metrics_data
        
        for part in parts:
            if isinstance(current, dict):
                current = current.get(part)
            else:
                return None
        
        return float(current) if current is not None else None
    
    def _apply_aggregation(self, aggregation: str, metric_name: str, metrics_data: Dict[str, Any]) -> float:
        """Apply aggregation function to metric"""
        # This would need access to historical data
        # For now, return current value
        return self._extract_metric_value(metric_name, metrics_data) or 0
    
    def _check_condition(self, value: float, operator: str, threshold: float) -> bool:
        """Check if condition is met"""
        if operator == '>':
            return value > threshold
        elif operator == '>=':
            return value >= threshold
        elif operator == '<':
            return value < threshold
        elif operator == '<=':
            return value <= threshold
        elif operator == '==':
            return value == threshold
        elif operator == '!=':
            return value != threshold
        else:
            logging.warning(f"Unknown operator: {operator}")
            return False
    
    def _trigger_alert(self, rule: AlertRule, metrics_data: Dict[str, Any], timestamp: float, metric_value: float):
        """Trigger an alert"""
        # Check if alert already exists
        if rule.id in self.active_alerts:
            return  # Alert already firing
        
        # Generate alert ID
        alert_id = f"{rule.id}-{int(timestamp)}"
        
        # Create alert
        alert = Alert(
            id=alert_id,
            rule_id=rule.id,
            rule_name=rule.name,
            timestamp=timestamp,
            severity=rule.severity,
            status='firing',
            labels=rule.labels or {},
            annotations=rule.annotations or {},
            values={'value': metric_value}
        )
        
        # Generate fingerprint for deduplication
        alert.fingerprint = self._generate_fingerprint(alert)
        
        # Store alert
        self.active_alerts[rule.id] = alert
        
        logging.info(f"Alert triggered: {alert.rule_name} - {alert.severity} - {metric_value}")
        
        # Remove from pending
        if rule.id in self.pending_alerts:
            del self.pending_alerts[rule.id]
        
        return alert
    
    def _resolve_alert(self, rule: AlertRule, timestamp: float):
        """Resolve an alert"""
        if rule.id in self.active_alerts:
            alert = self.active_alerts[rule.id]
            alert.status = 'resolved'
            alert.annotations['resolved_at'] = str(timestamp)
            
            logging.info(f"Alert resolved: {alert.rule_name}")
            
            # Keep resolved alerts for a short time for historical tracking
            threading.Timer(300, lambda: self._cleanup_resolved_alert(rule.id)).start()
    
    def _cleanup_resolved_alert(self, rule_id: str):
        """Clean up resolved alert"""
        if rule_id in self.active_alerts:
            del self.active_alerts[rule_id]
    
    def _generate_fingerprint(self, alert: Alert) -> str:
        """Generate alert fingerprint for deduplication"""
        fingerprint_data = {
            'rule_id': alert.rule_id,
            'labels': alert.labels,
            'severity': alert.severity
        }
        
        fingerprint_string = json.dumps(fingerprint_data, sort_keys=True)
        return hashlib.md5(fingerprint_string.encode()).hexdigest()
    
    def get_active_alerts(self) -> List[Alert]:
        """Get all active (firing) alerts"""
        return [alert for alert in self.active_alerts.values() if alert.status == 'firing']
    
    def get_pending_alerts(self) -> List[Dict[str, Any]]:
        """Get all pending alerts"""
        return list(self.pending_alerts.values())
    
    def get_all_alerts(self) -> List[Alert]:
        """Get all alerts (active and recently resolved)"""
        return list(self.active_alerts.values())

class NotificationManager:
    """Manages notifications across different channels"""
    
    def __init__(self):
        self.notifications = {}
        self.notification_history = deque(maxlen=10000)
        self.rate_limiter = {}
        self.escalation_rules = {}
        
    def add_notification(self, notification: Notification):
        """Add a notification channel"""
        self.notifications[notification.id] = notification
        logging.info(f"Added notification channel: {notification.id} - {notification.type}")
    
    def remove_notification(self, notification_id: str):
        """Remove a notification channel"""
        if notification_id in self.notifications:
            del self.notifications[notification_id]
            logging.info(f"Removed notification channel: {notification_id}")
    
    def send_alert_notification(self, alert: Alert, channels: List[str] = None):
        """Send alert notification"""
        if channels is None:
            # Use channels specified in rule
            rule = self.get_rule_for_alert(alert)
            if rule and rule.notification_channels:
                channels = rule.notification_channels
            else:
                channels = list(self.notifications.keys())
        
        for channel_id in channels:
            if channel_id in self.notifications:
                notification = self.notifications[channel_id]
                
                if self._check_rate_limiting(notification, alert):
                    try:
                        self._send_notification(notification, alert)
                        self._record_notification(notification.id, alert, 'sent')
                    except Exception as e:
                        logging.error(f"Error sending notification via {channel_id}: {e}")
                        self._record_notification(notification.id, alert, 'failed')
                else:
                    self._record_notification(notification.id, alert, 'rate_limited')
    
    def _check_rate_limiting(self, notification: Notification, alert: Alert) -> bool:
        """Check if notification should be sent based on rate limiting"""
        if not notification.rate_limiting:
            return True
        
        rate_limit = notification.rate_limiting.get('max_per_hour', 100)
        time_window = notification.rate_limiting.get('time_window', 3600)
        
        key = f"{notification.id}_{alert.severity}"
        current_time = time.time()
        
        if key not in self.rate_limiter:
            self.rate_limiter[key] = deque()
        
        # Remove old entries
        while (self.rate_limiter[key] and 
               current_time - self.rate_limiter[key][0] > time_window):
            self.rate_limiter[key].popleft()
        
        # Check if we're under the limit
        return len(self.rate_limiter[key]) < rate_limit
    
    def _send_notification(self, notification: Notification, alert: Alert):
        """Send notification via specific channel"""
        if notification.type == 'email':
            self._send_email(notification, alert)
        elif notification.type == 'slack':
            self._send_slack(notification, alert)
        elif notification.type == 'webhook':
            self._send_webhook(notification, alert)
        elif notification.type == 'sms':
            self._send_sms(notification, alert)
        elif notification.type == 'pagerduty':
            self._send_pagerduty(notification, alert)
        else:
            logging.warning(f"Unknown notification type: {notification.type}")
    
    def _send_email(self, notification: Notification, alert: Alert):
        """Send email notification"""
        config = notification.config
        
        msg = MIMEMultipart()
        msg['From'] = config['from_email']
        msg['To'] = ', '.join(config['to_emails'])
        msg['Subject'] = f"[{alert.severity.upper()}] {alert.rule_name}"
        
        body = self._format_email_body(alert)
        msg.attach(MIMEText(body, 'html'))
        
        # Send email
        server = smtplib.SMTP(config['smtp_server'], config['smtp_port'])
        if config.get('use_tls', True):
            server.starttls()
        if config.get('username') and config.get('password'):
            server.login(config['username'], config['password'])
        
        server.send_message(msg)
        server.quit()
    
    def _send_slack(self, notification: Notification, alert: Alert):
        """Send Slack notification"""
        config = notification.config
        
        payload = {
            'text': f"[{alert.severity.upper()}] {alert.rule_name}",
            'attachments': [
                {
                    'color': self._get_slack_color(alert.severity),
                    'fields': [
                        {'title': 'Rule', 'value': alert.rule_name, 'short': True},
                        {'title': 'Severity', 'value': alert.severity.upper(), 'short': True},
                        {'title': 'Status', 'value': alert.status, 'short': True},
                        {'title': 'Time', 'value': datetime.fromtimestamp(alert.timestamp).isoformat(), 'short': False}
                    ]
                }
            ]
        }
        
        # Add alert details
        if alert.annotations:
            details_text = '\n'.join([f"{k}: {v}" for k, v in alert.annotations.items()])
            payload['attachments'][0]['fields'].append({
                'title': 'Details',
                'value': details_text,
                'short': False
            })
        
        requests.post(config['webhook_url'], json=payload)
    
    def _send_webhook(self, notification: Notification, alert: Alert):
        """Send webhook notification"""
        config = notification.config
        
        payload = {
            'alert': asdict(alert),
            'timestamp': time.time()
        }
        
        headers = config.get('headers', {})
        headers['Content-Type'] = 'application/json'
        
        response = requests.post(
            config['url'],
            json=payload,
            headers=headers,
            timeout=config.get('timeout', 10)
        )
        
        response.raise_for_status()
    
    def _send_sms(self, notification: Notification, alert: Alert):
        """Send SMS notification"""
        config = notification.config
        
        # This would integrate with an SMS service like Twilio
        # For now, just log the SMS
        message = f"[{alert.severity.upper()}] {alert.rule_name}"
        
        logging.info(f"SMS notification: {message}")
    
    def _send_pagerduty(self, notification: Notification, alert: Alert):
        """Send PagerDuty notification"""
        config = notification.config
        
        payload = {
            'routing_key': config['routing_key'],
            'event_action': 'trigger' if alert.status == 'firing' else 'resolve',
            'dedup_key': alert.fingerprint,
            'payload': {
                'summary': f"{alert.rule_name}: {alert.annotations.get('description', '')}",
                'severity': alert.severity,
                'source': alert.labels.get('source', 'monitoring-system'),
                'component': alert.labels.get('component'),
                'group': alert.labels.get('group'),
                'class': alert.labels.get('class'),
                'custom_details': alert.annotations
            }
        }
        
        response = requests.post(
            'https://events.pagerduty.com/v2/enqueue',
            json=payload,
            headers={'Content-Type': 'application/json'}
        )
        
        response.raise_for_status()
    
    def _format_email_body(self, alert: Alert) -> str:
        """Format email body for alert"""
        html = f"""
        <html>
        <head>
            <style>
                body {{ font-family: Arial, sans-serif; }}
                .header {{ background-color: #f0f0f0; padding: 20px; }}
                .content {{ padding: 20px; }}
                .severity-critical {{ color: #d32f2f; }}
                .severity-warning {{ color: #f57c00; }}
                .severity-info {{ color: #1976d2; }}
                table {{ border-collapse: collapse; width: 100%; }}
                th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
                th {{ background-color: #f2f2f2; }}
            </style>
        </head>
        <body>
            <div class="header">
                <h2 class="severity-{alert.severity}">[{alert.severity.upper()}] {alert.rule_name}</h2>
            </div>
            <div class="content">
                <table>
                    <tr><th>Field</th><th>Value</th></tr>
                    <tr><td>Rule ID</td><td>{alert.rule_id}</td></tr>
                    <tr><td>Alert ID</td><td>{alert.id}</td></tr>
                    <tr><td>Status</td><td>{alert.status}</td></tr>
                    <tr><td>Timestamp</td><td>{datetime.fromtimestamp(alert.timestamp).isoformat()}</td></tr>
                </table>
        """
        
        if alert.labels:
            html += "<h3>Labels</h3><table>"
            for key, value in alert.labels.items():
                html += f"<tr><td>{key}</td><td>{value}</td></tr>"
            html += "</table>"
        
        if alert.annotations:
            html += "<h3>Annotations</h3><table>"
            for key, value in alert.annotations.items():
                html += f"<tr><td>{key}</td><td>{value}</td></tr>"
            html += "</table>"
        
        html += "</div></body></html>"
        return html
    
    def _get_slack_color(self, severity: str) -> str:
        """Get Slack color for severity"""
        colors = {
            'critical': 'danger',
            'warning': 'warning',
            'info': 'good'
        }
        return colors.get(severity, '#36a64f')
    
    def _record_notification(self, notification_id: str, alert: Alert, status: str):
        """Record notification attempt"""
        self.notification_history.append({
            'timestamp': time.time(),
            'notification_id': notification_id,
            'alert_id': alert.id,
            'status': status
        })
    
    def get_rule_for_alert(self, alert: Alert) -> Optional[AlertRule]:
        """Get rule associated with alert"""
        # This would need to be passed from the alert engine
        return None

class EscalationManager:
    """Manages alert escalation based on time and severity"""
    
    def __init__(self, notification_manager: NotificationManager):
        self.notification_manager = notification_manager
        self.escalation_rules = {}
        self.escalation_state = {}
        
    def add_escalation_rule(self, rule_id: str, rule: Dict[str, Any]):
        """Add escalation rule"""
        self.escalation_rules[rule_id] = rule
        logging.info(f"Added escalation rule: {rule_id}")
    
    def check_escalation(self, alert: Alert, current_time: float):
        """Check if alert should be escalated"""
        if alert.status != 'firing':
            return
        
        alert_key = f"{alert.rule_id}_{alert.id}"
        
        # Initialize escalation state for this alert
        if alert_key not in self.escalation_state:
            self.escalation_state[alert_key] = {
                'escalation_level': 0,
                'last_escalation': alert.timestamp
            }
        
        state = self.escalation_state[alert_key]
        
        # Check each escalation rule
        for rule_id, rule in self.escalation_rules.items():
            if not rule.get('enabled', True):
                continue
            
            # Check time-based escalation
            if 'time_based' in rule:
                time_rule = rule['time_based']
                levels = time_rule.get('levels', [])
                
                for level_config in levels:
                    wait_time = level_config.get('wait_minutes', 0) * 60
                    
                    if (current_time - alert.timestamp >= wait_time and
                        state['escalation_level'] < level_config.get('level', 0)):
                        
                        self._escalate_alert(alert, level_config, rule_id)
                        state['escalation_level'] = level_config.get('level', 0)
                        state['last_escalation'] = current_time
                        break
            
            # Check severity-based escalation
            if 'severity_based' in rule:
                severity_rule = rule['severity_based']
                severity_levels = severity_rule.get('levels', {})
                
                if alert.severity in severity_levels:
                    level_config = severity_levels[alert.severity]
                    
                    if state['escalation_level'] < level_config.get('level', 0):
                        self._escalate_alert(alert, level_config, rule_id)
                        state['escalation_level'] = level_config.get('level', 0)
                        state['last_escalation'] = current_time
    
    def _escalate_alert(self, alert: Alert, level_config: Dict[str, Any], rule_id: str):
        """Escalate alert to next level"""
        escalation_channels = level_config.get('channels', [])
        message = level_config.get('message', f"Alert escalated: {alert.rule_name}")
        
        # Create escalation alert
        escalation_alert = Alert(
            id=f"{alert.id}_escalation",
            rule_id=alert.rule_id,
            rule_name=alert.rule_name,
            timestamp=time.time(),
            severity=alert.severity,
            status='firing',
            labels=alert.labels.copy(),
            annotations=alert.annotations.copy()
        )
        
        escalation_alert.annotations['escalation_message'] = message
        escalation_alert.annotations['escalation_level'] = level_config.get('level', 0)
        escalation_alert.annotations['escalation_reason'] = 'time_based' if 'time_based' in rule_id else 'severity_based'
        
        logging.info(f"Alert escalated: {alert.rule_name} - Level {level_config.get('level', 0)}")
        
        # Send escalation notifications
        self.notification_manager.send_alert_notification(escalation_alert, escalation_channels)

class AlertManager:
    """Main alert management system"""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.rule_engine = AlertRuleEngine()
        self.notification_manager = NotificationManager()
        self.escalation_manager = EscalationManager(self.notification_manager)
        
        # Storage
        self.alert_storage = AlertStorage(config.get('storage', {}))
        
        # Statistics
        self.stats = {
            'total_alerts': 0,
            'active_alerts': 0,
            'resolved_alerts': 0,
            'notifications_sent': 0,
            'notifications_failed': 0
        }
        
    def start(self):
        """Start alert management system"""
        # Load rules and notifications from storage
        self._load_configuration()
        
        # Start background tasks
        self._start_background_tasks()
        
        logging.info("Alert management system started")
    
    def stop(self):
        """Stop alert management system"""
        # Save current state
        self._save_configuration()
        logging.info("Alert management system stopped")
    
    def _load_configuration(self):
        """Load rules and notifications from storage"""
        try:
            # Load rules
            rules_data = self.alert_storage.load_rules()
            for rule_data in rules_data:
                rule = AlertRule(**rule_data)
                self.rule_engine.add_rule(rule)
            
            # Load notifications
            notifications_data = self.alert_storage.load_notifications()
            for notification_data in notifications_data:
                notification = Notification(**notification_data)
                self.notification_manager.add_notification(notification)
            
            # Load escalation rules
            escalation_data = self.alert_storage.load_escalation_rules()
            for rule_id, rule_config in escalation_data.items():
                self.escalation_manager.add_escalation_rule(rule_id, rule_config)
                
        except Exception as e:
            logging.error(f"Error loading configuration: {e}")
    
    def _save_configuration(self):
        """Save current configuration to storage"""
        try:
            # Save rules
            rules_data = [asdict(rule) for rule in self.rule_engine.rules.values()]
            self.alert_storage.save_rules(rules_data)
            
            # Save notifications
            notifications_data = [asdict(notification) for notification in self.notification_manager.notifications.values()]
            self.alert_storage.save_notifications(notifications_data)
            
            # Save escalation rules
            self.alert_storage.save_escalation_rules(self.escalation_manager.escalation_rules)
            
        except Exception as e:
            logging.error(f"Error saving configuration: {e}")
    
    def _start_background_tasks(self):
        """Start background monitoring tasks"""
        # Start escalation monitoring
        escalation_thread = threading.Thread(target=self._escalation_monitor)
        escalation_thread.daemon = True
        escalation_thread.start()
        
        # Start cleanup task
        cleanup_thread = threading.Thread(target=self._cleanup_old_alerts)
        cleanup_thread.daemon = True
        cleanup_thread.start()
    
    def evaluate_metrics(self, metrics_data: Dict[str, Any]):
        """Evaluate metrics against alert rules"""
        try:
            self.rule_engine.evaluate_rules(metrics_data)
            
            # Get triggered alerts
            active_alerts = self.rule_engine.get_active_alerts()
            
            for alert in active_alerts:
                # Send notifications
                self.notification_manager.send_alert_notification(alert)
                
                # Store alert
                self.alert_storage.store_alert(alert)
                
                # Update statistics
                self.stats['total_alerts'] += 1
                self.stats['active_alerts'] += 1
                
        except Exception as e:
            logging.error(f"Error evaluating metrics: {e}")
    
    def _escalation_monitor(self):
        """Monitor alerts for escalation"""
        while True:
            try:
                current_time = time.time()
                active_alerts = self.rule_engine.get_active_alerts()
                
                for alert in active_alerts:
                    self.escalation_manager.check_escalation(alert, current_time)
                
                time.sleep(60)  # Check every minute
                
            except Exception as e:
                logging.error(f"Error in escalation monitor: {e}")
                time.sleep(10)
    
    def _cleanup_old_alerts(self):
        """Clean up old resolved alerts"""
        while True:
            try:
                cutoff_time = time.time() - (30 * 24 * 3600)  # 30 days
                self.alert_storage.cleanup_old_alerts(cutoff_time)
                
                # Update statistics
                all_alerts = self.rule_engine.get_all_alerts()
                self.stats['active_alerts'] = len([a for a in all_alerts if a.status == 'firing'])
                self.stats['resolved_alerts'] = len([a for a in all_alerts if a.status == 'resolved'])
                
                time.sleep(3600)  # Clean up every hour
                
            except Exception as e:
                logging.error(f"Error in alert cleanup: {e}")
                time.sleep(10)
    
    def acknowledge_alert(self, alert_id: str, user: str = None):
        """Acknowledge an alert"""
        # This would implement alert acknowledgment
        # For now, just log it
        logging.info(f"Alert acknowledged: {alert_id} by {user}")
    
    def resolve_alert(self, alert_id: str, user: str = None):
        """Manually resolve an alert"""
        # This would implement manual alert resolution
        # For now, just log it
        logging.info(f"Alert resolved: {alert_id} by {user}")
    
    def get_alerts(self, status: str = None, severity: str = None, limit: int = 100) -> List[Alert]:
        """Get alerts with optional filtering"""
        all_alerts = self.rule_engine.get_all_alerts()
        
        filtered_alerts = all_alerts
        
        if status:
            filtered_alerts = [a for a in filtered_alerts if a.status == status]
        
        if severity:
            filtered_alerts = [a for a in filtered_alerts if a.severity == severity]
        
        return sorted(filtered_alerts, key=lambda x: x.timestamp, reverse=True)[:limit]
    
    def get_statistics(self) -> Dict[str, Any]:
        """Get alert system statistics"""
        return {
            **self.stats,
            'rules_count': len(self.rule_engine.rules),
            'notifications_count': len(self.notification_manager.notifications),
            'escalation_rules_count': len(self.escalation_manager.escalation_rules),
            'pending_alerts': len(self.rule_engine.get_pending_alerts())
        }

class AlertStorage:
    """Storage management for alerts, rules, and notifications"""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.db_path = config.get('database_path', 'monitoring_alerts.db')
        
    def store_alert(self, alert: Alert):
        """Store alert in database"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            # Create table if it doesn't exist
            cursor.execute('''
                CREATE TABLE IF NOT EXISTS alerts (
                    id TEXT PRIMARY KEY,
                    rule_id TEXT,
                    rule_name TEXT,
                    timestamp REAL,
                    severity TEXT,
                    status TEXT,
                    labels TEXT,
                    annotations TEXT,
                    values TEXT,
                    fingerprint TEXT
                )
            ''')
            
            cursor.execute('''
                INSERT OR REPLACE INTO alerts (
                    id, rule_id, rule_name, timestamp, severity, status,
                    labels, annotations, values, fingerprint
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ''', (
                alert.id, alert.rule_id, alert.rule_name, alert.timestamp,
                alert.severity, alert.status, json.dumps(alert.labels),
                json.dumps(alert.annotations), json.dumps(alert.values),
                alert.fingerprint
            ))
            
            conn.commit()
            conn.close()
        except Exception as e:
            logging.error(f"Error storing alert: {e}")
    
    def load_rules(self) -> List[Dict[str, Any]]:
        """Load alert rules from database"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            cursor.execute('SELECT rule_data FROM alert_rules')
            rows = cursor.fetchall()
            conn.close()
            
            return [json.loads(row[0]) for row in rows]
        except Exception as e:
            logging.error(f"Error loading rules: {e}")
            return []
    
    def save_rules(self, rules_data: List[Dict[str, Any]]):
        """Save alert rules to database"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            # Create table if it doesn't exist
            cursor.execute('''
                CREATE TABLE IF NOT EXISTS alert_rules (
                    rule_id TEXT PRIMARY KEY,
                    rule_data TEXT
                )
            ''')
            
            # Clear existing rules
            cursor.execute('DELETE FROM alert_rules')
            
            # Insert new rules
            for rule_data in rules_data:
                cursor.execute(
                    'INSERT INTO alert_rules (rule_id, rule_data) VALUES (?, ?)',
                    (rule_data['id'], json.dumps(rule_data))
                )
            
            conn.commit()
            conn.close()
        except Exception as e:
            logging.error(f"Error saving rules: {e}")
    
    def load_notifications(self) -> List[Dict[str, Any]]:
        """Load notifications from database"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            cursor.execute('SELECT notification_data FROM notifications')
            rows = cursor.fetchall()
            conn.close()
            
            return [json.loads(row[0]) for row in rows]
        except Exception as e:
            logging.error(f"Error loading notifications: {e}")
            return []
    
    def save_notifications(self, notifications_data: List[Dict[str, Any]]):
        """Save notifications to database"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            # Create table if it doesn't exist
            cursor.execute('''
                CREATE TABLE IF NOT EXISTS notifications (
                    notification_id TEXT PRIMARY KEY,
                    notification_data TEXT
                )
            ''')
            
            # Clear existing notifications
            cursor.execute('DELETE FROM notifications')
            
            # Insert new notifications
            for notification_data in notifications_data:
                cursor.execute(
                    'INSERT INTO notifications (notification_id, notification_data) VALUES (?, ?)',
                    (notification_data['id'], json.dumps(notification_data))
                )
            
            conn.commit()
            conn.close()
        except Exception as e:
            logging.error(f"Error saving notifications: {e}")
    
    def load_escalation_rules(self) -> Dict[str, Any]:
        """Load escalation rules from database"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            cursor.execute('SELECT rule_id, rule_data FROM escalation_rules')
            rows = cursor.fetchall()
            conn.close()
            
            return {row[0]: json.loads(row[1]) for row in rows}
        except Exception as e:
            logging.error(f"Error loading escalation rules: {e}")
            return {}
    
    def save_escalation_rules(self, escalation_rules: Dict[str, Any]):
        """Save escalation rules to database"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            # Create table if it doesn't exist
            cursor.execute('''
                CREATE TABLE IF NOT EXISTS escalation_rules (
                    rule_id TEXT PRIMARY KEY,
                    rule_data TEXT
                )
            ''')
            
            # Clear existing rules
            cursor.execute('DELETE FROM escalation_rules')
            
            # Insert new rules
            for rule_id, rule_data in escalation_rules.items():
                cursor.execute(
                    'INSERT INTO escalation_rules (rule_id, rule_data) VALUES (?, ?)',
                    (rule_id, json.dumps(rule_data))
                )
            
            conn.commit()
            conn.close()
        except Exception as e:
            logging.error(f"Error saving escalation rules: {e}")
    
    def cleanup_old_alerts(self, cutoff_time: float):
        """Clean up old alerts"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            cursor.execute('DELETE FROM alerts WHERE timestamp < ?', (cutoff_time,))
            deleted_count = cursor.rowcount
            
            conn.commit()
            conn.close()
            
            if deleted_count > 0:
                logging.info(f"Cleaned up {deleted_count} old alerts")
        except Exception as e:
            logging.error(f"Error cleaning up old alerts: {e}")

def create_default_rules() -> List[AlertRule]:
    """Create default alert rules"""
    return [
        AlertRule(
            id='high_cpu',
            name='High CPU Usage',
            description='CPU usage is above 80%',
            enabled=True,
            severity='warning',
            metric_name='cpu_usage',
            operator='>',
            threshold=80.0,
            duration=300,  # 5 minutes
            aggregation='avg',
            labels={'component': 'cpu'},
            annotations={'description': 'CPU usage has been above 80% for 5 minutes'},
            notification_channels=['email', 'slack']
        ),
        AlertRule(
            id='critical_cpu',
            name='Critical CPU Usage',
            description='CPU usage is above 90%',
            enabled=True,
            severity='critical',
            metric_name='cpu_usage',
            operator='>',
            threshold=90.0,
            duration=180,  # 3 minutes
            aggregation='avg',
            labels={'component': 'cpu'},
            annotations={'description': 'CPU usage is critically high'},
            notification_channels=['email', 'slack', 'pagerduty']
        ),
        AlertRule(
            id='high_memory',
            name='High Memory Usage',
            description='Memory usage is above 85%',
            enabled=True,
            severity='warning',
            metric_name='memory_usage',
            operator='>',
            threshold=85.0,
            duration=300,
            aggregation='avg',
            labels={'component': 'memory'},
            annotations={'description': 'Memory usage is above 85%'},
            notification_channels=['email']
        ),
        AlertRule(
            id='high_disk',
            name='High Disk Usage',
            description='Disk usage is above 90%',
            enabled=True,
            severity='critical',
            metric_name='disk_usage',
            operator='>',
            threshold=90.0,
            duration=60,
            aggregation='max',
            labels={'component': 'disk'},
            annotations={'description': 'Disk usage is critically high'},
            notification_channels=['email', 'slack', 'pagerduty']
        )
    ]

def create_default_notifications() -> List[Notification]:
    """Create default notification channels"""
    return [
        Notification(
            id='email_admin',
            name='Admin Email',
            type='email',
            config={
                'smtp_server': 'smtp.example.com',
                'smtp_port': 587,
                'from_email': 'alerts@multios.edu',
                'to_emails': ['admin@multios.edu'],
                'use_tls': True,
                'username': 'alerts@multios.edu',
                'password': 'password'
            },
            enabled=True,
            rate_limiting={'max_per_hour': 50}
        ),
        Notification(
            id='slack_alerts',
            name='Slack Alerts',
            type='slack',
            config={
                'webhook_url': 'https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK'
            },
            enabled=True,
            rate_limiting={'max_per_hour': 100}
        )
    ]

def main():
    """Main function to run the alert system"""
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )
    
    # Load configuration
    config_file = Path(__file__).parent.parent / 'config' / 'monitoring.yaml'
    
    if config_file.exists():
        with open(config_file, 'r') as f:
            config = yaml.safe_load(f)
    else:
        config = {
            'alerting': {
                'enabled': True,
                'storage': {
                    'database_path': 'monitoring_alerts.db'
                }
            }
        }
    
    # Create and start alert manager
    alert_manager = AlertManager(config.get('alerting', {}))
    
    try:
        # Start alert management
        alert_manager.start()
        
        # Add default rules and notifications
        default_rules = create_default_rules()
        for rule in default_rules:
            alert_manager.rule_engine.add_rule(rule)
        
        default_notifications = create_default_notifications()
        for notification in default_notifications:
            alert_manager.notification_manager.add_notification(notification)
        
        # Simulate metric evaluation
        while True:
            time.sleep(10)
            
            # Simulate metrics
            import random
            metrics_data = {
                'cpu_usage': random.uniform(10, 95),
                'memory_usage': random.uniform(20, 90),
                'disk_usage': random.uniform(15, 85),
                'network_in': random.randint(1000, 10000),
                'network_out': random.randint(1000, 10000)
            }
            
            alert_manager.evaluate_metrics(metrics_data)
            
    except KeyboardInterrupt:
        logging.info("Shutting down alert system...")
        alert_manager.stop()

if __name__ == '__main__':
    main()