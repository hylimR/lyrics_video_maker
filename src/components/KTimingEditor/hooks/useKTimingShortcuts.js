import { useEffect } from 'react';

/**
 * Hook to handle keyboard shortcuts for the KTiming Editor.
 *
 * @param {Object} props
 * @param {Function} props.onMark - Function to mark syllable (Space/K)
 * @param {Function} props.onUndoMark - Function to undo mark (Backspace)
 * @param {Function} props.onApplyAndNext - Function to confirm and move next (Enter)
 * @param {Function} props.onClose - Function to close editor (Escape)
 * @param {Function} props.onAutoSplit - Function to auto-split (A)
 * @param {Function} props.onRestartLine - Function to restart line (R)
 * @param {Function} props.onToggleLoop - Function to toggle loop (L)
 * @param {Function} props.onPlayPause - Function to toggle play/pause (P)
 * @param {Function} props.onSeek - Function to seek (Left/Right arrows)
 * @param {Function} props.onPrevLine - Function to go to previous line (Up arrow)
 * @param {Function} props.onNextLine - Function to go to next line (Down arrow)
 * @param {Function} props.onUndo - Global undo (Ctrl+Z)
 * @param {Function} props.onRedo - Global redo (Ctrl+Y / Ctrl+Shift+Z)
 * @param {number} props.currentTime - Current playback time
 * @param {Object} props.currentLyric - Current lyric object
 * @param {boolean} props.isPlaying - Whether audio is playing
 */
export const useKTimingShortcuts = ({
    onMark,
    onUndoMark,
    onApplyAndNext,
    onClose,
    onAutoSplit,
    onRestartLine,
    onToggleLoop,
    onPlayPause,
    onSeek,
    onPrevLine,
    onNextLine,
    onUndo,
    onRedo,
    currentTime,
    currentLyric,
    isPlaying
}) => {
    useEffect(() => {
        const handleKeyDown = (e) => {
            // Ignore if typing in input fields
            if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;

            switch (e.key) {
                case ' ':
                case 'k':
                case 'K':
                    e.preventDefault();
                    onMark(currentTime);
                    break;
                case 'Backspace':
                    e.preventDefault();
                    onUndoMark();
                    break;
                case 'Enter':
                    e.preventDefault();
                    onApplyAndNext();
                    break;
                case 'Escape':
                    e.preventDefault();
                    onClose();
                    break;
                case 'a':
                case 'A':
                    e.preventDefault();
                    onAutoSplit();
                    break;
                case 'r':
                case 'R':
                    e.preventDefault();
                    onRestartLine();
                    break;
                case 'l':
                case 'L':
                    e.preventDefault();
                    onToggleLoop();
                    break;
                case 'p':
                case 'P':
                    e.preventDefault();
                    onPlayPause();
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
                    onPrevLine();
                    break;
                case 'ArrowDown':
                    e.preventDefault();
                    onNextLine();
                    break;
                case 'z':
                case 'Z':
                    if (e.ctrlKey || e.metaKey) {
                        e.preventDefault();
                        if (e.shiftKey) onRedo?.(); else onUndo?.();
                    }
                    break;
                case 'y':
                case 'Y':
                    if (e.ctrlKey || e.metaKey) {
                        e.preventDefault();
                        onRedo?.();
                    }
                    break;
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [
        onMark,
        onUndoMark,
        onApplyAndNext,
        onClose,
        onAutoSplit,
        onRestartLine,
        onToggleLoop,
        onPlayPause,
        onSeek,
        onPrevLine,
        onNextLine,
        onUndo,
        onRedo,
        currentTime,
        currentLyric,
        isPlaying
    ]);
};
