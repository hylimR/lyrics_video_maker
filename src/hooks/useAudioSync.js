import { useRef, useCallback, useEffect, useState } from 'react';

/**
 * useAudioSync.js - Audio Playback Synchronization Hook
 * 
 * Handles audio element management, playback sync, and demo mode switching.
 * Extracted from App.jsx to reduce complexity and improve reusability.
 * 
 * Features:
 * - Real audio and simulated audio support
 * - Master/Client sync logic
 * - Demo mode toggle
 * - Playback broadcasting
 */

/**
 * @param {Object} options - Hook configuration
 * @param {boolean} options.isMaster - Whether this tab is the master
 * @param {boolean} options.isPlaying - Current playback state
 * @param {number} options.currentTime - Current playback time
 * @param {number} options.duration - Total duration
 * @param {Function} options.setPlayback - Function to update playback state
 * @param {Function} options.updateState - Function to update store state
 * @param {Object} options.simulatedAudioRef - Ref for simulated audio
 * @param {number} options.simDuration - Simulated audio duration
 */
export function useAudioSync({
    isMaster,
    isPlaying,
    currentTime,
    setPlayback,
    simulatedAudioRef,
}) {
    const realAudioRef = useRef(null);
    const [demoMode, setDemoMode] = useState(true);
    const [audioSource, setAudioSource] = useState(null);

    // Active audio ref based on mode
    const audioRef = demoMode ? simulatedAudioRef : realAudioRef;

    // Sync audio element with store time (Client side)
    useEffect(() => {
        if (!isMaster && audioRef.current) {
            const diff = Math.abs(audioRef.current.currentTime - currentTime);
            if (diff > 0.3) {
                audioRef.current.currentTime = currentTime;
            }

            if (isPlaying && (audioRef.current.paused || audioRef.current.ended)) {
                audioRef.current.play().catch(e => console.warn('Autoplay blocked', e));
            } else if (!isPlaying && !audioRef.current.paused) {
                audioRef.current.pause();
            }
        }
    }, [currentTime, isPlaying, isMaster, audioRef]);

    // Broadcast high-frequency time (Master side)
    useEffect(() => {
        if (!isMaster || !isPlaying) return;

        const interval = setInterval(() => {
            if (audioRef.current) {
                setPlayback({
                    currentTime: audioRef.current.currentTime,
                    isPlaying: true
                });
            }
        }, 100); // 10fps sync

        return () => clearInterval(interval);
    }, [isMaster, isPlaying, audioRef, setPlayback]);

    // Playback handlers
    const handlePlay = useCallback(() => {
        if (isMaster && audioRef.current) {
            audioRef.current.play();
            setPlayback({ isPlaying: true, currentTime: audioRef.current.currentTime });
        } else if (!isMaster) {
            setPlayback({ isPlaying: true });
        }
    }, [isMaster, audioRef, setPlayback]);

    const handlePause = useCallback(() => {
        if (isMaster && audioRef.current) {
            audioRef.current.pause();
            setPlayback({ isPlaying: false, currentTime: audioRef.current.currentTime });
        } else if (!isMaster) {
            setPlayback({ isPlaying: false });
        }
    }, [isMaster, audioRef, setPlayback]);

    const handleSeek = useCallback((time) => {
        if (isMaster && audioRef.current) {
            audioRef.current.currentTime = time;
            setPlayback({ currentTime: time });
        } else if (!isMaster) {
            setPlayback({ currentTime: time });
        }
    }, [isMaster, audioRef, setPlayback]);

    // Audio file loading
    const loadAudioFile = useCallback((url, filename) => {
        setAudioSource(url);
        setDemoMode(false);
        console.log('🎵 Loaded audio:', filename);
    }, []);

    // Toggle demo mode
    const toggleDemoMode = useCallback(() => {
        setDemoMode(prev => !prev);
    }, []);

    return {
        realAudioRef,
        audioRef,
        demoMode,
        audioSource,
        setAudioSource,
        setDemoMode,
        handlePlay,
        handlePause,
        handleSeek,
        loadAudioFile,
        toggleDemoMode,
    };
}

export default useAudioSync;
