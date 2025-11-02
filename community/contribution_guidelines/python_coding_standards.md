# Python Coding Standards

This document defines the coding standards and style guidelines for all Python code in the MultiOS project. These standards ensure code quality, maintainability, and consistency for Python scripts, tools, and utilities.

## 📋 Table of Contents

- [Code Formatting](#code-formatting)
- [Naming Conventions](#naming-conventions)
- [Type Hints and Annotations](#type-hints-and-annotations)
- [Error Handling](#error-handling)
- [Performance Guidelines](#performance-guidelines)
- [Testing Standards](#testing-standards)
- [Package Management](#package-management)
- [Documentation](#documentation)
- [Linting and CI](#linting-and-ci)

## 🎨 Code Formatting

### PEP 8 Compliance

All Python code must follow PEP 8 guidelines with the following project-specific additions:

**Line Length and Structure:**
- **Maximum line length**: 88 characters (Black formatter default)
- **Indentation**: 4 spaces (no tabs)
- **Line continuations**: Use backslashes sparingly, prefer implicit continuation
- **Blank lines**: 2 blank lines between top-level functions and classes

**Example Code Structure:**
```python
"""
MultiOS Python utility module for memory analysis.

This module provides utilities for analyzing memory usage patterns
in MultiOS kernel and userland applications.
"""

from typing import Dict, List, Optional, Union, Any
from dataclasses import dataclass
import logging
from pathlib import Path


@dataclass
class MemoryRegion:
    """Represents a memory region in the system."""
    start_address: int
    size: int
    permissions: str
    name: Optional[str] = None


class MemoryAnalyzer:
    """Analyzes memory usage and fragmentation patterns."""
    
    def __init__(self, log_level: int = logging.INFO) -> None:
        """Initialize the memory analyzer.
        
        Args:
            log_level: Logging level for debugging output.
        """
        self.logger = logging.getLogger(__name__)
        self.logger.setLevel(log_level)
        self.regions: Dict[str, MemoryRegion] = {}
        
    def analyze_memory_map(self, 
                          memory_map: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Analyze memory regions and return statistics.
        
        Args:
            memory_map: List of memory region dictionaries from kernel.
            
        Returns:
            Dictionary containing analysis results.
            
        Raises:
            ValueError: If memory map contains invalid entries.
        """
        if not memory_map:
            raise ValueError("Memory map cannot be empty")
            
        total_size = 0
        used_size = 0
        regions_by_type: Dict[str, int] = {}
        
        for region in memory_map:
            try:
                mem_region = MemoryRegion(
                    start_address=int(region['start'], 16),
                    size=int(region['size'], 16),
                    permissions=region['permissions'],
                    name=region.get('name')
                )
            except (KeyError, ValueError) as e:
                self.logger.error(f"Invalid memory region: {region}")
                raise ValueError(f"Invalid memory region format: {e}")
            
            # Update statistics
            total_size += mem_region.size
            if 'X' in mem_region.permissions:
                used_size += mem_region.size
                
            region_type = self._get_region_type(mem_region)
            regions_by_type[region_type] = regions_by_type.get(region_type, 0) + 1
            
            # Store for further analysis
            region_key = f"0x{mem_region.start_address:x}"
            self.regions[region_key] = mem_region
            
        fragmentation_ratio = (total_size - used_size) / total_size if total_size > 0 else 0
        
        return {
            'total_size': total_size,
            'used_size': used_size,
            'free_size': total_size - used_size,
            'fragmentation_ratio': fragmentation_ratio,
            'regions_by_type': regions_by_type,
            'region_count': len(memory_map)
        }
```

### Black Code Formatting

All code should be formatted using Black with the project configuration:

```toml
# pyproject.toml
[tool.black]
line-length = 88
target-version = ['py38', 'py39', 'py310', 'py311']
include = '\.pyi?$'
extend-exclude = '''
/(
    \.git
  | \.hg
  | \.mypy_cache
  | \.tox
  | \.venv
  | build
  | dist
)/
'''

[tool.isort]
profile = "black"
line_length = 88
multi_line_output = 3
include_trailing_comma = true
force_grid_wrap = 0
use_parentheses = true
ensure_newline_before_comments = true

[tool.mypy]
python_version = "3.8"
warn_return_any = true
warn_unused_configs = true
disallow_untyped_defs = true
check_untyped_defs = true
disallow_incomplete_defs = true
strict_optional = true
warn_redundant_casts = true
warn_unused_ignores = true
show_error_codes = true
```

## 🏷️ Naming Conventions

### Variables and Functions

```python
# Variables and functions: snake_case
max_buffer_size = 4096
current_pointer = 0x1000

def calculate_checksum(data: bytes) -> int:
    """Calculate checksum for data buffer."""
    checksum = 0
    for byte in data:
        checksum = (checksum + byte) & 0xFF
    return checksum

def initialize_memory_manager(config_path: str) -> bool:
    """Initialize memory manager with configuration."""
    return True
```

### Classes and Modules

```python
# Classes: PascalCase
class MemoryRegion:
    """Represents a memory region."""
    pass

class NetworkPacket:
    """Network packet with header and payload."""
    def __init__(self, header: bytes, payload: bytes) -> None:
        self.header = header
        self.payload = payload

# Modules: snake_case
# memory_analyzer.py
# network_utils.py
# device_drivers.py

# Package names: lowercase
# multios.utils
# multios.analysis
# multios.tools
```

### Constants

```python
# Constants: UPPER_CASE
MAX_MEMORY_SIZE = 0xFFFF_FFFF
DEFAULT_PAGE_SIZE = 4096
NETWORK_BUFFER_SIZE = 1500

# Enum-like classes
class MemoryType:
    """Memory region types."""
    KERNEL = "kernel"
    USER = "user"
    DEVICE = "device"
    DMA = "dma"
```

### Private vs Public

```python
class NetworkInterface:
    """Network interface for MultiOS networking stack."""
    
    def __init__(self, name: str, mac_address: str) -> None:
        self.name = name  # Public attribute
        self._mac_address = mac_address  # Protected by convention
        self.__state = "down"  # Name mangling for internal use
    
    @property
    def is_up(self) -> bool:
        """Check if interface is active."""
        return self.__state == "up"
    
    def up(self) -> None:
        """Bring interface up."""
        self.__state = "up"
        self._send_link_up_event()
    
    def _send_link_up_event(self) -> None:
        """Send link up notification (internal method)."""
        # Implementation
        pass
```

## 🔍 Type Hints and Annotations

### Basic Type Hints

```python
from typing import List, Dict, Optional, Union, Callable, Generic, TypeVar
from abc import ABC, abstractmethod

T = TypeVar('T')

class Container(Generic[T]):
    """Generic container for storing items."""
    
    def __init__(self) -> None:
        self._items: List[T] = []
    
    def add(self, item: T) -> None:
        """Add item to container."""
        self._items.append(item)
    
    def get(self, index: int) -> Optional[T]:
        """Get item at index."""
        return self._items[index] if 0 <= index < len(self._items) else None
    
    def get_all(self) -> List[T]:
        """Get all items in container."""
        return self._items.copy()

def process_data(data: List[Dict[str, Union[str, int]]]) -> Dict[str, Any]:
    """Process list of data dictionaries."""
    result: Dict[str, Any] = {}
    for item in data:
        key = item['name']
        value = item.get('value', 0)
        result[key] = value
    return result
```

### Advanced Type Patterns

```python
from typing import Protocol, TypedDict, Literal

class DriverProtocol(Protocol):
    """Protocol for device drivers."""
    
    def initialize(self) -> bool:
        """Initialize the driver."""
        ...
    
    def read(self, address: int, size: int) -> bytes:
        """Read data from device."""
        ...

class MemoryRegionConfig(TypedDict):
    """Configuration for memory regions."""
    start_address: int
    size: int
    permissions: Literal['R', 'W', 'X', 'RW', 'RX', 'WX', 'RWX']

def create_memory_regions(configs: List[MemoryRegionConfig]) -> List[MemoryRegion]:
    """Create memory regions from configuration."""
    return [MemoryRegion(**config) for config in configs]

# Union types for flexible return values
Result = Union[MemoryRegion, None]
Error = Union[ValueError, PermissionError]

def safe_allocate_memory(size: int, alignment: int = 1) -> Result:
    """Safely allocate memory with optional alignment."""
    if size <= 0:
        return None
    
    try:
        address = _allocate_raw_memory(size, alignment)
        return MemoryRegion(start_address=address, size=size, permissions='RW')
    except MemoryError:
        return None
```

## ⚠️ Error Handling

### Exception Hierarchy

```python
class MultiOSError(Exception):
    """Base exception for MultiOS operations."""
    pass

class MemoryError(MultiOSError):
    """Memory-related errors."""
    pass

class DeviceError(MultiOSError):
    """Device-related errors."""
    pass

class NetworkError(MultiOSError):
    """Network-related errors."""
    pass

class PermissionError(MultiOSError):
    """Permission-related errors."""
    pass
```

### Proper Error Handling

```python
import logging
from contextlib import contextmanager

logger = logging.getLogger(__name__)

def load_memory_map(file_path: str) -> List[Dict[str, Any]]:
    """Load memory map from JSON file."""
    try:
        with open(file_path, 'r') as f:
            data = json.load(f)
            return data['regions']
    except FileNotFoundError:
        logger.error(f"Memory map file not found: {file_path}")
        raise MemoryError(f"Memory map file not found: {file_path}")
    except json.JSONDecodeError as e:
        logger.error(f"Invalid JSON in memory map: {e}")
        raise MemoryError(f"Invalid JSON in memory map: {e}")
    except KeyError as e:
        logger.error(f"Missing required field in memory map: {e}")
        raise MemoryError(f"Invalid memory map structure: missing {e}")

@contextmanager
def safe_file_operation(path: str, mode: str):
    """Context manager for safe file operations."""
    file_handle = None
    try:
        file_handle = open(path, mode)
        yield file_handle
    except Exception as e:
        logger.error(f"File operation failed on {path}: {e}")
        raise
    finally:
        if file_handle:
            file_handle.close()

# Usage
def analyze_memory_file(path: str) -> Dict[str, Any]:
    """Analyze memory from file."""
    with safe_file_operation(path, 'r') as f:
        content = f.read()
        return parse_memory_content(content)
```

### Defensive Programming

```python
def validate_memory_region(region: Dict[str, Any]) -> bool:
    """Validate memory region configuration."""
    required_fields = ['start_address', 'size', 'permissions']
    
    # Check required fields
    for field in required_fields:
        if field not in region:
            logger.error(f"Missing required field: {field}")
            return False
    
    # Validate types
    try:
        start_addr = int(region['start_address'], 16) if isinstance(region['start_address'], str) else region['start_address']
        size = int(region['size'], 16) if isinstance(region['size'], str) else region['size']
        permissions = region['permissions']
    except (ValueError, TypeError) as e:
        logger.error(f"Invalid field types in memory region: {e}")
        return False
    
    # Validate ranges
    if start_addr < 0 or size <= 0:
        logger.error(f"Invalid address range: start={start_addr}, size={size}")
        return False
    
    if size > MAX_MEMORY_SIZE:
        logger.warning(f"Large memory region: {size} bytes")
    
    if permissions not in ['R', 'W', 'X', 'RW', 'RX', 'WX', 'RWX']:
        logger.error(f"Invalid permissions: {permissions}")
        return False
    
    return True

# Pre-condition checking
def process_memory_blocks(blocks: List[Dict[str, Any]]) -> List[MemoryRegion]:
    """Process memory blocks with validation."""
    if not blocks:
        raise ValueError("Memory blocks list cannot be empty")
    
    if len(blocks) > MAX_BLOCKS_PER_ANALYSIS:
        raise ValueError(f"Too many blocks: {len(blocks)} > {MAX_BLOCKS_PER_ANALYSIS}")
    
    valid_blocks = []
    for i, block in enumerate(blocks):
        try:
            if validate_memory_region(block):
                valid_blocks.append(MemoryRegion(**block))
            else:
                logger.warning(f"Skipping invalid block at index {i}")
        except Exception as e:
            logger.error(f"Error processing block {i}: {e}")
            continue
    
    if not valid_blocks:
        raise ValueError("No valid memory blocks found")
    
    return valid_blocks
```

## ⚡ Performance Guidelines

### Performance-Sensitive Code

```python
import functools
from typing import Callable, Any
import time

# Use caching for expensive operations
@functools.lru_cache(maxsize=128)
def calculate_memory_checksum(address: int, size: int) -> int:
    """Calculate checksum for memory region (cached)."""
    # Expensive calculation
    checksum = 0
    for i in range(size):
        # Simulate memory access
        checksum = (checksum + i) & 0xFF
    return checksum

# Use generators for memory efficiency
def read_large_file(file_path: str) -> bytes:
    """Read large file in chunks."""
    chunk_size = 8192
    data = bytearray()
    
    with open(file_path, 'rb') as f:
        while True:
            chunk = f.read(chunk_size)
            if not chunk:
                break
            data.extend(chunk)
    
    return bytes(data)

# Pre-allocate known-size collections
def create_memory_table(size: int) -> List[int]:
    """Create memory table with pre-allocated size."""
    return [0] * size  # Pre-allocate instead of appending

# Use appropriate data structures
from collections import defaultdict, deque

def analyze_memory_patterns(regions: List[MemoryRegion]) -> Dict[str, List[MemoryRegion]]:
    """Analyze memory regions by type."""
    # Use defaultdict for automatic initialization
    regions_by_type = defaultdict(list)
    
    for region in regions:
        region_type = get_region_type(region)
        regions_by_type[region_type].append(region)
    
    return dict(regions_by_type)

# Use deque for efficient queue operations
def process_memory_events(event_queue: List[Dict[str, Any]]) -> None:
    """Process memory events efficiently."""
    queue = deque(event_queue)
    
    while queue:
        event = queue.popleft()
        process_single_event(event)
```

### Memory Efficiency

```python
from typing import Iterator
import gc

def lazy_memory_scan(start: int, end: int, chunk_size: int = 1024) -> Iterator[bytes]:
    """Lazy memory scanner for large ranges."""
    current = start
    
    while current < end:
        # Yield chunks instead of loading all at once
        chunk_end = min(current + chunk_size, end)
        chunk = read_memory_chunk(current, chunk_end)
        yield chunk
        current = chunk_end

def memory_efficient_analysis(regions: List[MemoryRegion]) -> Dict[str, Any]:
    """Analyze memory regions without loading everything into memory."""
    total_size = 0
    active_regions = 0
    
    for region in regions:
        # Process one region at a time
        if region.permissions != 'RWX':
            active_regions += 1
            total_size += region.size
        
        # Explicit cleanup for long-running processes
        if active_regions % 1000 == 0:
            gc.collect()
    
    return {
        'total_size': total_size,
        'active_regions': active_regions,
        'average_size': total_size / active_regions if active_regions > 0 else 0
    }
```

## 🧪 Testing Standards

### Test Structure

```python
import pytest
import unittest
from unittest.mock import Mock, patch, MagicMock
from typing import Dict, Any

class TestMemoryAnalyzer:
    """Test cases for MemoryAnalyzer class."""
    
    @pytest.fixture
    def analyzer(self) -> MemoryAnalyzer:
        """Create analyzer instance for testing."""
        return MemoryAnalyzer(log_level=logging.DEBUG)
    
    @pytest.fixture
    def sample_memory_map(self) -> List[Dict[str, Any]]:
        """Create sample memory map for testing."""
        return [
            {
                'start': '0x00000000',
                'size': '0x00100000',
                'permissions': 'RWX',
                'name': 'kernel'
            },
            {
                'start': '0x00100000',
                'size': '0x00100000',
                'permissions': 'RW',
                'name': 'user_data'
            }
        ]
    
    def test_initialization(self, analyzer: MemoryAnalyzer) -> None:
        """Test analyzer initialization."""
        assert analyzer.logger.level == logging.DEBUG
        assert len(analyzer.regions) == 0
    
    def test_analyze_memory_map(self, 
                               analyzer: MemoryAnalyzer,
                               sample_memory_map: List[Dict[str, Any]]) -> None:
        """Test memory map analysis."""
        result = analyzer.analyze_memory_map(sample_memory_map)
        
        assert result['total_size'] == 0x00200000
        assert result['used_size'] == 0x00200000
        assert result['region_count'] == 2
        assert 'kernel' in result['regions_by_type']
    
    def test_invalid_memory_map(self, analyzer: MemoryAnalyzer) -> None:
        """Test error handling for invalid input."""
        with pytest.raises(ValueError, match="Memory map cannot be empty"):
            analyzer.analyze_memory_map([])
    
    @patch('multios.utils.memory_analyzer.read_memory_chunk')
    def test_memory_scan(self, mock_read: Mock, analyzer: MemoryAnalyzer) -> None:
        """Test memory scanning with mocked read function."""
        mock_read.return_value = b'\x00' * 1024
        
        scanner = MemoryScanner(analyzer)
        results = scanner.scan_range(0x1000, 0x2000, 1024)
        
        assert len(results) > 0
        mock_read.assert_called()
```

### Property-Based Testing

```python
from hypothesis import given, strategies as st

class TestMemoryProperties:
    """Property-based tests for memory operations."""
    
    @given(st.lists(st.tuples(st.integers(min_value=0), st.integers(min_value=1))))
    def test_memory_allocation_consistency(self, allocations) -> None:
        """Test that memory allocations don't overlap."""
        allocated_ranges = []
        
        for start, size in allocations:
            new_range = (start, start + size)
            
            # Check for overlaps
            for existing_range in allocated_ranges:
                assert not ranges_overlap(new_range, existing_range)
            
            allocated_ranges.append(new_range)
    
    @given(st.integers(min_value=1), st.integers(min_value=1))
    def test_alignment_preservation(self, address: int, alignment: int) -> None:
        """Test that alignment operations preserve constraints."""
        aligned = align_up(address, alignment)
        
        assert aligned >= address
        assert (aligned - address) < alignment
        assert aligned % alignment == 0
```

### Integration Tests

```python
class TestMemoryManagerIntegration:
    """Integration tests for memory manager."""
    
    @pytest.mark.integration
    def test_full_memory_lifecycle(self) -> None:
        """Test complete memory allocation/deallocation cycle."""
        manager = MemoryManager()
        
        # Allocate memory
        addr1 = manager.allocate(1024)
        addr2 = manager.allocate(2048)
        
        assert addr1 is not None
        assert addr2 is not None
        assert addr1 != addr2
        
        # Write and read data
        manager.write_memory(addr1, b'test data')
        data = manager.read_memory(addr1, 9)
        
        assert data == b'test data'
        
        # Deallocate
        assert manager.deallocate(addr1)
        assert manager.deallocate(addr2)
        
        # Verify cleanup
        assert not manager.deallocate(addr1)  # Should fail - already freed
```

## 📦 Package Management

### Dependency Management

```toml
# pyproject.toml
[project]
name = "multios-utils"
version = "0.1.0"
description = "MultiOS Python utilities and tools"
authors = [{name = "MultiOS Contributors", email = "contributors@multios.org"}]
license = {text = "MIT"}
readme = "README.md"
requires-python = ">=3.8"
classifiers = [
    "Development Status :: 3 - Alpha",
    "Intended Audience :: Developers",
    "License :: OSI Approved :: MIT License",
    "Programming Language :: Python :: 3",
    "Programming Language :: Python :: 3.8",
    "Programming Language :: Python :: 3.9",
    "Programming Language :: Python :: 3.10",
    "Programming Language :: Python :: 3.11",
]

dependencies = [
    "click>=8.0.0",
    "pydantic>=1.10.0",
    "typing-extensions>=4.0.0",
]

[project.optional-dependencies]
dev = [
    "pytest>=7.0.0",
    "pytest-cov>=4.0.0",
    "black>=22.0.0",
    "isort>=5.0.0",
    "flake8>=5.0.0",
    "mypy>=1.0.0",
    "pre-commit>=2.0.0",
]
test = [
    "pytest>=7.0.0",
    "pytest-cov>=4.0.0",
    "hypothesis>=6.0.0",
]
performance = [
    "memory-profiler>=0.60.0",
    "psutil>=5.0.0",
]

[project.scripts]
multios-analyze = "multios.cli.analyze:main"
multios-monitor = "multios.cli.monitor:main"
```

### Virtual Environment Setup

```bash
#!/bin/bash
# setup_dev_env.sh

# Create virtual environment
python -m venv venv

# Activate virtual environment
source venv/bin/activate

# Install development dependencies
pip install -e ".[dev]"

# Install pre-commit hooks
pre-commit install

echo "Development environment setup complete!"
```

## 📚 Documentation

### Module Documentation

```python
"""MultiOS Memory Analysis Utilities.

This package provides utilities for analyzing memory usage patterns
in MultiOS kernel and userland applications. It includes tools for:

- Memory region analysis and reporting
- Fragmentation detection and measurement
- Memory leak identification
- Performance profiling and optimization

Basic Usage:
    >>> from multios.utils.memory import MemoryAnalyzer
    >>> analyzer = MemoryAnalyzer()
    >>> regions = analyzer.scan_memory()
    >>> print(f"Found {len(regions)} memory regions")
"""

from .analyzer import MemoryAnalyzer
from .scanner import MemoryScanner
from .reporter import MemoryReporter

__all__ = ['MemoryAnalyzer', 'MemoryScanner', 'MemoryReporter']
__version__ = '1.0.0'

# Public API
def create_analyzer(config: Dict[str, Any]) -> MemoryAnalyzer:
    """Factory function for creating memory analyzers.
    
    Args:
        config: Configuration dictionary with settings.
        
    Returns:
        Configured MemoryAnalyzer instance.
        
    Raises:
        ConfigurationError: If configuration is invalid.
    """
    return MemoryAnalyzer.from_config(config)
```

### Docstring Standards

```python
def allocate_memory(size: int, alignment: int = 1) -> Optional[int]:
    """Allocate memory region with specified size and alignment.
    
    This function attempts to allocate a contiguous memory region
    with the given size and alignment requirements. It searches
    the available memory regions for a suitable location.
    
    Args:
        size: Number of bytes to allocate. Must be positive.
        alignment: Memory alignment requirement in bytes. Must be
            a power of two. Defaults to 1 (no alignment).
    
    Returns:
        Starting address of allocated memory region, or None if
        allocation fails due to insufficient memory.
    
    Raises:
        ValueError: If size is non-positive or alignment is not
            a power of two.
        MemoryError: If kernel is not initialized.
    
    Examples:
        Basic allocation:
            >>> addr = allocate_memory(4096)
            >>> print(f"Allocated at 0x{addr:x}")
        
        Aligned allocation:
            >>> addr = allocate_memory(1024, 4096)
            >>> print(f"Aligned at 0x{addr:x}")
    
    Note:
        This function operates in kernel context and may block
        if memory management subsystem is busy.
    """
    if size <= 0:
        raise ValueError("size must be positive")
    
    if alignment & (alignment - 1) != 0:
        raise ValueError("alignment must be a power of two")
    
    return _kernel_allocate(size, alignment)
```

## 🔧 Linting and CI

### Configuration Files

```ini
# .flake8
[flake8]
max-line-length = 88
extend-ignore = E203, W503, E501
exclude = 
    .git,
    __pycache__,
    build,
    dist,
    *.egg-info,
    .venv

# Specific rules
per-file-ignores =
    __init__.py:F401
    test_*.py:F401,F811
```

```toml
# mypy.ini
[mypy]
python_version = 3.8
warn_return_any = True
warn_unused_configs = True
disallow_untyped_defs = True
check_untyped_defs = True
disallow_incomplete_defs = True
strict_optional = True
warn_redundant_casts = True
warn_unused_ignores = True
show_error_codes = True

[mypy-tests.*]
disallow_untyped_defs = False
```

### Pre-commit Configuration

```yaml
# .pre-commit-config.yaml
repos:
  - repo: https://github.com/psf/black
    rev: 22.3.0
    hooks:
      - id: black
        language_version: python3
  
  - repo: https://github.com/pycqa/isort
    rev: 5.10.1
    hooks:
      - id: isort
        args: ["--profile", "black"]
  
  - repo: https://github.com/pycqa/flake8
    rev: 4.0.1
    hooks:
      - id: flake8
  
  - repo: https://github.com/pre-commit/mirrors-mypy
    rev: v0.950
    hooks:
      - id: mypy

  - repo: local
    hooks:
      - id: pytest
        name: pytest
        entry: pytest
        language: system
        pass_filenames: false
        always_run: true
```

### CI Configuration

```yaml
# .github/workflows/python.yml
name: Python CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        python-version: [3.8, 3.9, "3.10", 3.11]
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Set up Python ${{ matrix.python-version }}
      uses: actions/setup-python@v3
      with:
        python-version: ${{ matrix.python-version }}
    
    - name: Install dependencies
      run: |
        python -m pip install --upgrade pip
        pip install -e ".[dev,test]"
    
    - name: Lint with flake8
      run: |
        flake8 src tests
    
    - name: Check formatting with black
      run: |
        black --check src tests
    
    - name: Check imports with isort
      run: |
        isort --check-only src tests
    
    - name: Type checking with mypy
      run: |
        mypy src
    
    - name: Test with pytest
      run: |
        pytest --cov=src --cov-report=xml
    
    - name: Upload coverage
      uses: codecov/codecov-action@v3
      with:
        file: ./coverage.xml
```

---

## 📋 Checklist for Code Reviews

When reviewing Python code, ensure:

- [ ] Code follows PEP 8 guidelines (use black/isort check)
- [ ] Type hints are present for all public functions
- [ ] No mypy warnings or type errors
- [ ] Tests are included and have adequate coverage (>80%)
- [ ] Documentation strings follow Google style
- [ ] Error handling is appropriate (no bare except clauses)
- [ ] Performance considerations for loops and data structures
- [ ] Dependencies are minimal and justified
- [ ] Virtual environment setup is documented
- [ ] Pre-commit hooks are configured

*Last Updated: November 3, 2025*