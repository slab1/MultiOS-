"""
Educational Graph Database
Implements graph-based storage for social networks and relationship analysis
"""

import uuid
from typing import Dict, List, Any, Optional, Set, Tuple
from dataclasses import dataclass, asdict
from collections import defaultdict, deque
import time
import json
import heapq


@dataclass
class Property:
    """Represents a property in a node or edge"""
    key: str
    value: Any
    data_type: str = 'string'  # string, number, boolean, date, array
    
    def to_dict(self) -> Dict[str, Any]:
        return {'key': self.key, 'value': self.value, 'data_type': self.data_type}


@dataclass
class Node:
    """Represents a node in the graph"""
    node_id: str
    labels: List[str]  # Multiple labels support (Person, Student, etc.)
    properties: List[Property]
    
    def __post_init__(self):
        # Convert properties dict to Property objects if needed
        if self.properties and isinstance(self.properties[0], dict):
            self.properties = [Property(**prop) for prop in self.properties]
    
    def get_property(self, key: str) -> Any:
        """Get property value by key"""
        for prop in self.properties:
            if prop.key == key:
                return prop.value
        return None
    
    def set_property(self, key: str, value: Any, data_type: str = 'string'):
        """Set or update property"""
        for prop in self.properties:
            if prop.key == key:
                prop.value = value
                prop.data_type = data_type
                return
        
        # Add new property
        self.properties.append(Property(key, value, data_type))
    
    def has_label(self, label: str) -> bool:
        """Check if node has specific label"""
        return label in self.labels
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            'node_id': self.node_id,
            'labels': self.labels,
            'properties': [prop.to_dict() for prop in self.properties]
        }


@dataclass
class Edge:
    """Represents an edge in the graph"""
    edge_id: str
    source_id: str
    target_id: str
    edge_type: str  # Relationship type (FRIEND, FOLLOWS, etc.)
    properties: List[Property]
    
    def __post_init__(self):
        if self.properties and isinstance(self.properties[0], dict):
            self.properties = [Property(**prop) for prop in self.properties]
    
    def get_property(self, key: str) -> Any:
        """Get property value by key"""
        for prop in self.properties:
            if prop.key == key:
                return prop.value
        return None
    
    def set_property(self, key: str, value: Any, data_type: str = 'string'):
        """Set or update property"""
        for prop in self.properties:
            if prop.key == key:
                prop.value = value
                prop.data_type = data_type
                return
        
        self.properties.append(Property(key, value, data_type))
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            'edge_id': self.edge_id,
            'source_id': self.source_id,
            'target_id': self.target_id,
            'edge_type': self.edge_type,
            'properties': [prop.to_dict() for prop in self.properties]
        }


class GraphIndex:
    """Manages indexes for efficient graph traversal"""
    
    def __init__(self):
        self.node_index = defaultdict(set)  # label -> set of node_ids
        self.property_index = defaultdict(dict)  # property_key -> {value -> set of node_ids}
        self.edge_type_index = defaultdict(set)  # edge_type -> set of edge_ids
        self.adjacency_index = defaultdict(set)  # node_id -> set of connected edge_ids
    
    def add_node(self, node: Node):
        """Add node to index"""
        for label in node.labels:
            self.node_index[label].add(node.node_id)
        
        for prop in node.properties:
            if prop.key not in self.property_index:
                self.property_index[prop.key] = {}
            if prop.value not in self.property_index[prop.key]:
                self.property_index[prop.key][prop.value] = set()
            self.property_index[prop.key][prop.value].add(node.node_id)
    
    def remove_node(self, node_id: str, node: Node):
        """Remove node from index"""
        for label in node.labels:
            self.node_index[label].discard(node_id)
        
        for prop in node.properties:
            if prop.key in self.property_index:
                if prop.value in self.property_index[prop.key]:
                    self.property_index[prop.key][prop.value].discard(node_id)
                    if not self.property_index[prop.key][prop.value]:
                        del self.property_index[prop.key][prop.value]
    
    def add_edge(self, edge: Edge):
        """Add edge to index"""
        self.edge_type_index[edge.edge_type].add(edge.edge_id)
        self.adjacency_index[edge.source_id].add(edge.edge_id)
        self.adjacency_index[edge.target_id].add(edge.edge_id)
    
    def remove_edge(self, edge_id: str, edge: Edge):
        """Remove edge from index"""
        self.edge_type_index[edge.edge_type].discard(edge_id)
        self.adjacency_index[edge.source_id].discard(edge_id)
        self.adjacency_index[edge.target_id].discard(edge_id)
    
    def find_nodes_by_label(self, label: str) -> Set[str]:
        """Find nodes by label using index"""
        return self.node_index.get(label, set()).copy()
    
    def find_nodes_by_property(self, prop_key: str, prop_value: Any) -> Set[str]:
        """Find nodes by property value using index"""
        return self.property_index.get(prop_key, {}).get(prop_value, set()).copy()


class GraphDB:
    """
    Educational Graph Database
    Implements graph storage with traversal algorithms for social networks
    """
    
    def __init__(self):
        self.nodes = {}  # node_id -> Node
        self.edges = {}  # edge_id -> Edge
        self.node_edges = defaultdict(list)  # node_id -> list of connected edges
        self.index = GraphIndex()
        self.graph_stats = {
            'nodes': 0,
            'edges': 0,
            'labels': set(),
            'edge_types': set()
        }
    
    def create_node(self, labels: List[str], properties: Dict[str, Any] = None) -> str:
        """Create a new node"""
        node_id = str(uuid.uuid4())
        
        # Convert properties dict to Property objects
        prop_list = []
        if properties:
            for key, value in properties.items():
                data_type = self._detect_data_type(value)
                prop_list.append(Property(key, value, data_type))
        
        node = Node(node_id=node_id, labels=labels, properties=prop_list)
        self.nodes[node_id] = node
        self.index.add_node(node)
        
        # Update statistics
        self.graph_stats['nodes'] += 1
        self.graph_stats['labels'].update(labels)
        
        return node_id
    
    def _detect_data_type(self, value: Any) -> str:
        """Detect data type for properties"""
        if isinstance(value, bool):
            return 'boolean'
        elif isinstance(value, int):
            return 'integer'
        elif isinstance(value, float):
            return 'float'
        elif isinstance(value, str):
            return 'string'
        elif isinstance(value, list):
            return 'array'
        elif isinstance(value, dict):
            return 'object'
        else:
            return 'string'
    
    def get_node(self, node_id: str) -> Optional[Node]:
        """Get node by ID"""
        return self.nodes.get(node_id)
    
    def find_nodes(self, label: str = None, properties: Dict[str, Any] = None) -> List[Node]:
        """Find nodes by label and/or properties"""
        result_set = set()
        
        # Start with label filter if specified
        if label:
            result_set = self.index.find_nodes_by_label(label)
        else:
            result_set = set(self.nodes.keys())
        
        # Apply property filters
        if properties:
            for prop_key, prop_value in properties.items():
                matching_nodes = self.index.find_nodes_by_property(prop_key, prop_value)
                result_set = result_set.intersection(matching_nodes)
        
        # Return Node objects
        return [self.nodes[node_id] for node_id in result_set]
    
    def update_node(self, node_id: str, labels: List[str] = None, 
                   properties: Dict[str, Any] = None) -> bool:
        """Update node labels and/or properties"""
        if node_id not in self.nodes:
            return False
        
        node = self.nodes[node_id]
        
        # Update labels
        if labels is not None:
            # Remove old label from index
            old_labels = set(node.labels)
            for old_label in old_labels:
                self.index.remove_node(node_id, node)
            
            node.labels = labels
            
            # Add new labels to index
            self.index.add_node(node)
            
            # Update statistics
            self.graph_stats['labels'] = set()
            for n in self.nodes.values():
                self.graph_stats['labels'].update(n.labels)
        
        # Update properties
        if properties:
            for key, value in properties.items():
                node.set_property(key, value)
        
        return True
    
    def delete_node(self, node_id: str) -> bool:
        """Delete a node and all connected edges"""
        if node_id not in self.nodes:
            return False
        
        node = self.nodes[node_id]
        
        # Delete connected edges
        edges_to_delete = self.node_edges[node_id].copy()
        for edge_id in edges_to_delete:
            self.delete_edge(edge_id)
        
        # Remove from index
        self.index.remove_node(node_id, node)
        
        # Remove from graph
        del self.nodes[node_id]
        del self.node_edges[node_id]
        
        # Update statistics
        self.graph_stats['nodes'] -= 1
        
        return True
    
    def create_edge(self, source_id: str, target_id: str, edge_type: str, 
                   properties: Dict[str, Any] = None) -> str:
        """Create an edge between two nodes"""
        # Validate nodes exist
        if source_id not in self.nodes or target_id not in self.nodes:
            return None
        
        edge_id = str(uuid.uuid4())
        
        # Convert properties dict to Property objects
        prop_list = []
        if properties:
            for key, value in properties.items():
                data_type = self._detect_data_type(value)
                prop_list.append(Property(key, value, data_type))
        
        edge = Edge(edge_id=edge_id, source_id=source_id, target_id=target_id,
                   edge_type=edge_type, properties=prop_list)
        
        self.edges[edge_id] = edge
        self.node_edges[source_id].append(edge)
        self.node_edges[target_id].append(edge)
        
        # Add to index
        self.index.add_edge(edge)
        
        # Update statistics
        self.graph_stats['edges'] += 1
        self.graph_stats['edge_types'].add(edge_type)
        
        return edge_id
    
    def get_edge(self, edge_id: str) -> Optional[Edge]:
        """Get edge by ID"""
        return self.edges.get(edge_id)
    
    def find_edges(self, edge_type: str = None, source_id: str = None, 
                  target_id: str = None) -> List[Edge]:
        """Find edges by criteria"""
        result_edges = []
        
        for edge in self.edges.values():
            # Apply filters
            if edge_type and edge.edge_type != edge_type:
                continue
            if source_id and edge.source_id != source_id:
                continue
            if target_id and edge.target_id != target_id:
                continue
            
            result_edges.append(edge)
        
        return result_edges
    
    def update_edge(self, edge_id: str, edge_type: str = None, 
                   properties: Dict[str, Any] = None) -> bool:
        """Update edge type and/or properties"""
        if edge_id not in self.edges:
            return False
        
        edge = self.edges[edge_id]
        
        if edge_type:
            # Remove from old edge type index
            self.index.remove_edge(edge_id, edge)
            
            edge.edge_type = edge_type
            
            # Add to new edge type index
            self.index.add_edge(edge)
            
            # Update statistics
            self.graph_stats['edge_types'] = set()
            for e in self.edges.values():
                self.graph_stats['edge_types'].add(e.edge_type)
        
        if properties:
            for key, value in properties.items():
                edge.set_property(key, value)
        
        return True
    
    def delete_edge(self, edge_id: str) -> bool:
        """Delete an edge"""
        if edge_id not in self.edges:
            return False
        
        edge = self.edges[edge_id]
        
        # Remove from adjacency lists
        self.node_edges[edge.source_id] = [
            e for e in self.node_edges[edge.source_id] if e.edge_id != edge_id
        ]
        self.node_edges[edge.target_id] = [
            e for e in self.node_edges[edge.target_id] if e.edge_id != edge_id
        ]
        
        # Remove from index
        self.index.remove_edge(edge_id, edge)
        
        # Remove from edges
        del self.edges[edge_id]
        
        # Update statistics
        self.graph_stats['edges'] -= 1
        
        return True
    
    def traverse_bfs(self, start_node_id: str, max_depth: int = 1, 
                    edge_type: str = None) -> List[Tuple[str, int]]:
        """Breadth-first search traversal"""
        if start_node_id not in self.nodes:
            return []
        
        visited = set()
        queue = deque([(start_node_id, 0)])
        result = []
        
        while queue:
            node_id, depth = queue.popleft()
            
            if node_id in visited or depth > max_depth:
                continue
            
            visited.add(node_id)
            result.append((node_id, depth))
            
            # Explore neighbors
            for edge in self.node_edges[node_id]:
                # Filter by edge type if specified
                if edge_type and edge.edge_type != edge_type:
                    continue
                
                # Find neighbor
                neighbor_id = edge.target_id if edge.source_id == node_id else edge.source_id
                
                if neighbor_id not in visited:
                    queue.append((neighbor_id, depth + 1))
        
        return result
    
    def traverse_dfs(self, start_node_id: str, max_depth: int = 1, 
                    edge_type: str = None) -> List[Tuple[str, int]]:
        """Depth-first search traversal"""
        if start_node_id not in self.nodes:
            return []
        
        visited = set()
        result = []
        
        def dfs(node_id: str, depth: int):
            if node_id in visited or depth > max_depth:
                return
            
            visited.add(node_id)
            result.append((node_id, depth))
            
            # Explore neighbors
            for edge in self.node_edges[node_id]:
                if edge_type and edge.edge_type != edge_type:
                    continue
                
                neighbor_id = edge.target_id if edge.source_id == node_id else edge.source_id
                dfs(neighbor_id, depth + 1)
        
        dfs(start_node_id, 0)
        return result
    
    def shortest_path(self, start_node_id: str, end_node_id: str, 
                     edge_type: str = None) -> Optional[List[str]]:
        """Find shortest path between two nodes using BFS"""
        if start_node_id not in self.nodes or end_node_id not in self.nodes:
            return None
        
        if start_node_id == end_node_id:
            return [start_node_id]
        
        queue = deque([(start_node_id, [start_node_id])])
        visited = set([start_node_id])
        
        while queue:
            node_id, path = queue.popleft()
            
            for edge in self.node_edges[node_id]:
                if edge_type and edge.edge_type != edge_type:
                    continue
                
                neighbor_id = edge.target_id if edge.source_id == node_id else edge.source_id
                
                if neighbor_id == end_node_id:
                    return path + [neighbor_id]
                
                if neighbor_id not in visited:
                    visited.add(neighbor_id)
                    queue.append((neighbor_id, path + [neighbor_id]))
        
        return None
    
    def find_common_neighbors(self, node1_id: str, node2_id: str) -> List[str]:
        """Find common neighbors of two nodes"""
        if node1_id not in self.nodes or node2_id not in self.nodes:
            return []
        
        neighbors1 = set()
        for edge in self.node_edges[node1_id]:
            neighbor_id = edge.target_id if edge.source_id == node1_id else edge.source_id
            neighbors1.add(neighbor_id)
        
        neighbors2 = set()
        for edge in self.node_edges[node2_id]:
            neighbor_id = edge.target_id if edge.source_id == node2_id else edge.source_id
            neighbors2.add(neighbor_id)
        
        return list(neighbors1.intersection(neighbors2))
    
    def calculate_betweenness_centrality(self) -> Dict[str, float]:
        """Calculate betweenness centrality for all nodes"""
        centrality = {node_id: 0.0 for node_id in self.nodes}
        nodes = list(self.nodes.keys())
        
        for i, s in enumerate(nodes):
            for t in nodes[i+1:]:
                # Find shortest paths between all pairs
                for start in [s, t]:
                    paths = self._find_all_shortest_paths(start, t)
                    for path in paths:
                        # Increment centrality for nodes in path (excluding endpoints)
                        for node in path[1:-1]:
                            centrality[node] += 1 / len(paths)
        
        # Normalize
        n = len(nodes)
        if n > 2:
            for node_id in centrality:
                centrality[node_id] /= (n - 1) * (n - 2)
        
        return centrality
    
    def _find_all_shortest_paths(self, start: str, end: str) -> List[List[str]]:
        """Find all shortest paths between two nodes"""
        if start == end:
            return [[start]]
        
        queue = deque([(start, [start])])
        visited = {start}
        shortest_distance = None
        paths = []
        
        while queue:
            node, path = queue.popleft()
            
            if shortest_distance is not None and len(path) > shortest_distance:
                continue
            
            for edge in self.node_edges[node]:
                neighbor_id = edge.target_id if edge.source_id == node else edge.source_id
                
                if neighbor_id in path:  # Avoid cycles
                    continue
                
                new_path = path + [neighbor_id]
                
                if neighbor_id == end:
                    if shortest_distance is None:
                        shortest_distance = len(new_path)
                    paths.append(new_path)
                elif neighbor_id not in visited:
                    visited.add(neighbor_id)
                    queue.append((neighbor_id, new_path))
        
        return paths
    
    def get_neighbors(self, node_id: str, edge_type: str = None) -> List[str]:
        """Get neighbors of a node"""
        if node_id not in self.nodes:
            return []
        
        neighbors = []
        for edge in self.node_edges[node_id]:
            if edge_type and edge.edge_type != edge_type:
                continue
            
            neighbor_id = edge.target_id if edge.source_id == node_id else edge.source_id
            neighbors.append(neighbor_id)
        
        return neighbors
    
    def get_node_degree(self, node_id: str, edge_type: str = None) -> int:
        """Get degree of a node"""
        return len(self.get_neighbors(node_id, edge_type))
    
    def find_connected_components(self) -> List[List[str]]:
        """Find all connected components in the graph"""
        visited = set()
        components = []
        
        for node_id in self.nodes:
            if node_id in visited:
                continue
            
            # BFS to find connected component
            component = []
            queue = deque([node_id])
            visited.add(node_id)
            
            while queue:
                current = queue.popleft()
                component.append(current)
                
                for neighbor in self.get_neighbors(current):
                    if neighbor not in visited:
                        visited.add(neighbor)
                        queue.append(neighbor)
            
            components.append(component)
        
        return components
    
    def detect_communities(self, max_communities: int = 5) -> Dict[str, int]:
        """Simple community detection using connected components"""
        components = self.find_connected_components()
        community_map = {}
        
        for i, component in enumerate(components[:max_communities]):
            for node_id in component:
                community_map[node_id] = i
        
        return community_map
    
    def analyze_relationship_patterns(self) -> Dict[str, Any]:
        """Analyze relationship patterns in the graph"""
        analysis = {
            'total_nodes': len(self.nodes),
            'total_edges': len(self.edges),
            'density': self._calculate_graph_density(),
            'average_degree': self._calculate_average_degree(),
            'connected_components': len(self.find_connected_components()),
            'edge_types': list(self.graph_stats['edge_types']),
            'labels': list(self.graph_stats['labels']),
            'degree_distribution': self._get_degree_distribution()
        }
        
        # Social network specific analysis
        if 'PERSON' in self.graph_stats['labels'] and 'FRIEND' in self.graph_stats['edge_types']:
            analysis['social_network_analysis'] = self._analyze_social_network()
        
        return analysis
    
    def _calculate_graph_density(self) -> float:
        """Calculate graph density"""
        n = len(self.nodes)
        if n <= 1:
            return 0.0
        return (2 * len(self.edges)) / (n * (n - 1))
    
    def _calculate_average_degree(self) -> float:
        """Calculate average node degree"""
        if not self.nodes:
            return 0.0
        total_degree = sum(self.get_node_degree(node_id) for node_id in self.nodes)
        return total_degree / len(self.nodes)
    
    def _get_degree_distribution(self) -> Dict[int, int]:
        """Get degree distribution"""
        distribution = defaultdict(int)
        for node_id in self.nodes:
            degree = self.get_node_degree(node_id)
            distribution[degree] += 1
        return dict(distribution)
    
    def _analyze_social_network(self) -> Dict[str, Any]:
        """Analyze social network specific metrics"""
        # Get all people
        people = self.find_nodes('PERSON')
        people_ids = {person.node_id for person in people}
        
        # Calculate friendship metrics
        friendships = self.find_edges('FRIEND')
        total_friendships = len(friendships)
        
        # Average number of friends per person
        friend_counts = []
        for person in people:
            friends = self.get_neighbors(person.node_id, 'FRIEND')
            friend_counts.append(len(friends))
        
        avg_friends = sum(friend_counts) / len(friend_counts) if friend_counts else 0
        
        # Friend recommendations (friends of friends who aren't friends)
        recommendations = {}
        for person in people:
            person_friends = set(self.get_neighbors(person.node_id, 'FRIEND'))
            recommendations[person.node_id] = []
            
            for friend in person_friends:
                friends_of_friends = set(self.get_neighbors(friend, 'FRIEND'))
                potential_friends = friends_of_friends - person_friends - {person.node_id}
                recommendations[person.node_id].extend(list(potential_friends))
        
        return {
            'total_people': len(people),
            'total_friendships': total_friendships,
            'average_friends_per_person': avg_friends,
            'friend_recommendations': recommendations
        }
    
    def export_graph(self) -> str:
        """Export graph to JSON format"""
        graph_data = {
            'nodes': [node.to_dict() for node in self.nodes.values()],
            'edges': [edge.to_dict() for edge in self.edges.values()],
            'statistics': {
                'total_nodes': len(self.nodes),
                'total_edges': len(self.edges),
                'labels': list(self.graph_stats['labels']),
                'edge_types': list(self.graph_stats['edge_types'])
            }
        }
        return json.dumps(graph_data, indent=2)
    
    def import_graph(self, json_data: str) -> bool:
        """Import graph from JSON format"""
        try:
            graph_data = json.loads(json_data)
            
            # Clear existing data
            self.nodes.clear()
            self.edges.clear()
            self.node_edges.clear()
            self.index = GraphIndex()
            self.graph_stats = {
                'nodes': 0,
                'edges': 0,
                'labels': set(),
                'edge_types': set()
            }
            
            # Import nodes
            for node_data in graph_data.get('nodes', []):
                node = Node(**node_data)
                self.nodes[node.node_id] = node
                self.index.add_node(node)
                self.graph_stats['nodes'] += 1
                self.graph_stats['labels'].update(node.labels)
            
            # Import edges
            for edge_data in graph_data.get('edges', []):
                edge = Edge(**edge_data)
                self.edges[edge.edge_id] = edge
                self.node_edges[edge.source_id].append(edge)
                self.node_edges[edge.target_id].append(edge)
                self.index.add_edge(edge)
                self.graph_stats['edges'] += 1
                self.graph_stats['edge_types'].add(edge.edge_type)
            
            return True
        except Exception:
            return False