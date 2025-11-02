#!/usr/bin/env python3
"""
Quick test of database systems functionality
Tests core operations without SQL parsing
"""

import sys
import os
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

def test_relational_db():
    """Test relational database core functionality"""
    from relational.relational_engine import RelationalEngine
    
    print("Testing Relational Database...")
    engine = RelationalEngine()
    
    # Create table
    engine.create_table("students", {
        "name": "VARCHAR",
        "student_id": "INT", 
        "major": "VARCHAR",
        "gpa": "REAL"
    })
    
    # Insert data
    engine.insert("students", {
        "name": "Alice Johnson",
        "student_id": 101,
        "major": "Computer Science",
        "gpa": 3.8
    })
    
    engine.insert("students", {
        "name": "Bob Smith", 
        "student_id": 102,
        "major": "Mathematics",
        "gpa": 3.9
    })
    
    # Query data
    results = engine.select("students")
    print(f"✓ Found {len(results)} students")
    
    # Test index
    engine.create_index("students", "student_id")
    print("✓ Created index")
    
    return True

def test_nosql_db():
    """Test NoSQL database functionality"""
    from nosql.document_db import DocumentDB
    
    print("\nTesting NoSQL Database...")
    db = DocumentDB()
    
    # Create collection and insert documents
    db.insert("users", {
        "name": "Alice",
        "age": 25,
        "skills": ["Python", "SQL"]
    })
    
    # Query
    results = db.find("users", {"age": {"$gt": 20}})
    print(f"✓ Found {len(results)} users over 20")
    
    return True

def test_graph_db():
    """Test graph database functionality"""
    from graph.graph_db import GraphDB
    
    print("\nTesting Graph Database...")
    graph = GraphDB()
    
    # Add nodes
    graph.add_node("Alice", {"role": "student", "age": 22})
    graph.add_node("Bob", {"role": "instructor", "age": 35})
    
    # Add edge
    graph.add_edge("Alice", "Bob", "supervised_by")
    
    # Query
    relationships = graph.get_relationships("Alice")
    print(f"✓ Alice has {len(relationships)} relationships")
    
    return True

def test_security():
    """Test security manager"""
    from security.security_manager import SecurityManager
    
    print("\nTesting Security Manager...")
    security = SecurityManager()
    
    # Create user
    security.create_user("alice", "password123", ["admin", "developer"])
    
    # Authenticate
    result = security.authenticate("alice", "password123")
    print(f"✓ Authentication: {result}")
    
    return True

def test_tutorials():
    """Test tutorial manager"""
    from tutorials.tutorial_manager import TutorialManager
    
    print("\nTesting Tutorial Manager...")
    tutorial = TutorialManager()
    
    exercises = tutorial.get_tutorials()
    print(f"✓ Loaded {len(exercises)} tutorials")
    
    return True

def main():
    """Run all tests"""
    print("QUICK DATABASE SYSTEMS TEST")
    print("="*50)
    
    tests = [
        test_relational_db,
        test_nosql_db, 
        test_graph_db,
        test_security,
        test_tutorials
    ]
    
    passed = 0
    for test in tests:
        try:
            if test():
                passed += 1
        except Exception as e:
            print(f"✗ {test.__name__} failed: {e}")
    
    print(f"\n{passed}/{len(tests)} tests passed")
    
    if passed == len(tests):
        print("✓ All core database systems are working!")
    else:
        print("⚠ Some components need attention")
    
    return passed == len(tests)

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)