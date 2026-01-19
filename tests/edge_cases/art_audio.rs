//! Art Audio Edge Case Tests
//!
//! Tests edge cases in audio synthesis and processing:
//! - Frequencies at and beyond Nyquist limit
//! - Sample buffer overflow and underflow
//! - Zero attack envelope behavior
//! - Filter resonance at extremes
//! - Very long and very short audio buffers
//!
//! Based on: examples/spirits/music/synthesis.dol

use metadol::parse_file;

// =============================================================================
// Test Helpers
// =============================================================================

fn should_parse(source: &str) -> bool {
    parse_file(source).is_ok()
}

fn should_fail_parse(source: &str) -> bool {
    !parse_file(source).is_ok()
}

// =============================================================================
// Nyquist Limit Edge Cases
// =============================================================================

#[cfg(test)]
mod nyquist_limit {
    use super::*;

    /// Test frequency exactly at Nyquist limit
    #[test]
    fn test_frequency_at_nyquist() {
        let source = r#"
gene NyquistTest {
    osc has frequency: Float64
    osc has sample_rate: Int64

    fun nyquist_frequency() -> Float64 {
        // Nyquist = sample_rate / 2
        return this.sample_rate as Float64 / 2.0
    }

    fun is_at_nyquist() -> Bool {
        return this.frequency == this.nyquist_frequency()
    }

    rule nyquist_theorem {
        // Frequency must be less than Nyquist to avoid aliasing
        this.frequency < this.sample_rate as Float64 / 2.0
    }
}

exegesis {
    Tests oscillator at exactly the Nyquist frequency.
    At 44100Hz sample rate, Nyquist = 22050Hz.
    Frequency at Nyquist produces a square wave alternating +-1.
}
        "#;
        assert!(should_parse(source), "Nyquist frequency test should parse");
    }

    /// Test frequency just below Nyquist
    #[test]
    fn test_frequency_below_nyquist() {
        let source = r#"
gene BelowNyquist {
    osc has frequency: Float64
    osc has sample_rate: Int64

    fun just_below_nyquist() -> Float64 {
        // 1 Hz below Nyquist - valid frequency
        return (this.sample_rate as Float64 / 2.0) - 1.0
    }

    fun is_valid_frequency() -> Bool {
        return this.frequency < this.sample_rate as Float64 / 2.0
    }
}

exegesis {
    Tests oscillator just below Nyquist frequency.
    This is the highest frequency that can be accurately represented.
}
        "#;
        assert!(should_parse(source), "Below Nyquist test should parse");
    }

    /// Test frequency above Nyquist (aliasing)
    #[test]
    fn test_frequency_above_nyquist_aliasing() {
        let source = r#"
gene AliasingTest {
    osc has frequency: Float64
    osc has sample_rate: Int64

    fun aliased_frequency() -> Float64 {
        // Frequency above Nyquist aliases down
        // f_aliased = |f - n * sample_rate| where n makes it < Nyquist
        let nyquist = this.sample_rate as Float64 / 2.0
        if this.frequency > nyquist {
            // Fold back into valid range
            return this.sample_rate as Float64 - this.frequency
        }
        return this.frequency
    }

    fun test_folding() -> Bool {
        // 25000 Hz at 44100 Hz sample rate aliases to 19100 Hz
        return true
    }

    rule warns_aliasing {
        // Should warn when frequency exceeds Nyquist
        this.frequency < this.sample_rate as Float64 / 2.0
    }
}

exegesis {
    Tests frequency aliasing above Nyquist limit.
    25kHz at 44.1kHz sample rate folds back to 19.1kHz.
    BUG POTENTIAL: Aliasing may not be properly warned.
}
        "#;
        assert!(should_parse(source), "Aliasing test should parse");
    }

    /// Test frequency at exactly zero
    #[test]
    fn test_frequency_zero() {
        let source = r#"
gene ZeroFrequency {
    osc has frequency: Float64
    osc has sample_rate: Int64

    fun sample_at_zero_freq(t: Float64) -> Float64 {
        // 0 Hz produces DC offset (constant value)
        // sin(0) = 0, so zero freq sine wave is silence
        return sin(0.0)
    }

    fun is_dc_offset() -> Bool {
        return this.frequency == 0.0
    }

    rule non_negative_frequency {
        this.frequency >= 0.0
    }
}

exegesis {
    Tests oscillator at 0 Hz (DC offset / constant).
    Zero frequency sine wave produces silence (constant 0).
}
        "#;
        assert!(should_parse(source), "Zero frequency test should parse");
    }

    /// Test very high frequencies near sample rate
    #[test]
    fn test_frequency_near_sample_rate() {
        let source = r#"
gene HighFrequency {
    osc has frequency: Float64
    osc has sample_rate: Int64

    fun at_sample_rate() -> Float64 {
        // Frequency = sample rate produces constant value
        // One sample per cycle = no variation
        return this.sample_rate as Float64
    }

    fun above_sample_rate() -> Float64 {
        // Frequency > sample rate is severely aliased
        return this.sample_rate as Float64 * 2.0
    }

    rule frequency_limit {
        this.frequency <= this.sample_rate as Float64
    }
}

exegesis {
    Tests frequencies at and above sample rate.
    At sample rate, you get one sample per cycle (aliased silence).
    Above sample rate, aliasing produces lower frequency.
}
        "#;
        assert!(should_parse(source), "High frequency test should parse");
    }
}

// =============================================================================
// Sample Buffer Edge Cases
// =============================================================================

#[cfg(test)]
mod sample_buffer {
    use super::*;

    /// Test empty buffer (zero samples)
    #[test]
    fn test_empty_buffer() {
        let source = r#"
gene EmptyBuffer {
    buffer has samples: Vec
    buffer has sample_rate: Int64
    buffer has channels: Int8

    fun is_empty() -> Bool {
        return this.samples.length == 0
    }

    fun duration() -> Float64 {
        if this.samples.length == 0 {
            return 0.0
        }
        return this.samples.length as Float64 / (this.sample_rate as Float64 * this.channels as Float64)
    }

    fun peak() -> Float64 {
        if this.samples.length == 0 {
            return 0.0  // No peak for empty buffer
        }
        return 0.0
    }
}

exegesis {
    Tests audio buffer with zero samples.
    Duration should be 0, peak should be 0.
    BUG POTENTIAL: Division by zero if samples.length checked after division.
}
        "#;
        assert!(should_parse(source), "Empty buffer test should parse");
    }

    /// Test single sample buffer
    #[test]
    fn test_single_sample_buffer() {
        let source = r#"
gene SingleSample {
    buffer has samples: Vec
    buffer has sample_rate: Int64
    buffer has channels: Int8

    fun create_single_sample() -> Vec {
        return vec![0.5]
    }

    fun duration() -> Float64 {
        // 1 sample at 44100 Hz = 1/44100 seconds
        return 1.0 / this.sample_rate as Float64
    }

    fun is_minimum_valid() -> Bool {
        return this.samples.length >= 1
    }
}

exegesis {
    Tests audio buffer with exactly one sample.
    Minimum valid buffer for some operations.
}
        "#;
        assert!(
            should_parse(source),
            "Single sample buffer test should parse"
        );
    }

    /// Test very large buffer (memory considerations)
    #[test]
    fn test_very_large_buffer() {
        let source = r#"
gene LargeBuffer {
    buffer has samples: Vec
    buffer has sample_rate: Int64
    buffer has channels: Int8

    fun samples_for_duration(seconds: Float64) -> Int64 {
        // 10 minutes of stereo 44.1kHz = 52,920,000 samples
        return (seconds * this.sample_rate as Float64 * this.channels as Float64) as Int64
    }

    fun ten_minute_buffer() -> Int64 {
        return this.samples_for_duration(600.0)
    }

    fun one_hour_buffer() -> Int64 {
        return this.samples_for_duration(3600.0)
    }

    rule reasonable_size {
        // Warn for buffers > 10 minutes
        this.samples.length <= 52920000
    }
}

exegesis {
    Tests audio buffer size limits.
    10 minutes stereo 44.1kHz = ~53 million samples.
    BUG POTENTIAL: Memory allocation failure for very large buffers.
}
        "#;
        assert!(should_parse(source), "Large buffer test should parse");
    }

    /// Test buffer with sample values at boundaries
    #[test]
    fn test_buffer_sample_boundaries() {
        let source = r#"
gene SampleBoundaries {
    buffer has samples: Vec
    buffer has sample_rate: Int64

    fun sample_at_max() -> Float64 {
        return 1.0  // Maximum sample value
    }

    fun sample_at_min() -> Float64 {
        return -1.0  // Minimum sample value
    }

    fun sample_at_zero() -> Float64 {
        return 0.0  // Silence
    }

    fun clamp_sample(value: Float64) -> Float64 {
        if value < -1.0 {
            return -1.0
        }
        if value > 1.0 {
            return 1.0
        }
        return value
    }

    rule valid_sample_range {
        // All samples must be in [-1.0, 1.0]
        true
    }
}

exegesis {
    Tests audio sample values at boundaries.
    Samples must be in range [-1.0, 1.0].
    Values outside this range cause clipping/distortion.
}
        "#;
        assert!(should_parse(source), "Sample boundaries test should parse");
    }

    /// Test buffer overflow (clipping)
    #[test]
    fn test_buffer_overflow_clipping() {
        let source = r#"
gene BufferOverflow {
    buffer has samples: Vec
    buffer has sample_rate: Int64

    fun mix_may_overflow(a: Float64, b: Float64) -> Float64 {
        // Mixing two signals can exceed [-1, 1] range
        let mixed = a + b
        if mixed > 1.0 || mixed < -1.0 {
            // Clipping occurs - distortion
            return if mixed > 0.0 { 1.0 } else { -1.0 }
        }
        return mixed
    }

    fun hard_clip(value: Float64) -> Float64 {
        if value > 1.0 {
            return 1.0
        }
        if value < -1.0 {
            return -1.0
        }
        return value
    }

    fun soft_clip(value: Float64) -> Float64 {
        // Soft clipping using tanh for smoother distortion
        return tanh(value)
    }
}

exegesis {
    Tests sample overflow when mixing multiple signals.
    Hard clipping causes distortion, soft clipping smoother.
    BUG POTENTIAL: Mixing without normalization causes clipping.
}
        "#;
        assert!(should_parse(source), "Buffer overflow test should parse");
    }

    /// Test buffer with different sample rates
    #[test]
    fn test_buffer_sample_rates() {
        let source = r#"
gene SampleRates {
    buffer has samples: Vec
    buffer has sample_rate: Int64

    fun is_valid_sample_rate() -> Bool {
        // Common sample rates: 8000, 11025, 22050, 44100, 48000, 96000, 192000
        return this.sample_rate >= 8000 && this.sample_rate <= 192000
    }

    fun samples_per_ms() -> Float64 {
        return this.sample_rate as Float64 / 1000.0
    }

    rule valid_sample_rate {
        this.sample_rate >= 8000 && this.sample_rate <= 192000
    }
}

exegesis {
    Tests valid sample rate ranges.
    8000 Hz (telephone) to 192000 Hz (high-res audio).
}
        "#;
        assert!(should_parse(source), "Sample rates test should parse");
    }
}

// =============================================================================
// Envelope Edge Cases
// =============================================================================

#[cfg(test)]
mod envelope_edges {
    use super::*;

    /// Test envelope with zero attack time
    #[test]
    fn test_zero_attack_envelope() {
        let source = r#"
gene ZeroAttackEnvelope {
    env has attack: Float64
    env has decay: Float64
    env has sustain: Float64
    env has release: Float64

    fun sample_at_zero_attack(t: Float64) -> Float64 {
        // Zero attack = instant full volume
        // At t=0, should be at 1.0 (peak) immediately
        if this.attack <= 0.0 {
            if t < this.decay {
                // Decay phase starts immediately
                return 1.0 - (1.0 - this.sustain) * (t / this.decay)
            }
            return this.sustain
        }
        return 0.0
    }

    fun is_instant_attack() -> Bool {
        return this.attack <= 0.0
    }

    rule non_negative_attack {
        this.attack >= 0.0
    }
}

exegesis {
    Tests ADSR envelope with attack time of 0.
    Zero attack means instant transition to peak amplitude.
    BUG POTENTIAL: Division by zero if attack==0 used as divisor.
}
        "#;
        assert!(
            should_parse(source),
            "Zero attack envelope test should parse"
        );
    }

    /// Test envelope with zero decay time
    #[test]
    fn test_zero_decay_envelope() {
        let source = r#"
gene ZeroDecayEnvelope {
    env has attack: Float64
    env has decay: Float64
    env has sustain: Float64
    env has release: Float64

    fun sample_at_zero_decay(t: Float64, note_on_duration: Float64) -> Float64 {
        // Zero decay = instant transition to sustain level
        if this.attack > 0.0 && t < this.attack {
            return t / this.attack
        }
        // After attack, immediately at sustain
        if this.decay <= 0.0 {
            return this.sustain
        }
        return this.sustain
    }

    rule non_negative_decay {
        this.decay >= 0.0
    }
}

exegesis {
    Tests ADSR envelope with decay time of 0.
    Zero decay means instant transition from peak to sustain.
}
        "#;
        assert!(
            should_parse(source),
            "Zero decay envelope test should parse"
        );
    }

    /// Test envelope with zero release time
    #[test]
    fn test_zero_release_envelope() {
        let source = r#"
gene ZeroReleaseEnvelope {
    env has attack: Float64
    env has decay: Float64
    env has sustain: Float64
    env has release: Float64

    fun sample_at_zero_release(t: Float64, note_off_time: Float64) -> Float64 {
        // Zero release = instant silence on note off
        if t >= note_off_time && this.release <= 0.0 {
            return 0.0  // Instant cutoff
        }
        return this.sustain
    }

    fun is_hard_cutoff() -> Bool {
        return this.release <= 0.0
    }

    rule non_negative_release {
        this.release >= 0.0
    }
}

exegesis {
    Tests ADSR envelope with release time of 0.
    Zero release means instant silence on note off (hard cutoff).
    May cause audible clicks/pops due to discontinuity.
}
        "#;
        assert!(
            should_parse(source),
            "Zero release envelope test should parse"
        );
    }

    /// Test envelope with sustain at boundaries
    #[test]
    fn test_sustain_boundaries() {
        let source = r#"
gene SustainBoundaries {
    env has attack: Float64
    env has decay: Float64
    env has sustain: Float64
    env has release: Float64

    fun zero_sustain() -> Bool {
        // Sustain = 0 means decay to silence
        return this.sustain == 0.0
    }

    fun full_sustain() -> Bool {
        // Sustain = 1 means no decay (organ-like)
        return this.sustain == 1.0
    }

    rule valid_sustain {
        this.sustain >= 0.0 && this.sustain <= 1.0
    }
}

exegesis {
    Tests ADSR envelope sustain level at boundaries.
    sustain=0.0 decays to silence (percussive).
    sustain=1.0 never decays (organ-like).
}
        "#;
        assert!(should_parse(source), "Sustain boundaries test should parse");
    }

    /// Test envelope with all values at zero
    #[test]
    fn test_all_zero_envelope() {
        let source = r#"
gene AllZeroEnvelope {
    env has attack: Float64
    env has decay: Float64
    env has sustain: Float64
    env has release: Float64

    fun sample_all_zero(t: Float64) -> Float64 {
        // A=0, D=0, S=0, R=0 produces instant full volume then silence
        // This is essentially a click
        if t == 0.0 {
            return 1.0  // Instant peak
        }
        return 0.0  // Then instant silence
    }

    fun is_click() -> Bool {
        return this.attack <= 0.0 && this.decay <= 0.0 &&
               this.sustain <= 0.0 && this.release <= 0.0
    }
}

exegesis {
    Tests ADSR with all parameters at zero.
    This produces a single-sample click (impulse).
    BUG POTENTIAL: Multiple division-by-zero issues.
}
        "#;
        assert!(should_parse(source), "All zero envelope test should parse");
    }

    /// Test envelope with very long times
    #[test]
    fn test_very_long_envelope() {
        let source = r#"
gene LongEnvelope {
    env has attack: Float64
    env has decay: Float64
    env has sustain: Float64
    env has release: Float64

    fun ten_second_attack() -> Bool {
        return this.attack == 10.0
    }

    fun one_minute_release() -> Bool {
        return this.release == 60.0
    }

    fun total_time(note_duration: Float64) -> Float64 {
        return this.attack + this.decay + note_duration + this.release
    }

    rule reasonable_times {
        // Warn for envelope phases > 1 minute
        this.attack <= 60.0 && this.decay <= 60.0 && this.release <= 60.0
    }
}

exegesis {
    Tests ADSR with very long envelope times.
    Useful for ambient/drone music but may exceed buffer limits.
}
        "#;
        assert!(should_parse(source), "Long envelope test should parse");
    }
}

// =============================================================================
// Filter Resonance Edge Cases
// =============================================================================

#[cfg(test)]
mod filter_resonance {
    use super::*;

    /// Test filter with minimum resonance
    #[test]
    fn test_minimum_resonance() {
        let source = r#"
gene MinResonance {
    filter has filter_type: String
    filter has cutoff: Float64
    filter has resonance: Float64

    fun is_minimum_resonance() -> Bool {
        // Minimum Q typically 0.1 (no resonance peak)
        return this.resonance == 0.1
    }

    fun no_peak() -> Bool {
        // At minimum Q, filter has no resonant peak
        return this.resonance <= 0.707
    }

    rule valid_resonance {
        this.resonance >= 0.1 && this.resonance <= 30.0
    }
}

exegesis {
    Tests biquad filter with minimum resonance (Q=0.1).
    Low Q produces gentle rolloff with no resonant peak.
}
        "#;
        assert!(should_parse(source), "Minimum resonance test should parse");
    }

    /// Test filter with maximum resonance
    #[test]
    fn test_maximum_resonance() {
        let source = r#"
gene MaxResonance {
    filter has filter_type: String
    filter has cutoff: Float64
    filter has resonance: Float64

    fun is_maximum_resonance() -> Bool {
        // Q=30 produces extreme resonant peak
        return this.resonance == 30.0
    }

    fun may_self_oscillate() -> Bool {
        // Very high Q can cause self-oscillation
        return this.resonance >= 20.0
    }

    fun amplitude_at_cutoff(q: Float64) -> Float64 {
        // Amplitude at cutoff frequency increases with Q
        // At high Q, gain can exceed 20dB
        return q  // Simplified: gain roughly proportional to Q
    }

    rule extreme_resonance_warning {
        // Warn for Q > 20 (self-oscillation risk)
        this.resonance <= 20.0
    }
}

exegesis {
    Tests biquad filter with maximum resonance (Q=30).
    Very high Q can cause self-oscillation and clipping.
    BUG POTENTIAL: Output may exceed [-1, 1] without limiting.
}
        "#;
        assert!(should_parse(source), "Maximum resonance test should parse");
    }

    /// Test filter at Butterworth Q
    #[test]
    fn test_butterworth_resonance() {
        let source = r#"
gene ButterworthQ {
    filter has filter_type: String
    filter has cutoff: Float64
    filter has resonance: Float64

    fun is_butterworth() -> Bool {
        // Q = 1/sqrt(2) ≈ 0.707 is Butterworth (maximally flat)
        return abs(this.resonance - 0.7071067811865476) < 0.001
    }

    fun is_flat_response() -> Bool {
        // Butterworth has maximally flat passband
        return this.is_butterworth()
    }
}

exegesis {
    Tests filter at Butterworth Q (1/sqrt(2) ≈ 0.707).
    Butterworth provides maximally flat frequency response.
}
        "#;
        assert!(should_parse(source), "Butterworth Q test should parse");
    }

    /// Test filter cutoff at boundaries
    #[test]
    fn test_cutoff_boundaries() {
        let source = r#"
gene CutoffBoundaries {
    filter has filter_type: String
    filter has cutoff: Float64
    filter has resonance: Float64
    filter has sample_rate: Int64

    fun cutoff_at_minimum() -> Bool {
        // Minimum audible frequency ~20Hz
        return this.cutoff == 20.0
    }

    fun cutoff_at_maximum() -> Bool {
        // Maximum useful = Nyquist / 2 for stability
        return this.cutoff <= this.sample_rate as Float64 / 4.0
    }

    fun cutoff_near_nyquist() -> Bool {
        // Cutoff near Nyquist can cause instability
        return this.cutoff > this.sample_rate as Float64 * 0.45
    }

    rule valid_cutoff {
        this.cutoff >= 20.0 && this.cutoff <= 20000.0
    }
}

exegesis {
    Tests filter cutoff frequency at boundaries.
    Cutoff too close to Nyquist can cause filter instability.
    BUG POTENTIAL: Filter becomes unstable when cutoff > 0.45 * Nyquist.
}
        "#;
        assert!(should_parse(source), "Cutoff boundaries test should parse");
    }

    /// Test filter coefficient stability
    #[test]
    fn test_filter_stability() {
        let source = r#"
gene FilterStability {
    filter has filter_type: String
    filter has cutoff: Float64
    filter has resonance: Float64
    filter has sample_rate: Int64

    fun is_stable() -> Bool {
        // IIR filter stability: poles must be inside unit circle
        // High cutoff + high Q can push poles outside
        let omega = 2.0 * 3.14159 * this.cutoff / this.sample_rate as Float64
        let alpha = sin(omega) / (2.0 * this.resonance)

        // Simplified stability check
        return alpha < 1.0
    }

    fun check_coefficient_range() -> Bool {
        // Biquad coefficients should be in reasonable range
        return true
    }

    rule stable_filter {
        this.is_stable()
    }
}

exegesis {
    Tests biquad filter numerical stability.
    IIR filters become unstable when poles exit unit circle.
    BUG POTENTIAL: Extreme parameters can cause NaN/Infinity output.
}
        "#;
        assert!(should_parse(source), "Filter stability test should parse");
    }
}

// =============================================================================
// Audio Buffer Size Edge Cases
// =============================================================================

#[cfg(test)]
mod buffer_size_edges {
    use super::*;

    /// Test very short buffer (sub-millisecond)
    #[test]
    fn test_submillisecond_buffer() {
        let source = r#"
gene ShortBuffer {
    buffer has samples: Vec
    buffer has sample_rate: Int64

    fun samples_for_one_ms() -> Int64 {
        // 44 samples for 1ms at 44.1kHz
        return (this.sample_rate as Float64 / 1000.0) as Int64
    }

    fun samples_for_one_tenth_ms() -> Int64 {
        // ~4 samples for 0.1ms at 44.1kHz
        return (this.sample_rate as Float64 / 10000.0) as Int64
    }

    fun is_impractically_short() -> Bool {
        // Less than one period of 20Hz = 2205 samples
        return this.samples.length < 50
    }
}

exegesis {
    Tests very short audio buffers (sub-millisecond).
    May be too short to represent low frequencies accurately.
}
        "#;
        assert!(should_parse(source), "Short buffer test should parse");
    }

    /// Test buffer that can hold one cycle of lowest frequency
    #[test]
    fn test_one_cycle_buffer() {
        let source = r#"
gene OneCycleBuffer {
    buffer has samples: Vec
    buffer has sample_rate: Int64

    fun samples_for_one_cycle(freq: Float64) -> Int64 {
        // One cycle of 20Hz at 44.1kHz = 2205 samples
        return (this.sample_rate as Float64 / freq) as Int64
    }

    fun minimum_for_bass() -> Int64 {
        // Minimum buffer to capture one 20Hz cycle
        return this.samples_for_one_cycle(20.0)
    }
}

exegesis {
    Tests buffer sized for exactly one cycle of given frequency.
    One cycle of 20Hz at 44.1kHz = 2205 samples.
}
        "#;
        assert!(should_parse(source), "One cycle buffer test should parse");
    }

    /// Test buffer power-of-two sizes for FFT
    #[test]
    fn test_fft_buffer_sizes() {
        let source = r#"
gene FFTBufferSizes {
    buffer has samples: Vec
    buffer has sample_rate: Int64

    fun is_power_of_two(n: Int64) -> Bool {
        return n > 0 && (n & (n - 1)) == 0
    }

    fun nearest_power_of_two(n: Int64) -> Int64 {
        // Round up to nearest power of 2
        let mut power = 1
        while power < n {
            power = power * 2
        }
        return power
    }

    fun common_fft_sizes() -> Vec {
        // Common FFT sizes: 256, 512, 1024, 2048, 4096, 8192
        return vec![256, 512, 1024, 2048, 4096, 8192]
    }
}

exegesis {
    Tests buffer sizes suitable for FFT processing.
    FFT is most efficient with power-of-two sizes.
}
        "#;
        assert!(should_parse(source), "FFT buffer sizes test should parse");
    }

    /// Test buffer normalization edge cases
    #[test]
    fn test_normalization_edges() {
        let source = r#"
gene NormalizationEdges {
    buffer has samples: Vec
    buffer has sample_rate: Int64

    fun normalize_silent_buffer() -> Vec {
        // Peak = 0 means silent buffer
        // Division by zero risk in normalization
        let peak = 0.0
        if peak == 0.0 {
            return this.samples.clone()  // Return unchanged
        }
        return this.samples.clone()
    }

    fun normalize_full_scale() -> Vec {
        // Peak = 1.0, already normalized
        let peak = 1.0
        let gain = 1.0 / peak
        return this.samples.clone()
    }

    fun normalize_very_quiet() -> Vec {
        // Peak = 0.001 means very quiet
        // Normalization amplifies noise
        let peak = 0.001
        let gain = 1.0 / peak  // = 1000x amplification
        return this.samples.clone()
    }
}

exegesis {
    Tests audio buffer normalization edge cases.
    BUG POTENTIAL: Division by zero for silent buffers.
    BUG POTENTIAL: Extreme amplification for very quiet buffers.
}
        "#;
        assert!(
            should_parse(source),
            "Normalization edges test should parse"
        );
    }
}

// =============================================================================
// Oscillator Waveform Edge Cases
// =============================================================================

#[cfg(test)]
mod oscillator_edges {
    use super::*;

    /// Test phase at boundaries
    #[test]
    fn test_phase_boundaries() {
        let source = r#"
gene PhaseBoundaries {
    osc has phase: Float64

    fun phase_at_zero() -> Float64 {
        return 0.0
    }

    fun phase_at_pi() -> Float64 {
        return 3.14159265358979
    }

    fun phase_at_two_pi() -> Float64 {
        // 2*PI should wrap to 0
        return 6.28318530717959
    }

    fun normalize_phase(p: Float64) -> Float64 {
        let two_pi = 6.28318530717959
        let normalized = p % two_pi
        if normalized < 0.0 {
            return normalized + two_pi
        }
        return normalized
    }

    rule valid_phase {
        this.phase >= 0.0 && this.phase < 6.28318530717959
    }
}

exegesis {
    Tests oscillator phase at boundary values.
    Phase wraps at 2*PI. Negative phase should wrap to positive.
}
        "#;
        assert!(should_parse(source), "Phase boundaries test should parse");
    }

    /// Test amplitude at boundaries
    #[test]
    fn test_amplitude_boundaries() {
        let source = r#"
gene AmplitudeBoundaries {
    osc has amplitude: Float64

    fun silent() -> Bool {
        return this.amplitude == 0.0
    }

    fun full_volume() -> Bool {
        return this.amplitude == 1.0
    }

    fun is_overdrive() -> Bool {
        // Amplitude > 1 causes clipping
        return this.amplitude > 1.0
    }

    rule valid_amplitude {
        this.amplitude >= 0.0 && this.amplitude <= 1.0
    }
}

exegesis {
    Tests oscillator amplitude at boundary values.
    amplitude=0.0 is silence, amplitude=1.0 is full scale.
    amplitude>1.0 will cause clipping.
}
        "#;
        assert!(
            should_parse(source),
            "Amplitude boundaries test should parse"
        );
    }

    /// Test square wave at boundary samples
    #[test]
    fn test_square_wave_discontinuity() {
        let source = r#"
gene SquareWaveEdge {
    osc has frequency: Float64
    osc has amplitude: Float64

    fun square_sample(t: Float64) -> Float64 {
        let phase = 2.0 * 3.14159 * this.frequency * t
        // Discontinuity at zero crossing
        if sin(phase) >= 0.0 {
            return this.amplitude
        }
        return -this.amplitude
    }

    fun exactly_at_zero() -> Float64 {
        // What happens at exact zero crossing?
        // sin(0) = 0, so >= 0 returns positive
        return this.amplitude
    }

    fun aliasing_content() -> Bool {
        // Square waves contain infinite harmonics
        // Causes aliasing at high frequencies
        return true
    }
}

exegesis {
    Tests square wave at boundary conditions.
    Discontinuity at zero crossing.
    Contains infinite harmonics (aliasing risk).
}
        "#;
        assert!(should_parse(source), "Square wave edge test should parse");
    }

    /// Test noise generator edge cases
    #[test]
    fn test_noise_generator_edges() {
        let source = r#"
gene NoiseEdges {
    osc has frequency: Float64
    osc has amplitude: Float64

    fun white_noise() -> Float64 {
        // Random value in [-1, 1]
        return random() * 2.0 - 1.0
    }

    fun noise_rms() -> Float64 {
        // RMS of uniform white noise = 1/sqrt(3) ≈ 0.577
        return 0.5773502691896258
    }

    fun noise_peak() -> Float64 {
        // Peak can be +/- 1.0 (rare but possible)
        return 1.0
    }

    fun noise_crest_factor() -> Float64 {
        // Peak / RMS = sqrt(3) ≈ 1.732
        return 1.7320508075688772
    }
}

exegesis {
    Tests noise generator statistical properties.
    White noise: uniform distribution, RMS = 1/sqrt(3).
    Peak values at +/-1 are possible but rare.
}
        "#;
        assert!(
            should_parse(source),
            "Noise generator edges test should parse"
        );
    }
}

// =============================================================================
// Decibel Conversion Edge Cases
// =============================================================================

#[cfg(test)]
mod decibel_edges {
    use super::*;

    /// Test amplitude to dB at boundaries
    #[test]
    fn test_amplitude_to_db_boundaries() {
        let source = r#"
gene AmplitudeToDb {
    audio has amplitude: Float64

    fun full_scale_db() -> Float64 {
        // amplitude = 1.0 => 0 dBFS
        return 20.0 * log10(1.0)
    }

    fun half_amplitude_db() -> Float64 {
        // amplitude = 0.5 => -6.02 dBFS
        return 20.0 * log10(0.5)
    }

    fun zero_amplitude_db() -> Float64 {
        // amplitude = 0 => -infinity dBFS
        // log10(0) = -infinity
        return if this.amplitude <= 0.0 {
            -1000.0  // Represent as very negative number
        } else {
            20.0 * log10(this.amplitude)
        }
    }

    fun very_small_amplitude_db() -> Float64 {
        // amplitude = 0.0001 => -80 dBFS
        return 20.0 * log10(0.0001)
    }
}

exegesis {
    Tests amplitude to decibel conversion at boundaries.
    BUG POTENTIAL: log10(0) is negative infinity.
    BUG POTENTIAL: log10(negative) is NaN.
}
        "#;
        assert!(
            should_parse(source),
            "Amplitude to dB boundaries test should parse"
        );
    }

    /// Test dB to amplitude at boundaries
    #[test]
    fn test_db_to_amplitude_boundaries() {
        let source = r#"
gene DbToAmplitude {
    audio has db: Float64

    fun zero_db_amplitude() -> Float64 {
        // 0 dBFS => amplitude = 1.0
        return pow(10.0, 0.0 / 20.0)
    }

    fun minus_six_db_amplitude() -> Float64 {
        // -6 dBFS => amplitude ≈ 0.5
        return pow(10.0, -6.0 / 20.0)
    }

    fun very_negative_db_amplitude() -> Float64 {
        // -120 dBFS => amplitude = 0.000001
        return pow(10.0, -120.0 / 20.0)
    }

    fun positive_db_amplitude() -> Float64 {
        // +6 dBFS => amplitude ≈ 2.0 (clipping!)
        return pow(10.0, 6.0 / 20.0)
    }
}

exegesis {
    Tests decibel to amplitude conversion at boundaries.
    Positive dB values produce amplitudes > 1.0 (clipping).
    Very negative dB approaches but never reaches 0.
}
        "#;
        assert!(
            should_parse(source),
            "dB to amplitude boundaries test should parse"
        );
    }
}
