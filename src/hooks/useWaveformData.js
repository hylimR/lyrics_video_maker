import { useState, useEffect, useRef, useCallback } from 'react';

/**
 * useWaveformData - Extract waveform amplitude data from an audio source
 * 
 * Uses a Web Worker to offload heavy computation and prevent UI freezing.
 * 
 * @param {string} audioSource - URL of the audio file
 * @param {number} samplesPerSecond - Number of samples per second of audio (default: 100)
 * @returns {Object} - { waveformData, duration, isLoading, error, progress, getWaveformSlice }
 */
export function useWaveformData(audioSource, samplesPerSecond = 100) {
    const [waveformData, setWaveformData] = useState(null);
    const [duration, setDuration] = useState(0);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState(null);
    const [progress, setProgress] = useState(0);
    const audioContextRef = useRef(null);
    const workerRef = useRef(null);
    const cacheKeyRef = useRef(null);

    useEffect(() => {
        if (!audioSource) {
            setWaveformData(null);
            setDuration(0);
            return;
        }

        // Skip if we already processed this source
        if (cacheKeyRef.current === audioSource && waveformData) {
            return;
        }

        const extractWaveform = async () => {
            setIsLoading(true);
            setError(null);
            setProgress(0);

            try {
                // Create audio context if needed
                if (!audioContextRef.current) {
                    audioContextRef.current = new (window.AudioContext || window.webkitAudioContext)();
                }

                const ctx = audioContextRef.current;

                // Fetch the audio file
                const response = await fetch(audioSource);
                if (!response.ok) {
                    throw new Error(`Failed to fetch audio: ${response.status}`);
                }

                const arrayBuffer = await response.arrayBuffer();

                // Decode the audio data
                const audioBuffer = await ctx.decodeAudioData(arrayBuffer);

                // Get the raw PCM data
                const channelData = audioBuffer.getChannelData(0);
                const sampleRate = audioBuffer.sampleRate;
                const audioDuration = audioBuffer.duration;

                // Create worker if not exists
                if (!workerRef.current) {
                    workerRef.current = new Worker(
                        new URL('../workers/audioWorker.js', import.meta.url),
                        { type: 'module' }
                    );
                }

                const worker = workerRef.current;

                // Handle worker messages
                worker.onmessage = (e) => {
                    const { type, data, error: workerError } = e.data;

                    if (type === 'progress') {
                        setProgress(e.data.progress);
                    } else if (type === 'waveformResult') {
                        const peaks = new Float32Array(data.peaks);

                        setWaveformData({
                            peaks,
                            sampleRate: data.sampleRate,
                            originalSampleRate: data.originalSampleRate,
                            duration: data.duration
                        });
                        setDuration(data.duration);
                        cacheKeyRef.current = audioSource;
                        setIsLoading(false);
                        setProgress(1);

                        console.log(`🎵 Waveform extracted: ${peaks.length} samples, ${data.duration.toFixed(2)}s`);
                    } else if (type === 'error') {
                        setError(workerError);
                        setIsLoading(false);
                    }
                };

                worker.onerror = (e) => {
                    setError(e.message);
                    setIsLoading(false);
                };

                // Send data to worker (transfer the buffer for performance)
                const channelDataCopy = new Float32Array(channelData);
                worker.postMessage({
                    type: 'extractWaveform',
                    data: {
                        channelData: channelDataCopy.buffer,
                        sampleRate,
                        samplesPerSecond,
                        duration: audioDuration
                    }
                }, [channelDataCopy.buffer]);

            } catch (err) {
                console.error('Waveform extraction failed:', err);
                setError(err.message);
                setWaveformData(null);
                setIsLoading(false);
            }
        };

        extractWaveform();

    }, [audioSource, samplesPerSecond, waveformData]);

    // Cleanup worker
    useEffect(() => {
        return () => {
            if (workerRef.current) {
                workerRef.current.terminate();
                workerRef.current = null;
            }
        };
    }, []);

    /**
     * Get waveform data for a specific time range
     */
    const getWaveformSlice = useCallback((startTime, endTime) => {
        if (!waveformData || !waveformData.peaks) return null;

        const { peaks, sampleRate } = waveformData;
        const startSample = Math.floor(startTime * sampleRate);
        const endSample = Math.ceil(endTime * sampleRate);

        const clampedStart = Math.max(0, startSample);
        const clampedEnd = Math.min(peaks.length, endSample);

        if (clampedStart >= clampedEnd) return null;

        return peaks.slice(clampedStart, clampedEnd);
    }, [waveformData]);

    return {
        waveformData,
        duration,
        isLoading,
        error,
        progress,
        getWaveformSlice
    };
}

export default useWaveformData;
