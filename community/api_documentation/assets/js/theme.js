// MultiOS API Documentation - Theme Management
// Handle light/dark theme switching with persistence and system preference detection

class ThemeManager {
    constructor() {
        this.currentTheme = localStorage.getItem('multios_theme') || 'light';
        this.systemPreference = window.matchMedia('(prefers-color-scheme: dark)');
        this.listeners = [];
        
        this.init();
    }

    init() {
        this.setupThemeToggle();
        this.detectSystemPreference();
        this.applyTheme(this.currentTheme);
        this.setupSystemPreferenceListener();
    }

    setupThemeToggle() {
        const toggleButton = document.getElementById('theme-toggle');
        if (toggleButton) {
            toggleButton.addEventListener('click', () => {
                this.toggleTheme();
            });
        }

        // Setup theme selector if present
        const themeSelector = document.getElementById('theme-selector');
        if (themeSelector) {
            themeSelector.addEventListener('change', (e) => {
                this.setTheme(e.target.value);
            });
        }
    }

    detectSystemPreference() {
        const prefersDark = this.systemPreference.matches;
        const hasLocalStorage = localStorage.getItem('multios_theme') !== null;
        
        // If no theme is stored, use system preference
        if (!hasLocalStorage) {
            this.currentTheme = prefersDark ? 'dark' : 'light';
        }
    }

    setupSystemPreferenceListener() {
        this.systemPreference.addListener((e) => {
            const hasLocalStorage = localStorage.getItem('multios_theme') !== null;
            
            // Only follow system preference if user hasn't set a manual preference
            if (!hasLocalStorage) {
                this.currentTheme = e.matches ? 'dark' : 'light';
                this.applyTheme(this.currentTheme);
                this.updateThemeUI();
            }
        });
    }

    toggleTheme() {
        const newTheme = this.currentTheme === 'light' ? 'dark' : 'light';
        this.setTheme(newTheme);
    }

    setTheme(theme) {
        if (theme !== 'light' && theme !== 'dark') {
            console.warn(`Invalid theme: ${theme}`);
            return;
        }

        this.currentTheme = theme;
        this.applyTheme(theme);
        localStorage.setItem('multios_theme', theme);
        this.updateThemeUI();
        this.notifyListeners(theme);
    }

    applyTheme(theme) {
        document.documentElement.setAttribute('data-theme', theme);
        
        // Update meta theme-color for mobile browsers
        const metaThemeColor = document.querySelector('meta[name="theme-color"]');
        if (metaThemeColor) {
            metaThemeColor.setAttribute('content', theme === 'dark' ? '#0f172a' : '#ffffff');
        }

        // Update favicon based on theme
        this.updateFavicon(theme);
    }

    updateThemeUI() {
        // Update theme toggle icon
        const toggleButton = document.getElementById('theme-toggle');
        if (toggleButton) {
            const icon = toggleButton.querySelector('i');
            if (icon) {
                icon.className = this.currentTheme === 'light' ? 'fas fa-moon' : 'fas fa-sun';
            }
        }

        // Update theme selector if present
        const themeSelector = document.getElementById('theme-selector');
        if (themeSelector) {
            themeSelector.value = this.currentTheme;
        }

        // Update theme indicator
        const themeIndicator = document.getElementById('theme-indicator');
        if (themeIndicator) {
            themeIndicator.textContent = this.currentTheme === 'light' ? 'Light Mode' : 'Dark Mode';
        }
    }

    updateFavicon(theme) {
        // Create dynamic favicon based on theme
        const favicon = document.querySelector('link[rel="icon"]') || 
                       document.querySelector('link[rel="shortcut icon"]');
        
        if (favicon) {
            // For a real implementation, you might want to serve different favicons
            // or generate them dynamically based on the theme
            favicon.href = theme === 'dark' ? '/favicon-dark.ico' : '/favicon-light.ico';
        }
    }

    getCurrentTheme() {
        return this.currentTheme;
    }

    // Theme event handling
    onThemeChange(callback) {
        if (typeof callback === 'function') {
            this.listeners.push(callback);
        }
    }

    offThemeChange(callback) {
        const index = this.listeners.indexOf(callback);
        if (index > -1) {
            this.listeners.splice(index, 1);
        }
    }

    notifyListeners(theme) {
        this.listeners.forEach(callback => {
            try {
                callback(theme);
            } catch (error) {
                console.error('Error in theme change callback:', error);
            }
        });

        // Dispatch custom event for external listeners
        window.dispatchEvent(new CustomEvent('themeChanged', {
            detail: { theme }
        }));
    }

    // Utility methods for theme-aware components
    getThemeColors() {
        const root = getComputedStyle(document.documentElement);
        
        return {
            primary: root.getPropertyValue('--accent-primary').trim(),
            secondary: root.getPropertyValue('--accent-secondary').trim(),
            background: root.getPropertyValue('--bg-primary').trim(),
            surface: root.getPropertyValue('--bg-secondary').trim(),
            text: root.getPropertyValue('--text-primary').trim(),
            textSecondary: root.getPropertyValue('--text-secondary').trim(),
            border: root.getPropertyValue('--border-color').trim(),
            success: root.getPropertyValue('--success').trim(),
            warning: root.getPropertyValue('--warning').trim(),
            error: root.getPropertyValue('--error').trim()
        };
    }

    isDarkMode() {
        return this.currentTheme === 'dark';
    }

    // Theme transition handling
    prepareThemeTransition() {
        // Add transition class for smooth theme switching
        document.body.classList.add('theme-transition');
        
        // Remove transition class after transition completes
        setTimeout(() => {
            document.body.classList.remove('theme-transition');
        }, 300);
    }

    // Accessibility considerations
    setupAccessibility() {
        // Add keyboard support for theme toggle
        const toggleButton = document.getElementById('theme-toggle');
        if (toggleButton) {
            toggleButton.addEventListener('keydown', (e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    this.toggleTheme();
                }
            });
        }

        // Announce theme changes to screen readers
        this.onThemeChange((theme) => {
            const announcement = `Theme changed to ${theme} mode`;
            this.announceToScreenReader(announcement);
        });
    }

    announceToScreenReader(message) {
        const announcement = document.createElement('div');
        announcement.setAttribute('aria-live', 'polite');
        announcement.setAttribute('aria-atomic', 'true');
        announcement.style.cssText = `
            position: absolute;
            left: -10000px;
            width: 1px;
            height: 1px;
            overflow: hidden;
        `;
        announcement.textContent = message;
        
        document.body.appendChild(announcement);
        
        setTimeout(() => {
            document.body.removeChild(announcement);
        }, 1000);
    }

    // Color scheme preferences
    setupColorSchemePreferences() {
        // Respect user's color scheme preferences in CSS
        this.updateColorSchemeMeta();
        
        // Listen for system color scheme changes
        window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
            if (localStorage.getItem('multios_theme') === null) {
                this.updateColorSchemeMeta();
            }
        });
    }

    updateColorSchemeMeta() {
        let colorScheme = 'light';
        
        const storedTheme = localStorage.getItem('multios_theme');
        if (storedTheme) {
            colorScheme = storedTheme;
        } else {
            // Use system preference
            colorScheme = this.systemPreference.matches ? 'dark' : 'light';
        }

        // Update CSS color-scheme property
        document.documentElement.style.colorScheme = colorScheme;
        
        // Update meta theme-color
        const metaThemeColor = document.querySelector('meta[name="theme-color"]');
        if (metaThemeColor) {
            const themeColor = this.getThemeColors().background;
            metaThemeColor.setAttribute('content', themeColor);
        }
    }
}

// Theme-aware utilities
window.ThemeUtils = {
    // Apply theme-aware styles to dynamic content
    applyThemeAwareStyles(element) {
        const colors = window.themeManager?.getThemeColors() || {};
        
        // Apply CSS custom properties based on current theme
        Object.entries(colors).forEach(([key, value]) => {
            element.style.setProperty(`--color-${key}`, value);
        });
    },

    // Create theme-aware CSS variables
    createThemeVariables() {
        const colors = window.themeManager?.getThemeColors() || {};
        const style = document.createElement('style');
        
        style.textContent = `
            :root {
                ${Object.entries(colors).map(([key, value]) => 
                    `--color-${key}: ${value};`
                ).join('\n')}
            }
        `;
        
        document.head.appendChild(style);
        return style;
    },

    // Smoothly transition colors between themes
    animateColorTransition(element, fromColor, toColor, duration = 300) {
        element.style.transition = `color ${duration}ms ease, background-color ${duration}ms ease`;
        
        requestAnimationFrame(() => {
            element.style.color = toColor;
        });
    },

    // Get theme-appropriate icon
    getThemeIcon(lightIcon, darkIcon) {
        return window.themeManager?.isDarkMode() ? darkIcon : lightIcon;
    },

    // Create theme-aware gradient
    createThemeGradient(direction = '135deg', lightColors = [], darkColors = []) {
        const colors = window.themeManager?.isDarkMode() ? darkColors : lightColors;
        return `linear-gradient(${direction}, ${colors.join(', ')})`;
    }
};

// Initialize theme manager when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.themeManager = new ThemeManager();
    
    // Setup accessibility features
    window.themeManager.setupAccessibility();
    
    // Setup color scheme preferences
    window.themeManager.setupColorSchemePreferences();
    
    // Listen for theme changes to update charts and other components
    window.themeManager.onThemeChange((theme) => {
        // Update any theme-aware components
        window.dispatchEvent(new CustomEvent('themeUpdated', {
            detail: { theme }
        }));
    });
});

// Listen for theme changes from other tabs/windows
window.addEventListener('storage', (e) => {
    if (e.key === 'multios_theme' && e.newValue) {
        window.themeManager?.setTheme(e.newValue);
    }
});