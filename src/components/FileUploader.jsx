import { useCallback, useRef, useState } from 'react';
import { validateLyrics } from '@/utils/lyricParsers';
import { importSubtitleToKLyric } from '@/utils/KLyricFormat';
import { LYRIC_EXTENSIONS, AUDIO_EXTENSIONS } from '@/constants';
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { Upload, FileMusic, FileText, AlertCircle, CheckCircle, AlertTriangle } from "lucide-react";

/**
 * FileUploader.jsx - Lyric and Audio File Upload Component
 */
const FileUploader = ({ onLyricsLoaded, onAudioLoaded, currentLyricsCount }) => {
    const [isDragging, setIsDragging] = useState(false);
    const [lastFile, setLastFile] = useState(null);
    const [error, setError] = useState(null);
    const [parseInfo, setParseInfo] = useState(null);

    const lyricInputRef = useRef(null);
    const audioInputRef = useRef(null);

    const handleLyricFile = useCallback(async (file) => {
        setError(null);
        setParseInfo(null);

        try {
            const content = await file.text();

            // Convert to KLyric format (handles all formats: LRC, SRT, ASS, KLyric)
            const { klyric, legacy, format } = importSubtitleToKLyric(content, file.name);

            const validation = validateLyrics(legacy);

            if (!validation.valid) {
                setError(validation.errors.join(', '));
                return;
            }

            setLastFile({ name: file.name, type: 'lyrics' });
            setParseInfo({
                count: legacy.length,
                format: format.toUpperCase(),
                warnings: validation.warnings,
                metadata: klyric.meta,
                isKLyric: true
            });

            if (onLyricsLoaded) {
                // Pass both legacy format (for backward compatibility) and KLyric format
                onLyricsLoaded(legacy, klyric.meta, klyric);
            }

            console.log(`📝 Loaded ${legacy.length} lines from ${file.name} (${format} → KLyric)`);
        } catch (err) {
            setError(`Failed to parse file: ${err.message}`);
        }
    }, [onLyricsLoaded]);

    const handleAudioFile = useCallback((file) => {
        setError(null);

        try {
            const url = URL.createObjectURL(file);
            setLastFile({ name: file.name, type: 'audio' });

            if (onAudioLoaded) {
                onAudioLoaded(url, file.name);
            }

            console.log(`🎵 Loaded audio: ${file.name}`);
        } catch (err) {
            setError(`Failed to load audio: ${err.message}`);
        }
    }, [onAudioLoaded]);

    const handleFile = useCallback((file) => {
        const extension = '.' + file.name.split('.').pop().toLowerCase();

        if (LYRIC_EXTENSIONS.includes(extension)) {
            handleLyricFile(file);
        } else if (AUDIO_EXTENSIONS.includes(extension)) {
            handleAudioFile(file);
        } else {
            setError(`Unsupported file type: ${extension}`);
        }
    }, [handleLyricFile, handleAudioFile]);

    const handleDrop = useCallback((e) => {
        e.preventDefault();
        setIsDragging(false);

        const files = Array.from(e.dataTransfer.files);
        files.forEach(handleFile);
    }, [handleFile]);

    const handleDragOver = useCallback((e) => {
        e.preventDefault();
        setIsDragging(true);
    }, []);

    const handleDragLeave = useCallback((e) => {
        e.preventDefault();
        setIsDragging(false);
    }, []);

    const handleLyricInputChange = useCallback((e) => {
        const file = e.target.files?.[0];
        if (file) handleLyricFile(file);
    }, [handleLyricFile]);

    const handleAudioInputChange = useCallback((e) => {
        const file = e.target.files?.[0];
        if (file) handleAudioFile(file);
    }, [handleAudioFile]);

    return (
        <Card className="w-full bg-card/50 backdrop-blur-sm border-border/50">
            <CardContent className="p-6 space-y-6">
                {/* Drop Zone */}
                <div
                    className={cn(
                        "relative border-2 border-dashed rounded-xl p-8 transition-colors text-center cursor-pointer",
                        isDragging
                            ? "border-primary bg-primary/10"
                            : "border-border hover:border-primary/50 hover:bg-muted/50"
                    )}
                    onDrop={handleDrop}
                    onDragOver={handleDragOver}
                    onDragLeave={handleDragLeave}
                >
                    <div className="flex flex-col items-center gap-2 text-muted-foreground">
                        <Upload className={cn("w-10 h-10 mb-2", isDragging && "text-primary")} />
                        <span className="font-medium text-foreground">
                            {isDragging ? 'Drop file here!' : 'Drag & drop lyrics or audio'}
                        </span>
                        <span className="text-xs">
                            .lrc, .srt, .ass, .mp3, .wav
                        </span>
                    </div>
                </div>

                {/* File Input Buttons */}
                <div className="flex gap-3">
                    <Button
                        variant="outline"
                        className="flex-1 gap-2"
                        onClick={() => lyricInputRef.current?.click()}
                    >
                        <FileText className="w-4 h-4" />
                        Load Lyrics
                    </Button>
                    <Button
                        variant="outline"
                        className="flex-1 gap-2"
                        onClick={() => audioInputRef.current?.click()}
                    >
                        <FileMusic className="w-4 h-4" />
                        Load Audio
                    </Button>

                    <input
                        ref={lyricInputRef}
                        type="file"
                        accept=".lrc,.srt,.ass,.ssa,.klyric,.json"
                        onChange={handleLyricInputChange}
                        className="hidden"
                    />
                    <input
                        ref={audioInputRef}
                        type="file"
                        accept=".mp3,.wav,.ogg,.m4a,.flac,.aac"
                        onChange={handleAudioInputChange}
                        className="hidden"
                    />
                </div>

                {/* Status Display */}
                <div className="space-y-2">
                    {error && (
                        <div className="flex items-center gap-2 p-3 text-sm rounded-lg bg-destructive/10 text-destructive border border-destructive/20">
                            <AlertCircle className="w-4 h-4 shrink-0" />
                            {error}
                        </div>
                    )}

                    {parseInfo && (
                        <div className="flex items-center gap-2 p-3 text-sm rounded-lg bg-green-500/10 text-green-500 border border-green-500/20">
                            <CheckCircle className="w-4 h-4 shrink-0" />
                            <div className="flex flex-col">
                                <span>Loaded {parseInfo.count} lines ({parseInfo.format})</span>
                                {parseInfo.metadata?.ti && (
                                    <span className="text-xs opacity-80">{parseInfo.metadata.ti}</span>
                                )}
                            </div>
                        </div>
                    )}

                    {lastFile?.type === 'audio' && (
                        <div className="flex items-center gap-2 p-3 text-sm rounded-lg bg-purple-500/10 text-purple-500 border border-purple-500/20">
                            <FileMusic className="w-4 h-4 shrink-0" />
                            <span>{lastFile.name}</span>
                        </div>
                    )}

                    {parseInfo?.warnings?.length > 0 && (
                        <div className="flex items-center gap-2 p-3 text-sm rounded-lg bg-orange-500/10 text-orange-500 border border-orange-500/20">
                            <AlertTriangle className="w-4 h-4 shrink-0" />
                            <span>{parseInfo.warnings.length} warning(s)</span>
                        </div>
                    )}
                </div>

                {/* Current Status */}
                <div className="flex justify-between items-center pt-2 border-t border-border/50">
                    <span className="text-xs text-muted-foreground uppercase tracking-wider font-medium">Current lyrics</span>
                    <span className="text-sm font-mono bg-muted px-2 py-0.5 rounded">{currentLyricsCount} lines</span>
                </div>
            </CardContent>
        </Card>
    );
};

export default FileUploader;
