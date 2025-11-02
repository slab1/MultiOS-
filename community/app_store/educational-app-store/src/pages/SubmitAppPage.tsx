import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';
import { 
  Upload, 
  X, 
  Plus, 
  Minus,
  CheckCircle,
  AlertCircle,
  FileText,
  Image,
  Video
} from 'lucide-react';

interface Category {
  id: string;
  name: string;
  subcategories?: Subcategory[];
}

interface Subcategory {
  id: string;
  name: string;
}

interface AppSubmission {
  title: string;
  description: string;
  shortDescription: string;
  categoryId: string;
  subcategoryId: string;
  gradeLevels: string[];
  subjects: string[];
  tags: string[];
  price: number;
  currency: string;
  platform: string[];
  version: string;
  websiteUrl: string;
  downloadUrl: string;
  educationalContent: {
    learningObjectives: string[];
    curriculumStandards: string[];
    difficultyLevel: string;
    estimatedDuration: string;
  };
  technicalRequirements: {
    minimumOS: string[];
    internetRequired: boolean;
    storageRequired: number;
  };
}

const SubmitAppPage: React.FC = () => {
  const { user } = useAuth();
  const navigate = useNavigate();
  const [categories, setCategories] = useState<Category[]>([]);
  const [currentStep, setCurrentStep] = useState(1);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const [submission, setSubmission] = useState<AppSubmission>({
    title: '',
    description: '',
    shortDescription: '',
    categoryId: '',
    subcategoryId: '',
    gradeLevels: [],
    subjects: [],
    tags: [],
    price: 0,
    currency: 'USD',
    platform: [],
    version: '1.0.0',
    websiteUrl: '',
    downloadUrl: '',
    educationalContent: {
      learningObjectives: [''],
      curriculumStandards: [''],
      difficultyLevel: 'beginner',
      estimatedDuration: ''
    },
    technicalRequirements: {
      minimumOS: [],
      internetRequired: false,
      storageRequired: 100
    }
  });

  const [files, setFiles] = useState({
    icon: null as File | null,
    screenshots: [] as File[],
    video: null as File | null
  });

  const API_BASE = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3001/api';

  useEffect(() => {
    if (user?.role !== 'developer') {
      navigate('/dashboard');
      return;
    }
    loadCategories();
  }, [user, navigate]);

  const loadCategories = async () => {
    try {
      const response = await fetch(`${API_BASE}/categories?includeSubcategories=true`);
      const data = await response.json();
      if (data.success) {
        setCategories(data.data);
      }
    } catch (error) {
      console.error('Error loading categories:', error);
    }
  };

  const handleInputChange = (field: string, value: any) => {
    if (field.includes('.')) {
      const [parent, child] = field.split('.');
      setSubmission(prev => ({
        ...prev,
        [parent]: {
          ...prev[parent as keyof AppSubmission] as any,
          [child]: value
        }
      }));
    } else {
      setSubmission(prev => ({ ...prev, [field]: value }));
    }
  };

  const handleArrayChange = (field: keyof AppSubmission, index: number, value: string) => {
    setSubmission(prev => ({
      ...prev,
      [field]: (prev[field] as string[]).map((item, i) => i === index ? value : item)
    }));
  };

  const addArrayItem = (field: keyof AppSubmission) => {
    setSubmission(prev => ({
      ...prev,
      [field]: [...(prev[field] as string[]), '']
    }));
  };

  const removeArrayItem = (field: keyof AppSubmission, index: number) => {
    setSubmission(prev => ({
      ...prev,
      [field]: (prev[field] as string[]).filter((_, i) => i !== index)
    }));
  };

  const handleFileUpload = (field: keyof typeof files, file: File | null, multiple = false) => {
    if (multiple && file) {
      setFiles(prev => ({
        ...prev,
        [field]: [...(prev[field] as File[]), file]
      }));
    } else {
      setFiles(prev => ({ ...prev, [field]: file }));
    }
  };

  const removeFile = (field: keyof typeof files, index?: number) => {
    if (index !== undefined && Array.isArray(files[field])) {
      setFiles(prev => ({
        ...prev,
        [field]: (prev[field] as File[]).filter((_, i) => i !== index)
      }));
    } else {
      setFiles(prev => ({ ...prev, [field]: null }));
    }
  };

  const validateStep = (step: number): boolean => {
    switch (step) {
      case 1:
        return !!(submission.title && submission.description && submission.categoryId);
      case 2:
        return submission.platform.length > 0;
      case 3:
        return submission.educationalContent.learningObjectives.some(obj => obj.trim());
      case 4:
        return files.icon !== null;
      default:
        return true;
    }
  };

  const handleSubmit = async () => {
    if (!validateStep(4)) {
      setError('Please complete all required fields');
      return;
    }

    setLoading(true);
    setError('');

    try {
      const formData = new FormData();
      
      // Add form fields
      Object.entries(submission).forEach(([key, value]) => {
        if (typeof value === 'object') {
          formData.append(key, JSON.stringify(value));
        } else {
          formData.append(key, value.toString());
        }
      });

      // Add files
      if (files.icon) {
        formData.append('icon', files.icon);
      }
      files.screenshots.forEach((file, index) => {
        formData.append('screenshots', file);
      });
      if (files.video) {
        formData.append('video', files.video);
      }

      const response = await fetch(`${API_BASE}/apps`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('authToken')}`
        },
        body: formData
      });

      const data = await response.json();
      
      if (data.success) {
        navigate('/dashboard', { 
          state: { message: 'App submitted successfully! It will be reviewed within 3-5 business days.' }
        });
      } else {
        setError(data.error || 'Failed to submit app');
      }
    } catch (error) {
      console.error('Error submitting app:', error);
      setError('An error occurred while submitting your app');
    } finally {
      setLoading(false);
    }
  };

  const gradeLevelOptions = ['Pre-K', 'K-2', '3-5', '6-8', '9-12', 'College', 'Adult'];
  const subjectOptions = ['Mathematics', 'Science', 'Language Arts', 'Social Studies', 'Art', 'Music', 'Physical Education', 'Computer Science'];
  const platformOptions = ['Web', 'iOS', 'Android', 'Windows', 'macOS'];
  const osOptions = ['Windows 10+', 'macOS 10.15+', 'iOS 13+', 'Android 8+', 'Any modern web browser'];

  const renderStepIndicator = () => (
    <div className="flex items-center justify-center mb-8">
      {[1, 2, 3, 4, 5].map((step) => (
        <React.Fragment key={step}>
          <div className={`w-8 h-8 rounded-full flex items-center justify-center ${
            currentStep >= step ? 'bg-blue-600 text-white' : 'bg-gray-300 text-gray-600'
          }`}>
            {step}
          </div>
          {step < 5 && (
            <div className={`w-16 h-1 mx-2 ${
              currentStep > step ? 'bg-blue-600' : 'bg-gray-300'
            }`} />
          )}
        </React.Fragment>
      ))}
    </div>
  );

  const renderStep1 = () => (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold text-gray-900">Basic Information</h2>
      
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          App Title *
        </label>
        <input
          type="text"
          value={submission.title}
          onChange={(e) => handleInputChange('title', e.target.value)}
          className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
          placeholder="Enter your app title"
          required
        />
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Short Description *
        </label>
        <input
          type="text"
          value={submission.shortDescription}
          onChange={(e) => handleInputChange('shortDescription', e.target.value)}
          className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
          placeholder="Brief description (50-100 characters)"
          maxLength={100}
        />
        <p className="text-sm text-gray-500 mt-1">{submission.shortDescription.length}/100 characters</p>
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Full Description *
        </label>
        <textarea
          value={submission.description}
          onChange={(e) => handleInputChange('description', e.target.value)}
          rows={6}
          className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
          placeholder="Provide a detailed description of your app, its features, and educational value"
          required
        />
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Category *
          </label>
          <select
            value={submission.categoryId}
            onChange={(e) => handleInputChange('categoryId', e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
            required
          >
            <option value="">Select a category</option>
            {categories.map((category) => (
              <option key={category.id} value={category.id}>
                {category.name}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Subcategory
          </label>
          <select
            value={submission.subcategoryId}
            onChange={(e) => handleInputChange('subcategoryId', e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
            disabled={!submission.categoryId}
          >
            <option value="">Select a subcategory</option>
            {submission.categoryId && categories
              .find(c => c.id === submission.categoryId)?.subcategories
              ?.map((sub) => (
                <option key={sub.id} value={sub.id}>
                  {sub.name}
                </option>
              ))}
          </select>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Version
          </label>
          <input
            type="text"
            value={submission.version}
            onChange={(e) => handleInputChange('version', e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
            placeholder="1.0.0"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Price ($)
          </label>
          <input
            type="number"
            min="0"
            step="0.01"
            value={submission.price}
            onChange={(e) => handleInputChange('price', parseFloat(e.target.value) || 0)}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
            placeholder="0.00"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Storage Required (MB)
          </label>
          <input
            type="number"
            min="0"
            value={submission.technicalRequirements.storageRequired}
            onChange={(e) => handleInputChange('technicalRequirements.storageRequired', parseInt(e.target.value) || 0)}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
            placeholder="100"
          />
        </div>
      </div>
    </div>
  );

  const renderStep2 = () => (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold text-gray-900">Platform & Technical</h2>
      
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-3">
          Platforms Available *
        </label>
        <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
          {platformOptions.map((platform) => (
            <label key={platform} className="flex items-center">
              <input
                type="checkbox"
                checked={submission.platform.includes(platform)}
                onChange={(e) => {
                  const newPlatforms = e.target.checked
                    ? [...submission.platform, platform]
                    : submission.platform.filter(p => p !== platform);
                  handleInputChange('platform', newPlatforms);
                }}
                className="mr-2"
              />
              <span>{platform}</span>
            </label>
          ))}
        </div>
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700 mb-3">
          Minimum OS Requirements
        </label>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {osOptions.map((os) => (
            <label key={os} className="flex items-center">
              <input
                type="checkbox"
                checked={submission.technicalRequirements.minimumOS.includes(os)}
                onChange={(e) => {
                  const newOS = e.target.checked
                    ? [...submission.technicalRequirements.minimumOS, os]
                    : submission.technicalRequirements.minimumOS.filter(o => o !== os);
                  handleInputChange('technicalRequirements.minimumOS', newOS);
                }}
                className="mr-2"
              />
              <span>{os}</span>
            </label>
          ))}
        </div>
      </div>

      <div>
        <label className="flex items-center">
          <input
            type="checkbox"
            checked={submission.technicalRequirements.internetRequired}
            onChange={(e) => handleInputChange('technicalRequirements.internetRequired', e.target.checked)}
            className="mr-2"
          />
          <span className="text-sm font-medium text-gray-700">Internet connection required</span>
        </label>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Website URL
          </label>
          <input
            type="url"
            value={submission.websiteUrl}
            onChange={(e) => handleInputChange('websiteUrl', e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
            placeholder="https://yourapp.com"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Download URL
          </label>
          <input
            type="url"
            value={submission.downloadUrl}
            onChange={(e) => handleInputChange('downloadUrl', e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
            placeholder="https://download.yourapp.com"
          />
        </div>
      </div>
    </div>
  );

  const renderStep3 = () => (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold text-gray-900">Educational Content</h2>
      
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-3">
          Grade Levels
        </label>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          {gradeLevelOptions.map((level) => (
            <label key={level} className="flex items-center">
              <input
                type="checkbox"
                checked={submission.gradeLevels.includes(level)}
                onChange={(e) => {
                  const newLevels = e.target.checked
                    ? [...submission.gradeLevels, level]
                    : submission.gradeLevels.filter(l => l !== level);
                  handleInputChange('gradeLevels', newLevels);
                }}
                className="mr-2"
              />
              <span>{level}</span>
            </label>
          ))}
        </div>
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700 mb-3">
          Subjects
        </label>
        <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
          {subjectOptions.map((subject) => (
            <label key={subject} className="flex items-center">
              <input
                type="checkbox"
                checked={submission.subjects.includes(subject)}
                onChange={(e) => {
                  const newSubjects = e.target.checked
                    ? [...submission.subjects, subject]
                    : submission.subjects.filter(s => s !== subject);
                  handleInputChange('subjects', newSubjects);
                }}
                className="mr-2"
              />
              <span>{subject}</span>
            </label>
          ))}
        </div>
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700 mb-3">
          Learning Objectives *
        </label>
        {submission.educationalContent.learningObjectives.map((objective, index) => (
          <div key={index} className="flex items-center space-x-2 mb-2">
            <input
              type="text"
              value={objective}
              onChange={(e) => handleArrayChange('educationalContent.learningObjectives', index, e.target.value)}
              className="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
              placeholder="Describe a learning objective"
            />
            <button
              type="button"
              onClick={() => removeArrayItem('educationalContent.learningObjectives', index)}
              className="p-2 text-red-600 hover:bg-red-50 rounded-lg"
            >
              <Minus className="h-4 w-4" />
            </button>
          </div>
        ))}
        <button
          type="button"
          onClick={() => addArrayItem('educationalContent.learningObjectives')}
          className="flex items-center space-x-2 text-blue-600 hover:text-blue-700"
        >
          <Plus className="h-4 w-4" />
          <span>Add Learning Objective</span>
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Difficulty Level
          </label>
          <select
            value={submission.educationalContent.difficultyLevel}
            onChange={(e) => handleInputChange('educationalContent.difficultyLevel', e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
          >
            <option value="beginner">Beginner</option>
            <option value="intermediate">Intermediate</option>
            <option value="advanced">Advanced</option>
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Estimated Duration
          </label>
          <input
            type="text"
            value={submission.educationalContent.estimatedDuration}
            onChange={(e) => handleInputChange('educationalContent.estimatedDuration', e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
            placeholder="e.g., 30 minutes, 1 hour"
          />
        </div>
      </div>
    </div>
  );

  const renderStep4 = () => (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold text-gray-900">Media & Assets</h2>
      
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          App Icon *
        </label>
        <div className="border-2 border-dashed border-gray-300 rounded-lg p-6 text-center">
          {files.icon ? (
            <div className="flex items-center justify-center space-x-4">
              <img
                src={URL.createObjectURL(files.icon)}
                alt="App Icon"
                className="w-16 h-16 rounded-lg object-cover"
              />
              <div>
                <p className="text-sm font-medium">{files.icon.name}</p>
                <button
                  type="button"
                  onClick={() => removeFile('icon')}
                  className="text-red-600 hover:text-red-700 text-sm"
                >
                  Remove
                </button>
              </div>
            </div>
          ) : (
            <div>
              <Upload className="h-12 w-12 text-gray-400 mx-auto mb-4" />
              <div className="space-y-2">
                <label className="cursor-pointer">
                  <span className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700">
                    Upload Icon
                  </span>
                  <input
                    type="file"
                    accept="image/*"
                    onChange={(e) => handleFileUpload('icon', e.target.files?.[0] || null)}
                    className="hidden"
                  />
                </label>
                <p className="text-sm text-gray-600">PNG or JPG, max 5MB</p>
              </div>
            </div>
          )}
        </div>
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Screenshots
        </label>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          {files.screenshots.map((file, index) => (
            <div key={index} className="relative">
              <img
                src={URL.createObjectURL(file)}
                alt={`Screenshot ${index + 1}`}
                className="w-full h-24 object-cover rounded-lg"
              />
              <button
                type="button"
                onClick={() => removeFile('screenshots', index)}
                className="absolute top-1 right-1 bg-red-600 text-white rounded-full p-1 hover:bg-red-700"
              >
                <X className="h-3 w-3" />
              </button>
            </div>
          ))}
          {files.screenshots.length < 5 && (
            <div className="border-2 border-dashed border-gray-300 rounded-lg p-4 text-center">
              <label className="cursor-pointer">
                <Plus className="h-6 w-6 text-gray-400 mx-auto mb-2" />
                <span className="text-sm text-gray-600">Add Screenshot</span>
                <input
                  type="file"
                  accept="image/*"
                  onChange={(e) => handleFileUpload('screenshots', e.target.files?.[0] || null, true)}
                  className="hidden"
                />
              </label>
            </div>
          )}
        </div>
        <p className="text-sm text-gray-600 mt-2">Up to 5 screenshots. PNG or JPG recommended.</p>
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Demo Video (Optional)
        </label>
        <div className="border-2 border-dashed border-gray-300 rounded-lg p-6 text-center">
          {files.video ? (
            <div className="flex items-center justify-center space-x-4">
              <Video className="h-16 w-16 text-gray-400" />
              <div>
                <p className="text-sm font-medium">{files.video.name}</p>
                <button
                  type="button"
                  onClick={() => removeFile('video')}
                  className="text-red-600 hover:text-red-700 text-sm"
                >
                  Remove
                </button>
              </div>
            </div>
          ) : (
            <div>
              <Video className="h-12 w-12 text-gray-400 mx-auto mb-4" />
              <div className="space-y-2">
                <label className="cursor-pointer">
                  <span className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700">
                    Upload Video
                  </span>
                  <input
                    type="file"
                    accept="video/*"
                    onChange={(e) => handleFileUpload('video', e.target.files?.[0] || null)}
                    className="hidden"
                  />
                </label>
                <p className="text-sm text-gray-600">MP4 or MOV, max 50MB</p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );

  const renderStep5 = () => (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold text-gray-900">Review & Submit</h2>
      
      <div className="bg-gray-50 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Submission Summary</h3>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <h4 className="font-medium text-gray-900 mb-2">Basic Information</h4>
            <ul className="text-sm text-gray-600 space-y-1">
              <li><strong>Title:</strong> {submission.title}</li>
              <li><strong>Category:</strong> {categories.find(c => c.id === submission.categoryId)?.name}</li>
              <li><strong>Version:</strong> {submission.version}</li>
              <li><strong>Price:</strong> ${submission.price}</li>
            </ul>
          </div>
          
          <div>
            <h4 className="font-medium text-gray-900 mb-2">Platform & Content</h4>
            <ul className="text-sm text-gray-600 space-y-1">
              <li><strong>Platforms:</strong> {submission.platform.join(', ')}</li>
              <li><strong>Grade Levels:</strong> {submission.gradeLevels.join(', ') || 'None selected'}</li>
              <li><strong>Subjects:</strong> {submission.subjects.join(', ') || 'None selected'}</li>
              <li><strong>Files:</strong> {files.icon ? 'Icon uploaded' : 'No icon'}, {files.screenshots.length} screenshots</li>
            </ul>
          </div>
        </div>
      </div>

      <div className="bg-blue-50 border border-blue-200 rounded-lg p-6">
        <div className="flex items-start space-x-3">
          <CheckCircle className="h-6 w-6 text-blue-600 mt-0.5" />
          <div>
            <h3 className="font-semibold text-blue-900 mb-2">Review Process</h3>
            <p className="text-blue-800 text-sm">
              Your app will be reviewed by our team within 3-5 business days. We'll evaluate it based on:
            </p>
            <ul className="text-blue-800 text-sm mt-2 list-disc list-inside">
              <li>Educational value and learning outcomes</li>
              <li>User experience and accessibility</li>
              <li>Technical functionality and compatibility</li>
              <li>Content appropriateness and safety</li>
            </ul>
            <p className="text-blue-800 text-sm mt-2">
              You'll receive email updates about the review status and any required changes.
            </p>
          </div>
        </div>
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-4">
          <div className="flex items-center space-x-2">
            <AlertCircle className="h-5 w-5 text-red-400" />
            <p className="text-red-800 text-sm">{error}</p>
          </div>
        </div>
      )}
    </div>
  );

  if (user?.role !== 'developer') {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-center">
          <AlertCircle className="h-16 w-16 text-red-500 mx-auto mb-4" />
          <h2 className="text-2xl font-bold text-gray-900 mb-4">Access Denied</h2>
          <p className="text-gray-600 mb-4">Only developers can submit apps.</p>
          <Link to="/dashboard" className="text-blue-600 hover:text-blue-700">
            Return to Dashboard
          </Link>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="bg-white rounded-lg shadow-md p-8">
          {renderStepIndicator()}
          
          <form onSubmit={(e) => e.preventDefault()}>
            {currentStep === 1 && renderStep1()}
            {currentStep === 2 && renderStep2()}
            {currentStep === 3 && renderStep3()}
            {currentStep === 4 && renderStep4()}
            {currentStep === 5 && renderStep5()}
            
            <div className="flex justify-between mt-8 pt-6 border-t border-gray-200">
              <button
                type="button"
                onClick={() => setCurrentStep(Math.max(1, currentStep - 1))}
                disabled={currentStep === 1}
                className="px-6 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Previous
              </button>
              
              {currentStep < 5 ? (
                <button
                  type="button"
                  onClick={() => {
                    if (validateStep(currentStep)) {
                      setCurrentStep(currentStep + 1);
                      setError('');
                    } else {
                      setError('Please complete all required fields');
                    }
                  }}
                  className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
                >
                  Next
                </button>
              ) : (
                <button
                  type="button"
                  onClick={handleSubmit}
                  disabled={loading}
                  className="px-8 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 disabled:opacity-50"
                >
                  {loading ? 'Submitting...' : 'Submit for Review'}
                </button>
              )}
            </div>
          </form>
        </div>
      </div>
    </div>
  );
};

export default SubmitAppPage;