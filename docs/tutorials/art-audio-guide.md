# Art Spirit: Web Audio Integration Guide

This guide explains how to compile DOL Music Spirits to WebAssembly and integrate them with the Web Audio API for real-time browser-based audio synthesis, sequencing, and visualization.

## Prerequisites

- Rust toolchain with `wasm32-unknown-unknown` target
- `wasm-bindgen` and `wasm-pack` tools
- Basic understanding of Web Audio API concepts
- Familiarity with DOL Music Spirit modules

## Project Structure

```
music-spirit-web/
├── Spirit.dol              # Spirit manifest
├── audio_synthesis.dol     # DOL synthesis module
├── Cargo.toml              # Rust project config
├── src/
│   └── lib.rs              # WASM bindings
└── web/
    ├── index.html          # Main page
    ├── app.js              # Audio application
    ├── visualizer.js       # Visualization module
    └── style.css           # Styles
```

## Spirit Definition

### `Spirit.dol`

```dol
spirit music_spirit_web @ 0.1.0

metadata {
    name: "Music Spirit Web"
    description: "Browser-based audio synthesis using Web Audio API"
    author: "Your Name"
    license: "MIT"
}

modules {
    audio_synthesis: "./audio_synthesis.dol"
}

exports {
    // Synthesis types
    audio_synthesis.Waveform,
    audio_synthesis.Oscillator,
    audio_synthesis.Envelope,
    audio_synthesis.Filter,
    audio_synthesis.FilterType,
    audio_synthesis.LFO,
    audio_synthesis.AudioBuffer,

    // Functions
    audio_synthesis.generate_samples,
    audio_synthesis.apply_envelope,
    audio_synthesis.apply_filter,
    audio_synthesis.mix_buffers,
    audio_synthesis.normalize_buffer
}

dependencies {
    @univrs/physics: "^0.5.0"
}

target wasm {
    features: ["web-audio", "worklet"]
    optimize: "size"
}
```

### `audio_synthesis.dol`

```dol
// Audio Synthesis Module for Web Audio Integration
module audio_synthesis @ 0.1.0

use @univrs/physics.waves.{ Wave, superposition }

// ============================================================================
// CONSTANTS
// ============================================================================

pub const SAMPLE_RATE_WEB: u32 = 48000   // Web Audio default
pub const A4_FREQUENCY: f64 = 440.0
pub const BUFFER_SIZE: u32 = 128          // Web Audio render quantum

// ============================================================================
// WAVEFORM TYPES
// ============================================================================

pub gen Waveform {
    type: enum {
        Sine,
        Square,
        Sawtooth,
        Triangle,
        Noise
    }
}

// ============================================================================
// OSCILLATOR
// ============================================================================

pub gen Oscillator {
    has waveform: Waveform
    has frequency: f64
    has amplitude: f64
    has phase: f64
    has detune: f64  // cents

    fun sample(t: f64) -> f64 {
        let freq = this.frequency * pow(2.0, this.detune / 1200.0)
        let phase_t = 2.0 * PI * freq * t + this.phase
        return this.amplitude * match this.waveform {
            Waveform::Sine { sin(phase_t) }
            Waveform::Square { sign(sin(phase_t)) }
            Waveform::Sawtooth { 2.0 * (phase_t / (2.0 * PI) % 1.0) - 1.0 }
            Waveform::Triangle { 4.0 * abs((phase_t / (2.0 * PI) % 1.0) - 0.5) - 1.0 }
            Waveform::Noise { random() * 2.0 - 1.0 }
        }
    }

    fun render_buffer(start_time: f64, sample_rate: u32, size: u32) -> Vec<f64> {
        let mut samples = Vec::with_capacity(size)
        for i in 0..size {
            let t = start_time + i as f64 / sample_rate as f64
            samples.push(this.sample(t))
        }
        return samples
    }
}

// ============================================================================
// ENVELOPE (ADSR)
// ============================================================================

pub gen Envelope {
    has attack: f64
    has decay: f64
    has sustain: f64
    has release: f64

    fun sample(t: f64, gate: bool, release_time: Option<f64>) -> f64 {
        match release_time {
            Some(rel_t) {
                // In release phase
                let rel_elapsed = t - rel_t
                if rel_elapsed >= this.release { return 0.0 }
                let rel_level = this.sustain * (1.0 - rel_elapsed / this.release)
                return rel_level
            }
            None {
                if !gate { return 0.0 }
                // Attack
                if t < this.attack {
                    return t / this.attack
                }
                // Decay
                if t < this.attack + this.decay {
                    let decay_t = (t - this.attack) / this.decay
                    return 1.0 - (1.0 - this.sustain) * decay_t
                }
                // Sustain
                return this.sustain
            }
        }
    }
}

// ============================================================================
// FILTER
// ============================================================================

pub gen FilterType {
    type: enum { LowPass, HighPass, BandPass, Notch, Allpass }
}

pub gen BiquadCoeffs {
    has b0: f64
    has b1: f64
    has b2: f64
    has a1: f64
    has a2: f64
}

pub gen Filter {
    has filter_type: FilterType
    has cutoff: f64
    has resonance: f64

    fun compute_coeffs(sample_rate: f64) -> BiquadCoeffs {
        let omega = 2.0 * PI * this.cutoff / sample_rate
        let sin_w = sin(omega)
        let cos_w = cos(omega)
        let alpha = sin_w / (2.0 * this.resonance)

        match this.filter_type {
            FilterType::LowPass {
                let b0 = (1.0 - cos_w) / 2.0
                let b1 = 1.0 - cos_w
                let b2 = (1.0 - cos_w) / 2.0
                let a0 = 1.0 + alpha
                let a1 = -2.0 * cos_w
                let a2 = 1.0 - alpha
                return BiquadCoeffs {
                    b0: b0/a0, b1: b1/a0, b2: b2/a0,
                    a1: a1/a0, a2: a2/a0
                }
            }
            FilterType::HighPass {
                let b0 = (1.0 + cos_w) / 2.0
                let b1 = -(1.0 + cos_w)
                let b2 = (1.0 + cos_w) / 2.0
                let a0 = 1.0 + alpha
                let a1 = -2.0 * cos_w
                let a2 = 1.0 - alpha
                return BiquadCoeffs {
                    b0: b0/a0, b1: b1/a0, b2: b2/a0,
                    a1: a1/a0, a2: a2/a0
                }
            }
            // ... other filter types
        }
    }
}

// ============================================================================
// LFO (LOW FREQUENCY OSCILLATOR)
// ============================================================================

pub gen LFO {
    has waveform: Waveform
    has rate: f64        // Hz
    has depth: f64       // 0.0 - 1.0
    has center: f64      // Center value

    fun sample(t: f64) -> f64 {
        let osc = Oscillator {
            waveform: this.waveform,
            frequency: this.rate,
            amplitude: 1.0,
            phase: 0.0,
            detune: 0.0
        }
        return this.center + osc.sample(t) * this.depth * this.center
    }
}

// ============================================================================
// SYNTH VOICE
// ============================================================================

pub gen Voice {
    has oscillator: Oscillator
    has envelope: Envelope
    has filter: Option<Filter>
    has note_on_time: Option<f64>
    has note_off_time: Option<f64>

    fun is_active(t: f64) -> bool {
        match this.note_on_time {
            None { return false }
            Some(on_t) {
                match this.note_off_time {
                    None { return true }
                    Some(off_t) {
                        return t < off_t + this.envelope.release
                    }
                }
            }
        }
    }

    fun sample(t: f64) -> f64 {
        match this.note_on_time {
            None { return 0.0 }
            Some(on_t) {
                let local_t = t - on_t
                let osc_sample = this.oscillator.sample(local_t)
                let env_value = this.envelope.sample(
                    local_t,
                    this.note_off_time.is_none(),
                    this.note_off_time.map(|off| off - on_t)
                )
                return osc_sample * env_value
            }
        }
    }
}

// ============================================================================
// POLYPHONIC SYNTH
// ============================================================================

pub gen Synth {
    has voices: Vec<Voice>
    has max_voices: u8
    has master_gain: f64

    fun note_on(note: u8, velocity: f64, time: f64) -> Synth {
        let freq = 440.0 * pow(2.0, (note as f64 - 69.0) / 12.0)
        let voice = Voice {
            oscillator: Oscillator {
                waveform: Waveform::Sawtooth,
                frequency: freq,
                amplitude: velocity,
                phase: 0.0,
                detune: 0.0
            },
            envelope: Envelope {
                attack: 0.01,
                decay: 0.1,
                sustain: 0.7,
                release: 0.3
            },
            filter: None,
            note_on_time: Some(time),
            note_off_time: None
        }
        // Voice stealing logic...
        let mut new_voices = this.voices.clone()
        new_voices.push(voice)
        return Synth { voices: new_voices, max_voices: this.max_voices, master_gain: this.master_gain }
    }

    fun render(time: f64, buffer_size: u32, sample_rate: u32) -> Vec<f64> {
        let mut output = vec![0.0; buffer_size as usize]
        for voice in this.voices.iter() {
            if voice.is_active(time) {
                for i in 0..buffer_size {
                    let t = time + i as f64 / sample_rate as f64
                    output[i as usize] += voice.sample(t)
                }
            }
        }
        // Apply master gain and clipping
        for i in 0..buffer_size {
            output[i as usize] = clamp(output[i as usize] * this.master_gain, -1.0, 1.0)
        }
        return output
    }
}

// ============================================================================
// SEQUENCER
// ============================================================================

pub gen SequenceEvent {
    has time: f64       // In beats
    has note: u8        // MIDI note number
    has duration: f64   // In beats
    has velocity: f64   // 0.0 - 1.0
}

pub gen Sequence {
    has events: Vec<SequenceEvent>
    has tempo: f64      // BPM
    has loop_length: f64  // In beats

    fun beat_to_time(beat: f64) -> f64 {
        return beat * 60.0 / this.tempo
    }

    fun get_events_in_range(start_beat: f64, end_beat: f64) -> Vec<SequenceEvent> {
        return this.events.iter()
            .filter(|e| e.time >= start_beat && e.time < end_beat)
            .collect()
    }
}

docs {
    Audio Synthesis Module for Web Audio Integration

    Provides synthesis primitives compatible with Web Audio API:
    - Oscillator: Waveform generation
    - Envelope: ADSR amplitude shaping
    - Filter: Biquad frequency filtering
    - LFO: Modulation source
    - Voice: Complete synth voice
    - Synth: Polyphonic synthesizer
    - Sequence: Event-based sequencer
}
```

## Rust WASM Bindings

### `Cargo.toml`

```toml
[package]
name = "music-spirit-web"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3"
web-sys = { version = "0.3", features = [
    "AudioContext",
    "AudioDestinationNode",
    "AudioNode",
    "AudioParam",
    "AudioBuffer",
    "AudioBufferSourceNode",
    "AudioWorklet",
    "AudioWorkletNode",
    "GainNode",
    "OscillatorNode",
    "OscillatorType",
    "BiquadFilterNode",
    "BiquadFilterType",
    "AnalyserNode",
    "PeriodicWave",
    "console"
] }

[profile.release]
opt-level = "s"
lto = true
```

### `src/lib.rs`

```rust
use wasm_bindgen::prelude::*;
use web_sys::{
    AudioContext, AudioBuffer, GainNode, OscillatorNode,
    OscillatorType, BiquadFilterNode, BiquadFilterType, AnalyserNode
};
use std::f64::consts::PI;

// ============================================================================
// WAVEFORM TYPES
// ============================================================================

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
}

impl Waveform {
    pub fn to_oscillator_type(&self) -> OscillatorType {
        match self {
            Waveform::Sine => OscillatorType::Sine,
            Waveform::Square => OscillatorType::Square,
            Waveform::Sawtooth => OscillatorType::Sawtooth,
            Waveform::Triangle => OscillatorType::Triangle,
        }
    }
}

// ============================================================================
// ENVELOPE
// ============================================================================

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct Envelope {
    attack: f64,
    decay: f64,
    sustain: f64,
    release: f64,
}

#[wasm_bindgen]
impl Envelope {
    #[wasm_bindgen(constructor)]
    pub fn new(attack: f64, decay: f64, sustain: f64, release: f64) -> Envelope {
        Envelope { attack, decay, sustain, release }
    }

    #[wasm_bindgen(getter)]
    pub fn attack(&self) -> f64 { self.attack }

    #[wasm_bindgen(getter)]
    pub fn decay(&self) -> f64 { self.decay }

    #[wasm_bindgen(getter)]
    pub fn sustain(&self) -> f64 { self.sustain }

    #[wasm_bindgen(getter)]
    pub fn release(&self) -> f64 { self.release }

    // Preset envelopes
    pub fn pad() -> Envelope {
        Envelope::new(0.5, 0.5, 0.7, 1.0)
    }

    pub fn pluck() -> Envelope {
        Envelope::new(0.001, 0.2, 0.0, 0.1)
    }

    pub fn organ() -> Envelope {
        Envelope::new(0.01, 0.0, 1.0, 0.01)
    }

    pub fn strings() -> Envelope {
        Envelope::new(0.3, 0.2, 0.8, 0.5)
    }

    pub fn percussion() -> Envelope {
        Envelope::new(0.001, 0.1, 0.3, 0.2)
    }
}

// ============================================================================
// SYNTHESIZER
// ============================================================================

#[wasm_bindgen]
pub struct Synth {
    context: AudioContext,
    master_gain: GainNode,
    voices: Vec<Voice>,
    max_voices: usize,
}

struct Voice {
    oscillator: OscillatorNode,
    gain: GainNode,
    filter: Option<BiquadFilterNode>,
    note: u8,
    start_time: f64,
}

#[wasm_bindgen]
impl Synth {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Synth, JsValue> {
        let context = AudioContext::new()?;
        let master_gain = context.create_gain()?;
        master_gain.gain().set_value(0.5);
        master_gain.connect_with_audio_node(&context.destination())?;

        Ok(Synth {
            context,
            master_gain,
            voices: Vec::new(),
            max_voices: 8,
        })
    }

    pub fn set_master_volume(&self, volume: f64) {
        self.master_gain.gain().set_value(volume as f32);
    }

    pub fn note_on(
        &mut self,
        note: u8,
        velocity: f64,
        waveform: Waveform,
        envelope: &Envelope
    ) -> Result<(), JsValue> {
        // Voice stealing if at max polyphony
        if self.voices.len() >= self.max_voices {
            if let Some(voice) = self.voices.first() {
                voice.oscillator.stop()?;
            }
            self.voices.remove(0);
        }

        let freq = 440.0 * 2.0_f64.powf((note as f64 - 69.0) / 12.0);
        let current_time = self.context.current_time();

        // Create oscillator
        let oscillator = self.context.create_oscillator()?;
        oscillator.set_type(waveform.to_oscillator_type());
        oscillator.frequency().set_value(freq as f32);

        // Create gain for envelope
        let gain = self.context.create_gain()?;
        gain.gain().set_value(0.0);

        // Apply ADSR envelope
        let gain_param = gain.gain();
        gain_param.set_value_at_time(0.0, current_time)?;
        gain_param.linear_ramp_to_value_at_time(
            velocity as f32,
            current_time + envelope.attack
        )?;
        gain_param.linear_ramp_to_value_at_time(
            (velocity * envelope.sustain) as f32,
            current_time + envelope.attack + envelope.decay
        )?;

        // Connect: oscillator -> gain -> master
        oscillator.connect_with_audio_node(&gain)?;
        gain.connect_with_audio_node(&self.master_gain)?;

        oscillator.start()?;

        self.voices.push(Voice {
            oscillator,
            gain,
            filter: None,
            note,
            start_time: current_time,
        });

        Ok(())
    }

    pub fn note_off(&mut self, note: u8, envelope: &Envelope) -> Result<(), JsValue> {
        let current_time = self.context.current_time();

        // Find and release the voice
        if let Some(idx) = self.voices.iter().position(|v| v.note == note) {
            let voice = &self.voices[idx];

            // Apply release envelope
            let gain_param = voice.gain.gain();
            gain_param.cancel_scheduled_values(current_time)?;
            gain_param.set_value_at_time(gain_param.value(), current_time)?;
            gain_param.linear_ramp_to_value_at_time(0.0, current_time + envelope.release)?;

            // Schedule oscillator stop
            voice.oscillator.stop_with_when(current_time + envelope.release)?;
        }

        // Clean up finished voices
        self.voices.retain(|v| v.note != note);

        Ok(())
    }

    pub fn note_on_with_filter(
        &mut self,
        note: u8,
        velocity: f64,
        waveform: Waveform,
        envelope: &Envelope,
        filter_freq: f64,
        filter_q: f64
    ) -> Result<(), JsValue> {
        if self.voices.len() >= self.max_voices {
            if let Some(voice) = self.voices.first() {
                voice.oscillator.stop()?;
            }
            self.voices.remove(0);
        }

        let freq = 440.0 * 2.0_f64.powf((note as f64 - 69.0) / 12.0);
        let current_time = self.context.current_time();

        // Create oscillator
        let oscillator = self.context.create_oscillator()?;
        oscillator.set_type(waveform.to_oscillator_type());
        oscillator.frequency().set_value(freq as f32);

        // Create filter
        let filter = self.context.create_biquad_filter()?;
        filter.set_type(BiquadFilterType::Lowpass);
        filter.frequency().set_value(filter_freq as f32);
        filter.q().set_value(filter_q as f32);

        // Create gain for envelope
        let gain = self.context.create_gain()?;
        gain.gain().set_value(0.0);

        // Apply ADSR
        let gain_param = gain.gain();
        gain_param.set_value_at_time(0.0, current_time)?;
        gain_param.linear_ramp_to_value_at_time(
            velocity as f32,
            current_time + envelope.attack
        )?;
        gain_param.linear_ramp_to_value_at_time(
            (velocity * envelope.sustain) as f32,
            current_time + envelope.attack + envelope.decay
        )?;

        // Connect: oscillator -> filter -> gain -> master
        oscillator.connect_with_audio_node(&filter)?;
        filter.connect_with_audio_node(&gain)?;
        gain.connect_with_audio_node(&self.master_gain)?;

        oscillator.start()?;

        self.voices.push(Voice {
            oscillator,
            gain,
            filter: Some(filter),
            note,
            start_time: current_time,
        });

        Ok(())
    }

    pub fn current_time(&self) -> f64 {
        self.context.current_time()
    }

    pub fn sample_rate(&self) -> f64 {
        self.context.sample_rate() as f64
    }
}

// ============================================================================
// SEQUENCER
// ============================================================================

#[wasm_bindgen]
#[derive(Clone)]
pub struct SequenceEvent {
    time: f64,      // Time in beats
    note: u8,
    duration: f64,  // Duration in beats
    velocity: f64,
}

#[wasm_bindgen]
impl SequenceEvent {
    #[wasm_bindgen(constructor)]
    pub fn new(time: f64, note: u8, duration: f64, velocity: f64) -> SequenceEvent {
        SequenceEvent { time, note, duration, velocity }
    }

    #[wasm_bindgen(getter)]
    pub fn time(&self) -> f64 { self.time }

    #[wasm_bindgen(getter)]
    pub fn note(&self) -> u8 { self.note }

    #[wasm_bindgen(getter)]
    pub fn duration(&self) -> f64 { self.duration }

    #[wasm_bindgen(getter)]
    pub fn velocity(&self) -> f64 { self.velocity }
}

#[wasm_bindgen]
pub struct Sequencer {
    events: Vec<SequenceEvent>,
    tempo: f64,           // BPM
    loop_length: f64,     // In beats
    current_beat: f64,
    is_playing: bool,
    last_time: f64,
}

#[wasm_bindgen]
impl Sequencer {
    #[wasm_bindgen(constructor)]
    pub fn new(tempo: f64, loop_length: f64) -> Sequencer {
        Sequencer {
            events: Vec::new(),
            tempo,
            loop_length,
            current_beat: 0.0,
            is_playing: false,
            last_time: 0.0,
        }
    }

    pub fn add_event(&mut self, event: SequenceEvent) {
        self.events.push(event);
        self.events.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn set_tempo(&mut self, tempo: f64) {
        self.tempo = tempo;
    }

    pub fn play(&mut self, current_time: f64) {
        self.is_playing = true;
        self.last_time = current_time;
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
        self.current_beat = 0.0;
    }

    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    pub fn beat_to_seconds(&self, beats: f64) -> f64 {
        beats * 60.0 / self.tempo
    }

    pub fn seconds_to_beats(&self, seconds: f64) -> f64 {
        seconds * self.tempo / 60.0
    }

    // Returns events that should trigger in the given time window
    pub fn get_events(&self, start_beat: f64, end_beat: f64) -> Vec<SequenceEvent> {
        self.events.iter()
            .filter(|e| {
                let event_beat = e.time % self.loop_length;
                let adjusted_start = start_beat % self.loop_length;
                let adjusted_end = end_beat % self.loop_length;

                if adjusted_start <= adjusted_end {
                    event_beat >= adjusted_start && event_beat < adjusted_end
                } else {
                    // Loop wraparound
                    event_beat >= adjusted_start || event_beat < adjusted_end
                }
            })
            .cloned()
            .collect()
    }

    #[wasm_bindgen(getter)]
    pub fn current_beat(&self) -> f64 { self.current_beat }

    #[wasm_bindgen(getter)]
    pub fn is_playing(&self) -> bool { self.is_playing }

    #[wasm_bindgen(getter)]
    pub fn tempo(&self) -> f64 { self.tempo }
}

// ============================================================================
// AUDIO BUFFER GENERATION
// ============================================================================

#[wasm_bindgen]
pub fn generate_waveform_buffer(
    context: &AudioContext,
    waveform: Waveform,
    frequency: f64,
    duration: f64,
    amplitude: f64
) -> Result<AudioBuffer, JsValue> {
    let sample_rate = context.sample_rate();
    let num_samples = (duration * sample_rate as f64) as usize;

    let buffer = context.create_buffer(1, num_samples as u32, sample_rate)?;
    let mut channel_data = buffer.get_channel_data(0)?;

    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        let phase = 2.0 * PI * frequency * t;

        let sample = match waveform {
            Waveform::Sine => (phase).sin(),
            Waveform::Square => if (phase).sin() >= 0.0 { 1.0 } else { -1.0 },
            Waveform::Sawtooth => 2.0 * ((phase / (2.0 * PI)) % 1.0) - 1.0,
            Waveform::Triangle => 4.0 * ((phase / (2.0 * PI)) % 1.0 - 0.5).abs() - 1.0,
        };

        channel_data[i] = (sample * amplitude) as f32;
    }

    Ok(buffer)
}

// ============================================================================
// UTILITY: MIDI NOTE CONVERSION
// ============================================================================

#[wasm_bindgen]
pub fn midi_to_frequency(note: u8) -> f64 {
    440.0 * 2.0_f64.powf((note as f64 - 69.0) / 12.0)
}

#[wasm_bindgen]
pub fn frequency_to_midi(frequency: f64) -> u8 {
    (69.0 + 12.0 * (frequency / 440.0).log2()).round() as u8
}

#[wasm_bindgen]
pub fn note_name(midi_note: u8) -> String {
    const NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let note = NAMES[(midi_note % 12) as usize];
    let octave = (midi_note / 12) as i32 - 1;
    format!("{}{}", note, octave)
}

// ============================================================================
// ANALYSER UTILITIES
// ============================================================================

#[wasm_bindgen]
pub struct AnalyserData {
    time_domain: Vec<u8>,
    frequency_domain: Vec<u8>,
}

#[wasm_bindgen]
impl AnalyserData {
    pub fn from_analyser(analyser: &AnalyserNode) -> AnalyserData {
        let fft_size = analyser.fft_size() as usize;
        let freq_bin_count = analyser.frequency_bin_count() as usize;

        let mut time_domain = vec![0u8; fft_size];
        let mut frequency_domain = vec![0u8; freq_bin_count];

        analyser.get_byte_time_domain_data(&mut time_domain);
        analyser.get_byte_frequency_data(&mut frequency_domain);

        AnalyserData {
            time_domain,
            frequency_domain,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn time_domain(&self) -> Vec<u8> {
        self.time_domain.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn frequency_domain(&self) -> Vec<u8> {
        self.frequency_domain.clone()
    }
}
```

## Web Application

### `web/index.html`

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>DOL Music Spirit - Web Audio Synthesizer</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <div class="container">
        <header>
            <h1>DOL Music Spirit</h1>
            <p>Web Audio Synthesizer</p>
        </header>

        <!-- Transport Controls -->
        <section class="transport">
            <button id="playBtn" class="btn-primary">Play</button>
            <button id="stopBtn" class="btn-secondary">Stop</button>
            <div class="tempo-control">
                <label>Tempo: <span id="tempoValue">120</span> BPM</label>
                <input type="range" id="tempoSlider" min="60" max="200" value="120">
            </div>
            <div class="volume-control">
                <label>Volume: <span id="volumeValue">50</span>%</label>
                <input type="range" id="volumeSlider" min="0" max="100" value="50">
            </div>
        </section>

        <!-- Synth Controls -->
        <section class="synth-controls">
            <h2>Synthesizer</h2>

            <div class="control-group">
                <h3>Oscillator</h3>
                <div class="waveform-select">
                    <button data-wave="sine" class="wave-btn active">Sine</button>
                    <button data-wave="square" class="wave-btn">Square</button>
                    <button data-wave="sawtooth" class="wave-btn">Sawtooth</button>
                    <button data-wave="triangle" class="wave-btn">Triangle</button>
                </div>
            </div>

            <div class="control-group">
                <h3>Envelope (ADSR)</h3>
                <div class="adsr-controls">
                    <div class="slider-group">
                        <label>Attack</label>
                        <input type="range" id="attackSlider" min="0" max="2000" value="10">
                        <span id="attackValue">10ms</span>
                    </div>
                    <div class="slider-group">
                        <label>Decay</label>
                        <input type="range" id="decaySlider" min="0" max="2000" value="100">
                        <span id="decayValue">100ms</span>
                    </div>
                    <div class="slider-group">
                        <label>Sustain</label>
                        <input type="range" id="sustainSlider" min="0" max="100" value="70">
                        <span id="sustainValue">70%</span>
                    </div>
                    <div class="slider-group">
                        <label>Release</label>
                        <input type="range" id="releaseSlider" min="0" max="3000" value="300">
                        <span id="releaseValue">300ms</span>
                    </div>
                </div>
                <div class="envelope-presets">
                    <button data-preset="pad">Pad</button>
                    <button data-preset="pluck">Pluck</button>
                    <button data-preset="organ">Organ</button>
                    <button data-preset="strings">Strings</button>
                    <button data-preset="percussion">Percussion</button>
                </div>
            </div>

            <div class="control-group">
                <h3>Filter</h3>
                <div class="filter-controls">
                    <label>
                        <input type="checkbox" id="filterEnabled"> Enable Filter
                    </label>
                    <div class="slider-group">
                        <label>Cutoff</label>
                        <input type="range" id="filterCutoff" min="20" max="20000" value="2000">
                        <span id="cutoffValue">2000 Hz</span>
                    </div>
                    <div class="slider-group">
                        <label>Resonance</label>
                        <input type="range" id="filterQ" min="0.1" max="20" step="0.1" value="1">
                        <span id="qValue">1.0</span>
                    </div>
                </div>
            </div>
        </section>

        <!-- Keyboard -->
        <section class="keyboard-section">
            <h2>Keyboard</h2>
            <div class="keyboard" id="keyboard">
                <!-- Keys generated by JavaScript -->
            </div>
            <p class="keyboard-hint">Use keys A-L for white keys, W-P for black keys</p>
        </section>

        <!-- Sequencer -->
        <section class="sequencer-section">
            <h2>Step Sequencer</h2>
            <div class="sequencer-grid" id="sequencerGrid">
                <!-- Grid generated by JavaScript -->
            </div>
            <div class="sequencer-controls">
                <button id="clearSequence">Clear</button>
                <button id="randomSequence">Random</button>
                <select id="sequencePattern">
                    <option value="custom">Custom</option>
                    <option value="arpeggio">Arpeggio</option>
                    <option value="bass">Bass Line</option>
                    <option value="melody">Random Melody</option>
                </select>
            </div>
        </section>

        <!-- Visualization -->
        <section class="visualization-section">
            <h2>Audio Visualization</h2>
            <div class="viz-tabs">
                <button data-viz="waveform" class="viz-btn active">Waveform</button>
                <button data-viz="spectrum" class="viz-btn">Spectrum</button>
                <button data-viz="bars" class="viz-btn">Bars</button>
            </div>
            <canvas id="visualizer" width="800" height="200"></canvas>
        </section>
    </div>

    <script type="module" src="app.js"></script>
</body>
</html>
```

### `web/app.js`

```javascript
import init, {
    Synth,
    Envelope,
    Waveform,
    Sequencer,
    SequenceEvent,
    midi_to_frequency,
    note_name
} from './pkg/music_spirit_web.js';

// ============================================================================
// GLOBAL STATE
// ============================================================================

let synth = null;
let sequencer = null;
let analyserNode = null;
let audioContext = null;
let currentWaveform = Waveform.Sawtooth;
let currentEnvelope = null;
let isPlaying = false;
let activeNotes = new Map();
let visualizationMode = 'waveform';

// ============================================================================
// INITIALIZATION
// ============================================================================

async function initialize() {
    await init();

    // Create synthesizer
    synth = new Synth();

    // Set up analyser for visualization
    audioContext = new AudioContext();
    analyserNode = audioContext.createAnalyser();
    analyserNode.fftSize = 2048;

    // Connect master output to analyser
    // Note: In full implementation, connect synth output to analyser

    // Initialize envelope
    updateEnvelope();

    // Create sequencer
    sequencer = new Sequencer(120.0, 16.0);  // 120 BPM, 16 beats

    // Set up UI
    setupWaveformButtons();
    setupEnvelopeControls();
    setupFilterControls();
    setupKeyboard();
    setupSequencer();
    setupTransport();
    setupVisualization();
    setupKeyboardInput();

    console.log('Music Spirit Web initialized');
}

// ============================================================================
// WAVEFORM SELECTION
// ============================================================================

function setupWaveformButtons() {
    const buttons = document.querySelectorAll('.wave-btn');
    buttons.forEach(btn => {
        btn.addEventListener('click', () => {
            buttons.forEach(b => b.classList.remove('active'));
            btn.classList.add('active');

            const wave = btn.dataset.wave;
            switch (wave) {
                case 'sine': currentWaveform = Waveform.Sine; break;
                case 'square': currentWaveform = Waveform.Square; break;
                case 'sawtooth': currentWaveform = Waveform.Sawtooth; break;
                case 'triangle': currentWaveform = Waveform.Triangle; break;
            }
        });
    });
}

// ============================================================================
// ENVELOPE CONTROLS
// ============================================================================

function updateEnvelope() {
    const attack = parseInt(document.getElementById('attackSlider').value) / 1000;
    const decay = parseInt(document.getElementById('decaySlider').value) / 1000;
    const sustain = parseInt(document.getElementById('sustainSlider').value) / 100;
    const release = parseInt(document.getElementById('releaseSlider').value) / 1000;

    currentEnvelope = new Envelope(attack, decay, sustain, release);

    // Update display values
    document.getElementById('attackValue').textContent = `${Math.round(attack * 1000)}ms`;
    document.getElementById('decayValue').textContent = `${Math.round(decay * 1000)}ms`;
    document.getElementById('sustainValue').textContent = `${Math.round(sustain * 100)}%`;
    document.getElementById('releaseValue').textContent = `${Math.round(release * 1000)}ms`;
}

function setupEnvelopeControls() {
    ['attack', 'decay', 'sustain', 'release'].forEach(param => {
        const slider = document.getElementById(`${param}Slider`);
        slider.addEventListener('input', updateEnvelope);
    });

    // Preset buttons
    document.querySelectorAll('[data-preset]').forEach(btn => {
        btn.addEventListener('click', () => {
            const preset = btn.dataset.preset;
            let env;

            switch (preset) {
                case 'pad':
                    env = Envelope.pad();
                    break;
                case 'pluck':
                    env = Envelope.pluck();
                    break;
                case 'organ':
                    env = Envelope.organ();
                    break;
                case 'strings':
                    env = Envelope.strings();
                    break;
                case 'percussion':
                    env = Envelope.percussion();
                    break;
            }

            if (env) {
                document.getElementById('attackSlider').value = env.attack * 1000;
                document.getElementById('decaySlider').value = env.decay * 1000;
                document.getElementById('sustainSlider').value = env.sustain * 100;
                document.getElementById('releaseSlider').value = env.release * 1000;
                updateEnvelope();
            }
        });
    });
}

// ============================================================================
// FILTER CONTROLS
// ============================================================================

let filterEnabled = false;
let filterCutoff = 2000;
let filterQ = 1.0;

function setupFilterControls() {
    const enableCheckbox = document.getElementById('filterEnabled');
    const cutoffSlider = document.getElementById('filterCutoff');
    const qSlider = document.getElementById('filterQ');

    enableCheckbox.addEventListener('change', (e) => {
        filterEnabled = e.target.checked;
    });

    cutoffSlider.addEventListener('input', (e) => {
        filterCutoff = parseFloat(e.target.value);
        document.getElementById('cutoffValue').textContent = `${Math.round(filterCutoff)} Hz`;
    });

    qSlider.addEventListener('input', (e) => {
        filterQ = parseFloat(e.target.value);
        document.getElementById('qValue').textContent = filterQ.toFixed(1);
    });
}

// ============================================================================
// KEYBOARD
// ============================================================================

const KEY_MAP = {
    'a': 60, 'w': 61, 's': 62, 'e': 63, 'd': 64,
    'f': 65, 't': 66, 'g': 67, 'y': 68, 'h': 69,
    'u': 70, 'j': 71, 'k': 72, 'o': 73, 'l': 74,
    'p': 75, ';': 76
};

function setupKeyboard() {
    const keyboard = document.getElementById('keyboard');

    // Create 2 octaves (C4 to B5)
    const whiteKeys = [0, 2, 4, 5, 7, 9, 11];  // C, D, E, F, G, A, B
    const blackKeys = [1, 3, 6, 8, 10];         // C#, D#, F#, G#, A#

    for (let octave = 0; octave < 2; octave++) {
        const octaveDiv = document.createElement('div');
        octaveDiv.className = 'octave';

        // White keys
        whiteKeys.forEach(note => {
            const midiNote = 60 + octave * 12 + note;
            const key = document.createElement('div');
            key.className = 'white-key';
            key.dataset.note = midiNote;
            key.textContent = note_name(midiNote);

            key.addEventListener('mousedown', () => playNote(midiNote));
            key.addEventListener('mouseup', () => stopNote(midiNote));
            key.addEventListener('mouseleave', () => stopNote(midiNote));

            octaveDiv.appendChild(key);
        });

        // Black keys (overlay)
        const blackKeyContainer = document.createElement('div');
        blackKeyContainer.className = 'black-keys';

        blackKeys.forEach(note => {
            const midiNote = 60 + octave * 12 + note;
            const key = document.createElement('div');
            key.className = 'black-key';
            key.dataset.note = midiNote;

            // Position based on semitone
            const positions = { 1: 0, 3: 1, 6: 3, 8: 4, 10: 5 };
            key.style.left = `${positions[note] * 40 + 25}px`;

            key.addEventListener('mousedown', () => playNote(midiNote));
            key.addEventListener('mouseup', () => stopNote(midiNote));
            key.addEventListener('mouseleave', () => stopNote(midiNote));

            blackKeyContainer.appendChild(key);
        });

        octaveDiv.appendChild(blackKeyContainer);
        keyboard.appendChild(octaveDiv);
    }
}

function setupKeyboardInput() {
    document.addEventListener('keydown', (e) => {
        if (e.repeat) return;
        const note = KEY_MAP[e.key.toLowerCase()];
        if (note !== undefined) {
            playNote(note);
            highlightKey(note, true);
        }
    });

    document.addEventListener('keyup', (e) => {
        const note = KEY_MAP[e.key.toLowerCase()];
        if (note !== undefined) {
            stopNote(note);
            highlightKey(note, false);
        }
    });
}

function highlightKey(note, active) {
    const key = document.querySelector(`[data-note="${note}"]`);
    if (key) {
        if (active) {
            key.classList.add('active');
        } else {
            key.classList.remove('active');
        }
    }
}

function playNote(note, velocity = 0.8) {
    if (activeNotes.has(note)) return;

    activeNotes.set(note, true);

    if (filterEnabled) {
        synth.note_on_with_filter(
            note, velocity, currentWaveform, currentEnvelope,
            filterCutoff, filterQ
        );
    } else {
        synth.note_on(note, velocity, currentWaveform, currentEnvelope);
    }
}

function stopNote(note) {
    if (!activeNotes.has(note)) return;

    activeNotes.delete(note);
    synth.note_off(note, currentEnvelope);
}

// ============================================================================
// SEQUENCER
// ============================================================================

const SEQUENCER_STEPS = 16;
const SEQUENCER_ROWS = 8;  // Notes: C4 to G4

function setupSequencer() {
    const grid = document.getElementById('sequencerGrid');
    const baseNote = 60;  // C4

    // Create grid
    for (let row = SEQUENCER_ROWS - 1; row >= 0; row--) {
        const rowDiv = document.createElement('div');
        rowDiv.className = 'seq-row';

        // Note label
        const label = document.createElement('span');
        label.className = 'seq-label';
        label.textContent = note_name(baseNote + row);
        rowDiv.appendChild(label);

        // Steps
        for (let step = 0; step < SEQUENCER_STEPS; step++) {
            const cell = document.createElement('div');
            cell.className = 'seq-cell';
            cell.dataset.note = baseNote + row;
            cell.dataset.step = step;

            cell.addEventListener('click', () => {
                cell.classList.toggle('active');
                updateSequenceFromGrid();
            });

            rowDiv.appendChild(cell);
        }

        grid.appendChild(rowDiv);
    }

    // Pattern buttons
    document.getElementById('clearSequence').addEventListener('click', clearSequence);
    document.getElementById('randomSequence').addEventListener('click', randomSequence);
    document.getElementById('sequencePattern').addEventListener('change', loadPattern);
}

function updateSequenceFromGrid() {
    sequencer.clear();

    document.querySelectorAll('.seq-cell.active').forEach(cell => {
        const note = parseInt(cell.dataset.note);
        const step = parseInt(cell.dataset.step);

        const event = new SequenceEvent(step, note, 0.5, 0.8);
        sequencer.add_event(event);
    });
}

function clearSequence() {
    document.querySelectorAll('.seq-cell').forEach(cell => {
        cell.classList.remove('active');
    });
    sequencer.clear();
}

function randomSequence() {
    clearSequence();

    for (let step = 0; step < SEQUENCER_STEPS; step++) {
        if (Math.random() > 0.6) {
            const row = Math.floor(Math.random() * SEQUENCER_ROWS);
            const cell = document.querySelector(
                `.seq-cell[data-step="${step}"][data-note="${60 + row}"]`
            );
            if (cell) {
                cell.classList.add('active');
            }
        }
    }

    updateSequenceFromGrid();
}

function loadPattern(e) {
    clearSequence();

    const pattern = e.target.value;
    let notes = [];

    switch (pattern) {
        case 'arpeggio':
            // C Major arpeggio
            notes = [
                [0, 60], [1, 64], [2, 67], [3, 72],
                [4, 67], [5, 64], [6, 60], [7, 64],
                [8, 60], [9, 64], [10, 67], [11, 72],
                [12, 67], [13, 64], [14, 60], [15, 64]
            ];
            break;
        case 'bass':
            // Simple bass line
            notes = [
                [0, 60], [4, 60], [6, 62], [8, 63],
                [10, 60], [12, 60], [14, 62]
            ];
            break;
        case 'melody':
            // Random pentatonic melody
            const pentatonic = [60, 62, 64, 67, 69, 72];
            for (let i = 0; i < 16; i += 2) {
                if (Math.random() > 0.3) {
                    const note = pentatonic[Math.floor(Math.random() * pentatonic.length)];
                    notes.push([i, note]);
                }
            }
            break;
    }

    notes.forEach(([step, note]) => {
        if (note >= 60 && note < 60 + SEQUENCER_ROWS) {
            const cell = document.querySelector(
                `.seq-cell[data-step="${step}"][data-note="${note}"]`
            );
            if (cell) cell.classList.add('active');
        }
    });

    updateSequenceFromGrid();
}

// ============================================================================
// TRANSPORT (PLAY/STOP)
// ============================================================================

let schedulerInterval = null;
let currentStep = 0;

function setupTransport() {
    document.getElementById('playBtn').addEventListener('click', startPlayback);
    document.getElementById('stopBtn').addEventListener('click', stopPlayback);

    document.getElementById('tempoSlider').addEventListener('input', (e) => {
        const tempo = parseInt(e.target.value);
        document.getElementById('tempoValue').textContent = tempo;
        sequencer.set_tempo(tempo);
    });

    document.getElementById('volumeSlider').addEventListener('input', (e) => {
        const volume = parseInt(e.target.value);
        document.getElementById('volumeValue').textContent = volume;
        synth.set_master_volume(volume / 100);
    });
}

function startPlayback() {
    if (isPlaying) return;
    isPlaying = true;

    const tempo = sequencer.tempo;
    const stepDuration = 60000 / tempo / 4;  // 16th notes

    currentStep = 0;

    function scheduler() {
        // Highlight current step
        document.querySelectorAll('.seq-cell').forEach(cell => {
            cell.classList.remove('playing');
            if (parseInt(cell.dataset.step) === currentStep) {
                cell.classList.add('playing');
            }
        });

        // Get events for current step
        const events = sequencer.get_events(currentStep, currentStep + 0.5);
        events.forEach(event => {
            playNote(event.note, event.velocity);

            // Schedule note off
            const durationMs = sequencer.beat_to_seconds(event.duration) * 1000;
            setTimeout(() => stopNote(event.note), durationMs);
        });

        currentStep = (currentStep + 1) % SEQUENCER_STEPS;
    }

    scheduler();
    schedulerInterval = setInterval(scheduler, stepDuration);

    document.getElementById('playBtn').textContent = 'Playing...';
    document.getElementById('playBtn').disabled = true;
}

function stopPlayback() {
    isPlaying = false;

    if (schedulerInterval) {
        clearInterval(schedulerInterval);
        schedulerInterval = null;
    }

    currentStep = 0;

    document.querySelectorAll('.seq-cell').forEach(cell => {
        cell.classList.remove('playing');
    });

    // Stop all active notes
    activeNotes.forEach((_, note) => {
        synth.note_off(note, currentEnvelope);
    });
    activeNotes.clear();

    document.getElementById('playBtn').textContent = 'Play';
    document.getElementById('playBtn').disabled = false;
}

// ============================================================================
// VISUALIZATION
// ============================================================================

let animationFrameId = null;
const canvas = document.getElementById('visualizer');
const ctx = canvas.getContext('2d');

function setupVisualization() {
    document.querySelectorAll('.viz-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            document.querySelectorAll('.viz-btn').forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            visualizationMode = btn.dataset.viz;
        });
    });

    // Start visualization loop
    visualize();
}

function visualize() {
    animationFrameId = requestAnimationFrame(visualize);

    if (!analyserNode) return;

    const bufferLength = analyserNode.frequencyBinCount;
    const timeData = new Uint8Array(bufferLength);
    const freqData = new Uint8Array(bufferLength);

    analyserNode.getByteTimeDomainData(timeData);
    analyserNode.getByteFrequencyData(freqData);

    // Clear canvas
    ctx.fillStyle = '#1a1a2e';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    switch (visualizationMode) {
        case 'waveform':
            drawWaveform(timeData, bufferLength);
            break;
        case 'spectrum':
            drawSpectrum(freqData, bufferLength);
            break;
        case 'bars':
            drawBars(freqData, bufferLength);
            break;
    }
}

function drawWaveform(data, bufferLength) {
    ctx.lineWidth = 2;
    ctx.strokeStyle = '#00ff88';
    ctx.beginPath();

    const sliceWidth = canvas.width / bufferLength;
    let x = 0;

    for (let i = 0; i < bufferLength; i++) {
        const v = data[i] / 128.0;
        const y = v * canvas.height / 2;

        if (i === 0) {
            ctx.moveTo(x, y);
        } else {
            ctx.lineTo(x, y);
        }

        x += sliceWidth;
    }

    ctx.lineTo(canvas.width, canvas.height / 2);
    ctx.stroke();
}

function drawSpectrum(data, bufferLength) {
    const gradient = ctx.createLinearGradient(0, canvas.height, 0, 0);
    gradient.addColorStop(0, '#0066ff');
    gradient.addColorStop(0.5, '#00ff88');
    gradient.addColorStop(1, '#ff0066');

    ctx.fillStyle = gradient;
    ctx.beginPath();
    ctx.moveTo(0, canvas.height);

    const sliceWidth = canvas.width / bufferLength;
    let x = 0;

    for (let i = 0; i < bufferLength; i++) {
        const v = data[i] / 255.0;
        const y = canvas.height - v * canvas.height;

        ctx.lineTo(x, y);
        x += sliceWidth;
    }

    ctx.lineTo(canvas.width, canvas.height);
    ctx.closePath();
    ctx.fill();
}

function drawBars(data, bufferLength) {
    const barCount = 64;
    const barWidth = canvas.width / barCount - 2;
    const step = Math.floor(bufferLength / barCount);

    for (let i = 0; i < barCount; i++) {
        const v = data[i * step] / 255.0;
        const barHeight = v * canvas.height;

        // Color based on frequency
        const hue = (i / barCount) * 120 + 180;
        ctx.fillStyle = `hsl(${hue}, 80%, 50%)`;

        ctx.fillRect(
            i * (barWidth + 2),
            canvas.height - barHeight,
            barWidth,
            barHeight
        );
    }
}

// ============================================================================
// START APPLICATION
// ============================================================================

initialize().catch(console.error);
```

### `web/style.css`

```css
:root {
    --bg-primary: #1a1a2e;
    --bg-secondary: #16213e;
    --bg-tertiary: #0f3460;
    --accent-primary: #00ff88;
    --accent-secondary: #0066ff;
    --text-primary: #ffffff;
    --text-secondary: #a0a0a0;
}

* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

body {
    font-family: 'Segoe UI', system-ui, sans-serif;
    background: var(--bg-primary);
    color: var(--text-primary);
    min-height: 100vh;
    padding: 20px;
}

.container {
    max-width: 900px;
    margin: 0 auto;
}

header {
    text-align: center;
    margin-bottom: 30px;
}

header h1 {
    font-size: 2.5rem;
    background: linear-gradient(135deg, var(--accent-primary), var(--accent-secondary));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
}

section {
    background: var(--bg-secondary);
    border-radius: 12px;
    padding: 20px;
    margin-bottom: 20px;
}

h2 {
    color: var(--accent-primary);
    margin-bottom: 15px;
    font-size: 1.3rem;
}

h3 {
    color: var(--text-secondary);
    margin-bottom: 10px;
    font-size: 1rem;
}

/* Transport Controls */
.transport {
    display: flex;
    align-items: center;
    gap: 20px;
    flex-wrap: wrap;
}

.btn-primary, .btn-secondary {
    padding: 12px 30px;
    border: none;
    border-radius: 8px;
    font-size: 1rem;
    cursor: pointer;
    transition: transform 0.1s, box-shadow 0.2s;
}

.btn-primary {
    background: var(--accent-primary);
    color: var(--bg-primary);
    font-weight: bold;
}

.btn-secondary {
    background: var(--bg-tertiary);
    color: var(--text-primary);
}

.btn-primary:hover, .btn-secondary:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 255, 136, 0.3);
}

.btn-primary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
    transform: none;
}

.tempo-control, .volume-control {
    display: flex;
    flex-direction: column;
    gap: 5px;
}

input[type="range"] {
    width: 120px;
    accent-color: var(--accent-primary);
}

/* Synth Controls */
.control-group {
    margin-bottom: 20px;
    padding: 15px;
    background: var(--bg-tertiary);
    border-radius: 8px;
}

.waveform-select {
    display: flex;
    gap: 10px;
}

.wave-btn {
    padding: 10px 20px;
    background: var(--bg-secondary);
    border: 2px solid var(--bg-tertiary);
    border-radius: 6px;
    color: var(--text-primary);
    cursor: pointer;
    transition: all 0.2s;
}

.wave-btn.active {
    border-color: var(--accent-primary);
    background: rgba(0, 255, 136, 0.1);
}

.adsr-controls {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 15px;
    margin-bottom: 15px;
}

.slider-group {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 5px;
}

.slider-group input[type="range"] {
    width: 100%;
}

.envelope-presets {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
}

.envelope-presets button {
    padding: 8px 15px;
    background: var(--bg-secondary);
    border: 1px solid var(--text-secondary);
    border-radius: 4px;
    color: var(--text-primary);
    cursor: pointer;
    font-size: 0.85rem;
}

.envelope-presets button:hover {
    background: var(--bg-tertiary);
}

.filter-controls {
    display: flex;
    flex-direction: column;
    gap: 15px;
}

/* Keyboard */
.keyboard {
    display: flex;
    justify-content: center;
    padding: 20px 0;
}

.octave {
    position: relative;
    display: flex;
}

.white-key {
    width: 40px;
    height: 150px;
    background: linear-gradient(to bottom, #fff, #e0e0e0);
    border: 1px solid #999;
    border-radius: 0 0 6px 6px;
    cursor: pointer;
    display: flex;
    align-items: flex-end;
    justify-content: center;
    padding-bottom: 10px;
    font-size: 0.7rem;
    color: #333;
    transition: background 0.1s;
}

.white-key:hover {
    background: linear-gradient(to bottom, #f8f8f8, #d0d0d0);
}

.white-key.active {
    background: linear-gradient(to bottom, var(--accent-primary), #00cc6a);
}

.black-keys {
    position: absolute;
    top: 0;
    left: 0;
    display: flex;
}

.black-key {
    position: absolute;
    width: 30px;
    height: 100px;
    background: linear-gradient(to bottom, #333, #111);
    border-radius: 0 0 4px 4px;
    cursor: pointer;
    z-index: 1;
    transition: background 0.1s;
}

.black-key:hover {
    background: linear-gradient(to bottom, #444, #222);
}

.black-key.active {
    background: linear-gradient(to bottom, var(--accent-secondary), #0044aa);
}

.keyboard-hint {
    text-align: center;
    color: var(--text-secondary);
    font-size: 0.9rem;
    margin-top: 10px;
}

/* Sequencer */
.sequencer-grid {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 15px;
    background: var(--bg-tertiary);
    border-radius: 8px;
    overflow-x: auto;
}

.seq-row {
    display: flex;
    align-items: center;
    gap: 4px;
}

.seq-label {
    width: 40px;
    font-size: 0.8rem;
    color: var(--text-secondary);
    text-align: right;
    padding-right: 10px;
}

.seq-cell {
    width: 40px;
    height: 30px;
    background: var(--bg-secondary);
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.1s;
}

.seq-cell:hover {
    background: var(--bg-primary);
}

.seq-cell.active {
    background: var(--accent-primary);
}

.seq-cell.playing {
    box-shadow: 0 0 10px var(--accent-secondary);
}

.seq-cell.active.playing {
    background: var(--accent-secondary);
}

.sequencer-controls {
    display: flex;
    gap: 10px;
    margin-top: 15px;
}

.sequencer-controls button {
    padding: 8px 15px;
    background: var(--bg-tertiary);
    border: none;
    border-radius: 4px;
    color: var(--text-primary);
    cursor: pointer;
}

.sequencer-controls select {
    padding: 8px 15px;
    background: var(--bg-tertiary);
    border: none;
    border-radius: 4px;
    color: var(--text-primary);
}

/* Visualization */
.viz-tabs {
    display: flex;
    gap: 10px;
    margin-bottom: 15px;
}

.viz-btn {
    padding: 8px 20px;
    background: var(--bg-tertiary);
    border: none;
    border-radius: 4px;
    color: var(--text-primary);
    cursor: pointer;
}

.viz-btn.active {
    background: var(--accent-secondary);
}

#visualizer {
    width: 100%;
    height: 200px;
    border-radius: 8px;
    background: var(--bg-primary);
}

/* Responsive */
@media (max-width: 768px) {
    .adsr-controls {
        grid-template-columns: repeat(2, 1fr);
    }

    .transport {
        flex-direction: column;
        align-items: flex-start;
    }
}
```

## Building and Running

### 1. Build WASM Module

```bash
# Install wasm-pack if needed
cargo install wasm-pack

# Build the WASM package
wasm-pack build --target web --out-dir web/pkg
```

### 2. Serve the Application

```bash
# Using Python
python -m http.server 8000 --directory web

# Or using Node.js
npx serve web
```

### 3. Open in Browser

Navigate to `http://localhost:8000` and interact with the synthesizer.

## Advanced Features

### Custom AudioWorklet Processor

For sample-accurate timing and lower latency, use AudioWorklet:

```javascript
// audio-processor.js (register as AudioWorklet)
class SynthProcessor extends AudioWorkletProcessor {
    constructor() {
        super();
        this.voices = [];

        this.port.onmessage = (e) => {
            if (e.data.type === 'noteOn') {
                this.voices.push({
                    frequency: e.data.frequency,
                    phase: 0,
                    envelope: e.data.envelope,
                    startTime: currentTime
                });
            }
        };
    }

    process(inputs, outputs, parameters) {
        const output = outputs[0][0];

        for (let i = 0; i < output.length; i++) {
            let sample = 0;

            for (const voice of this.voices) {
                // Generate sample
                sample += Math.sin(voice.phase * 2 * Math.PI);
                voice.phase += voice.frequency / sampleRate;
                if (voice.phase > 1) voice.phase -= 1;
            }

            output[i] = sample * 0.3;
        }

        return true;
    }
}

registerProcessor('synth-processor', SynthProcessor);
```

Register and use in main app:

```javascript
await audioContext.audioWorklet.addModule('audio-processor.js');
const synthNode = new AudioWorkletNode(audioContext, 'synth-processor');
synthNode.connect(audioContext.destination);

// Send note on
synthNode.port.postMessage({
    type: 'noteOn',
    frequency: 440,
    envelope: { attack: 0.01, decay: 0.1, sustain: 0.7, release: 0.3 }
});
```

### MIDI Input Support

```javascript
async function setupMIDI() {
    if (!navigator.requestMIDIAccess) {
        console.log('Web MIDI not supported');
        return;
    }

    const midi = await navigator.requestMIDIAccess();

    midi.inputs.forEach(input => {
        input.onmidimessage = (e) => {
            const [status, note, velocity] = e.data;

            // Note on (status 144-159)
            if (status >= 144 && status <= 159 && velocity > 0) {
                playNote(note, velocity / 127);
            }
            // Note off (status 128-143 or velocity 0)
            else if (status >= 128 && status <= 143 || velocity === 0) {
                stopNote(note);
            }
        };
    });
}
```

### Recording and Export

```javascript
async function recordAudio(duration) {
    const dest = audioContext.createMediaStreamDestination();
    masterGain.connect(dest);

    const mediaRecorder = new MediaRecorder(dest.stream);
    const chunks = [];

    mediaRecorder.ondataavailable = (e) => chunks.push(e.data);

    mediaRecorder.onstop = () => {
        const blob = new Blob(chunks, { type: 'audio/webm' });
        const url = URL.createObjectURL(blob);

        // Create download link
        const a = document.createElement('a');
        a.href = url;
        a.download = 'recording.webm';
        a.click();
    };

    mediaRecorder.start();
    setTimeout(() => mediaRecorder.stop(), duration * 1000);
}
```

## Summary

This guide covered:

1. **Spirit Definition**: DOL manifest and synthesis module structure
2. **Rust WASM Bindings**: Synthesizer, sequencer, and utilities
3. **Web Audio Integration**: OscillatorNode, GainNode, BiquadFilterNode
4. **Real-time Synthesis**: ADSR envelopes, waveform selection, polyphony
5. **Sequencing**: Step sequencer with pattern presets
6. **Visualization**: Waveform, spectrum, and bar visualizations
7. **Advanced Topics**: AudioWorklet, MIDI input, recording

The combination of DOL's synthesis models with Web Audio API enables rich browser-based audio applications with precise control over sound generation and processing.
