import React from 'react';
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogDescription,
    DialogFooter
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import {
    FileJson, Clapperboard, FileType, Check, Copy, Download, X, Film
} from "lucide-react";
import { cn } from "@/lib/utils";

/**
 * ExportForm - The main form for choosing export format and options
 */
const ExportForm = ({
    onClose,
    lyrics,
    klyricDoc,
    resolution,
    exportFormat,
    setExportFormat,
    filename,
    setFilename,
    pretty,
    setPretty,
    exportStatus,
    isTauriAvailable,
    handleExport,
    handleCopyToClipboard
}) => {
    const lineCount = lyrics.length;
    const charCount = lyrics.reduce((sum, l) => sum + l.text.length, 0);
    const hasKLyricDoc = Boolean(klyricDoc);

    return (
        <Dialog open={true} onOpenChange={(open) => !open && onClose && onClose()}>
            <DialogContent className="sm:max-w-[500px] bg-background/80 backdrop-blur-2xl border-white/10 shadow-2xl">
                <DialogHeader>
                    <DialogTitle>Export Lyrics</DialogTitle>
                    <DialogDescription>
                        Choose a format to export your lyrics.
                    </DialogDescription>
                </DialogHeader>

                <div className="grid grid-cols-3 gap-4 mb-4">
                    <div className="flex flex-col items-center justify-center p-3 border rounded-lg bg-muted/20">
                        <span className="text-2xl font-bold">{lineCount}</span>
                        <span className="text-xs text-muted-foreground uppercase tracking-wider">Lines</span>
                    </div>
                    <div className="flex flex-col items-center justify-center p-3 border rounded-lg bg-muted/20">
                        <span className="text-2xl font-bold">{charCount}</span>
                        <span className="text-xs text-muted-foreground uppercase tracking-wider">Chars</span>
                    </div>
                    <div className="flex flex-col items-center justify-center p-3 border rounded-lg bg-muted/20">
                        <span className={cn("text-2xl font-bold", hasKLyricDoc ? "text-primary" : "text-muted-foreground")}>
                            {hasKLyricDoc ? <Check className="w-8 h-8" /> : <X className="w-8 h-8" />}
                        </span>
                        <span className="text-xs text-muted-foreground uppercase tracking-wider">KLyric Doc</span>
                    </div>
                </div>

                <div className="gap-4 flex flex-col">
                    <div className="space-y-3">
                        <Label>Format</Label>
                        <RadioGroup value={exportFormat} onValueChange={setExportFormat} className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                            <Label
                                htmlFor="klyric"
                                className={cn(
                                    "flex items-start space-x-3 space-y-0 rounded-md border p-4 cursor-pointer hover:bg-muted/50 transition-colors",
                                    exportFormat === "klyric" && "border-primary bg-primary/5"
                                )}
                            >
                                <RadioGroupItem value="klyric" id="klyric" className="mt-1" />
                                <div className="space-y-1">
                                    <div className="font-medium flex items-center">
                                        <FileJson className="w-4 h-4 mr-2 text-blue-500" />
                                        KLyric (.klyric)
                                    </div>
                                    <div className="text-xs text-muted-foreground">
                                        JSON format with full features and styling.
                                    </div>
                                </div>
                            </Label>

                            <Label
                                htmlFor="ass"
                                className={cn(
                                    "flex items-start space-x-3 space-y-0 rounded-md border p-4 cursor-pointer hover:bg-muted/50 transition-colors",
                                    exportFormat === "ass" && "border-primary bg-primary/5"
                                )}
                            >
                                <RadioGroupItem value="ass" id="ass" className="mt-1" />
                                <div className="space-y-1">
                                    <div className="font-medium flex items-center">
                                        <FileType className="w-4 h-4 mr-2 text-orange-500" />
                                        ASS (.ass)
                                    </div>
                                    <div className="text-xs text-muted-foreground">
                                        Standard karaoke subtitle format.
                                    </div>
                                </div>
                            </Label>

                            {isTauriAvailable && (
                                <Label
                                    htmlFor="video"
                                    className={cn(
                                        "flex items-start space-x-3 space-y-0 rounded-md border p-4 cursor-pointer hover:bg-muted/50 transition-colors sm:col-span-2",
                                        exportFormat === "video" && "border-primary bg-primary/5"
                                    )}
                                >
                                    <RadioGroupItem value="video" id="video" className="mt-1" />
                                    <div className="space-y-1">
                                        <div className="font-medium flex items-center">
                                            <Film className="w-4 h-4 mr-2 text-purple-500" />
                                            MP4 Video Export
                                        </div>
                                        <div className="text-xs text-muted-foreground">
                                            High-performance Rust backend rendering.
                                        </div>
                                    </div>
                                </Label>
                            )}
                        </RadioGroup>
                    </div>

                    {exportFormat !== 'video' && (
                        <div className="space-y-3">
                            <Label htmlFor="filename">Filename</Label>
                            <div className="relative">
                                <Input
                                    id="filename"
                                    value={filename}
                                    onChange={(e) => setFilename(e.target.value)}
                                    className="pr-16"
                                />
                                <div className="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none text-muted-foreground text-sm">
                                    {exportFormat === 'klyric' ? '.klyric' : '.ass'}
                                </div>
                            </div>
                        </div>
                    )}

                    {exportFormat === 'klyric' && (
                        <div className="flex items-center space-x-2">
                            <Checkbox id="pretty" checked={pretty} onCheckedChange={setPretty} />
                            <Label htmlFor="pretty" className="cursor-pointer">Pretty print (human readable)</Label>
                        </div>
                    )}

                    {exportFormat === 'video' && (
                        <div className="rounded-md bg-muted/50 p-4 text-sm text-muted-foreground space-y-1">
                            <p><strong>Resolution:</strong> {resolution.width}x{resolution.height}</p>
                            <p><strong>FPS:</strong> 30</p>
                        </div>
                    )}

                    {exportStatus && (
                        <div className={cn(
                            "text-sm p-2 rounded flex items-center gap-2",
                            exportStatus.type === 'success' ? "bg-green-500/10 text-green-500" : "bg-red-500/10 text-red-500"
                        )}>
                            {exportStatus.type === 'success' ? <Check className="w-4 h-4" /> : <X className="w-4 h-4" />}
                            {exportStatus.message}
                        </div>
                    )}
                </div>

                <DialogFooter className="gap-2 sm:gap-0">
                    {exportFormat !== 'video' && (
                        <Button variant="outline" onClick={handleCopyToClipboard}>
                            <Copy className="w-4 h-4 mr-2" /> Copy
                        </Button>
                    )}
                    <Button onClick={handleExport}>
                        {exportFormat === 'video' ? (
                            <>
                                <Clapperboard className="w-4 h-4 mr-2" /> Render Video
                            </>
                        ) : (
                            <>
                                <Download className="w-4 h-4 mr-2" /> Download
                            </>
                        )}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
};

export default ExportForm;
