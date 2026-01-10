import { useEffect, useRef, useMemo } from 'react';

/**
 * SpectrumView - Aegisub-style Spectrogram Visualization
 * 
 * Displays a pre-computed spectrogram showing frequency content over time.
 * - X-axis: Time
 * - Y-axis: Frequency (low at bottom, high at top)
 * - Color intensity: Amplitude
 * 
 * @param {Object} props
 * @param {Object} props.lyric - Current lyric line object
 * @param {number} props.currentTime - Current playback time
 * @param {Function} props.getSpectrogramSlice - Function to get spectrogram data for time range
 * @param {number} props.width - Canvas width
 * @param {number} props.height - Canvas height
 */
const SpectrumView = ({
    lyric,
    currentTime,
    getSpectrogramSlice,
    width = 600,
    height = 80
}) => {
    const canvasRef = useRef(null);
    const imageDataCacheRef = useRef(null);
    const cachedLyricRef = useRef(null);

    // Get spectrogram slice for current lyric line
    const lineSpectrogram = useMemo(() => {
        if (!lyric || !getSpectrogramSlice) return null;
        return getSpectrogramSlice(lyric.startTime, lyric.endTime);
    }, [lyric, getSpectrogramSlice]);

    // Color mapping function (intensity 0-1 to RGB)
    // Green-based color scheme like Aegisub
    const intensityToColor = (intensity) => {
        // Clamp to 0-1
        const v = Math.max(0, Math.min(1, intensity));

        // Color gradient: dark -> green -> yellow -> white
        if (v < 0.25) {
            // Dark to dim green
            const t = v / 0.25;
            return [
                Math.floor(10 * t),
                Math.floor(40 + 60 * t),
                Math.floor(20 * t)
            ];
        } else if (v < 0.5) {
            // Dim green to bright green
            const t = (v - 0.25) / 0.25;
            return [
                Math.floor(10 + 50 * t),
                Math.floor(100 + 100 * t),
                Math.floor(20 + 30 * t)
            ];
        } else if (v < 0.75) {
            // Bright green to yellow-green
            const t = (v - 0.5) / 0.25;
            return [
                Math.floor(60 + 140 * t),
                Math.floor(200 + 55 * t),
                Math.floor(50 - 20 * t)
            ];
        } else {
            // Yellow-green to bright yellow/white
            const t = (v - 0.75) / 0.25;
            return [
                Math.floor(200 + 55 * t),
                Math.floor(255),
                Math.floor(30 + 180 * t)
            ];
        }
    };

    // Draw the spectrogram
    useEffect(() => {
        const canvas = canvasRef.current;
        if (!canvas || !lyric) return;

        const ctx = canvas.getContext('2d');

        // Handle High DPI
        const dpr = window.devicePixelRatio || 1;
        canvas.width = width * dpr;
        canvas.height = height * dpr;
        ctx.scale(dpr, dpr);

        canvas.style.width = `${width}px`;
        canvas.style.height = `${height}px`;

        const duration = lyric.endTime - lyric.startTime;
        const timeToX = (t) => ((t - lyric.startTime) / duration) * width;

        // Clear with dark background
        ctx.fillStyle = 'rgba(0, 20, 15, 0.98)';
        ctx.fillRect(0, 0, width, height);

        // Draw spectrogram
        if (lineSpectrogram && lineSpectrogram.data && lineSpectrogram.numFrames > 0) {
            const { data, numFrames, numBins } = lineSpectrogram;

            // Only regenerate image data if lyric changed
            const lyricKey = `${lyric.startTime}-${lyric.endTime}`;
            if (cachedLyricRef.current !== lyricKey || !imageDataCacheRef.current) {
                // Create image data for the spectrogram
                const imageData = ctx.createImageData(width, height);
                const pixels = imageData.data;

                // Map spectrogram to canvas pixels
                for (let x = 0; x < width; x++) {
                    const frameIdx = Math.floor((x / width) * numFrames);

                    for (let y = 0; y < height; y++) {
                        // Flip Y so low frequencies are at bottom
                        const binIdx = Math.floor(((height - 1 - y) / height) * numBins);

                        // Get intensity value
                        const dataIdx = frameIdx * numBins + binIdx;
                        const intensity = data[dataIdx] || 0;

                        // Convert to color
                        const [r, g, b] = intensityToColor(intensity);

                        // Set pixel (RGBA)
                        const pixelIdx = (y * width + x) * 4;
                        pixels[pixelIdx] = r;
                        pixels[pixelIdx + 1] = g;
                        pixels[pixelIdx + 2] = b;
                        pixels[pixelIdx + 3] = 255; // Alpha
                    }
                }

                imageDataCacheRef.current = imageData;
                cachedLyricRef.current = lyricKey;
            }

            // Draw cached image
            if (imageDataCacheRef.current) {
                ctx.putImageData(imageDataCacheRef.current, 0, 0);
            }
        } else {
            // No spectrogram data - show loading message
            ctx.fillStyle = 'rgba(100, 255, 130, 0.5)';
            ctx.font = '11px JetBrains Mono, monospace';
            ctx.textAlign = 'center';
            ctx.fillText('Loading spectrogram...', width / 2, height / 2 + 4);
        }

        // Draw current time cursor (white line)
        const cursorX = timeToX(currentTime);
        if (cursorX >= 0 && cursorX <= width) {
            ctx.strokeStyle = 'rgba(255, 255, 255, 0.9)';
            ctx.lineWidth = 1;
            ctx.beginPath();
            ctx.moveTo(cursorX, 0);
            ctx.lineTo(cursorX, height);
            ctx.stroke();
        }

        // Draw frequency labels
        ctx.fillStyle = 'rgba(255, 255, 255, 0.5)';
        ctx.font = '8px JetBrains Mono, monospace';
        ctx.textAlign = 'left';
        ctx.fillText('HIGH', 4, 12);
        ctx.fillText('LOW', 4, height - 4);

    }, [lyric, currentTime, width, height, lineSpectrogram]);

    return (
        <canvas
            ref={canvasRef}
            className="spectrum-canvas"
            style={{
                borderRadius: '4px',
                cursor: 'default',
                width,
                height
            }}
        />
    );
};

export default SpectrumView;
