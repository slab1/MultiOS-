"""
Educational Relational Database Engine
Implements ACID properties and basic SQL operations for CS education
"""

import json
import threading
import time
from typing import Dict, List, Any, Optional, Tuple
from dataclasses import dataclass, asdict
from enum import Enum
import hashlib


class TransactionState(Enum):
    """Transaction states for ACID compliance"""
    ACTIVE = "ACTIVE"
    PREPARED = "PREPARED"
    COMMITTED = "COMMITTED"
    ABORTED = "ABORTED"


@dataclass
class Table:
    """Schema definition for a table"""
    name: str
    columns: Dict[str, str]  # column_name -> data_type
    primary_key: str
    indexes: Dict[str, str] = None  # index_name -> column_name


@dataclass
class DatabaseObject:
    """Base class for database objects"""
    name: str
    table_name: str
    data: Dict[str, Any]


class Transaction:
    """Transaction manager for ACID properties"""
    
    def __init__(self, tx_id: str):
        self.tx_id = tx_id
        self.state = TransactionState.ACTIVE
        self.operations = []
        self.start_time = time.time()
        self.lock_timeout = 30  # 30 seconds
        
    def add_operation(self, operation: str, table: str, data: Any):
        """Add operation to transaction"""
        self.operations.append({
            'operation': operation,
            'table': table,
            'data': data,
            'timestamp': time.time()
        })
    
    def commit(self):
        """Commit transaction"""
        self.state = TransactionState.COMMITTED
    
    def abort(self):
        """Abort transaction"""
        self.state = TransactionState.ABORTED
        self.operations.clear()


class LockManager:
    """Manages table-level locks for concurrency control"""
    
    def __init__(self):
        self.locks = {}  # table_name -> set of transaction_ids
        self.lock_waiters = {}  # table_name -> set of transaction_ids
    
    def acquire_lock(self, table: str, tx_id: str, lock_type: str = 'SHARED'):
        """Acquire lock on table"""
        if table not in self.locks:
            self.locks[table] = set()
            self.lock_waiters[table] = set()
        
        if lock_type == 'EXCLUSIVE':
            if self.locks[table]:
                self.lock_waiters[table].add(tx_id)
                return False
            self.locks[table].add(tx_id)
            return True
        else:  # SHARED
            if tx_id in self.lock_waiters[table]:
                return False
            self.locks[table].add(tx_id)
            return True
    
    def release_lock(self, table: str, tx_id: str):
        """Release lock on table"""
        if table in self.locks:
            self.locks[table].discard(tx_id)
            if not self.locks[table] and table in self.lock_waiters:
                self.lock_waiters[table].clear()


class BufferManager:
    """Manages data pages in memory"""
    
    def __init__(self, buffer_size: int = 100):
        self.buffer_size = buffer_size
        self.pages = {}  # page_id -> data
        self.page_lru = []  # LRU list
        self.dirty_pages = set()  # Dirty pages
        self.lock = threading.Lock()
    
    def get_page(self, page_id: str) -> Optional[Dict]:
        """Get page from buffer"""
        with self.lock:
            if page_id in self.pages:
                # Move to end (most recently used)
                self.page_lru.remove(page_id)
                self.page_lru.append(page_id)
                return self.pages[page_id]
            return None
    
    def load_page(self, page_id: str, data: Dict):
        """Load page into buffer"""
        with self.lock:
            if len(self.pages) >= self.buffer_size:
                # Evict least recently used page
                lru_page = self.page_lru.pop(0)
                if lru_page in self.dirty_pages:
                    # Write back to disk (simplified)
                    pass
                del self.pages[lru_page]
            
            self.pages[page_id] = data
            self.page_lru.append(page_id)
    
    def mark_dirty(self, page_id: str):
        """Mark page as dirty"""
        self.dirty_pages.add(page_id)


class RelationalEngine:
    """
    Educational Relational Database Engine
    Implements ACID properties and basic SQL operations
    """
    
    def __init__(self):
        self.tables = {}  # table_name -> Table
        self.data = {}    # table_name -> List[Dict[str, Any]]
        self.transactions = {}  # tx_id -> Transaction
        self.lock_manager = LockManager()
        self.buffer_manager = BufferManager()
        self.undo_log = []  # For recovery
        self.redo_log = []  # For recovery
        self.lock = threading.Lock()
        self.auto_commit = True
        
    def create_table(self, table_name: str, columns: Dict[str, str], primary_key: str) -> bool:
        """Create a new table"""
        with self.lock:
            if table_name in self.tables:
                return False
            
            self.tables[table_name] = Table(
                name=table_name,
                columns=columns,
                primary_key=primary_key,
                indexes={}
            )
            self.data[table_name] = []
            return True
    
    def begin_transaction(self, tx_id: str) -> bool:
        """Begin a new transaction"""
        with self.lock:
            if tx_id in self.transactions:
                return False
            self.transactions[tx_id] = Transaction(tx_id)
            return True
    
    def commit(self, tx_id: str) -> bool:
        """Commit a transaction (ACID - Atomicity, Consistency)"""
        with self.lock:
            if tx_id not in self.transactions:
                return False
            
            tx = self.transactions[tx_id]
            if tx.state != TransactionState.ACTIVE:
                return False
            
            # Write commit record to log
            self.undo_log.append({
                'type': 'COMMIT',
                'tx_id': tx_id,
                'timestamp': time.time(),
                'operations': tx.operations.copy()
            })
            
            tx.commit()
            
            # Release all locks
            for operation in tx.operations:
                self.lock_manager.release_lock(operation['table'], tx_id)
            
            del self.transactions[tx_id]
            return True
    
    def rollback(self, tx_id: str) -> bool:
        """Rollback a transaction (ACID - Durability)"""
        with self.lock:
            if tx_id not in self.transactions:
                return False
            
            tx = self.transactions[tx_id]
            if tx.state != TransactionState.ACTIVE:
                return False
            
            # Undo all operations
            for operation in reversed(tx.operations):
                if operation['operation'] == 'INSERT':
                    self._undo_insert(operation)
                elif operation['operation'] == 'UPDATE':
                    self._undo_update(operation)
                elif operation['operation'] == 'DELETE':
                    self._undo_delete(operation)
            
            # Write abort record to log
            self.undo_log.append({
                'type': 'ABORT',
                'tx_id': tx_id,
                'timestamp': time.time(),
                'operations': tx.operations.copy()
            })
            
            tx.abort()
            
            # Release all locks
            for operation in tx.operations:
                self.lock_manager.release_lock(operation['table'], tx_id)
            
            del self.transactions[tx_id]
            return True
    
    def _undo_insert(self, operation: Dict):
        """Undo an INSERT operation"""
        table = operation['table']
        data = operation['data']
        if table in self.data:
            self.data[table] = [row for row in self.data[table] if row != data]
    
    def _undo_update(self, operation: Dict):
        """Undo an UPDATE operation"""
        table = operation['table']
        old_data = operation['data']['old']
        if table in self.data:
            for i, row in enumerate(self.data[table]):
                if self._matches_row(row, old_data):
                    self.data[table][i] = old_data
                    break
    
    def _undo_delete(self, operation: Dict):
        """Undo a DELETE operation"""
        table = operation['table']
        deleted_row = operation['data']
        if table not in self.data:
            self.data[table] = []
        self.data[table].append(deleted_row)
    
    def _matches_row(self, row: Dict, target: Dict) -> bool:
        """Check if row matches target data"""
        for key, value in target.items():
            if row.get(key) != value:
                return False
        return True
    
    def insert(self, tx_id: str, table_name: str, data: Dict[str, Any]) -> bool:
        """Insert a row into a table (with transaction support)"""
        with self.lock:
            if tx_id not in self.transactions:
                return False
            
            tx = self.transactions[tx_id]
            if tx.state != TransactionState.ACTIVE:
                return False
            
            if table_name not in self.tables:
                return False
            
            # Acquire exclusive lock
            if not self.lock_manager.acquire_lock(table_name, tx_id, 'EXCLUSIVE'):
                return False
            
            # Check primary key constraint
            pk_col = self.tables[table_name].primary_key
            if pk_col in data:
                for row in self.data[table_name]:
                    if row.get(pk_col) == data[pk_col]:
                        self.lock_manager.release_lock(table_name, tx_id)
                        return False
            
            # Insert the data
            self.data[table_name].append(data)
            
            # Add to undo log for recovery
            self.undo_log.append({
                'type': 'INSERT',
                'tx_id': tx_id,
                'table': table_name,
                'data': data.copy(),
                'timestamp': time.time()
            })
            
            # Record operation for transaction
            tx.add_operation('INSERT', table_name, data.copy())
            
            return True
    
    def update(self, tx_id: str, table_name: str, condition: Dict, new_data: Dict) -> bool:
        """Update rows in a table (with transaction support)"""
        with self.lock:
            if tx_id not in self.transactions:
                return False
            
            tx = self.transactions[tx_id]
            if tx.state != TransactionState.ACTIVE:
                return False
            
            if table_name not in self.tables:
                return False
            
            # Acquire exclusive lock
            if not self.lock_manager.acquire_lock(table_name, tx_id, 'EXCLUSIVE'):
                return False
            
            updated_rows = 0
            for i, row in enumerate(self.data[table_name]):
                if self._matches_condition(row, condition):
                    old_row = row.copy()
                    row.update(new_data)
                    updated_rows += 1
                    
                    # Add to undo log
                    self.undo_log.append({
                        'type': 'UPDATE',
                        'tx_id': tx_id,
                        'table': table_name,
                        'data': {'old': old_row, 'new': row.copy()},
                        'timestamp': time.time()
                    })
                    
                    # Record operation for transaction
                    tx.add_operation('UPDATE', table_name, {'old': old_row, 'new': row.copy()})
            
            return updated_rows > 0
    
    def delete(self, tx_id: str, table_name: str, condition: Dict) -> bool:
        """Delete rows from a table (with transaction support)"""
        with self.lock:
            if tx_id not in self.transactions:
                return False
            
            tx = self.transactions[tx_id]
            if tx.state != TransactionState.ACTIVE:
                return False
            
            if table_name not in self.tables:
                return False
            
            # Acquire exclusive lock
            if not self.lock_manager.acquire_lock(table_name, tx_id, 'EXCLUSIVE'):
                return False
            
            deleted_rows = []
            self.data[table_name] = [
                row for row in self.data[table_name]
                if not self._matches_condition(row, condition) or (
                    deleted_rows.append(row) or False
                )
            ]
            
            for deleted_row in deleted_rows:
                # Add to undo log
                self.undo_log.append({
                    'type': 'DELETE',
                    'tx_id': tx_id,
                    'table': table_name,
                    'data': deleted_row,
                    'timestamp': time.time()
                })
                
                # Record operation for transaction
                tx.add_operation('DELETE', table_name, deleted_row)
            
            return len(deleted_rows) > 0
    
    def select(self, tx_id: str, table_name: str, condition: Dict = None, 
               columns: List[str] = None) -> List[Dict[str, Any]]:
        """Select rows from a table (with transaction support)"""
        with self.lock:
            if tx_id not in self.transactions:
                return []
            
            tx = self.transactions[tx_id]
            if tx.state != TransactionState.ACTIVE:
                return []
            
            if table_name not in self.tables:
                return []
            
            # Acquire shared lock
            if not self.lock_manager.acquire_lock(table_name, tx_id, 'SHARED'):
                return []
            
            # Filter and select data
            result = []
            for row in self.data[table_name]:
                if condition is None or self._matches_condition(row, condition):
                    if columns:
                        filtered_row = {col: row.get(col) for col in columns if col in row}
                    else:
                        filtered_row = row.copy()
                    result.append(filtered_row)
            
            return result
    
    def _matches_condition(self, row: Dict, condition: Dict) -> bool:
        """Check if row matches condition"""
        for key, value in condition.items():
            if row.get(key) != value:
                return False
        return True
    
    def create_index(self, table_name: str, index_name: str, column: str) -> bool:
        """Create an index on a column"""
        with self.lock:
            if table_name not in self.tables:
                return False
            
            table = self.tables[table_name]
            if column not in table.columns:
                return False
            
            table.indexes[index_name] = column
            return True
    
    def get_table_schema(self, table_name: str) -> Optional[Dict]:
        """Get table schema"""
        if table_name not in self.tables:
            return None
        
        table = self.tables[table_name]
        return {
            'name': table.name,
            'columns': table.columns,
            'primary_key': table.primary_key,
            'indexes': table.indexes
        }
    
    def get_all_tables(self) -> List[str]:
        """Get all table names"""
        return list(self.tables.keys())
    
    def get_transaction_status(self, tx_id: str) -> Optional[str]:
        """Get transaction status"""
        if tx_id in self.transactions:
            return self.transactions[tx_id].state.value
        return None
    
    def recover(self):
        """Recover database using undo log"""
        with self.lock:
            print("Starting recovery...")
            for log_entry in self.undo_log:
                if log_entry['type'] == 'ABORT':
                    tx_id = log_entry['tx_id']
                    # Undo all operations
                    for operation in reversed(log_entry['operations']):
                        if operation['operation'] == 'INSERT':
                            self._undo_insert(operation)
                        elif operation['operation'] == 'UPDATE':
                            self._undo_update(operation)
                        elif operation['operation'] == 'DELETE':
                            self._undo_delete(operation)
            
            # Clear undo log after recovery
            self.undo_log.clear()
            print("Recovery completed")