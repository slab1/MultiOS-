import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useForm, useFieldArray } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { toast } from 'react-hot-toast';
import { Plus, X, FileText, Upload, Save } from 'lucide-react';

import LoadingSpinner from '../Common/LoadingSpinner';
import { papersAPI } from '../../services/api';

const authorSchema = z.object({
  name: z.string().min(1, 'Author name is required'),
  email: z.string().email('Invalid email address'),
  affiliation: z.string().min(1, 'Affiliation is required')
});

const createPaperSchema = z.object({
  title: z.string().min(5, 'Title must be at least 5 characters'),
  abstract: z.string().min(50, 'Abstract must be at least 50 characters'),
  authors: z.array(authorSchema).min(1, 'At least one author is required'),
  keywords: z.string().transform(val => val.split(',').map(k => k.trim()).filter(k => k)),
  references: z.string().transform(val => val.split('\n').filter(r => r.trim())),
  tags: z.string().transform(val => val.split(',').map(t => t.trim()).filter(t => t)),
  latexContent: z.string().optional(),
  conferenceId: z.string().optional()
});

type CreatePaperFormData = z.infer<typeof createPaperSchema>;

export default function CreatePaper() {
  const navigate = useNavigate();
  const [isLoading, setIsLoading] = useState(false);
  const [uploading, setUploading] = useState(false);

  const {
    register,
    control,
    handleSubmit,
    watch,
    formState: { errors }
  } = useForm<CreatePaperFormData>({
    resolver: zodResolver(createPaperSchema),
    defaultValues: {
      title: '',
      abstract: '',
      authors: [{ name: '', email: '', affiliation: '' }],
      keywords: [],
      references: [],
      tags: []
    }
  });

  const { fields, append, remove } = useFieldArray({
    control,
    name: 'authors'
  });

  const onSubmit = async (data: CreatePaperFormData) => {
    setIsLoading(true);
    try {
      const paperData = {
        ...data,
        status: 'draft'
      };
      
      const response = await papersAPI.createPaper(paperData);
      toast.success('Paper created successfully!');
      navigate(`/papers/${response.data._id}`);
    } catch (error: any) {
      toast.error(error.message || 'Failed to create paper');
    } finally {
      setIsLoading(false);
    }
  };

  const addAuthor = () => {
    append({ name: '', email: '', affiliation: '' });
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-white shadow rounded-lg">
        <div className="px-6 py-4 border-b border-gray-200">
          <div className="flex items-center">
            <FileText className="h-6 w-6 text-indigo-600 mr-3" />
            <div>
              <h1 className="text-2xl font-semibold text-gray-900">Create New Paper</h1>
              <p className="mt-1 text-sm text-gray-600">
                Fill in the details to create a new academic paper.
              </p>
            </div>
          </div>
        </div>
      </div>

      <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
        {/* Basic Information */}
        <div className="bg-white shadow rounded-lg">
          <div className="px-6 py-4 border-b border-gray-200">
            <h2 className="text-lg font-medium text-gray-900">Basic Information</h2>
          </div>
          <div className="px-6 py-4 space-y-4">
            <div>
              <label htmlFor="title" className="block text-sm font-medium text-gray-700">
                Title *
              </label>
              <input
                {...register('title')}
                type="text"
                className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                placeholder="Enter the paper title"
              />
              {errors.title && (
                <p className="mt-1 text-sm text-red-600">{errors.title.message}</p>
              )}
            </div>

            <div>
              <label htmlFor="abstract" className="block text-sm font-medium text-gray-700">
                Abstract *
              </label>
              <textarea
                {...register('abstract')}
                rows={6}
                className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                placeholder="Enter the abstract"
              />
              {errors.abstract && (
                <p className="mt-1 text-sm text-red-600">{errors.abstract.message}</p>
              )}
            </div>

            <div>
              <label htmlFor="keywords" className="block text-sm font-medium text-gray-700">
                Keywords
              </label>
              <input
                {...register('keywords')}
                type="text"
                className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                placeholder="keyword1, keyword2, keyword3"
              />
              <p className="mt-1 text-sm text-gray-500">Separate keywords with commas</p>
            </div>

            <div>
              <label htmlFor="tags" className="block text-sm font-medium text-gray-700">
                Tags
              </label>
              <input
                {...register('tags')}
                type="text"
                className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                placeholder="tag1, tag2, tag3"
              />
              <p className="mt-1 text-sm text-gray-500">Separate tags with commas</p>
            </div>
          </div>
        </div>

        {/* Authors */}
        <div className="bg-white shadow rounded-lg">
          <div className="px-6 py-4 border-b border-gray-200">
            <div className="flex items-center justify-between">
              <h2 className="text-lg font-medium text-gray-900">Authors *</h2>
              <button
                type="button"
                onClick={addAuthor}
                className="inline-flex items-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-indigo-700 bg-indigo-100 hover:bg-indigo-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
              >
                <Plus className="h-4 w-4 mr-1" />
                Add Author
              </button>
            </div>
          </div>
          <div className="px-6 py-4 space-y-4">
            {fields.map((field, index) => (
              <div key={field.id} className="p-4 border border-gray-200 rounded-lg">
                <div className="flex items-center justify-between mb-4">
                  <h3 className="text-sm font-medium text-gray-900">
                    Author {index + 1}
                  </h3>
                  {fields.length > 1 && (
                    <button
                      type="button"
                      onClick={() => remove(index)}
                      className="text-red-600 hover:text-red-800"
                    >
                      <X className="h-4 w-4" />
                    </button>
                  )}
                </div>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-700">
                      Name *
                    </label>
                    <input
                      {...register(`authors.${index}.name` as const)}
                      type="text"
                      className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                      placeholder="Author name"
                    />
                    {errors.authors?.[index]?.name && (
                      <p className="mt-1 text-sm text-red-600">
                        {errors.authors[index]?.name?.message}
                      </p>
                    )}
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700">
                      Email *
                    </label>
                    <input
                      {...register(`authors.${index}.email` as const)}
                      type="email"
                      className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                      placeholder="author@example.com"
                    />
                    {errors.authors?.[index]?.email && (
                      <p className="mt-1 text-sm text-red-600">
                        {errors.authors[index]?.email?.message}
                      </p>
                    )}
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700">
                      Affiliation *
                    </label>
                    <input
                      {...register(`authors.${index}.affiliation` as const)}
                      type="text"
                      className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                      placeholder="University/Institution"
                    />
                    {errors.authors?.[index]?.affiliation && (
                      <p className="mt-1 text-sm text-red-600">
                        {errors.authors[index]?.affiliation?.message}
                      </p>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Content */}
        <div className="bg-white shadow rounded-lg">
          <div className="px-6 py-4 border-b border-gray-200">
            <h2 className="text-lg font-medium text-gray-900">Content</h2>
          </div>
          <div className="px-6 py-4 space-y-4">
            <div>
              <label htmlFor="references" className="block text-sm font-medium text-gray-700">
                References
              </label>
              <textarea
                {...register('references')}
                rows={6}
                className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                placeholder="Enter references, one per line"
              />
              <p className="mt-1 text-sm text-gray-500">
                Enter each reference on a new line
              </p>
            </div>

            <div>
              <label htmlFor="latexContent" className="block text-sm font-medium text-gray-700">
                LaTeX Content (Optional)
              </label>
              <textarea
                {...register('latexContent')}
                rows={8}
                className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm font-mono"
                placeholder="Enter LaTeX content for the paper"
              />
              <p className="mt-1 text-sm text-gray-500">
                You can paste LaTeX content here for LaTeX-based compilation
              </p>
            </div>
          </div>
        </div>

        {/* Actions */}
        <div className="bg-white shadow rounded-lg">
          <div className="px-6 py-4">
            <div className="flex justify-end space-x-3">
              <button
                type="button"
                onClick={() => navigate('/papers')}
                className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
              >
                Cancel
              </button>
              <button
                type="submit"
                disabled={isLoading}
                className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isLoading ? (
                  <LoadingSpinner size="sm" />
                ) : (
                  <>
                    <Save className="h-4 w-4 mr-2" />
                    Create Paper
                  </>
                )}
              </button>
            </div>
          </div>
        </div>
      </form>
    </div>
  );
}