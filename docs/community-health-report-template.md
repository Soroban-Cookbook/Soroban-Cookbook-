# Quarterly Community Health Report

Use this template once per calendar quarter. Copy it to
`docs/community-health-reports/YYYY-QN.md` (for example, `2026-Q3.md`) and
publish it within 15 days of quarter end. Link the published report from the
README. Do not fill gaps with estimates: mark unavailable data as `N/A` and
record why.

The first report establishes a baseline. From the second report onward, compare
each metric with the previous quarter and, when available, the same quarter a
year earlier. Describe the direction, likely causes, confidence, and any action
needed; a table of counts alone is not a trend analysis.

## Report Details

- **Reporting period:** [YYYY-MM-DD to YYYY-MM-DD]
- **Published:** [YYYY-MM-DD]
- **Prepared by:** [Maintainer or team]
- **Data checked on:** [YYYY-MM-DD]
- **Comparison period:** [Previous quarter and/or same quarter last year]

## Summary

- **What improved:** [Evidence and context]
- **What needs attention:** [Evidence and context]
- **What is uncertain or missing:** [Data gaps, limitations, or small samples]
- **Overall assessment:** [Healthy / Needs attention / Insufficient data, with rationale]

## Metrics

Use the same definitions and reporting window each quarter. Report counts with
their time window and compare response-time medians rather than averages where
possible. Exclude bots and duplicate events consistently.

| Area | Metric and definition | Current quarter | Previous quarter | Source / notes |
| --- | --- | ---: | ---: | --- |
| Participation | First-time contributors with a merged contribution | [ ] | [ ] | [ ] |
| Participation | Distinct human contributors with a merged PR or substantive review | [ ] | [ ] | [ ] |
| Participation | Contributors active in both this and the previous quarter | [ ] | [ ] | [ ] |
| Contributions | Pull requests opened / merged / closed without merge | [ ] / [ ] / [ ] | [ ] / [ ] / [ ] | [ ] |
| Responsiveness | Median time from PR opening to first maintainer response | [ ] | [ ] | [Hours or days] |
| Responsiveness | Median time from issue opening to first maintainer response | [ ] | [ ] | [Hours or days] |
| Issue health | Issues opened / closed during the quarter | [ ] / [ ] | [ ] / [ ] | [ ] |
| Issue health | Open issues older than 90 days at quarter end | [ ] | [ ] | [ ] |
| Community | Questions or discussion posts receiving a maintainer response within 7 days | [ ] / [ ] | [ ] / [ ] | [ ] |
| Feedback | Survey responses and recurring themes, if a survey ran | [ ] | [ ] | Aggregate only; link the survey summary |
| Safety | Code of Conduct reports received / resolved, if safe to report | [ ] / [ ] | [ ] / [ ] | Never include identifying or case details |

For each metric, state the numerator, denominator, and exclusions when a rate is
used. If a sample is too small to support a meaningful comparison, say so
instead of drawing a conclusion.

## Trend Analysis

Complete one row for each material change or risk. The first report should say
that historical trend data is unavailable and establish a baseline; do not
present a single-quarter snapshot as a trend.

| Metric | Change | Direction | Interpretation and confidence | Follow-up |
| --- | --- | --- | --- | --- |
| [Metric] | [Current vs. prior period; absolute or percent change] | [Improving / stable / worsening / mixed / baseline] | [Likely cause, evidence, caveat] | [Action ID or none, with reason] |

### Qualitative Signals

- **Contributor experience:** [Onboarding friction, review quality, recurring feedback]
- **Inclusion and safety:** [Aggregate observations only; avoid exposing reporters]
- **Documentation and project health:** [Findings from issues, discussions, and feedback]
- **Changes since last report:** [What changed and whether prior actions helped]

## Action Items

Create trackable GitHub issues for actions that need follow-up. Give each action
an owner, due date, and observable completion criteria. Carry open actions into
the next quarterly report.

| ID / issue | Action | Owner | Due date | Success measure | Status |
| --- | --- | --- | --- | --- | --- |
| [#] | [Specific, achievable action] | [Name / team] | [YYYY-MM-DD] | [Observable outcome] | [Open / in progress / done] |

## Data Sources

- [Repository Pulse](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/pulse)
- [Contributor activity](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/graphs/contributors)
- [GitHub Discussions](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/discussions)
- [GitHub community profile guidance](https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions)
- [Code of Conduct](../CODE_OF_CONDUCT.md)

Use only public, aggregated information in the report. Do not publish personal
data, private feedback, or details that could identify someone involved in a
moderation report. Keep sensitive records restricted to the maintainers who
need them.