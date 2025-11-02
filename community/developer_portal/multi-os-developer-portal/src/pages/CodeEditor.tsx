import React, { useState, useRef, useEffect } from 'react';
import Editor from '@monaco-editor/react';
import { 
  Play, 
  Square, 
  Save, 
  FolderOpen, 
  Settings, 
  Download,
  Upload,
  Share2,
  Copy,
  Check,
  Maximize2,
  Minimize2,
  Code,
  Terminal,
  FileText,
  Lightbulb,
  Users
} from 'lucide-react';

interface File {
  id: string;
  name: string;
  content: string;
  language: string;
}

export const CodeEditor: React.FC = () => {
  const [files, setFiles] = useState<File[]>([
    {
      id: '1',
      name: 'main.py',
      content: `# MultiOS Python Example
# Welcome to the MultiOS Developer Portal!

def greet_multiOS():
    print("Hello from MultiOS Developer Portal!")
    print("This is a Python code execution environment.")
    
    # Demonstrate basic Python features
    numbers = [1, 2, 3, 4, 5]
    squared_numbers = [x**2 for x in numbers]
    
    print(f"Original numbers: {numbers}")
    print(f"Squared numbers: {squared_numbers}")
    
    # Return a result
    return f"Processed {len(numbers)} numbers successfully"

if __name__ == "__main__":
    result = greet_multiOS()
    print(result)`,
      language: 'python'
    }
  ]);
  
  const [activeFile, setActiveFile] = useState('1');
  const [output, setOutput] = useState('');
  const [isExecuting, setIsExecuting] = useState(false);
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [showTemplates, setShowTemplates] = useState(false);
  const [language, setLanguage] = useState('python');
  const [theme, setTheme] = useState('vs-dark');
  const [copied, setCopied] = useState(false);
  const editorRef = useRef<any>(null);

  const editor = files.find(f => f.id === activeFile);

  const handleEditorDidMount = (editor: any, monaco: any) => {
    editorRef.current = editor;
    
    // Configure language support
    monaco.languages.typescript.typescriptDefaults.setCompilerOptions({
      target: monaco.languages.typescript.ScriptTarget.Latest,
      allowNonTsExtensions: true,
      moduleResolution: monaco.languages.typescript.ModuleResolutionKind.NodeJs,
      module: monaco.languages.typescript.ModuleKind.CommonJS,
      noEmit: true,
      esModuleInterop: true,
      jsx: monaco.languages.typescript.JsxEmit.React,
      reactNamespace: 'React',
      allowJs: true,
      typeRoots: ['node_modules/@types']
    });
  };

  const executeCode = async () => {
    if (!editorRef.current) return;
    
    setIsExecuting(true);
    setOutput('Executing code...\n\n');
    
    try {
      // Simulate code execution
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      if (editor?.language === 'python') {
        const result = executePython(editor.content);
        setOutput(result);
      } else if (editor?.language === 'javascript') {
        const result = executeJavaScript(editor.content);
        setOutput(result);
      } else if (editor?.language === 'rust') {
        const result = `Rust code execution simulation:
✅ Code compiled successfully
🚀 Execution completed

Note: In a real environment, this would be executed in a secure sandbox with actual Rust compiler.`;
        setOutput(result);
      } else {
        setOutput(`Language: ${editor?.language}\n✅ Code execution simulated successfully`);
      }
    } catch (error) {
      setOutput(`Error: ${error instanceof Error ? error.message : 'Unknown error occurred'}`);
    } finally {
      setIsExecuting(false);
    }
  };

  const executePython = (code: string): string => {
    try {
      // Simple Python execution simulation
      const lines = code.split('\n');
      let result = 'Python Execution Results:\n';
      result += '='.repeat(50) + '\n\n';
      
      lines.forEach((line, index) => {
        if (line.trim().startsWith('print(')) {
          const match = line.match(/print\((.*)\)/);
          if (match) {
            const content = match[1].replace(/["']/g, '');
            result += `> ${content}\n`;
          }
        } else if (line.trim().startsWith('#')) {
          result += `> ${line.trim()}\n`;
        }
      });
      
      result += '\n✅ Python code executed successfully!';
      return result;
    } catch (error) {
      return `❌ Python Execution Error: ${error instanceof Error ? error.message : 'Unknown error'}`;
    }
  };

  const executeJavaScript = (code: string): string => {
    try {
      // Simple JavaScript execution simulation
      const lines = code.split('\n');
      let result = 'JavaScript Execution Results:\n';
      result += '='.repeat(50) + '\n\n';
      
      lines.forEach((line) => {
        if (line.trim().startsWith('console.log(')) {
          const match = line.match(/console\.log\((.*)\)/);
          if (match) {
            const content = match[1].replace(/["']/g, '');
            result += `> ${content}\n`;
          }
        } else if (line.trim().startsWith('//')) {
          result += `> ${line.trim()}\n`;
        }
      });
      
      result += '\n✅ JavaScript code executed successfully!';
      return result;
    } catch (error) {
      return `❌ JavaScript Execution Error: ${error instanceof Error ? error.message : 'Unknown error'}`;
    }
  };

  const stopExecution = () => {
    setIsExecuting(false);
    setOutput(prev => prev + '\n⏹️ Execution stopped by user');
  };

  const saveCode = () => {
    if (!editorRef.current) return;
    
    const updatedFiles = files.map(file => 
      file.id === activeFile 
        ? { ...file, content: editorRef.current.getValue() }
        : file
    );
    setFiles(updatedFiles);
    
    // Show save confirmation
    setOutput(prev => prev + '\n💾 Code saved successfully!');
  };

  const addNewFile = () => {
    const newFile: File = {
      id: Date.now().toString(),
      name: `new_file.${language}`,
      content: getTemplateContent(language),
      language: language
    };
    setFiles([...files, newFile]);
    setActiveFile(newFile.id);
  };

  const getTemplateContent = (lang: string): string => {
    switch (lang) {
      case 'python':
        return `# New Python file
print("Hello from MultiOS!")

def main():
    # Your Python code here
    pass

if __name__ == "__main__":
    main()`;
      
      case 'javascript':
        return `// New JavaScript file
console.log("Hello from MultiOS!");

function main() {
    // Your JavaScript code here
}

main();`;
      
      case 'rust':
        return `// New Rust file
fn main() {
    println!("Hello from MultiOS!");
    
    // Your Rust code here
}`;
      
      default:
        return '// New file\n\n';
    }
  };

  const loadTemplate = (template: string) => {
    let templateContent = '';
    let templateLanguage = 'python';
    
    switch (template) {
      case 'web-app':
        templateLanguage = 'javascript';
        templateContent = `// Web Application Template
class MultiOSWebApp {
    constructor() {
        this.name = "MultiOS Web App";
        this.version = "1.0.0";
        console.log(\`\${this.name} v\${this.version} initialized\`);
    }
    
    start() {
        console.log("Starting web application...");
        return "Application started successfully";
    }
    
    render() {
        return {
            title: this.name,
            status: "running",
            timestamp: new Date().toISOString()
        };
    }
}

// Usage
const app = new MultiOSWebApp();
app.start();
console.log("Rendered:", app.render());`;
        break;
        
      case 'cli-tool':
        templateLanguage = 'python';
        templateContent = `#!/usr/bin/env python3
"""
MultiOS CLI Tool Template
"""

import sys
import argparse
from datetime import datetime

class MultiOSCLI:
    def __init__(self):
        self.name = "MultiOS CLI Tool"
        self.version = "1.0.0"
    
    def run(self, args):
        print(f"Running {self.name} v{self.version}")
        print(f"Command: {args.command}")
        print(f"Timestamp: {datetime.now().isoformat()}")
        
        if args.verbose:
            print("Verbose mode enabled")
        
        return True

def main():
    parser = argparse.ArgumentParser(description="MultiOS CLI Tool")
    parser.add_argument("command", help="Command to execute")
    parser.add_argument("--verbose", "-v", action="store_true", help="Enable verbose output")
    
    args = parser.parse_args()
    cli = MultiOSCLI()
    cli.run(args)

if __name__ == "__main__":
    main()`;
        break;
        
      case 'rust-lib':
        templateLanguage = 'rust';
        templateContent = `//! MultiOS Rust Library Template
//! 
//! This template demonstrates how to create a Rust library for MultiOS

/// Calculate the greatest common divisor (GCD) of two numbers
pub fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// Calculate the least common multiple (LCM) of two numbers
pub fn lcm(a: u64, b: u64) -> Option<u64> {
    if a == 0 || b == 0 {
        None
    } else {
        Some(a * b / gcd(a, b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(12, 8), 4);
        assert_eq!(gcd(17, 5), 1);
        assert_eq!(gcd(0, 5), 5);
    }

    #[test]
    fn test_lcm() {
        assert_eq!(lcm(12, 8), Some(24));
        assert_eq!(lcm(17, 5), Some(85));
        assert_eq!(lcm(0, 5), None);
    }
}`;
        break;
    }
    
    const newFile: File = {
      id: Date.now().toString(),
      name: `${template}.${templateLanguage === 'rust' ? 'rs' : templateLanguage}`,
      content: templateContent,
      language: templateLanguage
    };
    setFiles([...files, newFile]);
    setActiveFile(newFile.id);
    setShowTemplates(false);
  };

  const copyCode = () => {
    if (editorRef.current) {
      navigator.clipboard.writeText(editorRef.current.getValue());
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const downloadCode = () => {
    if (editorRef.current) {
      const content = editorRef.current.getValue();
      const blob = new Blob([content], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = editor?.name || 'code.txt';
      a.click();
      URL.revokeObjectURL(url);
    }
  };

  const deleteFile = (fileId: string) => {
    if (files.length === 1) return; // Don't delete the last file
    
    const updatedFiles = files.filter(f => f.id !== fileId);
    setFiles(updatedFiles);
    
    if (activeFile === fileId) {
      setActiveFile(updatedFiles[0]?.id || '');
    }
  };

  return (
    <div className={`${isFullscreen ? 'fixed inset-0 z-50 bg-slate-900' : 'h-screen'} flex flex-col`}>
      {/* Toolbar */}
      <div className="bg-slate-800 border-b border-slate-700 px-4 py-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-4">
            <div className="flex items-center space-x-2">
              <Code className="h-5 w-5 text-slate-400" />
              <span className="text-white font-medium">Code Editor</span>
            </div>
            
            <div className="flex items-center space-x-2">
              <select
                value={language}
                onChange={(e) => setLanguage(e.target.value)}
                className="bg-slate-700 text-white px-3 py-1 rounded border border-slate-600 text-sm"
              >
                <option value="python">Python</option>
                <option value="javascript">JavaScript</option>
                <option value="rust">Rust</option>
                <option value="typescript">TypeScript</option>
              </select>
              
              <select
                value={theme}
                onChange={(e) => setTheme(e.target.value)}
                className="bg-slate-700 text-white px-3 py-1 rounded border border-slate-600 text-sm"
              >
                <option value="vs-dark">Dark Theme</option>
                <option value="vs">Light Theme</option>
                <option value="hc-black">High Contrast</option>
              </select>
            </div>
          </div>
          
          <div className="flex items-center space-x-2">
            <button
              onClick={() => setShowTemplates(true)}
              className="flex items-center space-x-2 bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg transition-colors"
            >
              <Lightbulb className="h-4 w-4" />
              <span>Templates</span>
            </button>
            
            <button
              onClick={addNewFile}
              className="flex items-center space-x-2 bg-slate-600 hover:bg-slate-700 text-white px-4 py-2 rounded-lg transition-colors"
            >
              <FileText className="h-4 w-4" />
              <span>New File</span>
            </button>
            
            <button
              onClick={copyCode}
              className="flex items-center space-x-2 bg-slate-600 hover:bg-slate-700 text-white px-4 py-2 rounded-lg transition-colors"
            >
              {copied ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
            </button>
            
            <button
              onClick={downloadCode}
              className="flex items-center space-x-2 bg-slate-600 hover:bg-slate-700 text-white px-4 py-2 rounded-lg transition-colors"
            >
              <Download className="h-4 w-4" />
            </button>
            
            <button
              onClick={saveCode}
              className="flex items-center space-x-2 bg-green-600 hover:bg-green-700 text-white px-4 py-2 rounded-lg transition-colors"
            >
              <Save className="h-4 w-4" />
              <span>Save</span>
            </button>
            
            <button
              onClick={isExecuting ? stopExecution : executeCode}
              className={`flex items-center space-x-2 px-6 py-2 rounded-lg transition-colors ${
                isExecuting 
                  ? 'bg-red-600 hover:bg-red-700 text-white' 
                  : 'bg-blue-600 hover:bg-blue-700 text-white'
              }`}
            >
              {isExecuting ? <Square className="h-4 w-4" /> : <Play className="h-4 w-4" />}
              <span>{isExecuting ? 'Stop' : 'Run'}</span>
            </button>
            
            <button
              onClick={() => setIsFullscreen(!isFullscreen)}
              className="flex items-center space-x-2 bg-slate-600 hover:bg-slate-700 text-white px-4 py-2 rounded-lg transition-colors"
            >
              {isFullscreen ? <Minimize2 className="h-4 w-4" /> : <Maximize2 className="h-4 w-4" />}
            </button>
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 flex overflow-hidden">
        {/* File Tabs */}
        <div className="bg-slate-800 border-b border-slate-700 px-4 py-2 overflow-x-auto">
          <div className="flex space-x-2">
            {files.map((file) => (
              <div
                key={file.id}
                className={`flex items-center space-x-2 px-4 py-2 rounded-t-lg cursor-pointer transition-colors ${
                  activeFile === file.id
                    ? 'bg-slate-700 text-white'
                    : 'text-slate-400 hover:text-white hover:bg-slate-700'
                }`}
                onClick={() => setActiveFile(file.id)}
              >
                <span className="text-sm font-medium">{file.name}</span>
                {files.length > 1 && (
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      deleteFile(file.id);
                    }}
                    className="text-slate-400 hover:text-red-400 ml-2"
                  >
                    ×
                  </button>
                )}
              </div>
            ))}
          </div>
        </div>

        {/* Editor */}
        <div className="flex-1 flex">
          <div className="flex-1 border-r border-slate-700">
            <Editor
              height="100%"
              language={editor?.language || 'python'}
              theme={theme}
              value={editor?.content || ''}
              onChange={(value) => {
                if (value !== undefined) {
                  setFiles(files.map(file => 
                    file.id === activeFile 
                      ? { ...file, content: value }
                      : file
                  ));
                }
              }}
              onMount={handleEditorDidMount}
              options={{
                fontSize: 14,
                minimap: { enabled: true },
                scrollBeyondLastLine: false,
                wordWrap: 'on',
                tabSize: 2,
                insertSpaces: true,
                automaticLayout: true,
              }}
            />
          </div>

          {/* Output Panel */}
          <div className="w-1/3 bg-slate-900 text-white flex flex-col">
            <div className="bg-slate-800 px-4 py-3 border-b border-slate-700 flex items-center space-x-2">
              <Terminal className="h-4 w-4" />
              <span className="font-medium">Output</span>
            </div>
            <div className="flex-1 p-4 overflow-auto">
              <pre className="text-sm font-mono whitespace-pre-wrap text-slate-300">
                {output || 'Click "Run" to execute your code...'}
              </pre>
            </div>
          </div>
        </div>
      </div>

      {/* Templates Modal */}
      {showTemplates && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-full max-w-md mx-4">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold">Choose Template</h3>
              <button
                onClick={() => setShowTemplates(false)}
                className="text-slate-500 hover:text-slate-700"
              >
                ×
              </button>
            </div>
            
            <div className="space-y-3">
              <button
                onClick={() => loadTemplate('web-app')}
                className="w-full text-left p-3 rounded-lg border border-slate-200 hover:border-blue-300 hover:bg-blue-50 transition-colors"
              >
                <div className="font-medium text-slate-800">Web Application</div>
                <div className="text-sm text-slate-600">JavaScript web app template</div>
              </button>
              
              <button
                onClick={() => loadTemplate('cli-tool')}
                className="w-full text-left p-3 rounded-lg border border-slate-200 hover:border-blue-300 hover:bg-blue-50 transition-colors"
              >
                <div className="font-medium text-slate-800">CLI Tool</div>
                <div className="text-sm text-slate-600">Python command-line interface</div>
              </button>
              
              <button
                onClick={() => loadTemplate('rust-lib')}
                className="w-full text-left p-3 rounded-lg border border-slate-200 hover:border-blue-300 hover:bg-blue-50 transition-colors"
              >
                <div className="font-medium text-slate-800">Rust Library</div>
                <div className="text-sm text-slate-600">Rust library with tests</div>
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};