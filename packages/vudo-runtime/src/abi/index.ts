/**
 * DOL ABI (Application Binary Interface) Module
 *
 * This module provides the core ABI types and interfaces for DOL WASM-based applications.
 * It defines the contract between the DOL runtime and compiled DOL programs.
 *
 * The ABI is based on the Rust dol-abi crate and provides TypeScript equivalents
 * for seamless interoperability across the host-guest boundary.
 *
 * @example
 * ```typescript
 * import {
 *   Message,
 *   Response,
 *   AbiError,
 *   QualifiedId,
 *   ABI_VERSION,
 *   IMPORT_MODULE
 * } from '@vudo/runtime/abi';
 *
 * // Create a qualified identifier
 * const id = new QualifiedId('container', 'exists', '1.0.0');
 * console.log(id.toString()); // "container.exists.1.0.0"
 *
 * // Create a message
 * const msg = new Message(
 *   'msg-1',
 *   'call',
 *   { function: 'init', args: [] },
 *   { source: 'host', destination: 'spirit-1' }
 * );
 *
 * // Create a response
 * const response = Response.success('msg-1', { result: 'ok' });
 * console.log(response.toJSON());
 *
 * // Handle errors
 * const error = AbiError.invalidConfig('Missing required field');
 * console.error(error.toString());
 * ```
 */

// Re-export all types from individual modules
export * from './types';
export * from './message';
export * from './error';

// Export ABI version and import module name (matching Rust dol-abi)
/**
 * ABI version string
 * Used for version negotiation between host and guest
 * @constant
 */
export const ABI_VERSION = '0.1.0';

/**
 * WASM import module name
 * This is the namespace used for WASM imports (e.g., IMPORT_MODULE.sendMessage)
 * @constant
 */
export const IMPORT_MODULE = 'vudo';

import { QualifiedId } from './types.js';
import { Message, Response } from './message.js';
import { AbiError, AbiErrorType } from './error.js';

/**
 * Type guard to check if a value is a QualifiedId
 * @param value - Value to check
 * @returns True if value is a QualifiedId
 */
export function isQualifiedId(value: unknown): value is QualifiedId {
  return value instanceof QualifiedId;
}

/**
 * Type guard to check if a value is a Message
 * @param value - Value to check
 * @returns True if value is a Message
 */
export function isMessage(value: unknown): value is Message {
  return value instanceof Message;
}

/**
 * Type guard to check if a value is a Response
 * @param value - Value to check
 * @returns True if value is a Response
 */
export function isResponse(value: unknown): value is Response {
  return value instanceof Response;
}

/**
 * Type guard to check if a value is an AbiError
 * @param value - Value to check
 * @returns True if value is an AbiError
 */
export function isAbiError(value: unknown): value is AbiError {
  return value instanceof AbiError;
}

/**
 * ABI compatibility checker
 * Verifies that host and guest are compatible versions
 */
export class AbiCompat {
  /**
   * Check if two ABI versions are compatible
   * Uses semantic versioning rules
   * @param hostVersion - Host ABI version
   * @param guestVersion - Guest ABI version
   * @returns True if versions are compatible
   */
  static compatible(hostVersion: string, guestVersion: string): boolean {
    const [hostMajor, hostMinor] = hostVersion.split('.').map(Number);
    const [guestMajor, guestMinor] = guestVersion.split('.').map(Number);

    // Major version must match
    if (hostMajor !== guestMajor) {
      return false;
    }

    // Minor version of host must be >= guest
    return hostMinor >= guestMinor;
  }

  /**
   * Get a version negotiation message
   * @param version - Version string
   * @returns Message requesting version negotiation
   */
  static versionMessage(version: string): Message {
    return new Message('version-check', 'version_check', { abi_version: version }, {
      source: 'host',
      destination: 'guest',
    });
  }
}

// Type definitions for common serialization patterns
/**
 * Serialized form of AbiError for transmission
 */
export interface SerializedAbiError {
  type: string;
  code: string;
  message: string;
  details?: unknown;
  stack?: string;
}

/**
 * Serialize an AbiError to JSON-compatible format
 * @param error - Error to serialize
 * @returns Serialized error
 */
export function serializeError(error: AbiError): SerializedAbiError {
  return error.toJSON();
}

/**
 * Deserialize an error from JSON format
 * @param data - Serialized error data
 * @returns AbiError instance
 */
export function deserializeError(data: SerializedAbiError): AbiError {
  return new AbiError(data.message, data.type as AbiErrorType, data.code, data.details);
}
