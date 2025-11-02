"""
Security Scanner Module
======================

Provides comprehensive security scanning for educational packages including
malware detection, signature verification, and integrity checking.
"""

import os
import json
import hashlib
import logging
import subprocess
import tempfile
import shutil
from pathlib import Path
from typing import Dict, List, Optional, Tuple, Any
from dataclasses import dataclass
from datetime import datetime
import re

from package_manager import PackageMetadata

logger = logging.getLogger(__name__)


@dataclass
class SecurityIssue:
    """Represents a security issue found in a package"""
    severity: str  # low, medium, high, critical
    type: str      # malware, unsigned, tampered, vulnerable, suspicious
    description: str
    file_path: Optional[str] = None
    line_number: Optional[int] = None
    recommendation: Optional[str] = None


@dataclass
class ScanResult:
    """Results of a security scan"""
    package_name: str
    package_version: str
    scan_timestamp: str
    passed: bool
    issues: List[SecurityIssue]
    signatures: Dict[str, str]
    checksums: Dict[str, str]
    scan_duration: float
    recommendations: List[str]


class SecurityScanner:
    """Comprehensive security scanner for educational packages"""
    
    def __init__(self, package_manager):
        self.pm = package_manager
        self.virus_total_api_key = os.environ.get('VIRUSTOTAL_API_KEY')
        self.signature_keys_dir = Path("/workspace/community/package_manager/security/keys")
        self.quarantine_dir = Path("/workspace/community/package_manager/security/quarantine")
        
        # Ensure directories exist
        self.signature_keys_dir.mkdir(parents=True, exist_ok=True)
        self.quarantine_dir.mkdir(parents=True, exist_ok=True)
        
        # Known suspicious file patterns
        self.suspicious_patterns = [
            r'eval\s*\(',
            r'exec\s*\(',
            r'subprocess\.call',
            r'os\.system',
            r'shell\s*=\s*',
            r'wget\s+',
            r'curl\s+',
            r'base64\.decode',
            r'__import__\s*\(',
            r'getattr\s*\(',
            r'setattr\s*\(',
            r'delattr\s*\(',
            r'pickle\.load',
            r'marshal\.loads',
        ]
        
        # Dangerous file extensions
        self.dangerous_extensions = ['.exe', '.bat', '.cmd', '.scr', '.pif', '.com']
        
    def scan_package(self, package_path: str, metadata: PackageMetadata) -> bool:
        """Perform comprehensive security scan of a package"""
        logger.info(f"Starting security scan for {metadata.name}")
        scan_start = datetime.now()
        
        try:
            issues = []
            
            # 1. Signature verification
            signature_issues = self._verify_signatures(package_path, metadata)
            issues.extend(signature_issues)
            
            # 2. Checksum verification
            checksum_issues = self._verify_checksums(package_path, metadata)
            issues.extend(checksum_issues)
            
            # 3. Content scanning
            content_issues = self._scan_package_content(package_path, metadata)
            issues.extend(content_issues)
            
            # 4. Vulnerability scanning
            vuln_issues = self._scan_for_vulnerabilities(package_path, metadata)
            issues.extend(vuln_issues)
            
            # 5. File integrity check
            integrity_issues = self._check_file_integrity(package_path)
            issues.extend(integrity_issues)
            
            # 6. Malware detection (if available)
            if self.virus_total_api_key:
                malware_issues = self._scan_for_malware(package_path)
                issues.extend(malware_issues)
            
            # Determine if scan passed
            high_or_critical_issues = [i for i in issues if i.severity in ['high', 'critical']]
            scan_passed = len(high_or_critical_issues) == 0
            
            scan_duration = (datetime.now() - scan_start).total_seconds()
            
            # Log results
            if scan_passed:
                logger.info(f"Security scan passed for {metadata.name}")
            else:
                logger.warning(f"Security scan failed for {metadata.name}: {len(issues)} issues found")
                for issue in issues:
                    logger.warning(f"  {issue.severity.upper()}: {issue.description}")
            
            # Store scan results
            self._store_scan_results(metadata, issues, scan_passed, scan_duration)
            
            # Quarantine if critical issues found
            if any(issue.severity == 'critical' for issue in issues):
                self._quarantine_package(package_path, metadata, issues)
            
            return scan_passed
            
        except Exception as e:
            logger.error(f"Error during security scan: {e}")
            return False
    
    def verify_package(self, package_path: str, metadata: PackageMetadata) -> bool:
        """Verify package integrity and authenticity"""
        logger.info(f"Verifying package: {metadata.name}")
        
        try:
            # Check file signature
            if not self._verify_package_signature(package_path, metadata):
                logger.error("Package signature verification failed")
                return False
            
            # Verify checksums
            if not self._verify_package_checksums(package_path, metadata):
                logger.error("Package checksum verification failed")
                return False
            
            # Check package integrity
            if not self._check_package_structure(package_path, metadata):
                logger.error("Package structure integrity check failed")
                return False
            
            # Verify metadata integrity
            if not self._verify_metadata_integrity(package_path):
                logger.error("Metadata integrity verification failed")
                return False
            
            logger.info(f"Package verification successful: {metadata.name}")
            return True
            
        except Exception as e:
            logger.error(f"Error during package verification: {e}")
            return False
    
    def scan_file(self, file_path: str) -> List[SecurityIssue]:
        """Scan a single file for security issues"""
        issues = []
        
        try:
            with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
                
            # Check for suspicious patterns
            for line_num, line in enumerate(content.split('\n'), 1):
                for pattern in self.suspicious_patterns:
                    if re.search(pattern, line, re.IGNORECASE):
                        issues.append(SecurityIssue(
                            severity='medium',
                            type='suspicious',
                            description=f"Suspicious pattern found: {pattern}",
                            file_path=file_path,
                            line_number=line_num,
                            recommendation='Review the code for potential security issues'
                        ))
            
            # Check file extensions
            if any(file_path.endswith(ext) for ext in self.dangerous_extensions):
                issues.append(SecurityIssue(
                    severity='high',
                    type='dangerous_file',
                    description=f"Dangerous file extension: {file_path}",
                    file_path=file_path,
                    recommendation='Remove or replace this file'
                ))
            
        except Exception as e:
            issues.append(SecurityIssue(
                severity='low',
                type='scan_error',
                description=f"Error scanning file {file_path}: {e}",
                file_path=file_path
            ))
        
        return issues
    
    def generate_security_report(self, package_name: str) -> Optional[ScanResult]:
        """Generate detailed security report for a package"""
        results_file = self._get_scan_results_file(package_name)
        
        if not results_file.exists():
            return None
        
        try:
            with open(results_file, 'r') as f:
                data = json.load(f)
                
            # Convert to ScanResult object
            issues = [SecurityIssue(**issue) for issue in data['issues']]
            
            return ScanResult(
                package_name=data['package_name'],
                package_version=data['package_version'],
                scan_timestamp=data['scan_timestamp'],
                passed=data['passed'],
                issues=issues,
                signatures=data.get('signatures', {}),
                checksums=data.get('checksums', {}),
                scan_duration=data.get('scan_duration', 0.0),
                recommendations=data.get('recommendations', [])
            )
            
        except Exception as e:
            logger.error(f"Error generating security report: {e}")
            return None
    
    def _verify_signatures(self, package_path: str, metadata: PackageMetadata) -> List[SecurityIssue]:
        """Verify package digital signatures"""
        issues = []
        
        # Check if signature exists
        signature_file = f"{package_path}.sig"
        if not os.path.exists(signature_file):
            if self.pm.config["security"]["require_signature"]:
                issues.append(SecurityIssue(
                    severity='high',
                    type='unsigned',
                    description='Package is not signed',
                    recommendation='Package must be signed before distribution'
                ))
            else:
                logger.warning('Package is not signed (signatures not required)')
                return issues
        
        try:
            # TODO: Implement actual signature verification
            # This would require GPG or similar cryptographic verification
            logger.info('Signature verification not implemented yet')
            
        except Exception as e:
            issues.append(SecurityIssue(
                severity='high',
                type='signature_error',
                description=f'Signature verification failed: {e}'
            ))
        
        return issues
    
    def _verify_checksums(self, package_path: str, metadata: PackageMetadata) -> List[SecurityIssue]:
        """Verify package checksums"""
        issues = []
        
        # Recalculate package checksum
        current_checksum = self._calculate_file_checksum(package_path)
        
        if current_checksum != metadata.checksum:
            issues.append(SecurityIssue(
                severity='critical',
                type='checksum_mismatch',
                description='Package checksum does not match metadata',
                recommendation='Package may be corrupted or tampered with'
            ))
        
        return issues
    
    def _scan_package_content(self, package_path: str, metadata: PackageMetadata) -> List[SecurityIssue]:
        """Scan package contents for security issues"""
        issues = []
        
        with tempfile.TemporaryDirectory() as temp_dir:
            try:
                # Extract package
                shutil.unpack_archive(package_path, temp_dir)
                
                # Scan all files
                temp_path = Path(temp_dir)
                for file_path in temp_path.rglob("*"):
                    if file_path.is_file():
                        relative_path = str(file_path.relative_to(temp_path))
                        file_issues = self.scan_file(str(file_path))
                        
                        # Add relative path to issues
                        for issue in file_issues:
                            if issue.file_path:
                                issue.file_path = relative_path
                        
                        issues.extend(file_issues)
                        
            except Exception as e:
                issues.append(SecurityIssue(
                    severity='medium',
                    type='extraction_error',
                    description=f'Error extracting package for content scan: {e}'
                ))
        
        return issues
    
    def _scan_for_vulnerabilities(self, package_path: str, metadata: PackageMetadata) -> List[SecurityIssue]:
        """Scan for known vulnerabilities in package"""
        issues = []
        
        try:
            # Check against vulnerability database
            # This would integrate with services like CVE, NVD, etc.
            
            # For now, check against known vulnerable patterns
            vulnerable_patterns = [
                r'pickle\.loads?\s*\(\s*raw_input',
                r'eval\s*\(\s*input',
                r'exec\s*\(\s*input',
                r'__import__\s*\(\s*raw_input',
            ]
            
            with tempfile.TemporaryDirectory() as temp_dir:
                shutil.unpack_archive(package_path, temp_dir)
                temp_path = Path(temp_dir)
                
                for file_path in temp_path.rglob("*.py"):
                    try:
                        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                            content = f.read()
                            
                        for pattern in vulnerable_patterns:
                            matches = re.finditer(pattern, content, re.IGNORECASE)
                            for match in matches:
                                issues.append(SecurityIssue(
                                    severity='high',
                                    type='vulnerability',
                                    description=f'Potential vulnerability: {pattern}',
                                    file_path=str(file_path.relative_to(temp_path)),
                                    recommendation='Review and update the code to eliminate security vulnerabilities'
                                ))
                                
                    except Exception as e:
                        logger.warning(f"Error scanning {file_path}: {e}")
                        
        except Exception as e:
            issues.append(SecurityIssue(
                severity='low',
                type='vulnerability_scan_error',
                description=f'Error during vulnerability scan: {e}'
            ))
        
        return issues
    
    def _check_file_integrity(self, package_path: str) -> List[SecurityIssue]:
        """Check file integrity within package"""
        issues = []
        
        # Check for suspicious file names
        suspicious_names = ['hack', 'exploit', 'virus', 'malware', 'backdoor']
        
        with tempfile.TemporaryDirectory() as temp_dir:
            try:
                shutil.unpack_archive(package_path, temp_dir)
                temp_path = Path(temp_dir)
                
                for file_path in temp_path.rglob("*"):
                    if file_path.is_file():
                        file_name = file_path.name.lower()
                        for suspicious in suspicious_names:
                            if suspicious in file_name:
                                issues.append(SecurityIssue(
                                    severity='medium',
                                    type='suspicious_filename',
                                    description=f'Suspicious file name: {file_name}',
                                    file_path=str(file_path.relative_to(temp_path)),
                                    recommendation='Review file content and purpose'
                                ))
                                
            except Exception as e:
                issues.append(SecurityIssue(
                    severity='medium',
                    type='integrity_check_error',
                    description=f'Error checking file integrity: {e}'
                ))
        
        return issues
    
    def _scan_for_malware(self, package_path: str) -> List[SecurityIssue]:
        """Scan package for malware using VirusTotal API"""
        issues = []
        
        if not self.virus_total_api_key:
            logger.warning('VirusTotal API key not configured, skipping malware scan')
            return issues
        
        try:
            # This would integrate with VirusTotal API
            # For demonstration purposes, we'll skip the actual implementation
            logger.info('Malware scanning not implemented yet')
            
        except Exception as e:
            issues.append(SecurityIssue(
                severity='medium',
                type='malware_scan_error',
                description=f'Error during malware scan: {e}'
            ))
        
        return issues
    
    def _verify_package_signature(self, package_path: str, metadata: PackageMetadata) -> bool:
        """Verify package digital signature"""
        # TODO: Implement actual signature verification
        # This would use GPG or similar cryptographic verification
        return True
    
    def _verify_package_checksums(self, package_path: str, metadata: PackageMetadata) -> bool:
        """Verify package checksums"""
        current_checksum = self._calculate_file_checksum(package_path)
        return current_checksum == metadata.checksum
    
    def _check_package_structure(self, package_path: str, metadata: PackageMetadata) -> bool:
        """Check package structure integrity"""
        try:
            with tempfile.TemporaryDirectory() as temp_dir:
                shutil.unpack_archive(package_path, temp_dir)
                temp_path = Path(temp_dir)
                
                # Check for metadata.json
                metadata_file = temp_path / "metadata.json"
                if not metadata_file.exists():
                    return False
                
                # Validate metadata
                with open(metadata_file, 'r') as f:
                    package_metadata = json.load(f)
                
                # Check required fields
                required_fields = ['name', 'version', 'description', 'author', 'type']
                for field in required_fields:
                    if field not in package_metadata:
                        return False
                
                return True
                
        except Exception:
            return False
    
    def _verify_metadata_integrity(self, package_path: str) -> bool:
        """Verify metadata integrity"""
        # TODO: Implement metadata integrity verification
        return True
    
    def _calculate_file_checksum(self, file_path: str) -> str:
        """Calculate SHA256 checksum of file"""
        sha256_hash = hashlib.sha256()
        with open(file_path, "rb") as f:
            for chunk in iter(lambda: f.read(4096), b""):
                sha256_hash.update(chunk)
        return sha256_hash.hexdigest()
    
    def _store_scan_results(self, metadata: PackageMetadata, issues: List[SecurityIssue], 
                           passed: bool, scan_duration: float):
        """Store scan results to file"""
        results_file = self._get_scan_results_file(metadata.name)
        
        results = {
            'package_name': metadata.name,
            'package_version': metadata.version,
            'scan_timestamp': datetime.now().isoformat(),
            'passed': passed,
            'issues': [self._security_issue_to_dict(issue) for issue in issues],
            'scan_duration': scan_duration,
            'recommendations': self._generate_recommendations(issues)
        }
        
        with open(results_file, 'w') as f:
            json.dump(results, f, indent=2)
    
    def _get_scan_results_file(self, package_name: str) -> Path:
        """Get path to scan results file"""
        return self.pm.metadata_dir / f"{package_name}_security.json"
    
    def _security_issue_to_dict(self, issue: SecurityIssue) -> Dict[str, Any]:
        """Convert SecurityIssue to dictionary"""
        return {
            'severity': issue.severity,
            'type': issue.type,
            'description': issue.description,
            'file_path': issue.file_path,
            'line_number': issue.line_number,
            'recommendation': issue.recommendation
        }
    
    def _generate_recommendations(self, issues: List[SecurityIssue]) -> List[str]:
        """Generate recommendations based on security issues"""
        recommendations = []
        
        # Count issues by severity
        critical_count = len([i for i in issues if i.severity == 'critical'])
        high_count = len([i for i in issues if i.severity == 'high'])
        medium_count = len([i for i in issues if i.severity == 'medium'])
        
        if critical_count > 0:
            recommendations.append("Critical security issues found. Package should not be installed until resolved.")
        
        if high_count > 0:
            recommendations.append("High severity issues found. Review and resolve before installation.")
        
        if medium_count > 0:
            recommendations.append("Medium severity issues found. Consider resolving before installation.")
        
        # Specific recommendations based on issue types
        issue_types = [issue.type for issue in issues]
        
        if 'unsigned' in issue_types:
            recommendations.append("Package should be signed by a trusted developer.")
        
        if 'vulnerability' in issue_types:
            recommendations.append("Review code for security vulnerabilities and update dependencies.")
        
        if 'malware' in issue_types:
            recommendations.append("Malware detected. Package should be thoroughly reviewed.")
        
        return recommendations
    
    def _quarantine_package(self, package_path: str, metadata: PackageMetadata, 
                           issues: List[SecurityIssue]):
        """Quarantine a package with critical security issues"""
        quarantine_path = self.quarantine_dir / f"{metadata.name}_{metadata.version}_{datetime.now().strftime('%Y%m%d_%H%M%S')}"
        
        try:
            shutil.move(package_path, quarantine_path)
            logger.error(f"Package quarantined due to critical issues: {quarantine_path}")
            
            # Store quarantine information
            quarantine_info = {
                'package_name': metadata.name,
                'package_version': metadata.version,
                'quarantine_timestamp': datetime.now().isoformat(),
                'issues': [self._security_issue_to_dict(issue) for issue in issues],
                'quarantine_path': str(quarantine_path)
            }
            
            with open(f"{quarantine_path}.json", 'w') as f:
                json.dump(quarantine_info, f, indent=2)
                
        except Exception as e:
            logger.error(f"Error quarantining package: {e}")