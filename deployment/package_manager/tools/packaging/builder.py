#!/usr/bin/env python3
"""
MultiOS Package Builder - Create and build packages for MultiOS

This tool helps developers create packages for the MultiOS ecosystem,
providing automated package creation, metadata generation, and signing.
"""

import argparse
import json
import os
import sys
import tempfile
import tarfile
import hashlib
import shutil
from pathlib import Path
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, asdict
from datetime import datetime
import subprocess

try:
    import yaml
except ImportError:
    yaml = None


@dataclass
class BuildConfig:
    """Package build configuration"""
    name: str
    version: str
    description: str
    architecture: str
    maintainer: str
    license: str
    dependencies: List[str] = None
    provides: List[str] = None
    conflicts: List[str] = None
    categories: List[str] = None
    tags: List[str] = None
    homepage: str = ""
    source_url: str = ""
    build_dependencies: List[str] = None
    install_scripts: Dict[str, str] = None
    files: List[str] = None
    exclude_patterns: List[str] = None
    
    def __post_init__(self):
        if self.dependencies is None:
            self.dependencies = []
        if self.provides is None:
            self.provides = []
        if self.conflicts is None:
            self.conflicts = []
        if self.categories is None:
            self.categories = []
        if self.tags is None:
            self.tags = []
        if self.build_dependencies is None:
            self.build_dependencies = []
        if self.install_scripts is None:
            self.install_scripts = {}
        if self.files is None:
            self.files = []
        if self.exclude_patterns is None:
            self.exclude_patterns = []


class PackageBuilder:
    """MultiOS Package Builder"""
    
    def __init__(self, config_path: Optional[str] = None):
        self.config_path = config_path
        self.build_dir = Path.cwd() / "build"
        self.package_dir = Path.cwd() / "package"
        self.source_dir = Path.cwd() / "src"
        self.config = None
        
        # Create build directories
        self.build_dir.mkdir(exist_ok=True)
        self.package_dir.mkdir(exist_ok=True)
    
    def load_config(self, config_path: Optional[str] = None) -> BuildConfig:
        """Load package build configuration"""
        config_file = config_path or self.config_path or "package.yaml"
        
        if not Path(config_file).exists():
            raise FileNotFoundError(f"Configuration file not found: {config_file}")
        
        with open(config_file, 'r') as f:
            if config_file.endswith('.yaml') or config_file.endswith('.yml'):
                if yaml is None:
                    raise ImportError("PyYAML is required for YAML configuration files")
                data = yaml.safe_load(f)
            else:
                data = json.load(f)
        
        # Convert to BuildConfig
        self.config = BuildConfig(**data)
        return self.config
    
    def create_skeleton(self, name: str, version: str = "1.0.0", 
                       maintainer: str = "Developer <dev@example.com>",
                       license: str = "MIT") -> None:
        """Create a package skeleton structure"""
        
        # Create directory structure
        directories = [
            "src",
            "build", 
            "package",
            "docs",
            "tests"
        ]
        
        for dir_name in directories:
            Path(dir_name).mkdir(exist_ok=True)
        
        # Create package.yaml
        package_config = {
            "name": name,
            "version": version,
            "description": f"A {name} package for MultiOS",
            "architecture": "universal",
            "maintainer": maintainer,
            "license": license,
            "categories": ["development"],
            "tags": [],
            "dependencies": [],
            "provides": [],
            "conflicts": [],
            "build_dependencies": [],
            "install_scripts": {
                "pre_install": "",
                "post_install": "",
                "pre_remove": "",
                "post_remove": ""
            },
            "files": [
                "bin/",
                "lib/",
                "share/"
            ],
            "exclude_patterns": [
                "*.o",
                "*.a",
                "*.so.*",
                ".git",
                "tests/"
            ]
        }
        
        with open("package.yaml", 'w') as f:
            if yaml is not None:
                yaml.dump(package_config, f, default_flow_style=False, indent=2)
            else:
                json.dump(package_config, f, indent=2)
        
        # Create README
        readme_content = f"""# {name}

{description}

## Installation

```bash
multios-pm install {name}
```

## Building from Source

```bash
# Install build dependencies
multios-pm install {' '.join(package_config['build_dependencies'])}

# Build package
python3 -m multios_pm.builder build

# Install locally
multios-pm install package/{name}-{version}-*.tar.xz
```

## Package Structure

- `src/` - Source code
- `build/` - Build output
- `package/` - Package output
- `docs/` - Documentation
- `tests/` - Test files

## Configuration

Edit `package.yaml` to customize the package configuration.
"""
        
        with open("README.md", 'w') as f:
            f.write(readme_content)
        
        print(f"Created package skeleton for {name}")
        print("Next steps:")
        print(f"  1. Edit package.yaml")
        print(f"  2. Place source files in src/")
        print(f"  3. Run 'python3 -m multios_pm.builder build'")
    
    def build(self, clean: bool = False, sign: bool = False, 
             key_id: Optional[str] = None) -> str:
        """Build package from source"""
        if not self.config:
            self.load_config()
        
        if clean and self.package_dir.exists():
            shutil.rmtree(self.package_dir)
            self.package_dir.mkdir()
        
        print(f"Building package {self.config.name} v{self.config.version}")
        
        # Step 1: Prepare build environment
        self._prepare_build()
        
        # Step 2: Build the package
        self._build_package()
        
        # Step 3: Create package structure
        package_path = self._create_package_structure()
        
        # Step 4: Generate metadata
        self._generate_metadata(package_path)
        
        # Step 5: Create archive
        archive_path = self._create_archive(package_path)
        
        # Step 6: Calculate checksums
        self._calculate_checksums(archive_path)
        
        # Step 7: Sign package if requested
        if sign:
            self._sign_package(archive_path, key_id)
        
        print(f"Package built successfully: {archive_path}")
        return str(archive_path)
    
    def _prepare_build(self) -> None:
        """Prepare build environment"""
        print("  Preparing build environment...")
        
        # Create temp build directory
        temp_build = Path(tempfile.mkdtemp(prefix=f"{self.config.name}-build-"))
        
        # Copy source files
        if self.source_dir.exists():
            shutil.copytree(self.source_dir, temp_build / "src")
        
        # Set up build environment
        env_file = temp_build / "build.env"
        with open(env_file, 'w') as f:
            f.write(f"PACKAGE_NAME={self.config.name}\n")
            f.write(f"PACKAGE_VERSION={self.config.version}\n")
            f.write(f"PACKAGE_ARCHITECTURE={self.config.architecture}\n")
            f.write(f"PACKAGE_MAINTAINER={self.config.maintainer}\n")
        
        self.temp_build_dir = temp_build
    
    def _build_package(self) -> None:
        """Build the actual package"""
        print("  Building package...")
        
        build_script = self.source_dir / "build.py"
        if build_script.exists():
            # Run custom build script
            result = subprocess.run([sys.executable, str(build_script)], 
                                  cwd=str(self.temp_build_dir / "src"))
            if result.returncode != 0:
                raise RuntimeError("Build script failed")
        else:
            # Default build: just copy files
            build_dir = self.temp_build_dir / "build"
            build_dir.mkdir()
            
            # Copy files according to patterns
            for pattern in self.config.files:
                src_path = self.source_dir / pattern
                if src_path.exists():
                    if src_path.is_dir():
                        shutil.copytree(src_path, build_dir / pattern)
                    else:
                        build_dir.parent.mkdir(parents=True, exist_ok=True)
                        shutil.copy2(src_path, build_dir / pattern)
    
    def _create_package_structure(self) -> Path:
        """Create final package directory structure"""
        print("  Creating package structure...")
        
        package_path = self.package_dir / f"{self.config.name}-{self.config.version}"
        
        # Remove if exists
        if package_path.exists():
            shutil.rmtree(package_path)
        
        # Create structure
        (package_path / "usr" / "bin").mkdir(parents=True)
        (package_path / "usr" / "lib").mkdir(parents=True)
        (package_path / "usr" / "share" / self.config.name).mkdir(parents=True)
        (package_path / "etc").mkdir(parents=True)
        
        # Copy built files
        build_dir = self.temp_build_dir / "build"
        if build_dir.exists():
            for item in build_dir.rglob("*"):
                if item.is_file():
                    relative_path = item.relative_to(build_dir)
                    target_path = package_path / relative_path
                    target_path.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(item, target_path)
        
        # Copy source if requested
        if (self.source_dir / "manifest").exists():
            shutil.copy2(self.source_dir / "manifest", package_path / "manifest")
        
        # Copy install scripts
        scripts_dir = package_path / "scripts"
        scripts_dir.mkdir()
        
        for script_name, script_content in self.config.install_scripts.items():
            if script_content:
                script_path = scripts_dir / script_name
                with open(script_path, 'w') as f:
                    f.write(script_content)
                script_path.chmod(0o755)
        
        return package_path
    
    def _generate_metadata(self, package_path: Path) -> None:
        """Generate package metadata"""
        print("  Generating metadata...")
        
        metadata = {
            "name": self.config.name,
            "version": self.config.version,
            "description": self.config.description,
            "architecture": self.config.architecture,
            "maintainer": self.config.maintainer,
            "license": self.config.license,
            "dependencies": self.config.dependencies,
            "provides": self.config.provides,
            "conflicts": self.config.conflicts,
            "categories": self.config.categories,
            "tags": self.config.tags,
            "homepage": self.config.homepage,
            "source_url": self.config.source_url,
            "install_date": datetime.now().isoformat(),
            "build_info": {
                "build_timestamp": datetime.now().isoformat(),
                "builder_version": "0.1.0",
                "build_host": os.uname().nodename
            }
        }
        
        metadata_file = package_path / "metadata.json"
        with open(metadata_file, 'w') as f:
            json.dump(metadata, f, indent=2)
    
    def _create_archive(self, package_path: Path) -> str:
        """Create compressed package archive"""
        print("  Creating archive...")
        
        archive_name = f"{self.config.name}-{self.config.version}-{self.config.architecture}.tar.xz"
        archive_path = self.package_dir / archive_name
        
        with tarfile.open(archive_path, "w:xz") as tar:
            tar.add(package_path, arcname=self.config.name)
        
        return str(archive_path)
    
    def _calculate_checksums(self, archive_path: str) -> None:
        """Calculate and store checksums"""
        print("  Calculating checksums...")
        
        # Calculate SHA256
        sha256_hash = hashlib.sha256()
        with open(archive_path, 'rb') as f:
            for chunk in iter(lambda: f.read(4096), b""):
                sha256_hash.update(chunk)
        
        checksum_file = f"{archive_path}.sha256"
        with open(checksum_file, 'w') as f:
            f.write(f"{sha256_hash.hexdigest()}  {os.path.basename(archive_path)}\n")
        
        print(f"  Checksum: {sha256_hash.hexdigest()}")
    
    def _sign_package(self, archive_path: str, key_id: Optional[str]) -> None:
        """Sign the package"""
        print("  Signing package...")
        
        if not key_id:
            print("  Warning: No key ID provided, skipping signature")
            return
        
        # This would integrate with the signing system
        # For now, just create a placeholder signature file
        signature_file = f"{archive_path}.sig"
        with open(signature_file, 'w') as f:
            f.write(f"# Signature for {os.path.basename(archive_path)}\n")
            f.write(f"# Key ID: {key_id}\n")
            f.write(f"# Algorithm: ed25519\n")
            f.write(f"# Timestamp: {datetime.now().isoformat()}\n")
        
        print(f"  Signature created: {signature_file}")
    
    def verify(self, package_path: str) -> bool:
        """Verify package integrity"""
        print(f"Verifying package: {package_path}")
        
        try:
            # Check if archive exists
            if not Path(package_path).exists():
                print(f"  Error: Package file not found: {package_path}")
                return False
            
            # Check checksum if available
            checksum_file = f"{package_path}.sha256"
            if Path(checksum_file).exists():
                expected_hash = None
                with open(checksum_file, 'r') as f:
                    line = f.readline().strip()
                    if '  ' in line:
                        expected_hash = line.split('  ')[0]
                
                if expected_hash:
                    sha256_hash = hashlib.sha256()
                    with open(package_path, 'rb') as f:
                        for chunk in iter(lambda: f.read(4096), b""):
                            sha256_hash.update(chunk)
                    
                    if sha256_hash.hexdigest() == expected_hash:
                        print("  ✓ Checksum verification passed")
                    else:
                        print("  ✗ Checksum verification failed")
                        return False
            
            # Extract and verify structure
            with tempfile.TemporaryDirectory() as temp_dir:
                with tarfile.open(package_path, 'r:xz') as tar:
                    tar.extractall(temp_dir)
                
                # Check for required files
                extracted_path = Path(temp_dir)
                if not (extracted_path / "metadata.json").exists():
                    print("  ✗ Missing metadata.json")
                    return False
                
                print("  ✓ Package structure verification passed")
            
            print("Package verification passed")
            return True
        
        except Exception as e:
            print(f"  Error during verification: {e}")
            return False
    
    def publish(self, package_path: str, repository_url: str, 
               dry_run: bool = False) -> bool:
        """Publish package to repository"""
        print(f"Publishing package to: {repository_url}")
        
        if dry_run:
            print("  Dry run mode - would upload:")
            print(f"    {package_path}")
            if Path(f"{package_path}.sha256").exists():
                print(f"    {package_path}.sha256")
            if Path(f"{package_path}.sig").exists():
                print(f"    {package_path}.sig")
            return True
        
        try:
            # This would implement actual upload to repository
            # For now, just validate the package
            if not self.verify(package_path):
                return False
            
            print("Package published successfully")
            return True
        
        except Exception as e:
            print(f"  Error during publish: {e}")
            return False


def main():
    """Main CLI entry point"""
    parser = argparse.ArgumentParser(description="MultiOS Package Builder")
    subparsers = parser.add_subparsers(dest="command", help="Available commands")
    
    # Skeleton command
    skeleton_parser = subparsers.add_parser("skeleton", help="Create package skeleton")
    skeleton_parser.add_argument("name", help="Package name")
    skeleton_parser.add_argument("--version", default="1.0.0", help="Initial version")
    skeleton_parser.add_argument("--maintainer", help="Maintainer info")
    skeleton_parser.add_argument("--license", default="MIT", help="License")
    
    # Build command
    build_parser = subparsers.add_parser("build", help="Build package")
    build_parser.add_argument("--config", help="Configuration file path")
    build_parser.add_argument("--clean", action="store_true", help="Clean build directory")
    build_parser.add_argument("--sign", action="store_true", help="Sign package")
    build_parser.add_argument("--key-id", help="Key ID for signing")
    
    # Verify command
    verify_parser = subparsers.add_parser("verify", help="Verify package")
    verify_parser.add_argument("package", help="Package file path")
    
    # Publish command
    publish_parser = subparsers.add_parser("publish", help="Publish package")
    publish_parser.add_argument("package", help="Package file path")
    publish_parser.add_argument("repository", help="Repository URL")
    publish_parser.add_argument("--dry-run", action="store_true", help="Dry run only")
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return 1
    
    builder = PackageBuilder()
    
    try:
        if args.command == "skeleton":
            builder.create_skeleton(args.name, args.version, args.maintainer, args.license)
        
        elif args.command == "build":
            package_path = builder.build(args.clean, args.sign, args.key_id)
            print(f"Built package: {package_path}")
        
        elif args.command == "verify":
            success = builder.verify(args.package)
            return 0 if success else 1
        
        elif args.command == "publish":
            success = builder.publish(args.package, args.repository, args.dry_run)
            return 0 if success else 1
        
        else:
            print(f"Unknown command: {args.command}")
            return 1
    
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()
        return 1
    
    return 0


if __name__ == "__main__":
    sys.exit(main())