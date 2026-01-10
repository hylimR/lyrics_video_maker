/**
 * audioWorker.js - Web Worker for heavy audio processing
 * 
 * Offloads waveform and spectrogram calculations to a background thread
 * to prevent UI freezing.
 */

// Fast FFT implementation using Cooley-Tukey algorithm
function fft(real, imag) {
    const N = real.length;

    if (N <= 1) return;

    // Bit-reversal permutation
    for (let i = 0, j = 0; i < N; i++) {
        if (i < j) {
            [real[i], real[j]] = [real[j], real[i]];
            [imag[i], imag[j]] = [imag[j], imag[i]];
        }
        let k = N >> 1;
        while (k <= j) {
            j -= k;
            k >>= 1;
        }
        j += k;
    }

    // Cooley-Tukey FFT
    for (let len = 2; len <= N; len <<= 1) {
        const halfLen = len >> 1;
        const angleStep = -2 * Math.PI / len;

        for (let i = 0; i < N; i += len) {
            for (let j = 0; j < halfLen; j++) {
                const angle = angleStep * j;
                const cos = Math.cos(angle);
                const sin = Math.sin(angle);

                const evenIdx = i + j;
                const oddIdx = i + j + halfLen;

                const tReal = real[oddIdx] * cos - imag[oddIdx] * sin;
                const tImag = real[oddIdx] * sin + imag[oddIdx] * cos;

                real[oddIdx] = real[evenIdx] - tReal;
                imag[oddIdx] = imag[evenIdx] - tImag;
                real[evenIdx] = real[evenIdx] + tReal;
                imag[evenIdx] = imag[evenIdx] + tImag;
            }
        }
    }
}

/**
 * Extract waveform peaks from audio data
 */
function extractWaveform(channelData, samplesPerSecond) {
    const audioDuration = channelData.length / 44100; // Approximate, actual rate passed separately
    const totalSamples = Math.ceil(audioDuration * samplesPerSecond);
    const samplesPerBucket = Math.floor(channelData.length / totalSamples);

    const peaks = new Float32Array(totalSamples);

    for (let i = 0; i < totalSamples; i++) {
        const start = i * samplesPerBucket;
        const end = Math.min(start + samplesPerBucket, channelData.length);

        let maxAbs = 0;
        for (let j = start; j < end; j++) {
            const absValue = Math.abs(channelData[j]);
            if (absValue > maxAbs) {
                maxAbs = absValue;
            }
        }

        peaks[i] = maxAbs;
    }

    // Normalize
    let maxPeak = 0;
    for (let i = 0; i < peaks.length; i++) {
        if (peaks[i] > maxPeak) maxPeak = peaks[i];
    }

    if (maxPeak > 0) {
        for (let i = 0; i < peaks.length; i++) {
            peaks[i] = peaks[i] / maxPeak;
        }
    }

    return peaks;
}

/**
 * Extract spectrogram using fast FFT
 */
function extractSpectrogram(channelData, sampleRate, fftSize, hopSize) {
    const numFrames = Math.floor((channelData.length - fftSize) / hopSize) + 1;
    const numBins = fftSize / 2;

    const spectrogram = new Float32Array(numFrames * numBins);

    // Hanning window
    const hanningWindow = new Float32Array(fftSize);
    for (let i = 0; i < fftSize; i++) {
        hanningWindow[i] = 0.5 * (1 - Math.cos(2 * Math.PI * i / (fftSize - 1)));
    }

    // Process each frame
    for (let frame = 0; frame < numFrames; frame++) {
        const startSample = frame * hopSize;

        // Extract and window the frame
        const real = new Float32Array(fftSize);
        const imag = new Float32Array(fftSize);

        for (let i = 0; i < fftSize; i++) {
            const idx = startSample + i;
            real[i] = (idx < channelData.length ? channelData[idx] : 0) * hanningWindow[i];
            imag[i] = 0;
        }

        // Compute FFT
        fft(real, imag);

        // Store magnitudes (only positive frequencies)
        for (let bin = 0; bin < numBins; bin++) {
            const magnitude = Math.sqrt(real[bin] * real[bin] + imag[bin] * imag[bin]) / fftSize;
            spectrogram[frame * numBins + bin] = magnitude;
        }

        // Report progress every 100 frames
        if (frame % 100 === 0) {
            self.postMessage({
                type: 'progress',
                task: 'spectrogram',
                progress: frame / numFrames
            });
        }
    }

    // Normalize to dB scale
    let maxVal = 0;
    for (let i = 0; i < spectrogram.length; i++) {
        spectrogram[i] = Math.log10(spectrogram[i] + 1e-10) + 10;
        if (spectrogram[i] > maxVal) maxVal = spectrogram[i];
    }

    if (maxVal > 0) {
        for (let i = 0; i < spectrogram.length; i++) {
            spectrogram[i] = Math.max(0, spectrogram[i] / maxVal);
        }
    }

    return { spectrogram, numFrames, numBins };
}

// Message handler
self.onmessage = function (e) {
    const { type, data } = e.data;

    try {
        if (type === 'extractWaveform') {
            const { channelData, sampleRate, samplesPerSecond, duration } = data;

            self.postMessage({ type: 'progress', task: 'waveform', progress: 0 });

            const peaks = extractWaveform(new Float32Array(channelData), samplesPerSecond);

            self.postMessage({
                type: 'waveformResult',
                data: {
                    peaks: peaks.buffer,
                    sampleRate: samplesPerSecond,
                    originalSampleRate: sampleRate,
                    duration
                }
            }, [peaks.buffer]);

        } else if (type === 'extractSpectrogram') {
            const { channelData, sampleRate, fftSize, hopSize, duration } = data;

            self.postMessage({ type: 'progress', task: 'spectrogram', progress: 0 });

            const result = extractSpectrogram(
                new Float32Array(channelData),
                sampleRate,
                fftSize,
                hopSize
            );

            self.postMessage({
                type: 'spectrogramResult',
                data: {
                    data: result.spectrogram.buffer,
                    numFrames: result.numFrames,
                    numBins: result.numBins,
                    sampleRate,
                    hopSize,
                    duration,
                    framesPerSecond: sampleRate / hopSize
                }
            }, [result.spectrogram.buffer]);
        }
    } catch (error) {
        self.postMessage({
            type: 'error',
            error: error.message
        });
    }
};
