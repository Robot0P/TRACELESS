import React from 'react';

const SecurityShield: React.FC = () => {
    return (
        <div className="relative w-[400px] h-[400px] flex items-center justify-center">
            {/* 3D Chip Base Container - Top Down View (Flat) */}
            <div className="relative w-64 h-64 perspective-1000">

                {/* Outer Tech Ring / Circuit Base */}
                <div className="absolute inset-[-40px] border border-accent/20 rounded-lg animate-pulse-slow"></div>
                <div className="absolute inset-[-20px] border border-accent/10 rounded-lg"></div>

                {/* Animated Circuit Lines (SVG Overlay) */}
                <svg className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[180%] h-[180%] pointer-events-none opacity-40">
                    <g className="animate-spin-slow origin-center">
                        <circle cx="50%" cy="50%" r="40%" fill="none" stroke="currentColor" strokeWidth="1" className="text-accent" strokeDasharray="10 10" />
                    </g>
                    <g className="animate-reverse-spin origin-center">
                        <circle cx="50%" cy="50%" r="30%" fill="none" stroke="currentColor" strokeWidth="1" className="text-accent/50" strokeDasharray="5 5" />
                    </g>
                </svg>

                {/* Main Chip Platform */}
                <div className="absolute inset-0 bg-gray-900 border border-gray-700 rounded-lg shadow-2xl overflow-hidden flex items-center justify-center backdrop-blur-md bg-opacity-80">

                    {/* Inner Grid Pattern */}
                    <div className="absolute inset-0 opacity-20"
                        style={{ backgroundImage: 'linear-gradient(#D9943F 1px, transparent 1px), linear-gradient(90deg, #D9943F 1px, transparent 1px)', backgroundSize: '20px 20px' }}>
                    </div>

                    {/* Core Pedestal */}
                    <div className="w-40 h-40 bg-gray-800 border border-gray-600 rounded-lg shadow-inner flex items-center justify-center relative override-z-index">

                        {/* The Shield Container */}
                        <div className="relative z-10">
                            <div className="animate-float">
                                {/* Shield Glow */}
                                <div className="absolute inset-0 bg-accent/30 blur-xl rounded-full animate-pulse"></div>

                                {/* Shield SVG */}
                                <svg width="100" height="120" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="0" className="text-accent drop-shadow-[0_0_15px_rgba(217,148,63,0.8)]">
                                    <path fill="#D9943F" d="M12 2L3 7V13C3 18.5 7 21 12 22C17 21 21 18.5 21 13V7L12 2Z" />
                                    <path fill="rgba(255,255,255,0.2)" d="M12 2L3 7V13C3 18.5 7 21 12 22V2Z" />
                                    {/* Add a checkmark or symbol for detail */}
                                    <path d="M9 12l2 2 4-4" stroke="rgba(255,255,255,0.8)" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" />
                                </svg>
                            </div>
                        </div>

                        {/* Scanning Laser Effect */}
                        <div className="absolute inset-0 bg-gradient-to-b from-transparent via-accent/20 to-transparent w-full h-[30%] animate-scan top-0 z-20"></div>
                    </div>
                </div>

                {/* Corner Accents */}
                <div className="absolute top-0 left-0 w-4 h-4 border-t-2 border-l-2 border-accent"></div>
                <div className="absolute top-0 right-0 w-4 h-4 border-t-2 border-r-2 border-accent"></div>
                <div className="absolute bottom-0 left-0 w-4 h-4 border-b-2 border-l-2 border-accent"></div>
                <div className="absolute bottom-0 right-0 w-4 h-4 border-b-2 border-r-2 border-accent"></div>

            </div>
        </div>
    );
};

export default SecurityShield;
