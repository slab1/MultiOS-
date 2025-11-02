"""
Educational Database Query Optimizer and Indexing System
Implements various indexing techniques and query optimization strategies
"""

import heapq
import time
import math
from typing import Dict, List, Any, Optional, Tuple, Set
from dataclasses import dataclass, asdict
from enum import Enum
import random
import bisect


class IndexType(Enum):
    """Types of indexes"""
    BTREE = "BTREE"
    HASH = "HASH"
    BITMAP = "BITMAP"
    FULL_TEXT = "FULL_TEXT"
    COMPOUND = "COMPOUND"


class QueryType(Enum):
    """Types of queries for optimization"""
    SELECT = "SELECT"
    INSERT = "INSERT"
    UPDATE = "UPDATE"
    DELETE = "DELETE"
    JOIN = "JOIN"
    AGGREGATION = "AGGREGATION"


class AccessMethod(Enum):
    """Access methods for query execution"""
    TABLE_SCAN = "TABLE_SCAN"
    INDEX_SCAN = "INDEX_SCAN"
    INDEX_LOOKUP = "INDEX_LOOKUP"
    SORT_MERGE_JOIN = "SORT_MERGE_JOIN"
    NESTED_LOOP_JOIN = "NESTED_LOOP_JOIN"
    HASH_JOIN = "HASH_JOIN"


@dataclass
class IndexStats:
    """Statistics about an index"""
    index_type: IndexType
    cardinality: int  # Number of unique values
    selectivity: float  # Selectivity of index (lower is better)
    depth: int  # For tree-based indexes
    size_bytes: int
    last_updated: float
    
    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


@dataclass
class QueryPlan:
    """Represents a query execution plan"""
    query_type: QueryType
    tables: List[str]
    access_methods: List[AccessMethod]
    cost: float
    estimated_rows: int
    operations: List[str]
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            'query_type': self.query_type.value,
            'tables': self.tables,
            'access_methods': [method.value for method in self.access_methods],
            'cost': self.cost,
            'estimated_rows': self.estimated_rows,
            'operations': self.operations
        }


@dataclass
class QueryStatistics:
    """Statistics collected during query execution"""
    total_time: float
    cpu_time: float
    io_operations: int
    rows_processed: int
    rows_returned: int
    memory_used: int
    
    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


class BTreeIndex:
    """Educational B-Tree index implementation"""
    
    def __init__(self, max_keys_per_node: int = 4):
        self.max_keys_per_node = max_keys_per_node
        self.root = None
        self.key_count = 0
        self.depth = 0
        
    def insert(self, key: Any, value: Any):
        """Insert key-value pair into B-Tree"""
        if self.root is None:
            self.root = BTreeNode(self.max_keys_per_node)
        
        # Simple insertion logic (educational purposes)
        # In a real implementation, this would handle splitting
        self._insert_non_full(self.root, key, value)
        self.key_count += 1
        
        # Handle root splitting
        if self.root.is_full():
            new_root = BTreeNode(self.max_keys_per_node)
            new_root.children.append(self.root)
            self._split_child(new_root, 0)
            self.root = new_root
            self.depth += 1
    
    def _insert_non_full(self, node: 'BTreeNode', key: Any, value: Any):
        """Insert into non-full node"""
        if node.is_leaf():
            # Insert into leaf
            i = len(node.keys) - 1
            node.keys.append(None)
            node.values.append(None)
            
            while i >= 0 and key < node.keys[i]:
                node.keys[i + 1] = node.keys[i]
                node.values[i + 1] = node.values[i]
                i -= 1
            
            node.keys[i + 1] = key
            node.values[i + 1] = value
        else:
            # Find child to insert into
            i = len(node.keys) - 1
            while i >= 0 and key < node.keys[i]:
                i -= 1
            i += 1
            
            if node.children[i].is_full():
                self._split_child(node, i)
                if key > node.keys[i]:
                    i += 1
            
            self._insert_non_full(node.children[i], key, value)
    
    def _split_child(self, parent: 'BTreeNode', child_index: int):
        """Split a full child node"""
        full_child = parent.children[child_index]
        mid_point = self.max_keys_per_node // 2
        
        new_child = BTreeNode(self.max_keys_per_node)
        
        # Move upper half to new child
        new_child.keys = full_child.keys[mid_point + 1:]
        new_child.values = full_child.values[mid_point + 1:]
        full_child.keys = full_child.keys[:mid_point]
        full_child.values = full_child.values[:mid_point]
        
        # Move children if not leaf
        if not full_child.is_leaf():
            new_child.children = full_child.children[mid_point + 1:]
            full_child.children = full_child.children[:mid_point + 1]
        
        # Insert new child into parent
        parent.keys.insert(child_index, full_child.keys.pop())
        parent.values.insert(child_index, full_child.values.pop())
        parent.children.insert(child_index + 1, new_child)
    
    def search(self, key: Any) -> Optional[Any]:
        """Search for key in B-Tree"""
        return self._search(self.root, key)
    
    def _search(self, node: 'BTreeNode', key: Any) -> Optional[Any]:
        """Recursively search for key"""
        if node is None:
            return None
        
        # Find position to search
        i = 0
        while i < len(node.keys) and key > node.keys[i]:
            i += 1
        
        # Check if key is in current node
        if i < len(node.keys) and key == node.keys[i]:
            return node.values[i]
        
        # Check if leaf
        if node.is_leaf():
            return None
        
        # Search in appropriate child
        return self._search(node.children[i], key)
    
    def range_query(self, start_key: Any, end_key: Any) -> List[Tuple[Any, Any]]:
        """Perform range query"""
        results = []
        self._range_search(self.root, start_key, end_key, results)
        return results
    
    def _range_search(self, node: 'BTreeNode', start_key: Any, end_key: Any, results: List[Tuple[Any, Any]]):
        """Recursively perform range search"""
        if node is None:
            return
        
        i = 0
        while i < len(node.keys) and start_key > node.keys[i]:
            i += 1
        
        if not node.is_leaf():
            self._range_search(node.children[i], start_key, end_key, results)
        
        # Add matching keys from current node
        while i < len(node.keys) and node.keys[i] <= end_key:
            results.append((node.keys[i], node.values[i]))
            i += 1
        
        if not node.is_leaf():
            self._range_search(node.children[i], start_key, end_key, results)
    
    def get_statistics(self) -> Dict[str, Any]:
        """Get B-Tree statistics"""
        if self.root is None:
            return {'size': 0, 'depth': 0, 'nodes': 0}
        
        node_count = self._count_nodes(self.root)
        return {
            'size': self.key_count,
            'depth': self.depth,
            'nodes': node_count,
            'avg_keys_per_node': self.key_count / max(1, node_count)
        }
    
    def _count_nodes(self, node: 'BTreeNode') -> int:
        """Count total nodes in B-Tree"""
        if node is None:
            return 0
        
        count = 1
        for child in node.children:
            count += self._count_nodes(child)
        return count


class BTreeNode:
    """Node in B-Tree"""
    
    def __init__(self, max_keys_per_node: int):
        self.keys = []
        self.values = []
        self.children = []
        self.max_keys = max_keys_per_node
    
    def is_leaf(self) -> bool:
        """Check if node is leaf"""
        return len(self.children) == 0
    
    def is_full(self) -> bool:
        """Check if node is full"""
        return len(self.keys) >= self.max_keys


class HashIndex:
    """Educational Hash index implementation"""
    
    def __init__(self, bucket_size: int = 100):
        self.bucket_size = bucket_size
        self.buckets = [HashBucket() for _ in range(bucket_size)]
        self.key_count = 0
    
    def _hash_function(self, key: Any) -> int:
        """Simple hash function"""
        return hash(str(key)) % self.bucket_size
    
    def insert(self, key: Any, value: Any):
        """Insert key-value pair into hash index"""
        bucket_index = self._hash_function(key)
        bucket = self.buckets[bucket_index]
        
        # Check if key already exists
        for i, (k, v) in enumerate(bucket.entries):
            if k == key:
                bucket.entries[i] = (key, value)  # Update existing
                return
        
        # Add new entry
        bucket.entries.append((key, value))
        self.key_count += 1
    
    def search(self, key: Any) -> Optional[Any]:
        """Search for key in hash index"""
        bucket_index = self._hash_function(key)
        bucket = self.buckets[bucket_index]
        
        for k, v in bucket.entries:
            if k == key:
                return v
        return None
    
    def delete(self, key: Any) -> bool:
        """Delete key from hash index"""
        bucket_index = self._hash_function(key)
        bucket = self.buckets[bucket_index]
        
        for i, (k, v) in enumerate(bucket.entries):
            if k == key:
                del bucket.entries[i]
                self.key_count -= 1
                return True
        return False
    
    def get_statistics(self) -> Dict[str, Any]:
        """Get hash index statistics"""
        total_entries = sum(len(bucket.entries) for bucket in self.buckets)
        max_bucket_size = max(len(bucket.entries) for bucket in self.buckets) if self.buckets else 0
        avg_bucket_size = total_entries / self.bucket_size if self.bucket_size > 0 else 0
        
        return {
            'size': self.key_count,
            'bucket_count': self.bucket_size,
            'max_bucket_size': max_bucket_size,
            'avg_bucket_size': avg_bucket_size,
            'load_factor': total_entries / self.bucket_size if self.bucket_size > 0 else 0
        }


class HashBucket:
    """Bucket for hash index"""
    
    def __init__(self):
        self.entries = []  # List of (key, value) tuples


class BitmapIndex:
    """Educational Bitmap index implementation"""
    
    def __init__(self, value_mapping: Dict[Any, int]):
        self.value_mapping = value_mapping  # Maps value to bitmap index
        self.bitmaps = {}  # value -> bitmap (list of ints)
        self.total_rows = 0
        
        # Initialize bitmaps for each unique value
        for value in value_mapping:
            self.bitmaps[value] = [0] * 10  # Start with 10 ints
    
    def insert(self, value: Any):
        """Insert value into bitmap index"""
        if value not in self.value_mapping:
            # Add new value to mapping
            index = len(self.value_mapping)
            self.value_mapping[value] = index
            self.bitmaps[value] = [0] * ((self.total_rows // 64) + 1)
        
        # Set bit for this value
        bitmap_index = self.value_mapping[value]
        word_index = self.total_rows // 64
        bit_position = self.total_rows % 64
        
        # Ensure bitmap is large enough
        while len(self.bitmaps[value]) <= word_index:
            for v in self.bitmaps:
                self.bitmaps[v].append(0)
        
        # Set the bit
        self.bitmaps[value][word_index] |= (1 << bit_position)
        self.total_rows += 1
    
    def search_equals(self, value: Any) -> List[int]:
        """Search for rows where value equals given value"""
        if value not in self.value_mapping:
            return []
        
        bitmap_index = self.value_mapping[value]
        result_rows = []
        
        for word_index, word in enumerate(self.bitmaps[value]):
            if word != 0:
                bit_position = 0
                while bit_position < 64:
                    if word & (1 << bit_position):
                        row_id = word_index * 64 + bit_position
                        if row_id < self.total_rows:
                            result_rows.append(row_id)
                    bit_position += 1
        
        return result_rows
    
    def search_range(self, start_value: Any, end_value: Any) -> List[int]:
        """Search for rows where value is in range"""
        result_rows = []
        
        # Get all values in range
        values_in_range = []
        for value, index in self.value_mapping.items():
            if start_value <= value <= end_value:
                values_in_range.append(value)
        
        # Union of bitmaps
        for value in values_in_range:
            bitmap_rows = self.search_equals(value)
            result_rows.extend(bitmap_rows)
        
        return sorted(set(result_rows))  # Remove duplicates
    
    def get_statistics(self) -> Dict[str, Any]:
        """Get bitmap index statistics"""
        unique_values = len(self.value_mapping)
        bitmap_size_bytes = sum(len(bitmap) * 4 for bitmap in self.bitmaps.values())
        
        return {
            'unique_values': unique_values,
            'total_rows': self.total_rows,
            'bitmap_size_bytes': bitmap_size_bytes,
            'compression_ratio': bitmap_size_bytes / max(1, self.total_rows * 4)  # Compare to dense array
        }


class FullTextIndex:
    """Educational Full-text index implementation"""
    
    def __init__(self):
        self.inverted_index = {}  # word -> set of document IDs
        self.documents = {}  # doc_id -> document content
        self.total_docs = 0
    
    def add_document(self, doc_id: str, content: str):
        """Add document to full-text index"""
        self.documents[doc_id] = content
        self.total_docs += 1
        
        # Tokenize content
        words = self._tokenize(content.lower())
        
        # Add to inverted index
        for word in words:
            if word not in self.inverted_index:
                self.inverted_index[word] = set()
            self.inverted_index[word].add(doc_id)
    
    def _tokenize(self, text: str) -> List[str]:
        """Simple tokenization"""
        import re
        # Remove punctuation and split into words
        words = re.findall(r'\b[a-zA-Z]+\b', text.lower())
        return [word for word in words if len(word) > 2]  # Filter short words
    
    def search(self, query: str, operator: str = 'AND') -> Set[str]:
        """Search documents containing query terms"""
        query_terms = self._tokenize(query.lower())
        if not query_terms:
            return set()
        
        result_sets = []
        for term in query_terms:
            if term in self.inverted_index:
                result_sets.append(self.inverted_index[term])
            else:
                # Term not found
                if operator == 'AND':
                    return set()  # AND with empty set is empty
                else:
                    continue
        
        if not result_sets:
            return set()
        
        if operator == 'AND':
            # Intersection of all sets
            result = result_sets[0]
            for doc_set in result_sets[1:]:
                result = result.intersection(doc_set)
            return result
        else:  # OR
            # Union of all sets
            result = set()
            for doc_set in result_sets:
                result = result.union(doc_set)
            return result
    
    def get_statistics(self) -> Dict[str, Any]:
        """Get full-text index statistics"""
        total_terms = sum(len(doc_set) for doc_set in self.inverted_index.values())
        avg_docs_per_term = total_terms / max(1, len(self.inverted_index))
        
        return {
            'total_documents': self.total_docs,
            'unique_terms': len(self.inverted_index),
            'total_term_occurrences': total_terms,
            'avg_docs_per_term': avg_docs_per_term,
            'avg_doc_length': sum(len(doc) for doc in self.documents.values()) / max(1, self.total_docs)
        }


class CompoundIndex:
    """Educational Compound (composite) index implementation"""
    
    def __init__(self, columns: List[str]):
        self.columns = columns
        self.entries = []  # List of (key_tuple, value) tuples
        self.is_sorted = True
    
    def insert(self, values: List[Any], value: Any):
        """Insert into compound index"""
        key_tuple = tuple(values)
        self.entries.append((key_tuple, value))
        self.is_sorted = False
    
    def search(self, values: List[Any]) -> List[Any]:
        """Search for exact match"""
        if len(values) != len(self.columns):
            return []
        
        key_tuple = tuple(values)
        results = []
        
        for key, value in self.entries:
            if key == key_tuple:
                results.append(value)
        
        return results
    
    def search_prefix(self, prefix_values: List[Any]) -> List[Any]:
        """Search for records with given prefix"""
        if len(prefix_values) >= len(self.columns):
            return self.search(prefix_values)
        
        results = []
        prefix_tuple = tuple(prefix_values)
        
        # Binary search if sorted, otherwise linear search
        if self.is_sorted:
            left = bisect.bisect_left(self.entries, (prefix_tuple,))
            right = bisect.bisect_right(self.entries, (prefix_tuple,))
            for i in range(left, right):
                results.append(self.entries[i][1])
        else:
            for key, value in self.entries:
                if key[:len(prefix_values)] == prefix_tuple:
                    results.append(value)
        
        return results
    
    def range_query(self, start_values: List[Any], end_values: List[Any]) -> List[Any]:
        """Search for records in value range"""
        results = []
        start_tuple = tuple(start_values)
        end_tuple = tuple(end_values)
        
        if self.is_sorted:
            left = bisect.bisect_left(self.entries, (start_tuple,))
            right = bisect.bisect_right(self.entries, (end_tuple,))
            for i in range(left, right):
                results.append(self.entries[i][1])
        else:
            for key, value in self.entries:
                if start_tuple <= key <= end_tuple:
                    results.append(value)
        
        return results
    
    def sort(self):
        """Sort entries for efficient searching"""
        self.entries.sort(key=lambda x: x[0])
        self.is_sorted = True
    
    def get_statistics(self) -> Dict[str, Any]:
        """Get compound index statistics"""
        return {
            'columns': self.columns,
            'size': len(self.entries),
            'is_sorted': self.is_sorted,
            'avg_key_length': sum(len(key) for key, _ in self.entries) / max(1, len(self.entries))
        }


class DatabaseIndex:
    """Unified interface for different index types"""
    
    def __init__(self, index_name: str, table_name: str, columns: List[str], 
                 index_type: IndexType = IndexType.BTREE):
        self.index_name = index_name
        self.table_name = table_name
        self.columns = columns
        self.index_type = index_type
        
        # Initialize underlying index
        if index_type == IndexType.BTREE:
            self.index = BTreeIndex()
        elif index_type == IndexType.HASH:
            self.index = HashIndex()
        elif index_type == IndexType.COMPOUND:
            self.index = CompoundIndex(columns)
        else:
            self.index = BTreeIndex()  # Default
        
        # Index statistics
        self.stats = IndexStats(
            index_type=index_type,
            cardinality=0,
            selectivity=1.0,
            depth=0,
            size_bytes=0,
            last_updated=time.time()
        )
    
    def insert(self, key: Any, value: Any):
        """Insert into index"""
        self.index.insert(key, value)
        self._update_statistics()
    
    def search(self, key: Any) -> Optional[Any]:
        """Search in index"""
        return self.index.search(key)
    
    def range_query(self, start_key: Any, end_key: Any) -> List[Tuple[Any, Any]]:
        """Range query"""
        if hasattr(self.index, 'range_query'):
            return self.index.range_query(start_key, end_key)
        else:
            # Fallback for indexes that don't support range queries
            return []
    
    def get_statistics(self) -> Dict[str, Any]:
        """Get index statistics"""
        base_stats = self.index.get_statistics()
        self.stats = IndexStats(
            index_type=self.index_type,
            cardinality=self.stats.cardinality,
            selectivity=self.stats.selectivity,
            depth=base_stats.get('depth', 0),
            size_bytes=base_stats.get('size', 0) * 8,  # Estimate
            last_updated=time.time()
        )
        return base_stats
    
    def _update_statistics(self):
        """Update index statistics"""
        # Calculate cardinality and selectivity
        # This would be more sophisticated in a real implementation
        self.stats.cardinality = min(1000, len(self.index.entries) if hasattr(self.index, 'entries') else 100)
        self.stats.selectivity = 1.0 / max(1, self.stats.cardinality)
        self.stats.last_updated = time.time()


class QueryOptimizer:
    """
    Educational Query Optimizer
    Implements cost-based query optimization
    """
    
    def __init__(self, db_stats: Dict[str, Any]):
        self.db_stats = db_stats
        self.optimization_history = []
    
    def optimize_query(self, query: Dict[str, Any], available_indexes: Dict[str, DatabaseIndex]) -> QueryPlan:
        """Generate optimized query execution plan"""
        start_time = time.time()
        
        # Analyze query
        query_type = self._determine_query_type(query)
        tables = self._extract_tables(query)
        
        # Generate candidate plans
        candidate_plans = self._generate_candidate_plans(query, query_type, tables, available_indexes)
        
        # Select best plan based on cost
        best_plan = self._select_best_plan(candidate_plans)
        
        # Log optimization decision
        optimization_time = time.time() - start_time
        self.optimization_history.append({
            'query': query,
            'plan': best_plan,
            'optimization_time': optimization_time,
            'timestamp': time.time()
        })
        
        return best_plan
    
    def _determine_query_type(self, query: Dict[str, Any]) -> QueryType:
        """Determine query type"""
        if 'SELECT' in query:
            return QueryType.SELECT
        elif 'INSERT' in query:
            return QueryType.INSERT
        elif 'UPDATE' in query:
            return QueryType.UPDATE
        elif 'DELETE' in query:
            return QueryType.DELETE
        else:
            return QueryType.SELECT  # Default
    
    def _extract_tables(self, query: Dict[str, Any]) -> List[str]:
        """Extract tables from query"""
        # Simplified table extraction
        if 'FROM' in query:
            return [query['FROM']]
        return ['default_table']
    
    def _generate_candidate_plans(self, query: Dict[str, Any], query_type: QueryType, 
                                tables: List[str], available_indexes: Dict[str, DatabaseIndex]) -> List[QueryPlan]:
        """Generate candidate execution plans"""
        candidate_plans = []
        
        if query_type == QueryType.SELECT:
            candidate_plans = self._generate_select_plans(query, tables, available_indexes)
        elif query_type == QueryType.UPDATE:
            candidate_plans = self._generate_update_plans(query, tables, available_indexes)
        
        return candidate_plans
    
    def _generate_select_plans(self, query: Dict[str, Any], tables: List[str], 
                             available_indexes: Dict[str, DatabaseIndex]) -> List[QueryPlan]:
        """Generate candidate SELECT plans"""
        plans = []
        
        # Plan 1: Full table scan
        table_scan_plan = QueryPlan(
            query_type=QueryType.SELECT,
            tables=tables,
            access_methods=[AccessMethod.TABLE_SCAN],
            cost=self._estimate_table_scan_cost(tables[0]),
            estimated_rows=self.db_stats.get('total_rows', 10000),
            operations=['Table Scan', 'Filter', 'Project']
        )
        plans.append(table_scan_plan)
        
        # Plan 2: Index scan if suitable index exists
        where_clause = query.get('WHERE', {})
        for index_name, index in available_indexes.items():
            if self._is_index_applicable(index, where_clause):
                index_scan_plan = QueryPlan(
                    query_type=QueryType.SELECT,
                    tables=tables,
                    access_methods=[AccessMethod.INDEX_SCAN],
                    cost=self._estimate_index_scan_cost(index),
                    estimated_rows=self._estimate_rows_from_index(index, where_clause),
                    operations=['Index Scan', 'Filter', 'Project']
                )
                plans.append(index_scan_plan)
        
        # Plan 3: Index lookup for point queries
        if self._is_point_query(where_clause):
            for index_name, index in available_indexes.items():
                if self._is_index_applicable(index, where_clause):
                    index_lookup_plan = QueryPlan(
                        query_type=QueryType.SELECT,
                        tables=tables,
                        access_methods=[AccessMethod.INDEX_LOOKUP],
                        cost=self._estimate_index_lookup_cost(index),
                        estimated_rows=1,
                        operations=['Index Lookup', 'Fetch', 'Project']
                    )
                    plans.append(index_lookup_plan)
        
        return plans
    
    def _generate_update_plans(self, query: Dict[str, Any], tables: List[str], 
                             available_indexes: Dict[str, DatabaseIndex]) -> List[QueryPlan]:
        """Generate candidate UPDATE plans"""
        plans = []
        
        # Update requires updating indexes
        index_update_cost = len(available_indexes) * 10
        
        update_plan = QueryPlan(
            query_type=QueryType.UPDATE,
            tables=tables,
            access_methods=[AccessMethod.TABLE_SCAN],
            cost=self._estimate_table_scan_cost(tables[0]) + index_update_cost,
            estimated_rows=self.db_stats.get('total_rows', 10000) * 0.1,  # Assume 10% update
            operations=['Find Records', 'Update', 'Update Indexes']
        )
        plans.append(update_plan)
        
        return plans
    
    def _select_best_plan(self, plans: List[QueryPlan]) -> QueryPlan:
        """Select best plan based on cost"""
        return min(plans, key=lambda p: p.cost)
    
    def _estimate_table_scan_cost(self, table_name: str) -> float:
        """Estimate cost of table scan"""
        total_rows = self.db_stats.get('total_rows', 10000)
        return total_rows * 0.001  # Cost per row
    
    def _estimate_index_scan_cost(self, index: DatabaseIndex) -> float:
        """Estimate cost of index scan"""
        if index.index_type == IndexType.BTREE:
            return index.stats.cardinality * 0.01  # Logarithmic cost
        elif index.index_type == IndexType.HASH:
            return index.stats.cardinality * 0.001  # Constant time
        else:
            return index.stats.cardinality * 0.005
    
    def _estimate_index_lookup_cost(self, index: DatabaseIndex) -> float:
        """Estimate cost of index lookup"""
        if index.index_type == IndexType.BTREE:
            return math.log2(max(1, index.stats.cardinality)) * 0.001
        elif index.index_type == IndexType.HASH:
            return 0.001  # O(1) lookup
        else:
            return 0.01
    
    def _estimate_rows_from_index(self, index: DatabaseIndex, where_clause: Dict[str, Any]) -> int:
        """Estimate number of rows returned from index"""
        # Simplified selectivity estimation
        if index.stats.cardinality > 0:
            selectivity = 1.0 / index.stats.cardinality
            total_rows = self.db_stats.get('total_rows', 10000)
            return max(1, int(total_rows * selectivity))
        return 1
    
    def _is_index_applicable(self, index: DatabaseIndex, where_clause: Dict[str, Any]) -> bool:
        """Check if index can be used for query"""
        # Check if index columns match query conditions
        for column in index.columns:
            if column in where_clause:
                return True
        return False
    
    def _is_point_query(self, where_clause: Dict[str, Any]) -> bool:
        """Check if query is a point query (equality condition)"""
        for key, value in where_clause.items():
            if isinstance(value, (str, int, float)):  # Simple equality
                return True
        return False
    
    def get_optimization_statistics(self) -> Dict[str, Any]:
        """Get query optimization statistics"""
        if not self.optimization_history:
            return {'total_optimizations': 0}
        
        avg_optimization_time = sum(opt['optimization_time'] for opt in self.optimization_history) / len(self.optimization_history)
        
        plan_usage = {}
        for opt in self.optimization_history:
            for method in opt['plan'].access_methods:
                method_name = method.value
                plan_usage[method_name] = plan_usage.get(method_name, 0) + 1
        
        return {
            'total_optimizations': len(self.optimization_history),
            'avg_optimization_time': avg_optimization_time,
            'plan_usage': plan_usage,
            'total_time_saved': sum(1 for _ in self.optimization_history)  # Simplified
        }


def demonstrate_indexing():
    """Demonstrate different indexing techniques"""
    print("\n" + "="*60)
    print("INDEXING TECHNIQUES DEMONSTRATION")
    print("="*60)
    
    # B-Tree Index Demo
    print("\n1. B-Tree Index Demonstration...")
    btree = BTreeIndex(max_keys_per_node=4)
    
    # Insert data
    import random
    for i in range(50):
        key = random.randint(1, 1000)
        value = f"value_{key}"
        btree.insert(key, value)
    
    # Search operations
    search_key = random.randint(1, 1000)
    result = btree.search(search_key)
    print(f"   Search for {search_key}: {result is not None}")
    
    # Range query
    range_results = btree.range_query(100, 200)
    print(f"   Range query [100, 200]: {len(range_results)} results")
    
    # Statistics
    btree_stats = btree.get_statistics()
    print(f"   B-Tree stats: {btree_stats}")
    
    # Hash Index Demo
    print("\n2. Hash Index Demonstration...")
    hash_index = HashIndex(bucket_size=10)
    
    # Insert data
    for i in range(30):
        key = f"key_{i:03d}"
        value = f"value_{i}"
        hash_index.insert(key, value)
    
    # Search
    search_key = "key_015"
    result = hash_index.search(search_key)
    print(f"   Search for {search_key}: {result}")
    
    # Statistics
    hash_stats = hash_index.get_statistics()
    print(f"   Hash index stats: {hash_stats}")
    
    # Bitmap Index Demo
    print("\n3. Bitmap Index Demonstration...")
    # Simple value mapping for categories
    value_mapping = {'low': 0, 'medium': 1, 'high': 2, 'critical': 3}
    bitmap_index = BitmapIndex(value_mapping)
    
    # Insert data
    import random
    values = ['low', 'medium', 'high', 'critical']
    for i in range(20):
        value = random.choice(values)
        bitmap_index.insert(value)
    
    # Search operations
    low_rows = bitmap_index.search_equals('low')
    high_rows = bitmap_index.search_equals('high')
    range_rows = bitmap_index.search_range('medium', 'critical')
    
    print(f"   'low' rows: {len(low_rows)}")
    print(f"   'high' rows: {len(high_rows)}")
    print(f"   'medium' to 'critical' range: {len(range_rows)} rows")
    
    # Statistics
    bitmap_stats = bitmap_index.get_statistics()
    print(f"   Bitmap index stats: {bitmap_stats}")
    
    # Full-Text Index Demo
    print("\n4. Full-Text Index Demonstration...")
    ft_index = FullTextIndex()
    
    # Add documents
    documents = [
        ("doc1", "The quick brown fox jumps over the lazy dog"),
        ("doc2", "Database systems are fascinating and complex"),
        ("doc3", "Query optimization is important for performance"),
        ("doc4", "Indexing techniques improve search speed"),
        ("doc5", "B-trees and hash tables are common data structures")
    ]
    
    for doc_id, content in documents:
        ft_index.add_document(doc_id, content)
    
    # Search operations
    fox_results = ft_index.search("fox")
    database_results = ft_index.search("database")
    performance_results = ft_index.search("performance optimization", 'AND')
    
    print(f"   'fox' results: {fox_results}")
    print(f"   'database' results: {database_results}")
    print(f"   'performance AND optimization' results: {performance_results}")
    
    # Statistics
    ft_stats = ft_index.get_statistics()
    print(f"   Full-text index stats: {ft_stats}")
    
    # Compound Index Demo
    print("\n5. Compound Index Demonstration...")
    compound_index = CompoundIndex(['last_name', 'first_name'])
    
    # Insert data
    people = [
        (['Smith', 'John'], 'employee_001'),
        (['Smith', 'Alice'], 'employee_002'),
        (['Johnson', 'Bob'], 'employee_003'),
        (['Brown', 'Carol'], 'employee_004'),
        (['Smith', 'David'], 'employee_005')
    ]
    
    for values, value in people:
        compound_index.insert(values, value)
    
    # Search operations
    smith_results = compound_index.search_prefix(['Smith'])
    smith_john = compound_index.search(['Smith', 'John'])
    range_results = compound_index.range_query(['Johnson', 'A'], ['Smith', 'Z'])
    
    print(f"   All Smiths: {smith_results}")
    print(f"   Smith, John: {smith_john}")
    print(f"   Name range query: {len(range_results)} results")
    
    # Statistics
    compound_stats = compound_index.get_statistics()
    print(f"   Compound index stats: {compound_stats}")


def demonstrate_query_optimization():
    """Demonstrate query optimization"""
    print("\n" + "="*60)
    print("QUERY OPTIMIZATION DEMONSTRATION")
    print("="*60)
    
    # Setup database statistics
    db_stats = {
        'total_rows': 100000,
        'table_sizes': {
            'users': 50000,
            'orders': 100000,
            'products': 10000
        }
    }
    
    optimizer = QueryOptimizer(db_stats)
    
    # Create sample indexes
    indexes = {}
    indexes['user_id_idx'] = DatabaseIndex('user_id_idx', 'users', ['user_id'], IndexType.BTREE)
    indexes['email_idx'] = DatabaseIndex('email_idx', 'users', ['email'], IndexType.HASH)
    indexes['order_date_idx'] = DatabaseIndex('order_date_idx', 'orders', ['order_date'], IndexType.BTREE)
    indexes['compound_idx'] = DatabaseIndex('compound_idx', 'users', ['last_name', 'first_name'], IndexType.COMPOUND)
    
    # Simulate index data
    for index in indexes.values():
        for i in range(1000):
            index.insert(i, f"value_{i}")
    
    print("\n1. Query Optimization Examples...")
    
    # Example 1: Point query
    query1 = {
        'SELECT': ['user_id', 'email'],
        'FROM': 'users',
        'WHERE': {'user_id': 12345}
    }
    
    plan1 = optimizer.optimize_query(query1, indexes)
    print(f"\n   Query 1: {query1}")
    print(f"   Optimized plan cost: {plan1.cost:.4f}")
    print(f"   Access method: {plan1.access_methods[0].value}")
    print(f"   Estimated rows: {plan1.estimated_rows}")
    print(f"   Operations: {', '.join(plan1.operations)}")
    
    # Example 2: Range query
    query2 = {
        'SELECT': ['*'],
        'FROM': 'orders',
        'WHERE': {'order_date': {'$gte': '2023-01-01'}}
    }
    
    plan2 = optimizer.optimize_query(query2, indexes)
    print(f"\n   Query 2: {query2}")
    print(f"   Optimized plan cost: {plan2.cost:.4f}")
    print(f"   Access method: {plan2.access_methods[0].value}")
    print(f"   Estimated rows: {plan2.estimated_rows}")
    print(f"   Operations: {', '.join(plan2.operations)}")
    
    # Example 3: Update query
    query3 = {
        'UPDATE': 'users',
        'SET': {'last_login': '2023-11-03'},
        'WHERE': {'email': 'user@example.com'}
    }
    
    plan3 = optimizer.optimize_query(query3, indexes)
    print(f"\n   Query 3: {query3}")
    print(f"   Optimized plan cost: {plan3.cost:.4f}")
    print(f"   Access method: {plan3.access_methods[0].value}")
    print(f"   Estimated rows: {plan3.estimated_rows}")
    print(f"   Operations: {', '.join(plan3.operations)}")
    
    # Optimization statistics
    print("\n2. Optimization Statistics...")
    opt_stats = optimizer.get_optimization_statistics()
    print(f"   Total optimizations: {opt_stats['total_optimizations']}")
    print(f"   Average optimization time: {opt_stats['avg_optimization_time']:.6f}s")
    print(f"   Plan usage: {opt_stats['plan_usage']}")


def demonstrate_performance_comparison():
    """Demonstrate performance comparison between different access methods"""
    print("\n" + "="*60)
    print("PERFORMANCE COMPARISON DEMONSTRATION")
    print("="*60)
    
    print("\n1. Building indexes for comparison...")
    
    # Create different index types for the same data
    data_size = 10000
    keys = list(range(data_size))
    values = [f"value_{i}" for i in keys]
    
    # B-Tree
    print("   Building B-Tree index...")
    btree = BTreeIndex()
    start_time = time.time()
    for key, value in zip(keys, values):
        btree.insert(key, value)
    btree_build_time = time.time() - start_time
    
    # Hash
    print("   Building Hash index...")
    hash_index = HashIndex()
    start_time = time.time()
    for key, value in zip(keys, values):
        hash_index.insert(key, value)
    hash_build_time = time.time() - start_time
    
    print("\n2. Index Build Performance...")
    print(f"   B-Tree build time: {btree_build_time:.4f}s")
    print(f"   Hash build time: {hash_build_time:.4f}s")
    
    print("\n3. Query Performance Comparison...")
    
    # Point queries
    print("   Point queries (1000 searches)...")
    
    # B-Tree point queries
    start_time = time.time()
    for _ in range(1000):
        search_key = random.choice(keys)
        btree.search(search_key)
    btree_point_time = time.time() - start_time
    
    # Hash point queries
    start_time = time.time()
    for _ in range(1000):
        search_key = random.choice(keys)
        hash_index.search(search_key)
    hash_point_time = time.time() - start_time
    
    print(f"   B-Tree point queries: {btree_point_time:.4f}s ({1000/btree_point_time:.0f} queries/sec)")
    print(f"   Hash point queries: {hash_point_time:.4f}s ({1000/hash_point_time:.0f} queries/sec)")
    
    # Range queries (B-Tree only)
    print("   Range queries (100 ranges)...")
    start_time = time.time()
    for _ in range(100):
        start_key = random.randint(0, 8000)
        end_key = start_key + 500
        btree.range_query(start_key, end_key)
    btree_range_time = time.time() - start_time
    
    print(f"   B-Tree range queries: {btree_range_time:.4f}s ({100/btree_range_time:.0f} ranges/sec)")
    print(f"   Hash range queries: Not supported (unordered)")
    
    print("\n4. Index Size Comparison...")
    btree_stats = btree.get_statistics()
    hash_stats = hash_index.get_statistics()
    
    print(f"   B-Tree nodes: {btree_stats['nodes']}")
    print(f"   B-Tree depth: {btree_stats['depth']}")
    print(f"   Hash buckets: {hash_stats['bucket_count']}")
    print(f"   Hash load factor: {hash_stats['load_factor']:.3f}")
    
    print("\n5. Trade-offs Summary...")
    print("   B-Tree:")
    print("     + Supports range queries")
    print("     + Ordered data structure")
    print("     + Good for most operations")
    print("     - Slower point queries than hash")
    print()
    print("   Hash:")
    print("     + Very fast point queries O(1)")
    print("     + Efficient for exact matches")
    print("     - No range query support")
    print("     - No ordering guarantee")


def main():
    """Main demonstration function"""
    print("DATABASE QUERY OPTIMIZATION AND INDEXING DEMO")
    print("Demonstrating Index Types and Query Optimization")
    print("="*80)
    
    try:
        demonstrate_indexing()
        demonstrate_query_optimization()
        demonstrate_performance_comparison()
        
        print("\n" + "="*80)
        print("QUERY OPTIMIZATION DEMO COMPLETED")
        print("="*80)
        print("\nKey Concepts Demonstrated:")
        print("✓ B-Tree indexes for range queries")
        print("✓ Hash indexes for fast point queries")
        print("✓ Bitmap indexes for low cardinality")
        print("✓ Full-text indexes for document search")
        print("✓ Compound indexes for multi-column queries")
        print("✓ Cost-based query optimization")
        print("✓ Access method selection")
        print("✓ Performance comparison techniques")
        print("\nThis educational system provides hands-on learning of:")
        print("- Database index design and implementation")
        print("- Query optimization algorithms")
        print("- Access path selection")
        print("- Performance benchmarking")
        print("- Index trade-offs and selection criteria")
        print("- Cost estimation models")
        print("- Database system internals")
        
    except Exception as e:
        print(f"Error during demonstration: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    main()