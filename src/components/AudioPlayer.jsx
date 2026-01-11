import { forwardRef, useImperativeHandle, useEffect } from 'react';
import { useAudio } from 'react-use';

/**
 * AudioPlayer - Wrapper around react-use's useAudio hook.
 * 
 * Provides an imperative interface matching HTMLMediaElement partially,
 * allowing the parent to control playback/seeking while delegating 
 * state management to the hook.
 */
const AudioPlayer = forwardRef(({ src, onPlay, onPause, onEnded, onDurationChange }, ref) => {
    // useAudio returns [audioElement, state, controls, ref]
    // We pass props directly to the audio element creation
    const [audio, state, controls, audioRef] = useAudio({
        src,
        autoPlay: false,
        onEnded: () => onEnded?.(),
        onPlay: () => onPlay?.(),
        onPause: () => onPause?.(),
    });

    // Expose control API to parent
    useImperativeHandle(ref, () => ({
        play: () => controls.play(),
        pause: () => controls.pause(),
        seek: (time) => controls.seek(time),

        // Getters mirroring HTMLMediaElement properties used by App.jsx
        get currentTime() { return state.time; },
        get duration() { return state.duration; },
        get paused() { return state.paused; },
        get ended() { return state.duration > 0 && state.time >= state.duration; },

        set currentTime(v) { controls.seek(v); },

        // Access to raw element if specialized listeners are needed
        raw: audioRef.current
    }));

    // Sync Duration changes
    useEffect(() => {
        if (state.duration > 0) {
            onDurationChange?.(state.duration);
        }
    }, [state.duration, onDurationChange]);

    return audio;
});

export default AudioPlayer;
