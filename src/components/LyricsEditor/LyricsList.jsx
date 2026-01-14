import { useRef, useEffect, memo } from 'react';
import LyricLineEditor from './LyricLineEditor';

const LyricsList = ({
    localLyrics,
    activeIndex,
    currentTimeRef,
    availableFonts,
    onUpdate,
    onDelete,
    onSetStartTime,
    onSetEndTime,
    onMoveUp,
    onMoveDown
}) => {
    const listRef = useRef(null);

    // Scroll to active line
    useEffect(() => {
        if (activeIndex >= 0 && listRef.current) {
            const activeElement = listRef.current.children[activeIndex];
            if (activeElement) {
                activeElement.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
            }
        }
    }, [activeIndex]);

    return (
        <div className="lyrics-list" ref={listRef}>
            {localLyrics.map((lyric, index) => (
                <LyricLineEditor
                    key={`${index}-${lyric.startTime}`}
                    lyric={lyric}
                    index={index}
                    isActive={index === activeIndex}
                    currentTimeRef={currentTimeRef}
                    onUpdate={onUpdate}
                    onDelete={onDelete}
                    onSetStartTime={onSetStartTime}
                    onSetEndTime={onSetEndTime}
                    onMoveUp={onMoveUp}
                    onMoveDown={onMoveDown}
                    isFirst={index === 0}
                    isLast={index === localLyrics.length - 1}
                    availableFonts={availableFonts}
                />
            ))}

            {localLyrics.length === 0 && (
                <div className="empty-state">
                    No lyrics. Click "Add Line" to start.
                </div>
            )}
        </div>
    );
};

export default memo(LyricsList);
