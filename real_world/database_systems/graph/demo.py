"""
Educational Demo for Graph Database
Demonstrates social network analysis, graph traversal, and relationship queries
"""

import sys
import os
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from graph_db import GraphDB
import json
import random


def demonstrate_social_network():
    """Demonstrate social network analysis"""
    print("\n" + "="*60)
    print("SOCIAL NETWORK DEMONSTRATION")
    print("="*60)
    
    db = GraphDB()
    
    # Create social network
    print("\n1. Creating social network...")
    
    # Create people
    people_data = [
        {'name': 'Alice Johnson', 'age': 28, 'city': 'New York', 'job': 'Engineer'},
        {'name': 'Bob Smith', 'age': 35, 'city': 'Los Angeles', 'job': 'Designer'},
        {'name': 'Carol Davis', 'age': 24, 'city': 'Chicago', 'job': 'Student'},
        {'name': 'David Brown', 'age': 42, 'city': 'Seattle', 'job': 'Manager'},
        {'name': 'Emma Wilson', 'age': 31, 'city': 'Boston', 'job': 'Doctor'},
        {'name': 'Frank Miller', 'age': 29, 'city': 'New York', 'job': 'Lawyer'},
        {'name': 'Grace Lee', 'age': 33, 'city': 'San Francisco', 'job': 'Entrepreneur'},
        {'name': 'Henry Chen', 'age': 26, 'city': 'Los Angeles', 'job': 'Engineer'},
        {'name': 'Ivy Rodriguez', 'age': 37, 'city': 'Miami', 'job': 'Teacher'},
        {'name': 'Jack Thompson', 'age': 44, 'city': 'Seattle', 'job': 'Manager'}
    ]
    
    people_ids = {}
    for person in people_data:
        node_id = db.create_node(['PERSON'], person)
        people_ids[person['name']] = node_id
        print(f"✓ Created person: {person['name']}")
    
    # Create friendships
    print("\n2. Creating friendships...")
    friendships = [
        ('Alice Johnson', 'Bob Smith'),
        ('Alice Johnson', 'Carol Davis'),
        ('Bob Smith', 'David Brown'),
        ('Carol Davis', 'Emma Wilson'),
        ('David Brown', 'Frank Miller'),
        ('Emma Wilson', 'Grace Lee'),
        ('Frank Miller', 'Henry Chen'),
        ('Grace Lee', 'Ivy Rodriguez'),
        ('Henry Chen', 'Jack Thompson'),
        ('Alice Johnson', 'Grace Lee'),
        ('Bob Smith', 'Henry Chen'),
        ('Carol Davis', 'Frank Miller'),
        ('David Brown', 'Emma Wilson'),
        ('Ivy Rodriguez', 'Jack Thompson')
    ]
    
    for person1, person2 in friendships:
        edge_id = db.create_edge(
            people_ids[person1], 
            people_ids[person2], 
            'FRIEND',
            {'since': '2020-01-01', 'strength': random.choice([0.7, 0.8, 0.9, 1.0])}
        )
        print(f"✓ {person1} ↔ {person2} (friends since 2020)")
    
    print(f"\n3. Network statistics:")
    stats = db.analyze_relationship_patterns()
    print(f"   Total people: {stats['total_nodes']}")
    print(f"   Total friendships: {stats['total_edges']}")
    print(f"   Average friends per person: {stats['social_network_analysis']['average_friends_per_person']:.1f}")
    print(f"   Network density: {stats['density']:.3f}")
    
    # Find friends of a person
    print("\n4. Finding friends...")
    alice_id = people_ids['Alice Johnson']
    alice_friends = db.get_neighbors(alice_id, 'FRIEND')
    print(f"✓ Alice's friends:")
    for friend_id in alice_friends:
        friend_node = db.get_node(friend_id)
        friend_name = friend_node.get_property('name')
        print(f"   - {friend_name}")
    
    # Find mutual friends
    print("\n5. Finding mutual friends...")
    alice_id = people_ids['Alice Johnson']
    bob_id = people_ids['Bob Smith']
    mutual_friends = db.find_common_neighbors(alice_id, bob_id)
    print(f"✓ Alice and Bob's mutual friends:")
    for friend_id in mutual_friends:
        friend_node = db.get_node(friend_id)
        friend_name = friend_node.get_property('name')
        print(f"   - {friend_name}")
    
    # Friend recommendations
    print("\n6. Friend recommendations...")
    for person_name in ['Alice Johnson', 'Bob Smith', 'Carol Davis']:
        person_id = people_ids[person_name]
        neighbors = db.get_neighbors(person_id, 'FRIEND')
        person_friends = set(neighbors)
        
        # Get friends of friends
        recommendations = set()
        for friend_id in neighbors:
            friend_of_friends = db.get_neighbors(friend_id, 'FRIEND')
            recommendations.update(friend_of_friends)
        
        # Remove existing friends and self
        recommendations = recommendations - person_friends - {person_id}
        
        print(f"✓ {person_name}'s recommendations:")
        for rec_id in list(recommendations)[:3]:  # Show top 3
            rec_node = db.get_node(rec_id)
            rec_name = rec_node.get_property('name')
            print(f"   - {rec_name}")
    
    # Find shortest path between people
    print("\n7. Finding connections...")
    path = db.shortest_path(
        people_ids['Alice Johnson'], 
        people_ids['Jack Thompson']
    )
    
    if path:
        path_names = []
        for node_id in path:
            node = db.get_node(node_id)
            name = node.get_property('name')
            path_names.append(name)
        print(f"✓ Alice → Jack path: {' → '.join(path_names)}")
        print(f"   (Distance: {len(path) - 1} hops)")
    
    # Degree centrality
    print("\n8. Most connected people...")
    degrees = {}
    for person_name in people_data:
        node_id = people_ids[person_name['name']]
        degree = db.get_node_degree(node_id, 'FRIEND')
        degrees[person_name['name']] = degree
    
    sorted_degrees = sorted(degrees.items(), key=lambda x: x[1], reverse=True)
    for name, degree in sorted_degrees[:5]:
        print(f"   - {name}: {degree} friends")
    
    # Community detection
    print("\n9. Finding communities...")
    communities = db.detect_communities()
    community_groups = {}
    for node_id, community_id in communities.items():
        if community_id not in community_groups:
            community_groups[community_id] = []
        
        node = db.get_node(node_id)
        if node and 'PERSON' in node.labels:
            name = node.get_property('name')
            community_groups[community_id].append(name)
    
    for comm_id, members in community_groups.items():
        print(f"   Community {comm_id + 1}: {', '.join(members)}")


def demonstrate_graph_traversal():
    """Demonstrate graph traversal algorithms"""
    print("\n" + "="*60)
    print("GRAPH TRAVERSAL ALGORITHMS DEMONSTRATION")
    print("="*60)
    
    db = GraphDB()
    _create_complex_graph(db)
    
    # Pick a start node
    start_node = list(db.nodes.keys())[0]
    start_node_obj = db.get_node(start_node)
    start_name = start_node_obj.get_property('name') if start_node_obj else start_node
    
    print(f"\n1. Starting traversal from: {start_name}")
    
    # Breadth-First Search
    print("\n2. Breadth-First Search (BFS) traversal...")
    bfs_result = db.traverse_bfs(start_node, max_depth=2)
    print("   BFS results (node, depth):")
    for node_id, depth in bfs_result[:10]:  # Show first 10
        node = db.get_node(node_id)
        name = node.get_property('name') if node else f"Node_{node_id[:8]}"
        print(f"   - {name} (depth {depth})")
    
    # Depth-First Search
    print("\n3. Depth-First Search (DFS) traversal...")
    dfs_result = db.traverse_dfs(start_node, max_depth=2)
    print("   DFS results (node, depth):")
    for node_id, depth in dfs_result[:10]:  # Show first 10
        node = db.get_node(node_id)
        name = node.get_property('name') if node else f"Node_{node_id[:8]}"
        print(f"   - {name} (depth {depth})")
    
    # Path finding examples
    print("\n4. Path finding examples...")
    nodes_list = list(db.nodes.keys())
    if len(nodes_list) >= 3:
        node1 = nodes_list[0]
        node2 = nodes_list[len(nodes_list)//2]
        node3 = nodes_list[-1]
        
        for start, end in [(node1, node2), (node1, node3)]:
            path = db.shortest_path(start, end)
            if path:
                path_names = []
                for node_id in path:
                    node = db.get_node(node_id)
                    name = node.get_property('name') if node else f"Node_{node_id[:8]}"
                    path_names.append(name)
                print(f"   Path {' → '.join(path_names)} ({len(path) - 1} hops)")
            else:
                print(f"   No path found between nodes")


def demonstrate_relationship_analysis():
    """Demonstrate relationship analysis features"""
    print("\n" + "="*60)
    print("RELATIONSHIP ANALYSIS DEMONSTRATION")
    print("="*60)
    
    db = GraphDB()
    _create_complex_graph(db)
    
    # Graph statistics
    print("\n1. Graph statistics...")
    stats = db.analyze_relationship_patterns()
    print(f"   Total nodes: {stats['total_nodes']}")
    print(f"   Total edges: {stats['total_edges']}")
    print(f"   Graph density: {stats['density']:.4f}")
    print(f"   Average degree: {stats['average_degree']:.2f}")
    print(f"   Connected components: {stats['connected_components']}")
    print(f"   Node labels: {stats['labels']}")
    print(f"   Edge types: {stats['edge_types']}")
    
    # Degree distribution
    print("\n2. Degree distribution...")
    degree_dist = stats['degree_distribution']
    for degree in sorted(degree_dist.keys())[:10]:
        count = degree_dist[degree]
        print(f"   Nodes with degree {degree}: {count}")
    
    # Betweenness centrality
    print("\n3. Betweenness centrality (top nodes)...")
    centrality = db.calculate_betweenness_centrality()
    sorted_centrality = sorted(centrality.items(), key=lambda x: x[1], reverse=True)
    
    for node_id, score in sorted_centrality[:5]:
        node = db.get_node(node_id)
        name = node.get_property('name') if node else f"Node_{node_id[:8]}"
        print(f"   - {name}: {score:.4f}")
    
    # Connected components
    print("\n4. Connected components...")
    components = db.find_connected_components()
    for i, component in enumerate(components):
        if len(component) <= 10:  # Show details for small components
            print(f"   Component {i + 1} ({len(component)} nodes):")
            for node_id in component[:5]:  # Show first 5 nodes
                node = db.get_node(node_id)
                name = node.get_property('name') if node else f"Node_{node_id[:8]}"
                print(f"     - {name}")
            if len(component) > 5:
                print(f"     ... and {len(component) - 5} more")
        else:
            print(f"   Component {i + 1}: {len(component)} nodes")
    
    # Relationship patterns
    print("\n5. Relationship type analysis...")
    edge_types = list(stats['edge_types'])
    for edge_type in edge_types:
        edges = db.find_edges(edge_type)
        print(f"   {edge_type} relationships: {len(edges)}")


def demonstrate_complex_relationships():
    """Demonstrate complex relationship scenarios"""
    print("\n" + "="*60)
    print("COMPLEX RELATIONSHIP SCENARIOS")
    print("="*60)
    
    db = GraphDB()
    
    print("\n1. Creating company organizational structure...")
    
    # Create company
    company_id = db.create_node(['COMPANY'], {
        'name': 'TechCorp',
        'founded': 2020,
        'employees': 150,
        'revenue': 5000000
    })
    
    # Create departments
    dept_data = [
        {'name': 'Engineering', 'headcount': 60, 'budget': 2000000},
        {'name': 'Sales', 'headcount': 30, 'budget': 1500000},
        {'name': 'Marketing', 'headcount': 25, 'budget': 800000},
        {'name': 'HR', 'headcount': 15, 'budget': 500000},
        {'name': 'Finance', 'headcount': 20, 'budget': 600000}
    ]
    
    dept_ids = {}
    for dept in dept_data:
        dept_id = db.create_node(['DEPARTMENT'], dept)
        dept_ids[dept['name']] = dept_id
        db.create_edge(company_id, dept_id, 'HAS_DEPARTMENT')
        print(f"✓ Created department: {dept['name']}")
    
    print("\n2. Creating employee hierarchy...")
    
    # Create employees
    emp_data = [
        {'name': 'John CEO', 'title': 'CEO', 'salary': 200000, 'level': 10},
        {'name': 'Sarah CTO', 'title': 'CTO', 'salary': 180000, 'level': 9},
        {'name': 'Mike VP Eng', 'title': 'VP Engineering', 'salary': 160000, 'level': 8},
        {'name': 'Lisa Sales Dir', 'title': 'Sales Director', 'salary': 150000, 'level': 7},
        {'name': 'Tom Eng Mgr', 'title': 'Engineering Manager', 'salary': 140000, 'level': 6},
        {'name': 'Amy Dev', 'title': 'Senior Developer', 'salary': 120000, 'level': 5},
        {'name': 'Bob Dev', 'title': 'Developer', 'salary': 100000, 'level': 4}
    ]
    
    emp_ids = {}
    for emp in emp_data:
        emp_id = db.create_node(['EMPLOYEE'], emp)
        emp_ids[emp['name']] = emp_id
        print(f"✓ Created employee: {emp['name']}")
    
    # Create reporting relationships
    reporting_structure = [
        ('John CEO', 'Sarah CTO', 'MANAGES'),
        ('John CEO', 'Lisa Sales Dir', 'MANAGES'),
        ('Sarah CTO', 'Mike VP Eng', 'MANAGES'),
        ('Mike VP Eng', 'Tom Eng Mgr', 'MANAGES'),
        ('Tom Eng Mgr', 'Amy Dev', 'MANAGES'),
        ('Tom Eng Mgr', 'Bob Dev', 'MANAGES')
    ]
    
    for manager, employee, rel_type in reporting_structure:
        edge_id = db.create_edge(
            emp_ids[manager], 
            emp_ids[employee], 
            rel_type,
            {'effective_date': '2020-01-01'}
        )
        print(f"✓ {manager} → {employee} ({rel_type})")
    
    # Create department assignments
    dept_assignments = [
        ('Sarah CTO', 'Engineering'),
        ('Mike VP Eng', 'Engineering'),
        ('Tom Eng Mgr', 'Engineering'),
        ('Amy Dev', 'Engineering'),
        ('Bob Dev', 'Engineering'),
        ('Lisa Sales Dir', 'Sales')
    ]
    
    for emp_name, dept_name in dept_assignments:
        edge_id = db.create_edge(
            emp_ids[emp_name], 
            dept_ids[dept_name], 
            'BELONGS_TO'
        )
        print(f"✓ {emp_name} → {dept_name} (belongs to)")
    
    print("\n3. Organizational queries...")
    
    # Find all employees in Engineering department
    engineering_employees = db.traverse_bfs(
        dept_ids['Engineering'], 
        max_depth=2, 
        edge_type='BELONGS_TO'
    )
    print(f"\n✓ Employees in Engineering:")
    for node_id, depth in engineering_employees:
        if depth == 2:  # Skip the department itself
            node = db.get_node(node_id)
            if node and 'EMPLOYEE' in node.labels:
                name = node.get_property('name')
                title = node.get_property('title')
                print(f"   - {name} ({title})")
    
    # Find reporting chain
    print(f"\n✓ Reporting chain (CEO down):")
    ceo_reports = db.traverse_bfs(emp_ids['John CEO'], max_depth=4, edge_type='MANAGES')
    for node_id, depth in ceo_reports:
        if depth > 0:
            node = db.get_node(node_id)
            if node and 'EMPLOYEE' in node.labels:
                name = node.get_property('name')
                title = node.get_property('title')
                indent = "  " * (depth - 1)
                print(f"   {indent}- {name} ({title})")
    
    # Find cross-department relationships
    print(f"\n4. Cross-department analysis...")
    all_depts = list(dept_ids.keys())
    for i, dept1 in enumerate(all_depts[:3]):  # Check first 3 departments
        for dept2 in all_depts[i+1:i+3]:
            # Find employees who might bridge departments
            dept1_emps = db.traverse_bfs(dept_ids[dept1], max_depth=1, edge_type='BELONGS_TO')
            dept2_emps = db.traverse_bfs(dept_ids[dept2], max_depth=1, edge_type='BELONGS_TO')
            
            # Find common direct reports
            dept1_reports = {node_id for node_id, _ in dept1_emps}
            dept2_reports = {node_id for node_id, _ in dept2_emps}
            common_reports = dept1_reports.intersection(dept2_reports)
            
            if common_reports:
                print(f"   {dept1} and {dept2} share management structure")


def _create_complex_graph(db):
    """Create a complex graph for demonstration"""
    # Create nodes with different labels
    for i in range(15):
        node_id = db.create_node(['PERSON'], {
            'name': f'Person_{i:02d}',
            'age': 20 + i,
            'city': random.choice(['New York', 'Los Angeles', 'Chicago', 'Seattle']),
            'interests': random.sample(['Music', 'Sports', 'Movies', 'Books', 'Travel'], 2)
        })
    
    # Create various types of relationships
    node_ids = list(db.nodes.keys())
    
    # Friend relationships
    for _ in range(20):
        node1, node2 = random.sample(node_ids, 2)
        db.create_edge(node1, node2, 'FRIEND', {
            'since': '2020-01-01',
            'strength': random.uniform(0.5, 1.0)
        })
    
    # Work relationships
    for _ in range(10):
        node1, node2 = random.sample(node_ids, 2)
        db.create_edge(node1, node2, 'COLLEAGUE', {
            'project': random.choice(['Project_A', 'Project_B', 'Project_C']),
            'since': '2021-01-01'
        })
    
    # Family relationships
    family_pairs = [('PERSON_00', 'PERSON_01'), ('PERSON_02', 'PERSON_03'), 
                   ('PERSON_04', 'PERSON_05')]
    
    # Convert to node IDs (simplified - in real usage, would query by name)
    for i in range(min(len(family_pairs), len(node_ids))):
        node1 = node_ids[i]
        node2 = node_ids[i + 5] if i + 5 < len(node_ids) else node_ids[0]
        db.create_edge(node1, node2, 'SIBLING', {
            'shared_parents': True
        })


def main():
    """Main demonstration function"""
    print("GRAPH DATABASE EDUCATIONAL DEMO")
    print("Demonstrating Social Networks, Graph Traversal, and Relationship Analysis")
    print("="*80)
    
    try:
        demonstrate_social_network()
        demonstrate_graph_traversal()
        demonstrate_relationship_analysis()
        demonstrate_complex_relationships()
        
        print("\n" + "="*80)
        print("GRAPH DATABASE DEMO COMPLETED")
        print("="*80)
        print("\nKey Concepts Demonstrated:")
        print("✓ Graph data structure and representation")
        print("✓ Social network analysis and metrics")
        print("✓ Graph traversal algorithms (BFS, DFS)")
        print("✓ Path finding and shortest path algorithms")
        print("✓ Relationship modeling and queries")
        print("✓ Community detection and clustering")
        print("✓ Centrality measures and network analysis")
        print("✓ Complex relationship hierarchies")
        print("\nThis educational database provides hands-on learning of:")
        print("- NoSQL graph databases vs relational databases")
        print("- Network theory and graph algorithms")
        print("- Social network analysis techniques")
        print("- Relationship modeling for complex data")
        print("- Graph traversal and path finding")
        print("- Network metrics and centrality measures")
        
    except Exception as e:
        print(f"Error during demonstration: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    main()