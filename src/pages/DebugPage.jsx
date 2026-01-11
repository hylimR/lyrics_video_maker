import { useEffect } from 'react';
import { useAppStore, initStore } from '@/store/useAppStore';
import JsonTree from '@/components/ui/JsonTree';

const DebugPage = () => {
    const state = useAppStore();

    useEffect(() => {
        const cleanup = initStore();
        return cleanup;
    }, []);

    return (
        <div className="h-screen w-full bg-black text-gray-200 flex flex-col overflow-hidden">
            <header className="flex-none p-4 border-b border-white/10 flex justify-between items-center bg-[#111]">
                <h1 className="text-sm font-bold text-green-400 font-mono tracking-wider">
                    APP_STATE_DEBUGGER
                </h1>
                <div className="text-xs text-gray-500 font-mono">
                    {new Date().toISOString()}
                </div>
            </header>

            <main className="flex-1 overflow-auto p-4 custom-scrollbar">
                <JsonTree data={state} />
            </main>
        </div>
    );
};

export default DebugPage;
