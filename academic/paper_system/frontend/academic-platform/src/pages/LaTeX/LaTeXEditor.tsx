import React, { useState, useEffect, useRef } from 'react';
import { latexAPI } from '../../services/api';
import { useNotifications } from '../../contexts/NotificationContext';
import LoadingSpinner from '../../components/Common/LoadingSpinner';
import {
  DocumentTextIcon,
  PlayIcon,
  DocumentArrowDownIcon,
  ClipboardIcon,
  CodeBracketIcon,
} from '@heroicons/react/24/outline';

interface LaTeXEditorProps {
  initialLatex?: string;
  onSave?: (latex: string) => void;
  paperId?: string;
  template?: string;
  readOnly?: boolean;
}

const LaTeXEditor: React.FC<LaTeXEditorProps> = ({
  initialLatex = '',
  onSave,
  paperId,
  template = 'acm',
  readOnly = false,
}) => {
  const [latex, setLatex] = useState(initialLatex);
  const [compiledPdf, setCompiledPdf] = useState<string | null>(null);
  const [isCompiling, setIsCompiling] = useState(false);
  const [compileError, setCompileError] = useState<string | null>(null);
  const [wordCount, setWordCount] = useState(0);
  const [showPreview, setShowPreview] = useState(false);
  const [activeTab, setActiveTab] = useState<'editor' | 'preview'>('editor');
  
  const editorRef = useRef<HTMLTextAreaElement>(null);
  const { showError, showSuccess } = useNotifications();

  // Load template on mount
  useEffect(() => {
    if (!initialLatex && template) {
      loadTemplate(template);
    }
  }, [template, initialLatex]);

  // Update word count
  useEffect(() => {
    const count = latex.replace(/\\cite\{[^}]*\}/g, '').replace(/[^\w\s]/g, '').split(/\s+/).filter(word => word.length > 0).length;
    setWordCount(count);
  }, [latex]);

  const loadTemplate = async (templateType: string) => {
    try {
      const response = await latexAPI.getTemplate(templateType);
      setLatex(response.data.template);
      showSuccess('Template loaded successfully');
    } catch (error) {
      showError('Failed to load template');
    }
  };

  const compileLatex = async () => {
    if (!latex.trim()) {
      showError('Please enter some LaTeX content to compile');
      return;
    }

    setIsCompiling(true);
    setCompileError(null);

    try {
      const response = await latexAPI.compile({
        latex,
        mainFile: 'main.tex',
        packages: ['amsmath', 'amssymb', 'graphicx', 'booktabs', 'natbib'],
        bibliographyEngine: 'bibtex',
      });

      setCompiledPdf(response.data.pdfBase64);
      showSuccess('LaTeX compiled successfully');
      setActiveTab('preview');
    } catch (error: any) {
      const errorMessage = error.response?.data?.error || 'Compilation failed';
      setCompileError(errorMessage);
      showError(errorMessage);
    } finally {
      setIsCompiling(false);
    }
  };

  const validateLatex = async () => {
    if (!paperId) {
      showError('Paper ID is required for validation');
      return;
    }

    try {
      const response = await latexAPI.validate({
        paperId,
        validationType: 'full',
        strictMode: true,
      });

      if (response.data.isValid) {
        showSuccess('LaTeX validation passed');
      } else {
        setCompileError(response.data.errors.join('\n'));
        showError('LaTeX validation failed');
      }
    } catch (error: any) {
      const errorMessage = error.response?.data?.error || 'Validation failed';
      showError(errorMessage);
    }
  };

  const convertFormat = async (targetFormat: 'html' | 'markdown') => {
    try {
      const response = await latexAPI.convert({
        latex,
        targetFormat,
        options: {
          includeBibliography: true,
          includeFigures: true,
        },
      });

      // Create download link
      const blob = new Blob([response.data.content], { 
        type: targetFormat === 'html' ? 'text/html' : 'text/markdown' 
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `converted.${targetFormat === 'html' ? 'html' : 'md'}`;
      a.click();
      URL.revokeObjectURL(url);
      
      showSuccess(`Converted to ${targetFormat} successfully`);
    } catch (error: any) {
      showError('Conversion failed');
    }
  };

  const copyToClipboard = async () => {
    try {
      await navigator.clipboard.writeText(latex);
      showSuccess('LaTeX copied to clipboard');
    } catch (error) {
      showError('Failed to copy to clipboard');
    }
  };

  const handleSave = () => {
    if (onSave) {
      onSave(latex);
      showSuccess('LaTeX saved successfully');
    }
  };

  const formatLatex = () => {
    // Basic LaTeX formatting
    let formatted = latex
      .replace(/\\begin\{document\}/, '\n\\begin{document}\n')
      .replace(/\\end\{document\}/, '\n\\end{document}\n')
      .replace(/\\section\{/, '\n\\section{')
      .replace(/\\subsection\{/, '\n\\subsection{')
      .replace(/\\subsubsection\{/, '\n\\subsubsection{')
      .replace(/\n\n+/g, '\n\n');

    setLatex(formatted);
  };

  const insertCommand = (command: string) => {
    if (!editorRef.current) return;
    
    const textarea = editorRef.current;
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const selectedText = latex.substring(start, end);
    
    let newText = latex;
    if (selectedText) {
      newText = latex.substring(0, start) + command.replace('%s', selectedText) + latex.substring(end);
    } else {
      newText = latex.substring(0, start) + command + latex.substring(end);
    }
    
    setLatex(newText);
    
    // Restore cursor position
    setTimeout(() => {
      textarea.focus();
      textarea.setSelectionRange(start + command.length, start + command.length);
    }, 0);
  };

  return (
    <div className="bg-white shadow-lg rounded-lg">
      {/* Header */}
      <div className="px-6 py-4 border-b border-gray-200">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <DocumentTextIcon className="h-6 w-6 text-indigo-600" />
            <h2 className="text-xl font-semibold text-gray-900">LaTeX Editor</h2>
          </div>
          
          <div className="flex items-center space-x-4">
            <div className="text-sm text-gray-500">
              Words: {wordCount}
            </div>
            <div className="flex items-center space-x-2">
              <button
                onClick={() => setActiveTab('editor')}
                className={`px-3 py-1 text-sm rounded ${
                  activeTab === 'editor'
                    ? 'bg-indigo-100 text-indigo-700'
                    : 'text-gray-500 hover:text-gray-700'
                }`}
              >
                Editor
              </button>
              <button
                onClick={() => setActiveTab('preview')}
                className={`px-3 py-1 text-sm rounded ${
                  activeTab === 'preview'
                    ? 'bg-indigo-100 text-indigo-700'
                    : 'text-gray-500 hover:text-gray-700'
                }`}
              >
                Preview
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Toolbar */}
      <div className="px-6 py-3 border-b border-gray-200 bg-gray-50">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-2">
            {!readOnly && (
              <>
                <button
                  onClick={compileLatex}
                  disabled={isCompiling}
                  className="inline-flex items-center px-3 py-1.5 border border-transparent text-sm font-medium rounded-md text-white bg-green-600 hover:bg-green-700 disabled:opacity-50"
                >
                  {isCompiling ? (
                    <LoadingSpinner size="sm" className="mr-1" />
                  ) : (
                    <PlayIcon className="h-4 w-4 mr-1" />
                  )}
                  Compile
                </button>
                
                <button
                  onClick={validateLatex}
                  className="inline-flex items-center px-3 py-1.5 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
                >
                  Validate
                </button>
                
                <button
                  onClick={formatLatex}
                  className="inline-flex items-center px-3 py-1.5 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
                >
                  Format
                </button>
              </>
            )}
            
            <div className="border-l border-gray-300 pl-2 ml-2">
              <select
                onChange={(e) => insertCommand(e.target.value)}
                className="text-sm border border-gray-300 rounded px-2 py-1"
                defaultValue=""
              >
                <option value="" disabled>Insert...</option>
                <option value="\\section{%s}">Section</option>
                <option value="\\subsection{%s}">Subsection</option>
                <option value="\\textbf{%s}">Bold</option>
                <option value="\\textit{%s}">Italic</option>
                <option value="\\cite{%s}">Citation</option>
                <option value="\\begin{figure}\n  \\centering\n  \\includegraphics[width=0.8\\textwidth]{%s}\n  \\caption{%s}\n  \\label{fig:%s}\n\\end{figure}">Figure</option>
                <option value="\\begin{table}\n  \\centering\n  \\caption{%s}\n  \\label{tab:%s}\n  \\begin{tabular}{cc}\n    %s\n  \\end{tabular}\n\\end{table}">Table</option>
              </select>
            </div>
          </div>
          
          <div className="flex items-center space-x-2">
            <button
              onClick={copyToClipboard}
              className="inline-flex items-center px-3 py-1.5 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
            >
              <ClipboardIcon className="h-4 w-4 mr-1" />
              Copy
            </button>
            
            <div className="relative">
              <select
                onChange={(e) => convertFormat(e.target.value as 'html' | 'markdown')}
                className="text-sm border border-gray-300 rounded px-2 py-1"
                defaultValue=""
              >
                <option value="" disabled>Convert...</option>
                <option value="html">To HTML</option>
                <option value="markdown">To Markdown</option>
              </select>
            </div>
            
            {!readOnly && (
              <button
                onClick={handleSave}
                className="inline-flex items-center px-3 py-1.5 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700"
              >
                Save
              </button>
            )}
          </div>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1">
        {activeTab === 'editor' ? (
          <div className="flex h-96">
            <div className="flex-1">
              <textarea
                ref={editorRef}
                value={latex}
                onChange={(e) => setLatex(e.target.value)}
                readOnly={readOnly}
                className="w-full h-full p-4 font-mono text-sm border-0 resize-none focus:ring-0"
                placeholder="Enter your LaTeX code here..."
                spellCheck={false}
              />
            </div>
            
            {/* Compile Error Display */}
            {compileError && (
              <div className="w-80 border-l border-gray-200 bg-red-50 p-4">
                <h3 className="text-sm font-medium text-red-800 mb-2">Compilation Error</h3>
                <pre className="text-xs text-red-700 whitespace-pre-wrap">{compileError}</pre>
              </div>
            )}
          </div>
        ) : (
          <div className="h-96 overflow-auto">
            {compiledPdf ? (
              <iframe
                src={`data:application/pdf;base64,${compiledPdf}`}
                className="w-full h-full border-0"
                title="Compiled PDF Preview"
              />
            ) : (
              <div className="flex items-center justify-center h-full text-gray-500">
                <div className="text-center">
                  <DocumentTextIcon className="h-12 w-12 mx-auto mb-4" />
                  <p>Compile your LaTeX to see the preview</p>
                </div>
              </div>
            )}
          </div>
        )}
      </div>
      
      {/* Footer */}
      <div className="px-6 py-3 border-t border-gray-200 bg-gray-50">
        <div className="flex items-center justify-between text-sm text-gray-500">
          <div className="flex items-center space-x-4">
            <span>Template: {template.toUpperCase()}</span>
            {compiledPdf && (
              <span className="text-green-600">✓ Compiled successfully</span>
            )}
          </div>
          
          <div className="flex items-center space-x-2">
            <CodeBracketIcon className="h-4 w-4" />
            <span>LaTeX Editor v1.0</span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default LaTeXEditor;