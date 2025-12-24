/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        bg: {
          DEFAULT: '#1F2024',
          sidebar: '#18191D',
          card: '#2B2C30',
        },
        accent: {
          DEFAULT: '#D9943F',
          hover: '#c28538',
        },
        success: '#52C41A',
        warning: '#FAAD14',
        danger: '#F5222D',
        text: {
          main: '#E0E0E0',
          secondary: '#9CA3AF',
        }
      },
      animation: {
        'fadeIn': 'fadeIn 0.5s ease-out forwards',
        'slideInUp': 'slideInUp 0.5s ease-out forwards',
        'slideInDown': 'slideInDown 0.5s ease-out forwards',
        'pulse-slow': 'pulse 4s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'float': 'float 3s ease-in-out infinite',
        'scan': 'scan 3s linear infinite',
        'scanPulse': 'scanPulse 1.5s ease-in-out infinite',
        'spin-slow': 'spin 10s linear infinite',
        'reverse-spin': 'spin 15s linear infinite reverse',
        'radar-spin': 'spin 4s linear infinite',
        'radar-pulse': 'radarPulse 2s ease-in-out infinite',
        'radar-pulse-slow': 'radarPulse 2.5s ease-in-out infinite',
        'radar-pulse-slower': 'radarPulse 1.8s ease-in-out infinite',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideInUp: {
          '0%': { opacity: '0', transform: 'translateY(20px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        slideInDown: {
          '0%': { opacity: '0', transform: 'translateY(-20px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        float: {
          '0%, 100%': { transform: 'translateY(0)' },
          '50%': { transform: 'translateY(-10px)' },
        },
        scan: {
          '0%': { top: '-20%', opacity: '0' },
          '50%': { opacity: '1' },
          '100%': { top: '120%', opacity: '0' },
        },
        scanPulse: {
          '0%, 100%': { transform: 'scale(1)', opacity: '1' },
          '50%': { transform: 'scale(1.05)', opacity: '0.8' },
        },
        radarPulse: {
          '0%, 100%': { opacity: '0.8' },
          '50%': { opacity: '1' },
        },
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', '-apple-system', 'sans-serif'],
      },
      maxWidth: {
        'content': '1200px',
      }
    },
  },
  plugins: [],
}
