"""
API Compliance Testing Generator for MultiOS
Generates comprehensive API compliance tests including specification adherence,
error handling, and interface consistency
"""

import random
import json
import logging
from typing import Dict, List, Any, Optional, Tuple, Union
from dataclasses import dataclass
from enum import Enum
from urllib.parse import urlparse, parse_qs
import base64

class APIStandard(Enum):
    """API standards and protocols"""
    REST = "rest"
    GRAPHQL = "graphql"
    RPC = "rpc"
    SOAP = "soap"
    WEBSOCKET = "websocket"
    HTTP2 = "http2"
    OPENAPI = "openapi"

class ComplianceCheck(Enum):
    """Types of API compliance checks"""
    REQUEST_VALIDATION = "request_validation"
    RESPONSE_VALIDATION = "response_validation"
    HTTP_STATUS_CODES = "http_status_codes"
    CONTENT_TYPE_HANDLING = "content_type_handling"
    AUTHENTICATION = "authentication"
    AUTHORIZATION = "authorization"
    RATE_LIMITING = "rate_limiting"
    ERROR_HANDLING = "error_handling"
    API_VERSIONING = "api_versioning"
    DOCUMENTATION_COMPLIANCE = "documentation_compliance"

@dataclass
class APIEndpoint:
    """API endpoint definition"""
    path: str
    method: str
    required_headers: List[str]
    optional_headers: List[str]
    required_params: List[str]
    optional_params: List[str]
    request_body_schema: Optional[Dict[str, Any]]
    response_schema: Optional[Dict[str, Any]]
    status_codes: List[int]

class APIComplianceGenerator:
    """Generates API compliance test scenarios"""
    
    def __init__(self, seed: Optional[int] = None):
        self.logger = logging.getLogger(__name__)
        if seed:
            random.seed(seed)
        
        self.api_standards = self._initialize_api_standards()
        self.endpoint_templates = self._initialize_endpoint_templates()
        
    def _initialize_api_standards(self) -> Dict[str, Dict[str, Any]]:
        """Initialize API standards and specifications"""
        return {
            "rest": {
                "methods": ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"],
                "status_codes": {
                    200: "OK", 201: "Created", 204: "No Content",
                    400: "Bad Request", 401: "Unauthorized", 403: "Forbidden",
                    404: "Not Found", 409: "Conflict", 422: "Unprocessable Entity",
                    500: "Internal Server Error", 503: "Service Unavailable"
                },
                "content_types": ["application/json", "application/xml", "text/plain"],
                "required_headers": ["Content-Type", "Accept"],
                "naming_convention": "kebab-case"
            },
            "graphql": {
                "operations": ["query", "mutation", "subscription"],
                "status_codes": {200: "OK"},
                "content_types": ["application/json"],
                "required_headers": ["Content-Type"],
                "features": ["introspection", "variables", "fragments", "aliases"]
            },
            "rpc": {
                "methods": ["call", "notify"],
                "status_codes": {200: "OK", 400: "Bad Request", 500: "Internal Error"},
                "content_types": ["application/json", "application/octet-stream"],
                "protocols": ["json-rpc", "xml-rpc", "grpc"]
            }
        }
    
    def _initialize_endpoint_templates(self) -> Dict[str, APIEndpoint]:
        """Initialize endpoint templates"""
        return {
            "user_crud": APIEndpoint(
                path="/api/v1/users",
                method="GET",
                required_headers=["Authorization"],
                optional_headers=["Accept", "Content-Type", "X-Request-ID"],
                required_params=[],
                optional_params=["limit", "offset", "sort", "filter"],
                request_body_schema=None,
                response_schema={
                    "type": "object",
                    "properties": {
                        "users": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "id": {"type": "integer"},
                                    "username": {"type": "string"},
                                    "email": {"type": "string"},
                                    "created_at": {"type": "string", "format": "date-time"}
                                }
                            }
                        }
                    }
                },
                status_codes=[200, 400, 401, 500]
            ),
            "user_create": APIEndpoint(
                path="/api/v1/users",
                method="POST",
                required_headers=["Authorization", "Content-Type"],
                optional_headers=["Accept", "X-Request-ID"],
                required_params=[],
                optional_params=[],
                request_body_schema={
                    "type": "object",
                    "required": ["username", "email", "password"],
                    "properties": {
                        "username": {"type": "string", "minLength": 3, "maxLength": 50},
                        "email": {"type": "string", "format": "email"},
                        "password": {"type": "string", "minLength": 8},
                        "first_name": {"type": "string"},
                        "last_name": {"type": "string"}
                    }
                },
                response_schema={
                    "type": "object",
                    "properties": {
                        "id": {"type": "integer"},
                        "username": {"type": "string"},
                        "email": {"type": "string"},
                        "created_at": {"type": "string", "format": "date-time"}
                    }
                },
                status_codes=[201, 400, 401, 409, 422, 500]
            ),
            "file_upload": APIEndpoint(
                path="/api/v1/files",
                method="POST",
                required_headers=["Authorization", "Content-Type"],
                optional_headers=["Accept", "X-Upload-Type"],
                required_params=[],
                optional_params=[],
                request_body_schema={
                    "type": "object",
                    "required": ["file_name", "file_content"],
                    "properties": {
                        "file_name": {"type": "string"},
                        "file_content": {"type": "string"},
                        "file_size": {"type": "integer"},
                        "mime_type": {"type": "string"}
                    }
                },
                response_schema={
                    "type": "object",
                    "properties": {
                        "file_id": {"type": "string"},
                        "file_name": {"type": "string"},
                        "file_size": {"type": "integer"},
                        "uploaded_at": {"type": "string", "format": "date-time"}
                    }
                },
                status_codes=[201, 400, 401, 413, 500]
            )
        }
    
    async def generate_api_tests(self, component: str, iterations: int = 1000,
                               parameters: Optional[Dict[str, Any]] = None) -> List[Dict[str, Any]]:
        """Generate API compliance test suite"""
        test_cases = []
        parameters = parameters or {}
        
        # Generate different types of API tests
        test_cases.extend(await self._generate_request_validation_tests(component, iterations // 4, parameters))
        test_cases.extend(await self._generate_response_validation_tests(component, iterations // 4, parameters))
        test_cases.extend(await self._generate_authentication_tests(component, iterations // 4, parameters))
        test_cases.extend(await self._generate_error_handling_tests(component, iterations // 6, parameters))
        test_cases.extend(await self._generate_content_type_tests(component, iterations // 6, parameters))
        test_cases.extend(await self._generate_versioning_tests(component, iterations // 8, parameters))
        
        return test_cases
    
    async def _generate_request_validation_tests(self, component: str, count: int,
                                               parameters: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Generate request validation test cases"""
        test_cases = []
        
        validation_scenarios = [
            "missing_required_header",
            "invalid_header_format",
            "missing_required_parameter",
            "invalid_parameter_type",
            "parameter_value_out_of_range",
            "malformed_request_body",
            "invalid_json",
            "missing_content_type",
            "unsupported_content_type",
            "invalid_accept_header"
        ]
        
        for i in range(count):
            scenario = validation_scenarios[i % len(validation_scenarios)]
            endpoint = random.choice(list(self.endpoint_templates.values()))
            
            test_case = {
                "id": f"request_validation_{i}",
                "name": f"Request Validation - {scenario}",
                "type": "api_compliance",
                "description": f"Test {component} request validation: {scenario}",
                "component": component,
                "compliance_check": ComplianceCheck.REQUEST_VALIDATION.value,
                "input_data": {
                    "endpoint": endpoint,
                    "scenario": scenario,
                    "test_method": endpoint.method,
                    "test_path": endpoint.path,
                    "headers": self._generate_invalid_headers(endpoint, scenario),
                    "parameters": self._generate_invalid_parameters(endpoint, scenario),
                    "body": self._generate_invalid_body(endpoint, scenario)
                },
                "expected_behavior": "API should validate requests and return appropriate errors",
                "priority": 5,
                "category": "api_compliance",
                "test_steps": [
                    f"1. Create request with {scenario} violation",
                    "2. Send request to API endpoint",
                    "3. Verify validation error response",
                    "4. Check status code (400/422 expected)",
                    "5. Verify error message format"
                ],
                "assertions": [
                    "assert validation error returned",
                    "assert appropriate status code (400/422)",
                    "assert error message is descriptive",
                    "assert no sensitive data in error response",
                    "assert request is not processed"
                ],
                "validation_rules": {
                    "required_headers": endpoint.required_headers,
                    "required_parameters": endpoint.required_params,
                    "content_types": ["application/json"],
                    "request_body_schema": endpoint.request_body_schema
                }
            }
            
            test_cases.append(test_case)
        
        return test_cases
    
    async def _generate_response_validation_tests(self, component: str, count: int,
                                                parameters: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Generate response validation test cases"""
        test_cases = []
        
        response_scenarios = [
            "missing_required_field",
            "invalid_field_type",
            "unexpected_field",
            "invalid_enum_value",
            "missing_content_type_header",
            "incorrect_status_code",
            "invalid_json_structure",
            "truncated_response",
            "invalid_date_format",
            "negative_id_value"
        ]
        
        for i in range(count):
            scenario = response_scenarios[i % len(response_scenarios)]
            endpoint = random.choice(list(self.endpoint_templates.values()))
            
            test_case = {
                "id": f"response_validation_{i}",
                "name": f"Response Validation - {scenario}",
                "type": "api_compliance",
                "description": f"Test {component} response validation: {scenario}",
                "component": component,
                "compliance_check": ComplianceCheck.RESPONSE_VALIDATION.value,
                "input_data": {
                    "endpoint": endpoint,
                    "scenario": scenario,
                    "test_method": endpoint.method,
                    "test_path": endpoint.path,
                    "expected_status_codes": endpoint.status_codes,
                    "response_schema": endpoint.response_schema,
                    "mock_response": self._generate_mock_response(endpoint, scenario)
                },
                "expected_behavior": "API responses should be validated against schema",
                "priority": 4,
                "category": "api_compliance",
                "test_steps": [
                    f"1. Mock API response with {scenario}",
                    "2. Validate response against schema",
                    "3. Check for schema violations",
                    "4. Verify error handling",
                    "5. Ensure no data corruption"
                ],
                "assertions": [
                    "assert response matches schema",
                    "assert required fields present",
                    "assert field types correct",
                    "assert no unexpected fields (if strict)",
                    "assert status codes match expectations"
                ],
                "schema_validation": {
                    "required_fields": self._get_required_fields(endpoint.response_schema),
                    "field_types": self._get_field_types(endpoint.response_schema),
                    "strict_validation": True
                }
            }
            
            test_cases.append(test_case)
        
        return test_cases
    
    async def _generate_authentication_tests(self, component: str, count: int,
                                           parameters: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Generate authentication test cases"""
        test_cases = []
        
        auth_scenarios = [
            "missing_auth_header",
            "invalid_token_format",
            "expired_token",
            "insufficient_permissions",
            "wrong_credentials",
            "bearer_token_malformed",
            "basic_auth_invalid",
            "api_key_missing",
            "api_key_invalid",
            "token_replay_attack"
        ]
        
        for i in range(count):
            scenario = auth_scenarios[i % len(auth_scenarios)]
            auth_method = random.choice(["bearer", "basic", "api_key", "oauth2"])
            endpoint = random.choice(list(self.endpoint_templates.values()))
            
            test_case = {
                "id": f"authentication_{i}",
                "name": f"Authentication - {scenario}",
                "type": "api_compliance",
                "description": f"Test {component} authentication: {scenario}",
                "component": component,
                "compliance_check": ComplianceCheck.AUTHENTICATION.value,
                "input_data": {
                    "scenario": scenario,
                    "auth_method": auth_method,
                    "endpoint": endpoint,
                    "test_method": endpoint.method,
                    "test_path": endpoint.path,
                    "auth_header": self._generate_invalid_auth(auth_method, scenario),
                    "request_data": self._generate_valid_request_data(endpoint)
                },
                "expected_behavior": "API should properly authenticate and authorize requests",
                "priority": 5,
                "category": "api_compliance",
                "test_steps": [
                    f"1. Prepare {scenario} authentication",
                    "2. Send request to protected endpoint",
                    "3. Verify authentication failure",
                    "4. Check 401/403 status code",
                    "5. Verify no sensitive data leaked"
                ],
                "assertions": [
                    "assert authentication failure (401/403)",
                    "assert no sensitive data in response",
                    "assert appropriate error message",
                    "assert rate limiting still applies",
                    "assert no unauthorized access"
                ],
                "security_measures": {
                    "credential_hashing": True,
                    "token_expiration": True,
                    "rate_limiting": True,
                    "audit_logging": True
                }
            }
            
            test_cases.append(test_case)
        
        return test_cases
    
    async def _generate_error_handling_tests(self, component: str, count: int,
                                           parameters: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Generate error handling test cases"""
        test_cases = []
        
        error_scenarios = [
            "internal_server_error",
            "service_unavailable",
            "rate_limit_exceeded",
            "validation_error",
            "not_found_error",
            "conflict_error",
            "unauthorized_error",
            "forbidden_error",
            "payload_too_large",
            "unsupported_media_type"
        ]
        
        for i in range(count):
            scenario = error_scenarios[i % len(error_scenarios)]
            endpoint = random.choice(list(self.endpoint_templates.values()))
            
            test_case = {
                "id": f"error_handling_{i}",
                "name": f"Error Handling - {scenario}",
                "type": "api_compliance",
                "description": f"Test {component} error handling: {scenario}",
                "component": component,
                "compliance_check": ComplianceCheck.ERROR_HANDLING.value,
                "input_data": {
                    "scenario": scenario,
                    "endpoint": endpoint,
                    "test_method": endpoint.method,
                    "test_path": endpoint.path,
                    "trigger_conditions": self._get_error_trigger_conditions(scenario),
                    "expected_status_code": self._get_error_status_code(scenario),
                    "expected_error_structure": self._get_standard_error_structure()
                },
                "expected_behavior": "API should handle errors gracefully and consistently",
                "priority": 4,
                "category": "api_compliance",
                "test_steps": [
                    f"1. Trigger {scenario} error condition",
                    "2. Receive error response",
                    "3. Validate error format",
                    "4. Check error message appropriateness",
                    "5. Verify no internal details leaked"
                ],
                "assertions": [
                    "assert proper error status code",
                    "assert error message is user-friendly",
                    "assert no stack traces or internal errors",
                    "assert error structure is consistent",
                    "assert rate limiting still works"
                ],
                "error_standards": {
                    "http_status_codes": True,
                    "error_codes": True,
                    "error_messages": True,
                    "error_details": "minimal"
                }
            }
            
            test_cases.append(test_case)
        
        return test_cases
    
    async def _generate_content_type_tests(self, component: str, count: int,
                                         parameters: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Generate content type handling test cases"""
        test_cases = []
        
        content_scenarios = [
            "missing_content_type",
            "unsupported_content_type",
            "invalid_content_type",
            "charset_mismatch",
            "content_type_priority",
            "response_content_type_mismatch",
            "multipart_malformed",
            "base64_invalid",
            "encoding_error",
            "character_set_error"
        ]
        
        for i in range(count):
            scenario = content_scenarios[i % len(content_scenarios)]
            content_type = random.choice(["application/json", "application/xml", "text/plain"])
            endpoint = random.choice(list(self.endpoint_templates.values()))
            
            test_case = {
                "id": f"content_type_{i}",
                "name": f"Content Type - {scenario}",
                "type": "api_compliance",
                "description": f"Test {component} content type handling: {scenario}",
                "component": component,
                "compliance_check": ComplianceCheck.CONTENT_TYPE_HANDLING.value,
                "input_data": {
                    "scenario": scenario,
                    "endpoint": endpoint,
                    "content_type": content_type,
                    "test_method": endpoint.method,
                    "test_path": endpoint.path,
                    "request_body": self._generate_content_body(content_type),
                    "headers": self._generate_content_headers(content_type, scenario)
                },
                "expected_behavior": "API should handle content types correctly",
                "priority": 3,
                "category": "api_compliance",
                "test_steps": [
                    f"1. Prepare request with {scenario}",
                    "2. Set appropriate content type header",
                    "3. Send request with malformed/incorrect content",
                    "4. Verify content type handling",
                    "5. Check error response"
                ],
                "assertions": [
                    "assert content type validation works",
                    "assert appropriate error for invalid content",
                    "assert character encoding handled correctly",
                    "assert binary content supported",
                    "assert content negotiation works"
                ],
                "content_handling": {
                    "supported_types": ["application/json", "application/xml", "text/plain"],
                    "charset_support": ["UTF-8", "ASCII"],
                    "encoding_support": ["base64", "gzip"],
                    "size_limits": 10485760  # 10MB
                }
            }
            
            test_cases.append(test_case)
        
        return test_cases
    
    async def _generate_versioning_tests(self, component: str, count: int,
                                       parameters: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Generate API versioning test cases"""
        test_cases = []
        
        versioning_scenarios = [
            "version_header_missing",
            "invalid_version_format",
            "unsupported_version",
            "version_deprecation",
            "version_compatibility",
            "version_negotiation",
            "version_specific_behavior",
            "version_migration",
            "version_sunset",
            "backwards_compatibility"
        ]
        
        for i in range(count):
            scenario = versioning_scenarios[i % len(versioning_scenarios)]
            version_strategy = random.choice(["header", "path", "query_param"])
            endpoint = random.choice(list(self.endpoint_templates.values()))
            
            test_case = {
                "id": f"versioning_{i}",
                "name": f"API Versioning - {scenario}",
                "type": "api_compliance",
                "description": f"Test {component} API versioning: {scenario}",
                "component": component,
                "compliance_check": ComplianceCheck.API_VERSIONING.value,
                "input_data": {
                    "scenario": scenario,
                    "version_strategy": version_strategy,
                    "endpoint": endpoint,
                    "test_method": endpoint.method,
                    "test_path": endpoint.path,
                    "version": random.choice(["v1", "v2", "v1.1", "2023-01", "latest"]),
                    "version_headers": self._generate_version_headers(version_strategy, scenario)
                },
                "expected_behavior": "API versioning should work correctly across versions",
                "priority": 3,
                "category": "api_compliance",
                "test_steps": [
                    f"1. Test {scenario} versioning scenario",
                    "2. Send request with version specification",
                    "3. Verify version-specific behavior",
                    "4. Check version compatibility",
                    "5. Validate version negotiation"
                ],
                "assertions": [
                    "assert version is correctly parsed",
                    "assert version-specific logic works",
                    "assert backwards compatibility maintained",
                    "assert appropriate version warnings",
                    "assert version lifecycle respected"
                ],
                "versioning_policy": {
                    "current_version": "v1",
                    "supported_versions": ["v1", "v2"],
                    "deprecated_versions": ["v0"],
                    "sunset_policy": "6_months_notice"
                }
            }
            
            test_cases.append(test_case)
        
        return test_cases
    
    # Helper methods for generating test data
    def _generate_invalid_headers(self, endpoint: APIEndpoint, scenario: str) -> Dict[str, str]:
        """Generate invalid headers for testing"""
        invalid_headers = {
            "missing_required_header": {
                # Missing required header
            },
            "invalid_header_format": {
                "Authorization": "InvalidFormat",
                "Content-Type": "invalid/type"
            },
            "missing_content_type": {
                "Authorization": "Bearer valid_token",
                # No Content-Type
            },
            "unsupported_content_type": {
                "Content-Type": "application/unknown"
            }
        }
        
        return invalid_headers.get(scenario, {"Invalid-Header": "invalid_value"})
    
    def _generate_invalid_parameters(self, endpoint: APIEndpoint, scenario: str) -> Dict[str, Any]:
        """Generate invalid parameters for testing"""
        invalid_params = {
            "missing_required_parameter": {},
            "invalid_parameter_type": {
                "limit": "not_a_number",
                "offset": "invalid"
            },
            "parameter_value_out_of_range": {
                "limit": 1000000,
                "offset": -100
            }
        }
        
        return invalid_params.get(scenario, {"invalid_param": "invalid_value"})
    
    def _generate_invalid_body(self, endpoint: APIEndpoint, scenario: str) -> Any:
        """Generate invalid request body for testing"""
        invalid_bodies = {
            "malformed_request_body": "{invalid json",
            "invalid_json": {"valid": "json", "extra": "brackets}}",
            "missing_required_field": {
                "username": "testuser"
                # Missing required email and password
            },
            "invalid_field_type": {
                "username": 123,  # Should be string
                "email": "valid@email.com",
                "password": "validpassword"
            }
        }
        
        return invalid_bodies.get(scenario, {"invalid": "body"})
    
    def _generate_mock_response(self, endpoint: APIEndpoint, scenario: str) -> Dict[str, Any]:
        """Generate mock response for testing"""
        if endpoint.response_schema:
            response = {"valid": "response"}
            # Add violations based on scenario
            if scenario == "missing_required_field":
                response.pop("id", None)  # Remove required field
            elif scenario == "invalid_field_type":
                response["id"] = "should_be_integer"  # Wrong type
            elif scenario == "unexpected_field":
                response["unexpected_field"] = "unexpected_value"
            return response
        return {"mock": "response"}
    
    def _generate_invalid_auth(self, auth_method: str, scenario: str) -> str:
        """Generate invalid authentication for testing"""
        auth_methods = {
            "bearer": {
                "missing_auth_header": None,
                "invalid_token_format": "InvalidFormat123",
                "expired_token": "Bearer expired_token_123",
                "bearer_token_malformed": "Bearer malformed token"
            },
            "basic": {
                "basic_auth_invalid": "Basic " + base64.b64encode(b"invalid:credentials").decode()
            }
        }
        
        method_auth = auth_methods.get(auth_method, {})
        return method_auth.get(scenario, "InvalidAuth")
    
    def _generate_valid_request_data(self, endpoint: APIEndpoint) -> Dict[str, Any]:
        """Generate valid request data for testing"""
        if endpoint.path == "/api/v1/users":
            if endpoint.method == "GET":
                return {"limit": 10, "offset": 0}
            elif endpoint.method == "POST":
                return {
                    "username": "testuser",
                    "email": "test@example.com",
                    "password": "testpassword123"
                }
        elif endpoint.path == "/api/v1/files":
            return {
                "file_name": "test.txt",
                "file_content": "test content",
                "file_size": 11,
                "mime_type": "text/plain"
            }
        
        return {}
    
    def _get_error_trigger_conditions(self, scenario: str) -> Dict[str, Any]:
        """Get conditions that trigger different error scenarios"""
        conditions = {
            "internal_server_error": {"simulate": "internal_error"},
            "service_unavailable": {"simulate": "database_down"},
            "rate_limit_exceeded": {"send_requests": 1000},
            "validation_error": {"send_invalid_data": True},
            "not_found_error": {"resource_id": "nonexistent"},
            "conflict_error": {"resource_exists": True},
            "unauthorized_error": {"no_auth": True},
            "forbidden_error": {"insufficient_permissions": True},
            "payload_too_large": {"large_payload": True},
            "unsupported_media_type": {"wrong_content_type": True}
        }
        
        return conditions.get(scenario, {})
    
    def _get_error_status_code(self, scenario: str) -> int:
        """Get expected status code for error scenario"""
        status_codes = {
            "internal_server_error": 500,
            "service_unavailable": 503,
            "rate_limit_exceeded": 429,
            "validation_error": 422,
            "not_found_error": 404,
            "conflict_error": 409,
            "unauthorized_error": 401,
            "forbidden_error": 403,
            "payload_too_large": 413,
            "unsupported_media_type": 415
        }
        
        return status_codes.get(scenario, 500)
    
    def _get_standard_error_structure(self) -> Dict[str, Any]:
        """Get standard error response structure"""
        return {
            "type": "object",
            "required": ["error", "message", "code"],
            "properties": {
                "error": {"type": "string"},
                "message": {"type": "string"},
                "code": {"type": "integer"},
                "details": {"type": "object"}
            }
        }
    
    def _generate_content_body(self, content_type: str) -> str:
        """Generate content body for testing"""
        if content_type == "application/json":
            return '{"test": "content"}'
        elif content_type == "application/xml":
            return '<?xml version="1.0"?><test>content</test>'
        elif content_type == "text/plain":
            return "test content"
        return "test content"
    
    def _generate_content_headers(self, content_type: str, scenario: str) -> Dict[str, str]:
        """Generate content type headers for testing"""
        headers = {"Content-Type": content_type}
        
        if scenario == "missing_content_type":
            headers.pop("Content-Type", None)
        elif scenario == "unsupported_content_type":
            headers["Content-Type"] = "application/unsupported"
        elif scenario == "charset_mismatch":
            headers["Content-Type"] = f"{content_type}; charset=invalid"
        
        return headers
    
    def _generate_version_headers(self, strategy: str, scenario: str) -> Dict[str, str]:
        """Generate version headers for testing"""
        headers = {}
        
        if strategy == "header":
            if scenario == "version_header_missing":
                # No version header
                pass
            elif scenario == "invalid_version_format":
                headers["API-Version"] = "invalid_format"
            elif scenario == "unsupported_version":
                headers["API-Version"] = "v999"
        
        return headers
    
    def _get_required_fields(self, schema: Optional[Dict[str, Any]]) -> List[str]:
        """Extract required fields from schema"""
        if not schema:
            return []
        
        if "properties" in schema and "required" in schema:
            return schema["required"]
        
        return []
    
    def _get_field_types(self, schema: Optional[Dict[str, Any]]) -> Dict[str, str]:
        """Extract field types from schema"""
        if not schema or "properties" not in schema:
            return {}
        
        return {field: prop.get("type", "unknown") 
                for field, prop in schema["properties"].items()}