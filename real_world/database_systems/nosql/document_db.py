"""
Educational NoSQL Document Database
Implements document-based storage with JSON support for CS education
"""

import json
import uuid
import hashlib
from typing import Dict, List, Any, Optional, Union
from dataclasses import dataclass, asdict
from collections import defaultdict
import time
import threading


@dataclass
class Document:
    """Represents a document in the database"""
    _id: str
    collection: str
    data: Dict[str, Any]
    metadata: Dict[str, Any]
    
    def to_json(self) -> str:
        """Convert document to JSON string"""
        return json.dumps({
            '_id': self._id,
            'collection': self.collection,
            'data': self.data,
            'metadata': self.metadata
        }, indent=2)
    
    @classmethod
    def from_json(cls, json_str: str) -> 'Document':
        """Create document from JSON string"""
        obj = json.loads(json_str)
        return cls(
            _id=obj['_id'],
            collection=obj['collection'],
            data=obj['data'],
            metadata=obj['metadata']
        )


@dataclass
class Index:
    """Represents an index on document fields"""
    name: str
    collection: str
    fields: List[str]
    index_type: str = 'B-TREE'  # B-TREE, HASH, COMPOUND
    is_unique: bool = False
    
    def get_key(self, doc: Document) -> Any:
        """Generate index key from document"""
        if len(self.fields) == 1:
            field = self.fields[0]
            return self._get_field_value(doc, field)
        else:
            return tuple(self._get_field_value(doc, field) for field in self.fields)
    
    def _get_field_value(self, doc: Document, field: str) -> Any:
        """Get field value from document (supports dot notation)"""
        keys = field.split('.')
        value = doc.data
        for key in keys:
            if isinstance(value, dict) and key in value:
                value = value[key]
            else:
                return None
        return value


class QueryOptimizer:
    """Optimizes NoSQL queries using indexes and query plans"""
    
    def __init__(self):
        self.query_cache = {}  # Cache for query results
        self.collection_stats = defaultdict(dict)
    
    def analyze_query(self, collection: str, query: Dict[str, Any]) -> Dict:
        """Analyze query for optimization opportunities"""
        plan = {
            'collection': collection,
            'query': query,
            'indexes_used': [],
            'scan_type': 'full',
            'estimated_cost': 0
        }
        
        # Analyze query predicates
        for field, value in query.items():
            if field.startswith('$'):
                continue  # Skip operators
            
            # Check if field is indexed
            # This would check against actual indexes in real implementation
            if self._is_field_indexed(collection, field):
                plan['indexes_used'].append(field)
                plan['scan_type'] = 'indexed'
                plan['estimated_cost'] = 1
            else:
                plan['estimated_cost'] += 100  # Full scan cost
        
        return plan
    
    def _is_field_indexed(self, collection: str, field: str) -> bool:
        """Check if field is indexed in collection"""
        # Simplified - would check actual indexes
        return field in ['_id', 'name', 'email', 'created_at']
    
    def generate_query_plan(self, collection: str, query: Dict[str, Any]) -> Dict:
        """Generate execution plan for query"""
        plan = {
            'type': 'collection_scan',
            'steps': []
        }
        
        # Add steps based on query structure
        if query:
            plan['type'] = 'indexed_scan'
            plan['steps'].append('Use index')
        
        plan['steps'].append('Filter documents')
        plan['steps'].append('Apply projection')
        plan['steps'].append('Sort results')
        
        return plan


class DocumentDB:
    """
    Educational NoSQL Document Database
    Supports collections, document storage, querying, and indexing
    """
    
    def __init__(self):
        self.collections = {}  # collection_name -> List[Document]
        self.indexes = defaultdict(dict)  # collection_name -> field -> Index
        self.query_optimizer = QueryOptimizer()
        self.lock = threading.RLock()
        self.document_count = 0
        self.collection_stats = defaultdict(lambda: {
            'document_count': 0,
            'avg_doc_size': 0,
            'index_count': 0
        })
    
    def create_collection(self, collection_name: str) -> bool:
        """Create a new collection"""
        with self.lock:
            if collection_name in self.collections:
                return False
            
            self.collections[collection_name] = []
            self.indexes[collection_name] = {}
            self.collection_stats[collection_name] = {
                'document_count': 0,
                'avg_doc_size': 0,
                'index_count': 0
            }
            return True
    
    def insert_document(self, collection: str, data: Dict[str, Any], 
                       metadata: Dict[str, Any] = None) -> str:
        """Insert a document into a collection"""
        with self.lock:
            # Generate unique document ID
            doc_id = str(uuid.uuid4())
            
            # Create metadata
            if metadata is None:
                metadata = {}
            
            metadata.update({
                'created_at': time.time(),
                'updated_at': time.time(),
                'version': 1,
                'checksum': hashlib.md5(json.dumps(data, sort_keys=True).encode()).hexdigest()
            })
            
            # Create document
            document = Document(
                _id=doc_id,
                collection=collection,
                data=data,
                metadata=metadata
            )
            
            # Add to collection
            if collection not in self.collections:
                self.create_collection(collection)
            
            self.collections[collection].append(document)
            self.document_count += 1
            
            # Update indexes
            self._update_indexes(collection, document)
            
            # Update collection statistics
            stats = self.collection_stats[collection]
            stats['document_count'] += 1
            stats['avg_doc_size'] = (stats['avg_doc_size'] * (stats['document_count'] - 1) + 
                                   len(json.dumps(data))) / stats['document_count']
            
            return doc_id
    
    def _update_indexes(self, collection: str, document: Document):
        """Update all indexes for a collection"""
        if collection in self.indexes:
            for index_name, index in self.indexes[collection].items():
                try:
                    key = index.get_key(document)
                    if key is not None:
                        # In real implementation, store in index structure
                        pass
                except Exception:
                    # Index update failed, continue
                    pass
    
    def find_by_id(self, collection: str, doc_id: str) -> Optional[Document]:
        """Find document by ID"""
        with self.lock:
            if collection not in self.collections:
                return None
            
            for doc in self.collections[collection]:
                if doc._id == doc_id:
                    return doc
            
            return None
    
    def find_one(self, collection: str, query: Dict[str, Any] = None) -> Optional[Document]:
        """Find one document matching query"""
        results = self.find(collection, query, limit=1)
        return results[0] if results else None
    
    def find(self, collection: str, query: Dict[str, Any] = None, 
             projection: Dict[str, int] = None, sort: List[str] = None,
             limit: int = None, skip: int = 0) -> List[Document]:
        """Find documents matching query"""
        with self.lock:
            if collection not in self.collections:
                return []
            
            # Generate query plan for optimization
            plan = self.query_optimizer.analyze_query(collection, query or {})
            
            # Filter documents based on query
            results = []
            for doc in self.collections[collection]:
                if self._matches_query(doc, query):
                    results.append(doc)
            
            # Apply projection
            if projection:
                results = self._apply_projection(results, projection)
            
            # Apply sorting
            if sort:
                results = self._apply_sorting(results, sort)
            
            # Apply skip and limit
            if skip:
                results = results[skip:]
            if limit:
                results = results[:limit]
            
            return results
    
    def _matches_query(self, doc: Document, query: Dict[str, Any]) -> bool:
        """Check if document matches query criteria"""
        if not query:
            return True
        
        for key, value in query.items():
            if key.startswith('$'):
                # Handle query operators
                if not self._apply_operator(doc, key, value):
                    return False
            else:
                # Simple field comparison
                if not self._compare_field(doc, key, value):
                    return False
        
        return True
    
    def _compare_field(self, doc: Document, field: str, value: Any) -> bool:
        """Compare document field with value"""
        if field == '_id':
            return doc._id == value
        elif field in doc.data:
            return doc.data[field] == value
        elif field.startswith('metadata.'):
            # Access metadata fields
            meta_field = field.replace('metadata.', '')
            return doc.metadata.get(meta_field) == value
        return False
    
    def _apply_operator(self, doc: Document, operator: str, value: Any) -> bool:
        """Apply query operators"""
        if operator == '$and':
            return all(self._matches_query(doc, cond) for cond in value)
        elif operator == '$or':
            return any(self._matches_query(doc, cond) for cond in value)
        elif operator == '$not':
            return not self._matches_query(doc, value)
        elif operator == '$gt':
            field, threshold = list(value.items())[0]
            return self._get_field_value(doc, field) > threshold
        elif operator == '$gte':
            field, threshold = list(value.items())[0]
            return self._get_field_value(doc, field) >= threshold
        elif operator == '$lt':
            field, threshold = list(value.items())[0]
            return self._get_field_value(doc, field) < threshold
        elif operator == '$lte':
            field, threshold = list(value.items())[0]
            return self._get_field_value(doc, field) <= threshold
        elif operator == '$ne':
            field, target = list(value.items())[0]
            return self._get_field_value(doc, field) != target
        elif operator == '$in':
            field, values = list(value.items())[0]
            field_value = self._get_field_value(doc, field)
            return field_value in values
        elif operator == '$regex':
            field, pattern = list(value.items())[0]
            import re
            field_value = str(self._get_field_value(doc, field) or '')
            return bool(re.search(pattern, field_value, re.IGNORECASE))
        
        return True
    
    def _get_field_value(self, doc: Document, field: str) -> Any:
        """Get field value from document (supports nested fields)"""
        if field == '_id':
            return doc._id
        elif field in doc.data:
            return doc.data[field]
        elif field.startswith('metadata.'):
            meta_field = field.replace('metadata.', '')
            return doc.metadata.get(meta_field)
        elif '.' in field:
            # Handle nested fields
            keys = field.split('.')
            value = doc.data
            for key in keys:
                if isinstance(value, dict) and key in value:
                    value = value[key]
                else:
                    return None
            return value
        return None
    
    def _apply_projection(self, results: List[Document], projection: Dict[str, int]) -> List[Document]:
        """Apply field projection to results"""
        projected_results = []
        
        for doc in results:
            projected_doc = Document(
                _id=doc._id,
                collection=doc.collection,
                data={},
                metadata=doc.metadata.copy()
            )
            
            for field, include in projection.items():
                if include:
                    value = self._get_field_value(doc, field)
                    if value is not None:
                        projected_doc.data[field] = value
            
            projected_results.append(projected_doc)
        
        return projected_results
    
    def _apply_sorting(self, results: List[Document], sort: List[str]) -> List[Document]:
        """Apply sorting to results"""
        def sort_key(doc):
            key_values = []
            for field in sort:
                descending = field.startswith('-')
                actual_field = field[1:] if descending else field
                value = self._get_field_value(doc, actual_field) or ''
                
                if descending:
                    key_values.append((-value if isinstance(value, (int, float)) else value, doc._id))
                else:
                    key_values.append((value, doc._id))
            
            return tuple(key_values)
        
        return sorted(results, key=sort_key)
    
    def update_one(self, collection: str, query: Dict[str, Any], 
                   update: Dict[str, Any]) -> bool:
        """Update one document matching query"""
        with self.lock:
            results = self.find(collection, query, limit=1)
            if not results:
                return False
            
            doc = results[0]
            
            # Apply update operators
            for field, value in update.items():
                if field.startswith('$set'):
                    # Set field value
                    actual_field = list(value.keys())[0]
                    actual_value = list(value.values())[0]
                    doc.data[actual_field] = actual_value
                elif field.startswith('$unset'):
                    # Unset field value
                    actual_field = list(value.keys())[0]
                    if actual_field in doc.data:
                        del doc.data[actual_field]
                elif field.startswith('$inc'):
                    # Increment field value
                    actual_field = list(value.keys())[0]
                    increment = list(value.values())[0]
                    current_value = doc.data.get(actual_field, 0)
                    doc.data[actual_field] = current_value + increment
                elif field == '$push':
                    # Push to array
                    actual_field = list(value.keys())[0]
                    actual_value = list(value.values())[0]
                    if actual_field not in doc.data:
                        doc.data[actual_field] = []
                    doc.data[actual_field].append(actual_value)
            
            # Update metadata
            doc.metadata['updated_at'] = time.time()
            doc.metadata['version'] = doc.metadata.get('version', 1) + 1
            
            # Update indexes
            self._update_indexes(collection, doc)
            
            return True
    
    def update_many(self, collection: str, query: Dict[str, Any], 
                    update: Dict[str, Any]) -> int:
        """Update multiple documents matching query"""
        with self.lock:
            updated_count = 0
            results = self.find(collection, query)
            
            for doc in results:
                if self.update_one(collection, {'_id': doc._id}, update):
                    updated_count += 1
            
            return updated_count
    
    def delete_one(self, collection: str, query: Dict[str, Any]) -> bool:
        """Delete one document matching query"""
        with self.lock:
            results = self.find(collection, query, limit=1)
            if not results:
                return False
            
            doc = results[0]
            self.collections[collection].remove(doc)
            self.document_count -= 1
            
            # Update collection statistics
            stats = self.collection_stats[collection]
            stats['document_count'] -= 1
            
            return True
    
    def delete_many(self, collection: str, query: Dict[str, Any]) -> int:
        """Delete multiple documents matching query"""
        with self.lock:
            results = self.find(collection, query)
            deleted_count = len(results)
            
            for doc in results:
                if doc in self.collections[collection]:
                    self.collections[collection].remove(doc)
                    self.document_count -= 1
            
            # Update collection statistics
            stats = self.collection_stats[collection]
            stats['document_count'] -= deleted_count
            
            return deleted_count
    
    def create_index(self, collection: str, fields: List[str], 
                    index_name: str = None, index_type: str = 'B-TREE', 
                    unique: bool = False) -> bool:
        """Create index on specified fields"""
        with self.lock:
            if collection not in self.collections:
                return False
            
            if index_name is None:
                index_name = '_'.join(fields) + '_idx'
            
            index = Index(
                name=index_name,
                collection=collection,
                fields=fields,
                index_type=index_type,
                is_unique=unique
            )
            
            if collection not in self.indexes:
                self.indexes[collection] = {}
            
            self.indexes[collection][index_name] = index
            
            # Update collection statistics
            stats = self.collection_stats[collection]
            stats['index_count'] += 1
            
            return True
    
    def drop_collection(self, collection: str) -> bool:
        """Drop a collection"""
        with self.lock:
            if collection not in self.collections:
                return False
            
            # Remove collection
            del self.collections[collection]
            del self.indexes[collection]
            del self.collection_stats[collection]
            
            # Update global statistics
            self.document_count = sum(
                len(docs) for docs in self.collections.values()
            )
            
            return True
    
    def get_collection_stats(self, collection: str) -> Optional[Dict]:
        """Get collection statistics"""
        if collection not in self.collection_stats:
            return None
        
        stats = self.collection_stats[collection].copy()
        stats['name'] = collection
        return stats
    
    def list_collections(self) -> List[str]:
        """List all collections"""
        return list(self.collections.keys())
    
    def aggregate(self, collection: str, pipeline: List[Dict[str, Any]]) -> List[Document]:
        """Execute aggregation pipeline"""
        with self.lock:
            if collection not in self.collections:
                return []
            
            # Get all documents initially
            documents = self.collections[collection].copy()
            
            # Apply aggregation stages
            for stage in pipeline:
                stage_type, stage_config = list(stage.items())[0]
                
                if stage_type == '$match':
                    documents = [doc for doc in documents 
                               if self._matches_query(doc, stage_config)]
                elif stage_type == '$project':
                    documents = self._apply_projection(documents, stage_config)
                elif stage_type == '$sort':
                    documents = self._apply_sorting(documents, 
                                                  [field for field, _ in stage_config.items()])
                elif stage_type == '$limit':
                    documents = documents[:stage_config]
                elif stage_type == '$skip':
                    documents = documents[stage_config:]
                elif stage_type == '$group':
                    # Simple grouping implementation
                    documents = self._apply_grouping(documents, stage_config)
            
            return documents
    
    def _apply_grouping(self, documents: List[Document], group_config: Dict) -> List[Document]:
        """Apply grouping operation"""
        grouped = defaultdict(list)
        
        # Get group by field
        group_by = group_config.get('_id', None)
        if group_by:
            for doc in documents:
                key = self._get_field_value(doc, group_by)
                grouped[key].append(doc)
        else:
            grouped['all'] = documents
        
        # Calculate aggregate values
        results = []
        for key, docs in grouped.items():
            group_doc = Document(
                _id=str(uuid.uuid4()),
                collection=docs[0].collection if docs else '',
                data={'_id': key},
                metadata={}
            )
            
            # Calculate aggregates like $sum, $avg, $max, $min
            for field, agg_func in group_config.items():
                if field == '_id':
                    continue
                
                if agg_func.startswith('$'):
                    agg_name = agg_func[1:]  # Remove $
                    if agg_name == 'sum':
                        total = sum(float(doc.data.get(field, 0)) for doc in docs)
                        group_doc.data[field] = total
                    elif agg_name == 'avg':
                        values = [float(doc.data.get(field, 0)) for doc in docs]
                        group_doc.data[field] = sum(values) / len(values) if values else 0
                    elif agg_name == 'max':
                        values = [float(doc.data.get(field, 0)) for doc in docs]
                        group_doc.data[field] = max(values) if values else 0
                    elif agg_name == 'min':
                        values = [float(doc.data.get(field, 0)) for doc in docs]
                        group_doc.data[field] = min(values) if values else 0
                    elif agg_name == 'count':
                        group_doc.data[field] = len(docs)
            
            results.append(group_doc)
        
        return results
    
    def export_collection(self, collection: str) -> str:
        """Export collection to JSON"""
        if collection not in self.collections:
            return "{}"
        
        export_data = {
            'collection': collection,
            'document_count': len(self.collections[collection]),
            'documents': [doc.to_json() for doc in self.collections[collection]]
        }
        
        return json.dumps(export_data, indent=2)
    
    def import_collection(self, json_data: str) -> bool:
        """Import collection from JSON"""
        try:
            data = json.loads(json_data)
            
            collection = data['collection']
            self.create_collection(collection)
            
            for doc_json in data['documents']:
                doc = Document.from_json(doc_json)
                if doc.collection == collection:
                    self.collections[collection].append(doc)
                    self._update_indexes(collection, doc)
            
            return True
        except Exception:
            return False