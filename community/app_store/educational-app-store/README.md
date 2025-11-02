# Educational App Store

A comprehensive platform for discovering, managing, and distributing educational applications. Built with React frontend and Express.js backend with PostgreSQL database.

## 🌟 Features

### For Students/Educators
- **Browse Apps**: Search and filter educational apps by category, grade level, and subject
- **App Details**: Comprehensive app pages with screenshots, reviews, and ratings
- **User Profiles**: Personalized dashboards and favorites
- **Reviews & Ratings**: Community-driven reviews and ratings system

### For Developers
- **App Submission**: Submit and manage educational applications
- **Developer Dashboard**: Track app performance and analytics
- **Content Management**: Upload app assets, screenshots, and documentation
- **Version Control**: Manage app updates and changelogs

### For Administrators
- **App Moderation**: Review and approve app submissions
- **User Management**: Manage developer and educator accounts
- **Analytics Dashboard**: Track platform usage and app performance
- **Content Curation**: Feature and promote high-quality educational apps

## 🛠 Tech Stack

### Frontend
- **React 18** with TypeScript
- **Vite** for build tooling
- **Tailwind CSS** for styling
- **React Router** for navigation
- **Radix UI** components
- **React Hook Form** for forms

### Backend
- **Express.js** with TypeScript
- **PostgreSQL** database
- **JWT** authentication
- **Multer** for file uploads
- **Winston** for logging

## 🚀 Quick Start

### Prerequisites
- Node.js 18+ and pnpm
- PostgreSQL 12+

### Installation

1. **Clone and setup:**
   ```bash
   git clone <repository-url>
   cd educational-app-store
   chmod +x setup.sh
   ./setup.sh
   ```

2. **Database setup:**
   ```bash
   cd backend
   npm run db:migrate  # Create database schema
   npm run db:seed     # Add sample data
   ```

3. **Start development servers:**
   
   Terminal 1 (Backend):
   ```bash
   cd backend
   npm run dev
   ```
   
   Terminal 2 (Frontend):
   ```bash
   npm run dev
   ```

4. **Access the application:**
   - Frontend: http://localhost:3000
   - Backend API: http://localhost:3001

### Default Accounts

**Admin Account:**
- Email: `admin@edustore.com`
- Password: `admin123`

**Developer Account:**
- Email: `developer@edustore.com`
- Password: `dev123`

## 📁 Project Structure

```
educational-app-store/
├── backend/                 # Express.js API server
│   ├── controllers/        # Route controllers
│   ├── database/          # Database schema and connection
│   ├── api/              # API routes
│   ├── utils/            # Authentication and utilities
│   ├── scripts/          # Database migration/seeding
│   └── server.ts         # Main server file
├── src/                   # React frontend
│   ├── components/       # Reusable UI components
│   ├── contexts/        # React contexts for state
│   ├── pages/           # Main application pages
│   ├── hooks/           # Custom React hooks
│   └── lib/             # Utility functions
└── public/              # Static assets
```

## 🚀 Deployment

### Backend Deployment
1. Build the application: `npm run build`
2. Set production environment variables
3. Run migrations: `npm run db:migrate`
4. Start server: `npm start`

### Frontend Deployment
1. Build for production: `npm run build`
2. Deploy `dist` folder to static hosting
3. Configure environment variables for API endpoint

---

Built with ❤️ for educators and students worldwide.
