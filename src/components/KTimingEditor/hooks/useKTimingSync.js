import { useState, useRef, useCallback, useEffect, useMemo } from 'react';

/**
 * Hook to manage local state of the KTiming Editor and sync it with the global lyrics object.
 *
 * @param {Object} props
 * @param {number} props.lineIndex - Current line index
 * @param {Array} props.lyrics - Global lyrics array
 * @param {Function} props.onLyricsChange - Callback to update global lyrics
 */
export const useKTimingSync = ({
    lineIndex,
    lyrics,
    onLyricsChange
}) => {
    const prevLineIndexRef = useRef(lineIndex);
    const isInternalUpdate = useRef(false);

    const currentLyric = lyrics[lineIndex];

    const [syllables, setSyllables] = useState([]);
    const [charCustomizations, setCharCustomizations] = useState({});
    const [lineProperties, setLineProperties] = useState({});

    // Push updates to lyrics (Real-time sync)
    const pushUpdates = useCallback((syls, customs, lineProps = lineProperties) => {
        if (!currentLyric) return;

        isInternalUpdate.current = true;
        const updatedLyrics = [...lyrics];
        const updatedLine = {
            ...currentLyric,
            syllables: syls,
        };

        if (Object.keys(customs).length > 0) {
            updatedLine.charCustomizations = customs;
        } else {
            delete updatedLine.charCustomizations;
        }

        // Apply line-level properties (KLYRIC format)
        if (Object.keys(lineProps).length > 0) {
            if (lineProps.layout) updatedLine.layout = lineProps.layout;
            if (lineProps.transform) updatedLine.transform = lineProps.transform;
            if (lineProps.style) updatedLine.style = lineProps.style;
            if (lineProps.font) updatedLine.font = lineProps.font;
            if (lineProps.stroke) updatedLine.stroke = lineProps.stroke;
            if (lineProps.shadow) updatedLine.shadow = lineProps.shadow;
            if (lineProps.effect) updatedLine.effect = lineProps.effect;
            if (lineProps.animation) updatedLine.animation = lineProps.animation;
        }

        updatedLyrics[lineIndex] = updatedLine;
        onLyricsChange(updatedLyrics, `K-Timing Update`);
    }, [currentLyric, lyrics, lineIndex, onLyricsChange, lineProperties]);

    // Handle per-character customization changes with real-time sync
    const handleCharCustomizationChange = useCallback((charIndex, customization) => {
        const newCustoms = { ...charCustomizations, [charIndex]: customization };
        setCharCustomizations(newCustoms);
        pushUpdates(syllables, newCustoms, lineProperties);
    }, [charCustomizations, pushUpdates, syllables, lineProperties]);

    // Handle line-level property changes
    const handleLinePropertyChange = useCallback((property, value) => {
        const newProps = { ...lineProperties };

        // Handle nested properties (e.g., 'transform.x', 'layout.mode')
        const parts = property.split('.');
        if (parts.length === 2) {
            const [category, key] = parts;
            const categoryObj = { ...(newProps[category] || {}) };

            if (value === null || value === '' || value === undefined) {
                delete categoryObj[key];
            } else {
                categoryObj[key] = value;
            }

            if (Object.keys(categoryObj).length === 0) {
                delete newProps[category];
            } else {
                newProps[category] = categoryObj;
            }
        } else {
            if (value === null || value === '' || value === undefined) {
                delete newProps[property];
            } else {
                newProps[property] = value;
            }
        }

        setLineProperties(newProps);
        pushUpdates(syllables, charCustomizations, newProps);
    }, [lineProperties, pushUpdates, syllables, charCustomizations]);

    // Initialize state when line changes
    useEffect(() => {
        const lineChanged = lineIndex !== prevLineIndexRef.current;
        prevLineIndexRef.current = lineIndex;

        // Skip reload if it's an internal update and we're on the same line
        if (isInternalUpdate.current && !lineChanged) {
            isInternalUpdate.current = false;
            return;
        }
        isInternalUpdate.current = false;

        if (currentLyric?.syllables) {
            setSyllables([...currentLyric.syllables]);
        } else {
            setSyllables([]);
        }

        if (currentLyric?.charCustomizations) {
            setCharCustomizations({ ...currentLyric.charCustomizations });
        } else {
            setCharCustomizations({});
        }

        // Load line-level properties
        const props = {};
        if (currentLyric?.layout) props.layout = { ...currentLyric.layout };
        if (currentLyric?.transform) props.transform = { ...currentLyric.transform };
        if (currentLyric?.style) props.style = currentLyric.style;
        if (currentLyric?.font) props.font = { ...currentLyric.font };
        if (currentLyric?.stroke) props.stroke = { ...currentLyric.stroke };
        if (currentLyric?.shadow) props.shadow = { ...currentLyric.shadow };
        if (currentLyric?.effect) props.effect = currentLyric.effect;
        if (currentLyric?.animation) props.animation = currentLyric.animation;

        // Deep compare to avoid infinite update loops
        if (JSON.stringify(lineProperties) !== JSON.stringify(props)) {
            setLineProperties(props);
        }
    }, [lineIndex, currentLyric, lineProperties]);

    return {
        syllables,
        setSyllables,
        charCustomizations,
        setCharCustomizations,
        lineProperties,
        setLineProperties,
        pushUpdates,
        handleCharCustomizationChange,
        handleLinePropertyChange,
        isInternalUpdate
    };
};
