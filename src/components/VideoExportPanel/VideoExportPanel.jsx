/**
 * VideoExportPanel Component
 * 
 * Provides UI for video export functionality with options for resolution,
 * codec, quality, and progress tracking.
 */

import { useState, useCallback, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useVideoExport, RenderPhase, isTauri } from '@/hooks/useVideoExport';
import './VideoExportPanel.css';

/**
 * Resolution presets
 */
const RESOLUTIONS = [
    { label: '4K (3840×2160)', width: 3840, height: 2160 },
    { label: '1080p (1920×1080)', width: 1920, height: 1080 },
    { label: '720p (1280×720)', width: 1280, height: 720 },
    { label: '480p (854×480)', width: 854, height: 480 },
];

/**
 * Codec options
 */
const CODECS = [
    { label: 'H.264 (Best Compatibility)', value: 'h264' },
    { label: 'H.265/HEVC (Better Compression)', value: 'h265' },
    { label: 'VP9 (WebM)', value: 'vp9' },
    { label: 'AV1 (Best Compression, Slow)', value: 'av1' },
];

/**
 * Quality presets (CRF values, lower = better quality)
 */
const QUALITY_PRESETS = [
    { label: 'Highest (CRF 18)', value: 18 },
    { label: 'High (CRF 20)', value: 20 },
    { label: 'Medium (CRF 23)', value: 23 },
    { label: 'Low (CRF 28)', value: 28 },
    { label: 'Lowest (CRF 35)', value: 35 },
];

/**
 * FPS options
 */
const FPS_OPTIONS = [
    { label: '60 fps', value: 60 },
    { label: '30 fps', value: 30 },
    { label: '24 fps', value: 24 },
];

export function VideoExportPanel({
    klyricDocument,
    audioPath = null,
    onClose,
    onComplete,
    isStandalone = false,
}) {
    // Export hook
    const {
        isRendering,
        progress,
        error,
        outputPath,
        isTauriAvailable,
        ffmpegAvailable,
        ffmpegVersion,
        startRender,
        cancelRender,
        formatEta,
        reset,
        downloadFfmpeg,
    } = useVideoExport();

    // Export options state
    const [resolution, setResolution] = useState(RESOLUTIONS[1]); // 1080p default
    const [codec, setCodec] = useState('h264');
    const [quality, setQuality] = useState(23);
    const [fps, setFps] = useState(30);
    const [useHwAccel, setUseHwAccel] = useState(false);
    const [previewMode, setPreviewMode] = useState(false);
    const [isDownloading, setIsDownloading] = useState(false);
    const [downloadError, setDownloadError] = useState(null);
    const [previewFrame, setPreviewFrame] = useState(null);
    const [enablePreview, setEnablePreview] = useState(true);

    // Listen for preview frames
    useEffect(() => {
        if (!isRendering) {
            setPreviewFrame(null);
            return;
        }

        let unlisten = null;

        const setupListener = async () => {
            try {
                unlisten = await listen('render-frame', (event) => {
                    console.log('Received frame, size:', event.payload?.length);
                    setPreviewFrame(event.payload);
                });
            } catch (err) {
                console.warn('Failed to setup frame listener', err);
            }
        };

        setupListener();

        return () => {
            if (unlisten) unlisten();
        };
    }, [isRendering]);

    /**
     * Handle export start
     */
    const handleExport = useCallback(async () => {
        if (!klyricDocument) {
            alert('No document to export');
            return;
        }

        // 10s preview mode
        const customDuration = previewMode ? 10.0 : null;

        const result = await startRender(klyricDocument, audioPath, {
            width: resolution.width,
            height: resolution.height,
            fps,
            quality,
            codec,
            useHwAccel,
            customDuration,
            enablePreview,
        });

        if (result && onComplete) {
            onComplete(result);
        }
    }, [klyricDocument, audioPath, resolution, fps, quality, codec, useHwAccel, previewMode, startRender, onComplete, enablePreview]);

    /**
     * Handle cancel
     */
    const handleCancel = useCallback(() => {
        if (isRendering) {
            cancelRender();
        } else {
            reset();
            onClose?.();
        }
    }, [isRendering, cancelRender, reset, onClose]);

    /**
     * Get phase label
     */
    const getPhaseLabel = () => {
        switch (progress.phase) {
            case RenderPhase.INITIALIZING:
                return '初始化编码器...';
            case RenderPhase.RENDERING:
                return `渲染中... ${progress.currentFrame}/${progress.totalFrames}`;
            case RenderPhase.MUXING:
                return '合并音频...';
            case RenderPhase.COMPLETE:
                return '✅ 导出完成!';
            case RenderPhase.CANCELLED:
                return '⚠️ 已取消';
            case RenderPhase.ERROR:
                return '❌ 导出失败';
            default:
                return '准备就绪';
        }
    };

    // Not available in browser
    if (!isTauriAvailable) {
        return (
            <div className="video-export-panel">
                <div className="export-header">
                    <h3>🎬 视频导出</h3>
                    <button className="close-btn" onClick={onClose}>✕</button>
                </div>
                <div className="export-unavailable">
                    <p>⚠️ 视频导出功能仅在桌面应用中可用。</p>
                    <p>请下载 Lyric Video Maker 桌面版以使用此功能。</p>
                </div>
            </div>
        );
    }

    // FFmpeg not available - offer auto-download
    if (ffmpegAvailable === false) {
        const handleDownloadFfmpeg = async () => {
            setIsDownloading(true);
            setDownloadError(null);
            try {
                await downloadFfmpeg();
            } catch (err) {
                setDownloadError(err.toString());
            } finally {
                setIsDownloading(false);
            }
        };

        return (
            <div className="video-export-panel">
                <div className="export-header">
                    <h3>🎬 视频导出</h3>
                    <button className="close-btn" onClick={onClose}>✕</button>
                </div>
                <div className="export-unavailable ffmpeg-missing">
                    <p>⚠️ 未检测到 FFmpeg</p>
                    <p>FFmpeg 是视频编码所必需的。您可以自动下载或手动安装。</p>

                    {isDownloading ? (
                        <div className="download-progress">
                            <div className="spinner"></div>
                            <p>正在下载 FFmpeg...</p>
                            <p className="download-note">这可能需要几分钟，请耐心等待</p>
                        </div>
                    ) : (
                        <>
                            <button
                                className="download-ffmpeg-btn"
                                onClick={handleDownloadFfmpeg}
                            >
                                ⬇️ 自动下载 FFmpeg (~80MB)
                            </button>

                            <div className="divider">
                                <span>或</span>
                            </div>

                            <a
                                href="https://ffmpeg.org/download.html"
                                target="_blank"
                                rel="noopener noreferrer"
                                className="ffmpeg-download-link"
                            >
                                手动安装 FFmpeg →
                            </a>
                        </>
                    )}

                    {downloadError && (
                        <div className="download-error">
                            <p>❌ 下载失败: {downloadError}</p>
                        </div>
                    )}
                </div>
            </div>
        );
    }

    return (
        <div className={`video-export-panel ${isStandalone ? 'standalone' : ''}`}>
            <div className="export-header">
                <h3>🎬 视频导出</h3>
                <button className="close-btn" onClick={onClose}>✕</button>
            </div>

            {/* FFmpeg version info */}
            {ffmpegVersion && (
                <div className="ffmpeg-info">
                    <span className="ffmpeg-badge">✓ FFmpeg</span>
                    <span className="ffmpeg-version">{ffmpegVersion.split(' ')[2]}</span>
                </div>
            )}

            {isRendering || progress.phase === RenderPhase.COMPLETE ? (
                <div className="export-progress">



                    {/* Live Preview */}
                    {isRendering && previewFrame && enablePreview && (
                        <div className="live-preview" style={{ marginBottom: '15px', textAlign: 'center' }}>
                            <img
                                src={`data:image/jpeg;base64,${previewFrame}`}
                                alt="Export Preview"
                                style={{
                                    maxWidth: '100%',
                                    maxHeight: '270px',
                                    borderRadius: '6px',
                                    boxShadow: '0 4px 12px rgba(0,0,0,0.3)',
                                    border: '1px solid rgba(255,255,255,0.1)'
                                }}
                            />
                        </div>
                    )}

                    <div className="progress-phase">{getPhaseLabel()}</div>

                    <div className="progress-bar-container">
                        <div
                            className="progress-bar"
                            style={{ width: `${progress.percentage}%` }}
                        />
                        <span className="progress-text">{progress.percentage.toFixed(1)}%</span>
                    </div>

                    <div className="progress-stats">
                        <div className="stat">
                            <span className="stat-label">帧</span>
                            <span className="stat-value">{progress.currentFrame} / {progress.totalFrames}</span>
                        </div>
                        <div className="stat">
                            <span className="stat-label">速度</span>
                            <span className="stat-value">{progress.renderFps.toFixed(1)} fps</span>
                        </div>
                        <div className="stat">
                            <span className="stat-label">剩余时间</span>
                            <span className="stat-value">{formatEta(progress.etaSeconds)}</span>
                        </div>
                    </div>

                    {progress.phase === RenderPhase.COMPLETE && outputPath && (
                        <div className="export-complete">
                            <p>文件已保存至:</p>
                            <code className="output-path">{outputPath}</code>
                        </div>
                    )}

                    <div className="progress-actions">
                        {isRendering ? (
                            <button className="cancel-btn" onClick={handleCancel}>
                                取消渲染
                            </button>
                        ) : (
                            <button className="done-btn" onClick={onClose}>
                                完成
                            </button>
                        )}
                    </div>
                </div>
            ) : (
                /* Export options form */
                <div className="export-options">
                    {/* Resolution */}
                    <div className="option-group">
                        <label>分辨率</label>
                        <select
                            value={`${resolution.width}x${resolution.height}`}
                            onChange={(e) => {
                                const [w, h] = e.target.value.split('x').map(Number);
                                setResolution({ width: w, height: h, label: e.target.options[e.target.selectedIndex].text });
                            }}
                        >
                            {RESOLUTIONS.map(r => (
                                <option key={r.label} value={`${r.width}x${r.height}`}>
                                    {r.label}
                                </option>
                            ))}
                        </select>
                    </div>

                    {/* FPS */}
                    <div className="option-group">
                        <label>帧率</label>
                        <select value={fps} onChange={(e) => setFps(Number(e.target.value))}>
                            {FPS_OPTIONS.map(f => (
                                <option key={f.value} value={f.value}>{f.label}</option>
                            ))}
                        </select>
                    </div>

                    {/* Codec */}
                    <div className="option-group">
                        <label>编码器</label>
                        <select value={codec} onChange={(e) => setCodec(e.target.value)}>
                            {CODECS.map(c => (
                                <option key={c.value} value={c.value}>{c.label}</option>
                            ))}
                        </select>
                    </div>

                    {/* Quality */}
                    <div className="option-group">
                        <label>质量</label>
                        <select value={quality} onChange={(e) => setQuality(Number(e.target.value))}>
                            {QUALITY_PRESETS.map(q => (
                                <option key={q.value} value={q.value}>{q.label}</option>
                            ))}
                        </select>
                    </div>

                    {/* Hardware acceleration */}
                    <div className="option-group checkbox-group">
                        <label>
                            <input
                                type="checkbox"
                                checked={useHwAccel}
                                onChange={(e) => setUseHwAccel(e.target.checked)}
                            />
                            使用硬件加速 (NVENC/VideoToolbox)
                        </label>
                    </div>

                    {/* Preview Mode */}
                    {/* Preview Mode */}
                    <div className="option-group checkbox-group">
                        <label>
                            <input
                                type="checkbox"
                                checked={previewMode}
                                onChange={(e) => setPreviewMode(e.target.checked)}
                            />
                            仅导出前 10 秒预览
                        </label>
                    </div>

                    {/* Enable Preview */}
                    <div className="option-group checkbox-group">
                        <label>
                            <input
                                type="checkbox"
                                checked={enablePreview}
                                onChange={(e) => setEnablePreview(e.target.checked)}
                            />
                            启用实时预览 (可能会影响导出速度)
                        </label>
                    </div>

                    {/* Audio info */}
                    {audioPath && (
                        <div className="audio-info">
                            <span className="audio-badge">🎵 包含音频</span>
                            <span className="audio-path">{audioPath.split(/[\\/]/).pop()}</span>
                        </div>
                    )}

                    {/* Error display */}
                    {error && (
                        <div className="export-error">
                            <p>❌ {error}</p>
                        </div>
                    )}

                    {/* Action buttons */}
                    <div className="export-actions">
                        <button className="cancel-btn" onClick={onClose}>
                            取消
                        </button>
                        <button
                            className="export-btn"
                            onClick={handleExport}
                            disabled={!klyricDocument}
                        >
                            🎬 开始导出
                        </button>
                    </div>
                </div>
            )}
        </div>
    );
}

export default VideoExportPanel;
