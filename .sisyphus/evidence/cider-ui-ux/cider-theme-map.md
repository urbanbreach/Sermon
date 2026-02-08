# Cider Theme Map

Per-view dynamic vs static surface decisions based on Cider reference analysis.

| View/Route | Dynamic Background Source | Static Surfaces | Glass Tint Intensity | Accent Usage | Notes |
| --- | --- | --- | --- | --- | --- |
| `albums` | Subtle ambient glow from visible album artwork (diffuse, low intensity) | Left nav sidebar, right queue sidebar (dark acrylic ~70% opacity black) | Medium blur on sidebars, high blur on player bar | White text, grey secondary, accent from playing track | Grid content area allows background glow to show through |
| `album-detail` | Heavy blurred gradient from album artwork (saturated, fills top half, fades to black) | Left nav sidebar, right queue sidebar (dark smoke glass) | High blur on sidebars, very high on player bar | Play/Shuffle buttons tinted with artwork accent | Album art gets colored drop shadow/glow matching dominant color |
| `artists` | Neutral dark gradient (no specific artwork reference in grid view) | Left nav sidebar, right queue sidebar | Medium blur | Standard white/grey hierarchy | Infer from albums pattern - minimal dynamic theming in grid |
| `artist-detail` | Blurred gradient from artist header image (if available) or neutral | Left nav sidebar, right queue sidebar | Medium-high blur | Artist name prominent, accent from artwork | Similar to album-detail but with artist focus |
| `tracks` | Minimal/neutral dark background with subtle artwork influence from playing track | Left nav sidebar, right lyrics panel (if shown) | Medium blur on all panels | Standard text hierarchy | Table-focused view, dynamic theming subdued |
| `search-results` | Neutral dark gradient | All sidebars | Low-medium blur | Standard hierarchy | Functional view, minimal dynamic theming |
| `now-playing` | Full-viewport mesh gradient from current artwork (high saturation, animated) | Floating glass control pill, floating queue panel | Very high blur (40px+) on glass containers | Text glow on metadata, accent color from artwork | Immersive mode - background IS the artwork blur |
| `lyrics-fullscreen` | Full-viewport mesh gradient from current artwork (extreme blur, atmospheric) | Floating glass lyrics card, floating control pill | Maximum blur on lyrics card (frosted white/dark), high on controls | Active lyric text glow (pink/white), accent tint on container borders | Karaoke-style focus: active line bright+scaled, inactive dimmed+blurred |
| `preferences` | Neutral dark gradient (static) | All panels | Low blur | Standard hierarchy | Settings should be readable, no dynamic theming |

## Glass Effect Specifications

### Sidebar Glass (Left Nav, Right Queue)
- Background: `rgba(0, 0, 0, 0.5)` to `rgba(0, 0, 0, 0.7)`
- Backdrop blur: `blur(20px)` to `blur(30px)`
- Border: `1px solid rgba(255, 255, 255, 0.1)`
- Corner radius: `0` (full height panels)

### Floating Player Bar
- Background: `rgba(0, 0, 0, 0.3)` to `rgba(0, 0, 0, 0.5)` with artwork tint
- Backdrop blur: `blur(40px)`
- Border: `1px solid rgba(255, 255, 255, 0.15)`
- Corner radius: `16px` to `24px` (pill shape: `9999px` for capsule)
- Shadow: `0 8px 32px rgba(0, 0, 0, 0.4)`

### Lyrics Card (Now Playing/Fullscreen)
- Background: `rgba(0, 0, 0, 0.3)` or `rgba(255, 255, 255, 0.1)` depending on mode
- Backdrop blur: `blur(40px)` to `blur(60px)`
- Border: `1px solid rgba(255, 255, 255, 0.1)`
- Corner radius: `24px` to `32px`

### Dynamic Background Generation
1. Extract dominant color from current artwork
2. Apply Gaussian blur at extreme radius (100px+)
3. Overlay with dark gradient (`linear-gradient(to bottom, transparent 0%, rgba(0,0,0,0.8) 100%)`)
4. Optional: Add noise texture at 2-5% opacity for depth

## Typography Hierarchy

| Element | Size | Weight | Color | Effects |
| --- | --- | --- | --- | --- |
| Album/Artist Title | 32px+ | Bold/ExtraBold | White 100% | None |
| Metadata (Year/Genre) | 12px | Regular | White 60%, uppercase, tracked | None |
| Track Title | 14-16px | Medium | White 90% | None |
| Track Artist | 14px | Regular | White 60% | None |
| Active Lyric | 36-48px | Bold | White 100% | text-shadow glow, scale 1.05 |
| Inactive Lyric | 24-30px | Regular | White 30-40% | blur(2px), scale 0.95 |
| Player Metadata | 12-14px | Medium | White 90% | None |

## Icon Style

- Stroke width: 1.5px to 2px
- Style: Outline/line art (not filled)
- Corner radius: Rounded
- Active state: Filled or higher opacity
- Color: White 80% default, White 100% on hover/active
