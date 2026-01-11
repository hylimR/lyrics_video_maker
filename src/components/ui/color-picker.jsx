import React, { useState, useEffect } from "react";
import { HexColorPicker, HexAlphaColorPicker } from "react-colorful";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { cn } from "@/lib/utils";

/**
 * ColorPicker - A robust color picker component using react-colorful.
 *
 * @param {string} value - The current color value (hex string).
 * @param {function} onChange - Callback when color changes.
 * @param {string} label - Optional label for the input.
 * @param {boolean} alpha - Whether to enable alpha channel support.
 */
const ColorPicker = ({ value, onChange, label, alpha = true, className }) => {
    // Initialize state with value prop
    const [color, setColor] = useState(value || "#000000");

    // Sync state with value prop when it changes
    useEffect(() => {
        if (value && value !== color) {
            setColor(value);
        }
    // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [value]);

    const handleChange = (newColor) => {
        setColor(newColor);
        onChange(newColor);
    };

    const Picker = alpha ? HexAlphaColorPicker : HexColorPicker;

    return (
        <div className={cn("space-y-3", className)}>
            {label && <Label>{label}</Label>}
            <div className="flex gap-2">
                <Popover>
                    <PopoverTrigger asChild>
                        <div
                            className="relative w-10 h-10 rounded-md overflow-hidden border border-input shadow-sm cursor-pointer hover:ring-2 hover:ring-ring transition-all"
                            style={{ backgroundColor: color }}
                        >
                            {/* Checkerboard background for transparency visualization */}
                            <div className="absolute inset-0 -z-10 bg-[url('data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAAMUlEQVQ4T2NkYGAQYcAP3uCTZhw1gGGYhAGBZIA/nYDCgBDAm9BGDWAAJyRCgLaBCAAgXwixzAS0pgAAAABJRU5ErkJggg==')] opacity-20" />
                        </div>
                    </PopoverTrigger>
                    <PopoverContent className="w-auto p-3" align="start">
                        <Picker color={color} onChange={handleChange} />
                    </PopoverContent>
                </Popover>

                <Input
                    value={color}
                    onChange={(e) => handleChange(e.target.value)}
                    className="flex-1 font-mono uppercase"
                    maxLength={alpha ? 9 : 7}
                />
            </div>
        </div>
    );
};

export default ColorPicker;
