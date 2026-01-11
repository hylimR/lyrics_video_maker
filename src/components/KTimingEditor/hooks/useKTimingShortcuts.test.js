import { renderHook } from '@testing-library/react';
import { useKTimingShortcuts } from './useKTimingShortcuts';
import { vi } from 'vitest';

describe('useKTimingShortcuts', () => {
    let props;

    beforeEach(() => {
        props = {
            onMark: vi.fn(),
            onUndoMark: vi.fn(),
            onApplyAndNext: vi.fn(),
            onClose: vi.fn(),
            onAutoSplit: vi.fn(),
            onRestartLine: vi.fn(),
            onToggleLoop: vi.fn(),
            onPlayPause: vi.fn(),
            onSeek: vi.fn(),
            onPrevLine: vi.fn(),
            onNextLine: vi.fn(),
            onUndo: vi.fn(),
            onRedo: vi.fn(),
            currentTime: 10,
            currentLyric: { startTime: 5, endTime: 15 },
            isPlaying: true
        };
    });

    const triggerKey = (key, modifiers = {}) => {
        const event = new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true, ...modifiers });
        window.dispatchEvent(event);
        return event;
    };

    it('should call onMark when Space or K is pressed', () => {
        renderHook(() => useKTimingShortcuts(props));
        triggerKey(' ');
        expect(props.onMark).toHaveBeenCalledWith(10);

        triggerKey('k');
        expect(props.onMark).toHaveBeenCalledTimes(2);
    });

    it('should call onUndoMark when Backspace is pressed', () => {
        renderHook(() => useKTimingShortcuts(props));
        triggerKey('Backspace');
        expect(props.onUndoMark).toHaveBeenCalled();
    });

    it('should call onSeek with correct delta on Arrow keys', () => {
        renderHook(() => useKTimingShortcuts(props));

        // Left arrow (-0.1s)
        triggerKey('ArrowLeft');
        expect(props.onSeek).toHaveBeenCalledWith(9.9);

        // Right arrow (+0.1s)
        triggerKey('ArrowRight');
        expect(props.onSeek).toHaveBeenCalledWith(10.1);

        // Shift + Left arrow (-1s)
        triggerKey('ArrowLeft', { shiftKey: true });
        expect(props.onSeek).toHaveBeenCalledWith(9);
    });

    it('should not trigger if input is focused', () => {
        renderHook(() => useKTimingShortcuts(props));

        const input = document.createElement('input');
        document.body.appendChild(input);
        input.focus();

        const event = new KeyboardEvent('keydown', { key: ' ', bubbles: true });
        Object.defineProperty(event, 'target', { value: input }); // Mock target
        window.dispatchEvent(event);

        expect(props.onMark).not.toHaveBeenCalled();
        document.body.removeChild(input);
    });
});
