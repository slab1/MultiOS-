#!/usr/bin/env python3
"""
MultiOS Enterprise Deployment Tools - Installation Validation Script
"""

import sys
import importlib.util
from pathlib import Path

def validate_module(module_path: str, module_name: str) -> bool:
    """Validate that a module can be imported"""
    try:
        spec = importlib.util.spec_from_file_location(module_name, module_path)
        if spec is None:
            return False
        module = importlib.util.module_from_spec(spec)
        # Just try to load the module to validate syntax
        spec.loader.exec_module(module)
        return True
    except Exception as e:
        print(f"❌ Failed to import {module_name}: {e}")
        return False

def main():
    """Validate all enterprise tools components"""
    base_path = Path(__file__).parent
    
    print("🔍 Validating MultiOS Enterprise Deployment Tools...")
    print("=" * 60)
    
    # Define all modules to validate
    modules_to_validate = [
        ("core/__init__.py", "core.__init__"),
        ("core/models.py", "core.models"),
        ("core/utils.py", "core.utils"),
        ("core/manager.py", "core.manager"),
        ("pxe_installer/pxe_server.py", "pxe_installer.pxe_server"),
        ("config_management/template_manager.py", "config_management.template_manager"),
        ("user_management/user_manager.py", "user_management.user_manager"),
        ("license_tracking/license_manager.py", "license_tracking.license_manager"),
        ("system_monitoring/monitor.py", "system_monitoring.monitor"),
        ("software_deployment/package_manager.py", "software_deployment.package_manager"),
        ("update_distribution/update_server.py", "update_distribution.update_server"),
        ("inventory_management/inventory.py", "inventory_management.inventory"),
        ("ldap_integration/directory_integration.py", "ldap_integration.directory_integration"),
        ("automation/deployment_automation.py", "automation.deployment_automation"),
        ("lab_templates/lab_manager.py", "lab_templates.lab_manager"),
        ("resource_scheduling/scheduler.py", "resource_scheduling.scheduler"),
        ("analytics/analytics_engine.py", "analytics.analytics_engine"),
    ]
    
    success_count = 0
    total_count = len(modules_to_validate)
    
    for module_file, module_name in modules_to_validate:
        module_path = base_path / module_file
        if module_path.exists():
            if validate_module(str(module_path), module_name):
                print(f"✅ {module_name}")
                success_count += 1
            else:
                print(f"❌ {module_name}")
        else:
            print(f"⚠️  {module_name} - File not found")
            total_count -= 1
    
    print("=" * 60)
    print(f"📊 Validation Results: {success_count}/{total_count} modules loaded successfully")
    
    if success_count == total_count:
        print("🎉 All enterprise tools components validated successfully!")
        print("\n🚀 Ready for deployment!")
        print("\nNext steps:")
        print("1. Run: ./scripts/setup_multios_enterprise.sh")
        print("2. Configure: /etc/multios-enterprise/config.yaml")
        print("3. Start using: ./scripts/multios-enterprise --help")
        return 0
    else:
        print("⚠️  Some components failed validation. Check the errors above.")
        return 1

if __name__ == "__main__":
    sys.exit(main())