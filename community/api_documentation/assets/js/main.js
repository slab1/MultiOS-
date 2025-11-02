// MultiOS API Documentation - Main JavaScript
// Core functionality for navigation, search, and user interactions

class DocumentationApp {
    constructor() {
        this.currentSection = 'overview';
        this.searchIndex = null;
        this.fuse = null;
        this.theme = localStorage.getItem('theme') || 'light';
        
        this.init();
    }

    init() {
        this.setupTheme();
        this.setupNavigation();
        this.setupSearch();
        this.setupInteractiveElements();
        this.loadSearchIndex();
        
        // Handle initial section based on URL hash
        this.handleInitialSection();
        
        console.log('MultiOS API Documentation initialized');
    }

    // Theme Management
    setupTheme() {
        const themeToggle = document.getElementById('theme-toggle');
        if (themeToggle) {
            themeToggle.addEventListener('click', () => this.toggleTheme());
        }
        
        // Set initial theme
        document.documentElement.setAttribute('data-theme', this.theme);
        this.updateThemeIcon();
    }

    toggleTheme() {
        this.theme = this.theme === 'light' ? 'dark' : 'light';
        document.documentElement.setAttribute('data-theme', this.theme);
        localStorage.setItem('theme', this.theme);
        this.updateThemeIcon();
        
        // Trigger custom event for other components
        window.dispatchEvent(new CustomEvent('themeChanged', {
            detail: { theme: this.theme }
        }));
    }

    updateThemeIcon() {
        const themeToggle = document.getElementById('theme-toggle');
        if (themeToggle) {
            const icon = themeToggle.querySelector('i');
            if (icon) {
                icon.className = this.theme === 'light' ? 'fas fa-moon' : 'fas fa-sun';
            }
        }
    }

    // Navigation Management
    setupNavigation() {
        const navLinks = document.querySelectorAll('.nav-link');
        const contentSections = document.querySelectorAll('.content-section');

        // Handle navigation clicks
        navLinks.forEach(link => {
            link.addEventListener('click', (e) => {
                e.preventDefault();
                
                // Get target section
                const href = link.getAttribute('href');
                const sectionId = href?.startsWith('#') ? href.slice(1) : null;
                
                if (sectionId) {
                    this.showSection(sectionId);
                    this.setActiveNavLink(link);
                } else if (href && !href.startsWith('#')) {
                    // External navigation
                    window.location.href = href;
                }
            });
        });

        // Handle browser back/forward
        window.addEventListener('popstate', (e) => {
            if (e.state && e.state.section) {
                this.showSection(e.state.section);
            }
        });
    }

    showSection(sectionId) {
        const contentSections = document.querySelectorAll('.content-section');
        const navLinks = document.querySelectorAll('.nav-link');

        // Hide all sections
        contentSections.forEach(section => {
            section.classList.remove('active');
        });

        // Show target section
        const targetSection = document.getElementById(sectionId);
        if (targetSection) {
            targetSection.classList.add('active');
            this.currentSection = sectionId;
            
            // Update URL without page reload
            window.history.pushState(
                { section: sectionId },
                '',
                `#${sectionId}`
            );
            
            // Initialize section-specific content
            this.initializeSection(sectionId);
        }

        // Update navigation
        this.setActiveNavLink(document.querySelector(`[href="#${sectionId}"]`));
    }

    setActiveNavLink(activeLink) {
        const navLinks = document.querySelectorAll('.nav-link');
        navLinks.forEach(link => link.classList.remove('active'));
        
        if (activeLink) {
            activeLink.classList.add('active');
        }
    }

    handleInitialSection() {
        const hash = window.location.hash.slice(1);
        if (hash) {
            this.showSection(hash);
        }
    }

    initializeSection(sectionId) {
        switch (sectionId) {
            case 'overview':
                this.initializeOverviewSection();
                break;
            case 'quick-start':
                this.initializeQuickStartSection();
                break;
            default:
                // Lazy loading for other sections
                this.lazyLoadSection(sectionId);
        }
    }

    initializeOverviewSection() {
        // Animate hero stats
        this.animateStats();
        
        // Setup feature cards
        this.setupFeatureCards();
    }

    initializeQuickStartSection() {
        // Highlight active code blocks
        this.highlightCodeBlocks();
        
        // Setup copy buttons for code examples
        this.setupCopyButtons();
    }

    animateStats() {
        const stats = document.querySelectorAll('.stat-number');
        stats.forEach(stat => {
            const finalValue = parseInt(stat.textContent.replace(/\D/g, ''));
            const suffix = stat.textContent.replace(/\d/g, '');
            
            let currentValue = 0;
            const increment = finalValue / 30;
            const timer = setInterval(() => {
                currentValue += increment;
                if (currentValue >= finalValue) {
                    currentValue = finalValue;
                    clearInterval(timer);
                }
                stat.textContent = Math.floor(currentValue) + suffix;
            }, 50);
        });
    }

    setupFeatureCards() {
        const cards = document.querySelectorAll('.feature-card');
        cards.forEach(card => {
            card.addEventListener('click', () => {
                const link = card.querySelector('a');
                if (link) {
                    window.location.href = link.getAttribute('href');
                }
            });
        });
    }

    // Search Functionality
    setupSearch() {
        const searchInput = document.getElementById('global-search');
        const searchResults = document.getElementById('search-results');

        if (!searchInput || !searchResults) return;

        let searchTimeout;

        searchInput.addEventListener('input', (e) => {
            clearTimeout(searchTimeout);
            const query = e.target.value.trim();
            
            if (query.length < 2) {
                this.hideSearchResults();
                return;
            }

            searchTimeout = setTimeout(() => {
                this.performSearch(query);
            }, 300);
        });

        searchInput.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
                e.preventDefault();
                this.performSearch(e.target.value.trim());
            } else if (e.key === 'Escape') {
                this.hideSearchResults();
                searchInput.blur();
            }
        });

        // Hide search results when clicking outside
        document.addEventListener('click', (e) => {
            if (!searchInput.contains(e.target) && !searchResults.contains(e.target)) {
                this.hideSearchResults();
            }
        });
    }

    async loadSearchIndex() {
        try {
            const response = await fetch('search_indices/search-index.json');
            const indexData = await response.json();
            
            // Initialize Fuse.js for fuzzy search
            this.fuse = new Fuse(indexData, {
                keys: ['title', 'description', 'tags', 'category'],
                threshold: 0.3,
                includeScore: true,
                includeMatches: true
            });
        } catch (error) {
            console.warn('Failed to load search index:', error);
        }
    }

    performSearch(query) {
        if (!this.fuse || !query.trim()) {
            this.hideSearchResults();
            return;
        }

        const results = this.fuse.search(query);
        this.displaySearchResults(results.slice(0, 10));
    }

    displaySearchResults(results) {
        const searchResults = document.getElementById('search-results');
        if (!searchResults) return;

        if (results.length === 0) {
            searchResults.innerHTML = '<div class="search-result-item"><div class="search-result-title">No results found</div></div>';
        } else {
            searchResults.innerHTML = results.map(result => {
                const item = result.item;
                return `
                    <div class="search-result-item" onclick="app.selectSearchResult('${item.url}', '${item.title}')">
                        <div class="search-result-title">${item.title}</div>
                        <div class="search-result-description">${item.description}</div>
                        <div class="search-result-type">${item.category}</div>
                    </div>
                `;
            }).join('');
        }

        searchResults.classList.add('show');
    }

    selectSearchResult(url, title) {
        this.hideSearchResults();
        
        if (url.startsWith('#')) {
            // Internal navigation
            const sectionId = url.slice(1);
            this.showSection(sectionId);
        } else {
            // External navigation
            window.location.href = url;
        }
    }

    hideSearchResults() {
        const searchResults = document.getElementById('search-results');
        if (searchResults) {
            searchResults.classList.remove('show');
        }
    }

    // Interactive Elements
    setupInteractiveElements() {
        // Setup expandable sections
        this.setupExpandableSections();
        
        // Setup tabs
        this.setupTabs();
        
        // Setup modals
        this.setupModals();
        
        // Setup tooltips
        this.setupTooltips();
    }

    setupExpandableSections() {
        const expandableHeaders = document.querySelectorAll('[data-expandable]');
        
        expandableHeaders.forEach(header => {
            header.addEventListener('click', () => {
                const container = header.parentElement;
                container.classList.toggle('expanded');
                
                // Update ARIA attributes
                const isExpanded = container.classList.contains('expanded');
                header.setAttribute('aria-expanded', isExpanded);
            });
        });
    }

    setupTabs() {
        const tabLists = document.querySelectorAll('[role="tablist"]');
        
        tabLists.forEach(tabList => {
            const tabs = tabList.querySelectorAll('[role="tab"]');
            const tabPanels = tabList.parentElement.querySelectorAll('[role="tabpanel"]');
            
            tabs.forEach((tab, index) => {
                tab.addEventListener('click', () => {
                    this.activateTab(tab, tabs, tabPanels);
                });
                
                // Keyboard navigation
                tab.addEventListener('keydown', (e) => {
                    let newIndex;
                    
                    switch (e.key) {
                        case 'ArrowLeft':
                            newIndex = index > 0 ? index - 1 : tabs.length - 1;
                            break;
                        case 'ArrowRight':
                            newIndex = index < tabs.length - 1 ? index + 1 : 0;
                            break;
                        case 'Home':
                            newIndex = 0;
                            break;
                        case 'End':
                            newIndex = tabs.length - 1;
                            break;
                        default:
                            return;
                    }
                    
                    e.preventDefault();
                    this.activateTab(tabs[newIndex], tabs, tabPanels);
                    tabs[newIndex].focus();
                });
            });
        });
    }

    activateTab(activeTab, allTabs, tabPanels) {
        // Deactivate all tabs
        allTabs.forEach(tab => {
            tab.classList.remove('active');
            tab.setAttribute('aria-selected', 'false');
        });
        
        // Activate selected tab
        activeTab.classList.add('active');
        activeTab.setAttribute('aria-selected', 'true');
        
        // Hide all panels
        tabPanels.forEach(panel => {
            panel.classList.remove('active');
        });
        
        // Show corresponding panel
        const panelId = activeTab.getAttribute('aria-controls');
        const panel = document.getElementById(panelId);
        if (panel) {
            panel.classList.add('active');
        }
    }

    setupModals() {
        const modalTriggers = document.querySelectorAll('[data-modal]');
        
        modalTriggers.forEach(trigger => {
            trigger.addEventListener('click', (e) => {
                e.preventDefault();
                const modalId = trigger.getAttribute('data-modal');
                this.openModal(modalId);
            });
        });
        
        // Close modals when clicking overlay or close button
        document.addEventListener('click', (e) => {
            if (e.target.classList.contains('modal')) {
                this.closeModal(e.target.id);
            }
        });
        
        // Close modals with Escape key
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                const openModal = document.querySelector('.modal.show');
                if (openModal) {
                    this.closeModal(openModal.id);
                }
            }
        });
    }

    openModal(modalId) {
        const modal = document.getElementById(modalId);
        if (modal) {
            modal.classList.add('show');
            document.body.style.overflow = 'hidden';
            
            // Focus first focusable element
            const firstFocusable = modal.querySelector('button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])');
            if (firstFocusable) {
                firstFocusable.focus();
            }
        }
    }

    closeModal(modalId) {
        const modal = document.getElementById(modalId);
        if (modal) {
            modal.classList.remove('show');
            document.body.style.overflow = '';
        }
    }

    setupTooltips() {
        const tooltips = document.querySelectorAll('[data-tooltip]');
        
        tooltips.forEach(element => {
            element.addEventListener('mouseenter', (e) => {
                this.showTooltip(e.target, e.target.getAttribute('data-tooltip'));
            });
            
            element.addEventListener('mouseleave', () => {
                this.hideTooltip();
            });
        });
    }

    showTooltip(element, text) {
        const tooltip = document.createElement('div');
        tooltip.className = 'tooltip-popup';
        tooltip.textContent = text;
        tooltip.style.cssText = `
            position: absolute;
            background: var(--text-primary);
            color: var(--bg-primary);
            padding: var(--spacing-xs) var(--spacing-sm);
            border-radius: 4px;
            font-size: 0.75rem;
            z-index: 1000;
            pointer-events: none;
            white-space: nowrap;
        `;
        
        document.body.appendChild(tooltip);
        
        // Position tooltip
        const rect = element.getBoundingClientRect();
        tooltip.style.left = rect.left + (rect.width / 2) - (tooltip.offsetWidth / 2) + 'px';
        tooltip.style.top = rect.top - tooltip.offsetHeight - 5 + 'px';
        
        this.currentTooltip = tooltip;
    }

    hideTooltip() {
        if (this.currentTooltip) {
            document.body.removeChild(this.currentTooltip);
            this.currentTooltip = null;
        }
    }

    // Utility Functions
    highlightCodeBlocks() {
        if (window.Prism) {
            Prism.highlightAll();
        }
    }

    setupCopyButtons() {
        const codeBlocks = document.querySelectorAll('.code-block pre');
        
        codeBlocks.forEach(block => {
            const container = block.parentElement;
            
            // Add copy button if not already present
            if (!container.querySelector('.copy-button')) {
                const button = document.createElement('button');
                button.className = 'btn btn-sm copy-button';
                button.innerHTML = '<i class="fas fa-copy"></i> Copy';
                button.style.cssText = `
                    position: absolute;
                    top: var(--spacing-md);
                    right: var(--spacing-md);
                    z-index: 10;
                `;
                
                container.style.position = 'relative';
                container.appendChild(button);
                
                button.addEventListener('click', () => {
                    this.copyToClipboard(block.textContent, button);
                });
            }
        });
    }

    async copyToClipboard(text, button) {
        try {
            await navigator.clipboard.writeText(text);
            
            const originalText = button.innerHTML;
            button.innerHTML = '<i class="fas fa-check"></i> Copied';
            button.style.background = 'var(--success)';
            
            setTimeout(() => {
                button.innerHTML = originalText;
                button.style.background = '';
            }, 2000);
        } catch (error) {
            console.error('Failed to copy to clipboard:', error);
        }
    }

    lazyLoadSection(sectionId) {
        // Placeholder for lazy loading section content
        console.log(`Lazy loading section: ${sectionId}`);
    }

    // Public API
    navigateToSection(sectionId) {
        this.showSection(sectionId);
    }

    getCurrentSection() {
        return this.currentSection;
    }

    getTheme() {
        return this.theme;
    }
}

// Initialize the application when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.app = new DocumentationApp();
});

// Utility functions for external use
window.DocumentationUtils = {
    showNotification: (message, type = 'info') => {
        const notification = document.createElement('div');
        notification.className = `notification notification-${type}`;
        notification.style.cssText = `
            position: fixed;
            top: var(--spacing-lg);
            right: var(--spacing-lg);
            background: var(--bg-primary);
            border: 1px solid var(--border-color);
            border-radius: 8px;
            padding: var(--spacing-lg);
            box-shadow: var(--shadow-lg);
            z-index: 3000;
            max-width: 400px;
        `;
        notification.innerHTML = `
            <div style="display: flex; align-items: center; gap: var(--spacing-sm);">
                <i class="fas fa-${type === 'success' ? 'check-circle' : type === 'error' ? 'exclamation-circle' : 'info-circle'}"></i>
                <span>${message}</span>
            </div>
        `;
        
        document.body.appendChild(notification);
        
        setTimeout(() => {
            notification.style.opacity = '0';
            setTimeout(() => {
                document.body.removeChild(notification);
            }, 300);
        }, 3000);
    },

    formatCode: (code, language = 'rust') => {
        // Simple code formatting utility
        if (window.Prism) {
            const grammar = Prism.languages[language] || Prism.languages.rust;
            return Prism.highlight(code, grammar, language);
        }
        return code;
    },

    debounce: (func, wait) => {
        let timeout;
        return function executedFunction(...args) {
            const later = () => {
                clearTimeout(timeout);
                func(...args);
            };
            clearTimeout(timeout);
            timeout = setTimeout(later, wait);
        };
    }
};