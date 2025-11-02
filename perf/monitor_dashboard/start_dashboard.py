#!/usr/bin/env python3
"""
Performance Monitoring Dashboard - Main Startup Script
Initializes and starts all components of the monitoring system
"""

import os
import sys
import argparse
import signal
import logging
import time
from pathlib import Path

# Add the backend directory to Python path
backend_dir = Path(__file__).parent / 'backend'
sys.path.insert(0, str(backend_dir))

from config_manager import ConfigManager
from system_monitor import SystemMonitor
from alert_manager import AlertManager
from report_generator import ReportGenerator
from web_dashboard import app, socketio

class PerformanceDashboard:
    def __init__(self):
        self.config = ConfigManager()
        self.monitor = None
        self.alert_manager = None
        self.report_generator = None
        self.web_running = False
        self.setup_logging()
    
    def setup_logging(self):
        """Setup logging configuration"""
        log_config = self.config.get_logging_config()
        
        # Create logs directory
        os.makedirs('logs', exist_ok=True)
        
        # Setup logging
        logging.basicConfig(
            level=getattr(logging, log_config.get('level', 'INFO').upper()),
            format=log_config.get('format', '%(asctime)s - %(name)s - %(levelname)s - %(message)s'),
            handlers=[
                logging.FileHandler(log_config.get('file', 'logs/monitor.log')),
                logging.StreamHandler()
            ]
        )
        
        self.logger = logging.getLogger('PerformanceDashboard')
        self.logger.info("Performance Monitoring Dashboard starting...")
    
    def initialize_components(self):
        """Initialize all dashboard components"""
        try:
            self.logger.info("Initializing components...")
            
            # Initialize monitor
            db_config = self.config.get_database_config()
            self.monitor = SystemMonitor(
                db_path=db_config.get('path', 'data/monitor.db'),
                history_size=self.config.get('monitoring.history_size', 1000)
            )
            
            # Update thresholds from config
            thresholds = self.config.get_thresholds()
            self.monitor.thresholds.update(thresholds)
            
            # Initialize alert manager
            self.alert_manager = AlertManager(db_config.get('path', 'data/monitor.db'))
            
            # Configure notifications
            notif_config = self.config.get_notification_config()
            
            # Email notifications
            if notif_config.get('email', {}).get('enabled'):
                email_config = notif_config['email']
                self.alert_manager.configure_email_notifications(
                    smtp_server=email_config['smtp_server'],
                    smtp_port=email_config['smtp_port'],
                    username=email_config['username'],
                    password=email_config['password'],
                    to_addresses=email_config['to_addresses']
                )
            
            # Webhook notifications
            if notif_config.get('webhook', {}).get('enabled'):
                webhook_config = notif_config['webhook']
                self.alert_manager.configure_webhook_notifications(
                    webhook_url=webhook_config['url'],
                    headers=webhook_config.get('headers', {})
                )
            
            # Initialize report generator
            self.report_generator = ReportGenerator(db_config.get('path', 'data/monitor.db'))
            
            self.logger.info("Components initialized successfully")
            return True
            
        except Exception as e:
            self.logger.error(f"Error initializing components: {e}")
            return False
    
    def start_web_dashboard(self):
        """Start the web dashboard"""
        try:
            web_config = self.config.get_web_config()
            
            self.logger.info("Starting web dashboard...")
            self.logger.info(f"Dashboard will be available at http://{web_config.get('host', '0.0.0.0')}:{web_config.get('port', 5000)}")
            
            self.web_running = True
            
            # Start the Flask-SocketIO app
            socketio.run(
                app,
                debug=web_config.get('debug', False),
                host=web_config.get('host', '0.0.0.0'),
                port=web_config.get('port', 5000)
            )
            
        except Exception as e:
            self.logger.error(f"Error starting web dashboard: {e}")
            self.web_running = False
    
    def start_monitoring_only(self):
        """Start monitoring without web interface"""
        try:
            monitoring_config = self.config.get_monitoring_config()
            interval = monitoring_config.get('interval', 5)
            
            self.logger.info(f"Starting monitoring with {interval}s interval...")
            self.monitor.start_monitoring(interval)
            
            # Keep running until interrupted
            while True:
                time.sleep(60)  # Check every minute
            
        except KeyboardInterrupt:
            self.logger.info("Monitoring stopped by user")
            self.monitor.stop_monitoring()
        except Exception as e:
            self.logger.error(f"Error in monitoring: {e}")
    
    def run_cli_only(self, command: str):
        """Run CLI command and exit"""
        try:
            from monitor_cli import cli
            sys.argv = ['monitor', command] + sys.argv[2:]
            cli()
        except Exception as e:
            self.logger.error(f"Error running CLI: {e}")
    
    def generate_initial_report(self):
        """Generate an initial performance report"""
        try:
            self.logger.info("Generating initial performance report...")
            
            os.makedirs('reports', exist_ok=True)
            
            # Generate reports in different formats
            hours = 24
            timestamp = time.strftime('%Y%m%d_%H%M%S')
            
            # PDF report
            pdf_path = f'reports/initial_report_{timestamp}.pdf'
            self.report_generator.generate_pdf_report(hours, pdf_path)
            
            # HTML report
            html_path = f'reports/initial_report_{timestamp}.html'
            self.report_generator.generate_html_report(hours, html_path)
            
            self.logger.info("Initial reports generated successfully")
            
        except Exception as e:
            self.logger.error(f"Error generating initial report: {e}")
    
    def setup_directories(self):
        """Create necessary directories"""
        directories = [
            'data',
            'logs',
            'reports',
            'config',
            'backend/templates',
            'backend/static'
        ]
        
        for directory in directories:
            os.makedirs(directory, exist_ok=True)
    
    def signal_handler(self, signum, frame):
        """Handle shutdown signals"""
        self.logger.info(f"Received signal {signum}, shutting down...")
        
        if self.monitor:
            self.monitor.stop_monitoring()
        
        self.logger.info("Shutdown complete")
        sys.exit(0)

def main():
    parser = argparse.ArgumentParser(description='Performance Monitoring Dashboard')
    parser.add_argument('--mode', choices=['web', 'monitor', 'cli', 'report'], 
                       default='web', help='Execution mode')
    parser.add_argument('--config', help='Configuration file path')
    parser.add_argument('--cli-command', help='CLI command to execute')
    parser.add_argument('--report-hours', type=int, default=24, 
                       help='Hours for report generation')
    parser.add_argument('--version', action='version', version='Performance Dashboard v1.0')
    
    args = parser.parse_args()
    
    # Change to script directory
    script_dir = Path(__file__).parent
    os.chdir(script_dir)
    
    # Initialize dashboard
    dashboard = PerformanceDashboard()
    
    # Setup signal handlers
    signal.signal(signal.SIGINT, dashboard.signal_handler)
    signal.signal(signal.SIGTERM, dashboard.signal_handler)
    
    # Create necessary directories
    dashboard.setup_directories()
    
    # Initialize components
    if not dashboard.initialize_components():
        print("Failed to initialize components")
        sys.exit(1)
    
    try:
        if args.mode == 'web':
            # Start web dashboard
            dashboard.start_web_dashboard()
        
        elif args.mode == 'monitor':
            # Start monitoring only
            dashboard.start_monitoring_only()
        
        elif args.mode == 'cli':
            # Run CLI command
            if args.cli_command:
                dashboard.run_cli_only(args.cli_command)
            else:
                print("Please specify a CLI command with --cli-command")
                sys.exit(1)
        
        elif args.mode == 'report':
            # Generate report and exit
            dashboard.generate_initial_report()
        
        else:
            print(f"Unknown mode: {args.mode}")
            sys.exit(1)
    
    except Exception as e:
        logging.error(f"Fatal error: {e}")
        sys.exit(1)

if __name__ == '__main__':
    main()