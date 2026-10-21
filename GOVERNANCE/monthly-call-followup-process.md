# Monthly Call Follow-up Process

The follow-up process converts discussions from the monthly call into documented actions, recordings, and feedback for the next cycle.

## Timeline

| Deadline | Owner | Action |
| -------- | ----- | ------ |
| +24 hours | Note-taker | Publish raw notes to `GOVERNANCE/notes/YYYY-MM-notes.md` |
| +48 hours | Host + Note-taker | Create follow-up entry in GitHub Discussions with recording, transcript, and action items |
| +72 hours | Note-taker | Publish closed captions and final transcript link |
| +7 days | Maintainer | Review action items and sync with GitHub Issues and project board |

## Notes Template

Copy and complete this template for every call:

```markdown
# Community Call Notes — YYYY-MM

## Recording

- Link: ...
- Transcript: ...

## Decisions

- List any decisions made during the call.

## Action Items

| Action | Owner | Due Date | GitHub Issue / Status |
|--------|-------|----------|----------------------|
| ...    | ...   | ...      | #issue / open         |

## Questions Carried Forward

| Question | Owner | Due Date |
|----------|-------|----------|
| ...      | ...   | ...      |
```

## Action Item Handling

- Each action item must have a single owner and a due date.
- If no owner volunteers during the call, the host assigns it to a maintainer.
- Duplicate or stale action items are merged into the existing issue before the next call.
- Action items that are not resolved by the next call are reviewed in the follow-up process and either extended or closed with explanation.

## Storage & Archive

- Call notes, recordings, and transcripts are stored under `GOVERNANCE/notes/`.
- The archive index is updated in [docs/README.md](../book/src/README.md) or linked from the community section.
- Recordings older than 2 years that have no active references may be moved to cold storage, but links must remain valid.

## Feedback Loop

- After publishing the follow-up, prompt attendees to rate the call via a short form or GitHub Discussions thread.
- Collect feedback on:
  - Audio/video quality
  - Agenda pacing
  - Speaker content
  - Q&A effectiveness
- Review feedback at the start of the planning call for the next session.

## Escalation

- If an action item is blocked, the owner notifies the host within 48 hours.
- The host escalates to the maintainer team during the weekly async sync.
- If a participant feels an action item was missed or mishandled, they open a GitHub Issue tagged `community-followup`.
