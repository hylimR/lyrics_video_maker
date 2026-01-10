import { useRef, useEffect, useCallback, useState } from 'react';

/**
 * useAudioAnalyzer - Web Audio API Analyzer Hook
 * 
 * Connects to an audio element and provides real-time frequency/waveform data
 * for visualization purposes.
 * 
 * Features:
 * - FFT-based frequency analysis
 * - Time-domain waveform data
 * - Automatic audio context management
 * - Proper cleanup and resource management
 * 
 * @param {Object} options - Configuration options
 * @param {React.RefObject} options.audioRef - Reference to the audio element
 * @param {number} options.fftSize - FFT size (power of 2, default 2048)
 * @param {number} options.smoothingTimeConstant - Smoothing (0-1, default 0.8)
 */
export function useAudioAnalyzer({ audioRef, fftSize = 2048, smoothingTimeConstant = 0.8 }) {
    const audioContextRef = useRef(null);
    const analyserRef = useRef(null);
    const sourceNodeRef = useRef(null);
    const [isConnected, setIsConnected] = useState(false);

    const isConnectedRef = useRef(false);
    useEffect(() => {
        isConnectedRef.current = isConnected;
    }, [isConnected]);

    // Initialize audio context and analyser
    const connect = useCallback(() => {
        const audio = audioRef?.current;
        if (!audio) {
            console.warn('🔊 AudioAnalyzer: No audio element found');
            return false;
        }

        // Don't reconnect if already connected
        if (sourceNodeRef.current && isConnectedRef.current) {
            return true;
        }

        try {
            // Create or reuse audio context
            if (!audioContextRef.current) {
                audioContextRef.current = new (window.AudioContext || window.webkitAudioContext)();
            }

            const ctx = audioContextRef.current;

            // Resume if suspended (required by browsers after user interaction)
            if (ctx.state === 'suspended') {
                ctx.resume();
            }

            // Create analyser node
            if (!analyserRef.current) {
                analyserRef.current = ctx.createAnalyser();
                analyserRef.current.fftSize = fftSize;
                analyserRef.current.smoothingTimeConstant = smoothingTimeConstant;
            }

            // Create source from audio element (only once per audio element)
            if (!sourceNodeRef.current) {
                // Verify it's a real DOM element before creating source
                if (audio instanceof Element) {
                    sourceNodeRef.current = ctx.createMediaElementSource(audio);
                    sourceNodeRef.current.connect(analyserRef.current);
                    analyserRef.current.connect(ctx.destination);
                } else {
                    console.warn('🔊 AudioAnalyzer: Audio ref is not a DOM element, skipping analysis');
                    return false;
                }
            }

            setIsConnected(true);
            console.log('🔊 AudioAnalyzer connected successfully');
            return true;
        } catch (error) {
            console.error('🔊 AudioAnalyzer connection failed:', error);
            return false;
        }
    }, [audioRef, fftSize, smoothingTimeConstant]);

    // Get frequency data (for spectrum visualization)
    const getFrequencyData = useCallback(() => {
        if (!analyserRef.current) return null;

        const bufferLength = analyserRef.current.frequencyBinCount;
        const dataArray = new Uint8Array(bufferLength);
        analyserRef.current.getByteFrequencyData(dataArray);
        return dataArray;
    }, []);

    // Get normalized frequency data (0-1 range)
    const getNormalizedFrequencyData = useCallback(() => {
        const data = getFrequencyData();
        if (!data) return null;

        const normalized = new Float32Array(data.length);
        for (let i = 0; i < data.length; i++) {
            normalized[i] = data[i] / 255;
        }
        return normalized;
    }, [getFrequencyData]);

    // Get time domain data (for waveform visualization)
    const getTimeDomainData = useCallback(() => {
        if (!analyserRef.current) return null;

        const bufferLength = analyserRef.current.frequencyBinCount;
        const dataArray = new Uint8Array(bufferLength);
        analyserRef.current.getByteTimeDomainData(dataArray);
        return dataArray;
    }, []);

    // Get normalized time domain data (centered around 0)
    const getNormalizedTimeDomainData = useCallback(() => {
        const data = getTimeDomainData();
        if (!data) return null;

        const normalized = new Float32Array(data.length);
        for (let i = 0; i < data.length; i++) {
            // Convert 0-255 to -1 to 1
            normalized[i] = (data[i] - 128) / 128;
        }
        return normalized;
    }, [getTimeDomainData]);

    // Get average volume level (0-1)
    const getAverageVolume = useCallback(() => {
        const data = getFrequencyData();
        if (!data || data.length === 0) return 0;

        const sum = data.reduce((acc, val) => acc + val, 0);
        return sum / (data.length * 255);
    }, [getFrequencyData]);

    // Get frequency bands (bass, mid, treble)
    const getFrequencyBands = useCallback(() => {
        const data = getFrequencyData();
        if (!data || data.length === 0) {
            return { bass: 0, mid: 0, treble: 0, overall: 0 };
        }

        const len = data.length;
        const bassEnd = Math.floor(len * 0.1);      // 0-10% = bass
        const midEnd = Math.floor(len * 0.5);       // 10-50% = mid
        // 50-100% = treble

        let bassSum = 0, midSum = 0, trebleSum = 0;

        for (let i = 0; i < len; i++) {
            if (i < bassEnd) {
                bassSum += data[i];
            } else if (i < midEnd) {
                midSum += data[i];
            } else {
                trebleSum += data[i];
            }
        }

        const bass = bassSum / (bassEnd * 255);
        const mid = midSum / ((midEnd - bassEnd) * 255);
        const treble = trebleSum / ((len - midEnd) * 255);
        const overall = (bass + mid + treble) / 3;

        return { bass, mid, treble, overall };
    }, [getFrequencyData]);

    // Disconnect and cleanup
    const disconnect = useCallback(() => {
        if (sourceNodeRef.current) {
            try {
                sourceNodeRef.current.disconnect();
            } catch {
                // Ignore disconnection errors
            }
            sourceNodeRef.current = null;
        }
        if (analyserRef.current) {
            try {
                analyserRef.current.disconnect();
            } catch {
                // Ignore disconnection errors
            }
            analyserRef.current = null;
        }
        setIsConnected(false);
    }, []);

    // Auto-connect when audio element changes
    useEffect(() => {
        const audio = audioRef?.current;
        if (!audio) return;

        // Check if audio element supports event listeners
        const hasEvents = typeof audio.addEventListener === 'function';

        // Connect on play event
        const handlePlay = () => {
            connect();
        };

        if (hasEvents) {
            audio.addEventListener('play', handlePlay);
        }

        // If audio is already playing, connect
        if (!audio.paused) {
            setTimeout(() => connect(), 0);
        }

        return () => {
            if (hasEvents) {
                audio.removeEventListener('play', handlePlay);
            }
            // Disconnect when switching audio sources
            disconnect();
        };
    }, [audioRef, connect, disconnect]);

    // Cleanup on unmount
    useEffect(() => {
        return () => {
            disconnect();
            if (audioContextRef.current) {
                audioContextRef.current.close().catch(() => { });
                audioContextRef.current = null;
            }
        };
    }, [disconnect]);

    return {
        isConnected,
        connect,
        disconnect,
        getFrequencyData,
        getNormalizedFrequencyData,
        getTimeDomainData,
        getNormalizedTimeDomainData,
        getAverageVolume,
        getFrequencyBands,
    };
}

export default useAudioAnalyzer;
