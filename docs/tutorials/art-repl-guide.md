# Art REPL Guide: Interactive Creative Coding with DOL

This guide teaches you to use DOL's Art Spirits interactively in the REPL for creative coding. You'll learn to create colors, generate fractals, synthesize sounds, build animations, and export your creations.

## Prerequisites

Install the DOL toolchain with REPL support:

```bash
cargo install --path . --features repl
```

Launch the REPL:

```bash
dol repl
```

## Loading Art Spirits

### Visual Spirit

Load the visual arts toolkit for colors, geometry, and fractals:

```dol
use @univrs/visual.color.{ RGB, HSL, ColorGradient, lerp_color, rgb_to_hsl, hsl_to_rgb }
use @univrs/visual.geometry.{ Point2D, Circle, Polygon, golden_spiral }
use @univrs/visual.fractal.{ Complex, Mandelbrot, Julia, KochSnowflake }
use @univrs/visual.pattern.{ perlin_noise, Noise2D }
```

### Music Spirit

Load the music toolkit for sound synthesis:

```dol
use @univrs/music.synthesis.{ Oscillator, Waveform, Envelope, AudioBuffer, generate_samples }
use @univrs/music.theory.{ Note, Scale, Chord, Mode, note_to_frequency, build_scale }
use @univrs/music.rhythm.{ Tempo, TimeSignature, euclidean_rhythm }
```

### Animation Spirit

Load animation primitives:

```dol
use @univrs/animation.keyframes.{ Keyframe, Track, Animation, EasingFn, evaluate_track }
use @univrs/animation.particles.{ Particle, Emitter, ParticleSystem }
```

### Generative Spirit

Load generative art tools:

```dol
use @univrs/generative.lsystems.{ LSystem, TurtleConfig, turtle_interpret, tree_lsystem }
use @univrs/generative.noise.{ perlin_2d, simplex_2d, fbm, NoiseConfig }
use @univrs/generative.cellular.{ CellGrid, GameOfLife, conway_step }
```

## Working with Colors

### Creating Colors

Create colors in various color spaces:

```dol
// RGB - Red, Green, Blue (0-255)
let red = RGB { r: 255, g: 0, b: 0 }
let forest_green = RGB { r: 34, g: 139, b: 34 }

// RGBA - with transparency
let semi_transparent_blue = RGBA { r: 0, g: 0, b: 255, a: 128 }

// HSL - Hue (0-360), Saturation (0-1), Lightness (0-1)
let bright_orange = HSL { h: 30.0, s: 1.0, l: 0.5 }

// Convert between color spaces
let orange_rgb = hsl_to_rgb(bright_orange)
let red_hsl = rgb_to_hsl(red)

// Check the values
print(red_hsl)  // HSL { h: 0.0, s: 1.0, l: 0.5 }
```

### Color Manipulation

Transform colors interactively:

```dol
// Create a base color in HSL for easy manipulation
let base = HSL { h: 200.0, s: 0.7, l: 0.5 }  // Steel blue

// Rotate hue to get complementary color
let complement = base.rotate(180.0)
print(complement.h)  // 20.0 (orange)

// Adjust saturation and lightness
let desaturated = base.saturate(-0.3)  // Less saturated
let lighter = base.lighten(0.2)        // Lighter version
let darker = base.lighten(-0.2)        // Darker version

// Convert results to RGB
let base_rgb = hsl_to_rgb(base)
let complement_rgb = hsl_to_rgb(complement)
```

### Creating Gradients

Build color gradients for smooth transitions:

```dol
// Create gradient stops
let stop1 = GradientStop { position: 0.0, color: RGB { r: 255, g: 0, b: 0 } }
let stop2 = GradientStop { position: 0.5, color: RGB { r: 255, g: 255, b: 0 } }
let stop3 = GradientStop { position: 1.0, color: RGB { r: 0, g: 255, b: 0 } }

// Create the gradient
let sunset = ColorGradient {
    stops: vec![stop1, stop2, stop3],
    interpolation: InterpolationMode::Linear
}

// Sample colors along the gradient
let color_at_25 = sunset.sample(0.25)  // Orange-ish
let color_at_75 = sunset.sample(0.75)  // Yellow-green
print(color_at_25)  // RGB { r: 255, g: 128, b: 0 }
```

### Color Harmonies

Generate harmonious color palettes:

```dol
// Start with a base hue
let base_hsl = HSL { h: 210.0, s: 0.8, l: 0.5 }  // Blue

// Generate triadic colors (120 degrees apart)
let (c1, c2, c3) = triadic_colors(base_hsl)

// Generate analogous colors (neighbors on color wheel)
let analogous = analogous_colors(base_hsl, 5, 60.0)  // 5 colors, 60 degree spread

// Generate a full palette using harmony rules
let palette = harmonious_palette(base_hsl, ColorHarmony::SplitComplementary, 3)
print(palette.colors.length)  // 3
```

### Color Blending

Interpolate between colors:

```dol
let blue = RGB { r: 0, g: 0, b: 255 }
let red = RGB { r: 255, g: 0, b: 0 }

// Linear interpolation
let purple = lerp_color(blue, red, 0.5)
print(purple)  // RGB { r: 127, g: 0, b: 127 }

// Create a series of blended colors
for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
    let c = lerp_color(blue, red, t)
    print(t, "->", c)
}
```

## Generating Fractals

### Mandelbrot Set

Explore the famous Mandelbrot fractal:

```dol
// Create a Mandelbrot explorer
let mandelbrot = Mandelbrot {
    center: Complex { re: -0.5, im: 0.0 },
    zoom: 1.0,
    max_iterations: 100,
    escape_radius: 2.0
}

// Test if a point is in the set
let c1 = Complex { re: 0.0, im: 0.0 }
let c2 = Complex { re: 1.0, im: 1.0 }
print(mandelbrot.in_set(c1))  // true (in the set)
print(mandelbrot.in_set(c2))  // false (escapes)

// Get iteration count for coloring
let (iterations, z_final) = mandelbrot.iterate(c2)
print(iterations)  // Number of iterations before escape

// Smooth coloring
let smooth = mandelbrot.smooth_iterations(c2)
print(smooth)  // Floating-point iteration count
```

### Julia Sets

Generate Julia set variations:

```dol
// Classic Julia set
let julia = Julia {
    c: Complex { re: -0.7, im: 0.27015 },  // Parameter defines the shape
    center: Complex { re: 0.0, im: 0.0 },
    zoom: 1.0,
    max_iterations: 100,
    escape_radius: 2.0
}

// Sample points
let z = Complex { re: 0.0, im: 0.5 }
let (iter, _) = julia.iterate(z)
print(iter)

// Try different c values for different shapes
let julia_dendrite = Julia {
    c: Complex { re: 0.0, im: 1.0 },  // Dendrite
    center: Complex { re: 0.0, im: 0.0 },
    zoom: 1.0,
    max_iterations: 100,
    escape_radius: 2.0
}

let julia_rabbit = Julia {
    c: Complex { re: -0.123, im: 0.745 },  // Douady's rabbit
    center: Complex { re: 0.0, im: 0.0 },
    zoom: 1.0,
    max_iterations: 100,
    escape_radius: 2.0
}
```

### Koch Snowflake

Create the classic fractal snowflake:

```dol
let snowflake = KochSnowflake {
    center: Point2D { x: 0.0, y: 0.0 },
    radius: 100.0,
    depth: 4
}

let points = snowflake.generate()
print("Generated", points.length, "points")

// The snowflake has infinite perimeter but finite area
// Each iteration multiplies the perimeter by 4/3
```

### L-System Trees

Generate organic plant structures:

```dol
// Create a tree L-system
let tree = tree_lsystem(25.0)  // 25 degree branching angle
print(tree.axiom)  // "X"
print(tree.angle)  // 25.0

// Grow the tree through iterations
let grown = tree.grow(5)
print(grown.length)  // String length grows exponentially

// Configure turtle graphics interpretation
let config = TurtleConfig {
    start_position: Point2D { x: 400.0, y: 600.0 },
    start_angle: 90.0,  // Point upward
    line_length: 5.0,
    angle_delta: tree.angle,
    length_scale: 1.0
}

// Interpret the L-system string as graphics
let result = turtle_interpret(grown, config)
print("Generated", result.lines.length, "line segments")

// Each line segment is a (Point2D, Point2D) tuple
for (start, end) in result.lines[0..5] {
    print(start, "->", end)
}
```

### Other L-Systems

Try classic fractal patterns:

```dol
// Koch curve
let koch = koch_curve_lsystem()
let koch_string = koch.grow(4)

// Sierpinski triangle
let sierpinski = sierpinski_lsystem()
let sierpinski_string = sierpinski.grow(6)

// Dragon curve
let dragon = dragon_curve_lsystem()
let dragon_string = dragon.grow(10)

// Hilbert space-filling curve
let hilbert = hilbert_curve_lsystem()
let hilbert_string = hilbert.grow(5)
```

## Synthesizing Sounds

### Creating Oscillators

Build basic waveforms:

```dol
// Sine wave - pure tone
let sine_osc = Oscillator {
    waveform: Waveform::Sine,
    frequency: 440.0,  // A4 = 440 Hz
    amplitude: 0.8,
    phase: 0.0
}

// Square wave - hollow, clarinet-like
let square_osc = Oscillator {
    waveform: Waveform::Square,
    frequency: 220.0,  // A3
    amplitude: 0.5,
    phase: 0.0
}

// Sawtooth - bright, brassy
let saw_osc = Oscillator {
    waveform: Waveform::Sawtooth,
    frequency: 110.0,  // A2
    amplitude: 0.6,
    phase: 0.0
}

// Sample a single value
let sample = sine_osc.sample(0.001)  // Sample at t=1ms
print(sample)
```

### Shaping with Envelopes

Apply ADSR envelopes for natural sounds:

```dol
// Pad sound - slow attack, sustained
let pad_env = Envelope {
    attack: 0.5,    // 500ms to reach full volume
    decay: 0.3,     // 300ms to decay to sustain
    sustain: 0.7,   // Hold at 70% volume
    release: 1.0    // 1 second release
}

// Plucked string - fast attack, quick decay
let pluck_env = Envelope {
    attack: 0.001,  // 1ms attack
    decay: 0.2,     // 200ms decay
    sustain: 0.0,   // No sustain
    release: 0.1    // 100ms release
}

// Organ - instant attack/release
let organ_env = Envelope {
    attack: 0.01,
    decay: 0.0,
    sustain: 1.0,
    release: 0.01
}

// Sample envelope at a time point
let env_value = pad_env.sample(0.3, 2.0)  // t=0.3s, note_on=2s
print(env_value)
```

### Generating Audio Samples

Create complete audio buffers:

```dol
// Generate 2 seconds of audio
let osc = Oscillator {
    waveform: Waveform::Sine,
    frequency: 440.0,
    amplitude: 0.8,
    phase: 0.0
}
let env = Envelope { attack: 0.1, decay: 0.2, sustain: 0.6, release: 0.3 }

let buffer = generate_samples(osc, env, 2.0)
print("Sample rate:", buffer.sample_rate)
print("Channels:", buffer.channels)
print("Duration:", buffer.duration(), "seconds")
print("Total samples:", buffer.samples.length)

// Check some sample values
print("First sample:", buffer.samples[0])
print("Peak amplitude:", buffer.peak())
print("RMS level:", buffer.rms())
```

### Music Theory: Notes and Scales

Work with musical concepts:

```dol
// Create a note (MIDI style)
let a4 = Note { pitch: 69, octave: 4, duration: 1.0 }  // A4
let c4 = Note { pitch: 60, octave: 4, duration: 1.0 }  // Middle C

// Convert to frequency
let a4_freq = note_to_frequency(a4)
print(a4_freq)  // 440.0 Hz

// Build a major scale
let c_major = build_scale(c4, Mode::Major)
print("C Major scale degrees:", c_major.intervals.length)

// Build chords
let c_maj_chord = chord_from_scale(c_major, 1)  // I chord
let g_maj_chord = chord_from_scale(c_major, 5)  // V chord

// Generate an arpeggio
let arpeggio = arpeggiate(c_maj_chord, "up_down")
print("Arpeggio notes:", arpeggio.notes.length)
```

### Creating Rhythms

Build rhythmic patterns:

```dol
// Set tempo
let tempo = Tempo { bpm: 120.0 }

// Convert beats to seconds
let beat_duration = beats_to_seconds(1.0, tempo)
print(beat_duration)  // 0.5 seconds per beat at 120 BPM

// Create a Euclidean rhythm
let rhythm = euclidean_rhythm(5, 8)  // 5 hits in 8 steps
print(rhythm.hits)  // [true, false, true, false, true, false, true, false]

// Add swing
let swung = swing(rhythm, 0.3)
```

## Creating Animations

### Keyframe Animation

Build keyframe-based animations:

```dol
// Create keyframes for opacity animation
let kf1 = Keyframe { time: 0.0, value: 0.0, easing: EasingFn::EaseOut }
let kf2 = Keyframe { time: 0.5, value: 1.0, easing: EasingFn::Linear }
let kf3 = Keyframe { time: 1.5, value: 1.0, easing: EasingFn::EaseIn }
let kf4 = Keyframe { time: 2.0, value: 0.0, easing: EasingFn::Linear }

// Create a track
let opacity_track = Track {
    keyframes: vec![kf1, kf2, kf3, kf4],
    name: "opacity"
}

// Evaluate at different times
for t in [0.0, 0.25, 0.5, 1.0, 1.5, 1.75, 2.0] {
    let value = evaluate_track(opacity_track, t)
    print("t=", t, "value=", value)
}
```

### Easing Functions

Apply different easing curves:

```dol
// Test various easing functions at t=0.5
let t = 0.5

print("Linear:", ease_linear(t))           // 0.5
print("Ease In Quad:", ease_in_quad(t))    // 0.25
print("Ease Out Quad:", ease_out_quad(t))  // 0.75
print("Ease In Out Cubic:", ease_in_out_cubic(t))  // 0.5

// Bounce effect
print("Bounce Out:", ease_out_bounce(t))

// Elastic effect
print("Elastic Out:", ease_out_elastic(t))

// Custom cubic bezier (CSS-style)
let p1 = Point2D { x: 0.42, y: 0.0 }
let p2 = Point2D { x: 0.58, y: 1.0 }
let custom = bezier_easing(p1, p2, t)
print("Custom bezier:", custom)
```

### Particle Systems

Create dynamic particle effects:

```dol
// Create an emitter
let emitter = Emitter {
    position: Point2D { x: 400.0, y: 300.0 },
    rate: 10.0,  // 10 particles per second
    shape: EmitterShape::Circle
}

// Create initial particles
let particle = Particle {
    position: Point2D { x: 400.0, y: 300.0 },
    velocity: Point2D { x: 50.0, y: -100.0 },
    life: 2.0,
    color: RGB { r: 255, g: 200, b: 100 }
}

// Simulate particle movement
let gravity = Force { direction: Point2D { x: 0.0, y: 98.0 }, strength: 1.0 }
let dt = 0.016  // ~60fps

let updated = update_particle(particle, vec![gravity], dt)
print("New position:", updated.position)
print("Remaining life:", updated.life)
```

## Noise and Patterns

### Perlin Noise

Generate smooth procedural noise:

```dol
// Configure noise
let config = NoiseConfig {
    seed: 42,
    octaves: 4,
    lacunarity: 2.0,
    persistence: 0.5
}

// Sample 2D Perlin noise
let value = perlin_2d(0.5, 0.5, config)
print("Noise at (0.5, 0.5):", value)  // Value in [-1, 1]

// Sample a grid of noise values
for y in 0..5 {
    let row = ""
    for x in 0..10 {
        let n = perlin_2d(x as f64 * 0.1, y as f64 * 0.1, config)
        let char = if n > 0.3 { "#" } else if n > 0.0 { "+" } else if n > -0.3 { "." } else { " " }
        row = row + char
    }
    print(row)
}
```

### Fractional Brownian Motion

Layer noise for natural textures:

```dol
// FBM combines multiple octaves of noise
let config = NoiseConfig {
    seed: 123,
    octaves: 6,
    lacunarity: 2.0,
    persistence: 0.5
}

let fbm_value = fbm(0.5, 0.5, config)
print("FBM value:", fbm_value)

// Turbulence uses absolute values for cloudier effect
let turb_value = turbulence(0.5, 0.5, config)
print("Turbulence:", turb_value)
```

### Cellular Automata

Run Game of Life simulations:

```dol
// Create a grid
let grid = randomize_grid(20, 10, 0.3)  // 30% alive

// Print initial state
print("Initial state:")
for y in 0..grid.height {
    let row = ""
    for x in 0..grid.width {
        let alive = grid.cells[y * grid.width + x]
        row = row + (if alive { "#" } else { "." })
    }
    print(row)
}

// Run one step
let next_grid = conway_step(grid)

print("\nAfter one step:")
for y in 0..next_grid.height {
    let row = ""
    for x in 0..next_grid.width {
        let alive = next_grid.cells[y * next_grid.width + x]
        row = row + (if alive { "#" } else { "." })
    }
    print(row)
}
```

## Exporting Your Creations

### Export to Canvas Data

Prepare visual data for HTML5 Canvas:

```dol
// Generate Mandelbrot pixel data
let mandelbrot = Mandelbrot::default()
let width = 800
let height = 600

// Create pixel array (RGBA format)
let pixels = vec![]
for py in 0..height {
    for px in 0..width {
        let c = mandelbrot.pixel_to_complex(px, py, width, height)
        let smooth_iter = mandelbrot.smooth_iterations(c)

        // Map iteration count to color
        let t = smooth_iter / mandelbrot.max_iterations as f64
        let color = gradient.sample(t)

        pixels.push(color.r)
        pixels.push(color.g)
        pixels.push(color.b)
        pixels.push(255)  // Alpha
    }
}

// Export as JSON for JavaScript consumption
let json = export_pixels_json(pixels, width, height)
print(json)
```

### Export Audio Buffer

Prepare audio for Web Audio API:

```dol
// Generate a melody
let notes = vec![
    Note { pitch: 60, octave: 4, duration: 0.5 },  // C
    Note { pitch: 62, octave: 4, duration: 0.5 },  // D
    Note { pitch: 64, octave: 4, duration: 0.5 },  // E
    Note { pitch: 60, octave: 4, duration: 0.5 },  // C
]

let env = Envelope { attack: 0.01, decay: 0.1, sustain: 0.7, release: 0.2 }

// Generate audio for each note
let combined_buffer = AudioBuffer {
    samples: vec![],
    sample_rate: 44100,
    channels: 1
}

for note in notes {
    let freq = note_to_frequency(note)
    let osc = Oscillator {
        waveform: Waveform::Sine,
        frequency: freq,
        amplitude: 0.5,
        phase: 0.0
    }
    let note_buffer = generate_samples(osc, env, note.duration)
    combined_buffer = mix_buffers(combined_buffer, note_buffer)
}

// Normalize to prevent clipping
let final_buffer = normalize_buffer(combined_buffer)

// Export as WAV or raw float array
let wav_bytes = export_wav(final_buffer)
let float_array = final_buffer.samples
```

### Export Animation Data

Save animation for playback:

```dol
// Create a complete animation
let anim = Animation {
    tracks: vec![
        Track { name: "x", keyframes: vec![
            Keyframe { time: 0.0, value: 0.0, easing: EasingFn::EaseInOut },
            Keyframe { time: 2.0, value: 800.0, easing: EasingFn::Linear }
        ]},
        Track { name: "y", keyframes: vec![
            Keyframe { time: 0.0, value: 300.0, easing: EasingFn::Bounce },
            Keyframe { time: 1.0, value: 100.0, easing: EasingFn::Bounce },
            Keyframe { time: 2.0, value: 300.0, easing: EasingFn::Linear }
        ]},
        Track { name: "scale", keyframes: vec![
            Keyframe { time: 0.0, value: 1.0, easing: EasingFn::Elastic },
            Keyframe { time: 1.0, value: 2.0, easing: EasingFn::Elastic },
            Keyframe { time: 2.0, value: 1.0, easing: EasingFn::Linear }
        ]}
    ],
    duration: 2.0,
    looping: true
}

// Sample animation at 60fps and export
let fps = 60.0
let frame_count = (anim.duration * fps) as u64

let frames = vec![]
for i in 0..frame_count {
    let t = i as f64 / fps
    let state = AnimationState { time: t, playing: true, speed: 1.0, direction: 1 }
    let values = evaluate_animation(anim, state)
    frames.push(values)
}

let json = export_animation_json(frames)
print(json)
```

## Interactive Workflow Tips

### Quick Experimentation

Use the REPL for rapid iteration:

```dol
// Bind intermediate results to variables
let base = HSL { h: 0.0, s: 0.8, l: 0.5 }

// Iterate through hues quickly
for h in [0, 30, 60, 90, 120, 150, 180, 210, 240, 270, 300, 330] {
    let color = HSL { h: h as f64, s: base.s, l: base.l }
    let rgb = hsl_to_rgb(color)
    print(h, ":", rgb.to_hex())
}
```

### Save and Load State

Preserve your work between sessions:

```dol
// Save current state
save_session("my_art_session.dol")

// Load previous session
load_session("my_art_session.dol")
```

### Combining Spirits

Mix different art domains:

```dol
// Use noise to modulate audio
let noise_config = NoiseConfig { seed: 42, octaves: 3, lacunarity: 2.0, persistence: 0.5 }

let osc = Oscillator { waveform: Waveform::Sine, frequency: 440.0, amplitude: 0.8, phase: 0.0 }
let env = Envelope { attack: 0.1, decay: 0.2, sustain: 0.6, release: 0.3 }

// Modulate frequency with noise
let samples = vec![]
for i in 0..44100 {
    let t = i as f64 / 44100.0
    let noise = perlin_2d(t * 5.0, 0.0, noise_config)
    let freq_mod = 440.0 + noise * 50.0  // +-50 Hz variation
    let sample = sin(2.0 * PI * freq_mod * t) * env.sample(t, 1.0)
    samples.push(sample)
}
```

## Next Steps

- **[Art Canvas Guide](art-canvas-guide.md)**: Compile to WASM and render in browser
- **[Art Audio Guide](art-audio-guide.md)**: Connect to Web Audio API for real-time sound
- **[Visual Spirit Reference](../reference/visual-spirit.md)**: Complete API documentation
- **[Music Spirit Reference](../reference/music-spirit.md)**: Full synthesis documentation

## Summary

You've learned to:
1. **Load Art Spirits** for visual, audio, animation, and generative work
2. **Create and manipulate colors** in multiple color spaces
3. **Generate fractals** including Mandelbrot, Julia, and L-systems
4. **Synthesize sounds** with oscillators, envelopes, and music theory
5. **Build animations** with keyframes and easing functions
6. **Work with noise** for procedural content
7. **Export creations** for use in web applications

The DOL REPL enables rapid creative experimentation. Combine these techniques to create unique generative artworks, interactive visualizations, and algorithmic compositions.
