/**
 * Error types for the DOL ABI
 *
 * These types mirror the Rust error enum and provide structured
 * error handling across the host-guest boundary.
 */

/**
 * Discriminant enum for ABI error types
 * Must match the Rust enum variant order
 */
export enum AbiErrorType {
  InvalidConfig = 'InvalidConfig',
  InvalidMessage = 'InvalidMessage',
  HostError = 'HostError',
  TypeMismatch = 'TypeMismatch',
  Other = 'Other',
}

/**
 * Base ABI error class
 *
 * All ABI errors inherit from this class. It provides structured
 * error information that can be serialized for transmission across
 * the host-guest boundary.
 *
 * @example
 * try {
 *   // Some operation
 * } catch (err) {
 *   if (err instanceof AbiError) {
 *     console.error(`${err.type}: ${err.message}`);
 *   }
 * }
 */
export class AbiError extends Error {
  /** Type of error (for serialization and matching) */
  type: AbiErrorType;

  /** Error code for programmatic handling */
  code: string;

  /** Nested error details (if any) */
  details?: unknown;

  /** Stack trace context */
  context?: string;

  /**
   * Create a new AbiError
   * @param message - Human-readable error message
   * @param type - Error type discriminant
   * @param code - Error code for programmatic matching
   * @param details - Optional error details
   */
  constructor(
    message: string,
    type: AbiErrorType = AbiErrorType.Other,
    code: string = 'UNKNOWN_ERROR',
    details?: unknown
  ) {
    super(message);
    this.name = 'AbiError';
    this.type = type;
    this.code = code;
    this.details = details;
    this.context = new Error().stack;

    // Maintain proper prototype chain
    Object.setPrototypeOf(this, AbiError.prototype);
  }

  /**
   * Serialize error to JSON-compatible object
   * Suitable for transmission in Response messages
   */
  toJSON(): {
    type: string;
    code: string;
    message: string;
    details?: unknown;
    stack?: string;
  } {
    return {
      type: this.type,
      code: this.code,
      message: this.message,
      details: this.details,
      stack: this.context,
    };
  }

  /**
   * Serialize error to string for logging
   */
  toString(): string {
    const parts = [`[${this.type}:${this.code}]`, this.message];
    if (this.details) {
      parts.push(`Details: ${JSON.stringify(this.details)}`);
    }
    return parts.join(' ');
  }

  /**
   * Create an InvalidConfig error
   * @param message - Error message
   * @param details - Optional error details
   * @returns AbiError instance
   */
  static invalidConfig(message: string, details?: unknown): AbiError {
    return new AbiError(message, AbiErrorType.InvalidConfig, 'INVALID_CONFIG', details);
  }

  /**
   * Create an InvalidMessage error
   * @param message - Error message
   * @param details - Optional error details
   * @returns AbiError instance
   */
  static invalidMessage(message: string, details?: unknown): AbiError {
    return new AbiError(message, AbiErrorType.InvalidMessage, 'INVALID_MESSAGE', details);
  }

  /**
   * Create a HostError
   * @param message - Error message
   * @param details - Optional error details
   * @returns AbiError instance
   */
  static hostError(message: string, details?: unknown): AbiError {
    return new AbiError(message, AbiErrorType.HostError, 'HOST_ERROR', details);
  }

  /**
   * Create a TypeMismatch error
   * @param expected - Expected type
   * @param received - Received type
   * @param details - Optional error details
   * @returns AbiError instance
   */
  static typeMismatch(expected: string, received: string, details?: unknown): AbiError {
    const message = `Expected ${expected} but received ${received}`;
    return new AbiError(message, AbiErrorType.TypeMismatch, 'TYPE_MISMATCH', {
      expected,
      received,
      ...details,
    });
  }

  /**
   * Create a generic error
   * @param message - Error message
   * @param details - Optional error details
   * @returns AbiError instance
   */
  static other(message: string, details?: unknown): AbiError {
    return new AbiError(message, AbiErrorType.Other, 'ERROR', details);
  }
}

/**
 * Result type for ABI operations
 * Mirrors Rust's Result<T, E> pattern
 *
 * @example
 * const result: AbiResult<number> = {
 *   ok: true,
 *   value: 42
 * };
 *
 * const error: AbiResult<number> = {
 *   ok: false,
 *   error: new AbiError('Operation failed')
 * };
 */
export type AbiResult<T> = {
  ok: true;
  value: T;
} | {
  ok: false;
  error: AbiError;
};

/**
 * Create a successful result
 * @param value - Result value
 * @returns AbiResult with ok=true
 */
export function ok<T>(value: T): AbiResult<T> {
  return { ok: true, value };
}

/**
 * Create an error result
 * @param error - Error value
 * @returns AbiResult with ok=false
 */
export function err<T>(error: AbiError | string): AbiResult<T> {
  const abiError = typeof error === 'string' ? AbiError.other(error) : error;
  return { ok: false, error: abiError };
}

/**
 * Check if a result is ok
 * @param result - Result to check
 * @returns True if ok
 */
export function isOk<T>(result: AbiResult<T>): result is { ok: true; value: T } {
  return result.ok === true;
}

/**
 * Check if a result is an error
 * @param result - Result to check
 * @returns True if error
 */
export function isErr<T>(result: AbiResult<T>): result is { ok: false; error: AbiError } {
  return result.ok === false;
}

/**
 * Extract value from result or throw
 * @param result - Result to unwrap
 * @returns Value if ok
 * @throws {AbiError} If result is error
 */
export function unwrap<T>(result: AbiResult<T>): T {
  if (isOk(result)) {
    return result.value;
  }
  throw result.error;
}

/**
 * Extract value with default fallback
 * @param result - Result to unwrap
 * @param defaultValue - Default value if error
 * @returns Value if ok, otherwise defaultValue
 */
export function unwrapOr<T>(result: AbiResult<T>, defaultValue: T): T {
  if (isOk(result)) {
    return result.value;
  }
  return defaultValue;
}

/**
 * Transform result value with a mapping function
 * @param result - Result to map
 * @param fn - Mapping function
 * @returns New result with mapped value
 */
export function map<T, U>(result: AbiResult<T>, fn: (value: T) => U): AbiResult<U> {
  if (isOk(result)) {
    return ok(fn(result.value));
  }
  return result as unknown as AbiResult<U>;
}

/**
 * Chain results with a function that returns a result
 * @param result - Result to chain
 * @param fn - Function returning a result
 * @returns Chained result
 */
export function flatMap<T, U>(result: AbiResult<T>, fn: (value: T) => AbiResult<U>): AbiResult<U> {
  if (isOk(result)) {
    return fn(result.value);
  }
  return result as unknown as AbiResult<U>;
}

/**
 * Helper to convert promises to AbiResult
 * @param promise - Promise to convert
 * @returns AbiResult of the resolved value
 */
export async function fromPromise<T>(promise: Promise<T>): Promise<AbiResult<T>> {
  try {
    return ok(await promise);
  } catch (error) {
    const abiError =
      error instanceof AbiError
        ? error
        : AbiError.other(error instanceof Error ? error.message : String(error));
    return err(abiError);
  }
}
