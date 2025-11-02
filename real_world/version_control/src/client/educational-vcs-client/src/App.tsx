import React, { useState, useEffect } from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { Toaster } from 'react-hot-toast';
import Sidebar from './components/Sidebar';
import RepositoryView from './components/RepositoryView';
import CollaborativeEditor from './components/CollaborativeEditor';
import CodeReview from './components/CodeReview';
import AssignmentView from './components/AssignmentView';
import StudentDashboard from './components/StudentDashboard';
import InstructorDashboard from './components/InstructorDashboard';
import VersionControlTutorial from './components/VersionControlTutorial';
import QualityAnalysis from './components/QualityAnalysis';
import { VCSProvider } from './context/VCSContext';
import { io } from 'socket.io-client';
import './App.css';

const SOCKET_URL = process.env.REACT_APP_SOCKET_URL || 'http://localhost:5000';
const API_URL = process.env.REACT_APP_API_URL || 'http://localhost:5000/api';

function App() {
  const [socket, setSocket] = useState(null);
  const [currentUser, setCurrentUser] = useState(null);
  const [currentRepo, setCurrentRepo] = useState(null);

  useEffect(() => {
    // Initialize socket connection
    const newSocket = io(SOCKET_URL);
    setSocket(newSocket);

    // Load user from localStorage
    const savedUser = localStorage.getItem('vcs_user');
    if (savedUser) {
      setCurrentUser(JSON.parse(savedUser));
    } else {
      // Create a demo user for educational purposes
      const demoUser = {
        id: 'demo_user_' + Math.random().toString(36).substr(2, 9),
        name: 'Demo Student',
        email: 'student@demo.edu',
        role: 'student',
        avatar: '👨‍🎓'
      };
      setCurrentUser(demoUser);
      localStorage.setItem('vcs_user', JSON.stringify(demoUser));
    }

    return () => newSocket.close();
  }, []);

  const handleRepositoryChange = (repo) => {
    setCurrentRepo(repo);
  };

  return (
    <VCSProvider value={{ socket, currentUser, currentRepo, API_URL }}>
      <Router>
        <div className="min-h-screen bg-gray-50">
          <Toaster position="top-right" />
          
          <div className="flex h-screen">
            <Sidebar 
              currentRepo={currentRepo} 
              onRepoChange={handleRepositoryChange}
              currentUser={currentUser}
            />
            
            <main className="flex-1 overflow-hidden">
              <Routes>
                <Route path="/" element={<Navigate to="/dashboard" replace />} />
                
                <Route 
                  path="/dashboard" 
                  element={<StudentDashboard />} 
                />
                
                <Route 
                  path="/instructor" 
                  element={<InstructorDashboard />} 
                />
                
                <Route 
                  path="/repository/*" 
                  element={
                    currentRepo ? (
                      <RepositoryView currentRepo={currentRepo} />
                    ) : (
                      <div className="flex items-center justify-center h-full">
                        <div className="text-center">
                          <h2 className="text-2xl font-bold text-gray-700 mb-4">
                            Select or Create a Repository
                          </h2>
                          <p className="text-gray-500">
                            Choose a repository from the sidebar to get started
                          </p>
                        </div>
                      </div>
                    )
                  } 
                />
                
                <Route 
                  path="/editor/:filePath?" 
                  element={
                    <CollaborativeEditor 
                      socket={socket} 
                    />
                  } 
                />
                
                <Route 
                  path="/review/:reviewId?" 
                  element={<CodeReview />} 
                />
                
                <Route 
                  path="/assignments" 
                  element={<AssignmentView />} 
                />
                
                <Route 
                  path="/tutorial" 
                  element={<VersionControlTutorial />} 
                />
                
                <Route 
                  path="/quality" 
                  element={<QualityAnalysis />} 
                />
              </Routes>
            </main>
          </div>
        </div>
      </Router>
    </VCSProvider>
  );
}

export default App;
