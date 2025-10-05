export interface ConversionConfig {
  document: DocumentConfig;
  styles: StyleConfig;
  elements: ElementConfig;
}

export interface DocumentConfig {
  page_size: PageSize;
  margins: Margins;
  default_font: FontConfig;
}

export interface StyleConfig {
  headings: Record<number, HeadingStyle>;
  paragraph: ParagraphStyle;
  code_block: CodeBlockStyle;
  table: TableStyle;
}

export interface ElementConfig {
  image: ImageConfig;
  list: ListConfig;
  link: LinkConfig;
}

export interface PageSize {
  width: number;
  height: number;
}

export interface Margins {
  top: number;
  bottom: number;
  left: number;
  right: number;
}

export interface FontConfig {
  name: string;
  size: number;
}

export interface HeadingStyle {
  font: FontConfig;
  bold: boolean;
  color?: string;
}

export interface ParagraphStyle {
  font: FontConfig;
  line_spacing: number;
  indent: number;
}

export interface CodeBlockStyle {
  font: FontConfig;
  background_color: string;
  border: boolean;
}

export interface TableStyle {
  border: boolean;
  header_background: string;
  font: FontConfig;
}

export interface ImageConfig {
  max_width: number;
  alignment: 'left' | 'center' | 'right';
}

export interface ListConfig {
  indent: number;
  spacing: number;
}

export interface LinkConfig {
  color: string;
  underline: boolean;
}

export interface ConvertRequest {
  markdown: string;
  config?: string;
  natural_language?: string;
}

export interface ConvertResponse {
  success: boolean;
  file_data?: ArrayBuffer;
  error?: string;
}

export interface ConfigUpdateRequest {
  config: string;
  natural_language: string;
}

export interface ConfigUpdateResponse {
  success: boolean;
  updated_config?: string;
  error?: string;
}