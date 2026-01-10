import { create } from 'zustand';
import { persist, subscribeWithSelector } from 'zustand/middleware';
import { SYNC_CONFIG, HISTORY_CONFIG, DEFAULTS } from '@/constants';

// Extract config values
const { CHANNEL_NAME } = SYNC_CONFIG;
const { MAX_UNDO_STEPS: MAX_HISTORY } = HISTORY_CONFIG;

// Broadcast Channel
const channel = new BroadcastChannel(CHANNEL_NAME);
const tabId = `tab-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

/**
 * Zustand Store for App State
 * Implements Peer-to-Peer Data Flow:
 * - Any Window: Updates store -> Broadcasts
 * - Other Windows: Receive broadcast -> Update store (silent)
 */
export const useAppStore = create(
    subscribeWithSelector(
        persist(
            (set, get) => ({
                // --- Content State (Persisted) ---
                lyrics: [],
                klyricDoc: null,
                resolution: DEFAULTS.RESOLUTION,
                selectedEffect: DEFAULTS.EFFECT,
                globalStyle: {}, // Initialize with empty object or default style
                duration: DEFAULTS.DURATION,
                selectedFont: null,

                // --- Playback State (Transient) ---
                currentTime: 0,
                isPlaying: false,

                // --- System State (Transient) ---
                isMaster: false,
                isMasterOnline: false,
                availableFonts: [],
                loadingFonts: new Set(),

                // --- History State (Transient) ---
                past: [],
                future: [],

                // --- K-Timing State (Persisted via partialize) ---
                ktiming: {
                    lineIndex: 0,
                    loopMode: true,
                    markingIndex: 0,
                    markStartTime: null
                },

                // --- Actions ---

                // K-Timing Actions
                setKTimingIndex: (index) => set(state => ({
                    ktiming: { ...state.ktiming, lineIndex: index, markingIndex: 0, markStartTime: null }
                })),

                setKTimingLoop: (loop) => set(state => ({
                    ktiming: { ...state.ktiming, loopMode: loop }
                })),

                // Logic moved from KTimingEditor
                markSyllable: (currentTime) => {
                    const state = get();
                    const { lyrics, ktiming } = state;
                    const { lineIndex, markingIndex, markStartTime } = ktiming;
                    const currentLine = lyrics[lineIndex];

                    if (!currentLine) return;

                    const charCount = currentLine.text.length;
                    if (markingIndex >= charCount) return;

                    let newSyllables = currentLine.syllables ? [...currentLine.syllables] : [];
                    let newMarkStartTime = markStartTime;
                    let newMarkingIndex = markingIndex;

                    if (markStartTime === null) {
                        // Start new syllable
                        newMarkStartTime = currentTime;
                    } else {
                        // End current syllable
                        const duration = currentTime - markStartTime;
                        const newSyllable = {
                            text: currentLine.text[markingIndex],
                            duration: Math.max(0.05, duration),
                            startOffset: markStartTime - currentLine.startTime,
                            charStart: markingIndex,
                            charEnd: markingIndex + 1,
                        };

                        // Update or push
                        const existingIndex = newSyllables.findIndex(s => s.charStart === markingIndex);
                        if (existingIndex >= 0) {
                            newSyllables[existingIndex] = newSyllable;
                        } else {
                            newSyllables.push(newSyllable);
                            newSyllables.sort((a, b) => a.charStart - b.charStart);
                        }

                        newMarkingIndex++;
                        newMarkStartTime = currentTime;
                    }

                    // Update State directly (will trigger broadcast via updateState wrapper if used, but here we invoke set directly so we need to ensure broadcast happens if we want P2P)
                    // We'll reuse updateState logic to keep it simple, but we need to merge it carefully
                    const updatedLyrics = [...lyrics];
                    updatedLyrics[lineIndex] = { ...currentLine, syllables: newSyllables };

                    // We use the store's internal set() but we should probably use updateState to get history/sync?
                    // Let's manually compose it to avoid circular dependency or context loss
                    state.updateState({
                        lyrics: updatedLyrics,
                        ktiming: { ...ktiming, markingIndex: newMarkingIndex, markStartTime: newMarkStartTime }
                    }, 'Mark Syllable');
                },

                undoMark: () => {
                    const state = get();
                    const { lyrics, ktiming } = state;
                    const { lineIndex, markingIndex } = ktiming;

                    if (markingIndex <= 0) return;

                    const currentLine = lyrics[lineIndex];
                    if (!currentLine || !currentLine.syllables) return;

                    const newSyllables = currentLine.syllables.slice(0, -1);
                    const updatedLyrics = [...lyrics];
                    updatedLyrics[lineIndex] = { ...currentLine, syllables: newSyllables };

                    state.updateState({
                        lyrics: updatedLyrics,
                        ktiming: { ...ktiming, markingIndex: markingIndex - 1, markStartTime: null }
                    }, 'Undo Mark');
                },

                autoSplitTerm: () => {
                    const state = get();
                    const { lyrics, ktiming } = state;
                    const { lineIndex } = ktiming;
                    const currentLine = lyrics[lineIndex];

                    if (!currentLine) return;

                    const chars = [...currentLine.text];
                    const duration = currentLine.endTime - currentLine.startTime;
                    const charDuration = duration / chars.length;

                    const newSyllables = chars.map((char, i) => ({
                        text: char,
                        duration: charDuration,
                        startOffset: i * charDuration,
                        charStart: i,
                        charEnd: i + 1,
                    }));

                    const updatedLyrics = [...lyrics];
                    updatedLyrics[lineIndex] = { ...currentLine, syllables: newSyllables };

                    state.updateState({
                        lyrics: updatedLyrics,
                        ktiming: { ...ktiming, markingIndex: chars.length }
                    }, 'Auto Split');
                },

                resetLineTiming: () => {
                    const state = get();
                    const { ktiming } = state;
                    // Just reset markers, don't delete syllables unless explicit? 
                    // Legacy behavior was: restart line seeking, reset markers. 
                    // It didn't delete syllables.
                    // But "Restart [R]" usually implies just seeking and resetting the "recording" state.
                    set(state => ({
                        ktiming: { ...state.ktiming, markingIndex: 0, markStartTime: null }
                    }));
                },

                // General State Update
                updateState: (updates, actionName, options = {}) => {
                    const state = get();
                    const { skipHistory = false } = options;

                    // Check if this is a "content" update that needs history
                    const isContentUpdate = !skipHistory && ('lyrics' in updates || 'klyricDoc' in updates || 'resolution' in updates || 'selectedEffect' in updates || 'selectedFont' in updates || 'globalStyle' in updates);

                    if (isContentUpdate) {
                        // Save to history before applying
                        const currentContent = {
                            lyrics: state.lyrics,
                            klyricDoc: state.klyricDoc,
                            resolution: state.resolution,
                            selectedEffect: state.selectedEffect,
                            selectedFont: state.selectedFont,
                            globalStyle: state.globalStyle,
                            duration: state.duration
                        };

                        set((prev) => ({
                            past: [...prev.past.slice(-MAX_HISTORY), currentContent],
                            future: []
                        }));
                    }

                    // Apply update locally
                    set(updates);

                    // Broadcast new state
                    const newState = get();
                    channel.postMessage({
                        type: 'STATE_SYNC',
                        payload: {
                            lyrics: newState.lyrics,
                            klyricDoc: newState.klyricDoc,
                            resolution: newState.resolution,
                            selectedEffect: newState.selectedEffect,
                            globalStyle: newState.globalStyle,
                            duration: newState.duration,
                            currentTime: newState.currentTime,
                            isPlaying: newState.isPlaying,
                            ktiming: newState.ktiming // Sync KTiming state too
                        },
                        from: tabId
                    });
                },

                // Dedicated Playback Update (High Frequency / Optimization)
                setPlayback: (playback) => {
                    // Apply locally
                    set(playback);

                    // Broadcast playback specific message (lighter weight)
                    channel.postMessage({
                        type: 'PLAYBACK_SYNC',
                        payload: playback,
                        from: tabId
                    });
                },

                // System Actions
                // Master Logic
                setIsMaster: (isMaster) => set({ isMaster }),
                setIsMasterOnline: (isOnline) => set({ isMasterOnline: isOnline }),
                setAvailableFonts: (fonts) => set({ availableFonts: fonts }),
                addLoadingFont: (fontName) => set((state) => {
                    const newSet = new Set(state.loadingFonts);
                    newSet.add(fontName);
                    return { loadingFonts: newSet };
                }),
                removeLoadingFont: (fontName) => set((state) => {
                    const newSet = new Set(state.loadingFonts);
                    newSet.delete(fontName);
                    return { loadingFonts: newSet };
                }),

                // History Actions (Master Only)
                undo: () => {
                    const state = get();
                    if (state.past.length === 0) return;

                    const previous = state.past[state.past.length - 1];
                    const newPast = state.past.slice(0, -1);

                    // Current state becomes future
                    const currentContent = {
                        lyrics: state.lyrics,
                        klyricDoc: state.klyricDoc,
                        resolution: state.resolution,
                        selectedEffect: state.selectedEffect,
                        duration: state.duration
                    };

                    set({
                        ...previous,
                        past: newPast,
                        future: [currentContent, ...state.future]
                    });

                    // Broadcast after undo
                    get().broadcastSync();
                },

                redo: () => {
                    const state = get();
                    if (state.future.length === 0) return;

                    const next = state.future[0];
                    const newFuture = state.future.slice(1);

                    // Current becomes past
                    const currentContent = {
                        lyrics: state.lyrics,
                        klyricDoc: state.klyricDoc,
                        resolution: state.resolution,
                        selectedEffect: state.selectedEffect,
                        duration: state.duration
                    };

                    set({
                        ...next,
                        past: [...state.past, currentContent],
                        future: newFuture
                    });

                    // Broadcast after redo
                    get().broadcastSync();
                },

                // Helper to manually trigger broadcast
                broadcastSync: () => {
                    const s = get();
                    channel.postMessage({
                        type: 'STATE_SYNC',
                        payload: {
                            lyrics: s.lyrics,
                            klyricDoc: s.klyricDoc,
                            resolution: s.resolution,
                            selectedEffect: s.selectedEffect,
                            duration: s.duration,
                            currentTime: s.currentTime,
                            isPlaying: s.isPlaying,
                            ktiming: s.ktiming
                        },
                        from: tabId
                    });
                }
            }),
            {
                name: 'lyric-video-storage',
                // Only persist content, not playback or system status
                partialize: (state) => ({
                    lyrics: state.lyrics,
                    klyricDoc: state.klyricDoc,
                    resolution: state.resolution,
                    selectedEffect: state.selectedEffect,
                    globalStyle: state.globalStyle,
                    duration: state.duration,
                    ktiming: state.ktiming // Persist KTiming preference
                }),
            }
        )
    )
);

// --- Initialization & Network Logic ---

export function initStore() {
    const store = useAppStore;

    // Message Handler
    channel.onmessage = (event) => {
        const { type, payload, from } = event.data;
        const state = store.getState();

        if (from === tabId) return; // Ignore self

        switch (type) {
            case 'STATE_SYNC':
                // Received state from peer
                // Update local state without triggering another broadcast (we use setState directly for this usually, but here we just need to NOT call updateState)
                // However, Zustand's 'set' is internal. `useAppStore.setState` is the public API.
                // We do NOT want to add to history here probably? Or do we?
                // Usually we sustain history sync via 'state' but history stack itself is separate?
                // For simplicity, let's keep history local to the tab that performed the action, 
                // BUT if we want true collaborative undo, we'd sync history too.
                // Here: Just apply the content.
                useAppStore.setState({
                    ...payload,
                    // Preserve local history stacks so we don't wipe them on sync
                    past: state.past,
                    future: state.future
                });
                break;

            case 'PLAYBACK_SYNC':
                // Direct apply
                useAppStore.setState(payload);
                break;
            case 'REQUEST_SYNC':
                // Peer requested sync, broadcast our state
                // Only master or everyone? Everyone is fine, latest timestamp wins usually, 
                // but here we have no timestamps. 
                // Let's assume the active editor (Master) should respond.
                // Or just everyone responds? If everyone responds, we might get flood.
                // Ideally only 'Master' response.
                if (state.isMaster || state.lyrics.length > 0) {
                    state.broadcastSync();
                }
                break;
        }
    };

    // Request Sync on Init (in case we are a new window)
    channel.postMessage({ type: 'REQUEST_SYNC', from: tabId });

    // Cleanup
    return () => {
        // Do NOT close the channel, as it is a module singleton. 
        // Just unbind listeners.
        channel.onmessage = null;
    };
}
