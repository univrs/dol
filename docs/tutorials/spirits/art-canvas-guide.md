# Art Canvas Guide: Visual Spirit to Browser Canvas

This guide shows you how to compile DOL Visual Spirits to WebAssembly and render them in HTML5 Canvas for interactive browser-based visualizations.

## Prerequisites

- Rust with `wasm32-unknown-unknown` target
- `wasm-pack` or manual WASM compilation
- A web server for testing (e.g., `python -m http.server`)

Install the WASM target:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

## Project Structure

Create a project for browser deployment:

```
my-visual-art/
├── dol/
│   └── visual_art.dol      # DOL source
├── src/
│   └── lib.rs              # Rust WASM wrapper
├── www/
│   ├── index.html          # HTML page
│   ├── app.js              # JavaScript integration
│   └── style.css           # Styling
├── Cargo.toml
└── Spirit.dol              # Spirit manifest
```

## Writing the Visual Spirit

### Spirit Manifest

Create `Spirit.dol`:

```dol
spirit VisualArt {
    has name: "visual-art"
    has version: "0.1.0"
    has authors: ["Your Name <you@example.com>"]
    has license: "MIT"

    has lib: "dol/visual_art.dol"

    has dependencies: [
        "@univrs/visual @ >=0.1.0"
    ]

    has features: ["wasm32", "canvas-export"]

    docs {
        Visual art Spirit compiled to WASM for browser rendering.
        Generates Mandelbrot fractals, color gradients, and L-system trees.
    }
}
```

### DOL Source

Create `dol/visual_art.dol`:

```dol
module visual_art @ 0.1.0

use @univrs/visual.color.{ RGB, HSL, ColorGradient, GradientStop, hsl_to_rgb, lerp_color }
use @univrs/visual.fractal.{ Complex, Mandelbrot, Julia }
use @univrs/visual.geometry.{ Point2D }
use @univrs/generative.lsystems.{ LSystem, TurtleConfig, turtle_interpret, tree_lsystem }

// ============================================================================
// MANDELBROT RENDERING
// ============================================================================

pub gen MandelbrotRenderer {
    has width: u32
    has height: u32
    has center_re: f64
    has center_im: f64
    has zoom: f64
    has max_iterations: u32

    pub fun new(width: u32, height: u32) -> MandelbrotRenderer {
        return MandelbrotRenderer {
            width: width,
            height: height,
            center_re: -0.5,
            center_im: 0.0,
            zoom: 1.0,
            max_iterations: 100
        }
    }

    pub fun set_view(center_re: f64, center_im: f64, zoom: f64) -> MandelbrotRenderer {
        return MandelbrotRenderer {
            width: this.width,
            height: this.height,
            center_re: center_re,
            center_im: center_im,
            zoom: zoom,
            max_iterations: this.max_iterations
        }
    }

    pub fun render_pixel(px: u32, py: u32) -> (u8, u8, u8, u8) {
        let aspect = this.width as f64 / this.height as f64
        let scale = 3.0 / this.zoom

        let re = this.center_re + (px as f64 / this.width as f64 - 0.5) * scale * aspect
        let im = this.center_im + (py as f64 / this.height as f64 - 0.5) * scale

        let c = Complex { re: re, im: im }
        let iterations = this.iterate(c)

        if iterations == this.max_iterations {
            return (0, 0, 0, 255)  // Black for points in set
        }

        // Smooth coloring
        let t = iterations as f64 / this.max_iterations as f64
        let hue = 240.0 - t * 240.0  // Blue to red gradient
        let color = hsl_to_rgb(HSL { h: hue, s: 0.8, l: 0.5 })

        return (color.r, color.g, color.b, 255)
    }

    fun iterate(c: Complex) -> u32 {
        let z = Complex { re: 0.0, im: 0.0 }
        let escape_sq = 4.0

        for i in 0..this.max_iterations {
            if z.re * z.re + z.im * z.im > escape_sq {
                return i
            }
            let new_re = z.re * z.re - z.im * z.im + c.re
            let new_im = 2.0 * z.re * z.im + c.im
            z = Complex { re: new_re, im: new_im }
        }

        return this.max_iterations
    }

    docs {
        Renders Mandelbrot set to RGBA pixel data.
        Supports zooming and panning.
    }
}

// ============================================================================
// COLOR GRADIENT GENERATOR
// ============================================================================

pub gen GradientGenerator {
    has width: u32
    has height: u32

    pub fun new(width: u32, height: u32) -> GradientGenerator {
        return GradientGenerator { width: width, height: height }
    }

    pub fun render_linear_gradient(
        start_r: u8, start_g: u8, start_b: u8,
        end_r: u8, end_g: u8, end_b: u8,
        horizontal: bool
    ) -> Vec<u8> {
        let start = RGB { r: start_r, g: start_g, b: start_b }
        let end = RGB { r: end_r, g: end_g, b: end_b }

        let pixels = vec![]
        for y in 0..this.height {
            for x in 0..this.width {
                let t = if horizontal {
                    x as f64 / (this.width - 1) as f64
                } else {
                    y as f64 / (this.height - 1) as f64
                }
                let color = lerp_color(start, end, t)
                pixels.push(color.r)
                pixels.push(color.g)
                pixels.push(color.b)
                pixels.push(255)
            }
        }
        return pixels
    }

    pub fun render_radial_gradient(
        center_r: u8, center_g: u8, center_b: u8,
        edge_r: u8, edge_g: u8, edge_b: u8
    ) -> Vec<u8> {
        let center = RGB { r: center_r, g: center_g, b: center_b }
        let edge = RGB { r: edge_r, g: edge_g, b: edge_b }
        let cx = this.width as f64 / 2.0
        let cy = this.height as f64 / 2.0
        let max_dist = sqrt(cx * cx + cy * cy)

        let pixels = vec![]
        for y in 0..this.height {
            for x in 0..this.width {
                let dx = x as f64 - cx
                let dy = y as f64 - cy
                let dist = sqrt(dx * dx + dy * dy)
                let t = clamp(dist / max_dist, 0.0, 1.0)
                let color = lerp_color(center, edge, t)
                pixels.push(color.r)
                pixels.push(color.g)
                pixels.push(color.b)
                pixels.push(255)
            }
        }
        return pixels
    }

    docs {
        Generates color gradient images.
    }
}

// ============================================================================
// L-SYSTEM TREE RENDERER
// ============================================================================

pub gen TreeRenderer {
    has angle: f64
    has iterations: u32
    has line_length: f64

    pub fun new() -> TreeRenderer {
        return TreeRenderer {
            angle: 25.0,
            iterations: 5,
            line_length: 5.0
        }
    }

    pub fun with_params(angle: f64, iterations: u32, line_length: f64) -> TreeRenderer {
        return TreeRenderer {
            angle: angle,
            iterations: iterations,
            line_length: line_length
        }
    }

    pub fun generate_lines(start_x: f64, start_y: f64) -> Vec<f64> {
        let tree = tree_lsystem(this.angle)
        let grown = tree.grow(this.iterations)

        let config = TurtleConfig {
            start_position: Point2D { x: start_x, y: start_y },
            start_angle: 90.0,
            line_length: this.line_length,
            angle_delta: this.angle,
            length_scale: 1.0
        }

        let result = turtle_interpret(grown, config)

        // Flatten lines to [x1, y1, x2, y2, x1, y1, x2, y2, ...]
        let line_data = vec![]
        for (start, end) in result.lines {
            line_data.push(start.x)
            line_data.push(start.y)
            line_data.push(end.x)
            line_data.push(end.y)
        }
        return line_data
    }

    docs {
        Generates L-system tree as line segments.
    }
}

// ============================================================================
// JULIA SET RENDERER
// ============================================================================

pub gen JuliaRenderer {
    has width: u32
    has height: u32
    has c_re: f64
    has c_im: f64
    has max_iterations: u32

    pub fun new(width: u32, height: u32) -> JuliaRenderer {
        return JuliaRenderer {
            width: width,
            height: height,
            c_re: -0.7,
            c_im: 0.27015,
            max_iterations: 100
        }
    }

    pub fun set_c(c_re: f64, c_im: f64) -> JuliaRenderer {
        return JuliaRenderer {
            width: this.width,
            height: this.height,
            c_re: c_re,
            c_im: c_im,
            max_iterations: this.max_iterations
        }
    }

    pub fun render_pixel(px: u32, py: u32, zoom: f64) -> (u8, u8, u8, u8) {
        let scale = 3.0 / zoom
        let re = (px as f64 / this.width as f64 - 0.5) * scale
        let im = (py as f64 / this.height as f64 - 0.5) * scale

        let z = Complex { re: re, im: im }
        let c = Complex { re: this.c_re, im: this.c_im }
        let iterations = this.iterate(z, c)

        if iterations == this.max_iterations {
            return (0, 0, 0, 255)
        }

        let t = iterations as f64 / this.max_iterations as f64
        let hue = t * 360.0
        let color = hsl_to_rgb(HSL { h: hue, s: 0.9, l: 0.5 })

        return (color.r, color.g, color.b, 255)
    }

    fun iterate(z0: Complex, c: Complex) -> u32 {
        let z = z0
        let escape_sq = 4.0

        for i in 0..this.max_iterations {
            if z.re * z.re + z.im * z.im > escape_sq {
                return i
            }
            let new_re = z.re * z.re - z.im * z.im + c.re
            let new_im = 2.0 * z.re * z.im + c.im
            z = Complex { re: new_re, im: new_im }
        }

        return this.max_iterations
    }

    docs {
        Renders Julia set fractals.
        The c parameter determines the fractal shape.
    }
}

docs {
    Visual Art WASM module for browser canvas rendering.

    Provides renderers for:
    - Mandelbrot set fractals
    - Julia set fractals
    - Color gradients
    - L-system trees

    All renderers output RGBA pixel data or line coordinates
    suitable for direct use with HTML5 Canvas.
}
```

## Rust WASM Wrapper

Create `Cargo.toml`:

```toml
[package]
name = "visual-art-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3"
web-sys = { version = "0.3", features = ["console"] }

[profile.release]
lto = true
opt-level = "z"
```

Create `src/lib.rs`:

```rust
use wasm_bindgen::prelude::*;

// Constants
const PI: f64 = std::f64::consts::PI;

#[wasm_bindgen]
pub struct MandelbrotRenderer {
    width: u32,
    height: u32,
    center_re: f64,
    center_im: f64,
    zoom: f64,
    max_iterations: u32,
}

#[wasm_bindgen]
impl MandelbrotRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> MandelbrotRenderer {
        MandelbrotRenderer {
            width,
            height,
            center_re: -0.5,
            center_im: 0.0,
            zoom: 1.0,
            max_iterations: 100,
        }
    }

    pub fn set_view(&mut self, center_re: f64, center_im: f64, zoom: f64) {
        self.center_re = center_re;
        self.center_im = center_im;
        self.zoom = zoom;
    }

    pub fn set_max_iterations(&mut self, max_iter: u32) {
        self.max_iterations = max_iter;
    }

    /// Renders the entire fractal to RGBA pixel data
    pub fn render(&self) -> Vec<u8> {
        let mut pixels = Vec::with_capacity((self.width * self.height * 4) as usize);
        let aspect = self.width as f64 / self.height as f64;
        let scale = 3.0 / self.zoom;

        for py in 0..self.height {
            for px in 0..self.width {
                let re = self.center_re + (px as f64 / self.width as f64 - 0.5) * scale * aspect;
                let im = self.center_im + (py as f64 / self.height as f64 - 0.5) * scale;

                let iterations = self.iterate(re, im);
                let (r, g, b, a) = self.iteration_to_color(iterations);

                pixels.push(r);
                pixels.push(g);
                pixels.push(b);
                pixels.push(a);
            }
        }

        pixels
    }

    fn iterate(&self, c_re: f64, c_im: f64) -> u32 {
        let mut z_re = 0.0;
        let mut z_im = 0.0;
        let escape_sq = 4.0;

        for i in 0..self.max_iterations {
            let z_re_sq = z_re * z_re;
            let z_im_sq = z_im * z_im;

            if z_re_sq + z_im_sq > escape_sq {
                return i;
            }

            let new_z_re = z_re_sq - z_im_sq + c_re;
            let new_z_im = 2.0 * z_re * z_im + c_im;
            z_re = new_z_re;
            z_im = new_z_im;
        }

        self.max_iterations
    }

    fn iteration_to_color(&self, iterations: u32) -> (u8, u8, u8, u8) {
        if iterations == self.max_iterations {
            return (0, 0, 0, 255);
        }

        let t = iterations as f64 / self.max_iterations as f64;
        let hue = 240.0 - t * 240.0;
        let (r, g, b) = hsl_to_rgb(hue, 0.8, 0.5);
        (r, g, b, 255)
    }
}

#[wasm_bindgen]
pub struct JuliaRenderer {
    width: u32,
    height: u32,
    c_re: f64,
    c_im: f64,
    zoom: f64,
    max_iterations: u32,
}

#[wasm_bindgen]
impl JuliaRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> JuliaRenderer {
        JuliaRenderer {
            width,
            height,
            c_re: -0.7,
            c_im: 0.27015,
            zoom: 1.0,
            max_iterations: 100,
        }
    }

    pub fn set_c(&mut self, c_re: f64, c_im: f64) {
        self.c_re = c_re;
        self.c_im = c_im;
    }

    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom = zoom;
    }

    pub fn render(&self) -> Vec<u8> {
        let mut pixels = Vec::with_capacity((self.width * self.height * 4) as usize);
        let scale = 3.0 / self.zoom;

        for py in 0..self.height {
            for px in 0..self.width {
                let z_re = (px as f64 / self.width as f64 - 0.5) * scale;
                let z_im = (py as f64 / self.height as f64 - 0.5) * scale;

                let iterations = self.iterate(z_re, z_im);
                let (r, g, b, a) = self.iteration_to_color(iterations);

                pixels.push(r);
                pixels.push(g);
                pixels.push(b);
                pixels.push(a);
            }
        }

        pixels
    }

    fn iterate(&self, z_re: f64, z_im: f64) -> u32 {
        let mut z_re = z_re;
        let mut z_im = z_im;
        let escape_sq = 4.0;

        for i in 0..self.max_iterations {
            let z_re_sq = z_re * z_re;
            let z_im_sq = z_im * z_im;

            if z_re_sq + z_im_sq > escape_sq {
                return i;
            }

            let new_z_re = z_re_sq - z_im_sq + self.c_re;
            let new_z_im = 2.0 * z_re * z_im + self.c_im;
            z_re = new_z_re;
            z_im = new_z_im;
        }

        self.max_iterations
    }

    fn iteration_to_color(&self, iterations: u32) -> (u8, u8, u8, u8) {
        if iterations == self.max_iterations {
            return (0, 0, 0, 255);
        }

        let t = iterations as f64 / self.max_iterations as f64;
        let hue = t * 360.0;
        let (r, g, b) = hsl_to_rgb(hue, 0.9, 0.5);
        (r, g, b, 255)
    }
}

#[wasm_bindgen]
pub struct TreeRenderer {
    angle: f64,
    iterations: u32,
    line_length: f64,
}

#[wasm_bindgen]
impl TreeRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> TreeRenderer {
        TreeRenderer {
            angle: 25.0,
            iterations: 5,
            line_length: 5.0,
        }
    }

    pub fn set_params(&mut self, angle: f64, iterations: u32, line_length: f64) {
        self.angle = angle;
        self.iterations = iterations;
        self.line_length = line_length;
    }

    /// Returns line segments as [x1, y1, x2, y2, ...]
    pub fn generate_lines(&self, start_x: f64, start_y: f64) -> Vec<f64> {
        let system = self.create_tree_system();
        let expanded = self.expand_system(&system);
        self.interpret_turtle(&expanded, start_x, start_y)
    }

    fn create_tree_system(&self) -> (String, Vec<(char, String)>) {
        (
            "X".to_string(),
            vec![
                ('X', "F+[[X]-X]-F[-FX]+X".to_string()),
                ('F', "FF".to_string()),
            ],
        )
    }

    fn expand_system(&self, system: &(String, Vec<(char, String)>)) -> String {
        let mut current = system.0.clone();

        for _ in 0..self.iterations {
            let mut next = String::new();
            for c in current.chars() {
                let mut found = false;
                for (symbol, replacement) in &system.1 {
                    if c == *symbol {
                        next.push_str(replacement);
                        found = true;
                        break;
                    }
                }
                if !found {
                    next.push(c);
                }
            }
            current = next;
        }

        current
    }

    fn interpret_turtle(&self, commands: &str, start_x: f64, start_y: f64) -> Vec<f64> {
        let mut lines = Vec::new();
        let mut x = start_x;
        let mut y = start_y;
        let mut angle = -PI / 2.0; // Point upward
        let mut stack: Vec<(f64, f64, f64)> = Vec::new();
        let angle_rad = self.angle * PI / 180.0;

        for c in commands.chars() {
            match c {
                'F' | 'G' => {
                    let new_x = x + self.line_length * angle.cos();
                    let new_y = y + self.line_length * angle.sin();
                    lines.push(x);
                    lines.push(y);
                    lines.push(new_x);
                    lines.push(new_y);
                    x = new_x;
                    y = new_y;
                }
                '+' => {
                    angle += angle_rad;
                }
                '-' => {
                    angle -= angle_rad;
                }
                '[' => {
                    stack.push((x, y, angle));
                }
                ']' => {
                    if let Some((sx, sy, sa)) = stack.pop() {
                        x = sx;
                        y = sy;
                        angle = sa;
                    }
                }
                _ => {}
            }
        }

        lines
    }
}

#[wasm_bindgen]
pub struct GradientRenderer {
    width: u32,
    height: u32,
}

#[wasm_bindgen]
impl GradientRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> GradientRenderer {
        GradientRenderer { width, height }
    }

    pub fn render_linear(
        &self,
        r1: u8, g1: u8, b1: u8,
        r2: u8, g2: u8, b2: u8,
        horizontal: bool,
    ) -> Vec<u8> {
        let mut pixels = Vec::with_capacity((self.width * self.height * 4) as usize);

        for y in 0..self.height {
            for x in 0..self.width {
                let t = if horizontal {
                    x as f64 / (self.width - 1) as f64
                } else {
                    y as f64 / (self.height - 1) as f64
                };

                let r = lerp(r1 as f64, r2 as f64, t) as u8;
                let g = lerp(g1 as f64, g2 as f64, t) as u8;
                let b = lerp(b1 as f64, b2 as f64, t) as u8;

                pixels.push(r);
                pixels.push(g);
                pixels.push(b);
                pixels.push(255);
            }
        }

        pixels
    }

    pub fn render_radial(
        &self,
        r1: u8, g1: u8, b1: u8,
        r2: u8, g2: u8, b2: u8,
    ) -> Vec<u8> {
        let mut pixels = Vec::with_capacity((self.width * self.height * 4) as usize);
        let cx = self.width as f64 / 2.0;
        let cy = self.height as f64 / 2.0;
        let max_dist = (cx * cx + cy * cy).sqrt();

        for y in 0..self.height {
            for x in 0..self.width {
                let dx = x as f64 - cx;
                let dy = y as f64 - cy;
                let dist = (dx * dx + dy * dy).sqrt();
                let t = (dist / max_dist).min(1.0);

                let r = lerp(r1 as f64, r2 as f64, t) as u8;
                let g = lerp(g1 as f64, g2 as f64, t) as u8;
                let b = lerp(b1 as f64, b2 as f64, t) as u8;

                pixels.push(r);
                pixels.push(g);
                pixels.push(b);
                pixels.push(255);
            }
        }

        pixels
    }
}

// Helper functions
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    if s == 0.0 {
        let gray = (l * 255.0) as u8;
        return (gray, gray, gray);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let h_norm = h / 360.0;

    let r = hue_to_rgb(p, q, h_norm + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h_norm);
    let b = hue_to_rgb(p, q, h_norm - 1.0 / 3.0);

    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn hue_to_rgb(p: f64, q: f64, t: f64) -> f64 {
    let mut t = t;
    if t < 0.0 { t += 1.0; }
    if t > 1.0 { t -= 1.0; }

    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}
```

## Compiling to WASM

Build the WASM module:

```bash
wasm-pack build --target web --out-dir www/pkg
```

Or using cargo directly:

```bash
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/visual_art_wasm.wasm --out-dir www/pkg --web
```

## HTML Integration

Create `www/index.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>DOL Visual Art - Canvas Demo</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <div id="app">
        <header>
            <h1>DOL Visual Art</h1>
            <nav>
                <button id="btn-mandelbrot" class="active">Mandelbrot</button>
                <button id="btn-julia">Julia Set</button>
                <button id="btn-tree">L-System Tree</button>
                <button id="btn-gradient">Gradients</button>
            </nav>
        </header>

        <main>
            <div id="canvas-container">
                <canvas id="main-canvas" width="800" height="600"></canvas>
            </div>

            <aside id="controls">
                <!-- Mandelbrot controls -->
                <div id="mandelbrot-controls" class="control-panel">
                    <h3>Mandelbrot Set</h3>
                    <label>
                        Max Iterations:
                        <input type="range" id="mandelbrot-iterations" min="50" max="500" value="100">
                        <span id="mandelbrot-iterations-value">100</span>
                    </label>
                    <p>Click and drag to pan. Scroll to zoom.</p>
                    <button id="mandelbrot-reset">Reset View</button>
                </div>

                <!-- Julia controls -->
                <div id="julia-controls" class="control-panel" style="display:none">
                    <h3>Julia Set</h3>
                    <label>
                        C Real:
                        <input type="range" id="julia-c-re" min="-2" max="2" step="0.01" value="-0.7">
                        <span id="julia-c-re-value">-0.7</span>
                    </label>
                    <label>
                        C Imaginary:
                        <input type="range" id="julia-c-im" min="-2" max="2" step="0.01" value="0.27">
                        <span id="julia-c-im-value">0.27</span>
                    </label>
                    <button id="julia-animate">Animate C</button>
                </div>

                <!-- Tree controls -->
                <div id="tree-controls" class="control-panel" style="display:none">
                    <h3>L-System Tree</h3>
                    <label>
                        Branch Angle:
                        <input type="range" id="tree-angle" min="10" max="45" value="25">
                        <span id="tree-angle-value">25</span>
                    </label>
                    <label>
                        Iterations:
                        <input type="range" id="tree-iterations" min="1" max="7" value="5">
                        <span id="tree-iterations-value">5</span>
                    </label>
                    <label>
                        Line Length:
                        <input type="range" id="tree-length" min="1" max="15" value="5">
                        <span id="tree-length-value">5</span>
                    </label>
                    <button id="tree-regenerate">Regenerate</button>
                </div>

                <!-- Gradient controls -->
                <div id="gradient-controls" class="control-panel" style="display:none">
                    <h3>Color Gradients</h3>
                    <label>
                        Type:
                        <select id="gradient-type">
                            <option value="linear-h">Linear Horizontal</option>
                            <option value="linear-v">Linear Vertical</option>
                            <option value="radial">Radial</option>
                        </select>
                    </label>
                    <label>
                        Start Color:
                        <input type="color" id="gradient-start" value="#ff0000">
                    </label>
                    <label>
                        End Color:
                        <input type="color" id="gradient-end" value="#0000ff">
                    </label>
                </div>
            </aside>
        </main>

        <footer>
            <p>Generated with DOL Visual Spirit v0.1.0</p>
        </footer>
    </div>

    <script type="module" src="app.js"></script>
</body>
</html>
```

## JavaScript Application

Create `www/app.js`:

```javascript
import init, {
    MandelbrotRenderer,
    JuliaRenderer,
    TreeRenderer,
    GradientRenderer
} from './pkg/visual_art_wasm.js';

// Global state
let canvas, ctx;
let currentMode = 'mandelbrot';
let mandelbrotRenderer, juliaRenderer, treeRenderer, gradientRenderer;
let animationId = null;

// Mandelbrot state
let mandelbrotView = { centerRe: -0.5, centerIm: 0, zoom: 1 };
let isDragging = false;
let dragStart = { x: 0, y: 0 };

async function main() {
    // Initialize WASM
    await init();

    // Get canvas and context
    canvas = document.getElementById('main-canvas');
    ctx = canvas.getContext('2d');

    // Create renderers
    mandelbrotRenderer = new MandelbrotRenderer(canvas.width, canvas.height);
    juliaRenderer = new JuliaRenderer(canvas.width, canvas.height);
    treeRenderer = new TreeRenderer();
    gradientRenderer = new GradientRenderer(canvas.width, canvas.height);

    // Setup event listeners
    setupNavigation();
    setupMandelbrotControls();
    setupJuliaControls();
    setupTreeControls();
    setupGradientControls();

    // Initial render
    renderMandelbrot();
}

// ============================================================================
// RENDERING FUNCTIONS
// ============================================================================

function renderMandelbrot() {
    mandelbrotRenderer.set_view(
        mandelbrotView.centerRe,
        mandelbrotView.centerIm,
        mandelbrotView.zoom
    );

    const pixels = mandelbrotRenderer.render();
    drawPixels(pixels);
}

function renderJulia() {
    const pixels = juliaRenderer.render();
    drawPixels(pixels);
}

function renderTree() {
    // Clear canvas with sky gradient
    const skyGradient = ctx.createLinearGradient(0, 0, 0, canvas.height);
    skyGradient.addColorStop(0, '#87CEEB');
    skyGradient.addColorStop(1, '#E0F4FF');
    ctx.fillStyle = skyGradient;
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Get line segments from WASM
    const lines = treeRenderer.generate_lines(canvas.width / 2, canvas.height - 50);

    // Draw lines
    ctx.strokeStyle = '#5D4037';
    ctx.lineWidth = 1;
    ctx.lineCap = 'round';

    for (let i = 0; i < lines.length; i += 4) {
        const x1 = lines[i];
        const y1 = lines[i + 1];
        const x2 = lines[i + 2];
        const y2 = lines[i + 3];

        ctx.beginPath();
        ctx.moveTo(x1, y1);
        ctx.lineTo(x2, y2);
        ctx.stroke();
    }
}

function renderGradient() {
    const type = document.getElementById('gradient-type').value;
    const startColor = hexToRgb(document.getElementById('gradient-start').value);
    const endColor = hexToRgb(document.getElementById('gradient-end').value);

    let pixels;
    if (type === 'radial') {
        pixels = gradientRenderer.render_radial(
            startColor.r, startColor.g, startColor.b,
            endColor.r, endColor.g, endColor.b
        );
    } else {
        const horizontal = type === 'linear-h';
        pixels = gradientRenderer.render_linear(
            startColor.r, startColor.g, startColor.b,
            endColor.r, endColor.g, endColor.b,
            horizontal
        );
    }

    drawPixels(pixels);
}

function drawPixels(pixels) {
    const imageData = ctx.createImageData(canvas.width, canvas.height);
    imageData.data.set(pixels);
    ctx.putImageData(imageData, 0, 0);
}

// ============================================================================
// NAVIGATION
// ============================================================================

function setupNavigation() {
    const buttons = {
        'btn-mandelbrot': 'mandelbrot',
        'btn-julia': 'julia',
        'btn-tree': 'tree',
        'btn-gradient': 'gradient'
    };

    Object.entries(buttons).forEach(([btnId, mode]) => {
        document.getElementById(btnId).addEventListener('click', () => {
            switchMode(mode);
        });
    });
}

function switchMode(mode) {
    // Cancel any running animation
    if (animationId) {
        cancelAnimationFrame(animationId);
        animationId = null;
    }

    // Update button states
    document.querySelectorAll('nav button').forEach(btn => btn.classList.remove('active'));
    document.getElementById(`btn-${mode}`).classList.add('active');

    // Show/hide control panels
    document.querySelectorAll('.control-panel').forEach(panel => panel.style.display = 'none');
    document.getElementById(`${mode}-controls`).style.display = 'block';

    currentMode = mode;

    // Render new mode
    switch (mode) {
        case 'mandelbrot': renderMandelbrot(); break;
        case 'julia': renderJulia(); break;
        case 'tree': renderTree(); break;
        case 'gradient': renderGradient(); break;
    }
}

// ============================================================================
// MANDELBROT CONTROLS
// ============================================================================

function setupMandelbrotControls() {
    const iterSlider = document.getElementById('mandelbrot-iterations');
    const iterValue = document.getElementById('mandelbrot-iterations-value');

    iterSlider.addEventListener('input', () => {
        iterValue.textContent = iterSlider.value;
        mandelbrotRenderer.set_max_iterations(parseInt(iterSlider.value));
        if (currentMode === 'mandelbrot') renderMandelbrot();
    });

    document.getElementById('mandelbrot-reset').addEventListener('click', () => {
        mandelbrotView = { centerRe: -0.5, centerIm: 0, zoom: 1 };
        renderMandelbrot();
    });

    // Pan and zoom
    canvas.addEventListener('mousedown', (e) => {
        if (currentMode !== 'mandelbrot') return;
        isDragging = true;
        dragStart = { x: e.offsetX, y: e.offsetY };
    });

    canvas.addEventListener('mousemove', (e) => {
        if (currentMode !== 'mandelbrot' || !isDragging) return;

        const dx = e.offsetX - dragStart.x;
        const dy = e.offsetY - dragStart.y;

        const scale = 3.0 / mandelbrotView.zoom;
        const aspect = canvas.width / canvas.height;

        mandelbrotView.centerRe -= (dx / canvas.width) * scale * aspect;
        mandelbrotView.centerIm -= (dy / canvas.height) * scale;

        dragStart = { x: e.offsetX, y: e.offsetY };
        renderMandelbrot();
    });

    canvas.addEventListener('mouseup', () => isDragging = false);
    canvas.addEventListener('mouseleave', () => isDragging = false);

    canvas.addEventListener('wheel', (e) => {
        if (currentMode !== 'mandelbrot') return;
        e.preventDefault();

        const zoomFactor = e.deltaY < 0 ? 1.2 : 0.8;
        mandelbrotView.zoom *= zoomFactor;
        renderMandelbrot();
    });
}

// ============================================================================
// JULIA CONTROLS
// ============================================================================

function setupJuliaControls() {
    const cReSlider = document.getElementById('julia-c-re');
    const cImSlider = document.getElementById('julia-c-im');
    const cReValue = document.getElementById('julia-c-re-value');
    const cImValue = document.getElementById('julia-c-im-value');

    function updateJulia() {
        cReValue.textContent = cReSlider.value;
        cImValue.textContent = cImSlider.value;
        juliaRenderer.set_c(parseFloat(cReSlider.value), parseFloat(cImSlider.value));
        if (currentMode === 'julia') renderJulia();
    }

    cReSlider.addEventListener('input', updateJulia);
    cImSlider.addEventListener('input', updateJulia);

    // Animate C parameter
    document.getElementById('julia-animate').addEventListener('click', () => {
        if (animationId) {
            cancelAnimationFrame(animationId);
            animationId = null;
            return;
        }

        let t = 0;
        function animate() {
            t += 0.02;

            // Trace a path through interesting Julia set parameters
            const cRe = 0.7885 * Math.cos(t);
            const cIm = 0.7885 * Math.sin(t);

            cReSlider.value = cRe.toFixed(3);
            cImSlider.value = cIm.toFixed(3);
            cReValue.textContent = cRe.toFixed(3);
            cImValue.textContent = cIm.toFixed(3);

            juliaRenderer.set_c(cRe, cIm);
            renderJulia();

            animationId = requestAnimationFrame(animate);
        }

        animate();
    });
}

// ============================================================================
// TREE CONTROLS
// ============================================================================

function setupTreeControls() {
    const angleSlider = document.getElementById('tree-angle');
    const iterSlider = document.getElementById('tree-iterations');
    const lengthSlider = document.getElementById('tree-length');

    function updateTree() {
        document.getElementById('tree-angle-value').textContent = angleSlider.value;
        document.getElementById('tree-iterations-value').textContent = iterSlider.value;
        document.getElementById('tree-length-value').textContent = lengthSlider.value;

        treeRenderer.set_params(
            parseFloat(angleSlider.value),
            parseInt(iterSlider.value),
            parseFloat(lengthSlider.value)
        );

        if (currentMode === 'tree') renderTree();
    }

    angleSlider.addEventListener('input', updateTree);
    iterSlider.addEventListener('input', updateTree);
    lengthSlider.addEventListener('input', updateTree);

    document.getElementById('tree-regenerate').addEventListener('click', updateTree);
}

// ============================================================================
// GRADIENT CONTROLS
// ============================================================================

function setupGradientControls() {
    const typeSelect = document.getElementById('gradient-type');
    const startColor = document.getElementById('gradient-start');
    const endColor = document.getElementById('gradient-end');

    function updateGradient() {
        if (currentMode === 'gradient') renderGradient();
    }

    typeSelect.addEventListener('change', updateGradient);
    startColor.addEventListener('input', updateGradient);
    endColor.addEventListener('input', updateGradient);
}

// ============================================================================
// UTILITIES
// ============================================================================

function hexToRgb(hex) {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result ? {
        r: parseInt(result[1], 16),
        g: parseInt(result[2], 16),
        b: parseInt(result[3], 16)
    } : { r: 0, g: 0, b: 0 };
}

// Start the application
main();
```

## CSS Styling

Create `www/style.css`:

```css
* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: #1a1a2e;
    color: #eee;
    min-height: 100vh;
}

#app {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
}

header {
    background: #16213e;
    padding: 1rem 2rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
}

header h1 {
    font-size: 1.5rem;
    color: #e94560;
}

nav button {
    background: #0f3460;
    border: none;
    color: #eee;
    padding: 0.5rem 1rem;
    margin-left: 0.5rem;
    border-radius: 4px;
    cursor: pointer;
    transition: background 0.2s;
}

nav button:hover {
    background: #e94560;
}

nav button.active {
    background: #e94560;
}

main {
    display: flex;
    flex: 1;
    padding: 1rem;
    gap: 1rem;
}

#canvas-container {
    flex: 1;
    display: flex;
    justify-content: center;
    align-items: center;
    background: #16213e;
    border-radius: 8px;
    padding: 1rem;
}

canvas {
    border-radius: 4px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
}

aside {
    width: 280px;
    background: #16213e;
    border-radius: 8px;
    padding: 1rem;
}

.control-panel h3 {
    color: #e94560;
    margin-bottom: 1rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid #0f3460;
}

.control-panel label {
    display: block;
    margin-bottom: 1rem;
}

.control-panel input[type="range"] {
    width: 100%;
    margin-top: 0.5rem;
}

.control-panel input[type="color"] {
    width: 100%;
    height: 40px;
    margin-top: 0.5rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
}

.control-panel select {
    width: 100%;
    padding: 0.5rem;
    margin-top: 0.5rem;
    background: #0f3460;
    border: none;
    color: #eee;
    border-radius: 4px;
}

.control-panel button {
    width: 100%;
    padding: 0.75rem;
    margin-top: 1rem;
    background: #e94560;
    border: none;
    color: #fff;
    border-radius: 4px;
    cursor: pointer;
    font-weight: bold;
    transition: background 0.2s;
}

.control-panel button:hover {
    background: #ff6b6b;
}

.control-panel p {
    font-size: 0.85rem;
    color: #888;
    margin-top: 0.5rem;
}

footer {
    text-align: center;
    padding: 1rem;
    color: #666;
    font-size: 0.85rem;
}
```

## Running the Demo

1. Build the WASM module:
```bash
wasm-pack build --target web --out-dir www/pkg
```

2. Serve the files:
```bash
cd www
python -m http.server 8080
```

3. Open `http://localhost:8080` in your browser.

## Animation Loop Pattern

For smooth animations, use `requestAnimationFrame`:

```javascript
class AnimatedFractal {
    constructor(canvas, renderer) {
        this.canvas = canvas;
        this.ctx = canvas.getContext('2d');
        this.renderer = renderer;
        this.isRunning = false;
        this.lastTime = 0;
    }

    start() {
        this.isRunning = true;
        this.lastTime = performance.now();
        this.animate();
    }

    stop() {
        this.isRunning = false;
    }

    animate(currentTime = performance.now()) {
        if (!this.isRunning) return;

        const deltaTime = (currentTime - this.lastTime) / 1000;
        this.lastTime = currentTime;

        this.update(deltaTime);
        this.render();

        requestAnimationFrame((t) => this.animate(t));
    }

    update(dt) {
        // Update animation state
        this.time = (this.time || 0) + dt;
    }

    render() {
        const pixels = this.renderer.render();
        const imageData = this.ctx.createImageData(
            this.canvas.width,
            this.canvas.height
        );
        imageData.data.set(pixels);
        this.ctx.putImageData(imageData, 0, 0);
    }
}
```

## Image Export

Add image export functionality:

```javascript
function exportCanvas(filename = 'artwork.png') {
    const link = document.createElement('a');
    link.download = filename;
    link.href = canvas.toDataURL('image/png');
    link.click();
}

// Add export button
document.getElementById('export-btn').addEventListener('click', () => {
    exportCanvas(`${currentMode}-${Date.now()}.png`);
});
```

## User Interaction

Handle mouse and touch events for interactive fractals:

```javascript
function setupInteraction(canvas, onPan, onZoom) {
    let isPanning = false;
    let lastPos = { x: 0, y: 0 };

    canvas.addEventListener('pointerdown', (e) => {
        isPanning = true;
        lastPos = { x: e.clientX, y: e.clientY };
        canvas.setPointerCapture(e.pointerId);
    });

    canvas.addEventListener('pointermove', (e) => {
        if (!isPanning) return;

        const dx = e.clientX - lastPos.x;
        const dy = e.clientY - lastPos.y;
        lastPos = { x: e.clientX, y: e.clientY };

        onPan(dx, dy);
    });

    canvas.addEventListener('pointerup', (e) => {
        isPanning = false;
        canvas.releasePointerCapture(e.pointerId);
    });

    canvas.addEventListener('wheel', (e) => {
        e.preventDefault();
        const zoomIn = e.deltaY < 0;
        onZoom(zoomIn, e.offsetX, e.offsetY);
    });
}
```

## Next Steps

- **[Art Audio Guide](art-audio-guide.md)**: Add Web Audio API for sound synthesis
- **[Art REPL Guide](art-repl-guide.md)**: Interactive experimentation
- Explore WebGL for hardware-accelerated rendering
- Add touch support for mobile devices

## Summary

You've learned to:
1. **Structure a WASM project** for browser deployment
2. **Write Rust WASM bindings** using wasm-bindgen
3. **Render fractals** to HTML5 Canvas
4. **Implement user interaction** for pan, zoom, and parameter control
5. **Create animation loops** with requestAnimationFrame
6. **Export images** from canvas
7. **Build a complete visual art application** with multiple rendering modes
