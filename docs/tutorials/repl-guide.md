# DOL REPL Guide

> Interactive exploration of DOL with the VUDO REPL

## Overview

The DOL REPL (Read-Eval-Print Loop) provides an interactive environment for exploring DOL code, testing Spirit functions, and learning the language. This guide demonstrates REPL workflows using examples from the Physics Spirit.

## Starting the REPL

### VUDO Web REPL

Access the web-based REPL at [vudo.univrs.io](https://vudo.univrs.io):

```
1. Navigate to vudo.univrs.io
2. The editor panel (left) accepts DOL source code
3. The output panel (right) shows AST, Rust output, and execution results
4. Use the tabs to switch between views: AST | Rust | WASM | Run
```

### Command-Line REPL

For local development:

```bash
# Start the REPL
cargo run --bin dol-repl

# Or with a Spirit preloaded
cargo run --bin dol-repl -- --load examples/spirits/physics/Spirit.dol
```

## Basic REPL Commands

| Command | Description |
|---------|-------------|
| `:help` | Show available commands |
| `:load <path>` | Load a DOL file or Spirit |
| `:ast` | Toggle AST display |
| `:clear` | Clear the output |
| `:type <expr>` | Show the type of an expression |
| `:quit` | Exit the REPL |

## Working with the Physics Spirit

The Physics Spirit demonstrates DOL's modeling capabilities with physical constants, particles, and mechanics.

### Loading the Physics Spirit

```dol
// Load the Physics Spirit
use physics::*

// Or load specific modules
use physics::constants
use physics::particles
use physics::mechanics
```

### Exploring Physical Constants

```dol
// The lib module provides fundamental constants
>>> SPEED_OF_LIGHT
299792458.0  // m/s

>>> PLANCK_CONSTANT
6.62607015e-34  // J·s

>>> GRAVITATIONAL_CONSTANT
6.6743e-11  // m³/(kg·s²)

>>> ELECTRON_MASS
9.1093837015e-31  // kg
```

### Working with Particles

```dol
// Create particles using the particles module
use physics::particles

// Create fundamental particles
>>> let e = electron()
Particle { name: "Electron", symbol: "e⁻", mass: 9.1093837015e-31, ... }

>>> let p = proton()
Particle { name: "Proton", symbol: "p⁺", mass: 1.67262192369e-27, ... }

// Access particle properties
>>> e.charge
-1.602176634e-19  // Coulombs

>>> p.spin
0.5  // dimensionless

// Calculate particle energy
>>> rest_energy(e)
8.187105776823886e-14  // Joules

>>> rest_energy_mev(e)
0.511  // MeV
```

### Calculating with Mechanics

```dol
use physics::mechanics

// Calculate kinetic energy
>>> let v = 1000.0  // m/s
>>> let m = 10.0    // kg
>>> kinetic_energy(m, v)
5000000.0  // Joules

// Calculate gravitational potential energy
>>> let h = 100.0  // meters
>>> gravitational_potential_energy(m, h)
9810.0  // Joules (using g = 9.81 m/s²)

// Calculate gravitational force between two masses
>>> let m1 = 5.972e24  // Earth's mass (kg)
>>> let m2 = 1.0       // 1 kg object
>>> let r = 6.371e6    // Earth's radius (m)
>>> gravitational_force(m1, m2, r)
9.82  // Newtons (approximately)

// Calculate escape velocity
>>> escape_velocity(m1, r)
11186.0  // m/s (approximately 11.2 km/s)
```

### Relativistic Calculations

```dol
use physics::relativity

// Calculate Lorentz factor (gamma)
>>> let v = 0.9 * SPEED_OF_LIGHT  // 90% speed of light
>>> lorentz_factor(v)
2.294  // dimensionless

// Calculate relativistic kinetic energy
>>> let m = 1.0  // kg
>>> relativistic_kinetic_energy(m, v)
1.16e17  // Joules

// Time dilation
>>> let proper_time = 1.0  // second
>>> time_dilation(proper_time, v)
2.294  // seconds (observer time)

// Length contraction
>>> let proper_length = 1.0  // meter
>>> length_contraction(proper_length, v)
0.436  // meters (observer length)
```

## Interactive Examples

### Example 1: Particle Physics Exploration

```dol
// Session: Exploring the Standard Model

use physics::particles

// Create quarks
>>> let up = up_quark()
>>> let down = down_quark()

// Verify proton composition (uud)
>>> let proton_charge = 2.0 * up.charge + down.charge
>>> proton_charge
1.602176634e-19  // Matches proton charge!

// Verify neutron composition (udd)
>>> let neutron_charge = up.charge + 2.0 * down.charge
>>> neutron_charge
0.0  // Neutral, as expected

// Calculate proton mass from quarks (approximate)
>>> proton_mass_from_quarks(up, up, down)
1.67e-27  // Most mass comes from binding energy
```

### Example 2: Orbital Mechanics

```dol
// Session: Calculate satellite orbits

use physics::mechanics
use physics::constants

// Geostationary orbit calculation
>>> let earth_mass = 5.972e24  // kg
>>> let orbital_period = 86400.0  // 24 hours in seconds

// Calculate required orbital radius
>>> let r = orbital_radius(earth_mass, orbital_period)
>>> r / 1000.0
42164.0  // km from Earth's center

// Calculate orbital velocity
>>> orbital_velocity(earth_mass, r)
3075.0  // m/s

// Calculate gravitational acceleration at that altitude
>>> let altitude = r - 6.371e6  // subtract Earth's radius
>>> altitude / 1000.0
35786.0  // km altitude (matches geostationary orbit!)
```

### Example 3: Quantum Mechanics

```dol
// Session: Quantum calculations

use physics::quantum
use physics::constants

// Calculate photon energy from wavelength
>>> let wavelength = 500e-9  // 500 nm (green light)
>>> photon_energy(wavelength)
3.97e-19  // Joules

>>> photon_energy_ev(wavelength)
2.48  // electron volts

// Calculate de Broglie wavelength
>>> let electron_velocity = 1e6  // m/s
>>> de_broglie_wavelength(ELECTRON_MASS, electron_velocity)
7.27e-10  // meters (0.727 nm)

// Heisenberg uncertainty principle
>>> let delta_x = 1e-10  // 0.1 nm position uncertainty
>>> minimum_momentum_uncertainty(delta_x)
5.27e-25  // kg·m/s

// This gives velocity uncertainty for electron:
>>> 5.27e-25 / ELECTRON_MASS
5.79e5  // m/s uncertainty
```

## REPL Tips and Tricks

### Tab Completion

The REPL supports tab completion for:
- Module names: `physics::<TAB>` shows `constants`, `particles`, `mechanics`
- Function names: `kinetic_<TAB>` completes to `kinetic_energy`
- Variable names: Local variables are autocompleted

### History Navigation

```
↑ / ↓     Navigate through command history
Ctrl+R    Reverse search through history
Ctrl+P    Previous command
Ctrl+N    Next command
```

### Multi-line Input

Use `\` at the end of a line for continuation:

```dol
>>> let complex_calculation = \
...     kinetic_energy(mass, velocity) + \
...     gravitational_potential_energy(mass, height)
```

Or use brackets to span multiple lines:

```dol
>>> let particle = Particle {
...     name: "Custom",
...     mass: 1.0,
...     charge: 0.0
... }
```

### Inspecting Types

```dol
>>> :type electron()
Particle

>>> :type kinetic_energy
fun(f64, f64) -> f64

>>> :type SPEED_OF_LIGHT
f64
```

### Debugging Output

```dol
// Enable verbose mode
>>> :verbose on

>>> kinetic_energy(10.0, 100.0)
[DEBUG] kinetic_energy called with m=10.0, v=100.0
[DEBUG] Computing 0.5 * m * v^2
[DEBUG] Result: 50000.0
50000.0
```

## Working with Multiple Spirits

The REPL can load multiple Spirits simultaneously:

```dol
// Load Physics and Chemistry Spirits
use physics::*
use chemistry::*

// Use constants from both
>>> SPEED_OF_LIGHT  // from physics
299792458.0

>>> AVOGADRO       // from chemistry
6.02214076e23

// Cross-Spirit calculations
>>> let photon_e = photon_energy(500e-9)  // physics
>>> let moles = 1.0
>>> let total_energy = photon_e * AVOGADRO * moles
>>> total_energy
239000.0  // Joules per mole of photons
```

## Saving REPL Sessions

```dol
// Save current session to file
>>> :save session.dol

// Load a previous session
>>> :source session.dol

// Export as notebook
>>> :export session.ipynb
```

## Error Handling in REPL

The REPL provides helpful error messages:

```dol
>>> kinetic_energy(10.0)
Error: Function `kinetic_energy` expects 2 arguments, got 1
  Expected: fun(mass: f64, velocity: f64) -> f64

>>> SPEED_OF_LITE  // typo
Error: Unknown identifier `SPEED_OF_LITE`
  Did you mean: SPEED_OF_LIGHT?

>>> let x: i32 = 3.14
Error: Type mismatch
  Expected: i32
  Found: f64
  Hint: Use `3.14 as i32` for explicit conversion
```

## Summary

The DOL REPL provides a powerful interactive environment for:

1. **Learning DOL** - Explore language features interactively
2. **Testing Spirits** - Validate Spirit implementations
3. **Scientific Computing** - Perform calculations with physical constants
4. **Prototyping** - Quickly test ideas before writing full programs

### Key Commands

| Command | Description |
|---------|-------------|
| `:load <path>` | Load a Spirit or DOL file |
| `:type <expr>` | Show expression type |
| `:ast` | Toggle AST view |
| `:verbose` | Enable debug output |
| `:save <path>` | Save session |
| `:help` | Show all commands |

### Next Steps

- **[CLI Guide](cli-guide.md)** - Learn the command-line tools
- **[WASM Guide](wasm-guide.md)** - Compile DOL to WebAssembly
- **[Spirit Development](spirit-development.md)** - Create your own Spirits
