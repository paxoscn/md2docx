/**
 * Numbering utilities for preview functionality
 */

export interface NumberingState {
  counters: number[];
}

export interface NumberingFormat {
  levels: number[];
  template: string;
}

/**
 * Parse numbering format string (e.g., "%1.%2." -> [1, 2])
 */
export function parseNumberingFormat(format: string): NumberingFormat | null {
  if (!format || format.trim() === '') {
    return null;
  }

  const levels: number[] = [];
  const placeholderRegex = /%(\d+)/g;
  let match;

  while ((match = placeholderRegex.exec(format)) !== null) {
    const level = parseInt(match[1], 10);
    if (level >= 1 && level <= 6) {
      levels.push(level);
    }
  }

  // Validate that levels are sequential starting from 1
  if (levels.length === 0) {
    return null;
  }

  const sortedLevels = [...levels].sort((a, b) => a - b);
  for (let i = 0; i < sortedLevels.length; i++) {
    if (sortedLevels[i] !== i + 1) {
      return null; // Levels must be sequential starting from 1
    }
  }

  return {
    levels: sortedLevels,
    template: format,
  };
}

/**
 * Create initial numbering state
 */
export function createNumberingState(): NumberingState {
  return {
    counters: [0, 0, 0, 0, 0, 0], // H1-H6 counters
  };
}

/**
 * Update numbering state for a heading level
 */
export function updateNumberingState(state: NumberingState, level: number): NumberingState {
  const newCounters = [...state.counters];
  
  // Increment current level
  if (level >= 1 && level <= 6) {
    newCounters[level - 1]++;
    
    // Reset all lower levels
    for (let i = level; i < 6; i++) {
      newCounters[i] = 0;
    }
  }
  
  return {
    counters: newCounters,
  };
}

/**
 * Format numbering based on format string and current state
 */
export function formatNumbering(format: string, state: NumberingState): string {
  const parsedFormat = parseNumberingFormat(format);
  if (!parsedFormat) {
    return '';
  }

  let result = parsedFormat.template;
  
  for (const level of parsedFormat.levels) {
    const counter = state.counters[level - 1];
    result = result.replace(`%${level}`, counter.toString());
  }
  
  return result;
}

/**
 * Validate numbering format
 */
export function validateNumberingFormat(format: string): { valid: boolean; error?: string } {
  if (!format || format.trim() === '') {
    return { valid: false, error: 'Numbering format cannot be empty' };
  }

  const parsedFormat = parseNumberingFormat(format);
  if (!parsedFormat) {
    return { valid: false, error: 'Invalid numbering format. Use placeholders like %1, %1.%2, etc.' };
  }

  // Check for valid level sequence
  const levels = parsedFormat.levels;
  if (levels.length === 0) {
    return { valid: false, error: 'No valid level placeholders found' };
  }

  // Ensure levels start from 1 and are sequential
  for (let i = 0; i < levels.length; i++) {
    if (levels[i] !== i + 1) {
      return { valid: false, error: 'Level placeholders must be sequential starting from %1' };
    }
  }

  return { valid: true };
}

/**
 * Generate sample headings with numbering for preview
 */
export function generateNumberingPreview(
  headingConfigs: Record<number, { numbering?: string }>
): Array<{ level: number; text: string; numbering: string }> {
  const state = createNumberingState();
  const preview: Array<{ level: number; text: string; numbering: string }> = [];
  
  // Sample heading structure for preview
  const sampleHeadings = [
    { level: 1, text: 'Introduction' },
    { level: 2, text: 'Overview' },
    { level: 2, text: 'Objectives' },
    { level: 3, text: 'Primary Goals' },
    { level: 3, text: 'Secondary Goals' },
    { level: 1, text: 'Methodology' },
    { level: 2, text: 'Approach' },
    { level: 3, text: 'Data Collection' },
    { level: 4, text: 'Survey Design' },
    { level: 2, text: 'Analysis' },
  ];

  let currentState = state;
  
  for (const heading of sampleHeadings) {
    currentState = updateNumberingState(currentState, heading.level);
    
    const config = headingConfigs[heading.level];
    let numbering = '';
    
    if (config?.numbering) {
      numbering = formatNumbering(config.numbering, currentState);
    }
    
    preview.push({
      level: heading.level,
      text: heading.text,
      numbering,
    });
  }
  
  return preview;
}