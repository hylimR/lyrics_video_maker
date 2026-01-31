use anyhow::{anyhow, Result};
use evalexpr::{
    eval_with_context, ContextWithMutableVariables, DefaultNumericTypes, HashMapContext, Value,
};

#[derive(Debug, Clone)]
pub struct EvaluationContext {
    pub t: f64,
    pub progress: f64,
    pub width: f64,
    pub height: f64,
    pub index: Option<usize>,
    pub count: Option<usize>,
    pub char_width: Option<f64>,
    pub char_height: Option<f64>,
}

impl Default for EvaluationContext {
    fn default() -> Self {
        Self {
            t: 0.0,
            progress: 0.0,
            width: 1920.0,
            height: 1080.0,
            index: None,
            count: None,
            char_width: None,
            char_height: None,
        }
    }
}

pub struct ExpressionEvaluator;

impl ExpressionEvaluator {
    pub fn evaluate(expression: &str, context: &EvaluationContext) -> Result<f64> {
        let mut ctx = HashMapContext::<DefaultNumericTypes>::new();

        // Standard timing variables
        ctx.set_value("t".into(), Value::Float(context.t))?;
        ctx.set_value("progress".into(), Value::Float(context.progress))?;

        // Dimensions
        ctx.set_value("width".into(), Value::Float(context.width))?;
        ctx.set_value("height".into(), Value::Float(context.height))?;

        // Per-character variables
        if let Some(idx) = context.index {
            ctx.set_value("index".into(), Value::Int(idx as i64))?;
            ctx.set_value("i".into(), Value::Int(idx as i64))?; // Short alias
        }
        if let Some(cnt) = context.count {
            ctx.set_value("count".into(), Value::Int(cnt as i64))?;
        }
        if let Some(cw) = context.char_width {
            ctx.set_value("char_width".into(), Value::Float(cw))?;
        }
        if let Some(ch) = context.char_height {
            ctx.set_value("char_height".into(), Value::Float(ch))?;
        }

        // Math constants are built-in to evalexpr (PI, etc.)

        match eval_with_context(expression, &ctx) {
            Ok(Value::Float(f)) => Ok(f),
            Ok(Value::Int(val)) => {
                let i: i64 = val;
                Ok(i as f64)
            }
            Ok(Value::Boolean(b)) => Ok(if b { 1.0 } else { 0.0 }),
            Ok(v) => Err(anyhow!("Expression returned non-numeric value: {:?}", v)),
            Err(e) => Err(anyhow!("Evaluation error: {}", e)),
        }
    }

    /// Verify if an expression string is valid
    pub fn validate(expression: &str) -> bool {
        // Try parsing with dummy context
        let ctx = EvaluationContext::default();
        Self::evaluate(expression, &ctx).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_math() {
        let ctx = EvaluationContext::default();
        assert_eq!(ExpressionEvaluator::evaluate("1 + 1", &ctx).unwrap(), 2.0);
        assert_eq!(
            ExpressionEvaluator::evaluate("10 * 0.5", &ctx).unwrap(),
            5.0
        );
    }

    #[test]
    fn test_variables() {
        let ctx = EvaluationContext {
            t: 2.5,
            index: Some(5),
            ..Default::default()
        };

        assert_eq!(ExpressionEvaluator::evaluate("t * 2", &ctx).unwrap(), 5.0);
        assert_eq!(
            ExpressionEvaluator::evaluate("index + 1", &ctx).unwrap(),
            6.0
        );
        assert_eq!(ExpressionEvaluator::evaluate("i + 1", &ctx).unwrap(), 6.0);
    }

    #[test]
    fn test_logic_to_float() {
        let ctx = EvaluationContext {
            t: 0.5,
            ..Default::default()
        };

        // true -> 1.0
        assert_eq!(ExpressionEvaluator::evaluate("t > 0", &ctx).unwrap(), 1.0);
        // false -> 0.0
        assert_eq!(ExpressionEvaluator::evaluate("t > 1", &ctx).unwrap(), 0.0);
    }

    #[test]
    fn test_complex_expression() {
        let ctx = EvaluationContext {
            t: 0.5,
            index: Some(0),
            ..Default::default()
        };

        // Typical "blink" expression
        // sin(t * PI)
        let val = ExpressionEvaluator::evaluate("math::sin(t * math::consts::PI)", &ctx).unwrap();
        assert!((val - 1.0).abs() < 0.001);
    }
}
