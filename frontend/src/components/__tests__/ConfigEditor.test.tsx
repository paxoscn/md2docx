import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import ConfigEditor from '../ConfigEditor';
import { useAppStore } from '../../stores/useAppStore';
import { ApiService } from '../../services/api';

// Mock the store
vi.mock('../../stores/useAppStore');
vi.mock('../../services/api');

const mockUseAppStore = vi.mocked(useAppStore);
const mockApiService = vi.mocked(ApiService);

describe('ConfigEditor', () => {
  const mockSetConfig = vi.fn();
  const mockSetError = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    
    // Mock the validation API
    mockApiService.validateConfig.mockResolvedValue({
      success: true,
      valid: true,
      parsed_config: {
        document: {
          page_size: { width: 595, height: 842 },
          margins: { top: 72, bottom: 72, left: 72, right: 72 },
          default_font: { name: 'Times New Roman', size: 12 },
        },
        styles: {
          headings: {
            1: {
              font: { name: 'Times New Roman', size: 18 },
              bold: true,
            },
          },
          paragraph: {
            font: { name: 'Times New Roman', size: 12 },
            line_spacing: 1.15,
            indent: 0,
          },
          code_block: {
            font: { name: 'Courier New', size: 10 },
            background_color: '#f5f5f5',
            border: true,
          },
          table: {
            border: true,
            header_background: '#f0f0f0',
            font: { name: 'Times New Roman', size: 11 },
          },
        },
        elements: {
          image: { max_width: 500, alignment: 'center' },
          list: { indent: 20, spacing: 6 },
          link: { color: '#0066cc', underline: true },
        },
      },
    });
    
    const validConfig = `document:
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

    mockUseAppStore.mockReturnValue({
      config: validConfig,
      setConfig: mockSetConfig,
      setError: mockSetError,
      selectedFile: null,
      setSelectedFile: vi.fn(),
      isConverting: false,
      setIsConverting: vi.fn(),
      error: null,
      convertedFile: null,
      setConvertedFile: vi.fn(),
    });
  });

  it('renders configuration editor with natural language input', () => {
    render(<ConfigEditor />);
    
    expect(screen.getByText('Configuration Editor')).toBeInTheDocument();
    expect(screen.getByText('Natural Language Configuration')).toBeInTheDocument();
    expect(screen.getByPlaceholderText(/Make all headings bold and increase font size/)).toBeInTheDocument();
  });

  it('validates YAML configuration in real-time', async () => {
    render(<ConfigEditor />);
    
    // Should show valid status for valid YAML
    await waitFor(() => {
      expect(screen.getByText('Valid')).toBeInTheDocument();
    });
  });

  it('shows error for invalid YAML', async () => {
    // Mock validation failure
    mockApiService.validateConfig.mockResolvedValue({
      success: true,
      valid: false,
      error: 'Invalid YAML syntax',
    });

    mockUseAppStore.mockReturnValue({
      config: 'invalid: yaml: [',
      setConfig: mockSetConfig,
      setError: mockSetError,
      selectedFile: null,
      setSelectedFile: vi.fn(),
      isConverting: false,
      setIsConverting: vi.fn(),
      error: null,
      convertedFile: null,
      setConvertedFile: vi.fn(),
    });

    render(<ConfigEditor />);
    
    await waitFor(() => {
      expect(screen.getByText('Invalid')).toBeInTheDocument();
    });
  });

  it('calls API service for natural language updates', async () => {
    mockApiService.updateConfig.mockResolvedValue({
      success: true,
      updated_config: 'updated: config',
    });

    render(<ConfigEditor />);
    
    const input = screen.getByPlaceholderText(/Make all headings bold and increase font size/);
    const button = screen.getByText('Update Config');
    
    fireEvent.change(input, { target: { value: 'Make headings bold' } });
    fireEvent.click(button);
    
    await waitFor(() => {
      expect(mockApiService.updateConfig).toHaveBeenCalledWith({
        config: expect.any(String),
        natural_language: 'Make headings bold',
      });
    });
  });
});