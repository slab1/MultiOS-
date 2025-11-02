"""
Educational Database Tutorials Manager
Provides structured learning materials and hands-on SQL exercises
"""

import json
import time
import random
from typing import Dict, List, Any, Optional, Tuple
from dataclasses import dataclass, asdict
from enum import Enum
import os


class DifficultyLevel(Enum):
    """Tutorial difficulty levels"""
    BEGINNER = "BEGINNER"
    INTERMEDIATE = "INTERMEDIATE"
    ADVANCED = "ADVANCED"
    EXPERT = "EXPERT"


class Topic(Enum):
    """Database topics"""
    SQL_BASICS = "SQL_BASICS"
    QUERIES = "QUERIES"
    JOINS = "JOINS"
    AGGREGATION = "AGGREGATION"
    SUBQUERIES = "SUBQUERIES"
    INDEXES = "INDEXES"
    TRANSACTIONS = "TRANSACTIONS"
    PERFORMANCE = "PERFORMANCE"
    NORMALIZATION = "NORMALIZATION"
    SECURITY = "SECURITY"
    NOSQL = "NOSQL"
    GRAPH_DB = "GRAPH_DB"


@dataclass
class Exercise:
    """Database exercise with expected results"""
    exercise_id: str
    title: str
    description: str
    difficulty: DifficultyLevel
    topic: Topic
    sql_query: str
    expected_columns: List[str]
    expected_rows_range: Tuple[int, int]
    hints: List[str]
    solution: str
    points: int
    estimated_time: int  # minutes
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            'exercise_id': self.exercise_id,
            'title': self.title,
            'description': self.description,
            'difficulty': self.difficulty.value,
            'topic': self.topic.value,
            'hints': self.hints,
            'points': self.points,
            'estimated_time': self.estimated_time
        }


@dataclass
class Tutorial:
    """Database tutorial"""
    tutorial_id: str
    title: str
    description: str
    difficulty: DifficultyLevel
    topic: Topic
    content: str
    exercises: List[Exercise]
    prerequisites: List[str]
    learning_objectives: List[str]
    estimated_time: int  # minutes
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            'tutorial_id': self.tutorial_id,
            'title': self.title,
            'description': self.description,
            'difficulty': self.difficulty.value,
            'topic': self.topic.value,
            'learning_objectives': self.learning_objectives,
            'estimated_time': self.estimated_time,
            'exercise_count': len(self.exercises)
        }


@dataclass
class StudentProgress:
    """Student progress tracking"""
    student_id: str
    completed_exercises: List[str]
    current_tutorial: Optional[str]
    points_earned: int
    total_time_spent: int  # minutes
    last_activity: float
    exercise_results: Dict[str, Dict[str, Any]]
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            'student_id': self.student_id,
            'completed_exercises': self.completed_exercises,
            'current_tutorial': self.current_tutorial,
            'points_earned': self.points_earned,
            'total_time_spent': self.total_time_spent,
            'last_activity': self.last_activity,
            'exercise_results': self.exercise_results
        }


class DatabaseSchema:
    """Educational database schema for tutorials"""
    
    def __init__(self):
        self.schemas = {}
        self.sample_data = {}
        self._initialize_schemas()
    
    def _initialize_schemas(self):
        """Initialize educational database schemas"""
        # E-Commerce Schema
        self.schemas['ecommerce'] = {
            'tables': {
                'customers': [
                    'customer_id INT PRIMARY KEY',
                    'first_name VARCHAR(50)',
                    'last_name VARCHAR(50)',
                    'email VARCHAR(100) UNIQUE',
                    'phone VARCHAR(20)',
                    'registration_date DATE',
                    'country VARCHAR(50)'
                ],
                'products': [
                    'product_id INT PRIMARY KEY',
                    'product_name VARCHAR(200)',
                    'category VARCHAR(100)',
                    'price DECIMAL(10,2)',
                    'stock_quantity INT',
                    'description TEXT',
                    'created_date DATE'
                ],
                'orders': [
                    'order_id INT PRIMARY KEY',
                    'customer_id INT',
                    'order_date DATE',
                    'total_amount DECIMAL(10,2)',
                    'status VARCHAR(20)',
                    'shipping_address TEXT'
                ],
                'order_items': [
                    'item_id INT PRIMARY KEY',
                    'order_id INT',
                    'product_id INT',
                    'quantity INT',
                    'unit_price DECIMAL(10,2)',
                    'subtotal DECIMAL(10,2)'
                ],
                'reviews': [
                    'review_id INT PRIMARY KEY',
                    'product_id INT',
                    'customer_id INT',
                    'rating INT',
                    'review_text TEXT',
                    'review_date DATE'
                ]
            },
            'foreign_keys': [
                'orders.customer_id -> customers.customer_id',
                'order_items.order_id -> orders.order_id',
                'order_items.product_id -> products.product_id',
                'reviews.product_id -> products.product_id',
                'reviews.customer_id -> customers.customer_id'
            ]
        }
        
        # University Schema
        self.schemas['university'] = {
            'tables': {
                'students': [
                    'student_id INT PRIMARY KEY',
                    'first_name VARCHAR(50)',
                    'last_name VARCHAR(50)',
                    'email VARCHAR(100) UNIQUE',
                    'major VARCHAR(100)',
                    'gpa DECIMAL(3,2)',
                    'enrollment_date DATE'
                ],
                'courses': [
                    'course_id INT PRIMARY KEY',
                    'course_name VARCHAR(200)',
                    'department VARCHAR(100)',
                    'credits INT',
                    'prerequisites TEXT'
                ],
                'enrollments': [
                    'enrollment_id INT PRIMARY KEY',
                    'student_id INT',
                    'course_id INT',
                    'semester VARCHAR(20)',
                    'year INT',
                    'grade CHAR(1)'
                ],
                'instructors': [
                    'instructor_id INT PRIMARY KEY',
                    'first_name VARCHAR(50)',
                    'last_name VARCHAR(50)',
                    'department VARCHAR(100)',
                    'hire_date DATE',
                    'salary DECIMAL(10,2)'
                ],
                'course_instructors': [
                    'assignment_id INT PRIMARY KEY',
                    'course_id INT',
                    'instructor_id INT',
                    'semester VARCHAR(20)',
                    'year INT'
                ]
            },
            'foreign_keys': [
                'enrollments.student_id -> students.student_id',
                'enrollments.course_id -> courses.course_id',
                'course_instructors.course_id -> courses.course_id',
                'course_instructors.instructor_id -> instructors.instructor_id'
            ]
        }
        
        # HR Schema
        self.schemas['hr'] = {
            'tables': {
                'employees': [
                    'employee_id INT PRIMARY KEY',
                    'first_name VARCHAR(50)',
                    'last_name VARCHAR(50)',
                    'email VARCHAR(100) UNIQUE',
                    'phone VARCHAR(20)',
                    'hire_date DATE',
                    'salary DECIMAL(10,2)',
                    'job_title VARCHAR(100)',
                    'department VARCHAR(100)'
                ],
                'departments': [
                    'department_id INT PRIMARY KEY',
                    'department_name VARCHAR(100)',
                    'manager_id INT',
                    'location VARCHAR(100)',
                    'budget DECIMAL(12,2)'
                ],
                'projects': [
                    'project_id INT PRIMARY KEY',
                    'project_name VARCHAR(200)',
                    'department_id INT',
                    'budget DECIMAL(12,2)',
                    'start_date DATE',
                    'end_date DATE'
                ],
                'project_assignments': [
                    'assignment_id INT PRIMARY KEY',
                    'employee_id INT',
                    'project_id INT',
                    'role VARCHAR(100)',
                    'hours_worked DECIMAL(5,2)',
                    'assignment_date DATE'
                ],
                'salary_history': [
                    'history_id INT PRIMARY KEY',
                    'employee_id INT',
                    'old_salary DECIMAL(10,2)',
                    'new_salary DECIMAL(10,2)',
                    'change_date DATE',
                    'reason VARCHAR(200)'
                ]
            },
            'foreign_keys': [
                'employees.department_id -> departments.department_id',
                'departments.manager_id -> employees.employee_id',
                'projects.department_id -> departments.department_id',
                'project_assignments.employee_id -> employees.employee_id',
                'project_assignments.project_id -> projects.project_id',
                'salary_history.employee_id -> employees.employee_id'
            ]
        }
    
    def get_schema_sql(self, schema_name: str) -> str:
        """Get SQL for creating database schema"""
        if schema_name not in self.schemas:
            return ""
        
        schema = self.schemas[schema_name]
        sql_statements = []
        
        # Create tables
        for table_name, columns in schema['tables'].items():
            columns_sql = ",\n    ".join(columns)
            create_table = f"CREATE TABLE {table_name} (\n    {columns_sql}\n);"
            sql_statements.append(create_table)
        
        # Add foreign keys
        for fk in schema['foreign_keys']:
            sql_statements.append(f"ALTER TABLE {fk.split('->')[0].strip()} ADD FOREIGN KEY ({fk.split('->')[0].split('.')[1].strip()}) REFERENCES {fk.split('->')[1].strip()};")
        
        return "\n\n".join(sql_statements)
    
    def get_sample_data(self, schema_name: str) -> Dict[str, List[Dict[str, Any]]]:
        """Get sample data for schema"""
        # Generate sample data for each schema
        if schema_name == 'ecommerce':
            return self._generate_ecommerce_data()
        elif schema_name == 'university':
            return self._generate_university_data()
        elif schema_name == 'hr':
            return self._generate_hr_data()
        return {}
    
    def _generate_ecommerce_data(self) -> Dict[str, List[Dict[str, Any]]]:
        """Generate e-commerce sample data"""
        import random
        from datetime import datetime, timedelta
        
        # Generate customers
        customers = []
        first_names = ['John', 'Jane', 'Mike', 'Sarah', 'David', 'Emily', 'Chris', 'Lisa', 'Tom', 'Anna']
        last_names = ['Smith', 'Johnson', 'Brown', 'Davis', 'Wilson', 'Moore', 'Taylor', 'Anderson', 'Thomas', 'Jackson']
        countries = ['USA', 'Canada', 'UK', 'Germany', 'France', 'Australia']
        
        for i in range(50):
            customers.append({
                'customer_id': i + 1,
                'first_name': random.choice(first_names),
                'last_name': random.choice(last_names),
                'email': f"customer{i+1}@example.com",
                'phone': f"555-{random.randint(1000, 9999)}",
                'registration_date': (datetime.now() - timedelta(days=random.randint(1, 365))).date(),
                'country': random.choice(countries)
            })
        
        # Generate products
        products = []
        categories = ['Electronics', 'Books', 'Clothing', 'Home & Garden', 'Sports', 'Toys']
        product_names = ['Laptop', 'Mouse', 'Keyboard', 'Monitor', 'Phone', 'Tablet', 'Headphones', 'Camera']
        
        for i in range(100):
            products.append({
                'product_id': i + 1,
                'product_name': f"{random.choice(product_names)} Model {i+1}",
                'category': random.choice(categories),
                'price': round(random.uniform(10.0, 1000.0), 2),
                'stock_quantity': random.randint(0, 1000),
                'description': f"High-quality {random.choice(product_names).lower()}",
                'created_date': (datetime.now() - timedelta(days=random.randint(1, 365))).date()
            })
        
        # Generate orders
        orders = []
        statuses = ['pending', 'shipped', 'delivered', 'cancelled']
        
        for i in range(200):
            orders.append({
                'order_id': i + 1,
                'customer_id': random.randint(1, 50),
                'order_date': (datetime.now() - timedelta(days=random.randint(1, 90))).date(),
                'total_amount': round(random.uniform(20.0, 2000.0), 2),
                'status': random.choice(statuses),
                'shipping_address': f"{random.randint(1, 9999)} Main St, City {random.randint(1, 100)}, {random.choice(countries)}"
            })
        
        # Generate order items
        order_items = []
        for i in range(500):
            order_items.append({
                'item_id': i + 1,
                'order_id': random.randint(1, 200),
                'product_id': random.randint(1, 100),
                'quantity': random.randint(1, 5),
                'unit_price': round(random.uniform(10.0, 500.0), 2),
                'subtotal': 0  # Will be calculated
            })
            
            # Calculate subtotal
            order_items[-1]['subtotal'] = round(order_items[-1]['quantity'] * order_items[-1]['unit_price'], 2)
        
        # Generate reviews
        reviews = []
        for i in range(300):
            reviews.append({
                'review_id': i + 1,
                'product_id': random.randint(1, 100),
                'customer_id': random.randint(1, 50),
                'rating': random.randint(1, 5),
                'review_text': f"Great product! Rating: {random.randint(1, 5)}/5",
                'review_date': (datetime.now() - timedelta(days=random.randint(1, 365))).date()
            })
        
        return {
            'customers': customers,
            'products': products,
            'orders': orders,
            'order_items': order_items,
            'reviews': reviews
        }
    
    def _generate_university_data(self) -> Dict[str, List[Dict[str, Any]]]:
        """Generate university sample data"""
        import random
        from datetime import datetime, timedelta
        
        # Similar data generation for university schema...
        # (Implementation similar to e-commerce but with university-specific data)
        return {}
    
    def _generate_hr_data(self) -> Dict[str, List[Dict[str, Any]]]:
        """Generate HR sample data"""
        import random
        from datetime import datetime, timedelta
        
        # Similar data generation for HR schema...
        # (Implementation similar to e-commerce but with HR-specific data)
        return {}


class TutorialManager:
    """Manages educational database tutorials and exercises"""
    
    def __init__(self):
        self.tutorials: Dict[str, Tutorial] = {}
        self.exercises: Dict[str, Exercise] = {}
        self.student_progress: Dict[str, StudentProgress] = {}
        self.schema_manager = DatabaseSchema()
        self._initialize_tutorials()
    
    def _initialize_tutorials(self):
        """Initialize educational tutorials"""
        # SQL Basics Tutorial
        sql_basics_exercises = [
            Exercise(
                exercise_id="SB_001",
                title="Basic SELECT Statement",
                description="Write a SELECT statement to retrieve all columns from the customers table.",
                difficulty=DifficultyLevel.BEGINNER,
                topic=Topic.SQL_BASICS,
                sql_query="SELECT * FROM customers;",
                expected_columns=['customer_id', 'first_name', 'last_name', 'email', 'phone', 'registration_date', 'country'],
                expected_rows_range=(1, 100),
                hints=[
                    "Use SELECT * to retrieve all columns",
                    "Remember the FROM clause specifies the table name",
                    "End SQL statements with a semicolon"
                ],
                solution="SELECT * FROM customers;",
                points=10,
                estimated_time=5
            ),
            Exercise(
                exercise_id="SB_002",
                title="Selecting Specific Columns",
                description="Select only the first_name, last_name, and email columns from customers.",
                difficulty=DifficultyLevel.BEGINNER,
                topic=Topic.SQL_BASICS,
                sql_query="SELECT first_name, last_name, email FROM customers;",
                expected_columns=['first_name', 'last_name', 'email'],
                expected_rows_range=(1, 100),
                hints=[
                    "List column names separated by commas",
                    "Column order matters in the result",
                    "Use exact column names from the schema"
                ],
                solution="SELECT first_name, last_name, email FROM customers;",
                points=10,
                estimated_time=5
            ),
            Exercise(
                exercise_id="SB_003",
                title="Filtering with WHERE",
                description="Find all products with a price greater than 100.",
                difficulty=DifficultyLevel.BEGINNER,
                topic=Topic.QUERIES,
                sql_query="SELECT * FROM products WHERE price > 100;",
                expected_columns=['product_id', 'product_name', 'category', 'price', 'stock_quantity', 'description', 'created_date'],
                expected_rows_range=(1, 100),
                hints=[
                    "Use WHERE clause to filter rows",
                    "Compare prices using > operator",
                    "Numeric comparisons don't need quotes"
                ],
                solution="SELECT * FROM products WHERE price > 100;",
                points=15,
                estimated_time=10
            )
        ]
        
        sql_basics_tutorial = Tutorial(
            tutorial_id="SQL_BASICS_001",
            title="SQL Basics: Getting Started",
            description="Learn fundamental SQL concepts including SELECT statements, filtering, and basic queries.",
            difficulty=DifficultyLevel.BEGINNER,
            topic=Topic.SQL_BASICS,
            content="""
# SQL Basics Tutorial

## What is SQL?
SQL (Structured Query Language) is a programming language designed for managing and querying relational databases.

## Basic SELECT Statement
The SELECT statement is used to retrieve data from a database:

```sql
SELECT column1, column2 FROM table_name;
```

## Key Components:
- **SELECT**: Specifies which columns to retrieve
- **FROM**: Specifies which table to query
- **WHERE**: Filters rows based on conditions
- **ORDER BY**: Sorts the results

## Common Operators:
- **Comparison**: =, !=, <, >, <=, >=
- **Logical**: AND, OR, NOT
- **Pattern Matching**: LIKE, IN

## Best Practices:
- Always use meaningful column names
- Test queries with small result sets first
- Use WHERE clauses to limit data when possible
            """,
            exercises=sql_basics_exercises,
            prerequisites=[],
            learning_objectives=[
                "Understand basic SQL syntax",
                "Write simple SELECT statements",
                "Filter data using WHERE clauses",
                "Use comparison operators"
            ],
            estimated_time=60
        )
        
        # Query Optimization Tutorial
        query_optimization_exercises = [
            Exercise(
                exercise_id="QO_001",
                title="Using Indexes for Fast Queries",
                description="Create an index on the email column and write a query that would benefit from it.",
                difficulty=DifficultyLevel.INTERMEDIATE,
                topic=Topic.INDEXES,
                sql_query="CREATE INDEX idx_customer_email ON customers(email); SELECT * FROM customers WHERE email = 'customer1@example.com';",
                expected_columns=['customer_id', 'first_name', 'last_name', 'email', 'phone', 'registration_date', 'country'],
                expected_rows_range=(0, 1),
                hints=[
                    "Indexes speed up WHERE clause lookups",
                    "CREATE INDEX statement format: CREATE INDEX index_name ON table_name(column)",
                    "Look for columns used frequently in WHERE clauses"
                ],
                solution="CREATE INDEX idx_customer_email ON customers(email); SELECT * FROM customers WHERE email = 'customer1@example.com';",
                points=25,
                estimated_time=15
            )
        ]
        
        query_optimization_tutorial = Tutorial(
            tutorial_id="QUERY_OPT_001",
            title="Query Optimization and Indexes",
            description="Learn about database indexes and how they improve query performance.",
            difficulty=DifficultyLevel.INTERMEDIATE,
            topic=Topic.PERFORMANCE,
            content="""
# Query Optimization Tutorial

## What are Indexes?
Indexes are data structures that improve the speed of data retrieval operations on database tables.

## Types of Indexes:
1. **B-Tree Indexes**: Default type, good for range queries
2. **Hash Indexes**: Excellent for equality comparisons
3. **Bitmap Indexes**: Efficient for low cardinality columns
4. **Full-text Indexes**: For text search operations

## When to Create Indexes:
- Columns frequently used in WHERE clauses
- Columns used in JOIN operations
- Columns used for sorting (ORDER BY)
- Columns with high selectivity (many unique values)

## Index Best Practices:
- Don't over-index (impacts INSERT/UPDATE performance)
- Monitor index usage
- Consider maintenance overhead
- Use EXPLAIN to analyze query plans
            """,
            exercises=query_optimization_exercises,
            prerequisites=["SQL_BASICS_001"],
            learning_objectives=[
                "Understand different types of indexes",
                "Know when to create indexes",
                "Analyze query execution plans",
                "Optimize database performance"
            ],
            estimated_time=90
        )
        
        # Store tutorials
        self.tutorials[sql_basics_tutorial.tutorial_id] = sql_basics_tutorial
        self.tutorials[query_optimization_tutorial.tutorial_id] = query_optimization_tutorial
        
        # Store exercises
        for exercise in sql_basics_exercises + query_optimization_exercises:
            self.exercises[exercise.exercise_id] = exercise
    
    def get_tutorial(self, tutorial_id: str) -> Optional[Tutorial]:
        """Get tutorial by ID"""
        return self.tutorials.get(tutorial_id)
    
    def get_tutorials_by_topic(self, topic: Topic) -> List[Tutorial]:
        """Get all tutorials for a specific topic"""
        return [tutorial for tutorial in self.tutorials.values() if tutorial.topic == topic]
    
    def get_tutorials_by_difficulty(self, difficulty: DifficultyLevel) -> List[Tutorial]:
        """Get all tutorials for a specific difficulty level"""
        return [tutorial for tutorial in self.tutorials.values() if tutorial.difficulty == difficulty]
    
    def start_tutorial(self, student_id: str, tutorial_id: str) -> bool:
        """Start a tutorial for a student"""
        if tutorial_id not in self.tutorials:
            return False
        
        if student_id not in self.student_progress:
            self.student_progress[student_id] = StudentProgress(
                student_id=student_id,
                completed_exercises=[],
                current_tutorial=None,
                points_earned=0,
                total_time_spent=0,
                last_activity=time.time(),
                exercise_results={}
            )
        
        self.student_progress[student_id].current_tutorial = tutorial_id
        self.student_progress[student_id].last_activity = time.time()
        return True
    
    def submit_exercise(self, student_id: str, exercise_id: str, 
                       sql_query: str, execution_time: float) -> Dict[str, Any]:
        """Submit exercise solution"""
        if exercise_id not in self.exercises:
            return {'success': False, 'error': 'Exercise not found'}
        
        exercise = self.exercises[exercise_id]
        
        # Basic validation (in real implementation, would execute against actual database)
        is_correct = self._validate_exercise_solution(exercise, sql_query)
        
        result = {
            'exercise_id': exercise_id,
            'success': is_correct,
            'sql_query': sql_query,
            'execution_time': execution_time,
            'timestamp': time.time(),
            'points_earned': exercise.points if is_correct else 0,
            'feedback': self._generate_feedback(exercise, sql_query, is_correct)
        }
        
        # Update student progress
        if student_id in self.student_progress:
            progress = self.student_progress[student_id]
            progress.last_activity = time.time()
            progress.total_time_spent += exercise.estimated_time
            
            if exercise_id not in progress.completed_exercises and is_correct:
                progress.completed_exercises.append(exercise_id)
                progress.points_earned += exercise.points
            
            progress.exercise_results[exercise_id] = result
        
        return result
    
    def _validate_exercise_solution(self, exercise: Exercise, sql_query: str) -> bool:
        """Basic validation of exercise solution"""
        # Simple validation logic - in real implementation would:
        # 1. Execute the SQL query against a test database
        # 2. Compare results with expected output
        # 3. Check for correct syntax and logic
        
        # For educational purposes, use simple string matching
        expected_query = exercise.sql_query.lower().strip()
        submitted_query = sql_query.lower().strip()
        
        # Check for basic patterns
        if 'select' in expected_query and 'select' not in submitted_query:
            return False
        
        if 'where' in expected_query and 'where' not in submitted_query:
            return False
        
        if 'group by' in expected_query and 'group' not in submitted_query:
            return False
        
        # Basic syntax check
        if not submitted_query.endswith(';'):
            return False
        
        return True
    
    def _generate_feedback(self, exercise: Exercise, sql_query: str, is_correct: bool) -> str:
        """Generate feedback for exercise submission"""
        if is_correct:
            return f"Excellent! Your query correctly solves the '{exercise.title}' exercise. You earned {exercise.points} points."
        else:
            hints = exercise.hints
            feedback = f"Your query doesn't match the expected solution for '{exercise.title}'. "
            if hints:
                feedback += f"Here's a hint: {hints[0]}"
            return feedback
    
    def get_student_progress(self, student_id: str) -> Optional[Dict[str, Any]]:
        """Get student progress information"""
        if student_id not in self.student_progress:
            return None
        
        progress = self.student_progress[student_id]
        current_tutorial = None
        
        if progress.current_tutorial:
            current_tutorial = self.tutorials[progress.current_tutorial].to_dict()
        
        return {
            'progress': progress.to_dict(),
            'current_tutorial': current_tutorial,
            'completed_tutorials': len([t for t in self.tutorials.values() 
                                      if all(ex.exercise_id in progress.completed_exercises 
                                           for ex in t.exercises)]),
            'total_tutorials': len(self.tutorials),
            'completion_rate': len(progress.completed_exercises) / max(1, len(self.exercises)) * 100
        }
    
    def get_leaderboard(self, limit: int = 10) -> List[Dict[str, Any]]:
        """Get student leaderboard"""
        leaderboard = []
        
        for student_id, progress in self.student_progress.items():
            leaderboard.append({
                'student_id': student_id,
                'points_earned': progress.points_earned,
                'completed_exercises': len(progress.completed_exercises),
                'total_time_spent': progress.total_time_spent,
                'last_activity': progress.last_activity
            })
        
        # Sort by points (descending) then by completion count
        leaderboard.sort(key=lambda x: (-x['points_earned'], -x['completed_exercises']))
        
        return leaderboard[:limit]
    
    def get_tutorial_statistics(self) -> Dict[str, Any]:
        """Get tutorial system statistics"""
        total_students = len(self.student_progress)
        total_exercises = len(self.exercises)
        total_tutorials = len(self.tutorials)
        
        # Calculate average completion rate
        completion_rates = []
        total_points = 0
        total_time = 0
        
        for progress in self.student_progress.values():
            completion_rate = len(progress.completed_exercises) / max(1, total_exercises)
            completion_rates.append(completion_rate)
            total_points += progress.points_earned
            total_time += progress.total_time_spent
        
        avg_completion_rate = sum(completion_rates) / max(1, len(completion_rates)) * 100
        avg_points_per_student = total_points / max(1, total_students)
        avg_time_per_student = total_time / max(1, total_students)
        
        # Topic distribution
        topic_counts = {}
        for tutorial in self.tutorials.values():
            topic = tutorial.topic.value
            topic_counts[topic] = topic_counts.get(topic, 0) + 1
        
        # Difficulty distribution
        difficulty_counts = {}
        for tutorial in self.tutorials.values():
            difficulty = tutorial.difficulty.value
            difficulty_counts[difficulty] = difficulty_counts.get(difficulty, 0) + 1
        
        return {
            'total_students': total_students,
            'total_exercises': total_exercises,
            'total_tutorials': total_tutorials,
            'average_completion_rate': avg_completion_rate,
            'average_points_per_student': avg_points_per_student,
            'average_time_per_student': avg_time_per_student,
            'topic_distribution': topic_counts,
            'difficulty_distribution': difficulty_counts,
            'most_popular_tutorial': max(self.tutorials.values(), 
                                       key=lambda t: sum(1 for p in self.student_progress.values() 
                                                       if t.tutorial_id == p.current_tutorial)).title
        }
    
    def export_tutorial_data(self) -> Dict[str, Any]:
        """Export all tutorial data"""
        return {
            'tutorials': [tutorial.to_dict() for tutorial in self.tutorials.values()],
            'exercises': [exercise.to_dict() for exercise in self.exercises.values()],
            'schemas': self.schema_manager.schemas,
            'statistics': self.get_tutorial_statistics()
        }
    
    def import_tutorial_data(self, data: Dict[str, Any]) -> bool:
        """Import tutorial data"""
        try:
            # Import tutorials
            for tutorial_data in data.get('tutorials', []):
                tutorial = Tutorial(**tutorial_data)
                self.tutorials[tutorial.tutorial_id] = tutorial
            
            # Import exercises
            for exercise_data in data.get('exercises', []):
                exercise = Exercise(**exercise_data)
                self.exercises[exercise.exercise_id] = exercise
            
            return True
        except Exception:
            return False


def demonstrate_tutorial_system():
    """Demonstrate tutorial system functionality"""
    print("\n" + "="*60)
    print("DATABASE TUTORIAL SYSTEM DEMONSTRATION")
    print("="*60)
    
    manager = TutorialManager()
    
    print("\n1. Available Tutorials...")
    tutorials = list(manager.tutorials.values())
    for tutorial in tutorials:
        print(f"   - {tutorial.title} ({tutorial.difficulty.value})")
        print(f"     Topic: {tutorial.topic.value}")
        print(f"     Duration: {tutorial.estimated_time} minutes")
        print(f"     Exercises: {len(tutorial.exercises)}")
        print()
    
    print("2. Tutorial Content Example...")
    sql_basics = manager.get_tutorial("SQL_BASICS_001")
    if sql_basics:
        print(f"   Tutorial: {sql_basics.title}")
        print(f"   Learning Objectives:")
        for obj in sql_basics.learning_objectives:
            print(f"     - {obj}")
        print()
    
    print("3. Exercise Examples...")
    exercises = list(manager.exercises.values())[:3]
    for exercise in exercises:
        print(f"   Exercise: {exercise.title}")
        print(f"     Difficulty: {exercise.difficulty.value}")
        print(f"     Points: {exercise.points}")
        print(f"     Estimated Time: {exercise.estimated_time} minutes")
        print(f"     Hints: {len(exercise.hints)} available")
        print()
    
    print("4. Starting a Tutorial...")
    student_id = "student_001"
    success = manager.start_tutorial(student_id, "SQL_BASICS_001")
    if success:
        print(f"✓ Started tutorial for {student_id}")
    
    print("\n5. Submitting Exercise Solutions...")
    
    # Simulate exercise submissions
    submissions = [
        ("SB_001", "SELECT * FROM customers;", True, 0.05),
        ("SB_002", "SELECT first_name, last_name, email FROM customers;", True, 0.03),
        ("SB_003", "SELECT * FROM products WHERE price > 100;", True, 0.07)
    ]
    
    for exercise_id, query, correct, exec_time in submissions:
        result = manager.submit_exercise(student_id, exercise_id, query, exec_time)
        print(f"   Exercise {exercise_id}: {'✓ PASS' if result['success'] else '✗ FAIL'}")
        print(f"     Points earned: {result['points_earned']}")
        print(f"     Feedback: {result['feedback']}")
        print()
    
    print("6. Student Progress...")
    progress = manager.get_student_progress(student_id)
    if progress:
        print(f"   Student ID: {progress['progress']['student_id']}")
        print(f"   Completed Exercises: {len(progress['progress']['completed_exercises'])}")
        print(f"   Total Points: {progress['progress']['points_earned']}")
        print(f"   Completion Rate: {progress['completion_rate']:.1f}%")
        print(f"   Time Spent: {progress['progress']['total_time_spent']} minutes")
    
    print("\n7. Database Schema Examples...")
    schema_sql = manager.schema_manager.get_schema_sql('ecommerce')
    print("   E-Commerce Schema:")
    print("   " + schema_sql.replace('\n', '\n   ')[:200] + "...")
    
    print("\n8. Tutorial Statistics...")
    stats = manager.get_tutorial_statistics()
    print(f"   Total Students: {stats['total_students']}")
    print(f"   Total Tutorials: {stats['total_tutorials']}")
    print(f"   Total Exercises: {stats['total_exercises']}")
    print(f"   Average Completion Rate: {stats['average_completion_rate']:.1f}%")
    print(f"   Topic Distribution: {stats['topic_distribution']}")
    
    print("\n9. Creating Sample Students...")
    sample_students = [
        ("student_001", "Alice Johnson", 85, 120),
        ("student_002", "Bob Smith", 92, 135),
        ("student_003", "Carol Davis", 78, 100)
    ]
    
    for student_id, name, points, time_spent in sample_students:
        if student_id not in manager.student_progress:
            manager.student_progress[student_id] = StudentProgress(
                student_id=student_id,
                completed_exercises=[f"SB_{i:03d}" for i in range(1, 10)],
                current_tutorial=None,
                points_earned=points,
                total_time_spent=time_spent,
                last_activity=time.time(),
                exercise_results={}
            )
    
    print("\n10. Leaderboard...")
    leaderboard = manager.get_leaderboard(5)
    print("   Top Students:")
    for i, student in enumerate(leaderboard, 1):
        print(f"     {i}. {student['student_id']}: {student['points_earned']} points, {student['completed_exercises']} exercises")


def demonstrate_learning_paths():
    """Demonstrate structured learning paths"""
    print("\n" + "="*60)
    print("STRUCTURED LEARNING PATHS")
    print("="*60)
    
    manager = TutorialManager()
    
    learning_paths = {
        "Beginner SQL Developer": [
            "SQL_BASICS_001",
            "QUERIES_001",
            "JOINS_001",
            "AGGREGATION_001"
        ],
        "Database Administrator": [
            "SQL_BASICS_001",
            "INDEXES_001",
            "PERFORMANCE_001",
            "SECURITY_001",
            "TRANSACTIONS_001"
        ],
        "Data Analyst": [
            "SQL_BASICS_001",
            "QUERIES_001",
            "AGGREGATION_001",
            "SUBQUERIES_001"
        ]
    }
    
    print("\n1. Available Learning Paths...")
    for path_name, tutorials in learning_paths.items():
        print(f"   {path_name}:")
        for tutorial_id in tutorials:
            if tutorial_id in manager.tutorials:
                tutorial = manager.tutorials[tutorial_id]
                print(f"     - {tutorial.title}")
        print()
    
    print("2. Personalized Learning Recommendations...")
    
    # Simulate student skill assessment
    student_skills = {
        "Basic SQL": True,
        "Complex Queries": False,
        "Joins": False,
        "Aggregation": False,
        "Indexes": False,
        "Performance": False
    }
    
    recommendations = []
    
    if not student_skills["Basic SQL"]:
        recommendations.append("Complete SQL Basics tutorial first")
    elif not student_skills["Complex Queries"]:
        recommendations.append("Learn advanced query techniques")
    elif not student_skills["Joins"]:
        recommendations.append("Master table joins for relational data")
    elif not student_skills["Aggregation"]:
        recommendations.append("Learn aggregation functions for data analysis")
    else:
        recommendations.append("Consider advanced topics like performance optimization")
    
    print("   Personalized Recommendations:")
    for rec in recommendations:
        print(f"     - {rec}")
    
    print("\n3. Skill Assessment Quiz...")
    
    quiz_questions = [
        {
            "question": "What does SELECT * FROM customers WHERE country = 'USA' do?",
            "options": [
                "A) Returns all customers from USA",
                "B) Creates a new table with USA customers",
                "C) Updates all customers to be from USA",
                "D) Deletes customers not from USA"
            ],
            "correct": "A"
        },
        {
            "question": "Which clause is used to sort query results?",
            "options": [
                "A) SORT BY",
                "B) ORDER BY",
                "C) GROUP BY",
                "D) ARRANGE BY"
            ],
            "correct": "B"
        },
        {
            "question": "What does a database index primarily improve?",
            "options": [
                "A) INSERT performance",
                "B) SELECT performance",
                "C) DELETE performance",
                "D) All operations equally"
            ],
            "correct": "B"
        }
    ]
    
    print("   Sample Quiz Questions:")
    for i, q in enumerate(quiz_questions, 1):
        print(f"     {i}. {q['question']}")
        for option in q['options']:
            print(f"        {option}")
        print(f"     Correct Answer: {q['correct']}")
        print()


def main():
    """Main demonstration function"""
    print("DATABASE TUTORIALS AND LEARNING SYSTEM")
    print("Structured SQL Education with Hands-on Exercises")
    print("="*80)
    
    try:
        demonstrate_tutorial_system()
        demonstrate_learning_paths()
        
        print("\n" + "="*80)
        print("TUTORIAL SYSTEM DEMO COMPLETED")
        print("="*80)
        print("\nKey Features Demonstrated:")
        print("✓ Structured tutorial content with learning objectives")
        print("✓ Hands-on SQL exercises with validation")
        print("✓ Progress tracking and scoring system")
        print("✓ Multiple database schemas for practice")
        print("✓ Personalized learning recommendations")
        print("✓ Skill assessment and quiz system")
        print("✓ Leaderboard and social learning features")
        print("✓ Tutorial analytics and statistics")
        print("\nThis educational system provides:")
        print("- Progressive learning from beginner to expert")
        print("- Hands-on practice with real database scenarios")
        print("- Immediate feedback on exercise submissions")
        print("- Structured curriculum aligned with industry needs")
        print("- Performance tracking and skill assessment")
        print("- Interactive learning with multiple database schemas")
        print("- Comprehensive tutorial analytics")
        
    except Exception as e:
        print(f"Error during demonstration: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    main()