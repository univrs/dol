/**
 * MessageBus tests
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { MessageBus, createMessageBus, createMessagingLoa } from '../src/messagebus.js';

describe('MessageBus', () => {
  let bus: MessageBus;

  beforeEach(() => {
    bus = createMessageBus();
  });

  describe('registration', () => {
    it('should register Spirits', () => {
      bus.register('alice');
      bus.register('bob');

      expect(bus.isRegistered('alice')).toBe(true);
      expect(bus.isRegistered('bob')).toBe(true);
      expect(bus.isRegistered('unknown')).toBe(false);
    });

    it('should throw when registering duplicate Spirit', () => {
      bus.register('alice');

      expect(() => bus.register('alice')).toThrow(
        "Spirit 'alice' is already registered on the message bus"
      );
    });

    it('should unregister Spirits', () => {
      bus.register('alice');
      expect(bus.isRegistered('alice')).toBe(true);

      bus.unregister('alice');
      expect(bus.isRegistered('alice')).toBe(false);
    });

    it('should list registered Spirits', () => {
      bus.register('alice');
      bus.register('bob');
      bus.register('charlie');

      const spirits = bus.spirits();
      expect(spirits).toContain('alice');
      expect(spirits).toContain('bob');
      expect(spirits).toContain('charlie');
      expect(spirits.length).toBe(3);
    });
  });

  describe('send and recv', () => {
    beforeEach(() => {
      bus.register('ping');
      bus.register('pong');
    });

    it('should send and receive messages', () => {
      const payload = new Uint8Array([1, 2, 3, 4]);
      const success = bus.send('ping', 'pong', 1, payload);

      expect(success).toBe(true);

      const msg = bus.recv('pong');
      expect(msg).toBeDefined();
      expect(msg!.from).toBe('ping');
      expect(msg!.to).toBe('pong');
      expect(msg!.channel).toBe(1);
      expect(msg!.payload).toEqual(payload);
    });

    it('should return undefined when no message', () => {
      const msg = bus.recv('ping');
      expect(msg).toBeUndefined();
    });

    it('should fail to send to unknown Spirit', () => {
      const payload = new Uint8Array([1]);
      const success = bus.send('ping', 'unknown', 1, payload);

      expect(success).toBe(false);
    });

    it('should filter by channel', () => {
      bus.send('ping', 'pong', 1, new Uint8Array([1]));
      bus.send('ping', 'pong', 2, new Uint8Array([2]));
      bus.send('ping', 'pong', 1, new Uint8Array([3]));

      // Receive channel 2 first (even though channel 1 was sent first)
      const msg2 = bus.recv('pong', 2);
      expect(msg2).toBeDefined();
      expect(msg2!.channel).toBe(2);
      expect(msg2!.payload[0]).toBe(2);

      // Receive channel 1 messages in order
      const msg1a = bus.recv('pong', 1);
      expect(msg1a!.payload[0]).toBe(1);

      const msg1b = bus.recv('pong', 1);
      expect(msg1b!.payload[0]).toBe(3);

      // No more messages
      expect(bus.recv('pong')).toBeUndefined();
    });

    it('should receive any channel with channel 0', () => {
      bus.send('ping', 'pong', 1, new Uint8Array([1]));
      bus.send('ping', 'pong', 2, new Uint8Array([2]));

      // Channel 0 means any channel, receives in order
      const msg1 = bus.recv('pong', 0);
      expect(msg1!.channel).toBe(1);

      const msg2 = bus.recv('pong', 0);
      expect(msg2!.channel).toBe(2);
    });
  });

  describe('pending', () => {
    beforeEach(() => {
      bus.register('alice');
      bus.register('bob');
    });

    it('should count pending messages', () => {
      expect(bus.pending('alice')).toBe(0);

      bus.send('bob', 'alice', 1, new Uint8Array([1]));
      expect(bus.pending('alice')).toBe(1);

      bus.send('bob', 'alice', 2, new Uint8Array([2]));
      expect(bus.pending('alice')).toBe(2);

      bus.recv('alice');
      expect(bus.pending('alice')).toBe(1);
    });

    it('should count pending by channel', () => {
      bus.send('bob', 'alice', 1, new Uint8Array([1]));
      bus.send('bob', 'alice', 2, new Uint8Array([2]));
      bus.send('bob', 'alice', 1, new Uint8Array([3]));

      expect(bus.pending('alice', 1)).toBe(2);
      expect(bus.pending('alice', 2)).toBe(1);
      expect(bus.pending('alice', 3)).toBe(0);
      expect(bus.pending('alice', 0)).toBe(3); // 0 = all channels
    });

    it('should return 0 for unknown Spirit', () => {
      expect(bus.pending('unknown')).toBe(0);
    });
  });

  describe('peek', () => {
    beforeEach(() => {
      bus.register('alice');
    });

    it('should peek without removing message', () => {
      bus.send('bob', 'alice', 1, new Uint8Array([42]));

      const peeked = bus.peek('alice');
      expect(peeked).toBeDefined();
      expect(peeked!.payload[0]).toBe(42);

      // Message still there
      expect(bus.pending('alice')).toBe(1);

      // Can still receive it
      const received = bus.recv('alice');
      expect(received!.payload[0]).toBe(42);

      // Now it's gone
      expect(bus.pending('alice')).toBe(0);
    });

    it('should peek by channel', () => {
      bus.send('bob', 'alice', 1, new Uint8Array([1]));
      bus.send('bob', 'alice', 2, new Uint8Array([2]));

      const peeked = bus.peek('alice', 2);
      expect(peeked!.payload[0]).toBe(2);
    });
  });

  describe('clear', () => {
    beforeEach(() => {
      bus.register('alice');
      bus.register('bob');
    });

    it('should clear messages for a Spirit', () => {
      bus.send('bob', 'alice', 1, new Uint8Array([1]));
      bus.send('bob', 'alice', 2, new Uint8Array([2]));

      expect(bus.pending('alice')).toBe(2);

      bus.clear('alice');
      expect(bus.pending('alice')).toBe(0);
    });

    it('should clear all messages', () => {
      bus.send('alice', 'bob', 1, new Uint8Array([1]));
      bus.send('bob', 'alice', 1, new Uint8Array([2]));

      expect(bus.pending('alice')).toBe(1);
      expect(bus.pending('bob')).toBe(1);

      bus.clearAll();

      expect(bus.pending('alice')).toBe(0);
      expect(bus.pending('bob')).toBe(0);
    });
  });

  describe('message handlers', () => {
    beforeEach(() => {
      bus.register('alice');
    });

    it('should call handlers when message arrives', () => {
      const received: string[] = [];

      bus.onMessage('alice', (msg) => {
        received.push(`${msg.from}:${msg.channel}`);
      });

      bus.send('bob', 'alice', 1, new Uint8Array([1]));
      bus.send('charlie', 'alice', 2, new Uint8Array([2]));

      expect(received).toEqual(['bob:1', 'charlie:2']);
    });

    it('should support multiple handlers', () => {
      let count = 0;

      bus.onMessage('alice', () => count++);
      bus.onMessage('alice', () => count++);

      bus.send('bob', 'alice', 1, new Uint8Array([1]));

      expect(count).toBe(2);
    });
  });
});

describe('createMessageBus', () => {
  it('should create a new MessageBus', () => {
    const bus = createMessageBus();
    expect(bus).toBeInstanceOf(MessageBus);
  });

  it('should accept debug option', () => {
    const bus = createMessageBus({ debug: true });
    expect(bus).toBeInstanceOf(MessageBus);
  });
});

describe('createMessagingLoa', () => {
  let bus: MessageBus;

  beforeEach(() => {
    bus = createMessageBus();
    bus.register('ping');
    bus.register('pong');
  });

  it('should create a Loa with messaging capabilities', () => {
    const loa = createMessagingLoa(bus, 'ping');

    expect(loa.name).toBe('messaging:ping');
    expect(loa.version).toBe('1.0.0');
    expect(loa.capabilities).toContain('send');
    expect(loa.capabilities).toContain('recv');
    expect(loa.capabilities).toContain('pending');
  });

  it('should provide vudo_send, vudo_recv, vudo_pending functions', () => {
    const loa = createMessagingLoa(bus, 'ping');

    // Create mock context with memory
    const memory = new WebAssembly.Memory({ initial: 1 });
    const context = { memory };

    const funcs = loa.provides(context);

    expect(typeof funcs.vudo_send).toBe('function');
    expect(typeof funcs.vudo_recv).toBe('function');
    expect(typeof funcs.vudo_pending).toBe('function');
  });

  it('should send messages via vudo_send', () => {
    const loa = createMessagingLoa(bus, 'ping');

    const memory = new WebAssembly.Memory({ initial: 1 });
    const context = { memory };
    const funcs = loa.provides(context);

    // Write "pong" to memory at offset 0
    const memView = new Uint8Array(memory.buffer);
    const dest = new TextEncoder().encode('pong');
    memView.set(dest, 0);

    // Write payload at offset 100
    const payload = new Uint8Array([1, 2, 3]);
    memView.set(payload, 100);

    // Send message: to_ptr=0, to_len=4, channel=1, payload_ptr=100, payload_len=3
    const result = funcs.vudo_send(0, 4, 1, 100, 3);
    expect(result).toBe(1); // success

    // Verify message was received
    expect(bus.pending('pong')).toBe(1);

    const msg = bus.recv('pong');
    expect(msg!.from).toBe('ping');
    expect(msg!.channel).toBe(1);
    expect(Array.from(msg!.payload)).toEqual([1, 2, 3]);
  });

  it('should receive messages via vudo_recv', () => {
    const loa = createMessagingLoa(bus, 'pong');

    const memory = new WebAssembly.Memory({ initial: 1 });
    const context = { memory };
    const funcs = loa.provides(context);

    // Send a message to pong
    bus.send('ping', 'pong', 1, new Uint8Array([10, 20, 30]));

    // Receive it: channel=1, from_buf=0, from_max=32, payload_buf=100, payload_max=64
    const result = funcs.vudo_recv(1, 0, 32, 100, 64);
    expect(result).toBe(3); // payload length

    // Check memory contents
    const memView = new Uint8Array(memory.buffer);

    // From should be "ping" + null terminator
    const fromBytes = memView.slice(0, 5);
    expect(new TextDecoder().decode(fromBytes.slice(0, 4))).toBe('ping');
    expect(fromBytes[4]).toBe(0); // null terminator

    // Payload should be [10, 20, 30]
    expect(Array.from(memView.slice(100, 103))).toEqual([10, 20, 30]);
  });

  it('should return -1 when no message', () => {
    const loa = createMessagingLoa(bus, 'pong');

    const memory = new WebAssembly.Memory({ initial: 1 });
    const context = { memory };
    const funcs = loa.provides(context);

    const result = funcs.vudo_recv(1, 0, 32, 100, 64);
    expect(result).toBe(-1);
  });

  it('should check pending via vudo_pending', () => {
    const loa = createMessagingLoa(bus, 'pong');

    const memory = new WebAssembly.Memory({ initial: 1 });
    const context = { memory };
    const funcs = loa.provides(context);

    expect(funcs.vudo_pending(0)).toBe(0);

    bus.send('ping', 'pong', 1, new Uint8Array([1]));
    bus.send('ping', 'pong', 2, new Uint8Array([2]));

    expect(funcs.vudo_pending(0)).toBe(2); // all channels
    expect(funcs.vudo_pending(1)).toBe(1);
    expect(funcs.vudo_pending(2)).toBe(1);
  });
});
