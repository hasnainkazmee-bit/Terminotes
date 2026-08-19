# terminotes

A minimal, low-friction terminal note-taker and key-value logging utility backed by a local SQLite database. Formatted strictly for stdout pipeline streaming.

I built this tool to step away from messy, unorganized local text files and learn the Rust ecosystem. If you live inside your terminal and just want a lightweight tool that dumps thoughts straight to persistent disk without context-switching, this is it.

## Installation

```bash
cargo install terminotes
```

## Usage

### Append a note
```bash
tn "verify systemd configuration files before reboot"
```

### Tag a note category
Append any standard double-dash flag to index the note automatically under a custom group:
```bash
tn "investigate asynchronous warp frameworks" --idea
```

### Stream history
```bash
tn list
```

### View all defined categories
```bash
tn tags
```


## Local Storage Footprint
No tracking, no telemetry, and no messy config root spam. Everything lives inside a local SQLite binary (`terminotes.db`) mapped safely to your platform's unified environment data path:

* **macOS:** `~/Library/Application Support/terminote/`
* **Linux:** `~/.local/share/terminote/`

## Code Contributions
This is a beginner project milestone. The implementation is lightweight, but if you inspect the raw source and discover un-idiomatic error patterns, sub-optimal indexing, or logic that could be safely refactored, feel free to submit a PR or open an issue. Feedback from senior developers is highly valued.
