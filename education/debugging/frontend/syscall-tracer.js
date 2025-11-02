/**
 * System Call Tracer Module
 * Handles system call tracing, parameter inspection, and syscall analysis
 */

class SyscallTracer {
    constructor() {
        this.app = null;
        this.isTracing = false;
        this.syscalls = [];
        this.filteredSyscalls = [];
        this.callHistory = new Map();
        this.breakpoints = new Set();
        this.filters = {
            enabled: false,
            syscallNames: new Set(),
            processId: null,
            threadId: null
        };
        this.statistics = {
            totalCalls: 0,
            callsPerSecond: 0,
            mostFrequent: [],
            avgDuration: 0
        };
        this.updateTimer = null;
        this.startTime = null;
    }

    setApp(app) {
        this.app = app;
    }

    async start() {
        if (this.isTracing) {
            if (this.app) {
                this.app.showNotification('Tracing is already active', 'warning');
            }
            return;
        }

        try {
            this.isTracing = true;
            this.startTime = Date.now();
            this.syscalls = [];
            this.callHistory.clear();
            
            await this.initializeTracing();
            this.startMonitoring();
            
            if (this.app) {
                this.updateSessionStatus('tracing');
                this.app.showNotification('System call tracing started', 'success');
            }
            
            console.log('🔍 System call tracing started');
        } catch (error) {
            this.isTracing = false;
            if (this.app) {
                this.app.showNotification(`Failed to start tracing: ${error.message}`, 'error');
            }
        }
    }

    async stop() {
        if (!this.isTracing) {
            return;
        }

        try {
            this.isTracing = false;
            this.stopMonitoring();
            
            await this.finalizeTracing();
            
            if (this.app) {
                this.updateSessionStatus('stopped');
                this.app.showNotification('System call tracing stopped', 'info');
            }
            
            console.log('⏹️ System call tracing stopped');
        } catch (error) {
            if (this.app) {
                this.app.showNotification(`Failed to stop tracing: ${error.message}`, 'error');
            }
        }
    }

    async initializeTracing() {
        try {
            const response = await fetch('/api/debug/syscall/start', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    filters: this.filters,
                    breakpoints: Array.from(this.breakpoints)
                })
            });
            
            if (!response.ok) {
                throw new Error(`Failed to initialize tracing: ${response.statusText}`);
            }
            
            const data = await response.json();
            return data;
        } catch (error) {
            // Mock initialization for demonstration
            console.log('Using mock system call tracing');
            return { success: true, sessionId: 'mock-session' };
        }
    }

    startMonitoring() {
        // Start generating mock system calls
        this.updateTimer = setInterval(() => {
            if (this.isTracing) {
                this.generateMockSyscall();
                this.updateStatistics();
                this.renderSyscallLog();
            }
        }, 100 + Math.random() * 200); // Random interval for realism
    }

    stopMonitoring() {
        if (this.updateTimer) {
            clearInterval(this.updateTimer);
            this.updateTimer = null;
        }
    }

    async finalizeTracing() {
        try {
            const response = await fetch('/api/debug/syscall/stop', {
                method: 'POST'
            });
            
            if (!response.ok) {
                throw new Error(`Failed to finalize tracing: ${response.statusText}`);
            }
        } catch (error) {
            console.warn('Failed to finalize tracing:', error);
        }
    }

    generateMockSyscall() {
        const syscallTemplates = [
            {
                name: 'read',
                params: [
                    () => Math.floor(Math.random() * 1000), // fd
                    () => this.generateBuffer(),            // buf
                    () => Math.floor(Math.random() * 1024)  // count
                ],
                result: () => Math.floor(Math.random() * 100),
                duration: () => Math.random() * 50 + 1
            },
            {
                name: 'write',
                params: [
                    () => Math.floor(Math.random() * 1000), // fd
                    () => this.generateBuffer(),            // buf
                    () => Math.floor(Math.random() * 1024)  // count
                ],
                result: () => Math.floor(Math.random() * 1024),
                duration: () => Math.random() * 30 + 1
            },
            {
                name: 'open',
                params: [
                    () => this.generatePath(),              // pathname
                    () => Math.floor(Math.random() * 8),    // flags
                    () => Math.floor(Math.random() * 0x1FF) // mode
                ],
                result: () => Math.floor(Math.random() * 1000),
                duration: () => Math.random() * 100 + 10
            },
            {
                name: 'close',
                params: [
                    () => Math.floor(Math.random() * 1000)  // fd
                ],
                result: () => 0,
                duration: () => Math.random() * 20 + 1
            },
            {
                name: 'fork',
                params: [],
                result: () => Math.random() > 0.5 ? 0 : Math.floor(Math.random() * 32768),
                duration: () => Math.random() * 200 + 50
            },
            {
                name: 'execve',
                params: [
                    () => this.generatePath(),              // filename
                    () => this.generateArgs(),              // argv
                    () => this.generateEnvp()               // envp
                ],
                result: () => Math.random() > 0.8 ? -1 : 0,
                duration: () => Math.random() * 1000 + 100
            },
            {
                name: 'wait4',
                params: [
                    () => -1,                               // pid
                    () => this.generateStatus(),            // status
                    () => Math.floor(Math.random() * 8),    // options
                    () => this.generateRusage()             // rusage
                ],
                result: () => Math.floor(Math.random() * 32768),
                duration: () => Math.random() * 500 + 10
            },
            {
                name: 'mmap',
                params: [
                    () => 0,                                // addr
                    () => Math.floor(Math.random() * 0x10000) + 0x1000, // length
                    () => Math.floor(Math.random() * 7) + 1, // prot
                    () => Math.floor(Math.random() * 2) + 1, // flags
                    () => Math.floor(Math.random() * 100),  // fd
                    () => Math.floor(Math.random() * 0x1000) // offset
                ],
                result: () => Math.floor(Math.random() * 0x100000000),
                duration: () => Math.random() * 100 + 20
            },
            {
                name: 'munmap',
                params: [
                    () => Math.floor(Math.random() * 0x100000000), // addr
                    () => Math.floor(Math.random() * 0x10000) + 0x1000 // length
                ],
                result: () => 0,
                duration: () => Math.random() * 30 + 5
            },
            {
                name: 'socket',
                params: [
                    () => Math.floor(Math.random() * 3) + 1, // domain
                    () => Math.floor(Math.random() * 3) + 1, // type
                    () => Math.floor(Math.random() * 3) + 1  // protocol
                ],
                result: () => Math.floor(Math.random() * 1000),
                duration: () => Math.random() * 50 + 10
            },
            {
                name: 'connect',
                params: [
                    () => Math.floor(Math.random() * 1000), // sockfd
                    () => this.generateSockaddr(),           // addr
                    () => Math.floor(Math.random() * 64) + 8 // addrlen
                ],
                result: () => Math.random() > 0.7 ? -1 : 0,
                duration: () => Math.random() * 200 + 50
            },
            {
                name: 'recv',
                params: [
                    () => Math.floor(Math.random() * 1000), // sockfd
                    () => this.generateBuffer(),            // buf
                    () => Math.floor(Math.random() * 1024), // len
                    () => Math.floor(Math.random() * 8)     // flags
                ],
                result: () => Math.floor(Math.random() * 1024),
                duration: () => Math.random() * 100 + 20
            },
            {
                name: 'send',
                params: [
                    () => Math.floor(Math.random() * 1000), // sockfd
                    () => this.generateBuffer(),            // buf
                    () => Math.floor(Math.random() * 1024), // len
                    () => Math.floor(Math.random() * 8)     // flags
                ],
                result: () => Math.floor(Math.random() * 1024),
                duration: () => Math.random() * 80 + 10
            },
            {
                name: 'chmod',
                params: [
                    () => this.generatePath(),              // pathname
                    () => Math.floor(Math.random() * 0x1FF) // mode
                ],
                result: () => Math.random() > 0.9 ? -1 : 0,
                duration: () => Math.random() * 100 + 20
            },
            {
                name: 'gettimeofday',
                params: [
                    () => this.generateTimeval(),           // tv
                    () => null                              // tz
                ],
                result: () => 0,
                duration: () => Math.random() * 10 + 1
            }
        ];

        const template = syscallTemplates[Math.floor(Math.random() * syscallTemplates.length)];
        
        const syscall = {
            id: Date.now() + Math.random(),
            name: template.name,
            timestamp: new Date(),
            processId: Math.floor(Math.random() * 10000) + 1000,
            threadId: Math.floor(Math.random() * 100) + 1,
            parameters: template.params.map(paramFunc => paramFunc()),
            result: template.result(),
            duration: template.duration(),
            address: 0x1000 + Math.floor(Math.random() * 0x1000),
            error: null,
            returnType: this.getReturnType(template.name)
        };

        // Add error simulation
        if (Math.random() < 0.05) { // 5% chance of error
            syscall.error = this.generateError();
            syscall.result = -1;
        }

        // Filter syscalls based on current filters
        if (this.shouldFilterSyscall(syscall)) {
            return;
        }

        this.syscalls.unshift(syscall); // Add to beginning
        
        // Limit history
        if (this.syscalls.length > 1000) {
            this.syscalls = this.syscalls.slice(0, 1000);
        }

        // Update history
        const historyEntry = this.callHistory.get(syscall.name) || [];
        historyEntry.push(syscall);
        this.callHistory.set(syscall.name, historyEntry);

        // Check for breakpoints
        if (this.breakpoints.has(syscall.name)) {
            this.triggerBreakpoint(syscall);
        }
    }

    shouldFilterSyscall(syscall) {
        if (!this.filters.enabled) return false;
        
        if (this.filters.syscallNames.size > 0 && 
            !this.filters.syscallNames.has(syscall.name)) {
            return true;
        }
        
        if (this.filters.processId && syscall.processId !== this.filters.processId) {
            return true;
        }
        
        if (this.filters.threadId && syscall.threadId !== this.filters.threadId) {
            return true;
        }
        
        return false;
    }

    generateMockSyscall() {
        // This method is called periodically by the monitoring timer
        setTimeout(() => {
            this.generateMockSyscallInternal();
        }, Math.random() * 500);
    }

    generateMockSyscallInternal() {
        const mockSyscalls = [
            { name: 'read', args: ['fd=3', 'buf=0x7fff...', 'count=1024'], result: 512 },
            { name: 'write', args: ['fd=1', 'buf=0x7fff...', 'count=13'], result: 13 },
            { name: 'open', args: ['path="/tmp/file.txt"', 'flags=O_RDWR', 'mode=0644'], result: 3 },
            { name: 'close', args: ['fd=3'], result: 0 },
            { name: 'fork', args: [], result: 1234 },
            { name: 'execve', args: ['path="/bin/ls"', 'argv=["ls", "-la"]', 'envp=...'], result: 0 },
            { name: 'mmap', args: ['addr=0', 'length=4096', 'prot=PROT_READ|PROT_WRITE'], result: 0x7f8c... },
            { name: 'socket', args: ['domain=AF_INET', 'type=SOCK_STREAM', 'protocol=0'], result: 4 },
            { name: 'connect', args: ['sockfd=4', 'addr=...', 'addrlen=16'], result: 0 },
            { name: 'gettimeofday', args: ['tv=0x7fff...', 'tz=NULL'], result: 0 }
        ];
        
        const syscall = mockSyscalls[Math.floor(Math.random() * mockSyscalls.length)];
        
        const syscallEntry = {
            id: Date.now() + Math.random(),
            name: syscall.name,
            timestamp: new Date().toISOString(),
            args: syscall.args,
            result: syscall.result,
            duration: Math.random() * 100,
            processId: Math.floor(Math.random() * 5000) + 1000
        };
        
        this.syscalls.unshift(syscallEntry);
        
        if (this.syscalls.length > 100) {
            this.syscalls = this.syscalls.slice(0, 100);
        }
    }

    generateBuffer() {
        return `0x${Math.floor(Math.random() * 0xFFFFFFFF).toString(16)}`;
    }

    generatePath() {
        const paths = [
            '/tmp/file.txt',
            '/home/user/document.pdf',
            '/var/log/system.log',
            '/dev/null',
            '/proc/self/maps',
            '/etc/passwd',
            '/lib/libc.so.6',
            '/bin/bash',
            '/usr/bin/python3',
            '/home/user/.bashrc'
        ];
        return paths[Math.floor(Math.random() * paths.length)];
    }

    generateArgs() {
        return `["arg1", "arg2", "arg3"]`;
    }

    generateEnvp() {
        return `["PATH=/usr/bin", "HOME=/home/user", "USER=user"]`;
    }

    generateStatus() {
        return `0x${Math.floor(Math.random() * 0x10000).toString(16)}`;
    }

    generateRusage() {
        return `struct rusage @ 0x${Math.floor(Math.random() * 0xFFFFFFFF).toString(16)}`;
    }

    generateSockaddr() {
        return `struct sockaddr_in @ 0x${Math.floor(Math.random() * 0xFFFFFFFF).toString(16)}`;
    }

    generateTimeval() {
        return `struct timeval {${Math.floor(Math.random() * 1000000)}, ${Math.floor(Math.random() * 1000000)}}`;
    }

    generateError() {
        const errors = ['EACCES', 'ENOENT', 'EIO', 'ENOMEM', 'EBUSY', 'EEXIST'];
        return errors[Math.floor(Math.random() * errors.length)];
    }

    getReturnType(syscallName) {
        const returnTypes = {
            'read': 'ssize_t',
            'write': 'ssize_t',
            'open': 'int',
            'close': 'int',
            'fork': 'pid_t',
            'execve': 'int',
            'mmap': 'void*',
            'socket': 'int',
            'connect': 'int',
            'gettimeofday': 'int'
        };
        return returnTypes[syscallName] || 'long';
    }

    triggerBreakpoint(syscall) {
        if (this.app) {
            this.app.showNotification(`Breakpoint hit: ${syscall.name}`, 'warning');
        }
        
        // Pause tracing
        this.stop();
    }

    renderSyscallLog() {
        const syscallLog = document.getElementById('syscallLog');
        if (!syscallLog) return;
        
        const maxEntries = 20; // Limit display
        const displaySyscalls = this.syscalls.slice(0, maxEntries);
        
        syscallLog.innerHTML = displaySyscalls.map(syscall => `
            <div class="syscall-item ${syscall.error ? 'error' : ''}">
                <div class="syscall-header">
                    <span class="syscall-name">${syscall.name}</span>
                    <span class="syscall-timestamp">${syscall.timestamp.toLocaleTimeString()}</span>
                </div>
                <div class="syscall-details">
                    <div class="syscall-params">
                        ${syscall.parameters ? syscall.parameters.join(', ') : syscall.args ? syscall.args.join(', ') : ''}
                    </div>
                    <div class="syscall-result">
                        = ${syscall.result} ${syscall.error ? `(${syscall.error})` : ''}
                    </div>
                    <div class="syscall-meta">
                        PID: ${syscall.processId} | TID: ${syscall.threadId} | ${syscall.duration.toFixed(2)}ms
                    </div>
                </div>
            </div>
        `).join('');
        
        // Scroll to top to show newest entries
        syscallLog.scrollTop = 0;
    }

    updateStatistics() {
        const now = Date.now();
        const elapsed = (now - this.startTime) / 1000; // seconds
        
        this.statistics.totalCalls = this.syscalls.length;
        this.statistics.callsPerSecond = elapsed > 0 ? this.statistics.totalCalls / elapsed : 0;
        
        // Most frequent syscalls
        const syscallCounts = {};
        this.syscalls.forEach(syscall => {
            syscallCounts[syscall.name] = (syscallCounts[syscall.name] || 0) + 1;
        });
        
        this.statistics.mostFrequent = Object.entries(syscallCounts)
            .sort(([,a], [,b]) => b - a)
            .slice(0, 5)
            .map(([name, count]) => ({ name, count }));
        
        // Average duration
        const durations = this.syscalls.map(s => s.duration).filter(d => d != null);
        this.statistics.avgDuration = durations.length > 0 
            ? durations.reduce((sum, d) => sum + d, 0) / durations.length 
            : 0;
    }

    updateSessionStatus(status) {
        if (this.app && this.app.updateSessionStatus) {
            this.app.updateSessionStatus(status);
        }
    }

    setFilters(filters) {
        this.filters = { ...this.filters, ...filters };
        this.renderSyscallLog();
    }

    addBreakpoint(syscallName) {
        this.breakpoints.add(syscallName);
    }

    removeBreakpoint(syscallName) {
        this.breakpoints.delete(syscallName);
    }

    getFilteredSyscalls() {
        if (!this.filters.enabled) {
            return this.syscalls;
        }
        
        return this.syscalls.filter(syscall => this.shouldFilterSyscall(syscall));
    }

    searchSyscalls(query) {
        const lowerQuery = query.toLowerCase();
        return this.syscalls.filter(syscall => 
            syscall.name.toLowerCase().includes(lowerQuery) ||
            JSON.stringify(syscall.parameters).toLowerCase().includes(lowerQuery) ||
            String(syscall.result).includes(query)
        );
    }

    exportTrace() {
        return {
            sessionStart: this.startTime,
            totalSyscalls: this.syscalls.length,
            statistics: this.statistics,
            syscalls: this.syscalls,
            filters: this.filters,
            breakpoints: Array.from(this.breakpoints)
        };
    }

    importTrace(traceData) {
        try {
            this.syscalls = traceData.syscalls || [];
            this.statistics = traceData.statistics || this.statistics;
            this.filters = traceData.filters || this.filters;
            this.breakpoints = new Set(traceData.breakpoints || []);
            
            this.renderSyscallLog();
            
            if (this.app) {
                this.app.showNotification('Trace data imported', 'success');
            }
        } catch (error) {
            if (this.app) {
                this.app.showNotification(`Failed to import trace: ${error.message}`, 'error');
            }
        }
    }

    analyzePerformance() {
        if (this.syscalls.length === 0) {
            return null;
        }
        
        const analysis = {
            totalDuration: this.syscalls.reduce((sum, s) => sum + s.duration, 0),
            avgDuration: this.statistics.avgDuration,
            slowestSyscalls: this.syscalls
                .filter(s => s.duration != null)
                .sort((a, b) => b.duration - a.duration)
                .slice(0, 5),
            errorRate: (this.syscalls.filter(s => s.error).length / this.syscalls.length) * 100,
            syscallDistribution: {}
        };
        
        // Distribution by syscall name
        this.syscalls.forEach(syscall => {
            analysis.syscallDistribution[syscall.name] = (analysis.syscallDistribution[syscall.name] || 0) + 1;
        });
        
        return analysis;
    }

    detectAnomalies() {
        const anomalies = [];
        
        // Long duration anomalies
        const durationThreshold = this.statistics.avgDuration * 3;
        this.syscalls.forEach(syscall => {
            if (syscall.duration > durationThreshold && syscall.duration != null) {
                anomalies.push({
                    type: 'slow_syscall',
                    syscall: syscall.name,
                    duration: syscall.duration,
                    threshold: durationThreshold,
                    description: `System call ${syscall.name} took unusually long (${syscall.duration.toFixed(2)}ms)`
                });
            }
        });
        
        // High frequency anomalies
        const recentSyscalls = this.syscalls.slice(0, 10);
        const frequencyThreshold = 3;
        
        recentSyscalls.forEach(syscall => {
            const recentCount = recentSyscalls.filter(s => s.name === syscall.name).length;
            if (recentCount > frequencyThreshold) {
                anomalies.push({
                    type: 'high_frequency',
                    syscall: syscall.name,
                    count: recentCount,
                    threshold: frequencyThreshold,
                    description: `High frequency of ${syscall.name} calls detected`
                });
            }
        });
        
        // Error patterns
        const errors = this.syscalls.filter(s => s.error);
        if (errors.length > this.syscalls.length * 0.1) {
            anomalies.push({
                type: 'high_error_rate',
                errorRate: (errors.length / this.syscalls.length) * 100,
                description: 'High error rate in system calls'
            });
        }
        
        return anomalies;
    }

    clearTrace() {
        this.syscalls = [];
        this.callHistory.clear();
        this.statistics = {
            totalCalls: 0,
            callsPerSecond: 0,
            mostFrequent: [],
            avgDuration: 0
        };
        this.renderSyscallLog();
    }

    destroy() {
        this.stop();
        this.syscalls = [];
        this.callHistory.clear();
        this.breakpoints.clear();
        this.filters.syscallNames.clear();
    }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = SyscallTracer;
}