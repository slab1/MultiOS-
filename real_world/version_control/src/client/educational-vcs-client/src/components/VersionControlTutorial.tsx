import React, { useState, useEffect } from 'react';
import { 
  PlayIcon,
  CheckCircleIcon,
  ArrowRightIcon,
  BookOpenIcon,
  CodeBracketIcon,
  UserGroupIcon,
  ExclamationTriangleIcon,
  LightBulbIcon
} from '@heroicons/react/24/outline';

const VersionControlTutorial = () => {
  const [currentModule, setCurrentModule] = useState(0);
  const [completedLessons, setCompletedLessons] = useState([]);
  const [activeExercise, setActiveExercise] = useState(null);
  const [userProgress, setUserProgress] = useState({});

  const tutorials = [
    {
      id: 'basics',
      title: 'Git Basics',
      description: 'Learn the fundamental concepts of version control',
      icon: BookOpenIcon,
      duration: '30 min',
      lessons: [
        {
          id: 'what-is-git',
          title: 'What is Version Control?',
          content: `
            <h3>What is Version Control?</h3>
            <p>Version control is like a time machine for your code. It tracks every change you make to your files, 
            allowing you to:</p>
            <ul>
              <li>Go back to any previous version of your code</li>
              <li>Compare different versions side by side</li>
              <li>See who made what changes and when</li>
              <li>Work on different features without interfering with each other</li>
            </ul>
            
            <h4>Real-World Analogy</h4>
            <p>Think of version control like Google Docs history, but much more powerful. 
            Instead of just undoing changes, you can:</p>
            <ul>
              <li>Create "branches" to experiment with new ideas</li>
              <li>Merge your experiments back into the main document</li>
              <li>See exactly what changed between versions</li>
              <li>Work on the same document with multiple people simultaneously</li>
            </ul>
          `,
          exercise: 'concept-check'
        },
        {
          id: 'repository',
          title: 'Understanding Repositories',
          content: `
            <h3>What is a Repository?</h3>
            <p>A repository (or "repo") is a special folder that contains your project AND 
            a complete history of all changes made to it.</p>
            
            <h4>Repository Structure</h4>
            <pre><code>my-project/
├── .edu_vcs/          # The version control folder (usually hidden)
│   ├── objects/       # Stores all versions of your files
│   ├── refs/          # Points to different versions (branches)
│   └── config         # Repository settings
├── src/               # Your actual project files
├── docs/              # Documentation
└── README.md          # Project description
            </code></pre>
            
            <h4>Key Components</h4>
            <ul>
              <li><strong>Objects:</strong> All your file versions, compressed and stored efficiently</li>
              <li><strong>Refs:</strong> Named pointers to important versions (like "main" branch)</li>
              <li><strong>Config:</strong> Repository settings and metadata</li>
            </ul>
          `,
          exercise: 'create-repo'
        },
        {
          id: 'commits',
          title: 'Making Commits',
          content: `
            <h3>What is a Commit?</h3>
            <p>A commit is like taking a snapshot of your project at a specific moment in time. 
            Each commit captures:</p>
            <ul>
              <li>Which files changed</li>
              <li>What changed in those files</li>
              <li>Who made the changes</li>
              <li>When the changes were made</li>
              <li>A message describing why the changes were made</li>
            </ul>
            
            <h4>Commit Anatomy</h4>
            <pre><code>commit abc12345
Author: Alice Smith <alice@school.edu>
Date:   Mon Dec 4 10:30:00 2024 -0500

    Add user authentication feature

    - Implement login form
    - Add session management
    - Create user profile page</code></pre>
            
            <h4>Best Practices</h4>
            <ul>
              <li>Write clear, descriptive commit messages</li>
              <li>Commit related changes together</li>
              <li>Commit frequently (small, logical chunks)</li>
              <li>Don't commit sensitive information (passwords, API keys)</li>
            </ul>
          `,
          exercise: 'practice-commit'
        }
      ]
    },
    {
      id: 'branching',
      title: 'Branching and Merging',
      description: 'Learn to work with branches for parallel development',
      icon: CodeBracketIcon,
      duration: '45 min',
      lessons: [
        {
          id: 'branches-intro',
          title: 'Introduction to Branches',
          content: `
            <h3>What are Branches?</h3>
            <p>Branches allow you to work on different versions of your project simultaneously. 
            Think of it like having multiple copies of your project where each copy can evolve independently.</p>
            
            <h4>Common Branching Strategies</h4>
            <ul>
              <li><strong>main:</strong> The stable, production-ready version</li>
              <li><strong>develop:</strong> Integration branch for features</li>
              <li><strong>feature/*:</strong> Individual feature development</li>
              <li><strong>bugfix/*:</strong> Bug fixes</li>
              <li><strong>release/*:</strong> Preparation for releases</li>
            </ul>
            
            <h4>Visual Representation</h4>
            <pre><code>main:     A---B---C---F---G
                \         /
feature:  D---E---------H</code></pre>
            
            <p>This shows feature branch H being merged back into main at point G.</p>
          `,
          exercise: 'branch-workflow'
        },
        {
          id: 'merging',
          title: 'Merging Branches',
          content: `
            <h3>Merging: Combining Changes</h3>
            <p>Merging combines changes from one branch into another. There are different types of merges:</p>
            
            <h4>Fast-Forward Merge</h4>
            <p>When the target branch hasn't changed since you branched from it:</p>
            <pre><code>Before:  main: A---B
                feature:    C---D

After:   main: A---B---C---D</code></pre>
            
            <h4>Merge Commit</h4>
            <p>When both branches have new changes:</p>
            <pre><code>Before:  main: A---B---E
                feature:    C---D

After:   main: A---B---E---F
                      \\     /
                  feature: C---D</code></pre>
            
            <h4>Merge Conflicts</h4>
            <p>Sometimes Git can't automatically merge changes. This happens when the same lines 
            were modified in both branches. You'll need to resolve these manually.</p>
          `,
          exercise: 'resolve-conflict'
        }
      ]
    },
    {
      id: 'collaboration',
      title: 'Collaborative Development',
      description: 'Work effectively with others using version control',
      icon: UserGroupIcon,
      duration: '60 min',
      lessons: [
        {
          id: 'workflows',
          title: 'Team Workflows',
          content: `
            <h3>Collaborative Workflows</h3>
            <p>Effective team collaboration requires established workflows and conventions.</p>
            
            <h4>Feature Branch Workflow</h4>
            <ol>
              <li>Create a feature branch from main</li>
              <li>Develop your feature on the branch</li>
              <li>Push your branch to remote repository</li>
              <li>Create a pull request for review</li>
              <li>Address review comments</li>
              <li>Merge to main when approved</li>
            </ol>
            
            <h4>Code Review Process</h4>
            <ul>
              <li><strong>Peer Review:</strong> Team members review each other's code</li>
              <li><strong>Educational Comments:</strong> Learn from others' approaches</li>
              <li><strong>Quality Checks:</strong> Ensure code meets standards</li>
              <li><strong>Knowledge Sharing:</strong> Share techniques and best practices</li>
            </ul>
            
            <h4>Communication is Key</h4>
            <p>Always communicate with your team about:</p>
            <ul>
              <li>What you're working on</li>
              <li>Problems you encounter</li>
              <li>Questions about the codebase</li>
              <li>Proposals for improvements</li>
            </ul>
          `,
          exercise: 'collaborative-practice'
        },
        {
          id: 'conflicts',
          title: 'Resolving Conflicts',
          content: `
            <h3>Handling Merge Conflicts</h3>
            <p>Conflicts are a normal part of collaboration. Here's how to handle them:</p>
            
            <h4>Identifying Conflicts</h4>
            <pre><code>&lt;&lt;&lt;&lt;&lt;&lt;&lt; HEAD
Your changes
=======
Incoming changes
&gt;&gt;&gt;&gt;&gt;&gt;&gt; feature-branch</code></pre>
            
            <h4>Resolution Steps</h4>
            <ol>
              <li>Don't panic! Conflicts are normal</li>
              <li>Read the conflict markers carefully</li>
              <li>Understand what both changes were trying to do</li>
              <li>Decide on the final version (sometimes both are needed)</li>
              <li>Remove the conflict markers</li>
              <li>Test the resolved code</li>
              <li>Commit the resolution</li>
            </ol>
            
            <h4>Prevention Tips</h4>
            <ul>
              <li>Pull frequently before pushing</li>
              <li>Work on different files when possible</li>
              <li>Communicate about major changes</li>
              <li>Use small, focused commits</li>
            </ul>
          `,
          exercise: 'conflict-resolution'
        }
      ]
    },
    {
      id: 'best-practices',
      title: 'Best Practices',
      description: 'Advanced tips for professional version control',
      icon: LightBulbIcon,
      duration: '40 min',
      lessons: [
        {
          id: 'commit-messages',
          title: 'Writing Great Commits',
          content: `
            <h3>Crafting Perfect Commit Messages</h3>
            
            <h4>Good Commit Message Structure</h4>
            <pre><code>type(scope): short description (50 chars max)

Longer explanation if needed (72 chars per line)

- Bullet points for complex changes
- Reference issue numbers
- Mention breaking changes</code></pre>
            
            <h4>Common Commit Types</h4>
            <ul>
              <li><strong>feat:</strong> New feature</li>
              <li><strong>fix:</strong> Bug fix</li>
              <li><strong>docs:</strong> Documentation changes</li>
              <li><strong>style:</strong> Formatting changes</li>
              <li><strong>refactor:</strong> Code restructuring</li>
              <li><strong>test:</strong> Adding tests</li>
              <li><strong>chore:</strong> Maintenance tasks</li>
            </ul>
            
            <h4>Examples</h4>
            <pre><code>feat(auth): add OAuth2 login support

- Implement Google OAuth2 flow
- Add user session management  
- Update authentication middleware
- Add logout functionality

Fixes #123

refactor(api): simplify user model validation

- Combine duplicate validation logic
- Use centralized error handling
- Improve error messages
- Add comprehensive tests</code></pre>
          `,
          exercise: 'commit-crafting'
        }
      ]
    }
  ];

  useEffect(() => {
    // Load progress from localStorage
    const savedProgress = localStorage.getItem('tutorial_progress');
    if (savedProgress) {
      setUserProgress(JSON.parse(savedProgress));
    }
    
    const savedCompleted = localStorage.getItem('completed_lessons');
    if (savedCompleted) {
      setCompletedLessons(JSON.parse(savedCompleted));
    }
  }, []);

  const saveProgress = (newProgress) => {
    setUserProgress(newProgress);
    localStorage.setItem('tutorial_progress', JSON.stringify(newProgress));
  };

  const markLessonComplete = (moduleId, lessonId) => {
    const key = `${moduleId}-${lessonId}`;
    if (!completedLessons.includes(key)) {
      const newCompleted = [...completedLessons, key];
      setCompletedLessons(newCompleted);
      localStorage.setItem('completed_lessons', JSON.stringify(newCompleted));
    }
  };

  const getOverallProgress = () => {
    const totalLessons = tutorials.reduce((acc, module) => acc + module.lessons.length, 0);
    const completedCount = completedLessons.length;
    return Math.round((completedCount / totalLessons) * 100);
  };

  const renderLessonContent = (lesson) => {
    return (
      <div className="prose max-w-none">
        <div dangerouslySetInnerHTML={{ __html: lesson.content }} />
        
        {/* Exercise Section */}
        {lesson.exercise && (
          <div className="mt-8 p-6 bg-blue-50 rounded-lg border border-blue-200">
            <h4 className="text-lg font-semibold text-blue-900 mb-4 flex items-center">
              <CodeBracketIcon className="w-5 h-5 mr-2" />
              Practice Exercise
            </h4>
            
            <div className="space-y-4">
              <p className="text-blue-800">
                Let's practice what you learned! Complete this interactive exercise to reinforce your understanding.
              </p>
              
              <div className="bg-white p-4 rounded border">
                <h5 className="font-medium text-gray-900 mb-2">Exercise: {lesson.title}</h5>
                <p className="text-gray-700 mb-4">
                  {getExerciseDescription(lesson.exercise)}
                </p>
                
                <div className="space-y-3">
                  <textarea
                    placeholder="Write your answer or code here..."
                    className="w-full h-32 p-3 border border-gray-300 rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                  
                  <div className="flex space-x-3">
                    <button
                      onClick={() => markLessonComplete(currentModule, lesson.id)}
                      className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                    >
                      Submit Answer
                    </button>
                    
                    <button className="px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors">
                      Show Hint
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    );
  };

  const getExerciseDescription = (exerciseType) => {
    const descriptions = {
      'concept-check': 'Answer these questions to test your understanding of version control concepts.',
      'create-repo': 'Practice creating your first repository and understanding its structure.',
      'practice-commit': 'Write a commit message for a given scenario and explain your choice.',
      'branch-workflow': 'Design a branching strategy for a team project.',
      'resolve-conflict': 'Given a conflict scenario, decide how to resolve it.',
      'collaborative-practice': 'Plan a collaborative workflow for a team assignment.',
      'conflict-resolution': 'Walk through resolving a complex merge conflict.',
      'commit-crafting': 'Rewrite these commit messages to make them more professional.'
    };
    
    return descriptions[exerciseType] || 'Complete the following exercise to practice your skills.';
  };

  const currentTutorial = tutorials[currentModule];
  const currentLesson = currentTutorial?.lessons[0]; // Simplified for demo

  return (
    <div className="h-full flex">
      {/* Sidebar */}
      <div className="w-80 bg-white border-r border-gray-200 overflow-y-auto">
        <div className="p-6 border-b border-gray-200">
          <h1 className="text-2xl font-bold text-gray-900">VCS Tutorial</h1>
          <p className="text-gray-600 mt-1">Interactive Learning</p>
          
          {/* Progress Bar */}
          <div className="mt-4">
            <div className="flex justify-between text-sm text-gray-600 mb-1">
              <span>Overall Progress</span>
              <span>{getOverallProgress()}%</span>
            </div>
            <div className="w-full bg-gray-200 rounded-full h-2">
              <div 
                className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                style={{ width: `${getOverallProgress()}%` }}
              ></div>
            </div>
          </div>
        </div>

        <div className="p-4">
          <div className="space-y-2">
            {tutorials.map((module, index) => {
              const Icon = module.icon;
              const moduleCompleted = module.lessons.every(lesson => 
                completedLessons.includes(`${index}-${lesson.id}`)
              );
              
              return (
                <button
                  key={module.id}
                  onClick={() => setCurrentModule(index)}
                  className={`
                    w-full text-left p-4 rounded-lg border transition-colors
                    ${currentModule === index
                      ? 'bg-blue-50 border-blue-200 text-blue-900'
                      : 'bg-white border-gray-200 text-gray-700 hover:bg-gray-50'
                    }
                  `}
                >
                  <div className="flex items-center space-x-3">
                    <Icon className="w-6 h-6" />
                    <div className="flex-1">
                      <h3 className="font-medium">{module.title}</h3>
                      <p className="text-sm text-gray-600">{module.description}</p>
                      <div className="flex items-center justify-between mt-2">
                        <span className="text-xs text-gray-500">{module.duration}</span>
                        {moduleCompleted && (
                          <CheckCircleIcon className="w-4 h-4 text-green-500" />
                        )}
                      </div>
                    </div>
                  </div>
                </button>
              );
            })}
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 overflow-y-auto">
        {currentTutorial && (
          <div className="p-8">
            {/* Module Header */}
            <div className="mb-8">
              <div className="flex items-center space-x-3 mb-4">
                <currentTutorial.icon className="w-8 h-8 text-blue-600" />
                <div>
                  <h1 className="text-3xl font-bold text-gray-900">{currentTutorial.title}</h1>
                  <p className="text-gray-600">{currentTutorial.description}</p>
                </div>
              </div>
              
              {/* Module Progress */}
              <div className="bg-gray-200 rounded-full h-2 w-full">
                <div 
                  className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                  style={{ 
                    width: `${(completedLessons.filter(key => key.startsWith(`${currentModule}-`)).length / currentTutorial.lessons.length) * 100}%` 
                  }}
                ></div>
              </div>
            </div>

            {/* Lesson Navigation */}
            <div className="flex space-x-4 mb-8 overflow-x-auto">
              {currentTutorial.lessons.map((lesson, index) => {
                const isCompleted = completedLessons.includes(`${currentModule}-${lesson.id}`);
                
                return (
                  <button
                    key={lesson.id}
                    className={`
                      flex items-center space-x-2 px-4 py-2 rounded-lg border transition-colors whitespace-nowrap
                      ${isCompleted
                        ? 'bg-green-50 border-green-200 text-green-800'
                        : index === 0
                        ? 'bg-blue-50 border-blue-200 text-blue-800'
                        : 'bg-white border-gray-200 text-gray-700 hover:bg-gray-50'
                      }
                    `}
                  >
                    {isCompleted ? (
                      <CheckCircleIcon className="w-4 h-4" />
                    ) : (
                      <PlayIcon className="w-4 h-4" />
                    )}
                    <span className="text-sm font-medium">{lesson.title}</span>
                  </button>
                );
              })}
            </div>

            {/* Current Lesson */}
            {currentLesson && (
              <div>
                <h2 className="text-2xl font-bold text-gray-900 mb-6">
                  {currentLesson.title}
                </h2>
                
                {renderLessonContent(currentLesson)}
                
                {/* Action Buttons */}
                <div className="mt-8 flex justify-between items-center">
                  <button
                    className="px-6 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors"
                    disabled={currentModule === 0}
                  >
                    Previous Module
                  </button>
                  
                  <button
                    onClick={() => markLessonComplete(currentModule, currentLesson.id)}
                    className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors flex items-center space-x-2"
                  >
                    <CheckCircleIcon className="w-4 h-4" />
                    <span>Mark as Complete</span>
                  </button>
                  
                  <button
                    className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors flex items-center space-x-2"
                    disabled={currentModule === tutorials.length - 1}
                  >
                    <span>Next Module</span>
                    <ArrowRightIcon className="w-4 h-4" />
                  </button>
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};

export default VersionControlTutorial;
