import { useState, useEffect, useCallback } from 'react';
import { useAppStore } from '../stores/useAppStore';
import { ApiService } from '../services/api';
import { YamlEditor } from './YamlEditor';
import { ConfigPreview } from './ConfigPreview';
import type { ConversionConfig } from '../types';
import * as yaml from 'js-yaml';

export default function ConfigEditor() {
  const { config, setConfig, setError } = useAppStore();
  const [naturalLanguagePrompt, setNaturalLanguagePrompt] = useState('');
  const [isUpdating, setIsUpdating] = useState(false);
  const [validationError, setValidationError] = useState<string | null>(null);
  const [parsedConfig, setParsedConfig] = useState<ConversionConfig | null>(null);
  const [isValidating, setIsValidating] = useState(false);

  // Debounced validation function
  const validateConfig = useCallback(
    debounce((configText: string) => {
      setIsValidating(true);
      try {
        const parsed = yaml.load(configText) as ConversionConfig;
        
        // Basic validation
        if (!parsed || typeof parsed !== 'object') {
          throw new Error('Configuration must be a valid object');
        }
        
        if (!parsed.document) {
          throw new Error('Missing required "document" section');
        }
        
        if (!parsed.styles) {
          throw new Error('Missing required "styles" section');
        }
        
        if (!parsed.elements) {
          throw new Error('Missing required "elements" section');
        }

        setParsedConfig(parsed);
        setValidationError(null);
      } catch (error) {
        setParsedConfig(null);
        setValidationError(error instanceof Error ? error.message : 'Invalid YAML format');
      } finally {
        setIsValidating(false);
      }
    }, 500),
    []
  );

  // Validate config whenever it changes
  useEffect(() => {
    if (config.trim()) {
      validateConfig(config);
    }
  }, [config, validateConfig]);

  const handleConfigChange = (value: string) => {
    setConfig(value);
  };

  const handleNaturalLanguageUpdate = async () => {
    if (!naturalLanguagePrompt.trim()) return;
    
    setIsUpdating(true);
    setError(null);
    
    try {
      const response = await ApiService.updateConfig({
        config: config,
        natural_language: naturalLanguagePrompt
      });

      if (response.success && response.updated_config) {
        setConfig(response.updated_config);
        setNaturalLanguagePrompt('');
      } else {
        setError(response.error || 'Failed to update configuration');
      }
    } catch (error) {
      setError(error instanceof Error ? error.message : 'Network error occurred');
    } finally {
      setIsUpdating(false);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
      handleNaturalLanguageUpdate();
    }
  };

  return (
    <div className="desktop-spacing">
      <div>
        <h2 className="section-title">
          Configuration Editor
        </h2>
        
        {/* Natural Language Input */}
        <div className="mb-6 p-4 bg-blue-50 rounded-lg border border-blue-200">
          <h3 className="text-sm font-medium text-blue-900 mb-2">
            Natural Language Configuration
          </h3>
          <p className="text-xs text-blue-700 mb-3">
            Describe how you want to modify the configuration in plain English.
            Press Ctrl+Enter (Cmd+Enter on Mac) to apply changes.
          </p>
          <div className="space-y-3">
            <textarea
              value={naturalLanguagePrompt}
              onChange={(e) => setNaturalLanguagePrompt(e.target.value)}
              onKeyDown={handleKeyPress}
              placeholder="e.g., Make all headings bold and increase font size to 16pt, or change paragraph spacing to 1.5"
              rows={3}
              className="w-full form-textarea-compact border-blue-300"
            />
            <div className="flex justify-between items-center">
              <div className="text-xs text-blue-600">
                Examples: "Make H1 headings 20pt Arial", "Set margins to 1 inch", "Use Calibri font for paragraphs"
              </div>
              <button
                onClick={handleNaturalLanguageUpdate}
                disabled={!naturalLanguagePrompt.trim() || isUpdating}
                className="btn-primary btn-small"
              >
                {isUpdating ? (
                  <span className="flex items-center">
                    <svg className="animate-spin -ml-1 mr-2 h-3 w-3 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                      <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                      <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                    Updating...
                  </span>
                ) : (
                  'Update Config'
                )}
              </button>
            </div>
          </div>
        </div>

        {/* YAML Editor */}
        <div>
          <div className="flex justify-between items-center mb-2">
            <label className="block text-sm font-medium text-gray-700">
              YAML Configuration
            </label>
            <div className="flex items-center space-x-2">
              {isValidating && (
                <span className="text-xs text-gray-500 flex items-center">
                  <svg className="animate-spin -ml-1 mr-1 h-3 w-3 text-gray-500" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                  Validating...
                </span>
              )}
              {!isValidating && validationError && (
                <span className="text-xs text-red-600 flex items-center">
                  <svg className="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 20 20">
                    <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
                  </svg>
                  Invalid
                </span>
              )}
              {!isValidating && !validationError && parsedConfig && (
                <span className="text-xs text-green-600 flex items-center">
                  <svg className="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 20 20">
                    <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                  </svg>
                  Valid
                </span>
              )}
            </div>
          </div>
          
          <YamlEditor
            value={config}
            onChange={handleConfigChange}
            error={validationError}
          />
          
          <p className="mt-2 text-xs text-gray-500">
            Edit the YAML configuration directly or use the natural language input above.
            Changes are validated in real-time.
          </p>
        </div>
      </div>

      {/* Configuration Preview */}
      <ConfigPreview 
        config={parsedConfig}
        error={validationError}
        isValidating={isValidating}
      />
    </div>
  );
}

// Debounce utility function
function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: ReturnType<typeof setTimeout>;
  return (...args: Parameters<T>) => {
    clearTimeout(timeout);
    timeout = setTimeout(() => func(...args), wait);
  };
}