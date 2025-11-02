# MultiOS Developer Portal

A comprehensive developer portal with online code editor, real-time execution environment, project templates, interactive tutorials, and community features for MultiOS development.

## 🚀 Features

### 💻 Online Code Editor
- **Monaco Editor Integration**: Full-featured code editor with syntax highlighting
- **Multi-language Support**: Python, JavaScript, TypeScript, Rust, and more
- **Real-time Execution**: Instant code execution with output display
- **Intelligent Features**: Auto-completion, error detection, and code suggestions
- **Multiple Themes**: Dark, light, and high contrast themes

### 🛠️ Development Tools
- **Project Templates**: Ready-to-use starter kits and boilerplates
- **Real-time Collaboration**: Multi-user coding sessions with live updates
- **Version Control Integration**: Git integration for project management
- **Code Sharing**: Easy project sharing and collaboration
- **Export Options**: Download projects as ZIP files

### 📚 Learning Resources
- **Interactive Tutorials**: Step-by-step learning with embedded coding exercises
- **Comprehensive Documentation**: Detailed guides and API references
- **Learning Paths**: Structured curriculum for different skill levels
- **Progress Tracking**: Track your learning progress and achievements
- **Skill Assessments**: Interactive quizzes and coding challenges

### 👥 Community Features
- **Project Showcase**: Share and discover community projects
- **Developer Profiles**: Showcase your work and skills
- **Community Discussions**: Forums and chat for collaboration
- **Achievement System**: Gamified learning with badges and rewards
- **Mentorship**: Connect with experienced developers

### 🔧 Technical Stack
- **Frontend**: React 18, TypeScript, Tailwind CSS, Vite
- **Backend**: Node.js, Express.js, Socket.io
- **Code Editor**: Monaco Editor (VS Code editor)
- **Real-time**: WebSocket communication
- **Database**: PostgreSQL (for production), SQLite (for development)
- **Deployment**: Docker containers, CI/CD pipelines

## 🏃‍♂️ Quick Start

### Prerequisites
- Node.js 18+ 
- pnpm (recommended) or npm
- Python 3.8+ (for backend services)
- Git

### Installation

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd developer_portal/multi-os-developer-portal
   ```

2. **Install dependencies**
   ```bash
   # Frontend dependencies
   pnpm install
   
   # Backend dependencies
   cd server
   pnpm install
   cd ..
   ```

3. **Start the development servers**

   **Backend Server** (Terminal 1):
   ```bash
   cd server
   pnpm dev
   # Server runs on http://localhost:3001
   ```

   **Frontend Development** (Terminal 2):
   ```bash
   pnpm dev
   # Frontend runs on http://localhost:5173
   ```

4. **Open your browser**
   Navigate to `http://localhost:5173` to access the developer portal.

## 📁 Project Structure

```
developer_portal/
├── multi-os-developer-portal/     # Frontend React application
│   ├── src/
│   │   ├── components/            # Reusable UI components
│   │   ├── pages/                # Page components
│   │   ├── hooks/                # Custom React hooks
│   │   └── lib/                  # Utility functions
│   ├── public/                   # Static assets
│   └── server/                   # Backend server
├── docs/                         # Documentation
└── README.md
```

## 🎯 Core Components

### Code Editor (`/src/pages/CodeEditor.tsx`)
- Monaco editor integration
- Real-time syntax highlighting
- Multi-language support
- Code execution environment
- Template system

### Templates (`/src/pages/Templates.tsx`)
- Project template gallery
- Filter and search functionality
- One-click template creation
- Category-based organization

### Tutorials (`/src/pages/Tutorials.tsx`)
- Interactive learning system
- Progress tracking
- Embedded coding exercises
- Multi-level difficulty

### Community (`/src/pages/Community.tsx`)
- Project showcase
- Developer profiles
- Community discussions
- Achievement system

### Resources (`/src/pages/Resources.tsx`)
- Documentation library
- Learning paths
- Tool recommendations
- External resources

## 🔧 Backend API

### REST Endpoints

#### Code Execution
```http
POST /api/execute
Content-Type: application/json

{
  "code": "print('Hello, MultiOS!')",
  "language": "python",
  "sessionId": "session-123"
}
```

#### Session Management
```http
POST /api/session
# Returns: { "sessionId": "uuid" }

GET /api/session/:sessionId
# Returns session state and user information
```

#### Templates
```http
GET /api/templates
# Returns: Array of available templates
```

### WebSocket Events

#### Real-time Collaboration
```javascript
// Join coding session
socket.emit('join-session', sessionId);

// Handle code changes
socket.emit('code-change', { sessionId, code, changes });

// Handle cursor updates
socket.emit('cursor-change', { sessionId, cursor });

// Language changes
socket.emit('language-change', { sessionId, language });
```

## 🌐 Deployment

### Frontend Deployment (Vercel/Netlify)

1. **Build the frontend**
   ```bash
   pnpm build
   ```

2. **Deploy to Vercel**
   ```bash
   npx vercel --prod
   ```

3. **Deploy to Netlify**
   - Connect your repository
   - Set build command: `pnpm build`
   - Set publish directory: `dist`

### Backend Deployment (Railway/Heroku)

1. **Prepare the server**
   ```bash
   cd server
   npm install
   ```

2. **Set environment variables**
   ```bash
   NODE_ENV=production
   PORT=3001
   DATABASE_URL=postgresql://...
   ```

3. **Deploy to Railway**
   ```bash
   railway login
   railway deploy
   ```

### Docker Deployment

1. **Build Docker images**
   ```bash
   # Frontend
   docker build -t multios-frontend .
   
   # Backend
   cd server
   docker build -t multios-backend .
   ```

2. **Run with Docker Compose**
   ```bash
   docker-compose up -d
   ```

## 🔒 Security Features

- **Sandboxed Code Execution**: VM2 for secure JavaScript execution
- **Input Validation**: Comprehensive input sanitization
- **Rate Limiting**: API rate limiting to prevent abuse
- **CORS Protection**: Proper CORS configuration
- **Authentication**: JWT-based authentication system
- **Content Security Policy**: CSP headers for XSS protection

## 🧪 Testing

### Frontend Testing
```bash
# Unit tests
pnpm test

# E2E tests
pnpm test:e2e

# Coverage report
pnpm test:coverage
```

### Backend Testing
```bash
cd server
npm test
```

## 📈 Performance

- **Code Splitting**: Automatic route-based code splitting
- **Lazy Loading**: Components and routes loaded on demand
- **Caching**: Redis caching for API responses
- **CDN**: Static assets served via CDN
- **Minification**: Production builds with minification

## 🤝 Contributing

1. **Fork the repository**
2. **Create a feature branch**
   ```bash
   git checkout -b feature/amazing-feature
   ```
3. **Commit your changes**
   ```bash
   git commit -m 'Add amazing feature'
   ```
4. **Push to the branch**
   ```bash
   git push origin feature/amazing-feature
   ```
5. **Open a Pull Request**

### Development Guidelines

- Follow TypeScript best practices
- Use ESLint and Prettier for code formatting
- Write tests for new features
- Update documentation as needed
- Follow semantic versioning

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

- **Documentation**: [docs.multios.dev](https://docs.multios.dev)
- **Community Discord**: [discord.gg/multios](https://discord.gg/multios)
- **GitHub Issues**: [Report bugs and feature requests](https://github.com/multios/dev-portal/issues)
- **Email Support**: support@multios.dev

## 🎉 Acknowledgments

- [Monaco Editor](https://microsoft.github.io/monaco-editor/) for the code editing experience
- [React](https://reactjs.org/) for the frontend framework
- [Tailwind CSS](https://tailwindcss.com/) for styling
- [Socket.io](https://socket.io/) for real-time communication
- [Express.js](https://expressjs.com/) for the backend framework

---

**Built with ❤️ for the MultiOS developer community**
