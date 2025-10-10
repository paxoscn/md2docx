import { create } from 'zustand';

interface AppState {
  // File upload state
  selectedFile: File | null;
  setSelectedFile: (file: File | null) => void;
  
  // Configuration state
  config: string;
  setConfig: (config: string) => void;
  
  // Conversion state
  isConverting: boolean;
  setIsConverting: (converting: boolean) => void;
  
  // Error state
  error: string | null;
  setError: (error: string | null) => void;
  
  // Success state
  convertedFile: ArrayBuffer | null;
  setConvertedFile: (file: ArrayBuffer | null) => void;
}

const defaultConfig = `document:
  page_size:
    width: 595
    height: 842
  margins:
    top: 72
    bottom: 72
    left: 72
    right: 72
  default_font:
    name: "Times New Roman"
    size: 12

styles:
  headings:
    1:
      font:
        name: "Times New Roman"
        size: 18
      bold: true
      numbering: "%1."
    2:
      font:
        name: "Times New Roman"
        size: 16
      bold: true
      numbering: "%1.%2."
    3:
      font:
        name: "Times New Roman"
        size: 14
      bold: true
      numbering: "%1.%2.%3"
  paragraph:
    font:
      name: "Times New Roman"
      size: 12
    line_spacing: 1.15
    indent: 0
  code_block:
    font:
      name: "Courier New"
      size: 10
    background_color: "#f5f5f5"
    border: true
  table:
    border: true
    header_background: "#f0f0f0"
    font:
      name: "Times New Roman"
      size: 11

elements:
  image:
    max_width: 500
    alignment: "center"
  list:
    indent: 20
    spacing: 6
  link:
    color: "#0066cc"
    underline: true`;

export const useAppStore = create<AppState>((set) => ({
  selectedFile: null,
  setSelectedFile: (file) => set({ selectedFile: file }),
  
  config: defaultConfig,
  setConfig: (config) => set({ config }),
  
  isConverting: false,
  setIsConverting: (converting) => set({ isConverting: converting }),
  
  error: null,
  setError: (error) => set({ error }),
  
  convertedFile: null,
  setConvertedFile: (file) => set({ convertedFile: file }),
}));