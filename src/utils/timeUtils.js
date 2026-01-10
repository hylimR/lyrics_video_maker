export const formatTime = (seconds) => {
    if (typeof seconds !== 'number' || isNaN(seconds)) return '0:00.00';
    const mins = Math.floor(seconds / 60);
    const secs = (seconds % 60).toFixed(2);
    return `${mins}:${secs.padStart(5, '0')}`;
};

export const parseTimeInput = (value) => {
    // Parse mm:ss.xx or ss.xx format
    const parts = value.split(':');
    if (parts.length === 2) {
        const mins = parseInt(parts[0], 10) || 0;
        const secs = parseFloat(parts[1]) || 0;
        return mins * 60 + secs;
    }
    return parseFloat(value) || 0;
};
