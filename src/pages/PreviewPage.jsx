import { useEffect } from 'react';
import WasmPreview from '../components/WasmPreview';
import { useAppStore, initStore } from '../store/useAppStore';
import './PreviewPage.css';

/**
 * PreviewPage - Full screen preview that syncs with master
 * 
 * Opens in a new tab and receives real-time updates from the
 * centralized Zustand store (peer-connected).
 */
const PreviewPage = () => {
    // Global Store
    const {
        lyrics,
        resolution,
        currentTime,
        isPlaying
    } = useAppStore();

    // Initialize Store (Networking)
    useEffect(() => {
        const cleanup = initStore();
        // Force document title
        document.title = '🎬 Full Preview - Lyric Video';
        return () => {
            cleanup();
            if (window.__TAURI_INTERNALS__) {
                import('@tauri-apps/api/event').then(({ emit }) => {
                    emit('preview-closed');
                }).catch(e => console.error('Failed to emit cleanup event', e));
            }
        };
    }, []);

    // KLyric document from store
    const klyricDoc = useAppStore.getState().klyricDoc;

    return (
        <div className="preview-page">
            {/* Debug Overlay */}
            <div style={{
                position: 'fixed', bottom: 10, right: 10, zIndex: 9999,
                background: 'rgba(0,0,0,0.7)', color: '#0f0', padding: '10px',
                fontSize: '12px', pointerEvents: 'none', fontFamily: 'monospace'
            }}>
                DEBUG: Lyrics: {lyrics.length} | Res: {resolution.width}x{resolution.height} |
                Time: {currentTime.toFixed(2)} | Playing: {String(isPlaying)} |
                WASM Doc: {klyricDoc ? 'Yes' : 'No'}
            </div>

            {/* Status bar */}
            <div className="preview-status-bar">
                <span className="connection-status connected">
                    🟢 Synced
                </span>
                <span className="resolution-display">
                    📺 {resolution.width}×{resolution.height}
                </span>
                <span className="time-display">
                    ⏱️ {currentTime.toFixed(1)}s
                </span>
                <span className="playback-status">
                    {isPlaying ? '▶️ Playing' : '⏸️ Paused'}
                </span>
            </div>

            {/* Full screen canvas */}
            {lyrics.length > 0 ? (
                <WasmPreview
                    key={`preview-${resolution.width}x${resolution.height}`}
                    width={resolution.width}
                    height={resolution.height}
                    klyricDoc={klyricDoc}
                    lyrics={lyrics}
                    currentTime={currentTime}
                />
            ) : (
                <div className="preview-waiting">
                    <h2>🎬 Full Preview Mode</h2>
                    <p>Waiting for lyrics data...</p>
                    <p className="hint">Make sure the main editor has lyrics loaded.</p>
                </div>
            )}
        </div>
    );
};

export default PreviewPage;
