/**
 * ExportPage.jsx - Standalone page for video export
 */
import { useEffect, useState } from 'react';
import VideoExportPanel from '@/components/VideoExportPanel';
import { getCurrentWindow } from '@tauri-apps/api/window';

const ExportPage = () => {
    const [exportData, setExportData] = useState(null);

    useEffect(() => {
        // Load data from localStorage
        try {
            const stored = localStorage.getItem('pending_export_data');
            if (stored) {
                const parsed = JSON.parse(stored);
                setExportData(parsed);
                // Clear storage after loading? Maybe keep for refresh safety
            }
        } catch (e) {
            console.error('Failed to load export data', e);
        }
    }, []);

    const handleClose = async () => {
        try {
            await getCurrentWindow().close();
        } catch (e) {
            console.warn('Failed to close window', e);
        }
    };

    if (!exportData) {
        return (
            <div className="export-page-loading">
                <p>Loading export data...</p>
            </div>
        );
    }

    return (
        <div className="export-page-container" style={{
            height: '100vh',
            width: '100vw',
            background: '#1a1a20',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center'
        }}>
            <VideoExportPanel
                klyricDocument={exportData.klyricDocument}
                audioPath={exportData.audioPath}
                isStandalone={true}
                onClose={handleClose}
                onComplete={(result) => {
                    console.log('Export complete', result);
                    // Optional: keep window open to show success? 
                    // VideoExportPanel handles "Done" state, so we can let user close it via "Done" button which calls onClose
                }}
            />
        </div>
    );
};

export default ExportPage;
