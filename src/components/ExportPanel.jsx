import { useCallback, useState } from 'react';
import { serializeKLyric, klyricToASS, lyricsToKLyric } from '@/utils/KLyricFormat';
import { sanitizeKLyricDoc } from '@/utils/KLyricSanitizer';
import { useAppStore } from '@/store/useAppStore';
import { useVideoExport } from '@/hooks/useVideoExport';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

import ExportProgressDialog from './Export/ExportProgressDialog';
import ExportForm from './Export/ExportForm';

/**
 * ExportPanel.jsx - Export Lyrics to KLyric, ASS, or Video (MP4)
 */
const ExportPanel = ({ onClose }) => {
    const { lyrics, klyricDoc, resolution, duration, audioSource } = useAppStore();
    const [exportFormat, setExportFormat] = useState('klyric');
    const [filename, setFilename] = useState('lyrics');
    const [pretty, setPretty] = useState(true);
    const [exportStatus, setExportStatus] = useState(null);

    // Video export hook
    const {
        isTauriAvailable,
        isRendering,
        progress,
        cancelRender,
    } = useVideoExport();

    // Get or generate KLyric document
    const getKLyricDoc = useCallback(() => {
        if (klyricDoc) {
            return klyricDoc; // Use existing KLYRIC doc if available
        }
        // Generate from legacy lyrics if klyricDoc not available
        return lyricsToKLyric(lyrics, {}, { resolution, duration });
    }, [klyricDoc, lyrics, resolution, duration]);

    // Sanitize and convert KLyric doc for Rust backend via centralized sanitizer
    const getSanitizedDoc = useCallback((doc) => {
        return sanitizeKLyricDoc(doc, {
            duration,
            resolution,
            title: doc?.meta?.title || 'Untitled',
            artist: doc?.meta?.artist || ''
        });
    }, [duration, resolution]);

    const handleExport = useCallback(async () => {
        try {
            const doc = getKLyricDoc();

            // Video Export - open new window
            if (exportFormat === 'video') {
                if (!isTauriAvailable) {
                    alert('Video export is only available in the desktop app.');
                    return;
                }

                // Store data for the new window
                const exportPayload = {
                    klyricDocument: getSanitizedDoc(doc),
                    audioPath: audioSource
                };
                localStorage.setItem('pending_export_data', JSON.stringify(exportPayload));

                // Open Export Window
                try {
                    console.log('Attempting to create export window...');
                    const webview = new WebviewWindow('export-window', {
                        url: '/export',
                        title: 'Export Video',
                        width: 1000,
                        height: 800,
                        resizable: true,
                        center: true,
                    });

                    webview.once('tauri://created', function () {
                        console.log('Export window created successfully');
                    });

                    webview.once('tauri://error', function (e) {
                        console.error('Failed to create export window (tauri://error)', e);
                        alert(`Failed to create export window: ${JSON.stringify(e)}`);
                    });

                    onClose?.(); // Close the export panel
                } catch (e) {
                    console.error('Window creation error:', e);
                    alert(`Window creation error: ${e.message}`);
                }
                return;
            }

            // File Export
            let content, extension, mimeType;

            if (exportFormat === 'klyric') {
                content = serializeKLyric(doc, pretty);
                extension = '.klyric';
                mimeType = 'application/json';
            } else if (exportFormat === 'ass') {
                content = klyricToASS(doc);
                extension = '.ass';
                mimeType = 'text/plain';
            } else {
                throw new Error('Unknown export format');
            }

            // Create download
            const blob = new Blob([content], { type: mimeType });
            const url = URL.createObjectURL(blob);
            const link = document.createElement('a');
            link.href = url;
            link.download = `${filename}${extension}`;
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);
            URL.revokeObjectURL(url);

            setExportStatus({ type: 'success', message: `Exported ${filename}${extension}` });
            console.log(`📤 Exported ${filename}${extension} (${content.length} bytes)`);
        } catch (error) {
            setExportStatus({ type: 'error', message: error.message });
            console.error('Export failed:', error);
        }
    }, [exportFormat, filename, pretty, getKLyricDoc, isTauriAvailable, audioSource, onClose, getSanitizedDoc]);

    const handleCopyToClipboard = useCallback(async () => {
        try {
            const doc = getKLyricDoc();
            let content;

            if (exportFormat === 'klyric') {
                content = serializeKLyric(doc, pretty);
            } else {
                content = klyricToASS(doc);
            }

            await navigator.clipboard.writeText(content);
            setExportStatus({ type: 'success', message: 'Copied to clipboard!' });
        } catch (error) { // eslint-disable-line no-unused-vars
            setExportStatus({ type: 'error', message: 'Failed to copy to clipboard' });
        }
    }, [exportFormat, pretty, getKLyricDoc]);

    // If rendering, show rendering progress dialog content
    if (isRendering) {
        return <ExportProgressDialog open={true} progress={progress} cancelRender={cancelRender} />;
    }

    return (
        <ExportForm
            onClose={onClose}
            lyrics={lyrics}
            klyricDoc={klyricDoc}
            resolution={resolution}
            exportFormat={exportFormat}
            setExportFormat={setExportFormat}
            filename={filename}
            setFilename={setFilename}
            pretty={pretty}
            setPretty={setPretty}
            exportStatus={exportStatus}
            isTauriAvailable={isTauriAvailable}
            handleExport={handleExport}
            handleCopyToClipboard={handleCopyToClipboard}
        />
    );
};

export default ExportPanel;
