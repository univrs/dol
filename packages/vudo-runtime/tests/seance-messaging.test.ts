/**
 * Séance Messaging Tests - Inter-Spirit Communication
 *
 * These tests validate the Ping/Pong pattern where Spirits communicate
 * via the MessageBus integrated into the Séance.
 */

import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { Seance, createSeance, withSeance } from '../src/seance.js';
import { loadSpirit } from '../src/spirit.js';

// Minimal WASM module that exports an 'add' function
// Used as a placeholder Spirit for messaging tests
const MINIMAL_WASM = new Uint8Array([
  0x00, 0x61, 0x73, 0x6d, // magic
  0x01, 0x00, 0x00, 0x00, // version
  0x01, 0x07,             // type section
  0x01, 0x60, 0x02, 0x7e, 0x7e, 0x01, 0x7e, // (func (param i64 i64) (result i64))
  0x03, 0x02,             // func section
  0x01, 0x00,             // function 0 uses type 0
  0x07, 0x07,             // export section
  0x01, 0x03, 0x61, 0x64, 0x64, 0x00, 0x00, // export "add" func 0
  0x0a, 0x09,             // code section
  0x01, 0x07, 0x00,       // function body
  0x20, 0x00,             // local.get 0
  0x20, 0x01,             // local.get 1
  0x7c,                   // i64.add
  0x0b,                   // end
]);

describe('Séance Messaging', () => {
  let seance: Seance;

  beforeEach(() => {
    seance = createSeance({ debug: false });
  });

  afterEach(async () => {
    await seance.dismiss();
  });

  describe('Ping/Pong Pattern', () => {
    it('should allow sending messages between Spirits', async () => {
      // Summon two Spirits
      await seance.summon('ping', MINIMAL_WASM);
      await seance.summon('pong', MINIMAL_WASM);

      // Send a message from ping to pong
      const payload = new Uint8Array([42, 43, 44]);
      const success = seance.send('ping', 'pong', 1, payload);

      expect(success).toBe(true);
      expect(seance.pending('pong')).toBe(1);
    });

    it('should track pending messages per Spirit', async () => {
      await seance.summon('ping', MINIMAL_WASM);
      await seance.summon('pong', MINIMAL_WASM);

      // Send multiple messages
      seance.send('ping', 'pong', 1, new Uint8Array([1]));
      seance.send('ping', 'pong', 2, new Uint8Array([2]));
      seance.send('pong', 'ping', 1, new Uint8Array([3]));

      expect(seance.pending('pong')).toBe(2);
      expect(seance.pending('ping')).toBe(1);
      expect(seance.pending('pong', 1)).toBe(1);
      expect(seance.pending('pong', 2)).toBe(1);
    });

    it('should receive messages via MessageBus', async () => {
      await seance.summon('ping', MINIMAL_WASM);
      await seance.summon('pong', MINIMAL_WASM);

      // Send a message
      seance.send('ping', 'pong', 1, new Uint8Array([10, 20, 30]));

      // Receive via MessageBus
      const msg = seance.messages.recv('pong');

      expect(msg).toBeDefined();
      expect(msg!.from).toBe('ping');
      expect(msg!.to).toBe('pong');
      expect(msg!.channel).toBe(1);
      expect(Array.from(msg!.payload)).toEqual([10, 20, 30]);
    });

    it('should simulate Ping/Pong exchange', async () => {
      await seance.summon('ping', MINIMAL_WASM);
      await seance.summon('pong', MINIMAL_WASM);

      // Channel definitions
      const CHANNEL_PING = 1;
      const CHANNEL_PONG = 2;

      // Ping sends to Pong
      seance.send('ping', 'pong', CHANNEL_PING, new Uint8Array([0x01]));

      // Pong receives and responds
      const pingMsg = seance.messages.recv('pong', CHANNEL_PING);
      expect(pingMsg).toBeDefined();
      expect(pingMsg!.from).toBe('ping');

      // Pong sends response back to Ping
      seance.send('pong', 'ping', CHANNEL_PONG, new Uint8Array([0x02]));

      // Ping receives response
      const pongMsg = seance.messages.recv('ping', CHANNEL_PONG);
      expect(pongMsg).toBeDefined();
      expect(pongMsg!.from).toBe('pong');
      expect(pongMsg!.payload[0]).toBe(0x02);
    });
  });

  describe('Broadcast', () => {
    it('should broadcast to all Spirits except sender', async () => {
      await seance.summon('coordinator', MINIMAL_WASM);
      await seance.summon('worker1', MINIMAL_WASM);
      await seance.summon('worker2', MINIMAL_WASM);
      await seance.summon('worker3', MINIMAL_WASM);

      // Broadcast from coordinator
      const delivered = seance.broadcast(
        'coordinator',
        1,
        new Uint8Array([0xFF])
      );

      expect(delivered).toBe(3); // All workers received

      // Each worker should have the message
      expect(seance.pending('worker1')).toBe(1);
      expect(seance.pending('worker2')).toBe(1);
      expect(seance.pending('worker3')).toBe(1);

      // Coordinator should not receive its own broadcast
      expect(seance.pending('coordinator')).toBe(0);
    });
  });

  describe('Message Cleanup on Release', () => {
    it('should clear messages when Spirit is released', async () => {
      await seance.summon('sender', MINIMAL_WASM);
      await seance.summon('receiver', MINIMAL_WASM);

      // Send some messages
      seance.send('sender', 'receiver', 1, new Uint8Array([1]));
      seance.send('sender', 'receiver', 1, new Uint8Array([2]));

      expect(seance.pending('receiver')).toBe(2);

      // Release the receiver
      await seance.release('receiver');

      // Spirit should be unregistered from MessageBus
      expect(seance.messages.isRegistered('receiver')).toBe(false);
    });

    it('should clear all messages on dismiss', async () => {
      await seance.summon('a', MINIMAL_WASM);
      await seance.summon('b', MINIMAL_WASM);

      seance.send('a', 'b', 1, new Uint8Array([1]));
      seance.send('b', 'a', 1, new Uint8Array([2]));

      // Dismiss clears everything
      await seance.dismiss();

      // After dismiss, MessageBus should be empty
      expect(seance.messages.spirits()).toEqual([]);
    });
  });

  describe('MessageBus Access', () => {
    it('should expose MessageBus via messages property', async () => {
      await seance.summon('test', MINIMAL_WASM);

      expect(seance.messages).toBeDefined();
      expect(seance.messages.isRegistered('test')).toBe(true);
    });
  });
});

describe('Multi-Spirit Coordination Patterns', () => {
  it('should support request/response pattern', async () => {
    await withSeance(async (seance) => {
      await seance.summon('client', MINIMAL_WASM);
      await seance.summon('server', MINIMAL_WASM);

      const REQUEST = 1;
      const RESPONSE = 2;

      // Client sends request
      seance.send('client', 'server', REQUEST, new Uint8Array([100]));

      // Server receives request
      const req = seance.messages.recv('server', REQUEST);
      expect(req).toBeDefined();

      // Server sends response
      const result = req!.payload[0] * 2; // Double the input
      seance.send('server', 'client', RESPONSE, new Uint8Array([result]));

      // Client receives response
      const res = seance.messages.recv('client', RESPONSE);
      expect(res).toBeDefined();
      expect(res!.payload[0]).toBe(200);
    });
  });

  it('should support fan-out pattern', async () => {
    await withSeance(async (seance) => {
      await seance.summon('producer', MINIMAL_WASM);
      await seance.summon('consumer1', MINIMAL_WASM);
      await seance.summon('consumer2', MINIMAL_WASM);

      // Producer sends to all consumers
      const tasks = [1, 2, 3, 4, 5];
      for (const task of tasks) {
        const target = task % 2 === 1 ? 'consumer1' : 'consumer2';
        seance.send('producer', target, 1, new Uint8Array([task]));
      }

      // consumer1 gets odd tasks: 1, 3, 5
      expect(seance.pending('consumer1')).toBe(3);

      // consumer2 gets even tasks: 2, 4
      expect(seance.pending('consumer2')).toBe(2);
    });
  });

  it('should support pipeline pattern', async () => {
    await withSeance(async (seance) => {
      await seance.summon('stage1', MINIMAL_WASM);
      await seance.summon('stage2', MINIMAL_WASM);
      await seance.summon('stage3', MINIMAL_WASM);

      // Input data flows through pipeline
      const input = 10;

      // Stage 1 -> Stage 2
      seance.send('stage1', 'stage2', 1, new Uint8Array([input]));
      const msg1 = seance.messages.recv('stage2');
      expect(msg1!.payload[0]).toBe(10);

      // Stage 2 -> Stage 3 (doubles input)
      const stage2Output = msg1!.payload[0] * 2;
      seance.send('stage2', 'stage3', 1, new Uint8Array([stage2Output]));

      const msg2 = seance.messages.recv('stage3');
      expect(msg2!.payload[0]).toBe(20);
    });
  });
});

describe('Channel-based Communication', () => {
  it('should support multiple channels', async () => {
    await withSeance(async (seance) => {
      await seance.summon('sender', MINIMAL_WASM);
      await seance.summon('receiver', MINIMAL_WASM);

      const CONTROL = 1;
      const DATA = 2;
      const STATUS = 3;

      // Send on different channels
      seance.send('sender', 'receiver', CONTROL, new Uint8Array([0x01]));
      seance.send('sender', 'receiver', DATA, new Uint8Array([0x10, 0x20]));
      seance.send('sender', 'receiver', STATUS, new Uint8Array([0xFF]));

      // Receive by channel
      const controlMsg = seance.messages.recv('receiver', CONTROL);
      expect(controlMsg!.payload[0]).toBe(0x01);

      const dataMsg = seance.messages.recv('receiver', DATA);
      expect(Array.from(dataMsg!.payload)).toEqual([0x10, 0x20]);

      const statusMsg = seance.messages.recv('receiver', STATUS);
      expect(statusMsg!.payload[0]).toBe(0xFF);
    });
  });

  it('should filter by channel when receiving', async () => {
    await withSeance(async (seance) => {
      await seance.summon('a', MINIMAL_WASM);
      await seance.summon('b', MINIMAL_WASM);

      // Send messages on different channels
      seance.send('a', 'b', 1, new Uint8Array([1]));
      seance.send('a', 'b', 2, new Uint8Array([2]));
      seance.send('a', 'b', 1, new Uint8Array([3]));

      // Request channel 2 first
      const ch2Msg = seance.messages.recv('b', 2);
      expect(ch2Msg!.payload[0]).toBe(2);

      // Then channel 1 (should get first message on channel 1)
      const ch1Msg1 = seance.messages.recv('b', 1);
      expect(ch1Msg1!.payload[0]).toBe(1);

      const ch1Msg2 = seance.messages.recv('b', 1);
      expect(ch1Msg2!.payload[0]).toBe(3);
    });
  });
});
