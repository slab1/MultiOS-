const express = require('express');
const router = express.Router();

// Syscall tracking
let activeTraces = new Map();
let syscallHistory = new Map();
let syscallStats = new Map();

// Start system call tracing
router.post('/start', async (req, res) => {
  try {
    const { processId, filters = {}, breakpoints = [] } = req.body;
    
    if (!processId) {
      return res.status(400).json({ error: 'Process ID is required' });
    }
    
    const traceId = `${processId}_${Date.now()}`;
    const trace = {
      id: traceId,
      processId,
      status: 'active',
      started: new Date().toISOString(),
      filters,
      breakpoints,
      syscalls: [],
      stats: {
        totalCalls: 0,
        callsPerSecond: 0,
        mostFrequent: [],
        errorCount: 0
      }
    };
    
    activeTraces.set(traceId, trace);
    syscallHistory.set(processId, []);
    
    // Start mock tracing
    startMockTracing(traceId);
    
    res.json({
      success: true,
      traceId,
      message: 'System call tracing started'
    });
  } catch (error) {
    console.error('Error starting syscall trace:', error);
    res.status(500).json({ error: 'Failed to start system call tracing' });
  }
});

// Stop system call tracing
router.post('/stop', async (req, res) => {
  try {
    const { traceId } = req.body;
    
    if (!traceId) {
      return res.status(400).json({ error: 'Trace ID is required' });
    }
    
    const trace = activeTraces.get(traceId);
    if (!trace) {
      return res.status(404).json({ error: 'Trace not found' });
    }
    
    trace.status = 'stopped';
    trace.stopped = new Date().toISOString();
    
    // Move to history
    const history = syscallHistory.get(trace.processId) || [];
    history.push(...trace.syscalls);
    syscallHistory.set(trace.processId, history);
    
    res.json({
      success: true,
      trace,
      message: 'System call tracing stopped'
    });
  } catch (error) {
    console.error('Error stopping syscall trace:', error);
    res.status(500).json({ error: 'Failed to stop system call tracing' });
  }
});

// Get active traces
router.get('/active', async (req, res) => {
  try {
    const traces = Array.from(activeTraces.values()).filter(trace => trace.status === 'active');
    
    res.json({
      traces,
      count: traces.length
    });
  } catch (error) {
    console.error('Error getting active traces:', error);
    res.status(500).json({ error: 'Failed to get active traces' });
  }
});

// Get syscall trace for a process
router.get('/process/:pid', async (req, res) => {
  try {
    const { pid } = req.params;
    const { limit = 100, offset = 0 } = req.query;
    
    const history = syscallHistory.get(pid) || [];
    const recentSyscalls = history.slice(-parseInt(limit)).slice(parseInt(offset));
    
    res.json({
      processId: pid,
      syscalls: recentSyscalls,
      total: history.length,
      limit: parseInt(limit),
      offset: parseInt(offset)
    });
  } catch (error) {
    console.error('Error getting syscall trace:', error);
    res.status(500).json({ error: 'Failed to get syscall trace' });
  }
});

// Get syscall statistics
router.get('/stats/:pid', async (req, res) => {
  try {
    const { pid } = req.params;
    
    const history = syscallHistory.get(pid) || [];
    const stats = calculateSyscallStats(history);
    
    res.json({
      processId: pid,
      stats,
      timestamp: new Date().toISOString()
    });
  } catch (error) {
    console.error('Error getting syscall stats:', error);
    res.status(500).json({ error: 'Failed to get syscall statistics' });
  }
});

// Add syscall breakpoint
router.post('/breakpoint', async (req, res) => {
  try {
    const { syscallName, condition, traceId } = req.body;
    
    if (!syscallName) {
      return res.status(400).json({ error: 'System call name is required' });
    }
    
    const breakpoint = {
      id: Date.now(),
      syscallName,
      condition: condition || null,
      hitCount: 0,
      created: new Date().toISOString()
    };
    
    // Add to active trace if specified
    if (traceId) {
      const trace = activeTraces.get(traceId);
      if (trace) {
        trace.breakpoints.push(breakpoint);
      }
    }
    
    res.json({
      success: true,
      breakpoint
    });
  } catch (error) {
    console.error('Error adding syscall breakpoint:', error);
    res.status(500).json({ error: 'Failed to add syscall breakpoint' });
  }
});

// Remove syscall breakpoint
router.delete('/breakpoint/:id', async (req, res) => {
  try {
    const { id } = req.params;
    
    let removed = false;
    
    // Remove from active traces
    for (const [traceId, trace] of activeTraces) {
      const index = trace.breakpoints.findIndex(bp => bp.id === parseInt(id));
      if (index !== -1) {
        trace.breakpoints.splice(index, 1);
        removed = true;
        break;
      }
    }
    
    if (removed) {
      res.json({ success: true, message: 'Syscall breakpoint removed' });
    } else {
      res.status(404).json({ error: 'Syscall breakpoint not found' });
    }
  } catch (error) {
    console.error('Error removing syscall breakpoint:', error);
    res.status(500).json({ error: 'Failed to remove syscall breakpoint' });
  }
});

// Get syscall analysis
router.get('/analysis/:pid', async (req, res) => {
  try {
    const { pid } = req.params;
    
    const history = syscallHistory.get(pid) || [];
    const analysis = performSyscallAnalysis(history);
    
    res.json({
      processId: pid,
      analysis,
      timestamp: new Date().toISOString()
    });
  } catch (error) {
    console.error('Error getting syscall analysis:', error);
    res.status(500).json({ error: 'Failed to get syscall analysis' });
  }
});

// Search syscalls
router.get('/search/:pid', async (req, res) => {
  try {
    const { pid } = req.params;
    const { query, type = 'name', limit = 50 } = req.query;
    
    if (!query) {
      return res.status(400).json({ error: 'Search query is required' });
    }
    
    const history = syscallHistory.get(pid) || [];
    let results = [];
    
    switch (type) {
      case 'name':
        results = history.filter(sc => sc.name.toLowerCase().includes(query.toLowerCase()));
        break;
      case 'params':
        results = history.filter(sc => 
          JSON.stringify(sc.parameters).toLowerCase().includes(query.toLowerCase())
        );
        break;
      case 'result':
        results = history.filter(sc => String(sc.result).includes(query));
        break;
      default:
        results = history.filter(sc => 
          sc.name.toLowerCase().includes(query.toLowerCase()) ||
          JSON.stringify(sc.parameters).toLowerCase().includes(query.toLowerCase()) ||
          String(sc.result).includes(query)
        );
    }
    
    results = results.slice(0, parseInt(limit));
    
    res.json({
      processId: pid,
      query,
      type,
      results,
      total: results.length
    });
  } catch (error) {
    console.error('Error searching syscalls:', error);
    res.status(500).json({ error: 'Failed to search syscalls' });
  }
});

// Export syscall trace
router.get('/export/:pid', async (req, res) => {
  try {
    const { pid } = req.params;
    const { format = 'json' } = req.query;
    
    const history = syscallHistory.get(pid) || [];
    const stats = calculateSyscallStats(history);
    const analysis = performSyscallAnalysis(history);
    
    const exportData = {
      processId: pid,
      exported: new Date().toISOString(),
      format,
      stats,
      analysis,
      syscalls: history
    };
    
    if (format === 'json') {
      res.json(exportData);
    } else {
      res.status(400).json({ error: 'Unsupported export format' });
    }
  } catch (error) {
    console.error('Error exporting syscall trace:', error);
    res.status(500).json({ error: 'Failed to export syscall trace' });
  }
});

// Mock syscall data generation
function generateMockSyscall() {
  const syscalls = [
    {
      name: 'read',
      parameters: [getRandomFD(), generateBuffer(), getRandomSize()],
      result: Math.floor(Math.random() * 1024),
      duration: Math.random() * 50 + 1,
      error: Math.random() < 0.05 ? generateError() : null
    },
    {
      name: 'write',
      parameters: [getRandomFD(), generateBuffer(), getRandomSize()],
      result: Math.floor(Math.random() * 1024),
      duration: Math.random() * 30 + 1,
      error: null
    },
    {
      name: 'open',
      parameters: [generatePath(), getRandomFlags(), getRandomMode()],
      result: Math.floor(Math.random() * 1000),
      duration: Math.random() * 100 + 10,
      error: Math.random() < 0.1 ? generateError() : null
    },
    {
      name: 'close',
      parameters: [getRandomFD()],
      result: 0,
      duration: Math.random() * 20 + 1,
      error: null
    },
    {
      name: 'fork',
      parameters: [],
      result: Math.random() > 0.5 ? 0 : Math.floor(Math.random() * 32768),
      duration: Math.random() * 200 + 50,
      error: null
    },
    {
      name: 'execve',
      parameters: [generatePath(), generateArgs(), generateEnvp()],
      result: Math.random() > 0.8 ? -1 : 0,
      duration: Math.random() * 1000 + 100,
      error: Math.random() < 0.05 ? generateError() : null
    },
    {
      name: 'mmap',
      parameters: [0, getRandomSize(), getRandomProt(), getRandomFlags(), getRandomFD(), getRandomOffset()],
      result: '0x' + Math.floor(Math.random() * 0x100000000).toString(16),
      duration: Math.random() * 100 + 20,
      error: Math.random() < 0.03 ? generateError() : null
    },
    {
      name: 'socket',
      parameters: [getRandomDomain(), getRandomType(), getRandomProtocol()],
      result: Math.floor(Math.random() * 1000),
      duration: Math.random() * 50 + 10,
      error: null
    },
    {
      name: 'connect',
      parameters: [getRandomFD(), generateSockaddr(), 16],
      result: Math.random() > 0.7 ? -1 : 0,
      duration: Math.random() * 200 + 50,
      error: Math.random() < 0.1 ? generateError() : null
    },
    {
      name: 'gettimeofday',
      parameters: [generateTimeval(), null],
      result: 0,
      duration: Math.random() * 10 + 1,
      error: null
    }
  ];
  
  const syscall = syscalls[Math.floor(Math.random() * syscalls.length)];
  
  return {
    id: Date.now() + Math.random(),
    name: syscall.name,
    timestamp: new Date(),
    parameters: syscall.parameters,
    result: syscall.result,
    duration: syscall.duration,
    error: syscall.error,
    address: '0x' + Math.floor(Math.random() * 0x100000000).toString(16)
  };
}

function startMockTracing(traceId) {
  const interval = setInterval(() => {
    const trace = activeTraces.get(traceId);
    if (!trace || trace.status !== 'active') {
      clearInterval(interval);
      return;
    }
    
    // Generate mock syscall
    const syscall = generateMockSyscall();
    
    // Add to trace
    trace.syscalls.push(syscall);
    trace.stats.totalCalls++;
    
    // Check for breakpoints
    const hitBreakpoint = trace.breakpoints.find(bp => 
      bp.syscallName === syscall.name && 
      (!bp.condition || evaluateCondition(bp.condition, syscall))
    );
    
    if (hitBreakpoint) {
      hitBreakpoint.hitCount++;
      trace.status = 'stopped';
      clearInterval(interval);
      
      // Log breakpoint hit
      console.log(`Breakpoint hit: ${syscall.name} (trace: ${traceId})`);
    }
    
    // Limit trace size
    if (trace.syscalls.length > 10000) {
      trace.syscalls = trace.syscalls.slice(-5000);
    }
  }, 100 + Math.random() * 200);
  
  // Store interval for cleanup
  if (!activeTraces.has(traceId + '_interval')) {
    activeTraces.set(traceId + '_interval', interval);
  }
}

function calculateSyscallStats(syscalls) {
  const stats = {
    totalCalls: syscalls.length,
    uniqueSyscalls: new Set(syscalls.map(sc => sc.name)).size,
    errorCount: syscalls.filter(sc => sc.error).length,
    avgDuration: 0,
    totalDuration: 0,
    mostFrequent: [],
    slowest: [],
    errorRate: 0
  };
  
  if (syscalls.length > 0) {
    stats.totalDuration = syscalls.reduce((sum, sc) => sum + sc.duration, 0);
    stats.avgDuration = stats.totalDuration / syscalls.length;
    stats.errorRate = (stats.errorCount / syscalls.length) * 100;
    
    // Most frequent syscalls
    const syscallCounts = {};
    syscalls.forEach(sc => {
      syscallCounts[sc.name] = (syscallCounts[sc.name] || 0) + 1;
    });
    
    stats.mostFrequent = Object.entries(syscallCounts)
      .sort(([,a], [,b]) => b - a)
      .slice(0, 5)
      .map(([name, count]) => ({ name, count }));
    
    // Slowest syscalls
    stats.slowest = syscalls
      .filter(sc => sc.duration != null)
      .sort((a, b) => b.duration - a.duration)
      .slice(0, 5)
      .map(sc => ({ name: sc.name, duration: sc.duration, timestamp: sc.timestamp }));
  }
  
  return stats;
}

function performSyscallAnalysis(syscalls) {
  const analysis = {
    patterns: detectPatterns(syscalls),
    anomalies: detectAnomalies(syscalls),
    performance: analyzePerformance(syscalls),
    security: analyzeSecurity(syscalls),
    recommendations: generateRecommendations(syscalls)
  };
  
  return analysis;
}

function detectPatterns(syscalls) {
  const patterns = [];
  
  // Pattern: Rapid succession of similar syscalls
  const recentSyscalls = syscalls.slice(-20);
  const syscallGroups = {};
  recentSyscalls.forEach(sc => {
    syscallGroups[sc.name] = (syscallGroups[sc.name] || 0) + 1;
  });
  
  Object.entries(syscallGroups).forEach(([name, count]) => {
    if (count > 3) {
      patterns.push({
        type: 'high_frequency',
        syscall: name,
        count,
        timeframe: '20 syscalls',
        description: `High frequency of ${name} syscalls detected`
      });
    }
  });
  
  // Pattern: Error bursts
  const errorBursts = detectErrorBursts(syscalls);
  if (errorBursts.length > 0) {
    patterns.push(...errorBursts);
  }
  
  return patterns;
}

function detectAnomalies(syscalls) {
  const anomalies = [];
  
  // Anomaly: Long duration syscalls
  const durationThreshold = 1000; // 1 second
  const longSyscalls = syscalls.filter(sc => sc.duration > durationThreshold);
  
  if (longSyscalls.length > 0) {
    anomalies.push({
      type: 'slow_syscalls',
      count: longSyscalls.length,
      threshold: durationThreshold,
      description: 'Unusually slow system calls detected',
      examples: longSyscalls.slice(0, 3)
    });
  }
  
  // Anomaly: High error rate
  const errorRate = (syscalls.filter(sc => sc.error).length / syscalls.length) * 100;
  if (errorRate > 10) {
    anomalies.push({
      type: 'high_error_rate',
      rate: errorRate,
      threshold: 10,
      description: 'High error rate in system calls'
    });
  }
  
  // Anomaly: Suspicious syscalls
  const suspiciousSyscalls = syscalls.filter(sc => 
    ['ptrace', 'process_vm_readv', 'process_vm_writev', 'kill'].includes(sc.name)
  );
  
  if (suspiciousSyscalls.length > 0) {
    anomalies.push({
      type: 'suspicious_syscalls',
      count: suspiciousSyscalls.length,
      syscalls: suspiciousSyscalls.map(sc => sc.name),
      description: 'Potentially suspicious system calls detected'
    });
  }
  
  return anomalies;
}

function analyzePerformance(syscalls) {
  const performance = {
    totalTime: syscalls.reduce((sum, sc) => sum + sc.duration, 0),
    avgTime: syscalls.length > 0 ? syscalls.reduce((sum, sc) => sum + sc.duration, 0) / syscalls.length : 0,
    bottlenecks: [],
    efficiency: 'good'
  };
  
  // Find bottlenecks
  const syscallTimes = {};
  syscalls.forEach(sc => {
    syscallTimes[sc.name] = (syscallTimes[sc.name] || 0) + sc.duration;
  });
  
  Object.entries(syscallTimes).forEach(([name, totalTime]) => {
    const count = syscalls.filter(sc => sc.name === name).length;
    const avgTime = totalTime / count;
    
    if (avgTime > 100) { // Average > 100ms
      performance.bottlenecks.push({
        syscall: name,
        totalTime,
        avgTime,
        count
      });
    }
  });
  
  performance.bottlenecks.sort((a, b) => b.avgTime - a.avgTime);
  
  if (performance.bottlenecks.length > 0) {
    performance.efficiency = 'poor';
  }
  
  return performance;
}

function analyzeSecurity(syscalls) {
  const security = {
    threats: [],
    permissions: {},
    networkActivity: false,
    fileOperations: []
  };
  
  // Check for security-relevant syscalls
  const securityRelevant = ['execve', 'ptrace', 'kill', 'chmod', 'mknod', 'mount'];
  
  syscalls.forEach(sc => {
    if (securityRelevant.includes(sc.name)) {
      security.threats.push({
        syscall: sc.name,
        parameters: sc.parameters,
        timestamp: sc.timestamp,
        risk: calculateRisk(sc)
      });
    }
    
    // Track file operations
    if (['open', 'openat', 'unlink', 'rename'].includes(sc.name)) {
      security.fileOperations.push({
        operation: sc.name,
        path: sc.parameters[0],
        timestamp: sc.timestamp
      });
    }
    
    // Track network activity
    if (['socket', 'connect', 'send', 'recv'].includes(sc.name)) {
      security.networkActivity = true;
    }
  });
  
  return security;
}

function generateRecommendations(syscalls) {
  const recommendations = [];
  
  // Performance recommendations
  const stats = calculateSyscallStats(syscalls);
  if (stats.avgDuration > 100) {
    recommendations.push({
      type: 'performance',
      priority: 'high',
      description: 'High average syscall duration detected',
      action: 'Investigate slow system calls and optimize I/O operations'
    });
  }
  
  // Error rate recommendations
  if (stats.errorRate > 5) {
    recommendations.push({
      type: 'reliability',
      priority: 'medium',
      description: 'High error rate in system calls',
      action: 'Review error handling and input validation'
    });
  }
  
  // Security recommendations
  const execveCount = syscalls.filter(sc => sc.name === 'execve').length;
  if (execveCount > 10) {
    recommendations.push({
      type: 'security',
      priority: 'medium',
      description: 'High number of execve calls detected',
      action: 'Review execve usage for potential security risks'
    });
  }
  
  return recommendations;
}

// Helper functions for generating mock data
function getRandomFD() {
  return Math.floor(Math.random() * 1000);
}

function generateBuffer() {
  return '0x' + Math.floor(Math.random() * 0xFFFFFFFF).toString(16);
}

function getRandomSize() {
  const sizes = [512, 1024, 2048, 4096, 8192];
  return sizes[Math.floor(Math.random() * sizes.length)];
}

function generatePath() {
  const paths = [
    '/tmp/file.txt',
    '/home/user/document.pdf',
    '/var/log/system.log',
    '/dev/null',
    '/proc/self/maps',
    '/etc/passwd',
    '/lib/libc.so.6',
    '/bin/bash'
  ];
  return paths[Math.floor(Math.random() * paths.length)];
}

function getRandomFlags() {
  const flags = [0, 1, 2, 64, 512, 577, 1089];
  return flags[Math.floor(Math.random() * flags.length)];
}

function getRandomMode() {
  return Math.floor(Math.random() * 0x1FF);
}

function generateArgs() {
  return '["arg1", "arg2", "arg3"]';
}

function generateEnvp() {
  return '["PATH=/usr/bin", "HOME=/home/user"]';
}

function getRandomProt() {
  const prots = [0, 1, 2, 3, 4, 5, 6, 7];
  return prots[Math.floor(Math.random() * prots.length)];
}

function getRandomFlags() {
  const flags = [0, 1, 2, 32, 33, 34, 35];
  return flags[Math.floor(Math.random() * flags.length)];
}

function getRandomOffset() {
  return Math.floor(Math.random() * 0x1000);
}

function getRandomDomain() {
  return Math.floor(Math.random() * 3) + 1; // 1-3
}

function getRandomType() {
  return Math.floor(Math.random() * 3) + 1; // 1-3
}

function getRandomProtocol() {
  return Math.floor(Math.random() * 3) + 1; // 1-3
}

function generateSockaddr() {
  return 'struct sockaddr_in @ 0x' + Math.floor(Math.random() * 0xFFFFFFFF).toString(16);
}

function generateTimeval() {
  return `struct timeval {${Math.floor(Math.random() * 1000000)}, ${Math.floor(Math.random() * 1000000)}}`;
}

function generateError() {
  const errors = ['EACCES', 'ENOENT', 'EIO', 'ENOMEM', 'EBUSY', 'EEXIST', 'EINTR', 'EAGAIN'];
  return errors[Math.floor(Math.random() * errors.length)];
}

function evaluateCondition(condition, syscall) {
  // Simple condition evaluation - in reality this would be more sophisticated
  return condition.includes('name') ? condition.includes(syscall.name) : true;
}

function detectErrorBursts(syscalls) {
  const bursts = [];
  const recentErrors = syscalls.slice(-50).filter(sc => sc.error);
  
  if (recentErrors.length > 5) {
    bursts.push({
      type: 'error_burst',
      count: recentErrors.length,
      timeframe: 'last 50 syscalls',
      description: 'Burst of errors detected'
    });
  }
  
  return bursts;
}

function calculateRisk(syscall) {
  let risk = 'low';
  
  if (['execve', 'ptrace'].includes(syscall.name)) {
    risk = 'high';
  } else if (['kill', 'chmod'].includes(syscall.name)) {
    risk = 'medium';
  }
  
  return risk;
}

module.exports = router;