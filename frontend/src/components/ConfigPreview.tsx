import type { ConversionConfig } from '../types';

interface ConfigPreviewProps {
  config: ConversionConfig | null;
  error: string | null;
  isValidating: boolean;
}

export function ConfigPreview({ config, error, isValidating }: ConfigPreviewProps) {
  if (isValidating) {
    return (
      <div className="bg-gray-50 rounded-lg p-4">
        <h3 className="text-sm font-medium text-gray-900 mb-2">
          Configuration Preview
        </h3>
        <div className="flex items-center text-sm text-gray-600">
          <svg className="animate-spin -ml-1 mr-2 h-4 w-4 text-gray-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          Validating configuration...
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 rounded-lg p-4 border border-red-200">
        <h3 className="text-sm font-medium text-red-900 mb-2">
          Configuration Preview
        </h3>
        <div className="text-sm text-red-700">
          <p className="font-medium">Configuration Error:</p>
          <p className="mt-1">{error}</p>
          <p className="mt-2 text-xs">Please fix the YAML syntax errors above to see the preview.</p>
        </div>
      </div>
    );
  }

  if (!config) {
    return (
      <div className="bg-gray-50 rounded-lg p-4">
        <h3 className="text-sm font-medium text-gray-900 mb-2">
          Configuration Preview
        </h3>
        <div className="text-sm text-gray-600">
          <p>No valid configuration to preview.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-green-50 rounded-lg p-4 border border-green-200">
      <h3 className="text-sm font-medium text-green-900 mb-4">
        Configuration Preview
      </h3>
      
      <div className="space-y-4 text-sm">
        {/* Document Settings */}
        <div className="bg-white rounded-md p-3 border border-green-200">
          <h4 className="font-medium text-green-900 mb-2">Document Settings</h4>
          <div className="grid grid-cols-2 gap-3 text-xs">
            <div>
              <span className="text-gray-600">Page Size:</span>
              <span className="ml-1 font-mono">
                {config.document.page_size.width} × {config.document.page_size.height}
              </span>
            </div>
            <div>
              <span className="text-gray-600">Default Font:</span>
              <span className="ml-1 font-mono">
                {config.document.default_font.name} {config.document.default_font.size}pt
              </span>
            </div>
            <div className="col-span-2">
              <span className="text-gray-600">Margins:</span>
              <span className="ml-1 font-mono">
                T:{config.document.margins.top} R:{config.document.margins.right} B:{config.document.margins.bottom} L:{config.document.margins.left}
              </span>
            </div>
          </div>
        </div>

        {/* Heading Styles */}
        <div className="bg-white rounded-md p-3 border border-green-200">
          <h4 className="font-medium text-green-900 mb-2">Heading Styles</h4>
          <div className="space-y-2">
            {Object.entries(config.styles.headings).map(([level, style]) => (
              <div key={level} className="flex items-center justify-between text-xs">
                <span className="text-gray-600">H{level}:</span>
                <span className="font-mono">
                  {style.font.name} {style.font.size}pt
                  {style.bold && <span className="ml-1 font-bold">Bold</span>}
                  {style.color && <span className="ml-1" style={{ color: style.color }}>●</span>}
                </span>
              </div>
            ))}
          </div>
        </div>

        {/* Text Styles */}
        <div className="bg-white rounded-md p-3 border border-green-200">
          <h4 className="font-medium text-green-900 mb-2">Text Styles</h4>
          <div className="space-y-2 text-xs">
            <div className="flex justify-between">
              <span className="text-gray-600">Paragraph:</span>
              <span className="font-mono">
                {config.styles.paragraph.font.name} {config.styles.paragraph.font.size}pt, 
                Line: {config.styles.paragraph.line_spacing}x
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600">Code Block:</span>
              <span className="font-mono">
                {config.styles.code_block.font.name} {config.styles.code_block.font.size}pt
                <span 
                  className="ml-1 px-1 rounded text-xs"
                  style={{ backgroundColor: config.styles.code_block.background_color }}
                >
                  BG
                </span>
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600">Table:</span>
              <span className="font-mono">
                {config.styles.table.font.name} {config.styles.table.font.size}pt
                {config.styles.table.border && <span className="ml-1">Border</span>}
              </span>
            </div>
          </div>
        </div>

        {/* Element Settings */}
        <div className="bg-white rounded-md p-3 border border-green-200">
          <h4 className="font-medium text-green-900 mb-2">Element Settings</h4>
          <div className="space-y-2 text-xs">
            <div className="flex justify-between">
              <span className="text-gray-600">Images:</span>
              <span className="font-mono">
                Max: {config.elements.image.max_width}px, 
                Align: {config.elements.image.alignment}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600">Lists:</span>
              <span className="font-mono">
                Indent: {config.elements.list.indent}px, 
                Spacing: {config.elements.list.spacing}px
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600">Links:</span>
              <span className="font-mono">
                <span style={{ color: config.elements.link.color }}>Color</span>
                {config.elements.link.underline && <span className="ml-1 underline">Underline</span>}
              </span>
            </div>
          </div>
        </div>

        {/* Status */}
        <div className="flex items-center text-green-700">
          <svg className="w-4 h-4 mr-2" fill="currentColor" viewBox="0 0 20 20">
            <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
          </svg>
          <span className="text-sm font-medium">Configuration is valid and ready to use</span>
        </div>
      </div>
    </div>
  );
}