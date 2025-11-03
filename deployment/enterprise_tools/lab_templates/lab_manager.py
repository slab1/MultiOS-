"""
Educational Lab Environment Templates Management
"""

import json
import logging
from datetime import datetime
from typing import Dict, List, Optional, Any
from pathlib import Path

from ..core.models import LabTemplate

class LabManager:
    """Manager for educational lab environment templates"""
    
    def __init__(self):
        self.lab_templates = {}
        self.logger = logging.getLogger(__name__)
        
        self._setup_directories()
        self._load_predefined_templates()
    
    def _setup_directories(self) -> None:
        """Create lab template directories"""
        directories = [
            "/var/lib/multios-enterprise/labs",
            "/var/lib/multios-enterprise/labs/templates",
            "/var/lib/multios-enterprise/labs/sessions"
        ]
        
        for directory in directories:
            Path(directory).mkdir(parents=True, exist_ok=True)
    
    def _load_predefined_templates(self) -> None:
        """Load predefined lab templates"""
        # Programming Lab
        self.create_template({
            'template_id': 'prog-lab-001',
            'name': 'Introductory Programming Lab',
            'description': 'Basic programming concepts and hands-on coding exercises',
            'target_audience': 'Computer Science 101 students',
            'estimated_duration': 120,
            'required_systems': 25,
            'software_requirements': [
                'python3', 'gcc', 'code', 'git', 'vim'
            ],
            'hardware_requirements': {
                'min_memory_gb': 8,
                'min_storage_gb': 50,
                'network_required': True
            },
            'setup_scripts': ['configure_python_env', 'setup_git_repos'],
            'learning_objectives': [
                'Understand basic programming concepts',
                'Write and test simple programs',
                'Use version control systems'
            ]
        })
        
        # Web Development Lab
        self.create_template({
            'template_id': 'web-dev-001',
            'name': 'Web Development Workshop',
            'description': 'HTML, CSS, JavaScript, and modern web frameworks',
            'target_audience': 'Web Development course students',
            'estimated_duration': 180,
            'required_systems': 20,
            'software_requirements': [
                'nodejs', 'npm', 'code', 'git', 'nginx'
            ],
            'hardware_requirements': {
                'min_memory_gb': 12,
                'min_storage_gb': 100,
                'network_required': True
            },
            'setup_scripts': ['install_node_packages', 'configure_web_server'],
            'learning_objectives': [
                'Create responsive web pages',
                'Understand client-server architecture',
                'Deploy web applications'
            ]
        })
        
        # Data Science Lab
        self.create_template({
            'template_id': 'data-sci-001',
            'name': 'Data Science Fundamentals',
            'description': 'Data analysis, visualization, and machine learning basics',
            'target_audience': 'Data Science and Statistics students',
            'estimated_duration': 150,
            'required_systems': 15,
            'software_requirements': [
                'python3', 'jupyter', 'numpy', 'pandas', 'matplotlib'
            ],
            'hardware_requirements': {
                'min_memory_gb': 16,
                'min_storage_gb': 200,
                'network_required': True
            },
            'setup_scripts': ['install_python_packages', 'setup_jupyter'],
            'learning_objectives': [
                'Analyze datasets using Python',
                'Create data visualizations',
                'Apply basic machine learning'
            ]
        })
        
        # Cybersecurity Lab
        self.create_template({
            'template_id': 'cyber-sec-001',
            'name': 'Cybersecurity Fundamentals',
            'description': 'Network security, ethical hacking, and security tools',
            'target_audience': 'Cybersecurity students',
            'estimated_duration': 120,
            'required_systems': 20,
            'software_requirements': [
                'wireshark', 'nmap', 'metasploit', 'burpsuite'
            ],
            'hardware_requirements': {
                'min_memory_gb': 8,
                'min_storage_gb': 100,
                'network_required': True
            },
            'setup_scripts': ['install_security_tools', 'configure_isolated_network'],
            'learning_objectives': [
                'Understand network security principles',
                'Use security analysis tools',
                'Implement security measures'
            ]
        })
    
    def create_template(self, template_data: Dict[str, Any]) -> bool:
        """Create a new lab template"""
        try:
            # Validate required fields
            required_fields = ['template_id', 'name', 'description', 'target_audience', 
                             'estimated_duration', 'required_systems']
            
            for field in required_fields:
                if field not in template_data:
                    self.logger.error(f"Missing required field: {field}")
                    return False
            
            # Create lab template object
            lab_template = LabTemplate(
                template_id=template_data['template_id'],
                name=template_data['name'],
                description=template_data['description'],
                target_audience=template_data['target_audience'],
                estimated_duration=template_data['estimated_duration'],
                required_systems=template_data['required_systems'],
                software_requirements=template_data.get('software_requirements', []),
                hardware_requirements=template_data.get('hardware_requirements', {}),
                network_config=template_data.get('network_config', {}),
                setup_scripts=template_data.get('setup_scripts', []),
                cleanup_scripts=template_data.get('cleanup_scripts', []),
                learning_objectives=template_data.get('learning_objectives', [])
            )
            
            # Store template
            self.lab_templates[lab_template.template_id] = lab_template
            
            # Save to storage
            self._save_template(lab_template)
            
            self.logger.info(f"Created lab template: {lab_template.name}")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to create lab template: {e}")
            return False
    
    def deploy_lab_environment(self, template_id: str, site_id: str, 
                              session_data: Dict[str, Any]) -> Optional[str]:
        """Deploy a lab environment based on template"""
        try:
            if template_id not in self.lab_templates:
                self.logger.error(f"Lab template {template_id} not found")
                return None
            
            template = self.lab_templates[template_id]
            
            # Create lab session
            session_id = f"SESSION-{session_data.get('session_name', 'unnamed')}-{datetime.now().strftime('%Y%m%d_%H%M%S')}"
            
            session = {
                'session_id': session_id,
                'template_id': template_id,
                'template_name': template.name,
                'site_id': site_id,
                'session_data': session_data,
                'status': 'deploying',
                'created': datetime.now().isoformat(),
                'systems_allocated': [],
                'software_installed': [],
                'setup_completed': False
            }
            
            # Simulate lab deployment
            session['status'] = 'active'
            session['setup_completed'] = True
            session['systems_allocated'] = [f"system-{i:03d}" for i in range(1, template.required_systems + 1)]
            session['software_installed'] = template.software_requirements
            
            self._save_lab_session(session)
            
            self.logger.info(f"Deployed lab environment {template.name} at site {site_id}")
            return session_id
            
        except Exception as e:
            self.logger.error(f"Failed to deploy lab environment: {e}")
            return None
    
    def _save_template(self, lab_template: LabTemplate) -> None:
        """Save lab template to storage"""
        try:
            template_file = Path("/var/lib/multios-enterprise/labs/templates") / f"{lab_template.template_id}.json"
            
            template_data = {
                'template_id': lab_template.template_id,
                'name': lab_template.name,
                'description': lab_template.description,
                'target_audience': lab_template.target_audience,
                'estimated_duration': lab_template.estimated_duration,
                'required_systems': lab_template.required_systems,
                'software_requirements': lab_template.software_requirements,
                'hardware_requirements': lab_template.hardware_requirements,
                'network_config': lab_template.network_config,
                'setup_scripts': lab_template.setup_scripts,
                'cleanup_scripts': lab_template.cleanup_scripts,
                'learning_objectives': lab_template.learning_objectives
            }
            
            with open(template_file, 'w') as f:
                json.dump(template_data, f, indent=2)
                
        except Exception as e:
            self.logger.error(f"Failed to save lab template: {e}")
    
    def _save_lab_session(self, session: Dict[str, Any]) -> None:
        """Save lab session data"""
        try:
            session_file = Path("/var/lib/multios-enterprise/labs/sessions") / f"{session['session_id']}.json"
            
            with open(session_file, 'w') as f:
                json.dump(session, f, indent=2)
                
        except Exception as e:
            self.logger.error(f"Failed to save lab session: {e}")
    
    def list_templates(self) -> List[LabTemplate]:
        """List all available lab templates"""
        return list(self.lab_templates.values())
    
    def get_template(self, template_id: str) -> Optional[LabTemplate]:
        """Get a specific lab template"""
        return self.lab_templates.get(template_id)
    
    def get_available_templates(self, audience: Optional[str] = None) -> List[LabTemplate]:
        """Get templates filtered by target audience"""
        templates = list(self.lab_templates.values())
        
        if audience:
            templates = [t for t in templates if audience.lower() in t.target_audience.lower()]
        
        return templates
