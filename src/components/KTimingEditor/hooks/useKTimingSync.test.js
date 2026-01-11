import { renderHook, act } from '@testing-library/react';
import { useKTimingSync } from './useKTimingSync';
import { vi } from 'vitest';

describe('useKTimingSync', () => {
    let lyrics;
    let onLyricsChange;

    beforeEach(() => {
        lyrics = [
            { text: 'Line 1', startTime: 0, endTime: 5, syllables: [] },
            { text: 'Line 2', startTime: 5, endTime: 10, syllables: [] }
        ];
        onLyricsChange = vi.fn();
    });

    it('should initialize state from lyrics', () => {
        const { result } = renderHook(() => useKTimingSync({
            lineIndex: 0,
            lyrics,
            onLyricsChange
        }));

        expect(result.current.syllables).toEqual([]);
        expect(result.current.charCustomizations).toEqual({});
        expect(result.current.lineProperties).toEqual({});
    });

    it('should push updates when charCustomizations change', () => {
        const { result } = renderHook(() => useKTimingSync({
            lineIndex: 0,
            lyrics,
            onLyricsChange
        }));

        act(() => {
            result.current.handleCharCustomizationChange(0, { color: 'red' });
        });

        expect(result.current.charCustomizations).toEqual({ 0: { color: 'red' } });
        expect(onLyricsChange).toHaveBeenCalledTimes(1);

        const updatedLyrics = onLyricsChange.mock.calls[0][0];
        expect(updatedLyrics[0].charCustomizations).toEqual({ 0: { color: 'red' } });
    });

    it('should push updates when lineProperties change', () => {
        const { result } = renderHook(() => useKTimingSync({
            lineIndex: 0,
            lyrics,
            onLyricsChange
        }));

        act(() => {
            result.current.handleLinePropertyChange('layout.mode', 'center');
        });

        expect(result.current.lineProperties).toEqual({ layout: { mode: 'center' } });

        expect(onLyricsChange).toHaveBeenCalled();
         const updatedLyrics = onLyricsChange.mock.calls[0][0];
         expect(updatedLyrics[0].layout.mode).toBe('center');
    });
});
