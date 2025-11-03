// MultiOS Monitoring Dashboard JavaScript
// Real-time monitoring dashboard with comprehensive analytics

class MonitoringDashboard {
    constructor() {
        this.websocket = null;
        this.charts = new Map();
        this.updateInterval = null;
        this.settings = {
            theme: localStorage.getItem('dashboard_theme') || 'dark',
            refreshRate: parseInt(localStorage.getItem('dashboard_refresh_rate')) || 5,
            enableNotifications: localStorage.getItem('dashboard_notifications') === 'true',
            enableSound: localStorage.getItem('dashboard_sound') === 'true'
        };
        
        this.init();
    }

    init() {
        this.setupEventListeners();
        this.setupWebSocket();
        this.initializeCharts();
        this.startDataCollection();
        this.loadInitialData();
        this.applyTheme();
    }

    setupEventListeners() {
        // Tab navigation
        document.querySelectorAll('.nav-tabs li').forEach(tab => {
            tab.addEventListener('click', (e) => this.switchTab(e.target.dataset.tab));
        });

        // Refresh button
        document.getElementById('refreshBtn')?.addEventListener('click', () => this.refreshData());

        // Settings modal
        document.getElementById('settingsBtn')?.addEventListener('click', () => this.openSettings());
        document.getElementById('closeSettings')?.addEventListener('click', () => this.closeSettings());
        document.getElementById('saveSettings')?.addEventListener('click', () => this.saveSettings());
        document.getElementById('resetSettings')?.addEventListener('click', () => this.resetSettings());

        // Modal close on outside click
        window.addEventListener('click', (e) => {
            const modal = document.getElementById('settingsModal');
            if (e.target === modal) this.closeSettings();
        });

        // Filter and search controls
        this.setupFilterControls();

        // Alert management
        this.setupAlertControls();

        // Export functionality
        document.getElementById('exportData')?.addEventListener('click', () => this.exportData());

        // Log controls
        this.setupLogControls();

        // Report generation
        document.getElementById('generateReport')?.addEventListener('click', () => this.generateReport());
    }

    setupFilterControls() {
        // Time range selector
        const timeRangeSelect = document.getElementById('timeRange');
        if (timeRangeSelect) {
            timeRangeSelect.addEventListener('change', (e) => this.updateTimeRange(e.target.value));
        }

        // Log filters
        const logLevel = document.getElementById('logLevel');
        if (logLevel) {
            logLevel.addEventListener('change', (e) => this.filterLogs(e.target.value));
        }

        const logSearch = document.getElementById('logSearch');
        if (logSearch) {
            logSearch.addEventListener('input', (e) => this.searchLogs(e.target.value));
        }

        // Alert filters
        this.setupAlertFilters();
    }

    setupAlertFilters() {
        const alertSeverity = document.getElementById('alertSeverity');
        const alertStatus = document.getElementById('alertStatus');
        
        if (alertSeverity) {
            alertSeverity.addEventListener('change', (e) => this.filterAlerts('severity', e.target.value));
        }
        
        if (alertStatus) {
            alertStatus.addEventListener('change', (e) => this.filterAlerts('status', e.target.value));
        }
    }

    setupAlertControls() {
        document.getElementById('acknowledgeAlert')?.addEventListener('click', () => this.acknowledgeSelectedAlert());
        document.getElementById('resolveAlert')?.addEventListener('click', () => this.resolveSelectedAlert());
    }

    setupLogControls() {
        document.getElementById('clearLogs')?.addEventListener('click', () => this.clearLogs());
    }

    setupWebSocket() {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws/monitoring`;
        
        this.websocket = new WebSocket(wsUrl);
        
        this.websocket.onopen = () => {
            console.log('WebSocket connected');
            this.updateConnectionStatus('connected');
        };

        this.websocket.onmessage = (event) => {
            const data = JSON.parse(event.data);
            this.handleRealtimeUpdate(data);
        };

        this.websocket.onclose = () => {
            console.log('WebSocket disconnected');
            this.updateConnectionStatus('disconnected');
            // Attempt to reconnect after 5 seconds
            setTimeout(() => this.setupWebSocket(), 5000);
        };

        this.websocket.onerror = (error) => {
            console.error('WebSocket error:', error);
            this.updateConnectionStatus('error');
        };
    }

    updateConnectionStatus(status) {
        const statusIndicator = document.getElementById('systemStatus');
        if (!statusIndicator) return;

        const icon = statusIndicator.querySelector('i');
        const text = statusIndicator.querySelector('span');

        switch (status) {
            case 'connected':
                statusIndicator.className = 'status-indicator';
                icon.className = 'fas fa-circle';
                text.textContent = 'Connected';
                statusIndicator.style.color = 'var(--success-color)';
                break;
            case 'disconnected':
            case 'error':
                statusIndicator.className = 'status-indicator warning';
                icon.className = 'fas fa-exclamation-triangle';
                text.textContent = status === 'error' ? 'Connection Error' : 'Disconnected';
                statusIndicator.style.color = 'var(--warning-color)';
                break;
        }
    }

    handleRealtimeUpdate(data) {
        switch (data.type) {
            case 'system_metrics':
                this.updateSystemMetrics(data.payload);
                break;
            case 'alerts':
                this.updateAlerts(data.payload);
                break;
            case 'logs':
                this.updateLogs(data.payload);
                break;
            case 'network_stats':
                this.updateNetworkStats(data.payload);
                break;
            case 'security_events':
                this.updateSecurityEvents(data.payload);
                break;
            case 'educational_data':
                this.updateEducationalData(data.payload);
                break;
        }
    }

    updateSystemMetrics(metrics) {
        // Update CPU usage
        const cpuProgress = document.getElementById('cpuProgress');
        const cpuValue = document.getElementById('cpuValue');
        if (cpuProgress && cpuValue) {
            const cpuPercent = metrics.cpu_usage || 0;
            cpuProgress.style.width = `${cpuPercent}%`;
            cpuValue.textContent = `${cpuPercent.toFixed(1)}%`;
            
            // Change color based on usage
            if (cpuPercent > 80) {
                cpuProgress.style.background = 'var(--error-color)';
            } else if (cpuPercent > 60) {
                cpuProgress.style.background = 'var(--warning-color)';
            } else {
                cpuProgress.style.background = 'linear-gradient(90deg, var(--primary-color), var(--success-color))';
            }
        }

        // Update memory usage
        const memoryProgress = document.getElementById('memoryProgress');
        const memoryValue = document.getElementById('memoryValue');
        if (memoryProgress && memoryValue) {
            const memoryPercent = metrics.memory_usage || 0;
            memoryProgress.style.width = `${memoryPercent}%`;
            memoryValue.textContent = `${memoryPercent.toFixed(1)}%`;
            
            // Change color based on usage
            if (memoryPercent > 85) {
                memoryProgress.style.background = 'var(--error-color)';
            } else if (memoryPercent > 70) {
                memoryProgress.style.background = 'var(--warning-color)';
            } else {
                memoryProgress.style.background = 'linear-gradient(90deg, var(--primary-color), var(--success-color))';
            }
        }

        // Update disk usage
        const diskProgress = document.getElementById('diskProgress');
        const diskValue = document.getElementById('diskValue');
        if (diskProgress && diskValue) {
            const diskPercent = metrics.disk_usage || 0;
            diskProgress.style.width = `${diskPercent}%`;
            diskValue.textContent = `${diskPercent.toFixed(1)}%`;
            
            // Change color based on usage
            if (diskPercent > 90) {
                diskProgress.style.background = 'var(--error-color)';
            } else if (diskPercent > 80) {
                diskProgress.style.background = 'var(--warning-color)';
            } else {
                diskProgress.style.background = 'linear-gradient(90deg, var(--primary-color), var(--success-color))';
            }
        }

        // Update network stats
        const networkIn = document.getElementById('networkIn');
        const networkOut = document.getElementById('networkOut');
        if (networkIn && networkOut) {
            networkIn.textContent = `↑ ${this.formatBytes(metrics.network_in || 0)}/s`;
            networkOut.textContent = `↓ ${this.formatBytes(metrics.network_out || 0)}/s`;
        }

        // Update top processes
        this.updateProcessList(metrics.top_processes || []);

        // Update charts
        this.updatePerformanceChart(metrics);
        this.updateMemoryChart(metrics);
    }

    updateProcessList(processes) {
        const processList = document.getElementById('processList');
        if (!processList) return;

        processList.innerHTML = processes.map(process => `
            <div class="process-item">
                <span class="process-name">${process.name}</span>
                <div>
                    <span class="process-cpu">${process.cpu}% CPU</span>
                    <span class="process-memory">${process.memory}% MEM</span>
                </div>
            </div>
        `).join('');
    }

    updateAlerts(alerts) {
        const alertsList = document.getElementById('alertsList');
        if (!alertsList) return;

        if (alerts.length === 0) {
            alertsList.innerHTML = `
                <div class="alert-item success">
                    <i class="fas fa-check-circle"></i>
                    <span>No active alerts</span>
                </div>
            `;
            return;
        }

        alertsList.innerHTML = alerts.map(alert => `
            <div class="alert-item ${alert.severity}">
                <i class="fas ${this.getAlertIcon(alert.severity)}"></i>
                <span>${alert.message}</span>
            </div>
        `).join('');

        // Update alerts table
        this.updateAlertsTable(alerts);
    }

    updateAlertsTable(alerts) {
        const alertsTable = document.getElementById('alertsTable');
        if (!alertsTable) return;

        alertsTable.innerHTML = `
            <table class="alerts-table">
                <thead>
                    <tr>
                        <th>Time</th>
                        <th>Severity</th>
                        <th>Source</th>
                        <th>Message</th>
                        <th>Status</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>
                    ${alerts.map(alert => `
                        <tr>
                            <td>${this.formatTime(alert.timestamp)}</td>
                            <td><span class="alert-severity ${alert.severity}">${alert.severity.toUpperCase()}</span></td>
                            <td>${alert.source}</td>
                            <td>${alert.message}</td>
                            <td><span class="alert-status ${alert.status}">${alert.status.toUpperCase()}</span></td>
                            <td>
                                <button class="btn btn-sm btn-secondary" onclick="dashboard.acknowledgeAlert('${alert.id}')">Acknowledge</button>
                                <button class="btn btn-sm btn-success" onclick="dashboard.resolveAlert('${alert.id}')">Resolve</button>
                            </td>
                        </tr>
                    `).join('')}
                </tbody>
            </table>
        `;
    }

    updateLogs(logs) {
        const logViewer = document.getElementById('logViewer');
        if (!logViewer) return;

        logViewer.innerHTML = logs.map(log => `
            <div class="log-entry">
                <span class="log-timestamp">${this.formatTime(log.timestamp)}</span>
                <span class="log-level ${log.level}">${log.level.toUpperCase()}</span>
                <span class="log-message">${log.message}</span>
            </div>
        `).join('');
    }

    updateNetworkStats(stats) {
        // Update network chart
        this.updateNetworkChart(stats);
        
        // Update active connections
        this.updateActiveConnections(stats.connections || []);
        
        // Update interface statistics
        this.updateInterfaceStats(stats.interfaces || []);
    }

    updateActiveConnections(connections) {
        const activeConnections = document.getElementById('activeConnections');
        if (!activeConnections) return;

        activeConnections.innerHTML = connections.map(conn => `
            <div class="connection-item">
                <span class="connection-remote">${conn.remote}</span>
                <span class="connection-status ${conn.status}">${conn.status.toUpperCase()}</span>
            </div>
        `).join('');
    }

    updateInterfaceStats(interfaces) {
        const interfaceStats = document.getElementById('interfaceStats');
        if (!interfaceStats) return;

        interfaceStats.innerHTML = interfaces.map(intf => `
            <div class="interface-item">
                <div class="interface-name">${intf.name}</div>
                <div class="interface-stats">
                    <span>In: ${this.formatBytes(intf.bytes_in)}</span>
                    <span>Out: ${this.formatBytes(intf.bytes_out)}</span>
                    <span>Packets In: ${intf.packets_in}</span>
                    <span>Packets Out: ${intf.packets_out}</span>
                </div>
            </div>
        `).join('');
    }

    updateSecurityEvents(events) {
        const securityEvents = document.getElementById('securityEvents');
        if (!securityEvents) return;

        securityEvents.innerHTML = events.map(event => `
            <div class="security-event ${event.severity}">
                <div class="event-time">${this.formatTime(event.timestamp)}</div>
                <div class="event-details">
                    <div class="event-type">${event.type}</div>
                    <div class="event-description">${event.description}</div>
                </div>
            </div>
        `).join('');
    }

    updateEducationalData(data) {
        // Update lab usage chart
        this.updateLabUsageChart(data.lab_usage || []);
        
        // Update student activity chart
        this.updateStudentActivityChart(data.student_activity || []);
        
        // Update educational KPIs
        this.updateEducationalKPIs(data.kpis || {});
        
        // Update course resource usage
        this.updateCourseResourceUsage(data.courses || []);
    }

    updateEducationalKPIs(kpis) {
        const educationalKPIs = document.getElementById('educationalKPIs');
        if (!educationalKPIs) return;

        educationalKPIs.innerHTML = Object.entries(kpis).map(([key, value]) => `
            <div class="kpi-item">
                <div class="kpi-value ${this.getKPIStatus(key, value)}">${value.current}</div>
                <div class="kpi-label">${value.label}</div>
                <div class="kpi-change ${value.change >= 0 ? 'positive' : 'negative'}">
                    ${value.change >= 0 ? '+' : ''}${value.change}% vs last period
                </div>
            </div>
        `).join('');
    }

    updateCourseResourceUsage(courses) {
        const courseResourceUsage = document.getElementById('courseResourceUsage');
        if (!courseResourceUsage) return;

        courseResourceUsage.innerHTML = courses.map(course => `
            <div class="course-item">
                <div class="course-name">${course.name}</div>
                <div class="course-metrics">
                    <div class="metric-row">
                        <span>Active Users:</span>
                        <span>${course.active_users}</span>
                    </div>
                    <div class="metric-row">
                        <span>CPU Usage:</span>
                        <span>${course.cpu_usage}%</span>
                    </div>
                    <div class="metric-row">
                        <span>Memory Usage:</span>
                        <span>${course.memory_usage}%</span>
                    </div>
                    <div class="metric-row">
                        <span>Storage Usage:</span>
                        <span>${course.storage_usage}%</span>
                    </div>
                </div>
            </div>
        `).join('');
    }

    initializeCharts() {
        this.createPerformanceChart();
        this.createMemoryChart();
        this.createNetworkChart();
        this.createLabUsageChart();
        this.createStudentActivityChart();
        this.createForecastChart();
        this.createAlertTrendsChart();
    }

    createPerformanceChart() {
        const ctx = document.getElementById('performanceChart');
        if (!ctx) return;

        this.charts.set('performance', new Chart(ctx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [{
                    label: 'CPU Usage',
                    data: [],
                    borderColor: '#2563eb',
                    backgroundColor: 'rgba(37, 99, 235, 0.1)',
                    tension: 0.4
                }, {
                    label: 'Memory Usage',
                    data: [],
                    borderColor: '#10b981',
                    backgroundColor: 'rgba(16, 185, 129, 0.1)',
                    tension: 0.4
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: {
                        beginAtZero: true,
                        max: 100
                    }
                }
            }
        }));
    }

    createMemoryChart() {
        const ctx = document.getElementById('memoryChart');
        if (!ctx) return;

        this.charts.set('memory', new Chart(ctx, {
            type: 'doughnut',
            data: {
                labels: ['Used', 'Available'],
                datasets: [{
                    data: [0, 100],
                    backgroundColor: ['#ef4444', '#10b981'],
                    borderWidth: 0
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                cutout: '70%'
            }
        }));
    }

    createNetworkChart() {
        const ctx = document.getElementById('networkChart');
        if (!ctx) return;

        this.charts.set('network', new Chart(ctx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [{
                    label: 'Incoming',
                    data: [],
                    borderColor: '#3b82f6',
                    backgroundColor: 'rgba(59, 130, 246, 0.1)',
                    tension: 0.4
                }, {
                    label: 'Outgoing',
                    data: [],
                    borderColor: '#10b981',
                    backgroundColor: 'rgba(16, 185, 129, 0.1)',
                    tension: 0.4
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: {
                        beginAtZero: true,
                        ticks: {
                            callback: function(value) {
                                return window.dashboard ? window.dashboard.formatBytes(value) : value;
                            }
                        }
                    }
                }
            }
        }));
    }

    createLabUsageChart() {
        const ctx = document.getElementById('labUsageChart');
        if (!ctx) return;

        this.charts.set('labUsage', new Chart(ctx, {
            type: 'bar',
            data: {
                labels: [],
                datasets: [{
                    label: 'Active Sessions',
                    data: [],
                    backgroundColor: '#3b82f6',
                    borderRadius: 4
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: {
                        beginAtZero: true
                    }
                }
            }
        }));
    }

    createStudentActivityChart() {
        const ctx = document.getElementById('studentActivityChart');
        if (!ctx) return;

        this.charts.set('studentActivity', new Chart(ctx, {
            type: 'pie',
            data: {
                labels: ['Active Learning', 'Idle', 'Break', 'Group Work'],
                datasets: [{
                    data: [],
                    backgroundColor: ['#10b981', '#f59e0b', '#ef4444', '#3b82f6'],
                    borderWidth: 0
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false
            }
        }));
    }

    createForecastChart() {
        const ctx = document.getElementById('forecastChart');
        if (!ctx) return;

        this.charts.set('forecast', new Chart(ctx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [{
                    label: 'Current',
                    data: [],
                    borderColor: '#3b82f6',
                    backgroundColor: 'rgba(59, 130, 246, 0.1)',
                    tension: 0.4
                }, {
                    label: 'Forecast',
                    data: [],
                    borderColor: '#ef4444',
                    backgroundColor: 'rgba(239, 68, 68, 0.1)',
                    borderDash: [5, 5],
                    tension: 0.4
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: {
                        beginAtZero: true,
                        max: 100
                    }
                }
            }
        }));
    }

    createAlertTrendsChart() {
        const ctx = document.getElementById('alertTrendsChart');
        if (!ctx) return;

        this.charts.set('alertTrends', new Chart(ctx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [{
                    label: 'Critical',
                    data: [],
                    borderColor: '#ef4444',
                    backgroundColor: 'rgba(239, 68, 68, 0.1)',
                    tension: 0.4
                }, {
                    label: 'Warning',
                    data: [],
                    borderColor: '#f59e0b',
                    backgroundColor: 'rgba(245, 158, 11, 0.1)',
                    tension: 0.4
                }, {
                    label: 'Info',
                    data: [],
                    borderColor: '#3b82f6',
                    backgroundColor: 'rgba(59, 130, 246, 0.1)',
                    tension: 0.4
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: {
                        beginAtZero: true
                    }
                }
            }
        }));
    }

    updatePerformanceChart(metrics) {
        const chart = this.charts.get('performance');
        if (!chart) return;

        const now = new Date().toLocaleTimeString();
        
        chart.data.labels.push(now);
        chart.data.datasets[0].data.push(metrics.cpu_usage || 0);
        chart.data.datasets[1].data.push(metrics.memory_usage || 0);

        // Keep only last 20 data points
        if (chart.data.labels.length > 20) {
            chart.data.labels.shift();
            chart.data.datasets.forEach(dataset => dataset.data.shift());
        }

        chart.update('none');
    }

    updateMemoryChart(metrics) {
        const chart = this.charts.get('memory');
        if (!chart) return;

        const used = metrics.memory_usage || 0;
        const available = 100 - used;

        chart.data.datasets[0].data = [used, available];
        chart.update('none');
    }

    updateNetworkChart(stats) {
        const chart = this.charts.get('network');
        if (!chart) return;

        const now = new Date().toLocaleTimeString();
        
        chart.data.labels.push(now);
        chart.data.datasets[0].data.push(stats.network_in || 0);
        chart.data.datasets[1].data.push(stats.network_out || 0);

        // Keep only last 20 data points
        if (chart.data.labels.length > 20) {
            chart.data.labels.shift();
            chart.data.datasets.forEach(dataset => dataset.data.shift());
        }

        chart.update('none');
    }

    updateLabUsageChart(data) {
        const chart = this.charts.get('labUsage');
        if (!chart) return;

        chart.data.labels = data.map(item => item.time);
        chart.data.datasets[0].data = data.map(item => item.sessions);
        chart.update('none');
    }

    updateStudentActivityChart(data) {
        const chart = this.charts.get('studentActivity');
        if (!chart) return;

        chart.data.datasets[0].data = [
            data.active_learning || 0,
            data.idle || 0,
            data.break || 0,
            data.group_work || 0
        ];
        chart.update('none');
    }

    switchTab(tabName) {
        // Update tab navigation
        document.querySelectorAll('.nav-tabs li').forEach(tab => {
            tab.classList.remove('active');
        });
        document.querySelector(`[data-tab="${tabName}"]`).classList.add('active');

        // Update tab content
        document.querySelectorAll('.tab-content').forEach(content => {
            content.classList.remove('active');
        });
        document.getElementById(tabName).classList.add('active');

        // Load tab-specific data
        this.loadTabData(tabName);
    }

    loadTabData(tabName) {
        switch (tabName) {
            case 'system':
                this.loadSystemHealthData();
                break;
            case 'performance':
                this.loadPerformanceData();
                break;
            case 'logs':
                this.loadLogData();
                break;
            case 'network':
                this.loadNetworkData();
                break;
            case 'security':
                this.loadSecurityData();
                break;
            case 'educational':
                this.loadEducationalData();
                break;
            case 'compliance':
                this.loadComplianceData();
                break;
            case 'alerts':
                this.loadAlertsData();
                break;
        }
    }

    loadSystemHealthData() {
        fetch('/api/system/health')
            .then(response => response.json())
            .then(data => {
                this.updateHardwareSensors(data.sensors || []);
                this.updateCpuDetails(data.cpu || {});
                this.updateStorageHealth(data.storage || {});
            })
            .catch(error => console.error('Error loading system health data:', error));
    }

    updateHardwareSensors(sensors) {
        const hardwareSensors = document.getElementById('hardwareSensors');
        if (!hardwareSensors) return;

        hardwareSensors.innerHTML = sensors.map(sensor => `
            <div class="sensor-item">
                <div class="sensor-value">${sensor.value}</div>
                <div class="sensor-label">${sensor.name}</div>
                <div class="sensor-status ${sensor.status}">${sensor.status.toUpperCase()}</div>
            </div>
        `).join('');
    }

    updateCpuDetails(cpu) {
        const cpuDetails = document.getElementById('cpuDetails');
        if (!cpuDetails) return;

        cpuDetails.innerHTML = `
            <div class="cpu-detail-item">
                <span>Model:</span> <span>${cpu.model}</span>
            </div>
            <div class="cpu-detail-item">
                <span>Cores:</span> <span>${cpu.cores}</span>
            </div>
            <div class="cpu-detail-item">
                <span>Frequency:</span> <span>${cpu.frequency}</span>
            </div>
            <div class="cpu-detail-item">
                <span>Temperature:</span> <span>${cpu.temperature}°C</span>
            </div>
        `;
    }

    updateStorageHealth(storage) {
        const storageHealth = document.getElementById('storageHealth');
        if (!storageHealth) return;

        storageHealth.innerHTML = `
            <div class="storage-detail-item">
                <span>Total Space:</span> <span>${this.formatBytes(storage.total)}</span>
            </div>
            <div class="storage-detail-item">
                <span>Used Space:</span> <span>${this.formatBytes(storage.used)}</span>
            </div>
            <div class="storage-detail-item">
                <span>Available:</span> <span>${this.formatBytes(storage.available)}</span>
            </div>
            <div class="storage-detail-item">
                <span>Health Status:</span> <span class="status-badge ${storage.health}">${storage.health}</span>
            </div>
        `;
    }

    loadInitialData() {
        // Load all initial data
        this.loadSystemMetrics();
        this.loadAlerts();
        this.loadLogs();
        this.loadNetworkStats();
        this.loadSecurityEvents();
        this.loadEducationalData();
        this.loadComplianceData();
    }

    loadSystemMetrics() {
        fetch('/api/system/metrics')
            .then(response => response.json())
            .then(data => this.updateSystemMetrics(data))
            .catch(error => console.error('Error loading system metrics:', error));
    }

    loadAlerts() {
        fetch('/api/alerts')
            .then(response => response.json())
            .then(data => this.updateAlerts(data))
            .catch(error => console.error('Error loading alerts:', error));
    }

    loadLogs() {
        fetch('/api/logs?limit=50')
            .then(response => response.json())
            .then(data => this.updateLogs(data))
            .catch(error => console.error('Error loading logs:', error));
    }

    loadNetworkStats() {
        fetch('/api/network/stats')
            .then(response => response.json())
            .then(data => this.updateNetworkStats(data))
            .catch(error => console.error('Error loading network stats:', error));
    }

    loadSecurityEvents() {
        fetch('/api/security/events')
            .then(response => response.json())
            .then(data => this.updateSecurityEvents(data))
            .catch(error => console.error('Error loading security events:', error));
    }

    loadEducationalData() {
        fetch('/api/educational/analytics')
            .then(response => response.json())
            .then(data => this.updateEducationalData(data))
            .catch(error => console.error('Error loading educational data:', error));
    }

    loadComplianceData() {
        fetch('/api/compliance/status')
            .then(response => response.json())
            .then(data => this.updateComplianceStatus(data))
            .catch(error => console.error('Error loading compliance data:', error));
    }

    updateComplianceStatus(data) {
        const complianceStatus = document.getElementById('complianceStatus');
        if (!complianceStatus) return;

        complianceStatus.innerHTML = Object.entries(data).map(([standard, status]) => `
            <div class="compliance-item">
                <div class="compliance-status ${status.compliant ? 'compliant' : 'non-compliant'}">
                    <i class="fas ${status.compliant ? 'fa-check-circle' : 'fa-times-circle'}"></i>
                </div>
                <div class="compliance-name">${standard.toUpperCase()}</div>
                <div class="compliance-score">${status.score}% Compliant</div>
            </div>
        `).join('');
    }

    startDataCollection() {
        this.updateInterval = setInterval(() => {
            this.refreshData();
        }, this.settings.refreshRate * 1000);
    }

    refreshData() {
        // Refresh all data
        this.loadSystemMetrics();
        this.loadAlerts();
        
        // Update timestamp for last refresh
        const refreshBtn = document.getElementById('refreshBtn');
        if (refreshBtn) {
            const icon = refreshBtn.querySelector('i');
            icon.style.animation = 'spin 1s linear';
            setTimeout(() => {
                icon.style.animation = '';
            }, 1000);
        }
    }

    openSettings() {
        const modal = document.getElementById('settingsModal');
        if (modal) {
            // Load current settings
            document.getElementById('themeSelect').value = this.settings.theme;
            document.getElementById('refreshRateSelect').value = this.settings.refreshRate;
            document.getElementById('enableNotifications').checked = this.settings.enableNotifications;
            document.getElementById('enableSound').checked = this.settings.enableSound;
            
            modal.style.display = 'block';
        }
    }

    closeSettings() {
        const modal = document.getElementById('settingsModal');
        if (modal) {
            modal.style.display = 'none';
        }
    }

    saveSettings() {
        this.settings.theme = document.getElementById('themeSelect').value;
        this.settings.refreshRate = parseInt(document.getElementById('refreshRateSelect').value);
        this.settings.enableNotifications = document.getElementById('enableNotifications').checked;
        this.settings.enableSound = document.getElementById('enableSound').checked;

        // Save to localStorage
        localStorage.setItem('dashboard_theme', this.settings.theme);
        localStorage.setItem('dashboard_refresh_rate', this.settings.refreshRate);
        localStorage.setItem('dashboard_notifications', this.settings.enableNotifications);
        localStorage.setItem('dashboard_sound', this.settings.enableSound);

        // Apply settings
        this.applyTheme();
        
        // Restart data collection with new interval
        if (this.updateInterval) {
            clearInterval(this.updateInterval);
            this.startDataCollection();
        }

        this.closeSettings();
        this.showNotification('Settings saved successfully', 'success');
    }

    resetSettings() {
        this.settings = {
            theme: 'dark',
            refreshRate: 5,
            enableNotifications: false,
            enableSound: false
        };

        this.saveSettings();
    }

    applyTheme() {
        document.body.className = `theme-${this.settings.theme}`;
        
        // Update chart themes
        this.charts.forEach(chart => {
            chart.options.plugins.legend.labels.color = getComputedStyle(document.body).getPropertyValue('--text-primary');
            chart.options.scales.x.ticks.color = getComputedStyle(document.body).getPropertyValue('--text-secondary');
            chart.options.scales.y.ticks.color = getComputedStyle(document.body).getPropertyValue('--text-secondary');
            chart.update('none');
        });
    }

    exportData() {
        const data = {
            timestamp: new Date().toISOString(),
            system_metrics: {},
            alerts: [],
            logs: [],
            export_format: 'json'
        };

        // Collect all current data
        fetch('/api/export/data')
            .then(response => response.json())
            .then(exportData => {
                const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' });
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = `multios-monitoring-${new Date().toISOString().split('T')[0]}.json`;
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
                URL.revokeObjectURL(url);
                
                this.showNotification('Data exported successfully', 'success');
            })
            .catch(error => {
                console.error('Error exporting data:', error);
                this.showNotification('Error exporting data', 'error');
            });
    }

    generateReport() {
        const reportType = document.getElementById('reportType').value;
        const reportPeriod = document.getElementById('reportPeriod').value;
        
        fetch('/api/compliance/report', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                type: reportType,
                period: reportPeriod,
                format: 'pdf'
            })
        })
        .then(response => response.blob())
        .then(pdfBlob => {
            const url = URL.createObjectURL(pdfBlob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `${reportType}-report-${reportPeriod}.pdf`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
            
            this.showNotification('Report generated successfully', 'success');
        })
        .catch(error => {
            console.error('Error generating report:', error);
            this.showNotification('Error generating report', 'error');
        });
    }

    acknowledgeAlert(alertId) {
        fetch(`/api/alerts/${alertId}/acknowledge`, { method: 'POST' })
            .then(response => response.json())
            .then(() => {
                this.loadAlerts();
                this.showNotification('Alert acknowledged', 'success');
            })
            .catch(error => {
                console.error('Error acknowledging alert:', error);
                this.showNotification('Error acknowledging alert', 'error');
            });
    }

    resolveAlert(alertId) {
        fetch(`/api/alerts/${alertId}/resolve`, { method: 'POST' })
            .then(response => response.json())
            .then(() => {
                this.loadAlerts();
                this.showNotification('Alert resolved', 'success');
            })
            .catch(error => {
                console.error('Error resolving alert:', error);
                this.showNotification('Error resolving alert', 'error');
            });
    }

    clearLogs() {
        const logViewer = document.getElementById('logViewer');
        if (logViewer) {
            logViewer.innerHTML = '';
        }
    }

    filterLogs(level) {
        const logEntries = document.querySelectorAll('.log-entry');
        logEntries.forEach(entry => {
            if (level === 'all' || entry.querySelector('.log-level').classList.contains(level)) {
                entry.style.display = 'flex';
            } else {
                entry.style.display = 'none';
            }
        });
    }

    searchLogs(query) {
        const logEntries = document.querySelectorAll('.log-entry');
        logEntries.forEach(entry => {
            const message = entry.querySelector('.log-message').textContent.toLowerCase();
            if (message.includes(query.toLowerCase())) {
                entry.style.display = 'flex';
            } else {
                entry.style.display = 'none';
            }
        });
    }

    filterAlerts(type, value) {
        // Implementation for filtering alerts
        this.loadAlerts();
    }

    // Utility methods
    formatBytes(bytes) {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }

    formatTime(timestamp) {
        return new Date(timestamp).toLocaleString();
    }

    getAlertIcon(severity) {
        switch (severity) {
            case 'critical':
                return 'fa-exclamation-circle';
            case 'warning':
                return 'fa-exclamation-triangle';
            case 'info':
                return 'fa-info-circle';
            default:
                return 'fa-bell';
        }
    }

    getKPIStatus(key, value) {
        // Determine KPI status based on thresholds
        const thresholds = {
            'lab_utilization': { good: 70, warning: 40 },
            'student_engagement': { good: 80, warning: 60 },
            'resource_efficiency': { good: 90, warning: 70 }
        };
        
        const threshold = thresholds[key] || { good: 80, warning: 60 };
        if (value.current >= threshold.good) return 'good';
        if (value.current >= threshold.warning) return 'warning';
        return 'critical';
    }

    showNotification(message, type = 'info') {
        if (!this.settings.enableNotifications) return;

        const notification = document.createElement('div');
        notification.className = `notification ${type}`;
        notification.innerHTML = `
            <div class="notification-header">
                <div class="notification-title">${type.toUpperCase()}</div>
                <div class="notification-close">&times;</div>
            </div>
            <div class="notification-message">${message}</div>
        `;

        document.body.appendChild(notification);

        // Auto-remove after 5 seconds
        setTimeout(() => {
            if (notification.parentNode) {
                notification.parentNode.removeChild(notification);
            }
        }, 5000);

        // Close button
        notification.querySelector('.notification-close').onclick = () => {
            if (notification.parentNode) {
                notification.parentNode.removeChild(notification);
            }
        };

        // Sound notification
        if (this.settings.enableSound) {
            this.playNotificationSound(type);
        }
    }

    playNotificationSound(type) {
        // Create audio context for notification sounds
        const audioContext = new (window.AudioContext || window.webkitAudioContext)();
        const oscillator = audioContext.createOscillator();
        const gainNode = audioContext.createGain();

        oscillator.connect(gainNode);
        gainNode.connect(audioContext.destination);

        // Different frequencies for different alert types
        const frequencies = {
            'success': 800,
            'warning': 600,
            'error': 400,
            'info': 1000
        };

        oscillator.frequency.setValueAtTime(frequencies[type] || 1000, audioContext.currentTime);
        gainNode.gain.setValueAtTime(0.1, audioContext.currentTime);
        gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 0.5);

        oscillator.start(audioContext.currentTime);
        oscillator.stop(audioContext.currentTime + 0.5);
    }
}

// Initialize dashboard when page loads
let dashboard;
document.addEventListener('DOMContentLoaded', () => {
    dashboard = new MonitoringDashboard();
    window.dashboard = dashboard; // Make it globally accessible for onclick handlers
});

// Handle page visibility change to pause/resume updates
document.addEventListener('visibilitychange', () => {
    if (document.hidden) {
        if (dashboard && dashboard.updateInterval) {
            clearInterval(dashboard.updateInterval);
        }
    } else {
        if (dashboard) {
            dashboard.startDataCollection();
        }
    }
});