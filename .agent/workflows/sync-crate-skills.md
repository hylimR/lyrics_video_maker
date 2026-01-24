---
description: Sync dynamic skills for Cargo.toml dependencies
argument-hint: [--force] [crate_names...]
---

# Sync Crate Skills

Scan Cargo.toml and generate skills for dependencies that don't have local skills yet.

Arguments: $ARGUMENTS
- `--force`: Regenerate all skills even if they exist
- `crate_names`: Optional specific crates to sync (space-separated)

---

## Instructions

### 1. Find Cargo.toml Files

```bash
# Check for Cargo.toml in current directory
if [ -f "Cargo.toml" ]; then
    # Check if it's a workspace
    grep -q "\[workspace\]" Cargo.toml
fi
```

**Workspace handling:**
- If `[workspace]` section exists, find `members = [...]`
- Parse each member path
- Collect Cargo.toml from each member directory

### 2. Parse Dependencies

For each Cargo.toml, extract:
- `[dependencies]` section
- `[dev-dependencies]` section

Parse crate names and versions:
```toml
tokio = { version = "1.40", features = ["full"] }
serde = "1.0"
```

### 3. Check Existing Skills

For each crate, check if skill exists:
```bash
ls ~/.claude/skills/{crate_name}/SKILL.md
```

If `--force` flag is set, skip this check.

### 4. Generate Missing Skills

For each missing crate skill:

#### 4a. Check actionbook for llms.txt

```
search_actions("{crate_name} llms.txt")
```

If found:
```
get_action_by_id(action_id)
# Save content to ~/tmp/{crate_name}-llms.txt
```

#### 4b. Generate llms.txt if not in actionbook

If not found in actionbook:
```
/create-llms-for-skills https://docs.rs/{crate_name}/latest/{crate_name}/
```

#### 4c. Create skills from llms.txt

```
/create-skills-via-llms {crate_name} {llms_path} {version}
```

### 5. Report Results

Output summary:
```
Synced skills for:
- tokio (1.40.0) - created
- serde (1.0.215) - created
- axum (0.7.9) - already exists, skipped

Skills location: ~/.claude/skills/
```

---

## Tool Priority

1. **actionbook MCP** - Check for pre-generated llms.txt first
2. **/create-llms-for-skills** - Generate if not in actionbook
   - Uses **agent-browser CLI** (preferred)
   - Falls back to **WebFetch** if agent-browser unavailable
3. **/create-skills-via-llms** - Create skills from llms.txt

**DO NOT use:**
- Chrome MCP for documentation fetching
- Direct Fetch without agent-browser attempt first

---

## Example Usage

```bash
# Sync all dependencies from current project
/sync-crate-skills

# Force regenerate all skills
/sync-crate-skills --force

# Sync specific crates only
/sync-crate-skills tokio serde

# Force regenerate specific crate
/sync-crate-skills --force tokio
```

---

## Output Location

All skills are created in: `~/.claude/skills/`

This is the local dynamic skills directory, not committed to repositories.
