# OpenPaste Design System

## Overview

OpenPaste uses a modern, clean design system built on Tailwind CSS. The design prioritizes clarity, efficiency, and accessibility while maintaining a professional appearance suitable for a productivity tool.

## Design Principles

1. **Clarity First:** Information should be immediately understandable
2. **Efficiency:** Minimize clicks and keystrokes
3. **Consistency:** Uniform patterns across the application
4. **Accessibility:** Usable by everyone
5. **Performance:** Fast and responsive
6. **Keyboard-First:** Fully navigable without mouse

## Color System

### Primary Colors

**Blue (Accent):**
```css
--blue-50: #eff6ff;
--blue-100: #dbeafe;
--blue-200: #bfdbfe;
--blue-300: #93c5fd;
--blue-400: #60a5fa;
--blue-500: #3b82f6;  /* Primary */
--blue-600: #2563eb;
--blue-700: #1d4ed8;
--blue-800: #1e40af;
--blue-900: #1e3a8a;
```

### Neutral Colors

**Gray:**
```css
--gray-50: #f9fafb;
--gray-100: #f3f4f6;
--gray-200: #e5e7eb;
--gray-300: #d1d5db;
--gray-400: #9ca3af;
--gray-500: #6b7280;
--gray-600: #4b5563;
--gray-700: #374151;
--gray-800: #1f2937;
--gray-900: #111827;
```

### Semantic Colors

**Success (Green):**
```css
--green-500: #22c55e;
--green-600: #16a34a;
```

**Warning (Yellow):**
```css
--yellow-500: #eab308;
--yellow-600: #ca8a04;
```

**Error (Red):**
```css
--red-500: #ef4444;
--red-600: #dc2626;
```

**Info (Cyan):**
```css
--cyan-500: #06b6d4;
--cyan-600: #0891b2;
```

### Dark Mode

**Backgrounds:**
```css
--bg-primary: #1f2937;      /* gray-800 */
--bg-secondary: #111827;    /* gray-900 */
--bg-tertiary: #374151;     /* gray-700 */
```

**Text:**
```css
--text-primary: #f9fafb;    /* gray-50 */
--text-secondary: #9ca3af;  /* gray-400 */
--text-tertiary: #6b7280;   /* gray-500 */
```

**Borders:**
```css
--border: #374151;          /* gray-700 */
--border-light: #4b5563;    /* gray-600 */
```

### Light Mode

**Backgrounds:**
```css
--bg-primary: #ffffff;
--bg-secondary: #f9fafb;    /* gray-50 */
--bg-tertiary: #f3f4f6;     /* gray-100 */
```

**Text:**
```css
--text-primary: #111827;    /* gray-900 */
--text-secondary: #6b7280;   /* gray-500 */
--text-tertiary: #9ca3af;   /* gray-400 */
```

**Borders:**
```css
--border: #e5e7eb;          /* gray-200 */
--border-light: #d1d5db;    /* gray-300 */
```

## Typography

### Font Families

**UI Font:** Inter
```css
font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
```

**Code Font:** JetBrains Mono
```css
font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
```

### Font Scale

**Size Scale:**
```css
--text-xs: 0.75rem;    /* 12px */
--text-sm: 0.875rem;   /* 14px */
--text-base: 1rem;     /* 16px */
--text-lg: 1.125rem;   /* 18px */
--text-xl: 1.25rem;    /* 20px */
--text-2xl: 1.5rem;    /* 24px */
--text-3xl: 1.875rem;  /* 30px */
--text-4xl: 2.25rem;   /* 36px */
```

**Line Height:**
```css
--leading-tight: 1.25;
--leading-normal: 1.5;
--leading-relaxed: 1.625;
```

**Font Weight:**
```css
--font-light: 300;
--font-normal: 400;
--font-medium: 500;
--font-semibold: 600;
--font-bold: 700;
```

### Typography Usage

**Headings:**
- H1: text-3xl, font-bold, leading-tight
- H2: text-2xl, font-semibold, leading-tight
- H3: text-xl, font-semibold, leading-tight
- H4: text-lg, font-medium, leading-normal

**Body:**
- Body: text-base, font-normal, leading-normal
- Small: text-sm, font-normal, leading-normal
- Caption: text-xs, font-normal, leading-normal

**Code:**
- Inline: text-sm, code font
- Block: text-sm, code font, leading-relaxed

## Spacing

### Spacing Scale

```css
--space-0: 0;
--space-1: 0.25rem;   /* 4px */
--space-2: 0.5rem;    /* 8px */
--space-3: 0.75rem;   /* 12px */
--space-4: 1rem;      /* 16px */
--space-5: 1.25rem;   /* 20px */
--space-6: 1.5rem;    /* 24px */
--space-8: 2rem;      /* 32px */
--space-10: 2.5rem;   /* 40px */
--space-12: 3rem;     /* 48px */
--space-16: 4rem;     /* 64px */
--space-20: 5rem;     /* 80px */
--space-24: 6rem;     /* 96px */
```

### Spacing Guidelines

**Component Padding:**
- Small: space-2 to space-3
- Medium: space-3 to space-4
- Large: space-4 to space-6

**Gap Between Elements:**
- Tight: space-2
- Normal: space-4
- Loose: space-6

**Section Spacing:**
- Small sections: space-8
- Medium sections: space-12
- Large sections: space-16

## Layout

### Container

**Max Width:**
```css
--container-sm: 640px;
--container-md: 768px;
--container-lg: 1024px;
--container-xl: 1280px;
--container-2xl: 1536px;
```

**Padding:**
```css
--container-padding: space-4;
```

### Grid

**Columns:** 12-column grid

**Gutter:**
```css
--grid-gap: space-4;
```

**Breakpoints:**
```css
--breakpoint-sm: 640px;
--breakpoint-md: 768px;
--breakpoint-lg: 1024px;
--breakpoint-xl: 1280px;
--breakpoint-2xl: 1536px;
```

## Components

### Buttons

**Primary Button:**
```css
background: var(--blue-500);
color: white;
padding: space-2 space-4;
border-radius: 0.375rem;
font-weight: font-medium;
transition: all 150ms;

&:hover {
  background: var(--blue-600);
}

&:active {
  background: var(--blue-700);
}
```

**Secondary Button:**
```css
background: transparent;
color: var(--text-primary);
border: 1px solid var(--border);
padding: space-2 space-4;
border-radius: 0.375rem;
font-weight: font-medium;
transition: all 150ms;

&:hover {
  background: var(--bg-secondary);
}
```

**Ghost Button:**
```css
background: transparent;
color: var(--text-primary);
padding: space-2 space-4;
border-radius: 0.375rem;
font-weight: font-medium;
transition: all 150ms;

&:hover {
  background: var(--bg-tertiary);
}
```

**Icon Button:**
```css
background: transparent;
color: var(--text-secondary);
padding: space-2;
border-radius: 0.375rem;
transition: all 150ms;

&:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}
```

### Inputs

**Text Input:**
```css
background: var(--bg-primary);
border: 1px solid var(--border);
border-radius: 0.375rem;
padding: space-2 space-3;
color: var(--text-primary);
font-size: text-base;
transition: border-color 150ms;

&:focus {
  outline: none;
  border-color: var(--blue-500);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

&::placeholder {
  color: var(--text-tertiary);
}
```

**Search Input:**
```css
background: var(--bg-secondary);
border: 1px solid var(--border);
border-radius: 0.5rem;
padding: space-3 space-4;
color: var(--text-primary);
font-size: text-base;
transition: all 150ms;

&:focus {
  outline: none;
  border-color: var(--blue-500);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}
```

### Cards

**Card:**
```css
background: var(--bg-primary);
border: 1px solid var(--border);
border-radius: 0.5rem;
padding: space-4;
transition: all 150ms;

&:hover {
  border-color: var(--border-light);
}
```

**Card (Selected):**
```css
background: var(--bg-secondary);
border-color: var(--blue-500);
box-shadow: 0 0 0 1px var(--blue-500);
```

### Lists

**List Item:**
```css
padding: space-3 space-4;
border-bottom: 1px solid var(--border);
transition: background 100ms;

&:hover {
  background: var(--bg-secondary);
}

&:last-child {
  border-bottom: none;
}
```

**List Item (Selected):**
```css
background: var(--blue-50);
border-left: 3px solid var(--blue-500);
```

### Badges

**Badge:**
```css
background: var(--bg-tertiary);
color: var(--text-primary);
padding: space-1 space-2;
border-radius: 9999px;
font-size: text-xs;
font-weight: font-medium;
```

**Badge (Primary):**
```css
background: var(--blue-500);
color: white;
```

**Badge (Success):**
```css
background: var(--green-500);
color: white;
```

**Badge (Warning):**
```css
background: var(--yellow-500);
color: white;
```

**Badge (Error):**
```css
background: var(--red-500);
color: white;
```

### Chips

**Chip:**
```css
background: var(--bg-tertiary);
border: 1px solid var(--border);
padding: space-1 space-3;
border-radius: 9999px;
font-size: text-sm;
display: inline-flex;
align-items: center;
gap: space-2;
```

**Chip (Removable):**
```css
& .remove {
  cursor: pointer;
  opacity: 0.6;
  transition: opacity 150ms;
  
  &:hover {
    opacity: 1;
  }
}
```

### Modals

**Modal Overlay:**
```css
background: rgba(0, 0, 0, 0.5);
position: fixed;
inset: 0;
z-index: 50;
```

**Modal Content:**
```css
background: var(--bg-primary);
border-radius: 0.75rem;
max-width: 32rem;
width: 100%;
padding: space-6;
box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1);
```

### Dropdowns

**Dropdown Menu:**
```css
background: var(--bg-primary);
border: 1px solid var(--border);
border-radius: 0.5rem;
padding: space-2;
min-width: 12rem;
box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
```

**Dropdown Item:**
```css
padding: space-2 space-3;
border-radius: 0.25rem;
cursor: pointer;
transition: background 100ms;

&:hover {
  background: var(--bg-secondary);
}
```

### Tooltips

**Tooltip:**
```css
background: var(--gray-900);
color: white;
padding: space-2 space-3;
border-radius: 0.375rem;
font-size: text-sm;
max-width: 20rem;
z-index: 100;
```

### Progress Indicators

**Progress Bar:**
```css
background: var(--bg-tertiary);
border-radius: 9999px;
height: 0.5rem;
overflow: hidden;
```

**Progress Fill:**
```css
background: var(--blue-500);
height: 100%;
transition: width 300ms ease;
```

**Spinner:**
```css
border: 2px solid var(--border);
border-top-color: var(--blue-500);
border-radius: 50%;
width: 1.5rem;
height: 1.5rem;
animation: spin 1s linear infinite;
```

## Icons

### Icon Library

**Library:** Lucide React

**Icon Size:**
```css
--icon-xs: 1rem;      /* 16px */
--icon-sm: 1.25rem;   /* 20px */
--icon-base: 1.5rem;  /* 24px */
--icon-lg: 1.75rem;   /* 28px */
--icon-xl: 2rem;      /* 32px */
```

### Common Icons

**Clipboard:**
- Clipboard: `clipboard`
- Copy: `copy`
- Paste: `clipboard-check`

**Navigation:**
- Search: `search`
- Settings: `settings`
- Home: `home`
- Back: `arrow-left`
- Forward: `arrow-right`

**Actions:**
- Pin: `pin`
- Favorite: `star`
- Delete: `trash-2`
- Edit: `edit-2`
- Add: `plus`

**Status:**
- Lock: `lock`
- Unlock: `unlock`
- Sync: `refresh-cw`
- Check: `check`
- X: `x`

**Content Types:**
- Text: `file-text`
- Image: `image`
- File: `file`
- Code: `code`
- Link: `link`

## Shadows

### Shadow Scale

```css
--shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
--shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px 0 rgba(0, 0, 0, 0.06);
--shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
--shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05);
--shadow-xl: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
--shadow-2xl: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
```

### Shadow Usage

**Elevation:**
- Level 0: none
- Level 1: shadow-sm (cards, buttons)
- Level 2: shadow (dropdowns, tooltips)
- Level 3: shadow-md (modals, panels)
- Level 4: shadow-lg (popovers)
- Level 5: shadow-xl (dialogs)

## Border Radius

### Radius Scale

```css
--radius-none: 0;
--radius-sm: 0.125rem;   /* 2px */
--radius: 0.25rem;       /* 4px */
--radius-md: 0.375rem;   /* 6px */
--radius-lg: 0.5rem;     /* 8px */
--radius-xl: 0.75rem;    /* 12px */
--radius-2xl: 1rem;      /* 16px */
--radius-full: 9999px;
```

### Radius Usage

**Components:**
- Buttons: radius-md
- Inputs: radius-md
- Cards: radius-lg
- Badges/Chips: radius-full
- Modals: radius-xl

## Transitions

### Duration

```css
--duration-75: 75ms;
--duration-100: 100ms;
--duration-150: 150ms;
--duration-200: 200ms;
--duration-300: 300ms;
--duration-500: 500ms;
--duration-700: 700ms;
--duration-1000: 1000ms;
```

### Easing

```css
--ease-linear: linear;
--ease-in: cubic-bezier(0.4, 0, 1, 1);
--ease-out: cubic-bezier(0, 0, 0.2, 1);
--ease-in-out: cubic-bezier(0.4, 0, 0.2, 1);
```

### Transition Usage

**Hover:** duration-150, ease-out
**Focus:** duration-150, ease-out
**Modal:** duration-300, ease-out
**Page:** duration-200, ease-in-out

## Z-Index Scale

```css
--z-0: 0;
--z-10: 10;
--z-20: 20;
--z-30: 30;
--z-40: 40;
--z-50: 50;
--z-auto: auto;
```

### Z-Index Usage

- Base: z-0
- Dropdown: z-10
- Sticky: z-20
- Fixed: z-30
- Modal Backdrop: z-40
- Modal: z-50

## Animation

### Keyframes

**Fade In:**
```css
@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}
```

**Fade Out:**
```css
@keyframes fadeOut {
  from {
    opacity: 1;
  }
  to {
    opacity: 0;
  }
}
```

**Slide In Up:**
```css
@keyframes slideInUp {
  from {
    transform: translateY(100%);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}
```

**Slide In Down:**
```css
@keyframes slideInDown {
  from {
    transform: translateY(-100%);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}
```

**Scale In:**
```css
@keyframes scaleIn {
  from {
    transform: scale(0.95);
    opacity: 0;
  }
  to {
    transform: scale(1);
    opacity: 1;
  }
}
```

**Spin:**
```css
@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
```

**Pulse:**
```css
@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}
```

### Animation Usage

**Page Load:** fadeIn, duration-200
**Modal:** scaleIn, duration-300
**Dropdown:** slideInDown, duration-150
**Loading:** spin, duration-1000, infinite

## Accessibility

### Focus States

**Focus Ring:**
```css
&:focus-visible {
  outline: 2px solid var(--blue-500);
  outline-offset: 2px;
}
```

**Focus Within:**
```css
&:focus-within {
  border-color: var(--blue-500);
}
```

### Focus Management

**Skip Link:**
```css
.skip-link {
  position: absolute;
  top: -40px;
  left: 0;
  background: var(--blue-500);
  color: white;
  padding: space-2 space-4;
  z-index: 100;
  
  &:focus {
    top: 0;
  }
}
```

### ARIA Attributes

**Roles:**
- button
- navigation
- main
- dialog
- alert
- status

**Labels:**
- aria-label
- aria-labelledby
- aria-describedby

**States:**
- aria-expanded
- aria-selected
- aria-checked
- aria-disabled

### Color Contrast

**Minimum Contrast Ratios:**
- Normal text: 4.5:1
- Large text: 3:1
- UI components: 3:1

**WCAG AA Compliant:** All colors meet AA standards

**WCAG AAA Compliant:** Most colors meet AAA standards

## Responsive Design

### Breakpoints

**Mobile First:**
```css
/* Mobile (default) */
--breakpoint-mobile: 0px;

/* Tablet */
--breakpoint-tablet: 768px;

/* Desktop */
--breakpoint-desktop: 1024px;

/* Large Desktop */
--breakpoint-large: 1280px;
```

### Responsive Patterns

**Container Queries:** (Future)
- Component-based responsiveness
- Independent of viewport

**Fluid Typography:**
```css
font-size: clamp(1rem, 2.5vw, 1.25rem);
```

**Fluid Spacing:**
```css
padding: clamp(1rem, 5vw, 2rem);
```

## Dark Mode

### Implementation

**CSS Variables:**
```css
[data-theme="dark"] {
  --bg-primary: #1f2937;
  --bg-secondary: #111827;
  --text-primary: #f9fafb;
  --text-secondary: #9ca3af;
  --border: #374151;
}
```

**Automatic Detection:**
```css
@media (prefers-color-scheme: dark) {
  :root {
    --bg-primary: #1f2937;
    --bg-secondary: #111827;
    --text-primary: #f9fafb;
    --text-secondary: #9ca3af;
    --border: #374151;
  }
}
```

### Dark Mode Guidelines

**Don't Invert Colors:**
- Use specific dark mode colors
- Adjust saturation and lightness
- Maintain contrast

**Shadows in Dark Mode:**
- Reduce shadow intensity
- Use lighter shadows
- Consider no shadows for flat design

## Print Styles

```css
@media print {
  body {
    background: white;
    color: black;
  }
  
  .no-print {
    display: none;
  }
  
  a {
    text-decoration: underline;
    color: black;
  }
}
```

## Tailwind Configuration

### tailwind.config.js

```javascript
module.exports = {
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        blue: {
          50: '#eff6ff',
          500: '#3b82f6',
          600: '#2563eb',
          700: '#1d4ed8',
        },
        gray: {
          50: '#f9fafb',
          500: '#6b7280',
          900: '#111827',
        },
      },
      fontFamily: {
        sans: ['Inter', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'],
      },
      spacing: {
        '18': '4.5rem',
        '88': '22rem',
      },
    },
  },
  plugins: [
    require('@tailwindcss/forms'),
    require('@tailwindcss/typography'),
  ],
};
```

## Component Library

### Button Component

```typescript
interface ButtonProps {
  variant?: 'primary' | 'secondary' | 'ghost';
  size?: 'sm' | 'md' | 'lg';
  disabled?: boolean;
  children: React.ReactNode;
  onClick?: () => void;
}

export const Button: React.FC<ButtonProps> = ({
  variant = 'primary',
  size = 'md',
  disabled = false,
  children,
  onClick,
}) => {
  const baseStyles = 'font-medium rounded-lg transition-all duration-150';
  
  const variantStyles = {
    primary: 'bg-blue-500 text-white hover:bg-blue-600 active:bg-blue-700',
    secondary: 'bg-transparent border border-gray-300 hover:bg-gray-50',
    ghost: 'bg-transparent hover:bg-gray-100',
  };
  
  const sizeStyles = {
    sm: 'px-3 py-1.5 text-sm',
    md: 'px-4 py-2 text-base',
    lg: 'px-6 py-3 text-lg',
  };
  
  return (
    <button
      className={`${baseStyles} ${variantStyles[variant]} ${sizeStyles[size]} ${
        disabled ? 'opacity-50 cursor-not-allowed' : ''
      }`}
      disabled={disabled}
      onClick={onClick}
    >
      {children}
    </button>
  );
};
```

### Input Component

```typescript
interface InputProps {
  type?: 'text' | 'password' | 'email' | 'search';
  placeholder?: string;
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
}

export const Input: React.FC<InputProps> = ({
  type = 'text',
  placeholder,
  value,
  onChange,
  disabled = false,
}) => {
  return (
    <input
      type={type}
      placeholder={placeholder}
      value={value}
      onChange={(e) => onChange(e.target.value)}
      disabled={disabled}
      className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-150 disabled:opacity-50 disabled:cursor-not-allowed"
    />
  );
};
```

## Design Tokens

### Token Naming Convention

**Format:** `--{category}-{property}-{variant}-{state}`

**Examples:**
- `--color-primary-500`
- `--spacing-md`
- `--radius-lg`
- `--shadow-md`

### Token Categories

- **color:** Colors
- **spacing:** Spacing
- **typography:** Typography
- **radius:** Border radius
- **shadow:** Shadows
- **transition:** Transitions
- **z-index:** Z-index
