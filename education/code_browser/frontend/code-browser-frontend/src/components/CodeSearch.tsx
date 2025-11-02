import React, { useState, useRef, useEffect } from 'react';
import { Search, X, Filter } from 'lucide-react';

interface CodeSearchProps {
  onSearch: (query: string, filters: SearchFilters) => void;
  isSearching: boolean;
}

interface SearchFilters {
  caseSensitive: boolean;
  wholeWord: boolean;
  regexPattern: boolean;
  fileTypes: string[];
  includeComments: boolean;
}

export const CodeSearch: React.FC<CodeSearchProps> = ({ onSearch, isSearching }) => {
  const [query, setQuery] = useState('');
  const [showFilters, setShowFilters] = useState(false);
  const [filters, setFilters] = useState<SearchFilters>({
    caseSensitive: false,
    wholeWord: false,
    regexPattern: false,
    fileTypes: ['rs', 'c', 'cpp'],
    includeComments: true,
  });
  const searchRef = useRef<HTMLInputElement>(null);
  const filtersRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (filtersRef.current && !filtersRef.current.contains(event.target as Node)) {
        setShowFilters(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (query.trim()) {
      onSearch(query.trim(), filters);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      searchRef.current?.blur();
      setShowFilters(false);
    } else if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
      handleSubmit(e);
    }
  };

  const toggleFilter = (filterKey: keyof SearchFilters) => {
    setFilters(prev => ({
      ...prev,
      [filterKey]: !prev[filterKey]
    }));
  };

  const handleFileTypeToggle = (fileType: string) => {
    setFilters(prev => ({
      ...prev,
      fileTypes: prev.fileTypes.includes(fileType)
        ? prev.fileTypes.filter(ft => ft !== fileType)
        : [...prev.fileTypes, fileType]
    }));
  };

  return (
    <div className="relative">
      <form onSubmit={handleSubmit} className="flex items-center">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input
            ref={searchRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Search code, functions, variables..."
            className="pl-10 pr-4 py-2 w-80 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
          {isSearching && (
            <div className="absolute right-3 top-1/2 transform -translate-y-1/2">
              <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-600"></div>
            </div>
          )}
        </div>
        
        <button
          type="button"
          onClick={() => setShowFilters(!showFilters)}
          className={`ml-2 p-2 border border-gray-300 rounded-md hover:bg-gray-50 transition-colors duration-200 ${
            showFilters ? 'bg-blue-50 border-blue-300' : 'bg-white'
          }`}
          title="Search filters"
        >
          <Filter className="w-4 h-4 text-gray-600" />
        </button>
      </form>

      {/* Search Filters Dropdown */}
      {showFilters && (
        <div
          ref={filtersRef}
          className="absolute right-0 top-full mt-2 w-80 bg-white border border-gray-200 rounded-lg shadow-lg z-50"
        >
          <div className="p-4">
            <h3 className="text-sm font-medium text-gray-900 mb-3">Search Filters</h3>
            
            <div className="space-y-4">
              {/* Basic Filters */}
              <div>
                <h4 className="text-xs font-medium text-gray-700 mb-2">Matching Options</h4>
                <div className="space-y-2">
                  <label className="flex items-center">
                    <input
                      type="checkbox"
                      checked={filters.caseSensitive}
                      onChange={() => toggleFilter('caseSensitive')}
                      className="mr-2"
                    />
                    <span className="text-sm text-gray-700">Case sensitive</span>
                  </label>
                  
                  <label className="flex items-center">
                    <input
                      type="checkbox"
                      checked={filters.wholeWord}
                      onChange={() => toggleFilter('wholeWord')}
                      className="mr-2"
                    />
                    <span className="text-sm text-gray-700">Whole word</span>
                  </label>
                  
                  <label className="flex items-center">
                    <input
                      type="checkbox"
                      checked={filters.regexPattern}
                      onChange={() => toggleFilter('regexPattern')}
                      className="mr-2"
                    />
                    <span className="text-sm text-gray-700">Regular expression</span>
                  </label>
                  
                  <label className="flex items-center">
                    <input
                      type="checkbox"
                      checked={filters.includeComments}
                      onChange={() => toggleFilter('includeComments')}
                      className="mr-2"
                    />
                    <span className="text-sm text-gray-700">Include comments</span>
                  </label>
                </div>
              </div>

              {/* File Types */}
              <div>
                <h4 className="text-xs font-medium text-gray-700 mb-2">File Types</h4>
                <div className="flex flex-wrap gap-2">
                  {['rs', 'c', 'cpp', 'h', 'hpp', 'asm'].map(fileType => (
                    <button
                      key={fileType}
                      onClick={() => handleFileTypeToggle(fileType)}
                      className={`px-2 py-1 text-xs rounded border transition-colors duration-200 ${
                        filters.fileTypes.includes(fileType)
                          ? 'bg-blue-100 border-blue-300 text-blue-700'
                          : 'bg-gray-100 border-gray-300 text-gray-600 hover:bg-gray-200'
                      }`}
                    >
                      .{fileType}
                    </button>
                  ))}
                </div>
              </div>

              {/* Search Actions */}
              <div className="flex justify-between pt-2 border-t border-gray-200">
                <button
                  onClick={() => {
                    setQuery('');
                    setFilters({
                      caseSensitive: false,
                      wholeWord: false,
                      regexPattern: false,
                      fileTypes: ['rs', 'c', 'cpp'],
                      includeComments: true,
                    });
                  }}
                  className="text-sm text-gray-500 hover:text-gray-700"
                >
                  Clear all
                </button>
                
                <button
                  onClick={handleSubmit}
                  disabled={!query.trim() || isSearching}
                  className="px-4 py-2 bg-blue-600 text-white text-sm rounded hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {isSearching ? 'Searching...' : 'Search'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
