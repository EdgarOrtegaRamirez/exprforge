use exprforge::{evaluate, evaluate_with_env, Environment, Value};

#[test]
fn test_basic_arithmetic() {
    assert_eq!(evaluate("2 + 3").unwrap().as_number(), 5.0);
    assert_eq!(evaluate("10 - 4").unwrap().as_number(), 6.0);
    assert_eq!(evaluate("6 * 7").unwrap().as_number(), 42.0);
    assert_eq!(evaluate("20 / 4").unwrap().as_number(), 5.0);
    assert_eq!(evaluate("17 % 5").unwrap().as_number(), 2.0);
}

#[test]
fn test_operator_precedence() {
    assert_eq!(evaluate("2 + 3 * 4").unwrap().as_number(), 14.0);
    assert_eq!(evaluate("(2 + 3) * 4").unwrap().as_number(), 20.0);
    assert_eq!(evaluate("2 ^ 3 ^ 2").unwrap().as_number(), 512.0);
    assert_eq!(evaluate("-2 ^ 2").unwrap().as_number(), -4.0);
}

#[test]
fn test_functions() {
    let result = evaluate("sqrt(16)").unwrap();
    assert!((result.as_number() - 4.0).abs() < 1e-10);

    let result = evaluate("abs(-42)").unwrap();
    assert_eq!(result.as_number(), 42.0);

    let result = evaluate("max(3, 7, 2)").unwrap();
    assert_eq!(result.as_number(), 7.0);

    let result = evaluate("min(3, 7, 2)").unwrap();
    assert_eq!(result.as_number(), 2.0);

    let result = evaluate("fact(5)").unwrap();
    assert_eq!(result.as_number(), 120.0);
}

#[test]
fn test_trigonometry() {
    let result = evaluate("sin(0)").unwrap();
    assert!(result.as_number().abs() < 1e-10);

    let result = evaluate("cos(0)").unwrap();
    assert!((result.as_number() - 1.0).abs() < 1e-10);

    let result = evaluate("pi").unwrap();
    assert!((result.as_number() - std::f64::consts::PI).abs() < 1e-10);
}

#[test]
fn test_constants() {
    let result = evaluate("e").unwrap();
    assert!((result.as_number() - std::f64::consts::E).abs() < 1e-10);

    let result = evaluate("pi").unwrap();
    assert!((result.as_number() - std::f64::consts::PI).abs() < 1e-10);

    let result = evaluate("phi").unwrap();
    assert!((result.as_number() - 1.618033988749895).abs() < 1e-10);
}

#[test]
fn test_boolean_expressions() {
    assert!(evaluate("1 < 2").unwrap().as_boolean());
    assert!(evaluate("3 > 2").unwrap().as_boolean());
    assert!(evaluate("5 == 5").unwrap().as_boolean());
    assert!(evaluate("5 != 3").unwrap().as_boolean());
    assert!(!evaluate("true && false").unwrap().as_boolean());
    assert!(evaluate("true || false").unwrap().as_boolean());
    assert!(evaluate("!false").unwrap().as_boolean());
}

#[test]
fn test_variables() {
    let mut env = Environment::new();
    env.set("x", Value::Number(10.0));
    env.set("y", Value::Number(20.0));

    let result = evaluate_with_env("x + y", &mut env).unwrap();
    assert_eq!(result.as_number(), 30.0);

    let result = evaluate_with_env("x * y", &mut env).unwrap();
    assert_eq!(result.as_number(), 200.0);
}

#[test]
fn test_assignment() {
    let mut env = Environment::new();
    let result = evaluate_with_env("x = 42", &mut env).unwrap();
    assert_eq!(result.as_number(), 42.0);
    assert!(env.contains("x"));
    assert_eq!(env.get("x"), Some(&Value::Number(42.0)));
}

#[test]
fn test_nested_functions() {
    let result = evaluate("sqrt(abs(-16))").unwrap();
    assert!((result.as_number() - 4.0).abs() < 1e-10);

    let result = evaluate("max(min(10, 20), min(30, 5))").unwrap();
    assert_eq!(result.as_number(), 10.0);
}

#[test]
fn test_unit_arithmetic() {
    let result = evaluate("5 m + 3 m").unwrap();
    assert_eq!(result.as_number(), 8.0);

    let result = evaluate("10 m - 4 m").unwrap();
    assert_eq!(result.as_number(), 6.0);
}

#[test]
fn test_unit_conversion() {
    let result = evaluate("100 cm -> m").unwrap();
    assert!((result.as_number() - 1.0).abs() < 1e-10);

    let result = evaluate("1 km -> m").unwrap();
    assert!((result.as_number() - 1000.0).abs() < 1e-10);

    let result = evaluate("1 mile -> km").unwrap();
    assert!((result.as_number() - 1.609344).abs() < 1e-6);
}

#[test]
fn test_temperature_conversion() {
    let result = evaluate("0 C -> F").unwrap();
    assert!((result.as_number() - 32.0).abs() < 1e-10);

    let result = evaluate("100 C -> F").unwrap();
    assert!((result.as_number() - 212.0).abs() < 1e-10);

    let result = evaluate("32 F -> C").unwrap();
    assert!(result.as_number().abs() < 1e-10);
}

#[test]
fn test_unit_multiplication_division() {
    let result = evaluate("5 m * 3 m").unwrap();
    // 5 m * 3 m = 15 m^2
    assert!((result.as_number() - 15.0).abs() < 1e-10);

    let result = evaluate("10 m / 2 s").unwrap();
    // 10 m / 2 s = 5 m/s
    assert!((result.as_number() - 5.0).abs() < 1e-10);
}

#[test]
fn test_error_division_by_zero() {
    let result = evaluate("1 / 0");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("Division by zero"));
}

#[test]
fn test_error_undefined_variable() {
    let mut env = Environment::new();
    let result = evaluate_with_env("undefined_var", &mut env);
    assert!(result.is_err());
}

#[test]
fn test_error_syntax() {
    assert!(evaluate("2 +").is_err());
    assert!(evaluate("(2 + 3").is_err());
    assert!(evaluate("2 + + 3").is_err());
}

#[test]
fn test_error_incompatible_units() {
    let result = evaluate("5 m + 3 kg");
    assert!(result.is_err());
}

#[test]
fn test_complex_expression() {
    let result = evaluate("2 * (3 + 4) - sqrt(49) + abs(-3)").unwrap();
    // 2 * 7 - 7 + 3 = 14 - 7 + 3 = 10
    assert_eq!(result.as_number(), 10.0);
}

#[test]
fn test_scientific_notation() {
    let result = evaluate("1e2").unwrap();
    assert_eq!(result.as_number(), 100.0);

    let result = evaluate("1.5e3").unwrap();
    assert_eq!(result.as_number(), 1500.0);

    let result = evaluate("2.5E-2").unwrap();
    assert!((result.as_number() - 0.025).abs() < 1e-10);
}

#[test]
fn test_statistics() {
    let result = evaluate("mean(1, 2, 3, 4, 5)").unwrap();
    assert_eq!(result.as_number(), 3.0);

    let result = evaluate("sum(1, 2, 3, 4, 5)").unwrap();
    assert_eq!(result.as_number(), 15.0);

    let result = evaluate("max(1, 2, 3, 4, 5)").unwrap();
    assert_eq!(result.as_number(), 5.0);
}
