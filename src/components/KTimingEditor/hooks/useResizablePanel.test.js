import { renderHook, act } from '@testing-library/react';
import { useResizablePanel } from './useResizablePanel';
import { vi } from 'vitest';

describe('useResizablePanel', () => {
    beforeEach(() => {
        localStorage.clear();
        vi.clearAllMocks();
    });

    it('should initialize with default width', () => {
        const { result } = renderHook(() => useResizablePanel(300));
        expect(result.current.width).toBe(300);
    });

    it('should initialize with stored width if available', () => {
        localStorage.setItem('ui_ktiming_sidebar_width', '450');
        const { result } = renderHook(() => useResizablePanel(300));
        expect(result.current.width).toBe(450);
    });
});
