# Community Call Recording Workflow

This document defines the recording, captioning, upload, and archival process for Soroban Cookbook monthly community calls.

## Overview

```
Record (Jitsi) → Upload (YouTube) → Caption (YouTube auto + review) → Archive (GOVERNANCE/notes/)
```

## 1. Recording

### Method A: Jitsi Local Recording (default)

The host initiates local recording from within the Jitsi meeting.

**Steps:**

1. Open the meeting at `https://meet.jit.si/SorobanCookbookCommunityCall`.
2. Click the "..." (More) menu → **Start recording**.
3. Select **Local recording** (records to the host's browser).
4. Confirm — recording starts and an indicator appears for all participants.
5. At the end of the call, click **Stop recording**.
6. The file downloads automatically as a `.webm` (video) file.

**Notes:**

- Local recording captures all audio and video tracks and the screen share.
- The host's browser tab must remain open during recording.
- File size: ~200–500 MB for a 60-minute call.

### Method B: Server-side Jibri (self-hosted only)

If the project self-hosts Jitsi with [Jibri](https://github.com/jitsi/jibri), recording is server-side and does not depend on the host's browser.

1. Host clicks "Start recording" → Jibri records and saves to the server.
2. The file is available as an MP4 after the call ends.
3. Download from the Jibri recordings directory.

### Method C: OBS Studio (fallback)

If Jitsi recording fails, the co-host can capture the call using [OBS Studio](https://obsproject.com):

1. Set up a screen capture source for the Jitsi browser tab.
2. Capture system audio (for participant audio) and microphone (for host audio).
3. Record locally and export as MP4.

## 2. Post-Recording Processing

### Trimming

1. Use a video editor (e.g., [LosslessCut](https://github.com/mifi/lossless-cut), FFmpeg, or DaVinci Resolve) to trim the first 2 minutes (pre-call hallway chat) and the last 5 minutes if needed.
2. Export as MP4 (H.264 video, AAC audio) for YouTube compatibility.

### FFmpeg one-liner (trim first 2 minutes):

```bash
ffmpeg -ss 00:02:00 -i raw-recording.webm -c:v libx264 -c:a aac -movflags +faststart call-recording.mp4
```

## 3. Upload

### Primary destination: YouTube

1. Sign in to the project's YouTube account.
2. Upload the trimmed MP4.
3. Set visibility to **Unlisted** (accessible by link only) or **Public** per the host's decision.
4. Title: `Soroban Cookbook Community Call — YYYY-MM`.
5. Description:
   ```
   Monthly community call for the Soroban Cookbook project.

   Date: YYYY-MM-DD
   Agenda: [link]
   Notes & transcript: [link]

   License: MIT/Apache-2.0 (same as the repository)
   ```
6. Add to the "Community Calls" playlist.

### Alternative: PeerTube or media host

If the project prefers a decentralized host, upload to [PeerTube](https://joinpeertube.org) or an S3 bucket with a public link. The workflow is the same — produce an MP4 and make it accessible via URL.

## 4. Captioning

### Automated captions

YouTube auto-generates captions within 1–2 hours of upload. They appear in the YouTube player's CC menu.

### Review and correction (within 48 hours)

1. Go to YouTube Studio → Subtitles → select the video.
2. Download the auto-generated caption track (`.vtt` or `.srt`).
3. Review and correct technical terms (Soroban, Stellar, smart contracts, etc.).
4. Upload the corrected caption file back to YouTube.
5. Also save a transcript copy to `GOVERNANCE/notes/YYYY-MM-transcript.md` for the archive.

### Captioning with FFmpeg (for non-YouTube hosts)

If using a non-YouTube host, generate captions with:

```bash
# Extract audio
ffmpeg -i call-recording.mp4 -vn -acodec pcm_s16le -ar 16000 -ac 1 audio.wav

# Generate transcript with vosk (requires vosk-api)
python3 -c "
from vosk import Model, KaldiRecognizer
import json, wave
model = Model('vosk-model-small-en-us')
rec = KaldiRecognizer(model, 16000)
wf = wave.open('audio.wav', 'rb')
while True:
    data = wf.readframes(4000)
    if len(data) == 0: break
    if rec.AcceptWaveform(data):
        print(json.loads(rec.Result()).get('text',''))
print(json.loads(rec.FinalResult()).get('text',''))
" > transcript.txt
```

## 5. Archive

### Storage locations

| Artifact | Location | Retention |
| -------- | -------- | --------- |
| Recording (primary) | YouTube playlist | Permanent |
| Transcript | `GOVERNANCE/notes/YYYY-MM-notes.md` | Permanent |
| Timestamped outline | `GOVERNANCE/notes/YYYY-MM-notes.md` | Permanent |
| Raw recording | Cloud storage (S3/Drive) | 90 days, then deleted |
| Slides / demos | Repo (`GOVERNANCE/notes/`) or linked | Permanent |

### Notes file template

Use the notes template from [monthly-call-followup-process.md](../monthly-call-followup-process.md). The recording and transcript links are added to the "Recording" section of that template.

### Archive index

Update the notes directory with an `index.md` listing all past calls:

```markdown
# Community Call Archive

| Date | Recording | Transcript | Notes |
|------|-----------|------------|-------|
| YYYY-MM | [YouTube](link) | [transcript](link) | [notes](link) |
```

## 6. Retention Policy

- YouTube recordings: kept permanently (links remain valid).
- Raw recordings in cloud storage: deleted after 90 days once the YouTube version is live.
- Per the [follow-up process](../monthly-call-followup-process.md), recordings older than 2 years with no active references may move to cold storage, but links must remain valid.
- All materials are licensed under MIT/Apache-2.0 (same as the repository).

## 7. Troubleshooting

| Issue | Solution |
| ----- | -------- |
| Local recording failed to download | Re-join the meeting and try OBS Studio fallback |
| No audio in recording | Check Jitsi audio settings; ensure "Start audio" was not muted |
| WebM won't play | Convert with `ffmpeg -i raw.webm -c:v libx264 -c:a aac out.mp4` |
| Auto-captions inaccurate | Manual review is required per step 4 — focus on technical terms |
| YouTube upload too slow | Use `ffmpeg` to reduce resolution to 720p before upload |
