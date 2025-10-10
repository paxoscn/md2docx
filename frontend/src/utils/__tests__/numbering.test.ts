import { describe, it, expect } from 'vitest';
import {
  parseNumberingFormat,
  createNumberingState,
  updateNumberingState,
  formatNumbering,
  validateNumberingFormat,
  generateNumberingPreview,
} from '../numbering';

describe('parseNumberingFormat', () => {
  it('should parse valid single-level format', () => {
    const result = parseNumberingFormat('%1.');
    expect(result).toEqual({
      levels: [1],
      template: '%1.',
    });
  });

  it('should parse valid multi-level format', () => {
    const result = parseNumberingFormat('%1.%2.');
    expect(result).toEqual({
      levels: [1, 2],
      template: '%1.%2.',
    });
  });

  it('should parse three-level format', () => {
    const result = parseNumberingFormat('%1.%2.%3');
    expect(result).toEqual({
      levels: [1, 2, 3],
      template: '%1.%2.%3',
    });
  });

  it('should return null for empty format', () => {
    expect(parseNumberingFormat('')).toBeNull();
    expect(parseNumberingFormat('   ')).toBeNull();
  });

  it('should return null for format without placeholders', () => {
    expect(parseNumberingFormat('no placeholders')).toBeNull();
  });

  it('should return null for non-sequential levels', () => {
    expect(parseNumberingFormat('%1.%3.')).toBeNull();
    expect(parseNumberingFormat('%2.%3.')).toBeNull();
  });

  it('should return null for invalid level numbers', () => {
    expect(parseNumberingFormat('%0.')).toBeNull();
    expect(parseNumberingFormat('%7.')).toBeNull();
  });
});

describe('createNumberingState', () => {
  it('should create initial state with all counters at 0', () => {
    const state = createNumberingState();
    expect(state.counters).toEqual([0, 0, 0, 0, 0, 0]);
  });
});

describe('updateNumberingState', () => {
  it('should increment level 1 counter', () => {
    let state = createNumberingState();
    state = updateNumberingState(state, 1);
    expect(state.counters).toEqual([1, 0, 0, 0, 0, 0]);
  });

  it('should increment level 2 counter and reset lower levels', () => {
    let state = createNumberingState();
    state = updateNumberingState(state, 1);
    state = updateNumberingState(state, 2);
    state = updateNumberingState(state, 3);
    state = updateNumberingState(state, 2);
    expect(state.counters).toEqual([1, 2, 0, 0, 0, 0]);
  });

  it('should handle level transitions correctly', () => {
    let state = createNumberingState();
    state = updateNumberingState(state, 1); // 1.
    state = updateNumberingState(state, 2); // 1.1.
    state = updateNumberingState(state, 2); // 1.2.
    state = updateNumberingState(state, 1); // 2.
    expect(state.counters).toEqual([2, 0, 0, 0, 0, 0]);
  });

  it('should ignore invalid levels', () => {
    let state = createNumberingState();
    const originalState = { ...state };
    state = updateNumberingState(state, 0);
    expect(state.counters).toEqual(originalState.counters);
    
    state = updateNumberingState(state, 7);
    expect(state.counters).toEqual(originalState.counters);
  });
});

describe('formatNumbering', () => {
  it('should format single-level numbering', () => {
    let state = createNumberingState();
    state = updateNumberingState(state, 1);
    const result = formatNumbering('%1.', state);
    expect(result).toBe('1.');
  });

  it('should format multi-level numbering', () => {
    let state = createNumberingState();
    state = updateNumberingState(state, 1);
    state = updateNumberingState(state, 2);
    const result = formatNumbering('%1.%2.', state);
    expect(result).toBe('1.1.');
  });

  it('should format three-level numbering', () => {
    let state = createNumberingState();
    state = updateNumberingState(state, 1);
    state = updateNumberingState(state, 2);
    state = updateNumberingState(state, 3);
    const result = formatNumbering('%1.%2.%3', state);
    expect(result).toBe('1.1.1');
  });

  it('should return empty string for invalid format', () => {
    const state = createNumberingState();
    expect(formatNumbering('', state)).toBe('');
    expect(formatNumbering('invalid', state)).toBe('');
  });

  it('should handle custom separators', () => {
    let state = createNumberingState();
    state = updateNumberingState(state, 1);
    state = updateNumberingState(state, 2);
    const result = formatNumbering('%1-%2', state);
    expect(result).toBe('1-1');
  });
});

describe('validateNumberingFormat', () => {
  it('should validate correct formats', () => {
    expect(validateNumberingFormat('%1.')).toEqual({ valid: true });
    expect(validateNumberingFormat('%1.%2.')).toEqual({ valid: true });
    expect(validateNumberingFormat('%1.%2.%3')).toEqual({ valid: true });
    expect(validateNumberingFormat('%1-%2-%3')).toEqual({ valid: true });
  });

  it('should reject empty formats', () => {
    const result = validateNumberingFormat('');
    expect(result.valid).toBe(false);
    expect(result.error).toContain('cannot be empty');
  });

  it('should reject formats without placeholders', () => {
    const result = validateNumberingFormat('no placeholders');
    expect(result.valid).toBe(false);
    expect(result.error).toContain('Invalid numbering format');
  });

  it('should reject non-sequential levels', () => {
    const result = validateNumberingFormat('%1.%3.');
    expect(result.valid).toBe(false);
    expect(result.error).toContain('Invalid numbering format');
  });

  it('should reject formats starting with wrong level', () => {
    const result = validateNumberingFormat('%2.%3.');
    expect(result.valid).toBe(false);
    expect(result.error).toContain('Invalid numbering format');
  });
});

describe('generateNumberingPreview', () => {
  it('should generate preview with numbering', () => {
    const headingConfigs = {
      1: { numbering: '%1.' },
      2: { numbering: '%1.%2.' },
      3: { numbering: '%1.%2.%3' },
    };

    const preview = generateNumberingPreview(headingConfigs);
    
    expect(preview.length).toBeGreaterThan(0);
    
    // Check first few headings
    expect(preview[0]).toEqual({
      level: 1,
      text: 'Introduction',
      numbering: '1.',
    });
    
    expect(preview[1]).toEqual({
      level: 2,
      text: 'Overview',
      numbering: '1.1.',
    });
    
    expect(preview[2]).toEqual({
      level: 2,
      text: 'Objectives',
      numbering: '1.2.',
    });
    
    expect(preview[3]).toEqual({
      level: 3,
      text: 'Primary Goals',
      numbering: '1.2.1',
    });
  });

  it('should handle mixed numbering configurations', () => {
    const headingConfigs = {
      1: { numbering: '%1.' },
      2: {}, // No numbering for level 2
      3: { numbering: '%1.%2.%3' },
    };

    const preview = generateNumberingPreview(headingConfigs);
    
    // Level 1 should have numbering
    const level1Headings = preview.filter(h => h.level === 1);
    expect(level1Headings[0].numbering).toBe('1.');
    
    // Level 2 should have no numbering
    const level2Headings = preview.filter(h => h.level === 2);
    expect(level2Headings[0].numbering).toBe('');
    
    // Level 3 should have numbering (but will reflect actual state progression)
    const level3Headings = preview.filter(h => h.level === 3);
    expect(level3Headings[0].numbering).toBe('1.2.1');
  });

  it('should handle empty configuration', () => {
    const preview = generateNumberingPreview({});
    
    expect(preview.length).toBeGreaterThan(0);
    preview.forEach(heading => {
      expect(heading.numbering).toBe('');
    });
  });
});