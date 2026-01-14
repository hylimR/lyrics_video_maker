import { useState, useCallback, useRef, useEffect } from 'react';
import './LyricsEditor.css';
import LyricsList from './LyricsList';
import { formatTime } from '@/utils/timeUtils';

/**
 * LyricsEditor.jsx - In-App Lyrics Editor
 * 
 * Features:
 * - Edit lyric text
 * - Adjust start/end times with precision
 * - Add/remove lyric lines
 * - Sync timing with current playback position
 * - Reorder lyrics
 * - Export as LRC format
 */
const LyricsEditor = ({
    lyrics,
    currentTime,
    onLyricsChange,
    onClose,
    availableFonts
}) => {
    const [localLyrics, setLocalLyrics] = useState([...lyrics]);
    const [hasChanges, setHasChanges] = useState(false);

    // Use a ref for currentTime to avoid passing it as a prop to children
    // causing unnecessary re-renders of all lines every frame.
    const currentTimeRef = useRef(currentTime);
    useEffect(() => {
        currentTimeRef.current = currentTime;
    }, [currentTime]);

    // Find active line index
    const activeIndex = localLyrics.findIndex(l =>
        currentTime >= l.startTime && currentTime < l.endTime
    );

    const handleUpdate = useCallback((index, updates) => {
        setLocalLyrics(prev => {
            const newLyrics = [...prev];
            newLyrics[index] = { ...newLyrics[index], ...updates };
            return newLyrics;
        });
        setHasChanges(true);
    }, []);

    const handleDelete = useCallback((index) => {
        if (window.confirm('Delete this lyric line?')) {
            setLocalLyrics(prev => prev.filter((_, i) => i !== index));
            setHasChanges(true);
        }
    }, []);

    const handleAdd = useCallback(() => {
        const lastLine = localLyrics[localLyrics.length - 1];
        const newStartTime = lastLine ? lastLine.endTime + 0.5 : currentTime;

        setLocalLyrics(prev => [...prev, {
            text: 'New lyric line',
            startTime: newStartTime,
            endTime: newStartTime + 3,
        }]);
        setHasChanges(true);
    }, [localLyrics, currentTime]);

    const handleAddAtTime = useCallback(() => {
        setLocalLyrics(prev => {
            const newLine = {
                text: 'New lyric',
                startTime: currentTime,
                endTime: currentTime + 3,
            };

            // Insert in correct position
            const insertIndex = prev.findIndex(l => l.startTime > currentTime);
            if (insertIndex === -1) {
                return [...prev, newLine];
            }
            const newLyrics = [...prev];
            newLyrics.splice(insertIndex, 0, newLine);
            return newLyrics;
        });
        setHasChanges(true);
    }, [currentTime]);

    const handleSetStartTime = useCallback((index, time) => {
        handleUpdate(index, { startTime: time });
    }, [handleUpdate]);

    const handleSetEndTime = useCallback((index, time) => {
        handleUpdate(index, { endTime: time });
    }, [handleUpdate]);

    const handleMoveUp = useCallback((index) => {
        if (index === 0) return;
        setLocalLyrics(prev => {
            const newLyrics = [...prev];
            [newLyrics[index - 1], newLyrics[index]] = [newLyrics[index], newLyrics[index - 1]];
            return newLyrics;
        });
        setHasChanges(true);
    }, []);

    const handleMoveDown = useCallback((index) => {
        setLocalLyrics(prev => {
            if (index >= prev.length - 1) return prev;
            const newLyrics = [...prev];
            [newLyrics[index], newLyrics[index + 1]] = [newLyrics[index + 1], newLyrics[index]];
            return newLyrics;
        });
        setHasChanges(true);
    }, []);

    const handleSortByTime = useCallback(() => {
        setLocalLyrics(prev => [...prev].sort((a, b) => a.startTime - b.startTime));
        setHasChanges(true);
    }, []);

    const handleApply = useCallback(() => {
        onLyricsChange(localLyrics);
        setHasChanges(false);
    }, [localLyrics, onLyricsChange]);

    const handleExportLRC = useCallback(() => {
        const lrcContent = localLyrics.map(l => {
            const mins = Math.floor(l.startTime / 60);
            const secs = (l.startTime % 60).toFixed(2);
            return `[${mins.toString().padStart(2, '0')}:${secs.padStart(5, '0')}]${l.text}`;
        }).join('\n');

        const blob = new Blob([lrcContent], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'lyrics.lrc';
        a.click();
        URL.revokeObjectURL(url);
    }, [localLyrics]);

    return (
        <div className="lyrics-editor">
            <div className="editor-header">
                <h3>📝 Lyrics Editor</h3>
                <div className="header-actions">
                    <span className="current-time">⏱️ {formatTime(currentTime)}</span>
                    <button className="close-btn" onClick={onClose}>✕</button>
                </div>
            </div>

            <div className="editor-toolbar">
                <button className="toolbar-btn" onClick={handleAdd}>
                    ➕ Add Line
                </button>
                <button className="toolbar-btn" onClick={handleAddAtTime}>
                    ⏱️ Add at Time
                </button>
                <button className="toolbar-btn" onClick={handleSortByTime}>
                    📊 Sort by Time
                </button>
                <button className="toolbar-btn export" onClick={handleExportLRC}>
                    💾 Export LRC
                </button>
            </div>

            <LyricsList
                localLyrics={localLyrics}
                activeIndex={activeIndex}
                currentTimeRef={currentTimeRef}
                availableFonts={availableFonts}
                onUpdate={handleUpdate}
                onDelete={handleDelete}
                onSetStartTime={handleSetStartTime}
                onSetEndTime={handleSetEndTime}
                onMoveUp={handleMoveUp}
                onMoveDown={handleMoveDown}
            />

            <div className="editor-footer">
                <div className="footer-info">
                    {localLyrics.length} lines
                    {hasChanges && <span className="unsaved">• Unsaved changes</span>}
                </div>
                <div className="footer-actions">
                    <button
                        className="apply-btn"
                        onClick={handleApply}
                        disabled={!hasChanges}
                    >
                        ✓ Apply Changes
                    </button>
                </div>
            </div>
        </div>
    );
};

export default LyricsEditor;
