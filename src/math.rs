use std::str::FromStr;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use meval::Expr;

pub type CompiledFn = Arc<dyn Fn(f64) -> f64 + 'static>;

/// Compile a textual expression into an executable function of x.
pub fn compile_expr(src: &str) -> Result<CompiledFn> {
    let trimmed = src.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("expression is empty"));
    }

    let expr = Expr::from_str(trimmed).map_err(|err| anyhow!(err.to_string()))?;
    let func = expr.bind("x").map_err(|err| anyhow!(err.to_string()))?;
    let compiled: CompiledFn = Arc::new(move |x: f64| func(x));
    Ok(compiled)
}

/// Evaluate the expression at a single point without caching.
pub fn evaluate_once(src: &str, x: f64) -> Result<f64> {
    compile_expr(src).map(|f| f(x))
}

pub fn prettify_expression(expr: &str) -> String {
    let mut result = String::new();
    let mut chars = expr.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '^' {
            if let Some(&next) = chars.peek() {
                if let Some(super_char) = superscript_for(next) {
                    chars.next();
                    result.push(super_char);
                    continue;
                }
            }
            result.push('^');
        } else {
            result.push(ch);
        }
    }

    let mut prettified = result;
    for (from, to) in [
        ("sqrt", "√"),
        ("pi", "π"),
        ("tau", "τ"),
        ("theta", "θ"),
        ("phi", "φ"),
        ("lambda", "λ"),
    ] {
        prettified = prettified.replace(from, to);
    }

    prettified
}

fn superscript_for(ch: char) -> Option<char> {
    match ch {
        '0' => Some('⁰'),
        '1' => Some('¹'),
        '2' => Some('²'),
        '3' => Some('³'),
        '4' => Some('⁴'),
        '5' => Some('⁵'),
        '6' => Some('⁶'),
        '7' => Some('⁷'),
        '8' => Some('⁸'),
        '9' => Some('⁹'),
        '-' => Some('⁻'),
        '+' => Some('⁺'),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_and_evaluates_basic_expression() {
        let func = compile_expr("x^2 + 2*x + 1").expect("compile");
        let y = func(3.0);
        assert!((y - 16.0).abs() < 1e-9);

        let direct = evaluate_once("x^2 + 2*x + 1", 3.0).expect("evaluate_once");
        assert!((direct - 16.0).abs() < 1e-9);
    }

    #[test]
    fn rejects_empty_expression() {
        assert!(compile_expr("   ").is_err());
    }

    #[test]
    fn prettify_expression_replaces_tokens() {
        let pretty = prettify_expression("sqrt(x)^2 + pi");
        assert_eq!(pretty, "√(x)² + π");
    }
}
