import { DocumentTextIcon } from '@heroicons/react/24/outline';

export default function Header() {
  return (
    <header className="bg-white shadow-sm border-b border-gray-200">
      <div className="container mx-auto px-4 py-3">
        <div className="flex items-center space-x-3">
          <DocumentTextIcon className="h-6 w-6 text-blue-600" />
          <div>
            <h1 className="text-xl font-bold text-gray-900">
              Markdown to DOCX Converter
            </h1>
            <p className="text-xs text-gray-600 hidden sm:block">
              Convert your Markdown files to Word documents with custom formatting
            </p>
          </div>
        </div>
      </div>
    </header>
  );
}