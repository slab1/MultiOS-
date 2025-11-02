const express = require('express');
const cors = require('cors');
const helmet = require('helmet');
const rateLimit = require('express-rate-limit');
const path = require('path');
const fs = require('fs');

// Import routes
const debugRoutes = require('./routes/debug');
const scenariosRoutes = require('./routes/scenarios');
const memoryRoutes = require('./routes/memory');
const syscallRoutes = require('./routes/syscall');
const gdbRoutes = require('./routes/gdb');

const app = express();
const PORT = process.env.PORT || 8000;

// Middleware
app.use(helmet({
  contentSecurityPolicy: {
    directives: {
      defaultSrc: ["'self'"],
      styleSrc: ["'self'", "'unsafe-inline'"],
      scriptSrc: ["'self'", "'unsafe-inline'"],
      imgSrc: ["'self'", "data:", "https:"],
      connectSrc: ["'self'"],
      fontSrc: ["'self'"],
      objectSrc: ["'none'"],
      mediaSrc: ["'self'"],
      frameSrc: ["'none'"]
    }
  }
}));

app.use(cors({
  origin: ['http://localhost:3000', 'http://localhost:8080', 'http://localhost:8081'],
  credentials: true,
  methods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
  allowedHeaders: ['Content-Type', 'Authorization', 'X-Requested-With']
}));

app.use(express.json({ limit: '50mb' }));
app.use(express.urlencoded({ extended: true, limit: '50mb' }));

// Rate limiting
const limiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 1000, // limit each IP to 1000 requests per windowMs
  message: 'Too many requests from this IP, please try again later.',
  standardHeaders: true,
  legacyHeaders: false
});

app.use('/api', limiter);

// Health check endpoint
app.get('/api/health', (req, res) => {
  res.json({
    status: 'ok',
    timestamp: new Date().toISOString(),
    service: 'Interactive OS Debugging System',
    version: '1.0.0',
    uptime: process.uptime(),
    memory: process.memoryUsage()
  });
});

// Integration status endpoint
app.get('/api/status', (req, res) => {
  res.json({
    status: 'ok',
    integration: {
      available: true,
      version: '1.0.0',
      capabilities: [
        'ui_integration',
        'data_sync',
        'event_broadcast',
        'file_system_access',
        'process_control'
      ]
    },
    debugging: {
      gdb_available: true,
      memory_tracing: true,
      syscall_tracing: true,
      breakpoint_support: true
    }
  });
});

// MultiOS integration test endpoint
app.get('/api/integration/test', (req, res) => {
  res.json({
    status: 'ok',
    version: '1.0.0',
    capabilities: {
      ui_integration: true,
      data_sync: true,
      event_broadcast: true,
      remote_debugging: false,
      file_system_access: true,
      process_control: true
    },
    endpoints: {
      register: '/api/integration/register',
      unregister: '/api/integration/unregister',
      sync: '/api/integration/sync',
      settings: '/api/integration/settings'
    }
  });
});

// Static file serving for frontend
app.use(express.static(path.join(__dirname, '../frontend')));

// API Routes
app.use('/api/debug', debugRoutes);
app.use('/api/scenarios', scenariosRoutes);
app.use('/api/memory', memoryRoutes);
app.use('/api/syscall', syscallRoutes);
app.use('/api/gdb', gdbRoutes);

// Serve debug files
app.get('/api/debug/files', async (req, res) => {
  try {
    const filePath = req.query.path;
    
    if (!filePath) {
      return res.status(400).json({ error: 'File path is required' });
    }
    
    // In a real implementation, this would read from the actual filesystem
    // For demonstration, we'll return mock file content
    const content = await getMockFileContent(filePath);
    
    res.json({
      content,
      path: filePath,
      lastModified: new Date().toISOString(),
      size: content.length
    });
  } catch (error) {
    console.error('Error reading file:', error);
    res.status(500).json({ error: 'Failed to read file' });
  }
});

// Get system processes
app.get('/api/debug/processes', async (req, res) => {
  try {
    // In a real implementation, this would use system APIs
    const processes = [
      {
        id: 1234,
        name: 'debugged-program',
        state: 'Running',
        pid: 1234,
        parent: 1230,
        cpu: '25.3%',
        memory: '15.2 MB',
        user: 'debugger',
        startTime: new Date(Date.now() - 3600000).toISOString()
      },
      {
        id: 1230,
        name: 'bash',
        state: 'Running',
        pid: 1230,
        parent: 1200,
        cpu: '5.1%',
        memory: '8.1 MB',
        user: 'debugger',
        startTime: new Date(Date.now() - 7200000).toISOString()
      },
      {
        id: 1200,
        name: 'terminal',
        state: 'Running',
        pid: 1200,
        parent: 1,
        cpu: '2.3%',
        memory: '12.4 MB',
        user: 'debugger',
        startTime: new Date(Date.now() - 10800000).toISOString()
      }
    ];
    
    res.json(processes);
  } catch (error) {
    console.error('Error getting processes:', error);
    res.status(500).json({ error: 'Failed to get processes' });
  }
});

// Get threads for a process
app.get('/api/debug/processes/:pid/threads', async (req, res) => {
  try {
    const { pid } = req.params;
    
    // Mock thread data
    const threads = [
      {
        id: 1234,
        name: 'main thread',
        state: 'Running',
        priority: 0,
        cpu: '25.3%',
        stack: [
          { function: 'main', file: 'main.c', line: 10 },
          { function: '__libc_start_main', file: 'crt1.o', line: 0 }
        ]
      },
      {
        id: 1235,
        name: 'worker thread',
        state: 'Sleeping',
        priority: 0,
        cpu: '0.0%',
        stack: [
          { function: 'pthread_cond_wait', file: 'pthread_cond_wait.c', line: 78 },
          { function: 'worker_function', file: 'worker.c', line: 45 }
        ]
      }
    ];
    
    res.json(threads);
  } catch (error) {
    console.error('Error getting threads:', error);
    res.status(500).json({ error: 'Failed to get threads' });
  }
});

// Execute debug commands
app.post('/api/debug/commands', async (req, res) => {
  try {
    const { command } = req.body;
    
    if (!command) {
      return res.status(400).json({ error: 'Command is required' });
    }
    
    // Mock command execution
    const output = await executeMockCommand(command);
    
    res.json({
      command,
      output,
      exitCode: 0,
      timestamp: new Date().toISOString()
    });
  } catch (error) {
    console.error('Error executing command:', error);
    res.status(500).json({ error: 'Command execution failed', details: error.message });
  }
});

// Variable operations
app.get('/api/debug/variables', async (req, res) => {
  try {
    // Mock variable data
    const variables = [
      {
        name: 'x',
        type: 'int',
        value: 42,
        scope: 'local',
        line: 10,
        address: '0x7fff5fbff5bc',
        isPointer: false,
        isArray: false
      },
      {
        name: 'y',
        type: 'int',
        value: 84,
        scope: 'local',
        line: 11,
        address: '0x7fff5fbff5b8',
        isPointer: false,
        isArray: false
      },
      {
        name: 'result',
        type: 'int',
        value: 126,
        scope: 'local',
        line: 12,
        address: '0x7fff5fbff5b4',
        isPointer: false,
        isArray: false
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
        arrayLength: 12
      },
      {
        name: 'global_counter',
        type: 'int',
        value: 100,
        scope: 'global',
        line: 1,
        address: '0x601040',
        isPointer: false,
        isArray: false
      }
    ];
    
    res.json({ variables });
  } catch (error) {
    console.error('Error getting variables:', error);
    res.status(500).json({ error: 'Failed to get variables' });
  }
});

// Edit variable value
app.post('/api/debug/variables/edit', async (req, res) => {
  try {
    const { name, value } = req.body;
    
    if (!name) {
      return res.status(400).json({ error: 'Variable name is required' });
    }
    
    // Mock variable editing
    const updated = {
      name,
      value,
      timestamp: new Date().toISOString()
    };
    
    res.json({
      success: true,
      variable: updated
    });
  } catch (error) {
    console.error('Error editing variable:', error);
    res.status(500).json({ error: 'Failed to edit variable' });
  }
});

// Error handling middleware
app.use((err, req, res, next) => {
  console.error('Express error:', err);
  
  res.status(err.status || 500).json({
    error: err.message || 'Internal server error',
    stack: process.env.NODE_ENV === 'development' ? err.stack : undefined
  });
});

// 404 handler
app.use((req, res) => {
  res.status(404).json({
    error: 'Not found',
    path: req.path,
    method: req.method
  });
});

// Serve frontend for any other routes
app.get('*', (req, res) => {
  res.sendFile(path.join(__dirname, '../frontend/index.html'));
});

// Mock functions
async function getMockFileContent(filePath) {
  const mockFiles = {
    '/examples/main.c': `#include <stdio.h>
#include <stdlib.h>

int add(int a, int b) {
    return a + b;
}

int main() {
    int x = 10;
    int y = 20;
    int result = add(x, y);
    
    printf("Result: %d\\n", result);
    return 0;
}`,

    '/examples/hang_program.c': `#include <stdio.h>
#include <unistd.h>
#include <signal.h>

void timeout_handler(int sig) {
    printf("Timeout occurred!\\n");
    exit(1);
}

int main() {
    signal(SIGALRM, timeout_handler);
    alarm(3);
    
    printf("Starting program...\\n");
    printf("About to hang...\\n");
    while(1) {
        sleep(1);
    }
    
    printf("This should never be reached\\n");
    return 0;
}`,

    '/examples/memory_leak.c': `#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void allocate_memory() {
    char *buffer = malloc(1024);
    strcpy(buffer, "This memory will be leaked");
    printf("Allocated memory: %s\\n", buffer);
    // Missing: free(buffer);
}

int main() {
    printf("Memory leak demo\\n");
    
    for (int i = 0; i < 100; i++) {
        allocate_memory();
    }
    
    printf("Program completed\\n");
    return 0;
}`
  };
  
  return mockFiles[filePath] || `// Mock file content for ${filePath}
int main() {
    printf("Hello, Debug World!\\n");
    return 0;
}`;
}

async function executeMockCommand(command) {
  const commandMap = {
    'bt': `#0  main () at main.c:10
#1  0x00007ffff7a2e1e1 in __libc_start_main ()`,
    'info locals': `x = 10
y = 20
result = 30`,
    'info variables': `global_counter = 100`,
    'info threads': `  Id   Target Id         Frame 
* 1    Thread 0x7f8c (LWP 1234) main () at main.c:10`,
    'disassemble': `=> 0x55a1b4d2a000 <main+0>:    push   rbp
   0x55a1b4d2a001 <main+1>:    mov    rbp,rdi
   0x55a1b4d2a004 <main+4>:    mov    DWORD PTR [rbp-0x4],edi`,
    'info registers': `rax            0x0                 0
rbx            0x0                 0
rcx            0x55a1b4d2a000     140248095748864
rdx            0x0                 0`
  };
  
  const normalizedCommand = command.toLowerCase().trim();
  if (commandMap[normalizedCommand]) {
    return commandMap[normalizedCommand];
  }
  
  return `Mock output for: ${command}`;
}

// Start server
app.listen(PORT, () => {
  console.log(`
🛠️  Interactive OS Debugging System
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🚀 Server running on port ${PORT}
📁 Frontend: ${path.join(__dirname, '../frontend')}
🔗 API: http://localhost:${PORT}/api
🏥 Health: http://localhost:${PORT}/api/health
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
`);
  
  // Create necessary directories
  const directories = [
    path.join(__dirname, '../scenarios'),
    path.join(__dirname, '../assets'),
    path.join(__dirname, '../docs')
  ];
  
  directories.forEach(dir => {
    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true });
    }
  });
});

// Graceful shutdown
process.on('SIGTERM', () => {
  console.log('Received SIGTERM, shutting down gracefully');
  process.exit(0);
});

process.on('SIGINT', () => {
  console.log('Received SIGINT, shutting down gracefully');
  process.exit(0);
});

module.exports = app;