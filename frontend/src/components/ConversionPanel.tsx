import { useState } from 'react';
import { ArrowDownTrayIcon, Cog6ToothIcon, ExclamationTriangleIcon } from '@heroicons/react/24/outline';
import { useAppStore } from '../stores/useAppStore';

export default function ConversionPanel() {
  const { 
    selectedFile, 
    config, 
    isConverting, 
    setIsConverting, 
    convertedFile, 
    setConvertedFile, 
    setError,
    error
  } = useAppStore();
  
  const [conversionProgress, setConversionProgress] = useState(0);
  const [retryCount, setRetryCount] = useState(0);
  const [conversionTime, setConversionTime] = useState<number | null>(null);
  const maxRetries = 3;

  const handleConvert = async (isRetry = false) => {
    if (!selectedFile) return;

    if (!isRetry) {
      setRetryCount(0);
    }

    setIsConverting(true);
    setConversionProgress(0);
    setError(null);
    setConversionTime(null);

    const startTime = Date.now();

    try {
      // Start progress animation
      const progressInterval = setInterval(() => {
        setConversionProgress(prev => {
          if (prev >= 90) {
            clearInterval(progressInterval);
            return 90;
          }
          return prev + 10;
        });
      }, 200);

      // Read file content
      const markdownContent = await selectedFile.text();
      
      // Validate markdown content
      if (!markdownContent.trim()) {
        throw new Error('The selected file appears to be empty');
      }
      
      // Make actual API call
      const { ApiService } = await import('../services/api');
      const response = await ApiService.convertMarkdown({
        markdown: markdownContent,
        config: config || undefined,
      });
      
      clearInterval(progressInterval);
      setConversionProgress(100);
      
      if (response.success && response.file_data) {
        const endTime = Date.now();
        setConversionTime(endTime - startTime);
        setConvertedFile(response.file_data);
        setRetryCount(0);
      } else {
        throw new Error(response.error || 'Conversion failed');
      }
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Conversion failed';
      
      // Check if we should retry
      if (retryCount < maxRetries && (
        errorMessage.includes('Network error') || 
        errorMessage.includes('timeout') ||
        errorMessage.includes('fetch')
      )) {
        setRetryCount(prev => prev + 1);
        setError(`${errorMessage}. Retrying... (${retryCount + 1}/${maxRetries})`);
        
        // Retry after a short delay
        setTimeout(() => {
          handleConvert(true);
        }, 1000 * (retryCount + 1)); // Exponential backoff
        return;
      }
      
      setError(errorMessage);
    } finally {
      setIsConverting(false);
    }
  };

  const handleRetry = () => {
    setRetryCount(0);
    handleConvert();
  };

  const handleReset = () => {
    setConvertedFile(null);
    setError(null);
    setConversionProgress(0);
    setConversionTime(null);
    setRetryCount(0);
  };

  const handleDownload = () => {
    if (!convertedFile || !selectedFile) {
      setError('No converted file available for download');
      return;
    }

    try {
      const blob = new Blob([new Uint8Array(convertedFile)], {
        type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document'
      });
      
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      
      // Generate filename: replace .md extension with .docx, or add .docx if no extension
      const originalName = selectedFile.name;
      const downloadName = originalName.endsWith('.md') 
        ? originalName.replace(/\.md$/, '.docx')
        : `${originalName}.docx`;
      
      a.download = downloadName;
      a.style.display = 'none';
      
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      
      // Show success feedback
      console.log(`Downloaded: ${downloadName}`);
    } catch (error) {
      setError(`Download failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  };

  return (
    <div className="desktop-spacing">
      <div>
        <h2 className="section-title">
          Convert to DOCX
        </h2>
        
        {/* File and Config Summary */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
          <div className="bg-gray-50 rounded-lg p-4">
            <h3 className="subsection-title">
              Selected File
            </h3>
            {selectedFile ? (
              <div className="text-sm text-gray-600">
                <p className="font-medium">{selectedFile.name}</p>
                <p>{(selectedFile.size / 1024).toFixed(1)} KB</p>
              </div>
            ) : (
              <p className="text-sm text-gray-500">No file selected</p>
            )}
          </div>
          
          <div className="bg-gray-50 rounded-lg p-4">
            <h3 className="subsection-title">
              Configuration
            </h3>
            <div className="text-sm text-gray-600">
              <p>{config ? 'Custom configuration loaded' : 'Using default configuration'}</p>
              <p className="text-xs text-gray-500 mt-1">
                {config.split('\n').length} lines
              </p>
            </div>
          </div>
        </div>

        {/* Error Display */}
        {error && (
          <div className="bg-red-50 border border-red-200 rounded-md p-4 mb-4">
            <div className="flex items-start">
              <div className="flex-shrink-0">
                <ExclamationTriangleIcon className="h-5 w-5 text-red-400" />
              </div>
              <div className="ml-3 flex-1">
                <h3 className="text-sm font-medium text-red-800">
                  Conversion Error
                </h3>
                <div className="mt-2 text-sm text-red-700">
                  <p>{error}</p>
                </div>
                <div className="mt-3 flex flex-wrap gap-2">
                  {selectedFile && !isConverting && (
                    <button
                      onClick={handleRetry}
                      className="text-xs font-medium text-red-800 hover:text-red-900 bg-red-100 px-3 py-1.5 rounded transition-colors"
                    >
                      Try Again
                    </button>
                  )}
                  <button
                    onClick={() => setError(null)}
                    className="text-xs font-medium text-red-800 hover:text-red-900 px-2 py-1.5 rounded hover:bg-red-50 transition-colors"
                  >
                    Dismiss
                  </button>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Conversion Controls */}
        <div className="space-y-4">
          {!convertedFile && (
            <div className="flex justify-center">
              <button
                onClick={() => handleConvert()}
                disabled={!selectedFile || isConverting}
                className="btn-primary desktop-btn flex items-center"
              >
                {isConverting ? (
                  <>
                    <Cog6ToothIcon className="animate-spin -ml-1 mr-2 h-4 w-4" />
                    Converting...
                  </>
                ) : (
                  'Convert to DOCX'
                )}
              </button>
            </div>
          )}

          {/* Progress Bar */}
          {isConverting && (
            <div className="space-y-2 max-w-md mx-auto">
              <div className="w-full bg-gray-200 rounded-full h-2">
                <div
                  className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                  style={{ width: `${conversionProgress}%` }}
                />
              </div>
              <p className="text-sm text-gray-600 text-center">
                Processing your markdown file... {conversionProgress}%
              </p>
            </div>
          )}

          {/* Success and Download */}
          {convertedFile && (
            <div className="space-y-3">
              <div className="bg-green-50 border border-green-200 rounded-md p-4">
                <div className="flex items-start">
                  <div className="flex-shrink-0">
                    <svg className="h-5 w-5 text-green-400" viewBox="0 0 20 20" fill="currentColor">
                      <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                    </svg>
                  </div>
                  <div className="ml-3 flex-1">
                    <p className="text-sm font-medium text-green-800">
                      Conversion completed successfully!
                    </p>
                    <div className="text-xs text-green-600 mt-1 space-y-1">
                      <p>Your DOCX file is ready for download</p>
                      {conversionTime && (
                        <p>Conversion time: {(conversionTime / 1000).toFixed(1)}s</p>
                      )}
                      {convertedFile && (
                        <p>File size: {(convertedFile.byteLength / 1024).toFixed(1)} KB</p>
                      )}
                    </div>
                  </div>
                </div>
              </div>
              
              <div className="flex flex-col sm:flex-row gap-3 justify-center">
                <button
                  onClick={handleDownload}
                  className="btn-success desktop-btn flex items-center"
                >
                  <ArrowDownTrayIcon className="-ml-1 mr-2 h-4 w-4" />
                  Download DOCX File
                </button>
                
                <button
                  onClick={handleReset}
                  className="btn-secondary desktop-btn"
                >
                  Convert Another
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}