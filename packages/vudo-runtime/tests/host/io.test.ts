/**
 * Tests for I/O host functions
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { IOHostFunctions } from '../../src/host/io.js';
import { LogLevel } from '../../src/abi/types.js';
import { MockWasmMemory, MockLogger, createTestString } from './test-utils.js';

describe('IOHostFunctions', () => {
  let memory: MockWasmMemory;
  let logger: MockLogger;
  let io: IOHostFunctions;

  beforeEach(() => {
    memory = new MockWasmMemory(2 * 1024 * 1024); // 2MB for large string tests
    logger = new MockLogger();
    io = new IOHostFunctions({ memory, logger });
  });

  describe('vudo_print', () => {
    it('should print a string without newline', () => {
      const { ptr, len } = createTestString(memory, 'Hello, World!');
      io.print(ptr, len);

      expect(logger.messages).toHaveLength(1);
      expect(logger.messages[0]).toEqual({
        level: 'PRINT',
        message: 'Hello, World!',
      });
    });

    it('should handle empty strings', () => {
      const { ptr, len } = createTestString(memory, '');
      io.print(ptr, len);

      expect(logger.messages).toHaveLength(1);
      expect(logger.messages[0].message).toBe('');
    });

    it('should handle UTF-8 strings', () => {
      const { ptr, len } = createTestString(memory, '你好世界 🌍');
      io.print(ptr, len);

      expect(logger.messages).toHaveLength(1);
      expect(logger.messages[0].message).toBe('你好世界 🌍');
    });

    it('should handle invalid pointer gracefully', () => {
      io.print(-1, 10);

      // Should log an error
      expect(logger.getMessages('ERROR').length).toBeGreaterThan(0);
      expect(logger.hasMessage('ERROR', /vudo_print failed/)).toBe(true);
    });

    it('should handle out of bounds gracefully', () => {
      // Try to read beyond memory bounds
      const beyondBounds = memory.buffer.byteLength;
      io.print(beyondBounds, 1000);

      // Should log an error
      expect(logger.getMessages('ERROR').length).toBeGreaterThan(0);
      expect(logger.hasMessage('ERROR', /vudo_print failed/)).toBe(true);
    });

    it('should handle maximum string length', () => {
      const largeStr = 'x'.repeat(1024 * 1024); // 1MB
      const { ptr, len } = createTestString(memory, largeStr);
      io.print(ptr, len);

      expect(logger.messages[0].message).toBe(largeStr);
    });

    it('should reject strings exceeding maximum length', () => {
      const tooLarge = 1024 * 1024 + 1;
      io.print(1024, tooLarge);

      // Should log an error about exceeding max
      expect(logger.getMessages('ERROR').length).toBeGreaterThan(0);
      expect(logger.hasMessage('ERROR', /vudo_print failed/)).toBe(true);
    });
  });

  describe('vudo_println', () => {
    it('should print a string with newline', () => {
      const { ptr, len } = createTestString(memory, 'Hello, World!');
      io.println(ptr, len);

      expect(logger.messages).toHaveLength(1);
      expect(logger.messages[0]).toEqual({
        level: 'PRINTLN',
        message: 'Hello, World!',
      });
    });

    it('should handle multi-line strings', () => {
      const multiline = 'Line 1\nLine 2\nLine 3';
      const { ptr, len } = createTestString(memory, multiline);
      io.println(ptr, len);

      expect(logger.messages[0].message).toBe(multiline);
    });

    it('should handle invalid length gracefully', () => {
      io.println(1024, -1);

      expect(logger.hasMessage('ERROR', 'vudo_println failed')).toBe(true);
    });
  });

  describe('vudo_log', () => {
    it('should log DEBUG messages', () => {
      const { ptr, len } = createTestString(memory, 'Debug message');
      io.log(LogLevel.Debug, ptr, len);

      expect(logger.messages).toHaveLength(1);
      expect(logger.messages[0]).toEqual({
        level: 'DEBUG',
        message: 'Debug message',
      });
    });

    it('should log INFO messages', () => {
      const { ptr, len } = createTestString(memory, 'Info message');
      io.log(LogLevel.Info, ptr, len);

      expect(logger.messages[0].level).toBe('INFO');
      expect(logger.messages[0].message).toBe('Info message');
    });

    it('should log WARN messages', () => {
      const { ptr, len } = createTestString(memory, 'Warning message');
      io.log(LogLevel.Warn, ptr, len);

      expect(logger.messages[0].level).toBe('WARN');
      expect(logger.messages[0].message).toBe('Warning message');
    });

    it('should log ERROR messages', () => {
      const { ptr, len } = createTestString(memory, 'Error message');
      io.log(LogLevel.Error, ptr, len);

      expect(logger.messages[0].level).toBe('ERROR');
      expect(logger.messages[0].message).toBe('Error message');
    });

    it('should handle invalid log levels', () => {
      const { ptr, len } = createTestString(memory, 'Test message');
      io.log(999, ptr, len); // Invalid level

      expect(logger.hasMessage('WARN', 'Invalid log level')).toBe(true);
      expect(logger.hasMessage('ERROR', 'Test message')).toBe(true);
    });

    it('should handle negative log levels', () => {
      const { ptr, len } = createTestString(memory, 'Test message');
      io.log(-1, ptr, len);

      expect(logger.hasMessage('WARN', 'Invalid log level')).toBe(true);
    });

    it('should handle log with invalid string', () => {
      io.log(LogLevel.Info, -1, 10);

      expect(logger.hasMessage('ERROR', 'vudo_log failed')).toBe(true);
    });
  });

  describe('vudo_error', () => {
    it('should log error messages', () => {
      const { ptr, len } = createTestString(memory, 'Critical error');
      io.error(ptr, len);

      expect(logger.messages).toHaveLength(1);
      expect(logger.messages[0]).toEqual({
        level: 'ERROR',
        message: 'Critical error',
      });
    });

    it('should handle multiple errors', () => {
      const { ptr: ptr1, len: len1 } = createTestString(memory, 'Error 1');
      const { ptr: ptr2, len: len2 } = createTestString(memory, 'Error 2');

      io.error(ptr1, len1);
      io.error(ptr2, len2);

      expect(logger.messages).toHaveLength(2);
      expect(logger.messages[0].message).toBe('Error 1');
      expect(logger.messages[1].message).toBe('Error 2');
    });

    it('should handle empty error message', () => {
      const { ptr, len } = createTestString(memory, '');
      io.error(ptr, len);

      expect(logger.messages[0].level).toBe('ERROR');
      expect(logger.messages[0].message).toBe('');
    });

    it('should handle invalid error message gracefully', () => {
      io.error(999999, 100);

      // Should log an error (either the original or a failure message)
      expect(logger.getMessages('ERROR').length).toBeGreaterThan(0);
    });
  });

  describe('buildImports', () => {
    it('should return all I/O functions', () => {
      const imports = io.buildImports();

      expect(imports).toHaveProperty('vudo_print');
      expect(imports).toHaveProperty('vudo_println');
      expect(imports).toHaveProperty('vudo_log');
      expect(imports).toHaveProperty('vudo_error');
    });

    it('should bind functions correctly', () => {
      const imports = io.buildImports();
      const { ptr, len } = createTestString(memory, 'Test');

      (imports.vudo_print as Function)(ptr, len);

      expect(logger.messages[0].message).toBe('Test');
    });
  });

  describe('edge cases', () => {
    it('should handle zero-length strings', () => {
      io.print(1024, 0);

      // Should not fail, just log empty string
      expect(logger.messages).toHaveLength(1);
    });

    it('should handle pointer at memory boundary', () => {
      const str = 'test';
      const lastValidPtr = memory.buffer.byteLength - str.length;
      memory.u8.set(new TextEncoder().encode(str), lastValidPtr);

      io.print(lastValidPtr, str.length);

      expect(logger.messages[0].message).toBe(str);
    });

    it('should validate string length independently of pointer', () => {
      io.print(1024, -5);

      // Should log an error
      expect(logger.getMessages('ERROR').length).toBeGreaterThan(0);
      expect(logger.hasMessage('ERROR', /vudo_print failed/)).toBe(true);
    });

    it('should handle very long log messages', () => {
      const longMsg = 'x'.repeat(10000);
      const { ptr, len } = createTestString(memory, longMsg);

      io.log(LogLevel.Info, ptr, len);

      expect(logger.messages[0].message).toBe(longMsg);
    });
  });

  describe('custom configuration', () => {
    it('should respect custom max string length', () => {
      const customIO = new IOHostFunctions({
        memory,
        logger,
        maxStringLength: 100,
      });

      customIO.print(1024, 101);

      // Should log an error about exceeding max
      expect(logger.getMessages('ERROR').length).toBeGreaterThan(0);
      expect(logger.hasMessage('ERROR', /vudo_print failed/)).toBe(true);
    });

    it('should allow strings up to max length', () => {
      const customIO = new IOHostFunctions({
        memory,
        logger,
        maxStringLength: 50,
      });

      const { ptr, len } = createTestString(memory, 'x'.repeat(50));
      customIO.print(ptr, len);

      expect(logger.messages[0].message.length).toBe(50);
    });
  });
});
