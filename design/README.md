# Solferry design system

Quiet zinc surfaces and one warm light. The interface stays calm so the file in motion is the only thing that matters.

`tokens.json` is the single source. Run `python3 design/generate.py` after changing it. It rejects any color pair below 4.5:1 contrast and writes the desktop theme plus the Kotlin and Swift tokens in `generated/`. Never edit generated files.

## Principles

- **One accent.** Sun marks the single next action on a screen. If two things are sun, one of them is wrong.
- **Show the route.** People should always see how a file travels. Each route has its own color and is never used for anything else.
- **Say what happened.** Buttons name their result, errors name the fix, and nothing shows a raw error.
- **Quiet until needed.** Prompts appear only when a person has to decide. Trusted devices just work, and a notification confirms it.
- **Native first.** Mobile uses these tokens inside platform controls: Compose on Android, SwiftUI on iOS. It does not imitate the desktop.

## Structure

A desktop card is a frame around a well: one line and 3px of padding, then a darker well set into it. The well holds rows. Mobile drops the frame and groups rows the platform's way, a segmented list on Android and an inset group on iOS.

Nothing picks a radius by hand. Corners stay concentric through `outer = inner + inset`, built from the control radius: 10 for controls and rows, 14 for fields, 18 for wells, 22 for cards. Change `corner.control` and all of them move together.

## Color

| Token | Use |
|---|---|
| `page` | Window background and card frames |
| `well` | The recess content sits in |
| `raised` | Hovered and selected rows |
| `control` | Tiles and quiet controls |
| `line` | Frames and dividers |
| `ink`, `inkMuted` | Main and secondary text |
| `sun`, `sunDeep`, `onSun` | The primary action, its hover, and the text on it |
| `link` | Links |
| `success`, `warning`, `danger` | Outcomes only, never decoration |
| `dangerWell` | The well of a card that reports a failure |
| `routeLocal`, `routeOverlay`, `routeDirect`, `routeRelay` | The route a transfer takes |

## Type

Inter Tight for text and Geist Mono for numbers that change, so sizes and speeds don't jitter. Medium is the heaviest weight, nothing is bold. iOS uses SF Pro. Five sizes: `caption`, `label`, `body`, `title`, and `code` for the six digits people compare while pairing.

## Motion

Colors change instantly: hover and pressed states have no fade. Spacing steps by 4. Motion takes 120 ms for feedback, 180 ms for state changes and 280 ms for screens, moves things rather than recolors them, and turns off when the system asks for reduced motion.
