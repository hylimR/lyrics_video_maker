import { useCallback } from 'react';
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogFooter,
    DialogDescription
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Calendar, FileText, RotateCcw, Check, Sparkles } from "lucide-react";

/**
 * RestoreSessionDialog - Modal to choose between restoring previous session or starting new
 */
const RestoreSessionDialog = ({ sessionInfo, onRestore, onNewSession }) => {
    const formatDate = useCallback((date) => {
        if (!date) return 'Unknown';
        // Handle both string and Date objects
        const d = new Date(date);
        return d.toLocaleString();
    }, []);

    return (
        <Dialog open={true}>
            <DialogContent className="sm:max-w-md border-border/50 bg-background/95 backdrop-blur-xl shadow-2xl">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2 text-xl">
                        <RotateCcw className="w-5 h-5 text-primary" />
                        Restore Previous Session?
                    </DialogTitle>
                    <DialogDescription>
                        A previous editing session was found. Would you like to restore it?
                    </DialogDescription>
                </DialogHeader>

                <div className="grid gap-4 py-4">
                    <div className="grid grid-cols-2 gap-4">
                        <div className="flex flex-col space-y-1.5 p-3 rounded-lg bg-secondary/50 border border-border/50">
                            <span className="text-xs text-muted-foreground font-medium uppercase flex items-center gap-1">
                                <Calendar className="w-3 h-3" /> Last Saved
                            </span>
                            <span className="text-sm font-semibold truncate">
                                {formatDate(sessionInfo?.savedAt)}
                            </span>
                        </div>

                        <div className="flex flex-col space-y-1.5 p-3 rounded-lg bg-secondary/50 border border-border/50">
                            <span className="text-xs text-muted-foreground font-medium uppercase flex items-center gap-1">
                                <FileText className="w-3 h-3" /> Lyrics
                            </span>
                            <span className="text-sm font-semibold">
                                {sessionInfo?.lyricsCount || 0} lines
                            </span>
                        </div>

                        <div className="col-span-2 flex flex-col space-y-1.5 p-3 rounded-lg bg-secondary/50 border border-border/50">
                            <span className="text-xs text-muted-foreground font-medium uppercase flex items-center gap-1">
                                <RotateCcw className="w-3 h-3" /> Undo History
                            </span>
                            <span className="text-sm font-semibold">
                                {sessionInfo?.historySize || 0} steps available
                            </span>
                        </div>
                    </div>
                </div>

                <DialogFooter className="flex gap-2 sm:justify-between sm:gap-0">
                    <Button
                        variant="ghost"
                        onClick={onNewSession}
                        className="flex-1 sm:flex-none text-muted-foreground hover:text-foreground"
                    >
                        <Sparkles className="w-4 h-4 mr-2" />
                        Start New
                    </Button>
                    <Button
                        onClick={onRestore}
                        className="flex-1 sm:flex-none bg-primary text-primary-foreground hover:bg-primary/90"
                    >
                        <Check className="w-4 h-4 mr-2" />
                        Restore Session
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
};

export default RestoreSessionDialog;
