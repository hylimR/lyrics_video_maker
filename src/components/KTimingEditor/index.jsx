import { useState, useCallback, useEffect, useRef, useMemo } from 'react';
import './KTimingEditor.css';
import WaveformView from './WaveformView';
import DraggableTimeline from './DraggableTimeline';
import CharacterPropertyPanel from './CharacterPropertyPanel';
import { formatTime } from '@/utils/timeUtils';
import useWaveformData from '@/hooks/useWaveformData';
import FontSelector from '@/components/FontSelector';
import StyleEditor from '@/components/StyleEditor';
import { useAppStore } from '@/store/useAppStore';

/**
 * KTimingEditor.jsx - Aegisub-style K-Timing Editor
 * 
 * Features:
 * - Keyboard-driven syllable timing (press K/Space to mark)
 * - Waveform visualization (Aegisub-style PCM waveform)
 * - Spectrogram visualization (Aegisub-style frequency over time)
 * - Auto-split by character count
 * - Visual timeline with per-character markers
 * - Auto-pause at line end, auto-loop within line
 * 
 * Keyboard Controls:
 * - Space/K: Mark current syllable end
 * - Backspace: Undo last mark
 * - Enter: Confirm and move to next line
 * - Escape: Cancel and close
 * - A: Auto-split (even distribution)
 * - P: Play/Pause
 * - R: Restart line from beginning
 * - Left/Right: Seek ±0.1s
 * - Shift+Left/Right: Seek ±1s
 */
const KTimingEditor = ({
    lyrics,
    currentTime,
    isPlaying,
    audioSource = null, // URL of the audio file for waveform/spectrogram extraction
    hasRealAudio = false, // Whether real audio file is uploaded (not demo mode)
    resolution = { width: 1920, height: 1080 }, // Current project resolution
    onLyricsChange,
    onSeek,
    onPlay,
    onPause,
    onClose,
    onUndo,
    onRedo,
    canUndo = false,
    canRedo = false,
    availableFonts
}) => {
    // Waveform data extraction (Aegisub-style)
    const { getWaveformSlice } = useWaveformData(
        hasRealAudio ? audioSource : null,
        200 // samples per second - higher = more detail
    );



    // Persistent State Initialization
    const {
        ktiming,
        setKTimingIndex,
        setKTimingLoop,
        markSyllable,
        undoMark,
        autoSplitTerm,
        resetLineTiming,
        undo,
        redo
    } = useAppStore();

    const { lineIndex, loopMode, markingIndex, markStartTime } = ktiming;

    const [syllables, setSyllables] = useState([]);

    // UI Local State (Selection/Visuals)
    const [charCustomizations, setCharCustomizations] = useState({});
    const [lineProperties, setLineProperties] = useState({});
    const [editingChar, setEditingChar] = useState(null);
    const [containerWidth, setContainerWidth] = useState(800);
    const containerRef = useRef(null);
    const mainColumnRef = useRef(null);
    const isInternalUpdate = useRef(false);
    const isAutoAdvancing = useRef(false);
    const prevLineIndexRef = useRef(lineIndex);


    const currentLyric = lyrics[lineIndex];
    const chars = useMemo(() => currentLyric ? [...currentLyric.text] : [], [currentLyric]);

    // Handle Resize (Observe Main Column)
    useEffect(() => {
        if (!mainColumnRef.current) return;

        const observer = new ResizeObserver(entries => {
            for (const entry of entries) {
                // Main column width minus internal padding/border if any
                // We use contentRect which excludes padding
                const width = Math.max(100, entry.contentRect.width);
                setContainerWidth(width);
            }
        });

        observer.observe(mainColumnRef.current);
        return () => observer.disconnect();
    }, []);

    // Initial Save state on change (lineIndex / loopMode) is now handled by store persistence


    // Push updates to lyrics (Real-time sync)
    const pushUpdates = useCallback((syls, customs, lineProps = lineProperties) => {
        if (!currentLyric) return;

        isInternalUpdate.current = true;
        const updatedLyrics = [...lyrics];
        const updatedLine = {
            ...currentLyric,
            syllables: syls,
        };

        if (Object.keys(customs).length > 0) {
            updatedLine.charCustomizations = customs;
        } else {
            delete updatedLine.charCustomizations;
        }

        // Apply line-level properties (KLYRIC format)
        if (Object.keys(lineProps).length > 0) {
            if (lineProps.layout) updatedLine.layout = lineProps.layout;
            if (lineProps.transform) updatedLine.transform = lineProps.transform;
            if (lineProps.style) updatedLine.style = lineProps.style;
            if (lineProps.font) updatedLine.font = lineProps.font;
            if (lineProps.stroke) updatedLine.stroke = lineProps.stroke;
            if (lineProps.shadow) updatedLine.shadow = lineProps.shadow;
            if (lineProps.effect) updatedLine.effect = lineProps.effect;
            if (lineProps.animation) updatedLine.animation = lineProps.animation;
        }

        updatedLyrics[lineIndex] = updatedLine;
        onLyricsChange(updatedLyrics, `K-Timing Update`);
    }, [currentLyric, lyrics, lineIndex, onLyricsChange, lineProperties]);

    // IMMEDIATE SAVE on Drag/Modification - OPTIMIZED
    // We split into local update (visual) and commit (save)
    const handleSyllablesLocalUpdate = useCallback((newSyls) => {
        setSyllables(newSyls);
    }, []);

    const handleTimelineSave = useCallback(() => {
        pushUpdates(syllables, charCustomizations);
    }, [pushUpdates, syllables, charCustomizations]);

    // For non-drag actions (Mark, Auto, etc), we update and save immediately
    const updateSyllablesAndSave = useCallback((newSyls) => {
        setSyllables(newSyls);
        pushUpdates(newSyls, charCustomizations);
    }, [pushUpdates, charCustomizations]);

    // Handle per-character customization changes with real-time sync
    const handleCharCustomizationChange = useCallback((charIndex, customization) => {
        const newCustoms = { ...charCustomizations, [charIndex]: customization };
        setCharCustomizations(newCustoms);
        pushUpdates(syllables, newCustoms, lineProperties);
    }, [charCustomizations, pushUpdates, syllables, lineProperties]);

    // Handle line-level property changes
    const handleLinePropertyChange = useCallback((property, value) => {
        const newProps = { ...lineProperties };

        // Handle nested properties (e.g., 'transform.x', 'layout.mode')
        // Handle nested properties (e.g., 'transform.x', 'layout.mode')
        const parts = property.split('.');
        if (parts.length === 2) {
            const [category, key] = parts;
            // Create shallow copy of category object or empty object if undefined
            const categoryObj = { ...(newProps[category] || {}) };

            // If value is null/empty, remove the key; otherwise set it
            if (value === null || value === '' || value === undefined) {
                delete categoryObj[key];
            } else {
                categoryObj[key] = value;
            }

            // If category object is empty, remove the category from props; otherwise update it
            if (Object.keys(categoryObj).length === 0) {
                delete newProps[category];
            } else {
                newProps[category] = categoryObj;
            }
        } else {
            if (value === null || value === '' || value === undefined) {
                delete newProps[property];
            } else {
                newProps[property] = value;
            }
        }

        setLineProperties(newProps);
        pushUpdates(syllables, charCustomizations, newProps);
    }, [lineProperties, pushUpdates, syllables, charCustomizations]);

    // Map lineProperties to StyleEditor values
    const lineEditorValues = useMemo(() => ({
        layoutMode: lineProperties.layout?.mode,
        layoutAlign: lineProperties.layout?.align,
        layoutGap: lineProperties.layout?.gap,
        transformX: lineProperties.transform?.x,
        transformY: lineProperties.transform?.y,
        transformRotation: lineProperties.transform?.rotation,
        transformScale: lineProperties.transform?.scale,
        transformOpacity: lineProperties.transform?.opacity,
        fontFamily: lineProperties.font?.family,
        fontSize: lineProperties.font?.size,
        strokeWidth: lineProperties.stroke?.width,
        strokeColor: lineProperties.stroke?.color,
        shadowBlur: lineProperties.shadow?.blur,
        shadowX: lineProperties.shadow?.x,
        shadowY: lineProperties.shadow?.y,
        shadowColor: lineProperties.shadow?.color,
        effect: lineProperties.effect,
        animation: lineProperties.animation,
    }), [lineProperties]);

    const handleStyleEditorChange = useCallback((key, value) => {
        let propPath = key;
        switch (key) {
            case 'layoutMode': propPath = 'layout.mode'; break;
            case 'layoutAlign': propPath = 'layout.align'; break;
            case 'layoutGap': propPath = 'layout.gap'; break;
            case 'fontFamily': propPath = 'font.family'; break;
            case 'fontSize': propPath = 'font.size'; break;
            case 'strokeWidth': propPath = 'stroke.width'; break;
            case 'strokeColor': propPath = 'stroke.color'; break;
            case 'shadowBlur': propPath = 'shadow.blur'; break;
            case 'shadowX': propPath = 'shadow.x'; break;
            case 'shadowY': propPath = 'shadow.y'; break;
            case 'shadowColor': propPath = 'shadow.color'; break;
            case 'transformX': propPath = 'transform.x'; break;
            case 'transformY': propPath = 'transform.y'; break;
            case 'transformRotation': propPath = 'transform.rotation'; break;
            case 'transformScale': propPath = 'transform.scale'; break;
            case 'transformOpacity': propPath = 'transform.opacity'; break;
            case 'effect': propPath = 'effect'; break;
            case 'animation': propPath = 'animation'; break;
        }
        handleLinePropertyChange(propPath, value);
    }, [handleLinePropertyChange]);

    // Initialize syllables and charCustomizations when line changes (Data Load)
    useEffect(() => {
        const lineChanged = lineIndex !== prevLineIndexRef.current;
        prevLineIndexRef.current = lineIndex;

        // Only skip if it's an internal update AND we're on the same line
        // Always reload data when switching to a different line
        if (isInternalUpdate.current && !lineChanged) {
            isInternalUpdate.current = false;
            return;
        }
        isInternalUpdate.current = false;

        if (currentLyric?.syllables) {
            setSyllables([...currentLyric.syllables]);
        } else {
            setSyllables([]);
        }

        if (currentLyric?.charCustomizations) {
            setCharCustomizations({ ...currentLyric.charCustomizations });
        } else {
            setCharCustomizations({});
        }

        // Load line-level properties
        const props = {};
        if (currentLyric?.layout) props.layout = { ...currentLyric.layout };
        if (currentLyric?.transform) props.transform = { ...currentLyric.transform };
        if (currentLyric?.style) props.style = currentLyric.style;
        if (currentLyric?.font) props.font = { ...currentLyric.font };
        if (currentLyric?.stroke) props.stroke = { ...currentLyric.stroke };
        if (currentLyric?.shadow) props.shadow = { ...currentLyric.shadow };
        if (currentLyric?.effect) props.effect = currentLyric.effect;
        if (currentLyric?.animation) props.animation = currentLyric.animation;

        // Deep compare to avoid infinite update loops
        if (JSON.stringify(lineProperties) !== JSON.stringify(props)) {
            setLineProperties(props);
        }
    }, [lineIndex, currentLyric, lineProperties]);

    // Reset cursor/markers ONLY when traversing lines
    // eslint-disable-next-line react-hooks/exhaustive-deps
    useEffect(() => {
        // markingIndex / markStartTime are reset by store action setKTimingIndex
        setEditingChar(null);
    }, [lineIndex]);

    // Seek to line start when line changes
    // eslint-disable-next-line react-hooks/exhaustive-deps
    useEffect(() => {
        if (currentLyric && onSeek) {
            // Only seek if this wasn't an auto-advance AND wasn't a sync from timeline scrub
            // We check if the current time is completely outside the new line's range
            // If we are ALREADY inside the new line (e.g. from scrubbing), don't seek to start
            const isInsideLine = currentTime >= currentLyric.startTime && currentTime <= currentLyric.endTime;

            if (!isAutoAdvancing.current && !isInsideLine) {
                onSeek(currentLyric.startTime);
            }
            // Reset the flag
            isAutoAdvancing.current = false;
        }
    }, [lineIndex]);

    // Auto-pause at line end or loop back to start, or proceed to next line
    // eslint-disable-next-line react-hooks/exhaustive-deps
    useEffect(() => {
        if (!currentLyric || !isPlaying) return;

        if (currentTime >= currentLyric.endTime) {
            console.log('⏱️ [KTiming] Time check:', { currentTime, endTime: currentLyric.endTime, loopMode, lineIndex });
            if (loopMode) {
                // Loop mode: restart the current line
                console.log('🔄 [KTiming] Looping line', lineIndex);
                // Atomic Seek + Play (prevents pause glitch)
                onSeek?.(currentLyric.startTime, true);
            } else {
                // Non-loop mode: proceed to next line automatically
                if (lineIndex < lyrics.length - 1) {
                    console.log('➡️ [KTiming] Auto-advancing to next line');
                    // Save current state and move to next line
                    pushUpdates(syllables, charCustomizations, lineProperties);
                    isAutoAdvancing.current = true; // Flag as auto-advance
                    setKTimingIndex(lineIndex + 1);
                } else {
                    // Last line: just pause
                    console.log('🛑 [KTiming] End of lyrics, pausing');
                    onPause?.();
                }
            }
        }
    }, [currentTime, currentLyric, isPlaying, loopMode, onSeek, onPlay, onPause, lineIndex, lyrics.length, pushUpdates, syllables, charCustomizations, lineProperties]);

    // Sync current line to currentTime (Handle scrubbing)
    useEffect(() => {
        if (!currentLyric || !lyrics) return;

        // If we are significantly outside the current line range, find the correct line
        // Tolerance of 0.2s to prevent jitter during normal playback transitions
        if (currentTime < currentLyric.startTime - 0.2 || currentTime > currentLyric.endTime + 0.2) {
            const matchingIndex = lyrics.findIndex(l => currentTime >= l.startTime && currentTime < l.endTime);
            if (matchingIndex !== -1 && matchingIndex !== lineIndex) {
                // Prevent auto-advance loop by checking if we just auto-advanced
                if (!isAutoAdvancing.current) {
                    setKTimingIndex(matchingIndex);
                    // Note: We don't onSeek here because we are REACTING to time change
                }
            }
        }
    }, [currentTime, currentLyric, lyrics, lineIndex]);


    // Restart from line beginning
    const restartLine = useCallback(() => {
        if (!currentLyric || !onSeek) return;
        onSeek(currentLyric.startTime);
        resetLineTiming();
    }, [currentLyric, onSeek, resetLineTiming]);

    // Apply changes and move to next line
    const applyAndNext = useCallback(() => {
        if (!currentLyric) return;

        // Ensure current state is saved (it is by updateSyllablesAndSave, but good to confirm)
        pushUpdates(syllables, charCustomizations, lineProperties);

        // Move to next line
        if (lineIndex < lyrics.length - 1) {
            setKTimingIndex(lineIndex + 1);
        }
    }, [lyrics, lineIndex, currentLyric, syllables, charCustomizations, lineProperties, pushUpdates, setKTimingIndex]);

    // Handle character selection and seek to timestamp
    const handleCharSelect = useCallback((charIndex) => {
        if (charIndex === null) {
            setEditingChar(null);
            return;
        }

        const syl = syllables.find(s => charIndex >= s.charStart && charIndex < s.charStart + s.text.length);
        if (syl && currentLyric && onSeek) {
            const timestamp = currentLyric.startTime + syl.startOffset;
            onSeek(timestamp);
        }
        setEditingChar(charIndex);
    }, [syllables, currentLyric, onSeek]);




    // Keyboard handler
    useEffect(() => {
        const handleKeyDown = (e) => {
            if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;

            switch (e.key) {
                case ' ':
                case 'k':
                case 'K':
                    e.preventDefault();
                    markSyllable(currentTime);
                    break;
                case 'Backspace':
                    e.preventDefault();
                    undoMark();
                    break;
                case 'Enter':
                    e.preventDefault();
                    applyAndNext();
                    break;
                case 'Escape':
                    e.preventDefault();
                    onClose();
                    break;
                case 'a':
                case 'A':
                    e.preventDefault();
                    autoSplitTerm();
                    break;
                case 'r':
                case 'R':
                    e.preventDefault();
                    restartLine();
                    break;
                case 'l':
                case 'L':
                    e.preventDefault();
                    setKTimingLoop(!loopMode);
                    break;
                case 'p':
                case 'P':
                    e.preventDefault();
                    if (isPlaying) onPause?.(); else onPlay?.();
                    break;
                case 'ArrowLeft':
                    e.preventDefault();
                    if (onSeek && currentLyric) {
                        const delta = e.shiftKey ? -1 : -0.1;
                        onSeek(Math.max(currentLyric.startTime, currentTime + delta));
                    }
                    break;
                case 'ArrowRight':
                    e.preventDefault();
                    if (onSeek && currentLyric) {
                        const delta = e.shiftKey ? 1 : 0.1;
                        onSeek(Math.min(currentLyric.endTime, currentTime + delta));
                    }
                    break;
                case 'ArrowUp':
                    e.preventDefault();
                    if (lineIndex > 0) setKTimingIndex(lineIndex - 1);
                    break;
                case 'ArrowDown':
                    e.preventDefault();
                    if (lineIndex < lyrics.length - 1) setKTimingIndex(lineIndex + 1);
                    break;
                case 'z':
                case 'Z':
                    if (e.ctrlKey || e.metaKey) {
                        e.preventDefault();
                        if (e.shiftKey) redo?.(); else undo?.();
                    }
                    break;
                case 'y':
                case 'Y':
                    if (e.ctrlKey || e.metaKey) {
                        e.preventDefault();
                        redo?.();
                    }
                    break;
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [markSyllable, undoMark, applyAndNext, autoSplitTerm, restartLine, isPlaying, onPlay, onPause, onSeek, currentTime, currentLyric, lineIndex, lyrics.length, onClose, undo, redo]);

    useEffect(() => {
        containerRef.current?.focus();
    }, []);

    if (!currentLyric) {
        return (
            <div className="k-timing-editor">
                <div className="k-timing-header">
                    <h3>🎹 K-Timing Editor</h3>
                    <button onClick={onClose}>✕</button>
                </div>
                <div className="k-timing-empty">No lyrics to edit</div>
            </div>
        );
    }

    // --- Resizable Panel Logic ---
    const [sidePanelWidth, setSidePanelWidth] = useState(() => {
        const saved = localStorage.getItem('ui_ktiming_sidebar_width');
        return saved ? parseFloat(saved) : 300;
    });
    const isResizingRef = useRef(false);
    const currentWidthRef = useRef(sidePanelWidth); // Track for event listener

    // Keep ref in sync
    useEffect(() => {
        currentWidthRef.current = sidePanelWidth;
    }, [sidePanelWidth]);

    const handleMouseDownDivider = useCallback((e) => {
        isResizingRef.current = true;
        document.body.style.cursor = 'col-resize';
        document.body.style.userSelect = 'none'; // Prevent text selection
        e.preventDefault();
    }, []);

    useEffect(() => {
        const handleMouseMove = (e) => {
            if (!isResizingRef.current || !containerRef.current) return;

            // Calculate new width relative to container right edge
            const containerRect = containerRef.current.getBoundingClientRect();
            const newWidth = containerRect.right - e.clientX;

            // Clamp width (min 200px, max 50% of container)
            const maxWidth = containerRect.width / 2;
            setSidePanelWidth(Math.max(200, Math.min(maxWidth, newWidth)));
        };

        const handleMouseUp = () => {
            if (isResizingRef.current) {
                isResizingRef.current = false;
                document.body.style.cursor = '';
                document.body.style.userSelect = '';
                // Save to local storage on drag end
                localStorage.setItem('ui_ktiming_sidebar_width', currentWidthRef.current.toString());
            }
        };

        window.addEventListener('mousemove', handleMouseMove);
        window.addEventListener('mouseup', handleMouseUp);
        return () => {
            window.removeEventListener('mousemove', handleMouseMove);
            window.removeEventListener('mouseup', handleMouseUp);
        };
    }, []);

    return (
        <div className="k-timing-editor" ref={containerRef} tabIndex={0}>
            <div className="k-timing-header">
                <h3>🎹 K-Timing Editor</h3>
                <div className="header-info">
                    <select
                        className="line-selector"
                        value={lineIndex}
                        onChange={(e) => {
                            const newIndex = parseInt(e.target.value, 10);
                            setKTimingIndex(newIndex);
                            isAutoAdvancing.current = false; // Manual switch
                        }}
                    >
                        {lyrics.map((l, i) => (
                            <option key={i} value={i}>
                                #{i + 1}: {l.text.substring(0, 30)}{l.text.length > 30 ? '...' : ''}
                            </option>
                        ))}
                    </select>
                    <span className="total-lines">/ {lyrics.length}</span>
                    <span className="current-time">⏱️ {formatTime(currentTime)}</span>
                </div>
            </div>

            <div className="k-timing-body">
                {/* Main Column: Waveform, Timeline, Character Props */}
                <div className="k-timing-main-column" ref={mainColumnRef}>
                    {/* Waveform View - Aegisub-style PCM waveform */}
                    {hasRealAudio && (
                        <div className="waveform-container">
                            <WaveformView
                                lyric={currentLyric}
                                syllables={syllables}
                                currentTime={currentTime}
                                onSeek={onSeek}
                                getWaveformSlice={getWaveformSlice}
                                width={containerWidth}
                                height={80}
                            />
                        </div>
                    )}

                    {/* Draggable Timeline (Unified Editor) */}
                    <DraggableTimeline
                        key={lineIndex}
                        lyric={currentLyric}
                        syllables={syllables}
                        onSyllablesChange={handleSyllablesLocalUpdate}
                        currentTime={currentTime}
                        onSeek={onSeek}
                        width={containerWidth}
                        charCustomizations={charCustomizations}
                        editingChar={editingChar}
                        onEditChar={handleCharSelect}
                        onDragEnd={handleTimelineSave}
                    />

                    {/* Character Properties Panel (Only when selected) */}
                    {editingChar !== null && (
                        <CharacterPropertyPanel
                            charIndex={editingChar}
                            char={chars[editingChar]}
                            customization={charCustomizations?.[editingChar]}
                            resolution={resolution}
                            onCustomizationChange={handleCharCustomizationChange}
                            onClose={() => setEditingChar(null)}
                            availableFonts={availableFonts}
                        />
                    )}
                </div>

                {/* Draggable Divider */}
                <div
                    className="k-timing-divider"
                    onMouseDown={handleMouseDownDivider}
                    title="Drag to resize"
                />

                {/* Right Column: Line Properties Panel (Reflecting current line) */}
                <div
                    className="k-timing-side-column"
                    style={{ width: sidePanelWidth }}
                >
                    <div className="line-properties-panel">
                        <div className="panel-header">
                            <h4>📐 Line Properties</h4>
                        </div>
                        <StyleEditor
                            mode="line"
                            values={lineEditorValues}
                            onChange={handleStyleEditorChange}
                            availableFonts={availableFonts}
                        />
                    </div>
                </div>
            </div>
        </div>
    );
};

export default KTimingEditor;
