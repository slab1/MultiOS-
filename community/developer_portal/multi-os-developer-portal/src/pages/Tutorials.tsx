import React, { useState } from 'react';
import { 
  BookOpen, 
  Play, 
  CheckCircle, 
  Circle, 
  Clock, 
  Star, 
  Users, 
  Target,
  ArrowRight,
  ArrowLeft,
  Code,
  Search,
  Filter,
  Grid3X3,
  List,
  Award,
  TrendingUp,
  Zap
} from 'lucide-react';

interface Tutorial {
  id: string;
  title: string;
  description: string;
  category: string;
  difficulty: 'Beginner' | 'Intermediate' | 'Advanced';
  duration: string;
  rating: number;
  students: number;
  completed: number;
  lessons: number;
  tags: string[];
  language: string;
  prerequisites: string[];
  learningObjectives: string[];
  progress?: number;
}

interface Lesson {
  id: string;
  title: string;
  content: string;
  exercise?: {
    instructions: string;
    starterCode: string;
    solution: string;
    language: string;
  };
  completed?: boolean;
}

export const Tutorials: React.FC = () => {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedCategory, setSelectedCategory] = useState('All');
  const [selectedDifficulty, setSelectedDifficulty] = useState('All');
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [selectedTutorial, setSelectedTutorial] = useState<Tutorial | null>(null);
  const [currentLesson, setCurrentLesson] = useState(0);

  const tutorials: Tutorial[] = [
    {
      id: '1',
      title: 'MultiOS Python Fundamentals',
      description: 'Master the basics of Python programming for MultiOS applications, including syntax, data structures, and best practices.',
      category: 'Programming Basics',
      difficulty: 'Beginner',
      duration: '6 hours',
      rating: 4.8,
      students: 1250,
      completed: 890,
      lessons: 12,
      language: 'Python',
      tags: ['basics', 'syntax', 'data-structures'],
      prerequisites: ['No prior programming experience required'],
      learningObjectives: [
        'Understand Python syntax and basic concepts',
        'Work with data structures and control flow',
        'Write clean, maintainable code',
        'Build simple MultiOS applications'
      ],
      progress: 35
    },
    {
      id: '2',
      title: 'Rust for MultiOS Systems',
      description: 'Learn system programming with Rust, focusing on performance, safety, and MultiOS-specific optimizations.',
      category: 'System Programming',
      difficulty: 'Advanced',
      duration: '12 hours',
      rating: 4.9,
      students: 890,
      completed: 567,
      lessons: 20,
      language: 'Rust',
      tags: ['systems', 'performance', 'memory-safety'],
      prerequisites: ['C/C++ experience recommended', 'Basic understanding of operating systems'],
      learningObjectives: [
        'Master Rust language fundamentals',
        'Understand memory management and safety',
        'Build high-performance system applications',
        'Optimize code for MultiOS platforms'
      ],
      progress: 0
    },
    {
      id: '3',
      title: 'JavaScript Web Development',
      description: 'Build modern web applications using JavaScript, covering ES6+, async programming, and DOM manipulation.',
      category: 'Web Development',
      difficulty: 'Intermediate',
      duration: '8 hours',
      rating: 4.7,
      students: 1567,
      completed: 1120,
      lessons: 16,
      language: 'JavaScript',
      tags: ['web', 'ES6', 'async', 'DOM'],
      prerequisites: ['HTML/CSS basics', 'Programming fundamentals'],
      learningObjectives: [
        'Master modern JavaScript features',
        'Build interactive web applications',
        'Understand asynchronous programming',
        'Create responsive user interfaces'
      ],
      progress: 80
    },
    {
      id: '4',
      title: 'Database Design & Management',
      description: 'Learn to design, implement, and manage databases for MultiOS applications using SQL and NoSQL approaches.',
      category: 'Database',
      difficulty: 'Intermediate',
      duration: '10 hours',
      rating: 4.6,
      students: 980,
      completed: 678,
      lessons: 14,
      language: 'SQL',
      tags: ['database', 'SQL', 'NoSQL', 'schema-design'],
      prerequisites: ['Basic programming knowledge'],
      learningObjectives: [
        'Design efficient database schemas',
        'Write complex SQL queries',
        'Understand NoSQL concepts',
        'Implement data persistence in MultiOS apps'
      ],
      progress: 0
    },
    {
      id: '5',
      title: 'API Development with Python',
      description: 'Build robust REST APIs using Python frameworks like FastAPI and Django, including authentication and testing.',
      category: 'Backend Development',
      difficulty: 'Intermediate',
      duration: '9 hours',
      rating: 4.8,
      students: 1123,
      completed: 789,
      lessons: 15,
      language: 'Python',
      tags: ['API', 'REST', 'FastAPI', 'authentication'],
      prerequisites: ['Python fundamentals', 'HTTP concepts'],
      learningObjectives: [
        'Design and build REST APIs',
        'Implement authentication and authorization',
        'Write comprehensive tests',
        'Deploy APIs to production'
      ],
      progress: 25
    },
    {
      id: '6',
      title: 'MultiOS Mobile Development',
      description: 'Develop cross-platform mobile applications using React Native and Flutter for iOS and Android.',
      category: 'Mobile Development',
      difficulty: 'Intermediate',
      duration: '11 hours',
      rating: 4.7,
      students: 856,
      completed: 543,
      lessons: 18,
      language: 'JavaScript',
      tags: ['mobile', 'cross-platform', 'React-Native', 'Flutter'],
      prerequisites: ['JavaScript basics', 'Mobile app concepts'],
      learningObjectives: [
        'Build cross-platform mobile apps',
        'Implement navigation and state management',
        'Work with device APIs',
        'Deploy to app stores'
      ],
      progress: 0
    }
  ];

  const categories = ['All', 'Programming Basics', 'System Programming', 'Web Development', 'Mobile Development', 'Backend Development', 'Database'];
  const difficulties = ['All', 'Beginner', 'Intermediate', 'Advanced'];

  const sampleLessons: Lesson[] = [
    {
      id: '1',
      title: 'Introduction to Python',
      content: `# Introduction to Python for MultiOS

Python is a powerful, high-level programming language that's perfect for MultiOS development. In this lesson, you'll learn:

## What is Python?
Python is an interpreted, object-oriented, high-level programming language with dynamic semantics. Its high-level built-in data structures, combined with dynamic typing and dynamic binding, make it very attractive for Rapid Application Development.

## Why Python for MultiOS?
- **Easy to learn**: Clean, readable syntax
- **Powerful libraries**: Extensive ecosystem
- **Cross-platform**: Runs on all major operating systems
- **Fast development**: Rapid prototyping capabilities

## Your First Python Program
Let's start with the classic "Hello World" example:`,
      exercise: {
        instructions: 'Write a Python program that prints "Hello, MultiOS!" to the console.',
        starterCode: `# Write your Python code here
# Print "Hello, MultiOS!" to the console


`,
        solution: `print("Hello, MultiOS!")`,
        language: 'python'
      }
    },
    {
      id: '2',
      title: 'Variables and Data Types',
      content: `# Variables and Data Types in Python

Python has several built-in data types that you can use to store different kinds of information.

## Common Data Types:
- **int**: Whole numbers (1, 42, -5)
- **float**: Decimal numbers (3.14, -2.5)
- **str**: Text strings ("Hello", 'Python')
- **bool**: True or False values
- **list**: Ordered collections of items
- **dict**: Key-value pairs

## Variable Naming Rules:
- Must start with a letter or underscore
- Can contain letters, numbers, and underscores
- Case-sensitive
- Cannot use Python keywords`,
      exercise: {
        instructions: 'Create variables of different data types and print their values with their types.',
        starterCode: `# Create variables of different types
# Integer: age = 25
# Float: price = 19.99
# String: name = "MultiOS"
# Boolean: is_student = True

# Print each variable and its type using type()



`,
        solution: `# Create variables of different types
age = 25
price = 19.99
name = "MultiOS"
is_student = True

# Print each variable and its type
print(f"Age: {age} (type: {type(age).__name__})")
print(f"Price: {price} (type: {type(price).__name__})")
print(f"Name: {name} (type: {type(name).__name__})")
print(f"Is Student: {is_student} (type: {type(is_student).__name__})")`,
        language: 'python'
      }
    }
  ];

  const filteredTutorials = tutorials.filter(tutorial => {
    const matchesSearch = tutorial.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         tutorial.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         tutorial.tags.some(tag => tag.toLowerCase().includes(searchTerm.toLowerCase()));
    
    const matchesCategory = selectedCategory === 'All' || tutorial.category === selectedCategory;
    const matchesDifficulty = selectedDifficulty === 'All' || tutorial.difficulty === selectedDifficulty;
    
    return matchesSearch && matchesCategory && matchesDifficulty;
  });

  const getDifficultyColor = (difficulty: string) => {
    switch (difficulty) {
      case 'Beginner':
        return 'bg-green-100 text-green-800';
      case 'Intermediate':
        return 'bg-yellow-100 text-yellow-800';
      case 'Advanced':
        return 'bg-red-100 text-red-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  if (selectedTutorial && !selectedTutorial.id) {
    // Tutorial detail view
    return (
      <div className="min-h-screen bg-slate-50">
        {/* Header */}
        <div className="bg-white border-b border-slate-200">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
            <div className="flex items-center justify-between">
              <button
                onClick={() => setSelectedTutorial(null)}
                className="flex items-center space-x-2 text-slate-600 hover:text-slate-900"
              >
                <ArrowLeft className="h-5 w-5" />
                <span>Back to Tutorials</span>
              </button>
              <button className="bg-gradient-to-r from-blue-600 to-purple-600 text-white px-6 py-2 rounded-lg">
                Continue Learning
              </button>
            </div>
          </div>
        </div>

        {/* Tutorial Content */}
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <div className="grid grid-cols-1 lg:grid-cols-4 gap-8">
            {/* Sidebar */}
            <div className="lg:col-span-1">
              <div className="bg-white rounded-xl shadow-lg border border-slate-200 p-6 sticky top-8">
                <h3 className="text-lg font-semibold text-slate-900 mb-4">Course Progress</h3>
                <div className="w-full bg-slate-200 rounded-full h-3 mb-4">
                  <div
                    className="bg-gradient-to-r from-blue-600 to-purple-600 h-3 rounded-full"
                    style={{ width: `${selectedTutorial.progress || 0}%` }}
                  ></div>
                </div>
                <p className="text-sm text-slate-600 mb-6">{selectedTutorial.progress || 0}% complete</p>

                <h4 className="font-medium text-slate-900 mb-3">Lessons</h4>
                <div className="space-y-2">
                  {sampleLessons.map((lesson, index) => (
                    <button
                      key={lesson.id}
                      onClick={() => setCurrentLesson(index)}
                      className={`w-full text-left p-3 rounded-lg transition-colors ${
                        currentLesson === index
                          ? 'bg-blue-100 text-blue-800'
                          : 'hover:bg-slate-100'
                      }`}
                    >
                      <div className="flex items-center space-x-2">
                        {lesson.completed ? (
                          <CheckCircle className="h-4 w-4 text-green-600" />
                        ) : (
                          <Circle className="h-4 w-4 text-slate-400" />
                        )}
                        <span className="text-sm font-medium">{lesson.title}</span>
                      </div>
                    </button>
                  ))}
                </div>
              </div>
            </div>

            {/* Main Content */}
            <div className="lg:col-span-3">
              <div className="bg-white rounded-xl shadow-lg border border-slate-200 overflow-hidden">
                {/* Lesson Header */}
                <div className="bg-gradient-to-r from-blue-600 to-purple-600 text-white p-6">
                  <div className="flex items-center justify-between">
                    <div>
                      <h1 className="text-2xl font-bold mb-2">
                        {sampleLessons[currentLesson]?.title}
                      </h1>
                      <p className="text-blue-100">
                        Lesson {currentLesson + 1} of {sampleLessons.length}
                      </p>
                    </div>
                    <div className="flex items-center space-x-2">
                      <button
                        onClick={() => setCurrentLesson(Math.max(0, currentLesson - 1))}
                        disabled={currentLesson === 0}
                        className="p-2 rounded-lg bg-white/20 hover:bg-white/30 disabled:opacity-50 disabled:cursor-not-allowed"
                      >
                        <ArrowLeft className="h-5 w-5" />
                      </button>
                      <button
                        onClick={() => setCurrentLesson(Math.min(sampleLessons.length - 1, currentLesson + 1))}
                        disabled={currentLesson === sampleLessons.length - 1}
                        className="p-2 rounded-lg bg-white/20 hover:bg-white/30 disabled:opacity-50 disabled:cursor-not-allowed"
                      >
                        <ArrowRight className="h-5 w-5" />
                      </button>
                    </div>
                  </div>
                </div>

                {/* Lesson Content */}
                <div className="p-6">
                  <div className="prose max-w-none">
                    <pre className="whitespace-pre-wrap text-slate-700 leading-relaxed">
                      {sampleLessons[currentLesson]?.content}
                    </pre>
                  </div>

                  {/* Exercise Section */}
                  {sampleLessons[currentLesson]?.exercise && (
                    <div className="mt-8 border-t border-slate-200 pt-8">
                      <h3 className="text-xl font-semibold text-slate-900 mb-4 flex items-center space-x-2">
                        <Zap className="h-5 w-5 text-yellow-500" />
                        <span>Practice Exercise</span>
                      </h3>
                      
                      <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-6">
                        <h4 className="font-medium text-yellow-800 mb-2">Instructions:</h4>
                        <p className="text-yellow-700">
                          {sampleLessons[currentLesson].exercise?.instructions}
                        </p>
                      </div>

                      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                        <div>
                          <h4 className="font-medium text-slate-900 mb-3">Starter Code:</h4>
                          <textarea
                            value={sampleLessons[currentLesson].exercise?.starterCode}
                            readOnly
                            className="w-full h-64 p-4 border border-slate-300 rounded-lg font-mono text-sm bg-slate-50"
                          />
                        </div>
                        
                        <div>
                          <h4 className="font-medium text-slate-900 mb-3">Solution:</h4>
                          <pre className="w-full h-64 p-4 border border-slate-300 rounded-lg font-mono text-sm bg-slate-50 overflow-auto">
                            {sampleLessons[currentLesson].exercise?.solution}
                          </pre>
                        </div>
                      </div>
                    </div>
                  )}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-slate-50">
      {/* Header */}
      <div className="bg-white border-b border-slate-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <div className="text-center">
            <div className="flex justify-center mb-4">
              <div className="bg-gradient-to-r from-blue-600 to-purple-600 p-3 rounded-xl">
                <BookOpen className="h-8 w-8 text-white" />
              </div>
            </div>
            <h1 className="text-4xl md:text-5xl font-bold text-slate-900 mb-4">
              Interactive Tutorials
            </h1>
            <p className="text-xl text-slate-600 max-w-3xl mx-auto">
              Learn MultiOS development through hands-on tutorials with embedded coding exercises, 
              real-time feedback, and progressive skill building.
            </p>
          </div>
        </div>
      </div>

      {/* Learning Path Stats */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
          <div className="bg-white rounded-xl shadow-lg border border-slate-200 p-6 text-center">
            <div className="bg-gradient-to-r from-blue-600 to-purple-600 p-3 rounded-xl w-fit mx-auto mb-4">
              <Target className="h-6 w-6 text-white" />
            </div>
            <div className="text-2xl font-bold text-slate-900 mb-2">50+</div>
            <div className="text-slate-600">Interactive Tutorials</div>
          </div>
          
          <div className="bg-white rounded-xl shadow-lg border border-slate-200 p-6 text-center">
            <div className="bg-gradient-to-r from-green-600 to-emerald-600 p-3 rounded-xl w-fit mx-auto mb-4">
              <Award className="h-6 w-6 text-white" />
            </div>
            <div className="text-2xl font-bold text-slate-900 mb-2">100+</div>
            <div className="text-slate-600">Coding Exercises</div>
          </div>
          
          <div className="bg-white rounded-xl shadow-lg border border-slate-200 p-6 text-center">
            <div className="bg-gradient-to-r from-yellow-600 to-orange-600 p-3 rounded-xl w-fit mx-auto mb-4">
              <Users className="h-6 w-6 text-white" />
            </div>
            <div className="text-2xl font-bold text-slate-900 mb-2">5,000+</div>
            <div className="text-slate-600">Active Learners</div>
          </div>
          
          <div className="bg-white rounded-xl shadow-lg border border-slate-200 p-6 text-center">
            <div className="bg-gradient-to-r from-purple-600 to-pink-600 p-3 rounded-xl w-fit mx-auto mb-4">
              <TrendingUp className="h-6 w-6 text-white" />
            </div>
            <div className="text-2xl font-bold text-slate-900 mb-2">95%</div>
            <div className="text-slate-600">Completion Rate</div>
          </div>
        </div>
      </div>

      {/* Filters and Search */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="bg-white rounded-xl shadow-lg border border-slate-200 p-6">
          <div className="flex flex-col lg:flex-row gap-4 items-center justify-between">
            {/* Search */}
            <div className="relative flex-1 max-w-md">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-slate-400" />
              <input
                type="text"
                placeholder="Search tutorials..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="w-full pl-10 pr-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              />
            </div>

            {/* Filters */}
            <div className="flex items-center space-x-4">
              <select
                value={selectedCategory}
                onChange={(e) => setSelectedCategory(e.target.value)}
                className="px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              >
                {categories.map(category => (
                  <option key={category} value={category}>{category}</option>
                ))}
              </select>

              <select
                value={selectedDifficulty}
                onChange={(e) => setSelectedDifficulty(e.target.value)}
                className="px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              >
                {difficulties.map(difficulty => (
                  <option key={difficulty} value={difficulty}>{difficulty}</option>
                ))}
              </select>

              {/* View Mode Toggle */}
              <div className="flex items-center border border-slate-300 rounded-lg overflow-hidden">
                <button
                  onClick={() => setViewMode('grid')}
                  className={`p-2 ${viewMode === 'grid' ? 'bg-blue-600 text-white' : 'text-slate-600 hover:bg-slate-100'}`}
                >
                  <Grid3X3 className="h-4 w-4" />
                </button>
                <button
                  onClick={() => setViewMode('list')}
                  className={`p-2 ${viewMode === 'list' ? 'bg-blue-600 text-white' : 'text-slate-600 hover:bg-slate-100'}`}
                >
                  <List className="h-4 w-4" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Tutorials Grid/List */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pb-16">
        {viewMode === 'grid' ? (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {filteredTutorials.map((tutorial) => (
              <div
                key={tutorial.id}
                className="bg-white rounded-xl shadow-lg border border-slate-200 overflow-hidden hover:shadow-xl transition-all duration-300"
              >
                <div className="p-6">
                  <div className="flex items-center justify-between mb-4">
                    <span className="text-sm font-medium text-slate-600">{tutorial.language}</span>
                    <span className={`px-2 py-1 rounded-full text-xs font-medium ${getDifficultyColor(tutorial.difficulty)}`}>
                      {tutorial.difficulty}
                    </span>
                  </div>
                  
                  <h3 className="text-xl font-bold text-slate-900 mb-2">{tutorial.title}</h3>
                  <p className="text-slate-600 mb-4 line-clamp-3">{tutorial.description}</p>
                  
                  <div className="flex flex-wrap gap-2 mb-4">
                    {tutorial.tags.slice(0, 3).map((tag, index) => (
                      <span key={index} className="px-2 py-1 bg-slate-100 text-slate-600 rounded-full text-xs">
                        {tag}
                      </span>
                    ))}
                    {tutorial.tags.length > 3 && (
                      <span className="px-2 py-1 bg-slate-100 text-slate-600 rounded-full text-xs">
                        +{tutorial.tags.length - 3} more
                      </span>
                    )}
                  </div>
                  
                  <div className="flex items-center justify-between mb-4">
                    <div className="flex items-center space-x-4 text-sm text-slate-500">
                      <div className="flex items-center space-x-1">
                        <Clock className="h-4 w-4" />
                        <span>{tutorial.duration}</span>
                      </div>
                      <div className="flex items-center space-x-1">
                        <BookOpen className="h-4 w-4" />
                        <span>{tutorial.lessons} lessons</span>
                      </div>
                    </div>
                    <div className="flex items-center space-x-1 text-sm">
                      <Star className="h-4 w-4 fill-current text-yellow-500" />
                      <span className="text-slate-600">{tutorial.rating}</span>
                    </div>
                  </div>

                  {/* Progress Bar */}
                  {tutorial.progress !== undefined && tutorial.progress > 0 && (
                    <div className="mb-4">
                      <div className="flex items-center justify-between text-sm text-slate-600 mb-1">
                        <span>Progress</span>
                        <span>{tutorial.progress}%</span>
                      </div>
                      <div className="w-full bg-slate-200 rounded-full h-2">
                        <div
                          className="bg-gradient-to-r from-blue-600 to-purple-600 h-2 rounded-full"
                          style={{ width: `${tutorial.progress}%` }}
                        ></div>
                      </div>
                    </div>
                  )}
                  
                  <button
                    onClick={() => setSelectedTutorial(tutorial)}
                    className="w-full bg-gradient-to-r from-blue-600 to-purple-600 text-white py-3 rounded-lg hover:shadow-lg transition-all duration-300 flex items-center justify-center space-x-2"
                  >
                    <Play className="h-4 w-4" />
                    <span>{tutorial.progress ? 'Continue' : 'Start'} Tutorial</span>
                  </button>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="space-y-6">
            {filteredTutorials.map((tutorial) => (
              <div
                key={tutorial.id}
                className="bg-white rounded-xl shadow-lg border border-slate-200 p-6 hover:shadow-xl transition-all duration-300"
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-4">
                    <div className="bg-gradient-to-r from-blue-600 to-purple-600 p-3 rounded-xl">
                      <BookOpen className="h-6 w-6 text-white" />
                    </div>
                    <div>
                      <div className="flex items-center space-x-3 mb-2">
                        <h3 className="text-xl font-bold text-slate-900">{tutorial.title}</h3>
                        <span className="text-sm font-medium text-slate-600">{tutorial.language}</span>
                        <span className={`px-2 py-1 rounded-full text-xs font-medium ${getDifficultyColor(tutorial.difficulty)}`}>
                          {tutorial.difficulty}
                        </span>
                      </div>
                      <p className="text-slate-600 mb-2">{tutorial.description}</p>
                      <div className="flex flex-wrap gap-2 mb-3">
                        {tutorial.tags.map((tag, index) => (
                          <span key={index} className="px-2 py-1 bg-slate-100 text-slate-600 rounded-full text-xs">
                            {tag}
                          </span>
                        ))}
                      </div>
                      <div className="flex items-center space-x-6 text-sm text-slate-500">
                        <div className="flex items-center space-x-1">
                          <Clock className="h-4 w-4" />
                          <span>{tutorial.duration}</span>
                        </div>
                        <div className="flex items-center space-x-1">
                          <BookOpen className="h-4 w-4" />
                          <span>{tutorial.lessons} lessons</span>
                        </div>
                        <div className="flex items-center space-x-1">
                          <Users className="h-4 w-4" />
                          <span>{tutorial.students} students</span>
                        </div>
                        <div className="flex items-center space-x-1">
                          <Star className="h-4 w-4 fill-current text-yellow-500" />
                          <span>{tutorial.rating}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                  
                  <div className="flex items-center space-x-4">
                    {tutorial.progress !== undefined && tutorial.progress > 0 && (
                      <div className="text-right text-sm text-slate-600">
                        <div className="mb-2">Progress: {tutorial.progress}%</div>
                        <div className="w-24 bg-slate-200 rounded-full h-2">
                          <div
                            className="bg-gradient-to-r from-blue-600 to-purple-600 h-2 rounded-full"
                            style={{ width: `${tutorial.progress}%` }}
                          ></div>
                        </div>
                      </div>
                    )}
                    
                    <button
                      onClick={() => setSelectedTutorial(tutorial)}
                      className="bg-gradient-to-r from-blue-600 to-purple-600 text-white px-6 py-3 rounded-lg hover:shadow-lg transition-all duration-300 flex items-center space-x-2"
                    >
                      <Play className="h-4 w-4" />
                      <span>{tutorial.progress ? 'Continue' : 'Start'}</span>
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};