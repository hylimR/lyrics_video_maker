## 2024-05-24 - Hoisting Effect Progress Calculation
Learning: Calculating effect progress and easing inside the glyph rendering loop is redundant when the effect parameters (delay, duration) are uniform for the line.
Insight: Even though `EffectType::Transition` might use expressions depending on `char_index`, the *progress* (time 0..1) of the effect depends only on `current_time` and the effect's timing configuration, which are line-constant.
Action: Hoist `EffectEngine::calculate_progress` and `EffectEngine::ease` outside the loop. Pass the pre-calculated `eased_progress` to the inner loop. This saves N (glyphs) * M (effects) expensive math calls (powf, sin, div) per frame.
