"""
LMS Integration Module for MultiOS Academic Platform

This module provides compatibility and integration capabilities for various Learning Management Systems
including Canvas, Blackboard, Moodle, and others.
"""

import requests
import json
from typing import Dict, List, Optional, Any
from dataclasses import dataclass
from enum import Enum
import xml.etree.ElementTree as ET
import base64
import hashlib
from datetime import datetime
import os


class LMSPlatform(Enum):
    """Supported LMS platforms"""
    CANVAS = "canvas"
    BLACKBOARD = "blackboard"
    MOODLE = "moodle"
    D2L = "d2l"
    GOOGLE_CLASSROOM = "google_classroom"
    SCORM = "scorm"
    XAPI = "xapi"


class IntegrationMethod(Enum):
    """Methods of LMS integration"""
    API = "api"
    LTI = "lti"
    SCORM = "scorm"
    FILE_IMPORT = "file_import"
    WEBHOOK = "webhook"


@dataclass
class LMSConnection:
    """Configuration for LMS connection"""
    platform: LMSPlatform
    base_url: str
    api_key: Optional[str] = None
    client_id: Optional[str] = None
    client_secret: Optional[str] = None
    access_token: Optional[str] = None
    refresh_token: Optional[str] = None
    timeout: int = 30
    verify_ssl: bool = True
    custom_headers: Optional[Dict[str, str]] = None


@dataclass
class CourseMapping:
    """Maps local course to LMS course"""
    local_course_id: str
    lms_course_id: str
    platform: LMSPlatform
    mapping_data: Dict[str, Any]
    created_at: datetime = None
    updated_at: datetime = None
    
    def __post_init__(self):
        if self.created_at is None:
            self.created_at = datetime.now()
        if self.updated_at is None:
            self.updated_at = datetime.now()


@dataclass
class EnrollmentData:
    """Student enrollment information"""
    student_id: str
    course_id: str
    lms_user_id: str
    lms_course_id: str
    role: str  # student, instructor, ta, etc.
    enrolled_at: datetime = None
    
    def __post_init__(self):
        if self.enrolled_at is None:
            self.enrolled_at = datetime.now()


class CanvasIntegration:
    """Canvas LMS integration"""
    
    def __init__(self, connection: LMSConnection):
        self.connection = connection
        self.base_url = connection.base_url.rstrip('/')
    
    def _make_request(self, method: str, endpoint: str, data: Dict = None) -> Dict[str, Any]:
        """Make authenticated request to Canvas API"""
        headers = {
            'Authorization': f'Bearer {self.connection.access_token}',
            'Content-Type': 'application/json'
        }
        
        if self.connection.custom_headers:
            headers.update(self.connection.custom_headers)
        
        url = f"{self.base_url}/api/v1/{endpoint}"
        
        try:
            if method.upper() == 'GET':
                response = requests.get(url, headers=headers, timeout=self.connection.timeout, 
                                      verify=self.connection.verify_ssl)
            elif method.upper() == 'POST':
                response = requests.post(url, headers=headers, json=data, timeout=self.connection.timeout,
                                       verify=self.connection.verify_ssl)
            elif method.upper() == 'PUT':
                response = requests.put(url, headers=headers, json=data, timeout=self.connection.timeout,
                                       verify=self.connection.verify_ssl)
            elif method.upper() == 'DELETE':
                response = requests.delete(url, headers=headers, timeout=self.connection.timeout,
                                          verify=self.connection.verify_ssl)
            else:
                raise ValueError(f"Unsupported HTTP method: {method}")
            
            response.raise_for_status()
            return response.json()
        
        except requests.exceptions.RequestException as e:
            print(f"Canvas API request failed: {e}")
            return {"error": str(e)}
    
    def create_course(self, name: str, course_code: str, is_public: bool = False) -> Dict[str, Any]:
        """Create a new course in Canvas"""
        course_data = {
            "course": {
                "name": name,
                "course_code": course_code,
                "is_public": is_public,
                "license": "private",
                "term_id": None,
                "account_id": None
            }
        }
        return self._make_request('POST', 'courses', course_data)
    
    def get_course(self, course_id: str) -> Dict[str, Any]:
        """Get course information from Canvas"""
        return self._make_request('GET', f'courses/{course_id}')
    
    def update_course(self, course_id: str, updates: Dict[str, Any]) -> Dict[str, Any]:
        """Update course in Canvas"""
        return self._make_request('PUT', f'courses/{course_id}', {"course": updates})
    
    def enroll_user(self, course_id: str, user_id: str, role: str = 'student') -> Dict[str, Any]:
        """Enroll user in course"""
        enrollment_data = {
            "enrollment": {
                "user_id": user_id,
                "type": "StudentEnrollment",
                "role": role,
                "enrollment_state": "active"
            }
        }
        return self._make_request('POST', f'courses/{course_id}/enrollments', enrollment_data)
    
    def create_assignment(self, course_id: str, assignment_data: Dict[str, Any]) -> Dict[str, Any]:
        """Create assignment in Canvas"""
        return self._make_request('POST', f'courses/{course_id}/assignments', 
                                 {"assignment": assignment_data})
    
    def create_module(self, course_id: str, module_name: str, position: int = 1) -> Dict[str, Any]:
        """Create module in Canvas"""
        module_data = {
            "module": {
                "name": module_name,
                "position": position,
                "require_sequential_progress": False,
                "prerequisite_module_ids": []
            }
        }
        return self._make_request('POST', f'courses/{course_id}/modules', module_data)
    
    def get_gradebook(self, course_id: str) -> Dict[str, Any]:
        """Get gradebook data"""
        return self._make_request('GET', f'courses/{course_id}/assignments')


class BlackboardIntegration:
    """Blackboard LMS integration"""
    
    def __init__(self, connection: LMSConnection):
        self.connection = connection
        self.base_url = connection.base_url.rstrip('/')
    
    def _make_request(self, method: str, endpoint: str, data: Dict = None) -> Dict[str, Any]:
        """Make authenticated request to Blackboard API"""
        headers = {
            'Authorization': f'Bearer {self.connection.access_token}',
            'Content-Type': 'application/json'
        }
        
        url = f"{self.base_url}/learn/api/public/v1/{endpoint}"
        
        try:
            if method.upper() == 'GET':
                response = requests.get(url, headers=headers, timeout=self.connection.timeout,
                                      verify=self.connection.verify_ssl)
            elif method.upper() == 'POST':
                response = requests.post(url, headers=headers, json=data, timeout=self.connection.timeout,
                                       verify=self.connection.verify_ssl)
            else:
                raise ValueError(f"Unsupported HTTP method: {method}")
            
            response.raise_for_status()
            return response.json()
        
        except requests.exceptions.RequestException as e:
            print(f"Blackboard API request failed: {e}")
            return {"error": str(e)}
    
    def create_course(self, course_id: str, name: str, description: str) -> Dict[str, Any]:
        """Create course in Blackboard"""
        course_data = {
            "courseId": course_id,
            "name": name,
            "description": description,
            "allowGuests": False,
            "readOnly": False
        }
        return self._make_request('POST', 'courses', course_data)
    
    def get_course(self, course_id: str) -> Dict[str, Any]:
        """Get course from Blackboard"""
        return self._make_request('GET', f'courses/{course_id}')
    
    def enroll_user(self, course_id: str, user_id: str, role: str = 'Student') -> Dict[str, Any]:
        """Enroll user in course"""
        enrollment_data = {
            "userId": user_id,
            "courseRoleId": role
        }
        return self._make_request('POST', f'courses/{course_id}/users', enrollment_data)


class MoodleIntegration:
    """Moodle LMS integration"""
    
    def __init__(self, connection: LMSConnection):
        self.connection = connection
        self.base_url = connection.base_url.rstrip('/')
    
    def _make_request(self, function: str, params: Dict = None) -> Dict[str, Any]:
        """Make request to Moodle web services"""
        if params is None:
            params = {}
        
        # Add required Moodle parameters
        params.update({
            'wstoken': self.connection.api_key,
            'wsfunction': function,
            'moodlewsrestformat': 'json'
        })
        
        url = f"{self.base_url}/webservice/rest/server.php"
        
        try:
            response = requests.post(url, data=params, timeout=self.connection.timeout,
                                   verify=self.connection.verify_ssl)
            response.raise_for_status()
            return response.json()
        
        except requests.exceptions.RequestException as e:
            print(f"Moodle API request failed: {e}")
            return {"error": str(e)}
    
    def create_course(self, shortname: str, fullname: str, summary: str) -> Dict[str, Any]:
        """Create course in Moodle"""
        params = {
            'courses[0][shortname]': shortname,
            'courses[0][fullname]': fullname,
            'courses[0][summary]': summary
        }
        return self._make_request('core_course_create_courses', params)
    
    def enroll_user(self, course_id: int, user_id: int, role_id: int = 5) -> Dict[str, Any]:
        """Enroll user in course"""
        params = {
            'enrolments[0][courseid]': course_id,
            'enrolments[0][userid]': user_id,
            'enrolments[0][roleid]': role_id
        }
        return self._make_request('enrol_manual_enrol_users', params)


class LTIIntegration:
    """Learning Tools Interoperability (LTI) integration"""
    
    def __init__(self, consumer_key: str, shared_secret: str):
        self.consumer_key = consumer_key
        self.shared_secret = shared_secret
    
    def validate_lti_request(self, request_data: Dict[str, str], signature: str) -> bool:
        """Validate LTI launch request signature"""
        # Create signature based on OAuth 1.0
        base_string = self._create_base_string(request_data)
        expected_signature = base64.b64encode(
            hashlib.sha1((self.shared_secret + '&' + base_string).encode()).digest()
        ).decode()
        
        return signature == expected_signature
    
    def _create_base_string(self, params: Dict[str, str]) -> str:
        """Create OAuth base string"""
        # Simplified implementation - in production, use proper OAuth library
        sorted_params = sorted(params.items())
        param_string = '&'.join(f'{key}={value}' for key, value in sorted_params)
        return f"POST&{param_string}"
    
    def extract_launch_data(self, request_data: Dict[str, str]) -> Dict[str, Any]:
        """Extract data from LTI launch request"""
        return {
            "user_id": request_data.get('user_id'),
            "user_name": request_data.get('lis_person_name_full'),
            "user_email": request_data.get('lis_person_contact_email_primary'),
            "course_id": request_data.get('context_id'),
            "course_title": request_data.get('context_title'),
            "roles": request_data.get('roles', '').split(','),
            "resource_link_id": request_data.get('resource_link_id'),
            "resource_link_title": request_data.get('resource_link_title')
        }


class SCORMPackage:
    """SCORM package handling"""
    
    def __init__(self, package_path: str):
        self.package_path = package_path
        self.manifest = self._parse_manifest()
    
    def _parse_manifest(self) -> Dict[str, Any]:
        """Parse SCORM imsmanifest.xml"""
        manifest_path = os.path.join(self.package_path, 'imsmanifest.xml')
        
        if not os.path.exists(manifest_path):
            return {}
        
        try:
            tree = ET.parse(manifest_path)
            root = tree.getroot()
            
            # Extract basic information
            manifest_data = {
                "identifier": root.get('identifier'),
                "version": root.get('version', '1.0'),
                "metadata": {},
                "organizations": [],
                "resources": []
            }
            
            # Extract metadata
            metadata = root.find('.//lom:general', {'lom': 'http://www.imsglobal.org/xsd/imscp_v1p1'})
            if metadata is not None:
                manifest_data["metadata"]["general"] = self._extract_lom_data(metadata)
            
            # Extract organizations
            for org in root.findall('.//organizations/organization'):
                org_data = {
                    "identifier": org.get('identifier'),
                    "title": org.get('title', ''),
                    "items": []
                }
                
                for item in org.findall('item'):
                    item_data = {
                        "identifier": item.get('identifier'),
                        "identifierref": item.get('identifierref'),
                        "title": item.find('title').text if item.find('title') is not None else ''
                    }
                    org_data["items"].append(item_data)
                
                manifest_data["organizations"].append(org_data)
            
            # Extract resources
            for resource in root.findall('.//resources/resource'):
                res_data = {
                    "identifier": resource.get('identifier'),
                    "type": resource.get('type'),
                    "href": resource.get('href', ''),
                    "files": []
                }
                
                for file_elem in resource.findall('file'):
                    res_data["files"].append(file_elem.get('href'))
                
                manifest_data["resources"].append(res_data)
            
            return manifest_data
        
        except Exception as e:
            print(f"Error parsing SCORM manifest: {e}")
            return {}
    
    def _extract_lom_data(self, element: ET.Element) -> Dict[str, Any]:
        """Extract Learning Object Metadata"""
        return {
            "title": element.find('lom:title', {'lom': 'http://www.imsglobal.org/xsd/imscp_v1p1'}) is not None,
            "description": "Extracted from SCORM manifest"
        }
    
    def get_launch_url(self) -> str:
        """Get the main launch URL for SCORM package"""
        # Return the href of the first resource
        if self.manifest and self.manifest.get("resources"):
            return self.manifest["resources"][0]["href"]
        return ""
    
    def extract_learning_objectives(self) -> List[str]:
        """Extract learning objectives from SCORM package"""
        objectives = []
        
        # This is a simplified extraction - real implementation would parse
        # more deeply into the LOM metadata
        if self.manifest and self.manifest.get("metadata"):
            objectives.append("Learn from SCORM content")
        
        return objectives


class LMSIntegrationManager:
    """Main LMS integration manager"""
    
    def __init__(self):
        self.connections: Dict[str, LMSConnection] = {}
        self.course_mappings: Dict[str, CourseMapping] = {}
        self.enrollments: Dict[str, EnrollmentData] = {}
        self.integrations = {
            LMSPlatform.CANVAS: CanvasIntegration,
            LMSPlatform.BLACKBOARD: BlackboardIntegration,
            LMSPlatform.MOODLE: MoodleIntegration
        }
    
    def add_connection(self, connection_id: str, connection: LMSConnection):
        """Add LMS connection configuration"""
        self.connections[connection_id] = connection
    
    def get_integration(self, connection_id: str):
        """Get integration handler for specific LMS"""
        if connection_id not in self.connections:
            return None
        
        connection = self.connections[connection_id]
        integration_class = self.integrations.get(connection.platform)
        
        if integration_class:
            return integration_class(connection)
        return None
    
    def sync_course_to_lms(self, local_course: Dict[str, Any], connection_id: str, 
                          lms_course_id: str = None) -> Dict[str, Any]:
        """Sync local course to LMS"""
        integration = self.get_integration(connection_id)
        if not integration:
            return {"error": "Integration not found"}
        
        connection = self.connections[connection_id]
        
        # Prepare course data for LMS
        course_data = {
            "name": local_course.get("title"),
            "course_code": local_course.get("code"),
            "description": local_course.get("description"),
            "is_public": local_course.get("is_public", False)
        }
        
        # Create or update course
        if lms_course_id:
            result = integration.update_course(lms_course_id, course_data)
        else:
            result = integration.create_course(
                course_data["name"],
                course_data["course_code"],
                course_data["is_public"]
            )
        
        # Store mapping
        if "id" in result:
            mapping = CourseMapping(
                local_course_id=local_course.get("id"),
                lms_course_id=result["id"],
                platform=connection.platform,
                mapping_data={"last_sync": datetime.now().isoformat()}
            )
            self.course_mappings[local_course.get("id")] = mapping
        
        return result
    
    def sync_enrollment_to_lms(self, enrollment: EnrollmentData, connection_id: str) -> Dict[str, Any]:
        """Sync enrollment to LMS"""
        integration = self.get_integration(connection_id)
        if not integration:
            return {"error": "Integration not found"}
        
        # Determine LMS course ID from mapping
        mapping = None
        for m in self.course_mappings.values():
            if m.local_course_id == enrollment.course_id:
                mapping = m
                break
        
        if not mapping:
            return {"error": "No course mapping found"}
        
        # Enroll user
        return integration.enroll_user(
            mapping.lms_course_id,
            enrollment.lms_user_id,
            enrollment.role
        )
    
    def import_lms_course(self, connection_id: str, lms_course_id: str) -> Dict[str, Any]:
        """Import course from LMS"""
        integration = self.get_integration(connection_id)
        if not integration:
            return {"error": "Integration not found"}
        
        # Get course data from LMS
        course_data = integration.get_course(lms_course_id)
        
        # Convert to local format
        local_course = {
            "title": course_data.get("name", ""),
            "description": course_data.get("public_description", ""),
            "code": course_data.get("course_code", ""),
            "is_public": course_data.get("is_public", False),
            "lms_id": lms_course_id,
            "imported_at": datetime.now().isoformat()
        }
        
        return local_course
    
    def create_lti_launch(self, resource_url: str, consumer_key: str, 
                         user_data: Dict[str, str]) -> Dict[str, str]:
        """Create LTI launch URL"""
        lti = LTIIntegration(consumer_key, "shared_secret")
        
        # Build LTI launch parameters
        params = {
            "lti_message_type": "basic-lti-launch-request",
            "lti_version": "LTI-1p0",
            "resource_link_id": "resource_link_123",
            "resource_link_title": "MultiOS Course",
            "user_id": user_data.get("user_id", "12345"),
            "lis_person_name_full": user_data.get("full_name", ""),
            "lis_person_contact_email_primary": user_data.get("email", ""),
            "roles": "Instructor",
            "context_id": "course_123",
            "context_title": "MultiOS Course",
            "launch_presentation_document_target": "iframe",
            "tool_consumer_instance_guid": "multios_instance",
            "tool_consumer_instance_name": "MultiOS Academic Platform"
        }
        
        # Generate signature
        base_string = lti._create_base_string(params)
        signature = base64.b64encode(
            hashlib.sha1(("shared_secret" + '&' + base_string).encode()).digest()
        ).decode()
        
        params["oauth_signature"] = signature
        
        # Build launch URL
        launch_url = resource_url + "?" + "&".join(
            f"{key}={value}" for key, value in params.items()
        )
        
        return {
            "launch_url": launch_url,
            "parameters": params
        }
    
    def process_scorm_package(self, package_path: str) -> Dict[str, Any]:
        """Process SCORM package"""
        try:
            scorm = SCORMPackage(package_path)
            
            return {
                "manifest": scorm.manifest,
                "launch_url": scorm.get_launch_url(),
                "learning_objectives": scorm.extract_learning_objectives(),
                "resources_count": len(scorm.manifest.get("resources", [])),
                "organizations_count": len(scorm.manifest.get("organizations", []))
            }
        
        except Exception as e:
            return {"error": f"Failed to process SCORM package: {str(e)}"}
    
    def get_integration_status(self) -> Dict[str, Any]:
        """Get status of all integrations"""
        status = {
            "connections": {},
            "total_mappings": len(self.course_mappings),
            "total_enrollments": len(self.enrollments)
        }
        
        for connection_id, connection in self.connections.items():
            # Test connection
            integration = self.get_integration(connection_id)
            connection_status = "unknown"
            
            if integration and connection.base_url:
                # Simple test - try to make a basic request
                connection_status = "connected"
            
            status["connections"][connection_id] = {
                "platform": connection.platform.value,
                "url": connection.base_url,
                "status": connection_status,
                "has_credentials": bool(connection.api_key or connection.access_token)
            }
        
        return status


# Example usage
if __name__ == "__main__":
    # Initialize LMS integration manager
    lms_manager = LMSIntegrationManager()
    
    # Add Canvas connection
    canvas_connection = LMSConnection(
        platform=LMSPlatform.CANVAS,
        base_url="https://canvas.example.edu",
        api_key="your_canvas_api_key"
    )
    lms_manager.add_connection("canvas_main", canvas_connection)
    
    # Add Moodle connection
    moodle_connection = LMSConnection(
        platform=LMSPlatform.MOODLE,
        base_url="https://moodle.example.edu",
        api_key="your_moodle_api_key"
    )
    lms_manager.add_connection("moodle_main", moodle_connection)
    
    print("LMS Integration Manager initialized!")
    
    # Get status
    status = lms_manager.get_integration_status()
    print("Integration Status:")
    print(json.dumps(status, indent=2))
    
    # Test LTI launch
    lti_data = lms_manager.create_lti_launch(
        "https://multios.edu/tool/launch",
        "consumer_key_123",
        {"user_id": "student_123", "full_name": "John Doe", "email": "john@example.com"}
    )
    print("\nLTI Launch Data:")
    print(json.dumps(lti_data, indent=2))