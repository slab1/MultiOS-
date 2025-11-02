"""
Core Repository class for educational version control system
Implements Git-like functionality with educational enhancements
"""

import os
import json
import hashlib
import shutil
from datetime import datetime
from typing import Dict, List, Optional, Set, Tuple, Any
from pathlib import Path
from dataclasses import dataclass, asdict
import zlib


@dataclass
class Commit:
    """Represents a commit in the repository"""
    hash: str
    parent_hashes: List[str]
    author: str
    message: str
    timestamp: datetime
    changes: Dict[str, str]  # file_path -> content_hash
    branch: str
    
    def to_dict(self) -> Dict:
        return {
            'hash': self.hash,
            'parent_hashes': self.parent_hashes,
            'author': self.author,
            'message': self.message,
            'timestamp': self.timestamp.isoformat(),
            'changes': self.changes,
            'branch': self.branch
        }
    
    @classmethod
    def from_dict(cls, data: Dict) -> 'Commit':
        return cls(
            hash=data['hash'],
            parent_hashes=data['parent_hashes'],
            author=data['author'],
            message=data['message'],
            timestamp=datetime.fromisoformat(data['timestamp']),
            changes=data['changes'],
            branch=data['branch']
        )


@dataclass
class Branch:
    """Represents a branch in the repository"""
    name: str
    head_commit: str
    created_at: datetime
    created_by: str
    description: str = ""
    
    def to_dict(self) -> Dict:
        return {
            'name': self.name,
            'head_commit': self.head_commit,
            'created_at': self.created_at.isoformat(),
            'created_by': self.created_by,
            'description': self.description
        }
    
    @classmethod
    def from_dict(cls, data: Dict) -> 'Branch':
        return cls(
            name=data['name'],
            head_commit=data['head_commit'],
            created_at=datetime.fromisoformat(data['created_at']),
            created_by=data['created_by'],
            description=data.get('description', '')
        )


class EducationalVCS:
    """
    Main repository class for educational version control
    Provides Git-like functionality with educational enhancements
    """
    
    def __init__(self, repo_path: str):
        self.repo_path = Path(repo_path)
        self.objects_dir = self.repo_path / '.edu_vcs' / 'objects'
        self.refs_dir = self.repo_path / '.edu_vcs' / 'refs'
        self.config_file = self.repo_path / '.edu_vcs' / 'config.json'
        self.commits_file = self.repo_path / '.edu_vcs' / 'commits.json'
        self.branches_file = self.repo_path / '.edu_vcs' / 'branches.json'
        self.conflicts_dir = self.repo_path / '.edu_vcs' / 'conflicts'
        
        self._init_repository()
    
    def _init_repository(self):
        """Initialize the repository structure"""
        self.objects_dir.mkdir(parents=True, exist_ok=True)
        self.refs_dir.mkdir(parents=True, exist_ok=True)
        self.conflicts_dir.mkdir(parents=True, exist_ok=True)
        
        # Create default config
        if not self.config_file.exists():
            config = {
                'repository_name': 'Educational Repository',
                'created_at': datetime.now().isoformat(),
                'main_branch': 'main',
                'collaborators': [],
                'permissions': {}
            }
            self._save_json(self.config_file, config)
        
        # Initialize data files
        if not self.commits_file.exists():
            self._save_json(self.commits_file, {})
        if not self.branches_file.exists():
            self._save_json(self.branches_file, {})
        
        # Create main branch
        if not (self.refs_dir / 'main').exists():
            self._create_initial_commit('main', 'System', 'Initial commit')
    
    def _save_json(self, file_path: Path, data: Any):
        """Save data to JSON file"""
        with open(file_path, 'w') as f:
            json.dump(data, f, indent=2)
    
    def _load_json(self, file_path: Path) -> Any:
        """Load data from JSON file"""
        with open(file_path, 'r') as f:
            return json.load(f)
    
    def _calculate_hash(self, content: str) -> str:
        """Calculate SHA-256 hash of content"""
        return hashlib.sha256(content.encode()).hexdigest()
    
    def _store_object(self, content: str, obj_type: str) -> str:
        """Store an object and return its hash"""
        obj_hash = self._calculate_hash(content)
        obj_dir = self.objects_dir / obj_type
        obj_dir.mkdir(exist_ok=True)
        
        obj_path = obj_dir / f"{obj_hash}.obj"
        if not obj_path.exists():
            compressed = zlib.compress(content.encode())
            with open(obj_path, 'wb') as f:
                f.write(compressed)
        
        return obj_hash
    
    def _get_object(self, obj_hash: str, obj_type: str) -> str:
        """Retrieve an object by its hash"""
        obj_path = self.objects_dir / obj_type / f"{obj_hash}.obj"
        if not obj_path.exists():
            raise ValueError(f"Object {obj_hash} not found")
        
        with open(obj_path, 'rb') as f:
            compressed = f.read()
        return zlib.decompress(compressed).decode()
    
    def _get_commits(self) -> Dict[str, Commit]:
        """Load all commits from storage"""
        commits_data = self._load_json(self.commits_file)
        return {hash_str: Commit.from_dict(data) for hash_str, data in commits_data.items()}
    
    def _save_commits(self, commits: Dict[str, Commit]):
        """Save commits to storage"""
        commits_data = {hash_str: commit.to_dict() for hash_str, commit in commits.items()}
        self._save_json(self.commits_file, commits_data)
    
    def _get_branches(self) -> Dict[str, Branch]:
        """Load all branches from storage"""
        branches_data = self._load_json(self.branches_file)
        return {name: Branch.from_dict(data) for name, data in branches_data.items()}
    
    def _save_branches(self, branches: Dict[str, Branch]):
        """Save branches to storage"""
        branches_data = {name: branch.to_dict() for name, branch in branches.items()}
        self._save_json(self.branches_file, branches_data)
    
    def _create_initial_commit(self, branch_name: str, author: str, message: str):
        """Create the initial commit for a new branch"""
        commit = Commit(
            hash="",
            parent_hashes=[],
            author=author,
            message=message,
            timestamp=datetime.now(),
            changes={},
            branch=branch_name
        )
        
        # Create commit object
        commit_hash = self._store_object(json.dumps(commit.to_dict()), 'commits')
        commit.hash = commit_hash
        
        # Save commits and update refs
        commits = self._get_commits()
        commits[commit_hash] = commit
        self._save_commits(commits)
        
        # Save branch
        branches = self._get_branches()
        branches[branch_name] = Branch(
            name=branch_name,
            head_commit=commit_hash,
            created_at=datetime.now(),
            created_by=author
        )
        self._save_branches(branches)
        
        # Update refs
        (self.refs_dir / branch_name).write_text(commit_hash)
    
    def init(self) -> Dict:
        """Initialize a new repository"""
        return {
            'status': 'success',
            'message': 'Educational VCS repository initialized',
            'main_branch': 'main'
        }
    
    def add(self, file_path: str, content: str, user: str) -> Dict:
        """Stage a file for commit"""
        # Calculate content hash
        content_hash = self._store_object(content, 'blobs')
        
        # Get current branch and HEAD
        current_branch = self._get_current_branch()
        branches = self._get_branches()
        head_commit_hash = branches[current_branch].head_commit
        
        # Get current commit
        commits = self._get_commits()
        current_commit = commits[head_commit_hash]
        
        # Create new commit with staged changes
        new_commit = Commit(
            hash="",
            parent_hashes=[head_commit_hash],
            author=user,
            message=f"Add {file_path}",
            timestamp=datetime.now(),
            changes={file_path: content_hash},
            branch=current_branch
        )
        
        # Create commit object
        commit_hash = self._store_object(json.dumps(new_commit.to_dict()), 'commits')
        new_commit.hash = commit_hash
        
        # Update commits and branches
        commits[commit_hash] = new_commit
        self._save_commits(commits)
        
        branches[current_branch].head_commit = commit_hash
        self._save_branches(branches)
        
        return {
            'status': 'success',
            'message': f'Staged {file_path}',
            'commit_hash': commit_hash
        }
    
    def commit(self, message: str, user: str) -> Dict:
        """Create a commit with staged changes"""
        current_branch = self._get_current_branch()
        branches = self._get_branches()
        head_commit_hash = branches[current_branch].head_commit
        
        commits = self._get_commits()
        head_commit = commits[head_commit_hash]
        
        # Create new commit
        new_commit = Commit(
            hash="",
            parent_hashes=[head_commit_hash],
            author=user,
            message=message,
            timestamp=datetime.now(),
            changes=head_commit.changes.copy(),
            branch=current_branch
        )
        
        # Create commit object
        commit_hash = self._store_object(json.dumps(new_commit.to_dict()), 'commits')
        new_commit.hash = commit_hash
        
        # Update commits and branches
        commits[commit_hash] = new_commit
        self._save_commits(commits)
        
        branches[current_branch].head_commit = commit_hash
        self._save_branches(branches)
        
        return {
            'status': 'success',
            'message': 'Commit created',
            'commit_hash': commit_hash
        }
    
    def branch(self, branch_name: str, user: str) -> Dict:
        """Create a new branch"""
        current_branch = self._get_current_branch()
        branches = self._get_branches()
        
        if branch_name in branches:
            return {
                'status': 'error',
                'message': f'Branch {branch_name} already exists'
            }
        
        # Get current HEAD
        current_head = branches[current_branch].head_commit
        
        # Create new branch
        branches[branch_name] = Branch(
            name=branch_name,
            head_commit=current_head,
            created_at=datetime.now(),
            created_by=user
        )
        self._save_branches(branches)
        
        return {
            'status': 'success',
            'message': f'Branch {branch_name} created'
        }
    
    def checkout(self, branch_name: str) -> Dict:
        """Switch to a different branch"""
        branches = self._get_branches()
        
        if branch_name not in branches:
            return {
                'status': 'error',
                'message': f'Branch {branch_name} does not exist'
            }
        
        # Update current branch reference
        (self.refs_dir / 'HEAD').write_text(f"refs/{branch_name}")
        
        return {
            'status': 'success',
            'message': f'Switched to branch {branch_name}'
        }
    
    def merge(self, source_branch: str, user: str, message: str = "") -> Dict:
        """Merge a branch into the current branch"""
        current_branch = self._get_current_branch()
        branches = self._get_branches()
        commits = self._get_commits()
        
        if source_branch not in branches:
            return {
                'status': 'error',
                'message': f'Branch {source_branch} does not exist'
            }
        
        # Check for conflicts
        conflicts = self._detect_conflicts(current_branch, source_branch)
        if conflicts:
            return {
                'status': 'conflict',
                'message': 'Merge conflicts detected',
                'conflicts': conflicts
            }
        
        # Create merge commit
        source_head = branches[source_branch].head_commit
        current_head = branches[current_branch].head_commit
        
        # Get source commit changes
        source_commit = commits[source_head]
        
        # Create merge commit
        merge_commit = Commit(
            hash="",
            parent_hashes=[current_head, source_head],
            author=user,
            message=message or f"Merge {source_branch} into {current_branch}",
            timestamp=datetime.now(),
            changes=source_commit.changes,
            branch=current_branch
        )
        
        # Create commit object
        commit_hash = self._store_object(json.dumps(merge_commit.to_dict()), 'commits')
        merge_commit.hash = commit_hash
        
        # Update commits and branches
        commits[commit_hash] = merge_commit
        self._save_commits(commits)
        
        branches[current_branch].head_commit = commit_hash
        self._save_branches(branches)
        
        return {
            'status': 'success',
            'message': f'Merged {source_branch} into {current_branch}'
        }
    
    def _detect_conflicts(self, branch1: str, branch2: str) -> List[str]:
        """Detect conflicts between two branches"""
        conflicts = []
        branches = self._get_branches()
        commits = self._get_commits()
        
        # Get commits for both branches
        commit1 = commits[branches[branch1].head_commit]
        commit2 = commits[branches[branch2].head_commit]
        
        # Check for conflicting changes to the same files
        for file_path in commit1.changes:
            if file_path in commit2.changes:
                conflicts.append(file_path)
        
        return conflicts
    
    def _get_current_branch(self) -> str:
        """Get the current branch name"""
        head_ref = self.refs_dir / 'HEAD'
        if head_ref.exists():
            content = head_ref.read_text().strip()
            if content.startswith('refs/'):
                return content[5:]
        return 'main'
    
    def get_log(self, branch_name: Optional[str] = None) -> List[Dict]:
        """Get commit history"""
        if not branch_name:
            branch_name = self._get_current_branch()
        
        branches = self._get_branches()
        commits = self._get_commits()
        
        if branch_name not in branches:
            return []
        
        log = []
        current_hash = branches[branch_name].head_commit
        
        while current_hash:
            if current_hash not in commits:
                break
            
            commit = commits[current_hash]
            log.append({
                'hash': commit.hash[:8],
                'author': commit.author,
                'message': commit.message,
                'timestamp': commit.timestamp.isoformat(),
                'parent_hashes': [h[:8] for h in commit.parent_hashes]
            })
            
            # Follow first parent (main line of history)
            if commit.parent_hashes:
                current_hash = commit.parent_hashes[0]
            else:
                break
        
        return log
    
    def get_status(self) -> Dict:
        """Get repository status"""
        current_branch = self._get_current_branch()
        branches = self._get_branches()
        
        return {
            'current_branch': current_branch,
            'branches': list(branches.keys()),
            'total_commits': len(self._get_commits()),
            'repository_path': str(self.repo_path)
        }
