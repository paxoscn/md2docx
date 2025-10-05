# Markdown to DOCX Converter - Frontend

A React-based web interface for converting Markdown files to Microsoft Word documents with customizable formatting.

## Features

- **File Upload**: Drag and drop interface for Markdown files
- **Configuration Editor**: YAML-based configuration with natural language support
- **Real-time Preview**: Live preview of configuration changes
- **Conversion**: Convert Markdown to DOCX with custom formatting
- **Download**: Download converted DOCX files

## Technology Stack

- **React 19** with TypeScript
- **Vite** for build tooling
- **Tailwind CSS** for styling
- **React Router** for navigation
- **Zustand** for state management
- **React Dropzone** for file uploads
- **Headless UI** for accessible components
- **Heroicons** for icons

## Getting Started

### Prerequisites

- Node.js 20.19+ or 22.12+
- npm or yarn

### Installation

```bash
npm install
```

### Development

```bash
npm run dev
```

The application will be available at `http://localhost:5173`

### Build

```bash
npm run build
```

### Preview Production Build

```bash
npm run preview
```

## Project Structure

```
src/
├── components/          # Reusable UI components
│   ├── Layout.tsx      # Main layout wrapper
│   ├── Header.tsx      # Application header
│   ├── FileUpload.tsx  # File upload component
│   ├── ConfigEditor.tsx # Configuration editor
│   └── ConversionPanel.tsx # Conversion interface
├── pages/              # Page components
│   └── HomePage.tsx    # Main application page
├── services/           # API services
│   └── api.ts         # Backend API client
├── stores/             # State management
│   └── useAppStore.ts # Main application store
├── types/              # TypeScript type definitions
│   └── index.ts       # Shared types
├── App.tsx            # Main application component
├── main.tsx           # Application entry point
└── index.css          # Global styles
```

## Configuration

The application expects a backend API running on `http://localhost:3000` with the following endpoints:

- `POST /api/convert` - Convert Markdown to DOCX
- `POST /api/config/update` - Update configuration with natural language
- `GET /api/health` - Health check

## Usage

1. **Upload File**: Drag and drop a Markdown file or click to select
2. **Configure**: Edit the YAML configuration or use natural language prompts
3. **Convert**: Click the convert button to process the file
4. **Download**: Download the generated DOCX file

## Development Notes

- The application uses Zustand for state management to keep things simple
- Tailwind CSS is configured for styling with a clean, modern design
- TypeScript is used throughout for type safety
- The build process includes linting and type checking