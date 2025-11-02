/**
 * GDB Console Module
 * Provides GDB-like functionality for interactive debugging
 */

class GDBConsole {
    constructor() {
        this.app = null;
        this.isVisible = false;
        this.commandHistory = [];
        this.historyIndex = -1;
        this.currentCommand = '';
        this.breakpoints = new Map();
        this.watchpoints = new Map();
        this.registers = new Map();
        this.disassemblyCache = new Map();
        this.sessionCommands = [];
        this.isConnected = false;
        
        // GDB command aliases and shortcuts
        this.commandAliases = {
            'b': 'break',
            'c': 'continue',
            'r': 'run',
            'n': 'next',
            's': 'step',
            'p': 'print',
            'pp': 'print',
            'pt': 'ptype',
            'bt': 'backtrace',
            'info': 'info',
            'x': 'examine',
            'i': 'info',
            'q': 'quit'
        };
        
        // Command completion
        this.commonCommands = [
            'run', 'continue', 'next', 'step', 'finish', 'until',
            'break', 'delete', 'disable', 'enable', 'clear',
            'print', 'display', 'watch', 'rwatch', 'awatch',
            'info', 'list', 'backtrace', 'frame', 'up', 'down',
            'quit', 'help', 'set', 'show', 'examine', 'disassemble',
            'thread', 'info threads', 'attach', 'detach', 'kill',
            'load', 'file', 'core', 'symbol', 'info registers',
            'info variables', 'info locals', 'info functions',
            'ptype', 'whatis', 'call', 'make', 'shell'
        ];
    }

    setApp(app) {
        this.app = app;
        this.initializeElements();
        this.initializeEventListeners();
    }

    initializeElements() {
        this.gdbConsole = document.getElementById('gdbConsole');
        this.gdbOutput = document.getElementById('gdbOutput');
        this.gdbInput = document.getElementById('gdbInput');
    }

    initializeEventListeners() {
        if (this.gdbInput) {
            this.gdbInput.addEventListener('keydown', (e) => this.handleKeyDown(e));
            this.gdbInput.addEventListener('input', (e) => this.handleInput(e));
        }
        
        // Console control buttons
        const runCommandBtn = document.getElementById('runGdbCommandBtn');
        const clearConsoleBtn = document.getElementById('clearGdbConsoleBtn');
        
        if (runCommandBtn) {
            runCommandBtn.addEventListener('click', () => this.executeCurrentCommand());
        }
        
        if (clearConsoleBtn) {
            clearConsoleBtn.addEventListener('click', () => this.clearConsole());
        }
    }

    handleKeyDown(event) {
        switch (event.key) {
            case 'Enter':
                event.preventDefault();
                this.executeCurrentCommand();
                break;
            case 'ArrowUp':
                event.preventDefault();
                this.navigateHistory(-1);
                break;
            case 'ArrowDown':
                event.preventDefault();
                this.navigateHistory(1);
                break;
            case 'Tab':
                event.preventDefault();
                this.completeCommand();
                break;
            case 'Escape':
                event.preventDefault();
                this.hide();
                break;
        }
    }

    handleInput(event) {
        this.currentCommand = event.target.value;
        // Could implement real-time command suggestions here
    }

    show() {
        if (!this.isVisible) {
            this.isVisible = true;
            this.gdbConsole.classList.add('visible');
            this.gdbInput.focus();
            this.addGdbOutput('(gdb) Welcome to GDB console', 'welcome');
        }
    }

    hide() {
        if (this.isVisible) {
            this.isVisible = false;
            this.gdbConsole.classList.remove('visible');
        }
    }

    toggle() {
        if (this.isVisible) {
            this.hide();
        } else {
            this.show();
        }
    }

    async executeCommand(command) {
        if (!command.trim()) {
            return;
        }

        this.addGdbOutput(`(gdb) ${command}`, 'command');
        this.commandHistory.push(command);
        this.historyIndex = -1;
        
        // Add to session commands
        this.sessionCommands.push({
            command,
            timestamp: new Date(),
            result: null,
            error: null
        });

        try {
            const result = await this.processCommand(command);
            this.addGdbOutput(result, 'result');
            
            // Update session command result
            const lastCommand = this.sessionCommands[this.sessionCommands.length - 1];
            if (lastCommand) {
                lastCommand.result = result;
            }
        } catch (error) {
            this.addGdbOutput(`Error: ${error.message}`, 'error');
            
            // Update session command error
            const lastCommand = this.sessionCommands[this.sessionCommands.length - 1];
            if (lastCommand) {
                lastCommand.error = error.message;
            }
        }

        this.currentCommand = '';
        this.gdbInput.value = '';
    }

    async processCommand(command) {
        const parts = command.trim().split(/\s+/);
        const cmd = parts[0].toLowerCase();
        const args = parts.slice(1);
        
        // Handle command aliases
        const actualCmd = this.commandAliases[cmd] || cmd;
        
        switch (actualCmd) {
            case 'run':
            case 'r':
                return await this.cmdRun(args);
                
            case 'continue':
            case 'c':
                return await this.cmdContinue(args);
                
            case 'next':
            case 'n':
                return await this.cmdNext(args);
                
            case 'step':
            case 's':
                return await this.cmdStep(args);
                
            case 'finish':
                return await this.cmdFinish(args);
                
            case 'until':
                return await this.cmdUntil(args);
                
            case 'break':
            case 'b':
                return await this.cmdBreak(args);
                
            case 'delete':
                return await this.cmdDelete(args);
                
            case 'info':
                return await this.cmdInfo(args);
                
            case 'print':
            case 'p':
            case 'pp':
                return await this.cmdPrint(args);
                
            case 'display':
                return await this.cmdDisplay(args);
                
            case 'watch':
                return await this.cmdWatch(args);
                
            case 'list':
            case 'l':
                return await this.cmdList(args);
                
            case 'backtrace':
            case 'bt':
                return await this.cmdBacktrace(args);
                
            case 'frame':
                return await this.cmdFrame(args);
                
            case 'up':
                return await this.cmdUp(args);
                
            case 'down':
                return await this.cmdDown(args);
                
            case 'examine':
            case 'x':
                return await this.cmdExamine(args);
                
            case 'disassemble':
                return await this.cmdDisassemble(args);
                
            case 'help':
            case 'h':
                return await this.cmdHelp(args);
                
            case 'quit':
            case 'q':
                return await this.cmdQuit(args);
                
            default:
                return `No symbol "${cmd}" in current context.`;
        }
    }

    async cmdRun(args) {
        if (this.app && this.app.debugCore) {
            await this.app.debugCore.runToNextBreakpoint();
            return 'Starting program: /path/to/program\nBreakpoint 1, main () at main.c:10';
        }
        
        return 'Starting program: /path/to/program\nBreakpoint 1, main () at main.c:10\n10\t    int x = 10;';
    }

    async cmdContinue(args) {
        if (this.app && this.app.debugCore) {
            await this.app.debugCore.continue();
            return 'Continuing...';
        }
        
        return 'Continuing...';
    }

    async cmdNext(args) {
        if (this.app && this.app.debugCore) {
            await this.app.debugCore.stepOver();
            const currentLine = this.app.debugCore.getCurrentLine();
            return `Next: ${currentLine}`;
        }
        
        return '11\t    int y = 20;';
    }

    async cmdStep(args) {
        if (this.app && this.app.debugCore) {
            await this.app.debugCore.stepInto();
            const currentLine = this.app.debugCore.getCurrentLine();
            return `Step: ${currentLine}`;
        }
        
        return 'Stepping into function call...';
    }

    async cmdFinish(args) {
        if (this.app && this.app.debugCore) {
            await this.app.debugCore.stepOut();
            return 'Finished function call';
        }
        
        return 'Run till exit from #0  add (a=10, b=20) at main.c:20\n0x0000555555555149 in main () at main.c:12';
    }

    async cmdUntil(args) {
        return 'Continuing until breakpoint or end of program';
    }

    async cmdBreak(args) {
        if (args.length === 0) {
            return this.listBreakpoints();
        }
        
        const location = args[0];
        let breakpoint;
        
        if (location.match(/^\d+$/)) {
            // Line number
            const line = parseInt(location);
            if (this.app && this.app.debugCore) {
                breakpoint = await this.app.debugCore.addBreakpoint(line);
            } else {
                breakpoint = { id: 1, line: line, enabled: true };
            }
        } else if (location.includes(':')) {
            // Function:line format
            const [functionName, line] = location.split(':');
            breakpoint = { id: 2, function: functionName, line: parseInt(line), enabled: true };
        } else {
            // Function name
            breakpoint = { id: 3, function: location, enabled: true };
        }
        
        this.breakpoints.set(breakpoint.id, breakpoint);
        return `Breakpoint ${breakpoint.id} at 0x${this.getMockAddress()}: file ${this.getMockFile()}, line ${breakpoint.line}.`;
    }

    async cmdDelete(args) {
        if (args.length === 0) {
            // Delete all breakpoints
            this.breakpoints.clear();
            return 'Delete all breakpoints? (y or n) y\nAll breakpoints cleared.';
        }
        
        const bpId = parseInt(args[0]);
        if (this.breakpoints.has(bpId)) {
            this.breakpoints.delete(bpId);
            return `Breakpoint ${bpId} deleted.`;
        } else {
            return `No breakpoint number ${bpId}.`;
        }
    }

    async cmdInfo(args) {
        if (args.length === 0) {
            return this.showInfoHelp();
        }
        
        const subcommand = args[0].toLowerCase();
        
        switch (subcommand) {
            case 'breakpoints':
            case 'b':
                return this.listBreakpoints();
                
            case 'threads':
                return this.listThreads();
                
            case 'registers':
                return this.listRegisters();
                
            case 'variables':
                return this.listVariables();
                
            case 'locals':
                return this.listLocals();
                
            case 'functions':
                return this.listFunctions();
                
            case 'args':
                return this.listArgs();
                
            case 'program':
                return this.getProgramStatus();
                
            default:
                return `Undefined info command: "info ${subcommand}".  Try "help info".`;
        }
    }

    async cmdPrint(args) {
        if (args.length === 0) {
            return 'No expression to print.';
        }
        
        const expression = args.join(' ');
        const value = this.evaluateExpression(expression);
        return `${expression} = ${value}`;
    }

    async cmdDisplay(args) {
        if (args.length === 0) {
            return this.listDisplay();
        }
        
        const expression = args.join(' ');
        const id = this.displayItems.length + 1;
        this.displayItems.push({ id, expression });
        return `${id}: ${expression} = ${this.evaluateExpression(expression)}`;
    }

    async cmdWatch(args) {
        if (args.length === 0) {
            return 'No expression to watch.';
        }
        
        const expression = args.join(' ');
        const id = this.watchpoints.size + 1;
        this.watchpoints.set(id, { id, expression });
        return `Hardware watchpoint ${id}: ${expression}`;
    }

    async cmdList(args) {
        if (args.length === 0) {
            return this.listSourceCode();
        }
        
        const arg = args[0];
        if (arg.match(/^\d+$/)) {
            // Line number
            return this.listSourceCode(parseInt(arg));
        } else if (arg.includes(':')) {
            // Function:line format
            const [functionName, line] = arg.split(':');
            return this.listSourceCode(parseInt(line), functionName);
        } else {
            // Function name
            return this.listFunctionSource(arg);
        }
    }

    async cmdBacktrace(args) {
        const callStack = this.getCallStack();
        return callStack.map((frame, index) => 
            `#${index}  ${frame.function} at ${frame.file}:${frame.line}`
        ).join('\n');
    }

    async cmdFrame(args) {
        if (args.length === 0) {
            return this.showCurrentFrame();
        }
        
        const frameNum = parseInt(args[0]);
        return `Switching to frame ${frameNum}`;
    }

    async cmdUp(args) {
        const count = args.length > 0 ? parseInt(args[0]) : 1;
        return `Up ${count} frame(s)`;
    }

    async cmdDown(args) {
        const count = args.length > 0 ? parseInt(args[0]) : 1;
        return `Down ${count} frame(s)`;
    }

    async cmdExamine(args) {
        if (args.length === 0) {
            return this.usageExamine();
        }
        
        const address = args[0];
        const format = args.length > 1 ? args[1] : 'x';
        const count = args.length > 2 ? parseInt(args[2]) : 1;
        
        return this.examineMemory(address, format, count);
    }

    async cmdDisassemble(args) {
        if (args.length === 0) {
            return 'No function specified.';
        }
        
        const functionName = args[0];
        return this.disassembleFunction(functionName);
    }

    async cmdHelp(args) {
        if (args.length === 0) {
            return this.showGeneralHelp();
        }
        
        const command = args[0].toLowerCase();
        return this.showCommandHelp(command);
    }

    async cmdQuit(args) {
        return this.quitGDB();
    }

    // Helper methods for command implementations
    listBreakpoints() {
        if (this.breakpoints.size === 0) {
            return 'No breakpoints or watchpoints.';
        }
        
        return Array.from(this.breakpoints.values()).map(bp => 
            `${bp.id} ${bp.enabled ? '' : 'dis '}breakpoint     keep y   ${bp.file || 'unknown'}:${bp.line}`
        ).join('\n');
    }

    listThreads() {
        return `  Id   Target Id         Frame 
* 1    Thread 0x7f8c (LWP 1234) main () at main.c:10`;
    }

    listRegisters() {
        return `rax            0x0                 0
rbx            0x0                 0
rcx            0x55a1b4d2a000     140248095748864
rdx            0x0                 0
rsi            0x0                 0
rdi            0x0                 0
rbp            0x7fff5fbff5b0     0x7fff5fbff5b0
rsp            0x7fff5fbff590     0x7fff5fbff590
r8             0x0                 0`;
    }

    listVariables() {
        if (this.app && this.app.variableInspector) {
            const variables = this.app.variableInspector.getVariables();
            return variables.map(v => `${v.name} = ${v.value} (${v.type})`).join('\n');
        }
        return 'global_counter = 100 (int)';
    }

    listLocals() {
        if (this.app && this.app.variableInspector) {
            const variables = this.app.variableInspector.getVariables()
                .filter(v => v.scope === 'local');
            return variables.map(v => `${v.name} = ${v.value} (${v.type})`).join('\n');
        }
        return 'x = 10 (int)\ny = 20 (int)\nresult = 30 (int)';
    }

    listFunctions() {
        return this.mockFunctions?.map(f => `${f.name} at ${f.file}:${f.line}`).join('\n') || 
               'add at main.c:8\nmain at main.c:10';
    }

    listArgs() {
        return 'No arguments.';
    }

    getProgramStatus() {
        const state = this.app?.debugCore?.getDebugState() || 'stopped';
        return `Program terminated with status ${state === 'stopped' ? 0 : 1}.`;
    }

    listDisplay() {
        if (this.displayItems.length === 0) {
            return 'No auto-display expressions now.';
        }
        
        return this.displayItems.map(item => 
            `${item.id}: ${item.expression} = ${this.evaluateExpression(item.expression)}`
        ).join('\n');
    }

    showCurrentFrame() {
        return `Current frame is at #0 main () at main.c:10`;
    }

    usageExamine() {
        return `Usage: x/FMT ADDRESS.
FMT is a repeat count followed by a format letter and a size letter.
Format letters are: o(octal), x(hex), d(decimal), u(unsigned decimal), 
t(binary), f(float), a(address), i(instruction), s(string), c(char).
Size letters are: b(byte), h(halfword), w(word), g(giant, 8 bytes).`;
    }

    examineMemory(address, format, count) {
        const mockData = {
            'x/16xb': '0x1000: 0x48 0x83 0xec 0x08 0x48 0x83 0xe4 0xf0 0x48 0x83 0xf9 0x01 0x74 0x10 0x48 0x83',
            'x/8xw': '0x1000: 0x08ec8348 0xf0e48348 0x0183f948 0x10837448'
        };
        
        return mockData[`${format}/${count}${address}`] || `0x1000: <memory contents>`;
    }

    disassembleFunction(functionName) {
        return `Dump of assembler code for function ${functionName}:
   0x0000555555555140 <+0>:     push   rbp
   0x0000555555555141 <+1>:     mov    rbp,rsp
   0x0000555555555144 <+4>:     mov    DWORD PTR [rbp-0x4],edi
   0x0000555555555147 <+7>:     mov    DWORD PTR [rbp-0x8],esi
   0x000055555555514a <+10>:    mov    eax,DWORD PTR [rbp-0x4]
   0x000055555555514d <+13>:    add    eax,DWORD PTR [rbp-0x8]
   0x0000555555555150 <+16>:    pop    rbp
   0x0000555555555151 <+17>:    ret    
End of assembler dump.`;
    }

    evaluateExpression(expression) {
        // Mock expression evaluation
        const mockValues = {
            'x': 10,
            'y': 20,
            'result': 30,
            'global_counter': 100,
            'count': 5,
            'total': 150,
            'buffer': '"Hello Debug"',
            'ptr': '0x7fff5fbff5bc'
        };
        
        if (mockValues[expression] !== undefined) {
            return mockValues[expression];
        }
        
        // Simple arithmetic expressions
        if (expression.includes('+')) {
            const parts = expression.split('+').map(p => p.trim());
            return parts.reduce((sum, part) => {
                const value = mockValues[part];
                return sum + (value || 0);
            }, 0);
        }
        
        if (expression.includes('-')) {
            const parts = expression.split('-').map(p => p.trim());
            return (mockValues[parts[0]] || 0) - (mockValues[parts[1]] || 0);
        }
        
        return 'Cannot evaluate expression';
    }

    getCallStack() {
        if (this.app && this.app.debugCore) {
            return this.app.debugCore.getCallStack();
        }
        
        return [
            { function: 'main', file: 'main.c', line: 12 },
            { function: 'add', file: 'main.c', line: 8 },
            { function: '__libc_start_main', file: 'crt1.o', line: 0 }
        ];
    }

    showInfoHelp() {
        return `List of info subcommands:

"info address" -- Describe where symbol SYMBOL is stored
"info all-registers" -- List of all registers and their contents
"info args" -- Argument variables of current stack frame
"info breakpoints" -- Status of user-settable breakpoints
"info catch" -- List of active exception handlers
"info channels" -- List of open channels
"info display" -- List of auto-display expressions
"info dll" -- List of dynamically loaded shared libraries
"info extensions" -- All filename extensions used for finding source files
"info files" -- GDB files
"info floating-point" -- Show floating point unit
"info functions" -- All function names
"info handle" -- How to handle signals
"info history" -- The history list
"info line" -- Core addresses of the code for a source line
"info locals" -- Local variables of current stack frame
"info proc" -- List of active processes
"info program" -- Execution status of the program
"info registers" -- List of registers
"info scope" -- List of symbols of a lexical block
"info selectors" -- All Objective-C method selectors
"info set" -- Show all GDB settings
"info source" -- Information about the current source file
"info sources" -- All source files
"info stack" -- Backtrace of the stack
"info symbol" -- Describe what symbol is at location
"info threads" -- Display info about threads
"info tracepoints" -- Status of tracepoints
"info type" -- Print a description of a type
"info types" -- All type names
"info variables" -- All global and static variable names
"info vectors" -- Show the FPU's state
"info warranty" -- GDB version and warranty disclaimer
"info watchpoints" -- Status of watchpoints`;
    }

    showGeneralHelp() {
        return `GDB, the GNU Debugger, version 8.3.0.

Type "help <topic>" to get help on a specific command.
Type "apropos <word>" to search for commands related to "word".
Available commands:

break, continue, delete, display, examine, finish, frame, help, info,
list, next, print, quit, run, set, show, step, until, watch, where, backtrace

For more information, type "help" followed by command name.`;
    }

    showCommandHelp(command) {
        const helpMessages = {
            'break': `Set a breakpoint at specified location.

Usage: break [location] [if condition]

Examples:
  break main
  break 10
  break myfunction if x > 0`,

            'continue': `Continue program being debugged, after signal or breakpoint.

Usage: continue [ignore-count]

Continue from the last signal or breakpoint.`,
            
            'next': `Step program, proceeding through subroutine calls.

Usage: next [count]

Execute the next source line, stepping over any function calls.`,
            
            'step': `Step program until it reaches a different source line.

Usage: step [count]

Execute one or more source lines, stepping into function calls.`,
            
            'print': `Print value of expression EXP.

Usage: print [EXP]

Display the value of the expression EXP in the most suitable format.`,
            
            'backtrace': `Print backtrace of all stack frames.

Usage: backtrace [full]

Print a backtrace of the entire call stack.`
        };
        
        return helpMessages[command] || `No help available for "${command}".`;
    }

    listSourceCode(line) {
        const code = this.getMockSourceCode();
        if (line) {
            return code.slice(Math.max(0, line - 3), Math.min(code.length, line + 7)).join('\n');
        }
        return code.join('\n');
    }

    listFunctionSource(functionName) {
        return this.listSourceCode();
    }

    getMockSourceCode() {
        return [
            '9\tvoid swap(int *a, int *b) {',
            '10\t    int temp = *a;',
            '11\t    *a = *b;',
            '12\t    *b = temp;',
            '13\t}',
            '14\t',
            '15\tint main() {',
            '16\t    int x = 10, y = 20;',
            '17\t    printf("Before swap: x=%d, y=%d\\n", x, y);',
            '18\t    swap(&x, &y);',
            '19\t    printf("After swap: x=%d, y=%d\\n", x, y);',
            '20\t    return 0;',
            '21\t}'
        ];
    }

    quitGDB() {
        this.hide();
        return 'Quitting...';
    }

    executeCurrentCommand() {
        const command = this.gdbInput.value.trim();
        if (command) {
            this.executeCommand(command);
        }
    }

    clearConsole() {
        this.gdbOutput.innerHTML = '';
    }

    addGdbOutput(message, type = 'result') {
        const messageEl = document.createElement('div');
        messageEl.className = `gdb-${type}`;
        messageEl.textContent = message;
        
        this.gdbOutput.appendChild(messageEl);
        this.gdbOutput.scrollTop = this.gdbOutput.scrollHeight;
    }

    navigateHistory(direction) {
        if (direction === -1) {
            // Navigate up (older commands)
            if (this.historyIndex < this.commandHistory.length - 1) {
                this.historyIndex++;
                this.gdbInput.value = this.commandHistory[this.commandHistory.length - 1 - this.historyIndex];
            }
        } else {
            // Navigate down (newer commands)
            if (this.historyIndex > 0) {
                this.historyIndex--;
                this.gdbInput.value = this.commandHistory[this.commandHistory.length - 1 - this.historyIndex];
            } else if (this.historyIndex === 0) {
                this.historyIndex = -1;
                this.gdbInput.value = this.currentCommand;
            }
        }
    }

    completeCommand() {
        const input = this.gdbInput.value.trim();
        if (!input) return;
        
        const matchingCommands = this.commonCommands.filter(cmd => 
            cmd.startsWith(input)
        );
        
        if (matchingCommands.length === 1) {
            this.gdbInput.value = matchingCommands[0] + ' ';
        } else if (matchingCommands.length > 1) {
            // Show completion suggestions
            const suggestions = matchingCommands.slice(0, 5).join('  ');
            this.addGdbOutput(`Possible completions: ${suggestions}`, 'completion');
        }
    }

    getMockAddress() {
        return Math.floor(Math.random() * 0x100000000).toString(16);
    }

    getMockFile() {
        return 'main.c';
    }

    exportSession() {
        return {
            commands: this.sessionCommands,
            breakpoints: Array.from(this.breakpoints.values()),
            watchpoints: Array.from(this.watchpoints.values()),
            displayItems: this.displayItems
        };
    }

    importSession(sessionData) {
        try {
            this.sessionCommands = sessionData.commands || [];
            this.breakpoints.clear();
            this.watchpoints.clear();
            
            (sessionData.breakpoints || []).forEach(bp => {
                this.breakpoints.set(bp.id, bp);
            });
            
            (sessionData.watchpoints || []).forEach(wp => {
                this.watchpoints.set(wp.id, wp);
            });
            
            this.displayItems = sessionData.displayItems || [];
            
            if (this.app) {
                this.app.showNotification('GDB session imported', 'success');
            }
        } catch (error) {
            if (this.app) {
                this.app.showNotification(`Failed to import session: ${error.message}`, 'error');
            }
        }
    }

    // Properties
    get displayItems() {
        return this._displayItems || (this._displayItems = []);
    }

    set displayItems(items) {
        this._displayItems = items;
    }

    destroy() {
        this.hide();
        this.commandHistory = [];
        this.breakpoints.clear();
        this.watchpoints.clear();
        this.registers.clear();
        this.disassemblyCache.clear();
        this.sessionCommands = [];
    }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = GDBConsole;
}