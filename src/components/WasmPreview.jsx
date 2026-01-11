import React, { useEffect, useRef, useState, useMemo } from 'react';
import { useAppStore } from '@/store/useAppStore';
import { getCachedFont, cacheFont } from '@/utils/fontCache';
import init, { KLyricWasmRenderer } from '../wasm/klyric_renderer';

/**
 * WasmPreview - Unified WASM-based lyric preview component
 * 
 * Renders KLyric documents using the Rust-based klyric-renderer compiled to WASM.
 * Loads fonts from local assets and renders lyrics at the current time.
 */
const WasmPreview = ({ width = 1920, height = 1080, klyricDoc, currentTime, lyrics, selectedFont, availableFonts, globalStyle }) => {
    const canvasRef = useRef(null);
    const rendererRef = useRef(null);
    const [isReady, setIsReady] = useState(false);
    const [fontLoaded, setFontLoaded] = useState(false);
    const [error, setError] = useState(null);
    const docKeyRef = useRef(null);
    const loadedFontsRef = useRef(new Set()); // Track loaded font names
    
    const { addLoadingFont, removeLoadingFont } = useAppStore();

    // Create minimal v2.0 document from legacy lyrics
    const v2Doc = useMemo(() => {
        if (klyricDoc && klyricDoc.version === '2.0' && klyricDoc.project && klyricDoc.lines) {
            return klyricDoc;
        }

        if (lyrics && lyrics.length > 0) {
            const maxEnd = Math.max(...lyrics.map(l => l.endTime || 0), 0);
            
            // Global Font Family (fallback to selectedFont or default)
            const fontFamily = globalStyle?.font?.family || selectedFont?.name || 'Noto Sans SC';
            const fontSize = globalStyle?.font?.size || 72;

            // Prepare Effects
            const effects = {};
            const activeEffects = [];

            // 1. Enter Animation
            const animName = globalStyle?.animation || 'default';
            const animDuration = globalStyle?.animationDuration || 0.5;

            if (animName === 'default') {
                effects['fadeIn'] = {
                    type: "transition",
                    trigger: "enter",
                    duration: animDuration,
                    easing: "easeOutCubic",
                    properties: {
                        opacity: { from: 0.0, to: 1.0 },
                        y: { from: 50.0, to: 0.0 }
                    }
                };
                activeEffects.push('fadeIn');
            } else if (animName !== 'none') {
                // For named presets like 'crossDissolve', 'slideUp', etc.
                // We define a custom effect wrapper to apply duration
                effects['customEnter'] = {
                    type: "transition", // Type is nominal here if preset is used
                    preset: animName,
                    trigger: "enter",
                    duration: animDuration
                };
                activeEffects.push('customEnter');
            }

            // 2. Continuous Effect (Not yet fully wired for duration override in UI, but ready)
            const effectName = globalStyle?.effect || 'none';
            if (effectName !== 'none') {
                if (['wobbly', 'pulse', 'rainbow'].includes(effectName)) {
                     // TODO: These are implemented as 'EffectType::Custom' or shader logic usually?
                     // Currently renderer expects them as names.
                     // For now, push the name directly or define a basic wrapper if supported.
                     // The new renderer logic expects names in `doc.effects` OR preset names.
                     // We will push the name directly to line.effects for legacy ones,
                     // but if we want custom params, we'd need a wrapper.
                     activeEffects.push(effectName);
                } else {
                    // It's likely a transition/preset we want to apply continuously?
                    // Or a particle effect.
                    activeEffects.push(effectName);
                }
            }

            return {
                version: '2.0',
                project: {
                    title: 'Preview',
                    duration: maxEnd + 2,
                    resolution: { width, height },
                    fps: 30
                },
                theme: {
                    background: {
                        type: 'solid',
                        color: '#000000',
                        opacity: 1.0
                    }
                },
                styles: {
                    base: {
                        font: {
                            family: fontFamily,
                            size: fontSize,
                            weight: 700,
                            style: 'normal',
                            letterSpacing: 0
                        },
                        stroke: globalStyle?.stroke,
                        shadow: globalStyle?.shadow,
                        colors: {
                            inactive: { fill: '#888888' },
                            active: { fill: '#FFFF00' },
                            complete: { fill: '#FFFFFF' }
                        }
                    }
                },
                effects: effects,
                lines: lyrics.map((line, lineIdx) => {
                    const chars = [];
                    let charIndex = 0;

                    if (line.syllables && line.syllables.length > 0) {
                        // Build chars from syllables with proper per-character timing
                        for (const syl of line.syllables) {
                            const sylChars = [...(syl.text || syl.char || '')];
                            const sylDuration = (syl.endTime || syl.end || 0) - (syl.startTime || syl.start || 0);
                            const charDur = sylDuration / Math.max(sylChars.length, 1);
                            const sylStart = syl.startTime || syl.start || 0;

                            for (let i = 0; i < sylChars.length; i++) {
                                const charStart = sylStart + (i * charDur);
                                const charEnd = charStart + charDur;

                                const charObj = {
                                    char: sylChars[i],
                                    start: charStart,
                                    end: charEnd
                                };

                                // Apply charCustomizations if present
                                const customization = line.charCustomizations?.[charIndex];
                                if (customization) {
                                    // Map font override
                                    if (customization.font) {
                                        charObj.font = customization.font;
                                    }
                                    if (customization.stroke) charObj.stroke = customization.stroke;
                                    if (customization.shadow) charObj.shadow = customization.shadow;
                                    
                                    charObj.transform = {
                                        x: customization.offsetX || 0,
                                        y: customization.offsetY || 0,
                                        rotation: customization.rotation || 0,
                                        scale: customization.scale ?? 1,
                                        scaleX: 1,
                                        scaleY: 1,
                                        opacity: customization.opacity ?? 1
                                    };
                                    if (customization.effect) {
                                        charObj.effects = [customization.effect];
                                    }
                                }

                                chars.push(charObj);
                                charIndex++;
                            }
                        }
                    } else {
                        // No syllables - create single char entry
                        chars.push({
                            char: line.text || '',
                            start: line.startTime || 0,
                            end: line.endTime || 0
                        });
                    }

                    // Map legacy line-level transform properties
                    const transform = {};
                    if (line.scale !== undefined) transform.scale = line.scale;
                    if (line.rotation !== undefined) transform.rotation = line.rotation;
                    if (line.opacity !== undefined) transform.opacity = line.opacity;

                    return {
                        id: `line-${lineIdx}`,
                        start: line.startTime || 0,
                        end: line.endTime || 0,
                        text: line.text || '',
                        style: 'base',
                        // Map line font override
                        font: line.font || undefined,
                        stroke: line.stroke || undefined,
                        shadow: line.shadow || undefined,
                        effects: [...activeEffects],
                        position: {
                            x: (width / 2) + (line.x || 0),
                            y: (height / 2) + (line.y || 0),
                            anchor: 'center'
                        },
                        transform: Object.keys(transform).length > 0 ? transform : undefined,
                        chars
                    };
                })
            };
        }

        return null;
    }, [klyricDoc, lyrics, width, height, selectedFont, globalStyle?.font?.family, globalStyle?.font?.size, globalStyle?.shadow, globalStyle?.stroke, globalStyle?.animation, globalStyle?.animationDuration, globalStyle?.effect]);

    // Load a font from public folder or URL
    async function loadFont(renderer, name, url) {
        try {
            if (loadedFontsRef.current.has(name)) return true;

            console.log(`🔤 Loading font "${name}" from ${url}...`);
            const response = await fetch(url);
            if (!response.ok) {
                throw new Error(`Failed to fetch font: ${response.status}`);
            }
            const arrayBuffer = await response.arrayBuffer();
            const uint8Array = new Uint8Array(arrayBuffer);

            renderer.load_font(name, uint8Array);
            loadedFontsRef.current.add(name);
            console.log(`✅ Font "${name}" loaded (${uint8Array.length} bytes)`);
            return true;
        } catch (e) {
            console.warn(`⚠️ Could not load font "${name}":`, e.message);
            return false;
        }
    }

    // Load system font if selected
    useEffect(() => {
        if (!isReady || !rendererRef.current) return;
        
        // Determine font to load (Global > Selected)
        const familyName = globalStyle?.font?.family || selectedFont?.name;
        if (!familyName) return;

        // Check if already loaded in this component instance
        if (loadedFontsRef.current.has(familyName)) return;

        // Find font info to get path
        let fontInfo = null;
        if (selectedFont && selectedFont.name === familyName) {
            fontInfo = selectedFont;
        } else if (availableFonts) {
            fontInfo = availableFonts.find(f => f.name === familyName || f.family === familyName);
        }

        if (!fontInfo) return;

        const loadSystemFont = async () => {
             // Only if running in Tauri
             if (!window.__TAURI_INTERNALS__) {
                 console.warn("System font loading skipped (browser mode)");
                 return;
             }
             
             addLoadingFont(fontInfo.name);
             
             try {
                // Check cache first (using path as key for uniqueness)
                let uint8Array = getCachedFont(fontInfo.path);
                let fromCache = true;

                if (!uint8Array) {
                    fromCache = false;
                    const { invoke } = await import('@tauri-apps/api/core');
                    console.log(`🔤 Loading system font: ${fontInfo.name} from ${fontInfo.path}`);
                    const bytes = await invoke('read_font_file', { path: fontInfo.path });
                    uint8Array = new Uint8Array(bytes);

                    // Cache the font data (LRU)
                    cacheFont(fontInfo.path, uint8Array);
                } else {
                    console.log(`⚡ Using cached font: ${fontInfo.name}`);
                }

                rendererRef.current.load_font(fontInfo.name, uint8Array);
                // Also register under family name if different, to be safe
                if (fontInfo.family && fontInfo.family !== fontInfo.name) {
                    rendererRef.current.load_font(fontInfo.family, uint8Array);
                }
                
                loadedFontsRef.current.add(fontInfo.name);
                if (fontInfo.family) loadedFontsRef.current.add(fontInfo.family);
                
                console.log(`✅ Loaded system font: ${fontInfo.name} ${fromCache ? '(cached)' : ''}`);
             } catch (e) {
                 console.error(`❌ Failed to load system font ${fontInfo.name}:`, e);
             } finally {
                 removeLoadingFont(fontInfo.name);
             }
        };
        
        loadSystemFont();
    }, [selectedFont, globalStyle, availableFonts, isReady, addLoadingFont, removeLoadingFont]);

    // Initialize WASM and load fonts
    useEffect(() => {
        let mounted = true;

        async function initWasm() {
            try {
                console.log('🔧 Initializing WASM renderer...');
                await init();
                if (!mounted) return;

                const renderer = new KLyricWasmRenderer(width, height);
                rendererRef.current = renderer;
                console.log('✅ WASM renderer initialized:', width, 'x', height);
                setIsReady(true);

                // Try to load fonts from various sources
                const fontSources = [
                    { name: 'NotoSansSC', url: '/fonts/NotoSansSC-Regular.otf' },
                    { name: 'NotoSansSC', url: '/fonts/NotoSansSC-Regular.ttf' },
                    { name: 'NotoSansSC', url: '/fonts/NotoSansSC-Bold.ttf' },
                    { name: 'Arial', url: '/fonts/arial.ttf' },
                ];

                let loaded = false;
                for (const { name, url } of fontSources) {
                    if (!mounted) return;
                    try {
                        loaded = await loadFont(renderer, name, url);
                        if (loaded) {
                            setFontLoaded(true);
                            break;
                        }
                    } catch {
                        // Continue to next source
                    }
                }

                if (!loaded) {
                    console.warn('⚠️ No fonts loaded - text will not render');
                }

                setError(null);
            } catch (err) {
                console.error('❌ Failed to init WASM:', err);
                setError(err.message);
            }
        }

        initWasm();

        return () => {
            mounted = false;
            if (rendererRef.current) {
                try {
                    rendererRef.current.free();
                } catch { /* ignore */ }
                rendererRef.current = null;
            }
        };
    }, [width, height]);

    // Load document when it changes
    useEffect(() => {
        if (!isReady || !rendererRef.current || !v2Doc) {
            return;
        }

        const docJson = JSON.stringify(v2Doc);
        if (docKeyRef.current === docJson) return;
        docKeyRef.current = docJson;

        try {
            rendererRef.current.load_document(docJson);
            console.log('✅ Document loaded with', v2Doc.lines?.length || 0, 'lines');
        } catch (e) {
            console.error('❌ Failed to load doc:', e);
            setError(String(e));
        }
    }, [isReady, v2Doc]);

    // Render frame on time change
    useEffect(() => {
        if (!isReady || !rendererRef.current || !canvasRef.current) return;

        try {
            const pixels = rendererRef.current.render_frame(currentTime);

            const canvas = canvasRef.current;
            const ctx = canvas.getContext('2d');

            if (pixels && pixels.length > 0) {
                const expectedSize = width * height * 4;
                if (pixels.length === expectedSize) {
                    const imageData = new ImageData(new Uint8ClampedArray(pixels), width, height);
                    ctx.putImageData(imageData, 0, 0);
                } else {
                    ctx.fillStyle = '#220';
                    ctx.fillRect(0, 0, width, height);
                    ctx.fillStyle = '#880';
                    ctx.font = '16px sans-serif';
                    ctx.textAlign = 'center';
                    ctx.fillText(`Size mismatch: ${pixels.length} vs ${expectedSize}`, width / 2, height / 2);
                }
            } else {
                // No pixels rendered
                ctx.fillStyle = '#111';
                ctx.fillRect(0, 0, width, height);
                ctx.fillStyle = '#555';
                ctx.font = '18px sans-serif';
                ctx.textAlign = 'center';
                ctx.fillText(`t=${currentTime.toFixed(2)}s`, width / 2, height / 2 - 20);
                if (!fontLoaded) {
                    ctx.fillStyle = '#a50';
                    ctx.fillText('⚠️ No font loaded - place .ttf in public/fonts/', width / 2, height / 2 + 20);
                }
                if (!v2Doc) {
                    ctx.fillText('No lyrics loaded', width / 2, height / 2 + 50);
                }
            }
        } catch (e) {
            console.error('❌ Render error:', e);
        }
    }, [isReady, currentTime, width, height, v2Doc, fontLoaded]);

    if (error) {
        return (
            <div style={{
                width: '100%',
                aspectRatio: `${width}/${height}`,
                background: '#300',
                color: '#f88',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                flexDirection: 'column',
                padding: '20px',
                textAlign: 'center'
            }}>
                <h3>⚠️ WASM Error</h3>
                <p style={{ fontSize: '12px', maxWidth: '80%' }}>{error}</p>
            </div>
        );
    }

    return (
        <canvas
            ref={canvasRef}
            width={width}
            height={height}
            style={{ width: '100%', height: 'auto', display: 'block', background: '#000' }}
        />
    );
};

export default WasmPreview;
