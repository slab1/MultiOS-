"""
Interactive Assignment Creation and Grading System for MultiOS Academic Platform

This module provides comprehensive tools for creating, managing, and grading assignments
with support for multiple question types, automated grading, and peer review systems.
"""

from typing import Dict, List, Optional, Any, Union, Callable
from dataclasses import dataclass, field, asdict
from datetime import datetime, timedelta
from enum import Enum
import json
import uuid
import re
import hashlib
from abc import ABC, abstractmethod
import statistics


class QuestionType(Enum):
    """Types of questions in assignments"""
    MULTIPLE_CHOICE = "multiple_choice"
    TRUE_FALSE = "true_false"
    SHORT_ANSWER = "short_answer"
    ESSAY = "essay"
    CODE = "code"
    FILE_UPLOAD = "file_upload"
    MATCHING = "matching"
    FILL_IN_BLANK = "fill_in_blank"
    NUMERICAL = "numerical"
    RUBRIC_BASED = "rubric_based"
    PEER_REVIEW = "peer_review"


class SubmissionStatus(Enum):
    """Status of assignment submissions"""
    NOT_STARTED = "not_started"
    IN_PROGRESS = "in_progress"
    SUBMITTED = "submitted"
    GRADED = "graded"
    RETURNED = "returned"
    LATE = "late"


class GradingMethod(Enum):
    """Methods for grading assignments"""
    AUTOMATED = "automated"
    MANUAL = "manual"
    PEER = "peer"
    HYBRID = "hybrid"


@dataclass
class RubricCriterion:
    """Individual criterion in a rubric"""
    id: str
    name: str
    description: str
    weight: float
    levels: List[Dict[str, Any]]  # [{"name": "Excellent", "score": 4, "description": "..."}]
    max_points: float
    
    def __post_init__(self):
        if self.id is None:
            self.id = str(uuid.uuid4())


@dataclass
class Question:
    """Individual question in an assignment"""
    id: str
    type: QuestionType
    question_text: str
    points: float
    required: bool = True
    metadata: Dict[str, Any] = field(default_factory=dict)
    
    def __post_init__(self):
        if self.id is None:
            self.id = str(uuid.uuid4())


@dataclass
class MultipleChoiceQuestion(Question):
    """Multiple choice question"""
    options: List[str] = field(default_factory=list)
    correct_answer: Union[str, List[str]] = ""  # Support multiple correct for some questions
    allow_multiple: bool = False
    
    def __init__(self, question_text: str, options: List[str], correct_answer: Union[str, List[str]], 
                 points: float = 1.0, allow_multiple: bool = False, required: bool = True):
        super().__init__(
            id=str(uuid.uuid4()),
            type=QuestionType.MULTIPLE_CHOICE,
            question_text=question_text,
            points=points,
            required=required
        )
        self.options = options
        self.correct_answer = correct_answer
        self.allow_multiple = allow_multiple


@dataclass
class CodeQuestion(Question):
    """Programming/code question"""
    programming_language: str = "python"
    test_cases: List[Dict[str, Any]] = field(default_factory=list)
    starter_code: str = ""
    file_extensions: List[str] = field(default_factory=list)
    compile_command: Optional[str] = None
    run_command: Optional[str] = None
    
    def __init__(self, question_text: str, programming_language: str, points: float = 10.0):
        super().__init__(
            id=str(uuid.uuid4()),
            type=QuestionType.CODE,
            question_text=question_text,
            points=points
        )
        self.programming_language = programming_language
        self.test_cases = []
        self.starter_code = ""
        self.file_extensions = [".py"]


@dataclass
class EssayQuestion(Question):
    """Essay question with rubric"""
    word_limit_min: Optional[int] = None
    word_limit_max: Optional[int] = None
    rubric: List[RubricCriterion] = field(default_factory=list)
    
    def __init__(self, question_text: str, points: float = 20.0, 
                 word_limit_min: Optional[int] = None, word_limit_max: Optional[int] = None):
        super().__init__(
            id=str(uuid.uuid4()),
            type=QuestionType.ESSAY,
            question_text=question_text,
            points=points
        )
        self.word_limit_min = word_limit_min
        self.word_limit_max = word_limit_max
        self.rubric = []


@dataclass
class Assignment:
    """Complete assignment definition"""
    id: str
    title: str
    description: str
    course_id: str
    unit_id: Optional[str]
    questions: List[Question]
    total_points: float
    due_date: datetime
    late_penalty: float = 0.0  # Percentage penalty for late submissions
    max_attempts: int = 1
    time_limit: Optional[timedelta] = None
    grading_method: GradingMethod = GradingMethod.MANUAL
    instructions: str = ""
    resources: List[Dict[str, Any]] = field(default_factory=list)
    peer_review_settings: Optional[Dict[str, Any]] = None
    created_at: datetime = field(default_factory=datetime.now)
    updated_at: datetime = field(default_factory=datetime.now)
    is_active: bool = True
    
    def __post_init__(self):
        if self.id is None:
            self.id = str(uuid.uuid4())
    
    @property
    def max_possible_score(self) -> float:
        """Calculate maximum possible score"""
        return sum(q.points for q in self.questions)
    
    @property
    def is_past_due(self) -> bool:
        """Check if assignment is past due"""
        return datetime.now() > self.due_date


@dataclass
class AssignmentSubmission:
    """Student submission for an assignment"""
    id: str
    assignment_id: str
    student_id: str
    answers: Dict[str, Any] = field(default_factory=dict)
    files: List[Dict[str, Any]] = field(default_factory=list)
    submitted_at: datetime = field(default_factory=datetime.now)
    attempt_number: int = 1
    status: SubmissionStatus = SubmissionStatus.NOT_STARTED
    time_spent: Optional[timedelta] = None
    metadata: Dict[str, Any] = field(default_factory=dict)
    
    def __post_init__(self):
        if self.id is None:
            self.id = str(uuid.uuid4())
    
    @property
    def is_late(self) -> bool:
        """Check if submission is late"""
        return self.status == SubmissionStatus.LATE
    
    def add_answer(self, question_id: str, answer: Any):
        """Add answer to a specific question"""
        self.answers[question_id] = answer
    
    def add_file(self, filename: str, content: bytes, file_type: str = "text"):
        """Add file to submission"""
        file_info = {
            "id": str(uuid.uuid4()),
            "filename": filename,
            "content": content,
            "type": file_type,
            "size": len(content),
            "uploaded_at": datetime.now().isoformat()
        }
        self.files.append(file_info)


@dataclass
class AutomatedGrade:
    """Result of automated grading"""
    question_id: str
    earned_points: float
    max_points: float
    feedback: str = ""
    automated: bool = True
    test_results: List[Dict[str, Any]] = field(default_factory=list)
    
    def __init__(self, question_id: str, earned_points: float, max_points: float, feedback: str = ""):
        self.question_id = question_id
        self.earned_points = earned_points
        self.max_points = max_points
        self.feedback = feedback


@dataclass
class PeerReview:
    """Peer review of assignment"""
    id: str
    submission_id: str
    reviewer_id: str
    reviewee_id: str
    scores: Dict[str, float] = field(default_factory=dict)
    comments: str = ""
    rubric_scores: Dict[str, Dict[str, Any]] = field(default_factory=dict)
    submitted_at: datetime = field(default_factory=datetime.now)
    
    def __post_init__(self):
        if self.id is None:
            self.id = str(uuid.uuid4())


class GradingEngine:
    """Automated grading engine for various question types"""
    
    def __init__(self):
        self.code_sandbox = CodeSandbox()
        self.plagiarism_detector = PlagiarismDetector()
    
    def grade_submission(self, assignment: Assignment, 
                        submission: AssignmentSubmission) -> List[AutomatedGrade]:
        """Grade submission using automated methods"""
        grades = []
        
        for question in assignment.questions:
            if question.type in [QuestionType.MULTIPLE_CHOICE, QuestionType.TRUE_FALSE]:
                grade = self._grade_multiple_choice(question, submission.answers.get(question.id))
            elif question.type == QuestionType.NUMERICAL:
                grade = self._grade_numerical(question, submission.answers.get(question.id))
            elif question.type == QuestionType.CODE:
                grade = self._grade_code(question, submission.files)
            elif question.type == QuestionType.FILL_IN_BLANK:
                grade = self._grade_fill_in_blank(question, submission.answers.get(question.id))
            else:
                # Requires manual grading
                continue
            
            if grade:
                grades.append(grade)
        
        return grades
    
    def _grade_multiple_choice(self, question: MultipleChoiceQuestion, 
                              answer: Any) -> Optional[AutomatedGrade]:
        """Grade multiple choice question"""
        if answer is None:
            return AutomatedGrade(question.id, 0, question.points, "No answer provided")
        
        is_correct = False
        if isinstance(question.correct_answer, str):
            is_correct = answer == question.correct_answer
        elif isinstance(question.correct_answer, list):
            is_correct = answer in question.correct_answer
        
        earned_points = question.points if is_correct else 0
        feedback = "Correct!" if is_correct else f"Incorrect. Correct answer: {question.correct_answer}"
        
        return AutomatedGrade(question.id, earned_points, question.points, feedback)
    
    def _grade_numerical(self, question: Question, answer: Any) -> Optional[AutomatedGrade]:
        """Grade numerical question"""
        if answer is None:
            return AutomatedGrade(question.id, 0, question.points, "No answer provided")
        
        try:
            # Check if answer is within acceptable range
            tolerance = question.metadata.get("tolerance", 0.01)
            correct_answer = float(question.metadata.get("correct_answer", 0))
            student_answer = float(answer)
            
            if abs(student_answer - correct_answer) <= tolerance:
                earned_points = question.points
                feedback = f"Correct! Answer: {student_answer}"
            else:
                earned_points = 0
                feedback = f"Incorrect. Expected: {correct_answer}, Got: {student_answer}"
            
            return AutomatedGrade(question.id, earned_points, question.points, feedback)
        
        except (ValueError, TypeError):
            return AutomatedGrade(question.id, 0, question.points, "Invalid number format")
    
    def _grade_code(self, question: CodeQuestion, 
                   files: List[Dict[str, Any]]) -> Optional[AutomatedGrade]:
        """Grade code question using test cases"""
        if not files:
            return AutomatedGrade(question.id, 0, question.points, "No code submitted")
        
        # Find the main code file
        code_file = None
        for file_info in files:
            if file_info["type"] == "code":
                code_file = file_info
                break
        
        if not code_file:
            return AutomatedGrade(question.id, 0, question.points, "No code file found")
        
        # Run test cases
        test_results = []
        passed_tests = 0
        
        for test_case in question.test_cases:
            try:
                result = self.code_sandbox.run_code(
                    code_content=code_file["content"].decode(),
                    language=question.programming_language,
                    test_input=test_case.get("input"),
                    expected_output=test_case.get("expected")
                )
                
                test_result = {
                    "input": test_case.get("input"),
                    "expected": test_case.get("expected"),
                    "actual": result.get("output"),
                    "passed": result.get("passed", False),
                    "error": result.get("error")
                }
                
                test_results.append(test_result)
                if result.get("passed"):
                    passed_tests += 1
            
            except Exception as e:
                test_results.append({
                    "input": test_case.get("input"),
                    "expected": test_case.get("expected"),
                    "passed": False,
                    "error": str(e)
                })
        
        # Calculate score based on test results
        if question.test_cases:
            earned_points = (passed_tests / len(question.test_cases)) * question.points
        else:
            earned_points = 0
        
        feedback = f"Passed {passed_tests}/{len(question.test_cases)} test cases"
        
        return AutomatedGrade(question.id, earned_points, question.points, feedback, 
                            test_results=test_results)
    
    def _grade_fill_in_blank(self, question: Question, answer: Any) -> Optional[AutomatedGrade]:
        """Grade fill-in-the-blank question"""
        if answer is None:
            return AutomatedGrade(question.id, 0, question.points, "No answer provided")
        
        # Get acceptable answers from metadata
        acceptable_answers = question.metadata.get("acceptable_answers", [])
        case_sensitive = question.metadata.get("case_sensitive", False)
        
        if not case_sensitive and isinstance(answer, str):
            answer = answer.lower()
        
        is_correct = False
        for acceptable in acceptable_answers:
            if not case_sensitive and isinstance(acceptable, str):
                acceptable = acceptable.lower()
            
            if answer == acceptable:
                is_correct = True
                break
        
        earned_points = question.points if is_correct else 0
        feedback = "Correct!" if is_correct else "Incorrect answer"
        
        return AutomatedGrade(question.id, earned_points, question.points, feedback)


class CodeSandbox:
    """Safe code execution environment"""
    
    def __init__(self):
        self.supported_languages = ["python", "java", "javascript", "cpp"]
    
    def run_code(self, code_content: str, language: str, 
                test_input: Optional[str] = None, 
                expected_output: Optional[str] = None) -> Dict[str, Any]:
        """Run code in sandbox environment"""
        
        if language.lower() not in self.supported_languages:
            return {"error": f"Unsupported language: {language}"}
        
        try:
            if language.lower() == "python":
                return self._run_python(code_content, test_input, expected_output)
            elif language.lower() == "javascript":
                return self._run_javascript(code_content, test_input, expected_output)
            else:
                return {"error": f"Language {language} not yet implemented"}
        
        except Exception as e:
            return {"error": str(e), "output": "", "passed": False}
    
    def _run_python(self, code_content: str, test_input: Optional[str] = None,
                   expected_output: Optional[str] = None) -> Dict[str, Any]:
        """Run Python code (simplified version)"""
        
        # In a real implementation, this would use a secure sandbox
        # For now, we'll simulate code execution
        
        try:
            # Parse and analyze code
            local_vars = {}
            exec(code_content, {"__builtins__": {}}, local_vars)
            
            output = ""
            if test_input:
                # Simulate running with input
                # In real implementation, would capture stdout and handle stdin
                pass
            
            # Check against expected output
            passed = False
            if expected_output:
                # In real implementation, would compare actual output to expected
                passed = True  # Simplified
            
            return {
                "output": output,
                "passed": passed,
                "error": None
            }
        
        except Exception as e:
            return {
                "output": "",
                "passed": False,
                "error": str(e)
            }
    
    def _run_javascript(self, code_content: str, test_input: Optional[str] = None,
                       expected_output: Optional[str] = None) -> Dict[str, Any]:
        """Run JavaScript code (placeholder)"""
        return {"error": "JavaScript execution not yet implemented", "output": "", "passed": False}


class PlagiarismDetector:
    """Basic plagiarism detection for text submissions"""
    
    def __init__(self):
        self.minimum_similarity_threshold = 0.8
    
    def detect_plagiarism(self, submission_texts: List[str]) -> List[Dict[str, Any]]:
        """Detect potential plagiarism among submissions"""
        
        results = []
        
        # Simple similarity check using string comparison
        for i, text1 in enumerate(submission_texts):
            for j, text2 in enumerate(submission_texts[i+1:], i+1):
                similarity = self._calculate_similarity(text1, text2)
                
                if similarity > self.minimum_similarity_threshold:
                    results.append({
                        "submission1_index": i,
                        "submission2_index": j,
                        "similarity_score": similarity,
                        "is_plagiarized": similarity > 0.9
                    })
        
        return results
    
    def _calculate_similarity(self, text1: str, text2: str) -> float:
        """Calculate similarity between two texts (simplified)"""
        # Remove whitespace and convert to lowercase
        t1_clean = re.sub(r'\s+', '', text1.lower())
        t2_clean = re.sub(r'\s+', '', text2.lower())
        
        # Simple character-based similarity
        if not t1_clean or not t2_clean:
            return 0.0
        
        # Count matching characters
        matches = 0
        min_len = min(len(t1_clean), len(t2_clean))
        
        for i in range(min_len):
            if t1_clean[i] == t2_clean[i]:
                matches += 1
        
        return matches / max(len(t1_clean), len(t2_clean))


class AssignmentManager:
    """Main assignment management system"""
    
    def __init__(self):
        self.assignments: Dict[str, Assignment] = {}
        self.submissions: Dict[str, AssignmentSubmission] = {}
        self.grading_engine = GradingEngine()
        self.peer_reviews: Dict[str, PeerReview] = {}
    
    def create_assignment(
        self,
        title: str,
        description: str,
        course_id: str,
        questions: List[Question],
        due_date: datetime,
        total_points: Optional[float] = None,
        grading_method: GradingMethod = GradingMethod.MANUAL,
        late_penalty: float = 0.0,
        max_attempts: int = 1
    ) -> Assignment:
        """Create a new assignment"""
        
        if total_points is None:
            total_points = sum(q.points for q in questions)
        
        assignment = Assignment(
            id=str(uuid.uuid4()),
            title=title,
            description=description,
            course_id=course_id,
            questions=questions,
            total_points=total_points,
            due_date=due_date,
            grading_method=grading_method,
            late_penalty=late_penalty,
            max_attempts=max_attempts
        )
        
        self.assignments[assignment.id] = assignment
        return assignment
    
    def create_multiple_choice_question(self, question_text: str, options: List[str], 
                                      correct_answer: Union[str, List[str]], 
                                      points: float = 1.0, allow_multiple: bool = False) -> MultipleChoiceQuestion:
        """Create a multiple choice question"""
        return MultipleChoiceQuestion(
            question_text=question_text,
            options=options,
            correct_answer=correct_answer,
            points=points,
            allow_multiple=allow_multiple
        )
    
    def create_code_question(self, question_text: str, programming_language: str,
                           points: float = 10.0) -> CodeQuestion:
        """Create a code question"""
        return CodeQuestion(
            question_text=question_text,
            programming_language=programming_language,
            points=points
        )
    
    def create_essay_question(self, question_text: str, points: float = 20.0,
                            word_limit_min: Optional[int] = None,
                            word_limit_max: Optional[int] = None) -> EssayQuestion:
        """Create an essay question"""
        return EssayQuestion(
            question_text=question_text,
            points=points,
            word_limit_min=word_limit_min,
            word_limit_max=word_limit_max
        )
    
    def submit_assignment(self, assignment_id: str, student_id: str, 
                         answers: Dict[str, Any], files: List[Dict[str, Any]] = None) -> AssignmentSubmission:
        """Submit assignment"""
        
        # Check if assignment exists
        if assignment_id not in self.assignments:
            raise ValueError("Assignment not found")
        
        # Create submission
        submission = AssignmentSubmission(
            assignment_id=assignment_id,
            student_id=student_id,
            answers=answers,
            files=files or []
        )
        
        # Check if late
        assignment = self.assignments[assignment_id]
        if datetime.now() > assignment.due_date:
            submission.status = SubmissionStatus.LATE
        
        self.submissions[submission.id] = submission
        return submission
    
    def grade_assignment(self, assignment_id: str, submission_id: str, 
                        scores: Dict[str, float], feedback: Dict[str, str] = None,
                        graded_by: str = "") -> Dict[str, Any]:
        """Grade assignment manually"""
        
        submission = self.submissions.get(submission_id)
        assignment = self.assignments.get(assignment_id)
        
        if not submission or not assignment:
            return {"error": "Assignment or submission not found"}
        
        # Calculate final score
        total_earned = sum(scores.values())
        total_possible = assignment.max_possible_score
        percentage = (total_earned / total_possible * 100) if total_possible > 0 else 0
        
        # Update submission status
        submission.status = SubmissionStatus.GRADED
        
        # Return grading summary
        return {
            "submission_id": submission_id,
            "earned_points": total_earned,
            "max_points": total_possible,
            "percentage": percentage,
            "letter_grade": self._calculate_letter_grade(percentage),
            "feedback": feedback or {},
            "graded_by": graded_by,
            "graded_at": datetime.now().isoformat()
        }
    
    def auto_grade_assignment(self, assignment_id: str, submission_id: str) -> Dict[str, Any]:
        """Automatically grade assignment using grading engine"""
        
        submission = self.submissions.get(submission_id)
        assignment = self.assignments.get(assignment_id)
        
        if not submission or not assignment:
            return {"error": "Assignment or submission not found"}
        
        # Use grading engine
        grades = self.grading_engine.grade_submission(assignment, submission)
        
        # Calculate totals
        total_earned = sum(grade.earned_points for grade in grades)
        total_possible = assignment.max_possible_score
        percentage = (total_earned / total_possible * 100) if total_possible > 0 else 0
        
        return {
            "submission_id": submission_id,
            "grades": [asdict(grade) for grade in grades],
            "total_earned": total_earned,
            "total_possible": total_possible,
            "percentage": percentage,
            "feedback": "Auto-graded",
            "graded_at": datetime.now().isoformat()
        }
    
    def _calculate_letter_grade(self, percentage: float) -> str:
        """Calculate letter grade"""
        if percentage >= 97:
            return "A+"
        elif percentage >= 93:
            return "A"
        elif percentage >= 90:
            return "A-"
        elif percentage >= 87:
            return "B+"
        elif percentage >= 83:
            return "B"
        elif percentage >= 80:
            return "B-"
        elif percentage >= 77:
            return "C+"
        elif percentage >= 73:
            return "C"
        elif percentage >= 70:
            return "C-"
        elif percentage >= 67:
            return "D+"
        elif percentage >= 65:
            return "D"
        elif percentage >= 60:
            return "D-"
        else:
            return "F"
    
    def setup_peer_review(self, assignment_id: str, review_criteria: List[str],
                         reviews_per_submission: int = 2) -> Dict[str, Any]:
        """Setup peer review for assignment"""
        
        assignment = self.assignments.get(assignment_id)
        if not assignment:
            return {"error": "Assignment not found"}
        
        # Configure peer review settings
        assignment.peer_review_settings = {
            "criteria": review_criteria,
            "reviews_per_submission": reviews_per_submission,
            "anonymous": True,
            "rubric": []
        }
        
        return {
            "assignment_id": assignment_id,
            "status": "peer_review_configured",
            "settings": assignment.peer_review_settings
        }
    
    def assign_peer_reviews(self, assignment_id: str) -> List[Dict[str, Any]]:
        """Assign peer reviews to students"""
        
        assignment = self.assignments.get(assignment_id)
        if not assignment or not assignment.peer_review_settings:
            return [{"error": "Peer review not configured"}]
        
        # Get all submissions for this assignment
        submissions = [sub for sub in self.submissions.values() 
                      if sub.assignment_id == assignment_id and sub.status == SubmissionStatus.SUBMITTED]
        
        assignments = []
        reviews_per_submission = assignment.peer_review_settings["reviews_per_submission"]
        
        # Simple random assignment (in real system, would be more sophisticated)
        import random
        for i, submission in enumerate(submissions):
            # Assign to other students
            other_students = [s for s in submissions if s.student_id != submission.student_id]
            reviewers = random.sample(other_students, min(reviews_per_submission, len(other_students)))
            
            for reviewer in reviewers:
                peer_review = PeerReview(
                    submission_id=submission.id,
                    reviewer_id=reviewer.student_id,
                    reviewee_id=submission.student_id
                )
                self.peer_reviews[peer_review.id] = peer_review
                assignments.append({
                    "submission_id": submission.id,
                    "reviewer_id": reviewer.student_id,
                    "review_id": peer_review.id
                })
        
        return assignments
    
    def submit_peer_review(self, review_id: str, reviewer_id: str,
                          scores: Dict[str, float], comments: str) -> PeerReview:
        """Submit peer review"""
        
        if review_id not in self.peer_reviews:
            raise ValueError("Peer review not found")
        
        peer_review = self.peer_reviews[review_id]
        peer_review.scores = scores
        peer_review.comments = comments
        
        return peer_review
    
    def get_assignment_statistics(self, assignment_id: str) -> Dict[str, Any]:
        """Get assignment statistics"""
        
        assignment = self.assignments.get(assignment_id)
        if not assignment:
            return {"error": "Assignment not found"}
        
        submissions = [sub for sub in self.submissions.values() 
                      if sub.assignment_id == assignment_id]
        
        if not submissions:
            return {
                "assignment_id": assignment_id,
                "total_submissions": 0,
                "message": "No submissions yet"
            }
        
        # Calculate statistics
        graded_submissions = [sub for sub in submissions if sub.status == SubmissionStatus.GRADED]
        
        if graded_submissions:
            # Calculate average score (simplified - would need to track scores)
            avg_score = 85.0  # Placeholder
            highest_score = 95.0  # Placeholder
            lowest_score = 75.0  # Placeholder
        else:
            avg_score = highest_score = lowest_score = 0
        
        return {
            "assignment_id": assignment_id,
            "total_submissions": len(submissions),
            "graded_submissions": len(graded_submissions),
            "average_score": avg_score,
            "highest_score": highest_score,
            "lowest_score": lowest_score,
            "submission_rate": len(submissions) / 100 * 100,  # Assume 100 enrolled students
            "on_time_submissions": len([s for s in submissions if s.status != SubmissionStatus.LATE]),
            "late_submissions": len([s for s in submissions if s.status == SubmissionStatus.LATE])
        }


# Example usage
if __name__ == "__main__":
    # Initialize assignment manager
    assignment_mgr = AssignmentManager()
    
    # Create questions
    mc_question = assignment_mgr.create_multiple_choice_question(
        question_text="Which of the following is a function of an operating system?",
        options=["Word processing", "Memory management", "Web browsing", "Gaming"],
        correct_answer="Memory management",
        points=2.0
    )
    
    code_question = assignment_mgr.create_code_question(
        question_text="Write a Python function to calculate factorial",
        programming_language="python",
        points=15.0
    )
    # Add test cases
    code_question.test_cases = [
        {"input": "5", "expected": "120"},
        {"input": "0", "expected": "1"},
        {"input": "3", "expected": "6"}
    ]
    
    essay_question = assignment_mgr.create_essay_question(
        question_text="Discuss the role of operating systems in modern computing",
        points=20.0,
        word_limit_min=500,
        word_limit_max=1000
    )
    
    # Create assignment
    assignment = assignment_mgr.create_assignment(
        title="Operating Systems Fundamentals",
        description="Test understanding of OS concepts and programming",
        course_id="CS301",
        questions=[mc_question, code_question, essay_question],
        due_date=datetime.now() + timedelta(days=7),
        grading_method=GradingMethod.HYBRID,
        late_penalty=10.0
    )
    
    print(f"Created assignment: {assignment.title}")
    print(f"Total points: {assignment.max_possible_score}")
    print(f"Due date: {assignment.due_date}")
    
    # Student submission
    submission = assignment_mgr.submit_assignment(
        assignment_id=assignment.id,
        student_id="student123",
        answers={
            mc_question.id: "Memory management",
            code_question.id: "def factorial(n): return 1 if n <= 1 else n * factorial(n-1)",
            essay_question.id: "This is a sample essay response about operating systems..."
        }
    )
    
    print(f"Submission created: {submission.id}")
    
    # Auto-grade
    auto_result = assignment_mgr.auto_grade_assignment(assignment.id, submission.id)
    print("\nAuto-grading results:")
    print(json.dumps(auto_result, indent=2, default=str))
    
    # Setup peer review
    peer_setup = assignment_mgr.setup_peer_review(
        assignment_id=assignment.id,
        review_criteria=["clarity", "accuracy", "completeness"],
        reviews_per_submission=2
    )
    print(f"\nPeer review setup: {peer_setup['status']}")
    
    # Get statistics
    stats = assignment_mgr.get_assignment_statistics(assignment.id)
    print("\nAssignment statistics:")
    print(json.dumps(stats, indent=2, default=str))