import React, { useCallback, useMemo } from 'react';
import { useAppStore } from '@/store/useAppStore';
import StyleEditor from '@/components/StyleEditor';
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Globe } from "lucide-react";

const GlobalStyleEditor = ({ availableFonts }) => {
    const { globalStyle, updateState, preferences } = useAppStore();

    // Map globalStyle to flat values
    const values = useMemo(() => ({
        fontFamily: globalStyle?.font?.family || preferences?.defaultFont || 'Noto Sans SC',
        fontSize: globalStyle?.font?.size || preferences?.defaultFontSize || 72,
        // Transform
        offsetX: globalStyle?.transform?.x || 0,
        offsetY: globalStyle?.transform?.y || preferences?.defaultVerticalOffset || 0,
        scale: globalStyle?.transform?.scale ?? 1,
        rotation: globalStyle?.transform?.rotation || 0,
        opacity: globalStyle?.transform?.opacity ?? 1,

        // Effects
        effect: globalStyle?.effect || preferences?.defaultEffect || 'none',
        animation: globalStyle?.animation || 'default',
        animationDuration: globalStyle?.animationDuration, // Optional, can be undefined

        // Stroke
        strokeWidth: globalStyle?.stroke?.width || 0,
        strokeColor: globalStyle?.stroke?.color || '#000000',
        shadowBlur: globalStyle?.shadow?.blur || 0,
        shadowX: globalStyle?.shadow?.x || 0,
        shadowY: globalStyle?.shadow?.y || 0,
        shadowColor: globalStyle?.shadow?.color || '#000000',
    }), [globalStyle]);

    const handleChange = useCallback((key, value) => {
        // Merge with current values
        const newValues = { ...values, [key]: value };

        // Construct globalStyle object
        const newStyle = { ...globalStyle };

        // Font
        newStyle.font = {
            family: newValues.fontFamily,
            size: newValues.fontSize
        };

        // Transform (Global offset/scale/rot)
        newStyle.transform = {
            x: newValues.offsetX ?? 0,
            y: newValues.offsetY ?? 0,
            scale: newValues.scale ?? 1,
            rotation: newValues.rotation ?? 0,
            opacity: newValues.opacity ?? 1
        };

        // Effect & Animation
        newStyle.effect = newValues.effect || 'none';
        newStyle.animation = newValues.animation || 'default';
        if (newValues.animationDuration) {
            newStyle.animationDuration = newValues.animationDuration;
        } else {
            delete newStyle.animationDuration;
        }

        // Stroke
        if (newValues.strokeWidth > 0) {
            newStyle.stroke = {
                width: newValues.strokeWidth,
                color: newValues.strokeColor
            };
        } else {
            delete newStyle.stroke;
        }

        // Shadow
        if (newValues.shadowBlur > 0 || newValues.shadowX !== 0 || newValues.shadowY !== 0) {
            newStyle.shadow = {
                blur: newValues.shadowBlur,
                x: newValues.shadowX,
                y: newValues.shadowY,
                color: newValues.shadowColor
            };
        } else {
            delete newStyle.shadow;
        }

        updateState({ globalStyle: newStyle }, 'Update Global Style');
    }, [globalStyle, values, updateState]);

    return (
        <Card className="h-full bg-card/50 backdrop-blur-sm border-border/50 flex flex-col">
            <CardHeader className="py-3 px-4 border-b border-border/50">
                <CardTitle className="text-sm font-medium flex items-center gap-2">
                    <Globe className="w-4 h-4 text-primary" />
                    Global Style
                </CardTitle>
            </CardHeader>
            <CardContent className="flex-1 p-0 overflow-hidden">
                <div className="h-full overflow-y-auto p-4">
                    <StyleEditor
                        mode="global"
                        values={values}
                        onChange={handleChange}
                        availableFonts={availableFonts}
                    />
                </div>
            </CardContent>
        </Card>
    );
};

export default GlobalStyleEditor;
