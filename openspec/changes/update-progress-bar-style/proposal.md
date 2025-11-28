# Change: Replace Progress Bars with Historical Sparkline Graphs

## Why
The current progress bars show only the instantaneous CPU/memory value as a filled bar. Modern resource monitors like btop display a **rolling time-series graph** showing usage history, which provides much more valuable insight into container behavior over time.

## What Changes
- Replace static percentage bars with historical sparkline graphs
- Store a history buffer of CPU and memory values per container (e.g., last 20 samples)
- Render history as braille-based vertical bar graphs, scrolling right-to-left
- Most recent value appears on the right, older values scroll left
- Use braille characters to represent different usage levels (0-100% mapped to vertical dot heights)

## Visual Comparison

**Current (static bar):**
```
████████████░░░░░░░░ 5.1%    ████████████░░░░░░░░ 219M/1G
```

**Proposed (historical sparkline):**
```
⣀⣀⣤⣤⣶⣿⣿⣶⣤⣀⣀⣤⣶⣿⣶⣤⣀⣀ 5.1%    ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿ 219M/1G
```

The sparkline shows how CPU spiked and fell over the last ~20 sample periods.

## Braille Height Mapping
Each braille cell has 4 vertical dot positions. Map percentage to height:
- 0-12.5%: `⠀` (empty)
- 12.5-25%: `⣀` (1 row)
- 25-50%: `⣤` (2 rows)
- 50-75%: `⣶` (3 rows)
- 75-100%: `⣿` (4 rows/full)

## Impact
- Affected specs: ui-rendering (new capability)
- Affected code:
  - `src/core/types.rs` - Add history buffer to `ContainerStats`
  - `src/docker/stats.rs` - Populate history on each stats update
  - `src/ui/container_list.rs` - Replace `create_progress_bar()` with sparkline renderer
