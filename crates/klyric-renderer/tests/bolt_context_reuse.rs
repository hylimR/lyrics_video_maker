use klyric_renderer::expressions::{EvaluationContext, FastEvaluationContext, ExpressionEvaluator};

#[test]
fn test_fast_context_reuse() {
    // 1. Setup base context
    let eval_ctx = EvaluationContext {
        t: 1.0,
        progress: 0.0,
        width: 100.0,
        height: 100.0,
        index: Some(0),
        count: Some(10),
        ..Default::default()
    };

    // 2. Create FastEvaluationContext (allocation)
    let mut fast_ctx = FastEvaluationContext::new(&eval_ctx);

    // 3. Compile expressions
    let progress_expr = ExpressionEvaluator::compile("progress").unwrap();
    let index_expr = ExpressionEvaluator::compile("index").unwrap();
    let derived_expr = ExpressionEvaluator::compile("progress * 10 + index").unwrap();

    // 4. Initial Check
    assert_eq!(ExpressionEvaluator::evaluate_node_fast(&progress_expr, &fast_ctx).unwrap(), 0.0);
    assert_eq!(ExpressionEvaluator::evaluate_node_fast(&index_expr, &fast_ctx).unwrap(), 0.0);

    // 5. Mutate Progress (Reusing context)
    fast_ctx.set_progress(0.5);

    // 6. Check update
    assert_eq!(ExpressionEvaluator::evaluate_node_fast(&progress_expr, &fast_ctx).unwrap(), 0.5);

    // 7. Mutate Index (Reusing context)
    fast_ctx.set_index(5);

    // 8. Check update
    assert_eq!(ExpressionEvaluator::evaluate_node_fast(&index_expr, &fast_ctx).unwrap(), 5.0);

    // 9. Check complex derived
    // 0.5 * 10 + 5 = 10.0
    assert_eq!(ExpressionEvaluator::evaluate_node_fast(&derived_expr, &fast_ctx).unwrap(), 10.0);
}
