import { useState, useCallback, useEffect, useRef } from 'react';

const LONG_PRESS_DELAY = 200; // ms to trigger block drag

/**
 * Draggable Timeline - Interactive syllable boundary editing
 * Supports:
 * - Edge dragging (resize via handles)
 * - Long-press block dragging (move entire block)
 * - Character-level selection for customization
 */
const DraggableTimeline = ({
    lyric,
    syllables,
    onSyllablesChange,
    currentTime,
    width = 800,
    charCustomizations = {},
    editingChar = null,
    onEditChar = null,
    onDragEnd = null
}) => {
    const containerRef = useRef(null);
    const [dragging, setDragging] = useState(null); // { index, edge: 'start' | 'end' | 'block' }
    const [hoverHandle, setHoverHandle] = useState(null);

    // Long press state
    const longPressTimerRef = useRef(null);
    const pressStartRef = useRef(null); // { index, x, y }
    const [isBlockDragging, setIsBlockDragging] = useState(false);

    const duration = lyric ? lyric.endTime - lyric.startTime : 0;
    const timeToX = useCallback((t) => lyric ? ((t - lyric.startTime) / duration) * width : 0, [lyric, duration, width]);
    const xToTime = useCallback((x) => lyric ? lyric.startTime + (x / width) * duration : 0, [lyric, duration, width]);

    // Handle edge resize (start/end handles)
    const handleMouseDown = useCallback((e, index, edge) => {
        e.preventDefault();
        e.stopPropagation();
        setDragging({ index, edge });
    }, []);

    // Handle block long press start
    const handleBlockMouseDown = useCallback((e, index) => {
        // Ignore if clicking on drag handles
        if (e.target.classList.contains('drag-handle')) return;

        e.preventDefault();
        pressStartRef.current = { index, x: e.clientX, y: e.clientY };

        // Start long press timer
        longPressTimerRef.current = setTimeout(() => {
            // Long press detected - start block dragging
            setIsBlockDragging(true);
            setDragging({ index, edge: 'block' });
            pressStartRef.current = null;
        }, LONG_PRESS_DELAY);
    }, []);

    // Handle block mouse up (for click detection)
    const handleBlockMouseUp = useCallback((e, syl) => {
        // Clear long press timer
        if (longPressTimerRef.current) {
            clearTimeout(longPressTimerRef.current);
            longPressTimerRef.current = null;
        }

        // If we were block dragging, don't trigger click
        if (isBlockDragging) {
            return;
        }

        // If press started on this block and wasn't a long press, it's a click
        if (pressStartRef.current && !dragging) {
            let targetIndex = syl.charStart;

            // Checks if a specific character was clicked
            const charNode = e.target.closest('.bit-char');
            if (charNode && charNode.dataset.charIndex) {
                targetIndex = parseInt(charNode.dataset.charIndex, 10);
            }

            const isEditing = editingChar === targetIndex;
            onEditChar && onEditChar(isEditing ? null : targetIndex);
        }

        pressStartRef.current = null;
    }, [isBlockDragging, dragging, editingChar, onEditChar]);

    const handleMouseMove = useCallback((e) => {
        if (!dragging || !containerRef.current || !syllables) return;

        const rect = containerRef.current.getBoundingClientRect();
        const x = Math.max(0, Math.min(width, e.clientX - rect.left));
        const time = xToTime(x);

        const { index, edge } = dragging;
        const newSyllables = [...syllables];
        const syl = newSyllables[index];

        if (edge === 'start') {
            // Resize from start edge
            const minStart = index === 0 ? 0 : newSyllables[index - 1].startOffset + newSyllables[index - 1].duration;
            const maxStart = syl.startOffset + syl.duration - 0.05;
            const newStart = Math.max(minStart, Math.min(maxStart, time - lyric.startTime));
            const oldStart = syl.startOffset;
            syl.duration += (oldStart - newStart);
            syl.startOffset = newStart;
        } else if (edge === 'end') {
            // Resize from end edge
            const minEnd = syl.startOffset + 0.05;
            const maxEnd = index === syllables.length - 1
                ? duration
                : newSyllables[index + 1].startOffset;
            const newEnd = Math.max(minEnd, Math.min(maxEnd, time - lyric.startTime));
            syl.duration = newEnd - syl.startOffset;
        } else if (edge === 'block') {
            // Move entire block - keep duration fixed, just change startOffset
            const targetOffset = time - lyric.startTime - (syl.duration / 2); // Center block on cursor

            // Calculate boundaries (don't push adjacent blocks)
            const minOffset = index === 0
                ? 0
                : newSyllables[index - 1].startOffset + newSyllables[index - 1].duration;
            const maxOffset = index === syllables.length - 1
                ? duration - syl.duration
                : newSyllables[index + 1].startOffset - syl.duration;

            // Clamp to valid range
            const newOffset = Math.max(minOffset, Math.min(maxOffset, targetOffset));
            syl.startOffset = newOffset;
        }

        onSyllablesChange(newSyllables);
    }, [dragging, syllables, lyric, width, duration, onSyllablesChange, xToTime]);

    const handleMouseUp = useCallback(() => {
        // Clear long press timer
        if (longPressTimerRef.current) {
            clearTimeout(longPressTimerRef.current);
            longPressTimerRef.current = null;
        }

        if (dragging) {
            onDragEnd && onDragEnd();
        }
        setDragging(null);
        setIsBlockDragging(false);
        pressStartRef.current = null;
    }, [dragging, onDragEnd]);

    // Cleanup timer on unmount
    useEffect(() => {
        return () => {
            if (longPressTimerRef.current) {
                clearTimeout(longPressTimerRef.current);
            }
        };
    }, []);

    useEffect(() => {
        if (dragging) {
            window.addEventListener('mousemove', handleMouseMove);
            window.addEventListener('mouseup', handleMouseUp);
            return () => {
                window.removeEventListener('mousemove', handleMouseMove);
                window.removeEventListener('mouseup', handleMouseUp);
            };
        }
    }, [dragging, handleMouseMove, handleMouseUp]);

    if (!lyric || !syllables || syllables.length === 0) {
        return (
            <div className="draggable-timeline empty" style={{ width }}>
                <span>No timing data - Press [A] to auto-split</span>
            </div>
        );
    }

    return (
        <div
            ref={containerRef}
            className="draggable-timeline"
            style={{ width }}
        >
            {syllables.map((syl, i) => {
                const x = timeToX(lyric.startTime + syl.startOffset);
                const w = (syl.duration / duration) * width;
                const sylStart = lyric.startTime + syl.startOffset;
                const sylEnd = sylStart + syl.duration;
                const isActive = currentTime >= sylStart && currentTime < sylEnd;
                // Use unique key combining lyric identity + syllable charStart + text to force re-render on line change
                const syllableKey = `${lyric.startTime}-${syl.charStart}-${syl.text}`;

                return (
                    <div
                        key={syllableKey}
                        className={`syllable-block ${isActive ? 'active' : ''} ${isBlockDragging && dragging?.index === i ? 'dragging' : ''}`}
                        style={{ left: x, width: Math.max(20, w), cursor: isBlockDragging ? 'grabbing' : 'pointer' }}
                        onMouseDown={(e) => handleBlockMouseDown(e, i)}
                        onMouseUp={(e) => handleBlockMouseUp(e, syl)}
                    >
                        {/* Start handle */}
                        <div
                            className={`drag-handle start ${hoverHandle === `${i}-start` ? 'hover' : ''}`}
                            onMouseDown={(e) => handleMouseDown(e, i, 'start')}
                            onMouseEnter={() => setHoverHandle(`${i}-start`)}
                            onMouseLeave={() => setHoverHandle(null)}
                        />

                        {/* Syllable content - Breakdown by char */}
                        <div className="syllable-content">
                            {[...syl.text].map((char, charIdx) => {
                                const globalCharIndex = syl.charStart + charIdx;
                                const isCharEditing = editingChar === globalCharIndex;
                                const hasCustomization = charCustomizations?.[globalCharIndex];

                                return (
                                    <span
                                        key={charIdx}
                                        className={`bit-char ${isCharEditing ? 'editing' : ''} ${hasCustomization ? 'customized' : ''}`}
                                        data-char-index={globalCharIndex}
                                        title="Click to customize"
                                    >
                                        {char}
                                        {hasCustomization && <span className="bit-dot" />}
                                    </span>
                                );
                            })}
                            <span className="syllable-time">{(syl.duration * 1000).toFixed(0)}</span>
                        </div>

                        {/* End handle */}
                        <div
                            className={`drag-handle end ${hoverHandle === `${i}-end` ? 'hover' : ''}`}
                            onMouseDown={(e) => handleMouseDown(e, i, 'end')}
                            onMouseEnter={() => setHoverHandle(`${i}-end`)}
                            onMouseLeave={() => setHoverHandle(null)}
                        />
                    </div>
                );
            })}

            {/* Playhead - Visual Only */}
            <div
                className="timeline-playhead"
                style={{ left: timeToX(currentTime) }}
            >
                <div className="playhead-knob" />
            </div>
        </div>
    );
};

export default DraggableTimeline;
