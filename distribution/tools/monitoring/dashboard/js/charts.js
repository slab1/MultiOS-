// Chart.js helpers and additional chart management
// Enhanced visualization utilities for MultiOS Monitoring Dashboard

class ChartManager {
    constructor() {
        this.chartDefaults = {
            responsive: true,
            maintainAspectRatio: false,
            animation: {
                duration: 750,
                easing: 'easeInOutQuart'
            },
            plugins: {
                legend: {
                    labels: {
                        color: '#f8fafc',
                        font: {
                            size: 12
                        }
                    }
                }
            },
            scales: {
                x: {
                    ticks: {
                        color: '#cbd5e1',
                        font: {
                            size: 10
                        }
                    },
                    grid: {
                        color: '#374151',
                        drawBorder: false
                    }
                },
                y: {
                    ticks: {
                        color: '#cbd5e1',
                        font: {
                            size: 10
                        }
                    },
                    grid: {
                        color: '#374151',
                        drawBorder: false
                    },
                    beginAtZero: true
                }
            }
        };
    }

    // Create advanced performance timeline chart
    createPerformanceTimelineChart() {
        const ctx = document.getElementById('performanceTimelineChart');
        if (!ctx) return null;

        return new Chart(ctx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [
                    {
                        label: 'CPU Usage',
                        data: [],
                        borderColor: '#3b82f6',
                        backgroundColor: 'rgba(59, 130, 246, 0.1)',
                        borderWidth: 2,
                        tension: 0.4,
                        fill: true
                    },
                    {
                        label: 'Memory Usage',
                        data: [],
                        borderColor: '#10b981',
                        backgroundColor: 'rgba(16, 185, 129, 0.1)',
                        borderWidth: 2,
                        tension: 0.4,
                        fill: true
                    },
                    {
                        label: 'Disk I/O',
                        data: [],
                        borderColor: '#f59e0b',
                        backgroundColor: 'rgba(245, 158, 11, 0.1)',
                        borderWidth: 2,
                        tension: 0.4,
                        fill: false
                    },
                    {
                        label: 'Network Load',
                        data: [],
                        borderColor: '#ef4444',
                        backgroundColor: 'rgba(239, 68, 68, 0.1)',
                        borderWidth: 2,
                        tension: 0.4,
                        fill: false
                    }
                ]
            },
            options: {
                ...this.chartDefaults,
                scales: {
                    ...this.chartDefaults.scales,
                    y: {
                        ...this.chartDefaults.scales.y,
                        max: 100,
                        title: {
                            display: true,
                            text: 'Usage %',
                            color: '#f8fafc'
                        }
                    },
                    x: {
                        ...this.chartDefaults.scales.x,
                        title: {
                            display: true,
                            text: 'Time',
                            color: '#f8fafc'
                        }
                    }
                },
                plugins: {
                    ...this.chartDefaults.plugins,
                    tooltip: {
                        mode: 'index',
                        intersect: false,
                        backgroundColor: 'rgba(30, 41, 59, 0.9)',
                        titleColor: '#f8fafc',
                        bodyColor: '#cbd5e1',
                        borderColor: '#374151',
                        borderWidth: 1
                    }
                },
                interaction: {
                    mode: 'nearest',
                    axis: 'x',
                    intersect: false
                }
            }
        });
    }

    // Create real-time gauge charts
    createGaugeChart(canvasId, label, value, maxValue = 100, colors = null) {
        const ctx = document.getElementById(canvasId);
        if (!ctx) return null;

        const colorSchemes = {
            cpu: ['#10b981', '#f59e0b', '#ef4444'],
            memory: ['#3b82f6', '#06b6d4', '#ef4444'],
            disk: ['#8b5cf6', '#a855f7', '#ef4444'],
            network: ['#06b6d4', '#0ea5e9', '#ef4444']
        };

        const scheme = colors || colorSchemes.cpu;

        return new Chart(ctx, {
            type: 'doughnut',
            data: {
                labels: ['Used', 'Available'],
                datasets: [{
                    data: [value, maxValue - value],
                    backgroundColor: [
                        this.getUsageColor(value, maxValue, scheme),
                        '#374151'
                    ],
                    borderWidth: 0,
                    cutout: '75%'
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                rotation: -90,
                circumference: 180,
                plugins: {
                    legend: {
                        display: false
                    },
                    tooltip: {
                        enabled: false
                    }
                }
            }
        });
    }

    // Create multi-metric comparison chart
    createComparisonChart() {
        const ctx = document.getElementById('comparisonChart');
        if (!ctx) return null;

        return new Chart(ctx, {
            type: 'radar',
            data: {
                labels: ['CPU', 'Memory', 'Disk', 'Network', 'Processes', 'Threads'],
                datasets: [
                    {
                        label: 'Current',
                        data: [0, 0, 0, 0, 0, 0],
                        borderColor: '#3b82f6',
                        backgroundColor: 'rgba(59, 130, 246, 0.2)',
                        borderWidth: 2
                    },
                    {
                        label: 'Peak (24h)',
                        data: [0, 0, 0, 0, 0, 0],
                        borderColor: '#ef4444',
                        backgroundColor: 'rgba(239, 68, 68, 0.2)',
                        borderWidth: 2
                    },
                    {
                        label: 'Average (24h)',
                        data: [0, 0, 0, 0, 0, 0],
                        borderColor: '#10b981',
                        backgroundColor: 'rgba(16, 185, 129, 0.2)',
                        borderWidth: 2
                    }
                ]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: {
                        labels: {
                            color: '#f8fafc'
                        }
                    }
                },
                scales: {
                    r: {
                        angleLines: {
                            color: '#374151'
                        },
                        grid: {
                            color: '#374151'
                        },
                        pointLabels: {
                            color: '#cbd5e1'
                        },
                        ticks: {
                            color: '#cbd5e1',
                            backdropColor: 'transparent'
                        },
                        beginAtZero: true,
                        max: 100
                    }
                }
            }
        });
    }

    // Create heatmap for system activity
    createHeatmapChart() {
        const ctx = document.getElementById('heatmapChart');
        if (!ctx) return null;

        return new Chart(ctx, {
            type: 'matrix',
            data: {
                datasets: [{
                    label: 'System Activity Heatmap',
                    data: this.generateHeatmapData(),
                    backgroundColor(context) {
                        const value = context.dataset.data[context.dataIndex].v;
                        const alpha = (value + 1) / 2;
                        return `rgba(59, 130, 246, ${alpha})`;
                    },
                    borderColor: '#374151',
                    borderWidth: 1,
                    width: ({chart}) => (chart.chartArea || {}).width / 24 - 1,
                    height: ({chart}) => (chart.chartArea || {}).height / 7 - 1
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: {
                        display: false
                    },
                    tooltip: {
                        callbacks: {
                            title() {
                                return '';
                            },
                            label(context) {
                                const v = context.dataset.data[context.dataIndex];
                                return ['Day: ' + v.day, 'Hour: ' + v.hour, 'Activity: ' + v.v];
                            }
                        }
                    }
                },
                scales: {
                    x: {
                        type: 'linear',
                        display: true,
                        position: 'bottom',
                        min: 0,
                        max: 23,
                        ticks: {
                            stepSize: 2,
                            color: '#cbd5e1'
                        },
                        grid: {
                            display: false
                        },
                        title: {
                            display: true,
                            text: 'Hour of Day',
                            color: '#f8fafc'
                        }
                    },
                    y: {
                        type: 'linear',
                        display: true,
                        min: 0,
                        max: 6,
                        ticks: {
                            callback(value) {
                                const days = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];
                                return days[value] || '';
                            },
                            color: '#cbd5e1'
                        },
                        grid: {
                            display: false
                        },
                        title: {
                            display: true,
                            text: 'Day of Week',
                            color: '#f8fafc'
                        }
                    }
                }
            }
        });
    }

    // Create error rate trend chart
    createErrorTrendChart() {
        const ctx = document.getElementById('errorTrendChart');
        if (!ctx) return null;

        return new Chart(ctx, {
            type: 'bar',
            data: {
                labels: [],
                datasets: [
                    {
                        label: 'Critical Errors',
                        data: [],
                        backgroundColor: '#ef4444',
                        borderRadius: 4,
                        borderSkipped: false,
                    },
                    {
                        label: 'Warnings',
                        data: [],
                        backgroundColor: '#f59e0b',
                        borderRadius: 4,
                        borderSkipped: false,
                    },
                    {
                        label: 'Info',
                        data: [],
                        backgroundColor: '#3b82f6',
                        borderRadius: 4,
                        borderSkipped: false,
                    }
                ]
            },
            options: {
                ...this.chartDefaults,
                scales: {
                    ...this.chartDefaults.scales,
                    y: {
                        ...this.chartDefaults.scales.y,
                        stacked: true,
                        title: {
                            display: true,
                            text: 'Number of Events',
                            color: '#f8fafc'
                        }
                    },
                    x: {
                        ...this.chartDefaults.scales.x,
                        stacked: true,
                        title: {
                            display: true,
                            text: 'Time Period',
                            color: '#f8fafc'
                        }
                    }
                },
                plugins: {
                    ...this.chartDefaults.plugins,
                    tooltip: {
                        mode: 'index',
                        intersect: false,
                        backgroundColor: 'rgba(30, 41, 59, 0.9)',
                        titleColor: '#f8fafc',
                        bodyColor: '#cbd5e1',
                        borderColor: '#374151',
                        borderWidth: 1
                    }
                }
            }
        });
    }

    // Create top processes chart
    createTopProcessesChart() {
        const ctx = document.getElementById('topProcessesChart');
        if (!ctx) return null;

        return new Chart(ctx, {
            type: 'horizontalBar',
            data: {
                labels: [],
                datasets: [{
                    label: 'CPU Usage %',
                    data: [],
                    backgroundColor: [
                        '#3b82f6', '#10b981', '#f59e0b', '#ef4444', 
                        '#8b5cf6', '#06b6d4', '#84cc16', '#f97316'
                    ],
                    borderWidth: 0,
                    borderRadius: 4
                }]
            },
            options: {
                ...this.chartDefaults,
                indexAxis: 'y',
                scales: {
                    x: {
                        ...this.chartDefaults.scales.x,
                        max: 100,
                        title: {
                            display: true,
                            text: 'CPU Usage %',
                            color: '#f8fafc'
                        }
                    },
                    y: {
                        ...this.chartDefaults.scales.y,
                        title: {
                            display: true,
                            text: 'Process',
                            color: '#f8fafc'
                        }
                    }
                },
                plugins: {
                    ...this.chartDefaults.plugins,
                    legend: {
                        display: false
                    },
                    tooltip: {
                        callbacks: {
                            label(context) {
                                return `CPU: ${context.parsed.x}%`;
                            }
                        }
                    }
                }
            }
        });
    }

    // Utility methods
    getUsageColor(value, maxValue, colorScheme) {
        const percentage = (value / maxValue) * 100;
        if (percentage < 50) return colorScheme[0];
        if (percentage < 80) return colorScheme[1];
        return colorScheme[2];
    }

    generateHeatmapData() {
        const data = [];
        for (let day = 0; day < 7; day++) {
            for (let hour = 0; hour < 24; hour++) {
                data.push({
                    x: hour,
                    y: day,
                    v: Math.random() * 2 - 1, // Random value between -1 and 1
                    day: ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'][day],
                    hour: hour
                });
            }
        }
        return data;
    }

    // Update real-time data for charts
    updateChartData(chart, newData, maxDataPoints = 50) {
        if (!chart) return;

        // For line and bar charts
        if (chart.config.type === 'line' || chart.config.type === 'bar' || chart.config.type === 'horizontalBar') {
            if (chart.data.labels.length >= maxDataPoints) {
                chart.data.labels.shift();
                chart.data.datasets.forEach(dataset => dataset.data.shift());
            }
            
            chart.data.labels.push(new Date().toLocaleTimeString());
            chart.data.datasets.forEach((dataset, index) => {
                if (newData[index] !== undefined) {
                    dataset.data.push(newData[index]);
                }
            });
        }
        
        // For doughnut charts
        if (chart.config.type === 'doughnut') {
            chart.data.datasets[0].data = [newData[0], newData[1]];
        }
        
        chart.update('none'); // No animation for real-time updates
    }

    // Animate chart updates
    animateChartUpdate(chart) {
        if (chart) {
            chart.update('active');
        }
    }

    // Resize chart when container resizes
    resizeChart(chart) {
        if (chart) {
            chart.resize();
        }
    }

    // Destroy chart
    destroyChart(chart) {
        if (chart) {
            chart.destroy();
        }
    }

    // Export chart as image
    exportChart(chart, filename) {
        if (chart) {
            const url = chart.toBase64Image();
            const link = document.createElement('a');
            link.download = `${filename}.png`;
            link.href = url;
            link.click();
        }
    }

    // Update chart theme
    updateChartTheme(chart, isDark = true) {
        if (!chart) return;

        const colors = isDark ? {
            text: '#f8fafc',
            grid: '#374151',
            ticks: '#cbd5e1'
        } : {
            text: '#1e293b',
            grid: '#e2e8f0',
            ticks: '#64748b'
        };

        chart.options.plugins.legend.labels.color = colors.text;
        
        if (chart.options.scales) {
            Object.keys(chart.options.scales).forEach(key => {
                const scale = chart.options.scales[key];
                if (scale.ticks) {
                    scale.ticks.color = colors.ticks;
                }
                if (scale.grid) {
                    scale.grid.color = colors.grid;
                }
                if (scale.title) {
                    scale.title.color = colors.text;
                }
            });
        }

        chart.update('none');
    }

    // Create custom plugin for gauge text
    createGaugeTextPlugin() {
        return {
            id: 'gaugeText',
            afterDraw(chart, args, options) {
                if (chart.config.type === 'doughnut') {
                    const {ctx, chartArea: {left, right, top, bottom, width, height}} = chart;
                    
                    ctx.save();
                    const text = options.text || '0%';
                    const subtext = options.subtext || '';
                    
                    // Main text
                    ctx.font = 'bold 24px Arial';
                    ctx.fillStyle = options.color || '#f8fafc';
                    ctx.textAlign = 'center';
                    ctx.textBaseline = 'middle';
                    ctx.fillText(text, left + width / 2, top + height / 2);
                    
                    // Subtext
                    if (subtext) {
                        ctx.font = '12px Arial';
                        ctx.fillStyle = '#cbd5e1';
                        ctx.fillText(subtext, left + width / 2, top + height / 2 + 20);
                    }
                    
                    ctx.restore();
                }
            }
        };
    }
}

// Register custom chart types and plugins
Chart.register(
    // Custom plugins
    {
        id: 'centerText',
        afterDraw: function(chart) {
            if (chart.config.type === 'doughnut') {
                const ctx = chart.ctx;
                const centerX = chart.chartArea.left + chart.chartArea.width / 2;
                const centerY = chart.chartArea.top + chart.chartArea.height / 2;
                
                ctx.save();
                ctx.font = 'bold 24px Arial';
                ctx.fillStyle = '#f8fafc';
                ctx.textAlign = 'center';
                ctx.textBaseline = 'middle';
                
                // Get the percentage value
                const value = chart.data.datasets[0].data[0];
                const percentage = Math.round(value);
                
                ctx.fillText(`${percentage}%`, centerX, centerY);
                ctx.restore();
            }
        }
    }
);

// Export for use in main dashboard
if (typeof window !== 'undefined') {
    window.ChartManager = ChartManager;
}