# Solferry design system

A ferry crossing at sundown: deep harbor water, one warm light. The interface stays calm so the file in motion is the only thing that matters.

`tokens.json` is the single source. Run `python3 design/generate.py` after changing it. It rejects any color pair below 4.5:1 contrast and writes the desktop theme plus the Kotlin and Swift tokens in `generated/`. Never edit generated files.

## Principles

- **One accent.** Sun marks the single next action on a screen. If two things are sun, one of them is wrong.
- **Show the route.** People should always see how a file travels. Each route has its own color and is never used for anything else.
- **Say what happened.** Buttons name their result, errors name the fix, and nothing shows a raw error.
- **Quiet until needed.** Prompts appear only when a person has to decide. Trusted devices just work, and a notification confirms it.
- **Native first.** Mobile uses these tokens inside platform controls: Compose on Android, SwiftUI on iOS. It does not imitate the desktop.

## Color

| Token | Use |
|---|---|
| `canvas` | Window background |
| `surface`, `surfaceRaised` | Cards, then hover and pressed states |
| `line` | Borders and dividers |
| `ink`, `inkMuted` | Main and secondary text |
| `sun`, `onSun` | The primary action and the text on it |
| `harbor` | Links |
| `success`, `warning`, `danger` | Outcomes only, never decoration |
| `routeLocal`, `routeOverlay`, `routeDirect`, `routeRelay` | The route a transfer takes |

## Scale

Spacing steps by 4. Radius is `sm` for controls, `md` for cards and `lg` for sheets. Type has four sizes: `caption`, `body`, `title`, and `code` for the six digits people compare while pairing. Motion takes 120 ms for feedback, 180 ms for state changes and 280 ms for screens, and turns off when the system asks for reduced motion.
