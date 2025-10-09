import { useCallback, useState } from 'react';
import { useDropzone } from 'react-dropzone';
import { DocumentTextIcon, CloudArrowUpIcon, XMarkIcon } from '@heroicons/react/24/outline';
import { CheckCircleIcon } from '@heroicons/react/24/solid';
import { useAppStore } from '../stores/useAppStore';

export default function FileUpload() {
  const { selectedFile, setSelectedFile, setError } = useAppStore();
  const [uploadProgress, setUploadProgress] = useState(0);
  const [isUploading, setIsUploading] = useState(false);
  const [fileContent, setFileContent] = useState<string>('');

  const simulateUploadProgress = useCallback(() => {
    setIsUploading(true);
    setUploadProgress(0);
    
    const interval = setInterval(() => {
      setUploadProgress((prev) => {
        if (prev >= 100) {
          clearInterval(interval);
          setIsUploading(false);
          return 100;
        }
        return prev + 10;
      });
    }, 100);
  }, []);

  const readFileContent = useCallback((file: File) => {
    const reader = new FileReader();
    reader.onload = (e) => {
      const content = e.target?.result as string;
      setFileContent(content);
    };
    reader.readAsText(file);
  }, []);

  const onDrop = useCallback((acceptedFiles: File[]) => {
    const file = acceptedFiles[0];
    if (file) {
      // Validate file type
      if (file.type === 'text/markdown' || file.name.endsWith('.md')) {
        // Validate file size (10MB limit)
        if (file.size > 10 * 1024 * 1024) {
          setError('File size must be less than 10MB');
          return;
        }
        
        setError(null);
        simulateUploadProgress();
        readFileContent(file);
        setSelectedFile(file);
      } else {
        setError('Please select a Markdown (.md) file');
      }
    }
  }, [setSelectedFile, setError, simulateUploadProgress, readFileContent]);

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    accept: {
      'text/markdown': ['.md'],
      'text/plain': ['.md'],
    },
    multiple: false,
    maxSize: 10 * 1024 * 1024, // 10MB
  });

  const removeFile = useCallback(() => {
    setSelectedFile(null);
    setFileContent('');
    setUploadProgress(0);
    setIsUploading(false);
    setError(null);
  }, [setSelectedFile, setError]);

  return (
    <div className="desktop-spacing">
      <div>
        <h2 className="section-title">
          Upload Markdown File
        </h2>
        
        <div
          {...getRootProps()}
          className={`drag-drop-area border-2 border-dashed rounded-lg p-6 text-center cursor-pointer ${
            isDragActive
              ? 'border-blue-400 bg-blue-50'
              : selectedFile && !isUploading
              ? 'border-green-400 bg-green-50'
              : isUploading
              ? 'border-blue-400 bg-blue-50'
              : 'border-gray-300 hover:border-gray-400'
          }`}
        >
          <input {...getInputProps()} />
          
          {isUploading ? (
            <div className="space-y-4">
              <CloudArrowUpIcon className="mx-auto h-12 w-12 text-blue-500 animate-pulse" />
              <div className="space-y-2">
                <p className="text-sm font-medium text-blue-700">
                  Uploading {selectedFile?.name}...
                </p>
                <div className="w-full bg-gray-200 rounded-full h-2">
                  <div 
                    className="bg-blue-600 h-2 rounded-full transition-all duration-300 ease-out"
                    style={{ width: `${uploadProgress}%` }}
                  ></div>
                </div>
                <p className="text-xs text-blue-600">
                  {uploadProgress}% complete
                </p>
              </div>
            </div>
          ) : selectedFile ? (
            <div className="space-y-2">
              <div className="flex items-center justify-center space-x-2">
                <DocumentTextIcon className="h-12 w-12 text-green-500" />
                <CheckCircleIcon className="h-6 w-6 text-green-500" />
              </div>
              <p className="text-sm font-medium text-green-700">
                {selectedFile.name}
              </p>
              <p className="text-xs text-green-600">
                {(selectedFile.size / 1024).toFixed(1)} KB
              </p>
              <div className="flex items-center justify-center space-x-3 mt-3">
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    removeFile();
                  }}
                  className="inline-flex items-center px-2.5 py-1 border border-red-300 text-xs font-medium rounded text-red-700 bg-white hover:bg-red-50 focus:outline-none focus:ring-1 focus:ring-red-500 transition-colors"
                >
                  <XMarkIcon className="h-3 w-3 mr-1" />
                  Remove
                </button>
                <p className="text-xs text-gray-500">
                  or click to replace
                </p>
              </div>
            </div>
          ) : (
            <div className="space-y-2">
              <CloudArrowUpIcon className="mx-auto h-10 w-10 text-gray-400" />
              <p className="text-sm font-medium text-gray-700">
                {isDragActive
                  ? 'Drop the file here'
                  : 'Drag and drop a Markdown file, or click to select'}
              </p>
              <p className="text-xs text-gray-500">
                Supports .md files up to 10MB
              </p>
            </div>
          )}
        </div>
      </div>

      {/* Upload Progress Bar (when uploading) */}
      {isUploading && (
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <div className="flex items-center space-x-3">
            <div className="flex-shrink-0">
              <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-blue-600"></div>
            </div>
            <div className="flex-1">
              <p className="text-sm font-medium text-blue-800">
                Processing file...
              </p>
              <p className="text-xs text-blue-600">
                Reading and validating content
              </p>
            </div>
          </div>
        </div>
      )}

      {/* File Details and Preview */}
      {selectedFile && !isUploading && (
        <div className="bg-gray-50 rounded-lg p-4 space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-medium text-gray-900">
              File Details
            </h3>
            <span className="status-indicator status-success">
              Ready
            </span>
          </div>
          
          <div className="grid grid-cols-2 gap-4 text-xs text-gray-600">
            <div>
              <p><span className="font-medium">Name:</span> {selectedFile.name}</p>
              <p><span className="font-medium">Size:</span> {(selectedFile.size / 1024).toFixed(1)} KB</p>
            </div>
            <div>
              <p><span className="font-medium">Type:</span> {selectedFile.type || 'text/markdown'}</p>
              <p><span className="font-medium">Modified:</span> {selectedFile.lastModified ? new Date(selectedFile.lastModified).toLocaleDateString() : 'Unknown'}</p>
            </div>
          </div>

          {/* Content Preview */}
          {fileContent && (
            <div className="border-t pt-4">
              <h4 className="text-sm font-medium text-gray-900 mb-2">
                Content Preview
              </h4>
              <div className="bg-white border rounded p-3 max-h-32 overflow-y-auto">
                <pre className="text-xs text-gray-700 whitespace-pre-wrap font-mono">
                  {fileContent.length > 500 
                    ? `${fileContent.substring(0, 500)}...` 
                    : fileContent
                  }
                </pre>
              </div>
              {fileContent.length > 500 && (
                <p className="text-xs text-gray-500 mt-1">
                  Showing first 500 characters of {fileContent.length} total
                </p>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
}