import React, { useState, useCallback } from 'react';
import { EFFECT_PRESETS } from '@/constants';
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { RotateCcw, X } from "lucide-react";
import FontSelector from '@/components/FontSelector';
import StyleEditor from '@/components/StyleEditor';

/**
 * Character Property Panel - Enhanced editor for KLyric character properties
 * 
 * Supports:
 * - Transform: offsetX, offsetY, scale, rotation, opacity
 * - Effects: Per-character effect selection
 * - Animations: Character-level animation presets
 */
const CharacterPropertyPanel = ({
    charIndex,
    char,
    customization,
    resolution = { width: 1920, height: 1080 },
    onCustomizationChange,
    onClose,
    availableFonts
}) => {
    // Resolution bounds for offsets
    const maxOffsetX = resolution.width;
    const maxOffsetY = resolution.height;

    // Current values with defaults
    const values = {
        offsetX: customization?.offsetX || 0,
        offsetY: customization?.offsetY || 0,
        scale: customization?.scale || 1,
        rotation: customization?.rotation || 0,
        opacity: customization?.opacity ?? 1,
        effect: customization?.effect || 'none',
        animation: customization?.animation || 'default',
        fontFamily: customization?.font?.family || '',
        fontSize: customization?.font?.size || '',
        strokeWidth: customization?.stroke?.width || 0,
        strokeColor: customization?.stroke?.color || '#000000',
        shadowX: customization?.shadow?.x || 0,
        shadowY: customization?.shadow?.y || 0,
        shadowBlur: customization?.shadow?.blur || 0,
        shadowColor: customization?.shadow?.color || '#000000',
    };

    // Clamp value to min/max range
    const clamp = (value, min, max) => Math.min(Math.max(value, min), max);

    const handleChange = useCallback((key, value) => {
        // Clamp offset values to resolution bounds
        let clampedValue = value;
        if (key === 'offsetX') {
            clampedValue = clamp(value, -maxOffsetX, maxOffsetX);
        } else if (key === 'offsetY') {
            clampedValue = clamp(value, -maxOffsetY, maxOffsetY);
        }

        // Construct new values based on current values + update
        const updatedFlat = { ...values, [key]: clampedValue };

        // Reconstruct the structured object
        const structured = {
            offsetX: updatedFlat.offsetX,
            offsetY: updatedFlat.offsetY,
            scale: updatedFlat.scale,
            rotation: updatedFlat.rotation,
            opacity: updatedFlat.opacity,
            effect: updatedFlat.effect,
            animation: updatedFlat.animation,
        };

        if (updatedFlat.fontFamily) {
            structured.font = {
                family: updatedFlat.fontFamily,
                size: updatedFlat.fontSize || customization?.font?.size || 72
            };
        }

        if (updatedFlat.strokeWidth > 0) {
            structured.stroke = {
                width: updatedFlat.strokeWidth,
                color: updatedFlat.strokeColor
            };
        }

        if (updatedFlat.shadowBlur > 0 || updatedFlat.shadowX !== 0 || updatedFlat.shadowY !== 0) {
            structured.shadow = {
                x: updatedFlat.shadowX,
                y: updatedFlat.shadowY,
                blur: updatedFlat.shadowBlur,
                color: updatedFlat.shadowColor
            };
        }

        onCustomizationChange(charIndex, structured);
    }, [values, charIndex, onCustomizationChange, maxOffsetX, maxOffsetY, customization]);

    // Default values for reset
    const defaults = {
        offsetX: 0,
        offsetY: 0,
        scale: 1,
        rotation: 0,
        opacity: 1,
        effect: 'none',
        animation: 'default'
    };

    const handleReset = useCallback(() => {
        onCustomizationChange(charIndex, defaults);
    }, [charIndex, onCustomizationChange]);

    return (
        <Card className="w-full bg-card/50 backdrop-blur-sm border-l border-border/50 flex flex-col rounded-none">
            <CardHeader className="py-4 px-6 border-b border-border/50 flex flex-row items-center justify-between space-y-0">
                <div className="flex items-center gap-3">
                    <div className="flex items-center justify-center w-10 h-10 rounded-lg bg-primary/10 text-primary font-bold text-xl border border-primary/20">
                        {char}
                    </div>
                    <div className="flex flex-col">
                        <CardTitle className="text-sm font-bold leading-none">Character Properties</CardTitle>
                        <p className="text-xs text-muted-foreground mt-1">Index: #{charIndex + 1}</p>
                    </div>
                </div>
                <div className="flex items-center gap-2">
                    <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8 text-muted-foreground hover:text-foreground"
                        onClick={handleReset}
                        title="Reset to Defaults"
                    >
                        <RotateCcw className="w-4 h-4" />
                    </Button>
                    <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8 text-muted-foreground hover:text-destructive"
                        onClick={onClose}
                        title="Close Panel"
                    >
                        <X className="w-4 h-4" />
                    </Button>
                </div>
            </CardHeader>

            <CardContent className="p-0">
                <div className="p-6">
                    <StyleEditor
                        mode="char"
                        values={values}
                        onChange={handleChange}
                        availableFonts={availableFonts}
                    />
                </div>
            </CardContent>
        </Card>
    );
};

export default CharacterPropertyPanel;

