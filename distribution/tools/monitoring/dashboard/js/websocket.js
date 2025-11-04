// WebSocket handler for real-time monitoring updates
// Handles live data streaming and connection management

class WebSocketManager {
    constructor(url = null, options = {}) {
        this.url = url || this.getWebSocketUrl();
        this.options = {
            reconnectInterval: options.reconnectInterval || 5000,
            maxReconnectAttempts: options.maxReconnectAttempts || 10,
            heartbeatInterval: options.heartbeatInterval || 30000,
            timeout: options.timeout || 30000,
            ...options
        };
        
        this.ws = null;
        this.reconnectAttempts = 0;
        this.heartbeatInterval = null;
        this.reconnectTimer = null;
        this.isConnecting = false;
        this.isClosed = false;
        
        this.callbacks = {
            onOpen: [],
            onMessage: [],
            onClose: [],
            onError: [],
            onReconnect: [],
            onHeartbeat: []
        };
        
        this.pendingMessages = [];
        this.isAuthenticated = false;
        this.authToken = null;
        
        this.connect();
    }

    getWebSocketUrl() {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const host = window.location.host;
        return `${protocol}//${host}/ws/monitoring`;
    }

    connect() {
        if (this.isConnecting || this.isClosed) return;
        
        this.isConnecting = true;
        console.log('Connecting to WebSocket:', this.url);
        
        try {
            this.ws = new WebSocket(this.url);
            
            this.ws.onopen = (event) => {
                console.log('WebSocket connected');
                this.isConnecting = false;
                this.isAuthenticated = false;
                this.reconnectAttempts = 0;
                this.clearReconnectTimer();
                this.startHeartbeat();
                
                // Send authentication if token is available
                if (this.authToken) {
                    this.send({
                        type: 'auth',
                        token: this.authToken
                    });
                }
                
                // Send pending messages
                this.flushPendingMessages();
                
                this.triggerCallbacks('onOpen', event);
            };
            
            this.ws.onmessage = (event) => {
                this.handleMessage(event);
            };
            
            this.ws.onclose = (event) => {
                console.log('WebSocket disconnected:', event.code, event.reason);
                this.isConnecting = false;
                this.isAuthenticated = false;
                this.stopHeartbeat();
                
                this.triggerCallbacks('onClose', event);
                
                // Attempt to reconnect if not intentionally closed
                if (!this.isClosed && event.code !== 1000) {
                    this.scheduleReconnect();
                }
            };
            
            this.ws.onerror = (error) => {
                console.error('WebSocket error:', error);
                this.triggerCallbacks('onError', error);
            };
            
        } catch (error) {
            console.error('Failed to create WebSocket:', error);
            this.isConnecting = false;
            this.scheduleReconnect();
        }
    }

    handleMessage(event) {
        try {
            const data = JSON.parse(event.data);
            
            // Handle authentication response
            if (data.type === 'auth_response') {
                if (data.success) {
                    this.isAuthenticated = true;
                    console.log('WebSocket authenticated');
                } else {
                    console.error('WebSocket authentication failed:', data.error);
                    this.triggerCallbacks('onError', new Error('Authentication failed'));
                }
                return;
            }
            
            // Handle heartbeat responses
            if (data.type === 'heartbeat') {
                this.triggerCallbacks('onHeartbeat', data);
                return;
            }
            
            // Handle regular messages
            this.triggerCallbacks('onMessage', data);
            
        } catch (error) {
            console.error('Error parsing WebSocket message:', error);
        }
    }

    send(message) {
        if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
            // Queue message for when connection is established
            this.pendingMessages.push(message);
            return false;
        }
        
        try {
            this.ws.send(JSON.stringify(message));
            return true;
        } catch (error) {
            console.error('Error sending WebSocket message:', error);
            return false;
        }
    }

    sendHeartbeat() {
        this.send({
            type: 'heartbeat',
            timestamp: Date.now()
        });
    }

    startHeartbeat() {
        this.stopHeartbeat();
        this.heartbeatInterval = setInterval(() => {
            this.sendHeartbeat();
        }, this.options.heartbeatInterval);
    }

    stopHeartbeat() {
        if (this.heartbeatInterval) {
            clearInterval(this.heartbeatInterval);
            this.heartbeatInterval = null;
        }
    }

    scheduleReconnect() {
        if (this.reconnectAttempts >= this.options.maxReconnectAttempts) {
            console.error('Max reconnection attempts reached');
            return;
        }
        
        this.reconnectAttempts++;
        const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);
        
        console.log(`Scheduling reconnection attempt ${this.reconnectAttempts} in ${delay}ms`);
        
        this.reconnectTimer = setTimeout(() => {
            this.triggerCallbacks('onReconnect', {
                attempt: this.reconnectAttempts,
                delay: delay
            });
            this.connect();
        }, delay);
    }

    clearReconnectTimer() {
        if (this.reconnectTimer) {
            clearTimeout(this.reconnectTimer);
            this.reconnectTimer = null;
        }
    }

    flushPendingMessages() {
        while (this.pendingMessages.length > 0) {
            const message = this.pendingMessages.shift();
            this.send(message);
        }
    }

    close() {
        this.isClosed = true;
        this.clearReconnectTimer();
        this.stopHeartbeat();
        
        if (this.ws) {
            this.ws.close(1000, 'Client closing connection');
            this.ws = null;
        }
    }

    // Authentication
    authenticate(token) {
        this.authToken = token;
        if (this.ws && this.ws.readyState === WebSocket.OPEN && !this.isAuthenticated) {
            this.send({
                type: 'auth',
                token: token
            });
        }
    }

    // Event handlers
    onOpen(callback) {
        this.callbacks.onOpen.push(callback);
    }

    onMessage(callback) {
        this.callbacks.onMessage.push(callback);
    }

    onClose(callback) {
        this.callbacks.onClose.push(callback);
    }

    onError(callback) {
        this.callbacks.onError.push(callback);
    }

    onReconnect(callback) {
        this.callbacks.onReconnect.push(callback);
    }

    onHeartbeat(callback) {
        this.callbacks.onHeartbeat.push(callback);
    }

    triggerCallbacks(eventType, data) {
        if (this.callbacks[eventType]) {
            this.callbacks[eventType].forEach(callback => {
                try {
                    callback(data);
                } catch (error) {
                    console.error(`Error in ${eventType} callback:`, error);
                }
            });
        }
    }

    // Connection status
    isConnected() {
        return this.ws && this.ws.readyState === WebSocket.OPEN && this.isAuthenticated;
    }

    getReadyState() {
        if (!this.ws) return WebSocket.CLOSED;
        return this.ws.readyState;
    }

    getReadyStateText() {
        const states = {
            [WebSocket.CONNECTING]: 'CONNECTING',
            [WebSocket.OPEN]: 'OPEN',
            [WebSocket.CLOSING]: 'CLOSING',
            [WebSocket.CLOSED]: 'CLOSED'
        };
        return states[this.getReadyState()] || 'UNKNOWN';
    }

    // Metrics
    getConnectionMetrics() {
        return {
            url: this.url,
            readyState: this.getReadyState(),
            readyStateText: this.getReadyStateText(),
            reconnectAttempts: this.reconnectAttempts,
            isAuthenticated: this.isAuthenticated,
            pendingMessages: this.pendingMessages.length,
            heartbeatInterval: this.options.heartbeatInterval
        };
    }

    // Reconfigure options
    updateOptions(newOptions) {
        this.options = { ...this.options, ...newOptions };
        
        // Restart heartbeat with new interval if changed
        if (newOptions.heartbeatInterval && this.heartbeatInterval) {
            this.startHeartbeat();
        }
    }

    // Manual reconnection
    reconnect() {
        if (this.ws) {
            this.ws.close();
        }
        this.reconnectAttempts = 0;
        this.connect();
    }
}

// Message routing and filtering
class MessageRouter {
    constructor() {
        this.routes = new Map();
        this.filters = [];
    }

    addRoute(type, handler, filter = null) {
        const routeKey = `${type}_${handler.toString().length}`;
        this.routes.set(routeKey, {
            type: type,
            handler: handler,
            filter: filter,
            messageCount: 0,
            lastMessage: null
        });
    }

    removeRoute(type, handler) {
        for (const [key, route] of this.routes.entries()) {
            if (route.type === type && route.handler === handler) {
                this.routes.delete(key);
                break;
            }
        }
    }

    addFilter(filter) {
        this.filters.push(filter);
    }

    removeFilter(filter) {
        const index = this.filters.indexOf(filter);
        if (index > -1) {
            this.filters.splice(index, 1);
        }
    }

    routeMessage(message) {
        // Apply filters
        for (const filter of this.filters) {
            if (!filter(message)) {
                return false;
            }
        }

        // Route to appropriate handlers
        let routed = false;
        for (const route of this.routes.values()) {
            if (route.type === message.type) {
                try {
                    // Apply route-specific filter if exists
                    if (!route.filter || route.filter(message)) {
                        route.handler(message);
                        route.messageCount++;
                        route.lastMessage = message;
                        routed = true;
                    }
                } catch (error) {
                    console.error('Error in message handler:', error);
                }
            }
        }

        return routed;
    }

    getMetrics() {
        const routes = [];
        for (const [key, route] of this.routes.entries()) {
            routes.push({
                type: route.type,
                messageCount: route.messageCount,
                lastMessage: route.lastMessage ? route.lastMessage.timestamp : null
            });
        }
        
        return {
            totalRoutes: routes.length,
            routes: routes,
            filters: this.filters.length
        };
    }
}

// Connection manager for multiple WebSocket connections
class ConnectionManager {
    constructor() {
        this.connections = new Map();
        this.messageRouter = new MessageRouter();
        this.globalHandlers = {
            onOpen: [],
            onClose: [],
            onError: []
        };
    }

    createConnection(name, url, options = {}) {
        const wsManager = new WebSocketManager(url, options);
        
        // Set up global handlers
        wsManager.onOpen((event) => {
            this.triggerGlobalHandlers('onOpen', { name, event });
        });
        
        wsManager.onClose((event) => {
            this.triggerGlobalHandlers('onClose', { name, event });
        });
        
        wsManager.onError((error) => {
            this.triggerGlobalHandlers('onError', { name, error });
        });
        
        // Set up message routing
        wsManager.onMessage((message) => {
            this.messageRouter.routeMessage({
                ...message,
                connection: name,
                timestamp: Date.now()
            });
        });
        
        this.connections.set(name, wsManager);
        return wsManager;
    }

    getConnection(name) {
        return this.connections.get(name);
    }

    removeConnection(name) {
        const connection = this.connections.get(name);
        if (connection) {
            connection.close();
            this.connections.delete(name);
        }
    }

    broadcast(message, excludeConnections = []) {
        for (const [name, connection] of this.connections.entries()) {
            if (!excludeConnections.includes(name) && connection.isConnected()) {
                connection.send(message);
            }
        }
    }

    addGlobalHandler(event, handler) {
        if (this.globalHandlers[event]) {
            this.globalHandlers[event].push(handler);
        }
    }

    triggerGlobalHandlers(event, data) {
        if (this.globalHandlers[event]) {
            this.globalHandlers[event].forEach(handler => {
                try {
                    handler(data);
                } catch (error) {
                    console.error(`Error in global ${event} handler:`, error);
                }
            });
        }
    }

    getAllMetrics() {
        const metrics = {
            totalConnections: this.connections.size,
            connections: {}
        };
        
        for (const [name, connection] of this.connections.entries()) {
            metrics.connections[name] = connection.getConnectionMetrics();
        }
        
        return metrics;
    }
}

// Export for global use
if (typeof window !== 'undefined') {
    window.WebSocketManager = WebSocketManager;
    window.MessageRouter = MessageRouter;
    window.ConnectionManager = ConnectionManager;
    
    // Create global instances
    window.connectionManager = new ConnectionManager();
    window.messageRouter = new MessageRouter();
}