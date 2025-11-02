"""
Conflict Resolution System for Educational VCS
Handles merge conflicts with educational hints and collaborative resolution
"""

import difflib
from typing import Dict, List, Tuple, Optional
from datetime import datetime
from pathlib import Path
import json


class ConflictResolution:
    """Handles conflict resolution for educational collaboration"""
    
    def __init__(self, vcs_repo):
        self.repo = vcs_repo
        self.conflicts_dir = vcs_repo.conflicts_dir
    
    def detect_merge_conflicts(self, base_content: str, current_content: str, 
                             incoming_content: str, file_path: str) -> Dict:
        """
        Detect merge conflicts between three-way merge
        Returns conflict information with suggested resolutions
        """
        conflicts = []
        
        # Split content into lines
        base_lines = base_content.splitlines(keepends=True)
        current_lines = current_content.splitlines(keepends=True)
        incoming_lines = incoming_content.splitlines(keepends=True)
        
        # Get diffs
        current_diff = list(difflib.unified_diff(
            base_lines, current_lines, lineterm='', n=0
        ))
        incoming_diff = list(difflib.unified_diff(
            base_lines, incoming_lines, lineterm='', n=0
        ))
        
        # Analyze conflicts
        conflicts = self._analyze_conflicts(
            base_lines, current_lines, incoming_lines, file_path
        )
        
        return {
            'has_conflicts': len(conflicts) > 0,
            'conflicts': conflicts,
            'conflict_count': len(conflicts),
            'suggested_resolution': self._suggest_resolution(conflicts)
        }
    
    def _analyze_conflicts(self, base: List[str], current: List[str], 
                          incoming: List[str], file_path: str) -> List[Dict]:
        """Analyze conflicts between current and incoming changes"""
        conflicts = []
        
        # Use difflib to find differences
        matcher = difflib.SequenceMatcher(None, base, current)
        
        for tag, i1, i2, j1, j2 in matcher.get_opcodes():
            if tag == 'replace':
                # Check if incoming has different changes to same area
                incoming_matcher = difflib.SequenceMatcher(None, base, incoming)
                
                for itag, ii1, ii2, ij1, ij2 in incoming_matcher.get_opcodes():
                    if itag == 'replace' and self._ranges_overlap(i1, i2, ii1, ii2):
                        # Found conflict
                        conflict = {
                            'type': 'content_conflict',
                            'file_path': file_path,
                            'base_lines': base[i1:i2],
                            'current_lines': current[i1:i2],
                            'incoming_lines': incoming[ij1:ij2],
                            'line_range': {'start': i1, 'end': i2},
                            'education_level': 'intermediate',
                            'hint': self._generate_conflict_hint(file_path, i1, i2)
                        }
                        conflicts.append(conflict)
        
        return conflicts
    
    def _ranges_overlap(self, start1: int, end1: int, start2: int, end2: int) -> bool:
        """Check if two ranges overlap"""
        return not (end1 <= start2 or end2 <= start1)
    
    def _generate_conflict_hint(self, file_path: str, start_line: int, end_line: int) -> str:
        """Generate educational hint for conflict resolution"""
        hints = [
            "This conflict occurred because both collaborators made changes to the same lines.",
            "Consider combining the useful parts from both versions.",
            "Look at the context around the conflict to understand what each change was trying to accomplish.",
            "Ask your partner what they were trying to fix or improve.",
            "Sometimes both changes are needed - don't just pick one!"
        ]
        return f"Line {start_line}-{end_line}: {hints[start_line % len(hints)]}"
    
    def _suggest_resolution(self, conflicts: List[Dict]) -> str:
        """Generate a suggested resolution approach"""
        if not conflicts:
            return "No conflicts detected - safe to merge"
        
        return (
            "Suggested resolution steps:\n"
            "1. Review each conflict carefully\n"
            "2. Understand what both versions were trying to accomplish\n"
            "3. Combine the best parts of both changes\n"
            "4. Test the merged code before committing\n"
            "5. Communicate with your collaborator about the resolution"
        )
    
    def create_merge_conflict_file(self, base: str, current: str, 
                                 incoming: str, file_path: str) -> str:
        """Create a conflict file with conflict markers for manual resolution"""
        conflict_content = self._format_conflict_markers(base, current, incoming)
        
        # Save conflict file
        conflict_file = self.conflicts_dir / f"{file_path.replace('/', '_')}_conflict.txt"
        conflict_file.write_text(conflict_content)
        
        return str(conflict_file)
    
    def _format_conflict_markers(self, base: str, current: str, incoming: str) -> str:
        """Format conflict with standard Git conflict markers"""
        return (
            f"<<<<<<< YOUR CHANGES\n"
            f"{current}\n"
            f"=======\n"
            f"{incoming}\n"
            f">>>>>>> INCOMING CHANGES\n"
        )
    
    def resolve_conflict_interactive(self, conflict: Dict, user_choice: str) -> str:
        """
        Resolve conflict based on user choice
        user_choice: 'current', 'incoming', 'both', 'custom'
        """
        if user_choice == 'current':
            return ''.join(conflict['current_lines'])
        elif user_choice == 'incoming':
            return ''.join(conflict['incoming_lines'])
        elif user_choice == 'both':
            return ''.join(conflict['current_lines'] + conflict['incoming_lines'])
        else:
            # Custom resolution would need additional input
            return ''.join(conflict['current_lines'])
    
    def get_collaboration_advice(self, conflict_type: str) -> Dict:
        """Get educational advice for different types of conflicts"""
        advice = {
            'content_conflict': {
                'description': 'Both collaborators edited the same lines differently',
                'communication_tips': [
                    'Use the communication feature to discuss the changes',
                    'Share screen or use pair programming mode',
                    'Use comments in the code to explain your reasoning'
                ],
                'best_practices': [
                    'Pull before pushing to avoid conflicts',
                    'Communicate with your team about major changes',
                    'Use feature branches for larger changes'
                ]
            },
            'concurrent_edits': {
                'description': 'Multiple people edited the same file simultaneously',
                'communication_tips': [
                    'Coordinate work schedules',
                    'Use assignment planning tools',
                    'Break large changes into smaller commits'
                ],
                'best_practices': [
                    'Work on different files when possible',
                    'Use branches for experimental features',
                    'Regular sync meetings with the team'
                ]
            }
        }
        
        return advice.get(conflict_type, {
            'description': 'General conflict resolution',
            'communication_tips': ['Communicate with your teammates'],
            'best_practices': ['Pull frequently and commit often']
        })


class CollaborativeEditor:
    """Real-time collaborative editing support"""
    
    def __init__(self, vcs_repo):
        self.repo = vcs_repo
        self.active_sessions = {}
        self.operational_transforms = {}
    
    def start_collaborative_session(self, file_path: str, user: str, 
                                  session_id: str) -> Dict:
        """Start a collaborative editing session"""
        # Load current file content
        try:
            content = self.repo._get_object(
                self.repo._get_current_content_hash(file_path),
                'blobs'
            )
        except:
            content = ""
        
        session = {
            'file_path': file_path,
            'users': [user],
            'content': content,
            'version': 0,
            'changes': [],
            'created_at': datetime.now()
        }
        
        self.active_sessions[session_id] = session
        
        return {
            'status': 'success',
            'session_id': session_id,
            'content': content,
            'version': 0
        }
    
    def apply_changes(self, session_id: str, changes: List[Dict], user: str) -> Dict:
        """Apply operational transforms to collaborative session"""
        if session_id not in self.active_sessions:
            return {'status': 'error', 'message': 'Session not found'}
        
        session = self.active_sessions[session_id]
        
        # Apply operational transforms
        transformed_content = self._apply_operational_transforms(
            session['content'], changes
        )
        
        # Update session
        session['content'] = transformed_content
        session['version'] += 1
        session['changes'].extend(changes)
        
        return {
            'status': 'success',
            'content': transformed_content,
            'version': session['version'],
            'conflicts': self._detect_real_time_conflicts(changes)
        }
    
    def _apply_operational_transforms(self, content: str, changes: List[Dict]) -> str:
        """Apply operational transforms to merge concurrent changes"""
        # Simplified operational transform
        lines = content.splitlines()
        
        for change in changes:
            op_type = change.get('type')
            pos = change.get('position', 0)
            text = change.get('text', '')
            
            if op_type == 'insert':
                lines.insert(pos, text)
            elif op_type == 'delete':
                if pos < len(lines):
                    lines.pop(pos)
            elif op_type == 'replace':
                if pos < len(lines):
                    lines[pos] = text
        
        return '\n'.join(lines)
    
    def _detect_real_time_conflicts(self, changes: List[Dict]) -> List[Dict]:
        """Detect conflicts in real-time collaborative editing"""
        conflicts = []
        
        # Simple conflict detection based on overlapping changes
        for i, change1 in enumerate(changes):
            for j, change2 in enumerate(changes[i+1:], i+1):
                if self._changes_overlap(change1, change2):
                    conflicts.append({
                        'type': 'operational_conflict',
                        'change1': change1,
                        'change2': change2,
                        'suggestion': 'Apply changes sequentially'
                    })
        
        return conflicts
    
    def _changes_overlap(self, change1: Dict, change2: Dict) -> bool:
        """Check if two changes overlap in the document"""
        # Simplified overlap detection
        pos1 = change1.get('position', 0)
        pos2 = change2.get('position', 0)
        
        return abs(pos1 - pos2) < 5  # Consider changes within 5 positions as overlapping
    
    def end_session(self, session_id: str) -> Dict:
        """End a collaborative session and save changes"""
        if session_id not in self.active_sessions:
            return {'status': 'error', 'message': 'Session not found'}
        
        session = self.active_sessions[session_id]
        
        # Save final content to VCS
        content_hash = self.repo._store_object(session['content'], 'blobs')
        
        # Clean up session
        del self.active_sessions[session_id]
        
        return {
            'status': 'success',
            'content_saved': True,
            'content_hash': content_hash
        }
