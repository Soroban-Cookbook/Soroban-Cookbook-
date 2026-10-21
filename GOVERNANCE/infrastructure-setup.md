# Community Call Infrastructure Setup

This document defines the concrete infrastructure for Soroban Cookbook monthly community calls. It satisfies the acceptance criteria of issue #379/#913: video conferencing platform selected, calendar created, registration system, recording setup, and documentation.

For process templates (agenda, moderation, Q&A, follow-up), see the other files in the [GOVERNANCE/](./README.md) directory.

---

## 1. Video Conferencing Platform

### Selection: Jitsi Meet

**Platform:** [Jitsi Meet](https://jitsi.org) — open-source, browser-based video conferencing.

**Meeting URL:** `https://meet.jit.si/SorobanCookbookCommunityCall`

**Rationale:**

| Requirement (from format guidelines) | Jitsi Meet Support |
| ------------------------------------- | ------------------ |
| Recording                             | Local recording (host-initiated) or server-side via Jibri deployment |
| Closed captions                       | Live captions via Vosk integration; transcript download available |
| Breakout rooms                        | Native support (up to 8 breakouts per call) |
| Chat / Q&A                            | Built-in chat with raise-hand and moderation tools |
| Cost                                  | Free on public instance (meet.jit.si); self-hostable for full control |
| Open source                           | Yes — Apache 2.0 license; aligns with project ethos |
| Browser access                        | No install required; works in Chromium, Firefox, Safari |

**Configuration:**

- Room name: `SorobanCookbookCommunityCall` (persistent on meet.jit.si)
- Lobby mode: Enabled — co-host admits attendees from the waiting room
- Password: Set per call and shared via the registration thread and calendar description
- Live streaming: Optional simulcast to YouTube Live via Jitsi's "Start Live Stream" feature (requires YouTube stream key)

**Host setup checklist (before each call):**

1. Open `https://meet.jit.si/SorobanCookbookCommunityCall` 10 minutes early.
2. Enable lobby mode (Settings → More → Enable lobby).
3. Assign co-host role to the designated co-host for the month.
4. Test audio, video, and screen share.
5. Start local recording (if not using server-side Jibri).

**Self-hosting option:**

For organizations that prefer full control, Jitsi can be self-deployed using the official [Jitsi Meet Docker](https://github.com/jitsi/docker-jitsi-meet) setup. This adds server-side recording (Jibri) and live transcription. See the deployment guide for details.

---

## 2. Calendar

**Calendar file:** [calendar/soroban-cookbook-community-call.ics](./calendar/soroban-cookbook-community-call.ics)

**Schedule:**

| Field | Value |
| ----- | ----- |
| Recurrence | First Tuesday of every month |
| Time | 16:00 UTC |
| Duration | 60 minutes (+ 5-minute buffer) |
| Time zones | 11:00 EST, 08:00 PST, 17:00 CET, 01:00+1 JST |
| Calendar system | ICS file in repo; also published as a GitHub Discussions thread |

The ICS file uses an `RRULE` for monthly recurrence on the first Tuesday. Import it into Google Calendar, Apple Calendar, Outlook, or any CalDAV client.

**Publishing the calendar:**

- The ICS file lives in the repository at `GOVERNANCE/calendar/`.
- A raw link is provided in CONTRIBUTING.md and the GOVERNANCE README for easy subscription.
- If the schedule changes, update the ICS file and the format guidelines in the same PR.

---

## 3. Registration System

**System:** GitHub Discussions — no external tooling required.

**Registration guide:** [registration/README.md](./registration/README.md)

The registration system uses a monthly GitHub Discussions thread where attendees RSVP. This keeps everything in-repo, accessible without an external account, and provides a natural place for pre-call questions (which the Q&A process already references).

**Monthly workflow:**

1. Host creates a new Discussion in the "Community Calls" category titled `Community Call RSVP — YYYY-MM`.
2. The first comment contains the registration form template (see [registration/README.md](./registration/README.md)).
3. Attendees reply with their RSVP, timezone, and topics of interest.
4. The host pins the discussion and links it from the agenda.

**Why GitHub Discussions over external forms:**

- No third-party service or API keys to manage.
- Attendees already have GitHub accounts (this is a developer community).
- Discussion threads naturally support pre-call questions and post-call follow-up.
- All data stays in the repository and is publicly auditable.

---

## 4. Recording Setup

**Recording guide:** [recording/README.md](./recording/README.md)

**Workflow:**

1. **Record:** Host initiates local recording in Jitsi (or server-side via Jibri if self-hosted). Recording includes both audio and video tracks.
2. **Upload:** After the call, the note-taker uploads the recording to the project's YouTube channel (or preferred media host). Upload as unlisted if the project prefers limited distribution.
3. **Caption:** YouTube auto-generates captions within 1–2 hours. The note-taker reviews and corrects them within 48 hours.
4. **Archive:** Link the recording, transcript, and timestamped outline in `GOVERNANCE/notes/YYYY-MM-notes.md`.
5. **License:** All recordings are published under the same open license as the repository (MIT/Apache-2.0).

**Storage locations:**

| Artifact | Location |
| -------- | -------- |
| Recording (primary) | YouTube (or media host) — linked from notes |
| Transcript | `GOVERNANCE/notes/YYYY-MM-notes.md` |
| Raw recording (optional) | Cloud storage (S3, Google Drive) — retained 90 days |
| Slides and demos | Linked from notes; stored in repo if markdown/text |

**Retention:**

- Recordings and transcripts are permanently linked from the notes archive.
- Raw recordings older than 90 days may be deleted from cloud storage once the edited version is on YouTube.
- Per the follow-up process, recordings older than 2 years with no active references may move to cold storage, but links must remain valid.

---

## 5. Documentation

All infrastructure is documented in the following in-repo files:

| Document | Purpose |
| -------- | ------- |
| [infrastructure-setup.md](./infrastructure-setup.md) | This document — platform, calendar, registration, recording overview |
| [calendar/soroban-cookbook-community-call.ics](./calendar/soroban-cookbook-community-call.ics) | Importable recurring calendar event |
| [registration/README.md](./registration/README.md) | Registration form template and workflow |
| [recording/README.md](./recording/README.md) | Recording, captioning, upload, and archival workflow |
| [monthly-call-format-guidelines.md](./monthly-call-format-guidelines.md) | Call format, platform standards, promotion runway |
| [monthly-call-agenda-template.md](./monthly-call-agenda-template.md) | Agenda template for each call |
| [monthly-call-moderation-guide.md](./monthly-call-moderation-guide.md) | Moderation roles and escalation |
| [monthly-call-qa-process.md](./monthly-call-qa-process.md) | Question collection and Q&A management |
| [monthly-call-followup-process.md](./monthly-call-followup-process.md) | Post-call follow-up and action item tracking |

---

## Acceptance Criteria Checklist

| Criterion | Status | Evidence |
| --------- | ------ | -------- |
| Video conferencing platform selected | ✅ | Jitsi Meet — see Section 1 |
| Calendar created | ✅ | ICS file — see Section 2 |
| Registration system | ✅ | GitHub Discussions — see Section 3 |
| Recording setup | ✅ | Jitsi local recording + YouTube archive — see Section 4 |
| Documentation | ✅ | This document + linked files — see Section 5 |
