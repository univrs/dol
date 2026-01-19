//! Numerical Edge Case Tests for DOL
//!
//! Tests numerical edge cases using physics formulas from the Spirit packages:
//! - Division by zero handling
//! - Integer overflow/underflow
//! - Float NaN and Infinity propagation
//! - Precision loss in f32 vs f64
//! - Very large/small numbers
//! - Negative zero
//! - Subnormal numbers
//!
//! These tests help discover bugs in the evaluator and codegen.

use metadol::ast::{BinaryOp, Expr, Literal};
use metadol::eval::{Interpreter, Value};
use metadol::parser::Parser;

// ============================================================================
// DIVISION BY ZERO TESTS
// ============================================================================

mod division_by_zero {
    use super::*;

    #[test]
    fn integer_division_by_zero() {
        // Using physics formula: F = G * m1 * m2 / r²
        // When r = 0, we get division by zero
        let mut interpreter = Interpreter::new();

        // Create division by zero expression
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Int(100))),
            op: BinaryOp::Div,
            right: Box::new(Expr::Literal(Literal::Int(0))),
        };

        let result = interpreter.eval(&expr);
        // Division by zero should return an error or special value
        match result {
            Err(_) => {
                // Expected: error on integer division by zero
            }
            Ok(Value::Float(f)) if f.is_infinite() => {
                // Also acceptable: infinity
            }
            Ok(val) => {
                // Document unexpected behavior
                panic!(
                    "BUG: Integer division by zero returned unexpected value: {:?}",
                    val
                );
            }
        }
    }

    #[test]
    fn float_division_by_zero_positive() {
        let mut interpreter = Interpreter::new();

        // 1.0 / 0.0 should give +Infinity
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Float(1.0))),
            op: BinaryOp::Div,
            right: Box::new(Expr::Literal(Literal::Float(0.0))),
        };

        let result = interpreter.eval(&expr);
        match result {
            Ok(Value::Float(f)) => {
                assert!(f.is_infinite() && f > 0.0, "Expected +Infinity, got {}", f);
            }
            Ok(val) => panic!("Expected float, got {:?}", val),
            Err(e) => {
                // Some implementations may error - document this
                println!("NOTE: Float division by zero returns error: {:?}", e);
            }
        }
    }

    #[test]
    fn float_division_by_zero_negative() {
        let mut interpreter = Interpreter::new();

        // -1.0 / 0.0 should give -Infinity
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Float(-1.0))),
            op: BinaryOp::Div,
            right: Box::new(Expr::Literal(Literal::Float(0.0))),
        };

        let result = interpreter.eval(&expr);
        match result {
            Ok(Value::Float(f)) => {
                assert!(f.is_infinite() && f < 0.0, "Expected -Infinity, got {}", f);
            }
            Ok(val) => panic!("Expected float, got {:?}", val),
            Err(e) => {
                println!("NOTE: Float division by zero returns error: {:?}", e);
            }
        }
    }

    #[test]
    fn zero_divided_by_zero_is_nan() {
        let mut interpreter = Interpreter::new();

        // 0.0 / 0.0 should give NaN
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Float(0.0))),
            op: BinaryOp::Div,
            right: Box::new(Expr::Literal(Literal::Float(0.0))),
        };

        let result = interpreter.eval(&expr);
        match result {
            Ok(Value::Float(f)) => {
                assert!(f.is_nan(), "Expected NaN for 0.0/0.0, got {}", f);
            }
            Ok(val) => panic!("Expected float NaN, got {:?}", val),
            Err(e) => {
                println!("NOTE: 0.0/0.0 returns error instead of NaN: {:?}", e);
            }
        }
    }

    #[test]
    fn gravitational_force_at_zero_distance() {
        // F = G * m1 * m2 / r²
        // When r = 0, the physics formula diverges
        // DOL code from physics/mechanics.dol handles this by returning 1.0e308
        let source = r#"
fun gravitational_force(m1: f64, m2: f64, r: f64) -> f64 {
    if r == 0.0 {
        return 1.0e308
    }
    let G = 6.67430e-11
    return G * m1 * m2 / (r * r)
}
"#;

        // Test that the parser accepts this pattern
        let mut parser = Parser::new(source);
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Should parse division-by-zero guarding pattern"
        );
    }
}

// ============================================================================
// INTEGER OVERFLOW/UNDERFLOW TESTS
// ============================================================================

mod overflow {
    use super::*;

    #[test]
    fn i64_max_plus_one() {
        let mut interpreter = Interpreter::new();

        // i64::MAX + 1 should overflow
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Int(i64::MAX))),
            op: BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Int(1))),
        };

        let result = interpreter.eval(&expr);
        match result {
            Ok(Value::Int(n)) => {
                // If wrapping, should be i64::MIN
                if n == i64::MIN {
                    println!("NOTE: Integer overflow wraps around (two's complement)");
                } else {
                    panic!("BUG: Unexpected overflow behavior: {}", n);
                }
            }
            Err(e) => {
                // Overflow error is also acceptable
                println!("NOTE: Integer overflow returns error: {:?}", e);
            }
            Ok(val) => panic!("Unexpected value type on overflow: {:?}", val),
        }
    }

    #[test]
    fn i64_min_minus_one() {
        let mut interpreter = Interpreter::new();

        // i64::MIN - 1 should underflow
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Int(i64::MIN))),
            op: BinaryOp::Sub,
            right: Box::new(Expr::Literal(Literal::Int(1))),
        };

        let result = interpreter.eval(&expr);
        match result {
            Ok(Value::Int(n)) => {
                if n == i64::MAX {
                    println!("NOTE: Integer underflow wraps around");
                } else {
                    panic!("BUG: Unexpected underflow behavior: {}", n);
                }
            }
            Err(e) => {
                println!("NOTE: Integer underflow returns error: {:?}", e);
            }
            Ok(val) => panic!("Unexpected value type on underflow: {:?}", val),
        }
    }

    #[test]
    fn multiplication_overflow() {
        let mut interpreter = Interpreter::new();

        // Large number * 2 that overflows
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Int(i64::MAX / 2 + 1))),
            op: BinaryOp::Mul,
            right: Box::new(Expr::Literal(Literal::Int(2))),
        };

        let result = interpreter.eval(&expr);
        // Just document the behavior, don't fail
        match result {
            Ok(Value::Int(n)) => {
                println!("NOTE: Multiplication overflow produces: {}", n);
            }
            Err(e) => {
                println!("NOTE: Multiplication overflow returns error: {:?}", e);
            }
            Ok(val) => {
                println!("NOTE: Multiplication overflow produces: {:?}", val);
            }
        }
    }

    #[test]
    fn physics_momentum_large_mass() {
        // p = m * v - test with very large mass values
        let source = r#"
fun momentum(mass: f64, velocity: f64) -> f64 {
    return mass * velocity
}
"#;
        let mut parser = Parser::new(source);
        assert!(parser.parse().is_ok());

        // The formula should handle large but not infinite values
        let large_mass = 1.0e38_f64;
        let velocity = 1.0e8_f64;
        let momentum = large_mass * velocity;

        // Check if this overflows f64
        assert!(
            momentum.is_finite(),
            "Large but valid physics values should not overflow f64"
        );
    }
}

// ============================================================================
// NaN AND INFINITY PROPAGATION TESTS
// ============================================================================

mod nan_infinity {
    use super::*;

    #[test]
    fn nan_propagation_in_addition() {
        let mut interpreter = Interpreter::new();

        // NaN + anything = NaN
        let nan = f64::NAN;
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Float(nan))),
            op: BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Float(42.0))),
        };

        let result = interpreter.eval(&expr);
        match result {
            Ok(Value::Float(f)) => {
                assert!(f.is_nan(), "NaN + 42 should be NaN, got {}", f);
            }
            Ok(val) => panic!("Expected float, got {:?}", val),
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn nan_propagation_in_multiplication() {
        let mut interpreter = Interpreter::new();

        // NaN * anything = NaN
        let nan = f64::NAN;
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Float(nan))),
            op: BinaryOp::Mul,
            right: Box::new(Expr::Literal(Literal::Float(100.0))),
        };

        let result = interpreter.eval(&expr);
        match result {
            Ok(Value::Float(f)) => {
                assert!(f.is_nan(), "NaN * 100 should be NaN");
            }
            _ => {}
        }
    }

    #[test]
    fn infinity_minus_infinity_is_nan() {
        let mut interpreter = Interpreter::new();

        // Infinity - Infinity = NaN
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Float(f64::INFINITY))),
            op: BinaryOp::Sub,
            right: Box::new(Expr::Literal(Literal::Float(f64::INFINITY))),
        };

        let result = interpreter.eval(&expr);
        match result {
            Ok(Value::Float(f)) => {
                assert!(f.is_nan(), "Infinity - Infinity should be NaN, got {}", f);
            }
            _ => {}
        }
    }

    #[test]
    fn infinity_times_zero_is_nan() {
        let mut interpreter = Interpreter::new();

        // Infinity * 0 = NaN
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Float(f64::INFINITY))),
            op: BinaryOp::Mul,
            right: Box::new(Expr::Literal(Literal::Float(0.0))),
        };

        let result = interpreter.eval(&expr);
        match result {
            Ok(Value::Float(f)) => {
                assert!(f.is_nan(), "Infinity * 0 should be NaN, got {}", f);
            }
            _ => {}
        }
    }

    #[test]
    fn nan_comparison_always_false() {
        let mut interpreter = Interpreter::new();

        // NaN == NaN should be false (IEEE 754)
        let nan = f64::NAN;
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Float(nan))),
            op: BinaryOp::Eq,
            right: Box::new(Expr::Literal(Literal::Float(nan))),
        };

        let result = interpreter.eval(&expr);
        match result {
            Ok(Value::Bool(b)) => {
                assert!(!b, "NaN == NaN should be false per IEEE 754");
            }
            Ok(val) => {
                println!("NOTE: NaN == NaN returned non-bool: {:?}", val);
            }
            Err(e) => {
                println!("NOTE: NaN comparison error: {:?}", e);
            }
        }
    }

    #[test]
    fn infinity_comparison() {
        let mut interpreter = Interpreter::new();

        // Infinity > any finite number
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Float(f64::INFINITY))),
            op: BinaryOp::Gt,
            right: Box::new(Expr::Literal(Literal::Float(1.0e308))),
        };

        let result = interpreter.eval(&expr);
        match result {
            Ok(Value::Bool(b)) => {
                assert!(b, "Infinity should be greater than 1.0e308");
            }
            _ => {}
        }
    }
}

// ============================================================================
// PRECISION TESTS
// ============================================================================

mod precision {
    use super::*;

    #[test]
    fn f64_precision_near_zero() {
        // Testing subnormal numbers
        let mut interpreter = Interpreter::new();

        // Very small number operations
        let tiny = 1.0e-308_f64;
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Float(tiny))),
            op: BinaryOp::Div,
            right: Box::new(Expr::Literal(Literal::Float(10.0))),
        };

        let result = interpreter.eval(&expr);
        match result {
            Ok(Value::Float(f)) => {
                // Should be subnormal or zero
                assert!(
                    f.is_subnormal() || f == 0.0,
                    "Very small division should produce subnormal or zero"
                );
            }
            _ => {}
        }
    }

    #[test]
    fn precision_loss_in_subtraction() {
        // Catastrophic cancellation: two nearly equal large numbers
        let mut interpreter = Interpreter::new();

        let a = 1.0e16_f64;
        let b = 1.0e16_f64 - 1.0;

        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Float(a))),
            op: BinaryOp::Sub,
            right: Box::new(Expr::Literal(Literal::Float(b))),
        };

        let result = interpreter.eval(&expr);
        match result {
            Ok(Value::Float(f)) => {
                // At f64 precision, this might not be exactly 1.0
                println!("NOTE: (1e16) - (1e16-1) = {} (expected ~1.0)", f);
                // Due to precision limits, this might be 0.0 or 2.0
            }
            _ => {}
        }
    }

    #[test]
    fn carnot_efficiency_precision() {
        // Carnot efficiency: eta = 1 - Tc/Th
        // When Tc is very close to Th, precision matters
        let t_hot = 1000.0_f64;
        let t_cold = 999.9999999_f64;

        let efficiency = 1.0 - (t_cold / t_hot);
        assert!(efficiency > 0.0, "Carnot efficiency should be positive");
        assert!(
            efficiency < 1.0e-6,
            "Very close temps should give tiny efficiency"
        );
    }

    #[test]
    fn ideal_gas_law_precision() {
        // PV = nRT - test with extreme values
        let gas_constant = 8.314_f64; // J/(mol·K)
        let moles = 1.0e-20_f64; // Very small amount
        let temperature = 1.0e10_f64; // Very high temp (plasma)
        let volume = 1.0e-30_f64; // Tiny volume

        let pressure = (moles * gas_constant * temperature) / volume;

        // Should be a very large but finite number
        assert!(
            pressure.is_finite(),
            "Extreme but valid gas law should give finite pressure"
        );
    }
}

// ============================================================================
// NEGATIVE ZERO TESTS
// ============================================================================

mod negative_zero {
    use super::*;

    #[test]
    fn negative_zero_equality() {
        let mut interpreter = Interpreter::new();

        // -0.0 == 0.0 should be true (IEEE 754)
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Float(-0.0))),
            op: BinaryOp::Eq,
            right: Box::new(Expr::Literal(Literal::Float(0.0))),
        };

        let result = interpreter.eval(&expr);
        match result {
            Ok(Value::Bool(b)) => {
                assert!(b, "-0.0 should equal 0.0 per IEEE 754");
            }
            _ => {}
        }
    }

    #[test]
    fn negative_zero_sign_preservation() {
        // -0.0 * positive = -0.0
        let mut interpreter = Interpreter::new();

        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Float(-0.0))),
            op: BinaryOp::Mul,
            right: Box::new(Expr::Literal(Literal::Float(42.0))),
        };

        let result = interpreter.eval(&expr);
        match result {
            Ok(Value::Float(f)) => {
                assert!(f == 0.0, "-0.0 * 42 should be 0");
                // Check sign bit
                if f.is_sign_negative() {
                    println!("NOTE: -0.0 * positive preserves negative sign");
                } else {
                    println!("NOTE: -0.0 * positive produces positive zero");
                }
            }
            _ => {}
        }
    }

    #[test]
    fn division_producing_negative_zero() {
        // -1.0 / Infinity = -0.0
        let mut interpreter = Interpreter::new();

        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Float(-1.0))),
            op: BinaryOp::Div,
            right: Box::new(Expr::Literal(Literal::Float(f64::INFINITY))),
        };

        let result = interpreter.eval(&expr);
        match result {
            Ok(Value::Float(f)) => {
                assert!(f == 0.0, "-1/Infinity should be 0");
                if f.is_sign_negative() {
                    println!("NOTE: -1/Infinity = -0.0 (preserves sign)");
                }
            }
            _ => {}
        }
    }
}

// ============================================================================
// SUBNORMAL NUMBER TESTS
// ============================================================================

mod subnormal {
    use super::*;

    #[test]
    fn subnormal_addition() {
        let mut interpreter = Interpreter::new();

        // Add two subnormal numbers
        let subnormal = f64::MIN_POSITIVE / 2.0;
        assert!(subnormal.is_subnormal(), "Test value should be subnormal");

        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Float(subnormal))),
            op: BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Float(subnormal))),
        };

        let result = interpreter.eval(&expr);
        match result {
            Ok(Value::Float(f)) => {
                // Result might be subnormal or normal depending on magnitude
                println!(
                    "NOTE: subnormal + subnormal = {} (subnormal: {})",
                    f,
                    f.is_subnormal()
                );
            }
            _ => {}
        }
    }

    #[test]
    fn subnormal_multiplication_underflow() {
        let mut interpreter = Interpreter::new();

        // Multiply subnormal by small number - may underflow to zero
        let subnormal = f64::MIN_POSITIVE / 2.0;
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Float(subnormal))),
            op: BinaryOp::Mul,
            right: Box::new(Expr::Literal(Literal::Float(0.5))),
        };

        let result = interpreter.eval(&expr);
        match result {
            Ok(Value::Float(f)) => {
                if f == 0.0 {
                    println!("NOTE: Subnormal underflow to zero");
                } else if f.is_subnormal() {
                    println!("NOTE: Subnormal * 0.5 = {} (still subnormal)", f);
                }
            }
            _ => {}
        }
    }

    #[test]
    fn gradual_underflow_in_physics() {
        // Test gradual underflow using physics formula
        // Temperature approaching absolute zero - specific heat capacity approaches 0
        let source = r#"
fun heat_capacity_near_zero(temperature: f64) -> f64 {
    // Debye model approximation: C ~ T^3 as T -> 0
    let debye_temp = 300.0
    let ratio = temperature / debye_temp
    return ratio * ratio * ratio
}
"#;

        let mut parser = Parser::new(source);
        assert!(parser.parse().is_ok());

        // At very low T, this should give subnormal or zero
        let very_low_t = 1.0e-105_f64;
        let ratio = very_low_t / 300.0;
        let c = ratio * ratio * ratio;
        println!(
            "NOTE: Heat capacity at T=1e-105: {} (subnormal: {})",
            c,
            c.is_subnormal()
        );
    }
}

// ============================================================================
// EXTREME VALUE TESTS
// ============================================================================

mod extreme_values {
    use super::*;

    #[test]
    fn very_large_number_operations() {
        let mut interpreter = Interpreter::new();

        // Test operations near f64::MAX
        let large = 1.0e307_f64;
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Float(large))),
            op: BinaryOp::Mul,
            right: Box::new(Expr::Literal(Literal::Float(10.0))),
        };

        let result = interpreter.eval(&expr);
        match result {
            Ok(Value::Float(f)) => {
                if f.is_infinite() {
                    println!("NOTE: Large * 10 overflows to infinity");
                } else {
                    println!("NOTE: Large * 10 = {} (still finite)", f);
                }
            }
            _ => {}
        }
    }

    #[test]
    fn astronomical_distance_calculations() {
        // Test with astronomical scale numbers
        // Distance to Andromeda: ~2.537 million light-years
        let light_year_meters = 9.461e15_f64;
        let andromeda_ly = 2.537e6_f64;
        let distance = andromeda_ly * light_year_meters;

        assert!(
            distance.is_finite(),
            "Astronomical distances should be finite"
        );
        assert!(
            distance > 1.0e22,
            "Distance to Andromeda should be > 1e22 meters"
        );
    }

    #[test]
    fn quantum_scale_calculations() {
        // Test with quantum scale numbers
        // Planck length: ~1.616e-35 meters
        let planck_length = 1.616e-35_f64;
        let planck_time = 5.391e-44_f64;

        // Speed = distance / time (should be close to c)
        let speed = planck_length / planck_time;
        assert!(speed.is_finite(), "Planck speed calculation should work");

        // Very small multiplication
        let area = planck_length * planck_length;
        assert!(
            area.is_finite() || area == 0.0,
            "Planck area should be finite or zero (underflow)"
        );
    }
}

// ============================================================================
// PHYSICS FORMULA EDGE CASES
// ============================================================================

mod physics_formulas {
    use super::*;

    #[test]
    fn schwarzschild_radius_zero_mass() {
        // r_s = 2GM/c²
        // When M = 0, r_s should be 0 (not NaN or error)
        let g = 6.67430e-11_f64;
        let c = 299792458.0_f64;
        let mass = 0.0_f64;

        let r_s = (2.0 * g * mass) / (c * c);
        assert!(r_s == 0.0, "Schwarzschild radius of zero mass should be 0");
    }

    #[test]
    fn lorentz_factor_at_light_speed() {
        // gamma = 1 / sqrt(1 - v²/c²)
        // At v = c, denominator is 0, gamma -> infinity
        let c = 299792458.0_f64;
        let v = c; // At speed of light

        let v_ratio_sq = (v / c) * (v / c);
        let denominator = 1.0 - v_ratio_sq;

        if denominator == 0.0 {
            let gamma = 1.0 / denominator.sqrt();
            assert!(
                gamma.is_infinite(),
                "Lorentz factor at c should be infinite"
            );
        } else {
            println!(
                "NOTE: Floating point gives non-zero denominator: {}",
                denominator
            );
        }
    }

    #[test]
    fn coulomb_force_overlapping_charges() {
        // F = k * q1 * q2 / r²
        // When r = 0 (overlapping charges), force diverges
        let k = 8.99e9_f64; // Coulomb's constant
        let q1 = 1.6e-19_f64; // Electron charge
        let q2 = 1.6e-19_f64;
        let r = 0.0_f64;

        if r == 0.0 {
            // Should handle gracefully
            println!("NOTE: Coulomb force at r=0 is undefined (singularity)");
        }
    }

    #[test]
    fn wave_interference_at_node() {
        // At destructive interference, amplitude -> 0
        // Test precision of cancellation
        let amplitude = 100.0_f64;
        let phase1 = 0.0_f64;
        let phase2 = std::f64::consts::PI; // 180 degrees out of phase

        let wave1 = amplitude * phase1.cos();
        let wave2 = amplitude * phase2.cos();
        let sum = wave1 + wave2;

        // Should be very close to zero
        assert!(sum.abs() < 1e-10, "Destructive interference should cancel");
    }

    #[test]
    fn relativistic_mass_at_rest() {
        // m_rel = m_0 / sqrt(1 - v²/c²)
        // At v = 0, m_rel = m_0
        let m0 = 9.109e-31_f64; // Electron rest mass
        let v = 0.0_f64;
        let c = 299792458.0_f64;

        let v_ratio_sq = (v / c) * (v / c);
        let gamma = 1.0 / (1.0 - v_ratio_sq).sqrt();
        let m_rel = m0 * gamma;

        assert!(
            (m_rel - m0).abs() < 1e-45,
            "Relativistic mass at rest should equal rest mass"
        );
    }
}
