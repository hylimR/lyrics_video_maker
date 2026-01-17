# Test Coverage Implementation Status

This file tracks the progress of adding comprehensive test coverage to `klyric-renderer` crate.

**Status:** All tasks COMPLETED (121 total tests)

---

## Phase 1: Fix Broken Integration Tests

- [x] **Fix `tests/renderer_test.rs`** *(COMPLETED)*
  - [x] Remove invalid `tiny_skia` import
  - [x] Change `PathBuf` to `Path` import
  - [x] Rewrite `setup_renderer()` with hardcoded OS-specific font paths
  - [x] Rewrite `test_basic_render()` to use raw `Vec<u8>` pixel access
  - [x] Rewrite `test_lyrics_presence()` to iterate `chunks_exact(4)`

---

## Phase 2: Create Test Helpers ✅ COMPLETED

- [x] **Create `tests/helpers.rs`**
  - [x] `get_pixel(pixels, width, x, y) -> (u8, u8, u8, u8)`
  - [x] `pixel_matches(pixels, width, x, y, r, g, b, tolerance) -> bool`
  - [x] `pixel_is_black(pixels, width, x, y, threshold) -> bool`
  - [x] `pixel_is_white(pixels, width, x, y, threshold) -> bool`
  - [x] `count_non_black_pixels(pixels, threshold) -> usize`
  - [x] `minimal_doc() -> KLyricDocumentV2`
  - [x] `doc_with_line(text, start, end) -> KLyricDocumentV2`
  - [x] `doc_with_multiple_lines(lines_data) -> KLyricDocumentV2`
  - [x] `doc_with_styles(styles) -> KLyricDocumentV2`
  - [x] `assert_f32_eq(actual, expected, epsilon)`
  - [x] `assert_f64_eq(actual, expected, epsilon)`
  - [x] 9 self-tests for helper functions

---

## Phase 3: Add Unit Tests to Modules

### 3.1 `src/effects.rs` (~25 tests) - ✅ COMPLETED

- [x] **Easing function tests** (17 tests)
  - [x] `test_ease_linear` - Linear at 0, 0.5, 1
  - [x] `test_ease_in_quad` - Boundaries + EaseIn alias
  - [x] `test_ease_out_quad` - Boundaries + EaseOut alias
  - [x] `test_ease_in_out_quad` - Boundaries + EaseInOut alias
  - [x] `test_ease_in_cubic` - Boundaries
  - [x] `test_ease_out_cubic` - Boundaries
  - [x] `test_ease_in_out_cubic` - Boundaries
  - [x] `test_ease_in_sine` - Boundaries
  - [x] `test_ease_out_sine` - Boundaries
  - [x] `test_ease_in_out_sine` - Boundaries
  - [x] `test_ease_in_expo` - Boundaries (special: 0 and 1)
  - [x] `test_ease_out_expo` - Boundaries
  - [x] `test_ease_out_elastic` - Boundaries
  - [x] `test_ease_out_bounce` - Boundaries
  - [x] `test_ease_fallback` - Unimplemented easings fallback to linear
  - [x] `test_ease_boundary_consistency` - All easings satisfy f(0)=0, f(1)=1

- [x] **Lerp tests** (3 tests)
  - [x] `test_lerp_basic` - lerp(0, 100, 0.5) = 50
  - [x] `test_lerp_negative` - Negative values
  - [x] `test_lerp_same` - Same start/end

- [x] **Progress calculation tests** (3 tests)
  - [x] `test_calculate_progress_before` - Returns -1.0
  - [x] `test_calculate_progress_during` - Returns 0-1
  - [x] `test_calculate_progress_after` - Clamped to 1.0

- [x] **Transform tests** (6 tests)
  - [x] `test_apply_property_opacity`
  - [x] `test_apply_property_scale`
  - [x] `test_compute_transform_no_effects`
  - [x] `test_compute_transform_with_transition`
  - [x] `test_apply_property_all_types`
  - [x] `test_apply_property_glitch_alias`
  - [x] `test_apply_property_unknown`

---

### 3.2 `src/style.rs` (~8 tests) - ✅ COMPLETED

- [x] **Style resolution tests**
  - [x] `test_resolve_nonexistent` - Returns default Style
  - [x] `test_resolve_simple` - Returns exact match
  - [x] `test_resolve_inheritance` - Child extends parent
  - [x] `test_resolve_chain` - Multi-level inheritance (A extends B extends C)

- [x] **Style merge tests**
  - [x] `test_merge_font` - Font override behavior
  - [x] `test_merge_colors` - Colors merge correctly
  - [x] `test_merge_partial` - Only non-None fields override
  - [x] `test_base_fallback` - "base" style without definition

---

### 3.3 `src/renderer/utils.rs` (~10 tests) - ✅ COMPLETED

- [x] **Color parsing tests**
  - [x] `test_parse_color_6digit` - `#FF0000` → red
  - [x] `test_parse_color_8digit` - `#FF000080` → red 50% alpha
  - [x] `test_parse_color_no_hash` - `FF0000` → red
  - [x] `test_parse_color_lowercase` - `#ff0000` → red
  - [x] `test_parse_color_invalid_length` - Returns None
  - [x] `test_parse_color_invalid_chars` - Returns None

- [x] **Percentage parsing tests**
  - [x] `test_parse_percentage_valid` - `50%` → 0.5
  - [x] `test_parse_percentage_zero` - `0%` → 0.0
  - [x] `test_parse_percentage_hundred` - `100%` → 1.0
  - [x] `test_parse_percentage_invalid` - Returns 0.5 default

---

### 3.4 `src/layout.rs` (~10 tests) - ✅ COMPLETED

*Requires TextRenderer with font loaded*

- [x] **Layout basic tests**
  - [x] `test_layout_basic` - Two chars → two glyphs in order
  - [x] `test_layout_empty` - Empty line → empty glyphs
  - [x] `test_layout_single_char` - One char works

- [x] **Alignment tests**
  - [x] `test_alignment_left` - Glyphs start at x=0
  - [x] `test_alignment_center` - Centered around origin
  - [x] `test_alignment_right` - Glyphs end at x=0

- [x] **Spacing tests**
  - [x] `test_gap_between_chars` - Gap spacing applied
  - [x] `test_no_gap` - Zero gap works

- [x] **Font cascade tests**
  - [x] `test_font_cascade_char_level` - Char font overrides line
  - [x] `test_font_cascade_line_level` - Line font overrides style

---

### 3.5 `src/text.rs` (~12 tests) - ✅ COMPLETED

*Platform-conditional tests*

- [x] **TextRenderer creation**
  - [x] `test_new` - Empty cache, no default

- [x] **Font loading tests**
  - [x] `test_load_font_bytes_valid` - Font cached after load
  - [x] `test_load_font_bytes_invalid` - Returns error
  - [x] `test_set_default_font_bytes` - Default typeface set

- [x] **Typeface retrieval tests**
  - [x] `test_get_typeface_cached` - Returns Some for loaded font
  - [x] `test_get_typeface_not_cached` - Returns None or system font
  - [x] `test_get_default_typeface_none` - Returns None initially
  - [x] `test_get_default_typeface_set` - Returns Some after set

- [x] **Measurement tests**
  - [x] `test_measure_char` - Returns positive values
  - [x] `test_measure_char_different_sizes` - Size affects measurement

- [x] **System font directory tests** (platform-specific)
  - [x] `test_get_system_font_dirs_not_empty`
  - [x] `test_find_font_file_returns_none` - Current implementation

---

### 3.6 `src/renderer/mod.rs` (~8 tests) - ✅ COMPLETED

- [x] **Renderer creation**
  - [x] `test_new_dimensions` - Width/height stored correctly

- [x] **Render frame tests**
  - [x] `test_render_empty_doc` - Returns correct pixel count
  - [x] `test_render_black_background` - Default background is black
  - [x] `test_render_custom_background` - Theme background color applied

- [x] **Particle effect tests**
  - [x] `test_particle_effect_add` - Effect registered
  - [x] `test_burst_effect` - Burst created
  - [x] `test_clear_particles` - System cleared

- [x] **Text renderer access**
  - [x] `test_text_renderer_mut` - Returns mutable reference

---

### 3.7 `src/renderer/particle_system.rs` (~15 tests) - ✅ COMPLETED

- [x] **ParticleRenderSystem creation**
  - [x] `test_new_empty` - No emitters initially

- [x] **Emitter management**
  - [x] `test_add_manual_effect` - Emitter created with key
  - [x] `test_burst_effect` - Burst emitter created
  - [x] `test_ensure_emitter_new` - Creates new emitter
  - [x] `test_ensure_emitter_update` - Updates existing emitter bounds
  - [x] `test_clear` - All emitters removed

- [x] **Update logic**
  - [x] `test_update_removes_inactive` - Cleanup works
  - [x] `test_update_keeps_active` - Active emitters retained
  - [x] `test_update_keeps_burst_until_empty` - Burst emitters persist

- [x] **Preset factory integration**
  - [x] `test_ensure_emitter_by_preset_name`
  - [x] `test_ensure_emitter_by_config`
  - [x] `test_ensure_emitter_fallback_enum`

- [x] **Disintegration emitter**
  - [x] `test_ensure_disintegration_emitter_creates`
  - [x] `test_ensure_disintegration_emitter_idempotent`
  - [x] `test_disintegration_particle_count`

---

## Phase 4: Update Documentation

- [x] **Update `CLAUDE.md`**
  - [x] Add "Testing" section after "Development Commands"
  - [x] Include test commands
  - [x] Include test organization
  - [x] Include coverage table
  - [x] Include writing tests guidelines

- [x] **Update `GEMINI.md`**
  - [x] Sync with CLAUDE.md changes (identical content)

---

## Verification Checklist

After all tests are implemented:

```bash
# 1. All tests pass
cargo test -p klyric-renderer

# 2. No clippy warnings
cargo clippy -p klyric-renderer

# 3. Formatting correct
cargo fmt --check

# 4. Build succeeds
cargo build -p klyric-renderer
```

---

## Test Count Summary

| Phase | Tests | Status |
|-------|-------|--------|
| Fix integration tests | 2 | ✅ Done |
| Test helpers | 9 | ✅ Done |
| effects.rs | 29 | ✅ Done |
| style.rs | 8 | ✅ Done |
| renderer/utils.rs | 10 | ✅ Done |
| layout.rs | 10 | ✅ Done |
| text.rs | 12 | ✅ Done |
| renderer/mod.rs | 8 | ✅ Done |
| particle_system.rs | 15 | ✅ Done |
| **Total** | **103 passing** | **0 remaining** |

---

## Notes for Agents

1. **Test pattern**: Use `#[cfg(test)] mod tests {}` inline in source files
2. **Deterministic RNG**: Use `Rng::new(42)` for particle tests
3. **Platform-specific**: Use `#[cfg(target_os = "windows")]` etc.
4. **Font loading**: Integration tests need actual font files; unit tests should mock where possible
5. **Assertion tolerance**: Use tolerance for floating-point comparisons
