# Monthly Call Follow-up Process

The follow-up process converts discussions from the monthly call into documented actions, recordings, and feedback for the next cycle.

## Timeline

| Deadline | Owner | Action |
| -------- | ----- | ------ |
| +24 hours | Note-taker | Publish notes to `GOVERNANCE/notes/YYYY-MM-notes.md` using the [notes template](./notes/TEMPLATE.md), then add the call to the [archive index](./notes/README.md) |
| +48 hours | Host + Note-taker | Upload the recording to the project's approved media host, add its stable link to the archive record, and create a GitHub Discussions follow-up with the recording, transcript, and action items |
| +72 hours | Note-taker | Publish reviewed closed captions and the final transcript link; update the archive record |
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

- The [Community Call Archive](./notes/README.md) is the canonical index. Store notes, transcripts, and captions as Markdown or text files under `GOVERNANCE/notes/`; link to large assets rather than committing them.
- Store video recordings on the project's approved media host, not in Git. Add a stable video URL to both the call's archive record and the index, and verify access before publishing.
- The archive index is linked from the repository's community documentation and the mdBook [Community Calls page](../book/src/community-calls.md).
- Keep published media links valid. If a recording must move, update its archive record and the index together.

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
