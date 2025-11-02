"""
Educational Demo for NoSQL Document Database
Demonstrates document storage, querying, and aggregation
"""

import sys
import os
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from document_db import DocumentDB
import json
import time


def demonstrate_document_operations():
    """Demonstrate basic document operations"""
    print("\n" + "="*60)
    print("DOCUMENT DATABASE OPERATIONS DEMONSTRATION")
    print("="*60)
    
    db = DocumentDB()
    
    # Create collections
    print("\n1. Creating collections...")
    db.create_collection('users')
    db.create_collection('products')
    db.create_collection('orders')
    print("✓ Created collections: users, products, orders")
    
    # Insert documents
    print("\n2. Inserting documents...")
    
    # Insert users
    user_data = [
        {
            'name': 'Alice Johnson',
            'email': 'alice@example.com',
            'age': 28,
            'address': {
                'street': '123 Main St',
                'city': 'New York',
                'zip': '10001'
            },
            'preferences': {
                'theme': 'dark',
                'notifications': True
            }
        },
        {
            'name': 'Bob Smith',
            'email': 'bob@example.com',
            'age': 35,
            'address': {
                'street': '456 Oak Ave',
                'city': 'Los Angeles',
                'zip': '90210'
            },
            'preferences': {
                'theme': 'light',
                'notifications': False
            }
        },
        {
            'name': 'Carol Davis',
            'email': 'carol@example.com',
            'age': 24,
            'address': {
                'street': '789 Pine St',
                'city': 'Chicago',
                'zip': '60601'
            },
            'preferences': {
                'theme': 'dark',
                'notifications': True
            }
        }
    ]
    
    user_ids = []
    for user in user_data:
        doc_id = db.insert_document('users', user)
        user_ids.append(doc_id)
        print(f"✓ Inserted user: {user['name']} (ID: {doc_id[:8]}...)")
    
    # Insert products
    product_data = [
        {
            'name': 'Laptop',
            'category': 'Electronics',
            'price': 999.99,
            'specs': {
                'cpu': 'Intel i7',
                'ram': '16GB',
                'storage': '512GB SSD'
            },
            'tags': ['computer', 'work', 'development']
        },
        {
            'name': 'Smartphone',
            'category': 'Electronics',
            'price': 599.99,
            'specs': {
                'os': 'Android',
                'storage': '128GB',
                'camera': '12MP'
            },
            'tags': ['mobile', 'communication']
        },
        {
            'name': 'Coffee Maker',
            'category': 'Kitchen',
            'price': 79.99,
            'specs': {
                'capacity': '12 cups',
                'type': 'Drip'
            },
            'tags': ['kitchen', 'appliance', 'coffee']
        }
    ]
    
    for product in product_data:
        doc_id = db.insert_document('products', product)
        print(f"✓ Inserted product: {product['name']} (ID: {doc_id[:8]}...)")
    
    print(f"\n3. Database statistics:")
    print(f"   Total documents: {db.document_count}")
    collections = db.list_collections()
    for collection in collections:
        stats = db.get_collection_stats(collection)
        print(f"   {collection}: {stats['document_count']} documents")


def demonstrate_querying():
    """Demonstrate query capabilities"""
    print("\n" + "="*60)
    print("QUERYING CAPABILITIES DEMONSTRATION")
    print("="*60)
    
    db = DocumentDB()
    _populate_sample_data(db)
    
    # Simple queries
    print("\n1. Simple queries...")
    
    # Find user by name
    user = db.find_one('users', {'name': 'Alice Johnson'})
    if user:
        print(f"✓ Found user: {user.data['name']}, Age: {user.data['age']}")
    
    # Find all electronics products
    electronics = db.find('products', {'category': 'Electronics'})
    print(f"✓ Found {len(electronics)} electronics products")
    for product in electronics:
        print(f"   - {product.data['name']}: ${product.data['price']}")
    
    # Query with projection
    print("\n2. Query with projection...")
    user_names = db.find('users', {}, projection={'name': 1, 'email': 1})
    print("✓ User names and emails:")
    for user in user_names:
        print(f"   - {user.data.get('name', 'N/A')}: {user.data.get('email', 'N/A')}")
    
    # Query with sorting
    print("\n3. Query with sorting...")
    sorted_products = db.find('products', sort=['-price'])
    print("✓ Products sorted by price (highest first):")
    for product in sorted_products:
        print(f"   - {product.data['name']}: ${product.data['price']}")
    
    # Query with operators
    print("\n4. Query with operators...")
    
    # Find users older than 25
    older_users = db.find('users', {'$gt': {'age': 25}})
    print(f"✓ Users older than 25: {len(older_users)}")
    
    # Find products with price less than 100
    cheap_products = db.find('products', {'$lt': {'price': 100}})
    print(f"✓ Products under $100: {len(cheap_products)}")
    for product in cheap_products:
        print(f"   - {product.data['name']}: ${product.data['price']}")
    
    # Complex query with AND
    dark_theme_users = db.find('users', {
        '$and': [
            {'preferences.theme': 'dark'},
            {'age': {'$gte': 25}}
        ]
    })
    print(f"✓ Users with dark theme and age >= 25: {len(dark_theme_users)}")


def demonstrate_aggregation():
    """Demonstrate aggregation pipeline"""
    print("\n" + "="*60)
    print("AGGREGATION PIPELINE DEMONSTRATION")
    print("="*60)
    
    db = DocumentDB()
    _populate_sample_data(db)
    
    # Add more products for aggregation
    for i in range(5):
        db.insert_document('products', {
            'name': f'Product {i}',
            'category': 'Electronics' if i % 2 == 0 else 'Kitchen',
            'price': 100 + i * 50,
            'rating': 4 + (i % 2)
        })
    
    # Simple aggregation examples
    print("\n1. Group by category...")
    pipeline = [
        {
            '$group': {
                '_id': '$category',
                'avg_price': '$avg',
                'count': '$count'
            }
        }
    ]
    
    results = db.aggregate('products', pipeline)
    for result in results:
        category = result.data.get('_id', 'Unknown')
        count = result.data.get('count', 0)
        avg_price = result.data.get('avg_price', 0)
        print(f"   {category}: {count} products, avg price: ${avg_price:.2f}")
    
    # Filter and group
    print("\n2. Filter and group...")
    pipeline = [
        {'$match': {'category': 'Electronics'}},
        {
            '$group': {
                '_id': '$category',
                'total_value': '$sum',
                'count': '$count'
            }
        }
    ]
    
    results = db.aggregate('products', pipeline)
    for result in results:
        total = result.data.get('total_value', 0)
        count = result.data.get('count', 0)
        print(f"   Electronics: {count} products, total value: ${total:.2f}")
    
    # Multiple aggregation stages
    print("\n3. Multi-stage aggregation...")
    pipeline = [
        {'$match': {'price': {'$lt': 1000}}},
        {'$sort': {'price': -1}},
        {'$limit': 3},
        {
            '$project': {
                'name': 1,
                'price': 1,
                'category': 1
            }
        }
    ]
    
    results = db.aggregate('products', pipeline)
    print("✓ Top 3 products under $1000:")
    for result in results:
        data = result.data
        print(f"   - {data.get('name', 'Unknown')}: ${data.get('price', 0)} ({data.get('category', 'Unknown')})")


def demonstrate_indexing():
    """Demonstrate indexing capabilities"""
    print("\n" + "="*60)
    print("INDEXING DEMONSTRATION")
    print("="*60)
    
    db = DocumentDB()
    _populate_sample_data(db)
    
    print("\n1. Creating indexes...")
    
    # Create single field indexes
    db.create_index('users', ['name'], 'idx_user_name')
    db.create_index('users', ['email'], 'idx_user_email', unique=True)
    db.create_index('products', ['category'], 'idx_product_category')
    db.create_index('products', ['price'], 'idx_product_price')
    
    print("✓ Created indexes:")
    print("   - User name index")
    print("   - User email index (unique)")
    print("   - Product category index")
    print("   - Product price index")
    
    # Create compound index
    print("\n2. Creating compound indexes...")
    db.create_index('products', ['category', 'price'], 'idx_product_cat_price')
    print("✓ Created compound index on (category, price)")
    
    # Show collection statistics
    print("\n3. Collection statistics with indexes:")
    for collection in db.list_collections():
        stats = db.get_collection_stats(collection)
        print(f"   {collection}:")
        print(f"     Documents: {stats['document_count']}")
        print(f"     Indexes: {stats['index_count']}")
        print(f"     Avg document size: {stats['avg_doc_size']:.0f} bytes")


def demonstrate_schema_flexibility():
    """Demonstrate NoSQL schema flexibility"""
    print("\n" + "="*60)
    print("SCHEMA FLEXIBILITY DEMONSTRATION")
    print("="*60)
    
    db = DocumentDB()
    
    print("\n1. Documents with different structures in same collection...")
    
    # Insert documents with different schemas
    db.insert_document('inventory', {
        'product_id': 'LAPTOP001',
        'type': 'laptop',
        'specs': {
            'cpu': 'Intel i7',
            'ram': '16GB',
            'storage': '512GB SSD'
        },
        'stock': 10,
        'location': 'Warehouse A'
    })
    
    db.insert_document('inventory', {
        'product_id': 'MOUSE001',
        'type': 'peripheral',
        'specs': {
            'connectivity': 'Wireless',
            'buttons': 3
        },
        'stock': 50,
        'vendor': 'TechCorp',
        'warranty_months': 24
    })
    
    db.insert_document('inventory', {
        'product_id': 'DESK001',
        'type': 'furniture',
        'specs': {
            'material': 'Oak',
            'dimensions': {'width': 120, 'height': 75, 'depth': 60}
        },
        'stock': 5,
        'requires_assembly': True
    })
    
    print("✓ Inserted documents with different schemas:")
    print("   - Laptop (with CPU/RAM/storage specs)")
    print("   - Mouse (with connectivity info and warranty)")
    print("   - Desk (with dimensions and assembly requirement)")
    
    print("\n2. Querying across different document structures...")
    
    # Find all products with 'specs' field
    all_specs = db.find('inventory', {'specs': {'$exists': True}})
    print(f"✓ Found {len(all_specs)} products with specs")
    
    # Find products with vendor field (only mouse has this)
    vendor_products = db.find('inventory', {'vendor': {'$exists': True}})
    print(f"✓ Found {len(vendor_products)} products with vendor info")
    
    # Find products with requires_assembly field (only desk has this)
    assembly_products = db.find('inventory', {'requires_assembly': True})
    print(f"✓ Found {len(assembly_products)} products requiring assembly")
    
    print("\n3. Demonstrating document evolution...")
    
    # Add new field to existing document
    db.update_one('inventory', {'product_id': 'LAPTOP001'}, {
        '$set': {
            'specs.gpu': 'NVIDIA RTX 3060',
            'warranty_months': 36,
            'category': 'High-end Laptops'
        }
    })
    
    laptop = db.find_one('inventory', {'product_id': 'LAPTOP001'})
    print("✓ Updated laptop with new fields (GPU, warranty, category)")
    print(f"   CPU: {laptop.data['specs']['cpu']}")
    print(f"   GPU: {laptop.data['specs']['gpu']}")
    print(f"   Warranty: {laptop.data['warranty_months']} months")


def demonstrate_json_operations():
    """Demonstrate JSON import/export operations"""
    print("\n" + "="*60)
    print("JSON IMPORT/EXPORT DEMONSTRATION")
    print("="*60)
    
    db = DocumentDB()
    _populate_sample_data(db)
    
    print("\n1. Exporting collection to JSON...")
    json_export = db.export_collection('users')
    print("✓ Exported users collection")
    print(f"   Export size: {len(json_export)} characters")
    
    # Create new database and import
    db2 = DocumentDB()
    print("\n2. Importing collection to new database...")
    
    success = db2.import_collection(json_export)
    if success:
        print("✓ Successfully imported users collection")
        print(f"   Imported {len(db2.find('users'))} documents")
    else:
        print("✗ Import failed")
    
    # Demonstrate nested JSON handling
    print("\n3. Complex nested JSON operations...")
    
    # Insert deeply nested document
    nested_doc = {
        'company': 'Tech Corp',
        'departments': [
            {
                'name': 'Engineering',
                'teams': [
                    {
                        'name': 'Backend',
                        'members': [
                            {'name': 'Alice', 'role': 'Senior Dev', 'skills': ['Python', 'Django']},
                            {'name': 'Bob', 'role': 'DevOps', 'skills': ['AWS', 'Docker']}
                        ]
                    },
                    {
                        'name': 'Frontend',
                        'members': [
                            {'name': 'Carol', 'role': 'UI/UX', 'skills': ['React', 'TypeScript']}
                        ]
                    }
                ],
                'budget': 500000,
                'location': 'San Francisco'
            }
        ],
        'founded': 2020,
        'metrics': {
            'revenue': {'2020': 100000, '2021': 250000, '2022': 500000},
            'employees': {'total': 25, 'engineering': 15, 'sales': 5, 'marketing': 5}
        }
    }
    
    doc_id = db.insert_document('companies', nested_doc)
    print(f"✓ Inserted deeply nested company document (ID: {doc_id[:8]}...)")
    
    # Query nested data
    backend_engineers = db.find('companies', {'departments.teams.members.skills': 'Python'})
    print(f"✓ Found {len(backend_engineers)} companies with Python developers")
    
    # Query by nested metric
    high_revenue = db.find('companies', {'metrics.revenue.2022': {'$gt': 400000}})
    print(f"✓ Found {len(high_revenue)} companies with 2022 revenue > $400,000")


def _populate_sample_data(db):
    """Populate database with sample data for demonstrations"""
    # Users
    users = [
        {'name': 'Alice Johnson', 'email': 'alice@example.com', 'age': 28, 'city': 'New York'},
        {'name': 'Bob Smith', 'email': 'bob@example.com', 'age': 35, 'city': 'Los Angeles'},
        {'name': 'Carol Davis', 'email': 'carol@example.com', 'age': 24, 'city': 'Chicago'},
        {'name': 'David Brown', 'email': 'david@example.com', 'age': 42, 'city': 'Seattle'},
        {'name': 'Emma Wilson', 'email': 'emma@example.com', 'age': 31, 'city': 'Boston'}
    ]
    
    for user in users:
        db.insert_document('users', user)
    
    # Products
    products = [
        {'name': 'Laptop', 'category': 'Electronics', 'price': 999.99, 'rating': 4.5},
        {'name': 'Mouse', 'category': 'Electronics', 'price': 29.99, 'rating': 4.2},
        {'name': 'Desk', 'category': 'Furniture', 'price': 199.99, 'rating': 4.0},
        {'name': 'Chair', 'category': 'Furniture', 'price': 149.99, 'rating': 4.3},
        {'name': 'Monitor', 'category': 'Electronics', 'price': 299.99, 'rating': 4.4}
    ]
    
    for product in products:
        db.insert_document('products', product)


def main():
    """Main demonstration function"""
    print("NOSQL DOCUMENT DATABASE EDUCATIONAL DEMO")
    print("Demonstrating Document Storage, Querying, and Aggregation")
    print("="*80)
    
    try:
        demonstrate_document_operations()
        demonstrate_querying()
        demonstrate_aggregation()
        demonstrate_indexing()
        demonstrate_schema_flexibility()
        demonstrate_json_operations()
        
        print("\n" + "="*80)
        print("NOSQL DATABASE DEMO COMPLETED")
        print("="*80)
        print("\nKey Concepts Demonstrated:")
        print("✓ Document-based storage with flexible schemas")
        print("✓ Rich query capabilities with operators")
        print("✓ Aggregation pipeline for data analysis")
        print("✓ Indexing for query optimization")
        print("✓ JSON import/export for data portability")
        print("✓ Schema evolution and document updates")
        print("✓ Nested document querying")
        print("\nThis educational database provides hands-on learning of:")
        print("- NoSQL vs SQL data modeling")
        print("- Document database concepts")
        print("- Flexible schema design")
        print("- Aggregation and data analysis")
        print("- Query optimization techniques")
        
    except Exception as e:
        print(f"Error during demonstration: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    main()