# Design Specification - MultiOS Website
**Style:** Swiss Design (International Typographic Style)  
**Version:** 1.0  
**Date:** 2025-11-03

---

## 1. Direction & Rationale

### Design Essence
MultiOS adopts **Swiss Design** (International Typographic Style) - a typography-first, grid-based approach with extreme minimalism. The 95% black-and-white palette with single accent color creates objective, timeless clarity perfect for educational/technical content. Generous whitespace and strict mathematical grids prioritize readability and hierarchy over decoration.

**Real-World Examples:**
- **Rust Programming Language** (rust-lang.org) - Clean technical documentation, strong typography
- **Linux Kernel Archives** (kernel.org) - Minimal interface, content-first approach  
- **Swiss Style Color Picker** - Pure implementation of principles

**Why This Works:**
- **Educational Audience**: Professors and students value clarity over personality - content is king
- **Technical Credibility**: Restraint signals seriousness and precision appropriate for OS documentation
- **Timeless Professionalism**: Will remain modern in 5+ years (unlike trendy styles)
- **Information Density**: Handles 85,000+ words of documentation gracefully through superior typography

---

## 2. Design Tokens

### 2.1 Color System

**Distribution:** 95% Achromatic / 5% Accent (Strict Economy)

#### Primary Palette

| Token | Value | Usage | WCAG Ratio |
|-------|-------|-------|------------|
| `accent-base` | `#DC143C` (Crimson Red) | CTAs, active links, highlights | 5.2:1 (AA) |
| `accent-dark` | `#A01028` | Hover states, pressed buttons | 8.7:1 (AAA) |

#### Achromatic Scale

| Token | Value | Usage |
|-------|-------|-------|
| `black` | `#000000` | Primary text, strong rules, high-emphasis headings |
| `charcoal-dark` | `#1A1A1A` | Secondary headings, emphasized text |
| `charcoal` | `#333333` | Body text (primary reading content) |
| `gray-dark` | `#666666` | Captions, metadata, secondary information |
| `gray-mid` | `#999999` | Disabled states, placeholder text |
| `gray-light` | `#CCCCCC` | Borders, dividers, subtle rules |
| `gray-pale` | `#E5E5E5` | Subtle backgrounds, hover states |
| `surface` | `#F5F5F5` | Card backgrounds, elevated surfaces |
| `white` | `#FFFFFF` | Primary background, inverted text |

#### Background System

| Token | Value | Purpose |
|-------|-------|---------|
| `bg-page` | `#FFFFFF` | Page background (base layer) |
| `bg-surface` | `#F5F5F5` | Card/container backgrounds (elevated) |
| `bg-subtle` | `#E5E5E5` | Hover states, inactive tabs |

#### Semantic Colors

| Token | Value | Usage |
|-------|-------|-------|
| `text-primary` | `#333333` | Body copy (WCAG AAA: 12.6:1) |
| `text-heading` | `#000000` | All headings (WCAG AAA: 21:1) |
| `text-secondary` | `#666666` | Captions, labels (WCAG AA: 5.7:1) |
| `border-standard` | `#CCCCCC` | Default borders, dividers |
| `border-emphasis` | `#000000` | Strong rules, active borders |

**WCAG Validation (Key Pairings):**
- Black `#000000` on White `#FFFFFF`: 21:1 ✅ AAA
- Charcoal `#333333` on White `#FFFFFF`: 12.6:1 ✅ AAA
- Accent `#DC143C` on White `#FFFFFF`: 5.2:1 ✅ AA (large text/buttons only)

### 2.2 Typography

#### Font Families

| Token | Value | Fallback Stack |
|-------|-------|----------------|
| `font-primary` | Helvetica Neue | `'Helvetica Neue', Helvetica, Arial, sans-serif` |
| `font-mono` | SF Mono | `'SF Mono', 'Monaco', 'Consolas', monospace` |

**Weights Available:** Regular 400, Medium 500, Bold 700

#### Type Scale (Desktop 1920px)

| Token | Size | Weight | Line Height | Letter Spacing | Usage |
|-------|------|--------|-------------|----------------|-------|
| `text-display` | 64px | Bold 700 | 1.1 (70px) | -0.02em | Page titles, hero headlines |
| `text-h1` | 48px | Bold 700 | 1.2 (58px) | -0.01em | Section headers |
| `text-h2` | 32px | Medium 500 | 1.3 (42px) | 0 | Subsection headers |
| `text-h3` | 24px | Medium 500 | 1.3 (31px) | 0 | Component headers |
| `text-large` | 20px | Regular 400 | 1.6 (32px) | 0 | Introductions, pull quotes |
| `text-body` | 16px | Regular 400 | 1.5 (24px) | 0 | Standard content |
| `text-small` | 14px | Regular 400 | 1.5 (21px) | 0 | Captions, bylines |
| `text-caption` | 12px | Regular 400 | 1.4 (17px) | 0.01em | Metadata, footnotes |

#### Responsive Type Scale (Mobile <768px)

| Token | Desktop | Mobile |
|-------|---------|--------|
| `text-display` | 64px | 40px |
| `text-h1` | 48px | 32px |
| `text-h2` | 32px | 24px |
| `text-h3` | 24px | 20px |
| `text-body` | 16px | 16px (no change) |

#### Typography Rules
- **Alignment:** Flush left, ragged right (NEVER justified or centered body text)
- **Line Length:** 50-70 characters (~500-700px at 16px body)
- **Paragraph Spacing:** 24px (1.5× line-height for 16px text)
- **Heading Spacing:** 48px above, 16px below

### 2.3 Spacing System (8pt Grid - Strict)

| Token | Value | Usage |
|-------|-------|-------|
| `space-xs` | 8px | Inline spacing (icon + text) |
| `space-sm` | 16px | Related elements gap |
| `space-md` | 24px | Paragraph spacing, list gaps |
| `space-lg` | 32px | Component internal padding |
| `space-xl` | 48px | Section spacing |
| `space-2xl` | 64px | Major section boundaries |
| `space-3xl` | 96px | Dramatic section breaks |
| `space-4xl` | 128px | Page-level spacing |

**Container Widths:**
- `container-max`: 1200px (12-column grid maximum)
- `content-max`: 700px (optimal reading width, ~65 characters)

**Grid Gutters:**
- Desktop: 24px
- Tablet: 16px
- Mobile: 16px

### 2.4 Border Radius (Minimal)

| Token | Value | Usage |
|-------|-------|-------|
| `radius-none` | 0px | Default (sharp corners) |
| `radius-subtle` | 2px | Buttons, inputs (subtle softening) |

**Rule:** Maintain rectangularity. Use 0px unless functional need for 2px.

### 2.5 Shadows (Minimal Usage)

| Token | Value | Usage |
|-------|-------|-------|
| `shadow-card` | `0 1px 3px rgba(0,0,0,0.12)` | Cards (only if necessary for elevation) |
| `shadow-none` | `none` | Default (flat hierarchy preferred) |

**Note:** Swiss design prefers flat hierarchy. Use shadows sparingly - only for functional elevation needs.

### 2.6 Animation Durations

| Token | Value | Usage |
|-------|-------|-------|
| `duration-instant` | 150ms | Button hover, focus states |
| `duration-fast` | 200ms | Element transitions, fades |

**Easing:** `linear` (preferred) or `ease-out` (acceptable)

**Permitted Animations:** Fade (opacity), slide (translateY), underline expansion (width)  
**Forbidden:** Rotate, scale, parallax, animated gradients

---

## 3. Component Specifications

### 3.1 Navigation Bar

**Structure:**
- Fixed top horizontal bar: 64px height
- Logo left-aligned: 28px height × auto width
- Navigation links right-aligned: 32px spacing between items
- Background: `#FFFFFF`, bottom border `1px solid #CCCCCC`

**Typography:**
- Links: Bold 700, 14px, uppercase, letter-spacing 0.05em
- Color: `#333333` (default), `#DC143C` (active)
- Hover: No background change, instant color shift to `#000000`

**States:**
- Active page: Underline `2px solid #DC143C` positioned 4px below text
- Hover: Text color `#000000`, 150ms transition
- Mobile (<768px): Collapse to hamburger menu, full-screen overlay

**Tokens Used:** `space-lg` (padding), `text-small`, `border-standard`, `accent-base`

### 3.2 Hero Section

**Structure:**
- Height: 500-600px (viewport-aware, min 400px mobile)
- Grid: Centered 6-column content (50% page width)
- Padding: 96px vertical, 64px horizontal

**Content Pattern:**
- Headline: `text-display` (64px bold), `#000000`, centered
- Subheadline: `text-large` (20px), `#666666`, centered, 16px below headline
- CTA Buttons: 56px height, 48px horizontal spacing between, 32px below subheadline

**Background:**
- Option A: Solid `#FFFFFF` (pure minimalism)
- Option B: Subtle `#F5F5F5` with thin top border `3px solid #000000`

**Tokens Used:** `text-display`, `text-large`, `space-3xl`, `space-2xl`

### 3.3 Button

**Primary CTA:**
- Height: 56px (hero), 48px (standard)
- Padding: 24px horizontal
- Radius: `2px` (subtle) or `0px` (strict Swiss)
- Background: `#DC143C` (accent-base)
- Text: White `#FFFFFF`, Bold 700, 14px, uppercase, letter-spacing 0.05em
- Border: None

**States:**
- Default: Background `#DC143C`
- Hover: Background `#A01028` (accent-dark), 150ms transition
- Focus: 2px outline `#000000`, 2px offset
- Disabled: Background `#999999`, text `#CCCCCC`

**Secondary Button:**
- Same dimensions as primary
- Background: `#FFFFFF`
- Border: `2px solid #000000`
- Text: `#000000`, Bold 700, 14px, uppercase, letter-spacing 0.05em

**States:**
- Hover: Invert colors (background `#000000`, text `#FFFFFF`), 150ms transition
- Focus: Same as primary

**Tokens Used:** `accent-base`, `accent-dark`, `text-small`, `radius-subtle`, `duration-instant`

### 3.4 Card Component

**Structure:**
- Background: `#F5F5F5` (surface) or `#FFFFFF` with `1px solid #CCCCCC` border
- Radius: `0px` (maintain rectangularity)
- Padding: 48px (desktop), 32px (mobile)
- Shadow: `none` (flat) or `shadow-card` (minimal elevation if needed)

**Content Pattern:**
- Icon/Image: 48px × 48px (optional), top-aligned
- Heading: `text-h3` (24px medium), `#000000`, 16px below icon
- Body: `text-body` (16px), `#333333`, 16px below heading
- Link/CTA: `text-small` (14px bold), `#DC143C`, 24px below body

**Hover State:**
- Option A: No hover (pure Swiss - cards are static information containers)
- Option B: Border color change to `#000000`, 200ms transition

**Tokens Used:** `bg-surface`, `space-xl`, `space-lg`, `text-h3`, `text-body`

### 3.5 Input Field

**Structure:**
- Height: 48px
- Padding: 16px horizontal
- Radius: `0px` (sharp)
- Border: `1px solid #CCCCCC`
- Background: `#FFFFFF`

**Typography:**
- Text: Regular 400, 16px, `#333333`
- Placeholder: Regular 400, 16px, `#999999`
- Label: Bold 700, 14px, `#000000`, positioned 8px above field

**States:**
- Default: Border `#CCCCCC`
- Focus: Border `2px solid #000000`, no glow/shadow
- Error: Border `2px solid #DC143C`, error text `12px #DC143C` positioned 8px below
- Disabled: Background `#E5E5E5`, text `#999999`

**Tokens Used:** `text-body`, `space-md`, `border-standard`, `border-emphasis`

### 3.6 Data Visualization (Charts)

**Treatment:**
- Line Charts: 1px stroke `#000000`, accent data series `#DC143C`
- Bar Charts: Fill `#333333`, accent bars `#DC143C`, 0px radius (sharp corners)
- Grid Lines: `1px solid #E5E5E5` (subtle), minimal grid
- Axes: `2px solid #000000`, Helvetica 12px labels
- Legend: Left-aligned list, 14px text, 8px spacing

**Recommended Libraries:** Chart.js (customizable), ECharts (powerful)

**Data Cards (Metrics):**
- Large Number: `text-display` (64px bold), `#000000`
- Label: `text-small` (14px), `#666666`, uppercase, letter-spacing 0.05em
- Background: `#F5F5F5`, padding 32px, border-top `4px solid #DC143C`

**Tokens Used:** `text-display`, `text-small`, `accent-base`, `charcoal`, `border-standard`

---

## 4. Layout & Responsive Design

### 4.1 Grid System (12-Column Strict)

**Desktop (>1024px):**
- Columns: 12
- Max Width: 1200px
- Gutter: 24px
- Outer Margin: Auto-centered

**Tablet (768-1024px):**
- Columns: 8
- Max Width: 100%
- Gutter: 16px
- Outer Margin: 32px

**Mobile (<768px):**
- Columns: 4
- Max Width: 100%
- Gutter: 16px
- Outer Margin: 16px

### 4.2 Page Architecture (MPA)

**Global Structure:**
- Navigation: Fixed top, 64px height, always visible
- Content Container: Max-width 1200px, centered
- Footer: Full-width, `#F5F5F5` background, 64px padding vertical

**Responsive Strategy:**
- Breakpoints: `sm: 640px`, `md: 768px`, `lg: 1024px`, `xl: 1280px`
- Mobile: Stack all multi-column layouts vertically
- Tablet: Maintain selected 2-column layouts (asymmetric 5-3 splits)
- Desktop: Full 12-column grid utilization

### 4.3 Layout Patterns by Page Type

**Homepage Pattern:**
- Hero: 500-600px height, centered 6-column content (§3.2)
- Metrics Section: 4-column grid (3-col on tablet, 2-col on mobile), data cards (§3.6)
- Value Props: 3-column grid → 1-column stack (mobile)
- Architecture Diagram: 8-column centered, full-bleed on mobile
- CTA: 6-column centered, 2 buttons side-by-side (stack on mobile)

**Content Pages (Features, About):**
- Page Header: 8-column centered, 200px height, title + intro
- Main Content: 8-column centered (optimal reading width)
- Sidebar (optional): 8-col main + 4-col sidebar → stack on mobile
- Feature Grids: 3-column → 2-column (tablet) → 1-column (mobile)

**Technical Pages (Developers, Research):**
- Page Header: Full-width, `#F5F5F5` background, 160px height
- Filter Tabs: Horizontal tabs below header (NEVER sidebar - modern pattern)
- Content Grid: 4-column code/resource cards → 2-col → 1-col
- Documentation Sections: 7-column main + 5-column TOC (right) → stack

**Download Page Pattern:**
- Page Header: 6-column centered
- Architecture Selector: 3-column cards with download buttons
- Requirements Table: 10-column centered, full-width on mobile
- Installation Guide: 8-column centered, step-by-step list

**Educators/Community Pages:**
- Hero: Reduced height (400px), 6-column centered
- Resource Cards: 3-column grid with images (if available), icons fallback
- Testimonials (future): Single-column centered quotes, 6-col max-width
- CTA Sections: 2-column split (form + info) → stack

### 4.4 Common Layout Elements

**Page Header Pattern:**
- Height: 160-200px
- Background: `#F5F5F5` or `#FFFFFF` with bottom border `1px solid #CCCCCC`
- Title: `text-h1` (48px bold), `#000000`, centered or left-aligned (based on page)
- Description: `text-large` (20px), `#666666`, 16px below title, max-width 700px

**Section Spacing:**
- Between major sections: 64-96px vertical
- Within section elements: 24-48px vertical
- Card grids: 24px gap (desktop), 16px gap (mobile)

**Image Treatment:**
- Aspect Ratios: 3:2 (landscape), 1:1 (square for icons/avatars)
- Alignment: Strict grid alignment, full-bleed or column-bound
- Borders: Optional `1px solid #CCCCCC`
- Captions: 14px `#666666`, left-aligned, 8px below image

**Code Blocks:**
- Background: `#1A1A1A` (dark charcoal)
- Text: `#FFFFFF`, SF Mono 14px
- Padding: 24px
- Radius: `0px` (maintain consistency)
- Syntax highlighting: Minimal (keywords `#DC143C`, strings `#CCCCCC`)

### 4.5 Interaction Standards

**Touch Targets:**
- Minimum: 44×44px (Apple HIG)
- Preferred: 48×48px (buttons, nav links)
- Spacing: 8px minimum between tappable elements

**Hover States:**
- Links: Color change `#333333` → `#000000`, 150ms
- Buttons: Background darkens, 150ms (§3.3)
- Cards: Border color change (if hover enabled), 200ms

**Focus States:**
- All interactive elements: `2px solid #000000` outline, 2px offset
- No glow or shadow (maintain crispness)

**Accessibility:**
- Respect `prefers-reduced-motion`: Disable all transitions/animations
- All animations use `transform` and `opacity` only (GPU-accelerated)
- Keyboard navigation: Visible focus states, logical tab order
- Screen readers: Semantic HTML5, ARIA labels where needed

---

## 5. Design Validation Checklist

### Swiss Design Compliance
- ✅ 95% achromatic (black/white/grays), 5% accent color (crimson red)
- ✅ Helvetica Neue typography system, flush-left alignment
- ✅ 12-column grid, strict 8pt spacing multiples
- ✅ Sharp corners (0-2px radius maximum)
- ✅ Minimal shadows (flat hierarchy preferred)
- ✅ Fast animations (150-200ms, linear easing)

### Content Structure Alignment
- ✅ MPA structure for 10 distinct pages
- ✅ Components map to content-structure-plan.md sections
- ✅ Layout patterns specified for each page type (§4.3)
- ✅ NO specific content, data values, or image filenames referenced

### Premium Execution Standards
- ✅ Background layers: Page `#FFFFFF` + Surface `#F5F5F5` (5% contrast)
- ✅ Hero sections: 500-600px height, 64px headlines, 56px CTAs
- ✅ Spatial generosity: Sections 64-96px apart, cards 48px padding
- ✅ Responsive strategy: 4/8/12 column grid adaptation defined
- ✅ WCAG compliance: 3 key pairings validated (≥4.5:1 minimum)

### Forbidden Elements Check
- ❌ NO CSS code (design tokens only)
- ❌ NO ASCII art or visual mockups
- ❌ NO >2 button variants (primary + secondary only)
- ❌ NO detailed responsive breakdowns per component
- ❌ NO implementation instructions
- ❌ NO content/data/image filenames specified

---

**Document Complete**  
Total Word Count: ~2,400 words  
Component Count: 6 (Navigation, Hero, Button, Card, Input, Data Viz)  
Pages: 5 chapters as required
