# Social Media Analytics

This guide defines a repeatable quarterly process for reporting on the Soroban Cookbook's public audience and engagement. It is a process and report template, not a claim that account metrics have already been collected. The initial baseline requires a maintainer to confirm the project's official profiles and have authorized access to each platform's analytics.

## Sources and Access

- Start with the [Soroban Cookbook YouTube channel](https://www.youtube.com/@SorobanCookbook). Use YouTube Studio analytics; do not record credentials or private audience data in this repository.
- Track other social profiles only after a maintainer confirms the official account URL and provides an authorized analytics source.
- The [Stellar Community Discord](https://discord.gg/stellardev) is a third-party community, not a Soroban Cookbook-owned social account. Include only public, project-relevant activity that can be reviewed under the server's rules and permissions.
- Use public GitHub activity as a separate community signal, not as a social follower count.

## Quarterly Reporting Process

At the end of each calendar quarter, the maintainer responsible for community reporting should:

1. Confirm which official profiles are in scope and which analytics sources they can access.
2. Collect comparable audience and engagement measurements for the quarter, recording the source and collection date.
3. Review public project mentions and comments; classify sentiment using the guidance below.
4. Complete the report template, call out unavailable metrics and explain why, and assign any follow-up actions an owner and due date.
5. Publish the reviewed report as `docs/reports/social-media/YYYY-QN.md` (for example, `docs/reports/social-media/2026-Q3.md`) and link it from the project README or documentation index.

Do not publish a report with invented values. If a profile, metric, or analytics permission is not confirmed, record it as unavailable and identify what is needed to collect it next quarter.

## Quarterly Metrics

Record the reporting period, collection date, source URL, and metric definition for each platform. If a source does not expose a metric, mark it `Unavailable`; do not estimate or combine incompatible platform measurements.

| Area | Record |
| --- | --- |
| Audience | Followers or subscribers at period start and end; net and percentage change |
| Engagement | Available likes, comments, shares, and other platform-defined interactions; include the denominator used for any rate |
| Mentions | Public mentions found, source links, dates, and whether they are relevant to the project |
| Sentiment | Human-reviewed positive, neutral, negative, or unclear classification, with a short rationale |
| Actions | Trends, follow-up owner, and due date |

Calculate audience change as `period end - period start`. Calculate percentage change as `(period end - period start) / period start * 100`; report it as unavailable when the starting value is zero or missing. For engagement rates, use the platform's documented denominator and state it next to the result. Compare like with like across periods.

Review mentions and comments in context. Classify each relevant item as positive, neutral, negative, or unclear, and summarize recurring themes rather than attributing sentiment to individuals. Sentiment labels are qualitative summaries, not a measurement of individual users; do not scrape private spaces or publish identifying details.

## Quarterly Report Template

Copy this section for each completed reporting period and fill it only from verifiable sources.

### Period: YYYY-QN

**Prepared on:** YYYY-MM-DD  
**Prepared by:** Maintainer or team

| Platform / source | Audience start | Audience end | Net change | Engagement and denominator | Mentions | Sentiment summary |
| --- | ---: | ---: | ---: | --- | ---: | --- |
|  |  |  |  |  |  |  |

**Notable trends:**

- 

**Actions, owner, and due date:**

- 

**Unavailable data and reason:**

- 

Publish one report each calendar quarter after an authorized maintainer has reviewed the source data. Until official profiles are confirmed, analytics access is available, and the first report is published, the analytics baseline remains pending.
