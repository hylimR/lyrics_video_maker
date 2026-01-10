import { useCallback } from 'react';
import { cn } from "@/lib/utils";
import { formatTime } from '@/utils/timeUtils';
import { useAppStore } from '@/store/useAppStore';
import { Button } from "@/components/ui/button";
import { Slider } from "@/components/ui/slider";
import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
} from "@/components/ui/tooltip";
import {
    Play, Pause, SkipBack, SkipForward, Repeat, RotateCcw,
    MousePointerClick, Delete, Scissors, ChevronUp, ChevronDown
} from "lucide-react";

/**
 * ControlPanel.jsx - YouTube Music Style Bottom Player Bar
 * 
 * Sleek, minimal player bar with playback controls, timeline, and settings.
 */
const ControlPanel = ({
    isPlaying,
    currentTime,
    duration,
    onPlay,
    onPause,
    onSeek,
}) => {
    const {
        ktiming,
        lyrics,
        setKTimingIndex,
        setKTimingLoop,
        markSyllable,
        undoMark,
        autoSplitTerm,
        resetLineTiming,
    } = useAppStore();

    const { lineIndex, loopMode, markingIndex } = ktiming;
    const currentLyric = lyrics[lineIndex];
    const chars = currentLyric ? currentLyric.text : '';

    const handleSeek = useCallback((value) => {
        onSeek(value[0]);
    }, [onSeek]);

    return (
        <TooltipProvider>
            <div className="fixed bottom-0 left-0 right-0 bg-background/60 backdrop-blur-2xl border-t border-white/10 z-50 shadow-2xl flex flex-col supports-[backdrop-filter]:bg-background/40">
                {/* Progress Timeline - Full Width at Top */}
                <div className="w-full h-1.5 relative group cursor-pointer">
                    <Slider
                        value={[currentTime]}
                        max={duration}
                        step={0.01}
                        onValueChange={handleSeek}
                        className="w-full absolute -top-1.5"
                    />
                    {/* Lyric Markers (Visual only for now, can be enhanced) */}
                    <div className="absolute top-0 left-0 w-full h-full pointer-events-none opacity-50">
                        {lyrics.map((lyric, index) => {
                            const markerPos = duration > 0 ? (lyric.startTime / duration) * 100 : 0;
                            return (
                                <div
                                    key={index}
                                    className="absolute top-0 w-0.5 h-full bg-primary/30 transform -translate-x-1/2"
                                    style={{ left: `${markerPos}%` }}
                                />
                            );
                        })}
                    </div>
                </div>

                {/* Main Controls Row */}
                <div className="flex items-center justify-between px-8 h-20 md:h-24 gap-8">
                    {/* Left: Playback Controls */}
                    <div className="flex items-center gap-6 w-1/3">
                        <Button
                            variant="ghost"
                            size="icon"
                            className="bg-primary hover:bg-primary/90 text-primary-foreground h-12 w-12 min-w-[3rem] rounded-full shadow-lg shadow-primary/20 hover:shadow-primary/40 transition-all"
                            onClick={isPlaying ? onPause : onPlay}
                            title={isPlaying ? 'Pause' : 'Play'}
                        >
                            {isPlaying ? <Pause className="fill-current w-6 h-6" /> : <Play className="fill-current w-6 h-6 ml-1" />}
                        </Button>

                        <div className="flex items-center text-sm font-medium font-mono text-muted-foreground space-x-2 pl-2">
                            <span>{formatTime(currentTime)}</span>
                            <span className="opacity-30">|</span>
                            <span>{formatTime(duration)}</span>
                        </div>
                    </div>

                    {/* Center: Current Lyric Display */}
                    <div className="flex-1 text-center truncate px-4 hidden md:block">
                        <div className="text-sm font-medium text-foreground/90 truncate">
                            {currentLyric?.text || <span className="text-muted-foreground italic">No lyrics loaded</span>}
                        </div>
                        <div className="text-xs text-muted-foreground opacity-70">
                            Line {lineIndex + 1} / {lyrics.length}
                        </div>
                    </div>

                    {/* Right: K-Timing Tools */}
                    <div className="flex items-center justify-end gap-6 w-1/3">
                        {/* Navigation */}
                        <div className="flex items-center border-r border-border/30 pr-4 gap-2">
                            <Tooltip>
                                <TooltipTrigger asChild>
                                    <Button variant="ghost" size="icon" className="h-8 w-8"
                                        onClick={() => setKTimingIndex(Math.max(0, lineIndex - 1))}
                                        disabled={lineIndex === 0}>
                                        <ChevronUp className="w-4 h-4" />
                                    </Button>
                                </TooltipTrigger>
                                <TooltipContent>Previous Line</TooltipContent>
                            </Tooltip>

                            <Tooltip>
                                <TooltipTrigger asChild>
                                    <Button variant="ghost" size="icon" className="h-8 w-8"
                                        onClick={() => setKTimingIndex(Math.min(lyrics.length - 1, lineIndex + 1))}
                                        disabled={lineIndex === lyrics.length - 1}>
                                        <ChevronDown className="w-4 h-4" />
                                    </Button>
                                </TooltipTrigger>
                                <TooltipContent>Next Line</TooltipContent>
                            </Tooltip>
                        </div>

                        {/* Loop & Restart */}
                        <div className="flex items-center border-r border-border/30 pr-4 gap-2">
                            <Tooltip>
                                <TooltipTrigger asChild>
                                    <Button variant="ghost" size="icon" className={cn("h-8 w-8 transition-all duration-300", loopMode ? 'text-primary drop-shadow-[0_0_8px_rgba(44,123,246,0.5)]' : 'text-muted-foreground')}
                                        onClick={() => setKTimingLoop(!loopMode)}>
                                        <Repeat className="w-4 h-4" />
                                    </Button>
                                </TooltipTrigger>
                                <TooltipContent>Toggle Loop (L)</TooltipContent>
                            </Tooltip>

                            <Tooltip>
                                <TooltipTrigger asChild>
                                    <Button variant="ghost" size="icon" className="h-8 w-8 text-muted-foreground"
                                        onClick={() => {
                                            if (currentLyric) onSeek(currentLyric.startTime);
                                            resetLineTiming();
                                        }}>
                                        <RotateCcw className="w-4 h-4" />
                                    </Button>
                                </TooltipTrigger>
                                <TooltipContent>Restart Line (R)</TooltipContent>
                            </Tooltip>
                        </div>

                        {/* Marking */}
                        <div className="flex items-center gap-3 pl-2">
                            <Tooltip>
                                <TooltipTrigger asChild>
                                    <Button variant="outline" className="h-10 px-6 font-bold tracking-widest hover:bg-primary/20 hover:text-primary hover:border-primary/50 text-sm border-2"
                                        onClick={() => markSyllable(currentTime)}
                                        disabled={!currentLyric || markingIndex >= chars.length}>
                                        TAP
                                    </Button>
                                </TooltipTrigger>
                                <TooltipContent>Mark Syllable (Space/K)</TooltipContent>
                            </Tooltip>

                            <Tooltip>
                                <TooltipTrigger asChild>
                                    <Button variant="ghost" size="icon" className="h-8 w-8 text-destructive/80 hover:text-destructive hover:bg-destructive/10"
                                        onClick={undoMark}
                                        disabled={!currentLyric || markingIndex === 0}>
                                        <Delete className="w-4 h-4" />
                                    </Button>
                                </TooltipTrigger>
                                <TooltipContent>Undo Mark (Backspace)</TooltipContent>
                            </Tooltip>

                            <Tooltip>
                                <TooltipTrigger asChild>
                                    <Button variant="ghost" size="icon" className="h-8 w-8"
                                        onClick={autoSplitTerm}
                                        disabled={!currentLyric}>
                                        <Scissors className="w-4 h-4" />
                                    </Button>
                                </TooltipTrigger>
                                <TooltipContent>Auto Split (A)</TooltipContent>
                            </Tooltip>
                        </div>
                    </div>
                </div>
            </div>
        </TooltipProvider>
    );
};

export default ControlPanel;
