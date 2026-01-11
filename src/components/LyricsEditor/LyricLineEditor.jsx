import { useState, useCallback, useRef, useEffect, memo } from 'react';
import { formatTime, parseTimeInput } from '@/utils/timeUtils';
import FontSelector from '@/components/FontSelector';

const LyricLineEditor = ({
    lyric,
    index,
    isActive,
    currentTimeRef,
    onUpdate,
    onDelete,
    onSetStartTime,
    onSetEndTime,
    onMoveUp,
    onMoveDown,
    isFirst,
    isLast,
    availableFonts
}) => {
    const [isEditing, setIsEditing] = useState(false);
    const [editText, setEditText] = useState(lyric.text);
    const [editStart, setEditStart] = useState(formatTime(lyric.startTime));
    const [editEnd, setEditEnd] = useState(formatTime(lyric.endTime));
    const [editFont, setEditFont] = useState(lyric.font?.family || '');

    const inputRef = useRef(null);

    useEffect(() => {
        setEditText(lyric.text);
        setEditStart(formatTime(lyric.startTime));
        setEditEnd(formatTime(lyric.endTime));
        setEditFont(lyric.font?.family || '');
    }, [lyric]);

    const handleSave = useCallback(() => {
        onUpdate(index, {
            text: editText,
            startTime: parseTimeInput(editStart),
            endTime: parseTimeInput(editEnd),
            font: editFont ? { family: editFont, size: lyric.font?.size || 72 } : undefined
        });
        setIsEditing(false);
    }, [index, editText, editStart, editEnd, editFont, lyric, onUpdate]);

    const handleCancel = useCallback(() => {
        setEditText(lyric.text);
        setEditStart(formatTime(lyric.startTime));
        setEditEnd(formatTime(lyric.endTime));
        setIsEditing(false);
    }, [lyric]);

    const handleKeyDown = useCallback((e) => {
        if (e.key === 'Enter') {
            handleSave();
        } else if (e.key === 'Escape') {
            handleCancel();
        }
    }, [handleSave, handleCancel]);

    useEffect(() => {
        if (isEditing && inputRef.current) {
            inputRef.current.focus();
            inputRef.current.select();
        }
    }, [isEditing]);

    return (
        <div className={`lyric-line-editor ${isActive ? 'active' : ''} ${isEditing ? 'editing' : ''}`}>
            <div className="line-number">{index + 1}</div>

            <div className="line-content">
                {isEditing ? (
                    <div className="edit-form">
                        <input
                            ref={inputRef}
                            type="text"
                            className="edit-text"
                            value={editText}
                            onChange={(e) => setEditText(e.target.value)}
                            onKeyDown={handleKeyDown}
                            placeholder="Lyric text..."
                        />
                        <div className="edit-times">
                            <div className="time-input-group">
                                <label>Start:</label>
                                <input
                                    type="text"
                                    className="edit-time"
                                    value={editStart}
                                    onChange={(e) => setEditStart(e.target.value)}
                                    onKeyDown={handleKeyDown}
                                    placeholder="0:00.00"
                                />
                                <button
                                    className="set-time-btn"
                                    onClick={() => {
                                        setEditStart(formatTime(currentTimeRef.current));
                                    }}
                                    title="Set to current time"
                                >
                                    ⏱️
                                </button>
                            </div>
                            <div className="time-input-group">
                                <label>End:</label>
                                <input
                                    type="text"
                                    className="edit-time"
                                    value={editEnd}
                                    onChange={(e) => setEditEnd(e.target.value)}
                                    onKeyDown={handleKeyDown}
                                    placeholder="0:00.00"
                                />
                                <button
                                    className="set-time-btn"
                                    onClick={() => {
                                        setEditEnd(formatTime(currentTimeRef.current));
                                    }}
                                    title="Set to current time"
                                >
                                    ⏱️
                                </button>
                            </div>
                        </div>
                        
                        {/* Font Selector */}
                        <div className="edit-font">
                             <FontSelector
                                 value={editFont}
                                 onChange={setEditFont}
                                 fonts={availableFonts}
                                 placeholder="Default Font"
                             />
                        </div>

                        <div className="edit-actions">
                            <button className="save-btn" onClick={handleSave}>✓ Save</button>
                            <button className="cancel-btn" onClick={handleCancel}>✕ Cancel</button>
                        </div>
                    </div>
                ) : (
                    <>
                        <div className="line-text" onClick={() => setIsEditing(true)}>
                            {lyric.text || '(empty)'}
                        </div>
                        <div className="line-times">
                            <span className="time start">{formatTime(lyric.startTime)}</span>
                            <span className="time-arrow">→</span>
                            <span className="time end">{formatTime(lyric.endTime)}</span>
                            <span className="duration">({(lyric.endTime - lyric.startTime).toFixed(1)}s)</span>
                        </div>
                    </>
                )}
            </div>

            {!isEditing && (
                <div className="line-actions">
                    <button
                        className="action-btn edit"
                        onClick={() => setIsEditing(true)}
                        title="Edit"
                    >
                        ✏️
                    </button>
                    <button
                        className="action-btn move-up"
                        onClick={() => onMoveUp(index)}
                        disabled={isFirst}
                        title="Move up"
                    >
                        ▲
                    </button>
                    <button
                        className="action-btn move-down"
                        onClick={() => onMoveDown(index)}
                        disabled={isLast}
                        title="Move down"
                    >
                        ▼
                    </button>
                    <button
                        className="action-btn set-start"
                        onClick={() => onSetStartTime(index, currentTimeRef.current)}
                        title="Set start to current time"
                    >
                        ⏱️S
                    </button>
                    <button
                        className="action-btn set-end"
                        onClick={() => onSetEndTime(index, currentTimeRef.current)}
                        title="Set end to current time"
                    >
                        ⏱️E
                    </button>
                    <button
                        className="action-btn delete"
                        onClick={() => onDelete(index)}
                        title="Delete"
                    >
                        🗑️
                    </button>
                </div>
            )}
        </div>
    );
};

export default memo(LyricLineEditor);
