# Community Call Registration

Registration for Soroban Cookbook monthly community calls uses GitHub Discussions — no external tooling or sign-up required.

## How to Register

1. Go to the [GitHub Discussions](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/discussions) tab.
2. Find the pinned thread titled **Community Call RSVP — YYYY-MM** (one is created for each call).
3. Reply to the thread using the registration form below.
4. That's it — you're registered. A reminder will go out 1 hour before the call via the calendar event.

## Registration Form Template

The host pastes this template into the first comment of each monthly RSVP Discussion thread:

```markdown
## Soroban Cookbook Community Call — YYYY-MM-DD

**Date:** YYYY-MM-DD
**Time:** 16:00 UTC (11:00 EST / 08:00 PST / 17:00 CET)
**Duration:** 60 minutes
**Platform:** Jitsi Meet — https://meet.jit.si/SorobanCookbookCommunityCall
**Agenda:** [link to agenda]

---

### RSVP

Reply to this thread with the following:

- [ ] Attending — add your name/GitHub handle below
- [ ] Interested but can't make this date
- [ ] Watching the recording later

### Attendee Info (optional)

| GitHub handle | Timezone | Topics of interest |
|---------------|----------|-------------------|
| @yourhandle   | UTC-5    | Token patterns, governance |

### Pre-call Questions

Submit questions you'd like discussed during the Q&A segment. One question per comment:

> **Q:** Your question here
> **Topic:** general / technical / meta

---
*Meeting password will be posted here 24 hours before the call.*
```

## Host Workflow

1. **14 days before the call:** Create a new Discussion in the "Community Calls" category. Title it `Community Call RSVP — YYYY-MM`.
2. **Pin the discussion** so it appears at the top of the Discussions list.
3. **10 days before the call:** Announce the RSVP thread on Discord and social channels per the [promotion runway](../monthly-call-format-guidelines.md#promotion-runway).
4. **24 hours before the call:** Edit the thread to add the Jitsi meeting password.
5. **1 hour before the call:** Post a reminder in the thread.
6. **After the call:** Link the recording and notes in the same thread, then close the RSVP discussion.

## Data Collected

| Field | Purpose | Required |
| ----- | ------- | -------- |
| Attendance intent | Headcount and planning | Yes |
| Timezone | Optimize scheduling across regions | No |
| Topics of interest | Shape agenda and feature spotlights | No |
| Pre-call questions | Feed the Q&A segment | No |

All data is public (GitHub Discussions is public by default) and stays in the repository.

## Accessibility

- No external account needed beyond GitHub (which contributors already have).
- The RSVP thread is also the pre-call question channel, reducing the number of places participants need to check.
- Participants who cannot attend live can still register as "watching the recording later" and receive follow-up notifications via the thread.
