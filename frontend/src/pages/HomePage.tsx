import { useState } from 'react';
import FileUpload from '../components/FileUpload';
import ConfigEditor from '../components/ConfigEditor';
import ConversionPanel from '../components/ConversionPanel';
import { useAppStore } from '../stores/useAppStore';

export default function HomePage() {
  const [activeTab, setActiveTab] = useState<'upload' | 'config' | 'convert'>('upload');
  const { selectedFile, config, error } = useAppStore();

  const tabs = [
    { id: 'upload', label: 'Upload File', disabled: false },
    { id: 'config', label: 'Configure', disabled: !selectedFile },
    { id: 'convert', label: 'Convert', disabled: !selectedFile || !config },
  ] as const;

  return (
    <div className="max-w-6xl mx-auto">
      {/* Tab Navigation */}
      <div className="border-b border-gray-200 mb-8">
        <nav className="-mb-px flex space-x-8">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => !tab.disabled && setActiveTab(tab.id)}
              className={`py-2 px-1 border-b-2 font-medium text-sm ${
                activeTab === tab.id
                  ? 'border-blue-500 text-blue-600'
                  : tab.disabled
                  ? 'border-transparent text-gray-400 cursor-not-allowed'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
              disabled={tab.disabled}
            >
              {tab.label}
            </button>
          ))}
        </nav>
      </div>

      {/* Error Display */}
      {error && (
        <div className="mb-6 bg-red-50 border border-red-200 rounded-md p-4">
          <div className="flex">
            <div className="ml-3">
              <h3 className="text-sm font-medium text-red-800">Error</h3>
              <div className="mt-2 text-sm text-red-700">
                <p>{error}</p>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Tab Content */}
      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        {activeTab === 'upload' && <FileUpload />}
        {activeTab === 'config' && <ConfigEditor />}
        {activeTab === 'convert' && <ConversionPanel />}
      </div>
    </div>
  );
}