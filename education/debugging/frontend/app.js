/**
 * Main Application Controller for Interactive OS Debugging System
 * Integrates all debugging components and manages the overall state
 */

class DebugApp {
    constructor() {
        this.debugSession = null;
        this.currentScenario = null;
        this.isGuidedMode = true;
        this.currentStep = 0;
        this.totalSteps = 0;
        this.sessionTimer = null;
        this.sessionStartTime = null;
        this.integratedWithMultiOS = false;
        this.apiEndpoint = 'http://localhost:8000/api';
        
        // Component instances
        this.debugCore = new DebugCore();
        this.memoryView = new MemoryView();
        this.variableInspector = new VariableInspector();
        this.syscallTracer = new SyscallTracer();
        this.gdbConsole = new GDBConsole();
        this.guidedTutorials = new GuidedTutorials();
        this.scenarios = new ScenarioLibrary();
        this.integration = new MultiOSIntegration();
        
        // UI elements
        this.initializeElements();
        
        // Event listeners
        this.initializeEventListeners();
        
        // Initialize components
        this.initializeComponents();
        
        // Start session timer
        this.startSessionTimer();
        
        console.log('🛠️ Interactive OS Debugging System initialized');
    }

    initializeElements() {
        // Header elements
        this.sessionStatusEl = document.getElementById('sessionStatus');
        this.sessionTimerEl = document.getElementById('sessionTimer');
        this.integrateBtn = document.getElementById('integrateBtn');
        
        // Scenario elements
        this.scenarioSelect = document.getElementById('scenarioSelect');
        this.loadScenarioBtn = document.getElementById('loadScenarioBtn');
        this.guidedModeCheckbox = document.getElementById('guidedMode');
        this.showHintsCheckbox = document.getElementById('showHints');
        this.startTutorialBtn = document.getElementById('startTutorialBtn');
        
        // Progress elements
        this.progressPanel = document.getElementById('progressPanel');
        this.progressFill = document.getElementById('progressFill');
        this.currentStepEl = document.getElementById('currentStep');
        this.totalStepsEl = document.getElementById('totalSteps');
        this.stepDescriptionEl = document.getElementById('stepDescription');
        
        // Code debugging elements
        this.codeDisplay = document.getElementById('codeDisplay');
        this.lineNumbers = document.getElementById('lineNumbers');
        this.filePathInput = document.getElementById('filePath');
        this.loadFileBtn = document.getElementById('loadFileBtn');
        this.breakpointsList = document.getElementById('breakpointsList');
        this.addBreakpointBtn = document.getElementById('addBreakpointBtn');
        
        // Quick action buttons
        this.resetScenarioBtn = document.getElementById('resetScenario');
        this.runToNextBreakpointBtn = document.getElementById('runToNextBreakpoint');
        this.stepIntoBtn = document.getElementById('stepInto');
        this.stepOverBtn = document.getElementById('stepOver');
        this.stepOutBtn = document.getElementById('stepOut');
        this.continueBtn = document.getElementById('continue');
        
        // Panel refresh buttons
        this.inspectVariablesBtn = document.getElementById('inspectVariablesBtn');
        this.refreshMemoryBtn = document.getElementById('refreshMemoryBtn');
        this.memoryViewTypeSelect = document.getElementById('memoryViewType');
        this.refreshProcessesBtn = document.getElementById('refreshProcessesBtn');
        this.startTracingBtn = document.getElementById('startTracingBtn');
        this.stopTracingBtn = document.getElementById('stopTracingBtn');
        
        // Terminal elements
        this.terminalOutput = document.getElementById('terminalOutput');
        this.terminalInput = document.getElementById('terminalInput');
        this.runCommandBtn = document.getElementById('runCommandBtn');
        this.clearTerminalBtn = document.getElementById('clearTerminalBtn');
        
        // Hints elements
        this.hintsContent = document.getElementById('hintsContent');
        this.toggleHintsBtn = document.getElementById('toggleHintsBtn');
        
        // Integration elements
        this.integrationModal = document.getElementById('integrationModal');
        this.uiThemeSelect = document.getElementById('uiTheme');
        this.panelLayoutSelect = document.getElementById('panelLayout');
        this.apiEndpointInput = document.getElementById('apiEndpoint');
        this.autoSyncCheckbox = document.getElementById('autoSync');
        this.saveIntegrationBtn = document.getElementById('saveIntegrationBtn');
        this.testConnectionBtn = document.getElementById('testConnectionBtn');
        this.closeModalBtn = document.getElementById('closeModalBtn');
        
        // Loading elements
        this.loadingSpinner = document.getElementById('loadingSpinner');
    }

    initializeEventListeners() {
        // Scenario loading
        this.loadScenarioBtn.addEventListener('click', () => this.loadScenario());
        this.startTutorialBtn.addEventListener('click', () => this.startTutorial());
        
        // Code debugging
        this.loadFileBtn.addEventListener('click', () => this.loadCodeFile());
        this.addBreakpointBtn.addEventListener('click', () => this.addBreakpoint());
        
        // Quick actions
        this.resetScenarioBtn.addEventListener('click', () => this.resetScenario());
        this.runToNextBreakpointBtn.addEventListener('click', () => this.runToNextBreakpoint());
        this.stepIntoBtn.addEventListener('click', () => this.stepInto());
        this.stepOverBtn.addEventListener('click', () => this.stepOver());
        this.stepOutBtn.addEventListener('click', () => this.stepOut());
        this.continueBtn.addEventListener('click', () => this.continueExecution());
        
        // Panel refreshes
        this.inspectVariablesBtn.addEventListener('click', () => this.variableInspector.refresh());
        this.refreshMemoryBtn.addEventListener('click', () => this.memoryView.refresh());
        this.refreshProcessesBtn.addEventListener('click', () => this.updateProcessView());
        this.startTracingBtn.addEventListener('click', () => this.syscallTracer.start());
        this.stopTracingBtn.addEventListener('click', () => this.syscallTracer.stop());
        
        // Terminal
        this.runCommandBtn.addEventListener('click', () => this.runTerminalCommand());
        this.clearTerminalBtn.addEventListener('click', () => this.clearTerminal());
        
        // Hints
        this.toggleHintsBtn.addEventListener('click', () => this.toggleHints());
        
        // Integration
        this.integrateBtn.addEventListener('click', () => this.showIntegrationModal());
        this.saveIntegrationBtn.addEventListener('click', () => this.saveIntegrationSettings());
        this.testConnectionBtn.addEventListener('click', () => this.testConnection());
        this.closeModalBtn.addEventListener('click', () => this.hideIntegrationModal());
        
        // Guided mode
        this.guidedModeCheckbox.addEventListener('change', (e) => {
            this.isGuidedMode = e.target.checked;
            this.guidedTutorials.setEnabled(this.isGuidedMode);
        });
        
        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => this.handleKeyboardShortcuts(e));
        
        // Memory view type change
        this.memoryViewTypeSelect.addEventListener('change', () => {
            this.memoryView.changeViewType(this.memoryViewTypeSelect.value);
        });
    }

    initializeComponents() {
        // Initialize all components with app reference
        this.debugCore.setApp(this);
        this.memoryView.setApp(this);
        this.variableInspector.setApp(this);
        this.syscallTracer.setApp(this);
        this.gdbConsole.setApp(this);
        this.guidedTutorials.setApp(this);
        this.scenarios.setApp(this);
        this.integration.setApp(this);
        
        // Load initial state
        this.loadInitialState();
    }

    async loadScenario() {
        const scenarioId = this.scenarioSelect.value;
        if (!scenarioId) {
            this.showNotification('Please select a scenario', 'warning');
            return;
        }

        try {
            this.showLoading();
            this.currentScenario = await this.scenarios.loadScenario(scenarioId);
            this.totalSteps = this.currentScenario.steps.length;
            
            // Update UI
            this.updateProgressUI();
            this.progressPanel.style.display = 'block';
            
            // Initialize scenario
            await this.currentScenario.initialize();
            
            // Start guided tutorial if enabled
            if (this.isGuidedMode) {
                await this.guidedTutorials.startScenario(this.currentScenario);
            }
            
            this.hideLoading();
            this.showNotification(`Scenario "${this.currentScenario.name}" loaded successfully`, 'success');
            
        } catch (error) {
            this.hideLoading();
            this.showNotification(`Failed to load scenario: ${error.message}`, 'error');
            console.error('Error loading scenario:', error);
        }
    }

    async startTutorial() {
        if (!this.currentScenario) {
            this.showNotification('Please load a scenario first', 'warning');
            return;
        }

        try {
            await this.guidedTutorials.startScenario(this.currentScenario);
            this.showNotification('Tutorial started', 'info');
        } catch (error) {
            this.showNotification(`Failed to start tutorial: ${error.message}`, 'error');
        }
    }

    async loadCodeFile() {
        const filePath = this.filePathInput.value.trim();
        if (!filePath) {
            this.showNotification('Please enter a file path', 'warning');
            return;
        }

        try {
            this.showLoading();
            const fileContent = await this.debugCore.loadFile(filePath);
            this.displayCode(fileContent, filePath);
            this.hideLoading();
            this.showNotification(`File "${filePath}" loaded successfully`, 'success');
        } catch (error) {
            this.hideLoading();
            this.showNotification(`Failed to load file: ${error.message}`, 'error');
        }
    }

    displayCode(content, filePath) {
        const lines = content.split('\n');
        
        // Generate line numbers
        this.lineNumbers.innerHTML = lines.map((_, index) => 
            `<div class="line-number" data-line="${index + 1}">${index + 1}</div>`
        ).join('');
        
        // Display code with syntax highlighting
        this.codeDisplay.innerHTML = this.highlightCode(content, filePath);
        
        // Make lines clickable for breakpoints
        this.lineNumbers.querySelectorAll('.line-number').forEach(line => {
            line.addEventListener('click', () => {
                const lineNum = parseInt(line.dataset.line);
                this.toggleBreakpoint(lineNum);
            });
        });
    }

    highlightCode(code, filePath) {
        // Simple syntax highlighting for common languages
        const extension = filePath.split('.').pop().toLowerCase();
        
        switch (extension) {
            case 'c':
            case 'cpp':
            case 'h':
                return this.highlightCLike(code);
            case 'java':
                return this.highlightJava(code);
            case 'py':
                return this.highlightPython(code);
            case 'js':
                return this.highlightJavaScript(code);
            default:
                return `<pre>${this.escapeHtml(code)}</pre>`;
        }
    }

    highlightCLike(code) {
        return code
            .replace(/(\/\/.*$)/gm, '<span class="syntax-comment">$1</span>')
            .replace(/(\/\*[\s\S]*?\*\/)/g, '<span class="syntax-comment">$1</span>')
            .replace(/\b(int|char|float|double|void|if|else|while|for|return|struct|class|public|private|protected|try|catch|throw)\b/g, '<span class="syntax-keyword">$1</span>')
            .replace(/("[^"]*")/g, '<span class="syntax-string">$1</span>')
            .replace(/(\d+)/g, '<span class="syntax-number">$1</span>')
            .replace(/\b([a-zA-Z_]\w*)\s*\(/g, '<span class="syntax-function">$1</span>(')
            .replace(/`([^`]*)`/g, '<span class="syntax-variable">$1</span>');
    }

    highlightJava(code) {
        return code
            .replace(/(\/\/.*$)/gm, '<span class="syntax-comment">$1</span>')
            .replace(/(\/\*[\s\S]*?\*\/)/g, '<span class="syntax-comment">$1</span>')
            .replace(/\b(public|private|protected|static|final|class|interface|extends|implements|if|else|while|for|try|catch|finally|throw|throws|return|int|char|float|double|boolean|void|String|null|true|false)\b/g, '<span class="syntax-keyword">$1</span>')
            .replace(/("[^"]*")/g, '<span class="syntax-string">$1</span>')
            .replace(/(\d+)/g, '<span class="syntax-number">$1</span>')
            .replace(/\b([a-zA-Z_]\w*)\s*\(/g, '<span class="syntax-function">$1</span>(');
    }

    highlightPython(code) {
        return code
            .replace(/(#.*$)/gm, '<span class="syntax-comment">$1</span>')
            .replace(/("""|''')([\s\S]*?)("""|''')/g, '<span class="syntax-comment">$1$2$3</span>')
            .replace(/\b(def|class|if|elif|else|while|for|try|except|finally|with|as|import|from|return|yield|break|continue|pass|lambda|and|or|not|in|is|True|False|None)\b/g, '<span class="syntax-keyword">$1</span>')
            .replace(/("[^"]*"|'[^']*')/g, '<span class="syntax-string">$1</span>')
            .replace(/(\d+)/g, '<span class="syntax-number">$1</span>')
            .replace(/\b([a-zA-Z_]\w*)\s*\(/g, '<span class="syntax-function">$1</span>(');
    }

    highlightJavaScript(code) {
        return code
            .replace(/(\/\/.*$)/gm, '<span class="syntax-comment">$1</span>')
            .replace(/(\/\*[\s\S]*?\*\/)/g, '<span class="syntax-comment">$1</span>')
            .replace(/\b(function|var|let|const|if|else|while|for|try|catch|finally|throw|return|yield|break|continue|switch|case|default|new|this|class|extends|super|import|from|export|async|await)\b/g, '<span class="syntax-keyword">$1</span>')
            .replace(/("[^"]*"|'[^']*')/g, '<span class="syntax-string">$1</span>')
            .replace(/(\d+)/g, '<span class="syntax-number">$1</span>')
            .replace(/\b([a-zA-Z_]\w*)\s*\(/g, '<span class="syntax-function">$1</span>(');
    }

    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    async addBreakpoint() {
        const lineNumber = prompt('Enter line number to set breakpoint:');
        if (!lineNumber || isNaN(lineNumber)) {
            return;
        }

        await this.debugCore.addBreakpoint(parseInt(lineNumber));
        this.updateBreakpointsList();
    }

    toggleBreakpoint(lineNumber) {
        this.debugCore.toggleBreakpoint(lineNumber);
        this.updateBreakpointsList();
    }

    updateBreakpointsList() {
        const breakpoints = this.debugCore.getBreakpoints();
        this.breakpointsList.innerHTML = breakpoints.map((bp, index) => `
            <div class="breakpoint-item">
                <div>
                    <span class="breakpoint-number">${index + 1}</span>
                    Line ${bp.line}
                </div>
                <button class="btn btn-sm btn-danger" onclick="app.debugCore.removeBreakpoint(${bp.line})">Remove</button>
            </div>
        `).join('');
    }

    async runToNextBreakpoint() {
        try {
            this.updateSessionStatus('debugging');
            await this.debugCore.runToNextBreakpoint();
            this.updateSessionStatus('paused');
            
            // Update UI after breakpoint hit
            this.updateCurrentLine();
            await this.variableInspector.refresh();
            this.showNotification('Reached breakpoint', 'info');
        } catch (error) {
            this.updateSessionStatus('error');
            this.showNotification(`Debug error: ${error.message}`, 'error');
        }
    }

    async stepInto() {
        try {
            this.updateSessionStatus('debugging');
            await this.debugCore.stepInto();
            this.updateSessionStatus('paused');
            this.updateCurrentLine();
            await this.variableInspector.refresh();
        } catch (error) {
            this.updateSessionStatus('error');
            this.showNotification(`Step error: ${error.message}`, 'error');
        }
    }

    async stepOver() {
        try {
            this.updateSessionStatus('debugging');
            await this.debugCore.stepOver();
            this.updateSessionStatus('paused');
            this.updateCurrentLine();
            await this.variableInspector.refresh();
        } catch (error) {
            this.updateSessionStatus('error');
            this.showNotification(`Step error: ${error.message}`, 'error');
        }
    }

    async stepOut() {
        try {
            this.updateSessionStatus('debugging');
            await this.debugCore.stepOut();
            this.updateSessionStatus('paused');
            this.updateCurrentLine();
            await this.variableInspector.refresh();
        } catch (error) {
            this.updateSessionStatus('error');
            this.showNotification(`Step error: ${error.message}`, 'error');
        }
    }

    async continueExecution() {
        try {
            this.updateSessionStatus('debugging');
            await this.debugCore.continue();
            this.updateSessionStatus('running');
        } catch (error) {
            this.updateSessionStatus('error');
            this.showNotification(`Continue error: ${error.message}`, 'error');
        }
    }

    updateCurrentLine() {
        const currentLine = this.debugCore.getCurrentLine();
        if (currentLine) {
            // Remove previous current line highlight
            document.querySelectorAll('.current-line').forEach(el => {
                el.classList.remove('current-line');
            });
            
            // Highlight current line
            const lineEl = this.lineNumbers.querySelector(`[data-line="${currentLine}"]`);
            if (lineEl) {
                lineEl.classList.add('current-line');
                lineEl.scrollIntoView({ behavior: 'smooth', block: 'center' });
            }
        }
    }

    async updateProcessView() {
        const processes = await this.debugCore.getProcesses();
        this.displayProcesses(processes);
    }

    displayProcesses(processes) {
        const processTree = document.getElementById('processTree');
        processTree.innerHTML = processes.map(process => `
            <div class="process-item" onclick="app.showProcessDetails('${process.id}')">
                <div class="process-icon">${process.icon || 'P'}</div>
                <div class="process-info">
                    <div class="process-name">${process.name}</div>
                    <div class="process-id">PID: ${process.id}</div>
                </div>
                <div class="process-status ${process.state.toLowerCase()}">${process.state}</div>
            </div>
        `).join('');
    }

    async showProcessDetails(processId) {
        const threads = await this.debugCore.getThreads(processId);
        this.displayThreads(threads);
    }

    displayThreads(threads) {
        const threadDetails = document.getElementById('threadDetails');
        threadDetails.innerHTML = threads.map(thread => `
            <div class="thread-item">
                <span class="thread-id">TID: ${thread.id}</span>
                <span>${thread.name}</span>
                <span class="thread-state ${thread.state.toLowerCase()}">${thread.state}</span>
            </div>
        `).join('');
        threadDetails.classList.add('visible');
    }

    async runTerminalCommand() {
        const command = this.terminalInput.value.trim();
        if (!command) return;

        try {
            this.addToTerminal(`$ ${command}`, 'command');
            const output = await this.debugCore.runCommand(command);
            this.addToTerminal(output, 'output');
            this.terminalInput.value = '';
        } catch (error) {
            this.addToTerminal(`Error: ${error.message}`, 'error');
        }
    }

    addToTerminal(message, type = 'output') {
        const timestamp = new Date().toLocaleTimeString();
        const messageClass = type === 'error' ? 'error' : type === 'command' ? 'command' : 'output';
        
        const messageEl = document.createElement('div');
        messageEl.className = `terminal-message ${messageClass}`;
        messageEl.innerHTML = `<span class="timestamp">[${timestamp}]</span> ${message}`;
        
        this.terminalOutput.appendChild(messageEl);
        this.terminalOutput.scrollTop = this.terminalOutput.scrollHeight;
    }

    clearTerminal() {
        this.terminalOutput.innerHTML = '';
    }

    updateSessionStatus(status) {
        this.sessionStatusEl.textContent = status.charAt(0).toUpperCase() + status.slice(1);
        this.sessionStatusEl.className = `status-indicator ${status}`;
    }

    updateProgressUI() {
        const progress = (this.currentStep / this.totalSteps) * 100;
        this.progressFill.style.width = `${progress}%`;
        this.currentStepEl.textContent = this.currentStep;
        this.totalStepsEl.textContent = this.totalSteps;
        
        if (this.currentScenario) {
            const step = this.currentScenario.steps[this.currentStep - 1];
            this.stepDescriptionEl.textContent = step ? step.description : '';
        }
    }

    nextStep() {
        if (this.currentStep < this.totalSteps) {
            this.currentStep++;
            this.updateProgressUI();
            
            if (this.isGuidedMode && this.showHintsCheckbox.checked) {
                this.showStepHint();
            }
        }
    }

    previousStep() {
        if (this.currentStep > 1) {
            this.currentStep--;
            this.updateProgressUI();
        }
    }

    showStepHint() {
        if (this.currentScenario && this.currentScenario.steps[this.currentStep - 1]) {
            const step = this.currentScenario.steps[this.currentStep - 1];
            if (step.hint) {
                this.addHint(step.hint);
            }
        }
    }

    addHint(hintText) {
        const hintItem = document.createElement('div');
        hintItem.className = 'hint-item';
        hintItem.innerHTML = `
            <div class="hint-header">
                <span class="hint-number">${this.currentStep}</span>
                <span class="hint-title">Step ${this.currentStep}</span>
            </div>
            <div class="hint-body">${hintText}</div>
        `;
        this.hintsContent.appendChild(hintItem);
    }

    toggleHints() {
        this.hintsContent.classList.toggle('collapsed');
    }

    showNotification(message, type = 'info') {
        // Create and show notification
        const notification = document.createElement('div');
        notification.className = `notification ${type}`;
        notification.innerHTML = `
            <div class="notification-header">
                <span class="notification-title">${type.charAt(0).toUpperCase() + type.slice(1)}</span>
                <button class="notification-close">&times;</button>
            </div>
            <div class="notification-body">${message}</div>
        `;
        
        document.body.appendChild(notification);
        
        // Show notification
        setTimeout(() => notification.classList.add('visible'), 100);
        
        // Auto-hide after 5 seconds
        setTimeout(() => {
            notification.classList.remove('visible');
            setTimeout(() => document.body.removeChild(notification), 300);
        }, 5000);
        
        // Close button
        notification.querySelector('.notification-close').addEventListener('click', () => {
            notification.classList.remove('visible');
            setTimeout(() => document.body.removeChild(notification), 300);
        });
    }

    showLoading() {
        this.loadingSpinner.classList.add('visible');
    }

    hideLoading() {
        this.loadingSpinner.classList.remove('visible');
    }

    showIntegrationModal() {
        this.integrationModal.classList.add('visible');
    }

    hideIntegrationModal() {
        this.integrationModal.classList.remove('visible');
    }

    async saveIntegrationSettings() {
        try {
            const settings = {
                uiTheme: this.uiThemeSelect.value,
                panelLayout: this.panelLayoutSelect.value,
                apiEndpoint: this.apiEndpointInput.value,
                autoSync: this.autoSyncCheckbox.checked
            };
            
            await this.integration.saveSettings(settings);
            this.integratedWithMultiOS = true;
            this.integrateBtn.textContent = 'Integrated ✓';
            this.integrateBtn.classList.add('success');
            
            this.hideIntegrationModal();
            this.showNotification('Integration settings saved successfully', 'success');
        } catch (error) {
            this.showNotification(`Failed to save settings: ${error.message}`, 'error');
        }
    }

    async testConnection() {
        try {
            this.showNotification('Testing connection...', 'info');
            const response = await this.integration.testConnection(this.apiEndpointInput.value);
            this.showNotification('Connection successful!', 'success');
        } catch (error) {
            this.showNotification(`Connection failed: ${error.message}`, 'error');
        }
    }

    startSessionTimer() {
        this.sessionStartTime = Date.now();
        this.sessionTimer = setInterval(() => {
            const elapsed = Date.now() - this.sessionStartTime;
            const hours = Math.floor(elapsed / 3600000);
            const minutes = Math.floor((elapsed % 3600000) / 60000);
            const seconds = Math.floor((elapsed % 60000) / 1000);
            
            this.sessionTimerEl.textContent = 
                `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
        }, 1000);
    }

    handleKeyboardShortcuts(event) {
        // Debug shortcuts
        if (event.ctrlKey || event.metaKey) {
            switch (event.key) {
                case 'r':
                    event.preventDefault();
                    this.runToNextBreakpoint();
                    break;
                case 's':
                    event.preventDefault();
                    this.stepOver();
                    break;
                case 'i':
                    event.preventDefault();
                    this.stepInto();
                    break;
                case 'o':
                    event.preventDefault();
                    this.stepOut();
                    break;
                case 'c':
                    event.preventDefault();
                    this.continueExecution();
                    break;
            }
        }
        
        // F5 - Continue
        if (event.key === 'F5') {
            event.preventDefault();
            this.continueExecution();
        }
        
        // F10 - Step Over
        if (event.key === 'F10') {
            event.preventDefault();
            this.stepOver();
        }
        
        // F11 - Step Into
        if (event.key === 'F11') {
            event.preventDefault();
            this.stepInto();
        }
        
        // Shift+F11 - Step Out
        if (event.key === 'F11' && event.shiftKey) {
            event.preventDefault();
            this.stepOut();
        }
    }

    async resetScenario() {
        if (this.currentScenario) {
            await this.currentScenario.reset();
            this.currentStep = 0;
            this.updateProgressUI();
            this.hintsContent.innerHTML = '';
            this.showNotification('Scenario reset successfully', 'success');
        }
    }

    loadInitialState() {
        // Load saved settings
        const savedSettings = localStorage.getItem('debugAppSettings');
        if (savedSettings) {
            const settings = JSON.parse(savedSettings);
            this.isGuidedMode = settings.isGuidedMode ?? true;
            this.guidedModeCheckbox.checked = this.isGuidedMode;
        }
        
        // Initialize with welcome message
        this.addToTerminal('Welcome to the Interactive OS Debugging System!', 'output');
        this.addToTerminal('Select a scenario to begin your debugging journey.', 'output');
        
        // Show initial hints
        this.addHint('Select a debugging scenario from the left panel to begin your interactive learning journey.');
    }

    // Public API for components to call
    notifyStepCompleted(stepId) {
        this.nextStep();
    }

    notifyBreakpointHit(line) {
        this.updateCurrentLine();
    }

    notifyError(error) {
        this.showNotification(`Error: ${error.message}`, 'error');
    }

    // Cleanup
    destroy() {
        if (this.sessionTimer) {
            clearInterval(this.sessionTimer);
        }
        
        // Clean up all components
        this.debugCore.destroy();
        this.memoryView.destroy();
        this.variableInspector.destroy();
        this.syscallTracer.destroy();
        this.gdbConsole.destroy();
        this.guidedTutorials.destroy();
        this.scenarios.destroy();
        this.integration.destroy();
    }
}

// Initialize the application when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.app = new DebugApp();
});

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = DebugApp;
}