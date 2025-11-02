// Performance Dashboard JavaScript
class PerformanceDashboard {
    constructor() {
        this.charts = {};
        this.updateInterval = null;
        this.isRealTimeActive = true;
        this.historicalData = [];
        this.maxDataPoints = 100;
        
        this.init();
    }

    init() {
        this.setupCharts();
        this.bindEvents();
        this.startRealTimeUpdates();
        this.loadHistoricalData();
    }

    setupCharts() {
        // CPU Chart
        this.setupCPUChart();
        // Memory Chart
        this.setupMemoryChart();
        // I/O Chart
        this.setupIOChart();
        // Network Chart
        this.setupNetworkChart();
        // Power Chart
        this.setupPowerChart();
        // Temperature Chart
        this.setupTemperatureChart();
        // Historical Chart
        this.setupHistoricalChart();
        // Protocol Chart
        this.setupProtocolChart();
    }

    setupCPUChart() {
        const ctx = document.getElementById('cpuChart').getContext('2d');
        this.charts.cpu = new Chart(ctx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [{
                    label: 'CPU Usage (%)',
                    data: [],
                    borderColor: '#667eea',
                    backgroundColor: 'rgba(102, 126, 234, 0.1)',
                    borderWidth: 2,
                    fill: true,
                    tension: 0.4
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: {
                        beginAtZero: true,
                        max: 100,
                        ticks: {
                            callback: function(value) {
                                return value + '%';
                            }
                        }
                    }
                },
                plugins: {
                    legend: {
                        display: false
                    }
                }
            }
        });
    }

    setupMemoryChart() {
        const ctx = document.getElementById('memoryChart').getContext('2d');
        this.charts.memory = new Chart(ctx, {
            type: 'doughnut',
            data: {
                labels: ['Used', 'Available', 'Cached', 'Buffers'],
                datasets: [{
                    data: [0, 0, 0, 0],
                    backgroundColor: [
                        '#ef4444',
                        '#22c55e',
                        '#3b82f6',
                        '#f59e0b'
                    ]
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: {
                        position: 'bottom'
                    }
                }
            }
        });
    }

    setupIOChart() {
        const ctx = document.getElementById('ioChart').getContext('2d');
        this.charts.io = new Chart(ctx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [
                    {
                        label: 'Read (MB/s)',
                        data: [],
                        borderColor: '#22c55e',
                        backgroundColor: 'rgba(34, 197, 94, 0.1)',
                        borderWidth: 2,
                        fill: false,
                        tension: 0.4
                    },
                    {
                        label: 'Write (MB/s)',
                        data: [],
                        borderColor: '#ef4444',
                        backgroundColor: 'rgba(239, 68, 68, 0.1)',
                        borderWidth: 2,
                        fill: false,
                        tension: 0.4
                    }
                ]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: {
                        beginAtZero: true,
                        ticks: {
                            callback: function(value) {
                                return value + ' MB/s';
                            }
                        }
                    }
                },
                plugins: {
                    legend: {
                        position: 'bottom'
                    }
                }
            }
        });
    }

    setupNetworkChart() {
        const ctx = document.getElementById('networkChart').getContext('2d');
        this.charts.network = new Chart(ctx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [
                    {
                        label: 'Upload (Mbps)',
                        data: [],
                        borderColor: '#667eea',
                        backgroundColor: 'rgba(102, 126, 234, 0.1)',
                        borderWidth: 2,
                        fill: false,
                        tension: 0.4
                    },
                    {
                        label: 'Download (Mbps)',
                        data: [],
                        borderColor: '#22c55e',
                        backgroundColor: 'rgba(34, 197, 94, 0.1)',
                        borderWidth: 2,
                        fill: false,
                        tension: 0.4
                    }
                ]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: {
                        beginAtZero: true,
                        ticks: {
                            callback: function(value) {
                                return value + ' Mbps';
                            }
                        }
                    }
                },
                plugins: {
                    legend: {
                        position: 'bottom'
                    }
                }
            }
        });
    }

    setupPowerChart() {
        const ctx = document.getElementById('powerChart').getContext('2d');
        this.charts.power = new Chart(ctx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [{
                    label: 'Power (W)',
                    data: [],
                    borderColor: '#f59e0b',
                    backgroundColor: 'rgba(245, 158, 11, 0.1)',
                    borderWidth: 2,
                    fill: true,
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
                                return value + ' W';
                            }
                        }
                    }
                },
                plugins: {
                    legend: {
                        display: false
                    }
                }
            }
        });
    }

    setupTemperatureChart() {
        const ctx = document.getElementById('temperatureChart').getContext('2d');
        this.charts.temperature = new Chart(ctx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [
                    {
                        label: 'CPU (°C)',
                        data: [],
                        borderColor: '#ef4444',
                        backgroundColor: 'rgba(239, 68, 68, 0.1)',
                        borderWidth: 2,
                        fill: false,
                        tension: 0.4
                    },
                    {
                        label: 'GPU (°C)',
                        data: [],
                        borderColor: '#3b82f6',
                        backgroundColor: 'rgba(59, 130, 246, 0.1)',
                        borderWidth: 2,
                        fill: false,
                        tension: 0.4
                    }
                ]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: {
                        beginAtZero: false,
                        ticks: {
                            callback: function(value) {
                                return value + '°C';
                            }
                        }
                    }
                },
                plugins: {
                    legend: {
                        position: 'bottom'
                    }
                }
            }
        });
    }

    setupHistoricalChart() {
        const ctx = document.getElementById('historicalChart').getContext('2d');
        this.charts.historical = new Chart(ctx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [
                    {
                        label: 'CPU %',
                        data: [],
                        borderColor: '#667eea',
                        backgroundColor: 'rgba(102, 126, 234, 0.1)',
                        borderWidth: 2,
                        fill: false,
                        tension: 0.4
                    },
                    {
                        label: 'Memory %',
                        data: [],
                        borderColor: '#22c55e',
                        backgroundColor: 'rgba(34, 197, 94, 0.1)',
                        borderWidth: 2,
                        fill: false,
                        tension: 0.4
                    },
                    {
                        label: 'I/O MB/s',
                        data: [],
                        borderColor: '#ef4444',
                        backgroundColor: 'rgba(239, 68, 68, 0.1)',
                        borderWidth: 2,
                        fill: false,
                        tension: 0.4
                    }
                ]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    x: {
                        type: 'time',
                        time: {
                            displayFormats: {
                                hour: 'HH:mm',
                                day: 'MMM DD'
                            }
                        }
                    },
                    y: {
                        beginAtZero: true
                    }
                },
                plugins: {
                    legend: {
                        position: 'top'
                    }
                }
            }
        });
    }

    setupProtocolChart() {
        const ctx = document.getElementById('protocolChart').getContext('2d');
        this.charts.protocol = new Chart(ctx, {
            type: 'pie',
            data: {
                labels: ['TCP', 'UDP', 'HTTP', 'HTTPS', 'DNS', 'Other'],
                datasets: [{
                    data: [0, 0, 0, 0, 0, 0],
                    backgroundColor: [
                        '#667eea',
                        '#22c55e',
                        '#ef4444',
                        '#f59e0b',
                        '#8b5cf6',
                        '#6b7280'
                    ]
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: {
                        position: 'bottom'
                    }
                }
            }
        });
    }

    bindEvents() {
        // Refresh interval change
        document.getElementById('refreshInterval').addEventListener('change', (e) => {
            this.restartRealTimeUpdates(parseInt(e.target.value));
        });

        // Export data
        document.getElementById('exportData').addEventListener('click', () => {
            this.exportData();
        });

        // Toggle real-time
        document.getElementById('toggleRealTime').addEventListener('click', (e) => {
            this.toggleRealTime(e.target);
        });

        // Time range selector
        document.getElementById('timeRange').addEventListener('change', (e) => {
            this.loadHistoricalData(e.target.value);
        });

        // Compare data
        document.getElementById('compareData').addEventListener('click', () => {
            this.comparePerformance();
        });

        // Modal close
        const modal = document.getElementById('modal');
        const closeBtn = document.getElementsByClassName('close')[0];
        closeBtn.onclick = () => {
            modal.style.display = 'none';
        };
        window.onclick = (event) => {
            if (event.target === modal) {
                modal.style.display = 'none';
            }
        };
    }

    startRealTimeUpdates() {
        this.updateInterval = setInterval(() => {
            this.updateRealTimeData();
        }, 2000);
    }

    restartRealTimeUpdates(interval) {
        if (this.updateInterval) {
            clearInterval(this.updateInterval);
        }
        this.startRealTimeUpdates();
    }

    toggleRealTime(button) {
        this.isRealTimeActive = !this.isRealTimeActive;
        if (this.isRealTimeActive) {
            button.textContent = 'Pause';
            this.startRealTimeUpdates();
        } else {
            button.textContent = 'Resume';
            clearInterval(this.updateInterval);
        }
    }

    async updateRealTimeData() {
        if (!this.isRealTimeActive) return;

        try {
            const data = await this.fetchPerformanceData();
            this.updateDashboard(data);
        } catch (error) {
            console.error('Error updating real-time data:', error);
        }
    }

    async fetchPerformanceData() {
        // Simulate API call - in real implementation, this would fetch from system monitoring
        const cpu = Math.random() * 100;
        const memory = Math.random() * 100;
        const ioRead = Math.random() * 100;
        const ioWrite = Math.random() * 80;
        const networkUpload = Math.random() * 50;
        const networkDownload = Math.random() * 80;
        const power = Math.random() * 200 + 50;
        const cpuTemp = Math.random() * 30 + 40;
        const gpuTemp = Math.random() * 20 + 50;

        return {
            timestamp: new Date(),
            cpu: {
                usage: cpu,
                cores: Array.from({length: 8}, () => Math.random() * 100),
                processes: this.generateProcesses()
            },
            memory: {
                used: memory,
                available: 100 - memory,
                cached: Math.random() * 30,
                buffers: Math.random() * 20,
                allocations: this.generateMemoryAllocations()
            },
            io: {
                read: ioRead,
                write: ioWrite,
                devices: this.generateIODevices()
            },
            network: {
                upload: networkUpload,
                download: networkDownload,
                protocols: this.generateNetworkProtocols()
            },
            power: {
                consumption: power,
                temperatures: {
                    cpu: cpuTemp,
                    gpu: gpuTemp,
                    zones: this.generateThermalZones()
                }
            },
            events: this.generateSystemEvents()
        };
    }

    generateProcesses() {
        const processes = ['chrome', 'firefox', 'code', 'node', 'python', 'systemd', 'kworker'];
        return processes.map(name => ({
            name,
            cpu: Math.random() * 20,
            memory: Math.random() * 500,
            pid: Math.floor(Math.random() * 10000)
        }));
    }

    generateMemoryAllocations() {
        const types = ['Kernel', 'Applications', 'Cache', 'Buffers', 'Swap'];
        return types.map(type => ({
            type,
            amount: Math.random() * 8,
            percentage: Math.random() * 100
        }));
    }

    generateIODevices() {
        const devices = ['sda', 'sdb', 'nvme0n1', 'loop0'];
        return devices.map(device => ({
            name: device,
            read: Math.random() * 50,
            write: Math.random() * 30,
            utilization: Math.random() * 100
        }));
    }

    generateNetworkProtocols() {
        return {
            tcp: Math.random() * 40,
            udp: Math.random() * 20,
            http: Math.random() * 15,
            https: Math.random() * 10,
            dns: Math.random() * 5,
            other: Math.random() * 10
        };
    }

    generateThermalZones() {
        const zones = ['CPU', 'GPU', 'Chipset', 'Battery', 'SSD', 'Power'];
        return zones.map(zone => ({
            name: zone,
            temperature: Math.random() * 40 + 30,
            status: Math.random() > 0.8 ? 'warning' : 'normal'
        }));
    }

    generateSystemEvents() {
        const events = [
            { type: 'Performance Warning', message: 'High CPU usage detected', timestamp: new Date() },
            { type: 'Process Started', message: 'chrome process started', timestamp: new Date() },
            { type: 'Memory Warning', message: 'Memory usage above 80%', timestamp: new Date() },
            { type: 'Network Activity', message: 'High network traffic', timestamp: new Date() }
        ];
        return events.slice(0, Math.floor(Math.random() * 4) + 1);
    }

    updateDashboard(data) {
        // Update CPU data
        this.updateCPUData(data);
        // Update Memory data
        this.updateMemoryData(data);
        // Update I/O data
        this.updateIOData(data);
        // Update Network data
        this.updateNetworkData(data);
        // Update Power data
        this.updatePowerData(data);
        // Update Temperature data
        this.updateTemperatureData(data);
        // Update Process list
        this.updateProcessList(data.cpu.processes);
        // Update Event log
        this.updateEventLog(data.events);
        // Update heatmaps
        this.updateHeatmaps(data);
        // Store historical data
        this.storeHistoricalData(data);
    }

    updateCPUData(data) {
        const chart = this.charts.cpu;
        const time = data.timestamp.toLocaleTimeString();
        
        chart.data.labels.push(time);
        chart.data.datasets[0].data.push(data.cpu.usage);
        
        if (chart.data.labels.length > 20) {
            chart.data.labels.shift();
            chart.data.datasets[0].data.shift();
        }
        
        chart.update('none');
        
        // Update metric summary
        const cpuElement = document.getElementById('cpuUsage');
        cpuElement.textContent = data.cpu.usage.toFixed(1) + '%';
        cpuElement.className = 'current-value';
        this.updateTrend('cpu', data.cpu.usage);
        
        // Update CPU heatmap
        this.updateCPUHeatmap(data.cpu.cores);
    }

    updateMemoryData(data) {
        const chart = this.charts.memory;
        const used = data.memory.used;
        const available = data.memory.available;
        const cached = data.memory.cached;
        const buffers = data.memory.buffers;
        
        chart.data.datasets[0].data = [used, available, cached, buffers];
        chart.update('none');
        
        // Update metric summary (convert to GB assuming 16GB total)
        const totalGB = 16;
        const usedGB = (used / 100) * totalGB;
        const memoryElement = document.getElementById('memoryUsage');
        memoryElement.textContent = usedGB.toFixed(1) + ' GB / ' + totalGB + ' GB';
        this.updateTrend('memory', used);
        
        // Update allocation breakdown
        this.updateMemoryAllocations(data.memory.allocations);
    }

    updateIOData(data) {
        const chart = this.charts.io;
        const time = data.timestamp.toLocaleTimeString();
        
        chart.data.labels.push(time);
        chart.data.datasets[0].data.push(data.io.read);
        chart.data.datasets[1].data.push(data.io.write);
        
        if (chart.data.labels.length > 20) {
            chart.data.labels.shift();
            chart.data.datasets[0].data.shift();
            chart.data.datasets[1].data.shift();
        }
        
        chart.update('none');
        
        // Update metric summary
        const totalIO = data.io.read + data.io.write;
        const ioElement = document.getElementById('ioThroughput');
        ioElement.textContent = totalIO.toFixed(1) + ' MB/s';
        this.updateTrend('io', totalIO);
        
        // Update device breakdown
        this.updateDeviceList(data.io.devices);
    }

    updateNetworkData(data) {
        const chart = this.charts.network;
        const time = data.timestamp.toLocaleTimeString();
        
        chart.data.labels.push(time);
        chart.data.datasets[0].data.push(data.network.upload);
        chart.data.datasets[1].data.push(data.network.download);
        
        if (chart.data.labels.length > 20) {
            chart.data.labels.shift();
            chart.data.datasets[0].data.shift();
            chart.data.datasets[1].data.shift();
        }
        
        chart.update('none');
        
        // Update metric summary
        const totalNetwork = data.network.upload + data.network.download;
        const networkElement = document.getElementById('networkTraffic');
        networkElement.textContent = totalNetwork.toFixed(1) + ' Mbps';
        this.updateTrend('network', totalNetwork);
        
        // Update protocol breakdown
        this.updateProtocolChart(data.network.protocols);
    }

    updatePowerData(data) {
        const chart = this.charts.power;
        const time = data.timestamp.toLocaleTimeString();
        
        chart.data.labels.push(time);
        chart.data.datasets[0].data.push(data.power.consumption);
        
        if (chart.data.labels.length > 20) {
            chart.data.labels.shift();
            chart.data.datasets[0].data.shift();
        }
        
        chart.update('none');
        
        // Update metric summary
        const powerElement = document.getElementById('powerUsage');
        powerElement.textContent = data.power.consumption.toFixed(1) + ' W';
        this.updateTrend('power', data.power.consumption);
    }

    updateTemperatureData(data) {
        const chart = this.charts.temperature;
        const time = data.timestamp.toLocaleTimeString();
        
        chart.data.labels.push(time);
        chart.data.datasets[0].data.push(data.power.temperatures.cpu);
        chart.data.datasets[1].data.push(data.power.temperatures.gpu);
        
        if (chart.data.labels.length > 20) {
            chart.data.labels.shift();
            chart.data.datasets[0].data.shift();
            chart.data.datasets[1].data.shift();
        }
        
        chart.update('none');
        
        // Update thermal visualization
        this.updateThermalVisualization(data.power.temperatures.zones);
    }

    updateCPUHeatmap(cores) {
        const heatmap = document.getElementById('cpuHeatmap');
        heatmap.innerHTML = '';
        
        cores.forEach((usage, index) => {
            const cell = document.createElement('div');
            cell.className = 'heatmap-cell';
            cell.textContent = (index + 1);
            cell.title = `Core ${index + 1}: ${usage.toFixed(1)}%`;
            
            if (usage < 30) {
                cell.classList.add('cool');
            } else if (usage < 70) {
                cell.classList.add('warm');
            } else {
                cell.classList.add('hot');
            }
            
            heatmap.appendChild(cell);
        });
    }

    updateMemoryAllocations(allocations) {
        const container = document.getElementById('memoryAllocations');
        container.innerHTML = '';
        
        allocations.forEach(allocation => {
            const item = document.createElement('div');
            item.className = 'allocation-item';
            item.innerHTML = `
                <span class="allocation-name">${allocation.type}</span>
                <span class="allocation-amount">${allocation.amount.toFixed(1)} GB</span>
            `;
            container.appendChild(item);
        });
    }

    updateDeviceList(devices) {
        const container = document.getElementById('deviceList');
        container.innerHTML = '';
        
        devices.forEach(device => {
            const item = document.createElement('div');
            item.className = 'device-item';
            const totalThroughput = device.read + device.write;
            item.innerHTML = `
                <span class="device-name">${device.name}</span>
                <span class="device-throughput">${totalThroughput.toFixed(1)} MB/s</span>
            `;
            container.appendChild(item);
        });
    }

    updateProtocolChart(protocols) {
        const chart = this.charts.protocol;
        chart.data.datasets[0].data = [
            protocols.tcp,
            protocols.udp,
            protocols.http,
            protocols.https,
            protocols.dns,
            protocols.other
        ];
        chart.update('none');
    }

    updateThermalVisualization(zones) {
        const container = document.getElementById('thermalVisualization');
        container.innerHTML = '';
        
        zones.forEach(zone => {
            const element = document.createElement('div');
            element.className = 'thermal-zone';
            
            let className = 'cool';
            if (zone.temperature > 60) {
                className = 'hot';
            } else if (zone.temperature > 45) {
                className = 'warm';
            }
            
            element.classList.add(className);
            element.innerHTML = `
                <div class="temperature">${zone.temperature.toFixed(1)}°C</div>
                <div class="zone-name">${zone.name}</div>
            `;
            
            container.appendChild(element);
        });
    }

    updateProcessList(processes) {
        const container = document.getElementById('processList');
        container.innerHTML = '';
        
        processes.slice(0, 10).forEach(process => {
            const item = document.createElement('div');
            item.className = 'process-item';
            item.innerHTML = `
                <span class="process-name">${process.name}</span>
                <span class="process-cpu">${process.cpu.toFixed(1)}% CPU</span>
            `;
            container.appendChild(item);
        });
    }

    updateEventLog(events) {
        const container = document.getElementById('eventLog');
        container.innerHTML = '';
        
        events.forEach(event => {
            const item = document.createElement('div');
            item.className = 'event-item';
            item.innerHTML = `
                <span class="event-type">${event.type}</span>
                <span class="event-time">${event.timestamp.toLocaleTimeString()}</span>
            `;
            container.appendChild(item);
        });
    }

    updateTrend(metric, currentValue) {
        // Simple trend calculation - in real implementation, would use historical data
        const trends = {
            cpu: 'stable',
            memory: 'up',
            io: 'stable',
            network: 'down',
            power: 'stable'
        };
        
        const trendElement = document.getElementById(metric + 'Trend');
        trendElement.textContent = trends[metric] === 'up' ? '↑' : trends[metric] === 'down' ? '↓' : '→';
        trendElement.className = 'trend ' + trends[metric];
    }

    updateHeatmaps(data) {
        // CPU heatmap is already updated in updateCPUData
        // Thermal zones are updated in updateTemperatureData
    }

    storeHistoricalData(data) {
        this.historicalData.push({
            timestamp: data.timestamp,
            cpu: data.cpu.usage,
            memory: data.memory.used,
            io: data.io.read + data.io.write,
            network: data.network.upload + data.network.download
        });
        
        if (this.historicalData.length > this.maxDataPoints) {
            this.historicalData.shift();
        }
    }

    async loadHistoricalData(timeRange = '24h') {
        // Simulate loading historical data
        const hours = timeRange === '1h' ? 1 : timeRange === '6h' ? 6 : timeRange === '24h' ? 24 : 168;
        const points = hours * 12; // 12 points per hour
        
        const chart = this.charts.historical;
        const labels = [];
        const cpuData = [];
        const memoryData = [];
        const ioData = [];
        
        for (let i = 0; i < points; i++) {
            const timestamp = new Date(Date.now() - (points - i) * 300000); // 5 minute intervals
            labels.push(timestamp);
            cpuData.push(Math.random() * 100);
            memoryData.push(Math.random() * 100);
            ioData.push(Math.random() * 200);
        }
        
        chart.data.labels = labels;
        chart.data.datasets[0].data = cpuData;
        chart.data.datasets[1].data = memoryData;
        chart.data.datasets[2].data = ioData;
        chart.update();
    }

    comparePerformance() {
        // Simple comparison functionality
        const comparisonPanel = document.getElementById('comparisonPanel');
        const container = comparisonPanel.querySelector('.comparison-metrics');
        
        if (this.historicalData.length < 2) {
            container.innerHTML = '<p>Not enough data for comparison</p>';
            return;
        }
        
        const current = this.historicalData[this.historicalData.length - 1];
        const previous = this.historicalData[this.historicalData.length - 2];
        
        const metrics = [
            {
                name: 'CPU Usage',
                current: current.cpu.toFixed(1) + '%',
                previous: previous.cpu.toFixed(1) + '%',
                change: this.calculateChange(current.cpu, previous.cpu)
            },
            {
                name: 'Memory Usage',
                current: current.memory.toFixed(1) + '%',
                previous: previous.memory.toFixed(1) + '%',
                change: this.calculateChange(current.memory, previous.memory)
            },
            {
                name: 'I/O Throughput',
                current: current.io.toFixed(1) + ' MB/s',
                previous: previous.io.toFixed(1) + ' MB/s',
                change: this.calculateChange(current.io, previous.io)
            },
            {
                name: 'Network Traffic',
                current: current.network.toFixed(1) + ' Mbps',
                previous: previous.network.toFixed(1) + ' Mbps',
                change: this.calculateChange(current.network, previous.network)
            }
        ];
        
        container.innerHTML = metrics.map(metric => `
            <div class="comparison-metric">
                <span class="metric-name">${metric.name}</span>
                <span class="metric-value">${metric.current} (${metric.change})</span>
            </div>
        `).join('');
    }

    calculateChange(current, previous) {
        const change = ((current - previous) / previous) * 100;
        const sign = change >= 0 ? '+' : '';
        return sign + change.toFixed(1) + '%';
    }

    exportData() {
        const data = {
            timestamp: new Date().toISOString(),
            historical: this.historicalData,
            current: {
                cpu: document.getElementById('cpuUsage').textContent,
                memory: document.getElementById('memoryUsage').textContent,
                io: document.getElementById('ioThroughput').textContent,
                network: document.getElementById('networkTraffic').textContent,
                power: document.getElementById('powerUsage').textContent
            }
        };
        
        const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `performance-data-${new Date().toISOString().split('T')[0]}.json`;
        a.click();
        URL.revokeObjectURL(url);
    }
}

// Initialize dashboard when page loads
document.addEventListener('DOMContentLoaded', () => {
    new PerformanceDashboard();
});