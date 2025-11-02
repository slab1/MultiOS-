/**
 * Variable Inspector Module
 * Handles variable tracking, inspection, and watch expressions
 */

class VariableInspector {
    constructor() {
        this.app = null;
        this.variables = new Map();
        this.watchExpressions = new Set();
        this.variableHistory = [];
        this.currentScope = 'local'; // local, global, static
        this.variableTypes = new Set();
        this.expandedVariables = new Set();
        this.filterTypes = new Set();
        this.sortBy = 'name'; // name, type, value, line
        this.sortDirection = 'asc'; // asc, desc
    }

    setApp(app) {
        this.app = app;
    }

    async refresh() {
        try {
            await this.fetchVariables();
            this.renderVariables();
            this.renderWatchExpressions();
        } catch (error) {
            console.error('Error refreshing variables:', error);
            if (this.app) {
                this.app.showNotification(`Variable refresh failed: ${error.message}`, 'error');
            }
        }
    }

    async fetchVariables() {
        try {
            const response = await fetch('/api/debug/variables');
            if (!response.ok) {
                throw new Error(`Failed to fetch variables: ${response.statusText}`);
            }
            
            const data = await response.json();
            this.updateVariableData(data.variables);
        } catch (error) {
            // Use mock data for demonstration
            const mockVariables = this.generateMockVariables();
            this.updateVariableData(mockVariables);
        }
    }

    generateMockVariables() {
        return [
            {
                name: 'x',
                type: 'int',
                value: 42,
                scope: 'local',
                line: 10,
                address: '0x7fff5fbff5bc',
                isPointer: false,
                isArray: false,
                arrayLength: null,
                structure: null
            },
            {
                name: 'y',
                type: 'int',
                value: 84,
                scope: 'local',
                line: 11,
                address: '0x7fff5fbff5b8',
                isPointer: false,
                isArray: false,
                arrayLength: null,
                structure: null
            },
            {
                name: 'result',
                type: 'int',
                value: 126,
                scope: 'local',
                line: 12,
                address: '0x7fff5fbff5b4',
                isPointer: false,
                isArray: false,
                arrayLength: null,
                structure: null
            },
            {
                name: 'buffer',
                type: 'char[]',
                value: 'Hello Debug',
                scope: 'local',
                line: 15,
                address: '0x7fff5fbff580',
                isPointer: true,
                isArray: true,
                arrayLength: 12,
                structure: null
            },
            {
                name: 'ptr',
                type: 'int*',
                value: '0x7fff5fbff5bc',
                scope: 'local',
                line: 16,
                address: '0x7fff5fbff578',
                isPointer: true,
                isArray: false,
                arrayLength: null,
                structure: null
            },
            {
                name: 'global_counter',
                type: 'int',
                value: 100,
                scope: 'global',
                line: 1,
                address: '0x601040',
                isPointer: false,
                isArray: false,
                arrayLength: null,
                structure: null
            },
            {
                name: 'user_data',
                type: 'struct UserData',
                value: null,
                scope: 'local',
                line: 20,
                address: '0x7fff5fbff540',
                isPointer: false,
                isArray: false,
                arrayLength: null,
                structure: {
                    name: 'TestUser',
                    age: 25,
                    scores: [85, 92, 78],
                    active: true
                }
            }
        ];
    }

    updateVariableData(newVariables) {
        // Capture current state for history
        this.variableHistory.push({
            timestamp: new Date().toISOString(),
            variables: new Map(this.variables)
        });
        
        // Limit history to last 10 states
        if (this.variableHistory.length > 10) {
            this.variableHistory.shift();
        }
        
        // Update variables
        this.variables.clear();
        newVariables.forEach(variable => {
            this.variables.set(variable.name, {
                ...variable,
                history: this.getVariableHistory(variable.name)
            });
            
            // Track variable types
            this.variableTypes.add(variable.type);
        });
    }

    getVariableHistory(varName) {
        const history = [];
        for (let i = this.variableHistory.length - 1; i >= 0; i--) {
            const state = this.variableHistory[i];
            const variable = state.variables.get(varName);
            if (variable) {
                history.push({
                    timestamp: state.timestamp,
                    value: variable.value
                });
            }
        }
        return history;
    }

    renderVariables() {
        const variablesTree = document.getElementById('variablesTree');
        if (!variablesTree) return;
        
        const filteredVariables = this.getFilteredVariables();
        const sortedVariables = this.getSortedVariables(filteredVariables);
        
        variablesTree.innerHTML = sortedVariables.map(variable => 
            this.renderVariableItem(variable)
        ).join('');
        
        // Add event listeners
        this.addVariableEventListeners();
    }

    getFilteredVariables() {
        let filtered = Array.from(this.variables.values());
        
        // Filter by scope
        if (this.currentScope !== 'all') {
            filtered = filtered.filter(v => v.scope === this.currentScope);
        }
        
        // Filter by type
        if (this.filterTypes.size > 0) {
            filtered = filtered.filter(v => this.filterTypes.has(v.type));
        }
        
        return filtered;
    }

    getSortedVariables(variables) {
        return variables.sort((a, b) => {
            let comparison = 0;
            
            switch (this.sortBy) {
                case 'name':
                    comparison = a.name.localeCompare(b.name);
                    break;
                case 'type':
                    comparison = a.type.localeCompare(b.type);
                    break;
                case 'value':
                    comparison = String(a.value).localeCompare(String(b.value));
                    break;
                case 'line':
                    comparison = a.line - b.line;
                    break;
            }
            
            return this.sortDirection === 'desc' ? -comparison : comparison;
        });
    }

    renderVariableItem(variable) {
        const isExpanded = this.expandedVariables.has(variable.name);
        const hasStructure = variable.structure && typeof variable.structure === 'object';
        const isPointer = variable.isPointer;
        const valueDisplay = this.formatVariableValue(variable);
        
        return `
            <div class="variable-item ${variable.scope}" data-variable="${variable.name}">
                <div class="variable-header">
                    <div class="variable-info">
                        ${hasStructure && isExpanded ? '▼' : hasStructure ? '▶' : '•'}
                        <span class="variable-name">${variable.name}</span>
                        <span class="variable-type">${variable.type}</span>
                        ${isPointer ? '<span class="pointer-indicator">*</span>' : ''}
                    </div>
                    <div class="variable-controls">
                        <button class="btn btn-sm btn-secondary" onclick="app.variableInspector.editValue('${variable.name}')">Edit</button>
                        <button class="btn btn-sm btn-info" onclick="app.variableInspector.watchVariable('${variable.name}')">Watch</button>
                    </div>
                </div>
                <div class="variable-value">
                    <span class="value-display">${valueDisplay}</span>
                    <span class="variable-address">${variable.address}</span>
                    <span class="variable-line">Line ${variable.line}</span>
                </div>
                ${hasStructure ? this.renderStructureTree(variable.structure, isExpanded) : ''}
                ${this.renderVariableHistory(variable.history)}
            </div>
        `;
    }

    renderStructureTree(structure, isExpanded) {
        if (!isExpanded) return '';
        
        const entries = Object.entries(structure);
        return `
            <div class="structure-tree">
                ${entries.map(([key, value]) => `
                    <div class="structure-item">
                        <span class="structure-key">${key}:</span>
                        <span class="structure-value">${this.formatValue(value)}</span>
                    </div>
                `).join('')}
            </div>
        `;
    }

    renderVariableHistory(history) {
        if (history.length <= 1) return '';
        
        return `
            <div class="variable-history">
                <h5>Value History</h5>
                <div class="history-timeline">
                    ${history.slice(-3).map(entry => `
                        <div class="history-entry">
                            <span class="history-time">${new Date(entry.timestamp).toLocaleTimeString()}</span>
                            <span class="history-value">${this.formatValue(entry.value)}</span>
                        </div>
                    `).join('')}
                </div>
            </div>
        `;
    }

    formatVariableValue(variable) {
        let value = variable.value;
        
        if (variable.isArray && variable.arrayLength) {
            if (typeof value === 'string') {
                value = `"${value}"`;
            } else if (Array.isArray(value)) {
                value = `[${value.join(', ')}]`;
            }
        }
        
        if (variable.isPointer && variable.pointerValue !== undefined) {
            value = `*(${variable.pointerValue})`;
        }
        
        return this.formatValue(value);
    }

    formatValue(value) {
        if (value === null || value === undefined) {
            return 'null';
        }
        
        if (typeof value === 'string') {
            return `"${value}"`;
        }
        
        if (typeof value === 'boolean') {
            return value ? 'true' : 'false';
        }
        
        if (typeof value === 'object') {
            return JSON.stringify(value, null, 2);
        }
        
        return String(value);
    }

    addVariableEventListeners() {
        // Add click listeners for expansion
        document.querySelectorAll('.variable-item').forEach(item => {
            item.addEventListener('click', (e) => {
                if (e.target.classList.contains('variable-info') || 
                    e.target.classList.contains('variable-name')) {
                    const varName = item.dataset.variable;
                    this.toggleVariableExpansion(varName);
                }
            });
        });
    }

    toggleVariableExpansion(varName) {
        if (this.expandedVariables.has(varName)) {
            this.expandedVariables.delete(varName);
        } else {
            this.expandedVariables.add(varName);
        }
        this.renderVariables();
    }

    renderWatchExpressions() {
        const watchList = document.getElementById('watchList');
        if (!watchList) return;
        
        const expressions = Array.from(this.watchExpressions);
        
        if (expressions.length === 0) {
            watchList.innerHTML = '<div class="no-watches">No watch expressions</div>';
            return;
        }
        
        watchList.innerHTML = expressions.map(expression => `
            <div class="watch-item" data-expression="${expression}">
                <div class="watch-expression">${expression}</div>
                <div class="watch-value">${this.evaluateWatchExpression(expression)}</div>
                <div class="watch-controls">
                    <button class="btn btn-sm btn-danger" onclick="app.variableInspector.removeWatchExpression('${expression}')">Remove</button>
                </div>
            </div>
        `).join('');
    }

    evaluateWatchExpression(expression) {
        try {
            // Mock expression evaluation
            const mockEvaluations = {
                'x': this.variables.get('x')?.value || 'undefined',
                'y': this.variables.get('y')?.value || 'undefined',
                'x + y': (this.variables.get('x')?.value || 0) + (this.variables.get('y')?.value || 0),
                'result': this.variables.get('result')?.value || 'undefined',
                '*ptr': '42',
                'strlen(buffer)': '12',
                'global_counter': this.variables.get('global_counter')?.value || 'undefined'
            };
            
            return mockEvaluations[expression] || 'Cannot evaluate';
        } catch (error) {
            return `Error: ${error.message}`;
        }
    }

    addWatchExpression(expression) {
        if (!expression.trim()) {
            if (this.app) {
                this.app.showNotification('Please enter a valid expression', 'warning');
            }
            return;
        }
        
        this.watchExpressions.add(expression.trim());
        this.renderWatchExpressions();
        
        if (this.app) {
            this.app.showNotification(`Watch expression added: ${expression}`, 'success');
        }
    }

    removeWatchExpression(expression) {
        this.watchExpressions.delete(expression);
        this.renderWatchExpressions();
    }

    watchVariable(varName) {
        this.addWatchExpression(varName);
    }

    async editValue(varName) {
        const variable = this.variables.get(varName);
        if (!variable) return;
        
        const newValue = prompt(`Enter new value for ${varName} (${variable.type}):`, variable.value);
        if (newValue === null) return; // User cancelled
        
        try {
            await this.updateVariableValue(varName, newValue);
            if (this.app) {
                this.app.showNotification(`Value updated for ${varName}`, 'success');
            }
        } catch (error) {
            if (this.app) {
                this.app.showNotification(`Failed to update ${varName}: ${error.message}`, 'error');
            }
        }
    }

    async updateVariableValue(varName, newValue) {
        try {
            const response = await fetch('/api/debug/variables/edit', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ name: varName, value: newValue })
            });
            
            if (!response.ok) {
                throw new Error(`Failed to update variable: ${response.statusText}`);
            }
            
            // Update local variable
            const variable = this.variables.get(varName);
            if (variable) {
                variable.value = this.parseValue(newValue, variable.type);
                this.variables.set(varName, variable);
                this.renderVariables();
            }
            
        } catch (error) {
            // Mock update for demonstration
            const variable = this.variables.get(varName);
            if (variable) {
                variable.value = this.parseValue(newValue, variable.type);
                this.variables.set(varName, variable);
                this.renderVariables();
            }
        }
    }

    parseValue(value, type) {
        switch (type) {
            case 'int':
                return parseInt(value, 10);
            case 'float':
            case 'double':
                return parseFloat(value);
            case 'bool':
            case 'boolean':
                return value === 'true' || value === '1';
            case 'char':
            case 'char[]':
            case 'string':
                return value.replace(/^"|"$/g, '');
            default:
                return value;
        }
    }

    setScope(scope) {
        this.currentScope = scope;
        this.renderVariables();
    }

    setFilterTypes(types) {
        this.filterTypes = new Set(types);
        this.renderVariables();
    }

    setSorting(sortBy, direction) {
        this.sortBy = sortBy;
        this.sortDirection = direction;
        this.renderVariables();
    }

    getVariableSummary() {
        const variables = Array.from(this.variables.values());
        return {
            total: variables.length,
            byScope: {
                local: variables.filter(v => v.scope === 'local').length,
                global: variables.filter(v => v.scope === 'global').length,
                static: variables.filter(v => v.scope === 'static').length
            },
            byType: this.getTypeDistribution(variables),
            pointers: variables.filter(v => v.isPointer).length,
            arrays: variables.filter(v => v.isArray).length,
            watchExpressions: this.watchExpressions.size
        };
    }

    getTypeDistribution(variables) {
        const distribution = {};
        variables.forEach(variable => {
            distribution[variable.type] = (distribution[variable.type] || 0) + 1;
        });
        return distribution;
    }

    findVariablesByPattern(pattern) {
        const regex = new RegExp(pattern, 'i');
        return Array.from(this.variables.values()).filter(variable => 
            regex.test(variable.name) || regex.test(variable.type) || 
            String(variable.value).includes(pattern)
        );
    }

    compareVariables(varName1, varName2) {
        const var1 = this.variables.get(varName1);
        const var2 = this.variables.get(varName2);
        
        if (!var1 || !var2) {
            return null;
        }
        
        return {
            name1: varName1,
            name2: varName2,
            typesEqual: var1.type === var2.type,
            valuesEqual: JSON.stringify(var1.value) === JSON.stringify(var2.value),
            scopeEqual: var1.scope === var2.scope,
            addressesEqual: var1.address === var2.address
        };
    }

    exportVariableState() {
        return {
            timestamp: new Date().toISOString(),
            variables: Array.from(this.variables.values()),
            watchExpressions: Array.from(this.watchExpressions),
            scope: this.currentScope,
            filterTypes: Array.from(this.filterTypes),
            sortBy: this.sortBy,
            sortDirection: this.sortDirection
        };
    }

    importVariableState(state) {
        try {
            this.variables.clear();
            state.variables.forEach(variable => {
                this.variables.set(variable.name, variable);
            });
            
            this.watchExpressions = new Set(state.watchExpressions);
            this.currentScope = state.scope;
            this.filterTypes = new Set(state.filterTypes);
            this.sortBy = state.sortBy;
            this.sortDirection = state.sortDirection;
            
            this.renderVariables();
            this.renderWatchExpressions();
            
            if (this.app) {
                this.app.showNotification('Variable state imported', 'success');
            }
        } catch (error) {
            if (this.app) {
                this.app.showNotification(`Failed to import variable state: ${error.message}`, 'error');
            }
        }
    }

    analyzeVariableAccess() {
        const analysis = {
            unused: [],
            modified: [],
            pointToSame: [],
            typeMismatches: []
        };
        
        // Simple analysis for demonstration
        this.variables.forEach((variable, name) => {
            // Check if variable was never read (simplified check)
            if (variable.history.length <= 1) {
                analysis.unused.push(name);
            }
            
            // Check for modifications
            if (variable.history.length > 2) {
                analysis.modified.push(name);
            }
        });
        
        return analysis;
    }

    generateMemoryReport() {
        const variables = Array.from(this.variables.values());
        let totalSize = 0;
        const sizeByType = {};
        const sizeByScope = {};
        
        variables.forEach(variable => {
            let size = this.estimateSize(variable);
            totalSize += size;
            
            sizeByType[variable.type] = (sizeByType[variable.type] || 0) + size;
            sizeByScope[variable.scope] = (sizeByScope[variable.scope] || 0) + size;
        });
        
        return {
            totalSize,
            sizeByType,
            sizeByScope,
            variableCount: variables.length,
            averageSize: totalSize / variables.length
        };
    }

    estimateSize(variable) {
        const typeSizes = {
            'int': 4,
            'float': 4,
            'double': 8,
            'char': 1,
            'bool': 1,
            'char*': 8,
            'void*': 8,
            'int*': 8,
            'char[]': variable.arrayLength || 64
        };
        
        const baseSize = typeSizes[variable.type] || 8; // Default to 8 bytes for pointers
        
        if (variable.isArray) {
            return variable.arrayLength || baseSize;
        }
        
        return baseSize;
    }

    destroy() {
        this.variables.clear();
        this.watchExpressions.clear();
        this.variableHistory = [];
        this.expandedVariables.clear();
        this.filterTypes.clear();
        this.variableTypes.clear();
    }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = VariableInspector;
}