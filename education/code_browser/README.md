# MultiOS Interactive Code Browser

A comprehensive educational platform for exploring and understanding MultiOS kernel code with real-time explanations, performance analysis, and debugging integration.

## 🌟 Features

### 1. **Interactive Code Browser**
- **Syntax-highlighted code viewer** with support for Rust, C, C++, and Assembly
- **Real-time inline explanations** for complex code sections
- **Function call graph visualization** with dependency tracking
- **Variable inspection and data flow analysis**
- **Educational comments** and learning modules
- **Search and filter capabilities** with contextual help

### 2. **Performance Analysis**
- **Performance hotspot identification** and explanation
- **Optimization suggestions** with educational context
- **Real-time performance metrics** and monitoring
- **Cache analysis and optimization opportunities**
- **System resource usage tracking**

### 3. **Educational Components**
- **Progressive learning modules** from beginner to expert
- **Interactive code examples** with step-by-step explanations
- **Best practices guidance** and security awareness
- **Assessment and progress tracking**
- **Community features and discussions**

### 4. **Debug Integration**
- **Seamless debugging interface** with breakpoints and watchpoints
- **Variable state inspection** during execution
- **Call stack visualization**
- **Memory layout viewing**
- **Register state monitoring**

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Frontend (React + TypeScript)            │
├─────────────────────────────────────────────────────────────┤
│  Components:                                                 │
│  ├── CodeBrowser (Main Interface)                           │
│  ├── CodeViewer (Syntax Highlighting + Explanations)        │
│  ├── CallGraph (Function Call Visualization)               │
│  ├── VariableTracker (Data Flow Analysis)                  │
│  ├── PerformanceHotspots (Performance Analysis)            │
│  ├── PerformanceDashboard (Metrics & Monitoring)           │
│  ├── EducationalModules (Learning Content)                 │
│  ├── DebugInterface (Interactive Debugging)                │
│  └── CodeSearch (Advanced Search & Filtering)              │
└─────────────────────────────────────────────────────────────┘
                               │
                               │ HTTP/WebSocket
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                   Backend (Rust + Actix)                    │
├─────────────────────────────────────────────────────────────┤
│  API Endpoints:                                             │
│  ├── /api/v1/analyze/code (Code Analysis)                  │
│  ├── /api/v1/callgraph/* (Function Call Graph)             │
│  ├── /api/v1/performance/* (Performance Analysis)          │
│  ├── /api/v1/dataflow/* (Data Flow Analysis)               │
│  ├── /api/v1/search (Code Search)                          │
│  └── /api/v1/debug/* (Debug Integration)                   │
│                                                             │
│  Core Modules:                                              │
│  ├── CodeAnalyzer (Syntax Parsing & Analysis)              │
│  ├── CallGraphAnalyzer (Function Dependencies)             │
│  ├── PerformanceAnalyzer (Hotspot Detection)               │
│  └── DataFlowAnalyzer (Variable Tracking)                  │
└─────────────────────────────────────────────────────────────┘
                               │
                               │ File System
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                    MultiOS Source Code                      │
├─────────────────────────────────────────────────────────────┤
│  Kernel Modules:                                            │
│  ├── kernel/src/main.rs (Entry Point)                      │
│  ├── kernel/src/memory/* (Memory Management)               │
│  ├── kernel/src/scheduler/* (Process Scheduling)          │
│  ├── kernel/src/interrupts/* (Interrupt Handling)         │
│  ├── kernel/src/syscall/* (System Calls)                   │
│  ├── kernel/src/drivers/* (Device Drivers)                 │
│  └── libraries/* (Supporting Libraries)                    │
└─────────────────────────────────────────────────────────────┘
```

## 🚀 Quick Start

### Prerequisites

- **Rust** (latest stable version)
- **Node.js** (v16 or higher)
- **pnpm** (package manager)
- **Git** (for version control)

### Installation

1. **Clone the repository:**
```bash
git clone <repository-url>
cd education/code_browser
```

2. **Backend Setup:**
```bash
cd backend
cargo build --release
cargo run --bin code-browser-backend
```

3. **Frontend Setup:**
```bash
cd frontend/code-browser-frontend
pnpm install
pnpm run dev
```

4. **Access the application:**
- Frontend: http://localhost:5173
- Backend API: http://localhost:8080

## 📚 Usage Guide

### Code Browsing

1. **Open Code Browser** - Navigate to the main interface
2. **Select File** - Choose from the MultiOS kernel source files
3. **View Analysis** - Explore syntax highlighting, functions, and variables
4. **Get Explanations** - Hover over code for inline explanations
5. **Navigate Functions** - Click on function names to see definitions

### Call Graph Analysis

1. **Switch to Call Graph** tab
2. **Visualize Dependencies** - See function relationships
3. **Analyze Complexity** - Review performance impact indicators
4. **Educational Context** - Learn about kernel architecture patterns

### Performance Analysis

1. **Identify Hotspots** - Review performance issues detected
2. **Optimization Tips** - Follow guided optimization suggestions
3. **Educational Resources** - Learn performance optimization techniques
4. **Real-time Monitoring** - Track system performance metrics

### Data Flow Tracking

1. **Variable Selection** - Choose variables to track
2. **Flow Visualization** - See variable usage patterns
3. **Lifetime Analysis** - Understand variable scope and lifetime
4. **Security Analysis** - Identify potential security issues

### Debug Integration

1. **Start Debugging** - Launch interactive debugging session
2. **Set Breakpoints** - Add breakpoints to investigate code
3. **Variable Inspection** - Monitor variable values during execution
4. **Call Stack Navigation** - Navigate through function calls

### Educational Modules

1. **Browse Learning Paths** - Start with beginner concepts
2. **Follow Progressive Modules** - Build knowledge systematically
3. **Interactive Examples** - Practice with code samples
4. **Assessment** - Test understanding through exercises

## 🔧 Configuration

### Backend Configuration

Create `backend/.env`:
```env
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
RUST_LOG=debug
```

### Frontend Configuration

Create `frontend/code-browser-frontend/.env.local`:
```env
VITE_API_URL=http://localhost:8080
VITE_WS_URL=ws://localhost:8080
```

### Custom Analysis Rules

Add analysis rules in `backend/src/config/analysis_rules.rs`:
```rust
pub struct AnalysisRules {
    pub complexity_thresholds: ComplexityThresholds,
    pub performance_patterns: Vec<PerformancePattern>,
    pub educational_content: Vec<EducationalRule>,
}
```

## 🎓 Educational Framework

### Learning Paths

1. **Beginner Path (2-4 hours)**
   - Operating system concepts
   - Rust for systems programming
   - Kernel architecture basics
   - Memory management introduction

2. **Intermediate Path (6-8 hours)**
   - Process management
   - Interrupt handling
   - Device driver development
   - Synchronization mechanisms

3. **Advanced Path (10-12 hours)**
   - Performance optimization
   - Multicore architecture
   - Advanced scheduling
   - Security mechanisms

4. **Expert Path (15-20 hours)**
   - Kernel debugging
   - Performance profiling
   - Custom driver development
   - Research projects

### Assessment System

- **Knowledge Checks** - Regular comprehension tests
- **Code Analysis** - Practical code review exercises
- **Performance Projects** - Optimization challenges
- **Debug Scenarios** - Real-world debugging problems

## 🔍 API Reference

### Code Analysis

```http
POST /api/v1/analyze/code
Content-Type: application/json

{
  "code": "fn main() { println!(\"Hello\"); }",
  "language": "rust",
  "file_path": "kernel/src/main.rs"
}
```

### Call Graph Generation

```http
POST /api/v1/callgraph/generate
Content-Type: application/json

{
  "file_path": "kernel/src/main.rs",
  "function_name": "main",
  "depth_limit": 5
}
```

### Performance Analysis

```http
POST /api/v1/performance/hotspots
Content-Type: application/json

{
  "code": "...",
  "file_path": "kernel/src/scheduler/mod.rs"
}
```

### Data Flow Analysis

```http
POST /api/v1/dataflow/analyze
Content-Type: application/json

{
  "code": "...",
  "file_path": "kernel/src/main.rs",
  "variable_name": "current_task"
}
```

## 🛠️ Development

### Project Structure

```
code_browser/
├── backend/                 # Rust backend server
│   ├── src/
│   │   ├── main.rs         # Server entry point
│   │   ├── api.rs          # API endpoints
│   │   ├── code_analyzer.rs # Code analysis logic
│   │   ├── call_graph.rs   # Function call graph
│   │   ├── performance_analyzer.rs # Performance analysis
│   │   ├── data_flow.rs    # Data flow analysis
│   │   └── utils.rs        # Shared utilities
│   └── Cargo.toml
├── frontend/               # React frontend
│   └── code-browser-frontend/
│       ├── src/
│       │   ├── components/ # React components
│       │   ├── App.tsx     # Main application
│       │   └── main.tsx    # Entry point
│       └── package.json
├── docs/                  # Documentation
└── examples/              # Example code and tutorials
```

### Adding New Analysis

1. **Backend**: Extend analyzers in `backend/src/`
2. **Frontend**: Add components in `frontend/code-browser-frontend/src/components/`
3. **API**: Define endpoints in `backend/src/api.rs`
4. **Documentation**: Update this README and add examples

### Testing

```bash
# Backend tests
cd backend
cargo test

# Frontend tests
cd frontend/code-browser-frontend
pnpm test

# Integration tests
./scripts/run-integration-tests.sh
```

## 🤝 Contributing

### Contribution Guidelines

1. **Code Quality**: Follow Rust and React best practices
2. **Documentation**: Update relevant documentation
3. **Testing**: Add tests for new features
4. **Educational Value**: Ensure content is pedagogically sound

### Development Workflow

1. **Fork** the repository
2. **Create** a feature branch
3. **Implement** changes with tests
4. **Document** new functionality
5. **Submit** pull request

## 📖 Additional Resources

### Documentation
- [API Documentation](./docs/api.md)
- [Component Guide](./docs/components.md)
- [Educational Framework](./docs/education.md)
- [Performance Guide](./docs/performance.md)

### Examples
- [Basic Code Analysis](./examples/basic-analysis.rs)
- [Custom Performance Rules](./examples/custom-performance.rs)
- [Educational Module Template](./examples/module-template.md)

### Learning Materials
- [Operating Systems Concepts](https://ostep.org/)
- [Rust Systems Programming](https://doc.rust-lang.org/book/)
- [Performance Optimization Techniques](./docs/performance-optimization.md)

## 📊 Metrics & Monitoring

### Performance Metrics
- Code complexity scores
- Function call frequencies
- Performance hotspot trends
- Memory usage patterns

### Educational Metrics
- Module completion rates
- Learning path progression
- Assessment scores
- Engagement statistics

### System Health
- API response times
- Error rates
- Resource utilization
- User session metrics

## 🛡️ Security Considerations

### Code Analysis Safety
- No execution of analyzed code
- Sandboxed analysis environment
- Input validation and sanitization
- Rate limiting for API endpoints

### Educational Content Security
- Content validation
- User-generated content filtering
- Privacy protection for user data
- Secure authentication for advanced features

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- MultiOS community contributors
- Educational technology researchers
- Open source visualization libraries
- Systems programming educators

## 📞 Support

- **Documentation**: Check the docs folder
- **Issues**: Create GitHub issues for bugs
- **Discussions**: Use GitHub discussions for questions
- **Email**: Contact the maintainers

---

**Happy Learning! 🎓**

*Start exploring the MultiOS kernel with interactive, real-time explanations and comprehensive performance analysis.*
