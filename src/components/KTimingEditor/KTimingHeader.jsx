import { formatTime } from '@/utils/timeUtils';

export const KTimingHeader = ({
    lyrics,
    lineIndex,
    currentTime,
    onLineChange,
    onClose
}) => {
    return (
        <div className="k-timing-header">
            <h3>🎹 K-Timing Editor</h3>
            <div className="header-info">
                <select
                    className="line-selector"
                    value={lineIndex}
                    onChange={(e) => {
                        const newIndex = parseInt(e.target.value, 10);
                        onLineChange(newIndex);
                    }}
                >
                    {lyrics.map((l, i) => (
                        <option key={i} value={i}>
                            #{i + 1}: {l.text.substring(0, 30)}{l.text.length > 30 ? '...' : ''}
                        </option>
                    ))}
                </select>
                <span className="total-lines">/ {lyrics.length}</span>
                <span className="current-time">⏱️ {formatTime(currentTime)}</span>
            </div>
            <button onClick={onClose} className="close-btn">✕</button>
        </div>
    );
};
