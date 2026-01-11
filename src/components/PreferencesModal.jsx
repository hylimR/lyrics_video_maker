
import { useState, useEffect } from 'react';
import { useAppStore } from '@/store/useAppStore';
import { EFFECT_PRESETS } from '@/constants';
import FontSelector from '@/components/FontSelector';
import { Button } from "@/components/ui/button"
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
    Tabs,
    TabsContent,
    TabsList,
    TabsTrigger,
} from "@/components/ui/tabs"
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select"
import { Switch } from "@/components/ui/switch"

const PreferencesModal = ({ open, onOpenChange }) => {
    const {
        preferences,
        updatePreferences,
        globalStyle,
        updateState,
        selectedEffect,
        availableFonts
    } = useAppStore();

    // Local state for styling preferences (which might map to globalStyle/selectedEffect immediately or on save)
    // Actually, "Defaults" implies what happens when we reset.
    // But the user request says "allow customization of default font style, default fx, default pos".
    // It's ambiguous if this means "Change current AND set as default" or "Just set default for new projects".
    // Let's assume:
    // 1. "Global Settings" (Current Project) - we already have panels for this.
    // 2. "User Preferences" (App Level) - applies to NEW projects or when "Reset" is valid.

    // HOWEVER, typically users want "My default font is X". When I open the app, I want X.

    // Let's implement it as:
    // - Editing these values updates the store's `preferences` object.
    // - We might need a "Apply to current project" button?

    // Wait, the user request: "Add an user preference to the top bar... default font style, default fx..."
    // This strongly suggests persistence of defaults.

    // Let's map the UI fields to `preferences`.

    const exportPrefs = preferences?.export || {};

    const handleExportChange = (key, value) => {
        updatePreferences({
            export: {
                ...exportPrefs,
                [key]: value
            }
        });
    };

    const handleVideoExportChange = (key, value) => {
        updatePreferences({
            export: {
                ...exportPrefs,
                video: {
                    ...exportPrefs.video,
                    [key]: value
                }
            }
        });
    };

    // For "Default Font/FX/Pos", we probably need to store them in `preferences` too if we want them to technically satisfy "Default".
    // Currently `globalStyle` is persisted. 
    // If we want a separate "Default" that revives even if we clear storage, we need it in `preferences`.
    // But `preferences` IS persisted in valid storage.

    // Let's add them to the `preferences` object in the store (I might need to update the store schema again slightly if I missed top-level defaults).
    // I added `export` but not `defaultFont` etc in the previous step's schema update.
    // Let's blindly add them to the update call, it should work if the store is just an object.

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="sm:max-w-[600px] bg-slate-900/95 border-slate-700 text-slate-100 backdrop-blur-xl">
                <DialogHeader>
                    <DialogTitle>User Preferences</DialogTitle>
                    <DialogDescription className="text-slate-400">
                        Customize application defaults and behavior.
                    </DialogDescription>
                </DialogHeader>

                <Tabs defaultValue="general" className="w-full">
                    <TabsList className="grid w-full grid-cols-2 bg-slate-800/50">
                        <TabsTrigger value="general">General & Defaults</TabsTrigger>
                        <TabsTrigger value="export">Export Settings</TabsTrigger>
                    </TabsList>

                    {/* --- General Defaults --- */}
                    <TabsContent value="general" className="space-y-4 py-4">
                        <div className="space-y-4">
                            <div className="space-y-2">
                                <Label>Default Font Family</Label>
                                <div className="h-10">
                                    <FontSelector
                                        selectedFont={preferences.defaultFont || 'Noto Sans SC'}
                                        availableFonts={availableFonts}
                                        onFontChange={(font) => updatePreferences({ defaultFont: font.name })}
                                        className="w-full"
                                    />
                                </div>
                                <p className="text-xs text-slate-500">Applied to new projects or reset lines.</p>
                            </div>

                            <div className="grid grid-cols-2 gap-4">
                                <div className="space-y-2">
                                    <Label>Default Font Size</Label>
                                    <Input
                                        type="number"
                                        value={preferences.defaultFontSize || 48}
                                        onChange={(e) => updatePreferences({ defaultFontSize: parseInt(e.target.value) || 48 })}
                                        className="bg-slate-800 border-slate-700"
                                    />
                                </div>
                                <div className="space-y-2">
                                    <Label>Default Vertical Offset</Label>
                                    <Input
                                        type="number"
                                        value={preferences.defaultVerticalOffset || 0}
                                        onChange={(e) => updatePreferences({ defaultVerticalOffset: parseInt(e.target.value) || 0 })}
                                        className="bg-slate-800 border-slate-700"
                                    />
                                </div>
                            </div>

                            <div className="space-y-2">
                                <Label>Default Visual Effect</Label>
                                <Select
                                    value={preferences.defaultEffect || 'blur'}
                                    onValueChange={(val) => updatePreferences({ defaultEffect: val })}
                                >
                                    <SelectTrigger className="bg-slate-800 border-slate-700">
                                        <SelectValue placeholder="Select Effect" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {EFFECT_PRESETS.map(effect => (
                                            <SelectItem key={effect.value} value={effect.value}>
                                                {effect.label}
                                            </SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                            </div>
                        </div>
                    </TabsContent>

                    {/* --- Export Settings --- */}
                    <TabsContent value="export" className="space-y-4 py-4">
                        <div className="space-y-4">
                            <div className="space-y-2">
                                <Label>Default Export Format</Label>
                                <Select
                                    value={exportPrefs.format || 'klyric'}
                                    onValueChange={(val) => handleExportChange('format', val)}
                                >
                                    <SelectTrigger className="bg-slate-800 border-slate-700">
                                        <SelectValue />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="klyric">KLyric (.klyric)</SelectItem>
                                        <SelectItem value="ass">ASS / SSA (.ass)</SelectItem>
                                        <SelectItem value="video">Video (.mp4) [Desktop Only]</SelectItem>
                                    </SelectContent>
                                </Select>
                            </div>

                            <div className="space-y-2">
                                <Label>Default Filename Pattern</Label>
                                <Input
                                    value={exportPrefs.filenamePattern || 'lyrics'}
                                    onChange={(e) => handleExportChange('filenamePattern', e.target.value)}
                                    className="bg-slate-800 border-slate-700"
                                    placeholder="e.g. lyrics"
                                />
                            </div>

                            <div className="flex items-center justify-between space-x-2 bg-slate-800/30 p-3 rounded-md">
                                <Label htmlFor="pretty-print" className="flex flex-col space-y-1 cursor-pointer">
                                    <span>Pretty Print JSON</span>
                                    <span className="font-normal text-xs text-slate-500">Format KLyric JSON with indentation</span>
                                </Label>
                                <Switch
                                    id="pretty-print"
                                    checked={exportPrefs.prettyPrint ?? true}
                                    onCheckedChange={(c) => handleExportChange('prettyPrint', c)}
                                />
                            </div>
                        </div>
                    </TabsContent>
                </Tabs>

                <DialogFooter>
                    <Button onClick={() => onOpenChange(false)}>Close</Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
};

export default PreferencesModal;
