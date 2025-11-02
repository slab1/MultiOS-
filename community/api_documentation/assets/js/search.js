// MultiOS API Documentation - Search Functionality
// Advanced search with fuzzy matching, filters, and real-time suggestions

class AdvancedSearch {
    constructor() {
        this.fuse = null;
        this.searchIndex = [];
        this.filters = {
            category: new Set(),
            language: new Set(),
            type: new Set(),
            difficulty: new Set()
        };
        this.currentResults = [];
        this.searchHistory = this.loadSearchHistory();
        
        this.init();
    }

    init() {
        this.setupSearchInterface();
        this.loadSearchIndex();
        this.setupSearchHistory();
        this.setupFilters();
    }

    async loadSearchIndex() {
        try {
            const response = await fetch('search_indices/complete-index.json');
            this.searchIndex = await response.json();
            
            // Initialize Fuse.js for fuzzy search
            this.fuse = new Fuse(this.searchIndex, {
                keys: [
                    { name: 'title', weight: 0.4 },
                    { name: 'description', weight: 0.3 },
                    { name: 'content', weight: 0.2 },
                    { name: 'tags', weight: 0.1 }
                ],
                threshold: 0.3,
                includeScore: true,
                includeMatches: true,
                minMatchCharLength: 2,
                ignoreLocation: true
            });
            
            console.log(`Loaded search index with ${this.searchIndex.length} items`);
        } catch (error) {
            console.error('Failed to load search index:', error);
            this.createFallbackIndex();
        }
    }

    createFallbackIndex() {
        // Create a basic fallback index if the main index fails to load
        this.searchIndex = [
            {
                id: 'kernel-api',
                title: 'Kernel API',
                description: 'Core kernel functionality and system calls',
                content: 'kernel initialization memory management process creation system calls',
                category: 'API Reference',
                language: 'rust',
                type: 'api',
                difficulty: 'intermediate',
                url: 'api_reference/kernel.html',
                tags: ['kernel', 'system', 'core']
            },
            {
                id: 'memory-management',
                title: 'Memory Management',
                description: 'Virtual and physical memory management APIs',
                content: 'memory allocation page tables virtual memory physical memory',
                category: 'API Reference',
                language: 'rust',
                type: 'api',
                difficulty: 'advanced',
                url: 'api_reference/memory.html',
                tags: ['memory', 'allocation', 'pages']
            },
            {
                id: 'quick-start',
                title: 'Quick Start Tutorial',
                description: 'Get started with MultiOS development',
                content: 'tutorial getting started installation first program',
                category: 'Tutorials',
                language: 'rust',
                type: 'tutorial',
                difficulty: 'beginner',
                url: 'tutorials/beginner/quick-start.html',
                tags: ['tutorial', 'beginner', 'setup']
            }
        ];

        this.fuse = new Fuse(this.searchIndex, {
            keys: ['title', 'description', 'content', 'tags'],
            threshold: 0.3,
            includeScore: true
        });
    }

    setupSearchInterface() {
        const searchPage = document.getElementById('advanced-search-page');
        if (!searchPage) return;

        const searchInput = document.getElementById('search-input');
        const searchResults = document.getElementById('search-results');
        const searchSuggestions = document.getElementById('search-suggestions');

        if (searchInput) {
            let searchTimeout;
            
            searchInput.addEventListener('input', (e) => {
                clearTimeout(searchTimeout);
                const query = e.target.value.trim();
                
                if (query.length >= 2) {
                    searchTimeout = setTimeout(() => {
                        this.performSearch(query, searchResults);
                        this.updateSuggestions(query, searchSuggestions);
                    }, 300);
                } else {
                    this.clearResults(searchResults);
                    this.hideSuggestions(searchSuggestions);
                }
            });

            searchInput.addEventListener('keydown', (e) => {
                if (e.key === 'Enter') {
                    e.preventDefault();
                    const query = e.target.value.trim();
                    if (query) {
                        this.performSearch(query, searchResults);
                        this.hideSuggestions(searchSuggestions);
                        this.addToSearchHistory(query);
                    }
                } else if (e.key === 'Escape') {
                    this.hideSuggestions(searchSuggestions);
                    searchInput.blur();
                }
            });
        }

        // Clear results when clicking outside
        document.addEventListener('click', (e) => {
            if (!searchInput?.contains(e.target) && !searchSuggestions?.contains(e.target)) {
                this.hideSuggestions(searchSuggestions);
            }
        });
    }

    performSearch(query, resultsContainer) {
        if (!this.fuse || !query.trim()) {
            this.displayNoResults(resultsContainer);
            return;
        }

        const startTime = performance.now();
        const results = this.fuse.search(query);
        const endTime = performance.now();

        // Apply filters
        const filteredResults = this.applyFilters(results);
        
        // Sort by relevance and recency
        const sortedResults = this.sortResults(filteredResults);

        this.currentResults = sortedResults;
        this.displayResults(sortedResults, resultsContainer, endTime - startTime);
        this.updateSearchStats(sortedResults.length, endTime - startTime);
    }

    applyFilters(results) {
        let filteredResults = [...results];

        // Apply category filter
        if (this.filters.category.size > 0) {
            filteredResults = filteredResults.filter(result => 
                this.filters.category.has(result.item.category)
            );
        }

        // Apply language filter
        if (this.filters.language.size > 0) {
            filteredResults = filteredResults.filter(result => 
                this.filters.language.has(result.item.language)
            );
        }

        // Apply type filter
        if (this.filters.type.size > 0) {
            filteredResults = filteredResults.filter(result => 
                this.filters.type.has(result.item.type)
            );
        }

        // Apply difficulty filter
        if (this.filters.difficulty.size > 0) {
            filteredResults = filteredResults.filter(result => 
                this.filters.difficulty.has(result.item.difficulty)
            );
        }

        return filteredResults;
    }

    sortResults(results) {
        return results.sort((a, b) => {
            // Primary sort: by score (relevance)
            if (a.score !== b.score) {
                return a.score - b.score;
            }

            // Secondary sort: by match count
            const aMatches = a.matches?.length || 0;
            const bMatches = b.matches?.length || 0;
            if (aMatches !== bMatches) {
                return bMatches - aMatches;
            }

            // Tertiary sort: by recency (if available)
            const aRecency = a.item.lastUpdated || 0;
            const bRecency = b.item.lastUpdated || 0;
            return bRecency - aRecency;
        });
    }

    displayResults(results, container, searchTime) {
        if (!container) return;

        if (results.length === 0) {
            this.displayNoResults(container);
            return;
        }

        const resultsHtml = results.map((result, index) => {
            const item = result.item;
            const highlightedContent = this.highlightMatches(item, result.matches);
            
            return `
                <div class="search-result-item" onclick="searchManager.selectResult('${item.url}', '${item.id}')">
                    <div class="search-result-header">
                        <h3 class="search-result-title">
                            <a href="${item.url}" onclick="event.stopPropagation()">${item.title}</a>
                        </h3>
                        <div class="search-result-badges">
                            <span class="badge badge-primary">${item.category}</span>
                            <span class="badge badge-secondary">${item.language}</span>
                            ${item.difficulty ? `<span class="badge badge-secondary">${item.difficulty}</span>` : ''}
                        </div>
                    </div>
                    <p class="search-result-description">${highlightedContent}</p>
                    <div class="search-result-meta">
                        <span class="search-result-url">${item.url}</span>
                        ${item.tags ? `<div class="search-result-tags">${item.tags.map(tag => `<span class="tag">${tag}</span>`).join('')}</div>` : ''}
                    </div>
                    ${result.score ? `<div class="search-result-score">Relevance: ${(100 - (result.score * 100)).toFixed(0)}%</div>` : ''}
                </div>
            `;
        }).join('');

        container.innerHTML = `
            <div class="search-results-header">
                <div class="search-results-count">
                    Found ${results.length} result${results.length !== 1 ? 's' : ''} in ${searchTime.toFixed(2)}ms
                </div>
            </div>
            <div class="search-results-list">
                ${resultsHtml}
            </div>
        `;
    }

    displayNoResults(container) {
        if (!container) return;

        container.innerHTML = `
            <div class="search-no-results">
                <div class="no-results-icon">
                    <i class="fas fa-search"></i>
                </div>
                <h3>No results found</h3>
                <p>Try adjusting your search terms or filters</p>
                <div class="search-suggestions">
                    <h4>Suggestions:</h4>
                    <ul>
                        <li>Check your spelling</li>
                        <li>Use more general terms</li>
                        <li>Try different keywords</li>
                        <li>Remove some filters</li>
                    </ul>
                </div>
            </div>
        `;
    }

    highlightMatches(item, matches) {
        if (!matches || matches.length === 0) {
            return item.description || item.content;
        }

        let content = item.description || item.content;
        
        // Create a comprehensive text for highlighting
        const fullText = `${item.title} ${item.description || ''} ${item.content || ''}`;
        
        matches.forEach(match => {
            if (match.key === 'content' || match.key === 'description') {
                const regex = new RegExp(`(${match.value})`, 'gi');
                content = content.replace(regex, '<mark>$1</mark>');
            }
        });

        return content;
    }

    updateSuggestions(query, suggestionsContainer) {
        if (!suggestionsContainer || query.length < 2) {
            this.hideSuggestions(suggestionsContainer);
            return;
        }

        // Generate suggestions based on the query
        const suggestions = this.generateSuggestions(query);
        
        if (suggestions.length === 0) {
            this.hideSuggestions(suggestionsContainer);
            return;
        }

        const suggestionsHtml = suggestions.map(suggestion => `
            <div class="search-suggestion-item" onclick="searchManager.selectSuggestion('${suggestion.text}', '${suggestion.type}')">
                <i class="fas fa-${suggestion.icon}"></i>
                <span>${suggestion.text}</span>
                <span class="suggestion-type">${suggestion.type}</span>
            </div>
        `).join('');

        suggestionsContainer.innerHTML = `
            <div class="search-suggestions-list">
                ${suggestionsHtml}
            </div>
        `;

        suggestionsContainer.classList.add('show');
    }

    generateSuggestions(query) {
        const suggestions = [];
        const queryLower = query.toLowerCase();

        // Add search history suggestions
        this.searchHistory
            .filter(term => term.toLowerCase().includes(queryLower))
            .slice(0, 3)
            .forEach(term => {
                suggestions.push({
                    text: term,
                    type: 'history',
                    icon: 'clock'
                });
            });

        // Add category suggestions
        const categories = [...new Set(this.searchIndex.map(item => item.category))];
        categories
            .filter(cat => cat.toLowerCase().includes(queryLower))
            .slice(0, 2)
            .forEach(category => {
                suggestions.push({
                    text: category,
                    type: 'category',
                    icon: 'folder'
                });
            });

        // Add tag suggestions
        const tags = [...new Set(this.searchIndex.flatMap(item => item.tags || []))];
        tags
            .filter(tag => tag.toLowerCase().includes(queryLower))
            .slice(0, 2)
            .forEach(tag => {
                suggestions.push({
                    text: tag,
                    type: 'tag',
                    icon: 'tag'
                });
            });

        return suggestions.slice(0, 6);
    }

    hideSuggestions(suggestionsContainer) {
        if (suggestionsContainer) {
            suggestionsContainer.classList.remove('show');
        }
    }

    selectSuggestion(text, type) {
        const searchInput = document.getElementById('search-input');
        if (searchInput) {
            searchInput.value = text;
            searchInput.focus();
        }
        this.hideSuggestions(document.getElementById('search-suggestions'));
    }

    selectResult(url, id) {
        // Navigate to the result
        window.location.href = url;
        
        // Track search analytics
        this.trackSearchSelection(id, url);
    }

    setupSearchHistory() {
        const searchInput = document.getElementById('search-input');
        if (searchInput) {
            searchInput.addEventListener('keydown', (e) => {
                if (e.key === 'Enter' && e.target.value.trim()) {
                    this.addToSearchHistory(e.target.value.trim());
                }
            });
        }
    }

    addToSearchHistory(query) {
        // Remove existing entry if present
        this.searchHistory = this.searchHistory.filter(term => term !== query);
        
        // Add to beginning of array
        this.searchHistory.unshift(query);
        
        // Limit history size
        if (this.searchHistory.length > 20) {
            this.searchHistory = this.searchHistory.slice(0, 20);
        }
        
        // Save to localStorage
        localStorage.setItem('multios_search_history', JSON.stringify(this.searchHistory));
    }

    loadSearchHistory() {
        try {
            const history = localStorage.getItem('multios_search_history');
            return history ? JSON.parse(history) : [];
        } catch {
            return [];
        }
    }

    setupFilters() {
        const filterElements = document.querySelectorAll('[data-filter]');
        
        filterElements.forEach(element => {
            element.addEventListener('change', () => {
                const filterType = element.getAttribute('data-filter');
                const filterValue = element.value || element.getAttribute('data-value');
                const isChecked = element.checked;
                
                this.toggleFilter(filterType, filterValue, isChecked);
                
                // Re-run search with new filters
                const searchInput = document.getElementById('search-input');
                if (searchInput?.value.trim()) {
                    this.performSearch(searchInput.value.trim(), document.getElementById('search-results'));
                }
            });
        });

        // Setup filter clear button
        const clearFiltersBtn = document.getElementById('clear-filters');
        if (clearFiltersBtn) {
            clearFiltersBtn.addEventListener('click', () => {
                this.clearAllFilters();
                
                // Re-run search
                const searchInput = document.getElementById('search-input');
                if (searchInput?.value.trim()) {
                    this.performSearch(searchInput.value.trim(), document.getElementById('search-results'));
                }
            });
        }
    }

    toggleFilter(filterType, filterValue, isChecked) {
        if (isChecked) {
            this.filters[filterType].add(filterValue);
        } else {
            this.filters[filterType].delete(filterValue);
        }
    }

    clearAllFilters() {
        Object.keys(this.filters).forEach(type => {
            this.filters[type].clear();
        });
        
        // Uncheck all filter checkboxes
        const filterElements = document.querySelectorAll('[data-filter]');
        filterElements.forEach(element => {
            element.checked = false;
        });
    }

    updateSearchStats(resultCount, searchTime) {
        const statsElement = document.getElementById('search-stats');
        if (statsElement) {
            statsElement.innerHTML = `
                <span>${resultCount} results</span>
                <span>•</span>
                <span>${searchTime.toFixed(2)}ms</span>
            `;
        }
    }

    trackSearchSelection(id, url) {
        // Simple analytics tracking
        console.log(`Search result selected: ${id} -> ${url}`);
        
        // In a real implementation, this would send data to an analytics service
    }

    // Advanced search features
    searchByCategory(category) {
        this.filters.category.clear();
        this.filters.category.add(category);
        
        const searchInput = document.getElementById('search-input');
        if (searchInput) {
            this.performSearch(searchInput.value.trim() || '*', document.getElementById('search-results'));
        }
    }

    searchByLanguage(language) {
        this.filters.language.clear();
        this.filters.language.add(language);
        
        const searchInput = document.getElementById('search-input');
        if (searchInput) {
            this.performSearch(searchInput.value.trim() || '*', document.getElementById('search-results'));
        }
    }

    exportResults() {
        if (this.currentResults.length === 0) {
            return;
        }

        const exportData = this.currentResults.map(result => ({
            title: result.item.title,
            description: result.item.description,
            category: result.item.category,
            url: result.item.url,
            tags: result.item.tags
        }));

        const blob = new Blob([JSON.stringify(exportData, null, 2)], {
            type: 'application/json'
        });

        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `multios-search-results-${Date.now()}.json`;
        a.click();
        
        URL.revokeObjectURL(url);
    }
}

// Initialize search manager when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.searchManager = new AdvancedSearch();
});