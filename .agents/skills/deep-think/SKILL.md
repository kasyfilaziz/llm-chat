---
name: deep-think
description: >
  Activates a deliberate, multi-solution reasoning framework that forces the LLM to think slower,
  broader, and deeper before committing to any single answer. Use this skill whenever the user's
  request is ambiguous, consequential, architectural, or involves trade-offs — for example:
  "help me design X", "how should I approach Y", "what's the best way to Z", "I need to decide
  between...", "my system does X but I need Y", or any problem-solving request that would benefit
  from structured exploration rather than a quick first-instinct answer. Also trigger when the user
  says "think carefully", "explore options", "don't rush", "tree of thought", "tot", or "/deep-think".
  This skill prevents premature convergence on a single solution by mandating parallel exploration
  of diverse approaches and explicit consequence mapping before any recommendation is made.
---

# Deep Think — Deliberate Multi-Solution Reasoning

This skill enforces **slow, wide, and deep thinking** before committing to any answer. It is modeled
on the Tree-of-Thought (ToT) reasoning pattern and is designed to work with any LLM.

The core philosophy: **a fast answer is not always the best answer.** By forcing exploration of
multiple distinct solution paths — each with their own consequences — you reduce the risk of
anchoring bias, tunnel vision, and unintended side effects.

---

## When This Skill Applies

Trigger this skill for any request that involves:
- Architectural or design decisions
- Choosing between approaches or technologies
- Debugging a non-obvious problem
- Planning a multi-step process or system
- Any request where being wrong has real cost

Do NOT apply this skill for simple factual lookups, single-step tasks, or when the user
explicitly requests a quick/brief answer.

---

## Phase 0 — User Profiling + Clarification Interview (MANDATORY, do not skip)

Before generating any solutions, this phase collects two types of information: **who the user is**
(Group A) and **what they actually need** (Group B). The goal is to ask everything in a single
message — but only ask what isn't already known.

---

### Step 0.1 — Scan the Context Window First

Before composing any questions, scan the full context window for existing User Profile signals.
These can appear in many forms:

| Source | What to look for |
|--------|-----------------|
| **System prompt** | Explicit user descriptions, role definitions, persona instructions |
| **Memory / user preferences** | Saved profile data, past session context, stated preferences |
| **Earlier in this conversation** | User has already described their role, background, or preferences |
| **The request itself** | Strong implicit signals — e.g., a user who pastes a stack trace and discusses race conditions is clearly technical |

For each of the 3 profile attributes, determine its **resolution status**:

```
Technical Level   : [KNOWN from context | PARTIAL — needs confirmation | UNKNOWN]
Role / Domain     : [KNOWN from context | PARTIAL — needs confirmation | UNKNOWN]
Output Preference : [KNOWN from context | PARTIAL — needs confirmation | UNKNOWN]
```

---

### Step 0.2 — Decide What to Ask

Apply these rules per attribute:

- **KNOWN** → Do not ask. Use the value from context directly.
- **PARTIAL** → Ask a single short confirmation question, not a full open question.
  e.g., *"I see you're a backend developer — is that right, or has your context changed?"*
- **UNKNOWN** → Ask the full Group A question for that attribute.

If **all 3 attributes are KNOWN**, skip Group A entirely and go straight to Group B.
Acknowledge the known profile briefly before the questions:
> *"Based on [source], I'll frame solutions for a [role] with [level] familiarity. A few
> questions about the problem itself before I begin:"*

---

### Group A — User Profile Questions (ask only for UNKNOWN or PARTIAL attributes)

1. **Technical level** — "How familiar are you with [the relevant domain/technology]?"
   > *e.g., "I'm new to this / I have some exposure / I work with this regularly / I'm an expert"*

2. **Role & context** — "What's your role, and how does this problem affect your day-to-day?"
   > *Helps distinguish a business owner asking about tech from a developer asking about
   > business strategy — same words, very different solutions.*

3. **Output preference** — "How would you like the solutions explained?"
   > *e.g., "Plain language with analogies / A mix of both / Full technical detail"*

---

### Group B — Problem Clarification (minimum 3 questions, always required)

These questions surface hidden constraints, unstated goals, and ambiguous scope.
They are **never skipped**, regardless of how much profile context is already available.

**Select questions that expose:**
- **Context**: What system / environment / codebase does this live in?
- **Constraints**: What is non-negotiable? (budget, timeline, tech stack, team skills)
- **Success criteria**: How will the user know the solution worked?
- **Risk tolerance**: Is speed, safety, reversibility, or simplicity the priority?
- **Scope**: Is this a quick fix or a long-term solution?

---

### Composing the Phase 0 Message

Combine only what needs to be asked into a **single message**. Do not split into multiple rounds.

**Scenario A — Profile fully unknown:**
> Before I explore solutions, I have a few questions — some to understand your background,
> some about the problem itself:
>
> **About you:**
> 1. [technical level]
> 2. [role & context]
> 3. [output preference]
>
> **About the problem:**
> 4–6. [Group B questions]

**Scenario B — Profile partially known:**
> Based on [context source], I'll treat you as a [inferred profile]. Just to confirm:
> - [confirmation question for PARTIAL attribute only]
>
> **About the problem:**
> 1–3. [Group B questions]

**Scenario C — Profile fully known:**
> Based on [context source], I'll frame everything for a [role] with [level] familiarity.
> A few questions about the problem before I begin:
>
> 1–3. [Group B questions only]

Do not proceed to Phase 1 until answers are received.

---

### Building the User Profile

After this step, lock in the final User Profile and carry it into all subsequent phases:

```
Technical Level   : [Non-technical | Semi-technical | Technical | Expert]
Role / Domain     : [e.g., business owner, product manager, developer, designer]
Output Preference : [Plain language + analogies | Mix | Full technical detail]
Source            : [context window | user answered | inferred + confirmed]
```

This profile is **passed to every subagent** in Phase 2 and governs the language, depth,
and framing of all solution documents and the final recommendation.

---

## Phase 1 — Generate 3 Diverse Solution Candidates

After receiving clarification, generate exactly **3 solution candidates**.

**Rules for solution candidates:**
- Each candidate must represent a **fundamentally different approach** — not just variations of
  the same idea. Think of them as different strategic directions, not stylistic tweaks.
- Solutions should span a spectrum: e.g., minimal/safe vs. ambitious/risky, quick vs. thorough,
  centralized vs. distributed, etc.
- Label them clearly: `Solution A`, `Solution B`, `Solution C`
- For each candidate, write a **2–3 sentence summary** only at this stage

Present all 3 summaries to the user before proceeding to Phase 2.

---

## Phase 2 — Deep Development via Subagents

For each solution candidate, spawn an **independent subagent** (or process it in a new isolated
context) to develop the full detail document. This isolation is intentional — it prevents the
primary context window from biasing each solution's development.

Each subagent receives only:
1. The original user request (with clarification answers)
2. The confirmed User Profile from Phase 0
3. The solution summary for its assigned candidate
4. The document template (see below)

**Subagent instruction template:**
```
You are developing Solution [A/B/C] for the following user request.

USER REQUEST: [original request + clarification answers]

USER PROFILE:
  Technical Level   : [Non-technical | Semi-technical | Technical | Expert]
  Role / Domain     : [from Phase 0 answers]
  Output Preference : [Plain language + analogies | Mix | Full technical detail]

YOUR ASSIGNED SOLUTION SUMMARY: [2-3 sentence summary]

WRITING INSTRUCTIONS — adapt all language, depth, and framing to the User Profile:

  Non-technical:
    - Avoid jargon entirely. If a technical term is unavoidable, explain it in plain words.
    - Use real-world analogies to explain abstract concepts.
    - Lead with "what this means for you" before explaining "how it works".
    - The Implementation section should read like a step-by-step guide, not a spec.

  Semi-technical:
    - Use common technical terms, but briefly explain domain-specific ones.
    - Balance conceptual clarity with practical steps.
    - Include enough detail that they could hand this off to a developer.

  Technical / Expert:
    - Full technical precision expected. Use correct terminology without explanation.
    - Focus on nuance, edge cases, trade-offs, and architectural rationale.
    - Skip basics; go straight to what makes this solution distinct.

Develop this solution fully and honestly — including its weaknesses and risks.
Do not compare it to other solutions. Do not recommend it over others. Just develop it completely.

Follow the document template below and save the result to:
  .tot/<issue_snake_case>/solution_[a/b/c].md

[paste document template]
```

---

## Phase 3 — Solution Documents

Each solution must be saved as a Markdown file at:
```
.tot/<issue_in_snake_case>/solution_a.md
.tot/<issue_in_snake_case>/solution_b.md
.tot/<issue_in_snake_case>/solution_c.md
```

The `<issue_in_snake_case>` folder name is derived from the user's original request.
Example: "Refactor authentication system" → `.tot/refactor_authentication_system/`

### Document Template

Each solution document MUST follow this exact structure:

```markdown
# Problem

[Restate the user's request in your own words. Include relevant context and constraints
surfaced during Phase 0. Be precise about what "solved" means here.]

# User Profile

| Attribute        | Value |
|------------------|-------|
| Technical Level  | [Non-technical / Semi-technical / Technical / Expert] |
| Role / Domain    | [from Phase 0 answers] |
| Output Style     | [Plain language / Mix / Full technical detail] |

> Note: All language, depth, and framing in this document are adapted to the profile above.

# Assumptions

[List every assumption this solution makes — about the user's environment, skill level,
available resources, existing system behavior, etc. Unstated assumptions are where
solutions fail.]

# Solution Summary

[2–3 sentence overview of this specific approach and the core mechanism it uses.]

# Detailed Implementation

[Step-by-step description of how to execute this solution. Be specific. Include:
- What needs to be built, changed, or removed
- Tools, libraries, patterns, or techniques involved
- Order of operations
- Any code, config, or command examples that illustrate the approach]

# Trade-offs

| Dimension        | Assessment |
|------------------|------------|
| Complexity       | Low / Medium / High |
| Time to implement| Estimate   |
| Reversibility    | Easy / Hard / Irreversible |
| Risk level       | Low / Medium / High |
| Scalability      | How well it handles growth |
| Maintainability  | Long-term burden |

# Consequences

## Positive Outcomes
[What gets better if this solution succeeds?]

## Risks & Failure Modes
[What could go wrong? What are the known weaknesses of this approach?]

## Second-Order Effects
[What downstream effects — intended or unintended — does this solution create?
Think: what new problems might this solution introduce 3–6 months later?]

# Verdict
[One sentence: "This solution is best for users who prioritize [X] over [Y]."]
```

---

## Phase 4 — Recommendation

After all 3 solution documents are complete, the primary agent produces a final recommendation.

The recommendation must:
1. **Name the recommended solution** clearly
2. **Justify the recommendation** based on the user's stated constraints AND their profile
   (technical level, role, output preference from Phase 0) — not just general best practices
3. **Acknowledge the runner-up** — when would the second-best option be preferable?
4. **Highlight the key trade-off** the user is accepting with the recommended choice
5. **Adapt the language** of the recommendation to match the User Profile — a non-technical
   user should receive a recommendation written in plain language, an expert in precise terms

**Recommendation format:**
```markdown
## Recommendation

Based on your priorities ([restate constraints from Phase 0]), **Solution [X] is the most suitable**.

[2–3 paragraph justification.]

**Runner-up:** Solution [Y] would be preferable if [specific condition].

**The core trade-off you're accepting:** [one clear sentence].
```

---

## Hard Rules (Never Violate)

1. **Do not skip Phase 0.** Always scan the context window first (Step 0.1), then ask only
   what isn't already known. Group B is never skippable. Group A is skipped only when all
   3 profile attributes are already KNOWN from context.
2. **Do not reuse solution ideas.** All 3 candidates must be genuinely distinct approaches.
3. **Do not modify files outside `.tot/`.** During this reasoning process, no other files should
   be created, edited, or deleted.
4. **Do not collapse to one solution early.** Maintain three independent paths until Phase 4.
5. **Subagent isolation is required** when available. If subagents are not available, process
   each solution in a fresh reasoning block, starting from scratch each time, to minimize
   cross-contamination from prior solutions.
6. **Consequences are mandatory.** A solution without a consequence analysis is incomplete.
7. **Do not recommend without evidence.** The Phase 4 recommendation must trace back to the
   user's stated priorities AND their User Profile from Phase 0.
8. **User Profile must propagate.** Every subagent and the Phase 4 recommendation must receive
   and actively apply the User Profile. A solution written in expert-level detail for a
   non-technical user is a failed solution, regardless of its technical correctness.

---

## Directory Convention

```
.tot/
└── <issue_snake_case>/
    ├── solution_a.md
    ├── solution_b.md
    └── solution_c.md
```

The `.tot/` directory is the single source of truth for all reasoning artifacts from this skill.
It should be committed to version control if the project uses one, so the reasoning trail is preserved.

---

## Quick Reference — Phase Checklist

| Phase | Action | Gate |
|-------|--------|------|
| 0 — Step 0.1 | Scan context window; determine KNOWN / PARTIAL / UNKNOWN for each profile attribute | Must complete before composing any questions |
| 0 — Step 0.2 | Ask only UNKNOWN/PARTIAL profile attributes (Group A) + ≥3 problem questions (Group B) in one message | Skip Group A entirely if all attributes are KNOWN |
| — | Lock in User Profile with source annotation | Carry into all subsequent phases |
| 1 | Generate 3 distinct solution summaries | All 3 must be meaningfully different |
| 2 | Spawn subagents per solution (with User Profile + solution summary) | Each subagent gets isolated context |
| 3 | Write solution docs to `.tot/` (with User Profile section) | All docs must use the full template |
| 4 | Deliver recommendation in user's language, referencing Phase 0 profile + constraints | Must be adapted to User Profile |
