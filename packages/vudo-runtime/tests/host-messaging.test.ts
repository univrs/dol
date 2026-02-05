/**
 * Tests for messaging host functions
 */

import { describe, it, expect, beforeEach } from 'vitest';
import {
  MessageBroker,
  createMessagingHost,
  type IMessageBroker,
} from '../src/host/messaging.js';
import { ResultCode } from '../src/abi/types.js';

describe('MessageBroker', () => {
  let broker: MessageBroker;

  beforeEach(() => {
    broker = new MessageBroker();
  });

  describe('registration', () => {
    it('should register Spirits', () => {
      broker.register('alice');
      broker.register('bob');

      expect(broker.spirits()).toContain('alice');
      expect(broker.spirits()).toContain('bob');
    });

    it('should throw when registering duplicate Spirit', () => {
      broker.register('alice');
      expect(() => broker.register('alice')).toThrow(
        "Spirit 'alice' is already registered"
      );
    });

    it('should unregister Spirits', () => {
      broker.register('alice');
      broker.unregister('alice');
      expect(broker.spirits()).not.toContain('alice');
    });

    it('should free messages when unregistering', () => {
      broker.register('alice');
      broker.register('bob');

      broker.send('bob', 'alice', new Uint8Array([1, 2, 3]));
      expect(broker.pending('alice')).toBe(1);

      broker.unregister('alice');
      expect(broker.pending('alice')).toBe(0);
    });
  });

  describe('send and recv', () => {
    beforeEach(() => {
      broker.register('ping');
      broker.register('pong');
    });

    it('should send and receive messages', () => {
      const payload = new Uint8Array([1, 2, 3, 4]);
      const result = broker.send('ping', 'pong', payload);

      expect(result).toBe(ResultCode.Success);
      expect(broker.pending('pong')).toBe(1);

      const msg = broker.recv('pong', 0);
      expect(msg).toBeDefined();
      expect(msg!.sender).toBe('ping');
      expect(msg!.recipient).toBe('pong');
      expect(msg!.payload).toEqual(payload);
      expect(msg!.id).toBeGreaterThan(0);
      expect(msg!.timestamp).toBeGreaterThan(0);
    });

    it('should return null when no message', () => {
      const msg = broker.recv('ping', 0);
      expect(msg).toBeNull();
    });

    it('should return error when sending to unknown Spirit', () => {
      const result = broker.send('ping', 'unknown', new Uint8Array([1]));
      expect(result).toBe(ResultCode.Error);
    });

    it('should copy payload data', () => {
      const payload = new Uint8Array([1, 2, 3]);
      broker.send('ping', 'pong', payload);

      // Modify original
      payload[0] = 99;

      // Received message should have original data
      const msg = broker.recv('pong', 0);
      expect(msg!.payload[0]).toBe(1);
    });

    it('should queue multiple messages', () => {
      broker.send('ping', 'pong', new Uint8Array([1]));
      broker.send('ping', 'pong', new Uint8Array([2]));
      broker.send('ping', 'pong', new Uint8Array([3]));

      expect(broker.pending('pong')).toBe(3);

      const msg1 = broker.recv('pong', 0);
      expect(msg1!.payload[0]).toBe(1);

      const msg2 = broker.recv('pong', 0);
      expect(msg2!.payload[0]).toBe(2);

      const msg3 = broker.recv('pong', 0);
      expect(msg3!.payload[0]).toBe(3);

      expect(broker.pending('pong')).toBe(0);
    });
  });

  describe('pending', () => {
    beforeEach(() => {
      broker.register('alice');
      broker.register('bob');
    });

    it('should count pending messages', () => {
      expect(broker.pending('alice')).toBe(0);

      broker.send('bob', 'alice', new Uint8Array([1]));
      expect(broker.pending('alice')).toBe(1);

      broker.send('bob', 'alice', new Uint8Array([2]));
      expect(broker.pending('alice')).toBe(2);

      broker.recv('alice', 0);
      expect(broker.pending('alice')).toBe(1);
    });

    it('should return 0 for unknown Spirit', () => {
      expect(broker.pending('unknown')).toBe(0);
    });
  });

  describe('broadcast', () => {
    beforeEach(() => {
      broker.register('alice');
      broker.register('bob');
      broker.register('charlie');
    });

    it('should broadcast to all Spirits', () => {
      const payload = new Uint8Array([42]);
      const result = broker.broadcast('admin', payload);

      expect(result).toBe(ResultCode.Success);
      expect(broker.pending('alice')).toBe(1);
      expect(broker.pending('bob')).toBe(1);
      expect(broker.pending('charlie')).toBe(1);

      const msgAlice = broker.recv('alice', 0);
      expect(msgAlice!.sender).toBe('admin');
      expect(msgAlice!.payload[0]).toBe(42);
    });

    it('should return success when no Spirits registered', () => {
      const emptyBroker = new MessageBroker();
      const result = emptyBroker.broadcast('admin', new Uint8Array([1]));
      expect(result).toBe(ResultCode.Success);
    });
  });

  describe('freeMessage', () => {
    beforeEach(() => {
      broker.register('alice');
      broker.register('bob');
    });

    it('should free allocated messages', () => {
      broker.send('bob', 'alice', new Uint8Array([1, 2, 3]));
      const msg = broker.recv('alice', 0);

      expect(msg).toBeDefined();
      broker.freeMessage(msg!.id);

      // Should be idempotent
      broker.freeMessage(msg!.id);
    });

    it('should handle freeing non-existent message ID', () => {
      broker.freeMessage(99999);
      // Should not throw
    });
  });

  describe('clear', () => {
    beforeEach(() => {
      broker.register('alice');
      broker.register('bob');
    });

    it('should clear messages for a Spirit', () => {
      broker.send('bob', 'alice', new Uint8Array([1]));
      broker.send('bob', 'alice', new Uint8Array([2]));

      expect(broker.pending('alice')).toBe(2);

      broker.clear('alice');
      expect(broker.pending('alice')).toBe(0);
    });

    it('should clear all messages', () => {
      broker.send('alice', 'bob', new Uint8Array([1]));
      broker.send('bob', 'alice', new Uint8Array([2]));

      expect(broker.pending('alice')).toBe(1);
      expect(broker.pending('bob')).toBe(1);

      broker.clearAll();

      expect(broker.pending('alice')).toBe(0);
      expect(broker.pending('bob')).toBe(0);
    });
  });
});

describe('createMessagingHost', () => {
  let broker: IMessageBroker;
  let memory: WebAssembly.Memory;

  beforeEach(() => {
    broker = new MessageBroker();
    broker.register('ping');
    broker.register('pong');
    memory = new WebAssembly.Memory({ initial: 1 });
  });

  describe('vudo_send', () => {
    it('should send messages via host function', () => {
      const host = createMessagingHost(broker, 'ping', memory);
      const memView = new Uint8Array(memory.buffer);

      // Write target "pong" at offset 0
      const target = new TextEncoder().encode('pong');
      memView.set(target, 0);

      // Write payload at offset 100
      const payload = new Uint8Array([10, 20, 30]);
      memView.set(payload, 100);

      // Call vudo_send(target_ptr=0, target_len=4, msg_ptr=100, msg_len=3)
      const result = host.vudo_send(0, 4, 100, 3);

      expect(result).toBe(ResultCode.Success);
      expect(broker.pending('pong')).toBe(1);

      const msg = broker.recv('pong', 0);
      expect(msg!.sender).toBe('ping');
      expect(Array.from(msg!.payload)).toEqual([10, 20, 30]);
    });

    it('should return error for unknown target', () => {
      const host = createMessagingHost(broker, 'ping', memory);
      const memView = new Uint8Array(memory.buffer);

      const target = new TextEncoder().encode('unknown');
      memView.set(target, 0);

      const result = host.vudo_send(0, 7, 100, 0);
      expect(result).toBe(ResultCode.Error);
    });

    it('should handle empty payload', () => {
      const host = createMessagingHost(broker, 'ping', memory);
      const memView = new Uint8Array(memory.buffer);

      const target = new TextEncoder().encode('pong');
      memView.set(target, 0);

      const result = host.vudo_send(0, 4, 100, 0);
      expect(result).toBe(ResultCode.Success);

      const msg = broker.recv('pong', 0);
      expect(msg!.payload.length).toBe(0);
    });
  });

  describe('vudo_recv', () => {
    it('should receive messages via host function', () => {
      const host = createMessagingHost(broker, 'pong', memory);

      // Send a message to pong
      broker.send('ping', 'pong', new Uint8Array([10, 20, 30]));

      // Receive: timeout_ms=0, out_ptr=0, out_len=1024
      const bytesWritten = host.vudo_recv(0, 0, 1024);

      expect(bytesWritten).toBeGreaterThan(0);

      // Read message from memory
      const dataView = new DataView(memory.buffer);
      const memView = new Uint8Array(memory.buffer);

      let offset = 0;

      // Read sender_len
      const senderLen = dataView.getUint32(offset, true);
      offset += 4;

      // Read sender
      const senderBytes = memView.slice(offset, offset + senderLen);
      const sender = new TextDecoder().decode(senderBytes);
      offset += senderLen;

      // Read payload_len
      const payloadLen = dataView.getUint32(offset, true);
      offset += 4;

      // Read payload
      const payload = Array.from(memView.slice(offset, offset + payloadLen));

      expect(sender).toBe('ping');
      expect(payload).toEqual([10, 20, 30]);
    });

    it('should return -1 when no messages', () => {
      const host = createMessagingHost(broker, 'pong', memory);
      const result = host.vudo_recv(0, 0, 1024);
      expect(result).toBe(-1);
    });

    it('should return -2 when buffer too small', () => {
      const host = createMessagingHost(broker, 'pong', memory);

      // Send a message
      broker.send('ping', 'pong', new Uint8Array([1, 2, 3, 4, 5]));

      // Try to receive with tiny buffer
      const result = host.vudo_recv(0, 0, 5);
      expect(result).toBe(-2);

      // Message should still be in queue
      expect(broker.pending('pong')).toBe(1);
    });

    it('should handle large messages', () => {
      const host = createMessagingHost(broker, 'pong', memory);

      // Send large payload
      const largePayload = new Uint8Array(1000);
      for (let i = 0; i < largePayload.length; i++) {
        largePayload[i] = i & 0xff;
      }

      broker.send('ping', 'pong', largePayload);

      const bytesWritten = host.vudo_recv(0, 0, 2048);
      expect(bytesWritten).toBeGreaterThan(1000);
    });
  });

  describe('vudo_pending', () => {
    it('should check pending messages', () => {
      const host = createMessagingHost(broker, 'pong', memory);

      expect(host.vudo_pending()).toBe(0);

      broker.send('ping', 'pong', new Uint8Array([1]));
      expect(host.vudo_pending()).toBe(1);

      broker.send('ping', 'pong', new Uint8Array([2]));
      expect(host.vudo_pending()).toBe(2);

      host.vudo_recv(0, 0, 1024);
      expect(host.vudo_pending()).toBe(1);
    });
  });

  describe('vudo_broadcast', () => {
    it('should broadcast messages', () => {
      broker.register('alice');
      broker.register('bob');
      const host = createMessagingHost(broker, 'admin', memory);

      const memView = new Uint8Array(memory.buffer);
      const payload = new Uint8Array([99, 88, 77]);
      memView.set(payload, 100);

      const result = host.vudo_broadcast(100, 3);

      expect(result).toBe(ResultCode.Success);
      expect(broker.pending('ping')).toBe(1);
      expect(broker.pending('pong')).toBe(1);
      expect(broker.pending('alice')).toBe(1);
      expect(broker.pending('bob')).toBe(1);
    });

    it('should handle empty broadcast', () => {
      const host = createMessagingHost(broker, 'admin', memory);
      const result = host.vudo_broadcast(0, 0);
      expect(result).toBe(ResultCode.Success);
    });
  });

  describe('vudo_free_message', () => {
    it('should free messages', () => {
      const host = createMessagingHost(broker, 'pong', memory);

      broker.send('ping', 'pong', new Uint8Array([1, 2, 3]));
      const msg = broker.recv('pong', 0);

      expect(msg).toBeDefined();
      host.vudo_free_message(msg!.id);

      // Should be idempotent
      host.vudo_free_message(msg!.id);
    });
  });

  describe('integration scenarios', () => {
    it('should handle ping-pong pattern', () => {
      const hostPing = createMessagingHost(broker, 'ping', memory);
      const hostPong = createMessagingHost(broker, 'pong', memory);

      const memView = new Uint8Array(memory.buffer);

      // Ping sends to Pong
      const target1 = new TextEncoder().encode('pong');
      memView.set(target1, 0);
      memView.set(new Uint8Array([1]), 100);

      hostPing.vudo_send(0, 4, 100, 1);

      // Pong receives
      expect(hostPong.vudo_pending()).toBe(1);
      hostPong.vudo_recv(0, 200, 512);

      // Pong replies
      const target2 = new TextEncoder().encode('ping');
      memView.set(target2, 0);
      memView.set(new Uint8Array([2]), 100);

      hostPong.vudo_send(0, 4, 100, 1);

      // Ping receives reply
      expect(hostPing.vudo_pending()).toBe(1);
      hostPing.vudo_recv(0, 300, 512);

      expect(broker.pending('ping')).toBe(0);
      expect(broker.pending('pong')).toBe(0);
    });

    it('should handle multi-Spirit communication', () => {
      broker.register('alice');
      broker.register('bob');
      broker.register('charlie');

      const hostAlice = createMessagingHost(broker, 'alice', memory);
      const hostBob = createMessagingHost(broker, 'bob', memory);
      const hostCharlie = createMessagingHost(broker, 'charlie', memory);

      const memView = new Uint8Array(memory.buffer);

      // Alice sends to Bob
      memView.set(new TextEncoder().encode('bob'), 0);
      memView.set([1], 100);
      hostAlice.vudo_send(0, 3, 100, 1);

      // Bob sends to Charlie
      memView.set(new TextEncoder().encode('charlie'), 0);
      memView.set([2], 100);
      hostBob.vudo_send(0, 7, 100, 1);

      // Charlie broadcasts
      memView.set([3], 100);
      hostCharlie.vudo_broadcast(100, 1);

      expect(hostAlice.vudo_pending()).toBe(1); // From Charlie's broadcast
      expect(hostBob.vudo_pending()).toBe(2); // From Alice + Charlie's broadcast
      expect(hostCharlie.vudo_pending()).toBe(2); // From Bob + self broadcast
    });
  });
});
