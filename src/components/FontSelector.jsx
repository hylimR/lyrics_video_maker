import React, { useState, useEffect, useMemo, useRef } from 'react';
import { useAppStore } from '@/store/useAppStore';
import { Button } from "@/components/ui/button";
import {
    Command,
    CommandEmpty,
    CommandGroup,
    CommandInput,
    CommandItem,
    CommandList,
    CommandSeparator
} from "@/components/ui/command";
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
} from "@/components/ui/popover";
import { Check, ChevronsUpDown, Star, Globe, Loader2 } from "lucide-react";
import { cn } from "@/lib/utils";

/**
 * FontSelector - A searchable, grouped dropdown for font selection using Shadcn.
 */
const FontSelector = ({
    value,
    onChange,
    onPreview,
    fonts = [],
    placeholder = "Select font...",
    disabled = false,
    allowInherited = false
}) => {
    const [open, setOpen] = useState(false);
    const [filterChinese, setFilterChinese] = useState(false);

    // Preview debounce logic
    const previewTimeoutRef = useRef(null);
    const initialValueRef = useRef(value);

    // Load favorites from local storage
    const [favorites, setFavorites] = useState(() => {
        try {
            return JSON.parse(localStorage.getItem('lyric-video-fav-fonts') || '[]');
        } catch { return []; }
    });

    // Global loading state
    const loadingFonts = useAppStore(state => state.loadingFonts);

    // Persist favorites
    useEffect(() => {
        localStorage.setItem('lyric-video-fav-fonts', JSON.stringify(favorites));
    }, [favorites]);

    // Handle preview
    const handlePreview = (fontName) => {
        if (!onPreview) return;
        if (previewTimeoutRef.current) clearTimeout(previewTimeoutRef.current);
        previewTimeoutRef.current = setTimeout(() => {
            onPreview(fontName);
        }, 150);
    };

    // Reset preview when closing without selection
    const handleClose = (isOpen) => {
        if (!isOpen && onPreview) {
            if (previewTimeoutRef.current) clearTimeout(previewTimeoutRef.current);
            onPreview(initialValueRef.current); // Revert to what it was when opened
        }
        if (isOpen) {
            initialValueRef.current = value;
        }
        setOpen(isOpen);
    };

    const toggleFavorite = (e, family) => {
        e.stopPropagation();
        e.preventDefault(); // Prevent closing dropdown
        setFavorites(prev =>
            prev.includes(family)
                ? prev.filter(f => f !== family)
                : [...prev, family]
        );
    };

    // Filter and Group Logic
    const { favoriteFonts, otherFonts } = useMemo(() => {
        let filtered = fonts;
        if (filterChinese) {
            filtered = filtered.filter(f => f.supportsChinese);
        }

        const favs = [];
        const others = [];

        // Group variants by family
        const familyMap = new Map();

        filtered.forEach(font => {
            const family = font.family || font.name;
            if (!familyMap.has(family)) {
                familyMap.set(family, { family, variants: [] });
            }
            familyMap.get(family).variants.push(font);
        });

        // Split into favorites and others
        Array.from(familyMap.values()).forEach(group => {
            if (favorites.includes(group.family)) {
                favs.push(group);
            } else {
                others.push(group);
            }
        });

        // Sort alphabetically
        favs.sort((a, b) => a.family.localeCompare(b.family));
        others.sort((a, b) => a.family.localeCompare(b.family));

        return { favoriteFonts: favs, otherFonts: others };
    }, [fonts, filterChinese, favorites]);

    return (
        <Popover open={open} onOpenChange={handleClose}>
            <PopoverTrigger asChild>
                <Button
                    variant="outline"
                    role="combobox"
                    aria-expanded={open}
                    disabled={disabled}
                    className="w-full justify-between font-normal"
                >
                    <span className="truncate">
                        {value || placeholder}
                    </span>
                    <ChevronsUpDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
                </Button>
            </PopoverTrigger>
            <PopoverContent className="w-[300px] p-0 bg-background/90 backdrop-blur-xl border-white/10" align="start">
                <Command>
                    {/* Toolbar for Chinese Filter */}
                    <div className="flex items-center p-2 border-b border-border/50 bg-muted/20 gap-2">
                        <Button
                            variant={filterChinese ? "secondary" : "ghost"}
                            size="sm"
                            className={cn("h-7 text-xs flex-1", filterChinese && "bg-primary/20 text-primary hover:bg-primary/30")}
                            onClick={() => setFilterChinese(!filterChinese)}
                        >
                            <Globe className="w-3 h-3 mr-1.5" />
                            Only Chinese Fonts
                        </Button>
                    </div>

                    <CommandInput placeholder="Search fonts..." />
                    <CommandList className="max-h-[300px]">
                        <CommandEmpty>No fonts found.</CommandEmpty>

                        {/* Default Option */}
                        {allowInherited && (
                            <CommandGroup heading="System">
                                <CommandItem
                                    value="inherited_default_option"
                                    onSelect={() => {
                                        onChange(null);
                                        initialValueRef.current = null;
                                        setOpen(false);
                                    }}
                                    className="flex items-center gap-2 cursor-pointer"
                                >
                                    <Check className={cn("mr-2 h-4 w-4", !value ? "opacity-100" : "opacity-0")} />
                                    <span>Default (Inherited)</span>
                                </CommandItem>
                            </CommandGroup>
                        )}

                        {/* Favorites Group */}
                        {favoriteFonts.length > 0 && (
                            <CommandGroup heading="Favorites">
                                {favoriteFonts.map(group => (
                                    group.variants.map(font => (
                                        <CommandItem
                                            key={font.name}
                                            value={font.name} // Command uses this for filtering
                                            onSelect={(currentValue) => {
                                                onChange(currentValue);
                                                initialValueRef.current = currentValue; // Commit change
                                                setOpen(false);
                                            }}
                                            onMouseEnter={() => handlePreview(font.name)}
                                            className="flex items-center justify-between group cursor-pointer"
                                        >
                                            <div className="flex items-center gap-2 truncate flex-1">
                                                <Check
                                                    className={cn(
                                                        "mr-2 h-4 w-4",
                                                        value === font.name ? "opacity-100" : "opacity-0"
                                                    )}
                                                />
                                                <span className="truncate">
                                                    {font.family} <span className="text-muted-foreground text-xs ml-1">{font.style !== 'Regular' ? font.style : ''}</span>
                                                </span>
                                            </div>
                                            <div className="flex items-center gap-1">
                                                {loadingFonts.has(font.name) && <Loader2 className="w-3 h-3 animate-spin text-muted-foreground" />}
                                                <Button
                                                    variant="ghost"
                                                    size="icon"
                                                    className="h-6 w-6 text-yellow-500/50 hover:text-yellow-500 hover:bg-transparent opacity-0 group-hover:opacity-100 transition-opacity"
                                                    onClick={(e) => toggleFavorite(e, group.family)}
                                                >
                                                    <Star className="w-3 h-3 fill-current" />
                                                </Button>
                                            </div>
                                        </CommandItem>
                                    ))
                                ))}
                            </CommandGroup>
                        )}

                        {(favoriteFonts.length > 0 && otherFonts.length > 0) && <CommandSeparator />}

                        {/* All Fonts Group */}
                        <CommandGroup heading="All Fonts">
                            {otherFonts.map(group => (
                                group.variants.map(font => (
                                    <CommandItem
                                        key={font.name}
                                        value={font.name}
                                        onSelect={(currentValue) => {
                                            onChange(currentValue);
                                            initialValueRef.current = currentValue;
                                            setOpen(false);
                                        }}
                                        onMouseEnter={() => handlePreview(font.name)}
                                        className="flex items-center justify-between group cursor-pointer"
                                    >
                                        <div className="flex items-center gap-2 truncate flex-1">
                                            <Check
                                                className={cn(
                                                    "mr-2 h-4 w-4",
                                                    value === font.name ? "opacity-100" : "opacity-0"
                                                )}
                                            />
                                            <span className="truncate">
                                                {font.family} <span className="text-muted-foreground text-xs ml-1">{font.style !== 'Regular' ? font.style : ''}</span>
                                            </span>
                                        </div>
                                        <div className="flex items-center gap-1">
                                            {loadingFonts.has(font.name) && <Loader2 className="w-3 h-3 animate-spin text-muted-foreground" />}
                                            <Button
                                                variant="ghost"
                                                size="icon"
                                                className="h-6 w-6 text-muted-foreground hover:text-yellow-500 hover:bg-transparent opacity-0 group-hover:opacity-100 transition-opacity"
                                                onClick={(e) => toggleFavorite(e, group.family)}
                                            >
                                                <Star className="w-3 h-3" />
                                            </Button>
                                        </div>
                                    </CommandItem>
                                ))
                            ))}
                        </CommandGroup>
                    </CommandList>
                </Command>
            </PopoverContent>
        </Popover>
    );
};

export default FontSelector;
