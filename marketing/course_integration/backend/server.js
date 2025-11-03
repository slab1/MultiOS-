const express = require('express');
const cors = require('cors');
const helmet = require('helmet');
const compression = require('compression');
const rateLimit = require('express-rate-limit');
const { RateLimiterRedis } = require('rate-limiter-flexible');
const redis = require('redis');
const http = require('http');
const socketIo = require('socket.io');
require('dotenv').config();

const app = express();
const server = http.createServer(app);
const io = socketIo(server, {
  cors: {
    origin: process.env.FRONTEND_URL || "http://localhost:3000",
    methods: ["GET", "POST"]
  }
});

// Import route modules
const authRoutes = require('./routes/auth');
const userRoutes = require('./routes/users');
const courseRoutes = require('./routes/courses');
const lmsRoutes = require('./routes/lms');
const assignmentRoutes = require('./routes/assignments');
const collaborationRoutes = require('./routes/collaboration');
const analyticsRoutes = require('./routes/analytics');
const contentRoutes = require('./routes/content');
const notificationRoutes = require('./routes/notifications');

// Import middleware
const auth = require('./middleware/auth');
const logger = require('./middleware/logger');
const errorHandler = require('./middleware/errorHandler');

// Redis configuration
let redisClient;
if (process.env.REDIS_URL) {
  redisClient = redis.createClient({
    url: process.env.REDIS_URL
  });
  redisClient.on('error', (err) => console.log('Redis Client Error', err));
  redisClient.connect();
}

// Rate limiting configuration
const rateLimiter = new RateLimiterRedis({
  storeClient: redisClient,
  keyPrefix: 'rl',
  points: 100, // Number of requests
  duration: 60, // Per 60 seconds
});

// Basic rate limiting
const limiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100, // Limit each IP to 100 requests per windowMs
  message: 'Too many requests from this IP, please try again later.',
  standardHeaders: true,
  legacyHeaders: false,
});

// Middleware
app.use(helmet({
  contentSecurityPolicy: {
    directives: {
      defaultSrc: ["'self'"],
      styleSrc: ["'self'", "'unsafe-inline'"],
      scriptSrc: ["'self'"],
      imgSrc: ["'self'", "data:", "https:"],
      connectSrc: ["'self'", "wss:", "ws:"]
    }
  }
}));

app.use(cors({
  origin: process.env.FRONTEND_URL || "http://localhost:3000",
  credentials: true
}));

app.use(compression());
app.use(express.json({ limit: '10mb' }));
app.use(express.urlencoded({ extended: true, limit: '10mb' }));
app.use(limiter);

// Custom rate limiting middleware
const rateLimitMiddleware = async (req, res, next) => {
  if (!redisClient) return next();
  
  try {
    await rateLimiter.consume(req.ip);
    next();
  } catch {
    res.status(429).json({ error: 'Too many requests' });
  }
};

// Apply custom rate limiter
app.use(rateLimitMiddleware);

// Request logging
app.use(logger);

// Health check endpoint
app.get('/health', (req, res) => {
  res.status(200).json({
    status: 'OK',
    timestamp: new Date().toISOString(),
    version: process.env.npm_package_version || '1.0.0',
    uptime: process.uptime()
  });
});

// API Routes
app.use('/api/auth', authRoutes);
app.use('/api/users', auth, userRoutes);
app.use('/api/courses', auth, courseRoutes);
app.use('/api/lms', auth, lmsRoutes);
app.use('/api/assignments', auth, assignmentRoutes);
app.use('/api/collaboration', auth, collaborationRoutes);
app.use('/api/analytics', auth, analyticsRoutes);
app.use('/api/content', auth, contentRoutes);
app.use('/api/notifications', auth, notificationRoutes);

// Socket.IO for real-time features
io.use((socket, next) => {
  const token = socket.handshake.auth.token;
  if (!token) {
    return next(new Error('Authentication error'));
  }
  
  // Verify token and add user info to socket
  const jwt = require('jsonwebtoken');
  try {
    const decoded = jwt.verify(token, process.env.JWT_SECRET);
    socket.userId = decoded.userId;
    next();
  } catch (err) {
    next(new Error('Authentication error'));
  }
});

io.on('connection', (socket) => {
  console.log(`User ${socket.userId} connected`);
  
  // Join user-specific room for notifications
  socket.join(`user_${socket.userId}`);
  
  // Handle course room joining for real-time collaboration
  socket.on('join_course', (courseId) => {
    socket.join(`course_${courseId}`);
    socket.to(`course_${courseId}`).emit('user_joined', {
      userId: socket.userId,
      courseId
    });
  });
  
  // Handle real-time code collaboration
  socket.on('code_change', (data) => {
    socket.to(`course_${data.courseId}`).emit('code_updated', {
      ...data,
      userId: socket.userId
    });
  });
  
  // Handle discussion updates
  socket.on('new_post', (data) => {
    io.to(`course_${data.courseId}`).emit('discussion_updated', {
      ...data,
      userId: socket.userId
    });
  });
  
  socket.on('disconnect', () => {
    console.log(`User ${socket.userId} disconnected`);
  });
});

// Global error handler
app.use(errorHandler);

// 404 handler
app.use('*', (req, res) => {
  res.status(404).json({
    error: 'Route not found',
    path: req.originalUrl
  });
});

// Database connection
const db = require('./config/database');

const PORT = process.env.PORT || 5000;

server.listen(PORT, () => {
  console.log(`MultiOS Course Integration Platform running on port ${PORT}`);
  console.log(`Environment: ${process.env.NODE_ENV || 'development'}`);
  
  // Initialize LMS sync scheduler
  const scheduler = require('./services/scheduler');
  scheduler.start();
});

module.exports = app;