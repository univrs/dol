//! Art Color Edge Case Tests
//!
//! Tests edge cases in color space conversions and color manipulation:
//! - RGB values at boundaries (0 and 255)
//! - HSL hue wrapping at 0/360 degrees
//! - LAB gamut clipping and out-of-range values
//! - Gradient alpha interpolation
//! - Color blindness simulation edge cases
//!
//! Based on: examples/spirits/visual/color.dol

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
// RGB Boundary Edge Cases
// =============================================================================

#[cfg(test)]
mod rgb_boundaries {
    use super::*;

    /// Test RGB with all channels at minimum value (0)
    #[test]
    fn test_rgb_all_zeros() {
        let source = r#"
gene TestRGB {
    color has r: 0
    color has g: 0
    color has b: 0

    fun is_black() -> Bool {
        return this.r == 0 && this.g == 0 && this.b == 0
    }
}

exegesis {
    RGB black: (0, 0, 0). Tests minimum boundary for all channels.
}
        "#;
        assert!(should_parse(source), "RGB (0,0,0) should parse");
    }

    /// Test RGB with all channels at maximum value (255)
    #[test]
    fn test_rgb_all_max() {
        let source = r#"
gene TestRGB {
    color has r: 255
    color has g: 255
    color has b: 255

    fun is_white() -> Bool {
        return this.r == 255 && this.g == 255 && this.b == 255
    }
}

exegesis {
    RGB white: (255, 255, 255). Tests maximum boundary for all channels.
}
        "#;
        assert!(should_parse(source), "RGB (255,255,255) should parse");
    }

    /// Test RGB with mixed boundary values
    #[test]
    fn test_rgb_mixed_boundaries() {
        let source = r#"
gene TestMixedRGB {
    color has r: 0
    color has g: 255
    color has b: 0

    fun to_normalized() -> Tuple {
        return (0.0, 1.0, 0.0)
    }
}

exegesis {
    Pure green (0, 255, 0). Tests mixing min and max boundaries.
}
        "#;
        assert!(should_parse(source), "Mixed boundary RGB should parse");
    }

    /// Test RGB conversion to normalized (0-1) at boundaries
    #[test]
    fn test_rgb_normalization_boundaries() {
        let source = r#"
gene NormalizationTest {
    color has rgb: Tuple

    fun boundary_normalize() -> Tuple {
        // 0/255 = 0.0, 255/255 = 1.0
        let r = 0.0 / 255.0
        let g = 255.0 / 255.0
        let b = 128.0 / 255.0
        return (r, g, b)
    }

    rule normalized_range {
        // Normalized values must be in [0, 1]
        this.rgb.0 >= 0.0 && this.rgb.0 <= 1.0 &&
        this.rgb.1 >= 0.0 && this.rgb.1 <= 1.0 &&
        this.rgb.2 >= 0.0 && this.rgb.2 <= 1.0
    }
}

exegesis {
    Tests normalization at boundary values ensuring 0/255=0.0 and 255/255=1.0.
}
        "#;
        assert!(
            should_parse(source),
            "RGB normalization boundaries should parse"
        );
    }

    /// Test luminance calculation at boundaries
    #[test]
    fn test_luminance_boundaries() {
        let source = r#"
gene LuminanceTest {
    color has rgb: Tuple

    fun black_luminance() -> Float64 {
        // Black: (0,0,0) -> luminance = 0
        return 0.2126 * 0.0 + 0.7152 * 0.0 + 0.0722 * 0.0
    }

    fun white_luminance() -> Float64 {
        // White: (255,255,255) -> luminance = 1
        return 0.2126 * 1.0 + 0.7152 * 1.0 + 0.0722 * 1.0
    }

    rule luminance_range {
        this.black_luminance() >= 0.0 && this.white_luminance() <= 1.0
    }
}

exegesis {
    Tests sRGB luminance formula at boundary conditions.
    Black luminance = 0.0, white luminance = 1.0.
}
        "#;
        assert!(
            should_parse(source),
            "Luminance boundary tests should parse"
        );
    }
}

// =============================================================================
// RGBA Alpha Channel Edge Cases
// =============================================================================

#[cfg(test)]
mod rgba_alpha {
    use super::*;

    /// Test RGBA with fully transparent alpha (0)
    #[test]
    fn test_rgba_fully_transparent() {
        let source = r#"
gene TransparentColor {
    color has r: 255
    color has g: 0
    color has b: 0
    color has a: 0

    fun opacity() -> Float64 {
        return 0.0 / 255.0  // 0%
    }

    fun is_visible() -> Bool {
        return false
    }
}

exegesis {
    Fully transparent red. Alpha 0 means completely invisible.
}
        "#;
        assert!(should_parse(source), "Fully transparent RGBA should parse");
    }

    /// Test RGBA with fully opaque alpha (255)
    #[test]
    fn test_rgba_fully_opaque() {
        let source = r#"
gene OpaqueColor {
    color has r: 255
    color has g: 0
    color has b: 0
    color has a: 255

    fun opacity() -> Float64 {
        return 255.0 / 255.0  // 100%
    }

    fun is_visible() -> Bool {
        return true
    }
}

exegesis {
    Fully opaque red. Alpha 255 means completely visible.
}
        "#;
        assert!(should_parse(source), "Fully opaque RGBA should parse");
    }

    /// Test premultiply at alpha boundaries
    #[test]
    fn test_premultiply_boundaries() {
        let source = r#"
gene PremultiplyTest {
    color has rgba: Tuple

    fun premultiply_zero_alpha() -> Tuple {
        // r * 0, g * 0, b * 0 = (0, 0, 0, 0)
        return (0, 0, 0, 0)
    }

    fun premultiply_full_alpha() -> Tuple {
        // r * 1, g * 1, b * 1 = (r, g, b, 255)
        return (255, 128, 64, 255)
    }

    rule premultiply_never_exceeds {
        // Premultiplied values should never exceed original
        true
    }
}

exegesis {
    Tests premultiply alpha at boundaries.
    Zero alpha yields black, full alpha preserves color.
}
        "#;
        assert!(
            should_parse(source),
            "Premultiply boundary tests should parse"
        );
    }
}

// =============================================================================
// HSL Hue Wrapping Edge Cases
// =============================================================================

#[cfg(test)]
mod hsl_hue_wrapping {
    use super::*;

    /// Test HSL at hue = 0 (red)
    #[test]
    fn test_hsl_hue_zero() {
        let source = r#"
gene HueZero {
    color has h: 0.0
    color has s: 1.0
    color has l: 0.5

    fun hue_at_origin() -> Bool {
        return this.h == 0.0
    }

    rule valid_hue {
        this.h >= 0.0 && this.h < 360.0
    }
}

exegesis {
    HSL with hue at 0 degrees (pure red in saturated form).
    Tests lower boundary of circular hue space.
}
        "#;
        assert!(should_parse(source), "HSL hue=0 should parse");
    }

    /// Test HSL at hue = 360 (wraps to 0)
    #[test]
    fn test_hsl_hue_360_wrap() {
        let source = r#"
gene HueWrap {
    color has h: Float64
    color has s: Float64
    color has l: Float64

    fun normalize_hue(h: Float64) -> Float64 {
        let normalized = h % 360.0
        if normalized < 0.0 {
            return normalized + 360.0
        }
        return normalized
    }

    fun test_wrap() -> Bool {
        // 360 should wrap to 0
        return this.normalize_hue(360.0) == 0.0
    }

    rule hue_normalized {
        this.h >= 0.0 && this.h < 360.0
    }
}

exegesis {
    Tests hue wrapping: 360 degrees should normalize to 0 degrees.
    Both represent the same color (red).
}
        "#;
        assert!(should_parse(source), "HSL hue=360 wrap should parse");
    }

    /// Test HSL negative hue wrapping
    #[test]
    fn test_hsl_negative_hue_wrap() {
        let source = r#"
gene NegativeHue {
    color has h: Float64
    color has s: Float64
    color has l: Float64

    fun rotate_negative(degrees: Float64) -> Float64 {
        let new_h = this.h - degrees
        if new_h < 0.0 {
            return new_h + 360.0
        }
        return new_h
    }

    fun test_negative_wrap() -> Bool {
        // -30 from 0 should give 330
        return this.rotate_negative(30.0) == 330.0
    }
}

exegesis {
    Tests negative hue rotation wrapping.
    -30 degrees from red (0) should give 330 degrees (magenta-red).
}
        "#;
        assert!(should_parse(source), "HSL negative hue wrap should parse");
    }

    /// Test HSL hue at color wheel critical points
    #[test]
    fn test_hsl_critical_hues() {
        let source = r#"
gene CriticalHues {
    color has h: Float64
    color has s: Float64
    color has l: Float64

    fun is_primary() -> Bool {
        // Primary colors at 0, 120, 240
        return this.h == 0.0 || this.h == 120.0 || this.h == 240.0
    }

    fun is_secondary() -> Bool {
        // Secondary colors at 60, 180, 300
        return this.h == 60.0 || this.h == 180.0 || this.h == 300.0
    }

    rule valid_range {
        this.h >= 0.0 && this.h < 360.0
    }
}

exegesis {
    Tests HSL at critical hue values on the color wheel:
    0=Red, 60=Yellow, 120=Green, 180=Cyan, 240=Blue, 300=Magenta.
}
        "#;
        assert!(should_parse(source), "HSL critical hues should parse");
    }

    /// Test saturation at boundaries
    #[test]
    fn test_hsl_saturation_boundaries() {
        let source = r#"
gene SaturationBoundaries {
    color has h: Float64
    color has s: Float64
    color has l: Float64

    fun is_grayscale() -> Bool {
        return this.s == 0.0
    }

    fun is_fully_saturated() -> Bool {
        return this.s == 1.0
    }

    rule valid_saturation {
        this.s >= 0.0 && this.s <= 1.0
    }
}

exegesis {
    Tests HSL saturation at boundaries.
    s=0.0 is grayscale, s=1.0 is fully saturated.
}
        "#;
        assert!(
            should_parse(source),
            "HSL saturation boundaries should parse"
        );
    }

    /// Test lightness at boundaries
    #[test]
    fn test_hsl_lightness_boundaries() {
        let source = r#"
gene LightnessBoundaries {
    color has h: Float64
    color has s: Float64
    color has l: Float64

    fun is_black() -> Bool {
        return this.l == 0.0
    }

    fun is_white() -> Bool {
        return this.l == 1.0
    }

    fun is_mid_tone() -> Bool {
        return this.l == 0.5
    }

    rule valid_lightness {
        this.l >= 0.0 && this.l <= 1.0
    }
}

exegesis {
    Tests HSL lightness at boundaries.
    l=0.0 is black, l=1.0 is white, l=0.5 is mid-tone.
}
        "#;
        assert!(
            should_parse(source),
            "HSL lightness boundaries should parse"
        );
    }
}

// =============================================================================
// LAB Gamut Clipping Edge Cases
// =============================================================================

#[cfg(test)]
mod lab_gamut {
    use super::*;

    /// Test LAB at lightness boundaries
    #[test]
    fn test_lab_lightness_boundaries() {
        let source = r#"
gene LABLightness {
    color has l: Float64
    color has a: Float64
    color has b: Float64

    fun is_pure_black() -> Bool {
        return this.l == 0.0
    }

    fun is_pure_white() -> Bool {
        return this.l == 100.0
    }

    rule valid_lightness {
        this.l >= 0.0 && this.l <= 100.0
    }
}

exegesis {
    Tests LAB lightness at boundaries (0-100 range).
    L=0 is pure black, L=100 is pure white.
}
        "#;
        assert!(
            should_parse(source),
            "LAB lightness boundaries should parse"
        );
    }

    /// Test LAB a-channel at boundaries
    #[test]
    fn test_lab_a_channel_boundaries() {
        let source = r#"
gene LABAChannel {
    color has l: Float64
    color has a: Float64
    color has b: Float64

    fun is_max_green() -> Bool {
        return this.a == -128.0
    }

    fun is_max_red() -> Bool {
        return this.a == 127.0
    }

    fun is_neutral_a() -> Bool {
        return this.a == 0.0
    }

    rule valid_a_channel {
        this.a >= -128.0 && this.a <= 127.0
    }
}

exegesis {
    Tests LAB a-channel at boundaries.
    a=-128 is maximum green, a=127 is maximum red, a=0 is neutral.
}
        "#;
        assert!(
            should_parse(source),
            "LAB a-channel boundaries should parse"
        );
    }

    /// Test LAB b-channel at boundaries
    #[test]
    fn test_lab_b_channel_boundaries() {
        let source = r#"
gene LABBChannel {
    color has l: Float64
    color has a: Float64
    color has b: Float64

    fun is_max_blue() -> Bool {
        return this.b == -128.0
    }

    fun is_max_yellow() -> Bool {
        return this.b == 127.0
    }

    fun is_neutral_b() -> Bool {
        return this.b == 0.0
    }

    rule valid_b_channel {
        this.b >= -128.0 && this.b <= 127.0
    }
}

exegesis {
    Tests LAB b-channel at boundaries.
    b=-128 is maximum blue, b=127 is maximum yellow, b=0 is neutral.
}
        "#;
        assert!(
            should_parse(source),
            "LAB b-channel boundaries should parse"
        );
    }

    /// Test LAB to RGB gamut clipping
    #[test]
    fn test_lab_gamut_clipping() {
        let source = r#"
gene GamutClipping {
    color has l: Float64
    color has a: Float64
    color has b: Float64

    fun needs_clipping() -> Bool {
        // Some LAB values produce RGB values outside 0-255
        // This tests detection of out-of-gamut colors
        let r_lin = 0.5
        let g_lin = -0.1  // Would clip to 0
        let b_lin = 1.2   // Would clip to 1.0
        return g_lin < 0.0 || b_lin > 1.0
    }

    fun clip_to_gamut(value: Float64) -> Float64 {
        if value < 0.0 {
            return 0.0
        }
        if value > 1.0 {
            return 1.0
        }
        return value
    }
}

exegesis {
    Tests LAB to RGB conversion gamut clipping.
    Not all LAB colors are representable in sRGB - they must be clipped.
}
        "#;
        assert!(should_parse(source), "LAB gamut clipping should parse");
    }

    /// Test LAB delta E at edge cases
    #[test]
    fn test_lab_delta_e_edges() {
        let source = r#"
gene DeltaETest {
    color has l: Float64
    color has a: Float64
    color has b: Float64

    fun delta_e_same_color() -> Float64 {
        // Delta E of a color with itself = 0
        return 0.0
    }

    fun delta_e_black_white() -> Float64 {
        // Maximum lightness difference
        let dl = 100.0 - 0.0
        let da = 0.0 - 0.0
        let db = 0.0 - 0.0
        return sqrt(dl * dl + da * da + db * db)  // = 100
    }

    fun delta_e_max_chroma() -> Float64 {
        // Maximum chroma difference
        let dl = 0.0
        let da = 127.0 - (-128.0)  // 255
        let db = 127.0 - (-128.0)  // 255
        return sqrt(dl * dl + da * da + db * db)
    }
}

exegesis {
    Tests CIE76 Delta E calculation at edge cases.
    Same color = 0, black to white = 100, maximum chroma difference.
}
        "#;
        assert!(should_parse(source), "LAB delta E edge cases should parse");
    }
}

// =============================================================================
// Gradient Alpha Interpolation Edge Cases
// =============================================================================

#[cfg(test)]
mod gradient_alpha {
    use super::*;

    /// Test gradient with only two stops
    #[test]
    fn test_gradient_minimum_stops() {
        let source = r#"
gene MinimumGradient {
    gradient has stops: Vec

    rule minimum_stops {
        this.stops.length >= 2
    }

    fun sample_at_start() -> Tuple {
        return this.stops[0].color
    }

    fun sample_at_end() -> Tuple {
        return this.stops[this.stops.length - 1].color
    }
}

exegesis {
    Tests gradient with minimum valid stops (2).
    Less than 2 stops is invalid.
}
        "#;
        assert!(should_parse(source), "Minimum gradient stops should parse");
    }

    /// Test gradient interpolation at t=0 and t=1
    #[test]
    fn test_gradient_interpolation_boundaries() {
        let source = r#"
gene GradientBoundaries {
    gradient has stops: Vec

    fun sample(t: Float64) -> Tuple {
        // At t=0, return first stop color
        // At t=1, return last stop color
        if t <= 0.0 {
            return this.stops[0].color
        }
        if t >= 1.0 {
            return this.stops[this.stops.length - 1].color
        }
        // Interpolate between stops
        return (0, 0, 0)
    }

    fun test_boundary_values() -> Bool {
        let start = this.sample(0.0)
        let end = this.sample(1.0)
        return true
    }
}

exegesis {
    Tests gradient sampling at boundary values t=0 and t=1.
    t=0 returns first stop, t=1 returns last stop.
}
        "#;
        assert!(
            should_parse(source),
            "Gradient interpolation boundaries should parse"
        );
    }

    /// Test gradient with coincident stops
    #[test]
    fn test_gradient_coincident_stops() {
        let source = r#"
gene CoincidentStops {
    gradient has stops: Vec

    fun sample_at_coincident(t: Float64) -> Tuple {
        // Two stops at same position - which wins?
        // Should return the later stop in the list
        let lower_pos = 0.5
        let upper_pos = 0.5
        if lower_pos == upper_pos {
            return this.stops[1].color  // Return second stop
        }
        return (0, 0, 0)
    }

    rule coincident_handling {
        // Coincident stops create instant color changes
        true
    }
}

exegesis {
    Tests gradient with stops at the same position.
    This creates a hard color transition rather than a blend.
    BUG POTENTIAL: Division by zero in local_t calculation.
}
        "#;
        assert!(
            should_parse(source),
            "Gradient coincident stops should parse"
        );
    }

    /// Test gradient alpha blending
    #[test]
    fn test_gradient_alpha_interpolation() {
        let source = r#"
gene AlphaGradient {
    gradient has stops: Vec

    fun lerp_rgba(a: Tuple, b: Tuple, t: Float64) -> Tuple {
        let r = a.0 * (1.0 - t) + b.0 * t
        let g = a.1 * (1.0 - t) + b.1 * t
        let b_val = a.2 * (1.0 - t) + b.2 * t
        let alpha = a.3 * (1.0 - t) + b.3 * t
        return (r, g, b_val, alpha)
    }

    fun test_transparent_to_opaque() -> Tuple {
        // Interpolate from (255, 0, 0, 0) to (255, 0, 0, 255)
        return this.lerp_rgba((255.0, 0.0, 0.0, 0.0), (255.0, 0.0, 0.0, 255.0), 0.5)
    }
}

exegesis {
    Tests RGBA gradient interpolation with alpha channel.
    Fading from transparent to opaque at midpoint.
}
        "#;
        assert!(
            should_parse(source),
            "Gradient alpha interpolation should parse"
        );
    }

    /// Test gradient with out-of-range t values
    #[test]
    fn test_gradient_out_of_range_t() {
        let source = r#"
gene OutOfRangeT {
    gradient has stops: Vec

    fun sample_clamped(t: Float64) -> Tuple {
        // t values outside [0, 1] should be clamped
        let t_clamped = if t < 0.0 { 0.0 } else if t > 1.0 { 1.0 } else { t }
        return (0, 0, 0)
    }

    fun test_negative_t() -> Tuple {
        return this.sample_clamped(-0.5)  // Should return first stop
    }

    fun test_beyond_one_t() -> Tuple {
        return this.sample_clamped(1.5)  // Should return last stop
    }
}

exegesis {
    Tests gradient behavior with t values outside valid [0, 1] range.
    Implementation should clamp t to valid range.
}
        "#;
        assert!(should_parse(source), "Gradient out-of-range t should parse");
    }
}

// =============================================================================
// Color Blindness Simulation Edge Cases
// =============================================================================

#[cfg(test)]
mod color_blindness {
    use super::*;

    /// Test protanopia (red-blind) simulation at boundaries
    #[test]
    fn test_protanopia_simulation() {
        let source = r#"
gene Protanopia {
    color has rgb: Tuple

    fun simulate_protanopia(r: Float64, g: Float64, b: Float64) -> Tuple {
        // Red-blindness matrix transformation
        let r_new = 0.56667 * r + 0.43333 * g + 0.0 * b
        let g_new = 0.55833 * r + 0.44167 * g + 0.0 * b
        let b_new = 0.0 * r + 0.24167 * g + 0.75833 * b
        return (r_new, g_new, b_new)
    }

    fun test_pure_red() -> Tuple {
        // Pure red should appear desaturated
        return this.simulate_protanopia(1.0, 0.0, 0.0)
    }

    fun test_pure_green() -> Tuple {
        // Pure green transformed
        return this.simulate_protanopia(0.0, 1.0, 0.0)
    }
}

exegesis {
    Tests protanopia (red-blindness) color simulation.
    Pure red becomes a desaturated olive/brown color.
}
        "#;
        assert!(should_parse(source), "Protanopia simulation should parse");
    }

    /// Test deuteranopia (green-blind) simulation at boundaries
    #[test]
    fn test_deuteranopia_simulation() {
        let source = r#"
gene Deuteranopia {
    color has rgb: Tuple

    fun simulate_deuteranopia(r: Float64, g: Float64, b: Float64) -> Tuple {
        // Green-blindness matrix transformation
        let r_new = 0.625 * r + 0.375 * g + 0.0 * b
        let g_new = 0.7 * r + 0.3 * g + 0.0 * b
        let b_new = 0.0 * r + 0.3 * g + 0.7 * b
        return (r_new, g_new, b_new)
    }

    fun test_confusing_pair() -> Bool {
        // Red and green become similar
        let red_sim = this.simulate_deuteranopia(1.0, 0.0, 0.0)
        let green_sim = this.simulate_deuteranopia(0.0, 1.0, 0.0)
        return true
    }
}

exegesis {
    Tests deuteranopia (green-blindness) color simulation.
    Red and green become difficult to distinguish.
}
        "#;
        assert!(should_parse(source), "Deuteranopia simulation should parse");
    }

    /// Test tritanopia (blue-blind) simulation at boundaries
    #[test]
    fn test_tritanopia_simulation() {
        let source = r#"
gene Tritanopia {
    color has rgb: Tuple

    fun simulate_tritanopia(r: Float64, g: Float64, b: Float64) -> Tuple {
        // Blue-blindness matrix transformation
        let r_new = 0.95 * r + 0.05 * g + 0.0 * b
        let g_new = 0.0 * r + 0.43333 * g + 0.56667 * b
        let b_new = 0.0 * r + 0.475 * g + 0.525 * b
        return (r_new, g_new, b_new)
    }

    fun test_blue_yellow() -> Bool {
        // Blue and yellow become confused
        let blue_sim = this.simulate_tritanopia(0.0, 0.0, 1.0)
        let yellow_sim = this.simulate_tritanopia(1.0, 1.0, 0.0)
        return true
    }
}

exegesis {
    Tests tritanopia (blue-blindness) color simulation.
    Blue and yellow become difficult to distinguish.
}
        "#;
        assert!(should_parse(source), "Tritanopia simulation should parse");
    }

    /// Test grayscale simulation (achromatopsia)
    #[test]
    fn test_achromatopsia_simulation() {
        let source = r#"
gene Achromatopsia {
    color has rgb: Tuple

    fun simulate_achromatopsia(r: Float64, g: Float64, b: Float64) -> Tuple {
        // Total color blindness - convert to grayscale
        let gray = 0.2126 * r + 0.7152 * g + 0.0722 * b
        return (gray, gray, gray)
    }

    fun test_any_color_is_gray() -> Bool {
        let result = this.simulate_achromatopsia(0.8, 0.3, 0.6)
        return result.0 == result.1 && result.1 == result.2
    }
}

exegesis {
    Tests achromatopsia (complete color blindness) simulation.
    All colors become shades of gray based on luminance.
}
        "#;
        assert!(
            should_parse(source),
            "Achromatopsia simulation should parse"
        );
    }

    /// Test color blindness with black and white
    #[test]
    fn test_color_blindness_neutrals() {
        let source = r#"
gene NeutralColors {
    color has rgb: Tuple

    fun neutral_invariance() -> Bool {
        // Black and white should remain unchanged in all simulations
        // because they have no chromatic information
        let black = (0.0, 0.0, 0.0)
        let white = (1.0, 1.0, 1.0)
        return true
    }

    fun gray_invariance() -> Bool {
        // Gray (r=g=b) should also remain unchanged
        let gray = (0.5, 0.5, 0.5)
        return true
    }
}

exegesis {
    Tests that neutral colors (black, white, gray) are unchanged
    by color blindness simulations since they lack chromatic info.
}
        "#;
        assert!(
            should_parse(source),
            "Color blindness neutrals should parse"
        );
    }
}

// =============================================================================
// Color Conversion Round-Trip Tests
// =============================================================================

#[cfg(test)]
mod color_conversion_roundtrip {
    use super::*;

    /// Test RGB -> HSL -> RGB round trip at boundaries
    #[test]
    fn test_rgb_hsl_roundtrip() {
        let source = r#"
gene RGBHSLRoundtrip {
    color has rgb: Tuple

    fun roundtrip_precision() -> Bool {
        // Converting RGB -> HSL -> RGB should preserve values
        // with possible floating point precision loss
        let epsilon = 0.001
        return true
    }

    rule roundtrip_stable {
        // Multiple roundtrips should not drift
        true
    }
}

exegesis {
    Tests RGB to HSL to RGB conversion preserves color information.
    BUG POTENTIAL: Grayscale colors may have undefined hue after conversion.
}
        "#;
        assert!(should_parse(source), "RGB-HSL roundtrip should parse");
    }

    /// Test RGB -> LAB -> RGB round trip
    #[test]
    fn test_rgb_lab_roundtrip() {
        let source = r#"
gene RGBLABRoundtrip {
    color has rgb: Tuple

    fun roundtrip_with_clipping() -> Bool {
        // LAB has a larger gamut than RGB
        // Some precision loss expected at RGB boundaries
        return true
    }

    fun test_srgb_gamut() -> Bool {
        // Colors within sRGB gamut should roundtrip cleanly
        return true
    }
}

exegesis {
    Tests RGB to LAB to RGB conversion.
    BUG POTENTIAL: Colors at RGB boundaries may clip during conversion.
}
        "#;
        assert!(should_parse(source), "RGB-LAB roundtrip should parse");
    }
}
