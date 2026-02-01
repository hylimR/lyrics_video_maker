## 2024-05-22 - [Performance] Redundant Hex Parsing in Render Loop
Learning: String parsing (`parse_color`) inside the main render loop (`render_line`) is a hidden performance cost. Even with a fast parser, doing it 5-7 times per line per frame adds up (e.g., 20 lines * 7 lookups * 60 fps = 8400 parses/sec).
Action: Implement a cache for resolved values (like `skia_safe::Color`) at the `Renderer` level, keyed by Style name. Pass these resolved values to the inner renderer. This changes O(Frames * Lines) parsing to O(Styles).
