/**
 * Message types for host-to-ABI communication
 *
 * These types define the message protocol between the DOL runtime (host)
 * and WASM modules (guests). All types are JSON-serializable.
 */

/**
 * Header information for a message
 * Contains metadata about the message for routing and tracking
 */
export interface MessageHeader {
  /** Unique message identifier for request-response matching */
  id: string;

  /** Message type classifier (e.g., "init", "call", "event") */
  msg_type: string;

  /** Source identifier (host or module name) */
  source: string;

  /** Destination identifier (module or host) */
  destination: string;

  /** Unix timestamp of message creation */
  timestamp: number;

  /** Message version/protocol version */
  version: string;

  /** Optional correlation ID for tracing message chains */
  correlation_id?: string;

  /** Message priority (0-3, higher = more important) */
  priority?: number;

  /** Optional timeout in milliseconds */
  timeout_ms?: number;
}

/**
 * Message payload wrapper
 * Encapsulates the actual data being transmitted
 */
export interface MessagePayload {
  /** Payload data (can be any JSON-serializable value) */
  data: unknown;

  /** Optional encoding hint (e.g., "utf-8", "base64") */
  encoding?: string;

  /** Optional content type (e.g., "application/json") */
  content_type?: string;

  /** Optional compression hint (e.g., "gzip") */
  compression?: string;
}

/**
 * A message from the host to a WASM module or vice versa
 *
 * This is the primary communication unit in the ABI. It combines
 * header information with payload data in a single serializable structure.
 *
 * @example
 * const msg = new Message(
 *   '123',
 *   'call',
 *   { function: 'init', args: [] },
 *   { source: 'host', destination: 'spirit-1' }
 * );
 */
export class Message {
  /** Message header with metadata */
  header: MessageHeader;

  /** Message payload wrapper */
  payload: MessagePayload;

  /**
   * Create a new message
   * @param id - Unique message identifier
   * @param msg_type - Type of message
   * @param data - Payload data
   * @param options - Optional message configuration
   */
  constructor(
    id: string,
    msg_type: string,
    data: unknown,
    options?: {
      source?: string;
      destination?: string;
      timestamp?: number;
      version?: string;
      correlation_id?: string;
      priority?: number;
      timeout_ms?: number;
      encoding?: string;
      content_type?: string;
      compression?: string;
    }
  ) {
    const now = Date.now();

    this.header = {
      id,
      msg_type,
      source: options?.source || 'unknown',
      destination: options?.destination || 'unknown',
      timestamp: options?.timestamp || Math.floor(now / 1000),
      version: options?.version || '1.0.0',
      correlation_id: options?.correlation_id,
      priority: options?.priority,
      timeout_ms: options?.timeout_ms,
    };

    this.payload = {
      data,
      encoding: options?.encoding,
      content_type: options?.content_type,
      compression: options?.compression,
    };
  }

  /**
   * Serialize to JSON (for transmission)
   * @returns JSON string representation
   */
  toJSON(): string {
    return JSON.stringify({
      header: this.header,
      payload: this.payload,
    });
  }

  /**
   * Deserialize from JSON string
   * @param json - JSON string representation
   * @returns Parsed Message
   */
  static fromJSON(json: string): Message {
    const parsed = JSON.parse(json) as {
      header: MessageHeader;
      payload: MessagePayload;
    };

    const msg = new Message(parsed.header.id, parsed.header.msg_type, parsed.payload.data, {
      source: parsed.header.source,
      destination: parsed.header.destination,
      timestamp: parsed.header.timestamp,
      version: parsed.header.version,
      correlation_id: parsed.header.correlation_id,
      priority: parsed.header.priority,
      timeout_ms: parsed.header.timeout_ms,
      encoding: parsed.payload.encoding,
      content_type: parsed.payload.content_type,
      compression: parsed.payload.compression,
    });

    return msg;
  }

  /**
   * Create a response to this message
   * @param success - Whether the operation succeeded
   * @param data - Response data
   * @param error - Optional error message
   * @returns Response message
   */
  response(success: boolean, data: unknown, error?: string): Response {
    return new Response(this.header.id, success, data, {
      correlation_id: this.header.id,
      source: this.header.destination,
      destination: this.header.source,
      error,
    });
  }
}

/**
 * A response message from a WASM module to the host (or vice versa)
 *
 * Responses are typically created in reply to a Message and include
 * a status code and optional error information.
 *
 * @example
 * const response = new Response(
 *   'req-123',
 *   true,
 *   { result: 42 },
 *   { source: 'spirit-1', destination: 'host' }
 * );
 */
export class Response {
  /** Message header with metadata */
  header: MessageHeader;

  /** Response status: true for success, false for error */
  success: boolean;

  /** Response data payload */
  data: unknown;

  /** Optional error message */
  error?: string;

  /**
   * Create a new response
   * @param id - Unique response identifier (should match request)
   * @param success - Whether the operation succeeded
   * @param data - Response data
   * @param options - Optional response configuration
   */
  constructor(
    id: string,
    success: boolean,
    data: unknown,
    options?: {
      source?: string;
      destination?: string;
      timestamp?: number;
      version?: string;
      correlation_id?: string;
      priority?: number;
      error?: string;
    }
  ) {
    const now = Date.now();

    this.header = {
      id,
      msg_type: 'response',
      source: options?.source || 'unknown',
      destination: options?.destination || 'unknown',
      timestamp: options?.timestamp || Math.floor(now / 1000),
      version: options?.version || '1.0.0',
      correlation_id: options?.correlation_id || id,
      priority: options?.priority,
    };

    this.success = success;
    this.data = data;
    this.error = options?.error;
  }

  /**
   * Create a successful response with data
   * @param id - Response identifier
   * @param data - Response data
   * @param options - Optional configuration
   * @returns Response instance
   */
  static success(
    id: string,
    data: unknown,
    options?: {
      source?: string;
      destination?: string;
      version?: string;
      priority?: number;
    }
  ): Response {
    return new Response(id, true, data, {
      source: options?.source,
      destination: options?.destination,
      version: options?.version,
      priority: options?.priority,
    });
  }

  /**
   * Create a failed response with error message
   * @param id - Response identifier
   * @param error - Error message
   * @param options - Optional configuration
   * @returns Response instance
   */
  static error(
    id: string,
    error: string,
    options?: {
      source?: string;
      destination?: string;
      version?: string;
      priority?: number;
    }
  ): Response {
    return new Response(id, false, null, {
      source: options?.source,
      destination: options?.destination,
      version: options?.version,
      priority: options?.priority,
      error,
    });
  }

  /**
   * Serialize to JSON (for transmission)
   * @returns JSON string representation
   */
  toJSON(): string {
    return JSON.stringify({
      header: this.header,
      success: this.success,
      data: this.data,
      error: this.error,
    });
  }

  /**
   * Deserialize from JSON string
   * @param json - JSON string representation
   * @returns Parsed Response
   */
  static fromJSON(json: string): Response {
    const parsed = JSON.parse(json) as {
      header: MessageHeader;
      success: boolean;
      data: unknown;
      error?: string;
    };

    const response = new Response(parsed.header.id, parsed.success, parsed.data, {
      source: parsed.header.source,
      destination: parsed.header.destination,
      timestamp: parsed.header.timestamp,
      version: parsed.header.version,
      correlation_id: parsed.header.correlation_id,
      priority: parsed.header.priority,
      error: parsed.error,
    });

    return response;
  }
}

/**
 * Options for sending a message
 * Allows customization of timeout and retry behavior
 */
export interface SendOptions {
  /** Timeout in milliseconds (default: 30000) */
  timeout?: number;

  /** Number of retry attempts on failure (default: 3) */
  retries?: number;

  /** Delay between retries in milliseconds (default: 100) */
  retryDelay?: number;

  /** Whether to throw on error (default: true) */
  throwOnError?: boolean;
}

/**
 * Message handler callback type
 * Receives a message and returns a response
 */
export type MessageHandler = (message: Message) => Promise<Response>;

/**
 * Message filter predicate
 * Returns true if message should be processed
 */
export type MessageFilter = (message: Message) => boolean;
