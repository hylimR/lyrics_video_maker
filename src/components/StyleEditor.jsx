import React from 'react';
import FontSelector from './FontSelector';
import {
    Tabs,
    TabsContent,
    TabsList,
    TabsTrigger,
} from "@/components/ui/tabs";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Slider } from "@/components/ui/slider";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { Card, CardContent } from "@/components/ui/card";
import { Layout, Type, PaintBucket, Move, Sparkles } from "lucide-react";
import { cn } from "@/lib/utils";

// Helper for Color Input with Label
const ColorInput = ({ label, value, onChange }) => (
    <div className="space-y-3">
        <Label>{label}</Label>
        <div className="flex gap-2">
            <div className="relative w-10 h-10 rounded-md overflow-hidden border border-input shadow-sm">
                <input
                    type="color"
                    value={value || '#000000'}
                    onChange={(e) => onChange(e.target.value)}
                    className="absolute -top-2 -left-2 w-16 h-16 cursor-pointer"
                />
            </div>
            <Input
                value={value || ''}
                onChange={(e) => onChange(e.target.value)}
                className="flex-1 font-mono uppercase"
                maxLength={7}
            />
        </div>
    </div>
);

// Helper for Range/Slider with Number Input
const RangeInput = ({ label, value, onChange, min = 0, max = 100, step = 1, unit = '', className, disabled }) => (
    <div className={cn("space-y-3", className)}>
        <Label className={cn(disabled && "text-muted-foreground")}>{label}</Label>
        <Slider
            value={[typeof value === 'number' ? value : 0]}
            onValueChange={(vals) => onChange(vals[0])}
            min={min}
            max={max}
            step={step}
            disabled={disabled}
            className="py-1"
        />
        <div className="flex items-center gap-2">
            <Input
                type="number"
                value={typeof value === 'number' && step < 1 ? parseFloat(value.toFixed(2)) : value}
                onChange={(e) => {
                    const val = parseFloat(e.target.value);
                    onChange(isNaN(val) ? 0 : val);
                }}
                step={step}
                disabled={disabled}
                className="font-mono"
            />
            {unit && <span className="text-sm text-muted-foreground w-8 shrink-0">{unit}</span>}
        </div>
    </div>
);

const StyleEditor = ({ mode = 'line', values, onChange, availableFonts }) => {

    const renderLayout = () => (
        <div className="space-y-8">
            <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                    <Label>Alignment</Label>
                    <Select value={values.alignment || 'center'} onValueChange={(val) => onChange('alignment', val)}>
                        <SelectTrigger>
                            <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="left">Left</SelectItem>
                            <SelectItem value="center">Center</SelectItem>
                            <SelectItem value="right">Right</SelectItem>
                        </SelectContent>
                    </Select>
                </div>

                <div className="space-y-2">
                    <Label>Direction</Label>
                    <Select value={values.direction || 'ltr'} onValueChange={(val) => onChange('direction', val)}>
                        <SelectTrigger>
                            <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="ltr">Horizontal (LTR)</SelectItem>
                            <SelectItem value="rtl">Horizontal (RTL)</SelectItem>
                            <SelectItem value="ttb">Vertical (TTB)</SelectItem>
                        </SelectContent>
                    </Select>
                </div>
            </div>

            <RangeInput
                label="Line Spacing (Leading)"
                value={values.lineSpacing || 0}
                onChange={(val) => onChange('lineSpacing', val)}
                min={-50} max={200} step={1} unit="px"
            />
            <RangeInput
                label="Letter Spacing (Tracking)"
                value={values.letterSpacing || 0}
                onChange={(val) => onChange('letterSpacing', val)}
                min={-20} max={100} step={1} unit="px"
            />
        </div>
    );

    const renderTransform = () => (
        <div className="space-y-8">
            <div className="grid grid-cols-2 gap-4">
                <RangeInput
                    label="Position X"
                    value={values.offsetX || 0}
                    onChange={(val) => onChange('offsetX', val)}
                    min={-1000}
                    max={1000}
                    step={1}
                    unit="px"
                />
                <RangeInput
                    label="Position Y"
                    value={values.offsetY || 0}
                    onChange={(val) => onChange('offsetY', val)}
                    min={-1000}
                    max={1000}
                    step={1}
                    unit="px"
                />
            </div>

            <RangeInput
                label="Scale"
                value={values.scale ?? 1}
                onChange={(val) => onChange('scale', val)}
                min={0} max={5} step={0.05}
            />
            <RangeInput
                label="Rotation"
                value={values.rotation || 0}
                onChange={(val) => onChange('rotation', val)}
                min={-180} max={180} step={1} unit="°"
            />
            <RangeInput
                label="Opacity"
                value={values.opacity ?? 1}
                onChange={(val) => onChange('opacity', val)}
                min={0} max={1} step={0.01}
            />
        </div>
    );

    const renderStyle = () => (
        <div className="space-y-8">
            <div className="space-y-2">
                <Label>Font Family</Label>
                <FontSelector
                    value={values.fontFamily}
                    onChange={(val) => onChange('fontFamily', val)}
                    fonts={availableFonts}
                    allowInherited={mode !== 'global'}
                />
            </div>

            <RangeInput
                label="Font Size"
                value={values.fontSize || 48}
                onChange={(val) => onChange('fontSize', val)}
                min={12} max={200} step={1} unit="px"
                disabled={!values.fontFamily && mode !== 'global'}
                className={cn(!values.fontFamily && mode !== 'global' && "opacity-50 pointer-events-none")}
            />

            <div className="grid grid-cols-2 gap-4">
                <ColorInput
                    label="Fill Color"
                    value={values.fillColor}
                    onChange={(val) => onChange('fillColor', val)}
                />
                <ColorInput
                    label="Stroke Color"
                    value={values.strokeColor}
                    onChange={(val) => onChange('strokeColor', val)}
                />
            </div>

            <RangeInput
                label="Stroke Width"
                value={values.strokeWidth || 0}
                onChange={(val) => onChange('strokeWidth', val)}
                min={0} max={20} step={0.5} unit="px"
            />

            <div className="pt-4 border-t border-border/50 space-y-4">
                <Label className="text-xs uppercase tracking-wider text-muted-foreground">Shadow</Label>
                <ColorInput
                    label="Shadow Color"
                    value={values.shadowColor}
                    onChange={(val) => onChange('shadowColor', val)}
                />
                <RangeInput
                    label="Blur Radius"
                    value={values.shadowBlur || 0}
                    onChange={(val) => onChange('shadowBlur', val)}
                    min={0} max={50} step={1} unit="px"
                />
                <div className="grid grid-cols-2 gap-4">
                    <RangeInput
                        label="Offset X"
                        value={values.shadowX || 0}
                        onChange={(val) => onChange('shadowX', val)}
                        min={-50} max={50} step={1}
                    />
                    <RangeInput
                        label="Offset Y"
                        value={values.shadowY || 0}
                        onChange={(val) => onChange('shadowY', val)}
                        min={-50} max={50} step={1}
                    />
                </div>
            </div>
        </div>
    );

    const renderEffects = () => (
        <div className="space-y-8">
            <div className="space-y-4">
                <div className="space-y-2">
                    <Label>In Animation</Label>
                    <Select value={values.animation || 'default'} onValueChange={(val) => onChange('animation', val)}>
                        <SelectTrigger>
                            <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="default">Fade In</SelectItem>
                            <SelectItem value="slideUp">Slide Up</SelectItem>
                            <SelectItem value="slideDown">Slide Down</SelectItem>
                            <SelectItem value="scaleUp">Scale Up</SelectItem>
                            <SelectItem value="typewriter">Typewriter</SelectItem>
                        </SelectContent>
                    </Select>
                </div>

                <div className="space-y-2">
                    <Label>Continuous Effect</Label>
                    <Select value={values.effect || 'none'} onValueChange={(val) => onChange('effect', val)}>
                        <SelectTrigger>
                            <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="none">None</SelectItem>
                            <SelectItem value="wobbly">Wobbly (Sine Wave)</SelectItem>
                            <SelectItem value="pulse">Pulse</SelectItem>
                            <SelectItem value="rainbow">Rainbow</SelectItem>
                            <SelectItem value="glitch">Glitch</SelectItem>
                            <SelectItem value="blur">Blur Focus</SelectItem>
                            <SelectItem value="glow">Neon Glow</SelectItem>
                        </SelectContent>
                    </Select>
                </div>
            </div>
        </div>
    );

    return (
        <Tabs defaultValue={mode === 'global' ? 'style' : 'transform'} className="w-full">
            <TabsList className="w-full grid grid-cols-4 mb-4">
                {mode !== 'global' && (
                    <TabsTrigger value="layout" className="text-xs px-1">
                        <Type className="w-3 h-3 md:mr-2" />
                        <span className="hidden md:inline">Layout</span>
                    </TabsTrigger>
                )}
                <TabsTrigger value="transform" className="text-xs px-1">
                    <Move className="w-3 h-3 md:mr-2" />
                    <span className="hidden md:inline">Pos</span>
                </TabsTrigger>
                <TabsTrigger value="style" className="text-xs px-1">
                    <PaintBucket className="w-3 h-3 md:mr-2" />
                    <span className="hidden md:inline">Style</span>
                </TabsTrigger>
                <TabsTrigger value="effects" className="text-xs px-1">
                    <Sparkles className="w-3 h-3 md:mr-2" />
                    <span className="hidden md:inline">FX</span>
                </TabsTrigger>
            </TabsList>

            <div className="mt-2 text-sm text-foreground">
                {mode !== 'global' && (
                    <TabsContent value="layout" className="mt-0">
                        {renderLayout()}
                    </TabsContent>
                )}
                <TabsContent value="transform" className="mt-0">
                    {renderTransform()}
                </TabsContent>
                <TabsContent value="style" className="mt-0">
                    {renderStyle()}
                </TabsContent>
                <TabsContent value="effects" className="mt-0">
                    {renderEffects()}
                </TabsContent>
            </div>
        </Tabs>
    );
};

export default StyleEditor;