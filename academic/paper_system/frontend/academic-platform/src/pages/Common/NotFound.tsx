import React from 'react';
import { Link } from 'react-router-dom';
import { Home, ArrowLeft } from 'lucide-react';

export default function NotFound() {
  return (
    <div className="min-h-screen bg-gray-50 flex flex-col justify-center py-12 sm:px-6 lg:px-8">
      <div className="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
        <div className="bg-white py-8 px-4 shadow sm:rounded-lg sm:px-10">
          <div className="text-center">
            <div className="mx-auto h-24 w-24 bg-gray-100 rounded-full flex items-center justify-center">
              <Home className="h-12 w-12 text-gray-400" />
            </div>
            <h2 className="mt-6 text-3xl font-bold text-gray-900">404</h2>
            <p className="mt-2 text-sm text-gray-600">
              Sorry, we couldn't find the page you're looking for.
            </p>
            <div className="mt-6">
              <Link
                to="/"
                className="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
              >
                <Home className="h-4 w-4 mr-2" />
                Go home
              </Link>
            </div>
            <div className="mt-4">
              <button
                onClick={() => window.history.back()}
                className="text-indigo-600 hover:text-indigo-500"
              >
                <ArrowLeft className="h-4 w-4 inline mr-1" />
                Go back
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}