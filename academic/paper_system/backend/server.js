const express = require('express');
const mongoose = require('mongoose');
const cors = require('cors');
const helmet = require('helmet');
const compression = require('compression');
const rateLimit = require('express-rate-limit');
const cron = require('node-cron');
const winston = require('winston');
require('dotenv').config();

// Import routes
const authRoutes = require('./routes/auth');
const paperRoutes = require('./routes/papers');
const reviewRoutes = require('./routes/reviews');
const userRoutes = require('./routes/users');
const conferenceRoutes = require('./routes/conferences');
const citationRoutes = require('./routes/citations');
const analyticsRoutes = require('./routes/analytics');
const latexRoutes = require('./routes/latex');
const researchRoutes = require('./routes/research');

// Configure logger
const logger = winston.createLogger({
  level: 'info',
  format: winston.format.combine(
    winston.format.timestamp(),
    winston.format.errors({ stack: true }),
    winston.format.json()
  ),
  defaultMeta: { service: 'academic-paper-system' },
  transports: [
    new winston.transports.File({ filename: 'logs/error.log', level: 'error' }),
    new winston.transports.File({ filename: 'logs/combined.log' }),
    new winston.transports.Console({
      format: winston.format.simple()
    })
  ]
});

const app = express();
const PORT = process.env.PORT || 5000;

// Rate limiting
const limiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100, // limit each IP to 100 requests per windowMs
  message: 'Too many requests from this IP, please try again later.'
});

// Middleware
app.use(helmet());
app.use(compression());
app.use(limiter);
app.use(cors({
  origin: process.env.FRONTEND_URL || 'http://localhost:3000',
  credentials: true
}));
app.use(express.json({ limit: '50mb' }));
app.use(express.urlencoded({ extended: true, limit: '50mb' }));

// MongoDB connection
mongoose.connect(process.env.MONGODB_URI || 'mongodb://localhost:27017/academic_papers', {
  useNewUrlParser: true,
  useUnifiedTopology: true,
})
.then(() => logger.info('Connected to MongoDB'))
.catch((error) => logger.error('MongoDB connection error:', error));

// Routes
app.use('/api/auth', authRoutes);
app.use('/api/papers', paperRoutes);
app.use('/api/reviews', reviewRoutes);
app.use('/api/users', userRoutes);
app.use('/api/conferences', conferenceRoutes);
app.use('/api/citations', citationRoutes);
app.use('/api/analytics', analyticsRoutes);
app.use('/api/latex', latexRoutes);
app.use('/api/research', researchRoutes);

// Health check endpoint
app.get('/api/health', (req, res) => {
  res.json({ 
    status: 'OK', 
    timestamp: new Date().toISOString(),
    uptime: process.uptime()
  });
});

// Error handling middleware
app.use((err, req, res, next) => {
  logger.error('Unhandled error:', err.stack);
  res.status(500).json({ 
    error: 'Internal Server Error',
    message: process.env.NODE_ENV === 'development' ? err.message : 'Something went wrong'
  });
});

// 404 handler
app.use((req, res) => {
  res.status(404).json({ error: 'Route not found' });
});

// Cron jobs for automated tasks
// Daily analytics update
cron.schedule('0 2 * * *', async () => {
  logger.info('Running daily analytics update');
  // Implement analytics update logic here
});

// Weekly email reminders for reviews
cron.schedule('0 9 * * 1', async () => {
  logger.info('Sending weekly review reminders');
  // Implement email reminder logic here
});

// Start server
app.listen(PORT, () => {
  logger.info(`Server running on port ${PORT}`);
  console.log(`Academic Paper System API running on port ${PORT}`);
});

module.exports = app;