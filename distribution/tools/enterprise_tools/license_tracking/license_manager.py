"""
Software License Tracking and Compliance Management System
"""

import os
import json
import csv
import logging
import hashlib
import subprocess
from pathlib import Path
from typing import Dict, List, Optional, Any
from datetime import datetime, timedelta
import re

from ..core.models import LicenseInfo, LicenseType, SystemInfo
from ..core.utils import execute_command, load_config, save_config

class LicenseManager:
    """Manager for software license tracking and compliance"""
    
    def __init__(self, config_path: str = "/etc/multios-enterprise/licenses.yaml"):
        self.config_path = config_path
        self.licenses = {}
        self.license_usage = {}
        self.compliance_rules = {}
        self.logger = logging.getLogger(__name__)
        
        self._load_configuration()
        self._setup_directories()
        self._load_licenses()
    
    def _load_configuration(self) -> None:
        """Load license management configuration"""
        # Default configuration
        self.config = {
            'compliance': {
                'check_interval_days': 7,
                'alert_threshold_percent': 90,
                'auto_expire_warnings_days': 30
            },
            'audit': {
                'log_allocation': True,
                'log_deallocation': True,
                'retention_days': 365
            },
            'reporting': {
                'generate_monthly_reports': True,
                'send_expiry_notifications': True
            },
            'integrations': {
                'active_directory': False,
                'inventory_management': True,
                'deployment_system': True
            }
        }
        
        # Load configuration file if exists
        if os.path.exists(self.config_path):
            try:
                loaded_config = load_config(self.config_path)
                self.config.update(loaded_config)
            except Exception as e:
                self.logger.warning(f"Failed to load config file: {e}")
    
    def _setup_directories(self) -> None:
        """Create license management directories"""
        directories = [
            "/var/lib/multios-enterprise/licenses",
            "/var/lib/multios-enterprise/license_reports",
            "/var/log/multios-enterprise/licenses",
            "/var/cache/multios-enterprise/license_checks"
        ]
        
        for directory in directories:
            Path(directory).mkdir(parents=True, exist_ok=True)
    
    def _load_licenses(self) -> None:
        """Load existing licenses from storage"""
        licenses_dir = Path("/var/lib/multios-enterprise/licenses")
        
        for license_file in licenses_dir.glob("*.json"):
            try:
                with open(license_file, 'r') as f:
                    license_data = json.load(f)
                
                # Convert string fields to appropriate types
                license_data['license_type'] = LicenseType(license_data['license_type'])
                license_data['purchase_date'] = datetime.fromisoformat(license_data['purchase_date'])
                if license_data.get('expiry_date'):
                    license_data['expiry_date'] = datetime.fromisoformat(license_data['expiry_date'])
                
                license = LicenseInfo(**license_data)
                self.licenses[license.license_id] = license
                
            except Exception as e:
                self.logger.error(f"Failed to load license from {license_file}: {e}")
        
        self.logger.info(f"Loaded {len(self.licenses)} licenses")
    
    def add_license(self, license_data: Dict[str, Any]) -> Optional[str]:
        """Add a new software license"""
        try:
            # Validate license data
            validation_result = self._validate_license_data(license_data)
            if not validation_result['valid']:
                self.logger.error(f"Invalid license data: {validation_result['error']}")
                return None
            
            # Generate license ID if not provided
            license_id = license_data.get('license_id') or self._generate_license_id(
                license_data['software_name'], license_data['vendor']
            )
            
            # Create license object
            license_info = LicenseInfo(
                license_id=license_id,
                software_name=license_data['software_name'],
                license_type=LicenseType(license_data['license_type']),
                total_licenses=license_data['total_licenses'],
                used_licenses=0,
                purchase_date=datetime.fromisoformat(license_data['purchase_date']),
                expiry_date=datetime.fromisoformat(license_data['expiry_date']) if license_data.get('expiry_date') else None,
                vendor=license_data['vendor'],
                license_key=license_data['license_key'],
                assigned_systems=[],
                compliance_status=True,
                cost_per_license=license_data.get('cost_per_license', 0.0)
            )
            
            # Store license
            self.licenses[license_id] = license_info
            
            # Save to storage
            self._save_license(license_info)
            
            # Setup compliance monitoring
            self._setup_compliance_monitoring(license_info)
            
            self.logger.info(f"Added license: {license_info.software_name} ({license_id})")
            return license_id
            
        except Exception as e:
            self.logger.error(f"Failed to add license: {e}")
            return None
    
    def _validate_license_data(self, license_data: Dict[str, Any]) -> Dict[str, Any]:
        """Validate license data"""
        required_fields = ['software_name', 'license_type', 'total_licenses', 'vendor', 'license_key']
        
        # Check required fields
        for field in required_fields:
            if field not in license_data or not license_data[field]:
                return {'valid': False, 'error': f'Missing required field: {field}'}
        
        # Validate license type
        try:
            LicenseType(license_data['license_type'])
        except ValueError:
            return {'valid': False, 'error': f'Invalid license type: {license_data["license_type"]}'}
        
        # Validate total licenses
        if not isinstance(license_data['total_licenses'], int) or license_data['total_licenses'] < 1:
            return {'valid': False, 'error': 'Total licenses must be a positive integer'}
        
        # Validate dates if provided
        if license_data.get('purchase_date'):
            try:
                datetime.fromisoformat(license_data['purchase_date'])
            except ValueError:
                return {'valid': False, 'error': 'Invalid purchase date format'}
        
        if license_data.get('expiry_date'):
            try:
                datetime.fromisoformat(license_data['expiry_date'])
            except ValueError:
                return {'valid': False, 'error': 'Invalid expiry date format'}
        
        # Check for duplicate license key
        for license in self.licenses.values():
            if license.license_key == license_data['license_key']:
                return {'valid': False, 'error': 'License key already exists'}
        
        return {'valid': True, 'error': None}
    
    def _generate_license_id(self, software_name: str, vendor: str) -> str:
        """Generate unique license ID"""
        # Create hash based on software name and vendor
        combined = f"{software_name}_{vendor}_{datetime.now().isoformat()}"
        hash_obj = hashlib.md5(combined.encode())
        return f"LIC-{hash_obj.hexdigest()[:8].upper()}"
    
    def _save_license(self, license_info: LicenseInfo) -> None:
        """Save license information to storage"""
        try:
            license_file = Path("/var/lib/multios-enterprise/licenses") / f"{license_info.license_id}.json"
            
            license_data = {
                'license_id': license_info.license_id,
                'software_name': license_info.software_name,
                'license_type': license_info.license_type.value,
                'total_licenses': license_info.total_licenses,
                'used_licenses': license_info.used_licenses,
                'purchase_date': license_info.purchase_date.isoformat(),
                'expiry_date': license_info.expiry_date.isoformat() if license_info.expiry_date else None,
                'vendor': license_info.vendor,
                'license_key': license_info.license_key,
                'assigned_systems': license_info.assigned_systems,
                'compliance_status': license_info.compliance_status,
                'cost_per_license': license_info.cost_per_license
            }
            
            with open(license_file, 'w') as f:
                json.dump(license_data, f, indent=2)
                
        except Exception as e:
            self.logger.error(f"Failed to save license {license_info.license_id}: {e}")
    
    def assign_license(self, license_id: str, system_id: str) -> bool:
        """Assign a license to a system"""
        try:
            if license_id not in self.licenses:
                self.logger.error(f"License {license_id} not found")
                return False
            
            license_info = self.licenses[license_id]
            
            # Check if license has capacity
            if license_info.used_licenses >= license_info.total_licenses:
                self.logger.error(f"License {license_id} has no available capacity")
                return False
            
            # Check if system already assigned
            if system_id in license_info.assigned_systems:
                self.logger.warning(f"System {system_id} already assigned to license {license_id}")
                return True
            
            # Assign license
            license_info.assigned_systems.append(system_id)
            license_info.used_licenses += 1
            
            # Update compliance status
            usage_percent = (license_info.used_licenses / license_info.total_licenses) * 100
            alert_threshold = self.config['compliance']['alert_threshold_percent']
            license_info.compliance_status = usage_percent <= alert_threshold
            
            # Save updated license
            self._save_license(license_info)
            
            # Log assignment
            self._log_license_event('allocation', license_id, system_id)
            
            self.logger.info(f"Assigned license {license_id} to system {system_id}")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to assign license {license_id}: {e}")
            return False
    
    def unassign_license(self, license_id: str, system_id: str) -> bool:
        """Unassign a license from a system"""
        try:
            if license_id not in self.licenses:
                self.logger.error(f"License {license_id} not found")
                return False
            
            license_info = self.licenses[license_id]
            
            # Check if system is assigned
            if system_id not in license_info.assigned_systems:
                self.logger.warning(f"System {system_id} not assigned to license {license_id}")
                return True
            
            # Unassign license
            license_info.assigned_systems.remove(system_id)
            license_info.used_licenses -= 1
            
            # Update compliance status
            usage_percent = (license_info.used_licenses / license_info.total_licenses) * 100
            alert_threshold = self.config['compliance']['alert_threshold_percent']
            license_info.compliance_status = usage_percent <= alert_threshold
            
            # Save updated license
            self._save_license(license_info)
            
            # Log unassignment
            self._log_license_event('deallocation', license_id, system_id)
            
            self.logger.info(f"Unassigned license {license_id} from system {system_id}")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to unassign license {license_id}: {e}")
            return False
    
    def _log_license_event(self, event_type: str, license_id: str, system_id: str) -> None:
        """Log license allocation/deallocation event"""
        try:
            log_entry = {
                'timestamp': datetime.now().isoformat(),
                'event_type': event_type,
                'license_id': license_id,
                'system_id': system_id
            }
            
            log_file = Path("/var/log/multios-enterprise/licenses") / f"{datetime.now().strftime('%Y-%m-%d')}.log"
            with open(log_file, 'a') as f:
                f.write(json.dumps(log_entry) + '\n')
                
        except Exception as e:
            self.logger.error(f"Failed to log license event: {e}")
    
    def bulk_assign_licenses(self, assignments: List[Dict[str, str]]) -> Dict[str, Any]:
        """Assign multiple licenses in bulk"""
        results = {
            'total': len(assignments),
            'successful': 0,
            'failed': 0,
            'results': []
        }
        
        for assignment in assignments:
            license_id = assignment['license_id']
            system_id = assignment['system_id']
            
            success = self.assign_license(license_id, system_id)
            
            result = {
                'license_id': license_id,
                'system_id': system_id,
                'success': success
            }
            
            results['results'].append(result)
            
            if success:
                results['successful'] += 1
            else:
                results['failed'] += 1
        
        self.logger.info(f"Bulk license assignment completed: {results['successful']}/{results['total']} successful")
        return results
    
    def check_compliance(self) -> Dict[str, Any]:
        """Check license compliance across all licenses"""
        try:
            compliance_report = {
                'timestamp': datetime.now().isoformat(),
                'total_licenses': len(self.licenses),
                'compliant_licenses': 0,
                'non_compliant_licenses': 0,
                'expiring_licenses': [],
                'expired_licenses': [],
                'overused_licenses': [],
                'details': []
            }
            
            alert_threshold = self.config['compliance']['alert_threshold_percent']
            warning_days = self.config['compliance']['auto_expire_warnings_days']
            
            for license_id, license_info in self.licenses.items():
                # Check usage compliance
                usage_percent = (license_info.used_licenses / license_info.total_licenses) * 100
                
                if usage_percent > 100:
                    compliance_report['overused_licenses'].append({
                        'license_id': license_id,
                        'software_name': license_info.software_name,
                        'used_licenses': license_info.used_licenses,
                        'total_licenses': license_info.total_licenses,
                        'usage_percent': usage_percent
                    })
                
                # Check expiry dates
                if license_info.expiry_date:
                    days_to_expiry = (license_info.expiry_date - datetime.now()).days
                    
                    if days_to_expiry < 0:
                        compliance_report['expired_licenses'].append({
                            'license_id': license_id,
                            'software_name': license_info.software_name,
                            'expiry_date': license_info.expiry_date.isoformat(),
                            'days_expired': abs(days_to_expiry)
                        })
                    elif days_to_expiry <= warning_days:
                        compliance_report['expiring_licenses'].append({
                            'license_id': license_id,
                            'software_name': license_info.software_name,
                            'expiry_date': license_info.expiry_date.isoformat(),
                            'days_to_expiry': days_to_expiry
                        })
                
                # Check overall compliance
                if license_info.compliance_status:
                    compliance_report['compliant_licenses'] += 1
                else:
                    compliance_report['non_compliant_licenses'] += 1
                
                # Add detailed information
                license_detail = {
                    'license_id': license_id,
                    'software_name': license_info.software_name,
                    'license_type': license_info.license_type.value,
                    'vendor': license_info.vendor,
                    'total_licenses': license_info.total_licenses,
                    'used_licenses': license_info.used_licenses,
                    'usage_percent': usage_percent,
                    'compliance_status': license_info.compliance_status,
                    'expiry_date': license_info.expiry_date.isoformat() if license_info.expiry_date else None,
                    'assigned_systems': len(license_info.assigned_systems)
                }
                
                compliance_report['details'].append(license_detail)
            
            # Save compliance report
            self._save_compliance_report(compliance_report)
            
            return compliance_report
            
        except Exception as e:
            self.logger.error(f"Failed to check compliance: {e}")
            return {}
    
    def _save_compliance_report(self, report: Dict[str, Any]) -> None:
        """Save compliance report to file"""
        try:
            report_file = Path("/var/lib/multios-enterprise/license_reports") / f"compliance_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"
            
            with open(report_file, 'w') as f:
                json.dump(report, f, indent=2)
                
        except Exception as e:
            self.logger.error(f"Failed to save compliance report: {e}")
    
    def _setup_compliance_monitoring(self, license_info: LicenseInfo) -> None:
        """Setup compliance monitoring for a license"""
        try:
            # Create monitoring entry
            monitoring_entry = {
                'license_id': license_info.license_id,
                'software_name': license_info.software_name,
                'alert_threshold': self.config['compliance']['alert_threshold_percent'],
                'next_check': (datetime.now() + timedelta(days=self.config['compliance']['check_interval_days'])).isoformat()
            }
            
            # In a full implementation, this would schedule automated checks
            self.logger.info(f"Setup compliance monitoring for {license_info.software_name}")
            
        except Exception as e:
            self.logger.error(f"Failed to setup compliance monitoring: {e}")
    
    def get_license(self, license_id: str) -> Optional[LicenseInfo]:
        """Get license information by ID"""
        return self.licenses.get(license_id)
    
    def list_licenses(self, software_name: Optional[str] = None, 
                     vendor: Optional[str] = None) -> List[LicenseInfo]:
        """List licenses with optional filtering"""
        licenses = list(self.licenses.values())
        
        if software_name:
            licenses = [lic for lic in licenses if lic.software_name == software_name]
        
        if vendor:
            licenses = [lic for lic in licenses if lic.vendor == vendor]
        
        return licenses
    
    def update_license(self, license_id: str, updates: Dict[str, Any]) -> bool:
        """Update license information"""
        try:
            if license_id not in self.licenses:
                self.logger.error(f"License {license_id} not found")
                return False
            
            license_info = self.licenses[license_id]
            
            # Update allowed fields
            allowed_updates = ['total_licenses', 'vendor', 'license_key', 
                             'expiry_date', 'cost_per_license']
            
            for field, value in updates.items():
                if field in allowed_updates:
                    if field == 'expiry_date' and value:
                        value = datetime.fromisoformat(value)
                    
                    setattr(license_info, field, value)
            
            # Recalculate usage percentage and compliance status
            usage_percent = (license_info.used_licenses / license_info.total_licenses) * 100
            alert_threshold = self.config['compliance']['alert_threshold_percent']
            license_info.compliance_status = usage_percent <= alert_threshold
            
            # Save updated license
            self._save_license(license_info)
            
            self.logger.info(f"Updated license: {license_info.software_name}")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to update license {license_id}: {e}")
            return False
    
    def delete_license(self, license_id: str) -> bool:
        """Delete a license"""
        try:
            if license_id not in self.licenses:
                self.logger.error(f"License {license_id} not found")
                return False
            
            license_info = self.licenses[license_id]
            
            # Check if license has active assignments
            if license_info.used_licenses > 0:
                self.logger.error(f"Cannot delete license {license_id} with active assignments")
                return False
            
            # Remove from registry
            del self.licenses[license_id]
            
            # Remove license file
            license_file = Path("/var/lib/multios-enterprise/licenses") / f"{license_id}.json"
            if license_file.exists():
                license_file.unlink()
            
            self.logger.info(f"Deleted license: {license_info.software_name}")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to delete license {license_id}: {e}")
            return False
    
    def generate_compliance_report(self, format_type: str = 'json') -> str:
        """Generate comprehensive compliance report"""
        try:
            compliance_data = self.check_compliance()
            
            if format_type.lower() == 'json':
                return json.dumps(compliance_data, indent=2)
            elif format_type.lower() == 'csv':
                return self._convert_compliance_to_csv(compliance_data)
            else:
                raise ValueError(f"Unsupported format: {format_type}")
                
        except Exception as e:
            self.logger.error(f"Failed to generate compliance report: {e}")
            return ""
    
    def _convert_compliance_to_csv(self, compliance_data: Dict[str, Any]) -> str:
        """Convert compliance data to CSV format"""
        import io
        
        output = io.StringIO()
        writer = csv.writer(output)
        
        # Write header
        writer.writerow([
            'License ID', 'Software Name', 'License Type', 'Vendor', 'Total Licenses',
            'Used Licenses', 'Usage %', 'Compliance Status', 'Expiry Date', 
            'Assigned Systems Count'
        ])
        
        # Write license data
        for license_detail in compliance_data.get('details', []):
            writer.writerow([
                license_detail['license_id'],
                license_detail['software_name'],
                license_detail['license_type'],
                license_detail['vendor'],
                license_detail['total_licenses'],
                license_detail['used_licenses'],
                f"{license_detail['usage_percent']:.1f}%",
                'Compliant' if license_detail['compliance_status'] else 'Non-Compliant',
                license_detail['expiry_date'] or 'Never',
                license_detail['assigned_systems']
            ])
        
        return output.getvalue()
    
    def import_licenses_from_csv(self, csv_path: str) -> Dict[str, Any]:
        """Import licenses from CSV file"""
        results = {
            'total': 0,
            'successful': 0,
            'failed': 0,
            'errors': []
        }
        
        try:
            licenses_data = []
            
            with open(csv_path, 'r', encoding='utf-8') as csvfile:
                reader = csv.DictReader(csvfile)
                
                for row in reader:
                    licenses_data.append({
                        'software_name': row.get('software_name', '').strip(),
                        'license_type': row.get('license_type', '').strip(),
                        'total_licenses': int(row.get('total_licenses', 0)),
                        'vendor': row.get('vendor', '').strip(),
                        'license_key': row.get('license_key', '').strip(),
                        'purchase_date': row.get('purchase_date', '').strip(),
                        'expiry_date': row.get('expiry_date', '').strip() or None,
                        'cost_per_license': float(row.get('cost_per_license', 0))
                    })
            
            results['total'] = len(licenses_data)
            
            # Add licenses in bulk
            for license_data in licenses_data:
                license_id = self.add_license(license_data)
                
                if license_id:
                    results['successful'] += 1
                else:
                    results['failed'] += 1
            
        except Exception as e:
            self.logger.error(f"Failed to import licenses from CSV: {e}")
            results['errors'].append(str(e))
        
        return results
