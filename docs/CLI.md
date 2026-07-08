# OpenPaste CLI Reference

## Overview

OpenPaste CLI (`openpaste`) provides command-line access to clipboard history, search, and management. The CLI communicates with the OpenPaste daemon via IPC.

## Installation

The CLI is installed as part of OpenPaste:

```bash
# From source
cargo install --path crates/clipboard-cli

# From package manager (future)
brew install openpaste  # macOS
apt install openpaste    # Linux
winget install openpaste # Windows
```

## Global Options

### --help, -h

Show help information.

```bash
openpaste --help
openpaste search --help
```

### --version, -V

Show version information.

```bash
openpaste --version
```

### --config, -c

Specify config file path.

```bash
openpaste --config ~/.config/openpaste/config.toml search
```

### --verbose, -v

Enable verbose output.

```bash
openpaste --verbose search
```

### --quiet, -q

Suppress output (except errors).

```bash
openpaste --quiet pin 123
```

### --json

Output in JSON format.

```bash
openpaste --json search "hello"
```

## Commands

### search

Search clipboard history.

**Usage:**
```bash
openpaste search [OPTIONS] <QUERY>
```

**Arguments:**
- `QUERY`: Search query

**Options:**
- `--limit, -l <N>`: Limit results (default: 50)
- `--offset, -o <N>`: Offset for pagination (default: 0)
- `--type <TYPE>`: Filter by content type (text, image, html, file)
- `--tag <TAG>`: Filter by tag
- `--collection <ID>`: Filter by collection ID
- `--after <DATE>`: Filter by date (YYYY-MM-DD or Unix timestamp)
- `--before <DATE>`: Filter by date (YYYY-MM-DD or Unix timestamp)
- `--pinned`: Show only pinned items
- `--favorite`: Show only favorite items
- `--format <FORMAT>`: Output format (table, json, plain)

**Examples:**
```bash
# Basic search
openpaste search "hello world"

# Search with limit
openpaste search --limit 10 "hello"

# Search by type
openpaste search --type image "screenshot"

# Search by tag
openpaste search --tag important "meeting"

# Search in collection
openpaste search --collection 1 "project"

# Search by date range
openpaste search --after 2024-01-01 --before 2024-12-31 "report"

# JSON output
openpaste --json search "hello"

# Plain text output
openpaste search --format plain "hello"
```

**Output (table):**
```
ID  Type    Preview                Source    Time
123 text    Hello, World!          Chrome    2m ago
124 image   Screenshot.png          VS Code   5m ago
125 url     https://example.com    Firefox   10m ago
```

**Output (json):**
```json
{
  "results": [
    {
      "id": 123,
      "content_type": "text",
      "content_preview": "Hello, World!",
      "created_at": 1704067200000,
      "source_app": "Chrome"
    }
  ],
  "total": 3
}
```

### get

Get clipboard item by ID.

**Usage:**
```bash
openpaste get [OPTIONS] <ID>
```

**Arguments:**
- `ID`: Item ID

**Options:**
- `--output, -o <FILE>`: Save to file
- `--copy`: Copy to clipboard
- `--preview`: Show preview only
- `--format <FORMAT>`: Output format (raw, json)

**Examples:**
```bash
# Get item
openpaste get 123

# Get and copy to clipboard
openpaste get --copy 123

# Get and save to file
openpaste get --output output.txt 123

# Get preview
openpaste get --preview 123

# JSON output
openpaste get --format json 123
```

### copy

Copy item to clipboard.

**Usage:**
```bash
openpaste copy [OPTIONS] <ID>
```

**Arguments:**
- `ID`: Item ID

**Options:**
- `--paste`: Paste after copying (platform-specific)

**Examples:**
```bash
# Copy item
openpaste copy 123

# Copy and paste
openpaste copy --paste 123
```

### list

List clipboard items.

**Usage:**
```bash
openpaste list [OPTIONS]
```

**Options:**
- `--limit, -l <N>`: Limit results (default: 50)
- `--offset, -o <N>`: Offset for pagination (default: 0)
- `--type <TYPE>`: Filter by content type
- `--tag <TAG>`: Filter by tag
- `--collection <ID>`: Filter by collection ID
- `--pinned`: Show only pinned items
- `--favorite`: Show only favorite items
- `--sort <FIELD>`: Sort by field (created_at, accessed_at, access_count)
- `--order <ORDER>`: Sort order (asc, desc)
- `--format <FORMAT>`: Output format (table, json, plain)

**Examples:**
```bash
# List recent items
openpaste list

# List with limit
openpaste list --limit 10

# List by type
openpaste list --type image

# List pinned items
openpaste list --pinned

# List sorted by access count
openpaste list --sort access_count --order desc

# JSON output
openpaste list --format json
```

### pin

Pin/unpin item.

**Usage:**
```bash
openpaste pin [OPTIONS] <ID>
```

**Arguments:**
- `ID`: Item ID

**Options:**
- `--unpin`: Unpin item (default: pin)

**Examples:**
```bash
# Pin item
openpaste pin 123

# Unpin item
openpaste pin --unpin 123
```

### favorite

Favorite/unfavorite item.

**Usage:**
```bash
openpaste favorite [OPTIONS] <ID>
```

**Arguments:**
- `ID`: Item ID

**Options:**
- `--unfavorite`: Unfavorite item (default: favorite)

**Examples:**
```bash
# Favorite item
openpaste favorite 123

# Unfavorite item
openpaste favorite --unfavorite 123
```

### delete

Delete item.

**Usage:**
```bash
openpaste delete [OPTIONS] <ID>
```

**Arguments:**
- `ID`: Item ID

**Options:**
- `--hard`: Hard delete (default: soft delete)
- `--force`: Skip confirmation

**Examples:**
```bash
# Soft delete
openpaste delete 123

# Hard delete
openpaste delete --hard 123

# Force delete (no confirmation)
openpaste delete --force 123
```

### tag

Manage item tags.

**Usage:**
```bash
openpaste tag <SUBCOMMAND>
```

**Subcommands:**
- `add <ID> <TAG>`: Add tag to item
- `remove <ID> <TAG>`: Remove tag from item
- `list <ID>`: List item tags

**Examples:**
```bash
# Add tag
openpaste tag add 123 important

# Remove tag
openpaste tag remove 123 important

# List tags
openpaste tag list 123
```

### collection

Manage collections.

**Usage:**
```bash
openpaste collection <SUBCOMMAND>
```

**Subcommands:**
- `list`: List collections
- `create <NAME>`: Create collection
- `delete <ID>`: Delete collection
- `add <ITEM_ID> <COLLECTION_ID>`: Add item to collection
- `remove <ITEM_ID> <COLLECTION_ID>`: Remove item from collection

**Examples:**
```bash
# List collections
openpaste collection list

# Create collection
openpaste collection create "Work"

# Delete collection
openpaste collection delete 2

# Add item to collection
openpaste collection add 123 2

# Remove item from collection
openpaste collection remove 123 2
```

### clipboard

Get/set clipboard content.

**Usage:**
```bash
openpaste clipboard <SUBCOMMAND>
```

**Subcommands:**
- `get`: Get current clipboard content
- `set <CONTENT>`: Set clipboard content
- `clear`: Clear clipboard

**Examples:**
```bash
# Get clipboard
openpaste clipboard get

# Set clipboard
openpaste clipboard set "Hello, World!"

# Clear clipboard
openpaste clipboard clear
```

### encryption

Manage encryption.

**Usage:**
```bash
openpaste encryption <SUBCOMMAND>
```

**Subcommands:**
- `status`: Show encryption status
- `unlock`: Unlock vault
- `lock`: Lock vault
- `change-password`: Change master password

**Examples:**
```bash
# Show status
openpaste encryption status

# Unlock vault
openpaste encryption unlock

# Lock vault
openpaste encryption lock

# Change password
openpaste encryption change-password
```

### sync

Manage synchronization.

**Usage:**
```bash
openpaste sync <SUBCOMMAND>
```

**Subcommands:**
- `status`: Show sync status
- `start`: Start sync
- `configure`: Configure sync provider

**Examples:**
```bash
# Show status
openpaste sync status

# Start sync
openpaste sync start

# Configure sync
openpaste sync configure
```

### daemon

Manage daemon.

**Usage:**
```bash
openpaste daemon <SUBCOMMAND>
```

**Subcommands:**
- `start`: Start daemon
- `stop`: Stop daemon
- `restart`: Restart daemon
- `status`: Show daemon status

**Examples:**
```bash
# Start daemon
openpaste daemon start

# Stop daemon
openpaste daemon stop

# Restart daemon
openpaste daemon restart

# Show status
openpaste daemon status
```

### status

Show system status.

**Usage:**
```bash
openpaste status [OPTIONS]
```

**Options:**
- `--json`: Output in JSON format

**Examples:**
```bash
# Show status
openpaste status

# JSON output
openpaste status --json
```

**Output:**
```
OpenPaste v0.1.0
Status: Running
Clipboard Watching: Enabled
Encryption: Enabled (Unlocked)
Item Count: 1,234
Uptime: 1h 30m
```

### serve

Start REST API server.

**Usage:**
```bash
openpaste serve [OPTIONS]
```

**Options:**
- `--host <HOST>`: Bind host (default: 127.0.0.1)
- `--port <PORT>`: Bind port (default: 7890)

**Examples:**
```bash
# Start server
openpaste serve

# Start on custom port
openpaste serve --port 8080
```

### export

Export clipboard data.

**Usage:**
```bash
openpaste export [OPTIONS] <FILE>
```

**Arguments:**
- `FILE`: Output file

**Options:**
- `--format <FORMAT>`: Export format (json, csv, txt)
- `--type <TYPE>`: Filter by content type
- `--after <DATE>`: Filter by date
- `--before <DATE>`: Filter by date
- `--include-content`: Include full content (default: preview only)

**Examples:**
```bash
# Export to JSON
openpaste export --format json backup.json

# Export to CSV
openpaste export --format csv backup.csv

# Export with content
openpaste export --include-content backup.json

# Export by date range
openpaste export --after 2024-01-01 backup.json
```

### import

Import clipboard data.

**Usage:**
```bash
openpaste import [OPTIONS] <FILE>
```

**Arguments:**
- `FILE`: Input file

**Options:**
- `--format <FORMAT>`: Import format (json, csv, txt)
- `--skip-duplicates`: Skip duplicate items
- `--merge`: Merge with existing data

**Examples:**
```bash
# Import from JSON
openpaste import --format json backup.json

# Import from CSV
openpaste import --format csv backup.csv

# Skip duplicates
openpaste import --skip-duplicates backup.json

# Merge with existing
openpaste import --merge backup.json
```

### config

Manage configuration.

**Usage:**
```bash
openpaste config <SUBCOMMAND>
```

**Subcommands:**
- `get <KEY>`: Get config value
- `set <KEY> <VALUE>`: Set config value
- `list`: List all config
- `edit`: Edit config file

**Examples:**
```bash
# Get config value
openpaste config get max_items

# Set config value
openpaste config set max_items 10000

# List all config
openpaste config list

# Edit config file
openpaste config edit
```

### plugin

Manage plugins.

**Usage:**
```bash
openpaste plugin <SUBCOMMAND>
```

**Subcommands:**
- `list`: List installed plugins
- `install <NAME/FILE>`: Install plugin
- `uninstall <NAME>`: Uninstall plugin
- `enable <NAME>`: Enable plugin
- `disable <NAME>`: Disable plugin
- `info <NAME>`: Show plugin info

**Examples:**
```bash
# List plugins
openpaste plugin list

# Install plugin
openpaste plugin install url-detector

# Install from file
openpaste plugin install ./my-plugin.zip

# Uninstall plugin
openpaste plugin uninstall url-detector

# Enable plugin
openpaste plugin enable url-detector

# Disable plugin
openpaste plugin disable url-detector

# Show plugin info
openpaste plugin info url-detector
```

### completion

Generate shell completion.

**Usage:**
```bash
openpaste completion <SHELL>
```

**Arguments:**
- `SHELL`: Shell type (bash, zsh, fish, powershell)

**Examples:**
```bash
# Bash completion
openpaste completion bash > ~/.local/share/bash-completion/completions/openpaste

# Zsh completion
openpaste completion zsh > ~/.zsh/completion/_openpaste

# Fish completion
openpaste completion fish > ~/.config/fish/completions/openpaste.fish

# PowerShell completion
openpaste completion powershell > openpaste.ps1
```

## Shell Integration

### Bash Integration

Add to `~/.bashrc`:
```bash
# OpenPaste completion
source ~/.local/share/bash-completion/completions/openpaste

# Quick search function
op-search() {
    openpaste search "$@" | fzf | awk '{print $1}' | xargs openpaste copy
}

# Quick copy function
op-copy() {
    openpaste get "$1" | openpaste clipboard set
}
```

### Zsh Integration

Add to `~/.zshrc`:
```zsh
# OpenPaste completion
source ~/.zsh/completion/_openpaste

# Quick search function
op-search() {
    openpaste search "$@" | fzf | awk '{print $1}' | xargs openpaste copy
}

# Quick copy function
op-copy() {
    openpaste get "$1" | openpaste clipboard set
}
```

### Fish Integration

Add to `~/.config/fish/config.fish`:
```fish
# OpenPaste completion
openpaste completion fish | source

# Quick search function
function op-search
    openpaste search $argv | fzf | awk '{print $1}' | xargs openpaste copy
end

# Quick copy function
function op-copy
    openpaste get $argv[1] | openpaste clipboard set
end
```

## Configuration File

### Config Location

**Linux:** `~/.config/openpaste/config.toml`

**macOS:** `~/Library/Application Support/OpenPaste/config.toml`

**Windows:** `%APPDATA%\OpenPaste\config.toml`

### Config Format

```toml
[general]
daemon_auto_start = true
show_tray_icon = true

[clipboard]
capture_text = true
capture_images = true
capture_html = true
capture_files = true
ignore_own_copies = true
max_items = 10000
max_age_days = 90
detect_duplicates = true
duplicate_action = "skip"

[search]
instant_search = true
debounce_ms = 150
highlight_matches = true
fuzzy_search = false
remove_diacritics = true
max_results = 50

[encryption]
enabled = false
encrypt_content = true
encrypt_metadata = false
encrypt_file_paths = false
auto_lock_minutes = 5

[api]
enabled = true
host = "127.0.0.1"
port = 7890
require_auth = true

[cli]
default_format = "table"
default_limit = 50
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid usage |
| 3 | Daemon not running |
| 4 | Permission denied |
| 5 | Item not found |
| 6 | Encryption locked |

## Examples

### Workflow Examples

**Search and copy:**
```bash
# Search for item and copy it
openpaste search "hello" | fzf | awk '{print $1}' | xargs openpaste copy
```

**Pin recent items:**
```bash
# Pin last 10 items
openpaste list --limit 10 | awk '{print $1}' | xargs openpaste pin
```

**Export and import:**
```bash
# Export to JSON
openpaste export --format json backup.json

# Import from JSON
openpaste import --format json backup.json
```

**Batch tag:**
```bash
# Tag all items from Chrome
openpaste list | grep Chrome | awk '{print $1}' | xargs -I {} openpaste tag add {} work
```

### Scripting Examples

**Bash script:**
```bash
#!/bin/bash
# Backup clipboard items older than 30 days

openpaste export --after $(date -d "30 days ago" +%Y-%m-%d) backup.json
```

**Python script:**
```python
#!/usr/bin/env python3
import subprocess
import json

# Search and process items
result = subprocess.run(['openpaste', '--json', 'search', 'hello'], 
                       capture_output=True, text=True)
items = json.loads(result.stdout)

for item in items['results']:
    print(f"Found: {item['content_preview']}")
```

**PowerShell script:**
```powershell
# Search and copy first result
$result = openpaste --json search "hello" | ConvertFrom-Json
$id = $result.results[0].id
openpaste copy $id
```

## Performance Tips

### Speed Up Commands

**Use JSON output:**
```bash
openpaste --json search "hello" # Faster than table formatting
```

**Limit results:**
```bash
openpaste search --limit 10 "hello" # Faster than unlimited
```

**Use specific filters:**
```bash
openpaste search --type text "hello" # Faster than searching all types
```

### Reduce Daemon Communication

**Batch operations:**
```bash
# Instead of multiple calls
openpaste pin 1
openpaste pin 2
openpaste pin 3

# Use single call (future feature)
openpaste pin 1 2 3
```

## Troubleshooting

### Daemon Not Running

**Error:**
```
Error: Daemon not running
```

**Solution:**
```bash
openpaste daemon start
```

### Permission Denied

**Error:**
```
Error: Permission denied
```

**Solution:**
```bash
# Check daemon status
openpaste daemon status

# Restart daemon
openpaste daemon restart
```

### Item Not Found

**Error:**
```
Error: Item not found
```

**Solution:**
```bash
# Search for item
openpaste search "query"

# List all items
openpaste list
```

### Encryption Locked

**Error:**
```
Error: Encryption locked
```

**Solution:**
```bash
openpaste encryption unlock
```

## Tips and Tricks

### Quick Access

**Create aliases:**
```bash
alias op='openpaste'
alias ops='openpaste search'
alias opc='openpaste copy'
alias opl='openpaste list'
```

### FZF Integration

**Interactive search:**
```bash
openpaste list | fzf | awk '{print $1}' | xargs openpaste copy
```

### Tmux Integration

**Copy to tmux clipboard:**
```bash
openpaste get 123 | tmux load-buffer -
```

### Vim Integration

**Copy from Vim:**
```vim
:read !openpaste get 123
```

**Copy to Vim:**
```vim
:w !openpaste clipboard set
```

## Advanced Usage

### Piping

**Chain commands:**
```bash
openpaste search "hello" | grep "Chrome" | awk '{print $1}' | xargs openpaste copy
```

### Parallel Operations

**Process multiple items:**
```bash
openpaste list | awk '{print $1}' | xargs -P 4 -I {} openpaste tag add {} work
```

### Conditional Operations

**Only pin if not already pinned:**
```bash
openpaste list | grep -v "Pinned" | awk '{print $1}' | xargs openpaste pin
```

## Environment Variables

**OPENPASTE_CONFIG:** Config file path
```bash
export OPENPASTE_CONFIG=~/.config/openpaste/custom.toml
```

**OPENPASTE_HOST:** API host
```bash
export OPENPASTE_HOST=127.0.0.1
```

**OPENPASTE_PORT:** API port
```bash
export OPENPASTE_PORT=7890
```

**OPENPASTE_TOKEN:** API token
```bash
export OPENPASTE_TOKEN=your_token_here
```
