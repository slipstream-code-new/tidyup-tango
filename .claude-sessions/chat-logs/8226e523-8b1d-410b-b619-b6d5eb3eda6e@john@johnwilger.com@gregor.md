# Chat Log: 8226e523-8b1d-410b-b619-b6d5eb3eda6e

| Field | Value |
|-------|-------|
| **Session ID** | `8226e523-8b1d-410b-b619-b6d5eb3eda6e` |
| **Date Range** | 2026-02-05 23:46 UTC -- 2026-02-05 23:46 UTC |
| **Git Branch** | `main` |

---

## 2026-02-05 23:46 UTC -- User

Base directory for this skill: /home/jwilger/.claude/plugins/cache/jwilger-claude-plugins/sdlc/13.2.0/skills/start

# Start Skill

**Version:** 1.1.0
**Portability:** Tool-specific

---

## Quick Start

Get started in under 1 minute.

### What This Does
Analyzes project state and suggests next appropriate action.

### Fastest Path
```bash
/sdlc:start
# Detects current phase and suggests:
# - No config? → /sdlc:setup
# - No domain? → /sdlc:design discover
# - No tasks? → /sdlc:plan
# - Ready to code? → /sdlc:work
# - On feature branch? → /sdlc:pr or /sdlc:review
```

---

## Common Examples

### Example 1: First Time
**Invoke:** `/sdlc:start`
**Detects:** No config
**Suggests:** `/sdlc:setup`

### Example 2: After Setup
**Invoke:** `/sdlc:start`
**Detects:** No event model
**Suggests:** `/sdlc:design discover`

### Example 3: Ready to Code
**Invoke:** `/sdlc:start`
**Detects:** Tasks exist
**Suggests:** `/sdlc:work`

---

## When to Use

**Use when:**
- Unsure what to do next
- Starting new session
- User asks "what should I do?"

**Related:** All other skills (routes to them)

---

## Auto-Invocation

Claude automatically invokes this skill when you say:
- "Let's start working"
- "What should I do next?"
- "I'm ready to begin"
- "Where do I start?"
- "Let's get started"

You don't need to type `/sdlc:start` explicitly - Claude will detect these requests and invoke the skill for you.

---

## Metadata

**Version:** 1.1.0 (2026-02-05): Progressive disclosure
**Dependencies:** event-modeling, orchestration-protocol

---

## 2026-02-05 23:46 UTC -- Assistant

I'll analyze your project state and suggest the next appropriate action.

---

## 2026-02-05 23:46 UTC -- Assistant

<details>
<summary>Tool usage (1 call)</summary>

- Read: /home/jwilger/projects/todo_list/.claude/sdlc.yaml

</details>

---

## 2026-02-05 23:46 UTC -- Assistant

<details>
<summary>Tool usage (1 call)</summary>

- Read: /home/jwilger/projects/todo_list/AGENTS.md

</details>

---

## 2026-02-05 23:46 UTC -- Assistant

<details>
<summary>Tool usage (1 call)</summary>

- Glob: `**/.event-model/**/*`

</details>

---

## 2026-02-05 23:46 UTC -- Assistant

<details>
<summary>Tool usage (1 call)</summary>

- Bash: Get current git branch

</details>

---

## 2026-02-05 23:46 UTC -- Assistant

## Project State Analysis

✅ **SDLC Configuration:** Complete (event-modeling mode, TDD enforced)  
✅ **Git Repository:** Configured with GitHub integration  
❌ **Event Model:** Not found (no `.event-model/` directory)  
✅ **Git Branch:** On `main` (clean working tree except for modified `.claude/sdlc.yaml`)

---

## 🎯 Next Step: Domain Discovery

Your project is configured but needs a domain model. You should start with **domain discovery** to understand the problem space before writing code.

### Recommended Action

Run the domain discovery workflow:

```bash
/sdlc:design discover
```

This will:
1. Interview you about the problem domain
2. Identify key workflows and use cases
3. Create an event model foundation
4. Generate user stories (Given-When-Then scenarios)
5. Prepare implementation tasks

---

## Alternative: Skip to Planning

If you already have a clear understanding of your domain and want to jump directly to creating implementation tasks, you can run:

```bash
/sdlc:plan
```

However, **domain discovery is recommended** for new projects to ensure you're building the right thing.

---

**What would you like to do?**

---

## 2026-02-05 23:46 UTC -- User

discovery

---

## 2026-02-05 23:46 UTC -- Assistant

<details>
<summary>Tool usage (1 call)</summary>

- Skill

</details>

---

## 2026-02-05 23:46 UTC -- User

[Request interrupted by user]

---
