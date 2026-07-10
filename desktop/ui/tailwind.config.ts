/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'loon-bg': '#090B10',
        'loon-fg': '#E2E8F0',
        'loon-primary': '#7DD3FC',
        'loon-secondary': '#64748B',
        'loon-border': '#1E293B',
        'loon-surface': '#111827',
        'loon-accent': '#38BDF8',
        'loon-muted': '#94A3B8',
        'loon-success': '#22C55E',
        'loon-warning': '#F59E0B',
        'loon-error': '#EF4444',
        'loon-info': '#38BDF8',
      },
      borderRadius: {
        'loon-sm': '4px',
        'loon-md': '8px',
        'loon-lg': '12px',
        'loon-full': '9999px',
      },
      spacing: {
        'loon-xs': '4px',
        'loon-sm': '8px',
        'loon-md': '16px',
        'loon-lg': '24px',
        'loon-xl': '32px',
        'loon-xxl': '48px',
      },
    },
  },
  plugins: [],
}
