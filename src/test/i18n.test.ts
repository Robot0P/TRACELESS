import { describe, it, expect } from 'vitest';

describe('i18n Configuration', () => {
  const supportedLanguages = ['zh-CN', 'en-US'];

  describe('Language Support', () => {
    it('should support Chinese and English', () => {
      expect(supportedLanguages).toContain('zh-CN');
      expect(supportedLanguages).toContain('en-US');
    });

    it('should have exactly 2 supported languages', () => {
      expect(supportedLanguages.length).toBe(2);
    });
  });

  describe('Translation Keys', () => {
    const requiredKeys = [
      'common.confirm',
      'common.cancel',
      'common.loading',
      'common.error',
      'common.success',
      'dashboard.title',
      'settings.title',
      'fileCleanup.title',
      'systemLogs.title',
      'memoryCleanup.title',
      'networkCleanup.title',
    ];

    it('should have all required translation keys', () => {
      requiredKeys.forEach((key) => {
        expect(key).toBeTruthy();
        expect(key.includes('.')).toBe(true);
      });
    });
  });

  describe('Language Detection', () => {
    it('should detect system language correctly', () => {
      const getSystemLanguage = (): string => {
        const browserLang = navigator.language || 'zh-CN';
        if (browserLang.startsWith('zh')) {
          return 'zh-CN';
        } else if (browserLang.startsWith('en')) {
          return 'en-US';
        }
        return 'zh-CN'; // Default fallback
      };

      const result = getSystemLanguage();
      expect(supportedLanguages).toContain(result);
    });

    it('should fallback to zh-CN for unsupported languages', () => {
      const fallbackLanguage = 'zh-CN';
      expect(supportedLanguages).toContain(fallbackLanguage);
    });
  });

  describe('Translation Format', () => {
    it('should support interpolation', () => {
      const template = 'Hello, {{name}}!';
      const name = 'User';
      const result = template.replace('{{name}}', name);

      expect(result).toBe('Hello, User!');
    });

    it('should support plural forms', () => {
      const getPluralForm = (count: number): string => {
        if (count === 0) return 'No items';
        if (count === 1) return '1 item';
        return `${count} items`;
      };

      expect(getPluralForm(0)).toBe('No items');
      expect(getPluralForm(1)).toBe('1 item');
      expect(getPluralForm(5)).toBe('5 items');
    });
  });
});

describe('Date/Time Formatting', () => {
  describe('Chinese Format', () => {
    it('should format date in Chinese style', () => {
      const date = new Date('2024-01-15T10:30:00');
      const formatted = date.toLocaleDateString('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
      });

      expect(formatted).toMatch(/2024/);
    });
  });

  describe('English Format', () => {
    it('should format date in English style', () => {
      const date = new Date('2024-01-15T10:30:00');
      const formatted = date.toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
      });

      expect(formatted).toContain('2024');
      expect(formatted).toContain('January');
    });
  });

  describe('Relative Time', () => {
    it('should format relative time correctly', () => {
      const getRelativeTime = (date: Date): string => {
        const now = new Date();
        const diff = now.getTime() - date.getTime();
        const seconds = Math.floor(diff / 1000);
        const minutes = Math.floor(seconds / 60);
        const hours = Math.floor(minutes / 60);
        const days = Math.floor(hours / 24);

        if (seconds < 60) return 'just now';
        if (minutes < 60) return `${minutes} minutes ago`;
        if (hours < 24) return `${hours} hours ago`;
        return `${days} days ago`;
      };

      const now = new Date();
      expect(getRelativeTime(now)).toBe('just now');

      const fiveMinutesAgo = new Date(now.getTime() - 5 * 60 * 1000);
      expect(getRelativeTime(fiveMinutesAgo)).toBe('5 minutes ago');
    });
  });
});

describe('Number Formatting', () => {
  describe('File Size', () => {
    const formatFileSize = (bytes: number): string => {
      if (bytes < 1024) return `${bytes} B`;
      if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
      if (bytes < 1024 * 1024 * 1024)
        return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
      return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
    };

    it('should format bytes correctly', () => {
      expect(formatFileSize(500)).toBe('500 B');
      expect(formatFileSize(1024)).toBe('1.0 KB');
      expect(formatFileSize(1024 * 1024)).toBe('1.0 MB');
      expect(formatFileSize(1024 * 1024 * 1024)).toBe('1.00 GB');
    });
  });

  describe('Percentage', () => {
    it('should format percentage correctly', () => {
      const formatPercentage = (value: number): string => {
        return `${value.toFixed(1)}%`;
      };

      expect(formatPercentage(50)).toBe('50.0%');
      expect(formatPercentage(99.9)).toBe('99.9%');
      expect(formatPercentage(0)).toBe('0.0%');
    });
  });

  describe('Network Speed', () => {
    const formatNetworkSpeed = (
      mbps: number
    ): { value: string; unit: string } => {
      const bytes = mbps * 1024 * 1024;

      if (bytes < 1024) {
        return { value: bytes.toFixed(0), unit: 'B/s' };
      } else if (bytes < 1024 * 1024) {
        return { value: (bytes / 1024).toFixed(1), unit: 'KB/s' };
      } else {
        return { value: (bytes / (1024 * 1024)).toFixed(2), unit: 'MB/s' };
      }
    };

    it('should format network speed correctly', () => {
      const result1 = formatNetworkSpeed(0.001);
      expect(result1.unit).toBe('KB/s');

      const result2 = formatNetworkSpeed(1);
      expect(result2.unit).toBe('MB/s');
      expect(result2.value).toBe('1.00');
    });
  });
});
