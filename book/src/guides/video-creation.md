# Video Creation Tools & Processes

This guide documents the tools, processes, and standards for creating video content for the Soroban Cookbook.

---

## Recording Software

| Tool | Platform | Use Case |
|------|----------|----------|
| **OBS Studio** (recommended) | Windows / macOS / Linux | Screen recording, webcam overlay |
| **ScreenFlow** | macOS | Polished screencasts with built-in editing |
| **Loom** | Browser | Quick informal walkthroughs |

### OBS Studio Configuration

```
Resolution: 1920x1080 (1080p)
FPS: 30 (screencasts) or 60 (UI demos)
Encoder: x264 / NVENC
Audio: 48 kHz, mono or stereo
Format: MKV (remux to MP4 after recording)
```

**Recommended scenes:**
- **Code Only** — full-screen IDE/terminal capture
- **Code + Webcam** — small webcam overlay in bottom-right corner
- **Slides + Webcam** — presentation mode with speaker visible

---

## Editing Software

| Tool | Platform | Best For |
|------|----------|----------|
| **DaVinci Resolve** (recommended) | Cross-platform | Professional editing, free tier available |
| **Kdenlive** | Linux | Open-source, lightweight |
| **iMovie** | macOS | Quick simple edits |

### DaVinci Resolve Project Settings

```
Timeline Resolution: 1920x1080
Timeline Frame Rate: 30 fps
Color Science: DaVinci YRGB
Export Codec: H.264 (YouTube optimized)
Export Bitrate: 10-15 Mbps
Audio: AAC 320 kbps
```

---

## Template & Style Guide

### Video Structure Template

Every Soroban Cookbook video follows this structure:

```
1. INTRO (15-30s)
   - Cookbook branding bumper (3s)
   - Topic title card
   - Brief "what you'll learn" overview

2. PREREQUISITES (15-30s)
   - Required knowledge
   - Tools needed
   - Link to relevant setup guide

3. MAIN CONTENT (4-6 min)
   - Concept explanation (diagrams/slides)
   - Live coding demonstration
   - Testing & verification

4. RECAP (30-60s)
   - Key takeaways (bullet points)
   - Link to source code in the Cookbook
   - Next video teaser

5. OUTRO (10s)
   - Subscribe/like CTA
   - Community links
```

### Visual Style Guide

| Element | Specification |
|---------|--------------|
| Font (code) | JetBrains Mono, 18px |
| Font (titles) | Inter Bold, 48px |
| Font (body) | Inter Regular, 24px |
| Background | Dark theme (#1a1a2e) |
| Accent color | Stellar blue (#3b82f6) |
| Code theme | One Dark Pro or Catppuccin Mocha |
| Aspect ratio | 16:9 |

### Branding Assets

Store in `assets/video/`:
```
assets/video/
  intro-bumper.mp4       # 3-second animated intro
  outro-card.png         # End screen template
  lower-third.png        # Speaker name overlay
  thumbnail-template.psd # YouTube thumbnail
  logo-watermark.png     # Corner watermark
```

### Thumbnail Guidelines

- **Resolution:** 1280x720 (minimum)
- **Text:** Max 5 words, readable at small sizes
- **Include:** Code snippet preview or diagram
- **Color:** High contrast, use Stellar blue accent

---

## YouTube Channel Setup

### Channel Configuration

```yaml
Channel Name: Soroban Cookbook
Handle: @SorobanCookbook
Category: Science & Technology
Description: |
  Learn Soroban smart contract development through practical examples.
  From basics to advanced patterns — building on Stellar, one recipe at a time.

  GitHub: https://github.com/Soroban-Cookbook/Soroban-Cookbook
  Docs: https://soroban-cookbook.github.io/Soroban-Cookbook/

Playlists:
  - "Getting Started with Soroban"
  - "Soroban Basics"
  - "DeFi Patterns"
  - "Governance & Voting"
  - "Testing & Best Practices"
  - "Advanced Patterns"
```

### Video Settings (Defaults)

```yaml
Visibility: Public
License: Creative Commons - Attribution
Category: Education
Language: English
Comments: Enabled (hold for review)
Tags:
  - soroban
  - stellar
  - smart contracts
  - rust
  - blockchain development
  - web3
```

---

## Upload Process

### Pre-Upload Checklist

- [ ] Video exported in H.264 at 1080p / 30fps
- [ ] Audio levels normalized (-14 LUFS target)
- [ ] Captions generated (auto or manual review)
- [ ] Thumbnail created from template
- [ ] Description written with timestamps
- [ ] Code links verified (point to correct branch/commit)
- [ ] Related cookbook page linked

### Upload Workflow

```
1. Export final video from DaVinci Resolve
   → Format: MP4, H.264, 10-15 Mbps
   → Audio: AAC 320 kbps

2. Upload to YouTube Studio
   → Set title: "[Category] Topic Name | Soroban Cookbook"
   → Add to relevant playlist
   → Set thumbnail
   → Add end screen (last 20s)
   → Add cards linking to related videos

3. Write description (template below)

4. Set publish time
   → Consistent schedule (e.g., Tuesdays 14:00 UTC)
   → Or publish immediately for launch content

5. Update cookbook documentation
   → Add video embed/link to relevant mdBook page
   → Update SUMMARY.md if new page created
   → Commit: "docs: add video link for <topic>"
```

### Description Template

```markdown
[Brief 1-2 sentence summary of what the video covers]

In this video, we [action verb] [topic] using [approach].

📖 Source Code: [GitHub link to example]
📚 Written Guide: [mdBook page link]

⏱️ Timestamps:
00:00 - Introduction
00:30 - Prerequisites
01:00 - [First section]
03:00 - [Second section]
05:00 - Testing
06:00 - Recap & Next Steps

🔗 Resources:
- Soroban Cookbook: https://soroban-cookbook.github.io/Soroban-Cookbook/
- Soroban Docs: https://soroban.stellar.org/docs
- Stellar Discord: https://discord.gg/stellar

#soroban #stellar #smartcontracts #rust #blockchain
```

### Post-Upload Steps

1. **Verify playback** — watch first 30s, check audio sync
2. **Check captions** — review auto-generated subtitles for accuracy
3. **Share** — post link in Stellar Discord and relevant channels
4. **Cross-reference** — update the mdBook page with embedded video link
5. **Monitor** — check comments in first 24h for questions

---

## Quick Reference

| Task | Tool | Command/Action |
|------|------|----------------|
| Record screen | OBS Studio | Start Recording (Ctrl+Shift+R) |
| Edit video | DaVinci Resolve | Import → Edit → Export |
| Generate captions | YouTube Studio | Auto-captions → Review |
| Create thumbnail | Template (PSD) | Edit text + screenshot |
| Upload | YouTube Studio | Create → Upload video |
| Link in docs | mdBook | Add link to `book/src/` page |
