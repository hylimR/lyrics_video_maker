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
    const isDragging = useRef(false);

    // Get waveform slice for current lyric line
    const lineWaveform = useMemo(() => {
        if (!lyric || !getWaveformSlice) return null;
        return getWaveformSlice(lyric.startTime, lyric.endTime);
    }, [lyric, getWaveformSlice]);

    const duration = lyric ? lyric.endTime - lyric.startTime : 0;

    // Calculate time from X coordinate (Component Scope)
    const xToTime = (x) => {
        if (!lyric) return 0;
        const boundedX = Math.max(0, Math.min(width, x));
        return lyric.startTime + (boundedX / width) * duration;
    };

    // Calculate X from time (Component Scope - Helper)
    const timeToX = (t) => {
        if (!lyric || duration === 0) return 0;
        return ((t - lyric.startTime) / duration) * width;
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

    // ⚡ Bolt Optimization: Cache the static waveform drawing to avoid O(W) loop on every frame
    const cachedWaveforms = useMemo(() => {
        if (!lineWaveform || lineWaveform.length === 0 || !width || !height) return null;

        const dpr = window.devicePixelRatio || 1;
        const w = Math.floor(width * dpr);
        const h = Math.floor(height * dpr);
        const samplesPerPixel = lineWaveform.length / width; // Use CSS width for sampling distribution

        // Create canvas for "unplayed" state
        const unplayedCanvas = document.createElement('canvas');
        unplayedCanvas.width = w;
        unplayedCanvas.height = h;
        const uCtx = unplayedCanvas.getContext('2d');
        uCtx.scale(dpr, dpr);

        // Create canvas for "played" state
        const playedCanvas = document.createElement('canvas');
        playedCanvas.width = w;
        playedCanvas.height = h;
        const pCtx = playedCanvas.getContext('2d');
        pCtx.scale(dpr, dpr);

        uCtx.fillStyle = 'rgba(60, 180, 80, 0.7)';
        pCtx.fillStyle = 'rgba(100, 255, 130, 0.95)';

        const centerY = height / 2;

        for (let x = 0; x < width; x++) {
            const sampleIdx = Math.floor(x * samplesPerPixel);
            const amplitude = lineWaveform[sampleIdx] || 0;
            const barHeight = amplitude * (height * 0.45); // Use CSS height

            // Draw to both canvases
            uCtx.fillRect(x, centerY - barHeight, 1, barHeight * 2);
            pCtx.fillRect(x, centerY - barHeight, 1, barHeight * 2);
        }

        return { unplayed: unplayedCanvas, played: playedCanvas };
    }, [lineWaveform, width, height]);

    // Draw the waveform
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

        const centerY = height / 2;

        // Clear with dark background
        ctx.fillStyle = 'rgba(0, 30, 20, 0.98)';
        ctx.fillRect(0, 0, width, height);

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

        // Draw waveform (Using Cached Bitmaps)
        if (cachedWaveforms) {
            // Draw unplayed version (full width)
            ctx.drawImage(cachedWaveforms.unplayed, 0, 0, width, height);

            // Draw played version (clipped)
            const cursorX = timeToX(currentTime);
            if (cursorX > 0) {
                ctx.save();
                ctx.beginPath();
                ctx.rect(0, 0, cursorX, height);
                ctx.clip();
                ctx.drawImage(cachedWaveforms.played, 0, 0, width, height);
                ctx.restore();
            }
        } else if (lineWaveform && lineWaveform.length > 0) {
             // Should usually use cache if lineWaveform exists.
             // If cache is somehow null but lineWaveform isn't (rare race), do nothing or fallback
        } else {
            // No waveform data - show loading message
            ctx.fillStyle = 'rgba(100, 255, 130, 0.5)';
            ctx.font = '11px JetBrains Mono, monospace';
            ctx.textAlign = 'center';
            ctx.fillText('Loading waveform...', width / 2, centerY + 4);
        }

        // Draw current time cursor (white line like Aegisub)
        const cursorX = timeToX(currentTime);
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

    }, [lyric, syllables, currentTime, width, height, lineWaveform, cachedWaveforms, duration]); // Added duration dependency, removed timeToX as it is constant-ish (depends on props)
    // Wait, timeToX depends on width, lyric, duration.
    // Since timeToX is defined in render scope, it is recreated every render.
    // So if I use it in useEffect, I should add it to dependency, or verify it doesn't cause issues.
    // Actually, `useEffect` depends on `width`, `lyric` anyway.
    // So reusing `timeToX` from scope is fine.

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
