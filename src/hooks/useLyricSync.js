import { useCallback, useMemo } from 'react';

/**
 * useLyricSync.js - Sync Logic Hook
 * 
 * This hook provides the core synchronization logic for mapping
 * audio playback time to lyric display state.
 * 
 * Key Features:
 * - Efficiently finds the active lyric line based on current time
 * - Calculates mask progress for the karaoke fill effect
 * - Memoized callbacks for performance in the render loop
 * 
 * Usage:
 * const { getActiveLineIndex, calculateMaskProgress } = useLyricSync(lyrics);
 * 
 * Inside Ticker:
 * const activeIndex = getActiveLineIndex(currentTime);
 * const progress = calculateMaskProgress(currentTime, lyrics[activeIndex]);
 */

/**
 * Custom hook for lyric synchronization
 * @param {Array} lyrics - Array of lyric objects with startTime, endTime, text
 */
export function useLyricSync(lyrics) {
    /**
     * Build a sorted timeline for binary search
     * This is more efficient than linear search for large lyric sets
     */
    const sortedLyrics = useMemo(() => {
        return [...lyrics].map((lyric, index) => ({
            ...lyric,
            originalIndex: index,
        })).sort((a, b) => a.startTime - b.startTime);
    }, [lyrics]);

    /**
     * Find the active line index using binary search
     * Returns -1 if no line is active (between lyrics or before first)
     */
    const getActiveLineIndex = useCallback((currentTime) => {
        // Edge cases
        if (sortedLyrics.length === 0) return -1;
        if (currentTime < sortedLyrics[0].startTime) return -1;
        if (currentTime >= sortedLyrics[sortedLyrics.length - 1].endTime) return -1;

        // Binary search for the active line
        let left = 0;
        let right = sortedLyrics.length - 1;

        while (left <= right) {
            const mid = Math.floor((left + right) / 2);
            const lyric = sortedLyrics[mid];

            if (currentTime >= lyric.startTime && currentTime < lyric.endTime) {
                // Found active line
                return lyric.originalIndex;
            } else if (currentTime < lyric.startTime) {
                right = mid - 1;
            } else {
                left = mid + 1;
            }
        }

        // No active line found (in a gap between lyrics)
        return -1;
    }, [sortedLyrics]);

    /**
     * Calculate the mask progress (0 to 1) for a given lyric
     * This drives the karaoke fill effect
     */
    const calculateMaskProgress = useCallback((currentTime, lyricData) => {
        if (!lyricData) return 0;

        const { startTime, endTime } = lyricData;
        const duration = endTime - startTime;

        if (duration <= 0) return 0;
        if (currentTime < startTime) return 0;
        if (currentTime >= endTime) return 1;

        return (currentTime - startTime) / duration;
    }, []);

    /**
     * Get upcoming lyrics (for preview/preparation)
     * Useful for pre-loading effects or showing next line preview
     */
    const getUpcomingLines = useCallback((currentTime, count = 3) => {
        return sortedLyrics
            .filter(lyric => lyric.startTime > currentTime)
            .slice(0, count)
            .map(lyric => lyric.originalIndex);
    }, [sortedLyrics]);

    /**
     * Get time until next line starts
     * Useful for scheduling entrance animations
     */
    const getTimeUntilNextLine = useCallback((currentTime) => {
        const nextLyric = sortedLyrics.find(lyric => lyric.startTime > currentTime);
        return nextLyric ? nextLyric.startTime - currentTime : Infinity;
    }, [sortedLyrics]);

    /**
     * Check if we're in a gap between lyrics
     */
    const isInGap = useCallback((currentTime) => {
        const activeIndex = getActiveLineIndex(currentTime);
        return activeIndex === -1;
    }, [getActiveLineIndex]);

    return {
        getActiveLineIndex,
        calculateMaskProgress,
        getUpcomingLines,
        getTimeUntilNextLine,
        isInGap,
    };
}

export default useLyricSync;
