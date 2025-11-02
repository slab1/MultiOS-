import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { ErrorBoundary } from './components/ErrorBoundary';
import { Layout } from './components/Layout';
import { Home } from './pages/Home';
import { CodeEditor } from './pages/CodeEditor';
import { Templates } from './pages/Templates';
import { Tutorials } from './pages/Tutorials';
import { Community } from './pages/Community';
import { Resources } from './pages/Resources';
import { ProjectViewer } from './pages/ProjectViewer';
import './App.css';

function App() {
  return (
    <ErrorBoundary>
      <Router>
        <Layout>
          <Routes>
            <Route path="/" element={<Home />} />
            <Route path="/editor" element={<CodeEditor />} />
            <Route path="/templates" element={<Templates />} />
            <Route path="/tutorials" element={<Tutorials />} />
            <Route path="/community" element={<Community />} />
            <Route path="/resources" element={<Resources />} />
            <Route path="/project/:id" element={<ProjectViewer />} />
          </Routes>
        </Layout>
      </Router>
    </ErrorBoundary>
  );
}

export default App;
