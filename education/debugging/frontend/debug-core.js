/**
 * Debug Core Module
 * Handles core debugging operations, breakpoints, and stepping functionality
 */

class DebugCore {
    constructor() {
        this.app = null;
        this.currentFile = null;
        this.currentLine = null;
        this.breakpoints = new Map();
        this.debugState = 'stopped'; // stopped, running, paused, error
        this.callStack = [];
        this.variables = new Map();
        this.watchExpressions = new Set();
        this.threads = new Map();
        
        // Debug session data
        this.sessionData = {
            startTime: null,
            instructions: [],
            memorySnapshots: [],
            variableChanges: []
        };
    }

    setApp(app) {
        this.app = app;
    }

    async loadFile(filePath) {
        try {
            // In a real implementation, this would load from the backend
            const response = await fetch(`/api/debug/files?path=${encodeURIComponent(filePath)}`);
            if (!response.ok) {
                throw new Error(`Failed to load file: ${response.statusText}`);
            }
            
            const data = await response.json();
            this.currentFile = {
                path: filePath,
                content: data.content,
                language: this.detectLanguage(filePath),
                lastModified: data.lastModified
            };
            
            // Parse variables and functions from the code
            this.parseCodeStructure();
            
            return data.content;
        } catch (error) {
            // Fallback: use mock data for demonstration
            return this.getMockFileContent(filePath);
        }
    }

    detectLanguage(filePath) {
        const extension = filePath.split('.').pop().toLowerCase();
        const languageMap = {
            'c': 'c',
            'cpp': 'cpp',
            'h': 'c',
            'hpp': 'cpp',
            'java': 'java',
            'py': 'python',
            'js': 'javascript',
            'ts': 'typescript',
            'go': 'go',
            'rs': 'rust'
        };
        return languageMap[extension] || 'text';
    }

    parseCodeStructure() {
        if (!this.currentFile?.content) return;
        
        const content = this.currentFile.content;
        const lines = content.split('\n');
        
        // Parse functions
        this.functions = [];
        lines.forEach((line, index) => {
            const trimmedLine = line.trim();
            if (this.isFunctionDeclaration(trimmedLine)) {
                this.functions.push({
                    name: this.extractFunctionName(trimmedLine),
                    line: index + 1,
                    parameters: this.extractParameters(trimmedLine)
                });
            }
        });
        
        // Parse variables
        this.variables.clear();
        lines.forEach((line, index) => {
            const varInfo = this.parseVariableDeclaration(line);
            if (varInfo) {
                this.variables.set(varInfo.name, {
                    ...varInfo,
                    line: index + 1,
                    value: this.getMockValue(varInfo.type)
                });
            }
        });
    }

    isFunctionDeclaration(line) {
        const functionPatterns = [
            /^\w+\s+\w+\s*\(/,  // C/C++/Java
            /^\w+\s+def\s+\w+/,  // Python
            /^\w+\s*:\s*\(/      // Go
        ];
        return functionPatterns.some(pattern => pattern.test(line));
    }

    extractFunctionName(line) {
        const match = line.match(/(\w+)\s*\(/);
        return match ? match[1] : 'unknown';
    }

    extractParameters(line) {
        const match = line.match(/\(([^)]*)\)/);
        if (!match) return [];
        
        return match[1].split(',')
            .map(p => p.trim())
            .filter(p => p.length > 0);
    }

    parseVariableDeclaration(line) {
        const varPatterns = [
            /^\s*(?:const\s+)?(\w+)\s+(\w+)\s*=/,  // C/C++/Java
            /^\s*(\w+)\s*=\s*(.+)$/                  // Python/JavaScript
        ];
        
        for (const pattern of varPatterns) {
            const match = line.match(pattern);
            if (match) {
                return {
                    name: match[2] || match[1],
                    type: this.inferType(match[2] || match[1], line),
                    value: match[3] || this.getMockValue(this.inferType(match[2] || match[1], line))
                };
            }
        }
        return null;
    }

    inferType(name, line) {
        const typeKeywords = {
            'int': 'int',
            'float': 'float',
            'double': 'double',
            'char': 'char',
            'string': 'string',
            'bool': 'boolean',
            'var': 'var',
            'let': 'var',
            'const': 'const'
        };
        
        for (const [keyword, type] of Object.entries(typeKeywords)) {
            if (line.includes(keyword)) {
                return type;
            }
        }
        
        return 'unknown';
    }

    getMockValue(type) {
        const mockValues = {
            'int': Math.floor(Math.random() * 1000),
            'float': (Math.random() * 100).toFixed(2),
            'double': (Math.random() * 1000).toFixed(3),
            'char': String.fromCharCode(65 + Math.floor(Math.random() * 26)),
            'string': `'MockString${Math.floor(Math.random() * 100)}'`,
            'boolean': Math.random() > 0.5,
            'pointer': `0x${Math.floor(Math.random() * 0x100000000).toString(16)}`
        };
        
        return mockValues[type] || 'null';
    }

    async addBreakpoint(line) {
        if (this.breakpoints.has(line)) {
            throw new Error(`Breakpoint already exists at line ${line}`);
        }
        
        const breakpoint = {
            id: Date.now(),
            line: line,
            enabled: true,
            condition: null,
            hitCount: 0,
            created: new Date().toISOString()
        };
        
        this.breakpoints.set(line, breakpoint);
        this.logDebugInstruction('breakpoint', { line, action: 'added' });
        
        return breakpoint;
    }

    toggleBreakpoint(line) {
        if (this.breakpoints.has(line)) {
            const bp = this.breakpoints.get(line);
            bp.enabled = !bp.enabled;
            this.logDebugInstruction('breakpoint', { line, action: 'toggled', enabled: bp.enabled });
        } else {
            this.addBreakpoint(line);
        }
    }

    removeBreakpoint(line) {
        if (this.breakpoints.delete(line)) {
            this.logDebugInstruction('breakpoint', { line, action: 'removed' });
        }
    }

    getBreakpoints() {
        return Array.from(this.breakpoints.values());
    }

    async runToNextBreakpoint() {
        if (this.breakpoints.size === 0) {
            throw new Error('No breakpoints set');
        }
        
        this.setDebugState('running');
        
        try {
            // Simulate running to next breakpoint
            await this.simulateExecution();
            
            // Find next breakpoint
            const nextBp = this.findNextBreakpoint();
            if (nextBp) {
                this.currentLine = nextBp.line;
                this.breakpoints.get(nextBp.line).hitCount++;
                this.setDebugState('paused');
                this.notifyBreakpointHit();
            } else {
                this.setDebugState('stopped');
                throw new Error('No breakpoints encountered');
            }
        } catch (error) {
            this.setDebugState('error');
            throw error;
        }
    }

    async stepInto() {
        if (this.debugState !== 'paused' && this.debugState !== 'stopped') {
            throw new Error('Cannot step when not paused');
        }
        
        this.setDebugState('stepping');
        
        try {
            // Simulate stepping into
            this.currentLine = (this.currentLine || 1) + 1;
            this.captureVariableState();
            this.setDebugState('paused');
            this.logDebugInstruction('step', { action: 'into', line: this.currentLine });
        } catch (error) {
            this.setDebugState('error');
            throw error;
        }
    }

    async stepOver() {
        if (this.debugState !== 'paused' && this.debugState !== 'stopped') {
            throw new Error('Cannot step when not paused');
        }
        
        this.setDebugState('stepping');
        
        try {
            // Simulate stepping over (skip function calls)
            this.currentLine = (this.currentLine || 1) + 1;
            this.captureVariableState();
            this.setDebugState('paused');
            this.logDebugInstruction('step', { action: 'over', line: this.currentLine });
        } catch (error) {
            this.setDebugState('error');
            throw error;
        }
    }

    async stepOut() {
        if (this.debugState !== 'paused') {
            throw new Error('Cannot step out when not paused');
        }
        
        this.setDebugState('stepping');
        
        try {
            // Simulate stepping out (return from function)
            this.currentLine = Math.max(1, (this.currentLine || 1) - 1);
            this.captureVariableState();
            this.setDebugState('paused');
            this.logDebugInstruction('step', { action: 'out', line: this.currentLine });
        } catch (error) {
            this.setDebugState('error');
            throw error;
        }
    }

    async continue() {
        if (this.debugState !== 'paused') {
            throw new Error('Cannot continue when not paused');
        }
        
        this.setDebugState('running');
        
        try {
            // Continue execution until next breakpoint or end
            await this.simulateExecution();
            
            const nextBp = this.findNextBreakpoint();
            if (nextBp) {
                this.currentLine = nextBp.line;
                this.breakpoints.get(nextBp.line).hitCount++;
                this.setDebugState('paused');
                this.notifyBreakpointHit();
            } else {
                this.setDebugState('stopped');
                this.currentLine = null;
            }
            
            this.logDebugInstruction('continue', {});
        } catch (error) {
            this.setDebugState('error');
            throw error;
        }
    }

    simulateExecution() {
        return new Promise((resolve) => {
            // Simulate execution time
            setTimeout(resolve, 1000 + Math.random() * 2000);
        });
    }

    findNextBreakpoint() {
        const enabledBreakpoints = Array.from(this.breakpoints.values())
            .filter(bp => bp.enabled);
        
        if (enabledBreakpoints.length === 0) return null;
        
        // Sort by line number and find the next one
        enabledBreakpoints.sort((a, b) => a.line - b.line);
        
        const nextBp = enabledBreakpoints.find(bp => 
            bp.line > (this.currentLine || 0)
        ) || enabledBreakpoints[0];
        
        return nextBp;
    }

    getCurrentLine() {
        return this.currentLine;
    }

    setDebugState(state) {
        const validStates = ['stopped', 'running', 'paused', 'error', 'stepping'];
        if (validStates.includes(state)) {
            this.debugState = state;
            this.logDebugInstruction('state', { state });
        }
    }

    captureVariableState() {
        const snapshot = {
            timestamp: new Date().toISOString(),
            line: this.currentLine,
            variables: {}
        };
        
        this.variables.forEach((value, name) => {
            snapshot.variables[name] = JSON.parse(JSON.stringify(value));
        });
        
        this.sessionData.variableChanges.push(snapshot);
    }

    getCallStack() {
        // Mock call stack
        const stackSize = 3 + Math.floor(Math.random() * 5);
        const stack = [];
        
        for (let i = 0; i < stackSize; i++) {
            const func = this.functions[i] || { name: `function${i}`, line: this.currentLine - i };
            stack.push({
                function: func.name,
                file: this.currentFile?.path || 'unknown',
                line: func.line - i,
                arguments: func.parameters?.map(p => `${p.type} ${p.name}`) || []
            });
        }
        
        return stack;
    }

    getVariables() {
        return Array.from(this.variables.entries()).map(([name, info]) => ({
            name,
            type: info.type,
            value: info.value,
            line: info.line,
            scope: this.determineScope(name, info.line)
        }));
    }

    determineScope(name, line) {
        // Simple scope determination
        const currentLine = this.currentLine || 1;
        if (Math.abs(line - currentLine) <= 5) {
            return 'local';
        }
        return 'global';
    }

    async runCommand(command) {
        try {
            const response = await fetch('/api/debug/commands', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ command })
            });
            
            if (!response.ok) {
                throw new Error(`Command failed: ${response.statusText}`);
            }
            
            const result = await response.json();
            this.logDebugInstruction('command', { command, result: result.output });
            
            return result.output;
        } catch (error) {
            // Fallback: mock command execution
            return this.executeMockCommand(command);
        }
    }

    executeMockCommand(command) {
        const commandMap = {
            'bt': this.getMockBacktrace(),
            'info locals': this.getMockLocals(),
            'info variables': this.getMockVariables(),
            'info threads': this.getMockThreads(),
            'disassemble': this.getMockDisassembly(),
            'info registers': this.getMockRegisters()
        };
        
        const normalizedCommand = command.toLowerCase().trim();
        if (commandMap[normalizedCommand]) {
            this.logDebugInstruction('command', { command, result: commandMap[normalizedCommand] });
            return commandMap[normalizedCommand];
        }
        
        return `Unknown command: ${command}`;
    }

    getMockBacktrace() {
        const stack = this.getCallStack();
        return stack.map(frame => 
            `#${stack.length - stack.indexOf(frame)} ${frame.function} at ${frame.file}:${frame.line}`
        ).join('\n');
    }

    getMockLocals() {
        const locals = Array.from(this.variables.values())
            .filter(v => v.line && Math.abs(v.line - (this.currentLine || 1)) <= 10);
        
        return locals.map(v => `${v.name} = ${v.value} (${v.type})`).join('\n') || 'No local variables';
    }

    getMockVariables() {
        return Array.from(this.variables.values())
            .map(v => `${v.name} = ${v.value} (${v.type})`).join('\n');
    }

    getMockThreads() {
        return `  Id   Target Id         Frame 
* 1    Thread 0x7f8c  main() at main.c:10`;
    }

    getMockDisassembly() {
        return `=> 0x55a1b4d2a000 <main+0>:    push   rbp
   0x55a1b4d2a001 <main+1>:    mov    rbp,rdi
   0x55a1b4d2a004 <main+4>:    mov    DWORD PTR [rbp-0x4],edi`;
    }

    getMockRegisters() {
        return `rax            0x0                 0
rbx            0x0                 0
rcx            0x55a1b4d2a000     140248095748864
rdx            0x0                 0`;
    }

    async getProcesses() {
        try {
            const response = await fetch('/api/debug/processes');
            const processes = await response.json();
            return processes;
        } catch (error) {
            // Return mock data
            return [
                {
                    id: 1234,
                    name: 'debugged-program',
                    state: 'Running',
                    pid: 1234,
                    parent: 1230,
                    cpu: '25%',
                    memory: '15.2 MB'
                },
                {
                    id: 1230,
                    name: 'bash',
                    state: 'Running',
                    pid: 1230,
                    parent: 1200,
                    cpu: '5%',
                    memory: '8.1 MB'
                }
            ];
        }
    }

    async getThreads(processId) {
        try {
            const response = await fetch(`/api/debug/processes/${processId}/threads`);
            const threads = await response.json();
            return threads;
        } catch (error) {
            // Return mock data
            return [
                {
                    id: 1234,
                    name: 'main thread',
                    state: 'Running',
                    priority: 0,
                    cpu: '25%'
                },
                {
                    id: 1235,
                    name: 'worker thread',
                    state: 'Sleeping',
                    priority: 0,
                    cpu: '0%'
                }
            ];
        }
    }

    getMockFileContent(filePath) {
        const mockFiles = {
            'main.c': `#include <stdio.h>
#include <stdlib.h>

int main() {
    int x = 10;
    int y = 20;
    int result = add(x, y);
    
    printf("Result: %d\\n", result);
    return 0;
}

int add(int a, int b) {
    return a + b;
}`,
            
            'example.c': `#include <stdio.h>
#include <pthread.h>

typedef struct {
    int* array;
    int size;
} ThreadData;

void* process_array(void* arg) {
    ThreadData* data = (ThreadData*)arg;
    
    for (int i = 0; i < data->size; i++) {
        data->array[i] *= 2;
    }
    
    return NULL;
}

int main() {
    int numbers[] = {1, 2, 3, 4, 5};
    int size = sizeof(numbers) / sizeof(numbers[0]);
    
    pthread_t thread;
    ThreadData data = {numbers, size};
    
    pthread_create(&thread, NULL, process_array, &data);
    pthread_join(thread, NULL);
    
    for (int i = 0; i < size; i++) {
        printf("%d ", numbers[i]);
    }
    
    return 0;
}`
        };
        
        const content = mockFiles[filePath] || `// Mock file content for ${filePath}
int main() {
    printf("Hello, Debug World!\\n");
    return 0;
}`;
        
        return content;
    }

    logDebugInstruction(type, data) {
        const instruction = {
            timestamp: new Date().toISOString(),
            type,
            data,
            debugState: this.debugState
        };
        
        this.sessionData.instructions.push(instruction);
        
        // Also log to terminal if available
        if (this.app) {
            this.app.addToTerminal(`[${instruction.timestamp}] ${type}: ${JSON.stringify(data)}`, 'debug');
        }
    }

    notifyBreakpointHit() {
        if (this.app && this.app.notifyBreakpointHit) {
            this.app.notifyBreakpointHit(this.currentLine);
        }
    }

    notifyError(error) {
        if (this.app && this.app.notifyError) {
            this.app.notifyError(error);
        }
    }

    getSessionData() {
        return this.sessionData;
    }

    getDebugState() {
        return this.debugState;
    }

    addWatchExpression(expression) {
        this.watchExpressions.add(expression);
        this.logDebugInstruction('watch', { action: 'added', expression });
    }

    removeWatchExpression(expression) {
        this.watchExpressions.delete(expression);
        this.logDebugInstruction('watch', { action: 'removed', expression });
    }

    getWatchExpressions() {
        return Array.from(this.watchExpressions);
    }

    evaluateWatchExpressions() {
        const results = [];
        this.watchExpressions.forEach(expression => {
            try {
                // In a real implementation, this would evaluate the expression
                const value = this.mockEvaluate(expression);
                results.push({ expression, value, valid: true });
            } catch (error) {
                results.push({ expression, value: error.message, valid: false });
            }
        });
        return results;
    }

    mockEvaluate(expression) {
        // Mock expression evaluation
        const mockValues = {
            'x': 10,
            'y': 20,
            'result': 30,
            'count': 5,
            'total': 100
        };
        
        if (mockValues[expression] !== undefined) {
            return mockValues[expression];
        }
        
        // Simple arithmetic
        if (expression.includes('+')) {
            const parts = expression.split('+');
            return parts.reduce((sum, part) => {
                const value = mockValues[part.trim()];
                return sum + (value || 0);
            }, 0);
        }
        
        return `Cannot evaluate: ${expression}`;
    }

    exportSession() {
        return {
            file: this.currentFile,
            breakpoints: this.getBreakpoints(),
            debugState: this.debugState,
            currentLine: this.currentLine,
            sessionData: this.sessionData,
            variables: this.getVariables(),
            callStack: this.getCallStack()
        };
    }

    destroy() {
        // Clean up resources
        this.breakpoints.clear();
        this.variables.clear();
        this.watchExpressions.clear();
        this.threads.clear();
        this.sessionData.instructions = [];
        this.currentFile = null;
        this.currentLine = null;
    }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = DebugCore;
}