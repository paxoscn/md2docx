import { useState } from 'react';
import ConfigEditor from './ConfigEditor';

export function ConfigEditorDemo() {
  const [showDemo, setShowDemo] = useState(false);

  return (
    <div className="max-w-4xl mx-auto p-6">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-900 mb-2">
          Configuration Editor Demo
        </h1>
        <p className="text-gray-600">
          This demo shows the enhanced configuration editor with YAML syntax highlighting,
          real-time validation, and natural language configuration updates.
        </p>
      </div>

      <div className="mb-4">
        <button
          onClick={() => setShowDemo(!showDemo)}
          className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
        >
          {showDemo ? 'Hide Demo' : 'Show Demo'}
        </button>
      </div>

      {showDemo && (
        <div className="bg-white rounded-lg shadow-lg border border-gray-200 p-6">
          <div className="mb-4 p-4 bg-blue-50 rounded-lg border border-blue-200">
            <h3 className="text-sm font-medium text-blue-900 mb-2">
              Demo Features
            </h3>
            <ul className="text-sm text-blue-700 space-y-1">
              <li>• <strong>YAML Editor:</strong> Monaco Editor with syntax highlighting</li>
              <li>• <strong>Real-time Validation:</strong> Instant feedback on YAML syntax and structure</li>
              <li>• <strong>Natural Language Updates:</strong> Modify configuration using plain English</li>
              <li>• <strong>Configuration Preview:</strong> Visual representation of parsed configuration</li>
              <li>• <strong>Error Handling:</strong> Clear error messages and visual indicators</li>
            </ul>
          </div>

          <div className="mb-4 p-4 bg-green-50 rounded-lg border border-green-200">
            <h3 className="text-sm font-medium text-green-900 mb-2">
              Try These Natural Language Commands:
            </h3>
            <ul className="text-sm text-green-700 space-y-1">
              <li>• "Make all headings bold and use Arial font"</li>
              <li>• "Set paragraph line spacing to 1.5"</li>
              <li>• "Change H1 font size to 20pt"</li>
              <li>• "Set margins to 1 inch on all sides"</li>
              <li>• "Use Calibri font for code blocks"</li>
            </ul>
          </div>

          <ConfigEditor />
        </div>
      )}
    </div>
  );
}