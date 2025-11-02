// MultiOS Interactive Documentation JavaScript

class DocumentationBrowser {
    constructor() {
        this.currentDoc = 'README.md';
        this.docHistory = [];
        this.docIndex = 0;
        this.sidebarNav = document.getElementById('navTree');
        this.contentArea = document.getElementById('docContent');
        this.breadcrumb = document.getElementById('breadcrumb');
        
        this.initializeApp();
    }

    async initializeApp() {
        this.setupEventListeners();
        await this.buildNavigationTree();
        await this.loadInitialDoc();
        this.initializeSearch();
    }

    setupEventListeners() {
        // Search functionality
        document.getElementById('searchBtn').addEventListener('click', () => this.performSearch());
        document.getElementById('searchInput').addEventListener('keypress', (e) => {
            if (e.key === 'Enter') this.performSearch();
        });

        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => {
            if (e.ctrlKey || e.metaKey) {
                switch(e.key) {
                    case 'f':
                        e.preventDefault();
                        document.getElementById('searchInput').focus();
                        break;
                    case 'n':
                        e.preventDefault();
                        this.nextDoc();
                        break;
                    case 'p':
                        e.preventDefault();
                        this.previousDoc();
                        break;
                }
            }
        });
    }

    async buildNavigationTree() {
        const structure = {
            '📚 Documentation': ['README.md'],
            '🚀 Getting Started': [
                'getting_started/README.md',
                'getting_started/installation.md'
            ],
            '👤 User Guide': [
                'user_guide/README.md'
            ],
            '🔧 Developer Guide': [
                'developer/README.md'
            ],
            '🏗️ Architecture': [
                'architecture/README.md'
            ],
            '📖 API Reference': [
                'api/README.md'
            ],
            '📝 Tutorials': [
                'tutorials/README.md'
            ],
            '🛠️ Setup & Tools': [
                'setup/README.md',
                'setup/debugging_setup.md',
                'setup/qemu_testing.md'
            ],
            '🔬 Research & Analysis': [
                'bootloader/README.md',
                'cross_compilation/cross_compilation_guide.md',
                'multi_device_design/multi_device_patterns.md'
            ]
        };

        this.sidebarNav.innerHTML = '';
        
        for (const [folder, files] of Object.entries(structure)) {
            const folderDiv = document.createElement('div');
            folderDiv.className = 'folder';
            folderDiv.textContent = folder;
            this.sidebarNav.appendChild(folderDiv);

            const ul = document.createElement('ul');
            files.forEach(file => {
                const li = document.createElement('li');
                const a = document.createElement('a');
                a.href = '#';
                a.textContent = this.getFileDisplayName(file);
                a.onclick = () => this.loadDoc(file);
                li.appendChild(a);
                ul.appendChild(li);
            });
            this.sidebarNav.appendChild(ul);
        }
    }

    getFileDisplayName(file) {
        const parts = file.split('/');
        const filename = parts[parts.length - 1];
        return filename.replace('.md', '').replace(/_/g, ' ');
    }

    async loadInitialDoc() {
        await this.loadDoc(this.currentDoc);
    }

    async loadDoc(filename) {
        this.showLoading();
        
        try {
            const response = await fetch(`docs/${filename}`);
            if (!response.ok) {
                throw new Error(`Failed to load ${filename}`);
            }
            
            const content = await response.text();
            const html = this.parseMarkdown(content);
            
            this.contentArea.innerHTML = html;
            this.currentDoc = filename;
            this.updateBreadcrumb();
            this.highlightCurrentNav();
            this.addCodeExampleListeners();
            
            // Scroll to top
            this.contentArea.scrollTop = 0;
            
            // Add to history
            this.docHistory.push(filename);
            this.docIndex = this.docHistory.length - 1;
            
            // Update navigation buttons
            this.updateNavButtons();
            
        } catch (error) {
            console.error('Error loading document:', error);
            this.contentArea.innerHTML = `
                <div class="error">
                    <h2>Error Loading Document</h2>
                    <p>Could not load ${filename}. Please check if the file exists.</p>
                    <p><strong>Error:</strong> ${error.message}</p>
                </div>
            `;
        }
    }

    parseMarkdown(content) {
        // Convert markdown to HTML
        let html = marked.parse(content);
        
        // Add syntax highlighting to code blocks
        html = html.replace(/<pre><code class="language-(\w+)">([\s\S]*?)<\/code><\/pre>/g, 
            (match, lang, code) => {
                const decodedCode = code
                    .replace(/&lt;/g, '<')
                    .replace(/&gt;/g, '>')
                    .replace(/&amp;/g, '&')
                    .replace(/&quot;/g, '"');
                return `<pre><code class="language-${lang}">${decodedCode}</code></pre>`;
            });

        // Enhance code examples
        html = html.replace(/```(\w+)?\n([\s\S]*?)```/g, 
            (match, lang, code) => {
                const language = lang || 'rust';
                const cleanCode = code.trim();
                return `
                    <div class="code-example">
                        <div class="code-header">
                            <span class="language">${language}</span>
                            <button class="copy-btn" onclick="copyCodeExample(\`${cleanCode}\`)">📋 Copy</button>
                            <button class="run-btn" onclick="openCodeModal('${language}', \`${cleanCode}\`)">🔍 View</button>
                        </div>
                        <pre><code class="language-${language}">${cleanCode}</code></pre>
                    </div>
                `;
            });

        return html;
    }

    addCodeExampleListeners() {
        // Initialize Prism.js for syntax highlighting
        if (typeof Prism !== 'undefined') {
            Prism.highlightAll();
        }

        // Add copy functionality to code blocks
        const copyButtons = document.querySelectorAll('.copy-btn');
        copyButtons.forEach(btn => {
            btn.addEventListener('click', (e) => {
                e.preventDefault();
                const code = btn.parentElement.nextElementSibling.querySelector('code').textContent;
                this.copyToClipboard(code);
                btn.textContent = '✅ Copied!';
                setTimeout(() => btn.textContent = '📋 Copy', 2000);
            });
        });
    }

    updateBreadcrumb() {
        const parts = this.currentDoc.split('/');
        let breadcrumb = '';
        
        parts.forEach((part, index) => {
            if (index > 0) breadcrumb += ' > ';
            breadcrumb += this.getFileDisplayName(part);
        });
        
        this.breadcrumb.textContent = breadcrumb;
    }

    highlightCurrentNav() {
        // Remove active class from all nav links
        document.querySelectorAll('.nav-tree a').forEach(link => {
            link.classList.remove('active');
        });

        // Add active class to current document link
        const currentLink = Array.from(document.querySelectorAll('.nav-tree a')).find(link => {
            return link.onclick && link.onclick.toString().includes(this.currentDoc);
        });
        
        if (currentLink) {
            currentLink.classList.add('active');
        }
    }

    updateNavButtons() {
        const prevBtn = document.getElementById('prevBtn');
        const nextBtn = document.getElementById('nextBtn');
        
        prevBtn.disabled = this.docIndex <= 0;
        nextBtn.disabled = this.docIndex >= this.docHistory.length - 1;
        
        if (prevBtn.disabled) prevBtn.style.opacity = '0.5';
        else prevBtn.style.opacity = '1';
        
        if (nextBtn.disabled) nextBtn.style.opacity = '0.5';
        else nextBtn.style.opacity = '1';
    }

    previousDoc() {
        if (this.docIndex > 0) {
            this.docIndex--;
            const prevDoc = this.docHistory[this.docIndex];
            this.loadDoc(prevDoc);
        }
    }

    nextDoc() {
        if (this.docIndex < this.docHistory.length - 1) {
            this.docIndex++;
            const nextDoc = this.docHistory[this.docIndex];
            this.loadDoc(nextDoc);
        }
    }

    initializeSearch() {
        this.docsContent = {};
        // Pre-load document contents for search
        this.preloadContent();
    }

    async preloadContent() {
        const docs = [
            'README.md',
            'getting_started/README.md',
            'getting_started/installation.md',
            'user_guide/README.md',
            'developer/README.md',
            'architecture/README.md',
            'api/README.md',
            'tutorials/README.md'
        ];

        for (const doc of docs) {
            try {
                const response = await fetch(`docs/${doc}`);
                if (response.ok) {
                    this.docsContent[doc] = await response.text();
                }
            } catch (error) {
                console.warn(`Could not preload ${doc}:`, error);
            }
        }
    }

    performSearch() {
        const query = document.getElementById('searchInput').value.toLowerCase().trim();
        if (!query) return;

        const results = [];
        
        for (const [doc, content] of Object.entries(this.docsContent)) {
            const lines = content.split('\n');
            lines.forEach((line, index) => {
                if (line.toLowerCase().includes(query)) {
                    const contextStart = Math.max(0, index - 2);
                    const contextEnd = Math.min(lines.length, index + 3);
                    const context = lines.slice(contextStart, contextEnd).join(' ');
                    
                    results.push({
                        file: doc,
                        line: index + 1,
                        content: context,
                        title: this.extractTitle(content) || this.getFileDisplayName(doc)
                    });
                }
            });
        }

        this.displaySearchResults(results, query);
    }

    extractTitle(content) {
        const titleMatch = content.match(/^#\s+(.+)$/m);
        return titleMatch ? titleMatch[1] : null;
    }

    displaySearchResults(results, query) {
        const modal = document.getElementById('searchModal');
        const resultsContainer = document.getElementById('searchResults');
        
        if (results.length === 0) {
            resultsContainer.innerHTML = `<p>No results found for "${query}"</p>`;
        } else {
            resultsContainer.innerHTML = results.map(result => {
                const highlightedContent = this.highlightText(result.content, query);
                return `
                    <div class="search-result" onclick="browser.loadDoc('${result.file}')">
                        <h4>${result.title}</h4>
                        <p><strong>File:</strong> ${result.file}</p>
                        <p><strong>Line ${result.line}:</strong> ${highlightedContent}</p>
                    </div>
                `;
            }).join('');
        }
        
        modal.style.display = 'block';
    }

    highlightText(text, query) {
        const regex = new RegExp(`(${query})`, 'gi');
        return text.replace(regex, '<span class="highlight">$1</span>');
    }

    showLoading() {
        this.contentArea.innerHTML = '<div class="loading">Loading documentation...</div>';
    }

    copyToClipboard(text) {
        if (navigator.clipboard) {
            navigator.clipboard.writeText(text);
        } else {
            // Fallback for older browsers
            const textArea = document.createElement('textarea');
            textArea.value = text;
            document.body.appendChild(textArea);
            textArea.select();
            document.execCommand('copy');
            document.body.removeChild(textArea);
        }
    }
}

// Global functions for HTML onclick handlers
function previousDoc() {
    if (browser) browser.previousDoc();
}

function nextDoc() {
    if (browser) browser.nextDoc();
}

function loadDoc(filename) {
    if (browser) browser.loadDoc(filename);
}

function toggleFullscreen() {
    const container = document.querySelector('.container');
    if (!document.fullscreenElement) {
        container.classList.add('fullscreen');
        container.requestFullscreen();
    } else {
        document.exitFullscreen();
        container.classList.remove('fullscreen');
    }
}

function printDoc() {
    window.print();
}

function closeSearchModal() {
    document.getElementById('searchModal').style.display = 'none';
}

function openCodeModal(language, code) {
    const modal = document.getElementById('codeModal');
    const title = document.getElementById('codeModalTitle');
    const codeElement = document.getElementById('codeExample');
    
    title.textContent = `${language.charAt(0).toUpperCase() + language.slice(1)} Code Example`;
    codeElement.textContent = code;
    codeElement.className = `language-${language}`;
    
    if (typeof Prism !== 'undefined') {
        Prism.highlightElement(codeElement);
    }
    
    modal.style.display = 'block';
}

function closeCodeModal() {
    document.getElementById('codeModal').style.display = 'none';
}

function copyCode() {
    const code = document.getElementById('codeExample').textContent;
    browser.copyToClipboard(code);
    alert('Code copied to clipboard!');
}

function copyCodeExample(code) {
    browser.copyToClipboard(code);
    alert('Code copied to clipboard!');
}

// Initialize the documentation browser
let browser;
document.addEventListener('DOMContentLoaded', () => {
    browser = new DocumentationBrowser();
});

// Close modals when clicking outside
window.addEventListener('click', (event) => {
    const searchModal = document.getElementById('searchModal');
    const codeModal = document.getElementById('codeModal');
    
    if (event.target === searchModal) {
        closeSearchModal();
    }
    if (event.target === codeModal) {
        closeCodeModal();
    }
});