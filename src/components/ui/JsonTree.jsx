import React, { useState } from 'react';
import { ChevronRight, ChevronDown, Copy } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';

/**
 * JsonTree - A recursive JSON viewer with collapsible nodes
 */
const JsonTree = ({ data, level = 0, name = null, isLast = true }) => {
    const [expanded, setExpanded] = useState(level < 2); // Default expand top 2 levels
    const [copied, setCopied] = useState(false);

    const isObject = data !== null && typeof data === 'object';
    const isArray = Array.isArray(data);
    const isEmpty = isObject && Object.keys(data).length === 0;

    const handleCopy = (e) => {
        e.stopPropagation();
        navigator.clipboard.writeText(JSON.stringify(data, null, 2));
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };

    if (!isObject) {
        return (
            <div className="pl-4 font-mono text-xs flex items-center hover:bg-white/5 py-0.5 rounded">
                {name && <span className="text-blue-400 mr-2 opacity-80">{name}:</span>}
                <span className={cn(
                    "break-all",
                    typeof data === 'string' ? "text-green-300" :
                        typeof data === 'number' ? "text-orange-300" :
                            typeof data === 'boolean' ? "text-purple-300" :
                                "text-gray-400"
                )}>
                    {JSON.stringify(data)}
                </span>
                {!isLast && <span className="text-gray-500">,</span>}
            </div>
        );
    }

    return (
        <div className="font-mono text-xs">
            <div
                className={cn(
                    "flex items-center cursor-pointer hover:bg-white/5 py-0.5 rounded group select-none",
                    level > 0 && "pl-4"
                )}
                onClick={() => !isEmpty && setExpanded(!expanded)}
            >
                <div className="w-4 h-4 mr-1 flex items-center justify-center text-gray-500">
                    {!isEmpty && (expanded ? <ChevronDown className="w-3 h-3" /> : <ChevronRight className="w-3 h-3" />)}
                </div>

                {name && <span className="text-blue-400 mr-2 opacity-80">{name}:</span>}

                <span className="text-yellow-500 opacity-80">
                    {isArray ? '[' : '{'}
                </span>

                {!expanded && !isEmpty && (
                    <span className="text-gray-500 mx-2 text-[10px] italic">
                        {isArray ? `${data.length} items` : `${Object.keys(data).length} keys`} ...
                    </span>
                )}

                {isEmpty && (
                    <span className="text-gray-500 mx-1"></span>
                )}

                {(!expanded || isEmpty) && (
                    <span className="text-yellow-500 opacity-80">
                        {isArray ? ']' : '}'}
                        {!isLast && ','}
                    </span>
                )}

                {/* Quick actions for objects */}
                <div className="ml-auto opacity-0 group-hover:opacity-100 transition-opacity pr-2 flex gap-2">
                    <Button
                        variant="ghost"
                        size="icon"
                        className="h-4 w-4 text-gray-400 hover:text-white"
                        onClick={handleCopy}
                        title="Copy JSON"
                    >
                        <Copy className="w-3 h-3" />
                    </Button>
                </div>
                {copied && <span className="text-green-500 text-[10px] ml-2">Copied!</span>}
            </div>

            {expanded && !isEmpty && (
                <div className="border-l border-white/10 ml-2">
                    {Object.entries(data).map(([key, value], index, arr) => (
                        <JsonTree
                            key={key}
                            name={isArray ? null : key}
                            data={value}
                            level={level + 1}
                            isLast={index === arr.length - 1}
                        />
                    ))}
                    <div className="pl-4 text-yellow-500 opacity-80 hover:bg-white/5 py-0.5 rounded">
                        {isArray ? ']' : '}'}
                        {!isLast && ','}
                    </div>
                </div>
            )}
        </div>
    );
};

export default JsonTree;
