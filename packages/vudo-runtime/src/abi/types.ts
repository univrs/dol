/**
 * Type definitions for the DOL ABI
 *
 * These types mirror the Rust ABI definitions and are designed
 * to serialize/deserialize cleanly with serde_json.
 */

/**
 * A qualified identifier in DOL (e.g., "domain.property" or "domain.property.version")
 *
 * @example
 * const id = new QualifiedId('container', 'exists');
 * const idWithVersion = new QualifiedId('container', 'exists', '1.0.0');
 */
export class QualifiedId {
  /**
   * Domain part of the identifier
   */
  domain: string;

  /**
   * Property part of the identifier
   */
  property: string;

  /**
   * Optional version part (semantic version)
   */
  version?: string;

  /**
   * Create a new qualified identifier
   * @param domain - Domain part
   * @param property - Property part
   * @param version - Optional version part
   */
  constructor(domain: string, property: string, version?: string) {
    this.domain = domain;
    this.property = property;
    this.version = version;
  }

  /**
   * Convert to string representation (e.g., "domain.property.1.0.0")
   */
  toString(): string {
    if (this.version) {
      return `${this.domain}.${this.property}.${this.version}`;
    }
    return `${this.domain}.${this.property}`;
  }

  /**
   * Parse a qualified identifier string into a QualifiedId
   * @param input - String in format "domain.property" or "domain.property.version"
   * @returns Parsed QualifiedId
   */
  static parse(input: string): QualifiedId {
    const parts = input.split('.');
    if (parts.length < 2) {
      throw new Error(`Invalid qualified identifier: ${input}`);
    }

    const domain = parts[0];
    const property = parts[1];
    const version = parts.length > 2 ? parts.slice(2).join('.') : undefined;

    return new QualifiedId(domain, property, version);
  }
}

/**
 * Serializable version of QualifiedId for JSON serialization
 * Mirrors the Rust serde representation
 */
export interface QualifiedIdSerialized {
  domain: string;
  property: string;
  version?: string;
}

/**
 * Standard effect type for logging and tracing
 * Used to represent side effects and state changes
 */
export interface StandardEffect {
  /** Type of effect (e.g., "log", "state_change", "function_call") */
  effect_type: string;

  /** Payload data for the effect */
  payload: unknown;

  /** Unix timestamp of when the effect occurred */
  timestamp: number;
}

/**
 * Standard event type for event-driven communication
 * Compatible with the event bus architecture
 */
export interface StandardEvent {
  /** Unique event identifier */
  id: string;

  /** Event type classifier */
  event_type: string;

  /** Originating component/module */
  source: string;

  /** Event payload data */
  data: unknown;

  /** Unix timestamp of event creation */
  timestamp: number;

  /** Optional metadata key-value pairs */
  metadata?: Record<string, unknown>;
}

/**
 * Logging level enumeration
 * Ordered from least to most severe
 */
export enum LogLevel {
  Debug = 0,
  Info = 1,
  Warn = 2,
  Error = 3,
}

/**
 * Result code enumeration for operation status
 * Mirrors Rust enum with numeric discriminants
 */
export enum ResultCode {
  Success = 0,
  Error = 1,
  Pending = 2,
  Timeout = 3,
}

/**
 * Convert LogLevel enum value to string
 * @param level - LogLevel enum value
 * @returns String representation
 */
export function logLevelToString(level: LogLevel): string {
  switch (level) {
    case LogLevel.Debug:
      return 'DEBUG';
    case LogLevel.Info:
      return 'INFO';
    case LogLevel.Warn:
      return 'WARN';
    case LogLevel.Error:
      return 'ERROR';
    default:
      return 'UNKNOWN';
  }
}

/**
 * Convert string to LogLevel enum
 * @param str - String representation
 * @returns LogLevel enum value
 */
export function stringToLogLevel(str: string): LogLevel {
  switch (str.toUpperCase()) {
    case 'DEBUG':
      return LogLevel.Debug;
    case 'INFO':
      return LogLevel.Info;
    case 'WARN':
      return LogLevel.Warn;
    case 'ERROR':
      return LogLevel.Error;
    default:
      throw new Error(`Unknown log level: ${str}`);
  }
}

/**
 * Convert ResultCode enum value to string
 * @param code - ResultCode enum value
 * @returns String representation
 */
export function resultCodeToString(code: ResultCode): string {
  switch (code) {
    case ResultCode.Success:
      return 'SUCCESS';
    case ResultCode.Error:
      return 'ERROR';
    case ResultCode.Pending:
      return 'PENDING';
    case ResultCode.Timeout:
      return 'TIMEOUT';
    default:
      return 'UNKNOWN';
  }
}

/**
 * Convert string to ResultCode enum
 * @param str - String representation
 * @returns ResultCode enum value
 */
export function stringToResultCode(str: string): ResultCode {
  switch (str.toUpperCase()) {
    case 'SUCCESS':
      return ResultCode.Success;
    case 'ERROR':
      return ResultCode.Error;
    case 'PENDING':
      return ResultCode.Pending;
    case 'TIMEOUT':
      return ResultCode.Timeout;
    default:
      throw new Error(`Unknown result code: ${str}`);
  }
}
