"""
Educational VCS Examples and Demonstrations

This module contains example code and scenarios for learning version control
concepts using the Educational VCS system.
"""

# Example 1: Basic Git-like operations
def git_basics_example():
    """
    Demonstrates basic version control concepts
    """
    # Repository initialization
    repo_path = "/workspace/demo-repos/git-basics"
    
    # Creating commits
    commit_message = "Add user authentication function"
    author = "Student Name"
    
    # File changes
    file_content = '''
def authenticate_user(username, password):
    """Authenticate user with username and password."""
    # TODO: Implement actual authentication
    return username == "admin" and password == "password123"

def hash_password(password):
    """Hash password for storage."""
    import hashlib
    return hashlib.sha256(password.encode()).hexdigest()
'''
    
    # Branch creation
    feature_branch = "feature/user-auth"
    main_branch = "main"
    
    # Merge operation
    merge_message = "Merge user authentication feature"
    
    print("Git Basics Example:")
    print("- Repository:", repo_path)
    print("- Commit:", commit_message)
    print("- Author:", author)
    print("- Branch created:", feature_branch)
    print("- Merge message:", merge_message)

# Example 2: Collaborative editing scenario
def collaborative_example():
    """
    Shows how collaborative editing works
    """
    # Multiple users editing the same file
    users = ["Alice", "Bob", "Carol"]
    
    # Real-time changes
    changes = [
        {"user": "Alice", "operation": "insert", "line": 5, "content": "# Added by Alice"},
        {"user": "Bob", "operation": "modify", "line": 10, "content": "# Modified by Bob"},
        {"user": "Carol", "operation": "insert", "line": 15, "content": "# Added by Carol"}
    ]
    
    # Conflict resolution
    conflicts = [
        {
            "type": "concurrent_edit",
            "file": "src/main.py",
            "line": 10,
            "alice_version": "# Bob's version",
            "carol_version": "# Carol's version",
            "resolution": "Combined both changes"
        }
    ]
    
    print("Collaborative Example:")
    print("Users:", users)
    print("Changes:", len(changes))
    print("Conflicts:", len(conflicts))

# Example 3: Code review workflow
def code_review_example():
    """
    Demonstrates the code review process
    """
    # Pull request details
    pull_request = {
        "title": "Add user authentication",
        "description": "Implement basic user authentication with password hashing",
        "author": "student123",
        "reviewers": ["instructor", "peer1"],
        "status": "pending_review"
    }
    
    # Review comments
    comments = [
        {
            "reviewer": "instructor",
            "line": 5,
            "type": "suggestion",
            "content": "Consider adding input validation for empty passwords",
            "resolved": False
        },
        {
            "reviewer": "peer1", 
            "line": 12,
            "type": "question",
            "content": "Should we use a stronger hashing algorithm?",
            "resolved": True
        },
        {
            "reviewer": "instructor",
            "line": 20,
            "type": "praise",
            "content": "Great error handling!",
            "resolved": True
        }
    ]
    
    # Approval workflow
    review_status = {
        "instructor": "approved",
        "peer1": "approved", 
        "overall_status": "approved",
        "can_merge": True
    }
    
    print("Code Review Example:")
    print("PR:", pull_request["title"])
    print("Comments:", len(comments))
    print("Status:", review_status["overall_status"])

# Example 4: Assignment submission
def assignment_example():
    """
    Shows assignment submission and grading workflow
    """
    # Assignment details
    assignment = {
        "id": "CS201-001",
        "title": "Git Basics Exercise",
        "description": "Practice basic Git operations",
        "criteria": [
            {"name": "Repository Creation", "weight": 0.2},
            {"name": "Commit Messages", "weight": 0.3},
            {"name": "Branching", "weight": 0.3},
            {"name": "Merge Conflicts", "weight": 0.2}
        ],
        "max_points": 100
    }
    
    # Student submission
    submission = {
        "student_id": "student123",
        "commit_hash": "abc123def456",
        "files": ["main.py", "README.md", "test.py"],
        "submitted_at": "2024-12-04T10:30:00Z",
        "status": "submitted"
    }
    
    # Grading
    grades = {
        "Repository Creation": {"score": 18, "max": 20},
        "Commit Messages": {"score": 28, "max": 30},
        "Branching": {"score": 25, "max": 30},
        "Merge Conflicts": {"score": 20, "max": 20}
    }
    
    total_score = sum(g["score"] for g in grades.values())
    percentage = (total_score / assignment["max_points"]) * 100
    
    print("Assignment Example:")
    print("Assignment:", assignment["title"])
    print("Student:", submission["student_id"])
    print("Total Score:", total_score, "/", assignment["max_points"])
    print("Percentage:", f"{percentage:.1f}%")

# Example 5: Code quality analysis
def quality_analysis_example():
    """
    Demonstrates automated code quality analysis
    """
    # Code being analyzed
    code = '''
def calculate_average(numbers):
    """
    Calculate the average of a list of numbers.
    
    Args:
        numbers (list): List of numerical values
        
    Returns:
        float: The average value
    """
    if not numbers:
        return 0.0
    
    return sum(numbers) / len(numbers)


def process_user_input(user_input):
    # Poor naming and no validation
    x = float(user_input)
    if x > 0:
        return True
    else:
        return False


# Very long function - needs refactoring
def complex_function(data, settings, options, config, parameters, flags, debug):
    result = []
    for item in data:
        if item:
            if settings.get('enabled'):
                if options.get('option1'):
                    if config.get('setting1'):
                        if parameters.get('param1'):
                            if flags.get('flag1'):
                                value = item['value'] * 2
                                if value > 100:
                                    if value < 200:
                                        processed = value + 10
                                        if processed < 150:
                                            result.append(processed)
    return result
'''
    
    # Quality issues detected
    issues = [
        {
            "type": "naming",
            "severity": "needs_improvement",
            "message": "Function 'complex_function' has poor naming",
            "suggestion": "Use descriptive names that explain the function's purpose",
            "line": 25
        },
        {
            "type": "complexity",
            "severity": "poor", 
            "message": "Function 'complex_function' is too long (27 lines)",
            "suggestion": "Break this function into smaller, focused functions",
            "line": 25
        },
        {
            "type": "nesting",
            "severity": "needs_improvement",
            "message": "Deep nesting detected (5 levels)",
            "suggestion": "Reduce nesting by using early returns or extracting logic",
            "line": 30
        },
        {
            "type": "documentation",
            "severity": "good",
            "message": "Function 'calculate_average' has good documentation",
            "suggestion": "Great job with the docstring!",
            "line": 1
        }
    ]
    
    # Quality score calculation
    overall_score = 75.5  # Based on issues weighted by severity
    
    # Educational feedback
    feedback = {
        "learning_objectives": [
            "Function Decomposition",
            "Code Readability",
            "Naming Conventions"
        ],
        "strengths": [
            "Good documentation for calculate_average function",
            "Proper error handling for empty lists"
        ],
        "areas_for_improvement": [
            "Break down complex_function into smaller parts",
            "Use more descriptive variable names",
            "Reduce nesting levels for better readability"
        ],
        "next_steps": [
            "Refactor complex_function using extract method pattern",
            "Review SOLID principles for better design",
            "Practice writing smaller, focused functions"
        ]
    }
    
    print("Quality Analysis Example:")
    print("Overall Score:", overall_score)
    print("Issues Found:", len(issues))
    print("Feedback Categories:", len(feedback))

def tutorial_scenarios():
    """
    Example scenarios for the interactive tutorials
    """
    scenarios = [
        {
            "title": "First Repository",
            "description": "Create your first repository and make an initial commit",
            "steps": [
                "1. Initialize a new repository",
                "2. Create a README.md file",
                "3. Stage the file with 'git add'",
                "4. Commit with a descriptive message"
            ],
            "learning_objectives": [
                "Understand repository structure",
                "Learn basic Git workflow",
                "Practice commit messages"
            ]
        },
        {
            "title": "Branching Basics", 
            "description": "Create a feature branch and merge it back to main",
            "steps": [
                "1. Create a new branch called 'feature/new-feature'",
                "2. Make changes on the feature branch",
                "3. Switch back to main branch",
                "4. Merge the feature branch",
                "5. Handle any merge conflicts"
            ],
            "learning_objectives": [
                "Understand branch concept",
                "Learn to create and switch branches",
                "Practice merging branches"
            ]
        },
        {
            "title": "Collaborative Workflow",
            "description": "Work with others using a shared repository",
            "steps": [
                "1. Clone the shared repository",
                "2. Create a feature branch",
                "3. Make your changes",
                "4. Push your branch to remote",
                "5. Create a pull request",
                "6. Respond to review feedback"
            ],
            "learning_objectives": [
                "Learn collaborative development",
                "Practice code reviews",
                "Understand team workflows"
            ]
        }
    ]
    
    print("Tutorial Scenarios:")
    for scenario in scenarios:
        print(f"\n{scenario['title']}:")
        print(f"  Description: {scenario['description']}")
        print(f"  Steps: {len(scenario['steps'])}")
        print(f"  Objectives: {len(scenario['learning_objectives'])}")

if __name__ == "__main__":
    print("Educational VCS Examples")
    print("=" * 50)
    
    git_basics_example()
    print()
    
    collaborative_example()
    print()
    
    code_review_example()
    print()
    
    assignment_example()
    print()
    
    quality_analysis_example()
    print()
    
    tutorial_scenarios()
