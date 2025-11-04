"""
Hardware and Software Inventory Management System
"""

import os
import json
import csv
import logging
import subprocess
from pathlib import Path
from typing import Dict, List, Optional, Any
from datetime import datetime

from ..core.models import SystemInfo, InventoryItem

class InventoryManager:
    """Manager for hardware and software inventory tracking"""
    
    def __init__(self):
        self.inventory = {}
        self.hardware_inventory = {}
        self.software_inventory = {}
        self.logger = logging.getLogger(__name__)
        
        self._setup_directories()
    
    def _setup_directories(self) -> None:
        """Create inventory directories"""
        directories = [
            "/var/lib/multios-enterprise/inventory",
            "/var/lib/multios-enterprise/inventory/exports",
            "/var/lib/multios-enterprise/inventory/reports"
        ]
        
        for directory in directories:
            Path(directory).mkdir(parents=True, exist_ok=True)
    
    def add_item(self, system_info: SystemInfo) -> bool:
        """Add system to inventory"""
        try:
            self.inventory[system_info.system_id] = system_info
            
            # Create initial hardware inventory entry
            hardware_item = InventoryItem(
                item_id=f"HW-{system_info.system_id}",
                system_id=system_info.system_id,
                item_type="hardware",
                name=f"{system_info.system_type.value.title()} System",
                version="1.0",
                manufacturer="Unknown",
                serial_number=None,
                install_date=datetime.now(),
                specifications={
                    "cpu_model": system_info.cpu_model,
                    "memory_gb": str(system_info.memory_gb),
                    "storage_gb": str(system_info.storage_gb),
                    "network_interface": system_info.network_interface
                }
            )
            
            self.hardware_inventory[f"HW-{system_info.system_id}"] = hardware_item
            
            self.logger.info(f"Added system to inventory: {system_info.hostname}")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to add system to inventory: {e}")
            return False
    
    def remove_system(self, system_id: str) -> bool:
        """Remove system from inventory"""
        try:
            if system_id in self.inventory:
                del self.inventory[system_id]
                
                # Remove all inventory items for this system
                self.hardware_inventory = {k: v for k, v in self.hardware_inventory.items() 
                                         if v.system_id != system_id}
                self.software_inventory = {k: v for k, v in self.software_inventory.items() 
                                         if v.system_id != system_id}
                
                self.logger.info(f"Removed system from inventory: {system_id}")
                return True
            return False
            
        except Exception as e:
            self.logger.error(f"Failed to remove system from inventory: {e}")
            return False
    
    def update_system_status(self, system_id: str, status: 'SystemStatus') -> bool:
        """Update system status in inventory"""
        try:
            if system_id in self.inventory:
                system_info = self.inventory[system_id]
                system_info.last_seen = datetime.now()
                
                # Save updated inventory
                self._save_inventory()
                
                return True
            return False
            
        except Exception as e:
            self.logger.error(f"Failed to update system status: {e}")
            return False
    
    def export_report(self, format_type: str = 'json') -> str:
        """Export complete inventory report"""
        try:
            report_data = {
                'generated': datetime.now().isoformat(),
                'total_systems': len(self.inventory),
                'hardware_items': len(self.hardware_inventory),
                'software_items': len(self.software_inventory),
                'systems': [
                    {
                        'system_id': sys.system_id,
                        'hostname': sys.hostname,
                        'system_type': sys.system_type.value,
                        'cpu_model': sys.cpu_model,
                        'memory_gb': sys.memory_gb,
                        'storage_gb': sys.storage_gb,
                        'site_id': sys.site_id,
                        'location': sys.location,
                        'last_seen': sys.last_seen.isoformat() if sys.last_seen else None
                    }
                    for sys in self.inventory.values()
                ]
            }
            
            if format_type.lower() == 'json':
                return json.dumps(report_data, indent=2)
            elif format_type.lower() == 'csv':
                return self._export_to_csv(report_data)
            else:
                raise ValueError(f"Unsupported format: {format_type}")
                
        except Exception as e:
            self.logger.error(f"Failed to export inventory report: {e}")
            return ""
    
    def _export_to_csv(self, report_data: Dict[str, Any]) -> str:
        """Convert report data to CSV format"""
        import io
        
        output = io.StringIO()
        writer = csv.writer(output)
        
        # Write header
        writer.writerow([
            'System ID', 'Hostname', 'Type', 'CPU Model', 'Memory (GB)', 
            'Storage (GB)', 'Site ID', 'Location', 'Last Seen'
        ])
        
        # Write system data
        for system in report_data['systems']:
            writer.writerow([
                system['system_id'],
                system['hostname'],
                system['system_type'],
                system['cpu_model'],
                system['memory_gb'],
                system['storage_gb'],
                system['site_id'],
                system['location'],
                system['last_seen'] or ''
            ])
        
        return output.getvalue()
    
    def _save_inventory(self) -> None:
        """Save inventory to storage"""
        try:
            inventory_file = Path("/var/lib/multios-enterprise/inventory/inventory.json")
            
            inventory_data = {
                'systems': {
                    sys_id: {
                        'system_id': sys.system_id,
                        'hostname': sys.hostname,
                        'ip_address': sys.ip_address,
                        'mac_address': sys.mac_address,
                        'system_type': sys.system_type.value,
                        'cpu_model': sys.cpu_model,
                        'memory_gb': sys.memory_gb,
                        'storage_gb': sys.storage_gb,
                        'network_interface': sys.network_interface,
                        'site_id': sys.site_id,
                        'location': sys.location,
                        'last_seen': sys.last_seen.isoformat() if sys.last_seen else None
                    }
                    for sys_id, sys in self.inventory.items()
                }
            }
            
            with open(inventory_file, 'w') as f:
                json.dump(inventory_data, f, indent=2)
                
        except Exception as e:
            self.logger.error(f"Failed to save inventory: {e}")
