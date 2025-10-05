import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import ConversionPanel from '../ConversionPanel';
import { useAppStore } from '../../stores/useAppStore';

// Mock the API service
vi.mock('../../services/api', () => ({
  ApiService: {
    convertMarkdown: vi.fn(),
  },
}));

// Mock the store
vi.mock('../../stores/useAppStore');

const mockUseAppStore = vi.mocked(useAppStore);

describe('ConversionPanel', () => {
  const mockSetIsConverting = vi.fn();
  const mockSetConvertedFile = vi.fn();
  const mockSetError = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    
    mockUseAppStore.mockReturnValue({
      selectedFile: null,
      config: 'test config',
      isConverting: false,
      setIsConverting: mockSetIsConverting,
      convertedFile: null,
      setConvertedFile: mockSetConvertedFile,
      error: null,
      setError: mockSetError,
      setSelectedFile: vi.fn(),
      setConfig: vi.fn(),
    });
  });

  it('renders conversion panel with disabled convert button when no file selected', () => {
    render(<ConversionPanel />);
    
    const convertButton = screen.getByRole('button', { name: 'Convert to DOCX' });
    expect(convertButton).toBeInTheDocument();
    expect(convertButton).toBeDisabled();
    expect(screen.getByText('No file selected')).toBeInTheDocument();
  });

  it('enables convert button when file is selected', () => {
    const mockFile = new File(['# Test'], 'test.md', { type: 'text/markdown' });
    
    mockUseAppStore.mockReturnValue({
      selectedFile: mockFile,
      config: 'test config',
      isConverting: false,
      setIsConverting: mockSetIsConverting,
      convertedFile: null,
      setConvertedFile: mockSetConvertedFile,
      error: null,
      setError: mockSetError,
      setSelectedFile: vi.fn(),
      setConfig: vi.fn(),
    });

    render(<ConversionPanel />);
    
    const convertButton = screen.getByRole('button', { name: 'Convert to DOCX' });
    expect(convertButton).toBeEnabled();
    expect(screen.getByText('test.md')).toBeInTheDocument();
  });

  it('shows converting state when conversion is in progress', () => {
    const mockFile = new File(['# Test'], 'test.md', { type: 'text/markdown' });
    
    mockUseAppStore.mockReturnValue({
      selectedFile: mockFile,
      config: 'test config',
      isConverting: true,
      setIsConverting: mockSetIsConverting,
      convertedFile: null,
      setConvertedFile: mockSetConvertedFile,
      error: null,
      setError: mockSetError,
      setSelectedFile: vi.fn(),
      setConfig: vi.fn(),
    });

    render(<ConversionPanel />);
    
    expect(screen.getByText('Converting...')).toBeInTheDocument();
    expect(screen.getByText(/Processing your markdown file/)).toBeInTheDocument();
  });

  it('shows download button when conversion is completed', () => {
    const mockFile = new File(['# Test'], 'test.md', { type: 'text/markdown' });
    const mockArrayBuffer = new ArrayBuffer(1024);
    
    mockUseAppStore.mockReturnValue({
      selectedFile: mockFile,
      config: 'test config',
      isConverting: false,
      setIsConverting: mockSetIsConverting,
      convertedFile: mockArrayBuffer,
      setConvertedFile: mockSetConvertedFile,
      error: null,
      setError: mockSetError,
      setSelectedFile: vi.fn(),
      setConfig: vi.fn(),
    });

    render(<ConversionPanel />);
    
    expect(screen.getByText('Conversion completed successfully!')).toBeInTheDocument();
    expect(screen.getByText('Download DOCX File')).toBeInTheDocument();
    expect(screen.getByText('Convert Another')).toBeInTheDocument();
    expect(screen.getByText('File size: 1.0 KB')).toBeInTheDocument();
  });

  it('shows error message when conversion fails', () => {
    const mockFile = new File(['# Test'], 'test.md', { type: 'text/markdown' });
    
    mockUseAppStore.mockReturnValue({
      selectedFile: mockFile,
      config: 'test config',
      isConverting: false,
      setIsConverting: mockSetIsConverting,
      convertedFile: null,
      setConvertedFile: mockSetConvertedFile,
      error: 'Conversion failed: Network error',
      setError: mockSetError,
      setSelectedFile: vi.fn(),
      setConfig: vi.fn(),
    });

    render(<ConversionPanel />);
    
    expect(screen.getByText('Conversion Error')).toBeInTheDocument();
    expect(screen.getByText('Conversion failed: Network error')).toBeInTheDocument();
    expect(screen.getByText('Try Again')).toBeInTheDocument();
    expect(screen.getByText('Dismiss')).toBeInTheDocument();
  });

  it('calls setError(null) when dismiss button is clicked', () => {
    const mockFile = new File(['# Test'], 'test.md', { type: 'text/markdown' });
    
    mockUseAppStore.mockReturnValue({
      selectedFile: mockFile,
      config: 'test config',
      isConverting: false,
      setIsConverting: mockSetIsConverting,
      convertedFile: null,
      setConvertedFile: mockSetConvertedFile,
      error: 'Test error',
      setError: mockSetError,
      setSelectedFile: vi.fn(),
      setConfig: vi.fn(),
    });

    render(<ConversionPanel />);
    
    fireEvent.click(screen.getByText('Dismiss'));
    expect(mockSetError).toHaveBeenCalledWith(null);
  });

  it('resets state when Convert Another button is clicked', () => {
    const mockFile = new File(['# Test'], 'test.md', { type: 'text/markdown' });
    const mockArrayBuffer = new ArrayBuffer(1024);
    
    mockUseAppStore.mockReturnValue({
      selectedFile: mockFile,
      config: 'test config',
      isConverting: false,
      setIsConverting: mockSetIsConverting,
      convertedFile: mockArrayBuffer,
      setConvertedFile: mockSetConvertedFile,
      error: null,
      setError: mockSetError,
      setSelectedFile: vi.fn(),
      setConfig: vi.fn(),
    });

    render(<ConversionPanel />);
    
    fireEvent.click(screen.getByText('Convert Another'));
    
    expect(mockSetConvertedFile).toHaveBeenCalledWith(null);
    expect(mockSetError).toHaveBeenCalledWith(null);
  });
});