import { useEffect, useRef, useMemo } from 'react';

const formatTime = (seconds) => {
    if (typeof seconds !== 'number' || isNaN(seconds)) return '0:00.00';
    const mins = Math.floor(seconds / 60);
    const secs = (seconds % 60).toFixed(2);
    return `${mins}:${secs.padStart(5, '0')}`;
};

/**
 * Waveform View Component - Aegisub-style audio waveform visualization
 * 
 * Features:
 * - Pre-computed PCM waveform display (not real-time frequency)
 * - Mirrored waveform (positive/negative amplitude)
 * - Interactive seeking by clicking/dragging
 * - Syllable boundary markers
 * - Current time cursor
 * 
 * @param {Object} props
 * @param {Object} props.lyric - Current lyric line object
 * @param {Array} props.syllables - Syllable timing data
 * @param {number} props.currentTime - Current playback time
 * @param {Function} props.onSeek - Seek callback
 * @param {Function} props.getWaveformSlice - Function to get waveform slice for time range
 * @param {number} props.width - Canvas width
 * @param {number} props.height - Canvas height
 */
const WaveformView = ({
    lyric,
    syllables,
    currentTime,
    onSeek,
    getWaveformSlice,
    width = 560,
    height = 80
}) => {
    const canvasRef = useRef(null);
    const cachedUnplayedRef = useRef(null);
    const cachedPlayedRef = useRef(null);
    const isDragging = useRef(false);

    // Get waveform slice for current lyric line
    const lineWaveform = useMemo(() => {
        if (!lyric || !getWaveformSlice) return null;
        return getWaveformSlice(lyric.startTime, lyric.endTime);
    }, [lyric, getWaveformSlice]);

    // Calculate time from X coordinate
    const xToTime = (x) => {
        if (!lyric) return 0;
        const duration = lyric.endTime - lyric.startTime;
        const boundedX = Math.max(0, Math.min(width, x));
        return lyric.startTime + (boundedX / width) * duration;
    };

    const handleMouseDown = (e) => {
        if (!onSeek) return;
        isDragging.current = true;

        const rect = canvasRef.current.getBoundingClientRect();
        const x = e.clientX - rect.left;
        onSeek(xToTime(x));
    };

    const handleMouseMove = (e) => {
        if (isDragging.current && onSeek) {
            const rect = canvasRef.current.getBoundingClientRect();
            const x = e.clientX - rect.left;
            onSeek(xToTime(x));
        }
    };

    const handleMouseUp = () => {
        isDragging.current = false;
    };

    const handleMouseLeave = () => {
        isDragging.current = false;
    };

    // Draw the static waveform into offscreen canvases
    useEffect(() => {
        if (!lyric || !width || !height) return;

        // Initialize offscreen canvases if needed
        if (!cachedUnplayedRef.current) cachedUnplayedRef.current = document.createElement('canvas');
        if (!cachedPlayedRef.current) cachedPlayedRef.current = document.createElement('canvas');

        const unplayedCanvas = cachedUnplayedRef.current;
        const playedCanvas = cachedPlayedRef.current;
        const ctxUnplayed = unplayedCanvas.getContext('2d');
        const ctxPlayed = playedCanvas.getContext('2d');

        // Handle High DPI
        const dpr = window.devicePixelRatio || 1;

        // Resize offscreen canvases
        [unplayedCanvas, playedCanvas].forEach(c => {
            if (c.width !== width * dpr || c.height !== height * dpr) {
                c.width = width * dpr;
                c.height = height * dpr;
            }
        });

        ctxUnplayed.setTransform(dpr, 0, 0, dpr, 0, 0);
        ctxPlayed.setTransform(dpr, 0, 0, dpr, 0, 0);

        const duration = lyric.endTime - lyric.startTime;
        const timeToX = (t) => ((t - lyric.startTime) / duration) * width;
        const centerY = height / 2;

        // Clear canvases with background color (for consistent blending/replacement)
        [ctxUnplayed, ctxPlayed].forEach(ctx => {
             ctx.fillStyle = 'rgba(0, 30, 20, 0.98)';
             ctx.fillRect(0, 0, width, height);
        });

        // Helper to draw common elements (grid, etc.)
        const drawGrid = (ctx) => {
            // Draw center line
            ctx.strokeStyle = 'rgba(50, 150, 80, 0.4)';
            ctx.lineWidth = 1;
            ctx.beginPath();
            ctx.moveTo(0, centerY);
            ctx.lineTo(width, centerY);
            ctx.stroke();

            // Draw syllable boundaries
            if (syllables && syllables.length > 0) {
                ctx.strokeStyle = 'rgba(255, 215, 0, 0.5)';
                ctx.lineWidth = 1;

                syllables.forEach((syl) => {
                    const x = timeToX(lyric.startTime + syl.startOffset);
                    ctx.beginPath();
                    ctx.moveTo(x, 0);
                    ctx.lineTo(x, height);
                    ctx.stroke();
                });
            }
        };

        // Draw grid on both layers
        drawGrid(ctxUnplayed);
        drawGrid(ctxPlayed);

        // Draw waveform
        if (lineWaveform && lineWaveform.length > 0) {
            const samplesPerPixel = lineWaveform.length / width;

            // Pre-calculate common values
            const heightFactor = height * 0.45;

            // Draw filled waveform (Aegisub style - mirrored)
            for (let x = 0; x < width; x++) {
                const sampleIdx = Math.floor(x * samplesPerPixel);
                const amplitude = lineWaveform[sampleIdx] || 0;

                // Calculate bar height (mirrored from center)
                const barHeight = amplitude * heightFactor;

                // Draw to Unplayed (Dimmer green)
                ctxUnplayed.fillStyle = 'rgba(60, 180, 80, 0.7)';
                ctxUnplayed.fillRect(x, centerY - barHeight, 1, barHeight * 2);

                // Draw to Played (Bright green)
                ctxPlayed.fillStyle = 'rgba(100, 255, 130, 0.95)';
                ctxPlayed.fillRect(x, centerY - barHeight, 1, barHeight * 2);
            }
        } else {
            // No waveform data - show loading message on BOTH
            [ctxUnplayed, ctxPlayed].forEach(ctx => {
                ctx.fillStyle = 'rgba(100, 255, 130, 0.5)';
                ctx.font = '11px JetBrains Mono, monospace';
                ctx.textAlign = 'center';
                ctx.fillText('Loading waveform...', width / 2, centerY + 4);
            });
        }

    }, [lyric, syllables, width, height, lineWaveform]);


    // Draw the dynamic frame (compositing)
    useEffect(() => {
        const canvas = canvasRef.current;
        if (!canvas || !lyric) return;

        const ctx = canvas.getContext('2d');
        const dpr = window.devicePixelRatio || 1;

        // Ensure canvas size matches (in case it changed)
        if (canvas.width !== width * dpr || canvas.height !== height * dpr) {
             canvas.width = width * dpr;
             canvas.height = height * dpr;
        }

        // Reset transform to identity for drawImage
        ctx.setTransform(1, 0, 0, 1, 0, 0);

        // 1. Draw Unplayed Layer (Background + Grid + Unplayed Bars)
        if (cachedUnplayedRef.current) {
            ctx.drawImage(cachedUnplayedRef.current, 0, 0);
        } else {
             ctx.fillStyle = 'rgba(0, 30, 20, 0.98)';
             ctx.fillRect(0, 0, canvas.width, canvas.height);
        }

        // 2. Draw Played Layer (Background + Grid + Played Bars) - Clipped
        const duration = lyric.endTime - lyric.startTime;
        const timeToX = (t) => ((t - lyric.startTime) / duration) * width;
        const cursorX = timeToX(currentTime); // logical pixels

        if (cachedPlayedRef.current && cursorX > 0) {
            ctx.save();
            ctx.beginPath();
            // Clip region in physical pixels
            ctx.rect(0, 0, cursorX * dpr, height * dpr);
            ctx.clip();
            ctx.drawImage(cachedPlayedRef.current, 0, 0);
            ctx.restore();
        }

        // Now set scale for UI elements (cursor, text) which use logical coordinates
        ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

        // Draw current time cursor (white line like Aegisub)
        if (cursorX >= 0 && cursorX <= width) {
            ctx.strokeStyle = 'rgba(255, 255, 255, 0.9)';
            ctx.lineWidth = 1;
            ctx.beginPath();
            ctx.moveTo(cursorX, 0);
            ctx.lineTo(cursorX, height);
            ctx.stroke();
        }

        // Draw time markers
        ctx.fillStyle = 'rgba(255, 255, 255, 0.5)';
        ctx.font = '9px JetBrains Mono, monospace';
        ctx.textAlign = 'left';
        ctx.fillText(formatTime(lyric.startTime), 4, 12);
        ctx.textAlign = 'right';
        ctx.fillText(formatTime(lyric.endTime), width - 4, 12);

        // Current time display at cursor
        ctx.fillStyle = 'rgba(255, 255, 255, 0.8)';
        ctx.textAlign = 'center';
        ctx.fillText(formatTime(currentTime), width / 2, height - 4);

    }, [currentTime, width, height, lyric, lineWaveform, syllables]);

    return (
        <canvas
            ref={canvasRef}
            className="waveform-canvas"
            onMouseDown={handleMouseDown}
            onMouseMove={handleMouseMove}
            onMouseUp={handleMouseUp}
            onMouseLeave={handleMouseLeave}
            style={{ cursor: 'pointer', width, height, borderRadius: '4px' }}
        />
    );
};

export default WaveformView;
