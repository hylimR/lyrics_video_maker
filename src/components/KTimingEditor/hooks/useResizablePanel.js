import { useState, useRef, useEffect, useCallback } from 'react';

export const useResizablePanel = (initialWidth = 300, minWidth = 200, storageKey = 'ui_ktiming_sidebar_width') => {
    const [width, setWidth] = useState(() => {
        const saved = localStorage.getItem(storageKey);
        return saved ? parseFloat(saved) : initialWidth;
    });

    const isResizingRef = useRef(false);
    const currentWidthRef = useRef(width);
    const containerRef = useRef(null);

    // Keep ref in sync
    useEffect(() => {
        currentWidthRef.current = width;
    }, [width]);

    const handleMouseDown = useCallback((e) => {
        isResizingRef.current = true;
        document.body.style.cursor = 'col-resize';
        document.body.style.userSelect = 'none'; // Prevent text selection
        e.preventDefault();
    }, []);

    useEffect(() => {
        const handleMouseMove = (e) => {
            if (!isResizingRef.current || !containerRef.current) return;

            // Calculate new width relative to container right edge
            const containerRect = containerRef.current.getBoundingClientRect();
            const newWidth = containerRect.right - e.clientX;

            // Clamp width (min 200px, max 50% of container)
            const maxWidth = containerRect.width / 2;
            const clampedWidth = Math.max(minWidth, Math.min(maxWidth, newWidth));

            setWidth(clampedWidth);
        };

        const handleMouseUp = () => {
            if (isResizingRef.current) {
                isResizingRef.current = false;
                document.body.style.cursor = '';
                document.body.style.userSelect = '';
                // Save to local storage on drag end
                localStorage.setItem(storageKey, currentWidthRef.current.toString());
            }
        };

        window.addEventListener('mousemove', handleMouseMove);
        window.addEventListener('mouseup', handleMouseUp);
        return () => {
            window.removeEventListener('mousemove', handleMouseMove);
            window.removeEventListener('mouseup', handleMouseUp);
        };
    }, [minWidth, storageKey]);

    return {
        width,
        containerRef,
        handleMouseDown
    };
};
