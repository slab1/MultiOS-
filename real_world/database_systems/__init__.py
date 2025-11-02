"""
Database Systems Educational Package

A comprehensive framework for learning database systems concepts through
hands-on implementation and interactive tutorials.

This package includes:
- Relational Database Engine (ACID, SQL parsing)
- NoSQL Document Database (flexible schemas)
- Graph Database (social networks, relationships)
- Distributed Database (sharding, replication)
- Security & Access Control (authentication, encryption)
- Query Optimization (indexing, performance)
- Educational Tutorials (hands-on exercises)
"""

__version__ = "1.0.0"
__author__ = "Database Systems Education Team"

# Import main classes for easy access
try:
    from relational.relational_engine import RelationalEngine
    from nosql.document_db import DocumentDB
    from graph.graph_db import GraphDB
    from distributed.distributed_db import DistributedDatabase, ConsistencyLevel
    from security.security_manager import SecurityManager
    from optimization.query_optimizer import QueryOptimizer, DatabaseIndex, IndexType
    from tutorials.tutorial_manager import TutorialManager, DifficultyLevel, Topic
    
    # Package-level exports
    __all__ = [
        'RelationalEngine',
        'DocumentDB', 
        'GraphDB',
        'DistributedDatabase',
        'ConsistencyLevel',
        'SecurityManager',
        'QueryOptimizer',
        'DatabaseIndex',
        'IndexType',
        'TutorialManager',
        'DifficultyLevel',
        'Topic'
    ]
    
except ImportError as e:
    # Allow package import even if dependencies are missing
    __all__ = []
    __import_warning__ = f"Warning: Some dependencies not available. {str(e)}"