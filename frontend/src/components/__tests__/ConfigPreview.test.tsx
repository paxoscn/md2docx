import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ConfigPreview } from '../ConfigPreview';
import type { ConversionConfig } from '../../types';

const mockConfig: ConversionConfig = {
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
        numbering: '%1.',
      },
      2: {
        font: { name: 'Times New Roman', size: 16 },
        bold: true,
        numbering: '%1.%2.',
      },
      3: {
        font: { name: 'Times New Roman', size: 14 },
        bold: true,
        numbering: '%1.%2.%3',
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
};

const mockConfigWithErrors: ConversionConfig = {
  ...mockConfig,
  styles: {
    ...mockConfig.styles,
    headings: {
      ...mockConfig.styles.headings,
      1: {
        ...mockConfig.styles.headings[1],
        numbering: '%1.%3.', // Invalid: skips level 2
      },
      2: {
        ...mockConfig.styles.headings[2],
        numbering: 'invalid format', // Invalid: no placeholders
      },
    },
  },
};

describe('ConfigPreview', () => {
  it('should render loading state', () => {
    render(<ConfigPreview config={null} error={null} isValidating={true} />);
    
    expect(screen.getByText('Validating configuration...')).toBeInTheDocument();
  });

  it('should render error state', () => {
    const error = 'Invalid YAML syntax';
    render(<ConfigPreview config={null} error={error} isValidating={false} />);
    
    expect(screen.getByText('Configuration Error:')).toBeInTheDocument();
    expect(screen.getByText(error)).toBeInTheDocument();
  });

  it('should render null config state', () => {
    render(<ConfigPreview config={null} error={null} isValidating={false} />);
    
    expect(screen.getByText('No valid configuration to preview.')).toBeInTheDocument();
  });

  it('should render valid configuration with numbering', () => {
    render(<ConfigPreview config={mockConfig} error={null} isValidating={false} />);
    
    // Check that configuration is marked as valid
    expect(screen.getByText('Configuration is valid and ready to use')).toBeInTheDocument();
    
    // Check heading styles section
    expect(screen.getByText('Heading Styles')).toBeInTheDocument();
    
    // Check that numbering sections exist
    expect(screen.getAllByText('Numbering:').length).toBeGreaterThan(0);
    
    // Check numbering preview section
    expect(screen.getByText('Numbering Preview')).toBeInTheDocument();
    expect(screen.getByText('This preview shows how numbering will appear in your document')).toBeInTheDocument();
    
    // Check some preview entries
    expect(screen.getByText('1.')).toBeInTheDocument();
    expect(screen.getByText('1.1.')).toBeInTheDocument();
    expect(screen.getByText('1.2.')).toBeInTheDocument();
    expect(screen.getByText('Introduction')).toBeInTheDocument();
    expect(screen.getByText('Overview')).toBeInTheDocument();
  });

  it('should display numbering errors', () => {
    render(<ConfigPreview config={mockConfigWithErrors} error={null} isValidating={false} />);
    
    // Check that numbering errors section is displayed
    expect(screen.getByText('Numbering Configuration Errors')).toBeInTheDocument();
    expect(screen.getByText('Please fix these errors for proper numbering functionality')).toBeInTheDocument();
    
    // Check specific error messages
    expect(screen.getAllByText((content, element) => {
      return element?.textContent?.includes('H1:') || false;
    })[0]).toBeInTheDocument();
    expect(screen.getAllByText((content, element) => {
      return element?.textContent?.includes('H2:') || false;
    })[0]).toBeInTheDocument();
  });

  it('should not show numbering preview when no numbering is configured', () => {
    const configWithoutNumbering: ConversionConfig = {
      ...mockConfig,
      styles: {
        ...mockConfig.styles,
        headings: {
          1: {
            font: { name: 'Times New Roman', size: 18 },
            bold: true,
            // No numbering field
          },
          2: {
            font: { name: 'Times New Roman', size: 16 },
            bold: true,
            // No numbering field
          },
        },
      },
    };

    render(<ConfigPreview config={configWithoutNumbering} error={null} isValidating={false} />);
    
    // Numbering preview should not be shown
    expect(screen.queryByText('Numbering Preview')).not.toBeInTheDocument();
    
    // Numbering errors should not be shown
    expect(screen.queryByText('Numbering Configuration Errors')).not.toBeInTheDocument();
  });

  it('should show mixed numbering configuration correctly', () => {
    const mixedConfig: ConversionConfig = {
      ...mockConfig,
      styles: {
        ...mockConfig.styles,
        headings: {
          1: {
            font: { name: 'Times New Roman', size: 18 },
            bold: true,
            numbering: '%1.',
          },
          2: {
            font: { name: 'Times New Roman', size: 16 },
            bold: true,
            // No numbering for level 2
          },
          3: {
            font: { name: 'Times New Roman', size: 14 },
            bold: true,
            numbering: '%1.%2.%3',
          },
        },
      },
    };

    render(<ConfigPreview config={mixedConfig} error={null} isValidating={false} />);
    
    // Should show numbering preview (since some levels have numbering)
    expect(screen.getByText('Numbering Preview')).toBeInTheDocument();
    
    // Should show some numbering elements
    expect(screen.getAllByText('Numbering:').length).toBeGreaterThan(0);
    
    // H2 should not show numbering info - check that there are fewer numbering elements than heading levels
    const numberingElements = screen.getAllByText('Numbering:');
    expect(numberingElements.length).toBeLessThan(3); // Should be 2 since H2 has no numbering
  });

  it('should handle document settings display', () => {
    render(<ConfigPreview config={mockConfig} error={null} isValidating={false} />);
    
    expect(screen.getByText('Document Settings')).toBeInTheDocument();
    expect(screen.getByText('595 × 842')).toBeInTheDocument();
    expect(screen.getByText('Times New Roman 12pt')).toBeInTheDocument();
    expect(screen.getByText('T:72 R:72 B:72 L:72')).toBeInTheDocument();
  });

  it('should handle text styles display', () => {
    render(<ConfigPreview config={mockConfig} error={null} isValidating={false} />);
    
    expect(screen.getByText('Text Styles')).toBeInTheDocument();
    expect(screen.getByText(/Times New Roman 12pt, Line: 1.15x/)).toBeInTheDocument();
    expect(screen.getByText(/Courier New 10pt/)).toBeInTheDocument();
  });

  it('should handle element settings display', () => {
    render(<ConfigPreview config={mockConfig} error={null} isValidating={false} />);
    
    expect(screen.getByText('Element Settings')).toBeInTheDocument();
    expect(screen.getByText(/Max: 500px, Align: center/)).toBeInTheDocument();
    expect(screen.getByText(/Indent: 20px, Spacing: 6px/)).toBeInTheDocument();
  });
});