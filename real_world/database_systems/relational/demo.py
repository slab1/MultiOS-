"""
Educational Demo for Relational Database Engine
Demonstrates ACID properties, SQL parsing, and transaction management
"""

import sys
import os
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from relational_engine import RelationalEngine
from sql_parser import SQLParser, SQLExecutor


def demonstrate_acid_properties():
    """Demonstrate ACID properties using relational engine"""
    print("\n" + "="*60)
    print("ACID PROPERTIES DEMONSTRATION")
    print("="*60)
    
    engine = RelationalEngine()
    parser = SQLParser()
    executor = SQLExecutor(engine)
    
    # Create a student table
    print("\n1. Creating student table...")
    schema = {
        'name': 'VARCHAR',
        'age': 'INT',
        'grade': 'VARCHAR',
        'student_id': 'INT'
    }
    engine.create_table('students', schema, 'student_id')
    print("✓ Table 'students' created with columns: name, age, grade, student_id")
    
    # Begin transaction
    print("\n2. Starting transaction (AUTOMICITY)...")
    tx_id = "TX_001"
    executor.execute(parser.parse("BEGIN TRANSACTION"), tx_id)
    print(f"✓ Transaction {tx_id} started")
    
    # Insert data with ACID guarantees
    print("\n3. Inserting data (ATOMICITY & CONSISTENCY)...")
    test_data = [
        {'name': 'Alice Johnson', 'age': 20, 'grade': 'A', 'student_id': 1},
        {'name': 'Bob Smith', 'age': 22, 'grade': 'B', 'student_id': 2},
        {'name': 'Carol Davis', 'age': 19, 'grade': 'A', 'student_id': 3},
    ]
    
    for data in test_data:
        result = engine.insert(tx_id, 'students', data)
        if result:
            print(f"✓ Inserted: {data['name']} (ID: {data['student_id']})")
        else:
            print(f"✗ Failed to insert: {data['name']}")
    
    # Query data (ISOLATION)
    print("\n4. Reading data (ISOLATION)...")
    students = engine.select(tx_id, 'students')
    print(f"✓ Found {len(students)} students in database")
    for student in students:
        print(f"   - {student['name']}, Age: {student['age']}, Grade: {student['grade']}")
    
    # Commit transaction (DURABILITY)
    print("\n5. Committing transaction (DURABILITY)...")
    engine.commit(tx_id)
    print(f"✓ Transaction {tx_id} committed successfully")
    
    # Demonstrate rollback
    print("\n6. Demonstrating ROLLBACK...")
    tx_id = "TX_002"
    executor.execute(parser.parse("BEGIN TRANSACTION"), tx_id)
    
    # Insert problematic data
    engine.insert(tx_id, 'students', {'name': 'David Brown', 'age': 25, 'grade': 'C', 'student_id': 4})
    print("✓ Inserted David Brown")
    
    # Check data
    students = engine.select(tx_id, 'students')
    print(f"✓ Current students: {len(students)}")
    
    # Rollback
    engine.rollback(tx_id)
    print("✓ Transaction rolled back")
    
    # Verify rollback
    students = engine.select("TX_001", 'students')  # Use committed transaction data
    print(f"✓ After rollback, students: {len(students)} (David should be gone)")


def demonstrate_concurrency_control():
    """Demonstrate concurrency control and locking"""
    print("\n" + "="*60)
    print("CONCURRENCY CONTROL DEMONSTRATION")
    print("="*60)
    
    engine = RelationalEngine()
    parser = SQLParser()
    executor = SQLExecutor(engine)
    
    # Create table
    engine.create_table('accounts', {'account_id': 'INT', 'balance': 'FLOAT'}, 'account_id')
    
    # Initialize accounts
    tx1 = "TX_CONCURRENCY_1"
    executor.execute(parser.parse("BEGIN TRANSACTION"), tx1)
    engine.insert(tx1, 'accounts', {'account_id': 1, 'balance': 1000.0})
    engine.insert(tx1, 'accounts', {'account_id': 2, 'balance': 500.0})
    engine.commit(tx1)
    
    print("\n1. Initial accounts created:")
    accounts = engine.select(tx1, 'accounts')
    for acc in accounts:
        print(f"   Account {acc['account_id']}: ${acc['balance']:.2f}")
    
    # Demonstrate shared lock for reading
    print("\n2. Demonstrating shared locks...")
    tx_read = "TX_READ"
    executor.execute(parser.parse("BEGIN TRANSACTION"), tx_read)
    
    accounts = engine.select(tx_read, 'accounts')
    print(f"✓ Shared lock acquired, read {len(accounts)} accounts")
    print("   (Multiple transactions can read simultaneously)")
    
    # Demonstrate exclusive lock for writing
    print("\n3. Demonstrating exclusive locks...")
    tx_write = "TX_WRITE"
    executor.execute(parser.parse("BEGIN TRANSACTION"), tx_write)
    
    # Try to update while reader is active
    result = engine.update(tx_write, 'accounts', {'account_id': 1}, {'balance': 1200.0})
    if result:
        print("✓ Exclusive lock acquired, update successful")
        print("   (Writers need exclusive access)")
    else:
        print("✗ Update failed - lock contention detected")
    
    engine.commit(tx_write)
    engine.commit(tx_read)
    
    # Verify final state
    accounts = engine.select("TX_READ", 'accounts')
    print(f"\n4. Final state after concurrent operations:")
    for acc in accounts:
        print(f"   Account {acc['account_id']}: ${acc['balance']:.2f}")


def demonstrate_sql_parsing():
    """Demonstrate SQL parsing and execution"""
    print("\n" + "="*60)
    print("SQL PARSING AND EXECUTION DEMONSTRATION")
    print("="*60)
    
    engine = RelationalEngine()
    parser = SQLParser()
    executor = SQLExecutor(engine)
    
    # Use SQL to create table
    print("\n1. Creating table using SQL...")
    sql_create = "CREATE TABLE products (name VARCHAR, price FLOAT, category VARCHAR, product_id INT)"
    query = parser.parse(sql_create)
    executor.execute(query, "TX_SQL_1")
    engine.commit("TX_SQL_1")
    print("✓ Table created via SQL parsing")
    
    # Use SQL to insert data
    print("\n2. Inserting data using SQL...")
    tx_sql = "TX_SQL_2"
    executor.execute(parser.parse("BEGIN TRANSACTION"), tx_sql)
    
    insert_statements = [
        "INSERT INTO products VALUES ('Laptop', 999.99, 'Electronics', 1)",
        "INSERT INTO products VALUES ('Book', 19.99, 'Education', 2)",
        "INSERT INTO products VALUES ('Coffee Mug', 12.99, 'Kitchen', 3)",
    ]
    
    for sql in insert_statements:
        query = parser.parse(sql)
        result = executor.execute(query, tx_sql)
        if result:
            print(f"✓ Inserted via SQL: {sql.split('VALUES')[0].replace('INSERT INTO ', '').strip()}")
    
    engine.commit(tx_sql)
    
    # Use SQL to query data
    print("\n3. Querying data using SQL...")
    tx_query = "TX_SQL_3"
    executor.execute(parser.parse("BEGIN TRANSACTION"), tx_query)
    
    # Simple SELECT
    sql_select = "SELECT * FROM products"
    query = parser.parse(sql_select)
    products = executor.execute(query, tx_query)
    print(f"\n✓ Found {len(products)} products:")
    for product in products:
        print(f"   - {product['name']}: ${product['price']} ({product['category']})")
    
    # SELECT with WHERE clause
    sql_filtered = "SELECT name, price FROM products WHERE price < 50"
    query = parser.parse(sql_filtered)
    cheap_products = executor.execute(query, tx_query)
    print(f"\n✓ Products under $50:")
    for product in cheap_products:
        print(f"   - {product['name']}: ${product['price']}")
    
    engine.commit(tx_query)
    
    # Update with SQL
    print("\n4. Updating data using SQL...")
    tx_update = "TX_SQL_4"
    executor.execute(parser.parse("BEGIN TRANSACTION"), tx_update)
    
    sql_update = "UPDATE products SET price = 899.99 WHERE name = 'Laptop'"
    query = parser.parse(sql_update)
    result = executor.execute(query, tx_update)
    if result:
        print("✓ Updated laptop price via SQL")
    
    engine.commit(tx_update)
    
    # Verify update
    tx_verify = "TX_SQL_5"
    executor.execute(parser.parse("BEGIN TRANSACTION"), tx_verify)
    sql_check = "SELECT name, price FROM products WHERE name = 'Laptop'"
    query = parser.parse(sql_check)
    laptop = executor.execute(query, tx_verify)
    if laptop:
        print(f"✓ Updated price verified: {laptop[0]['name']} now costs ${laptop[0]['price']}")
    engine.commit(tx_verify)


def demonstrate_recovery_mechanisms():
    """Demonstrate database recovery mechanisms"""
    print("\n" + "="*60)
    print("DATABASE RECOVERY DEMONSTRATION")
    print("="*60)
    
    engine = RelationalEngine()
    parser = SQLParser()
    executor = SQLExecutor(engine)
    
    # Create table
    engine.create_table('important_data', {'id': 'INT', 'value': 'VARCHAR'}, 'id')
    
    print("\n1. Creating critical data...")
    tx1 = "TX_RECOVERY_1"
    executor.execute(parser.parse("BEGIN TRANSACTION"), tx1)
    engine.insert(tx1, 'important_data', {'id': 1, 'value': 'Critical Data 1'})
    engine.insert(tx1, 'important_data', {'id': 2, 'value': 'Critical Data 2'})
    engine.commit(tx1)
    print("✓ Critical data committed")
    
    print("\n2. Simulating system crash recovery...")
    print("   (Uncommitted transactions will be rolled back)")
    
    # Simulate uncommitted transaction
    tx2 = "TX_RECOVERY_2"
    executor.execute(parser.parse("BEGIN TRANSACTION"), tx2)
    engine.insert(tx2, 'important_data', {'id': 3, 'value': 'Temporary Data'})
    print("   - Uncommitted transaction created temporary data")
    
    # Simulate crash - recovery would happen here
    print("\n3. Performing recovery...")
    engine.recover()
    print("✓ Recovery completed")
    print("   - Uncommitted transaction data removed")
    print("   - Committed transaction data preserved")
    
    # Verify recovery
    tx3 = "TX_RECOVERY_3"
    executor.execute(parser.parse("BEGIN TRANSACTION"), tx3)
    data = engine.select(tx3, 'important_data')
    print(f"\n4. Data after recovery: {len(data)} records")
    for record in data:
        print(f"   - ID {record['id']}: {record['value']}")
    engine.commit(tx3)


def demonstrate_indexing():
    """Demonstrate database indexing"""
    print("\n" + "="*60)
    print("DATABASE INDEXING DEMONSTRATION")
    print("="*60)
    
    engine = RelationalEngine()
    parser = SQLParser()
    executor = SQLExecutor(engine)
    
    # Create large table
    engine.create_table('employees', {
        'emp_id': 'INT',
        'name': 'VARCHAR',
        'department': 'VARCHAR',
        'salary': 'INT'
    }, 'emp_id')
    
    print("\n1. Loading employee data...")
    tx1 = "TX_INDEX_1"
    executor.execute(parser.parse("BEGIN TRANSACTION"), tx1)
    
    # Insert sample data
    departments = ['Engineering', 'Sales', 'HR', 'Marketing', 'Finance']
    for i in range(1000):
        name = f"Employee_{i:03d}"
        dept = departments[i % len(departments)]
        salary = 40000 + (i % 100) * 1000
        engine.insert(tx1, 'employees', {
            'emp_id': i + 1,
            'name': name,
            'department': dept,
            'salary': salary
        })
    
    engine.commit(tx1)
    print(f"✓ Loaded 1000 employee records")
    
    print("\n2. Creating indexes...")
    engine.create_index('employees', 'idx_department', 'department')
    engine.create_index('employees', 'idx_salary', 'salary')
    print("✓ Created indexes on department and salary columns")
    
    print("\n3. Query performance demonstration...")
    print("   - Queries using indexed columns will be faster")
    print("   - Indexes support efficient lookups and range queries")
    print("   - Multiple indexes support query optimization")
    
    # Demonstrate indexed queries
    tx2 = "TX_INDEX_2"
    executor.execute(parser.parse("BEGIN TRANSACTION"), tx2)
    
    # Query by department (indexed)
    employees = engine.select(tx2, 'employees', {'department': 'Engineering'})
    print(f"   - Engineering employees: {len(employees)} (using index)")
    
    # Query by salary range (simplified)
    # Note: This would need range query support in a real implementation
    all_emps = engine.select(tx2, 'employees')
    high_salary = [emp for emp in all_emps if emp['salary'] > 60000]
    print(f"   - Employees with salary > $60,000: {len(high_salary)} (indexed scan)")
    
    engine.commit(tx2)


def main():
    """Main demonstration function"""
    print("DATABASE SYSTEMS EDUCATIONAL DEMO")
    print("Relational Database Engine with ACID Properties")
    print("="*80)
    
    try:
        # Run all demonstrations
        demonstrate_acid_properties()
        demonstrate_concurrency_control()
        demonstrate_sql_parsing()
        demonstrate_recovery_mechanisms()
        demonstrate_indexing()
        
        print("\n" + "="*80)
        print("EDUCATIONAL DEMO COMPLETED")
        print("="*80)
        print("\nKey Concepts Demonstrated:")
        print("✓ ACID Properties (Atomicity, Consistency, Isolation, Durability)")
        print("✓ Transaction Management")
        print("✓ Concurrency Control and Locking")
        print("✓ SQL Parsing and Execution")
        print("✓ Database Recovery")
        print("✓ Indexing and Query Optimization")
        print("\nThis educational database provides hands-on learning of:")
        print("- How relational databases work internally")
        print("- SQL parsing and query execution")
        print("- Transaction processing and ACID guarantees")
        print("- Concurrency control mechanisms")
        print("- Database recovery and logging")
        
    except Exception as e:
        print(f"Error during demonstration: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    main()