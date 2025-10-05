import { useRef, useEffect } from 'react';
import Editor from '@monaco-editor/react';
import type { editor } from 'monaco-editor';

interface YamlEditorProps {
  value: string;
  onChange: (value: string) => void;
  error?: string | null;
  height?: number;
}

export function YamlEditor({ value, onChange, error, height = 400 }: YamlEditorProps) {
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null);

  const handleEditorDidMount = (editor: editor.IStandaloneCodeEditor) => {
    editorRef.current = editor;
    
    // Configure editor options
    editor.updateOptions({
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      fontSize: 13,
      lineNumbers: 'on',
      renderWhitespace: 'selection',
      tabSize: 2,
      insertSpaces: true,
      wordWrap: 'on',
      automaticLayout: true,
    });
  };

  const handleEditorChange = (value: string | undefined) => {
    if (value !== undefined) {
      onChange(value);
    }
  };

  // Update editor decorations based on error
  useEffect(() => {
    if (editorRef.current && error) {
      const model = editorRef.current.getModel();
      if (model) {
        // Try to extract line number from error message
        const lineMatch = error.match(/line (\d+)/i);
        const line = lineMatch ? parseInt(lineMatch[1], 10) : 1;
        
        const decorations = editorRef.current.deltaDecorations([], [
          {
            range: {
              startLineNumber: line,
              startColumn: 1,
              endLineNumber: line,
              endColumn: model.getLineMaxColumn(line),
            },
            options: {
              isWholeLine: true,
              className: 'error-line',
              glyphMarginClassName: 'error-glyph',
              hoverMessage: { value: error },
            },
          },
        ]);

        // Clean up decorations after a delay
        const timeout = setTimeout(() => {
          if (editorRef.current) {
            editorRef.current.deltaDecorations(decorations, []);
          }
        }, 5000);

        return () => clearTimeout(timeout);
      }
    }
  }, [error]);

  return (
    <div className="relative">
      <div 
        className={`border rounded-md overflow-hidden config-validation-transition ${
          error ? 'border-red-300' : 'border-gray-300'
        }`}
      >
        <Editor
          height={height}
          defaultLanguage="yaml"
          value={value}
          onChange={handleEditorChange}
          onMount={handleEditorDidMount}
          theme="vs"
          options={{
            readOnly: false,
            domReadOnly: false,
            selectOnLineNumbers: true,
            roundedSelection: false,
            cursorStyle: 'line',
            automaticLayout: true,
          }}
        />
      </div>
      
      {error && (
        <div className="mt-2 p-2 bg-red-50 border border-red-200 rounded-md">
          <div className="flex">
            <div className="flex-shrink-0">
              <svg className="h-4 w-4 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
              </svg>
            </div>
            <div className="ml-2">
              <p className="text-sm text-red-700">{error}</p>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}