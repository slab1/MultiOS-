"""
SQL Parser for Educational Database
Parses basic SQL statements for teaching database concepts
"""

import re
from typing import List, Dict, Any, Optional, Tuple
from dataclasses import dataclass


class SQLToken:
    """Represents a SQL token"""
    def __init__(self, token_type: str, value: str, position: int):
        self.token_type = token_type
        self.value = value
        self.position = position
    
    def __repr__(self):
        return f"Token({self.token_type}, '{self.value}', {self.position})"


class SQLTokenizer:
    """Tokenizes SQL input into tokens"""
    
    def __init__(self):
        self.keywords = {
            'SELECT', 'FROM', 'WHERE', 'INSERT', 'INTO', 'VALUES',
            'UPDATE', 'SET', 'DELETE', 'CREATE', 'TABLE', 'PRIMARY', 'KEY',
            'INDEX', 'ON', 'AND', 'OR', 'NOT', '=', '!=', '<>', '<', '>', '<=', '>=',
            'BEGIN', 'TRANSACTION', 'COMMIT', 'ROLLBACK', 'NULL', 'INT', 'VARCHAR',
            'TEXT', 'REAL', 'BOOLEAN', 'DATE', 'TIMESTAMP', 'BEGIN', 'END'
        }
    
    def tokenize(self, sql: str) -> List[SQLToken]:
        """Tokenize SQL string into tokens"""
        tokens = []
        position = 0
        sql = sql.strip()
        
        while position < len(sql):
            # Skip whitespace
            while position < len(sql) and sql[position].isspace():
                position += 1
            
            if position >= len(sql):
                break
            
            char = sql[position]
            
            # Handle string literals
            if char in ["'", '"']:
                start_pos = position
                position += 1
                value = ""
                
                while position < len(sql) and sql[position] != char:
                    if sql[position] == '\\':  # Escape sequence
                        position += 1
                        if position < len(sql):
                            value += sql[position]
                            position += 1
                    else:
                        value += sql[position]
                        position += 1
                
                if position < len(sql):
                    position += 1  # Skip closing quote
                
                tokens.append(SQLToken('STRING', value, start_pos))
            
            # Handle identifiers and keywords
            elif char.isalpha() or char == '_':
                start_pos = position
                value = ""
                
                while position < len(sql) and (sql[position].isalnum() or sql[position] == '_'):
                    value += sql[position]
                    position += 1
                
                # Check if it's a keyword
                if value.upper() in self.keywords:
                    tokens.append(SQLToken('KEYWORD', value.upper(), start_pos))
                else:
                    tokens.append(SQLToken('IDENTIFIER', value, start_pos))
            
            # Handle numbers
            elif char.isdigit():
                start_pos = position
                value = ""
                
                while position < len(sql) and sql[position].isdigit():
                    value += sql[position]
                    position += 1
                
                tokens.append(SQLToken('NUMBER', value, start_pos))
            
            # Handle operators and punctuation
            elif char in ['(', ')', ',', ';']:
                tokens.append(SQLToken('PUNCTUATION', char, position))
                position += 1
            
            elif char in ['=', '!', '<', '>']:
                start_pos = position
                value = char
                position += 1
                
                if position < len(sql) and sql[position] in ['=', '<', '>']:
                    value += sql[position]
                    position += 1
                
                tokens.append(SQLToken('OPERATOR', value, start_pos))
            
            else:
                raise SyntaxError(f"Unexpected character '{char}' at position {position}")
        
        return tokens


@dataclass
class ParsedQuery:
    """Represents a parsed SQL query"""
    query_type: str
    table_name: str
    columns: List[str]
    where_clause: Optional[Dict] = None
    insert_data: Optional[Dict] = None
    update_data: Optional[Dict] = None
    table_schema: Optional[Dict] = None


class SQLParser:
    """Parses SQL tokens into structured query objects"""
    
    def __init__(self):
        self.tokenizer = SQLTokenizer()
    
    def parse(self, sql: str) -> ParsedQuery:
        """Parse SQL string into a structured query"""
        tokens = self.tokenizer.tokenize(sql)
        token_iter = iter(tokens)
        
        # Look for first keyword
        first_token = self._get_next_token(token_iter)
        if not first_token or first_token.token_type != 'KEYWORD':
            raise SyntaxError("SQL statement must start with a keyword")
        
        query_type = first_token.value
        
        if query_type == 'SELECT':
            return self._parse_select(token_iter)
        elif query_type == 'INSERT':
            return self._parse_insert(token_iter)
        elif query_type == 'UPDATE':
            return self._parse_update(token_iter)
        elif query_type == 'DELETE':
            return self._parse_delete(token_iter)
        elif query_type == 'CREATE':
            return self._parse_create(token_iter)
        elif query_type == 'BEGIN':
            return self._parse_begin(token_iter)
        elif query_type == 'COMMIT':
            return self._parse_commit(token_iter)
        elif query_type == 'ROLLBACK':
            return self._parse_rollback(token_iter)
        else:
            raise SyntaxError(f"Unsupported SQL statement: {query_type}")
    
    def _get_next_token(self, token_iter) -> Optional[SQLToken]:
        """Get next token from iterator"""
        try:
            return next(token_iter)
        except StopIteration:
            return None
    
    def _parse_select(self, token_iter) -> ParsedQuery:
        """Parse SELECT statement"""
        # Parse columns
        columns = []
        token = self._get_next_token(token_iter)
        
        if not token or token.token_type != 'KEYWORD' or token.value != 'FROM':
            raise SyntaxError("SELECT must be followed by FROM")
        
        # Next token should be table name or *
        token = self._get_next_token(token_iter)
        if not token:
            raise SyntaxError("SELECT requires a table name")
        
        if token.value == '*':
            columns = ['*']
        else:
            columns = [token.value]
            # Handle comma-separated columns
            token = self._get_next_token(token_iter)
            while token and token.token_type == 'PUNCTUATION' and token.value == ',':
                token = self._get_next_token(token_iter)
                if token:
                    columns.append(token.value)
                    token = self._get_next_token(token_iter)
        
        if not token or token.token_type != 'KEYWORD':
            raise SyntaxError("Table name must be followed by FROM or WHERE")
        
        table_name = token.value if token.token_type == 'IDENTIFIER' else None
        
        # Parse WHERE clause (optional)
        where_clause = None
        if token and token.token_type == 'KEYWORD' and token.value == 'WHERE':
            where_clause = self._parse_where_clause(token_iter)
        
        return ParsedQuery(
            query_type='SELECT',
            table_name=table_name,
            columns=columns,
            where_clause=where_clause
        )
    
    def _parse_insert(self, token_iter) -> ParsedQuery:
        """Parse INSERT statement"""
        # INTO keyword
        token = self._get_next_token(token_iter)
        if not token or token.token_type != 'KEYWORD' or token.value != 'INTO':
            raise SyntaxError("INSERT must be followed by INTO")
        
        # Table name
        token = self._get_next_token(token_iter)
        if not token or token.token_type != 'IDENTIFIER':
            raise SyntaxError("INSERT INTO requires table name")
        table_name = token.value
        
        # VALUES keyword
        token = self._get_next_token(token_iter)
        if not token or token.token_type != 'KEYWORD' or token.value != 'VALUES':
            raise SyntaxError("INSERT requires VALUES clause")
        
        # Opening parenthesis
        token = self._get_next_token(token_iter)
        if not token or token.token_type != 'PUNCTUATION' or token.value != '(':
            raise SyntaxError("INSERT VALUES requires opening parenthesis")
        
        # Parse values
        insert_data = {}
        column_count = 0
        
        while True:
            token = self._get_next_token(token_iter)
            if not token:
                raise SyntaxError("Unterminated INSERT statement")
            
            if token.token_type == 'PUNCTUATION' and token.value == ')':
                break
            
            # Parse column value
            if token.token_type == 'KEYWORD' and token.value == 'NULL':
                value = None
            elif token.token_type == 'NUMBER':
                value = int(token.value) if '.' not in token.value else float(token.value)
            elif token.token_type == 'STRING':
                value = token.value
            else:
                raise SyntaxError(f"Invalid value: {token.value}")
            
            insert_data[f'col{column_count}'] = value
            column_count += 1
            
            # Next should be comma or closing paren
            token = self._get_next_token(token_iter)
            if token and token.token_type == 'PUNCTUATION' and token.value == ',':
                continue
            elif token and token.token_type == 'PUNCTUATION' and token.value == ')':
                break
            else:
                raise SyntaxError("Invalid INSERT syntax")
        
        return ParsedQuery(
            query_type='INSERT',
            table_name=table_name,
            columns=list(insert_data.keys()),
            insert_data=insert_data
        )
    
    def _parse_update(self, token_iter) -> ParsedQuery:
        """Parse UPDATE statement"""
        # Table name
        token = self._get_next_token(token_iter)
        if not token or token.token_type != 'IDENTIFIER':
            raise SyntaxError("UPDATE requires table name")
        table_name = token.value
        
        # SET keyword
        token = self._get_next_token(token_iter)
        if not token or token.token_type != 'KEYWORD' or token.value != 'SET':
            raise SyntaxError("UPDATE requires SET clause")
        
        # Parse assignments
        update_data = {}
        
        while True:
            # Column name
            token = self._get_next_token(token_iter)
            if not token or token.token_type != 'IDENTIFIER':
                raise SyntaxError("Invalid column name in SET clause")
            column = token.value
            
            # Equals sign
            token = self._get_next_token(token_iter)
            if not token or token.token_type != 'OPERATOR' or token.value != '=':
                raise SyntaxError("SET clause requires = operator")
            
            # Value
            token = self._get_next_token(token_iter)
            if not token:
                raise SyntaxError("Unterminated SET clause")
            
            if token.token_type == 'KEYWORD' and token.value == 'NULL':
                value = None
            elif token.token_type == 'NUMBER':
                value = int(token.value) if '.' not in token.value else float(token.value)
            elif token.token_type == 'STRING':
                value = token.value
            else:
                raise SyntaxError(f"Invalid value: {token.value}")
            
            update_data[column] = value
            
            # Check for WHERE clause
            token = self._get_next_token(token_iter)
            if token and token.token_type == 'KEYWORD' and token.value == 'WHERE':
                where_clause = self._parse_where_clause(token_iter)
                return ParsedQuery(
                    query_type='UPDATE',
                    table_name=table_name,
                    columns=list(update_data.keys()),
                    update_data=update_data,
                    where_clause=where_clause
                )
            elif token and token.token_type == 'PUNCTUATION' and token.value == ',':
                continue
            else:
                break
        
        return ParsedQuery(
            query_type='UPDATE',
            table_name=table_name,
            columns=list(update_data.keys()),
            update_data=update_data
        )
    
    def _parse_delete(self, token_iter) -> ParsedQuery:
        """Parse DELETE statement"""
        # FROM keyword
        token = self._get_next_token(token_iter)
        if not token or token.token_type != 'KEYWORD' or token.value != 'FROM':
            raise SyntaxError("DELETE must be followed by FROM")
        
        # Table name
        token = self._get_next_token(token_iter)
        if not token or token.token_type != 'IDENTIFIER':
            raise SyntaxError("DELETE FROM requires table name")
        table_name = token.value
        
        # WHERE clause (optional)
        where_clause = None
        token = self._get_next_token(token_iter)
        if token and token.token_type == 'KEYWORD' and token.value == 'WHERE':
            where_clause = self._parse_where_clause(token_iter)
        
        return ParsedQuery(
            query_type='DELETE',
            table_name=table_name,
            columns=[],
            where_clause=where_clause
        )
    
    def _parse_create(self, token_iter) -> ParsedQuery:
        """Parse CREATE statement"""
        # Should be TABLE keyword
        token = self._get_next_token(token_iter)
        if not token or token.token_type != 'KEYWORD' or token.value != 'TABLE':
            raise SyntaxError("CREATE must be followed by TABLE")
        
        # Table name
        token = self._get_next_token(token_iter)
        if not token or token.token_type != 'IDENTIFIER':
            raise SyntaxError("CREATE TABLE requires table name")
        table_name = token.value
        
        # Opening parenthesis
        token = self._get_next_token(token_iter)
        if not token or token.token_type != 'PUNCTUATION' or token.value != '(':
            raise SyntaxError("CREATE TABLE requires opening parenthesis")
        
        # Parse column definitions
        columns = {}
        primary_key = None
        
        while True:
            # Column name
            token = self._get_next_token(token_iter)
            if not token or token.token_type != 'IDENTIFIER':
                raise SyntaxError("Invalid column name")
            column_name = token.value
            
            # Data type
            token = self._get_next_token(token_iter)
            if not token or token.token_type != 'KEYWORD':
                raise SyntaxError("Invalid data type")
            data_type = token.value
            
            columns[column_name] = data_type
            
            # Check for PRIMARY KEY constraint
            token = self._get_next_token(token_iter)
            if token and token.token_type == 'KEYWORD' and token.value == 'PRIMARY':
                token = self._get_next_token(token_iter)
                if token and token.token_type == 'KEYWORD' and token.value == 'KEY':
                    primary_key = column_name
                    token = self._get_next_token(token_iter)
            
            # Comma or closing paren
            if not token or (token.token_type == 'PUNCTUATION' and token.value == ')'):
                break
            elif not (token.token_type == 'PUNCTUATION' and token.value == ','):
                raise SyntaxError("Invalid table definition")
        
        table_schema = {
            'columns': columns,
            'primary_key': primary_key
        }
        
        return ParsedQuery(
            query_type='CREATE',
            table_name=table_name,
            columns=list(columns.keys()),
            table_schema=table_schema
        )
    
    def _parse_begin(self, token_iter) -> ParsedQuery:
        """Parse BEGIN TRANSACTION statement"""
        token = self._get_next_token(token_iter)
        if not token or token.token_type != 'KEYWORD' or token.value != 'TRANSACTION':
            raise SyntaxError("BEGIN must be followed by TRANSACTION")
        
        return ParsedQuery(
            query_type='BEGIN',
            table_name='',
            columns=[]
        )
    
    def _parse_commit(self, token_iter) -> ParsedQuery:
        """Parse COMMIT statement"""
        return ParsedQuery(
            query_type='COMMIT',
            table_name='',
            columns=[]
        )
    
    def _parse_rollback(self, token_iter) -> ParsedQuery:
        """Parse ROLLBACK statement"""
        return ParsedQuery(
            query_type='ROLLBACK',
            table_name='',
            columns=[]
        )
    
    def _parse_where_clause(self, token_iter) -> Dict:
        """Parse WHERE clause conditions"""
        condition = {}
        
        while True:
            # Column name
            token = self._get_next_token(token_iter)
            if not token or token.token_type != 'IDENTIFIER':
                break  # No more conditions
            
            column = token.value
            
            # Operator
            token = self._get_next_token(token_iter)
            if not token or token.token_type != 'OPERATOR':
                raise SyntaxError("Invalid WHERE condition")
            
            operator = token.value
            
            # Value
            token = self._get_next_token(token_iter)
            if not token:
                raise SyntaxError("Unterminated WHERE clause")
            
            if token.token_type == 'KEYWORD' and token.value == 'NULL':
                value = None
            elif token.token_type == 'NUMBER':
                value = int(token.value) if '.' not in token.value else float(token.value)
            elif token.token_type == 'STRING':
                value = token.value
            else:
                raise SyntaxError(f"Invalid WHERE value: {token.value}")
            
            condition[column] = value
            
            # Check for AND/OR
            token = self._get_next_token(token_iter)
            if not token or token.token_type != 'KEYWORD':
                break
            elif token.value not in ['AND', 'OR']:
                break  # End of WHERE clause
        
        return condition


class SQLExecutor:
    """Executes parsed SQL queries on the relational engine"""
    
    def __init__(self, engine):
        self.engine = engine
    
    def execute(self, query: ParsedQuery, tx_id: str) -> Any:
        """Execute a parsed SQL query"""
        if query.query_type == 'SELECT':
            return self._execute_select(query, tx_id)
        elif query.query_type == 'INSERT':
            return self._execute_insert(query, tx_id)
        elif query.query_type == 'UPDATE':
            return self._execute_update(query, tx_id)
        elif query.query_type == 'DELETE':
            return self._execute_delete(query, tx_id)
        elif query.query_type == 'CREATE':
            return self._execute_create(query, tx_id)
        elif query.query_type == 'BEGIN':
            return self._execute_begin(tx_id)
        elif query.query_type == 'COMMIT':
            return self._execute_commit(tx_id)
        elif query.query_type == 'ROLLBACK':
            return self._execute_rollback(tx_id)
        else:
            raise ValueError(f"Unsupported query type: {query.query_type}")
    
    def _execute_select(self, query: ParsedQuery, tx_id: str) -> List[Dict]:
        """Execute SELECT query"""
        if query.table_name not in self.engine.tables:
            return []
        
        result = self.engine.select(tx_id, query.table_name, query.where_clause, query.columns)
        
        # Handle * wildcard
        if '*' in query.columns:
            return result
        
        return result
    
    def _execute_insert(self, query: ParsedQuery, tx_id: str) -> bool:
        """Execute INSERT query"""
        return self.engine.insert(tx_id, query.table_name, query.insert_data)
    
    def _execute_update(self, query: ParsedQuery, tx_id: str) -> bool:
        """Execute UPDATE query"""
        return self.engine.update(tx_id, query.table_name, query.where_clause or {}, query.update_data)
    
    def _execute_delete(self, query: ParsedQuery, tx_id: str) -> bool:
        """Execute DELETE query"""
        return self.engine.delete(tx_id, query.table_name, query.where_clause or {})
    
    def _execute_create(self, query: ParsedQuery, tx_id: str) -> bool:
        """Execute CREATE TABLE query"""
        table_schema = query.table_schema
        return self.engine.create_table(
            query.table_name,
            table_schema['columns'],
            table_schema['primary_key']
        )
    
    def _execute_begin(self, tx_id: str) -> bool:
        """Execute BEGIN TRANSACTION"""
        return self.engine.begin_transaction(tx_id)
    
    def _execute_commit(self, tx_id: str) -> bool:
        """Execute COMMIT"""
        return self.engine.commit(tx_id)
    
    def _execute_rollback(self, tx_id: str) -> bool:
        """Execute ROLLBACK"""
        return self.engine.rollback(tx_id)


def demonstrate_sql_parser():
    """Demonstrate SQL parser functionality"""
    parser = SQLParser()
    
    sql_statements = [
        "SELECT * FROM students",
        "SELECT name, age FROM students WHERE age > 20",
        "INSERT INTO students VALUES ('John', 25)",
        "UPDATE students SET age = 26 WHERE name = 'John'",
        "DELETE FROM students WHERE age < 18",
        "CREATE TABLE students (name VARCHAR, age INT, PRIMARY KEY name)",
        "BEGIN TRANSACTION",
        "COMMIT",
        "ROLLBACK"
    ]
    
    print("SQL Parser Demonstration")
    print("=" * 50)
    
    for sql in sql_statements:
        print(f"\nInput SQL: {sql}")
        try:
            query = parser.parse(sql)
            print(f"Parsed Query:")
            print(f"  Type: {query.query_type}")
            print(f"  Table: {query.table_name}")
            print(f"  Columns: {query.columns}")
            if query.where_clause:
                print(f"  WHERE: {query.where_clause}")
            if query.insert_data:
                print(f"  INSERT Data: {query.insert_data}")
            if query.update_data:
                print(f"  UPDATE Data: {query.update_data}")
            if query.table_schema:
                print(f"  Table Schema: {query.table_schema}")
        except Exception as e:
            print(f"Parse Error: {e}")


if __name__ == "__main__":
    demonstrate_sql_parser()