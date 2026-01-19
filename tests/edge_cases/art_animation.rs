//! Art Animation Edge Case Tests
//!
//! Tests edge cases in animation systems and keyframe interpolation:
//! - Keyframes at the same time (coincident keyframes)
//! - Very small time deltas
//! - Infinite loop detection
//! - Easing functions at t=0 and t=1
//! - Empty animation tracks
//!
//! Based on: examples/spirits/animation/keyframes.dol

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
// Coincident Keyframe Edge Cases
// =============================================================================

#[cfg(test)]
mod coincident_keyframes {
    use super::*;

    /// Test two keyframes at exactly the same time
    #[test]
    fn test_keyframes_same_time() {
        let source = r#"
gene CoincidentKeyframes {
    track has keyframes: Vec

    fun evaluate_at_coincident(t: Float64) -> Float64 {
        // Two keyframes at t=0.5, which value wins?
        let prev = 0.5
        let next = 0.5
        if prev == next {
            // Duration is 0, local_t would be 0/0
            // Should return the later keyframe's value
            return this.keyframes[1].value
        }
        return 0.0
    }

    rule sorted_keyframes {
        // Keyframes should be sorted by time
        // But what about equal times?
        true
    }
}

exegesis {
    Tests keyframes at exactly the same time.
    Duration = 0 causes division by zero in interpolation.
    BUG DISCOVERED: Division by zero when computing local_t.
    Solution: Return next keyframe value when duration == 0.
}
        "#;
        assert!(
            should_parse(source),
            "Coincident keyframes test should parse"
        );
    }

    /// Test multiple keyframes at the same time
    #[test]
    fn test_multiple_coincident_keyframes() {
        let source = r#"
gene MultipleCoincident {
    track has keyframes: Vec

    fun evaluate_triple_coincident(t: Float64) -> Float64 {
        // Three keyframes all at t=0.5
        // Which one is the "current" value?
        let time_a = 0.5
        let time_b = 0.5
        let time_c = 0.5

        // Last one in array order should win
        return this.keyframes[2].value
    }

    fun insert_sorted(kf: Tuple) -> Bool {
        // When inserting, equal times go after existing
        return true
    }
}

exegesis {
    Tests multiple (3+) keyframes at the same time.
    Array order determines which value is used.
    Last keyframe at that time takes precedence.
}
        "#;
        assert!(
            should_parse(source),
            "Multiple coincident keyframes test should parse"
        );
    }

    /// Test instant transition (step function)
    #[test]
    fn test_instant_transition() {
        let source = r#"
gene InstantTransition {
    track has keyframes: Vec

    fun create_step_function() -> Vec {
        // Use coincident keyframes for instant transitions
        return vec![
            (0.0, 0.0),   // Start at 0
            (0.5, 0.0),   // Still 0 at 0.5
            (0.5, 1.0),   // Instantly jump to 1 at 0.5
            (1.0, 1.0)    // End at 1
        ]
    }

    fun is_step_easing() -> Bool {
        // Detect when easing is "Step" (no interpolation)
        return true
    }
}

exegesis {
    Tests creating step functions via coincident keyframes.
    Two keyframes at same time with different values = instant jump.
    Useful for on/off states, visibility toggles.
}
        "#;
        assert!(should_parse(source), "Instant transition test should parse");
    }
}

// =============================================================================
// Time Delta Edge Cases
// =============================================================================

#[cfg(test)]
mod time_deltas {
    use super::*;

    /// Test very small time delta between keyframes
    #[test]
    fn test_very_small_time_delta() {
        let source = r#"
gene SmallTimeDelta {
    track has keyframes: Vec

    fun microsecond_delta() -> Float64 {
        // 1 microsecond = 0.000001 seconds
        return 0.000001
    }

    fun nanosecond_delta() -> Float64 {
        // 1 nanosecond = 0.000000001 seconds
        return 0.000000001
    }

    fun evaluate_at_tiny_delta(t: Float64) -> Float64 {
        // Very small deltas may cause floating point precision issues
        let prev_time = 0.5
        let next_time = 0.500000001
        let duration = next_time - prev_time  // ~1 nanosecond

        // local_t calculation may lose precision
        let local_t = (t - prev_time) / duration
        return local_t
    }

    rule minimum_delta {
        // Warn for deltas below floating point precision
        true
    }
}

exegesis {
    Tests keyframes with very small time deltas (microseconds, nanoseconds).
    BUG POTENTIAL: Floating point precision loss in local_t calculation.
    Solution: Clamp minimum delta to reasonable threshold (1ms?).
}
        "#;
        assert!(
            should_parse(source),
            "Very small time delta test should parse"
        );
    }

    /// Test epsilon time differences
    #[test]
    fn test_epsilon_time_difference() {
        let source = r#"
gene EpsilonTime {
    track has keyframes: Vec

    fun machine_epsilon() -> Float64 {
        // f64 epsilon ≈ 2.22e-16
        return 0.00000000000000022204
    }

    fun is_effectively_same_time(a: Float64, b: Float64) -> Bool {
        let epsilon = 0.0001  // Use practical epsilon (0.1ms)
        return abs(a - b) < epsilon
    }

    fun snap_to_grid(t: Float64, grid: Float64) -> Float64 {
        // Snap time to grid to avoid precision issues
        return round(t / grid) * grid
    }
}

exegesis {
    Tests time values that differ by machine epsilon.
    Such small differences should be treated as equal.
    Solution: Use practical epsilon for time comparisons.
}
        "#;
        assert!(
            should_parse(source),
            "Epsilon time difference test should parse"
        );
    }

    /// Test animation at very high frame rates
    #[test]
    fn test_high_frame_rate() {
        let source = r#"
gene HighFrameRate {
    animation has fps: Float64
    animation has duration: Float64

    fun delta_at_1000_fps() -> Float64 {
        return 1.0 / 1000.0  // 1ms per frame
    }

    fun delta_at_10000_fps() -> Float64 {
        return 1.0 / 10000.0  // 0.1ms per frame
    }

    fun frames_in_duration() -> Int64 {
        return (this.duration * this.fps) as Int64
    }

    rule reasonable_frame_rate {
        this.fps <= 1000.0  // Warn above 1000 fps
    }
}

exegesis {
    Tests animation evaluation at very high frame rates.
    1000+ FPS means sub-millisecond deltas.
    Keyframes too close together won't interpolate smoothly.
}
        "#;
        assert!(should_parse(source), "High frame rate test should parse");
    }

    /// Test negative time values
    #[test]
    fn test_negative_time() {
        let source = r#"
gene NegativeTime {
    track has keyframes: Vec

    fun evaluate_at_negative_time(t: Float64) -> Float64 {
        // t < 0 should return first keyframe value
        if t < 0.0 {
            return this.keyframes[0].value
        }
        return this.evaluate(t)
    }

    fun time_before_start() -> Float64 {
        return -1.0  // 1 second before animation start
    }

    rule non_negative_time {
        // Animation time should typically be >= 0
        true
    }
}

exegesis {
    Tests animation evaluation at negative time values.
    Before start of animation, return first keyframe value.
}
        "#;
        assert!(should_parse(source), "Negative time test should parse");
    }

    /// Test time beyond animation duration
    #[test]
    fn test_time_past_end() {
        let source = r#"
gene TimePastEnd {
    animation has duration: Float64
    animation has looping: Bool

    fun evaluate_past_end(t: Float64) -> Float64 {
        if t > this.duration && !this.looping {
            // Return last keyframe value
            return 1.0
        }
        if t > this.duration && this.looping {
            // Wrap time to valid range
            return this.evaluate(t % this.duration)
        }
        return this.evaluate(t)
    }

    fun far_future_time() -> Float64 {
        return 1000.0  // Way past a 1-second animation
    }
}

exegesis {
    Tests animation evaluation past the end of duration.
    Non-looping: hold last value.
    Looping: wrap time using modulo.
}
        "#;
        assert!(should_parse(source), "Time past end test should parse");
    }
}

// =============================================================================
// Infinite Loop Detection Edge Cases
// =============================================================================

#[cfg(test)]
mod infinite_loop_detection {
    use super::*;

    /// Test looping animation at boundary
    #[test]
    fn test_loop_at_boundary() {
        let source = r#"
gene LoopBoundary {
    animation has duration: Float64
    animation has looping: Bool

    fun time_at_exact_duration() -> Float64 {
        // t exactly equals duration in looping animation
        // Should this be t=0 or t=duration?
        let t = this.duration
        if this.looping {
            return t % this.duration  // Returns 0
        }
        return t
    }

    fun is_at_loop_point(t: Float64) -> Bool {
        return t % this.duration == 0.0 && t > 0.0
    }
}

exegesis {
    Tests looping animation when time exactly equals duration.
    t % duration = 0 at exact loop boundary.
    Animation should restart seamlessly.
}
        "#;
        assert!(should_parse(source), "Loop at boundary test should parse");
    }

    /// Test zero duration looping animation
    #[test]
    fn test_zero_duration_loop() {
        let source = r#"
gene ZeroDurationLoop {
    animation has duration: Float64
    animation has looping: Bool

    fun evaluate_zero_duration(t: Float64) -> Float64 {
        if this.duration <= 0.0 && this.looping {
            // t % 0 is undefined - infinite loop or NaN
            // Should detect and handle this case
            return 0.0
        }
        return t % this.duration
    }

    rule positive_duration {
        // Duration must be positive for valid animation
        this.duration > 0.0
    }
}

exegesis {
    Tests looping animation with zero duration.
    BUG DISCOVERED: t % 0 causes division by zero / undefined behavior.
    Solution: Require positive duration, return 0 if duration <= 0.
}
        "#;
        assert!(should_parse(source), "Zero duration loop test should parse");
    }

    /// Test very small duration looping animation
    #[test]
    fn test_tiny_duration_loop() {
        let source = r#"
gene TinyDurationLoop {
    animation has duration: Float64
    animation has looping: Bool

    fun evaluate_tiny_duration(t: Float64) -> Float64 {
        // Duration of 1 microsecond
        // At 60fps, each frame advances 16.67ms
        // Animation loops 16,670 times per frame!
        let duration = 0.000001
        let loops = floor(t / duration)
        return t - loops * duration
    }

    fun loops_per_second(fps: Float64) -> Float64 {
        return fps / this.duration
    }

    rule minimum_duration {
        // Warn for duration < 1ms to prevent excessive looping
        this.duration >= 0.001
    }
}

exegesis {
    Tests looping animation with very small duration.
    May cause excessive loop iterations per frame.
    BUG POTENTIAL: Performance issues with micro-duration loops.
}
        "#;
        assert!(should_parse(source), "Tiny duration loop test should parse");
    }

    /// Test ping-pong (reverse at end) looping
    #[test]
    fn test_pingpong_loop() {
        let source = r#"
gene PingPongLoop {
    animation has duration: Float64
    animation has pingpong: Bool

    fun evaluate_pingpong(t: Float64) -> Float64 {
        if !this.pingpong {
            return t % this.duration
        }

        // Ping-pong: forward then backward
        let cycle_duration = this.duration * 2.0
        let cycle_t = t % cycle_duration

        if cycle_t < this.duration {
            // Forward phase
            return cycle_t
        }
        // Backward phase
        return this.duration - (cycle_t - this.duration)
    }

    fun is_at_turnaround(t: Float64) -> Bool {
        let cycle_t = t % (this.duration * 2.0)
        return cycle_t == this.duration || cycle_t == 0.0
    }
}

exegesis {
    Tests ping-pong (yoyo) looping animation.
    Plays forward then backward seamlessly.
    Turnaround points at t=0 and t=duration.
}
        "#;
        assert!(should_parse(source), "Ping-pong loop test should parse");
    }

    /// Test loop count limit
    #[test]
    fn test_loop_count_limit() {
        let source = r#"
gene LoopCountLimit {
    animation has duration: Float64
    animation has max_loops: Int64

    fun evaluate_with_limit(t: Float64) -> Float64 {
        let current_loop = floor(t / this.duration) as Int64

        if current_loop >= this.max_loops {
            // Animation has completed all loops
            // Hold at end of last loop
            return this.duration
        }

        return t % this.duration
    }

    fun is_completed(t: Float64) -> Bool {
        let current_loop = floor(t / this.duration) as Int64
        return current_loop >= this.max_loops
    }

    rule positive_max_loops {
        this.max_loops >= 1
    }
}

exegesis {
    Tests animation with limited loop count.
    Prevents infinite looping by capping iterations.
}
        "#;
        assert!(should_parse(source), "Loop count limit test should parse");
    }
}

// =============================================================================
// Easing Function Boundary Edge Cases
// =============================================================================

#[cfg(test)]
mod easing_boundaries {
    use super::*;

    /// Test all easing functions at t=0
    #[test]
    fn test_easing_at_zero() {
        let source = r#"
gene EasingAtZero {
    easing has type: String

    fun linear_at_zero() -> Float64 {
        return 0.0  // Linear: f(0) = 0
    }

    fun ease_in_quad_at_zero() -> Float64 {
        return 0.0 * 0.0  // t^2 at t=0 = 0
    }

    fun ease_out_quad_at_zero() -> Float64 {
        return 0.0 * (2.0 - 0.0)  // t(2-t) at t=0 = 0
    }

    fun bounce_at_zero() -> Float64 {
        // Bounce should also return 0 at t=0
        return 0.0
    }

    rule easing_at_zero_is_zero {
        // All easing functions must return 0 at t=0
        true
    }
}

exegesis {
    Tests all easing functions return 0 at t=0.
    This is a fundamental requirement for animation continuity.
    All easing functions must satisfy f(0) = 0.
}
        "#;
        assert!(should_parse(source), "Easing at zero test should parse");
    }

    /// Test all easing functions at t=1
    #[test]
    fn test_easing_at_one() {
        let source = r#"
gene EasingAtOne {
    easing has type: String

    fun linear_at_one() -> Float64 {
        return 1.0  // Linear: f(1) = 1
    }

    fun ease_in_quad_at_one() -> Float64 {
        return 1.0 * 1.0  // t^2 at t=1 = 1
    }

    fun ease_out_quad_at_one() -> Float64 {
        return 1.0 * (2.0 - 1.0)  // t(2-t) at t=1 = 1
    }

    fun bounce_at_one() -> Float64 {
        // Bounce should return 1 at t=1
        return 1.0
    }

    rule easing_at_one_is_one {
        // All easing functions must return 1 at t=1
        true
    }
}

exegesis {
    Tests all easing functions return 1 at t=1.
    This is a fundamental requirement for reaching target value.
    All easing functions must satisfy f(1) = 1.
}
        "#;
        assert!(should_parse(source), "Easing at one test should parse");
    }

    /// Test easing at t=0.5 (midpoint)
    #[test]
    fn test_easing_at_midpoint() {
        let source = r#"
gene EasingMidpoint {
    easing has type: String

    fun linear_at_half() -> Float64 {
        return 0.5  // Linear: f(0.5) = 0.5
    }

    fun ease_in_quad_at_half() -> Float64 {
        return 0.5 * 0.5  // 0.25
    }

    fun ease_out_quad_at_half() -> Float64 {
        return 0.5 * (2.0 - 0.5)  // 0.75
    }

    fun ease_in_out_quad_at_half() -> Float64 {
        // ease-in-out should be at 0.5 at t=0.5 (symmetric)
        return 0.5
    }

    rule symmetric_ease_in_out {
        // ease-in-out functions should have f(0.5) = 0.5
        true
    }
}

exegesis {
    Tests easing functions at midpoint t=0.5.
    Linear is 0.5, ease-in is < 0.5, ease-out is > 0.5.
    Symmetric ease-in-out should be exactly 0.5.
}
        "#;
        assert!(should_parse(source), "Easing at midpoint test should parse");
    }

    /// Test elastic easing overshoot
    #[test]
    fn test_elastic_overshoot() {
        let source = r#"
gene ElasticOvershoot {
    easing has type: String

    fun elastic_out_overshoot() -> Bool {
        // Elastic out overshoots to > 1.0 before settling
        // This is intentional oscillation
        return true
    }

    fun elastic_max_overshoot() -> Float64 {
        // Maximum overshoot depends on amplitude/period
        // Typically around 1.1 for default settings
        return 1.1
    }

    fun elastic_undershoot() -> Bool {
        // Elastic in undershoots to < 0 before rising
        return true
    }

    rule bounded_overshoot {
        // Overshoot should be limited to prevent extreme values
        true
    }
}

exegesis {
    Tests elastic easing overshoot behavior.
    Elastic functions intentionally exceed [0,1] range.
    BUG POTENTIAL: Interpolated values may be < 0 or > 1.
    Solution: Clamp final interpolated values if needed.
}
        "#;
        assert!(should_parse(source), "Elastic overshoot test should parse");
    }

    /// Test back easing negative values
    #[test]
    fn test_back_easing_negative() {
        let source = r#"
gene BackEasingNegative {
    easing has type: String

    fun ease_in_back_negative() -> Bool {
        // ease-in-back goes negative before rising
        // At t ≈ 0.1, value ≈ -0.1
        return true
    }

    fun ease_out_back_exceeds_one() -> Bool {
        // ease-out-back exceeds 1.0 before settling
        return true
    }

    fun back_overshoot_constant() -> Float64 {
        // Standard overshoot constant s ≈ 1.70158
        // Produces 10% overshoot
        return 1.70158
    }

    rule handle_negative_values {
        // Back easing can produce values < 0
        // Interpolation must handle this
        true
    }
}

exegesis {
    Tests back easing negative value behavior.
    ease-in-back produces negative values early in animation.
    BUG POTENTIAL: Interpolated properties may become invalid.
    Solution: Use absolute value or clamp for properties that can't be negative.
}
        "#;
        assert!(
            should_parse(source),
            "Back easing negative test should parse"
        );
    }

    /// Test bounce easing special cases
    #[test]
    fn test_bounce_easing_special() {
        let source = r#"
gene BounceSpecial {
    easing has type: String

    fun bounce_out(t: Float64) -> Float64 {
        let n1 = 7.5625
        let d1 = 2.75

        if t < 1.0 / d1 {
            return n1 * t * t
        } else if t < 2.0 / d1 {
            let t1 = t - 1.5 / d1
            return n1 * t1 * t1 + 0.75
        } else if t < 2.5 / d1 {
            let t1 = t - 2.25 / d1
            return n1 * t1 * t1 + 0.9375
        }
        let t1 = t - 2.625 / d1
        return n1 * t1 * t1 + 0.984375
    }

    fun bounce_at_boundaries() -> Bool {
        // Verify bounce is continuous at segment boundaries
        return true
    }

    fun bounce_in_from_out() -> Float64 {
        // ease-in-bounce = 1 - ease-out-bounce(1-t)
        return 1.0 - this.bounce_out(1.0 - 0.5)
    }
}

exegesis {
    Tests bounce easing piecewise function.
    Bounce has 4 segments with different coefficients.
    Must be continuous at segment boundaries.
}
        "#;
        assert!(
            should_parse(source),
            "Bounce easing special test should parse"
        );
    }

    /// Test cubic bezier easing edge cases
    #[test]
    fn test_cubic_bezier_edges() {
        let source = r#"
gene CubicBezierEdges {
    easing has p1: Tuple
    easing has p2: Tuple

    fun linear_bezier() -> Tuple {
        // Control points for linear: (0.25, 0.25), (0.75, 0.75)
        // Should produce y = x
        return ((0.25, 0.25), (0.75, 0.75))
    }

    fun ease_bezier() -> Tuple {
        // CSS "ease": (0.25, 0.1, 0.25, 1.0)
        return ((0.25, 0.1), (0.25, 1.0))
    }

    fun extreme_control_points() -> Tuple {
        // Control points outside [0,1] for x
        // Can cause numerical issues in root finding
        return ((-0.5, 0.5), (1.5, 0.5))
    }

    fun newton_raphson_convergence() -> Bool {
        // Bezier easing uses Newton-Raphson to find t for x
        // May not converge for extreme control points
        return true
    }

    rule control_point_bounds {
        // X values should typically be in [0, 1]
        // Y values can exceed [0, 1] for overshoot
        true
    }
}

exegesis {
    Tests cubic bezier easing edge cases.
    Uses Newton-Raphson to solve for bezier parameter.
    BUG POTENTIAL: Newton-Raphson may not converge for extreme control points.
    Solution: Fallback to bisection if Newton-Raphson fails.
}
        "#;
        assert!(should_parse(source), "Cubic bezier edges test should parse");
    }

    /// Test easing with t outside [0,1]
    #[test]
    fn test_easing_outside_range() {
        let source = r#"
gene EasingOutsideRange {
    easing has type: String

    fun easing_at_negative(t: Float64) -> Float64 {
        // What happens when t < 0?
        // Should clamp to t=0 or extrapolate?
        if t < 0.0 {
            return 0.0  // Clamp approach
        }
        return t * t  // example ease-in-quad
    }

    fun easing_past_one(t: Float64) -> Float64 {
        // What happens when t > 1?
        // Should clamp to t=1 or extrapolate?
        if t > 1.0 {
            return 1.0  // Clamp approach
        }
        return t * t  // example ease-in-quad
    }

    fun extrapolate(t: Float64) -> Float64 {
        // Extrapolation: apply formula to t even if outside [0,1]
        // Can produce unexpected values
        return t * t  // At t=2, returns 4
    }

    rule clamp_input_time {
        // Typically clamp t to [0, 1] before applying easing
        true
    }
}

exegesis {
    Tests easing behavior with t outside valid [0,1] range.
    Extrapolation can produce extreme values.
    Solution: Clamp t before applying easing function.
}
        "#;
        assert!(
            should_parse(source),
            "Easing outside range test should parse"
        );
    }
}

// =============================================================================
// Empty Animation Track Edge Cases
// =============================================================================

#[cfg(test)]
mod empty_tracks {
    use super::*;

    /// Test track with zero keyframes
    #[test]
    fn test_zero_keyframes() {
        let source = r#"
gene ZeroKeyframes {
    track has keyframes: Vec
    track has name: String

    fun evaluate_empty(t: Float64) -> Float64 {
        if this.keyframes.length == 0 {
            // No keyframes - what to return?
            return 0.0  // Default value
        }
        return this.keyframes[0].value
    }

    fun duration_empty() -> Float64 {
        if this.keyframes.length == 0 {
            return 0.0
        }
        return this.keyframes.last().time - this.keyframes.first().time
    }

    rule at_least_one_keyframe {
        // Track should have at least one keyframe
        this.keyframes.length >= 1
    }
}

exegesis {
    Tests animation track with no keyframes.
    BUG POTENTIAL: Array access on empty keyframes vec.
    Solution: Return default value (0.0) for empty tracks.
}
        "#;
        assert!(should_parse(source), "Zero keyframes test should parse");
    }

    /// Test track with single keyframe
    #[test]
    fn test_single_keyframe() {
        let source = r#"
gene SingleKeyframe {
    track has keyframes: Vec
    track has name: String

    fun evaluate_single(t: Float64) -> Float64 {
        // Single keyframe = constant value
        if this.keyframes.length == 1 {
            return this.keyframes[0].value
        }
        return 0.0
    }

    fun duration_single() -> Float64 {
        // Duration with one keyframe is 0
        return 0.0
    }

    fun is_constant() -> Bool {
        return this.keyframes.length <= 1
    }
}

exegesis {
    Tests animation track with exactly one keyframe.
    Single keyframe means constant value (no interpolation).
    Duration is 0 since there's no range.
}
        "#;
        assert!(should_parse(source), "Single keyframe test should parse");
    }

    /// Test animation with no tracks
    #[test]
    fn test_no_tracks() {
        let source = r#"
gene NoTracks {
    animation has tracks: Vec
    animation has duration: Float64

    fun evaluate_no_tracks(t: Float64) -> Vec {
        if this.tracks.length == 0 {
            return vec![]  // Empty results
        }
        return this.evaluate_all(t)
    }

    fun is_empty_animation() -> Bool {
        return this.tracks.length == 0
    }

    fun add_first_track(track: Tuple) -> Bool {
        return true
    }
}

exegesis {
    Tests animation with zero tracks.
    Empty animation should return empty results, not error.
}
        "#;
        assert!(should_parse(source), "No tracks test should parse");
    }

    /// Test track access by non-existent name
    #[test]
    fn test_track_not_found() {
        let source = r#"
gene TrackNotFound {
    animation has tracks: Vec

    fun get_track_by_name(name: String) -> Option {
        for track in this.tracks {
            if track.name == name {
                return Some(track)
            }
        }
        return None
    }

    fun get_or_default(name: String, default: Float64) -> Float64 {
        match this.get_track_by_name(name) {
            Some(track) { return track.evaluate(0.0) }
            None { return default }
        }
    }
}

exegesis {
    Tests accessing animation track by name when it doesn't exist.
    Should return None/default, not error or crash.
}
        "#;
        assert!(should_parse(source), "Track not found test should parse");
    }
}

// =============================================================================
// Animation State Edge Cases
// =============================================================================

#[cfg(test)]
mod animation_state {
    use super::*;

    /// Test animation state at boundaries
    #[test]
    fn test_state_at_boundaries() {
        let source = r#"
gene StateBoundaries {
    state has time: Float64
    state has playing: Bool
    state has speed: Float64
    state has direction: Int8

    fun at_start() -> Bool {
        return this.time == 0.0
    }

    fun at_end(duration: Float64) -> Bool {
        return this.time >= duration
    }

    fun advance_to_exactly_end(dt: Float64, duration: Float64) -> Tuple {
        let new_time = this.time + dt * this.speed * this.direction as Float64
        if new_time >= duration {
            return (duration, false)  // Stop at end
        }
        return (new_time, true)  // Keep playing
    }
}

exegesis {
    Tests animation state at start (t=0) and end (t=duration).
    State should handle reaching exact end gracefully.
}
        "#;
        assert!(
            should_parse(source),
            "State at boundaries test should parse"
        );
    }

    /// Test negative playback speed
    #[test]
    fn test_negative_speed() {
        let source = r#"
gene NegativeSpeed {
    state has time: Float64
    state has playing: Bool
    state has speed: Float64
    state has direction: Int8

    fun is_valid_speed() -> Bool {
        // Speed should typically be >= 0
        // Negative direction handles reverse playback
        return this.speed >= 0.0
    }

    fun advance_reverse(dt: Float64) -> Float64 {
        // direction = -1 for reverse
        return this.time + dt * this.speed * this.direction as Float64
    }

    rule non_negative_speed {
        this.speed >= 0.0
    }
}

exegesis {
    Tests animation with negative speed.
    Negative speed should be handled via direction, not speed.
    Speed is magnitude, direction is sign.
}
        "#;
        assert!(should_parse(source), "Negative speed test should parse");
    }

    /// Test zero speed (paused)
    #[test]
    fn test_zero_speed() {
        let source = r#"
gene ZeroSpeed {
    state has time: Float64
    state has playing: Bool
    state has speed: Float64

    fun is_effectively_paused() -> Bool {
        return this.speed == 0.0 || !this.playing
    }

    fun advance_at_zero_speed(dt: Float64) -> Float64 {
        // time + dt * 0 = time (no change)
        return this.time + dt * this.speed
    }
}

exegesis {
    Tests animation at speed=0 (effectively paused).
    Advancing time has no effect when speed is 0.
}
        "#;
        assert!(should_parse(source), "Zero speed test should parse");
    }

    /// Test very high playback speed
    #[test]
    fn test_high_speed() {
        let source = r#"
gene HighSpeed {
    state has time: Float64
    state has speed: Float64

    fun advance_at_10x(dt: Float64) -> Float64 {
        // 10x speed - animation plays 10 times faster
        return this.time + dt * 10.0
    }

    fun frames_to_complete(duration: Float64, fps: Float64) -> Float64 {
        // At 10x speed, completes in 1/10 the time
        return duration / this.speed * fps
    }

    rule reasonable_speed {
        // Warn for extreme speeds that might cause issues
        this.speed <= 100.0
    }
}

exegesis {
    Tests animation at very high playback speed (10x+).
    May skip keyframes if dt*speed exceeds keyframe gaps.
}
        "#;
        assert!(should_parse(source), "High speed test should parse");
    }

    /// Test seeking to exact keyframe time
    #[test]
    fn test_seek_to_keyframe() {
        let source = r#"
gene SeekToKeyframe {
    state has time: Float64
    animation has keyframes: Vec
    animation has duration: Float64

    fun seek_to_first_keyframe() -> Float64 {
        return this.animation.keyframes[0].time
    }

    fun seek_to_last_keyframe() -> Float64 {
        return this.animation.keyframes.last().time
    }

    fun is_at_keyframe(t: Float64, epsilon: Float64) -> Bool {
        // Check if time is within epsilon of any keyframe
        // Simplified: check first and last keyframes
        let first_dist = abs(t - this.animation.keyframes[0].time)
        let last_dist = abs(t - this.animation.keyframes.last().time)
        return first_dist < epsilon || last_dist < epsilon
    }

    fun clamp_to_duration(t: Float64) -> Float64 {
        if t < 0.0 {
            return 0.0
        }
        if t > this.animation.duration {
            return this.animation.duration
        }
        return t
    }
}

exegesis {
    Tests seeking animation to exact keyframe times.
    Snapping to keyframes helps precise editing.
}
        "#;
        assert!(should_parse(source), "Seek to keyframe test should parse");
    }
}

// =============================================================================
// Interpolation Edge Cases
// =============================================================================

#[cfg(test)]
mod interpolation_edges {
    use super::*;

    /// Test interpolating identical values
    #[test]
    fn test_interpolate_same_values() {
        let source = r#"
gene InterpolateSame {
    track has keyframes: Vec

    fun lerp_same(a: Float64, b: Float64, t: Float64) -> Float64 {
        // When a == b, result is always a (or b)
        return a + (b - a) * t  // = a + 0 * t = a
    }

    fun test_constant_animation() -> Bool {
        // All keyframes have same value = constant
        let value = 5.0
        return this.lerp_same(value, value, 0.5) == value
    }
}

exegesis {
    Tests interpolation when start and end values are identical.
    Result should be that constant value for any t.
}
        "#;
        assert!(
            should_parse(source),
            "Interpolate same values test should parse"
        );
    }

    /// Test interpolating very large values
    #[test]
    fn test_interpolate_large_values() {
        let source = r#"
gene InterpolateLarge {
    track has keyframes: Vec

    fun lerp_large() -> Float64 {
        // Large values may have precision issues
        let a = 1000000000.0
        let b = 1000000001.0
        let t = 0.5
        return a + (b - a) * t
    }

    fun precision_loss() -> Bool {
        // Difference of 1 between billion-scale numbers
        // May lose precision in f64
        return true
    }
}

exegesis {
    Tests interpolation with very large numbers.
    BUG POTENTIAL: Floating point precision loss with large magnitudes.
}
        "#;
        assert!(
            should_parse(source),
            "Interpolate large values test should parse"
        );
    }

    /// Test interpolating across zero
    #[test]
    fn test_interpolate_across_zero() {
        let source = r#"
gene InterpolateAcrossZero {
    track has keyframes: Vec

    fun lerp_negative_to_positive(t: Float64) -> Float64 {
        let a = -10.0
        let b = 10.0
        return a + (b - a) * t
    }

    fun zero_crossing_time() -> Float64 {
        // When does value cross 0?
        // 0 = -10 + 20 * t => t = 0.5
        return 0.5
    }
}

exegesis {
    Tests interpolation from negative to positive values.
    Value crosses zero at midpoint.
}
        "#;
        assert!(
            should_parse(source),
            "Interpolate across zero test should parse"
        );
    }
}
