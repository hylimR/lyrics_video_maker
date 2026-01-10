import { useRef, useCallback, useState } from 'react';

/**
 * useSimulatedAudio.js - Simulated Audio Playback Hook
 * 
 * This hook provides a simulated audio controller that mimics
 * the behavior of an HTMLAudioElement. Useful for testing the
 * lyric sync system without needing an actual audio file.
 * 
 * Features:
 * - Simulates currentTime progression using requestAnimationFrame
 * - Supports play, pause, and seek operations
 * - Compatible with the same interface as audioRef.current
 */

export function useSimulatedAudio(initialDuration = 30) {
    const [duration, setDurationState] = useState(initialDuration);

    const simulatedAudioRef = useRef({
        currentTime: 0,
        duration: initialDuration,
        paused: true,
        _animationId: null,
        _lastTimestamp: null,

        play() {
            if (!this.paused) return Promise.resolve();
            this.paused = false;
            this._lastTimestamp = performance.now();
            this._tick();
            return Promise.resolve();
        },

        pause() {
            this.paused = true;
            if (this._animationId) {
                cancelAnimationFrame(this._animationId);
                this._animationId = null;
            }
        },

        _tick() {
            if (this.paused) return;

            const now = performance.now();
            const delta = (now - this._lastTimestamp) / 1000;
            this._lastTimestamp = now;

            this.currentTime += delta;

            if (this.currentTime >= this.duration) {
                this.currentTime = 0; // Loop
            }

            this._animationId = requestAnimationFrame(() => this._tick());
        }
    });

    const play = useCallback(() => {
        simulatedAudioRef.current.play();
    }, []);

    const pause = useCallback(() => {
        simulatedAudioRef.current.pause();
    }, []);

    const seek = useCallback((time) => {
        simulatedAudioRef.current.currentTime = Math.max(0, Math.min(time, simulatedAudioRef.current.duration));
    }, []);

    const getCurrentTime = useCallback(() => {
        return simulatedAudioRef.current.currentTime;
    }, []);

    const isPaused = useCallback(() => {
        return simulatedAudioRef.current.paused;
    }, []);

    const setDuration = useCallback((newDuration) => {
        simulatedAudioRef.current.duration = newDuration;
        setDurationState(newDuration);
    }, []);

    return {
        audioRef: simulatedAudioRef,
        play,
        pause,
        seek,
        getCurrentTime,
        isPaused,
        duration,
        setDuration,
    };
}

export default useSimulatedAudio;

