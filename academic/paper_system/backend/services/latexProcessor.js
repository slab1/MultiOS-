const { spawn } = require('child_process');
const fs = require('fs').promises;
const path = require('path');
const crypto = require('crypto');

class LaTeXProcessor {
  constructor(options = {}) {
    this.tempDir = options.tempDir || './temp/latex';
    this.outputDir = options.outputDir || './outputs/latex';
    this.maxFileSize = options.maxFileSize || 50 * 1024 * 1024; // 50MB
    this.timeout = options.timeout || 60000; // 60 seconds
    this.supportedPackages = options.supportedPackages || [
      'amsmath', 'amssymb', 'amsfonts', 'amscls',
      'graphicx', 'graphics',
      'booktabs', 'tabularx', 'longtable',
      'natbib', 'biblatex',
      'hyperref',
      'geometry', 'setspace', 'titlesec',
      'fancyhdr', 'enumitem',
      'listings', 'verbatim',
      'pdfpages',
      'tikz', 'pgf',
      'xcolor', 'color',
      'babel', 'inputenc', 'fontenc',
      'array', 'multirow', 'multicol',
      'supertabular', 'supertab'
    ];
    this.supportedEngines = options.supportedEngines || ['pdflatex', 'xelatex', 'lualatex'];
    this.bibliographyEngines = options.bibliographyEngines || ['bibtex', 'biber'];
  }

  // Create a temporary directory for compilation
  async createTempDir() {
    const timestamp = Date.now();
    const randomId = crypto.randomBytes(8).toString('hex');
    const dirName = `latex_${timestamp}_${randomId}`;
    const tempPath = path.join(this.tempDir, dirName);
    
    await fs.mkdir(tempPath, { recursive: true });
    return { tempPath, dirName };
  }

  // Clean up temporary directory
  async cleanupTempDir(tempPath) {
    try {
      await fs.rm(tempPath, { recursive: true, force: true });
    } catch (error) {
      console.warn(`Failed to cleanup temp directory ${tempPath}:`, error.message);
    }
  }

  // Validate LaTeX content
  validateLatex(latex) {
    const errors = [];
    const warnings = [];

    // Basic validation
    if (!latex || latex.trim().length === 0) {
      errors.push('LaTeX content is empty');
      return { valid: false, errors, warnings };
    }

    if (latex.length > this.maxFileSize) {
      errors.push(`LaTeX content exceeds maximum size of ${this.maxFileSize} bytes`);
    }

    // Check for basic structure
    const hasDocument = latex.includes('\\begin{document}') && latex.includes('\\end{document}');
    if (!hasDocument) {
      warnings.push('Missing \\begin{document} or \\end{document} environment');
    }

    // Check for suspicious patterns
    const suspiciousPatterns = [
      { pattern: /\\input\{[^}]*\}/, message: 'Use of \\input may cause compilation issues' },
      { pattern: /\\include\{[^}]*\}/, message: 'Use of \\include may cause compilation issues' },
      { pattern: /\\def\s*[^{]*{/, message: 'Low-level TeX definitions detected' },
      { pattern: /\\catcode/, message: 'Low-level TeX catcode changes detected' }
    ];

    suspiciousPatterns.forEach(({ pattern, message }) => {
      if (pattern.test(latex)) {
        warnings.push(message);
      }
    });

    // Check for common errors
    const commonErrors = [
      { pattern: /\\begin\{([^}]+)\}(?!.*\\end\{\1\})/, message: 'Unclosed environment detected' },
      { pattern: /\$\$/g, message: 'Use $$ for display math is discouraged; use \\[\\] instead' },
      { pattern: /\\marginpar\{/, message: 'Margin notes may not render correctly' },
      { pattern: /\\footnote\{[^}]*$/, message: 'Unclosed footnote detected' }
    ];

    commonErrors.forEach(({ pattern, message }) => {
      if (pattern.test(latex)) {
        warnings.push(message);
      }
    });

    // Check bibliography configuration
    const hasBibliography = latex.includes('\\bibliography') || latex.includes('\\bibliographystyle');
    const hasBiblatex = latex.includes('biblatex');
    
    if (hasBibliography && !hasBiblatex) {
      warnings.push('Consider using biblatex instead of traditional bibliography commands');
    }

    // Validate math environments
    const mathErrors = [
      { pattern: /\$(?:(?!\$).)*\$/g, message: 'Inline math should use \\(...\\) instead of $...$' },
      { pattern: /\$\$(?:(?!\$\$).)*\$\$/g, message: 'Display math should use \\[...\\] instead of $$...$$' }
    ];

    mathErrors.forEach(({ pattern, message }) => {
      const matches = latex.match(pattern);
      if (matches && matches.length > 0) {
        errors.push(`${message} (found ${matches.length} instances)`);
      }
    });

    return {
      valid: errors.length === 0,
      errors,
      warnings,
      stats: this.extractLatexStats(latex)
    };
  }

  // Extract statistics from LaTeX
  extractLatexStats(latex) {
    const stats = {
      totalWords: 0,
      sections: 0,
      subsections: 0,
      figures: 0,
      tables: 0,
      references: 0,
      citations: 0,
      equations: 0,
      algorithms: 0,
      codeBlocks: 0
    };

    // Count words (approximate)
    const textContent = latex
      .replace(/\\[a-zA-Z]+\{[^}]*\}/g, ' ') // Remove LaTeX commands with arguments
      .replace(/\\[a-zA-Z]+/g, ' ') // Remove LaTeX commands
      .replace(/[{}]/g, ' ') // Remove braces
      .replace(/\s+/g, ' ') // Normalize whitespace
      .trim();
    
    stats.totalWords = textContent.split(' ').filter(word => word.length > 0).length;

    // Count environments and commands
    stats.sections = (latex.match(/\\section\{/g) || []).length;
    stats.subsections = (latex.match(/\\subsection\{/g) || []).length;
    stats.figures = (latex.match(/\\begin\{figure\}/g) || []).length;
    stats.tables = (latex.match(/\\begin\{table\}/g) || []).length;
    stats.references = (latex.match(/\\ref\{/g) || []).length;
    stats.citations = (latex.match(/\\cite(?:[tp])?\{/g) || []).length;
    stats.equations = (latex.match(/\\begin\{equation\}/g) || []).length + (latex.match(/\$\$/g) || []).length / 2;
    stats.algorithms = (latex.match(/\\begin\{algorithm\}/g) || []).length;
    stats.codeBlocks = (latex.match(/\\begin\{lstlisting\}/g) || []).length + (latex.match(/\\begin\{verbatim\}/g) || []).length;

    return stats;
  }

  // Compile LaTeX document
  async compile(latex, options = {}) {
    const {
      engine = 'pdflatex',
      bibliographyEngine = 'bibtex',
      packages = [],
      bibliography = null,
      images = [],
      mainFile = 'main.tex'
    } = options;

    const validation = this.validateLatex(latex);
    if (!validation.valid) {
      throw new Error(`LaTeX validation failed: ${validation.errors.join(', ')}`);
    }

    const { tempPath } = await this.createTempDir();
    
    try {
      // Write main LaTeX file
      const mainFilePath = path.join(tempPath, mainFile);
      await fs.writeFile(mainFilePath, latex, 'utf8');

      // Write additional files (packages, images, bibliography)
      if (packages.length > 0) {
        await fs.writeFile(path.join(tempPath, 'packages.tex'), packages.join('\n'), 'utf8');
      }

      if (bibliography) {
        const bibContent = typeof bibliography === 'string' ? bibliography : 
          this.generateBibFile(bibliography);
        await fs.writeFile(path.join(tempPath, 'bibliography.bib'), bibContent, 'utf8');
      }

      // Handle images
      for (const image of images) {
        if (image.content && image.filename) {
          const imagePath = path.join(tempPath, image.filename);
          await fs.writeFile(imagePath, image.content);
        }
      }

      // Compilation steps
      const results = await this.performCompilation(tempPath, mainFile, {
        engine,
        bibliographyEngine
      });

      // Generate PDF
      const pdfResult = await this.extractPDF(tempPath, mainFile, engine);
      
      return {
        success: true,
        validation,
        compilation: results,
        pdf: pdfResult,
        outputFiles: await this.listOutputFiles(tempPath)
      };

    } catch (error) {
      throw new Error(`LaTeX compilation failed: ${error.message}`);
    } finally {
      await this.cleanupTempDir(tempPath);
    }
  }

  // Perform compilation steps
  async performCompilation(tempPath, mainFile, options) {
    const { engine, bibliographyEngine } = options;
    const results = {
      steps: [],
      errors: [],
      warnings: []
    };

    try {
      // Step 1: First LaTeX run
      const firstRun = await this.runLaTeX(tempPath, mainFile, engine);
      results.steps.push({
        step: 'first_latex',
        output: firstRun.stdout,
        errors: firstRun.stderr,
        success: firstRun.exitCode === 0
      });

      if (firstRun.exitCode !== 0) {
        throw new Error(`First LaTeX run failed: ${firstRun.stderr}`);
      }

      // Step 2: Bibliography processing
      const hasBibliography = await this.hasBibliography(tempPath, mainFile);
      if (hasBibliography) {
        const bibRun = await this.runBibliography(tempPath, bibliographyEngine);
        results.steps.push({
          step: 'bibliography',
          output: bibRun.stdout,
          errors: bibRun.stderr,
          success: bibRun.exitCode === 0
        });

        if (bibRun.exitCode !== 0) {
          results.warnings.push('Bibliography processing failed, continuing without references');
        }
      }

      // Step 3: Second LaTeX run (for references)
      const secondRun = await this.runLaTeX(tempPath, mainFile, engine);
      results.steps.push({
        step: 'second_latex',
        output: secondRun.stdout,
        errors: secondRun.stderr,
        success: secondRun.exitCode === 0
      });

      if (secondRun.exitCode !== 0) {
        results.warnings.push('Second LaTeX run failed, document may have unresolved references');
      }

      // Step 4: Third LaTeX run (if needed)
      const thirdRun = await this.runLaTeX(tempPath, mainFile, engine);
      results.steps.push({
        step: 'third_latex',
        output: thirdRun.stdout,
        errors: thirdRun.stderr,
        success: thirdRun.exitCode === 0
      });

      return results;

    } catch (error) {
      results.errors.push(error.message);
      return results;
    }
  }

  // Run LaTeX compilation
  runLaTeX(tempPath, mainFile, engine) {
    return new Promise((resolve, reject) => {
      const args = [
        '-interaction=nonstopmode',
        '-halt-on-error',
        '-output-directory', tempPath,
        path.join(tempPath, mainFile)
      ];

      const process = spawn(engine, args, {
        cwd: tempPath,
        timeout: this.timeout
      });

      let stdout = '';
      let stderr = '';

      process.stdout.on('data', (data) => {
        stdout += data.toString();
      });

      process.stderr.on('data', (data) => {
        stderr += data.toString();
      });

      process.on('close', (exitCode) => {
        resolve({ stdout, stderr, exitCode });
      });

      process.on('error', (error) => {
        reject(error);
      });
    });
  }

  // Run bibliography processing
  runBibliography(tempPath, engine) {
    return new Promise((resolve, reject) => {
      const args = [path.join(tempPath, 'bibliography')];

      const process = spawn(engine, args, {
        cwd: tempPath,
        timeout: this.timeout
      });

      let stdout = '';
      let stderr = '';

      process.stdout.on('data', (data) => {
        stdout += data.toString();
      });

      process.stderr.on('data', (data) => {
        stderr += data.toString();
      });

      process.on('close', (exitCode) => {
        resolve({ stdout, stderr, exitCode });
      });

      process.on('error', (error) => {
        reject(error);
      });
    });
  }

  // Check if document has bibliography
  async hasBibliography(tempPath, mainFile) {
    try {
      const content = await fs.readFile(path.join(tempPath, mainFile), 'utf8');
      return content.includes('\\bibliography') || content.includes('biblatex') || content.includes('\\bibitem');
    } catch {
      return false;
    }
  }

  // Extract PDF from compilation output
  async extractPDF(tempPath, mainFile, engine) {
    try {
      const pdfName = path.basename(mainFile, '.tex') + '.pdf';
      const pdfPath = path.join(tempPath, pdfName);
      
      const pdfBuffer = await fs.readFile(pdfPath);
      const pdfBase64 = pdfBuffer.toString('base64');
      
      return {
        base64: pdfBase64,
        size: pdfBuffer.length,
        path: pdfPath
      };
    } catch (error) {
      throw new Error('Failed to extract PDF: ' + error.message);
    }
  }

  // List output files
  async listOutputFiles(tempPath) {
    try {
      const files = await fs.readdir(tempPath);
      return files.map(filename => ({
        name: filename,
        path: path.join(tempPath, filename),
        size: 0 // Could be enhanced to include actual file sizes
      }));
    } catch (error) {
      return [];
    }
  }

  // Generate BibTeX file from citation data
  generateBibFile(citations) {
    if (typeof citations === 'string') {
      return citations;
    }

    let bibContent = '';
    
    citations.forEach(citation => {
      const type = citation.type || 'article';
      const key = citation.key || citation.bibtexKey || `${citation.firstAuthor}_${citation.year}`;
      
      let entry = `@${type}{${key},\n`;
      
      Object.entries(citation.fields || {}).forEach(([field, value]) => {
        if (value) {
          entry += `  ${field} = {${value}},\n`;
        }
      });
      
      entry = entry.replace(/,\n$/, '\n');
      entry += '}\n\n';
      
      bibContent += entry;
    });
    
    return bibContent;
  }

  // Convert LaTeX to other formats
  async convert(latex, targetFormat, options = {}) {
    const validation = this.validateLatex(latex);
    if (!validation.valid) {
      throw new Error(`LaTeX validation failed: ${validation.errors.join(', ')}`);
    }

    switch (targetFormat) {
      case 'html':
        return this.convertToHTML(latex, options);
      case 'markdown':
        return this.convertToMarkdown(latex, options);
      case 'text':
        return this.convertToText(latex, options);
      default:
        throw new Error(`Unsupported target format: ${targetFormat}`);
    }
  }

  // Convert LaTeX to HTML (simplified conversion)
  async convertToHTML(latex, options = {}) {
    let html = latex
      // Convert sections
      .replace(/\\section\{([^}]+)\}/g, '<h1>$1</h1>')
      .replace(/\\subsection\{([^}]+)\}/g, '<h2>$1</h2>')
      .replace(/\\subsubsection\{([^}]+)\}/g, '<h3>$1</h3>')
      // Convert emphasis
      .replace(/\\textbf\{([^}]+)\}/g, '<strong>$1</strong>')
      .replace(/\\textit\{([^}]+)\}/g, '<em>$1</em>')
      // Convert lists
      .replace(/\\begin\{itemize\}/g, '<ul>')
      .replace(/\\end\{itemize\}/g, '</ul>')
      .replace(/\\begin\{enumerate\}/g, '<ol>')
      .replace(/\\end\{enumerate\}/g, '</ol>')
      .replace(/\\item\s*/g, '<li>')
      // Convert environments
      .replace(/\\begin\{quote\}/g, '<blockquote>')
      .replace(/\\end\{quote\}/g, '</blockquote>')
      // Remove document environment
      .replace(/\\begin\{document\}/g, '<div class="document">')
      .replace(/\\end\{document\}/g, '</div>');

    return {
      html,
      metadata: {
        title: this.extractTitle(latex),
        sections: this.extractLatexStats(latex).sections,
        wordCount: this.extractLatexStats(latex).totalWords
      }
    };
  }

  // Convert LaTeX to Markdown
  async convertToMarkdown(latex, options = {}) {
    let markdown = latex
      // Convert sections
      .replace(/\\section\{([^}]+)\}/g, '# $1')
      .replace(/\\subsection\{([^}]+)\}/g, '## $1')
      .replace(/\\subsubsection\{([^}]+)\}/g, '### $1')
      // Convert emphasis
      .replace(/\\textbf\{([^}]+)\}/g, '**$1**')
      .replace(/\\textit\{([^}]+)\}/g, '*$1*')
      // Convert lists
      .replace(/\\begin\{itemize\}/g, '')
      .replace(/\\end\{itemize\}/g, '')
      .replace(/\\begin\{enumerate\}/g, '')
      .replace(/\\end\{enumerate\}/g, '')
      .replace(/\\item\s*/g, '- ')
      // Remove document environment
      .replace(/\\begin\{document\}/g, '')
      .replace(/\\end\{document\}/g, '');

    return {
      markdown,
      metadata: {
        title: this.extractTitle(latex),
        sections: this.extractLatexStats(latex).sections,
        wordCount: this.extractLatexStats(latex).totalWords
      }
    };
  }

  // Convert LaTeX to plain text
  async convertToText(latex, options = {}) {
    const text = latex
      // Remove LaTeX commands
      .replace(/\\[a-zA-Z]+\{[^}]*\}/g, ' ')
      .replace(/\\[a-zA-Z]+/g, ' ')
      // Remove braces
      .replace(/[{}]/g, ' ')
      // Normalize whitespace
      .replace(/\s+/g, ' ')
      .trim();

    return {
      text,
      metadata: {
        title: this.extractTitle(latex),
        wordCount: this.extractLatexStats(latex).totalWords,
        characterCount: text.length
      }
    };
  }

  // Extract title from LaTeX
  extractTitle(latex) {
    const titleMatch = latex.match(/\\title\{([^}]+)\}/);
    if (titleMatch) {
      return titleMatch[1];
    }
    
    // Fallback to first section or document content
    const sectionMatch = latex.match(/\\section\{([^}]+)\}/);
    if (sectionMatch) {
      return sectionMatch[1];
    }
    
    // Extract first non-empty line
    const lines = latex.split('\n');
    for (const line of lines) {
      const trimmed = line.trim();
      if (trimmed && !trimmed.startsWith('%') && !trimmed.startsWith('\\')) {
        return trimmed.substring(0, 100);
      }
    }
    
    return 'Untitled Document';
  }

  // Get LaTeX template
  async getTemplate(templateType) {
    const templates = {
      acm: `\\documentclass[acmsmall]{acmart}

\\usepackage{graphicx}
\\usepackage{booktabs}
\\usepackage{hyperref}
\\usepackage{siunitx}
\\usepackage{listings}
\\usepackage{color}

\\title{Your Paper Title}
\\author{Your Name}
\\affiliation{Your Institution}

\\begin{document}
\\maketitle

\\begin{abstract}
Your abstract goes here.
\\end{abstract}

\\section{Introduction}
Your introduction content goes here.

\\section{Related Work}
Your related work section goes here.

\\section{Methodology}
Your methodology section goes here.

\\section{Results}
Your results section goes here.

\\section{Conclusion}
Your conclusion goes here.

\\bibliographystyle{ACM-Reference-Format}
\\bibliography{references}

\\end{document}`,

      ieee: `\\documentclass[conference]{IEEEtran}

\\usepackage{graphicx}
\\usepackage{cite}
\\usepackage{amsmath}
\\usepackage{hyperref}
\\usepackage{color}

\\title{Your Paper Title}
\\author{Your Name}

\\begin{document}
\\maketitle

\\begin{abstract}
Your abstract goes here.
\\end{abstract}

\\section{Introduction}
Your introduction content goes here.

\\section{Related Work}
Your related work section goes here.

\\section{Methodology}
Your methodology section goes here.

\\section{Results}
Your results section goes here.

\\section{Conclusion}
Your conclusion goes here.

\\bibliographystyle{IEEEtran}
\\bibliography{references}

\\end{document}`,

      generic: `\\documentclass{article}

\\usepackage{graphicx}
\\usepackage{hyperref}
\\usepackage{geometry}
\\usepackage{setspace}

\\geometry{margin=1in}
\\doublespacing

\\title{Your Paper Title}
\\author{Your Name}

\\begin{document}
\\maketitle

\\begin{abstract}
Your abstract goes here.
\\end{abstract}

\\section{Introduction}
Your introduction content goes here.

\\section{Related Work}
Your related work section goes here.

\\section{Methodology}
Your methodology section goes here.

\\section{Results}
Your results section goes here.

\\section{Conclusion}
Your conclusion goes here.

\\bibliographystyle{plain}
\\bibliography{references}

\\end{document}`
    };

    return {
      template: templates[templateType] || templates.generic,
      type: templateType,
      supportedPackages: this.supportedPackages
    };
  }

  // Health check for LaTeX installation
  async healthCheck() {
    const checks = {
      engines: {},
      bibliographyEngines: {},
      packages: {},
      memory: {}
    };

    // Check LaTeX engines
    for (const engine of this.supportedEngines) {
      try {
        const result = await this.runCommand(engine, ['-version']);
        checks.engines[engine] = {
          available: true,
          version: result.stdout.split('\n')[0]
        };
      } catch (error) {
        checks.engines[engine] = {
          available: false,
          error: error.message
        };
      }
    }

    // Check bibliography engines
    for (const engine of this.bibliographyEngines) {
      try {
        const result = await this.runCommand(engine, ['-version']);
        checks.bibliographyEngines[engine] = {
          available: true,
          version: result.stdout.split('\n')[0]
        };
      } catch (error) {
        checks.bibliographyEngines[engine] = {
          available: false,
          error: error.message
        };
      }
    }

    return checks;
  }

  // Helper method to run commands
  runCommand(command, args) {
    return new Promise((resolve, reject) => {
      const process = spawn(command, args, { timeout: 10000 });
      
      let stdout = '';
      let stderr = '';

      process.stdout.on('data', (data) => {
        stdout += data.toString();
      });

      process.stderr.on('data', (data) => {
        stderr += data.toString();
      });

      process.on('close', (exitCode) => {
        resolve({ stdout, stderr, exitCode });
      });

      process.on('error', (error) => {
        reject(error);
      });
    });
  }
}

module.exports = LaTeXProcessor;