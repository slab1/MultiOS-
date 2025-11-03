const express = require('express');
const fs = require('fs').promises;
const path = require('path');
const { exec } = require('child_process');
const util = require('util');
const execPromise = util.promisify(exec);
const Paper = require('../models/Paper');
const { auth } = require('../middleware/auth');
const Joi = require('joi');

const router = express.Router();

// Validation schemas
const compileSchema = Joi.object({
  latex: Joi.string().required(),
  mainFile: Joi.string().default('main.tex'),
  packages: Joi.array().items(Joi.string()).default([]),
  bibliographyEngine: Joi.string().valid('bibtex', 'biber').default('bibtex')
});

const validateSchema = Joi.object({
  paperId: Joi.string().required(),
  validationType: Joi.string().valid('structure', 'formatting', 'content', 'comprehensive').default('comprehensive'),
  strictMode: Joi.boolean().default(false)
});

// Compile LaTeX document
router.post('/compile', auth, async (req, res) => {
  try {
    const { error, value } = compileSchema.validate(req.body);
    if (error) {
      return res.status(400).json({ 
        error: 'Validation failed',
        details: error.details.map(detail => detail.message)
      });
    }

    const { latex, mainFile, packages, bibliographyEngine } = value;
    
    // Create temporary directory for compilation
    const tempDir = path.join(__dirname, '../temp', `compile_${Date.now()}_${req.user.userId}`);
    await fs.mkdir(tempDir, { recursive: true });

    try {
      // Write main LaTeX file
      const mainTexPath = path.join(tempDir, mainFile);
      await fs.writeFile(mainTexPath, latex, 'utf8');

      // Create a basic LaTeX structure if needed
      const hasDocumentClass = latex.includes('\\documentclass');
      const hasBeginDocument = latex.includes('\\begin{document}');
      const hasEndDocument = latex.includes('\\end{document}');

      let processedLatex = latex;

      // Wrap in document if missing
      if (hasDocumentClass && !hasBeginDocument) {
        processedLatex = latex.replace('\\documentclass', '\\begin{document}\\n\\documentclass');
      }

      // Create minimal working LaTeX if needed
      if (!hasDocumentClass) {
        const minimalLatex = `\\documentclass[11pt]{article}
\\usepackage{geometry}
\\geometry{margin=1in}
\\usepackage{graphicx}
\\usepackage{hyperref}
\\usepackage{url}
\\usepackage{listings}
\\usepackage{color}
\\usepackage{array}
\\usepackage{longtable}
\\usepackage{booktabs}

\\title{Paper Title}
\\author{Author Name}
\\date{\\today}

\\begin{document}
\\maketitle

${processedLatex}

\\end{document}`;
        
        await fs.writeFile(mainTexPath, minimalLatex, 'utf8');
      }

      // Compilation commands
      const commands = [];
      
      // First pass
      commands.push(`cd "${tempDir}" && pdflatex -interaction=nonstopmode "${mainFile}"`);
      
      // Handle bibliography if present
      if (latex.includes('\\bibliography') || latex.includes('\\bibitem')) {
        commands.push(`cd "${tempDir}" && ${bibliographyEngine} "${mainFile.replace('.tex', '')}"`);
        commands.push(`cd "${tempDir}" && pdflatex -interaction=nonstopmode "${mainFile}"`);
      }
      
      // Final pass
      commands.push(`cd "${tempDir}" && pdflatex -interaction=nonstopmode "${mainFile}"`);

      // Execute compilation
      const compilationResults = [];
      let finalPdfPath = null;
      let logContent = '';
      
      for (let i = 0; i < commands.length; i++) {
        try {
          const { stdout, stderr } = await execPromise(commands[i], { timeout: 30000 });
          
          if (stdout) {
            logContent += `Pass ${i + 1} stdout:\n${stdout}\n`;
          }
          if (stderr) {
            logContent += `Pass ${i + 1} stderr:\n${stderr}\n`;
          }

          // Check if PDF was generated
          const pdfPath = path.join(tempDir, mainFile.replace('.tex', '.pdf'));
          try {
            await fs.access(pdfPath);
            finalPdfPath = pdfPath;
          } catch (err) {
            // PDF not generated yet, continue
          }

          compilationResults.push({
            pass: i + 1,
            success: true,
            stdout,
            stderr
          });

        } catch (error) {
          logContent += `Pass ${i + 1} error:\n${error.stderr || error.message}\n`;
          
          compilationResults.push({
            pass: i + 1,
            success: false,
            error: error.stderr || error.message
          });

          // Continue with remaining passes even if one fails
          if (error.message.includes('Timeout')) {
            break; // Stop if timeout occurred
          }
        }
      }

      // Read log file for detailed information
      const logPath = path.join(tempDir, mainFile.replace('.tex', '.log'));
      let detailedLog = '';
      try {
        detailedLog = await fs.readFile(logPath, 'utf8');
      } catch (err) {
        // Log file not found, use accumulated logs
      }

      // Check compilation success
      const success = finalPdfPath !== null && !logContent.includes('! Emergency stop');

      // Extract warnings and errors from log
      const warnings = [];
      const errors = [];
      const info = [];

      if (detailedLog) {
        const lines = detailedLog.split('\n');
        lines.forEach(line => {
          if (line.includes('Warning:')) {
            warnings.push(line.trim());
          } else if (line.startsWith('!')) {
            errors.push(line.trim());
          } else if (line.includes('LaTeX Warning:')) {
            warnings.push(line.trim());
          }
        });
      }

      // Return compilation results
      const result = {
        success,
        hasPdf: finalPdfPath !== null,
        compilationResults,
        warnings: warnings.slice(0, 20), // Limit to 20 warnings
        errors: errors.slice(0, 10), // Limit to 10 errors
        logContent: detailedLog || logContent,
        fileCount: 0,
        compilationTime: 0
      };

      if (finalPdfPath) {
        try {
          const pdfStats = await fs.stat(finalPdfPath);
          result.fileSize = pdfStats.size;
          result.compilationTime = Date.now() - parseInt(tempDir.split('_').pop());
        } catch (err) {
          // Stats not available
        }
      }

      res.json({
        message: success ? 'LaTeX compiled successfully' : 'LaTeX compilation completed with issues',
        result
      });

    } finally {
      // Clean up temporary files
      try {
        await fs.rmdir(tempDir, { recursive: true });
      } catch (err) {
        console.warn(`Failed to clean up temp directory: ${tempDir}`, err);
      }
    }

  } catch (error) {
    console.error('LaTeX compilation error:', error);
    res.status(500).json({ 
      error: 'LaTeX compilation failed',
      message: 'An error occurred during LaTeX compilation',
      details: error.message
    });
  }
});

// Validate paper structure and formatting
router.post('/validate', auth, async (req, res) => {
  try {
    const { error, value } = validateSchema.validate(req.body);
    if (error) {
      return res.status(400).json({ 
        error: 'Validation failed',
        details: error.details.map(detail => detail.message)
      });
    }

    const { paperId, validationType, strictMode } = value;

    // Get paper
    const paper = await Paper.findById(paperId).populate('createdBy');
    if (!paper) {
      return res.status(404).json({ error: 'Paper not found' });
    }

    // Check access permissions
    const isOwner = paper.createdBy._id.toString() === req.user.userId;
    const isEditor = req.user.role === 'editor' || req.user.role === 'admin';

    if (!isOwner && !isEditor) {
      return res.status(403).json({ error: 'Access denied' });
    }

    const latexContent = paper.content.latex;
    if (!latexContent) {
      return res.status(400).json({ error: 'No LaTeX content found for this paper' });
    }

    const validationResults = {
      paperId,
      validationType,
      strictMode,
      timestamp: new Date(),
      results: {
        structure: { score: 0, issues: [], suggestions: [] },
        formatting: { score: 0, issues: [], suggestions: [] },
        content: { score: 0, issues: [], suggestions: [] },
        technical: { score: 0, issues: [], suggestions: [] }
      },
      overallScore: 0,
      recommendations: []
    };

    // Validate document structure
    if (validationType === 'structure' || validationType === 'comprehensive') {
      const structureChecks = validateDocumentStructure(latexContent, strictMode);
      validationResults.results.structure = structureChecks;
    }

    // Validate formatting
    if (validationType === 'formatting' || validationType === 'comprehensive') {
      const formattingChecks = validateFormatting(latexContent, strictMode);
      validationResults.results.formatting = formattingChecks;
    }

    // Validate content
    if (validationType === 'content' || validationType === 'comprehensive') {
      const contentChecks = validateContent(latexContent, paper.abstract, strictMode);
      validationResults.results.content = contentChecks;
    }

    // Technical validation
    const technicalChecks = validateTechnical(latexContent, strictMode);
    validationResults.results.technical = technicalChecks;

    // Calculate overall score
    const scores = Object.values(validationResults.results).map(r => r.score);
    validationResults.overallScore = Math.round(
      scores.reduce((sum, score) => sum + score, 0) / scores.length
    );

    // Generate recommendations
    validationResults.recommendations = generateRecommendations(validationResults.results);

    res.json({
      message: 'Paper validation completed',
      validation: validationResults
    });

  } catch (error) {
    console.error('Paper validation error:', error);
    res.status(500).json({ 
      error: 'Validation failed',
      message: 'An error occurred during paper validation'
    });
  }
});

// Convert LaTeX to other formats
router.post('/convert', auth, async (req, res) => {
  try {
    const { latex, targetFormat, options = {} } = req.body;

    if (!latex || !targetFormat) {
      return res.status(400).json({ 
        error: 'Missing required fields',
        message: 'LaTeX content and target format are required'
      });
    }

    const supportedFormats = ['html', 'markdown', 'docx', 'rtf', 'txt'];
    if (!supportedFormats.includes(targetFormat)) {
      return res.status(400).json({ 
        error: 'Unsupported format',
        message: `Supported formats: ${supportedFormats.join(', ')}`
      });
    }

    // Create temporary directory
    const tempDir = path.join(__dirname, '../temp', `convert_${Date.now()}_${req.user.userId}`);
    await fs.mkdir(tempDir, { recursive: true });

    try {
      // Write LaTeX file
      const texPath = path.join(tempDir, 'document.tex');
      await fs.writeFile(texPath, latex, 'utf8');

      let conversionResult = null;

      // Simple LaTeX to HTML conversion (basic implementation)
      if (targetFormat === 'html') {
        conversionResult = await convertLatexToHtml(latex, options);
      } else if (targetFormat === 'markdown') {
        conversionResult = await convertLatexToMarkdown(latex, options);
      } else if (targetFormat === 'txt') {
        conversionResult = await convertLatexToText(latex, options);
      }

      res.json({
        message: `Converted to ${targetFormat} successfully`,
        convertedContent: conversionResult.content,
        metadata: conversionResult.metadata,
        warnings: conversionResult.warnings || []
      });

    } finally {
      // Clean up
      try {
        await fs.rmdir(tempDir, { recursive: true });
      } catch (err) {
        console.warn(`Failed to clean up temp directory: ${tempDir}`, err);
      }
    }

  } catch (error) {
    console.error('LaTeX conversion error:', error);
    res.status(500).json({ 
      error: 'Conversion failed',
      message: 'An error occurred during format conversion'
    });
  }
});

// Get LaTeX templates
router.get('/templates/:templateType', auth, async (req, res) => {
  try {
    const { templateType } = req.params;
    
    const templates = {
      'conference-paper': `\\documentclass[conference]{IEEEtran}
\\usepackage{geometry}
\\geometry{margin=1in}
\\usepackage{graphicx}
\\usepackage{url}
\\usepackage{cite}
\\usepackage{amsmath,amssymb}
\\usepackage{algorithm}
\\usepackage{algorithmic}
\\usepackage{array}
\\usepackage{booktabs}
\\usepackage{color}
\\usepackage{hyperref}

\\title{Title of Your Paper}
\\author{
\\IEEEauthorblockN{First Author\\IEEEauthorrefmark{1}, Second Author\\IEEEauthorrefmark{2}, and Third Author\\IEEEauthorrefmark{3}}
\\IEEEauthorblockA{\\IEEEauthorrefmark{1}Department of Computer Science, University Name\\
Email: first.author@university.edu}
\\IEEEauthorblockA{\\IEEEauthorrefmark{2}Research Institute Name\\
Email: second.author@research.org}
\\IEEEauthorblockA{\\IEEEauthorrefmark{3}Company Name\\
Email: third.author@company.com}
}

\\begin{document}
\\maketitle

\\begin{abstract}
Your abstract goes here. It should be a concise summary of the paper's contributions and findings.
\\end{abstract}

\\section{Introduction}
Your introduction goes here.

\\section{Related Work}
Your related work section goes here.

\\section{Methodology}
Your methodology section goes here.

\\section{Experimental Results}
Your experimental results section goes here.

\\section{Conclusion}
Your conclusion goes here.

\\bibliographystyle{IEEEtran}
\\bibliography{references}

\\end{document}`,
      
      'journal-article': `\\documentclass[11pt]{article}
\\usepackage{geometry}
\\geometry{margin=1in}
\\usepackage{graphicx}
\\usepackage{url}
\\usepackage{cite}
\\usepackage{amsmath,amssymb}
\\usepackage{algorithm}
\\usepackage{algorithmic}
\\usepackage{array}
\\usepackage{booktabs}
\\usepackage{color}
\\usepackage{hyperref}
\\usepackage{fancyhdr}

\\title{Title of Your Journal Article}
\\author{Your Name\\thanks{Your email address}}
\\date{\\today}

\\begin{document}
\\maketitle

\\begin{abstract}
Your abstract goes here. It should be a concise summary of the paper's contributions and findings.
\\end{abstract}

\\section{Introduction}
Your introduction goes here.

\\section{Related Work}
Your related work section goes here.

\\section{Problem Statement}
Your problem statement goes here.

\\section{Solution Approach}
Your solution approach goes here.

\\section{Experimental Evaluation}
Your experimental evaluation goes here.

\\section{Discussion}
Your discussion goes here.

\\section{Conclusion}
Your conclusion goes here.

\\bibliographystyle{plain}
\\bibliography{references}

\\end{document}`,

      'thesis': `\\documentclass[12pt]{report}
\\usepackage{geometry}
\\geometry{margin=1in}
\\usepackage{graphicx}
\\usepackage{url}
\\usepackage{cite}
\\usepackage{setspace}
\\usepackage{hyperref}

\\title{Title of Your Thesis}
\\author{Your Name}
\\date{\\today}

\\begin{document}

\\maketitle

\\begin{abstract}
\\doublespacing
Your abstract goes here.
\\end{abstract}

\\tableofcontents

\\listoffigures

\\listoftables

\\chapter{Introduction}
Your introduction goes here.

\\chapter{Literature Review}
Your literature review goes here.

\\chapter{Methodology}
Your methodology goes here.

\\chapter{Results}
Your results go here.

\\chapter{Discussion}
Your discussion goes here.

\\chapter{Conclusion}
Your conclusion goes here.

\\bibliographystyle{plain}
\\bibliography{references}

\\end{document}`
    };

    const template = templates[templateType];
    if (!template) {
      return res.status(404).json({ 
        error: 'Template not found',
        message: `Available templates: ${Object.keys(templates).join(', ')}`
      });
    }

    res.json({
      templateType,
      template,
      description: getTemplateDescription(templateType)
    });

  } catch (error) {
    console.error('Get template error:', error);
    res.status(500).json({ 
      error: 'Failed to get template',
      message: 'An error occurred while retrieving the template'
    });
  }
});

// Helper functions for validation and conversion
function validateDocumentStructure(latex, strictMode) {
  const issues = [];
  const suggestions = [];
  let score = 100;

  // Check for document class
  if (!latex.includes('\\documentclass')) {
    issues.push('Missing document class declaration');
    suggestions.push('Add \\documentclass command at the beginning of your document');
    score -= 15;
  }

  // Check for document environment
  if (!latex.includes('\\begin{document}')) {
    issues.push('Missing document environment');
    suggestions.push('Wrap your content in \\begin{document}...\\end{document}');
    score -= 20;
  }

  // Check for title
  if (!latex.includes('\\title{')) {
    issues.push('Missing document title');
    suggestions.push('Add a title using \\title{Your Title Here}');
    score -= 10;
  }

  // Check for author
  if (!latex.includes('\\author{')) {
    issues.push('Missing author information');
    suggestions.push('Add author information using \\author{Author Name}');
    score -= 10;
  }

  // Check for abstract
  if (!latex.includes('\\begin{abstract}')) {
    issues.push('Missing abstract');
    if (!strictMode) {
      suggestions.push('Consider adding an abstract for better readability');
    } else {
      suggestions.push('Add an abstract using \\begin{abstract}...\\end{abstract}');
    }
    score -= strictMode ? 10 : 5;
  }

  // Check for sections
  const sectionCount = (latex.match(/\\section\{/g) || []).length;
  if (sectionCount < 1) {
    issues.push('No sections found');
    suggestions.push('Organize your content using \\section{} commands');
    score -= 15;
  }

  return { score: Math.max(0, score), issues, suggestions };
}

function validateFormatting(latex, strictMode) {
  const issues = [];
  const suggestions = [];
  let score = 100;

  // Check for figure formatting
  const figureCount = (latex.match(/\\begin\{figure\}/g) || []).length;
  if (figureCount > 0) {
    const captionCount = (latex.match(/\\caption\{/g) || []).length;
    if (captionCount < figureCount) {
      issues.push('Some figures missing captions');
      suggestions.push('Add captions to all figures using \\caption{}');
      score -= 10;
    }
  }

  // Check for table formatting
  const tableCount = (latex.match(/\\begin\{table\}/g) || []).length;
  if (tableCount > 0) {
    const captionCount = (latex.match(/\\caption\{/g) || []).length;
    if (captionCount < tableCount) {
      issues.push('Some tables missing captions');
      suggestions.push('Add captions to all tables using \\caption{}');
      score -= 10;
    }
  }

  // Check for reference formatting
  const citeCount = (latex.match(/\\cite\{/g) || []).length;
  const bibliographyCount = (latex.match(/\\bibliography\{/g) || []).length;
  
  if (citeCount > 0 && bibliographyCount === 0) {
    issues.push('Citations found but no bibliography');
    suggestions.push('Add bibliography using \\bibliography{filename}');
    score -= 15;
  }

  return { score: Math.max(0, score), issues, suggestions };
}

function validateContent(latex, abstract, strictMode) {
  const issues = [];
  const suggestions = [];
  let score = 100;

  // Abstract validation
  if (abstract && abstract.length < 100) {
    issues.push('Abstract might be too short');
    suggestions.push('Consider expanding the abstract to at least 100 words');
    score -= 10;
  }

  // Content length check (rough estimation)
  const contentLength = latex.length;
  if (contentLength < 2000 && strictMode) {
    issues.push('Document might be too short');
    suggestions.push('Consider expanding the content for a complete paper');
    score -= 20;
  }

  return { score: Math.max(0, score), issues, suggestions };
}

function validateTechnical(latex, strictMode) {
  const issues = [];
  const suggestions = [];
  let score = 100;

  // Check for common LaTeX errors
  if (latex.includes('$$') && latex.includes('\\[')) {
    issues.push('Mixed inline and display math environments');
    suggestions.push('Choose either $...$ or \\[...\\] for math, not both');
    score -= 5;
  }

  // Check for special characters that might cause issues
  const problematicChars = ['%', '&', '#', '_'];
  const foundProblems = problematicChars.filter(char => 
    latex.includes(char) && !latex.includes('\\' + char)
  );

  if (foundProblems.length > 0) {
    issues.push('Potential special character issues');
    suggestions.push('Escape special characters (% & # _) with backslash');
    score -= 10;
  }

  return { score: Math.max(0, score), issues, suggestions };
}

function generateRecommendations(results) {
  const recommendations = [];

  Object.entries(results).forEach(([category, result]) => {
    if (result.score < 80) {
      recommendations.push({
        category,
        priority: result.score < 60 ? 'high' : 'medium',
        message: `${category} validation scored ${result.score}/100`,
        improvements: result.suggestions
      });
    }
  });

  return recommendations;
}

async function convertLatexToHtml(latex, options) {
  // Basic LaTeX to HTML conversion
  let html = latex
    .replace(/\\documentclass.*?\\n/, '')
    .replace(/\\begin\{document\}/, '')
    .replace(/\\end\{document\}/, '')
    .replace(/\\title\{(.*?)\}/, '<h1>$1</h1>')
    .replace(/\\author\{(.*?)\}/, '<p><strong>Author:</strong> $1</p>')
    .replace(/\\section\{(.*?)\}/g, '<h2>$1</h2>')
    .replace(/\\subsection\{(.*?)\}/g, '<h3>$1</h3>')
    .replace(/\\emph\{(.*?)\}/g, '<em>$1</em>')
    .replace(/\\textbf\{(.*?)\}/g, '<strong>$1</strong>')
    .replace(/\\begin\{abstract\}/, '<div class="abstract"><p><strong>Abstract:</strong> ')
    .replace(/\\end\{abstract\}/, '</p></div>');

  return {
    content: html,
    metadata: {
      originalLength: latex.length,
      convertedLength: html.length,
      conversionRatio: Math.round((html.length / latex.length) * 100)
    },
    warnings: ['Basic conversion - complex LaTeX features may not be preserved']
  };
}

async function convertLatexToMarkdown(latex, options) {
  // Basic LaTeX to Markdown conversion
  let markdown = latex
    .replace(/\\documentclass.*?\\n/, '')
    .replace(/\\begin\{document\}/, '')
    .replace(/\\end\{document\}/, '')
    .replace(/\\title\{(.*?)\}/, '# $1\n\n')
    .replace(/\\author\{(.*?)\}/, '**Author:** $1\n\n')
    .replace(/\\section\{(.*?)\}/g, '## $1\n\n')
    .replace(/\\subsection\{(.*?)\}/g, '### $1\n\n')
    .replace(/\\emph\{(.*?)\}/g, '*$1*')
    .replace(/\\textbf\{(.*?)\}/g, '**$1**');

  return {
    content: markdown,
    metadata: {
      originalLength: latex.length,
      convertedLength: markdown.length
    },
    warnings: ['Basic conversion - complex LaTeX features may not be preserved']
  };
}

async function convertLatexToText(latex, options) {
  // Remove all LaTeX commands, keep only text content
  let text = latex
    .replace(/\\[a-zA-Z]+\{[^}]*\}/g, '') // Remove commands with arguments
    .replace(/\\[a-zA-Z]+/g, '') // Remove commands without arguments
    .replace(/[{}]/g, '') // Remove remaining braces
    .replace(/\\n/g, '\n') // Convert \\n to actual newlines
    .trim();

  return {
    content: text,
    metadata: {
      originalLength: latex.length,
      convertedLength: text.length,
      compressionRatio: Math.round((text.length / latex.length) * 100)
    },
    warnings: ['All formatting and structure lost in plain text conversion']
  };
}

function getTemplateDescription(templateType) {
  const descriptions = {
    'conference-paper': 'IEEE conference paper template with standard formatting',
    'journal-article': 'Academic journal article template with abstract and sections',
    'thesis': 'Comprehensive thesis template with chapters and front matter'
  };
  
  return descriptions[templateType] || 'Custom LaTeX template';
}

module.exports = router;