# Social Media Analytics

This guide defines a repeatable quarterly process for reporting on the Soroban Cookbook's public audience and engagement. It is a process and report template, not a claim that account metrics have already been collected. The initial baseline requires an authorized maintainer with access to the relevant platform analytics.

## Sources and Access

- Start with the [Soroban Cookbook YouTube channel](https://www.youtube.com/@SorobanCookbook). Use YouTube Studio analytics; do not record credentials or private audience data in this repository.
- Track other social profiles only after a maintainer confirms the official account URL and provides an authorized analytics source.
- The [Stellar Community Discord](https://discord.gg/stellardev) is a third-party community, not a Soroban Cookbook-owned social account. Include only public, project-relevant activity that can be reviewed under the server's rules and permissions.
- Use public GitHub activity as a separate community signal, not as a social follower count.

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

Review mentions and comments in context. Sentiment labels are qualitative summaries, not a measurement of individual users, and should not expose personal or private information. Do not scrape private spaces or publish identifying details.

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

Publish one report each quarter after an authorized maintainer has reviewed the source data. Until that access and first report exist, the analytics baseline remains pending.