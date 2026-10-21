# User Surveys & Community Feedback Process

This directory governs the operational workflow, schedules, templates, and reporting of user surveys conducted for the **Soroban-Cookbook** repository.

---

## 📅 Distribution Schedule
Surveys are conducted on a **Quarterly Schedule** to ensure we consistently capture changes in developer sentiment, identify new developer needs promptly, and guide each phase of the cookbook development.

| Quarter | Distribution Date | Analysis & Reporting Date | Roadmap & Issue Integration |
|:---|:---|:---|:---|
| **Q1** | January 1 – January 15 | January 20 – January 25 | February 1 (Prioritize Q2 issues) |
| **Q2** | April 1 – April 15 | April 20 – April 25 | May 1 (Prioritize Q3 issues) |
| **Q3** | July 1 – July 15 | July 20 – July 25 | August 1 (Prioritize Q4 issues) |
| **Q4** | October 1 – October 15 | October 20 – October 25 | November 1 (Prioritize next year) |

---

## 🔄 Operational Workflow & Feedback Loop

Our community survey feedback loop operates in four structured stages:

```
+--------------------+      +--------------------+      +--------------------+      +--------------------+
|  1. Distribution   | ---> |    2. Analysis     | ---> |  3. Action Plan    | ---> | 4. Reporting Back  |
|  - Google Forms    |      |  - Tabulate quantitative|  - Map feedback to |      |  - Publish results |
|  - Discussions     |      |  - Sentiment analysis|    GitHub issues   |      |  - Open discussion |
+--------------------+      +--------------------+      +--------------------+      +--------------------+
```

### 1. Distribution & Collection
- **Platforms**:
  - **Google Forms & Typeform**: Used for quantitative questions, anonymous submissions, and high-level satisfaction trends.
  - **GitHub Discussions**: A dedicated thread is pinned at the start of each quarter under the "Surveys" category.
  - **Discord Announcement**: Pinned reminders are posted in the official Stellar Discord in `#soroban` and developer channels.
- **Duration**: Active collection runs for exactly **15 days**.

### 2. Analysis & Review
- Once the survey closes, the maintainer team compiles the quantitative rating scales (demographics, clarity, ease of setup).
- Open-ended suggestions are categorized by theme (e.g., "Requesting dynamic NFTs", "Improving macOS setup docs", "Testing mock issues").
- All responses are reviewed weekly during maintainer syncs.

### 3. Action Planning & Prioritization
- Feedback items meeting critical ecosystem needs are mapped to **actionable repository issues**.
- Issues are tagged with specific labels:
  - `community-feedback`: Any issue initiated from survey results.
  - `enhancement` / `documentation` / `bug`: Standard labels for routing.
- The most highly requested missing patterns or tutorials are immediately scheduled into the repository roadmap (`ROADMAP.md`).

### 4. Reporting Back to the Community
- An outcomes report (e.g., `Q3_2026_SURVEY_RESULTS.md`) is compiled and published in this folder (`docs/feedback-system/surveys/`).
- The team publishes a summary post in GitHub Discussions and Discord highlighting what we heard, what we are prioritizing, and links to the newly created GitHub issues. This ensures full transparency and closes the feedback loop.

---

## 🚀 How to Execute a New Survey (For Maintainers)

1. **Verify Config**: Ensure `docs/feedback-system/surveys/config.yml` is updated with any new question IDs or configuration parameters.
2. **Setup Platform**: Copy the questions from `USER_SURVEY_TEMPLATE.md` to your external platform (e.g., Google Forms).
3. **Launch**: Post announcements on GitHub, Twitter/X, and Discord using communication templates.
4. **Compile Outcomes**: At the end of the collection period, draft the outcomes report in this directory following the `Q3_2026_SURVEY_RESULTS.md` format.
5. **Create Issues**: Link identified actions to GitHub issues and update the roadmap.
