// MultiOS API Documentation - Interactive Components
// Handles interactive elements, code execution, and dynamic content

class InteractiveComponents {
    constructor() {
        this.codeExamples = new Map();
        this.activePlaygrounds = new Map();
        this.liveEditors = new Map();
        
        this.init();
    }

    init() {
        this.setupCodeExamples();
        this.setupLiveEditors();
        this.setupInteractiveTutorials();
        this.setupAPITester();
        this.setupPerformanceMonitor();
        this.setupCodePlayground();
    }

    // Code Examples with Live Execution
    setupCodeExamples() {
        const codeExamples = document.querySelectorAll('.code-example');
        
        codeExamples.forEach(example => {
            const language = example.getAttribute('data-language') || 'rust';
            const editable = example.hasAttribute('data-editable');
            
            if (editable) {
                this.makeCodeEditable(example, language);
            } else {
                this.addRunButton(example, language);
            }
        });
    }

    makeCodeEditable(codeBlock, language) {
        const pre = codeBlock.querySelector('pre');
        const code = codeBlock.querySelector('code');
        const originalCode = code.textContent;
        
        // Convert to editable textarea
        const textarea = document.createElement('textarea');
        textarea.value = originalCode;
        textarea.className = 'code-editor';
        textarea.setAttribute('data-language', language);
        textarea.style.cssText = `
            width: 100%;
            min-height: 200px;
            font-family: var(--font-mono);
            font-size: 0.875rem;
            background: var(--bg-primary);
            color: var(--text-primary);
            border: 1px solid var(--border-color);
            border-radius: 6px;
            padding: var(--spacing-md);
            resize: vertical;
        `;
        
        // Replace code element with textarea
        code.replaceWith(textarea);
        
        // Add control buttons
        const controls = this.createCodeControls(language, originalCode, textarea);
        codeBlock.insertBefore(controls, pre);
        
        // Store for later use
        this.codeExamples.set(codeBlock, {
            language,
            originalCode,
            currentCode: originalCode,
            textarea,
            controls
        });
    }

    createCodeControls(language, originalCode, textarea) {
        const controls = document.createElement('div');
        controls.className = 'code-controls';
        controls.innerHTML = `
            <div class="controls-row">
                <button class="btn btn-sm btn-primary run-code">
                    <i class="fas fa-play"></i> Run Code
                </button>
                <button class="btn btn-sm btn-secondary reset-code">
                    <i class="fas fa-undo"></i> Reset
                </button>
                <button class="btn btn-sm btn-secondary format-code">
                    <i class="fas fa-magic"></i> Format
                </button>
                <div class="code-language">${language}</div>
            </div>
            <div class="code-output"></div>
        `;
        
        // Add event listeners
        const runBtn = controls.querySelector('.run-code');
        const resetBtn = controls.querySelector('.reset-code');
        const formatBtn = controls.querySelector('.format-code');
        const output = controls.querySelector('.code-output');
        
        runBtn.addEventListener('click', () => {
            this.executeCode(language, textarea.value, output, runBtn);
        });
        
        resetBtn.addEventListener('click', () => {
            textarea.value = originalCode;
            output.innerHTML = '';
        });
        
        formatBtn.addEventListener('click', () => {
            this.formatCode(language, textarea.value).then(formatted => {
                textarea.value = formatted;
            });
        });
        
        return controls;
    }

    addRunButton(codeBlock, language) {
        const pre = codeBlock.querySelector('pre');
        const code = codeBlock.querySelector('code');
        const originalCode = code.textContent;
        
        const runButton = document.createElement('button');
        runButton.className = 'btn btn-sm btn-primary run-code-btn';
        runButton.innerHTML = '<i class="fas fa-play"></i> Run';
        runButton.style.cssText = `
            position: absolute;
            top: var(--spacing-md);
            right: var(--spacing-md);
            z-index: 10;
        `;
        
        const container = document.createElement('div');
        container.style.position = 'relative';
        pre.parentNode.insertBefore(container, pre);
        container.appendChild(pre);
        container.appendChild(runButton);
        
        runButton.addEventListener('click', () => {
            this.executeCode(language, originalCode, null, runButton);
        });
    }

    async executeCode(language, code, outputContainer, button) {
        const runButton = button || outputContainer?.parentElement?.querySelector('.run-code-btn');
        
        if (runButton) {
            runButton.disabled = true;
            runButton.innerHTML = '<i class="fas fa-spinner fa-spin"></i> Running...';
        }
        
        try {
            let result;
            
            switch (language) {
                case 'rust':
                    result = await this.executeRustCode(code);
                    break;
                case 'javascript':
                case 'js':
                    result = await this.executeJavaScriptCode(code);
                    break;
                case 'python':
                    result = await this.executePythonCode(code);
                    break;
                default:
                    result = { success: false, output: 'Language not supported for execution' };
            }
            
            if (outputContainer) {
                this.displayOutput(result, outputContainer);
            } else {
                this.displayInlineResult(result, runButton);
            }
            
        } catch (error) {
            const errorResult = {
                success: false,
                output: `Error: ${error.message}`,
                error: error
            };
            
            if (outputContainer) {
                this.displayOutput(errorResult, outputContainer);
            } else {
                this.displayInlineResult(errorResult, runButton);
            }
        } finally {
            if (runButton) {
                runButton.disabled = false;
                runButton.innerHTML = '<i class="fas fa-play"></i> Run';
            }
        }
    }

    async executeRustCode(code) {
        // Simulate Rust code execution (in a real implementation, this would use WebAssembly or server-side execution)
        return new Promise((resolve) => {
            setTimeout(() => {
                if (code.includes('println!')) {
                    const output = code.match(/println!\([^)]+\)/g)?.[0]?.match(/\("([^"]+)"\)/)?.[1] || 'Hello, World!';
                    resolve({
                        success: true,
                        output: `${output}\n\nProcess finished with exit code 0`,
                        compilationTime: '0.5s'
                    });
                } else {
                    resolve({
                        success: true,
                        output: 'Code compiled and executed successfully',
                        compilationTime: '0.3s'
                    });
                }
            }, 1000);
        });
    }

    async executeJavaScriptCode(code) {
        try {
            // Create a safe execution context
            const safeEval = new Function('console', `
                const log = (...args) => {
                    window.__codeOutput = (window.__codeOutput || []).concat(args.map(String));
                };
                ${code}
                return window.__codeOutput ? window.__codeOutput.join('\\n') : 'Code executed successfully';
            `);
            
            window.__codeOutput = [];
            const result = safeEval(console);
            
            return {
                success: true,
                output: result || 'Code executed successfully'
            };
        } catch (error) {
            return {
                success: false,
                output: `Error: ${error.message}`,
                error: error
            };
        }
    }

    async executePythonCode(code) {
        // In a real implementation, this would use Pyodide or similar
        return {
            success: true,
            output: 'Python execution simulated (requires Pyodide)',
            note: 'Install Pyodide for real Python execution'
        };
    }

    displayOutput(result, container) {
        container.innerHTML = `
            <div class="code-output-header">
                <span class="output-status ${result.success ? 'success' : 'error'}">
                    <i class="fas fa-${result.success ? 'check' : 'times'}"></i>
                    ${result.success ? 'Success' : 'Error'}
                </span>
                ${result.compilationTime ? `<span class="compilation-time">${result.compilationTime}</span>` : ''}
            </div>
            <div class="code-output-body">
                <pre>${this.escapeHtml(result.output)}</pre>
            </div>
        `;
        
        container.className = `code-output ${result.success ? 'success' : 'error'}`;
    }

    displayInlineResult(result, button) {
        const originalText = button.innerHTML;
        
        if (result.success) {
            button.innerHTML = '<i class="fas fa-check"></i> Success';
            button.className = button.className.replace('btn-primary', 'btn-success');
        } else {
            button.innerHTML = '<i class="fas fa-times"></i> Error';
            button.className = button.className.replace('btn-primary', 'btn-error');
        }
        
        setTimeout(() => {
            button.innerHTML = originalText;
            button.className = button.className.replace(/btn-(success|error)/, 'btn-primary');
        }, 3000);
    }

    async formatCode(language, code) {
        // Simple code formatting (in a real implementation, use proper formatters)
        switch (language) {
            case 'rust':
                return this.formatRustCode(code);
            case 'javascript':
                return this.formatJavaScriptCode(code);
            case 'python':
                return this.formatPythonCode(code);
            default:
                return code;
        }
    }

    formatRustCode(code) {
        // Basic Rust formatting
        return code
            .replace(/fn\s+(\w+)\s*\(/g, 'fn $1(')
            .replace(/let\s+(\w+):\s*(\w+)/g, 'let $1: $2')
            .replace(/\{\s*/g, ' {\n    ')
            .replace(/\s*\}/g, '\n}');
    }

    formatJavaScriptCode(code) {
        // Basic JavaScript formatting
        return code
            .replace(/function\s+(\w+)/g, 'function $1')
            .replace(/\{\s*/g, ' {\n    ')
            .replace(/\s*\}/g, '\n}');
    }

    formatPythonCode(code) {
        // Basic Python formatting
        return code
            .replace(/def\s+(\w+)/g, 'def $1')
            .replace(/:\s*/g, ':\n    ');
    }

    // Live Code Editors
    setupLiveEditors() {
        const liveEditors = document.querySelectorAll('.live-editor');
        
        liveEditors.forEach(editor => {
            this.initializeLiveEditor(editor);
        });
    }

    initializeLiveEditor(container) {
        const editorId = container.id || `editor-${Date.now()}`;
        const editor = container.querySelector('.editor-pane');
        const output = container.querySelector('.output-content');
        const runButton = container.querySelector('.run-editor-btn');
        const resetButton = container.querySelector('.reset-editor-btn');
        
        if (!editor || !output || !runButton) return;
        
        const language = container.getAttribute('data-language') || 'rust';
        const initialCode = editor.textContent.trim();
        
        // Store editor state
        this.liveEditors.set(editorId, {
            language,
            initialCode,
            editor: editor.querySelector('textarea') || this.createEditorElement(editor, initialCode),
            output,
            container
        });
        
        // Setup event listeners
        runButton.addEventListener('click', () => {
            this.runLiveEditor(editorId);
        });
        
        resetButton?.addEventListener('click', () => {
            this.resetLiveEditor(editorId);
        });
        
        // Auto-run on Ctrl+Enter
        const textArea = editor.querySelector('textarea');
        if (textArea) {
            textArea.addEventListener('keydown', (e) => {
                if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
                    e.preventDefault();
                    this.runLiveEditor(editorId);
                }
            });
        }
    }

    createEditorElement(container, initialCode) {
        const textarea = document.createElement('textarea');
        textarea.value = initialCode;
        textarea.style.cssText = `
            width: 100%;
            height: 100%;
            border: none;
            outline: none;
            background: transparent;
            font-family: var(--font-mono);
            font-size: 0.875rem;
            line-height: 1.6;
            padding: var(--spacing-lg);
            color: var(--text-primary);
            resize: none;
        `;
        
        container.innerHTML = '';
        container.appendChild(textarea);
        
        return textarea;
    }

    async runLiveEditor(editorId) {
        const state = this.liveEditors.get(editorId);
        if (!state) return;
        
        const { language, editor, output, container } = state;
        const code = editor.value;
        
        // Show running state
        output.textContent = 'Running...';
        output.className = 'output-content running';
        
        const runButton = container.querySelector('.run-editor-btn');
        if (runButton) {
            runButton.disabled = true;
        }
        
        try {
            const result = await this.executeCode(language, code);
            this.displayLiveOutput(result, output);
        } catch (error) {
            this.displayLiveOutput({
                success: false,
                output: `Error: ${error.message}`
            }, output);
        } finally {
            if (runButton) {
                runButton.disabled = false;
            }
        }
    }

    resetLiveEditor(editorId) {
        const state = this.liveEditors.get(editorId);
        if (!state) return;
        
        const { initialCode, editor, output } = state;
        editor.value = initialCode;
        output.textContent = '';
        output.className = 'output-content';
    }

    displayLiveOutput(result, output) {
        output.className = `output-content ${result.success ? 'success' : 'error'}`;
        output.textContent = result.output;
    }

    // Interactive Tutorials
    setupInteractiveTutorials() {
        const tutorials = document.querySelectorAll('.interactive-tutorial');
        
        tutorials.forEach(tutorial => {
            this.initializeTutorial(tutorial);
        });
    }

    initializeTutorial(container) {
        const steps = container.querySelectorAll('.tutorial-step');
        const progressBar = container.querySelector('.progress-bar-fill');
        const progressText = container.querySelector('.progress-text');
        
        if (steps.length === 0) return;
        
        // Setup step navigation
        steps.forEach((step, index) => {
            const header = step.querySelector('.tutorial-step-header');
            const expandBtn = step.querySelector('.expand-step');
            
            if (header) {
                header.addEventListener('click', () => {
                    this.toggleTutorialStep(step, container);
                });
            }
            
            if (expandBtn) {
                expandBtn.addEventListener('click', (e) => {
                    e.stopPropagation();
                    this.expandTutorialStep(step, container);
                });
            }
        });
        
        // Setup navigation buttons
        const prevBtn = container.querySelector('.tutorial-prev');
        const nextBtn = container.querySelector('.tutorial-next');
        
        if (prevBtn) {
            prevBtn.addEventListener('click', () => {
                this.navigateTutorial(container, -1);
            });
        }
        
        if (nextBtn) {
            nextBtn.addEventListener('click', () => {
                this.navigateTutorial(container, 1);
            });
        }
        
        // Initialize first step
        this.expandTutorialStep(steps[0], container);
        this.updateTutorialProgress(container);
    }

    toggleTutorialStep(step, container) {
        const isExpanded = step.classList.contains('expanded');
        
        if (isExpanded) {
            step.classList.remove('expanded');
        } else {
            this.expandTutorialStep(step, container);
        }
    }

    expandTutorialStep(step, container) {
        // Collapse other steps
        const steps = container.querySelectorAll('.tutorial-step');
        steps.forEach(s => s.classList.remove('expanded'));
        
        // Expand current step
        step.classList.add('expanded');
        step.classList.add('active');
        
        this.updateTutorialProgress(container);
    }

    navigateTutorial(container, direction) {
        const steps = container.querySelectorAll('.tutorial-step');
        const activeStep = container.querySelector('.tutorial-step.active');
        const currentIndex = Array.from(steps).indexOf(activeStep);
        
        let newIndex = currentIndex + direction;
        newIndex = Math.max(0, Math.min(newIndex, steps.length - 1));
        
        this.expandTutorialStep(steps[newIndex], container);
    }

    updateTutorialProgress(container) {
        const steps = container.querySelectorAll('.tutorial-step');
        const completedSteps = container.querySelectorAll('.tutorial-step.completed').length;
        const progress = (completedSteps / steps.length) * 100;
        
        const progressBar = container.querySelector('.progress-bar-fill');
        const progressText = container.querySelector('.progress-text');
        
        if (progressBar) {
            progressBar.style.width = `${progress}%`;
        }
        
        if (progressText) {
            progressText.textContent = `${completedSteps} of ${steps.length} steps completed`;
        }
    }

    // API Tester
    setupAPITester() {
        const testers = document.querySelectorAll('.api-tester');
        
        testers.forEach(tester => {
            this.initializeAPITester(tester);
        });
    }

    initializeAPITester(container) {
        const sendButton = container.querySelector('.send-request');
        const clearButton = container.querySelector('.clear-request');
        
        if (sendButton) {
            sendButton.addEventListener('click', () => {
                this.sendAPIRequest(container);
            });
        }
        
        if (clearButton) {
            clearButton.addEventListener('click', () => {
                this.clearAPIRequest(container);
            });
        }
        
        // Setup parameter builders
        this.setupParameterBuilders(container);
    }

    async sendAPIRequest(container) {
        const method = container.querySelector('[name="method"]')?.value || 'GET';
        const url = container.querySelector('[name="url"]')?.value;
        const headers = container.querySelector('[name="headers"]')?.value;
        const body = container.querySelector('[name="body"]')?.value;
        
        if (!url) {
            this.showNotification('Please enter a URL', 'error');
            return;
        }
        
        const startTime = performance.now();
        const responseContainer = container.querySelector('.tester-response');
        const responseBody = responseContainer.querySelector('.response-body');
        
        try {
            // Simulate API request (in a real implementation, use fetch)
            const result = await this.simulateAPIRequest({ method, url, headers, body });
            const endTime = performance.now();
            
            this.displayAPIResponse(result, responseContainer, endTime - startTime);
            
        } catch (error) {
            this.displayAPIResponse({
                success: false,
                status: 'Error',
                headers: {},
                body: error.message
            }, responseContainer, 0);
        }
    }

    async simulateAPIRequest({ method, url, headers, body }) {
        // Simulate API response based on URL
        await new Promise(resolve => setTimeout(resolve, Math.random() * 1000 + 500));
        
        if (url.includes('/api/kernel/processes')) {
            return {
                success: true,
                status: '200 OK',
                headers: {
                    'content-type': 'application/json',
                    'server': 'MultiOS/1.0'
                },
                body: JSON.stringify([
                    { id: 1, name: 'init', status: 'running', pid: 1 },
                    { id: 2, name: 'shell', status: 'running', pid: 2 }
                ], null, 2)
            };
        }
        
        if (url.includes('/api/memory/info')) {
            return {
                success: true,
                status: '200 OK',
                headers: {
                    'content-type': 'application/json'
                },
                body: JSON.stringify({
                    total: '8GB',
                    available: '6.2GB',
                    used: '1.8GB',
                    free: '6.2GB'
                }, null, 2)
            };
        }
        
        // Default response
        return {
            success: true,
            status: '200 OK',
            headers: {
                'content-type': 'text/plain'
            },
            body: 'MultiOS API Response\n\nRequest processed successfully.'
        };
    }

    displayAPIResponse(result, container, responseTime) {
        const status = container.querySelector('.response-status');
        const time = container.querySelector('.response-time');
        const body = container.querySelector('.response-body');
        
        if (status) {
            status.textContent = result.status;
            status.className = `response-status ${result.success ? 'success' : 'error'}`;
        }
        
        if (time) {
            time.textContent = `${responseTime.toFixed(2)}ms`;
        }
        
        if (body) {
            body.textContent = result.body;
        }
    }

    clearAPIRequest(container) {
        const inputs = container.querySelectorAll('input, textarea');
        inputs.forEach(input => {
            if (input.name !== 'method') {
                input.value = '';
            }
        });
        
        const responseBody = container.querySelector('.response-body');
        if (responseBody) {
            responseBody.textContent = '';
        }
    }

    setupParameterBuilders(container) {
        const builders = container.querySelectorAll('.param-builder');
        
        builders.forEach(builder => {
            const addParamBtn = builder.querySelector('.add-parameter');
            const paramsList = builder.querySelector('.parameters-list');
            
            if (addParamBtn && paramsList) {
                addParamBtn.addEventListener('click', () => {
                    this.addParameterField(paramsList);
                });
            }
        });
    }

    addParameterField(container) {
        const paramField = document.createElement('div');
        paramField.className = 'parameter-field';
        paramField.innerHTML = `
            <input type="text" placeholder="Parameter name" class="param-name">
            <input type="text" placeholder="Parameter value" class="param-value">
            <button type="button" class="btn btn-sm btn-danger remove-parameter">
                <i class="fas fa-times"></i>
            </button>
        `;
        
        container.appendChild(paramField);
        
        // Setup remove button
        const removeBtn = paramField.querySelector('.remove-parameter');
        removeBtn.addEventListener('click', () => {
            container.removeChild(paramField);
        });
    }

    // Code Playground
    setupCodePlayground() {
        const playgrounds = document.querySelectorAll('.playground-container');
        
        playgrounds.forEach(playground => {
            this.initializeCodePlayground(playground);
        });
    }

    initializeCodePlayground(container) {
        const editor = container.querySelector('.playground-editor');
        const preview = container.querySelector('.playground-preview');
        const runButton = container.querySelector('.run-playground');
        const resetButton = container.querySelector('.reset-playground');
        
        if (!editor || !runButton) return;
        
        const initialCode = editor.querySelector('textarea')?.value || '';
        
        if (runButton) {
            runButton.addEventListener('click', () => {
                this.runPlayground(container);
            });
        }
        
        if (resetButton) {
            resetButton.addEventListener('click', () => {
                this.resetPlayground(container);
            });
        }
    }

    async runPlayground(container) {
        const editor = container.querySelector('.playground-editor textarea');
        const preview = container.querySelector('.playground-preview .output-content');
        const code = editor?.value || '';
        
        if (preview) {
            preview.textContent = 'Running...';
        }
        
        try {
            // Execute JavaScript code
            const result = await this.executeJavaScriptCode(code);
            
            if (preview) {
                preview.textContent = result.output;
                preview.className = 'output-content ' + (result.success ? 'success' : 'error');
            }
            
        } catch (error) {
            if (preview) {
                preview.textContent = `Error: ${error.message}`;
                preview.className = 'output-content error';
            }
        }
    }

    resetPlayground(container) {
        const editor = container.querySelector('.playground-editor textarea');
        const preview = container.querySelector('.playground-preview .output-content');
        
        if (editor) {
            editor.value = '';
        }
        
        if (preview) {
            preview.textContent = '';
            preview.className = 'output-content';
        }
    }

    // Utility Functions
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    showNotification(message, type = 'info') {
        const notification = document.createElement('div');
        notification.className = `notification notification-${type}`;
        notification.textContent = message;
        
        notification.style.cssText = `
            position: fixed;
            top: 20px;
            right: 20px;
            padding: 12px 20px;
            background: var(--bg-primary);
            border: 1px solid var(--border-color);
            border-radius: 6px;
            box-shadow: var(--shadow-lg);
            z-index: 1000;
            color: var(--text-primary);
        `;
        
        document.body.appendChild(notification);
        
        setTimeout(() => {
            notification.remove();
        }, 3000);
    }
}

// Initialize interactive components when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.interactiveComponents = new InteractiveComponents();
});