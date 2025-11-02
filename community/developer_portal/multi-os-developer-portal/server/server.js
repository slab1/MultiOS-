import express from 'express';
import cors from 'cors';
import { createServer } from 'http';
import { Server } from 'socket.io';
import { VM } from 'vm2';
import { spawn } from 'child_process';
import { v4 as uuidv4 } from 'uuid';

const app = express();
const httpServer = createServer(app);
const io = new Server(httpServer, {
  cors: {
    origin: "*",
    methods: ["GET", "POST"]
  }
});

// Middleware
app.use(cors());
app.use(express.json());

// Store active coding sessions
const codingSessions = new Map();

// Code execution endpoints
app.post('/api/execute', async (req, res) => {
  try {
    const { code, language, sessionId } = req.body;
    
    if (!code || !language) {
      return res.status(400).json({ error: 'Code and language are required' });
    }

    let result;
    
    switch (language) {
      case 'javascript':
        result = await executeJavaScript(code);
        break;
      case 'python':
        result = await executePython(code);
        break;
      case 'rust':
        result = await executeRust(code);
        break;
      default:
        return res.status(400).json({ error: `Unsupported language: ${language}` });
    }

    res.json({
      success: true,
      output: result.output,
      error: result.error,
      executionTime: result.executionTime,
      sessionId
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

// Real-time collaboration
io.on('connection', (socket) => {
  console.log('User connected:', socket.id);

  // Join coding session
  socket.on('join-session', (sessionId) => {
    socket.join(sessionId);
    
    if (!codingSessions.has(sessionId)) {
      codingSessions.set(sessionId, {
        users: new Map(),
        code: '',
        language: 'javascript'
      });
    }
    
    const session = codingSessions.get(sessionId);
    session.users.set(socket.id, {
      id: socket.id,
      cursor: { line: 0, column: 0 },
      selection: null,
      username: `User${Object.keys(session.users).length + 1}`
    });
    
    // Send current session state to new user
    socket.emit('session-state', {
      code: session.code,
      language: session.language,
      users: Array.from(session.users.values())
    });
    
    // Notify others about new user
    socket.to(sessionId).emit('user-joined', {
      user: session.users.get(socket.id)
    });
  });

  // Handle code changes
  socket.on('code-change', (data) => {
    const { sessionId, code, changes } = data;
    const session = codingSessions.get(sessionId);
    
    if (session) {
      session.code = code;
      
      // Broadcast changes to all other users in the session
      socket.to(sessionId).emit('code-update', {
        code,
        changes,
        userId: socket.id
      });
    }
  });

  // Handle cursor position changes
  socket.on('cursor-change', (data) => {
    const { sessionId, cursor } = data;
    const session = codingSessions.get(sessionId);
    
    if (session && session.users.has(socket.id)) {
      const user = session.users.get(socket.id);
      user.cursor = cursor;
      
      // Broadcast cursor position to others
      socket.to(sessionId).emit('cursor-update', {
        userId: socket.id,
        cursor
      });
    }
  });

  // Handle language changes
  socket.on('language-change', (data) => {
    const { sessionId, language } = data;
    const session = codingSessions.get(sessionId);
    
    if (session) {
      session.language = language;
      
      // Broadcast language change
      io.to(sessionId).emit('language-update', {
        language,
        userId: socket.id
      });
    }
  });

  // Handle disconnection
  socket.on('disconnect', () => {
    console.log('User disconnected:', socket.id);
    
    // Remove user from all sessions
    codingSessions.forEach((session, sessionId) => {
      if (session.users.has(socket.id)) {
        session.users.delete(socket.id);
        
        // Notify others about user leaving
        socket.to(sessionId).emit('user-left', {
          userId: socket.id
        });
        
        // Clean up empty sessions
        if (session.users.size === 0) {
          codingSessions.delete(sessionId);
        }
      }
    });
  });
});

// JavaScript execution
async function executeJavaScript(code) {
  const startTime = Date.now();
  let output = '';
  let error = '';
  
  try {
    const vm = new VM({
      timeout: 10000,
      sandbox: {
        console: {
          log: (...args) => {
            output += args.map(arg => String(arg)).join(' ') + '\n';
          },
          error: (...args) => {
            error += args.map(arg => String(arg)).join(' ') + '\n';
          }
        }
      }
    });
    
    vm.run(code);
    
    const executionTime = Date.now() - startTime;
    return { output, error, executionTime };
  } catch (err) {
    const executionTime = Date.now() - startTime;
    return { 
      output: output, 
      error: err.message, 
      executionTime 
    };
  }
}

// Python execution (simulation)
async function executePython(code) {
  const startTime = Date.now();
  let output = '';
  let error = '';
  
  try {
    // For demo purposes, we'll simulate Python execution
    // In a real implementation, you'd use a secure Python execution environment
    const lines = code.split('\n');
    
    for (const line of lines) {
      if (line.trim().startsWith('print(')) {
        const match = line.match(/print\((.*)\)/);
        if (match) {
          const content = match[1].replace(/["']/g, '');
          output += content + '\n';
        }
      } else if (line.trim().startsWith('#')) {
        output += line.trim() + '\n';
      }
    }
    
    if (!output.trim()) {
      output += 'Python code executed successfully (simulated)\n';
    }
    
    const executionTime = Date.now() - startTime;
    return { output, error, executionTime };
  } catch (err) {
    const executionTime = Date.now() - startTime;
    return { 
      output: output, 
      error: err.message, 
      executionTime 
    };
  }
}

// Rust execution (simulation)
async function executeRust(code) {
  const startTime = Date.now();
  let output = '';
  let error = '';
  
  try {
    // For demo purposes, we'll simulate Rust compilation and execution
    // In a real implementation, you'd use a secure Rust compilation environment
    
    if (code.includes('println!')) {
      const matches = code.match(/println!\((.*?)\)/g);
      if (matches) {
        for (const match of matches) {
          const content = match.replace(/println!\(|\)/g, '').replace(/["']/g, '');
          output += content + '\n';
        }
      }
    }
    
    if (!output.trim()) {
      output += 'Rust code compiled successfully (simulated)\n';
      output += '🚀 Execution completed without errors\n';
    }
    
    const executionTime = Date.now() - startTime;
    return { output, error, executionTime };
  } catch (err) {
    const executionTime = Date.now() - startTime;
    return { 
      output: output, 
      error: err.message, 
      executionTime 
    };
  }
}

// API Routes
app.get('/api/health', (req, res) => {
  res.json({ 
    status: 'ok', 
    timestamp: new Date().toISOString(),
    uptime: process.uptime()
  });
});

// Get session information
app.get('/api/session/:sessionId', (req, res) => {
  const { sessionId } = req.params;
  const session = codingSessions.get(sessionId);
  
  if (!session) {
    return res.status(404).json({ error: 'Session not found' });
  }
  
  res.json({
    sessionId,
    code: session.code,
    language: session.language,
    users: Array.from(session.users.values())
  });
});

// Create new session
app.post('/api/session', (req, res) => {
  const sessionId = uuidv4();
  codingSessions.set(sessionId, {
    users: new Map(),
    code: '',
    language: 'javascript'
  });
  
  res.json({ sessionId });
});

// Project templates API
app.get('/api/templates', (req, res) => {
  const templates = [
    {
      id: 'web-app',
      name: 'Web Application',
      language: 'javascript',
      description: 'Modern web app template',
      code: `// Web Application Template
class MultiOSWebApp {
    constructor() {
        this.name = "MultiOS Web App";
        this.version = "1.0.0";
    }
    
    start() {
        console.log("Starting web application...");
        return "Application started successfully";
    }
}

const app = new MultiOSWebApp();
app.start();`
    },
    {
      id: 'cli-tool',
      name: 'CLI Tool',
      language: 'python',
      description: 'Command-line interface tool',
      code: `# CLI Tool Template
import sys

def main():
    print("MultiOS CLI Tool")
    print("=" * 20)
    
    if len(sys.argv) > 1:
        command = sys.argv[1]
        print(f"Executing command: {command}")
    else:
        print("No command provided")

if __name__ == "__main__":
    main()`
    },
    {
      id: 'rust-lib',
      name: 'Rust Library',
      language: 'rust',
      description: 'Rust library template',
      code: `//! MultiOS Rust Library Template

pub fn greet() -> String {
    "Hello from MultiOS Rust!".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_greet() {
        assert_eq!(greet(), "Hello from MultiOS Rust!");
    }
}`
    }
  ];
  
  res.json(templates);
});

// Serve static files in production
if (process.env.NODE_ENV === 'production') {
  app.use(express.static('../dist'));
  app.get('*', (req, res) => {
    res.sendFile('index.html', { root: '../dist' });
  });
}

const PORT = process.env.PORT || 3001;

httpServer.listen(PORT, () => {
  console.log(`MultiOS Developer Portal Server running on port ${PORT}`);
  console.log(`WebSocket server ready for real-time collaboration`);
});