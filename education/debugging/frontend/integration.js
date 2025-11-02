/**
 * MultiOS Integration Module
 * Handles integration with MultiOS UI and external systems
 */

class MultiOSIntegration {
    constructor() {
        this.app = null;
        this.isIntegrated = false;
        this.apiEndpoint = 'http://localhost:8000/api';
        this.settings = {
            uiTheme: 'light',
            panelLayout: 'default',
            autoSync: false,
            connectionTimeout: 5000,
            retryAttempts: 3
        };
        this.connectionStatus = 'disconnected';
        this.eventListeners = new Map();
        this.integrationPoints = new Map();
        this.syncQueue = [];
        this.lastSync = null;
        
        // Integration capabilities
        this.capabilities = {
            uiIntegration: true,
            dataSync: true,
            eventBroadcast: true,
            remoteDebugging: false,
            fileSystemAccess: false,
            processControl: false
        };
    }

    setApp(app) {
        this.app = app;
        this.initializeIntegration();
    }

    async initializeIntegration() {
        try {
            // Load saved settings
            await this.loadSettings();
            
            // Set up integration points
            this.setupIntegrationPoints();
            
            // Initialize event system
            this.setupEventSystem();
            
            // Set up periodic sync if enabled
            if (this.settings.autoSync) {
                this.startPeriodicSync();
            }
            
            console.log('🔗 MultiOS integration initialized');
        } catch (error) {
            console.error('Failed to initialize MultiOS integration:', error);
        }
    }

    async loadSettings() {
        try {
            const saved = localStorage.getItem('multios_integration_settings');
            if (saved) {
                this.settings = { ...this.settings, ...JSON.parse(saved) };
                this.apiEndpoint = this.settings.apiEndpoint;
            }
        } catch (error) {
            console.warn('Failed to load integration settings:', error);
        }
    }

    async saveSettings(settings) {
        try {
            this.settings = { ...this.settings, ...settings };
            localStorage.setItem('multios_integration_settings', JSON.stringify(this.settings));
            
            if (this.isConnected()) {
                // Sync settings to MultiOS
                await this.syncSettings();
            }
            
            console.log('💾 Integration settings saved');
        } catch (error) {
            throw new Error(`Failed to save settings: ${error.message}`);
        }
    }

    setupIntegrationPoints() {
        // Define integration points with MultiOS UI
        this.integrationPoints.set('debugger_panel', {
            name: 'Debugger Panel Integration',
            description: 'Integrate debugging interface into MultiOS panel system',
            enabled: true,
            priority: 'high'
        });
        
        this.integrationPoints.set('file_system', {
            name: 'File System Integration',
            description: 'Access files through MultiOS file system interface',
            enabled: true,
            priority: 'medium'
        });
        
        this.integrationPoints.set('terminal', {
            name: 'Terminal Integration',
            description: 'Use MultiOS terminal for debugging commands',
            enabled: true,
            priority: 'high'
        });
        
        this.integrationPoints.set('process_manager', {
            name: 'Process Manager Integration',
            description: 'Control processes through MultiOS process manager',
            enabled: true,
            priority: 'medium'
        });
        
        this.integrationPoints.set('system_monitor', {
            name: 'System Monitor Integration',
            description: 'Display system monitoring data in debugging context',
            enabled: true,
            priority: 'low'
        });
    }

    setupEventSystem() {
        // MultiOS event types that the debugging system can listen to
        const eventTypes = [
            'window_resized',
            'theme_changed',
            'panel_opened',
            'panel_closed',
            'file_opened',
            'file_saved',
            'process_started',
            'process_stopped',
            'system_resource_changed'
        ];
        
        eventTypes.forEach(eventType => {
            this.eventListeners.set(eventType, []);
        });
    }

    async connect() {
        try {
            this.updateConnectionStatus('connecting');
            
            const response = await this.makeRequest('/status', 'GET');
            
            if (response.status === 'ok') {
                this.isIntegrated = true;
                this.updateConnectionStatus('connected');
                
                // Initialize integrated features
                await this.initializeIntegratedFeatures();
                
                console.log('✅ Connected to MultiOS');
                return true;
            } else {
                throw new Error('Invalid response from MultiOS');
            }
        } catch (error) {
            this.updateConnectionStatus('error');
            this.isIntegrated = false;
            
            console.error('Failed to connect to MultiOS:', error);
            throw error;
        }
    }

    async disconnect() {
        try {
            await this.makeRequest('/integration/disconnect', 'POST');
            this.isIntegrated = false;
            this.updateConnectionStatus('disconnected');
            
            // Clean up integrated features
            this.cleanupIntegratedFeatures();
            
            console.log('🔌 Disconnected from MultiOS');
        } catch (error) {
            console.error('Error during disconnection:', error);
        }
    }

    isConnected() {
        return this.isIntegrated && this.connectionStatus === 'connected';
    }

    updateConnectionStatus(status) {
        this.connectionStatus = status;
        
        // Dispatch event to listeners
        this.dispatchEvent('connection_status_changed', {
            status,
            timestamp: new Date()
        });
    }

    async makeRequest(endpoint, method = 'GET', data = null) {
        const url = `${this.apiEndpoint}${endpoint}`;
        const options = {
            method,
            headers: {
                'Content-Type': 'application/json',
                'X-Integration-Token': this.getIntegrationToken()
            }
        };
        
        if (data) {
            options.body = JSON.stringify(data);
        }
        
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), this.settings.connectionTimeout);
        
        try {
            const response = await fetch(url, { ...options, signal: controller.signal });
            clearTimeout(timeoutId);
            
            if (!response.ok) {
                throw new Error(`HTTP ${response.status}: ${response.statusText}`);
            }
            
            return await response.json();
        } catch (error) {
            clearTimeout(timeoutId);
            
            if (error.name === 'AbortError') {
                throw new Error('Request timeout');
            }
            
            throw error;
        }
    }

    getIntegrationToken() {
        // In a real implementation, this would get the actual token
        return 'debug-system-integration-token';
    }

    async initializeIntegratedFeatures() {
        if (!this.isConnected()) return;
        
        try {
            // Register debugging interface with MultiOS
            await this.registerInterface();
            
            // Set up file system integration
            await this.setupFileSystemIntegration();
            
            // Set up terminal integration
            await this.setupTerminalIntegration();
            
            // Set up process manager integration
            await this.setupProcessManagerIntegration();
            
        } catch (error) {
            console.warn('Some integrated features failed to initialize:', error);
        }
    }

    async registerInterface() {
        const registration = {
            name: 'Interactive OS Debugging System',
            version: '1.0.0',
            capabilities: this.capabilities,
            integrationPoints: Array.from(this.integrationPoints.keys()),
            endpoint: '/debug-system'
        };
        
        return await this.makeRequest('/integration/register', 'POST', registration);
    }

    async setupFileSystemIntegration() {
        // Expose file operations to MultiOS
        await this.makeRequest('/integration/filesystem/register', 'POST', {
            handlers: {
                'read_file': this.handleFileRead.bind(this),
                'write_file': this.handleFileWrite.bind(this),
                'list_directory': this.handleListDirectory.bind(this)
            }
        });
    }

    async setupTerminalIntegration() {
        // Register terminal command handlers
        await this.makeRequest('/integration/terminal/register', 'POST', {
            commands: {
                'debug': this.handleDebugCommand.bind(this),
                'breakpoint': this.handleBreakpointCommand.bind(this),
                'variable': this.handleVariableCommand.bind(this)
            }
        });
    }

    async setupProcessManagerIntegration() {
        // Register process monitoring handlers
        await this.makeRequest('/integration/processes/register', 'POST', {
            handlers: {
                'get_processes': this.handleGetProcesses.bind(this),
                'debug_process': this.handleDebugProcess.bind(this)
            }
        });
    }

    cleanupIntegratedFeatures() {
        if (!this.isConnected()) return;
        
        // Unregister from MultiOS
        this.makeRequest('/integration/unregister', 'POST').catch(console.warn);
        
        // Clean up event listeners
        this.eventListeners.clear();
    }

    // File system integration handlers
    async handleFileRead(data) {
        try {
            const { path } = data;
            
            if (this.app && this.app.debugCore) {
                const content = await this.app.debugCore.loadFile(path);
                return {
                    success: true,
                    content,
                    metadata: {
                        size: content.length,
                        modified: new Date().toISOString()
                    }
                };
            }
            
            throw new Error('Debug core not available');
        } catch (error) {
            return {
                success: false,
                error: error.message
            };
        }
    }

    async handleFileWrite(data) {
        try {
            const { path, content } = data;
            
            // Mock file write
            console.log(`📝 Writing file: ${path}`);
            
            return {
                success: true,
                bytesWritten: content.length
            };
        } catch (error) {
            return {
                success: false,
                error: error.message
            };
        }
    }

    async handleListDirectory(data) {
        try {
            const { path } = data;
            
            // Mock directory listing
            const files = [
                { name: 'main.c', type: 'file', size: 1024 },
                { name: 'debug.c', type: 'file', size: 2048 },
                { name: 'src/', type: 'directory' }
            ];
            
            return {
                success: true,
                files
            };
        } catch (error) {
            return {
                success: false,
                error: error.message
            };
        }
    }

    // Terminal integration handlers
    async handleDebugCommand(data) {
        const { args } = data;
        
        try {
            if (this.app && this.app.gdbConsole) {
                const command = args.join(' ');
                const result = await this.app.gdbConsole.executeCommand(command);
                return {
                    success: true,
                    output: result
                };
            }
            
            throw new Error('GDB console not available');
        } catch (error) {
            return {
                success: false,
                error: error.message
            };
        }
    }

    async handleBreakpointCommand(data) {
        const { action, location } = data;
        
        try {
            if (this.app && this.app.debugCore) {
                if (action === 'add') {
                    const bp = await this.app.debugCore.addBreakpoint(parseInt(location));
                    return {
                        success: true,
                        breakpoint: bp
                    };
                } else if (action === 'remove') {
                    this.app.debugCore.removeBreakpoint(parseInt(location));
                    return {
                        success: true
                    };
                }
            }
            
            throw new Error('Debug core not available');
        } catch (error) {
            return {
                success: false,
                error: error.message
            };
        }
    }

    async handleVariableCommand(data) {
        const { action, variable } = data;
        
        try {
            if (this.app && this.app.variableInspector) {
                if (action === 'print') {
                    const value = this.app.variableInspector.evaluateWatchExpression(variable);
                    return {
                        success: true,
                        value
                    };
                } else if (action === 'watch') {
                    this.app.variableInspector.addWatchExpression(variable);
                    return {
                        success: true
                    };
                }
            }
            
            throw new Error('Variable inspector not available');
        } catch (error) {
            return {
                success: false,
                error: error.message
            };
        }
    }

    // Process manager integration handlers
    async handleGetProcesses(data) {
        try {
            if (this.app && this.app.debugCore) {
                const processes = await this.app.debugCore.getProcesses();
                return {
                    success: true,
                    processes
                };
            }
            
            throw new Error('Debug core not available');
        } catch (error) {
            return {
                success: false,
                error: error.message
            };
        }
    }

    async handleDebugProcess(data) {
        const { pid, action } = data;
        
        try {
            if (this.app && this.app.debugCore) {
                if (action === 'attach') {
                    // Mock attach
                    return {
                        success: true,
                        message: `Attached to process ${pid}`
                    };
                } else if (action === 'detach') {
                    // Mock detach
                    return {
                        success: true,
                        message: `Detached from process ${pid}`
                    };
                }
            }
            
            throw new Error('Debug core not available');
        } catch (error) {
            return {
                success: false,
                error: error.message
            };
        }
    }

    // Sync operations
    startPeriodicSync() {
        if (this.syncInterval) {
            clearInterval(this.syncInterval);
        }
        
        this.syncInterval = setInterval(() => {
            this.syncDebuggingState();
        }, 30000); // Sync every 30 seconds
    }

    stopPeriodicSync() {
        if (this.syncInterval) {
            clearInterval(this.syncInterval);
            this.syncInterval = null;
        }
    }

    async syncDebuggingState() {
        if (!this.isConnected()) return;
        
        try {
            const state = this.exportDebuggingState();
            
            await this.makeRequest('/integration/sync', 'POST', {
                type: 'debugging_state',
                data: state,
                timestamp: new Date().toISOString()
            });
            
            this.lastSync = new Date();
            
        } catch (error) {
            console.warn('Failed to sync debugging state:', error);
        }
    }

    exportDebuggingState() {
        const state = {
            session: {},
            breakpoints: [],
            variables: [],
            memory: {},
            syscalls: [],
            tutorials: {},
            scenarios: {}
        };
        
        if (this.app) {
            // Export state from all components
            if (this.app.debugCore) {
                state.session = this.app.debugCore.getSessionData();
                state.breakpoints = this.app.debugCore.getBreakpoints();
            }
            
            if (this.app.variableInspector) {
                state.variables = this.app.variableInspector.getVariables();
            }
            
            if (this.app.memoryView) {
                state.memory = this.app.memoryView.exportMemorySnapshot();
            }
            
            if (this.app.syscallTracer) {
                state.syscalls = this.app.syscallTracer.getFilteredSyscalls();
            }
            
            if (this.app.guidedTutorials) {
                state.tutorials = this.app.guidedTutorials.exportTutorialData();
            }
            
            if (this.app.scenarios) {
                state.scenarios = this.app.scenarios.exportScenarioData();
            }
        }
        
        return state;
    }

    async importDebuggingState(stateData) {
        try {
            if (stateData.breakpoints && this.app?.debugCore) {
                // Restore breakpoints
                // Implementation would depend on debugCore API
            }
            
            if (stateData.variables && this.app?.variableInspector) {
                // Restore variables
                this.app.variableInspector.importVariableState(stateData.variables);
            }
            
            if (stateData.memory && this.app?.memoryView) {
                // Restore memory state
                this.app.memoryView.importMemorySnapshot(stateData.memory);
            }
            
            if (stateData.tutorials && this.app?.guidedTutorials) {
                // Restore tutorial progress
                this.app.guidedTutorials.importTutorialData(stateData.tutorials);
            }
            
            if (stateData.scenarios && this.app?.scenarios) {
                // Restore scenario progress
                this.app.scenarios.importScenarioData(stateData.scenarios);
            }
            
            console.log('📥 Imported debugging state from MultiOS');
            
        } catch (error) {
            console.error('Failed to import debugging state:', error);
            throw error;
        }
    }

    async syncSettings() {
        if (!this.isConnected()) return;
        
        try {
            await this.makeRequest('/integration/settings', 'POST', {
                settings: this.settings,
                timestamp: new Date().toISOString()
            });
        } catch (error) {
            console.warn('Failed to sync settings:', error);
        }
    }

    // Event system
    addEventListener(eventType, listener) {
        if (!this.eventListeners.has(eventType)) {
            this.eventListeners.set(eventType, []);
        }
        this.eventListeners.get(eventType).push(listener);
    }

    removeEventListener(eventType, listener) {
        if (this.eventListeners.has(eventType)) {
            const listeners = this.eventListeners.get(eventType);
            const index = listeners.indexOf(listener);
            if (index > -1) {
                listeners.splice(index, 1);
            }
        }
    }

    dispatchEvent(eventType, data) {
        if (this.eventListeners.has(eventType)) {
            this.eventListeners.get(eventType).forEach(listener => {
                try {
                    listener(data);
                } catch (error) {
                    console.warn('Event listener error:', error);
                }
            });
        }
    }

    // UI integration
    async applyTheme(theme) {
        this.settings.uiTheme = theme;
        
        const body = document.body;
        
        switch (theme) {
            case 'dark':
                body.classList.add('theme-dark');
                body.classList.remove('theme-high-contrast');
                break;
            case 'high-contrast':
                body.classList.add('theme-high-contrast');
                body.classList.remove('theme-dark');
                break;
            default:
                body.classList.remove('theme-dark', 'theme-high-contrast');
        }
        
        // Notify MultiOS of theme change
        if (this.isConnected()) {
            this.dispatchEvent('theme_changed', { theme });
        }
    }

    async applyPanelLayout(layout) {
        this.settings.panelLayout = layout;
        
        const container = document.querySelector('.debug-container');
        if (!container) return;
        
        switch (layout) {
            case 'compact':
                container.classList.add('layout-compact');
                container.classList.remove('layout-fullscreen');
                break;
            case 'fullscreen':
                container.classList.add('layout-fullscreen');
                container.classList.remove('layout-compact');
                break;
            default:
                container.classList.remove('layout-compact', 'layout-fullscreen');
        }
        
        // Notify MultiOS of layout change
        if (this.isConnected()) {
            this.dispatchEvent('layout_changed', { layout });
        }
    }

    // Connection management
    async testConnection(endpoint) {
        const testEndpoint = endpoint || this.apiEndpoint;
        
        try {
            const controller = new AbortController();
            const timeoutId = setTimeout(() => controller.abort(), 5000);
            
            const response = await fetch(`${testEndpoint}/integration/test`, {
                method: 'GET',
                signal: controller.signal
            });
            
            clearTimeout(timeoutId);
            
            if (response.ok) {
                const data = await response.json();
                return {
                    success: true,
                    version: data.version,
                    capabilities: data.capabilities
                };
            } else {
                throw new Error(`HTTP ${response.status}: ${response.statusText}`);
            }
        } catch (error) {
            return {
                success: false,
                error: error.message
            };
        }
    }

    // Status and diagnostics
    getConnectionStatus() {
        return {
            status: this.connectionStatus,
            isConnected: this.isConnected(),
            lastSync: this.lastSync,
            endpoint: this.apiEndpoint,
            settings: this.settings,
            capabilities: this.capabilities,
            integrationPoints: Array.from(this.integrationPoints.entries())
        };
    }

    getDiagnostics() {
        return {
            timestamp: new Date().toISOString(),
            connection: this.getConnectionStatus(),
            queueSize: this.syncQueue.length,
            activeListeners: Object.fromEntries(
                Array.from(this.eventListeners.entries()).map(([type, listeners]) => [
                    type, listeners.length
                ])
            ),
            memoryUsage: this.getMemoryUsage()
        };
    }

    getMemoryUsage() {
        if (performance.memory) {
            return {
                used: Math.round(performance.memory.usedJSHeapSize / 1024 / 1024),
                total: Math.round(performance.memory.totalJSHeapSize / 1024 / 1024),
                limit: Math.round(performance.memory.jsHeapSizeLimit / 1024 / 1024)
            };
        }
        return null;
    }

    // Cleanup
    destroy() {
        this.stopPeriodicSync();
        this.cleanupIntegratedFeatures();
        this.eventListeners.clear();
        this.syncQueue = [];
        
        if (this.app) {
            this.app = null;
        }
    }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = MultiOSIntegration;
}