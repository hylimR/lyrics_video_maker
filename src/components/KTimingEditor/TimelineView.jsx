import { useEffect, useRef } from 'react';

/**
 * Timeline View Component - Syllable blocks with text
 */
const TimelineView = ({
    lyric,
    syllables,
    currentTime,
    width = 560
}) => {
    const canvasRef = useRef(null);

    useEffect(() => {
        const canvas = canvasRef.current;
        if (!canvas || !lyric) return;

        const ctx = canvas.getContext('2d');
        const height = 40;

        ctx.fillStyle = 'rgba(10, 10, 20, 0.95)';
        ctx.fillRect(0, 0, width, height);

        const startTime = lyric.startTime;
        const endTime = lyric.endTime;
        const duration = endTime - startTime;
        const timeToX = (t) => ((t - startTime) / duration) * width;

        // Draw syllable blocks
        if (syllables && syllables.length > 0) {
            syllables.forEach((syl) => {
                const x = timeToX(startTime + syl.startOffset);
                const w = (syl.duration / duration) * width;

                // Filled portion based on current time
                const sylStart = startTime + syl.startOffset;
                const sylEnd = sylStart + syl.duration;
                const filled = currentTime >= sylEnd ? 1 :
                    currentTime > sylStart ? (currentTime - sylStart) / syl.duration : 0;

                // Background
                ctx.fillStyle = 'rgba(255, 255, 255, 0.1)';
                ctx.fillRect(x + 1, 4, w - 2, height - 8);

                // Filled portion
                if (filled > 0) {
                    ctx.fillStyle = 'rgba(255, 215, 0, 0.5)';
                    ctx.fillRect(x + 1, 4, (w - 2) * filled, height - 8);
                }

                // Text
                ctx.fillStyle = filled > 0.5 ? '#ffd700' : 'rgba(255, 255, 255, 0.8)';
                ctx.font = 'bold 12px Inter, sans-serif';
                ctx.textAlign = 'center';
                ctx.textBaseline = 'middle';

                const textX = x + w / 2;
                const textWidth = ctx.measureText(syl.text).width;
                if (textWidth < w - 4) {
                    ctx.fillText(syl.text, textX, height / 2);
                }
            });
        } else {
            // No timing message
            ctx.fillStyle = 'rgba(255, 255, 255, 0.3)';
            ctx.font = '12px Inter, sans-serif';
            ctx.textAlign = 'center';
            ctx.textBaseline = 'middle';
            ctx.fillText('Press [A] to auto-split or [K/Space] to mark timing', width / 2, height / 2);
        }

        // Draw current time marker
        const cursorX = timeToX(currentTime);
        if (cursorX >= 0 && cursorX <= width) {
            ctx.strokeStyle = '#ff4444';
            ctx.lineWidth = 2;
            ctx.beginPath();
            ctx.moveTo(cursorX, 0);
            ctx.lineTo(cursorX, height);
            ctx.stroke();
        }

    }, [lyric, syllables, currentTime, width]);

    return (
        <canvas
            ref={canvasRef}
            width={width}
            height={40}
            className="timeline-canvas"
        />
    );
};

export default TimelineView;
