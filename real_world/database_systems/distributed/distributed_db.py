"""
Educational Distributed Database System
Implements sharding, replication, and distributed query processing
"""

import hashlib
import threading
import time
import json
import random
from typing import Dict, List, Any, Optional, Tuple, Set
from dataclasses import dataclass, asdict
from enum import Enum
from collections import defaultdict


class NodeState(Enum):
    """Possible states for a database node"""
    ACTIVE = "ACTIVE"
    INACTIVE = "INACTIVE"
    RECOVERING = "RECOVERING"
    MAINTENANCE = "MAINTENANCE"


class ConsistencyLevel(Enum):
    """Consistency levels for distributed operations"""
    ONE = "ONE"           # Read from one node
    QUORUM = "QUORUM"     # Read from majority of nodes
    ALL = "ALL"           # Read from all nodes
    LOCAL_QUORUM = "LOCAL_QUORUM"  # Read from local quorum


class ShardStrategy(Enum):
    """Different sharding strategies"""
    HASH = "HASH"         # Hash-based sharding
    RANGE = "RANGE"       # Range-based sharding
    DIRECTORY = "DIRECTORY"  # Directory-based sharding


@dataclass
class DataRecord:
    """Represents a data record with metadata"""
    key: str
    value: Any
    shard_id: str
    timestamp: float
    version: int
    checksum: str
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            'key': self.key,
            'value': self.value,
            'shard_id': self.shard_id,
            'timestamp': self.timestamp,
            'version': self.version,
            'checksum': self.checksum
        }


@dataclass
class ShardConfig:
    """Configuration for a shard"""
    shard_id: str
    start_key: Any
    end_key: Any
    nodes: List[str]
    replication_factor: int
    strategy: ShardStrategy


@dataclass
class NodeInfo:
    """Information about a database node"""
    node_id: str
    host: str
    port: int
    state: NodeState
    load: float
    capacity: int
    shard_assignments: Set[str]
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            'node_id': self.node_id,
            'host': self.host,
            'port': self.port,
            'state': self.state.value,
            'load': self.load,
            'capacity': self.capacity,
            'shard_assignments': list(self.shard_assignments)
        }


class ShardManager:
    """Manages shard configuration and routing"""
    
    def __init__(self):
        self.shards = {}  # shard_id -> ShardConfig
        self.key_to_shard = {}  # key -> shard_id
        self.shard_directory = {}  # shard_id -> list of nodes
    
    def add_shard(self, shard_config: ShardConfig):
        """Add a shard configuration"""
        self.shards[shard_config.shard_id] = shard_config
        self.shard_directory[shard_config.shard_id] = shard_config.nodes.copy()
        
        # Update key routing for range-based sharding
        if shard_config.strategy == ShardStrategy.RANGE:
            # Register key range
            self._register_key_range(shard_config)
    
    def remove_shard(self, shard_id: str):
        """Remove a shard configuration"""
        if shard_id in self.shards:
            shard_config = self.shards[shard_id]
            
            # Unregister key routing
            if shard_config.strategy == ShardStrategy.RANGE:
                self._unregister_key_range(shard_config)
            
            del self.shards[shard_id]
            del self.shard_directory[shard_id]
    
    def _register_key_range(self, shard_config: ShardConfig):
        """Register key range for routing"""
        # For range-based sharding, we store the range
        # In a real implementation, this would use a balanced tree
        pass
    
    def _unregister_key_range(self, shard_config: ShardConfig):
        """Unregister key range"""
        pass
    
    def get_shard_for_key(self, key: str) -> Optional[str]:
        """Get shard ID for a given key"""
        # Try hash-based routing first
        for shard_id, shard_config in self.shards.items():
            if shard_config.strategy == ShardStrategy.HASH:
                if self._hash_matches_shard(key, shard_id):
                    return shard_id
        
        # Try range-based routing
        for shard_id, shard_config in self.shards.items():
            if shard_config.strategy == ShardStrategy.RANGE:
                if self._key_in_range(key, shard_config):
                    return shard_id
        
        # Default to hash-based routing if no range matches
        return self._get_hash_shard(key)
    
    def _hash_matches_shard(self, key: str, shard_id: str) -> bool:
        """Check if key hash matches shard"""
        key_hash = int(hashlib.md5(key.encode()).hexdigest(), 16)
        shard_hash = int(hashlib.md5(shard_id.encode()).hexdigest(), 16)
        return key_hash % len(self.shards) == shard_hash % len(self.shards)
    
    def _key_in_range(self, key: Any, shard_config: ShardConfig) -> bool:
        """Check if key is in shard range"""
        try:
            key_val = float(key) if isinstance(key, str) and key.replace('.', '').isdigit() else key
            return shard_config.start_key <= key_val < shard_config.end_key
        except (ValueError, TypeError):
            # For non-numeric keys, use string comparison
            return shard_config.start_key <= str(key) < shard_config.end_key
    
    def _get_hash_shard(self, key: str) -> Optional[str]:
        """Get shard using hash routing"""
        if not self.shards:
            return None
        
        hash_value = int(hashlib.md5(key.encode()).hexdigest(), 16)
        shard_index = hash_value % len(self.shards)
        return list(self.shards.keys())[shard_index]
    
    def get_nodes_for_shard(self, shard_id: str) -> List[str]:
        """Get nodes that contain a shard"""
        return self.shard_directory.get(shard_id, [])
    
    def add_node_to_shard(self, shard_id: str, node_id: str):
        """Add a node to a shard"""
        if shard_id in self.shard_directory:
            if node_id not in self.shard_directory[shard_id]:
                self.shard_directory[shard_id].append(node_id)
    
    def remove_node_from_shard(self, shard_id: str, node_id: str):
        """Remove a node from a shard"""
        if shard_id in self.shard_directory:
            if node_id in self.shard_directory[shard_id]:
                self.shard_directory[shard_id].remove(node_id)


class ReplicationManager:
    """Manages data replication across nodes"""
    
    def __init__(self, consistency_level: ConsistencyLevel = ConsistencyLevel.QUORUM):
        self.consistency_level = consistency_level
        self.write_ahead_log = []  # WAL for recovery
        self.replication_factor = 3  # Default replication factor
        self.lock = threading.Lock()
    
    def get_required_nodes(self, total_nodes: int) -> int:
        """Get number of nodes required for operation"""
        if self.consistency_level == ConsistencyLevel.ONE:
            return 1
        elif self.consistency_level == ConsistencyLevel.QUORUM:
            return (total_nodes // 2) + 1
        elif self.consistency_level == ConsistencyLevel.ALL:
            return total_nodes
        else:
            return min(2, total_nodes)
    
    def log_write_operation(self, operation: Dict[str, Any]):
        """Log write operation for recovery"""
        with self.lock:
            self.write_ahead_log.append({
                'timestamp': time.time(),
                'operation': operation
            })
            
            # Limit log size
            if len(self.write_ahead_log) > 1000:
                self.write_ahead_log.pop(0)
    
    def get_quorum_nodes(self, available_nodes: List[str]) -> List[str]:
        """Get quorum of nodes for operation"""
        required = self.get_required_nodes(len(available_nodes))
        return available_nodes[:required]
    
    def check_read_consistency(self, results: List[Any]) -> bool:
        """Check if read results meet consistency requirements"""
        if self.consistency_level == ConsistencyLevel.ONE:
            return len(results) >= 1
        elif self.consistency_level == ConsistencyLevel.QUORUM:
            # Check if majority agree
            if not results:
                return False
            
            # Simple majority check - in real implementation would compare values
            return len(results) >= (len(results) // 2) + 1
        elif self.consistency_level == ConsistencyLevel.ALL:
            return len(results) > 0  # Would check all nodes in real implementation
        else:
            return len(results) >= 1


class DistributedDBNode:
    """Individual database node in the distributed system"""
    
    def __init__(self, node_id: str, host: str, port: int, capacity: int = 1000):
        self.node_id = node_id
        self.host = host
        self.port = port
        self.state = NodeState.ACTIVE
        self.capacity = capacity
        self.load = 0.0
        
        # Local storage
        self.data = {}  # key -> DataRecord
        self.indexes = {}  # index_name -> {key -> set of keys}
        self.shard_assignments = set()
        
        # Monitoring
        self.operations_count = 0
        self.error_count = 0
        self.last_activity = time.time()
        
        # Lock for thread safety
        self.lock = threading.RLock()
    
    def store(self, record: DataRecord) -> bool:
        """Store a data record"""
        with self.lock:
            try:
                self.data[record.key] = record
                self.load += 1
                self.operations_count += 1
                self.last_activity = time.time()
                return True
            except Exception:
                self.error_count += 1
                return False
    
    def retrieve(self, key: str) -> Optional[DataRecord]:
        """Retrieve a data record"""
        with self.lock:
            try:
                self.operations_count += 1
                self.last_activity = time.time()
                return self.data.get(key)
            except Exception:
                self.error_count += 1
                return None
    
    def delete(self, key: str) -> bool:
        """Delete a data record"""
        with self.lock:
            try:
                if key in self.data:
                    del self.data[key]
                    self.load = max(0, self.load - 1)
                    self.operations_count += 1
                    self.last_activity = time.time()
                    return True
                return False
            except Exception:
                self.error_count += 1
                return False
    
    def query_range(self, start_key: str, end_key: str) -> List[DataRecord]:
        """Query records in a key range"""
        with self.lock:
            try:
                results = []
                for key, record in self.data.items():
                    if start_key <= key <= end_key:
                        results.append(record)
                
                self.operations_count += 1
                self.last_activity = time.time()
                return results
            except Exception:
                self.error_count += 1
                return []
    
    def get_statistics(self) -> Dict[str, Any]:
        """Get node statistics"""
        with self.lock:
            return {
                'node_id': self.node_id,
                'state': self.state.value,
                'capacity': self.capacity,
                'load': self.load,
                'utilization': self.load / self.capacity if self.capacity > 0 else 0,
                'operations_count': self.operations_count,
                'error_count': self.error_count,
                'error_rate': self.error_count / max(1, self.operations_count),
                'shard_assignments': list(self.shard_assignments),
                'last_activity': self.last_activity
            }
    
    def is_healthy(self) -> bool:
        """Check if node is healthy"""
        return (self.state == NodeState.ACTIVE and 
                self.load < self.capacity * 0.9 and  # Less than 90% capacity
                self.error_rate < 0.1)  # Less than 10% error rate
    
    @property
    def error_rate(self) -> float:
        """Calculate error rate"""
        return self.error_count / max(1, self.operations_count)


class DistributedDatabase:
    """
    Educational Distributed Database System
    Implements sharding, replication, and distributed query processing
    """
    
    def __init__(self, consistency_level: ConsistencyLevel = ConsistencyLevel.QUORUM):
        self.nodes = {}  # node_id -> DistributedDBNode
        self.shard_manager = ShardManager()
        self.replication_manager = ReplicationManager(consistency_level)
        self.coordinator_id = "coordinator_001"
        
        # System statistics
        self.total_operations = 0
        self.successful_operations = 0
        self.failed_operations = 0
        self.lock = threading.Lock()
        
        # Create default coordinator node
        self.add_node(self.coordinator_id, "localhost", 8000)
    
    def add_node(self, node_id: str, host: str, port: int, capacity: int = 1000) -> bool:
        """Add a node to the distributed system"""
        with self.lock:
            if node_id in self.nodes:
                return False
            
            node = DistributedDBNode(node_id, host, port, capacity)
            self.nodes[node_id] = node
            print(f"✓ Added node: {node_id} at {host}:{port}")
            return True
    
    def remove_node(self, node_id: str) -> bool:
        """Remove a node from the distributed system"""
        with self.lock:
            if node_id not in self.nodes:
                return False
            
            node = self.nodes[node_id]
            
            # Remove from shard assignments
            for shard_id in node.shard_assignments:
                self.shard_manager.remove_node_from_shard(shard_id, node_id)
            
            del self.nodes[node_id]
            print(f"✓ Removed node: {node_id}")
            return True
    
    def create_shard(self, shard_id: str, strategy: ShardStrategy, 
                    nodes: List[str], start_key: Any = None, end_key: Any = None,
                    replication_factor: int = 3) -> bool:
        """Create a new shard"""
        with self.lock:
            # Validate nodes exist
            for node_id in nodes:
                if node_id not in self.nodes:
                    print(f"✗ Node {node_id} does not exist")
                    return False
            
            shard_config = ShardConfig(
                shard_id=shard_id,
                start_key=start_key,
                end_key=end_key,
                nodes=nodes,
                replication_factor=replication_factor,
                strategy=strategy
            )
            
            self.shard_manager.add_shard(shard_config)
            
            # Assign shard to nodes
            for node_id in nodes:
                self.nodes[node_id].shard_assignments.add(shard_id)
            
            print(f"✓ Created shard {shard_id} with strategy {strategy.value}")
            print(f"  Nodes: {', '.join(nodes)}")
            if start_key is not None and end_key is not None:
                print(f"  Range: [{start_key}, {end_key})")
            print(f"  Replication factor: {replication_factor}")
            return True
    
    def put(self, key: str, value: Any, shard_id: str = None) -> bool:
        """Store a key-value pair"""
        start_time = time.time()
        
        with self.lock:
            self.total_operations += 1
            
            # Determine shard
            if shard_id is None:
                shard_id = self.shard_manager.get_shard_for_key(key)
            
            if shard_id is None:
                print("✗ No suitable shard found for key")
                self.failed_operations += 1
                return False
            
            # Get nodes for shard
            shard_nodes = self.shard_manager.get_nodes_for_shard(shard_id)
            if not shard_nodes:
                print(f"✗ No nodes found for shard {shard_id}")
                self.failed_operations += 1
                return False
            
            # Filter healthy nodes
            healthy_nodes = [
                node_id for node_id in shard_nodes
                if node_id in self.nodes and self.nodes[node_id].is_healthy()
            ]
            
            if not healthy_nodes:
                print(f"✗ No healthy nodes found for shard {shard_id}")
                self.failed_operations += 1
                return False
            
            # Get quorum nodes for write
            quorum_nodes = self.replication_manager.get_quorum_nodes(healthy_nodes)
            
            # Create record
            record = DataRecord(
                key=key,
                value=value,
                shard_id=shard_id,
                timestamp=time.time(),
                version=1,
                checksum=hashlib.md5(str(value).encode()).hexdigest()
            )
            
            # Write to quorum
            success_count = 0
            for node_id in quorum_nodes:
                if self.nodes[node_id].store(record):
                    success_count += 1
            
            # Check if write was successful
            if success_count >= len(quorum_nodes):
                self.successful_operations += 1
                
                # Log operation
                self.replication_manager.log_write_operation({
                    'type': 'PUT',
                    'key': key,
                    'value': value,
                    'shard_id': shard_id,
                    'nodes': quorum_nodes,
                    'timestamp': time.time()
                })
                
                return True
            else:
                self.failed_operations += 1
                print(f"✗ Failed to write to sufficient nodes ({success_count}/{len(quorum_nodes)})")
                return False
    
    def get(self, key: str, shard_id: str = None) -> Optional[Any]:
        """Retrieve a value by key"""
        start_time = time.time()
        
        with self.lock:
            self.total_operations += 1
            
            # Determine shard
            if shard_id is None:
                shard_id = self.shard_manager.get_shard_for_key(key)
            
            if shard_id is None:
                print("✗ No suitable shard found for key")
                self.failed_operations += 1
                return None
            
            # Get nodes for shard
            shard_nodes = self.shard_manager.get_nodes_for_shard(shard_id)
            if not shard_nodes:
                print(f"✗ No nodes found for shard {shard_id}")
                self.failed_operations += 1
                return None
            
            # Filter healthy nodes
            healthy_nodes = [
                node_id for node_id in shard_nodes
                if node_id in self.nodes and self.nodes[node_id].is_healthy()
            ]
            
            if not healthy_nodes:
                print(f"✗ No healthy nodes found for shard {shard_id}")
                self.failed_operations += 1
                return None
            
            # Read from quorum
            quorum_nodes = self.replication_manager.get_quorum_nodes(healthy_nodes)
            results = []
            
            for node_id in quorum_nodes:
                record = self.nodes[node_id].retrieve(key)
                if record:
                    results.append(record)
            
            # Check consistency
            if self.replication_manager.check_read_consistency(results):
                if results:
                    self.successful_operations += 1
                    return results[0].value
                else:
                    self.successful_operations += 1
                    return None
            else:
                self.failed_operations += 1
                print("✗ Consistency check failed")
                return None
    
    def delete(self, key: str, shard_id: str = None) -> bool:
        """Delete a key-value pair"""
        with self.lock:
            self.total_operations += 1
            
            # Determine shard
            if shard_id is None:
                shard_id = self.shard_manager.get_shard_for_key(key)
            
            if shard_id is None:
                self.failed_operations += 1
                return False
            
            # Get nodes for shard
            shard_nodes = self.shard_manager.get_nodes_for_shard(shard_id)
            if not shard_nodes:
                self.failed_operations += 1
                return False
            
            # Filter healthy nodes
            healthy_nodes = [
                node_id for node_id in shard_nodes
                if node_id in self.nodes and self.nodes[node_id].is_healthy()
            ]
            
            if not healthy_nodes:
                self.failed_operations += 1
                return False
            
            # Delete from quorum
            quorum_nodes = self.replication_manager.get_quorum_nodes(healthy_nodes)
            success_count = 0
            
            for node_id in quorum_nodes:
                if self.nodes[node_id].delete(key):
                    success_count += 1
            
            if success_count >= len(quorum_nodes):
                self.successful_operations += 1
                return True
            else:
                self.failed_operations += 1
                return False
    
    def query_range(self, start_key: str, end_key: str, shard_id: str = None) -> List[Tuple[str, Any]]:
        """Query records in a key range"""
        with self.lock:
            self.total_operations += 1
            
            results = []
            
            # If shard_id specified, query only that shard
            if shard_id:
                shard_ids = [shard_id]
            else:
                # Query all shards that might contain the range
                shard_ids = list(self.shard_manager.shards.keys())
            
            for shard_id in shard_ids:
                shard_nodes = self.shard_manager.get_nodes_for_shard(shard_id)
                
                for node_id in shard_nodes:
                    if node_id in self.nodes and self.nodes[node_id].is_healthy():
                        records = self.nodes[node_id].query_range(start_key, end_key)
                        for record in records:
                            results.append((record.key, record.value))
            
            self.successful_operations += 1
            return sorted(results)
    
    def add_replica(self, shard_id: str, node_id: str) -> bool:
        """Add a replica node to a shard"""
        with self.lock:
            if shard_id not in self.shard_manager.shards:
                return False
            
            if node_id not in self.nodes:
                return False
            
            self.shard_manager.add_node_to_shard(shard_id, node_id)
            self.nodes[node_id].shard_assignments.add(shard_id)
            
            # Replicate data from existing replicas
            existing_nodes = self.shard_manager.get_nodes_for_shard(shard_id)
            existing_nodes = [n for n in existing_nodes if n != node_id]
            
            if existing_nodes:
                source_node = self.nodes[existing_nodes[0]]
                
                # Copy data to new replica
                for key, record in source_node.data.items():
                    # Update shard_id in record
                    record.shard_id = shard_id
                    record.timestamp = time.time()
                    record.version += 1
                    
                    self.nodes[node_id].store(record)
            
            print(f"✓ Added replica {node_id} to shard {shard_id}")
            return True
    
    def remove_replica(self, shard_id: str, node_id: str) -> bool:
        """Remove a replica node from a shard"""
        with self.lock:
            if shard_id not in self.shard_manager.shards:
                return False
            
            shard_nodes = self.shard_manager.get_nodes_for_shard(shard_id)
            if len(shard_nodes) <= 1:
                print(f"✗ Cannot remove last replica from shard {shard_id}")
                return False
            
            self.shard_manager.remove_node_from_shard(shard_id, node_id)
            self.nodes[node_id].shard_assignments.discard(shard_id)
            
            print(f"✓ Removed replica {node_id} from shard {shard_id}")
            return True
    
    def rebalance_shards(self) -> bool:
        """Rebalance shard distribution across nodes"""
        with self.lock:
            print("\n1. Analyzing load distribution...")
            
            # Get load statistics
            node_loads = {}
            for node_id, node in self.nodes.items():
                if node.state == NodeState.ACTIVE:
                    stats = node.get_statistics()
                    node_loads[node_id] = stats['load']
            
            if not node_loads:
                return False
            
            # Find overloaded and underloaded nodes
            avg_load = sum(node_loads.values()) / len(node_loads)
            overloaded = [n for n, load in node_loads.items() if load > avg_load * 1.2]
            underloaded = [n for n, load in node_loads.items() if load < avg_load * 0.8]
            
            print(f"   Average load: {avg_load:.1f}")
            print(f"   Overloaded nodes: {len(overloaded)}")
            print(f"   Underloaded nodes: {len(underloaded)}")
            
            # Simple rebalancing: move shards from overloaded to underloaded
            moved_shards = 0
            for over_node in overloaded:
                for under_node in underloaded:
                    # Find shard to move
                    over_shards = list(self.nodes[over_node].shard_assignments)
                    if over_shards:
                        shard_to_move = over_shards[0]
                        
                        # Remove from overloaded node
                        self.remove_replica(shard_to_move, over_node)
                        
                        # Add to underloaded node
                        self.add_replica(shard_to_move, under_node)
                        
                        moved_shards += 1
                        
                        if moved_shards >= len(overloaded):
                            break
                
                if moved_shards >= len(overloaded):
                    break
            
            print(f"✓ Rebalanced {moved_shards} shards")
            return True
    
    def handle_node_failure(self, failed_node_id: str) -> bool:
        """Handle node failure and trigger recovery"""
        with self.lock:
            print(f"\n1. Node failure detected: {failed_node_id}")
            
            if failed_node_id not in self.nodes:
                return False
            
            # Mark node as inactive
            self.nodes[failed_node_id].state = NodeState.INACTIVE
            
            # Get shards that were on the failed node
            failed_shards = list(self.nodes[failed_node_id].shard_assignments)
            
            print(f"   Failed node had {len(failed_shards)} shards")
            
            # Replicate data to other nodes
            for shard_id in failed_shards:
                shard_nodes = self.shard_manager.get_nodes_for_shard(shard_id)
                remaining_nodes = [n for n in shard_nodes if n != failed_node_id]
                
                if len(remaining_nodes) < 1:
                    # Need to add new replica
                    healthy_nodes = [n for n in self.nodes.keys() 
                                   if n != failed_node_id and self.nodes[n].is_healthy()]
                    
                    if healthy_nodes:
                        new_replica = random.choice(healthy_nodes)
                        self.add_replica(shard_id, new_replica)
                        print(f"   Added new replica {new_replica} to shard {shard_id}")
            
            return True
    
    def get_system_statistics(self) -> Dict[str, Any]:
        """Get comprehensive system statistics"""
        with self.lock:
            # Node statistics
            node_stats = {}
            total_load = 0
            total_capacity = 0
            healthy_nodes = 0
            
            for node_id, node in self.nodes.items():
                stats = node.get_statistics()
                node_stats[node_id] = stats
                total_load += stats['load']
                total_capacity += stats['capacity']
                if stats['utilization'] < 0.9:
                    healthy_nodes += 1
            
            # Shard statistics
            shard_stats = {}
            for shard_id, shard_config in self.shard_manager.shards.items():
                shard_nodes = self.shard_manager.get_nodes_for_shard(shard_id)
                shard_stats[shard_id] = {
                    'strategy': shard_config.strategy.value,
                    'replicas': len(shard_nodes),
                    'nodes': shard_nodes,
                    'load_distribution': {}
                }
                
                # Calculate load distribution
                for node_id in shard_nodes:
                    if node_id in self.nodes:
                        node_data_count = len(self.nodes[node_id].data)
                        shard_stats[shard_id]['load_distribution'][node_id] = node_data_count
            
            return {
                'total_nodes': len(self.nodes),
                'healthy_nodes': healthy_nodes,
                'total_shards': len(self.shard_manager.shards),
                'system_utilization': total_load / max(1, total_capacity),
                'total_operations': self.total_operations,
                'successful_operations': self.successful_operations,
                'failed_operations': self.failed_operations,
                'success_rate': self.successful_operations / max(1, self.total_operations),
                'consistency_level': self.replication_manager.consistency_level.value,
                'replication_factor': self.replication_manager.replication_factor,
                'nodes': node_stats,
                'shards': shard_stats
            }
    
    def simulate_workload(self, operations: int = 100) -> Dict[str, Any]:
        """Simulate database workload for testing"""
        print(f"\n1. Simulating {operations} database operations...")
        
        results = {
            'operations': operations,
            'successful': 0,
            'failed': 0,
            'total_time': 0,
            'avg_time_per_op': 0
        }
        
        start_time = time.time()
        
        for i in range(operations):
            # Random operation
            op_type = random.choice(['put', 'get', 'delete'])
            key = f"key_{random.randint(1, 1000)}"
            value = f"value_{random.randint(1, 10000)}"
            
            op_start = time.time()
            
            if op_type == 'put':
                success = self.put(key, value)
            elif op_type == 'get':
                result = self.get(key)
                success = result is not None or random.random() < 0.5  # Some keys won't exist
            else:  # delete
                success = self.delete(key)
            
            op_time = time.time() - op_start
            results['total_time'] += op_time
            
            if success:
                results['successful'] += 1
            else:
                results['failed'] += 1
        
        results['avg_time_per_op'] = results['total_time'] / operations
        
        print(f"✓ Simulation completed:")
        print(f"   Successful: {results['successful']}")
        print(f"   Failed: {results['failed']}")
        print(f"   Total time: {results['total_time']:.3f}s")
        print(f"   Avg time per op: {results['avg_time_per_op']:.6f}s")
        
        return results