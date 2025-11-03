import React, { useState, useEffect } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { toast } from 'react-hot-toast';
import {
  ArrowLeft,
  Edit,
  Download,
  Upload,
  FileText,
  Users,
  Calendar,
  Tag,
  MessageCircle,
  CheckCircle,
  XCircle,
  Clock,
  Eye,
  BookOpen
} from 'lucide-react';

import LoadingSpinner from '../Common/LoadingSpinner';
import { papersAPI } from '../../services/api';

interface Paper {
  _id: string;
  title: string;
  abstract: string;
  authors: Array<{
    name: string;
    email: string;
    affiliation: string;
  }>;
  status: 'draft' | 'submitted' | 'under_review' | 'accepted' | 'rejected';
  tags: string[];
  submissionDate: string;
  publicationDate?: string;
  conferenceId?: string;
  keywords: string[];
  references: string[];
  fileUrl?: string;
  latexContent?: string;
  createdAt: string;
  updatedAt: string;
  reviewHistory?: Array<{
    reviewerId: string;
    reviewerName: string;
    status: string;
    submittedAt: string;
    comments: string;
  }>;
}

export default function PaperDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [paper, setPaper] = useState<Paper | null>(null);
  const [loading, setLoading] = useState(true);
  const [uploading, setUploading] = useState(false);

  useEffect(() => {
    if (id) {
      fetchPaper();
    }
  }, [id]);

  const fetchPaper = async () => {
    try {
      setLoading(true);
      const response = await papersAPI.get(`/papers/${id}`);
      setPaper(response.data);
    } catch (error: any) {
      toast.error(error.message || 'Failed to fetch paper');
      navigate('/papers');
    } finally {
      setLoading(false);
    }
  };

  const handleFileUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    if (file.type !== 'application/pdf') {
      toast.error('Please upload a PDF file');
      return;
    }

    if (file.size > 10 * 1024 * 1024) { // 10MB
      toast.error('File size must be less than 10MB');
      return;
    }

    setUploading(true);
    try {
      const formData = new FormData();
      formData.append('file', file);
      
      await papersAPI.post(`/papers/${id}/upload`, formData, {
        headers: {
          'Content-Type': 'multipart/form-data',
        },
      });
      
      toast.success('File uploaded successfully');
      fetchPaper(); // Refresh paper data
    } catch (error: any) {
      toast.error(error.message || 'Failed to upload file');
    } finally {
      setUploading(false);
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'draft':
        return <Edit className="h-4 w-4" />;
      case 'submitted':
        return <Upload className="h-4 w-4" />;
      case 'under_review':
        return <Clock className="h-4 w-4" />;
      case 'accepted':
        return <CheckCircle className="h-4 w-4 text-green-600" />;
      case 'rejected':
        return <XCircle className="h-4 w-4 text-red-600" />;
      default:
        return <FileText className="h-4 w-4" />;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'draft': return 'bg-gray-100 text-gray-800';
      case 'submitted': return 'bg-blue-100 text-blue-800';
      case 'under_review': return 'bg-yellow-100 text-yellow-800';
      case 'accepted': return 'bg-green-100 text-green-800';
      case 'rejected': return 'bg-red-100 text-red-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric'
    });
  };

  if (loading) {
    return (
      <div className="flex justify-center py-12">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (!paper) {
    return (
      <div className="text-center py-12">
        <h3 className="text-lg font-medium text-gray-900">Paper not found</h3>
        <p className="text-gray-500 mt-2">The paper you're looking for doesn't exist.</p>
        <Link
          to="/papers"
          className="mt-4 inline-flex items-center text-indigo-600 hover:text-indigo-500"
        >
          <ArrowLeft className="h-4 w-4 mr-1" />
          Back to papers
        </Link>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-white shadow rounded-lg">
        <div className="px-6 py-4 border-b border-gray-200">
          <div className="flex items-center justify-between">
            <div className="flex items-center">
              <Link
                to="/papers"
                className="mr-4 text-gray-400 hover:text-gray-600"
              >
                <ArrowLeft className="h-5 w-5" />
              </Link>
              <div>
                <h1 className="text-2xl font-semibold text-gray-900">{paper.title}</h1>
                <div className="flex items-center mt-2 space-x-4">
                  <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(paper.status)}`}>
                    {getStatusIcon(paper.status)}
                    <span className="ml-1 capitalize">{paper.status.replace('_', ' ')}</span>
                  </span>
                  <span className="text-sm text-gray-500">
                    Created {formatDate(paper.createdAt)}
                  </span>
                  <span className="text-sm text-gray-500">
                    Updated {formatDate(paper.updatedAt)}
                  </span>
                </div>
              </div>
            </div>
            <div className="flex space-x-3">
              <Link
                to={`/papers/${id}/edit`}
                className="inline-flex items-center px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
              >
                <Edit className="h-4 w-4 mr-2" />
                Edit
              </Link>
              {paper.fileUrl && (
                <a
                  href={paper.fileUrl}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center px-4 py-2 border border-transparent rounded-md text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                >
                  <Download className="h-4 w-4 mr-2" />
                  Download
                </a>
              )}
            </div>
          </div>
        </div>

        {/* Authors */}
        <div className="px-6 py-4">
          <h3 className="text-lg font-medium text-gray-900 mb-3">Authors</h3>
          <div className="space-y-3">
            {paper.authors.map((author, index) => (
              <div key={index} className="flex items-center p-3 bg-gray-50 rounded-lg">
                <div className="flex-shrink-0">
                  <div className="h-10 w-10 bg-indigo-100 rounded-full flex items-center justify-center">
                    <Users className="h-5 w-5 text-indigo-600" />
                  </div>
                </div>
                <div className="ml-4">
                  <div className="text-sm font-medium text-gray-900">{author.name}</div>
                  <div className="text-sm text-gray-500">{author.email}</div>
                  <div className="text-sm text-gray-500">{author.affiliation}</div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Paper Details */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Main Content */}
        <div className="lg:col-span-2 space-y-6">
          {/* Abstract */}
          <div className="bg-white shadow rounded-lg">
            <div className="px-6 py-4 border-b border-gray-200">
              <h3 className="text-lg font-medium text-gray-900">Abstract</h3>
            </div>
            <div className="px-6 py-4">
              <p className="text-gray-700 whitespace-pre-wrap">{paper.abstract}</p>
            </div>
          </div>

          {/* Keywords */}
          {paper.keywords && paper.keywords.length > 0 && (
            <div className="bg-white shadow rounded-lg">
              <div className="px-6 py-4 border-b border-gray-200">
                <h3 className="text-lg font-medium text-gray-900">Keywords</h3>
              </div>
              <div className="px-6 py-4">
                <div className="flex flex-wrap gap-2">
                  {paper.keywords.map((keyword, index) => (
                    <span
                      key={index}
                      className="inline-flex items-center px-3 py-1 rounded-full text-sm bg-indigo-100 text-indigo-800"
                    >
                      <Tag className="h-3 w-3 mr-1" />
                      {keyword}
                    </span>
                  ))}
                </div>
              </div>
            </div>
          )}

          {/* References */}
          {paper.references && paper.references.length > 0 && (
            <div className="bg-white shadow rounded-lg">
              <div className="px-6 py-4 border-b border-gray-200">
                <h3 className="text-lg font-medium text-gray-900">References</h3>
              </div>
              <div className="px-6 py-4">
                <ol className="space-y-2">
                  {paper.references.map((reference, index) => (
                    <li key={index} className="text-sm text-gray-700">
                      {index + 1}. {reference}
                    </li>
                  ))}
                </ol>
              </div>
            </div>
          )}
        </div>

        {/* Sidebar */}
        <div className="space-y-6">
          {/* File Upload */}
          <div className="bg-white shadow rounded-lg">
            <div className="px-6 py-4 border-b border-gray-200">
              <h3 className="text-lg font-medium text-gray-900">File</h3>
            </div>
            <div className="px-6 py-4">
              {paper.fileUrl ? (
                <div className="space-y-3">
                  <div className="flex items-center p-3 bg-green-50 rounded-lg">
                    <FileText className="h-8 w-8 text-green-600" />
                    <div className="ml-3">
                      <p className="text-sm font-medium text-green-900">PDF uploaded</p>
                      <p className="text-xs text-green-700">Ready for download</p>
                    </div>
                  </div>
                  <a
                    href={paper.fileUrl}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="w-full inline-flex justify-center items-center px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50"
                  >
                    <Eye className="h-4 w-4 mr-2" />
                    View
                  </a>
                </div>
              ) : (
                <div className="space-y-3">
                  <p className="text-sm text-gray-500">
                    Upload a PDF file for this paper.
                  </p>
                  <label className="w-full flex justify-center items-center px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 cursor-pointer">
                    {uploading ? (
                      <LoadingSpinner size="sm" />
                    ) : (
                      <>
                        <Upload className="h-4 w-4 mr-2" />
                        Upload PDF
                      </>
                    )}
                    <input
                      type="file"
                      accept=".pdf"
                      onChange={handleFileUpload}
                      className="hidden"
                      disabled={uploading}
                    />
                  </label>
                </div>
              )}
            </div>
          </div>

          {/* Metadata */}
          <div className="bg-white shadow rounded-lg">
            <div className="px-6 py-4 border-b border-gray-200">
              <h3 className="text-lg font-medium text-gray-900">Metadata</h3>
            </div>
            <div className="px-6 py-4 space-y-3">
              <div className="flex items-center text-sm">
                <Calendar className="h-4 w-4 text-gray-400 mr-2" />
                <span className="text-gray-600">Created:</span>
                <span className="ml-auto font-medium">{formatDate(paper.createdAt)}</span>
              </div>
              <div className="flex items-center text-sm">
                <Calendar className="h-4 w-4 text-gray-400 mr-2" />
                <span className="text-gray-600">Updated:</span>
                <span className="ml-auto font-medium">{formatDate(paper.updatedAt)}</span>
              </div>
              {paper.submissionDate && (
                <div className="flex items-center text-sm">
                  <Upload className="h-4 w-4 text-gray-400 mr-2" />
                  <span className="text-gray-600">Submitted:</span>
                  <span className="ml-auto font-medium">{formatDate(paper.submissionDate)}</span>
                </div>
              )}
              {paper.publicationDate && (
                <div className="flex items-center text-sm">
                  <BookOpen className="h-4 w-4 text-gray-400 mr-2" />
                  <span className="text-gray-600">Published:</span>
                  <span className="ml-auto font-medium">{formatDate(paper.publicationDate)}</span>
                </div>
              )}
            </div>
          </div>

          {/* Actions */}
          <div className="bg-white shadow rounded-lg">
            <div className="px-6 py-4 border-b border-gray-200">
              <h3 className="text-lg font-medium text-gray-900">Actions</h3>
            </div>
            <div className="px-6 py-4 space-y-3">
              <Link
                to={`/reviews/submit/${paper._id}`}
                className="w-full inline-flex justify-center items-center px-4 py-2 border border-transparent rounded-md text-sm font-medium text-white bg-yellow-600 hover:bg-yellow-700"
              >
                <MessageCircle className="h-4 w-4 mr-2" />
                Submit Review
              </Link>
              {paper.latexContent && (
                <Link
                  to={`/latex-editor?paperId=${paper._id}`}
                  className="w-full inline-flex justify-center items-center px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50"
                >
                  <Edit className="h-4 w-4 mr-2" />
                  Edit LaTeX
                </Link>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}