/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{svelte,js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        apple: {
          primary: '#0066cc',
          'primary-focus': '#0071e3',
          'primary-on-dark': '#2997ff',
          ink: '#1d1d1f',
          body: '#1d1d1f',
          'body-on-dark': '#ffffff',
          'body-muted': '#cccccc',
          'ink-muted-80': '#333333',
          'ink-muted-48': '#7a7a7a',
          'divider-soft': '#f0f0f0',
          hairline: '#e0e0e0',
          'hairline-dark': '#3a3a3c',
          canvas: '#ffffff',
          'canvas-parchment': '#f5f5f7',
          'surface-pearl': '#fafafc',
          'surface-tile-1': '#272729',
          'surface-tile-2': '#2a2a2c',
          'surface-tile-3': '#252527',
          'surface-black': '#000000',
          'surface-chip-translucent': '#d2d2d7',
          'on-primary': '#ffffff',
          'on-dark': '#ffffff',
        },
        brand: {
          50: '#f0f9ff',
          500: '#0066cc',
          600: '#0071e3',
          900: '#1d1d1f'
        }
      },
      borderRadius: {
        'apple-xs': '5px',
        'apple-sm': '8px',
        'apple-md': '11px',
        'apple-lg': '18px',
        'apple-pill': '9999px',
        'apple-full': '9999px',
      },
      fontFamily: {
        sans: ['"SF Pro Text"', 'system-ui', '-apple-system', 'BlinkMacSystemFont', '"Segoe UI"', 'Roboto', 'sans-serif'],
        'apple-display': ['"SF Pro Display"', 'system-ui', '-apple-system', 'BlinkMacSystemFont', '"Segoe UI"', 'sans-serif'],
        'apple-text': ['"SF Pro Text"', 'system-ui', '-apple-system', 'BlinkMacSystemFont', '"Segoe UI"', 'sans-serif'],
      },
      letterSpacing: {
        'apple-tight': '-0.02em',
        'apple-hero': '-0.28px',
        'apple-headline': '-0.374px',
        'apple-caption': '-0.224px',
        'apple-fine': '-0.12px',
      }
    },
  },
  plugins: [],
}
