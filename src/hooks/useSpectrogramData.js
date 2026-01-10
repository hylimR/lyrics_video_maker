import { useState, useEffect, useRef, useCallback } from 'react';

/**
 * useSpectrogramData - Extract spectrogram data from an audio source
 * 
 * Uses a Web Worker with fast Cooley-Tukey FFT to offload heavy computation
 * and prevent UI freezing.
 * 
 * @param {string} audioSource - URL of the audio file
 * @param {number} fftSize - FFT window size (power of 2, default: 512)
 * @param {number} hopSize - Number of samples between FFT frames (default: fftSize/4)
 * @returns {Object} - { spectrogramData, duration, isLoading, error, progress, getSpectrogramSlice }
 */
export function useSpectrogramData(audioSource, fftSize = 512, hopSize = null) {
    const [spectrogramData, setSpectrogramData] = useState(null);
    const [duration, setDuration] = useState(0);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState(null);
    const [progress, setProgress] = useState(0);
    const workerRef = useRef(null);
    const cacheKeyRef = useRef(null);

    // Default hop size to fftSize / 4 for 75% overlap
    const actualHopSize = hopSize || Math.floor(fftSize / 4);

    useEffect(() => {
        if (!audioSource) {
            setSpectrogramData(null);
            setDuration(0);
            return;
        }

        // Skip if we already processed this source
        if (cacheKeyRef.current === audioSource && spectrogramData) {
            return;
        }

        const extractSpectrogram = async () => {
            setIsLoading(true);
            setError(null);
            setProgress(0);

            try {
                // Fetch the audio file
                const response = await fetch(audioSource);
                if (!response.ok) {
                    throw new Error(`Failed to fetch audio: ${response.status}`);
                }

                const arrayBuffer = await response.arrayBuffer();

                // Decode with a temporary audio context
                const AudioContextClass = window.AudioContext || window.webkitAudioContext;
                const tempCtx = new AudioContextClass();
                const audioBuffer = await tempCtx.decodeAudioData(arrayBuffer);
                await tempCtx.close();

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
                    } else if (type === 'spectrogramResult') {
                        const spectData = new Float32Array(data.data);

                        setSpectrogramData({
                            data: spectData,
                            numFrames: data.numFrames,
                            numBins: data.numBins,
                            sampleRate: data.sampleRate,
                            hopSize: data.hopSize,
                            duration: data.duration,
                            framesPerSecond: data.framesPerSecond
                        });
                        setDuration(data.duration);
                        cacheKeyRef.current = audioSource;
                        setIsLoading(false);
                        setProgress(1);

                        console.log(`📊 Spectrogram extracted: ${data.numFrames} frames × ${data.numBins} bins, ${data.duration.toFixed(2)}s`);
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
                    type: 'extractSpectrogram',
                    data: {
                        channelData: channelDataCopy.buffer,
                        sampleRate,
                        fftSize,
                        hopSize: actualHopSize,
                        duration: audioDuration
                    }
                }, [channelDataCopy.buffer]);

            } catch (err) {
                console.error('Spectrogram extraction failed:', err);
                setError(err.message);
                setSpectrogramData(null);
                setIsLoading(false);
            }
        };

        extractSpectrogram();

    }, [audioSource, fftSize, actualHopSize, spectrogramData]);

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
     * Get spectrogram data for a specific time range
     */
    const getSpectrogramSlice = useCallback((startTime, endTime) => {
        if (!spectrogramData || !spectrogramData.data) return null;

        const { data, numBins, framesPerSecond } = spectrogramData;
        const startFrame = Math.floor(startTime * framesPerSecond);
        const endFrame = Math.ceil(endTime * framesPerSecond);

        // Bounds check
        const totalFrames = spectrogramData.numFrames;
        const clampedStart = Math.max(0, startFrame);
        const clampedEnd = Math.min(totalFrames, endFrame);

        if (clampedStart >= clampedEnd) return null;

        const sliceFrames = clampedEnd - clampedStart;
        const sliceData = new Float32Array(sliceFrames * numBins);

        // Copy the slice
        for (let f = 0; f < sliceFrames; f++) {
            const srcOffset = (clampedStart + f) * numBins;
            const dstOffset = f * numBins;
            for (let b = 0; b < numBins; b++) {
                sliceData[dstOffset + b] = data[srcOffset + b];
            }
        }

        return {
            data: sliceData,
            numFrames: sliceFrames,
            numBins
        };
    }, [spectrogramData]);

    return {
        spectrogramData,
        duration,
        isLoading,
        error,
        progress,
        getSpectrogramSlice
    };
}

export default useSpectrogramData;
