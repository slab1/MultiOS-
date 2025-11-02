const express = require('express');
const router = express.Router();

// Mock data stores (in production, these would be in a database)
let debugSessions = new Map();
let breakpoints = new Map();
let variables = new Map();
let watchExpressions = new Map();

// Breakpoint management
router.post('/breakpoint', async (req, res) => {
  try {
    const { file, line, condition, enabled = true } = req.body;
    
    if (!file || !line) {
      return res.status(400).json({ error: 'File and line number are required' });
    }
    
    const breakpointId = Date.now();
    const breakpoint = {
      id: breakpointId,
      file,
      line: parseInt(line),
      condition: condition || null,
      enabled,
      hitCount: 0,
      created: new Date().toISOString(),
      hits: []
    };
    
    breakpoints.set(breakpointId, breakpoint);
    
    res.json({
      success: true,
      breakpoint
    });
  } catch (error) {
    console.error('Error creating breakpoint:', error);
    res.status(500).json({ error: 'Failed to create breakpoint' });
  }
});

router.get('/breakpoints', async (req, res) => {
  try {
    const allBreakpoints = Array.from(breakpoints.values());
    res.json({
      breakpoints: allBreakpoints,
      count: allBreakpoints.length
    });
  } catch (error) {
    console.error('Error getting breakpoints:', error);
    res.status(500).json({ error: 'Failed to get breakpoints' });
  }
});

router.delete('/breakpoint/:id', async (req, res) => {
  try {
    const { id } = req.params;
    
    if (breakpoints.has(parseInt(id))) {
      breakpoints.delete(parseInt(id));
      res.json({ success: true, message: 'Breakpoint deleted' });
    } else {
      res.status(404).json({ error: 'Breakpoint not found' });
    }
  } catch (error) {
    console.error('Error deleting breakpoint:', error);
    res.status(500).json({ error: 'Failed to delete breakpoint' });
  }
});

// Debug session management
router.post('/session', async (req, res) => {
  try {
    const { program, args = [], workingDir } = req.body;
    
    if (!program) {
      return res.status(400).json({ error: 'Program path is required' });
    }
    
    const sessionId = Date.now().toString();
    const session = {
      id: sessionId,
      program,
      args,
      workingDir: workingDir || process.cwd(),
      status: 'created',
      created: new Date().toISOString(),
      pid: null,
      breakpoints: [],
      variables: [],
      callStack: []
    };
    
    debugSessions.set(sessionId, session);
    
    res.json({
      success: true,
      session
    });
  } catch (error) {
    console.error('Error creating debug session:', error);
    res.status(500).json({ error: 'Failed to create debug session' });
  }
});

router.get('/session/:id', async (req, res) => {
  try {
    const { id } = req.params;
    
    if (debugSessions.has(id)) {
      const session = debugSessions.get(id);
      res.json(session);
    } else {
      res.status(404).json({ error: 'Debug session not found' });
    }
  } catch (error) {
    console.error('Error getting debug session:', error);
    res.status(500).json({ error: 'Failed to get debug session' });
  }
});

router.post('/session/:id/start', async (req, res) => {
  try {
    const { id } = req.params;
    
    if (!debugSessions.has(id)) {
      return res.status(404).json({ error: 'Debug session not found' });
    }
    
    const session = debugSessions.get(id);
    session.status = 'running';
    session.started = new Date().toISOString();
    session.pid = Math.floor(Math.random() * 32768) + 1000; // Mock PID
    
    // Mock debug session started
    res.json({
      success: true,
      session,
      message: 'Debug session started'
    });
  } catch (error) {
    console.error('Error starting debug session:', error);
    res.status(500).json({ error: 'Failed to start debug session' });
  }
});

router.post('/session/:id/stop', async (req, res) => {
  try {
    const { id } = req.params;
    
    if (!debugSessions.has(id)) {
      return res.status(404).json({ error: 'Debug session not found' });
    }
    
    const session = debugSessions.get(id);
    session.status = 'stopped';
    session.stopped = new Date().toISOString();
    
    res.json({
      success: true,
      session,
      message: 'Debug session stopped'
    });
  } catch (error) {
    console.error('Error stopping debug session:', error);
    res.status(500).json({ error: 'Failed to stop debug session' });
  }
});

// Stepping commands
router.post('/session/:id/step', async (req, res) => {
  try {
    const { id } = req.params;
    const { type } = req.body; // 'into', 'over', 'out'
    
    if (!debugSessions.has(id)) {
      return res.status(404).json({ error: 'Debug session not found' });
    }
    
    const session = debugSessions.get(id);
    
    // Mock stepping
    const stepResult = {
      type,
      line: Math.floor(Math.random() * 100) + 1,
      function: 'mock_function',
      file: 'mock_file.c',
      timestamp: new Date().toISOString()
    };
    
    res.json({
      success: true,
      step: stepResult
    });
  } catch (error) {
    console.error('Error stepping:', error);
    res.status(500).json({ error: 'Failed to step' });
  }
});

router.post('/session/:id/continue', async (req, res) => {
  try {
    const { id } = req.params;
    
    if (!debugSessions.has(id)) {
      return res.status(404).json({ error: 'Debug session not found' });
    }
    
    const session = debugSessions.get(id);
    
    // Mock continue execution
    const continueResult = {
      status: 'running',
      currentLine: Math.floor(Math.random() * 100) + 1,
      timestamp: new Date().toISOString()
    };
    
    res.json({
      success: true,
      result: continueResult
    });
  } catch (error) {
    console.error('Error continuing execution:', error);
    res.status(500).json({ error: 'Failed to continue execution' });
  }
});

router.post('/session/:id/run-to-breakpoint', async (req, res) => {
  try {
    const { id } = req.params;
    
    if (!debugSessions.has(id)) {
      return res.status(404).json({ error: 'Debug session not found' });
    }
    
    const session = debugSessions.get(id);
    
    // Mock run to breakpoint
    const activeBreakpoints = Array.from(breakpoints.values()).filter(bp => bp.enabled);
    const nextBreakpoint = activeBreakpoints[0] || { line: 50, file: 'main.c' };
    
    const runResult = {
      hitBreakpoint: true,
      breakpoint: nextBreakpoint,
      currentLine: nextBreakpoint.line,
      timestamp: new Date().toISOString()
    };
    
    // Increment hit count
    nextBreakpoint.hitCount++;
    nextBreakpoint.hits.push({
      timestamp: new Date().toISOString(),
      sessionId: id
    });
    
    res.json({
      success: true,
      result: runResult
    });
  } catch (error) {
    console.error('Error running to breakpoint:', error);
    res.status(500).json({ error: 'Failed to run to breakpoint' });
  }
});

// Call stack
router.get('/session/:id/call-stack', async (req, res) => {
  try {
    const { id } = req.params;
    
    if (!debugSessions.has(id)) {
      return res.status(404).json({ error: 'Debug session not found' });
    }
    
    // Mock call stack
    const callStack = [
      {
        level: 0,
        function: 'main',
        file: 'main.c',
        line: 10,
        address: '0x55a1b4d2a000',
        args: []
      },
      {
        level: 1,
        function: 'add',
        file: 'main.c',
        line: 5,
        address: '0x55a1b4d2a010',
        args: ['int a=10', 'int b=20']
      },
      {
        level: 2,
        function: '__libc_start_main',
        file: 'crt1.o',
        line: 0,
        address: '0x7ffff7a2e1e1',
        args: []
      }
    ];
    
    res.json({
      success: true,
      callStack
    });
  } catch (error) {
    console.error('Error getting call stack:', error);
    res.status(500).json({ error: 'Failed to get call stack' });
  }
});

// Memory inspection
router.get('/session/:id/memory', async (req, res) => {
  try {
    const { id } = req.params;
    const { address, size, format = 'hex' } = req.query;
    
    if (!debugSessions.has(id)) {
      return res.status(404).json({ error: 'Debug session not found' });
    }
    
    // Mock memory content
    const mockMemory = {
      address: address || '0x7fff5fbff5bc',
      size: parseInt(size) || 16,
      format,
      content: generateMockMemoryContent(parseInt(size) || 16),
      timestamp: new Date().toISOString()
    };
    
    res.json({
      success: true,
      memory: mockMemory
    });
  } catch (error) {
    console.error('Error inspecting memory:', error);
    res.status(500).json({ error: 'Failed to inspect memory' });
  }
});

// Watch expressions
router.post('/watch', async (req, res) => {
  try {
    const { expression, sessionId } = req.body;
    
    if (!expression) {
      return res.status(400).json({ error: 'Expression is required' });
    }
    
    const watchId = Date.now();
    const watchExpression = {
      id: watchId,
      expression,
      sessionId: sessionId || null,
      created: new Date().toISOString(),
      value: null,
      type: 'unknown'
    };
    
    watchExpressions.set(watchId, watchExpression);
    
    res.json({
      success: true,
      watch: watchExpression
    });
  } catch (error) {
    console.error('Error creating watch expression:', error);
    res.status(500).json({ error: 'Failed to create watch expression' });
  }
});

router.get('/watch', async (req, res) => {
  try {
    const allWatches = Array.from(watchExpressions.values());
    res.json({
      watches: allWatches,
      count: allWatches.length
    });
  } catch (error) {
    console.error('Error getting watch expressions:', error);
    res.status(500).json({ error: 'Failed to get watch expressions' });
  }
});

router.delete('/watch/:id', async (req, res) => {
  try {
    const { id } = req.params;
    
    if (watchExpressions.has(parseInt(id))) {
      watchExpressions.delete(parseInt(id));
      res.json({ success: true, message: 'Watch expression deleted' });
    } else {
      res.status(404).json({ error: 'Watch expression not found' });
    }
  } catch (error) {
    console.error('Error deleting watch expression:', error);
    res.status(500).json({ error: 'Failed to delete watch expression' });
  }
});

// Evaluate expression
router.post('/evaluate', async (req, res) => {
  try {
    const { expression, sessionId } = req.body;
    
    if (!expression) {
      return res.status(400).json({ error: 'Expression is required' });
    }
    
    // Mock expression evaluation
    const mockEvaluations = {
      'x': 10,
      'y': 20,
      'result': 30,
      'x + y': 30,
      'global_counter': 100,
      'strlen(buffer)': 12
    };
    
    const value = mockEvaluations[expression] || `Cannot evaluate: ${expression}`;
    const type = typeof value;
    
    res.json({
      success: true,
      expression,
      value,
      type,
      timestamp: new Date().toISOString()
    });
  } catch (error) {
    console.error('Error evaluating expression:', error);
    res.status(500).json({ error: 'Failed to evaluate expression' });
  }
});

// Helper function to generate mock memory content
function generateMockMemoryContent(size) {
  const bytes = [];
  for (let i = 0; i < size; i++) {
    bytes.push(Math.floor(Math.random() * 256));
  }
  
  // Format as hex string
  let hexString = '';
  for (let i = 0; i < bytes.length; i++) {
    hexString += bytes[i].toString(16).padStart(2, '0');
    if (i < bytes.length - 1) {
      hexString += ' ';
    }
  }
  
  return hexString;
}

// Export all sessions (for integration)
router.get('/sessions/export', async (req, res) => {
  try {
    const sessions = Object.fromEntries(debugSessions);
    res.json({
      success: true,
      sessions,
      breakpoints: Object.fromEntries(breakpoints),
      variables: Object.fromEntries(variables),
      watchExpressions: Object.fromEntries(watchExpressions)
    });
  } catch (error) {
    console.error('Error exporting sessions:', error);
    res.status(500).json({ error: 'Failed to export sessions' });
  }
});

// Import sessions (for integration)
router.post('/sessions/import', async (req, res) => {
  try {
    const { sessions, breakpoints: bpData, variables: varData, watchExpressions: watchData } = req.body;
    
    if (sessions) {
      Object.entries(sessions).forEach(([id, session]) => {
        debugSessions.set(id, session);
      });
    }
    
    if (bpData) {
      Object.entries(bpData).forEach(([id, bp]) => {
        breakpoints.set(parseInt(id), bp);
      });
    }
    
    if (varData) {
      Object.entries(varData).forEach(([id, variable]) => {
        variables.set(id, variable);
      });
    }
    
    if (watchData) {
      Object.entries(watchData).forEach(([id, watch]) => {
        watchExpressions.set(parseInt(id), watch);
      });
    }
    
    res.json({
      success: true,
      message: 'Sessions imported successfully'
    });
  } catch (error) {
    console.error('Error importing sessions:', error);
    res.status(500).json({ error: 'Failed to import sessions' });
  }
});

module.exports = router;