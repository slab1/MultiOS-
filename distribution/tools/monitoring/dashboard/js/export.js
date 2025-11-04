// Export functionality for monitoring dashboard
// Handles data export, report generation, and file operations

class ExportManager {
    constructor() {
        this.exportFormats = ['json', 'csv', 'pdf', 'xlsx', 'xml'];
        this.supportedCharts = ['png', 'svg', 'pdf'];
        this.maxExportSize = 100 * 1024 * 1024; // 100MB limit
        this.currentExport = null;
        
        // Export templates
        this.templates = {
            system_report: {
                title: 'System Health Report',
                sections: ['overview', 'performance', 'alerts', 'logs']
            },
            compliance_report: {
                title: 'Compliance Report',
                sections: ['compliance_status', 'audit_trail', 'security_events']
            },
            educational_report: {
                title: 'Educational Analytics Report',
                sections: ['lab_usage', 'student_activity', 'resource_utilization', 'kpis']
            },
            network_report: {
                title: 'Network Performance Report',
                sections: ['traffic_analysis', 'bandwidth_usage', 'connection_stats', 'security_events']
            },
            security_report: {
                title: 'Security Monitoring Report',
                sections: ['security_events', 'threat_detection', 'access_control', 'audit_trail']
            }
        };
    }

    // Export dashboard data
    async exportDashboardData(options = {}) {
        const {
            format = 'json',
            timeRange = '24h',
            includeCharts = false,
            includeLogs = true,
            includeMetrics = true,
            sections = null,
            filename = null
        } = options;

        try {
            this.showProgress('Preparing export...');
            
            // Collect data
            const exportData = await this.collectExportData({
                timeRange,
                includeCharts,
                includeLogs,
                includeMetrics,
                sections
            });

            // Generate filename
            const finalFilename = filename || this.generateFilename('dashboard_export', format);

            // Export based on format
            let success = false;
            switch (format.toLowerCase()) {
                case 'json':
                    success = await this.exportAsJSON(exportData, finalFilename);
                    break;
                case 'csv':
                    success = await this.exportAsCSV(exportData, finalFilename);
                    break;
                case 'pdf':
                    success = await this.exportAsPDF(exportData, finalFilename);
                    break;
                case 'xlsx':
                    success = await this.exportAsExcel(exportData, finalFilename);
                    break;
                case 'xml':
                    success = await this.exportAsXML(exportData, finalFilename);
                    break;
                default:
                    throw new Error(`Unsupported export format: ${format}`);
            }

            if (success) {
                this.showNotification('Export completed successfully', 'success');
            }

            return success;
        } catch (error) {
            console.error('Export error:', error);
            this.showNotification(`Export failed: ${error.message}`, 'error');
            return false;
        } finally {
            this.hideProgress();
        }
    }

    // Export chart as image
    async exportChart(chartId, options = {}) {
        const {
            format = 'png',
            quality = 1.0,
            backgroundColor = 'white',
            filename = null,
            width = null,
            height = null
        } = options;

        try {
            const chart = Chart.getChart(chartId);
            if (!chart) {
                throw new Error(`Chart not found: ${chartId}`);
            }

            // Configure export settings
            const originalWidth = chart.width;
            const originalHeight = chart.height;
            
            if (width || height) {
                chart.resize(width, height);
                chart.update();
            }

            // Generate filename
            const finalFilename = filename || `${chartId}_${Date.now()}.${format}`;

            // Export chart
            let dataURL = null;
            switch (format.toLowerCase()) {
                case 'png':
                    dataURL = chart.toBase64Image('image/png', quality);
                    break;
                case 'jpeg':
                case 'jpg':
                    dataURL = chart.toBase64Image('image/jpeg', quality);
                    break;
                case 'svg':
                    dataURL = chart.toBase64Image();
                    break;
                default:
                    throw new Error(`Unsupported chart format: ${format}`);
            }

            // Restore original size
            if (width || height) {
                chart.resize(originalWidth, originalHeight);
                chart.update();
            }

            // Download file
            const success = await this.downloadDataURL(dataURL, finalFilename, format);
            
            if (success) {
                this.showNotification(`Chart exported as ${format.toUpperCase()}`, 'success');
            }

            return success;
        } catch (error) {
            console.error('Chart export error:', error);
            this.showNotification(`Chart export failed: ${error.message}`, 'error');
            return false;
        }
    }

    // Generate compliance report
    async generateComplianceReport(standard, options = {}) {
        const {
            period = 'month',
            format = 'pdf',
            includeCharts = true,
            includeRecommendations = true,
            filename = null
        } = options;

        try {
            this.showProgress('Generating compliance report...');

            // Collect compliance data
            const complianceData = await this.collectComplianceData(standard, period);
            
            // Generate report content
            const report = {
                metadata: {
                    standard: standard,
                    period: period,
                    generated_at: new Date().toISOString(),
                    version: '1.0'
                },
                summary: this.generateComplianceSummary(complianceData),
                detailed_analysis: complianceData.detailed_analysis,
                recommendations: includeRecommendations ? this.generateRecommendations(complianceData) : [],
                charts: includeCharts ? await this.generateComplianceCharts(complianceData) : []
            };

            // Generate filename
            const finalFilename = filename || `${standard}_compliance_${period}_${Date.now()}.${format}`;

            // Export report
            let success = false;
            switch (format.toLowerCase()) {
                case 'pdf':
                    success = await this.exportComplianceAsPDF(report, finalFilename);
                    break;
                case 'html':
                    success = await this.exportComplianceAsHTML(report, finalFilename);
                    break;
                case 'json':
                    success = await this.exportAsJSON(report, finalFilename);
                    break;
                default:
                    throw new Error(`Unsupported report format: ${format}`);
            }

            if (success) {
                this.showNotification('Compliance report generated successfully', 'success');
            }

            return success;
        } catch (error) {
            console.error('Compliance report error:', error);
            this.showNotification(`Report generation failed: ${error.message}`, 'error');
            return false;
        } finally {
            this.hideProgress();
        }
    }

    // Export system metrics
    async exportSystemMetrics(options = {}) {
        const {
            timeRange = '24h',
            format = 'csv',
            includeHistorical = true,
            granularity = '5m',
            filename = null
        } = options;

        try {
            this.showProgress('Exporting system metrics...');

            // Collect metrics data
            const metricsData = await this.collectMetricsData(timeRange, granularity);
            
            // Process data based on format
            let processedData = metricsData;
            if (format.toLowerCase() === 'csv') {
                processedData = this.convertMetricsToCSV(metricsData);
            }

            // Generate filename
            const finalFilename = filename || `system_metrics_${timeRange}_${Date.now()}.${format}`;

            // Export data
            let success = false;
            if (format.toLowerCase() === 'csv') {
                success = await this.downloadText(processedData, finalFilename, 'text/csv');
            } else {
                success = await this.exportAsJSON(metricsData, finalFilename);
            }

            if (success) {
                this.showNotification('System metrics exported successfully', 'success');
            }

            return success;
        } catch (error) {
            console.error('Metrics export error:', error);
            this.showNotification(`Metrics export failed: ${error.message}`, 'error');
            return false;
        } finally {
            this.hideProgress();
        }
    }

    // Export logs
    async exportLogs(options = {}) {
        const {
            timeRange = '24h',
            logLevel = 'all',
            format = 'csv',
            maxEntries = 10000,
            filename = null
        } = options;

        try {
            this.showProgress('Exporting logs...');

            // Collect log data
            const logData = await this.collectLogData(timeRange, logLevel, maxEntries);
            
            // Generate filename
            const finalFilename = filename || `logs_${timeRange}_${Date.now()}.${format}`;

            // Export logs
            let success = false;
            switch (format.toLowerCase()) {
                case 'csv':
                    const csvData = this.convertLogsToCSV(logData);
                    success = await this.downloadText(csvData, finalFilename, 'text/csv');
                    break;
                case 'json':
                    success = await this.exportAsJSON(logData, finalFilename);
                    break;
                case 'txt':
                    const textData = this.convertLogsToText(logData);
                    success = await this.downloadText(textData, finalFilename, 'text/plain');
                    break;
                default:
                    throw new Error(`Unsupported log format: ${format}`);
            }

            if (success) {
                this.showNotification('Logs exported successfully', 'success');
            }

            return success;
        } catch (error) {
            console.error('Logs export error:', error);
            this.showNotification(`Logs export failed: ${error.message}`, 'error');
            return false;
        } finally {
            this.hideProgress();
        }
    }

    // Export educational analytics
    async exportEducationalAnalytics(options = {}) {
        const {
            timeRange = '30d',
            format = 'xlsx',
            includeCharts = true,
            anonymize = true,
            filename = null
        } = options;

        try {
            this.showProgress('Exporting educational analytics...');

            // Collect educational data
            const eduData = await this.collectEducationalData(timeRange, anonymize);
            
            // Generate filename
            const finalFilename = filename || `educational_analytics_${timeRange}_${Date.now()}.${format}`;

            // Export data
            let success = false;
            switch (format.toLowerCase()) {
                case 'xlsx':
                    success = await this.exportEducationalAsExcel(eduData, finalFilename, includeCharts);
                    break;
                case 'pdf':
                    success = await this.exportEducationalAsPDF(eduData, finalFilename, includeCharts);
                    break;
                case 'json':
                    success = await this.exportAsJSON(eduData, finalFilename);
                    break;
                default:
                    throw new Error(`Unsupported format: ${format}`);
            }

            if (success) {
                this.showNotification('Educational analytics exported successfully', 'success');
            }

            return success;
        } catch (error) {
            console.error('Educational analytics export error:', error);
            this.showNotification(`Export failed: ${error.message}`, 'error');
            return false;
        } finally {
            this.hideProgress();
        }
    }

    // Helper methods for data collection
    async collectExportData(options) {
        const data = {
            timestamp: new Date().toISOString(),
            export_options: options,
            system_metrics: null,
            alerts: null,
            logs: null,
            network_stats: null,
            security_events: null,
            educational_data: null
        };

        const promises = [];

        if (options.includeMetrics !== false) {
            promises.push(
                fetch('/api/system/metrics')
                    .then(res => res.json())
                    .then(data => { data.system_metrics = data; })
                    .catch(() => { data.system_metrics = {}; })
            );
        }

        if (options.includeLogs !== false) {
            promises.push(
                fetch('/api/logs?limit=1000')
                    .then(res => res.json())
                    .then(data => { data.logs = data; })
                    .catch(() => { data.logs = []; })
            );
        }

        promises.push(
            fetch('/api/alerts')
                .then(res => res.json())
                .then(data => { data.alerts = data; })
                .catch(() => { data.alerts = []; })
        );

        promises.push(
            fetch('/api/network/stats')
                .then(res => res.json())
                .then(data => { data.network_stats = data; })
                .catch(() => { data.network_stats = {}; })
        );

        promises.push(
            fetch('/api/security/events')
                .then(res => res.json())
                .then(data => { data.security_events = data; })
                .catch(() => { data.security_events = []; })
        );

        promises.push(
            fetch('/api/educational/analytics')
                .then(res => res.json())
                .then(data => { data.educational_data = data; })
                .catch(() => { data.educational_data = {}; })
        );

        await Promise.all(promises);
        return data;
    }

    async collectComplianceData(standard, period) {
        const response = await fetch(`/api/compliance/${standard}?period=${period}`);
        return await response.json();
    }

    async collectMetricsData(timeRange, granularity) {
        const response = await fetch(`/api/metrics?range=${timeRange}&granularity=${granularity}`);
        return await response.json();
    }

    async collectLogData(timeRange, level, maxEntries) {
        const params = new URLSearchParams({
            range: timeRange,
            level: level,
            limit: maxEntries.toString()
        });
        const response = await fetch(`/api/logs?${params}`);
        return await response.json();
    }

    async collectEducationalData(timeRange, anonymize) {
        const params = new URLSearchParams({
            range: timeRange,
            anonymize: anonymize.toString()
        });
        const response = await fetch(`/api/educational/analytics?${params}`);
        return await response.json();
    }

    // Export format implementations
    async exportAsJSON(data, filename) {
        const jsonString = JSON.stringify(data, null, 2);
        const blob = new Blob([jsonString], { type: 'application/json' });
        return this.downloadBlob(blob, filename);
    }

    async exportAsCSV(data, filename) {
        // Convert data to CSV format
        const csvString = this.convertDataToCSV(data);
        const blob = new Blob([csvString], { type: 'text/csv' });
        return this.downloadBlob(blob, filename);
    }

    async exportAsPDF(data, filename) {
        // Create PDF using jsPDF
        try {
            if (!window.jsPDF) {
                // Load jsPDF dynamically
                await this.loadScript('https://cdnjs.cloudflare.com/ajax/libs/jspdf/2.5.1/jspdf.umd.min.js');
            }

            const { jsPDF } = window.jspdf;
            const pdf = new jsPDF();

            // Add title
            pdf.setFontSize(20);
            pdf.text('MultiOS Monitoring Report', 20, 30);

            // Add timestamp
            pdf.setFontSize(12);
            pdf.text(`Generated: ${new Date().toLocaleString()}`, 20, 45);

            // Add summary data
            let y = 60;
            if (data.system_metrics) {
                pdf.text(`CPU Usage: ${data.system_metrics.cpu_usage || 0}%`, 20, y);
                y += 10;
                pdf.text(`Memory Usage: ${data.system_metrics.memory_usage || 0}%`, 20, y);
                y += 10;
                pdf.text(`Disk Usage: ${data.system_metrics.disk_usage || 0}%`, 20, y);
                y += 20;
            }

            // Add alerts section
            if (data.alerts && data.alerts.length > 0) {
                pdf.text(`Active Alerts: ${data.alerts.length}`, 20, y);
                y += 20;

                data.alerts.forEach(alert => {
                    pdf.text(`${alert.severity}: ${alert.message}`, 20, y);
                    y += 10;
                    if (y > 250) {
                        pdf.addPage();
                        y = 30;
                    }
                });
            }

            const pdfBlob = pdf.output('blob');
            return this.downloadBlob(pdfBlob, filename);
        } catch (error) {
            console.error('PDF generation error:', error);
            throw new Error('Failed to generate PDF');
        }
    }

    async exportAsExcel(data, filename) {
        try {
            // This would require a library like SheetJS
            // For now, fallback to CSV
            console.warn('Excel export not fully implemented, falling back to CSV');
            return this.exportAsCSV(data, filename.replace('.xlsx', '.csv'));
        } catch (error) {
            console.error('Excel export error:', error);
            throw new Error('Failed to export as Excel');
        }
    }

    async exportAsXML(data, filename) {
        const xmlString = this.convertToXML(data);
        const blob = new Blob([xmlString], { type: 'application/xml' });
        return this.downloadBlob(blob, filename);
    }

    // Download helpers
    async downloadBlob(blob, filename) {
        try {
            const url = URL.createObjectURL(blob);
            const link = document.createElement('a');
            link.href = url;
            link.download = filename;
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);
            URL.revokeObjectURL(url);
            return true;
        } catch (error) {
            console.error('Download error:', error);
            return false;
        }
    }

    async downloadDataURL(dataURL, filename, format) {
        try {
            const link = document.createElement('a');
            link.href = dataURL;
            link.download = filename;
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);
            return true;
        } catch (error) {
            console.error('DataURL download error:', error);
            return false;
        }
    }

    async downloadText(text, filename, mimeType) {
        const blob = new Blob([text], { type: mimeType });
        return this.downloadBlob(blob, filename);
    }

    // Conversion helpers
    convertDataToCSV(data) {
        let csv = '';
        
        // System metrics
        if (data.system_metrics) {
            csv += 'Metric,Value,Unit\n';
            csv += `CPU Usage,${data.system_metrics.cpu_usage || 0},%\n`;
            csv += `Memory Usage,${data.system_metrics.memory_usage || 0},%\n`;
            csv += `Disk Usage,${data.system_metrics.disk_usage || 0},%\n`;
            csv += '\n';
        }

        // Alerts
        if (data.alerts && data.alerts.length > 0) {
            csv += 'Alert Timestamp,Severity,Source,Message,Status\n';
            data.alerts.forEach(alert => {
                csv += `${alert.timestamp},${alert.severity},${alert.source},"${alert.message}",${alert.status}\n`;
            });
            csv += '\n';
        }

        return csv;
    }

    convertLogsToCSV(logs) {
        if (!Array.isArray(logs)) return '';
        
        let csv = 'Timestamp,Level,Source,Message\n';
        logs.forEach(log => {
            csv += `${log.timestamp},${log.level},${log.source},"${log.message}"\n`;
        });
        return csv;
    }

    convertLogsToText(logs) {
        if (!Array.isArray(logs)) return '';
        
        return logs.map(log => 
            `[${log.timestamp}] ${log.level.toUpperCase()}: ${log.message}`
        ).join('\n');
    }

    convertToXML(data) {
        let xml = '<?xml version="1.0" encoding="UTF-8"?>\n<monitoring_report>\n';
        
        if (data.system_metrics) {
            xml += '  <system_metrics>\n';
            xml += `    <cpu_usage>${data.system_metrics.cpu_usage || 0}</cpu_usage>\n`;
            xml += `    <memory_usage>${data.system_metrics.memory_usage || 0}</memory_usage>\n`;
            xml += `    <disk_usage>${data.system_metrics.disk_usage || 0}</disk_usage>\n`;
            xml += '  </system_metrics>\n';
        }
        
        xml += '</monitoring_report>';
        return xml;
    }

    // Utility methods
    generateFilename(prefix, format) {
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        return `${prefix}_${timestamp}.${format}`;
    }

    async loadScript(src) {
        return new Promise((resolve, reject) => {
            const script = document.createElement('script');
            script.src = src;
            script.onload = resolve;
            script.onerror = reject;
            document.head.appendChild(script);
        });
    }

    generateComplianceSummary(data) {
        return {
            overall_score: data.overall_score || 0,
            compliant_items: data.compliant_items || 0,
            non_compliant_items: data.non_compliant_items || 0,
            total_items: data.total_items || 0,
            critical_issues: data.critical_issues || 0,
            last_audit: data.last_audit || null
        };
    }

    generateRecommendations(data) {
        const recommendations = [];
        
        // Generate recommendations based on data
        if (data.non_compliant_items > 0) {
            recommendations.push({
                priority: 'high',
                category: 'compliance',
                description: `Address ${data.non_compliant_items} non-compliant items to improve overall compliance score`,
                estimated_effort: 'medium'
            });
        }

        if (data.critical_issues > 0) {
            recommendations.push({
                priority: 'critical',
                category: 'security',
                description: `Resolve ${data.critical_issues} critical security issues immediately`,
                estimated_effort: 'high'
            });
        }

        return recommendations;
    }

    async generateComplianceCharts(data) {
        const charts = [];
        
        // This would generate chart data for compliance visualization
        // Implementation depends on specific chart requirements
        
        return charts;
    }

    showProgress(message) {
        // Create or update progress indicator
        let progressEl = document.getElementById('export-progress');
        if (!progressEl) {
            progressEl = document.createElement('div');
            progressEl.id = 'export-progress';
            progressEl.className = 'export-progress';
            document.body.appendChild(progressEl);
        }
        
        progressEl.innerHTML = `
            <div class="progress-modal">
                <div class="progress-content">
                    <div class="progress-spinner"></div>
                    <div class="progress-message">${message}</div>
                </div>
            </div>
        `;
    }

    hideProgress() {
        const progressEl = document.getElementById('export-progress');
        if (progressEl) {
            progressEl.remove();
        }
    }

    showNotification(message, type = 'info') {
        // Show user notification
        const notification = document.createElement('div');
        notification.className = `notification ${type}`;
        notification.innerHTML = `
            <div class="notification-message">${message}</div>
        `;
        
        document.body.appendChild(notification);
        
        setTimeout(() => {
            if (notification.parentNode) {
                notification.parentNode.removeChild(notification);
            }
        }, 5000);
    }

    // Validation methods
    validateExportOptions(options) {
        const errors = [];
        
        if (!this.exportFormats.includes(options.format.toLowerCase())) {
            errors.push(`Unsupported format: ${options.format}`);
        }
        
        if (options.timeRange && !/^\d+[smhdw]$/.test(options.timeRange)) {
            errors.push('Invalid time range format. Use patterns like 30s, 5m, 2h, 1d, 1w');
        }
        
        if (options.maxEntries && options.maxEntries > 100000) {
            errors.push('Maximum export entries limited to 100,000');
        }
        
        return errors;
    }
}

// Export for global use
if (typeof window !== 'undefined') {
    window.ExportManager = ExportManager;
    window.exportManager = new ExportManager();
}