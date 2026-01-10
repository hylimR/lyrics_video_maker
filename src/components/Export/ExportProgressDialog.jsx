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
import { Progress } from "@/components/ui/progress";
import { Clapperboard } from "lucide-react";

/**
 * ExportProgressDialog - Shows the progress of the video rendering process
 */
const ExportProgressDialog = ({ open, progress, cancelRender }) => {
    return (
        <Dialog open={open} onOpenChange={(isOpen) => !isOpen && cancelRender()}>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2">
                        <Clapperboard className="w-5 h-5 animate-pulse text-primary" />
                        Rendering Video...
                    </DialogTitle>
                    <DialogDescription>
                        Please wait while your video is being rendered.
                    </DialogDescription>
                </DialogHeader>

                <div className="space-y-4 py-4">
                    <div className="flex justify-between text-sm text-muted-foreground">
                        <span>{progress.phase}</span>
                        <span>{Math.round(progress.percentage)}%</span>
                    </div>
                    <Progress value={progress.percentage} />
                    <div className="flex justify-between text-xs text-muted-foreground">
                        <span>Frame: {progress.currentFrame} / {progress.totalFrames}</span>
                        <span>ETA: {progress.etaSeconds > 0 ? `${progress.etaSeconds.toFixed(0)}s` : 'Calculating...'}</span>
                    </div>
                </div>

                <DialogFooter>
                    <Button variant="destructive" onClick={cancelRender}>Cancel Render</Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
};

export default ExportProgressDialog;
