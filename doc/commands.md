# Intern Local Command Sheet

Use these instructions in JSON command sheets (`index`, `instruction`, `tag`).

## Input
- `move <x> <y>`: Move mouse to absolute screen coordinates.
- `click <x> <y>`: Move and left-click.
- `doubleclick <x> <y>`: Move and double-click.
- `mousedown <x> <y>`: Move and press left button down.
- `mouseup <x> <y>`: Move and release left button.
- `drag <x1> <y1> <x2> <y2>`: Press at (x1,y1), move to (x2,y2), release.
- `scroll <delta>`: Scroll vertically (positive down, negative up).
- `type: <text>`: Type raw text.
- `key: <NAME>`: Press a single key name (ENTER, TAB, ESC, SPACE, or a single character).
- `hotkey: CTRL+SHIFT+S`: Press key combo. Supported: CTRL, SHIFT, ALT, WIN + a single key.

## Screen
- `screenshot <path>`: Capture primary screen to file.

## Control
- `wait <ms>`: Sleep for milliseconds.
- `noop`: Do nothing (useful for spacing in tests).
