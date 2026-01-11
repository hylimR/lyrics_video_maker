import { useRef, useState, useCallback, useEffect } from 'react';
import { useAppStore, initStore } from '@/store/useAppStore';
import WasmPreview from '@/components/WasmPreview';
import ControlPanel from '@/components/ControlPanel';
import FileUploader from '@/components/FileUploader';
import LyricsEditor from '@/components/LyricsEditor';
import KTimingEditor from '@/components/KTimingEditor';
import ExportPanel from '@/components/ExportPanel';
import PreferencesModal from '@/components/PreferencesModal';
import GlobalStyleEditor from '@/components/GlobalStyleEditor';
import AudioPlayer from '@/components/AudioPlayer';

import { importSubtitleToKLyric } from '@/utils/KLyricFormat';
import './App.css';

/**
 * App.jsx - Main Application Layout
 * 
 * Orchestrator using Zustand for centralized state management.
 * Integrated K-Timing Editor layout with compact preview.
 */
function App() {
  // --- Global Store ---
  const {
    lyrics,
    resolution,
    selectedEffect,
    selectedFont,
    globalStyle,
    availableFonts,
    duration,
    currentTime,
    isPlaying,
    updateState,
    setPlayback,
    setAvailableFonts,
    undo,
    redo,
    past,
    future
  } = useAppStore();

  // Initialize Store Logic (Networking - P2P)
  useEffect(() => {
    const cleanup = initStore();
    return cleanup;
  }, []);

  // --- Font Loading ---
  useEffect(() => {
    const loadFonts = async () => {
      try {
        if (window.__TAURI_INTERNALS__) {
          const { invoke } = await import('@tauri-apps/api/core');
          const fonts = await invoke('get_system_fonts');
          setAvailableFonts(fonts);
          console.log('Fonts loaded:', fonts.length);
        } else {
          // Browser mock
          setAvailableFonts([
            { name: 'Noto Sans SC', path: '', style: 'Regular' },
            { name: 'Arial', path: '', style: 'Regular' }
          ]);
        }
      } catch (e) {
        console.error('Failed to load fonts:', e);
      }
    };
    loadFonts();
  }, [setAvailableFonts]);

  // --- Audio System ---
  // Replaced manual audio ref with AudioPlayer ref
  const audioRef = useRef(null);

  const [demoLoaded, setDemoLoaded] = useState(false);
  const [audioSource, setAudioSource] = useState(null);

  // --- UI State ---
  const [showFilePanel, setShowFilePanel] = useState(false);
  const [showEditor, setShowEditor] = useState(false);
  const [showExportPanel, setShowExportPanel] = useState(false);
  const [showPreferences, setShowPreferences] = useState(false);
  const [isPreviewOpen, setIsPreviewOpen] = useState(false);

  // --- Preview Window State Management ---
  useEffect(() => {
    let unlistenFn;

    const initPreviewState = async () => {
      if (!window.__TAURI_INTERNALS__) return;

      try {
        const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow');
        const { listen } = await import('@tauri-apps/api/event');

        // Check if exists
        const win = await WebviewWindow.getByLabel('preview-window');
        if (win) setIsPreviewOpen(true);

        // Listen for close signal from the window
        unlistenFn = await listen('preview-closed', () => {
          setIsPreviewOpen(false);
        });

      } catch (e) {
        console.error('Preview init error:', e);
      }
    };

    initPreviewState();

    return () => {
      if (unlistenFn) unlistenFn();
    };
  }, []);

  const togglePreview = useCallback(async () => {
    if (!window.__TAURI_INTERNALS__) {
      window.open('/preview', '_blank');
      return;
    }

    try {
      const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow');
      const label = 'preview-window';
      const existing = await WebviewWindow.getByLabel(label);

      if (existing) {
        await existing.close();
        setIsPreviewOpen(false);
      } else {
        new WebviewWindow(label, {
          url: '/preview',
          title: 'Full Preview - Lyric Video',
          width: 1280,
          height: 720,
        });
        setIsPreviewOpen(true);
      }
    } catch (e) {
      console.error('Toggle preview failed:', e);
    }
  }, []);

  // --- Sync Effects (P2P Model) ---

  const [isAudioMaster, setIsAudioMaster] = useState(false);

  // --- Audio Master Election ---
  useEffect(() => {
    if (!navigator.locks) {
      setIsAudioMaster(true); // Fallback for environments without locks
      return;
    }

    const requestLock = async () => {
      // Request a lock held as long as the component is mounted
      navigator.locks.request('lyric-app-audio-master', async (lock) => {
        console.log('🎤 Acquired Audio Master Lock');
        setIsAudioMaster(true);
        // Hold the lock indefinitely until unmount (signal abort/promise resolve)
        await new Promise(() => { });
      });
    };

    requestLock();
  }, []);


  // --- Sync Effects (P2P Model) ---

  // 1. Sync Audio Element with Store Time (Incoming Sync)
  useEffect(() => {
    if (audioRef.current) {
      const diff = Math.abs(audioRef.current.currentTime - currentTime);

      // Only seek if difference is significant
      if (diff > 0.3) {
        audioRef.current.seek(currentTime);
      }

      // Handle Play/Pause logic
      if (isAudioMaster) {
        if (isPlaying && (audioRef.current.paused || audioRef.current.ended)) {
          // AudioPlayer exposed .play() via useImperativeHandle
          audioRef.current.play();
        } else if (!isPlaying && !audioRef.current.paused) {
          audioRef.current.pause();
        }
      } else {
        // Slave: Ensure we are paused to prevent sound, rely on visual sync
        if (!audioRef.current.paused) audioRef.current.pause();
      }
    }
  }, [currentTime, isPlaying, audioRef, isAudioMaster]);

  // 2. Broadcast High-Frequency Time (Outgoing Sync)
  // ONLY Master broadcasts time.
  useEffect(() => {
    if (!isPlaying || !isAudioMaster) return;

    const interval = setInterval(() => {
      if (audioRef.current) {
        setPlayback({
          currentTime: audioRef.current.currentTime,
          isPlaying: true
        });
      }
    }, 100);

    return () => clearInterval(interval);
  }, [isPlaying, audioRef, setPlayback, isAudioMaster]);




  // --- Event Handlers ---

  const handlePlay = useCallback(() => {
    // Simply update state; useEffect will handle the imperative play()
    setPlayback({ isPlaying: true });
  }, [setPlayback]);

  const handlePause = useCallback(() => {
    setPlayback({ isPlaying: false });
  }, [setPlayback]);

  const handleSeek = useCallback((time, shouldPlay = false) => {
    // Only update state; useEffect will handle the seek and play state
    if (shouldPlay) {
      setPlayback({ currentTime: time, isPlaying: true });
    } else {
      setPlayback({ currentTime: time });
    }
  }, [setPlayback]);

  // Handle Updates from Control Panel
  const handleResolutionChange = useCallback((newRes) => {
    updateState({ resolution: newRes }, 'Change Resolution');
  }, [updateState]);

  const handleEffectChange = useCallback((newEffect) => {
    updateState({ selectedEffect: newEffect }, 'Change Effect');
  }, [updateState]);

  const handleFontChange = useCallback((newFontName, isPreview = false) => {
    const font = availableFonts.find(f => f.name === newFontName);
    if (font) {
      updateState({ selectedFont: font }, 'Change Font', { skipHistory: isPreview });
    } else if (newFontName === '') {
      updateState({ selectedFont: null }, 'Reset Font', { skipHistory: isPreview });
    }
  }, [updateState, availableFonts]);

  // Initialization: Load Demo on Start
  useEffect(() => {
    if (!demoLoaded) {
      const loadDemoContent = async () => {
        try {
          const response = await fetch('/sample_karaoke.ass');
          if (response.ok) {
            const content = await response.text();
            // Convert to KLyric format
            const { klyric, legacy } = importSubtitleToKLyric(content, 'sample_karaoke.ass');
            if (legacy.length > 0) {
              const maxEnd = Math.max(...legacy.map(l => l.endTime));
              console.log('🎬 [App] Loaded Demo. Legacy items:', legacy.length, 'MaxEnd:', maxEnd, 'First Item:', legacy[0]);

              if (lyrics.length === 0) {
                updateState({ lyrics: legacy, klyricDoc: klyric, duration: maxEnd + 2 });
              }
              console.log('🎬 Demo ASS loaded (converted to KLyric)', klyric.meta);
            }
          }
        } catch (e) { console.warn('Demo load error', e); }

        try {
          const audioFiles = ['/wav_泡沫.wav', '/sample.wav'];
          for (const f of audioFiles) {
            try {
              const res = await fetch(f, { method: 'HEAD' });
              if (res.ok) {
                setAudioSource(f);
                console.log('🎵 Demo audio found:', f);
                break;
              }
            } catch { /* ignore */ }
          }
        } catch { /* ignore */ }

        setDemoLoaded(true);
      };

      loadDemoContent();
    }
  }, [demoLoaded, updateState, lyrics.length]);


  // File Handlers
  // Handle lyrics loaded from FileUploader (receives legacy format, metadata, and KLyric doc)
  const handleLyricsLoaded = useCallback((newLyrics, metadata, klyricDoc) => {
    const maxEnd = Math.max(...newLyrics.map(l => l.endTime));
    updateState({
      lyrics: newLyrics,
      klyricDoc: klyricDoc || null,  // Store the KLyric document in global state
      duration: maxEnd + 2,
      currentTime: 0
    });

    // Log KLyric conversion info if available
    if (klyricDoc) {
      console.log('📄 KLyric document stored:', {
        title: klyricDoc.meta?.title,
        lines: klyricDoc.lines?.length,
        version: klyricDoc.version
      });
    }
  }, [updateState]);

  const handleAudioLoaded = useCallback((url, fname) => {
    setAudioSource(url);
    updateState({ currentTime: 0 });
    console.log('🎵 User loaded audio:', fname);
  }, [updateState]);

  const handleLyricsEdited = useCallback((newLyrics) => {
    updateState({ lyrics: newLyrics });
  }, [updateState]);


  // Panel Toggles
  const toggleFilePanel = () => { setShowFilePanel(p => !p); setShowEditor(false); setShowExportPanel(false); };
  const toggleEditor = () => { setShowEditor(p => !p); setShowFilePanel(false); setShowExportPanel(false); };
  const toggleExportPanel = () => { setShowExportPanel(p => !p); setShowFilePanel(false); setShowEditor(false); };


  const handleDurationChange = useCallback((d) => {
    updateState({ duration: d }, 'Update Duration', { skipHistory: true });
  }, [updateState]);

  const handleEnded = useCallback(() => {
    setPlayback({ isPlaying: false });
  }, [setPlayback]);

  return (
    <div className="app">
      {/* Audio Element (Wrapped in AudioPlayer) */}
      <AudioPlayer
        ref={audioRef}
        src={audioSource || undefined}
        onDurationChange={handleDurationChange}
        onPlay={handlePlay}
        onPause={handlePause}
        onEnded={handleEnded}
      />

      {/* Top Navigation Bar */}
      <header className="navbar">
        {/* Left: Logo & Status */}
        <div className="navbar-left">
          <div className="logo">
            <span className="logo-icon">🎬</span>
            <span className="logo-text">LyricVid</span>
          </div>
          <div className="divider" />

          <button
            className={`nav-icon-btn ${past.length === 0 ? 'disabled' : ''}`}
            onClick={undo}
            disabled={past.length === 0}
            title="Undo (Ctrl+Z)"
          >
            ↩️
          </button>
          <button
            className={`nav-icon-btn ${future.length === 0 ? 'disabled' : ''}`}
            onClick={redo}
            disabled={future.length === 0}
            title="Redo (Ctrl+Shift+Z)"
          >
            ↪️
          </button>
        </div>

        {/* Center: Title/Status */}
        <div className="navbar-center opacity-50 text-sm font-medium tracking-widest">
          LYRIC VIDEO MAKER
        </div>

        {/* Right: Actions */}
        <div className="navbar-right">
          <button
            className={`nav-btn ${isPreviewOpen ? 'active' : 'primary'}`}
            onClick={togglePreview}
          >
            <span className="btn-icon">🖥️</span>
            <span className="btn-text">Full Preview</span>
          </button>
          <div className="divider" />
          <button
            className={`nav-btn ${showEditor ? 'active' : ''}`}
            onClick={toggleEditor}
          >
            <span className="btn-icon">✏️</span>
            <span className="btn-text">Editor</span>
          </button>
          <button
            className={`nav-btn ${showFilePanel ? 'active' : ''}`}
            onClick={toggleFilePanel}
          >
            <span className="btn-icon">📁</span>
            <span className="btn-text">Import</span>
          </button>
          <button
            className={`nav-btn ${showExportPanel ? 'active' : ''}`}
            onClick={toggleExportPanel}
          >
            <span className="btn-text">Export</span>
          </button>
          <div className="divider" />
          <button
            className={`nav-btn ${showPreferences ? 'active' : ''}`}
            onClick={() => setShowPreferences(true)}
            title="Settings"
          >
            <span className="btn-icon">⚙️</span>
          </button>
        </div>
      </header>

      {/* File Panel (Overlay) */}
      {showFilePanel && (
        <div className="overlay-panel file-panel">
          <div className="panel-header">
            <h4>📁 Import Files</h4>
            <button className="close-btn" onClick={toggleFilePanel}>✕</button>
          </div>
          <div className="panel-content">
            <FileUploader
              onLyricsLoaded={handleLyricsLoaded}
              onAudioLoaded={handleAudioLoaded}
              currentLyricsCount={lyrics.length}
            />
          </div>
        </div>
      )}

      {/* Lyrics Editor (Overlay) */}
      {showEditor && (
        <LyricsEditor
          lyrics={lyrics}
          currentTime={currentTime}
          onLyricsChange={handleLyricsEdited}
          onClose={toggleEditor}
          availableFonts={availableFonts}
        />
      )}

      {/* Export Panel (Overlay) */}
      {showExportPanel && (
        <ExportPanel onClose={toggleExportPanel} />
      )}

      {/* Preferences Modal */}
      <PreferencesModal
        open={showPreferences}
        onOpenChange={setShowPreferences}
      />

      {/* Main Content: Split Layout */}
      {/* Resizable Preview Panel Logic */}

      <MainLayout
        resolution={resolution}
        klyricDoc={useAppStore.getState().klyricDoc}
        lyrics={lyrics}
        currentTime={currentTime}
        selectedFont={selectedFont}
        availableFonts={availableFonts}
        globalStyle={globalStyle}
        isPlaying={isPlaying}
        audioSource={audioSource}
        handleLyricsEdited={handleLyricsEdited}
        handleSeek={handleSeek}
        handlePlay={handlePlay}
        handlePause={handlePause}
        undo={undo}
        redo={redo}
        past={past}
        future={future}
        toggleFilePanel={toggleFilePanel}
      />

      {/* Bottom Player Bar */}
      <ControlPanel
        isPlaying={isPlaying}
        currentTime={currentTime}
        duration={duration}
        onPlay={handlePlay}
        onPause={handlePause}
        onSeek={handleSeek}
      />

      {/* Master Offline Overlay - Removed as per user request */}
    </div>
  );
}

// Sub-component for Main Layout to keep App.jsx clean
const MainLayout = ({
  resolution, klyricDoc, lyrics, currentTime, selectedFont, availableFonts, globalStyle,
  isPlaying, audioSource, handleLyricsEdited, handleSeek, handlePlay, handlePause,
  undo, redo, past, future, toggleFilePanel
}) => {
  const [previewWidth, setPreviewWidth] = useState(() => {
    const saved = localStorage.getItem('ui_preview_panel_width');
    return saved ? parseFloat(saved) : 45;
  }); // Percentage
  const isResizingRef = useRef(false);
  const containerRef = useRef(null);
  const currentWidthRef = useRef(previewWidth); // Ref to track width for event listener

  // Keep ref in sync
  useEffect(() => {
    currentWidthRef.current = previewWidth;
  }, [previewWidth]);

  const handleMouseDown = useCallback((e) => {
    isResizingRef.current = true;
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
    e.preventDefault();
  }, []);

  useEffect(() => {
    const handleMouseMove = (e) => {
      if (!isResizingRef.current || !containerRef.current) return;
      const containerRect = containerRef.current.getBoundingClientRect();
      const newWidth = ((e.clientX - containerRect.left) / containerRect.width) * 100;
      setPreviewWidth(Math.max(20, Math.min(80, newWidth)));
    };

    const handleMouseUp = () => {
      if (isResizingRef.current) {
        isResizingRef.current = false;
        document.body.style.cursor = '';
        document.body.style.userSelect = '';
        // Save to local storage on drag end
        localStorage.setItem('ui_preview_panel_width', currentWidthRef.current.toString());
      }
    };

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };
  }, []);

  return (
    <main className="main-content" ref={containerRef}>
      {/* Left Section: Preview */}
      <section
        className="preview-section"
        style={{ width: `${previewWidth}%`, maxWidth: 'none' }}
      >
        <div className="preview-frame">
          <div
            className="preview-canvas"
            style={{ aspectRatio: `${resolution.width} / ${resolution.height}` }}
          >
            <WasmPreview
              key={`wasm-${resolution.width}x${resolution.height}`}
              width={resolution.width}
              height={resolution.height}
              klyricDoc={klyricDoc}
              lyrics={lyrics}
              currentTime={currentTime}
              selectedFont={selectedFont}
              availableFonts={availableFonts}
              globalStyle={globalStyle}
            />
          </div>
          <div className="preview-badges">
            <span className="badge resolution">{resolution.width}×{resolution.height}</span>
          </div>
        </div>

        {/* Global Style Editor */}
        <div className="global-editor-container">
          <GlobalStyleEditor availableFonts={availableFonts} />
        </div>
      </section>

      {/* Draggable Divider */}
      <div
        className="layout-divider"
        onMouseDown={handleMouseDown}
      />

      {/* Right Section: K-Timing Editor */}
      <section className="editor-section">
        {lyrics.length > 0 ? (
          <KTimingEditor
            lyrics={lyrics}
            currentTime={currentTime}
            isPlaying={isPlaying}
            audioSource={audioSource}
            hasRealAudio={Boolean(audioSource)}
            resolution={resolution}
            onLyricsChange={handleLyricsEdited}
            onSeek={handleSeek}
            onPlay={handlePlay}
            onPause={handlePause}
            onClose={() => { }}
            onUndo={undo}
            onRedo={redo}
            canUndo={past.length > 0}
            canRedo={future.length > 0}
            availableFonts={availableFonts}
          />
        ) : (
          <div className="editor-placeholder">
            <div className="placeholder-icon">🎹</div>
            <h3>K-Timing Editor</h3>
            <p>Load lyrics to start editing syllable timing</p>
            <button className="nav-btn primary" onClick={toggleFilePanel}>
              <span className="btn-icon">📁</span>
              <span className="btn-text">Import Lyrics</span>
            </button>
          </div>
        )}
      </section>
    </main>
  );
};

export default App;

