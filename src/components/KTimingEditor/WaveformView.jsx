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

        const duration = lyric.endTime - lyric.startTime;
        const timeToX = (t) => ((t - lyric.startTime) / duration) * width;
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

        // Draw waveform
        if (lineWaveform && lineWaveform.length > 0) {
            const samplesPerPixel = lineWaveform.length / width;

            // Draw filled waveform (Aegisub style - mirrored)
            for (let x = 0; x < width; x++) {
                const sampleIdx = Math.floor(x * samplesPerPixel);
                const amplitude = lineWaveform[sampleIdx] || 0;
                const sampleTime = lyric.startTime + (x / width) * duration;

                // Calculate bar height (mirrored from center)
                const barHeight = amplitude * (height * 0.45);

                // Color: Bright green for played, dimmer for unplayed
                if (sampleTime <= currentTime) {
                    // Played portion - bright green
                    ctx.fillStyle = 'rgba(100, 255, 130, 0.95)';
                } else {
                    // Unplayed - dimmer green
                    ctx.fillStyle = 'rgba(60, 180, 80, 0.7)';
                }

                // Draw mirrored bar (above and below center line)
                ctx.fillRect(x, centerY - barHeight, 1, barHeight * 2);
            }
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

    }, [lyric, syllables, currentTime, width, height, lineWaveform]);

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
