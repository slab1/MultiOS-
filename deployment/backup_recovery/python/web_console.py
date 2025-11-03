#!/usr/bin/env python3
"""
MultiOS Backup System Web Console
Enterprise-grade web interface for backup management
"""

from flask import Flask, render_template, request, jsonify, send_from_directory, redirect, url_for, flash
from flask_socketio import SocketIO, emit
from werkzeug.utils import secure_filename
import asyncio
import json
import os
import uuid
from datetime import datetime, timedelta
import threading
import queue
from pathlib import Path
import logging
from dataclasses import asdict

# Import our backup manager
from backup_manager import (
    BackupManagementAPI, BackupSpecification, RestoreSpecification,
    StorageLocation, BackupType, StorageType, CompressionAlgorithm,
    LabProfile, LabBackupManager, QuickRestoreManager, CloudBackupIntegration,
    BackupScheduler
)

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Flask application setup
app = Flask(__name__)
app.secret_key = 'multios-backup-console-secret-key'
socketio = SocketIO(app, cors_allowed_origins="*")

# Global state
backup_api = BackupManagementAPI()
job_status_queue = queue.Queue()

# Routes
@app.route('/')
def dashboard():
    """Main dashboard"""
    return render_template('dashboard.html')

@app.route('/backup')
def backup_page():
    """Backup management page"""
    return render_template('backup.html')

@app.route('/restore')
def restore_page():
    """Restore management page"""
    return render_template('restore.html')

@app.route('/schedules')
def schedules_page():
    """Schedule management page"""
    return render_template('schedules.html')

@app.route('/storage')
def storage_page():
    """Storage management page"""
    return render_template('storage.html')

@app.route('/lab-profiles')
def lab_profiles_page():
    """Lab profiles management page"""
    return render_template('lab_profiles.html')

@app.route('/recovery-media')
def recovery_media_page():
    """Recovery media management page"""
    return render_template('recovery_media.html')

@app.route('/settings')
def settings_page():
    """Settings page"""
    return render_template('settings.html')

# API Routes
@app.route('/api/status')
def get_status():
    """Get system status"""
    try:
        # This would normally call the backup API
        status = {
            'system': 'running',
            'active_backups': 2,
            'active_restores': 1,
            'total_backups': 25,
            'storage_used': '45.2 GB',
            'storage_available': '124.8 GB',
            'last_backup': '2024-01-15 14:30:00',
            'next_scheduled': '2024-01-16 02:00:00'
        }
        return jsonify(status)
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/api/backups', methods=['GET'])
def list_backups():
    """List all backups"""
    try:
        # This would call the backup API
        backups = [
            {
                'id': 'backup-001',
                'name': 'Full System Backup',
                'type': 'full',
                'status': 'completed',
                'size': '12.5 GB',
                'files': 12543,
                'created': '2024-01-15 14:30:00',
                'compressed': True,
                'encrypted': False
            },
            {
                'id': 'backup-002',
                'name': 'Incremental Home',
                'type': 'incremental',
                'status': 'completed',
                'size': '2.1 GB',
                'files': 3421,
                'created': '2024-01-15 16:45:00',
                'compressed': True,
                'encrypted': True
            }
        ]
        return jsonify(backups)
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/api/backups', methods=['POST'])
def create_backup():
    """Create a new backup"""
    try:
        data = request.get_json()
        
        # Validate required fields
        required_fields = ['name', 'type', 'sources']
        for field in required_fields:
            if field not in data:
                return jsonify({'error': f'Missing required field: {field}'}), 400
        
        # Create backup specification
        backup_spec = BackupSpecification(
            job_id=str(uuid.uuid4()),
            name=data['name'],
            backup_type=BackupType(data['type']),
            sources=[Path(s) for s in data['sources']],
            destination=StorageLocation(
                id=data.get('destination_id', 'local-default'),
                storage_type=StorageType.LOCAL,
                path=data.get('destination_path', '/var/lib/multios/backup'),
                is_default=True
            ),
            compression=CompressionAlgorithm(data.get('compression', 'zstd')),
            encryption_enabled=data.get('encryption', False),
            description=data.get('description'),
            verify_integrity=data.get('verify', True)
        )
        
        # This would create the backup through the API
        # job = await backup_api.create_backup(backup_spec)
        
        return jsonify({'success': True, 'job_id': backup_spec.job_id}), 201
        
    except Exception as e:
        logger.error(f"Error creating backup: {e}")
        return jsonify({'error': str(e)}), 500

@app.route('/api/backups/<backup_id>/start', methods=['POST'])
def start_backup(backup_id):
    """Start a backup job"""
    try:
        # This would start the backup through the API
        return jsonify({'success': True, 'job_id': backup_id})
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/api/backups/<backup_id>', methods=['DELETE'])
def delete_backup(backup_id):
    """Delete a backup"""
    try:
        # This would delete the backup through the API
        return jsonify({'success': True})
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/api/backups/<backup_id>/verify', methods=['GET'])
def verify_backup(backup_id):
    """Verify backup integrity"""
    try:
        quick = request.args.get('quick', 'false').lower() == 'true'
        
        # This would verify the backup through the API
        verification_result = {
            'backup_id': backup_id,
            'status': 'passed',
            'verified_at': datetime.now().isoformat(),
            'files_verified': 12543,
            'files_failed': 0,
            'assessment': 'Backup verification completed successfully',
            'integrity_checks': [
                {
                    'check_type': 'File Integrity',
                    'status': 'passed',
                    'details': 'All files verified'
                },
                {
                    'check_type': 'Metadata Integrity',
                    'status': 'passed',
                    'details': 'Metadata structure valid'
                }
            ]
        }
        
        return jsonify(verification_result)
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/api/restores', methods=['GET'])
def list_restores():
    """List restore jobs"""
    try:
        restores = [
            {
                'id': 'restore-001',
                'backup_id': 'backup-001',
                'target_path': '/tmp/restore',
                'status': 'running',
                'progress': 65,
                'created': '2024-01-15 18:30:00',
                'files_restored': 8234,
                'bytes_restored': '8.2 GB'
            }
        ]
        return jsonify(restores)
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/api/restores', methods=['POST'])
def create_restore():
    """Create a restore job"""
    try:
        data = request.get_json()
        
        # Validate required fields
        required_fields = ['backup_id', 'target_path']
        for field in required_fields:
            if field not in data:
                return jsonify({'error': f'Missing required field: {field}'}), 400
        
        restore_spec = RestoreSpecification(
            job_id=str(uuid.uuid4()),
            backup_id=data['backup_id'],
            target_path=Path(data['target_path']),
            include_paths=[Path(p) for p in data.get('include_paths', [])],
            exclude_paths=[Path(p) for p in data.get('exclude_paths', [])],
            verify_restore=data.get('verify', True),
            restore_permissions=data.get('restore_permissions', True)
        )
        
        # This would create the restore through the API
        # job = await backup_api.restore_backup(restore_spec)
        
        return jsonify({'success': True, 'job_id': restore_spec.job_id}), 201
        
    except Exception as e:
        logger.error(f"Error creating restore: {e}")
        return jsonify({'error': str(e)}), 500

@app.route('/api/quick-restore', methods=['POST'])
def quick_restore():
    """Quick restore operations"""
    try:
        data = request.get_json()
        restore_type = data.get('type')
        target_path = Path(data.get('target_path', '/tmp/quick-restore'))
        force = data.get('force', False)
        
        if restore_type == 'system_files':
            # Quick restore system files
            result = {'status': 'success', 'message': 'System files restoration completed'}
        elif restore_type == 'drivers':
            # Quick restore drivers
            result = {'status': 'success', 'message': 'Driver files restoration completed'}
        elif restore_type == 'documents':
            # Quick restore documents
            result = {'status': 'success', 'message': 'User documents restoration completed'}
        else:
            return jsonify({'error': 'Invalid restore type'}), 400
        
        return jsonify(result)
        
    except Exception as e:
        logger.error(f"Error in quick restore: {e}")
        return jsonify({'error': str(e)}), 500

@app.route('/api/lab-profiles', methods=['GET'])
def list_lab_profiles():
    """List lab profiles"""
    try:
        profiles = [
            {
                'id': 'cs101',
                'name': 'CS101 Introduction to Programming',
                'description': 'Lab environment for CS101 students',
                'sources': ['/home/students', '/opt/cs101'],
                'retention': '30 days',
                'schedule': 'Daily at 3:00 AM'
            },
            {
                'id': 'cs301',
                'name': 'CS301 Operating Systems',
                'description': 'OS development lab environment',
                'sources': ['/home/students', '/var/cs301', '/opt/os-dev'],
                'retention': '90 days',
                'schedule': 'Weekly on Sundays at 4:00 AM'
            }
        ]
        return jsonify(profiles)
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/api/lab-profiles', methods=['POST'])
def create_lab_profile():
    """Create a lab profile"""
    try:
        data = request.get_json()
        
        profile = LabProfile(
            id=data['id'],
            name=data['name'],
            description=data['description'],
            default_sources=[Path(s) for s in data['sources']],
            default_retention=data['retention'],
            schedule_settings=data['schedule'],
            custom_config=data.get('custom_config', {})
        )
        
        # This would save the profile through the LabBackupManager
        return jsonify({'success': True, 'profile_id': profile.id}), 201
        
    except Exception as e:
        logger.error(f"Error creating lab profile: {e}")
        return jsonify({'error': str(e)}), 500

@app.route('/api/lab-profiles/<profile_id>/apply', methods=['POST'])
def apply_lab_profile(profile_id):
    """Apply a lab profile"""
    try:
        # This would apply the profile and start a backup
        result = {
            'status': 'success',
            'message': f'Lab profile {profile_id} applied successfully',
            'backup_job_id': str(uuid.uuid4())
        }
        return jsonify(result)
    except Exception as e:
        logger.error(f"Error applying lab profile: {e}")
        return jsonify({'error': str(e)}), 500

@app.route('/api/recovery-media', methods=['GET'])
def list_recovery_media():
    """List recovery media"""
    try:
        media = [
            {
                'name': 'multios-recovery-2024-01-15',
                'type': 'ISO',
                'size': '2.1 GB',
                'created': '2024-01-15 14:00:00',
                'bootable': True,
                'tested': True
            },
            {
                'name': 'multios-recovery-2024-01-10',
                'type': 'USB',
                'size': '2.1 GB',
                'created': '2024-01-10 10:30:00',
                'bootable': True,
                'tested': True
            }
        ]
        return jsonify(media)
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/api/recovery-media', methods=['POST'])
def create_recovery_media():
    """Create bootable recovery media"""
    try:
        data = request.get_json()
        
        media_config = {
            'name': data.get('name', f'multios-recovery-{datetime.now().strftime("%Y-%m-%d")}'),
            'include_backups': data.get('include_backups', []),
            'create_usb': data.get('create_usb', False),
            'usb_device': data.get('usb_device'),
            'compression': data.get('compression', 'zstd'),
            'encryption': data.get('encryption', False)
        }
        
        # This would create the recovery media
        result = {
            'status': 'success',
            'message': 'Recovery media creation started',
            'media_name': media_config['name'],
            'estimated_time': '10-15 minutes'
        }
        
        return jsonify(result)
    except Exception as e:
        logger.error(f"Error creating recovery media: {e}")
        return jsonify({'error': str(e)}), 500

@app.route('/api/storage/locations', methods=['GET'])
def list_storage_locations():
    """List storage locations"""
    try:
        locations = [
            {
                'id': 'local-default',
                'name': 'Local Storage',
                'type': 'local',
                'path': '/var/lib/multios/backup',
                'available': '124.8 GB',
                'used': '45.2 GB',
                'is_default': True
            },
            {
                'id': 'network-nas',
                'name': 'Network Storage',
                'type': 'network',
                'path': '192.168.1.100:/backup',
                'available': '1.2 TB',
                'used': '245 GB',
                'is_default': False,
                'status': 'connected'
            }
        ]
        return jsonify(locations)
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/api/storage/locations', methods=['POST'])
def add_storage_location():
    """Add a storage location"""
    try:
        data = request.get_json()
        
        location = {
            'id': str(uuid.uuid4()),
            'name': data['name'],
            'type': data['type'],
            'path': data['path'],
            'config': data.get('config', {}),
            'is_default': data.get('is_default', False)
        }
        
        # This would save the location
        return jsonify({'success': True, 'location_id': location['id']}), 201
        
    except Exception as e:
        logger.error(f"Error adding storage location: {e}")
        return jsonify({'error': str(e)}), 500

@app.route('/api/schedules', methods=['GET'])
def list_schedules():
    """List backup schedules"""
    try:
        schedules = [
            {
                'id': 'daily-home',
                'name': 'Daily Home Backup',
                'cron': '0 2 * * *',
                'backup_type': 'incremental',
                'sources': ['/home'],
                'enabled': True,
                'next_run': '2024-01-16 02:00:00',
                'last_run': '2024-01-15 02:00:00',
                'status': 'active'
            },
            {
                'id': 'weekly-full',
                'name': 'Weekly Full System',
                'cron': '0 3 * * 0',
                'backup_type': 'full',
                'sources': ['/'],
                'enabled': True,
                'next_run': '2024-01-21 03:00:00',
                'last_run': '2024-01-14 03:00:00',
                'status': 'active'
            }
        ]
        return jsonify(schedules)
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/api/schedules', methods=['POST'])
def create_schedule():
    """Create a backup schedule"""
    try:
        data = request.get_json()
        
        schedule = {
            'id': str(uuid.uuid4()),
            'name': data['name'],
            'cron': data['cron_expression'],
            'backup_type': data['backup_type'],
            'sources': data['sources'],
            'enabled': data.get('enabled', True),
            'created': datetime.now().isoformat()
        }
        
        # This would save the schedule
        return jsonify({'success': True, 'schedule_id': schedule['id']}), 201
        
    except Exception as e:
        logger.error(f"Error creating schedule: {e}")
        return jsonify({'error': str(e)}), 500

# WebSocket events
@socketio.on('connect')
def handle_connect():
    """Handle client connection"""
    logger.info('Client connected to WebSocket')
    emit('status', {'message': 'Connected to backup system'})

@socketio.on('disconnect')
def handle_disconnect():
    """Handle client disconnection"""
    logger.info('Client disconnected from WebSocket')

@socketio.on('request_job_status')
def handle_job_status_request():
    """Handle request for job status updates"""
    emit('status_update', {'message': 'Job status monitoring started'})

# Background tasks
def monitor_jobs():
    """Background task to monitor job status"""
    while True:
        try:
            # This would check job status from the backup system
            job_updates = []  # Placeholder for actual job updates
            
            for update in job_updates:
                socketio.emit('job_update', update)
            
            socketio.sleep(5)  # Check every 5 seconds
        except Exception as e:
            logger.error(f"Error monitoring jobs: {e}")
            socketio.sleep(10)

# Static files
@app.route('/static/<path:filename>')
def static_files(filename):
    """Serve static files"""
    return send_from_directory('static', filename)

# Error handlers
@app.errorhandler(404)
def not_found_error(error):
    return render_template('404.html'), 404

@app.errorhandler(500)
def internal_error(error):
    return render_template('500.html'), 500

# Templates (basic HTML structure)
def create_templates():
    """Create basic HTML templates"""
    templates_dir = Path('templates')
    templates_dir.mkdir(exist_ok=True)
    
    # Base template
    base_template = '''<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}MultiOS Backup Console{% endblock %}</title>
    <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.1.3/dist/css/bootstrap.min.css" rel="stylesheet">
    <link href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.7.2/font/bootstrap-icons.css" rel="stylesheet">
    {% block styles %}{% endblock %}
</head>
<body>
    <nav class="navbar navbar-expand-lg navbar-dark bg-primary">
        <div class="container-fluid">
            <a class="navbar-brand" href="/">
                <i class="bi bi-shield-check"></i> MultiOS Backup
            </a>
            <button class="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="#navbarNav">
                <span class="navbar-toggler-icon"></span>
            </button>
            <div class="collapse navbar-collapse" id="navbarNav">
                <ul class="navbar-nav">
                    <li class="nav-item">
                        <a class="nav-link" href="/">Dashboard</a>
                    </li>
                    <li class="nav-item">
                        <a class="nav-link" href="/backup">Backup</a>
                    </li>
                    <li class="nav-item">
                        <a class="nav-link" href="/restore">Restore</a>
                    </li>
                    <li class="nav-item">
                        <a class="nav-link" href="/schedules">Schedules</a>
                    </li>
                    <li class="nav-item">
                        <a class="nav-link" href="/storage">Storage</a>
                    </li>
                    <li class="nav-item">
                        <a class="nav-link" href="/lab-profiles">Lab Profiles</a>
                    </li>
                    <li class="nav-item">
                        <a class="nav-link" href="/recovery-media">Recovery Media</a>
                    </li>
                    <li class="nav-item">
                        <a class="nav-link" href="/settings">Settings</a>
                    </li>
                </ul>
            </div>
        </div>
    </nav>
    
    <div class="container-fluid mt-3">
        {% with messages = get_flashed_messages() %}
            {% if messages %}
                <div class="alert alert-info">
                    {% for message in messages %}
                        {{ message }}<br>
                    {% endfor %}
                </div>
            {% endif %}
        {% endwith %}
        
        {% block content %}{% endblock %}
    </div>
    
    <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.1.3/dist/js/bootstrap.bundle.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/socket.io@4.0.1/socket.io.js"></script>
    {% block scripts %}{% endblock %}
</body>
</html>'''
    
    with open(templates_dir / 'base.html', 'w') as f:
        f.write(base_template)
    
    # Dashboard template
    dashboard_template = '''{% extends "base.html" %}
{% block title %}Dashboard - MultiOS Backup{% endblock %}
{% block content %}
<div class="row">
    <div class="col-md-3">
        <div class="card text-white bg-primary">
            <div class="card-body">
                <div class="d-flex justify-content-between">
                    <div>
                        <h5 class="card-title">Active Backups</h5>
                        <h2 id="activeBackups">2</h2>
                    </div>
                    <div class="align-self-center">
                        <i class="bi bi-cloud-arrow-up" style="font-size: 2rem;"></i>
                    </div>
                </div>
            </div>
        </div>
    </div>
    <div class="col-md-3">
        <div class="card text-white bg-success">
            <div class="card-body">
                <div class="d-flex justify-content-between">
                    <div>
                        <h5 class="card-title">Total Backups</h5>
                        <h2 id="totalBackups">25</h2>
                    </div>
                    <div class="align-self-center">
                        <i class="bi bi-archive" style="font-size: 2rem;"></i>
                    </div>
                </div>
            </div>
        </div>
    </div>
    <div class="col-md-3">
        <div class="card text-white bg-info">
            <div class="card-body">
                <div class="d-flex justify-content-between">
                    <div>
                        <h5 class="card-title">Storage Used</h5>
                        <h2 id="storageUsed">45.2 GB</h2>
                    </div>
                    <div class="align-self-center">
                        <i class="bi bi-hdd" style="font-size: 2rem;"></i>
                    </div>
                </div>
            </div>
        </div>
    </div>
    <div class="col-md-3">
        <div class="card text-white bg-warning">
            <div class="card-body">
                <div class="d-flex justify-content-between">
                    <div>
                        <h5 class="card-title">Last Backup</h5>
                        <h6 id="lastBackup">2 hours ago</h6>
                    </div>
                    <div class="align-self-center">
                        <i class="bi bi-clock" style="font-size: 2rem;"></i>
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>

<div class="row mt-4">
    <div class="col-md-8">
        <div class="card">
            <div class="card-header">
                <h5>Recent Backup Jobs</h5>
            </div>
            <div class="card-body">
                <table class="table">
                    <thead>
                        <tr>
                            <th>Name</th>
                            <th>Type</th>
                            <th>Status</th>
                            <th>Size</th>
                            <th>Created</th>
                        </tr>
                    </thead>
                    <tbody id="recentJobs">
                        <tr>
                            <td>System Backup</td>
                            <td>Full</td>
                            <td><span class="badge bg-success">Completed</span></td>
                            <td>12.5 GB</td>
                            <td>2024-01-15 14:30</td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    </div>
    <div class="col-md-4">
        <div class="card">
            <div class="card-header">
                <h5>Quick Actions</h5>
            </div>
            <div class="card-body">
                <button class="btn btn-primary w-100 mb-2" onclick="createBackup()">
                    <i class="bi bi-cloud-arrow-up"></i> Create Backup
                </button>
                <button class="btn btn-success w-100 mb-2" onclick="restoreBackup()">
                    <i class="bi bi-cloud-arrow-down"></i> Restore Backup
                </button>
                <button class="btn btn-info w-100 mb-2" onclick="verifyBackups()">
                    <i class="bi bi-shield-check"></i> Verify All
                </button>
                <button class="btn btn-warning w-100" onclick="createRecoveryMedia()">
                    <i class="bi bi-disc"></i> Recovery Media
                </button>
            </div>
        </div>
    </div>
</div>
{% endblock %}

{% block scripts %}
<script>
const socket = io();

socket.on('connect', function() {
    console.log('Connected to backup system');
});

socket.on('job_update', function(data) {
    console.log('Job update:', data);
    // Update UI with new job status
});

socket.on('status_update', function(data) {
    console.log('Status update:', data);
});

function createBackup() {
    window.location.href = '/backup';
}

function restoreBackup() {
    window.location.href = '/restore';
}

function verifyBackups() {
    // Trigger backup verification
}

function createRecoveryMedia() {
    window.location.href = '/recovery-media';
}
</script>
{% endblock %}'''
    
    with open(templates_dir / 'dashboard.html', 'w') as f:
        f.write(dashboard_template)
    
    # Create other basic templates
    for page in ['backup', 'restore', 'schedules', 'storage', 'lab_profiles', 'recovery_media', 'settings', '404', '500']:
        template_content = f'''{{% extends "base.html" %}}
{{% block title %}}{page.title()} - MultiOS Backup{{% endblock %}}
{{% block content %}}
<div class="row">
    <div class="col-12">
        <h1>{page.title()} Management</h1>
        <p>This page is under development.</p>
    </div>
</div>
{{% endblock %}}'''
        
        with open(templates_dir / f'{page}.html', 'w') as f:
            f.write(template_content)

if __name__ == '__main__':
    # Create templates directory and basic templates
    create_templates()
    
    # Start background job monitoring
    monitor_thread = threading.Thread(target=monitor_jobs, daemon=True)
    monitor_thread.start()
    
    # Run the Flask-SocketIO app
    socketio.run(app, host='0.0.0.0', port=8080, debug=True)