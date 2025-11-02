#!/usr/bin/env python3
"""
Educational VCS Demonstration Script

This script demonstrates all the key features of the Educational VCS system
by simulating user interactions and showing the system capabilities.
"""

import sys
import os
from pathlib import Path
import json
import time

# Add the src directory to the Python path
sys.path.insert(0, str(Path(__file__).parent / "src"))

from core.repository import EducationalVCS
from core.conflict_resolution import ConflictResolution, CollaborativeEditor
from core.review_system import EducationalCodeReview, ReviewStatus, ReviewPriority
from core.grading_system import AssignmentGradingSystem, SubmissionStatus
from core.quality_analyzer import CodeQualityAnalyzer


def print_section(title):
    """Print a formatted section header"""
    print("\n" + "=" * 60)
    print(f"🎓 {title}")
    print("=" * 60)


def print_step(step, description):
    """Print a formatted step"""
    print(f"\n📋 Step {step}: {description}")
    print("-" * 40)


def demonstrate_core_vcs():
    """Demonstrate core VCS functionality"""
    print_section("Core Version Control System")
    
    # Initialize repository
    repo_path = "/workspace/demo-repos/demo-project"
    print_step(1, "Creating a new repository")
    
    vcs = EducationalVCS(repo_path)
    result = vcs.init()
    print(f"   ✅ {result['message']}")
    
    # Create initial files
    print_step(2, "Creating and staging files")
    
    readme_content = """# Demo Project

This is a demonstration project for the Educational VCS system.

## Features
- Version control
- Collaborative editing  
- Code quality analysis
- Educational tutorials

## Getting Started
1. Clone the repository
2. Create a branch
3. Make your changes
4. Commit and push
"""
    
    main_py_content = '''#!/usr/bin/env python3
"""
Main application file for the demo project
"""

def greet(name):
    """Greet a user with a message."""
    return f"Hello, {name}! Welcome to Educational VCS."

def main():
    """Main application entry point."""
    name = input("Enter your name: ")
    greeting = greet(name)
    print(greeting)

if __name__ == "__main__":
    main()
'''
    
    add_result1 = vcs.add("README.md", readme_content, "Demo Student")
    print(f"   ✅ Staged README.md - {add_result1['message']}")
    
    add_result2 = vcs.add("main.py", main_py_content, "Demo Student")  
    print(f"   ✅ Staged main.py - {add_result2['message']}")
    
    # Create commit
    print_step(3, "Creating initial commit")
    commit_result = vcs.commit("Add initial project files", "Demo Student")
    print(f"   ✅ {commit_result['message']}")
    print(f"   📝 Commit hash: {commit_result['commit_hash'][:8]}")
    
    # Create branch
    print_step(4, "Creating feature branch")
    branch_result = vcs.branch("feature/new-feature", "Demo Student")
    print(f"   ✅ {branch_result['message']}")
    
    # Make changes on branch
    print_step(5, "Making changes on feature branch")
    
    updated_main = '''#!/usr/bin/env python3
"""
Main application file for the demo project
Enhanced with better error handling
"""

def greet(name):
    """Greet a user with a message."""
    if not name or not name.strip():
        return "Hello, Anonymous! Please enter a valid name."
    return f"Hello, {name.strip()}! Welcome to Educational VCS."

def validate_input(user_input):
    """Validate user input."""
    return user_input and len(user_input.strip()) > 0

def main():
    """Main application entry point."""
    try:
        name = input("Enter your name: ")
        if validate_input(name):
            greeting = greet(name)
            print(greeting)
        else:
            print("Invalid input. Please enter a valid name.")
    except KeyboardInterrupt:
        print("\\nGoodbye!")
    except Exception as e:
        print(f"An error occurred: {e}")

if __name__ == "__main__":
    main()
'''
    
    add_result3 = vcs.add("main.py", updated_main, "Demo Student")
    print(f"   ✅ Updated main.py on feature branch")
    
    commit_result2 = vcs.commit("Add error handling and input validation", "Demo Student")
    print(f"   ✅ {commit_result2['message']}")
    
    # Show repository status
    print_step(6, "Repository status")
    status = vcs.get_status()
    print(f"   📁 Current branch: {status['current_branch']}")
    print(f"   🌿 Available branches: {', '.join(status['branches'])}")
    print(f"   📊 Total commits: {status['total_commits']}")
    
    # Show commit history
    print_step(7, "Commit history")
    log = vcs.get_log()
    for commit in log:
        print(f"   📝 {commit['hash']} - {commit['message']}")
        print(f"      👤 {commit['author']} at {commit['timestamp'][:19]}")


def demonstrate_conflict_resolution():
    """Demonstrate conflict resolution features"""
    print_section("Conflict Resolution System")
    
    # Simulate conflict scenario
    repo_path = "/workspace/demo-repos/conflict-demo"
    vcs = EducationalVCS(repo_path)
    vcs.init()
    
    print_step(1, "Simulating merge conflict scenario")
    
    # Base content
    base_content = """def calculate_area(length, width):
    return length * width

def calculate_perimeter(length, width):
    return 2 * (length + width)
"""
    
    # Current branch changes
    current_content = """def calculate_area(length, width):
    # Added validation
    if length <= 0 or width <= 0:
        raise ValueError("Dimensions must be positive")
    return length * width

def calculate_perimeter(length, width):
    # Added validation  
    if length <= 0 or width <= 0:
        raise ValueError("Dimensions must be positive")
    return 2 * (length + width)
"""
    
    # Incoming branch changes
    incoming_content = """def calculate_area(length, width):
    # Added type checking
    if not isinstance(length, (int, float)) or not isinstance(width, (int, float)):
        raise TypeError("Dimensions must be numbers")
    return length * width

def calculate_perimeter(length, width):
    # Added type checking
    if not isinstance(length, (int, float)) or not isinstance(width, (int, float)):
        raise TypeError("Dimensions must be numbers")  
    return 2 * (length + width)
"""
    
    # Detect conflicts
    print_step(2, "Detecting conflicts")
    
    conflict_resolver = ConflictResolution(vcs)
    conflicts = conflict_resolver.detect_merge_conflicts(
        base_content, current_content, incoming_content, "geometry.py"
    )
    
    if conflicts['has_conflicts']:
        print(f"   ⚠️  Detected {conflicts['conflict_count']} conflicts")
        for i, conflict in enumerate(conflicts['conflicts'], 1):
            print(f"   🔍 Conflict {i}: {conflict['type']}")
            print(f"      📍 Location: Line {conflict['line_range']['start']}-{conflict['line_range']['end']}")
            print(f"      💡 Hint: {conflict['hint']}")
    else:
        print("   ✅ No conflicts detected")
    
    print_step(3, "Educational conflict resolution advice")
    advice = conflict_resolver.get_collaboration_advice('content_conflict')
    print(f"   📚 Description: {advice['description']}")
    print("   💬 Communication tips:")
    for tip in advice['communication_tips']:
        print(f"      • {tip}")
    print("   🌟 Best practices:")
    for practice in advice['best_practices']:
        print(f"      • {practice}")


def demonstrate_code_review():
    """Demonstrate code review system"""
    print_section("Code Review System")
    
    repo_path = "/workspace/demo-repos/review-demo"
    vcs = EducationalVCS(repo_path)
    vcs.init()
    
    print_step(1, "Creating pull request")
    
    review_system = EducationalCodeReview(vcs)
    review_id = review_system.create_review_request(
        "pr_auth_001",
        "alice@student.edu",
        "Add Authentication System",
        "Implement user authentication with secure password hashing",
        "main",
        "feature/auth-system",
        ["bob@student.edu", "carol@student.edu"],
        ReviewPriority.HIGH,
        "Learning secure coding practices"
    )
    
    print(f"   ✅ Created PR #review_{review_id}")
    print(f"   📝 Title: Add Authentication System")
    print(f"   🎯 Priority: {ReviewPriority.HIGH.value}")
    print(f"   👥 Reviewers: bob@student.edu, carol@student.edu")
    
    print_step(2, "Adding review comments")
    
    # Add comments
    comment1 = review_system.add_comment(
        review_id, 
        "auth.py",
        "alice@student.edu",
        "Great implementation! Consider adding rate limiting for failed login attempts.",
        25,
        "suggestion"
    )
    print(f"   💬 Comment 1 added by alice@student.edu")
    
    comment2 = review_system.add_comment(
        review_id,
        "password.py",
        "bob@student.edu", 
        "Should we use bcrypt instead of SHA-256 for better security?",
        15,
        "question"
    )
    print(f"   ❓ Question added by bob@student.edu")
    
    comment3 = review_system.add_comment(
        review_id,
        "utils.py",
        "carol@student.edu",
        "Excellent error handling in the validation functions!",
        8,
        "praise"
    )
    print(f"   🎉 Praise added by carol@student.edu")
    
    print_step(3, "Review process workflow")
    
    # Update review status
    status_result = review_system.update_review_status(
        review_id, 
        ReviewStatus.IN_REVIEW,
        "instructor@edu"
    )
    print(f"   🔄 Status updated: {status_result['message']}")
    
    # Get review summary
    summary = review_system.get_review_summary(review_id)
    print(f"   📊 Total comments: {summary['total_comments']}")
    print(f"   ✅ Resolved: {summary['resolved_comments']}")
    print(f"   ⏳ Unresolved: {summary['unresolved_comments']}")
    
    # Educational feedback
    print("   🎓 Educational feedback:")
    for point in summary['educational_feedback']['learning_points']:
        print(f"      • {point}")


def demonstrate_assignment_grading():
    """Demonstrate assignment and grading system"""
    print_section("Assignment and Grading System")
    
    repo_path = "/workspace/demo-repos/grading-demo"
    vcs = EducationalVCS(repo_path)
    vcs.init()
    
    print_step(1, "Creating assignment")
    
    grading_system = AssignmentGradingSystem(vcs)
    
    assignment_data = {
        "title": "Git Basics Exercise",
        "description": "Practice basic Git operations including commit, branch, and merge",
        "course_id": "CS201",
        "instructor": "professor@edu",
        "due_date": "2024-12-20T23:59:59",
        "max_points": 100,
        "criteria": [
            {"name": "Repository Setup", "description": "Properly initialized repository", 
             "max_points": 20, "weight": 0.2, "rubric_description": "Repository structure and initialization"},
            {"name": "Commit Quality", "description": "Clear and descriptive commit messages",
             "max_points": 30, "weight": 0.3, "rubric_description": "Message clarity and descriptiveness"},
            {"name": "Branching", "description": "Successful creation and use of branches",
             "max_points": 30, "weight": 0.3, "rubric_description": "Branch creation and management"},
            {"name": "Merge Conflicts", "description": "Proper handling of merge conflicts",
             "max_points": 20, "weight": 0.2, "rubric_description": "Conflict resolution skills"}
        ],
        "allowed_languages": ["python", "javascript"],
        "required_files": ["README.md", "main.py"],
        "plagiarism_check": True,
        "peer_review_required": True
    }
    
    assignment_id = grading_system.create_assignment(assignment_data)
    print(f"   ✅ Created assignment: {assignment_id}")
    print(f"   📚 Title: {assignment_data['title']}")
    print(f"   📅 Due: {assignment_data['due_date'][:10]}")
    print(f"   📊 Max points: {assignment_data['max_points']}")
    
    print_step(2, "Student submission")
    
    submission_id = grading_system.submit_assignment(
        assignment_id,
        "student123",
        "abc123def456",
        "main",
        ["README.md", "main.py", "test.py"]
    )
    
    print(f"   📤 Submission created: {submission_id}")
    print(f"   👤 Student: student123")
    print(f"   📁 Files: {', '.join(['README.md', 'main.py', 'test.py'])}")
    
    print_step(3, "Grading submission")
    
    criteria_scores = {
        "Repository Setup": 18,
        "Commit Quality": 25, 
        "Branching": 28,
        "Merge Conflicts": 20
    }
    
    grading_result = grading_system.grade_submission(
        submission_id,
        "professor@edu",
        criteria_scores,
        "Good work overall! Excellent conflict resolution. Consider improving commit message clarity."
    )
    
    print(f"   📊 Grading completed:")
    print(f"      Score: {grading_result['grade']}/100")
    print(f"      Percentage: {grading_result['percentage']:.1f}%")
    print(f"      Letter grade: {grading_result['letter_grade']}")
    
    print_step(4, "Student dashboard")
    
    dashboard = grading_system.get_student_dashboard("student123")
    print(f"   📈 Student progress:")
    print(f"      Total assignments: {dashboard['total_assignments']}")
    print(f"      Submitted: {dashboard['submitted_assignments']}")
    print(f"      Graded: {dashboard['graded_assignments']}")
    print(f"      Average grade: {dashboard['average_grade']:.1f}")
    print(f"      Completion rate: {dashboard['completion_rate']:.1f}%")


def demonstrate_quality_analysis():
    """Demonstrate code quality analysis"""
    print_section("Code Quality Analysis")
    
    print_step(1, "Analyzing Python code")
    
    # Code with various issues
    problematic_code = '''
def calc(n):
    x=sum(n)/len(n)
    return x

def f(i):
    if i>0:
        return True
    else:
        return False

def process_data(data, settings, options, config, parameters, flags, debug, verbose):
    result = []
    for item in data:
        if item:
            if settings.get('enabled'):
                if options.get('option1'):
                    if config.get('setting1'):
                        if parameters.get('param1'):
                            if flags.get('flag1'):
                                if debug:
                                    if verbose:
                                        value = item['value'] * 2
                                        if value > 100:
                                            if value < 200:
                                                processed = value + 10
                                                if processed < 150:
                                                    result.append(processed)
    return result

# No docstring for this function
def old_function():
    pass
'''
    
    analyzer = CodeQualityAnalyzer()
    report = analyzer.analyze_python_code(problematic_code, "demo_code.py")
    
    print(f"   🔍 Analysis completed for: {report.file_path}")
    print(f"   📊 Overall quality score: {report.overall_score:.1f}/100")
    print(f"   🎯 Grade: {get_score_grade(report.overall_score)}")
    
    print_step(2, "Issues found")
    
    if report.issues:
        print(f"   ⚠️  Found {len(report.issues)} issues:")
        for i, issue in enumerate(report.issues, 1):
            severity_icon = get_severity_icon(issue.severity)
            print(f"      {i}. {severity_icon} {issue.type.replace('_', ' ').title()}")
            print(f"         📝 {issue.message}")
            print(f"         💡 {issue.suggestion}")
            if issue.educational_note:
                print(f"         🎓 Note: {issue.educational_note}")
    else:
        print("   ✅ No issues found!")
    
    print_step(3, "Code strengths")
    
    if report.strengths:
        print("   🌟 Identified strengths:")
        for strength in report.strengths:
            print(f"      • {strength}")
    else:
        print("   🤔 No specific strengths identified")
    
    print_step(4, "Educational feedback")
    
    feedback = report.educational_feedback
    if feedback.get('learning_objectives'):
        print("   📚 Learning objectives:")
        for obj in feedback['learning_objectives']:
            print(f"      • {obj}")
    
    if feedback.get('concepts_to_review'):
        print("   🔍 Concepts to review:")
        for concept in feedback['concepts_to_review']:
            print(f"      • {concept}")
    
    if feedback.get('next_steps'):
        print("   🚀 Next steps:")
        for step in feedback['next_steps']:
            print(f"      • {step}")
    
    print_step(5, "Recommended resources")
    
    resources = analyzer.get_recommended_resources(report.issues)
    if resources['books']:
        print("   📖 Recommended books:")
        for book in resources['books'][:2]:
            print(f"      • {book}")
    
    if resources['articles']:
        print("   📰 Helpful articles:")
        for article in resources['articles'][:2]:
            print(f"      • {article}")


def get_score_grade(score):
    """Convert numeric score to letter grade"""
    if score >= 90: return 'A'
    elif score >= 80: return 'B'
    elif score >= 70: return 'C'
    elif score >= 60: return 'D'
    else: return 'F'


def get_severity_icon(severity):
    """Get icon for severity level"""
    icons = {
        'excellent': '🟢',
        'good': '🔵',
        'needs_improvement': '🟡',
        'poor': '🔴'
    }
    return icons.get(severity.value, '⚪')


def main():
    """Main demonstration function"""
    print("""
╔══════════════════════════════════════════════════════════════════════════╗
║                  Educational VCS System Demo                            ║
║                                                                          ║
║  This demonstration showcases all the key features of the               ║
║  Educational Version Control System including:                          ║
║                                                                          ║
║  • Core version control operations (commits, branches, merges)          ║
║  • Conflict resolution with educational guidance                        ║
║  • Collaborative editing and real-time features                         ║
║  • Code review workflows with peer learning                             ║
║  • Assignment submission and automated grading                          ║
║  • Code quality analysis with educational feedback                      ║
╚══════════════════════════════════════════════════════════════════════════╝
    """)
    
    try:
        # Run all demonstrations
        demonstrate_core_vcs()
        demonstrate_conflict_resolution()
        demonstrate_code_review()
        demonstrate_assignment_grading()
        demonstrate_quality_analysis()
        
        print("\n" + "=" * 60)
        print("🎉 Demonstration Complete!")
        print("=" * 60)
        print("""
🌟 All features have been demonstrated successfully!

🚀 To try the full interactive system:
   1. Run: python setup.py
   2. Open: http://localhost:3000
   3. Explore all the educational features!

💡 This system provides a complete learning environment for version control
   concepts with hands-on practice, peer collaboration, and automated feedback.

📚 Key Learning Outcomes:
   • Understanding Git and version control concepts
   • Collaborative development workflows  
   • Code review and quality assessment
   • Professional software development practices
   • Peer learning and knowledge sharing

Happy learning! 🎓✨
        """)
        
    except Exception as e:
        print(f"\n❌ Demonstration failed: {e}")
        print("   Please check the error and try again.")


if __name__ == "__main__":
    main()
