"""
Educational VCS Server API
RESTful API for the educational version control system
"""

from flask import Flask, request, jsonify
from flask_cors import CORS
from flask_socketio import SocketIO, emit
import json
from pathlib import Path
from typing import Dict, List, Optional
import threading
import time

from ..core.repository import EducationalVCS
from ..core.conflict_resolution import ConflictResolution, CollaborativeEditor
from ..core.review_system import EducationalCodeReview, ReviewStatus, ReviewPriority
from ..core.grading_system import AssignmentGradingSystem, SubmissionStatus
from ..core.quality_analyzer import CodeQualityAnalyzer


app = Flask(__name__)
app.config['SECRET_KEY'] = 'educational_vcs_secret_key'
CORS(app, origins="*")
socketio = SocketIO(app, cors_allowed_origins="*")

# Global instances (in production, use proper dependency injection)
vcs_repos = {}
active_sessions = {}


@app.route('/api/repos', methods=['POST'])
def create_repository():
    """Create a new repository"""
    try:
        data = request.get_json()
        repo_path = data.get('path')
        repo_name = data.get('name', 'Educational Repository')
        
        if not repo_path:
            return jsonify({'error': 'Repository path is required'}), 400
        
        # Create repository
        repo = EducationalVCS(repo_path)
        repo.init()
        
        # Store in memory
        vcs_repos[repo_path] = repo
        
        return jsonify({
            'status': 'success',
            'message': f'Repository created at {repo_path}',
            'repository_path': repo_path
        })
    
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@app.route('/api/repos/<path:repo_path>/status', methods=['GET'])
def get_repo_status(repo_path):
    """Get repository status"""
    try:
        if repo_path not in vcs_repos:
            return jsonify({'error': 'Repository not found'}), 404
        
        repo = vcs_repos[repo_path]
        status = repo.get_status()
        
        return jsonify(status)
    
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@app.route('/api/repos/<path:repo_path>/add', methods=['POST'])
def stage_file(repo_path):
    """Stage a file for commit"""
    try:
        if repo_path not in vcs_repos:
            return jsonify({'error': 'Repository not found'}), 404
        
        data = request.get_json()
        file_path = data.get('file_path')
        content = data.get('content')
        user = data.get('user', 'anonymous')
        
        repo = vcs_repos[repo_path]
        result = repo.add(file_path, content, user)
        
        return jsonify(result)
    
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@app.route('/api/repos/<path:repo_path>/commit', methods=['POST'])
def create_commit(repo_path):
    """Create a commit"""
    try:
        if repo_path not in vcs_repos:
            return jsonify({'error': 'Repository not found'}), 404
        
        data = request.get_json()
        message = data.get('message')
        user = data.get('user', 'anonymous')
        
        repo = vcs_repos[repo_path]
        result = repo.commit(message, user)
        
        # Emit update to connected clients
        socketio.emit('repository_updated', {
            'repo_path': repo_path,
            'type': 'commit',
            'user': user,
            'message': message
        })
        
        return jsonify(result)
    
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@app.route('/api/repos/<path:repo_path>/branch', methods=['POST'])
def create_branch(repo_path):
    """Create a new branch"""
    try:
        if repo_path not in vcs_repos:
            return jsonify({'error': 'Repository not found'}), 404
        
        data = request.get_json()
        branch_name = data.get('branch_name')
        user = data.get('user', 'anonymous')
        
        repo = vcs_repos[repo_path]
        result = repo.branch(branch_name, user)
        
        return jsonify(result)
    
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@app.route('/api/repos/<path:repo_path>/checkout', methods=['POST'])
def checkout_branch(repo_path):
    """Switch to a branch"""
    try:
        if repo_path not in vcs_repos:
            return jsonify({'error': 'Repository not found'}), 404
        
        data = request.get_json()
        branch_name = data.get('branch_name')
        
        repo = vcs_repos[repo_path]
        result = repo.checkout(branch_name)
        
        return jsonify(result)
    
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@app.route('/api/repos/<path:repo_path>/merge', methods=['POST'])
def merge_branches(repo_path):
    """Merge branches"""
    try:
        if repo_path not in vcs_repos:
            return jsonify({'error': 'Repository not found'}), 404
        
        data = request.get_json()
        source_branch = data.get('source_branch')
        user = data.get('user', 'anonymous')
        message = data.get('message', '')
        
        repo = vcs_repos[repo_path]
        result = repo.merge(source_branch, user, message)
        
        return jsonify(result)
    
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@app.route('/api/repos/<path:repo_path>/log', methods=['GET'])
def get_commit_log(repo_path):
    """Get commit history"""
    try:
        if repo_path not in vcs_repos:
            return jsonify({'error': 'Repository not found'}), 404
        
        branch_name = request.args.get('branch')
        
        repo = vcs_repos[repo_path]
        log = repo.get_log(branch_name)
        
        return jsonify({'commits': log})
    
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@app.route('/api/repos/<path:repo_path>/conflicts/resolve', methods=['POST'])
def resolve_conflicts(repo_path):
    """Resolve merge conflicts"""
    try:
        if repo_path not in vcs_repos:
            return jsonify({'error': 'Repository not found'}), 404
        
        data = request.get_json()
        base_content = data.get('base_content')
        current_content = data.get('current_content')
        incoming_content = data.get('incoming_content')
        file_path = data.get('file_path')
        
        repo = vcs_repos[repo_path]
        conflict_resolver = ConflictResolution(repo)
        
        conflicts = conflict_resolver.detect_merge_conflicts(
            base_content, current_content, incoming_content, file_path
        )
        
        return jsonify(conflicts)
    
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@app.route('/api/reviews', methods=['POST'])
def create_review():
    """Create a code review request"""
    try:
        data = request.get_json()
        repo_path = data.get('repo_path')
        pr_id = data.get('pr_id')
        author = data.get('author')
        title = data.get('title')
        description = data.get('description')
        target_branch = data.get('target_branch')
        source_branch = data.get('source_branch')
        reviewers = data.get('reviewers', [])
        priority = ReviewPriority(data.get('priority', 'medium'))
        educational_focus = data.get('educational_focus', '')
        
        if repo_path not in vcs_repos:
            return jsonify({'error': 'Repository not found'}), 404
        
        repo = vcs_repos[repo_path]
        review_system = EducationalCodeReview(repo)
        
        review_id = review_system.create_review_request(
            pr_id, author, title, description, target_branch,
            source_branch, reviewers, priority, educational_focus
        )
        
        return jsonify({
            'status': 'success',
            'review_id': review_id
        })
    
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@app.route('/api/reviews/<review_id>/comments', methods=['POST'])
def add_review_comment(review_id):
    """Add a comment to a review"""
    try:
        data = request.get_json()
        repo_path = data.get('repo_path')
        file_path = data.get('file_path')
        author = data.get('author')
        content = data.get('content')
        line_number = data.get('line_number')
        comment_type = data.get('comment_type', 'general')
        
        if repo_path not in vcs_repos:
            return jsonify({'error': 'Repository not found'}), 404
        
        repo = vcs_repos[repo_path]
        review_system = EducationalCodeReview(repo)
        
        comment_id = review_system.add_comment(
            review_id, file_path, author, content, line_number, comment_type
        )
        
        return jsonify({
            'status': 'success',
            'comment_id': comment_id
        })
    
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@app.route('/api/reviews/<review_id>/status', methods=['PUT'])
def update_review_status(review_id):
    """Update review status"""
    try:
        data = request.get_json()
        repo_path = data.get('repo_path')
        status = data.get('status')
        updater = data.get('updater', 'anonymous')
        
        if repo_path not in vcs_repos:
            return jsonify({'error': 'Repository not found'}), 404
        
        repo = vcs_repos[repo_path]
        review_system = EducationalCodeReview(repo)
        
        result = review_system.update_review_status(
            review_id, ReviewStatus(status), updater
        )
        
        return jsonify(result)
    
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@app.route('/api/assignments', methods=['POST'])
def create_assignment():
    """Create a new assignment"""
    try:
        data = request.get_json()
        repo_path = data.get('repo_path')
        
        if repo_path not in vcs_repos:
            return jsonify({'error': 'Repository not found'}), 404
        
        repo = vcs_repos[repo_path]
        grading_system = AssignmentGradingSystem(repo)
        
        assignment_id = grading_system.create_assignment(data)
        
        return jsonify({
            'status': 'success',
            'assignment_id': assignment_id
        })
    
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@app.route('/api/assignments/<assignment_id>/submit', methods=['POST'])
def submit_assignment(assignment_id):
    """Submit an assignment"""
    try:
        data = request.get_json()
        repo_path = data.get('repo_path')
        student_id = data.get('student_id')
        commit_hash = data.get('commit_hash')
        branch = data.get('branch', 'main')
        files = data.get('files', [])
        
        if repo_path not in vcs_repos:
            return jsonify({'error': 'Repository not found'}), 404
        
        repo = vcs_repos[repo_path]
        grading_system = AssignmentGradingSystem(repo)
        
        submission_id = grading_system.submit_assignment(
            assignment_id, student_id, commit_hash, branch, files
        )
        
        return jsonify({
            'status': 'success',
            'submission_id': submission_id
        })
    
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@app.route('/api/assignments/<assignment_id>/grade', methods=['POST'])
def grade_assignment(assignment_id):
    """Grade a submission"""
    try:
        data = request.get_json()
        repo_path = data.get('repo_path')
        submission_id = data.get('submission_id')
        grader_id = data.get('grader_id')
        criteria_scores = data.get('criteria_scores', {})
        feedback = data.get('feedback', '')
        
        if repo_path not in vcs_repos:
            return jsonify({'error': 'Repository not found'}), 404
        
        repo = vcs_repos[repo_path]
        grading_system = AssignmentGradingSystem(repo)
        
        result = grading_system.grade_submission(
            submission_id, grader_id, criteria_scores, feedback
        )
        
        return jsonify(result)
    
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@app.route('/api/students/<student_id>/dashboard', methods=['GET'])
def get_student_dashboard(student_id):
    """Get student dashboard"""
    try:
        repo_path = request.args.get('repo_path')
        
        if repo_path not in vcs_repos:
            return jsonify({'error': 'Repository not found'}), 404
        
        repo = vcs_repos[repo_path]
        grading_system = AssignmentGradingSystem(repo)
        
        dashboard = grading_system.get_student_dashboard(student_id)
        
        return jsonify(dashboard)
    
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@app.route('/api/quality/analyze', methods=['POST'])
def analyze_code_quality():
    """Analyze code quality"""
    try:
        data = request.get_json()
        code = data.get('code')
        file_path = data.get('file_path', 'unknown.py')
        language = data.get('language', 'python')
        
        analyzer = CodeQualityAnalyzer()
        
        if language == 'python':
            report = analyzer.analyze_python_code(code, file_path)
        else:
            return jsonify({'error': 'Language not supported yet'}), 400
        
        return jsonify(report.to_dict())
    
    except Exception as e:
        return jsonify({'error': str(e)}), 500


# WebSocket endpoints for real-time collaboration
@socketio.on('connect')
def handle_connect():
    """Handle client connection"""
    print(f'Client connected: {request.sid}')
    emit('connected', {'message': 'Connected to Educational VCS server'})


@socketio.on('join_session')
def handle_join_session(data):
    """Join a collaborative editing session"""
    session_id = data.get('session_id')
    file_path = data.get('file_path')
    user = data.get('user')
    
    # Store session info
    if session_id not in active_sessions:
        active_sessions[session_id] = {
            'users': [],
            'file_path': file_path
        }
    
    active_sessions[session_id]['users'].append({
        'sid': request.sid,
        'user': user,
        'joined_at': time.time()
    })
    
    emit('joined_session', {
        'session_id': session_id,
        'users': active_sessions[session_id]['users']
    }, broadcast=True)


@socketio.on('edit_content')
def handle_edit_content(data):
    """Handle content editing in collaborative session"""
    session_id = data.get('session_id')
    content = data.get('content')
    changes = data.get('changes', [])
    user = data.get('user')
    
    if session_id in active_sessions:
        # Broadcast changes to all users in session
        emit('content_updated', {
            'session_id': session_id,
            'content': content,
            'changes': changes,
            'user': user
        }, broadcast=True, skip_sid=request.sid)


@socketio.on('disconnect')
def handle_disconnect():
    """Handle client disconnection"""
    print(f'Client disconnected: {request.sid}')
    
    # Clean up from active sessions
    for session_id, session in active_sessions.items():
        session['users'] = [
            u for u in session['users'] 
            if u['sid'] != request.sid
        ]


# Health check endpoint
@app.route('/health', methods=['GET'])
def health_check():
    """Health check endpoint"""
    return jsonify({
        'status': 'healthy',
        'version': '1.0.0',
        'active_repos': len(vcs_repos),
        'active_sessions': len(active_sessions)
    })


if __name__ == '__main__':
    socketio.run(app, host='0.0.0.0', port=5000, debug=True)
