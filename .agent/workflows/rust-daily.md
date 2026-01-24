---
description: Generate Rust daily/weekly/monthly news report
argument-hint: [day|week|month] [--category ecosystem|official|foundation]
---

# Rust Daily Report

Generate a summarized report of Rust news from multiple sources.

Arguments: $ARGUMENTS
- `time_range` (optional): `day` | `week` | `month` (default: `week`)
- `--category` (optional): `ecosystem` | `official` | `foundation` | `all` (default: `all`)

---

## Sources

| Category | Sources |
|----------|---------|
| **Ecosystem** | Reddit r/rust, This Week in Rust |
| **Official** | Rust Blog, Inside Rust Blog |
| **Foundation** | Rust Foundation News, Blog, Events |

---

## Instructions

### 1. Parse Arguments

```
/rust-daily              → week, all categories
/rust-daily day          → last 24 hours, all
/rust-daily week         → last 7 days, all
/rust-daily month        → last 30 days, all
/rust-daily --category ecosystem  → week, ecosystem only
/rust-daily day --category official → day, official only
```

### 2. Check Cache

Check if recent cache exists:

```bash
cache_dir=~/.claude/cache/rust-daily/
cache_file=${cache_dir}/report-{date}-{time_range}-{category}.json

# If cache exists and < 4 hours old, use cached data
```

### 3. Launch Agent

Read and launch the rust-daily-reporter agent:

```
1. Read agents/rust-daily-reporter.md
2. Task(
     subagent_type: "general-purpose",
     run_in_background: true,  # Can work while fetching
     prompt: "Generate Rust news report.
       Time range: {time_range}
       Category: {category}
       Follow the rust-daily-reporter.md instructions."
   )
3. Wait for completion or continue other work
```

### 4. Format Output

Display the report in markdown format:

```markdown
# 🦀 Rust {Time_Range} Report

**Period:** {start_date} - {end_date} | **Generated:** {now}

---

## Ecosystem Highlights
{Reddit and TWIR content}

## Official Announcements
{Rust Blog and Inside Rust content}

## Foundation Updates
{Foundation news, blog, events}

---

**Next update:** Run `/rust-daily` to refresh
```

### 5. Save Cache

Save results for faster subsequent queries:

```bash
mkdir -p ~/.claude/cache/rust-daily/
# Save JSON with metadata
```

---

## Example Usage

```bash
# Get weekly Rust news (default)
/rust-daily

# Get today's Rust news
/rust-daily day

# Get monthly summary
/rust-daily month

# Get only ecosystem updates (Reddit, TWIR)
/rust-daily --category ecosystem

# Get official Rust project updates only
/rust-daily --category official

# Get Rust Foundation updates only
/rust-daily --category foundation

# Combine: today's official updates
/rust-daily day --category official
```

---

## Output Example

```markdown
# 🦀 Rust Weekly Report

**Period:** 2025-01-10 - 2025-01-17 | **Generated:** 2025-01-17 10:30

---

## 🌐 Ecosystem Highlights

### Reddit r/rust Hot Posts
| 🔥 | Title | Comments |
|----|-------|----------|
| 542 | [ANN] Tokio 2.0 released with... | 89 |
| 423 | Why I switched from Go to Rust | 156 |
| 312 | New async book chapter released | 45 |

### This Week in Rust #580
- **Crate of the Week:** `nom` parser combinator
- **Call for Participation:** Rust documentation improvements
- **Updates from Rust Community:** 5 blog posts, 3 videos

---

## 📢 Official Announcements

### Rust Blog
| Date | Post |
|------|------|
| Jan 15 | Announcing Rust 1.85.0 |
| Jan 12 | Security advisory for cargo |

### Inside Rust
| Date | Post |
|------|------|
| Jan 14 | Lang team design meeting notes |
| Jan 11 | Compiler team sprint recap |

---

## 🏛️ Rust Foundation

### News
- Jan 13: New Foundation member announcement

### Upcoming Events
| Date | Event | Location |
|------|-------|----------|
| Feb 1-3 | RustConf 2025 | Seattle, WA |
| Feb 15 | Rust Meetup | Virtual |

---

📊 **Summary:** 12 ecosystem posts, 4 official announcements, 2 foundation updates
🔄 **Refresh:** `/rust-daily` | 📅 **Archive:** `~/.claude/cache/rust-daily/`
```

---

## Tool Priority

1. **Task + rust-daily-reporter agent** (preferred)
2. **WebFetch** (fallback if agent unavailable)

**DO NOT use WebSearch** - use the structured agent approach for better results.

---

## Related Commands

- `/rust-features [version]` - Rust version changelog
- `/crate-info <crate>` - Crate information
- `/sync-crate-skills` - Sync project dependencies
