import React from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { AuthProvider } from './contexts/AuthContext';
import { ToastProvider } from './contexts/ToastContext';
import Layout from './components/Layout/Layout';
import ProtectedRoute from './components/Auth/ProtectedRoute';
import PublicRoute from './components/Auth/PublicRoute';

// Pages
import Login from './pages/Auth/Login';
import Dashboard from './pages/Dashboard/Dashboard';
import Courses from './pages/Courses/Courses';
import LMSIntegrations from './pages/LMS/LMSIntegrations';
import Assignments from './pages/Assignments/Assignments';
import Students from './pages/Students/Students';
import Analytics from './pages/Analytics/Analytics';

import './App.css';

function App() {
  return (
    <ToastProvider>
      <AuthProvider>
        <Router>
          <div className="App">
            <Routes>
              {/* Public routes */}
              <Route path="/login" element={
                <PublicRoute>
                  <Login />
                </PublicRoute>
              } />
              <Route path="/register" element={
                <PublicRoute>
                  <Register />
                </PublicRoute>
              } />
              
              {/* Protected routes */}
              <Route path="/" element={
                <ProtectedRoute>
                  <Layout />
                </ProtectedRoute>
              }>
                <Route index element={<Navigate to="/dashboard" replace />} />
                <Route path="dashboard" element={<Dashboard />} />
                
                {/* Course Management */}
                <Route path="courses" element={<Courses />} />
                <Route path="courses/create" element={
                  <ProtectedRoute allowedRoles={['instructor', 'administrator']}>
                    <CreateCourse />
                  </ProtectedRoute>
                } />
                <Route path="courses/:courseId" element={<CourseDetails />} />
                <Route path="courses/:courseId/edit" element={
                  <ProtectedRoute allowedRoles={['instructor', 'administrator']}>
                    <EditCourse />
                  </ProtectedRoute>
                } />
                
                {/* LMS Integration Management */}
                <Route path="lms" element={
                  <ProtectedRoute allowedRoles={['instructor', 'administrator']}>
                    <LMSIntegrations />
                  </ProtectedRoute>
                } />
                <Route path="lms/create" element={
                  <ProtectedRoute allowedRoles={['administrator']}>
                    <CreateIntegration />
                  </ProtectedRoute>
                } />
                <Route path="lms/:integrationId" element={<LMSDetails />} />
                
                {/* Assignment Management */}
                <Route path="assignments" element={<Assignments />} />
                <Route path="assignments/create" element={
                  <ProtectedRoute allowedRoles={['instructor', 'administrator']}>
                    <CreateAssignment />
                  </ProtectedRoute>
                } />
                <Route path="assignments/:assignmentId" element={<AssignmentDetails />} />
                
                {/* Student Management */}
                <Route path="students" element={
                  <ProtectedRoute allowedRoles={['instructor', 'administrator']}>
                    <Students />
                  </ProtectedRoute>
                } />
                
                {/* Analytics & Reporting */}
                <Route path="analytics" element={
                  <ProtectedRoute allowedRoles={['instructor', 'administrator']}>
                    <Analytics />
                  </ProtectedRoute>
                } />
                
                {/* Content Library */}
                <Route path="content" element={<ContentLibrary />} />
                
                {/* Collaboration */}
                <Route path="collaboration" element={<Collaboration />} />
                
                {/* User Management */}
                <Route path="profile" element={<Profile />} />
                <Route path="settings" element={
                  <ProtectedRoute allowedRoles={['administrator']}>
                    <Settings />
                  </ProtectedRoute>
                } />
              </Route>
              
              {/* 404 */}
              <Route path="*" element={<NotFound />} />
            </Routes>
          </div>
        </Router>
      </AuthProvider>
    </ToastProvider>
  );
}

export default App;