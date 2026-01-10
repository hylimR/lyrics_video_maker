/**
 * useVideoExport Hook
 * 
 * Provides video export functionality using Tauri's Rust backend.
 * Falls back to browser-based recording if running in web mode.
 */

import { useState, useCallback, useEffect } from 'react';

// Tauri API imports - will only work in Tauri context
let invoke, listen, save;
try {
    const tauriApi = await import('@tauri-apps/api/core');
    const tauriEvent = await import('@tauri-apps/api/event');
    const tauriDialog = await import('@tauri-apps/plugin-dialog');
    invoke = tauriApi.invoke;
    listen = tauriEvent.listen;
    save = tauriDialog.save;
} catch (e) {
    // Running in browser mode, Tauri not available
    console.log('Tauri API not available, running in browser mode');
}

/**
 * Check if we're running inside Tauri
 */
export const isTauri = () => {
    return typeof window !== 'undefined' && window.__TAURI_INTERNALS__ !== undefined;
};

/**
 * Render progress information
 */
export const RenderPhase = {
    IDLE: 'idle',
    INITIALIZING: 'initializing',
    RENDERING: 'rendering',
    MUXING: 'muxing',
    COMPLETE: 'complete',
    CANCELLED: 'cancelled',
    ERROR: 'error',
};

/**
 * Video export hook
 * 
 * @returns {Object} Export controls and state
 */
export function useVideoExport() {
    const [isRendering, setIsRendering] = useState(false);
    const [progress, setProgress] = useState({
        currentFrame: 0,
        totalFrames: 0,
        percentage: 0,
        etaSeconds: 0,
        phase: RenderPhase.IDLE,
        renderFps: 0,
    });
    const [error, setError] = useState(null);
    const [outputPath, setOutputPath] = useState(null);
    const [ffmpegVersion, setFfmpegVersion] = useState(null);
    const [ffmpegAvailable, setFfmpegAvailable] = useState(null);

    // Check FFmpeg availability on mount
    useEffect(() => {
        if (!isTauri() || !invoke) return;

        invoke('check_ffmpeg')
            .then(version => {
                setFfmpegVersion(version);
                setFfmpegAvailable(true);
            })
            .catch(() => {
                setFfmpegAvailable(false);
            });
    }, []);

    // Listen for progress events from Rust backend
    useEffect(() => {
        if (!isTauri() || !listen) return;

        const unlisten = listen('render-progress', (event) => {
            const data = event.payload;
            let phase = RenderPhase.RENDERING;

            if (data.phase === 'Complete') {
                phase = RenderPhase.COMPLETE;
            } else if (data.phase === 'Initializing encoder') {
                phase = RenderPhase.INITIALIZING;
            } else if (data.phase === 'Adding audio') {
                phase = RenderPhase.MUXING;
            } else if (data.phase === 'Finalizing video') {
                phase = RenderPhase.MUXING;
            }

            setProgress({
                currentFrame: data.currentFrame,
                totalFrames: data.totalFrames,
                percentage: data.percentage,
                etaSeconds: data.etaSeconds,
                phase,
                renderFps: data.renderFps || 0,
            });
        });

        return () => {
            unlisten.then(fn => fn());
        };
    }, []);

    /**
     * Start video rendering
     * 
     * @param {Object} klyricDocument - The KLYRIC document to render
     * @param {string} audioPath - Path to audio file (optional)
     * @param {Object} options - Render options
     */
    const startRender = useCallback(async (klyricDocument, audioPath = null, options = {}) => {
        if (!isTauri()) {
            setError('Video export requires the desktop app. Please download Lyric Video Maker.');
            return null;
        }

        if (!ffmpegAvailable) {
            setError('FFmpeg not found. Please install FFmpeg and ensure it is in your PATH.');
            return null;
        }

        setIsRendering(true);
        setError(null);
        setOutputPath(null);
        setProgress({
            currentFrame: 0,
            totalFrames: 0,
            percentage: 0,
            etaSeconds: 0,
            phase: RenderPhase.INITIALIZING,
            renderFps: 0,
        });

        try {
            // Determine file extension based on codec
            const codec = options.codec || 'h264';
            const extension = codec === 'vp9' || codec === 'av1' ? 'webm' : 'mp4';

            // Open save dialog
            const filePath = await save({
                filters: [
                    { name: 'MP4 Video', extensions: ['mp4'] },
                    { name: 'WebM Video', extensions: ['webm'] },
                ],
                defaultPath: `lyric_video_${Date.now()}.${extension}`,
            });

            if (!filePath) {
                setIsRendering(false);
                setProgress(prev => ({ ...prev, phase: RenderPhase.CANCELLED }));
                return null;
            }

            // Prepare render options
            const renderOptions = {
                width: options.width || 1920,
                height: options.height || 1080,
                fps: options.fps || 30,
                quality: options.quality || 23,
                codec: codec,
                audioOffset: options.audioOffset || 0,
                audioOffset: options.audioOffset || 0,
                useHwAccel: options.useHwAccel || false,
                customDuration: options.customDuration || null,
                enablePreview: options.enablePreview !== undefined ? options.enablePreview : true,
            };

            // Call Rust backend
            const result = await invoke('render_video', {
                klyricJson: JSON.stringify(klyricDocument),
                audioPath: audioPath,
                outputPath: filePath,
                options: renderOptions,
            });

            setOutputPath(result.outputPath);
            setProgress(prev => ({
                ...prev,
                phase: RenderPhase.COMPLETE,
                percentage: 100,
                renderFps: result.avgFps,
            }));
            setIsRendering(false);

            return result;
        } catch (err) {
            console.error('Render error:', err);
            setError(err.toString());
            setProgress(prev => ({ ...prev, phase: RenderPhase.ERROR }));
            setIsRendering(false);
            return null;
        }
    }, [ffmpegAvailable]);

    /**
     * Cancel ongoing render
     */
    const cancelRender = useCallback(async () => {
        if (!isTauri() || !isRendering) return;

        try {
            await invoke('cancel_render');
            setProgress(prev => ({ ...prev, phase: RenderPhase.CANCELLED }));
            setIsRendering(false);
        } catch (err) {
            console.error('Cancel error:', err);
        }
    }, [isRendering]);

    /**
     * Export a single frame as an image
     * 
     * @param {Object} klyricDocument - The KLYRIC document
     * @param {number} timestamp - Time in seconds
     * @param {Object} options - Export options (width, height)
     */
    const exportFrame = useCallback(async (klyricDocument, timestamp, options = {}) => {
        if (!isTauri()) {
            setError('Frame export requires the desktop app.');
            return null;
        }

        try {
            const filePath = await save({
                filters: [
                    { name: 'PNG Image', extensions: ['png'] },
                    { name: 'JPEG Image', extensions: ['jpg', 'jpeg'] },
                ],
                defaultPath: `frame_${timestamp.toFixed(2)}s.png`,
            });

            if (!filePath) return null;

            const result = await invoke('export_frame', {
                klyricJson: JSON.stringify(klyricDocument),
                timestamp,
                outputPath: filePath,
                width: options.width || 1920,
                height: options.height || 1080,
            });

            return result;
        } catch (err) {
            console.error('Export frame error:', err);
            setError(err.toString());
            return null;
        }
    }, []);

    /**
     * Get list of system fonts
     */
    const getSystemFonts = useCallback(async () => {
        if (!isTauri()) {
            return [];
        }

        try {
            const fonts = await invoke('get_system_fonts');
            return fonts;
        } catch (err) {
            console.error('Get fonts error:', err);
            return [];
        }
    }, []);

    /**
     * Reset export state
     */
    const reset = useCallback(() => {
        setIsRendering(false);
        setProgress({
            currentFrame: 0,
            totalFrames: 0,
            percentage: 0,
            etaSeconds: 0,
            phase: RenderPhase.IDLE,
            renderFps: 0,
        });
        setError(null);
        setOutputPath(null);
    }, []);

    /**
     * Format ETA as human-readable string
     */
    const formatEta = useCallback((seconds) => {
        if (!seconds || seconds <= 0) return '--:--';
        const mins = Math.floor(seconds / 60);
        const secs = Math.floor(seconds % 60);
        return `${mins}:${secs.toString().padStart(2, '0')}`;
    }, []);

    /**
     * Download FFmpeg using ffmpeg-sidecar
     */
    const downloadFfmpeg = useCallback(async () => {
        if (!isTauri()) {
            setError('FFmpeg download requires the desktop app.');
            return null;
        }

        try {
            const result = await invoke('download_ffmpeg');
            // Re-check FFmpeg availability after download
            const version = await invoke('check_ffmpeg');
            setFfmpegVersion(version);
            setFfmpegAvailable(true);
            return result;
        } catch (err) {
            console.error('FFmpeg download error:', err);
            setError(err.toString());
            return null;
        }
    }, []);

    /**
     * Check if FFmpeg is already downloaded via sidecar
     */
    const checkFfmpegDownloaded = useCallback(async () => {
        if (!isTauri()) return false;
        try {
            return await invoke('check_ffmpeg_downloaded');
        } catch {
            return false;
        }
    }, []);

    return {
        // State
        isRendering,
        progress,
        error,
        outputPath,
        isTauriAvailable: isTauri(),
        ffmpegAvailable,
        ffmpegVersion,

        // Actions
        startRender,
        cancelRender,
        exportFrame,
        getSystemFonts,
        reset,
        formatEta,
        downloadFfmpeg,
        checkFfmpegDownloaded,
    };
}

export default useVideoExport;
