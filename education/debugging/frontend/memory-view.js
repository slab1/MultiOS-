/**
 * Memory View Module
 * Handles memory visualization, heap/stack inspection, and real-time memory monitoring
 */

class MemoryView {
    constructor() {
        this.app = null;
        this.canvas = null;
        this.ctx = null;
        this.memoryData = null;
        this.viewType = 'heap'; // heap, stack, code, data
        this.animationId = null;
        this.isAnimating = false;
        this.selectedRegion = null;
        this.memoryUsage = {
            total: 0,
            used: 0,
            free: 0,
            percentage: 0
        };
        
        // Memory regions data
        this.regions = [];
        this.updateInterval = null;
    }

    setApp(app) {
        this.app = app;
        this.initializeElements();
        this.initializeEventListeners();
    }

    initializeElements() {
        this.canvas = document.getElementById('memoryCanvas');
        if (this.canvas) {
            this.ctx = this.canvas.getContext('2d');
        }
    }

    initializeEventListeners() {
        if (this.canvas) {
            this.canvas.addEventListener('click', (e) => this.handleCanvasClick(e));
            this.canvas.addEventListener('mousemove', (e) => this.handleCanvasMouseMove(e));
        }
    }

    async refresh() {
        try {
            this.showLoading();
            await this.fetchMemoryData();
            this.renderMemoryVisualization();
            this.updateMemoryDetails();
            this.hideLoading();
        } catch (error) {
            console.error('Error refreshing memory view:', error);
            this.hideLoading();
            if (this.app) {
                this.app.showNotification(`Memory refresh failed: ${error.message}`, 'error');
            }
        }
    }

    async fetchMemoryData() {
        try {
            const response = await fetch(`/api/debug/memory?view=${this.viewType}`);
            if (!response.ok) {
                throw new Error(`Failed to fetch memory data: ${response.statusText}`);
            }
            
            this.memoryData = await response.json();
            this.processMemoryData();
        } catch (error) {
            // Use mock data for demonstration
            this.memoryData = this.generateMockMemoryData();
            this.processMemoryData();
        }
    }

    generateMockMemoryData() {
        const regions = [];
        let currentAddress = 0x1000;
        
        // Generate mock memory regions based on view type
        const regionTypes = {
            'heap': ['heap', 'free', 'heap', 'heap', 'free'],
            'stack': ['stack', 'stack', 'stack', 'stack'],
            'code': ['code', 'code', 'code'],
            'data': ['data', 'bss', 'data']
        };
        
        const type = this.viewType;
        const regionTypeList = regionTypes[type] || ['heap', 'free'];
        
        for (let i = 0; i < regionTypeList.length; i++) {
            const regionType = regionTypeList[i];
            const size = this.getRegionSize(regionType, i);
            
            regions.push({
                id: i,
                type: regionType,
                address: currentAddress,
                size: size,
                used: regionType !== 'free',
                label: this.getRegionLabel(regionType, i),
                protection: this.getRegionProtection(regionType),
                allocation: this.getAllocationInfo(regionType),
                timestamp: Date.now() - Math.random() * 3600000
            });
            
            currentAddress += size;
        }
        
        return {
            regions,
            summary: {
                totalSize: currentAddress - 0x1000,
                usedSize: regions.filter(r => r.used).reduce((sum, r) => sum + r.size, 0),
                freeSize: regions.filter(r => !r.used).reduce((sum, r) => sum + r.size, 0)
            }
        };
    }

    getRegionSize(type, index) {
        const sizeMap = {
            'heap': [0x1000, 0x800, 0x2000, 0x400, 0x1000],
            'stack': [0x4000, 0x2000, 0x1000, 0x800],
            'code': [0x2000, 0x1500, 0x1800],
            'data': [0x500, 0x300, 0x400]
        };
        
        const sizes = sizeMap[this.viewType] || [0x1000];
        return sizes[index % sizes.length];
    }

    getRegionLabel(type, index) {
        const labelMap = {
            'heap': `Heap Block ${index + 1}`,
            'stack': `Stack Frame ${index + 1}`,
            'code': `Code Segment ${index + 1}`,
            'data': `Data Section ${index + 1}`,
            'free': `Free Space ${index + 1}`,
            'bss': `BSS Segment ${index + 1}`
        };
        
        return labelMap[type] || `${type} ${index + 1}`;
    }

    getRegionProtection(type) {
        const protectionMap = {
            'heap': 'read/write',
            'stack': 'read/write',
            'code': 'read/execute',
            'data': 'read/write',
            'free': 'none',
            'bss': 'read/write'
        };
        
        return protectionMap[type] || 'read/write';
    }

    getAllocationInfo(type) {
        const allocMap = {
            'heap': 'malloc()',
            'stack': 'automatic',
            'code': 'static',
            'data': 'static',
            'free': 'available',
            'bss': 'uninitialized'
        };
        
        return allocMap[type] || 'unknown';
    }

    processMemoryData() {
        if (!this.memoryData) return;
        
        this.regions = this.memoryData.regions || [];
        this.memoryUsage = {
            total: this.memoryData.summary?.totalSize || 0,
            used: this.memoryData.summary?.usedSize || 0,
            free: this.memoryData.summary?.freeSize || 0,
            percentage: this.memoryData.summary?.usedSize && this.memoryData.summary?.totalSize 
                ? Math.round((this.memoryData.summary.usedSize / this.memoryData.summary.totalSize) * 100)
                : 0
        };
    }

    renderMemoryVisualization() {
        if (!this.ctx || !this.canvas) return;
        
        // Clear canvas
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
        
        if (this.regions.length === 0) {
            this.renderEmptyState();
            return;
        }
        
        // Calculate layout
        const totalSize = this.regions.reduce((sum, region) => sum + region.size, 0);
        const canvasHeight = this.canvas.height - 40; // Leave space for labels
        
        let yOffset = 20;
        
        // Render each region
        this.regions.forEach((region, index) => {
            const height = (region.size / totalSize) * canvasHeight;
            const color = this.getRegionColor(region);
            const isSelected = this.selectedRegion === region.id;
            
            // Draw region rectangle
            this.ctx.fillStyle = color;
            this.ctx.fillRect(10, yOffset, this.canvas.width - 20, height);
            
            // Draw border
            this.ctx.strokeStyle = isSelected ? '#ff0000' : '#333';
            this.ctx.lineWidth = isSelected ? 3 : 1;
            this.ctx.strokeRect(10, yOffset, this.canvas.width - 20, height);
            
            // Draw region label
            this.ctx.fillStyle = this.getTextColor(color);
            this.ctx.font = '12px Arial';
            this.ctx.textAlign = 'center';
            
            const label = `${region.label}\n${this.formatSize(region.size)}`;
            const lines = label.split('\n');
            lines.forEach((line, lineIndex) => {
                this.ctx.fillText(
                    line,
                    this.canvas.width / 2,
                    yOffset + (height / 2) + (lineIndex - lines.length / 2 + 0.5) * 16
                );
            });
            
            // Draw address
            this.ctx.fillStyle = '#666';
            this.ctx.font = '10px Arial';
            this.ctx.textAlign = 'left';
            this.ctx.fillText(`0x${region.address.toString(16).toUpperCase()}`, 15, yOffset + 15);
            
            yOffset += height;
        });
        
        // Draw legend
        this.drawLegend();
    }

    renderEmptyState() {
        this.ctx.fillStyle = '#f0f0f0';
        this.ctx.fillRect(10, 20, this.canvas.width - 20, this.canvas.height - 40);
        
        this.ctx.fillStyle = '#666';
        this.ctx.font = '16px Arial';
        this.ctx.textAlign = 'center';
        this.ctx.fillText('No Memory Data Available', this.canvas.width / 2, this.canvas.height / 2);
    }

    drawLegend() {
        const legendY = this.canvas.height - 35;
        const legendX = 10;
        const legendWidth = this.canvas.width - 20;
        
        // Background
        this.ctx.fillStyle = '#f8f8f8';
        this.ctx.fillRect(legendX, legendY, legendWidth, 30);
        this.ctx.strokeStyle = '#ddd';
        this.ctx.strokeRect(legendX, legendY, legendWidth, 30);
        
        // Legend items
        const legendItems = [
            { type: 'heap', label: 'Heap' },
            { type: 'stack', label: 'Stack' },
            { type: 'code', label: 'Code' },
            { type: 'data', label: 'Data' },
            { type: 'free', label: 'Free' }
        ];
        
        const itemWidth = legendWidth / legendItems.length;
        legendItems.forEach((item, index) => {
            const x = legendX + index * itemWidth + 5;
            
            // Color box
            this.ctx.fillStyle = this.getRegionColor({ type: item.type });
            this.ctx.fillRect(x, legendY + 5, 15, 15);
            this.ctx.strokeStyle = '#333';
            this.ctx.strokeRect(x, legendY + 5, 15, 15);
            
            // Label
            this.ctx.fillStyle = '#333';
            this.ctx.font = '11px Arial';
            this.ctx.textAlign = 'left';
            this.ctx.fillText(item.label, x + 20, legendY + 17);
        });
    }

    getRegionColor(region) {
        const colorMap = {
            'heap': 'rgba(139, 92, 246, 0.7)',      // Purple
            'stack': 'rgba(16, 185, 129, 0.7)',     // Green
            'code': 'rgba(239, 68, 68, 0.7)',       // Red
            'data': 'rgba(245, 158, 11, 0.7)',      // Orange
            'free': 'rgba(156, 163, 175, 0.3)',     // Gray
            'bss': 'rgba(59, 130, 246, 0.7)'        // Blue
        };
        
        return colorMap[region.type] || 'rgba(156, 163, 175, 0.7)';
    }

    getTextColor(backgroundColor) {
        // Simple heuristic: if background is dark, use white text
        const rgb = backgroundColor.match(/\d+/g);
        if (rgb) {
            const brightness = (parseInt(rgb[0]) + parseInt(rgb[1]) + parseInt(rgb[2])) / 3;
            return brightness < 128 ? '#fff' : '#000';
        }
        return '#000';
    }

    formatSize(bytes) {
        const units = ['B', 'KB', 'MB', 'GB'];
        let size = bytes;
        let unitIndex = 0;
        
        while (size >= 1024 && unitIndex < units.length - 1) {
            size /= 1024;
            unitIndex++;
        }
        
        return `${size.toFixed(1)} ${units[unitIndex]}`;
    }

    updateMemoryDetails() {
        const memoryDetails = document.getElementById('memoryDetails');
        if (!memoryDetails || !this.memoryData) return;
        
        const summary = this.memoryData.summary;
        const usage = this.memoryUsage;
        
        memoryDetails.innerHTML = `
            <h4>Memory Usage Summary</h4>
            <div class="memory-summary">
                <div class="summary-item">
                    <span class="summary-label">Total Size:</span>
                    <span class="summary-value">${this.formatSize(usage.total)}</span>
                </div>
                <div class="summary-item">
                    <span class="summary-label">Used:</span>
                    <span class="summary-value used">${this.formatSize(usage.used)} (${usage.percentage}%)</span>
                </div>
                <div class="summary-item">
                    <span class="summary-label">Free:</span>
                    <span class="summary-value free">${this.formatSize(usage.free)}</span>
                </div>
            </div>
            
            <div class="memory-progress">
                <div class="progress-bar">
                    <div class="progress-fill used" style="width: ${usage.percentage}%"></div>
                    <div class="progress-fill free" style="width: ${100 - usage.percentage}%; left: ${usage.percentage}%"></div>
                </div>
            </div>
            
            <div class="memory-regions">
                <h5>Memory Regions (${this.regions.length})</h5>
                ${this.regions.map(region => `
                    <div class="memory-region ${region.used ? 'used' : 'free'}">
                        <div class="region-info">
                            <span class="region-label">${region.label}</span>
                            <span class="region-size">${this.formatSize(region.size)}</span>
                        </div>
                        <div class="region-meta">
                            <span class="region-address">0x${region.address.toString(16).toUpperCase()}</span>
                            <span class="region-type">${region.type}</span>
                        </div>
                    </div>
                `).join('')}
            </div>
        `;
    }

    changeViewType(newType) {
        if (this.viewType !== newType) {
            this.viewType = newType;
            this.refresh();
        }
    }

    handleCanvasClick(event) {
        if (!this.canvas) return;
        
        const rect = this.canvas.getBoundingClientRect();
        const x = event.clientX - rect.left;
        const y = event.clientY - rect.top;
        
        const region = this.getRegionAtPosition(x, y);
        if (region) {
            this.selectRegion(region.id);
        }
    }

    handleCanvasMouseMove(event) {
        if (!this.canvas) return;
        
        const rect = this.canvas.getBoundingClientRect();
        const x = event.clientX - rect.left;
        const y = event.clientY - rect.top;
        
        const region = this.getRegionAtPosition(x, y);
        
        // Update cursor
        this.canvas.style.cursor = region ? 'pointer' : 'default';
        
        // Show tooltip if needed
        if (this.app && region) {
            this.app.updateTooltip(`Click to view details for ${region.label}`);
        }
    }

    getRegionAtPosition(x, y) {
        if (!this.regions.length) return null;
        
        const totalSize = this.regions.reduce((sum, region) => sum + region.size, 0);
        const canvasHeight = this.canvas.height - 40;
        let yOffset = 20;
        
        for (const region of this.regions) {
            const height = (region.size / totalSize) * canvasHeight;
            
            if (x >= 10 && x <= this.canvas.width - 10 && 
                y >= yOffset && y <= yOffset + height) {
                return region;
            }
            
            yOffset += height;
        }
        
        return null;
    }

    selectRegion(regionId) {
        this.selectedRegion = regionId;
        this.renderMemoryVisualization();
        this.showRegionDetails(regionId);
    }

    showRegionDetails(regionId) {
        const region = this.regions.find(r => r.id === regionId);
        if (!region) return;
        
        // Show detailed information in a modal or sidebar
        this.app.showNotification(`Selected region: ${region.label}`, 'info');
        
        // You could add more detailed region information display here
        this.displayRegionDetails(region);
    }

    displayRegionDetails(region) {
        const details = `
            <h4>${region.label}</h4>
            <div class="region-details-grid">
                <div class="detail-item">
                    <label>Address:</label>
                    <span>0x${region.address.toString(16).toUpperCase()}</span>
                </div>
                <div class="detail-item">
                    <label>Size:</label>
                    <span>${this.formatSize(region.size)}</span>
                </div>
                <div class="detail-item">
                    <label>Type:</label>
                    <span>${region.type}</span>
                </div>
                <div class="detail-item">
                    <label>Protection:</label>
                    <span>${region.protection}</span>
                </div>
                <div class="detail-item">
                    <label>Allocation:</label>
                    <span>${region.allocation}</span>
                </div>
                <div class="detail-item">
                    <label>Status:</label>
                    <span class="${region.used ? 'used' : 'free'}">${region.used ? 'Used' : 'Free'}</span>
                </div>
            </div>
        `;
        
        // Update details panel
        const memoryDetails = document.getElementById('memoryDetails');
        if (memoryDetails) {
            memoryDetails.innerHTML = details + memoryDetails.innerHTML;
        }
    }

    startRealTimeMonitoring() {
        if (this.updateInterval) return;
        
        this.updateInterval = setInterval(() => {
            this.refresh();
        }, 2000); // Update every 2 seconds
        
        this.isAnimating = true;
    }

    stopRealTimeMonitoring() {
        if (this.updateInterval) {
            clearInterval(this.updateInterval);
            this.updateInterval = null;
        }
        
        this.isAnimating = false;
    }

    exportMemorySnapshot() {
        const snapshot = {
            timestamp: new Date().toISOString(),
            viewType: this.viewType,
            memoryUsage: this.memoryUsage,
            regions: this.regions,
            canvasImage: this.canvas ? this.canvas.toDataURL() : null
        };
        
        return snapshot;
    }

    importMemorySnapshot(snapshot) {
        try {
            this.viewType = snapshot.viewType;
            this.memoryUsage = snapshot.memoryUsage;
            this.regions = snapshot.regions;
            
            this.renderMemoryVisualization();
            this.updateMemoryDetails();
            
            if (this.app) {
                this.app.showNotification('Memory snapshot imported', 'success');
            }
        } catch (error) {
            if (this.app) {
                this.app.showNotification(`Failed to import snapshot: ${error.message}`, 'error');
            }
        }
    }

    showLoading() {
        // Visual feedback for loading
        if (this.canvas) {
            this.ctx.fillStyle = 'rgba(255, 255, 255, 0.8)';
            this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
            this.ctx.fillStyle = '#666';
            this.ctx.font = '16px Arial';
            this.ctx.textAlign = 'center';
            this.ctx.fillText('Loading...', this.canvas.width / 2, this.canvas.height / 2);
        }
    }

    hideLoading() {
        this.renderMemoryVisualization();
    }

    getMemoryStats() {
        return {
            viewType: this.viewType,
            totalRegions: this.regions.length,
            totalSize: this.memoryUsage.total,
            usedSize: this.memoryUsage.used,
            freeSize: this.memoryUsage.free,
            usagePercentage: this.memoryUsage.percentage,
            isMonitoring: this.isAnimating
        };
    }

    analyzeMemoryLeaks() {
        // Mock memory leak analysis
        const leaks = [];
        
        this.regions.forEach(region => {
            if (region.used && region.allocation === 'malloc()') {
                // Simulate leak detection based on age and size
                const age = Date.now() - region.timestamp;
                if (age > 300000 && region.size > 0x1000) { // Older than 5 minutes and larger than 4KB
                    leaks.push({
                        region: region,
                        reason: 'Potential memory leak: Large allocation held too long',
                        severity: region.size > 0x8000 ? 'high' : 'medium'
                    });
                }
            }
        });
        
        return leaks;
    }

    optimizeMemory() {
        // Mock memory optimization
        const suggestions = [];
        
        const totalFree = this.regions.filter(r => !r.used).reduce((sum, r) => sum + r.size, 0);
        const totalUsed = this.regions.filter(r => r.used).reduce((sum, r) => sum + r.size, 0);
        
        if (totalFree > totalUsed * 2) {
            suggestions.push({
                type: 'defragment',
                description: 'Consider defragmenting heap memory to reduce fragmentation'
            });
        }
        
        const unusedRegions = this.regions.filter(r => !r.used);
        if (unusedRegions.length > this.regions.length * 0.5) {
            suggestions.push({
                type: 'coalesce',
                description: 'Multiple free regions detected - consider coalescing'
            });
        }
        
        return suggestions;
    }

    destroy() {
        this.stopRealTimeMonitoring();
        this.regions = [];
        this.memoryData = null;
        this.selectedRegion = null;
        
        if (this.canvas) {
            this.ctx = null;
            this.canvas = null;
        }
    }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = MemoryView;
}