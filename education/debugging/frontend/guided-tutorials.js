/**
 * Guided Tutorials Module
 * Provides interactive step-by-step debugging tutorials and learning paths
 */

class GuidedTutorials {
    constructor() {
        this.app = null;
        this.isEnabled = true;
        this.currentTutorial = null;
        this.currentStep = 0;
        this.tutorialProgress = new Map();
        this.userInteractions = [];
        this.hints = [];
        this.milestones = [];
        
        // Tutorial data structure
        this.tutorials = new Map();
        this.activeSessions = new Map();
        
        // UI elements for guided mode
        this.guidedOverlay = null;
        this.stepIndicator = null;
        this.hintBox = null;
        this.progressBar = null;
        
        // Keyboard shortcuts for tutorials
        this.tutorialShortcuts = {
            'next': ['Enter', 'Space', 'N'],
            'previous': ['Backspace', 'P'],
            'skip': ['Escape', 'S'],
            'hint': ['H'],
            'help': ['?']
        };
    }

    setApp(app) {
        this.app = app;
        this.initializeGuidedInterface();
        this.loadTutorials();
    }

    initializeGuidedInterface() {
        // Create guided tutorial overlay
        this.guidedOverlay = document.createElement('div');
        this.guidedOverlay.className = 'guided-overlay';
        this.guidedOverlay.innerHTML = `
            <div class="guided-tutorial">
                <div class="guided-header">
                    <h3 class="guided-title"></h3>
                    <div class="guided-progress">
                        <div class="progress-track">
                            <div class="progress-fill"></div>
                        </div>
                        <span class="step-counter">Step 1/10</span>
                    </div>
                    <button class="guided-close">×</button>
                </div>
                <div class="guided-content">
                    <div class="guided-instructions">
                        <h4>Instructions</h4>
                        <div class="instruction-text"></div>
                        <div class="interactive-elements"></div>
                    </div>
                    <div class="guided-sidebar">
                        <div class="hints-panel">
                            <h5>Hints</h5>
                            <div class="hints-list"></div>
                            <button class="btn btn-sm btn-info show-hint-btn">Show Hint</button>
                        </div>
                        <div class="tips-panel">
                            <h5>💡 Tips</h5>
                            <div class="tips-content"></div>
                        </div>
                    </div>
                </div>
                <div class="guided-footer">
                    <button class="btn btn-secondary previous-step">Previous</button>
                    <button class="btn btn-primary next-step">Next</button>
                    <button class="btn btn-warning skip-step">Skip</button>
                    <button class="btn btn-danger exit-tutorial">Exit Tutorial</button>
                </div>
            </div>
        `;
        
        document.body.appendChild(this.guidedOverlay);
        
        // Initialize event listeners
        this.initializeGuidedEvents();
        
        // Hide by default
        this.guidedOverlay.style.display = 'none';
    }

    initializeGuidedEvents() {
        const closeBtn = this.guidedOverlay.querySelector('.guided-close');
        const nextBtn = this.guidedOverlay.querySelector('.next-step');
        const prevBtn = this.guidedOverlay.querySelector('.previous-step');
        const skipBtn = this.guidedOverlay.querySelector('.skip-step');
        const exitBtn = this.guidedOverlay.querySelector('.exit-tutorial');
        const hintBtn = this.guidedOverlay.querySelector('.show-hint-btn');
        
        closeBtn.addEventListener('click', () => this.hideTutorial());
        nextBtn.addEventListener('click', () => this.nextStep());
        prevBtn.addEventListener('click', () => this.previousStep());
        skipBtn.addEventListener('click', () => this.skipStep());
        exitBtn.addEventListener('click', () => this.exitTutorial());
        hintBtn.addEventListener('click', () => this.showHint());
        
        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => this.handleTutorialKeydown(e));
    }

    loadTutorials() {
        // Define built-in tutorials
        this.defineBuiltinTutorials();
    }

    defineBuiltinTutorials() {
        // Process Hang Investigation Tutorial
        this.addTutorial({
            id: 'process-hang',
            name: 'Process Hang Investigation',
            description: 'Learn to debug and resolve process hangs',
            difficulty: 'beginner',
            estimatedTime: 15,
            prerequisites: [],
            steps: [
                {
                    id: 1,
                    title: 'Initial Assessment',
                    description: 'Identify a hanging process',
                    instructions: 'We have a program that appears to be hanging. Let\'s start by identifying which process is not responding.',
                    actions: [
                        {
                            type: 'terminal',
                            command: 'ps aux | grep -E "(hung|sleep|loop)"',
                            expectedOutput: 'Should show processes with high CPU or in sleep state'
                        }
                    ],
                    hints: [
                        'Use `ps aux` to see all running processes',
                        'Look for processes with high CPU usage or in uninterruptible sleep (D state)',
                        'The grep pattern helps identify potentially problematic processes'
                    ],
                    interactiveElements: [
                        {
                            type: 'terminal',
                            placeholder: 'Enter command to check processes...'
                        }
                    ]
                },
                {
                    id: 2,
                    title: 'Examine Process Details',
                    description: 'Get detailed information about the hanging process',
                    instructions: 'Now let\'s examine the hanging process in detail to understand what might be causing the hang.',
                    actions: [
                        {
                            type: 'gdb',
                            command: 'attach <PID>',
                            expectedOutput: 'Should attach to the process'
                        },
                        {
                            type: 'gdb',
                            command: 'info threads',
                            expectedOutput: 'Show all threads in the process'
                        }
                    ],
                    hints: [
                        'Use the PID from the previous step',
                        'Thread information can reveal which thread is stuck',
                        'Look for threads in specific states (running, waiting, etc.)'
                    ]
                },
                {
                    id: 3,
                    title: 'Analyze Call Stack',
                    description: 'Examine the call stack to find where the process is stuck',
                    instructions: 'Let\'s look at the call stack to see exactly where each thread is stuck.',
                    actions: [
                        {
                            type: 'gdb',
                            command: 'thread apply all bt',
                            expectedOutput: 'Show backtrace for all threads'
                        }
                    ],
                    hints: [
                        'The backtrace shows the function call sequence',
                        'Look for system calls or library functions that might be blocking',
                        'The most recent calls are at the top of each stack'
                    ],
                    expectedOutcome: 'Identify the specific function or system call causing the hang'
                },
                {
                    id: 4,
                    title: 'Examine Variables',
                    description: 'Check relevant variables that might explain the hang',
                    instructions: 'Let\'s examine some key variables to understand the program state.',
                    actions: [
                        {
                            type: 'gdb',
                            command: 'print *variable_name',
                            expectedOutput: 'Show the value and type of the variable'
                        }
                    ],
                    hints: [
                        'Look for variables that might be in unexpected states',
                        'Check for null pointers or invalid values',
                        'Examine loop conditions and counters'
                    ]
                },
                {
                    id: 5,
                    title: 'System Resources',
                    description: 'Check system resources that might be exhausted',
                    instructions: 'Sometimes hangs are caused by exhausted system resources. Let\'s check.',
                    actions: [
                        {
                            type: 'terminal',
                            command: 'lsof -p <PID>',
                            expectedOutput: 'List open files for the process'
                        },
                        {
                            type: 'terminal',
                            command: 'ulimit -a',
                            expectedOutput: 'Show resource limits'
                        }
                    ],
                    hints: [
                        'Too many open files can cause hangs',
                        'Check for file descriptor exhaustion',
                        'Memory limits can also cause unexpected behavior'
                    ]
                },
                {
                    id: 6,
                    title: 'Resolution Strategy',
                    description: 'Develop a strategy to resolve the hang',
                    instructions: 'Based on our analysis, let\'s develop a solution.',
                    actions: [
                        {
                            type: 'interactive',
                            question: 'What would be your approach to fix this hang?',
                            options: [
                                'Kill and restart the process',
                                'Modify the code and recompile',
                                'Adjust system limits',
                                'Use timeout mechanisms'
                            ]
                        }
                    ],
                    hints: [
                        'Consider both immediate and long-term solutions',
                        'Prevention is better than cure',
                        'Some issues require code changes, others system configuration'
                    ]
                },
                {
                    id: 7,
                    title: 'Implementation',
                    description: 'Implement the chosen solution',
                    instructions: 'Now let\'s implement our chosen solution.',
                    actions: [
                        {
                            type: 'code',
                            action: 'modify',
                            description: 'Add timeout mechanism to prevent future hangs'
                        }
                    ],
                    expectedOutcome: 'Successfully implement a fix for the hang'
                },
                {
                    id: 8,
                    title: 'Verification',
                    description: 'Test that our fix works',
                    instructions: 'Let\'s verify that our solution resolves the hang.',
                    actions: [
                        {
                            type: 'terminal',
                            command: './fixed_program',
                            expectedOutput: 'Program should run without hanging'
                        }
                    ],
                    hints: [
                        'Test under various conditions',
                        'Monitor the program to ensure it completes',
                        'Check logs for any remaining issues'
                    ]
                },
                {
                    id: 9,
                    title: 'Prevention',
                    description: 'Implement monitoring and prevention measures',
                    instructions: 'Let\'s set up monitoring to prevent future issues.',
                    actions: [
                        {
                            type: 'terminal',
                            command: 'crontab -e',
                            expectedOutput: 'Add monitoring script to crontab'
                        }
                    ],
                    hints: [
                        'Automated monitoring can catch issues early',
                        'Regular health checks are important',
                        'Consider alerting mechanisms'
                    ]
                },
                {
                    id: 10,
                    title: 'Documentation',
                    description: 'Document what we learned',
                    instructions: 'Document the issue, analysis, and solution for future reference.',
                    actions: [
                        {
                            type: 'interactive',
                            question: 'Create a brief summary of the debugging process:',
                            placeholder: 'Describe the issue, how you identified it, and the solution...'
                        }
                    ],
                    expectedOutcome: 'Complete documentation of the debugging process'
                }
            ]
        });

        // Memory Leak Detection Tutorial
        this.addTutorial({
            id: 'memory-leak',
            name: 'Memory Leak Detection',
            description: 'Learn to identify and fix memory leaks',
            difficulty: 'intermediate',
            estimatedTime: 20,
            prerequisites: ['basic-gdb', 'process-hang'],
            steps: [
                {
                    id: 1,
                    title: 'Initial Memory Check',
                    description: 'Start with a baseline memory measurement',
                    instructions: 'Let\'s begin by measuring the initial memory usage of our program.',
                    actions: [
                        {
                            type: 'terminal',
                            command: 'ps aux | grep program_name',
                            expectedOutput: 'Show initial memory usage'
                        }
                    ],
                    hints: [
                        'Baseline measurements are crucial for comparison',
                        'Use RSS (Resident Set Size) for actual memory usage',
                        'VIRT shows virtual memory which may be different'
                    ]
                },
                {
                    id: 2,
                    title: 'Run with Valgrind',
                    description: 'Use Valgrind to detect memory leaks',
                    instructions: 'Valgrind is an excellent tool for detecting memory leaks. Let\'s use it.',
                    actions: [
                        {
                            type: 'terminal',
                            command: 'valgrind --leak-check=full ./program_name',
                            expectedOutput: 'Detailed memory leak report'
                        }
                    ],
                    hints: [
                        'Valgrind runs programs slower but shows exact leak locations',
                        'Look for "definitely lost" and "indirectly lost" blocks',
                        'The stack trace shows where leaked memory was allocated'
                    ]
                },
                {
                    id: 3,
                    title: 'Analyze Leak Report',
                    description: 'Understand the memory leak report',
                    instructions: 'Let\'s examine the Valgrind output to understand the leaks.',
                    actions: [
                        {
                            type: 'interactive',
                            question: 'Identify the main source of memory leaks:',
                            options: [
                                'Missing free() calls',
                                'Incorrect pointer handling',
                                'Memory not released on error paths',
                                'All of the above'
                            ]
                        }
                    ],
                    hints: [
                        'Memory leaks often occur in error handling paths',
                        'Look for malloc/free mismatches',
                        'Consider smart pointers or RAII patterns'
                    ]
                },
                {
                    id: 4,
                    title: 'Fix Memory Leaks',
                    description: 'Implement fixes for the identified leaks',
                    instructions: 'Now let\'s fix the memory leaks we identified.',
                    actions: [
                        {
                            type: 'code',
                            action: 'edit',
                            description: 'Add missing free() calls and fix pointer handling'
                        }
                    ],
                    hints: [
                        'Always free memory in the opposite order of allocation',
                        'Check all code paths, not just the success case',
                        'Consider using memory management libraries'
                    ]
                },
                {
                    id: 5,
                    title: 'Verify Fixes',
                    description: 'Test that our fixes work',
                    instructions: 'Let\'s verify that our fixes eliminated the memory leaks.',
                    actions: [
                        {
                            type: 'terminal',
                            command: 'valgrind --leak-check=full ./fixed_program',
                            expectedOutput: 'No memory leaks reported'
                        }
                    ],
                    expectedOutcome: 'Clean Valgrind report with no memory leaks'
                }
            ]
        });

        // Deadlock Scenario Tutorial
        this.addTutorial({
            id: 'deadlock',
            name: 'Deadlock Scenario',
            description: 'Learn to identify and resolve deadlocks',
            difficulty: 'advanced',
            estimatedTime: 25,
            prerequisites: ['memory-leak'],
            steps: [
                {
                    id: 1,
                    title: 'Reproduce Deadlock',
                    description: 'Create a scenario that triggers a deadlock',
                    instructions: 'Let\'s run the program that exhibits deadlock behavior.',
                    actions: [
                        {
                            type: 'terminal',
                            command: './deadlock_program',
                            expectedOutput: 'Program hangs without producing output'
                        }
                    ],
                    hints: [
                        'Deadlocks typically occur when multiple threads wait for each other',
                        'Look for circular wait conditions',
                        'The program may appear to be hung'
                    ]
                },
                {
                    id: 2,
                    title: 'Attach Debugger',
                    description: 'Attach GDB to the hanging process',
                    instructions: 'Let\'s attach GDB to examine the state of our deadlocked program.',
                    actions: [
                        {
                            type: 'gdb',
                            command: 'attach <PID>',
                            expectedOutput: 'Successfully attached to process'
                        }
                    ]
                },
                {
                    id: 3,
                    title: 'Thread Analysis',
                    description: 'Examine all threads and their states',
                    instructions: 'Let\'s see what all threads are doing.',
                    actions: [
                        {
                            type: 'gdb',
                            command: 'info threads',
                            expectedOutput: 'List all threads and their states'
                        },
                        {
                            type: 'gdb',
                            command: 'thread apply all bt',
                            expectedOutput: 'Backtrace for each thread'
                        }
                    ],
                    hints: [
                        'Deadlocked threads will often be waiting on locks',
                        'Look for threads in similar call stacks',
                        'Check for pthread_mutex_lock calls in backtraces'
                    ]
                },
                {
                    id: 4,
                    title: 'Identify Lock Order Violation',
                    description: 'Analyze the backtraces to find the lock order issue',
                    instructions: 'Deadlocks often occur due to inconsistent lock acquisition order.',
                    actions: [
                        {
                            type: 'interactive',
                            question: 'What pattern do you see in the backtraces?',
                            options: [
                                'Threads acquiring locks in different orders',
                                'Circular dependency between resources',
                                'Missing unlock calls',
                                'All of the above'
                            ]
                        }
                    ],
                    hints: [
                        'The "Dining Philosophers" problem is a classic deadlock example',
                        'Look for lock A then lock B vs lock B then lock A',
                        'Consistent lock ordering prevents deadlocks'
                    ]
                },
                {
                    id: 5,
                    title: 'Fix Lock Ordering',
                    description: 'Implement consistent lock ordering',
                    instructions: 'Let\'s fix the lock ordering issue.',
                    actions: [
                        {
                            type: 'code',
                            action: 'modify',
                            description: 'Ensure all threads acquire locks in the same order'
                        }
                    ],
                    expectedOutcome: 'Consistent lock acquisition order'
                },
                {
                    id: 6,
                    title: 'Alternative: Lock Timeout',
                    description: 'Implement timeout-based deadlock prevention',
                    instructions: 'As an alternative, we can implement timeout-based lock acquisition.',
                    actions: [
                        {
                            type: 'code',
                            action: 'add',
                            description: 'Use pthread_mutex_timedlock() with timeout'
                        }
                    ],
                    hints: [
                        'Timeout allows threads to back off and retry',
                        'Prevents indefinite waiting',
                        'Can be combined with lock ordering'
                    ]
                },
                {
                    id: 7,
                    title: 'Verification',
                    description: 'Test the deadlock fix',
                    instructions: 'Let\'s verify that our fixes resolve the deadlock.',
                    actions: [
                        {
                            type: 'terminal',
                            command: './fixed_program',
                            expectedOutput: 'Program completes successfully'
                        }
                    ],
                    expectedOutcome: 'Program runs without deadlocking'
                }
            ]
        });
    }

    addTutorial(tutorialData) {
        this.tutorials.set(tutorialData.id, tutorialData);
    }

    async startScenario(scenario) {
        if (!this.isEnabled) {
            return;
        }

        try {
            this.currentTutorial = scenario;
            this.currentStep = 0;
            this.userInteractions = [];
            this.hints = [];
            
            // Initialize progress tracking
            this.tutorialProgress.set(scenario.id, {
                started: new Date(),
                completedSteps: [],
                currentStep: 0,
                interactions: [],
                hintsUsed: []
            });
            
            this.showTutorial();
            this.renderCurrentStep();
            
            if (this.app) {
                this.app.showNotification(`Started tutorial: ${scenario.name}`, 'info');
            }
            
            console.log(`🎓 Started tutorial: ${scenario.name}`);
        } catch (error) {
            if (this.app) {
                this.app.showNotification(`Failed to start tutorial: ${error.message}`, 'error');
            }
        }
    }

    showTutorial() {
        if (this.guidedOverlay) {
            this.guidedOverlay.style.display = 'block';
            this.guidedOverlay.classList.add('visible');
        }
    }

    hideTutorial() {
        if (this.guidedOverlay) {
            this.guidedOverlay.style.display = 'none';
            this.guidedOverlay.classList.remove('visible');
        }
    }

    renderCurrentStep() {
        if (!this.currentTutorial || !this.guidedOverlay) {
            return;
        }

        const step = this.currentTutorial.steps[this.currentStep];
        if (!step) {
            this.completeTutorial();
            return;
        }

        // Update tutorial header
        const titleEl = this.guidedOverlay.querySelector('.guided-title');
        const instructionTextEl = this.guidedOverlay.querySelector('.instruction-text');
        const progressFillEl = this.guidedOverlay.querySelector('.progress-fill');
        const stepCounterEl = this.guidedOverlay.querySelector('.step-counter');
        
        titleEl.textContent = step.title;
        instructionTextEl.textContent = step.description;
        
        // Update progress
        const progress = ((this.currentStep + 1) / this.currentTutorial.steps.length) * 100;
        progressFillEl.style.width = `${progress}%`;
        stepCounterEl.textContent = `Step ${this.currentStep + 1}/${this.currentTutorial.steps.length}`;
        
        // Render interactive elements
        this.renderInteractiveElements(step.interactiveElements || []);
        
        // Render hints
        this.renderHints(step.hints || []);
        
        // Update navigation buttons
        this.updateNavigationButtons();
    }

    renderInteractiveElements(elements) {
        const container = this.guidedOverlay.querySelector('.interactive-elements');
        if (!container) return;
        
        container.innerHTML = elements.map((element, index) => {
            switch (element.type) {
                case 'terminal':
                    return this.createTerminalElement(element, index);
                case 'gdb':
                    return this.createGDBElement(element, index);
                case 'code':
                    return this.createCodeElement(element, index);
                case 'interactive':
                    return this.createInteractiveElement(element, index);
                default:
                    return `<div>Unknown element type: ${element.type}</div>`;
            }
        }).join('');
        
        // Add event listeners
        this.addElementEventListeners();
    }

    createTerminalElement(element, index) {
        return `
            <div class="terminal-guidance">
                <label>Terminal Command:</label>
                <div class="command-input-group">
                    <span class="prompt">$</span>
                    <input type="text" class="terminal-command" data-index="${index}" placeholder="${element.placeholder || 'Enter command...'}">
                    <button class="btn btn-sm btn-primary execute-command">Execute</button>
                </div>
                <div class="expected-output" style="display: none;">
                    <strong>Expected Output:</strong> ${element.expectedOutput || 'Command should execute successfully'}
                </div>
                <div class="command-output" style="display: none;"></div>
            </div>
        `;
    }

    createGDBElement(element, index) {
        return `
            <div class="gdb-guidance">
                <label>GDB Command:</label>
                <div class="command-input-group">
                    <span class="gdb-prompt">(gdb)</span>
                    <input type="text" class="gdb-command" data-index="${index}" placeholder="${element.placeholder || 'Enter GDB command...'}">
                    <button class="btn btn-sm btn-primary execute-gdb-command">Execute</button>
                </div>
                <div class="expected-output" style="display: none;">
                    <strong>Expected Output:</strong> ${element.expectedOutput || 'GDB command should execute successfully'}
                </div>
                <div class="gdb-output" style="display: none;"></div>
            </div>
        `;
    }

    createCodeElement(element, index) {
        return `
            <div class="code-guidance">
                <div class="code-action">
                    <strong>Code Action:</strong> ${element.description}
                </div>
                <div class="code-editor">
                    <textarea class="code-input" data-index="${index}" placeholder="Enter or modify code here..."></textarea>
                    <button class="btn btn-sm btn-secondary validate-code">Validate</button>
                </div>
            </div>
        `;
    }

    createInteractiveElement(element, index) {
        if (element.options) {
            return `
                <div class="interactive-guidance">
                    <label>${element.question}</label>
                    <div class="options-group">
                        ${element.options.map((option, optIndex) => `
                            <label class="option-label">
                                <input type="radio" name="interactive_${index}" value="${option}">
                                <span class="option-text">${option}</span>
                            </label>
                        `).join('')}
                    </div>
                    <button class="btn btn-sm btn-primary check-answer">Check Answer</button>
                </div>
            `;
        } else {
            return `
                <div class="interactive-guidance">
                    <label>${element.question}</label>
                    <textarea class="interactive-input" data-index="${index}" placeholder="${element.placeholder || 'Enter your response...'}"></textarea>
                    <button class="btn btn-sm btn-primary submit-response">Submit</button>
                </div>
            `;
        }
    }

    renderHints(hints) {
        const hintsList = this.guidedOverlay.querySelector('.hints-list');
        if (!hintsList) return;
        
        hintsList.innerHTML = hints.map((hint, index) => `
            <div class="hint-item" data-hint-index="${index}">
                <div class="hint-header">
                    <span class="hint-icon">💡</span>
                    <span class="hint-number">${index + 1}</span>
                </div>
                <div class="hint-content">${hint}</div>
            </div>
        `).join('');
    }

    updateNavigationButtons() {
        const prevBtn = this.guidedOverlay.querySelector('.previous-step');
        const nextBtn = this.guidedOverlay.querySelector('.next-step');
        
        // Update button states
        prevBtn.disabled = this.currentStep === 0;
        nextBtn.disabled = this.currentStep === this.currentTutorial.steps.length - 1;
        
        // Update button text
        if (this.currentStep === this.currentTutorial.steps.length - 1) {
            nextBtn.textContent = 'Complete';
        } else {
            nextBtn.textContent = 'Next';
        }
    }

    addElementEventListeners() {
        // Terminal commands
        document.querySelectorAll('.execute-command').forEach(btn => {
            btn.addEventListener('click', (e) => this.executeGuidedCommand(e.target));
        });
        
        // GDB commands
        document.querySelectorAll('.execute-gdb-command').forEach(btn => {
            btn.addEventListener('click', (e) => this.executeGuidedGDB(e.target));
        });
        
        // Code validation
        document.querySelectorAll('.validate-code').forEach(btn => {
            btn.addEventListener('click', (e) => this.validateCode(e.target));
        });
        
        // Interactive elements
        document.querySelectorAll('.check-answer').forEach(btn => {
            btn.addEventListener('click', (e) => this.checkAnswer(e.target));
        });
        
        document.querySelectorAll('.submit-response').forEach(btn => {
            btn.addEventListener('click', (e) => this.submitResponse(e.target));
        });
    }

    async executeGuidedCommand(button) {
        const inputGroup = button.closest('.command-input-group');
        const commandInput = inputGroup.querySelector('.terminal-command');
        const outputDiv = inputGroup.nextElementSibling.nextElementSibling;
        
        const command = commandInput.value.trim();
        if (!command) {
            return;
        }
        
        try {
            // Execute command
            const output = await this.executeCommand(command);
            
            // Show output
            outputDiv.style.display = 'block';
            outputDiv.innerHTML = `<strong>Output:</strong><pre>${output}</pre>`;
            
            // Record interaction
            this.recordInteraction('terminal', { command, output });
            
        } catch (error) {
            outputDiv.style.display = 'block';
            outputDiv.innerHTML = `<strong>Error:</strong> ${error.message}`;
        }
    }

    async executeGuidedGDB(button) {
        const inputGroup = button.closest('.command-input-group');
        const commandInput = inputGroup.querySelector('.gdb-command');
        const outputDiv = inputGroup.nextElementSibling.nextElementSibling;
        
        const command = commandInput.value.trim();
        if (!command) {
            return;
        }
        
        try {
            // Execute GDB command
            const output = await this.executeGDBCommand(command);
            
            // Show output
            outputDiv.style.display = 'block';
            outputDiv.innerHTML = `<strong>GDB Output:</strong><pre>${output}</pre>`;
            
            // Record interaction
            this.recordInteraction('gdb', { command, output });
            
        } catch (error) {
            outputDiv.style.display = 'block';
            outputDiv.innerHTML = `<strong>Error:</strong> ${error.message}`;
        }
    }

    validateCode(button) {
        const textarea = button.closest('.code-editor').querySelector('.code-input');
        const code = textarea.value.trim();
        
        if (!code) {
            this.showValidationResult(button, 'Please enter some code', 'error');
            return;
        }
        
        // Simulate code validation
        const isValid = this.mockValidateCode(code);
        
        if (isValid) {
            this.showValidationResult(button, 'Code looks good!', 'success');
            this.recordInteraction('code', { code, valid: true });
        } else {
            this.showValidationResult(button, 'Code may have issues. Please review.', 'warning');
            this.recordInteraction('code', { code, valid: false });
        }
    }

    checkAnswer(button) {
        const guidance = button.closest('.interactive-guidance');
        const selectedOption = guidance.querySelector('input[type="radio"]:checked');
        
        if (!selectedOption) {
            this.showValidationResult(button, 'Please select an option', 'warning');
            return;
        }
        
        const answer = selectedOption.value;
        const isCorrect = this.mockCheckAnswer(answer);
        
        if (isCorrect) {
            this.showValidationResult(button, 'Correct! Well done.', 'success');
            this.recordInteraction('interactive', { answer, correct: true });
        } else {
            this.showValidationResult(button, 'Not quite right. Try again or use a hint.', 'error');
            this.recordInteraction('interactive', { answer, correct: false });
        }
    }

    submitResponse(button) {
        const textarea = button.closest('.interactive-guidance').querySelector('.interactive-input');
        const response = textarea.value.trim();
        
        if (!response) {
            this.showValidationResult(button, 'Please enter a response', 'warning');
            return;
        }
        
        this.showValidationResult(button, 'Response recorded.', 'success');
        this.recordInteraction('interactive', { response, type: 'text' });
    }

    showValidationResult(button, message, type) {
        const guidance = button.closest('.interactive-guidance');
        let resultDiv = guidance.querySelector('.validation-result');
        
        if (!resultDiv) {
            resultDiv = document.createElement('div');
            resultDiv.className = 'validation-result';
            guidance.appendChild(resultDiv);
        }
        
        resultDiv.className = `validation-result ${type}`;
        resultDiv.textContent = message;
        resultDiv.style.display = 'block';
        
        setTimeout(() => {
            resultDiv.style.display = 'none';
        }, 3000);
    }

    nextStep() {
        if (this.currentStep < this.currentTutorial.steps.length - 1) {
            this.currentStep++;
            this.renderCurrentStep();
            this.updateProgress();
        } else {
            this.completeTutorial();
        }
    }

    previousStep() {
        if (this.currentStep > 0) {
            this.currentStep--;
            this.renderCurrentStep();
        }
    }

    skipStep() {
        if (confirm('Are you sure you want to skip this step?')) {
            this.nextStep();
        }
    }

    exitTutorial() {
        if (confirm('Are you sure you want to exit the tutorial? Progress will be lost.')) {
            this.hideTutorial();
            this.currentTutorial = null;
            this.currentStep = 0;
        }
    }

    showHint() {
        const step = this.currentTutorial.steps[this.currentStep];
        const hints = step.hints || [];
        
        if (hints.length === 0) {
            return;
        }
        
        const hintIndex = this.hints.length % hints.length;
        const hint = hints[hintIndex];
        
        // Add hint to history
        this.hints.push({
            step: this.currentStep,
            hint: hint,
            timestamp: new Date()
        });
        
        // Show hint in main hints panel if available
        if (this.app && this.app.addHint) {
            this.app.addHint(hint);
        }
        
        // Update progress tracking
        const progress = this.tutorialProgress.get(this.currentTutorial.id);
        if (progress) {
            progress.hintsUsed.push(hint);
        }
        
        this.app.showNotification('Hint added to hints panel', 'info');
    }

    completeTutorial() {
        const progress = this.tutorialProgress.get(this.currentTutorial.id);
        if (progress) {
            progress.completed = new Date();
        }
        
        this.hideTutorial();
        
        if (this.app) {
            this.app.showNotification(`Tutorial completed: ${this.currentTutorial.name}`, 'success');
        }
        
        console.log(`✅ Completed tutorial: ${this.currentTutorial.name}`);
    }

    recordInteraction(type, data) {
        const interaction = {
            type,
            step: this.currentStep,
            timestamp: new Date(),
            data
        };
        
        this.userInteractions.push(interaction);
        
        // Update progress tracking
        const progress = this.tutorialProgress.get(this.currentTutorial.id);
        if (progress) {
            progress.interactions.push(interaction);
        }
    }

    updateProgress() {
        const progress = this.tutorialProgress.get(this.currentTutorial.id);
        if (progress) {
            progress.currentStep = this.currentStep;
            progress.completedSteps.push(this.currentStep);
        }
    }

    executeCommand(command) {
        return new Promise((resolve, reject) => {
            // Mock command execution
            setTimeout(() => {
                resolve(`Mock output for: ${command}`);
            }, 500 + Math.random() * 1000);
        });
    }

    executeGDBCommand(command) {
        return new Promise((resolve, reject) => {
            // Mock GDB command execution
            setTimeout(() => {
                const mockOutputs = {
                    'info threads': '  Id   Target Id         Frame \n* 1    Thread 0x7f8c (LWP 1234) main () at main.c:10',
                    'backtrace': '#0  main () at main.c:10\n#1  0x00007ffff7a2e1e1 in __libc_start_main ()',
                    'list': '5\tint main() {\n6\t    int x = 10;\n7\t    printf("Hello World\\n");\n8\t    return 0;\n9\t}',
                    'next': '7\t    printf("Hello World\\n");'
                };
                
                const output = mockOutputs[command] || `Mock GDB output for: ${command}`;
                resolve(output);
            }, 300 + Math.random() * 700);
        });
    }

    mockValidateCode(code) {
        // Simple validation - check for common patterns
        const hasBasicSyntax = code.includes('{') && code.includes('}');
        const hasStatements = code.includes(';') || code.includes('\n');
        return hasBasicSyntax && hasStatements;
    }

    mockCheckAnswer(answer) {
        // Mock answer checking - in reality this would be more sophisticated
        const correctAnswers = ['All of the above', 'All threads acquiring locks in the same order'];
        return correctAnswers.includes(answer);
    }

    handleTutorialKeydown(event) {
        if (!this.guidedOverlay || this.guidedOverlay.style.display === 'none') {
            return;
        }
        
        const key = event.key;
        const shortcuts = this.tutorialShortcuts;
        
        if (shortcuts.next.includes(key)) {
            event.preventDefault();
            this.nextStep();
        } else if (shortcuts.previous.includes(key)) {
            event.preventDefault();
            this.previousStep();
        } else if (shortcuts.skip.includes(key)) {
            event.preventDefault();
            this.skipStep();
        } else if (shortcuts.hint.includes(key)) {
            event.preventDefault();
            this.showHint();
        }
    }

    getTutorialProgress(tutorialId) {
        return this.tutorialProgress.get(tutorialId);
    }

    getAllProgress() {
        return Object.fromEntries(this.tutorialProgress);
    }

    setEnabled(enabled) {
        this.isEnabled = enabled;
    }

    getTutorials() {
        return Array.from(this.tutorials.values());
    }

    getTutorial(id) {
        return this.tutorials.get(id);
    }

    getCurrentTutorial() {
        return this.currentTutorial;
    }

    getCurrentStep() {
        return this.currentStep;
    }

    exportTutorialData() {
        return {
            progress: this.getAllProgress(),
            interactions: this.userInteractions,
            hints: this.hints
        };
    }

    importTutorialData(data) {
        try {
            // Import progress data
            Object.entries(data.progress || {}).forEach(([id, progress]) => {
                this.tutorialProgress.set(id, progress);
            });
            
            this.userInteractions = data.interactions || [];
            this.hints = data.hints || [];
            
            if (this.app) {
                this.app.showNotification('Tutorial data imported', 'success');
            }
        } catch (error) {
            if (this.app) {
                this.app.showNotification(`Failed to import tutorial data: ${error.message}`, 'error');
            }
        }
    }

    destroy() {
        if (this.guidedOverlay && this.guidedOverlay.parentNode) {
            this.guidedOverlay.parentNode.removeChild(this.guidedOverlay);
        }
        
        this.tutorials.clear();
        this.tutorialProgress.clear();
        this.activeSessions.clear();
        this.userInteractions = [];
        this.hints = [];
    }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = GuidedTutorials;
}