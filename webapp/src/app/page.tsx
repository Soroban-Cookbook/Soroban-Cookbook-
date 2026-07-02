"use client";

import { useDeferredValue, useState } from "react";

type ShowcaseProject = {
  name: string;
  category: string;
  status: string;
  stack: string;
  summary: string;
  cookbookPatterns: string[];
};

const categories = [
  "All",
  "DeFi",
  "Infrastructure",
  "Governance",
  "Tokens",
  "Tooling",
  "Education",
];

const projects: ShowcaseProject[] = [
  {
    name: "Canary Bridge Monitor",
    category: "Infrastructure",
    status: "Testnet",
    stack: "Rust, Soroban, indexer",
    summary: "Watches bridge release events and flags suspicious flows for manual review.",
    cookbookPatterns: ["Bridge security", "Events", "Timelock"],
  },
  {
    name: "LedgerLift Vaults",
    category: "DeFi",
    status: "Production",
    stack: "Soroban, React, analytics",
    summary: "Automated vault strategies with pause controls and operator approvals.",
    cookbookPatterns: ["Multi-party auth", "Pause / unpause", "Token wrapper"],
  },
  {
    name: "GovMesh",
    category: "Governance",
    status: "Prototype",
    stack: "Soroban, Next.js",
    summary: "Proposal and execution flow with timelock and role-based approvals.",
    cookbookPatterns: ["Timelock", "RBAC modifiers", "Authentication"],
  },
  {
    name: "Sep41 Starter Kit",
    category: "Tokens",
    status: "Testnet",
    stack: "Soroban SDK, TypeScript scripts",
    summary: "Launch scaffold for metadata-rich fungible tokens with clean event indexing.",
    cookbookPatterns: ["SEP-41 token", "Events", "Validation patterns"],
  },
  {
    name: "Soroscope",
    category: "Tooling",
    status: "Production",
    stack: "Next.js, API workers, indexer",
    summary: "Searches contract events and example usage across a Soroban workspace.",
    cookbookPatterns: ["Cross-contract patterns", "Events", "Testing"],
  },
  {
    name: "Campus Soroban Lab",
    category: "Education",
    status: "Prototype",
    stack: "mdBook, workshop scripts",
    summary: "Teaching environment that packages cookbook examples into guided labs.",
    cookbookPatterns: ["Hello world", "Storage patterns", "Custom errors"],
  },
];

const submissionSteps = [
  "Prepare a short summary, repo URL, demo URL, and one screenshot or clip.",
  "State which cookbook examples, guides, or patterns the project used directly.",
  "Pick the best-fit category and current status so the listing stays searchable.",
  "Open a contribution with the showcase details and any supporting links.",
];

const templates = [
  {
    name: "Starter App",
    fit: "Teams shipping their first Soroban project from one or two cookbook examples.",
  },
  {
    name: "Protocol Integration",
    fit: "Apps combining tokens, auth, cross-contract flows, or governance patterns.",
  },
  {
    name: "Developer Tooling",
    fit: "Indexers, SDK helpers, testing harnesses, analytics, and operational tooling.",
  },
];

export default function HomePage() {
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState("All");
  const deferredQuery = useDeferredValue(query);
  const normalizedQuery = deferredQuery.trim().toLowerCase();

  const filteredProjects = projects.filter((project) => {
    const inCategory = category === "All" || project.category === category;
    const inSearch =
      normalizedQuery.length === 0 ||
      project.name.toLowerCase().includes(normalizedQuery) ||
      project.summary.toLowerCase().includes(normalizedQuery) ||
      project.stack.toLowerCase().includes(normalizedQuery) ||
      project.cookbookPatterns.some((pattern) =>
        pattern.toLowerCase().includes(normalizedQuery)
      );

    return inCategory && inSearch;
  });

  return (
    <div style={{ display: "grid", gap: "24px" }}>
      <section className="terminal-card" style={{ display: "grid", gap: "16px" }}>
        <div style={{ color: "#1d771d", fontSize: "0.8rem" }}>[ PROJECT_SHOWCASE ]</div>
        <h1 style={{ marginBottom: 0 }}>Built With The Soroban Cookbook</h1>
        <p style={{ maxWidth: "780px", lineHeight: 1.6 }}>
          A searchable catalog of projects, teams, and tools that apply the cookbook in real work.
          Keep technical reference content in mdBook and use the webapp for discovery, filtering, and submissions.
        </p>
      </section>

      <section
        className="terminal-card"
        style={{ display: "grid", gap: "16px", gridTemplateColumns: "repeat(auto-fit, minmax(240px, 1fr))" }}
      >
        <div>
          <label className="terminal-label" htmlFor="search-projects">Search Projects</label>
          <input
            id="search-projects"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="bridge, sep-41, governance, tooling"
          />
        </div>
        <div>
          <label className="terminal-label" htmlFor="category-filter">Filter By Category</label>
          <select
            id="category-filter"
            value={category}
            onChange={(event) => setCategory(event.target.value)}
          >
            {categories.map((option) => (
              <option key={option} value={option}>
                {option}
              </option>
            ))}
          </select>
        </div>
        <div style={{ alignSelf: "end", color: "#1d771d", fontSize: "0.85rem" }}>
          MATCHES: {filteredProjects.length}
        </div>
      </section>

      <section
        style={{ display: "grid", gap: "20px", gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))" }}
      >
        {filteredProjects.map((project) => (
          <article key={project.name} className="terminal-card" style={{ display: "grid", gap: "12px" }}>
            <div style={{ display: "flex", justifyContent: "space-between", gap: "12px", flexWrap: "wrap" }}>
              <strong className="glow-text">{project.name}</strong>
              <span className="amber-text">{project.status}</span>
            </div>
            <div style={{ color: "#1d771d", fontSize: "0.85rem" }}>
              CATEGORY: {project.category} | STACK: {project.stack}
            </div>
            <p style={{ lineHeight: 1.55 }}>{project.summary}</p>
            <div style={{ display: "flex", gap: "8px", flexWrap: "wrap" }}>
              {project.cookbookPatterns.map((pattern) => (
                <span
                  key={pattern}
                  style={{ border: "1px solid #222", padding: "6px 10px", fontSize: "0.8rem" }}
                >
                  {pattern}
                </span>
              ))}
            </div>
          </article>
        ))}
      </section>

      <section
        style={{ display: "grid", gap: "20px", gridTemplateColumns: "repeat(auto-fit, minmax(320px, 1fr))" }}
      >
        <div className="terminal-card" style={{ display: "grid", gap: "12px" }}>
          <h2 style={{ marginBottom: 0 }}>Submission Process</h2>
          {submissionSteps.map((step, index) => (
            <p key={step} style={{ lineHeight: 1.55 }}>
              {index + 1}. {step}
            </p>
          ))}
        </div>

        <div className="terminal-card" style={{ display: "grid", gap: "12px" }}>
          <h2 style={{ marginBottom: 0 }}>Project Templates</h2>
          {templates.map((template) => (
            <div key={template.name} style={{ border: "1px solid #222", padding: "12px" }}>
              <div className="glow-text" style={{ marginBottom: "6px" }}>{template.name}</div>
              <p style={{ lineHeight: 1.55 }}>{template.fit}</p>
            </div>
          ))}
        </div>
      </section>

      <section className="terminal-card" style={{ display: "grid", gap: "12px" }}>
        <h2 style={{ marginBottom: 0 }}>Categories</h2>
        <div style={{ display: "flex", gap: "10px", flexWrap: "wrap" }}>
          {categories.filter((item) => item !== "All").map((item) => (
            <span key={item} style={{ border: "1px solid #222", padding: "8px 12px" }}>
              {item}
            </span>
          ))}
        </div>
      </section>
    </div>
  );
}