import { useState, useCallback, useEffect, useRef, useMemo } from 'react';
import './KTimingEditor.css';
import WaveformView from './WaveformView';
import DraggableTimeline from './DraggableTimeline';
import CharacterPropertyPanel from './CharacterPropertyPanel';
import StyleEditor from '@/components/StyleEditor';
import { useAppStore } from '@/store/useAppStore';
import useWaveformData from '@/hooks/useWaveformData';

// Hooks
import { useResizablePanel } from './hooks/useResizablePanel';
import { useKTimingShortcuts } from './hooks/useKTimingShortcuts';
import { useKTimingSync } from './hooks/useKTimingSync';

// Components
import { KTimingHeader } from './KTimingHeader';

/**
 * KTimingEditor.jsx - Aegisub-style K-Timing Editor
 * Refactored to use Hooks for Logic and Components for UI
 */
const KTimingEditor = ({
    lyrics,
    currentTime,
    isPlaying,
    audioSource = null,
    hasRealAudio = false,
    resolution = { width: 1920, height: 1080 },
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

    const { lineIndex, loopMode } = ktiming;

    // --- Hooks ---

    // Waveform Data
    const { getWaveformSlice } = useWaveformData(
        hasRealAudio ? audioSource : null,
        200
    );

    // Sync Logic
    const {
        syllables,
        setSyllables,
        charCustomizations,
        setCharCustomizations,
        lineProperties,
        pushUpdates,
        handleCharCustomizationChange,
        handleLinePropertyChange,
    } = useKTimingSync({
        lineIndex,
        lyrics,
        onLyricsChange
    });

    // Resizable Panel
    const { width: sidePanelWidth, containerRef, handleMouseDown: handleMouseDownDivider } = useResizablePanel();

    // Local UI State
    const [editingChar, setEditingChar] = useState(null);
    const [containerWidth, setContainerWidth] = useState(800);
    const mainColumnRef = useRef(null);
    const isAutoAdvancing = useRef(false);

    const currentLyric = lyrics[lineIndex];
    const chars = useMemo(() => currentLyric ? [...currentLyric.text] : [], [currentLyric]);

    // --- Helper Functions ---

    const handleSyllablesLocalUpdate = useCallback((newSyls) => {
        setSyllables(newSyls);
    }, [setSyllables]);

    const handleTimelineSave = useCallback(() => {
        pushUpdates(syllables, charCustomizations);
    }, [pushUpdates, syllables, charCustomizations]);

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
        const map = {
            'layoutMode': 'layout.mode',
            'layoutAlign': 'layout.align',
            'layoutGap': 'layout.gap',
            'fontFamily': 'font.family',
            'fontSize': 'font.size',
            'strokeWidth': 'stroke.width',
            'strokeColor': 'stroke.color',
            'shadowBlur': 'shadow.blur',
            'shadowX': 'shadow.x',
            'shadowY': 'shadow.y',
            'shadowColor': 'shadow.color',
            'transformX': 'transform.x',
            'transformY': 'transform.y',
            'transformRotation': 'transform.rotation',
            'transformScale': 'transform.scale',
            'transformOpacity': 'transform.opacity',
            'effect': 'effect',
            'animation': 'animation',
        };
        if (map[key]) propPath = map[key];

        handleLinePropertyChange(propPath, value);
    }, [handleLinePropertyChange]);

    // --- Navigation Logic ---
    // (This could also be extracted to useKTimingNavigation, but keeping it here for now as it bridges UI and Logic)

    // Restart from line beginning
    const restartLine = useCallback(() => {
        if (!currentLyric || !onSeek) return;
        onSeek(currentLyric.startTime);
        resetLineTiming();
    }, [currentLyric, onSeek, resetLineTiming]);

    // Apply changes and move to next line
    const applyAndNext = useCallback(() => {
        if (!currentLyric) return;
        pushUpdates(syllables, charCustomizations, lineProperties);
        if (lineIndex < lyrics.length - 1) {
            setKTimingIndex(lineIndex + 1);
        }
    }, [lyrics, lineIndex, currentLyric, syllables, charCustomizations, lineProperties, pushUpdates, setKTimingIndex]);

    const handleLineChange = useCallback((newIndex) => {
        setKTimingIndex(newIndex);
        isAutoAdvancing.current = false;
    }, [setKTimingIndex]);


    // --- Side Effects ---

    // Handle Resize (Observe Main Column)
    useEffect(() => {
        if (!mainColumnRef.current) return;
        const observer = new ResizeObserver(entries => {
            for (const entry of entries) {
                const width = Math.max(100, entry.contentRect.width);
                setContainerWidth(width);
            }
        });
        observer.observe(mainColumnRef.current);
        return () => observer.disconnect();
    }, []);

    // Reset cursor/markers when line changes
    useEffect(() => {
        setEditingChar(null);
    }, [lineIndex]);

    // Seek to line start when line changes
    useEffect(() => {
        if (currentLyric && onSeek) {
            const isInsideLine = currentTime >= currentLyric.startTime && currentTime <= currentLyric.endTime;
            if (!isAutoAdvancing.current && !isInsideLine) {
                onSeek(currentLyric.startTime);
            }
            isAutoAdvancing.current = false;
        }
    }, [lineIndex, onSeek]); // Only run on explicit line change

    // Auto-pause / Loop / Auto-advance logic
    useEffect(() => {
        if (!currentLyric || !isPlaying) return;

        if (currentTime >= currentLyric.endTime) {
            if (loopMode) {
                onSeek?.(currentLyric.startTime, true);
            } else {
                if (lineIndex < lyrics.length - 1) {
                    pushUpdates(syllables, charCustomizations, lineProperties);
                    isAutoAdvancing.current = true;
                    setKTimingIndex(lineIndex + 1);
                } else {
                    onPause?.();
                }
            }
        }
    }, [currentTime, currentLyric, isPlaying, loopMode, onSeek, onPause, lineIndex, lyrics.length, pushUpdates, syllables, charCustomizations, lineProperties, setKTimingIndex]);

    // Sync current line to currentTime (Scrubbing support)
    useEffect(() => {
        if (!currentLyric || !lyrics) return;
        if (currentTime < currentLyric.startTime - 0.2 || currentTime > currentLyric.endTime + 0.2) {
            const matchingIndex = lyrics.findIndex(l => currentTime >= l.startTime && currentTime < l.endTime);
            if (matchingIndex !== -1 && matchingIndex !== lineIndex) {
                if (!isAutoAdvancing.current) {
                    setKTimingIndex(matchingIndex);
                }
            }
        }
    }, [currentTime, currentLyric, lyrics, lineIndex, setKTimingIndex]);

    useEffect(() => {
        containerRef.current?.focus();
    }, [containerRef]);


    // --- Shortcuts ---
    useKTimingShortcuts({
        onMark: markSyllable,
        onUndoMark: undoMark,
        onApplyAndNext: applyAndNext,
        onClose,
        onAutoSplit: autoSplitTerm,
        onRestartLine: restartLine,
        onToggleLoop: () => setKTimingLoop(!loopMode),
        onPlayPause: () => isPlaying ? onPause?.() : onPlay?.(),
        onSeek,
        onPrevLine: () => lineIndex > 0 && setKTimingIndex(lineIndex - 1),
        onNextLine: () => lineIndex < lyrics.length - 1 && setKTimingIndex(lineIndex + 1),
        onUndo,
        onRedo,
        currentTime,
        currentLyric,
        isPlaying
    });


    // --- Render ---

    if (!currentLyric) {
        return (
            <div className="k-timing-editor">
                <KTimingHeader
                    lyrics={lyrics || []}
                    lineIndex={lineIndex}
                    currentTime={currentTime}
                    onLineChange={handleLineChange}
                    onClose={onClose}
                />
                <div className="k-timing-empty">No lyrics to edit</div>
            </div>
        );
    }

    return (
        <div className="k-timing-editor" ref={containerRef} tabIndex={0}>
            <KTimingHeader
                lyrics={lyrics}
                lineIndex={lineIndex}
                currentTime={currentTime}
                onLineChange={handleLineChange}
                onClose={onClose}
            />

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
