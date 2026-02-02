use anyhow::{anyhow, Result};
use evalexpr::{
    build_operator_tree, eval_with_context, Context, DefaultNumericTypes, EvalexprError,
    EvalexprResult, Node, Value,
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

/// A lightweight context wrapper that avoids HashMap allocations
pub struct FastEvaluationContext {
    t: Value,
    progress: Value,
    width: Value,
    height: Value,
    index: Option<Value>,
    count: Option<Value>,
    char_width: Option<Value>,
    char_height: Option<Value>,
    // Cached constants to return references to
    pi: Value,
    e: Value,
}

impl FastEvaluationContext {
    pub fn new(ctx: &EvaluationContext) -> Self {
        Self {
            t: Value::Float(ctx.t),
            progress: Value::Float(ctx.progress),
            width: Value::Float(ctx.width),
            height: Value::Float(ctx.height),
            index: ctx.index.map(|v| Value::Int(v as i64)),
            count: ctx.count.map(|v| Value::Int(v as i64)),
            char_width: ctx.char_width.map(Value::Float),
            char_height: ctx.char_height.map(Value::Float),
            pi: Value::Float(std::f64::consts::PI),
            e: Value::Float(std::f64::consts::E),
        }
    }

    pub fn set_progress(&mut self, progress: f64) {
        self.progress = Value::Float(progress);
    }

    pub fn set_index(&mut self, index: usize) {
        self.index = Some(Value::Int(index as i64));
    }
}

impl Context for FastEvaluationContext {
    type NumericTypes = DefaultNumericTypes;

    fn get_value(&self, identifier: &str) -> Option<&Value> {
        match identifier {
            "t" => Some(&self.t),
            "progress" => Some(&self.progress),
            "width" => Some(&self.width),
            "height" => Some(&self.height),
            "index" | "i" => self.index.as_ref(),
            "count" => self.count.as_ref(),
            "char_width" => self.char_width.as_ref(),
            "char_height" => self.char_height.as_ref(),
            "PI" | "math::consts::PI" => Some(&self.pi),
            "E" | "math::consts::E" => Some(&self.e),
            _ => None,
        }
    }

    fn call_function(&self, identifier: &str, _argument: &Value) -> EvalexprResult<Value> {
        Err(EvalexprError::FunctionIdentifierNotFound(
            identifier.to_string(),
        ))
    }

    fn are_builtin_functions_disabled(&self) -> bool {
        false
    }

    fn set_builtin_functions_disabled(&mut self, _disabled: bool) -> EvalexprResult<()> {
        Ok(())
    }
}

pub struct ExpressionEvaluator;

impl ExpressionEvaluator {
    /// Pre-compile an expression for reuse
    pub fn compile(expression: &str) -> Result<Node> {
        build_operator_tree(expression).map_err(|e| anyhow!("Compilation error: {}", e))
    }

    /// Evaluate a pre-compiled node
    pub fn evaluate_node(node: &Node, context: &EvaluationContext) -> Result<f64> {
        // Optimization: Use FastEvaluationContext to avoid HashMap allocation
        let ctx = FastEvaluationContext::new(context);
        Self::evaluate_node_fast(node, &ctx)
    }

    /// Evaluate a pre-compiled node using an existing FastEvaluationContext
    pub fn evaluate_node_fast(node: &Node, ctx: &FastEvaluationContext) -> Result<f64> {
        match node.eval_with_context(ctx) {
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

    pub fn evaluate(expression: &str, context: &EvaluationContext) -> Result<f64> {
        // Optimization: Use FastEvaluationContext to avoid HashMap allocation
        let ctx = FastEvaluationContext::new(context);

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
    fn test_compiled_math() {
        let ctx = EvaluationContext::default();
        let node = ExpressionEvaluator::compile("1 + 1").unwrap();
        assert_eq!(
            ExpressionEvaluator::evaluate_node(&node, &ctx).unwrap(),
            2.0
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

    #[test]
    fn test_constants() {
        let ctx = EvaluationContext::default();
        // Test PI
        let val_pi = ExpressionEvaluator::evaluate("PI", &ctx).unwrap();
        assert!((val_pi - std::f64::consts::PI).abs() < 0.0001);

        // Test E
        let val_e = ExpressionEvaluator::evaluate("E", &ctx).unwrap();
        assert!((val_e - std::f64::consts::E).abs() < 0.0001);
    }
}
