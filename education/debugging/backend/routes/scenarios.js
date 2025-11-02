const express = require('express');
const router = express.Router();

// Scenario management
let scenarioLibrary = new Map();
let activeScenarios = new Map();
let userProgress = new Map();

// Initialize built-in scenarios
initializeScenarios();

// Get all scenarios
router.get('/', async (req, res) => {
  try {
    const { category, difficulty, search } = req.query;
    
    let scenarios = Array.from(scenarioLibrary.values());
    
    // Apply filters
    if (category) {
      scenarios = scenarios.filter(s => s.category === category);
    }
    
    if (difficulty) {
      scenarios = scenarios.filter(s => s.difficulty === parseInt(difficulty));
    }
    
    if (search) {
      const searchLower = search.toLowerCase();
      scenarios = scenarios.filter(s => 
        s.name.toLowerCase().includes(searchLower) ||
        s.description.toLowerCase().includes(searchLower) ||
        s.tags.some(tag => tag.toLowerCase().includes(searchLower))
      );
    }
    
    // Add progress information
    scenarios = scenarios.map(scenario => {
      const progress = userProgress.get(scenario.id);
      return {
        ...scenario,
        progress: progress ? {
          completed: progress.completed || false,
          currentStep: progress.currentStep || 0,
          totalSteps: scenario.steps?.length || 0,
          lastAccessed: progress.lastAccessed,
          timeSpent: progress.timeSpent || 0
        } : null
      };
    });
    
    res.json({
      scenarios,
      count: scenarios.length,
      filters: {
        categories: [...new Set(scenarios.map(s => s.category))],
        difficulties: [...new Set(scenarios.map(s => s.difficulty))].sort()
      }
    });
  } catch (error) {
    console.error('Error getting scenarios:', error);
    res.status(500).json({ error: 'Failed to get scenarios' });
  }
});

// Get specific scenario
router.get('/:id', async (req, res) => {
  try {
    const { id } = req.params;
    
    const scenario = scenarioLibrary.get(id);
    if (!scenario) {
      return res.status(404).json({ error: 'Scenario not found' });
    }
    
    const progress = userProgress.get(id);
    
    res.json({
      scenario: {
        ...scenario,
        progress: progress ? {
          completed: progress.completed || false,
          currentStep: progress.currentStep || 0,
          totalSteps: scenario.steps?.length || 0,
          lastAccessed: progress.lastAccessed,
          timeSpent: progress.timeSpent || 0,
          completedSteps: progress.completedSteps || []
        } : null
      }
    });
  } catch (error) {
    console.error('Error getting scenario:', error);
    res.status(500).json({ error: 'Failed to get scenario' });
  }
});

// Start a scenario
router.post('/:id/start', async (req, res) => {
  try {
    const { id } = req.params;
    const { userId = 'default' } = req.body;
    
    const scenario = scenarioLibrary.get(id);
    if (!scenario) {
      return res.status(404).json({ error: 'Scenario not found' });
    }
    
    const sessionId = `${userId}_${id}_${Date.now()}`;
    
    // Initialize scenario session
    const session = {
      id: sessionId,
      scenarioId: id,
      userId,
      status: 'active',
      started: new Date().toISOString(),
      currentStep: 0,
      completedSteps: [],
      interactions: [],
      score: 0,
      hintsUsed: [],
      timeSpent: 0
    };
    
    activeScenarios.set(sessionId, session);
    
    // Update user progress
    const progress = userProgress.get(id) || {
      scenarioId: id,
      userId,
      sessions: [],
      completed: false,
      bestScore: 0,
      totalTimeSpent: 0
    };
    
    progress.sessions.push({
      sessionId,
      started: session.started,
      status: 'active'
    });
    
    progress.lastAccessed = new Date().toISOString();
    userProgress.set(id, progress);
    
    res.json({
      success: true,
      session,
      message: 'Scenario started successfully'
    });
  } catch (error) {
    console.error('Error starting scenario:', error);
    res.status(500).json({ error: 'Failed to start scenario' });
  }
});

// Complete a step in a scenario
router.post('/:id/step', async (req, res) => {
  try {
    const { id } = req.params;
    const { sessionId, stepId, action, result, timeSpent } = req.body;
    
    const session = activeScenarios.get(sessionId);
    if (!session || session.scenarioId !== id) {
      return res.status(404).json({ error: 'Scenario session not found' });
    }
    
    const scenario = scenarioLibrary.get(id);
    if (!scenario) {
      return res.status(404).json({ error: 'Scenario not found' });
    }
    
    const step = scenario.steps.find(s => s.id === parseInt(stepId));
    if (!step) {
      return res.status(404).json({ error: 'Step not found' });
    }
    
    // Record interaction
    const interaction = {
      stepId: parseInt(stepId),
      action,
      result,
      timestamp: new Date().toISOString(),
      timeSpent: timeSpent || 0
    };
    
    session.interactions.push(interaction);
    session.timeSpent += timeSpent || 0;
    
    // Check if step is completed
    const isCompleted = evaluateStepCompletion(step, action, result);
    
    if (isCompleted) {
      session.completedSteps.push(parseInt(stepId));
      session.score += calculateStepScore(step, result);
      session.currentStep = Math.max(session.currentStep, parseInt(stepId) + 1);
    }
    
    // Check if scenario is completed
    let scenarioCompleted = false;
    if (session.completedSteps.length === scenario.steps.length) {
      scenarioCompleted = true;
      session.status = 'completed';
      session.completed = new Date().toISOString();
      
      // Update user progress
      const progress = userProgress.get(id);
      if (progress) {
        progress.completed = true;
        progress.totalTimeSpent += session.timeSpent;
        progress.bestScore = Math.max(progress.bestScore, session.score);
        
        // Update session status
        const activeSession = progress.sessions.find(s => s.sessionId === sessionId);
        if (activeSession) {
          activeSession.status = 'completed';
          activeSession.completed = session.completed;
          activeSession.score = session.score;
        }
      }
    }
    
    res.json({
      success: true,
      stepCompleted: isCompleted,
      scenarioCompleted,
      session: {
        currentStep: session.currentStep,
        completedSteps: session.completedSteps,
        score: session.score,
        timeSpent: session.timeSpent
      },
      nextStep: getNextStep(scenario, session.currentStep)
    });
  } catch (error) {
    console.error('Error completing step:', error);
    res.status(500).json({ error: 'Failed to complete step' });
  }
});

// Get scenario session
router.get('/:id/session/:sessionId', async (req, res) => {
  try {
    const { id, sessionId } = req.params;
    
    const session = activeScenarios.get(sessionId);
    if (!session || session.scenarioId !== id) {
      return res.status(404).json({ error: 'Scenario session not found' });
    }
    
    res.json({
      session,
      scenario: scenarioLibrary.get(id)
    });
  } catch (error) {
    console.error('Error getting session:', error);
    res.status(500).json({ error: 'Failed to get session' });
  }
});

// End scenario session
router.post('/:id/session/:sessionId/end', async (req, res) => {
  try {
    const { id, sessionId } = req.params;
    const { reason = 'user_ended' } = req.body;
    
    const session = activeScenarios.get(sessionId);
    if (!session || session.scenarioId !== id) {
      return res.status(404).json({ error: 'Scenario session not found' });
    }
    
    session.status = reason === 'completed' ? 'completed' : 'ended';
    session.ended = new Date().toISOString();
    
    // Move to history (clean up active session)
    activeScenarios.delete(sessionId);
    
    // Update user progress
    const progress = userProgress.get(id);
    if (progress) {
      const activeSession = progress.sessions.find(s => s.sessionId === sessionId);
      if (activeSession) {
        activeSession.status = session.status;
        activeSession.ended = session.ended;
        activeSession.score = session.score;
        activeSession.timeSpent = session.timeSpent;
      }
    }
    
    res.json({
      success: true,
      session,
      message: 'Scenario session ended'
    });
  } catch (error) {
    console.error('Error ending session:', error);
    res.status(500).json({ error: 'Failed to end session' });
  }
});

// Get user progress for a scenario
router.get('/:id/progress', async (req, res) => {
  try {
    const { id } = req.params;
    const { userId = 'default' } = req.query;
    
    const progress = userProgress.get(id);
    if (!progress) {
      return res.json({
        progress: null,
        message: 'No progress found for this scenario'
      });
    }
    
    const userProgressData = progress.sessions.find(s => s.userId === userId);
    if (!userProgressData) {
      return res.json({
        progress: null,
        message: 'No progress found for this user'
      });
    }
    
    res.json({
      progress: userProgressData,
      totalProgress: {
        sessions: progress.sessions.length,
        completed: progress.completed,
        bestScore: progress.bestScore,
        totalTimeSpent: progress.totalTimeSpent,
        lastAccessed: progress.lastAccessed
      }
    });
  } catch (error) {
    console.error('Error getting progress:', error);
    res.status(500).json({ error: 'Failed to get progress' });
  }
});

// Get all user progress
router.get('/progress/all', async (req, res) => {
  try {
    const { userId = 'default' } = req.query;
    
    const allProgress = {};
    userProgress.forEach((progress, scenarioId) => {
      allProgress[scenarioId] = {
        ...progress,
        sessions: progress.sessions.filter(s => s.userId === userId)
      };
    });
    
    res.json({
      progress: allProgress,
      totalScenarios: Object.keys(allProgress).length,
      completedScenarios: Object.values(allProgress).filter(p => p.completed).length
    });
  } catch (error) {
    console.error('Error getting all progress:', error);
    res.status(500).json({ error: 'Failed to get progress' });
  }
});

// Reset scenario progress
router.post('/:id/reset', async (req, res) => {
  try {
    const { id } = req.params;
    const { userId = 'default' } = req.body;
    
    const progress = userProgress.get(id);
    if (!progress) {
      return res.status(404).json({ error: 'No progress found to reset' });
    }
    
    // Reset user-specific progress
    progress.sessions = progress.sessions.filter(s => s.userId !== userId);
    
    if (progress.sessions.length === 0) {
      userProgress.delete(id);
    } else {
      userProgress.set(id, progress);
    }
    
    // End any active sessions
    Array.from(activeScenarios.values()).forEach(session => {
      if (session.scenarioId === id && session.userId === userId) {
        activeScenarios.delete(session.id);
      }
    });
    
    res.json({
      success: true,
      message: 'Scenario progress reset successfully'
    });
  } catch (error) {
    console.error('Error resetting progress:', error);
    res.status(500).json({ error: 'Failed to reset progress' });
  }
});

// Get scenario statistics
router.get('/:id/stats', async (req, res) => {
  try {
    const { id } = req.params;
    
    const scenario = scenarioLibrary.get(id);
    if (!scenario) {
      return res.status(404).json({ error: 'Scenario not found' });
    }
    
    const progress = userProgress.get(id);
    const stats = {
      scenario: {
        id: scenario.id,
        name: scenario.name,
        difficulty: scenario.difficulty,
        estimatedTime: scenario.estimatedTime
      },
      usage: {
        totalSessions: progress?.sessions.length || 0,
        completedSessions: progress?.sessions.filter(s => s.status === 'completed').length || 0,
        activeSessions: Array.from(activeScenarios.values()).filter(s => s.scenarioId === id).length,
        averageScore: calculateAverageScore(progress?.sessions || []),
        averageTime: calculateAverageTime(progress?.sessions || [])
      },
      difficulty: {
        attempts: progress?.sessions.length || 0,
        completionRate: calculateCompletionRate(progress?.sessions || []),
        averageTime: calculateAverageTime(progress?.sessions || [])
      }
    };
    
    res.json(stats);
  } catch (error) {
    console.error('Error getting scenario stats:', error);
    res.status(500).json({ error: 'Failed to get scenario statistics' });
  }
});

// Helper functions
function initializeScenarios() {
  const scenarios = [
    {
      id: 'process-hang',
      name: 'Process Hang Investigation',
      category: 'beginner',
      difficulty: 1,
      estimatedTime: 15,
      description: 'Learn to debug and resolve process hangs using system tools and GDB',
      tags: ['hang', 'process', 'debugging', 'gdb'],
      learningObjectives: [
        'Identify hanging processes',
        'Use system monitoring tools',
        'Analyze process behavior with GDB',
        'Implement timeout mechanisms'
      ],
      prerequisites: [],
      files: [
        {
          path: '/examples/hang_program.c',
          content: getHangProgramSource(),
          language: 'c'
        }
      ],
      setup: {
        commands: [
          'gcc -o hang_program hang_program.c',
          './hang_program &'
        ],
        environment: {
          'HANG_TIMEOUT': '5'
        }
      },
      steps: [
        {
          id: 1,
          title: 'Initial Assessment',
          description: 'Identify that the program is hung',
          instructions: 'The program appears to be hung. Use system tools to identify the hanging process.',
          expectedOutcome: 'Find the process with unusual behavior',
          validation: {
            type: 'command',
            command: 'ps aux | grep hang_program',
            expectedPattern: 'hang_program.*<defunct|hang_program.*R.*[0-9]{2,}%'
          },
          hints: [
            'Use `ps aux` to see all running processes',
            'Look for processes with high CPU usage or in uninterruptible sleep (D state)'
          ],
          score: 10
        },
        {
          id: 2,
          title: 'Attach Debugger',
          description: 'Attach GDB to the hanging process',
          instructions: 'Use GDB to attach to the hanging process and examine its state.',
          expectedOutcome: 'Successfully attach to the process',
          validation: {
            type: 'gdb',
            commands: ['attach <PID>'],
            expectedPattern: 'Attached to process.*pid'
          },
          hints: [
            'Use the PID from the previous step',
            'GDB will pause the process when attached'
          ],
          score: 15
        },
        {
          id: 3,
          title: 'Analyze Call Stack',
          description: 'Examine where the process is stuck',
          instructions: 'Look at the call stack to see where the process is waiting.',
          expectedOutcome: 'Identify the function call where the process is stuck',
          validation: {
            type: 'gdb',
            commands: ['thread apply all bt'],
            expectedPattern: 'main.*hang_program.c.*sleep'
          },
          hints: [
            'Look for system calls like sleep() or wait()',
            'Thread information shows which threads are active'
          ],
          score: 20
        },
        {
          id: 4,
          title: 'Fix Implementation',
          description: 'Implement timeout mechanism',
          instructions: 'Add a timeout mechanism to prevent the hang.',
          expectedOutcome: 'Working timeout implementation',
          validation: {
            type: 'code',
            checkFunction: 'timeoutImplements'
          },
          hints: [
            'Use alarm() to set a timeout',
            'Handle the SIGALRM signal appropriately'
          ],
          score: 25
        },
        {
          id: 5,
          title: 'Verification',
          description: 'Test that the fix works',
          instructions: 'Run the fixed program and verify it completes without hanging.',
          expectedOutcome: 'Program completes within timeout period',
          validation: {
            type: 'behavior',
            check: 'programCompletes',
            timeout: 10
          },
          hints: [
            'The program should complete within the timeout period',
            'No hanging behavior should be observed'
          ],
          score: 30
        }
      ]
    },
    
    {
      id: 'memory-leak',
      name: 'Memory Leak Detection',
      category: 'intermediate',
      difficulty: 2,
      estimatedTime: 20,
      description: 'Detect and fix memory leaks using Valgrind and manual analysis',
      tags: ['memory', 'leak', 'valgrind', 'c'],
      learningObjectives: [
        'Use Valgrind for memory leak detection',
        'Identify common memory leak patterns',
        'Implement proper memory management',
        'Use memory debugging tools'
      ],
      prerequisites: ['process-hang'],
      steps: [
        {
          id: 1,
          title: 'Initial Memory Check',
          description: 'Measure initial memory usage',
          instructions: 'Check the current memory usage of the program.',
          expectedOutcome: 'Baseline memory measurements',
          validation: {
            type: 'command',
            command: 'ps aux | grep memory_leak_program',
            expectedPattern: 'memory_leak_program.*[0-9]+\\.[0-9]+.*[0-9]+\\.[0-9]+'
          },
          score: 10
        },
        {
          id: 2,
          title: 'Run with Valgrind',
          description: 'Use Valgrind to detect memory leaks',
          instructions: 'Run the program under Valgrind to get detailed leak information.',
          expectedOutcome: 'Valgrind report showing memory leaks',
          validation: {
            type: 'valgrind',
            command: 'valgrind --leak-check=full ./memory_leak_program',
            expectedPattern: 'definitely lost.*bytes in.*blocks'
          },
          hints: [
            'Valgrind runs programs slower but shows exact leak locations',
            'Look for "definitely lost" and "indirectly lost" blocks'
          ],
          score: 20
        },
        {
          id: 3,
          title: 'Analyze Leak Report',
          description: 'Understand the memory leak report',
          instructions: 'Examine the Valgrind output to understand where memory is being leaked.',
          expectedOutcome: 'Identification of leak sources',
          validation: {
            type: 'analysis',
            checkFunction: 'leakAnalysis'
          },
          hints: [
            'Stack traces show where leaked memory was allocated',
            'Look for malloc/free mismatches'
          ],
          score: 25
        },
        {
          id: 4,
          title: 'Fix Memory Leaks',
          description: 'Implement proper memory management',
          instructions: 'Add missing free() calls to fix the memory leaks.',
          expectedOutcome: 'All dynamically allocated memory properly freed',
          validation: {
            type: 'valgrind',
            command: 'valgrind --leak-check=full ./fixed_program',
            expectedPattern: 'All heap blocks were freed -- no leaks are possible'
          },
          hints: [
            'Every malloc should have a corresponding free()',
            'Consider using smart pointers or RAII patterns'
          ],
          score: 30
        }
      ]
    },
    
    {
      id: 'deadlock',
      name: 'Deadlock Analysis',
      category: 'advanced',
      difficulty: 3,
      estimatedTime: 25,
      description: 'Debug and resolve deadlock situations in multi-threaded programs',
      tags: ['deadlock', 'threads', 'mutex', 'race-condition'],
      learningObjectives: [
        'Identify deadlock conditions',
        'Analyze thread states and locks',
        'Implement deadlock prevention strategies',
        'Use thread debugging tools'
      ],
      prerequisites: ['memory-leak'],
      steps: [
        {
          id: 1,
          title: 'Reproduce Deadlock',
          description: 'Run the program and observe deadlock behavior',
          instructions: 'Execute the program and confirm it deadlocks.',
          expectedOutcome: 'Program hangs due to circular wait',
          validation: {
            type: 'behavior',
            check: 'programHangs',
            timeout: 10
          },
          hints: [
            'Deadlocks typically occur when multiple threads wait for each other',
            'The program may appear to be hung'
          ],
          score: 10
        },
        {
          id: 2,
          title: 'Analyze Thread States',
          description: 'Use GDB to examine thread states',
          instructions: 'Attach GDB and examine all thread states and lock acquisition.',
          expectedOutcome: 'Identification of threads waiting for locks',
          validation: {
            type: 'gdb',
            commands: ['info threads', 'thread apply all bt'],
            expectedPattern: 'pthread_mutex_lock.*resource.*deadlock_program.c'
          },
          hints: [
            'Look for threads in waiting states',
            'Check for pthread_mutex_lock calls in backtraces'
          ],
          score: 20
        },
        {
          id: 3,
          title: 'Identify Root Cause',
          description: 'Find the lock ordering issue',
          instructions: 'Analyze the backtraces to find the circular dependency.',
          expectedOutcome: 'Identification of inconsistent lock ordering',
          validation: {
            type: 'analysis',
            checkFunction: 'deadlockAnalysis'
          },
          hints: [
            'Look for threads acquiring locks in different orders',
            'The "Dining Philosophers" problem is a classic example'
          ],
          score: 25
        },
        {
          id: 4,
          title: 'Fix Deadlock',
          description: 'Implement consistent lock ordering',
          instructions: 'Fix the lock ordering or add timeout mechanisms.',
          expectedOutcome: 'No deadlock occurs',
          validation: {
            type: 'behavior',
            check: 'programCompletes',
            timeout: 5
          },
          hints: [
            'Ensure all threads acquire locks in the same order',
            'Consider using pthread_mutex_timedlock()'
          ],
          score: 30
        }
      ]
    }
  ];
  
  scenarios.forEach(scenario => {
    scenarioLibrary.set(scenario.id, scenario);
  });
}

function evaluateStepCompletion(step, action, result) {
  // Simple step completion evaluation
  // In a real implementation, this would be more sophisticated
  return result && (result.success === true || result.correct === true);
}

function calculateStepScore(step, result) {
  let score = step.score || 10;
  
  // Bonus for hints not used
  if (!result.hintsUsed || result.hintsUsed.length === 0) {
    score = Math.round(score * 1.1);
  }
  
  // Penalty for multiple attempts
  if (result.attempts && result.attempts > 1) {
    score = Math.round(score * Math.max(0.5, 1 - (result.attempts - 1) * 0.1));
  }
  
  return score;
}

function getNextStep(scenario, currentStep) {
  return scenario.steps.find(step => step.id === currentStep + 1) || null;
}

function calculateAverageScore(sessions) {
  if (sessions.length === 0) return 0;
  
  const scores = sessions
    .filter(s => s.status === 'completed' && s.score != null)
    .map(s => s.score);
  
  return scores.length > 0 ? Math.round(scores.reduce((sum, score) => sum + score, 0) / scores.length) : 0;
}

function calculateAverageTime(sessions) {
  if (sessions.length === 0) return 0;
  
  const times = sessions
    .filter(s => s.status === 'completed' && s.timeSpent != null)
    .map(s => s.timeSpent);
  
  return times.length > 0 ? Math.round(times.reduce((sum, time) => sum + time, 0) / times.length) : 0;
}

function calculateCompletionRate(sessions) {
  if (sessions.length === 0) return 0;
  
  const completed = sessions.filter(s => s.status === 'completed').length;
  return Math.round((completed / sessions.length) * 100);
}

function getHangProgramSource() {
  return `#include <stdio.h>
#include <unistd.h>
#include <signal.h>

void timeout_handler(int sig) {
    printf("Timeout occurred!\\n");
    exit(1);
}

int main() {
    signal(SIGALRM, timeout_handler);
    alarm(3);
    
    printf("Starting program...\\n");
    printf("About to hang...\\n");
    while(1) {
        sleep(1);
    }
    
    printf("This should never be reached\\n");
    return 0;
}`;
}

module.exports = router;