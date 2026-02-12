# GTD Product Discovery

*Prepared by: Marty Cagan (Product Manager)*
*Status: APPROVED -- team consensus reached (9/9)*

---

## 1. The User Problem

Our simple todo list MVP validated that users want a lightweight, accessible tool for
capturing and tracking tasks. But a flat list with pending/completed states does not
help users decide **what to do next**. As the list grows, users face decision fatigue:
everything looks equally important, nothing has context, and there is no system for
handling multi-step outcomes, delegated work, or future ideas.

**The core problem**: Users need a trusted system that gets tasks out of their heads,
helps them decide what each item means, organizes items by context and actionability,
and surfaces the right work at the right time -- so they can engage with confidence
instead of anxiety.

This is the problem David Allen's Getting Things Done (GTD) methodology solves.

---

## 2. GTD Methodology Summary

GTD is a five-stage personal productivity workflow:

### Stage 1: Capture (Inbox)
Collect everything that has your attention into a single trusted inbox. The barrier to
entry must be as low as possible. Capture now, decide later.

### Stage 2: Clarify (Process)
Process each inbox item with a decision tree:
- **Is it actionable?**
  - **No** -> Trash it, file as Reference, or move to Someday/Maybe
  - **Yes** -> What is the next physical action?
    - Can it be done in **under 2 minutes**? -> Do it now
    - Should someone else do it? -> **Delegate** (track on Waiting For list)
    - Is it a multi-step outcome? -> Create a **Project** and identify the next action
    - Otherwise -> Add to **Next Actions** list with a context

### Stage 3: Organize
Place clarified items into the right list:
- **Next Actions**: Concrete, physical actions organized by context (@home, @computer,
  @errands, @phone, @anywhere)
- **Projects**: Any outcome requiring more than one action step. Each project must have
  at least one next action.
- **Waiting For**: Items delegated to others or blocked by external events. Tracked with
  the date delegated and who/what you're waiting on.
- **Someday/Maybe**: Ideas, aspirations, and "wouldn't it be cool if..." items. Not
  committed to, but reviewed regularly.
- **Reference**: Non-actionable information you may need later.
- **Calendar**: Date-specific actions and appointments (things that MUST happen on a
  specific day/time -- not aspirational deadlines).

### Stage 4: Reflect (Review)
Regular reviews keep the system trustworthy:
- **Daily**: Process inbox to zero. Check calendar and next actions.
- **Weekly Review** (the critical success factor):
  1. Get Clear: Process all inboxes to zero
  2. Get Current: Review all active projects and next actions, mark completions, add
     missing next actions
  3. Get Creative: Review someday/maybe, check upcoming calendar, trigger new ideas

### Stage 5: Engage (Do)
Choose what to work on based on four criteria:
1. **Context**: Where are you? What tools do you have?
2. **Time available**: How much time before your next commitment?
3. **Energy available**: How much mental/physical energy do you have?
4. **Priority**: Given the above constraints, what is most important?

---

## 3. Domain Concepts Map

### Core Entities

| GTD Concept | Description | Relationship to Current System |
|-------------|-------------|-------------------------------|
| **Inbox Item** | Raw, unclarified capture | Replaces current "pending" todo as the entry point |
| **Next Action** | A concrete, physical action ready to do | Evolved from current TodoItem::Pending |
| **Project** | An outcome requiring 2+ actions | New concept -- groups related next actions |
| **Context** | Where/how an action can be performed | New concept -- tags on next actions |
| **Waiting For** | Delegated/blocked item with tracking info | New concept |
| **Someday/Maybe** | Deferred idea, not committed to | New concept |
| **Reference** | Non-actionable information | Out of scope for v1 (notes/files system) |
| **Calendar Item** | Date/time-specific action | Out of scope for v1 (needs date picker, recurring) |

### State Machine (GTD Item Lifecycle)

```
                    capture()
  [nothing] ──────────────────> InboxItem
                                    │
                          clarify() │
                    ┌───────────────┼───────────────┐
                    │               │               │
                    v               v               v
               NextAction      SomedayMaybe      Trash
                    │               │            (deleted)
                    │               │
         complete() │    activate() │
                    │               │
                    v               v
                Completed      NextAction
                              (moves to active)

  Additionally:
  - NextAction can be moved to WaitingFor (delegate)
  - WaitingFor can be moved back to NextAction (received)
  - NextAction can be moved to SomedayMaybe (defer)
  - Any item can be linked to a Project
```

---

## 4. V1 Scope: Prioritized Feature List

### Guiding Principle
V1 delivers the core GTD loop: Capture -> Clarify -> Organize -> Review -> Engage.
We build the simplest version of each stage that enables the workflow. Advanced features
(calendar integration, natural language dates, mobile capture, file attachments) come
later.

### Must Have (V1 Core)

| # | Feature | User Problem Solved |
|---|---------|-------------------|
| 1 | **Inbox capture** | Quick capture of anything on your mind with minimal friction |
| 2 | **Clarify workflow** | Process inbox items: actionable or not? Next action or project? |
| 3 | **Next Actions list with contexts** | See only actions you can do right now, based on where you are |
| 4 | **Projects list** | Track multi-step outcomes; each project has at least one next action |
| 5 | **Waiting For list** | Track delegated items so nothing falls through the cracks |
| 6 | **Someday/Maybe list** | Park ideas without cluttering your action lists |
| 7 | **Weekly Review guided flow** | Structured review process to keep the system trustworthy |
| 8 | **Complete and delete actions** | Mark work done; remove irrelevant items |

### Should Have (V1 if time permits)

| # | Feature | User Problem Solved |
|---|---------|-------------------|
| 9 | **Custom contexts** | Users define their own contexts beyond defaults |
| 10 | **Project support materials** (notes) | Attach notes/plans to a project |
| 11 | **Inbox count badge** | Visual reminder of unprocessed items |
| 12 | **Move between lists** | Reclassify items as understanding changes |

### Could Have (V1 stretch goals)

| # | Feature | User Problem Solved |
|---|---------|-------------------|
| 13 | **Two-minute rule prompt** | During clarify, prompt user if action takes < 2 min |
| 14 | **Review scheduling** | Remind users when weekly review is due |
| 15 | **Bulk inbox processing** | Process multiple items in a flow |

### Out of Scope (Future)

| Feature | Why Not Now |
|---------|------------|
| Calendar / date-specific items | Requires date picker, time zones, recurring events |
| Reference file storage | Needs file upload, search, storage infrastructure |
| Horizons of Focus (goals, vision) | Higher-level planning; validate core workflow first |
| Natural language input ("call Bob tomorrow") | NLP parsing complexity |
| Email-to-inbox capture | Requires email receiving infrastructure |
| Mobile native app | Responsive web is sufficient |
| Sharing / delegation to other users | Multi-user collaboration is a different product |
| Tags beyond contexts | Contexts are sufficient for v1 organization |
| Search and filter | Lists should be manageable in size at v1 scale |
| Dark mode | CSS custom property swap; not a GTD feature |
| Drag-and-drop reordering | Nice UX but not core to GTD workflow |
| Recurring items | Adds scheduling complexity |

---

## 5. User Stories (V1 GTD)

### US-GTD-1: Capture to Inbox

**As a** GTD practitioner,
**I want to** quickly capture anything on my mind into my inbox,
**So that** I can get it out of my head and process it later.

**Acceptance Criteria:**
- Inbox capture input is always accessible (prominent on the main page)
- Capture is fast: type and press Enter, item appears in inbox
- No categorization required at capture time (that happens during clarify)
- Inbox items show in a list ordered by capture time (newest first)
- Inbox count is visible so users know how many items need processing
- Works without JavaScript; HTMX enhances

**Success Metric:** Capturing a thought takes under 3 seconds.

### US-GTD-2: Clarify Inbox Items

**As a** GTD practitioner,
**I want to** process each inbox item through the GTD clarify workflow,
**So that** I can decide what each item means and where it belongs.

**Acceptance Criteria:**
- User can process inbox items one at a time
- For each item, user can:
  - Mark as actionable -> specify next action text, assign context, optionally link to project
  - Mark as not actionable -> trash, move to someday/maybe, or (future) reference
  - If actionable and multi-step -> create a project and a first next action
  - If actionable and delegated -> move to Waiting For with a note about who/what
- After clarifying, item leaves the inbox and appears in its destination list
- Processing an item to Next Action requires a context selection
- User can skip an item and come back to it later
- Works without JavaScript; HTMX enhances

**Success Metric:** Processing a single inbox item takes under 15 seconds.

### US-GTD-3: View and Work from Next Actions

**As a** GTD practitioner,
**I want to** see my next actions filtered by context,
**So that** I can quickly find what I can do right now based on where I am.

**Acceptance Criteria:**
- Next Actions page shows all active next actions
- Actions can be filtered by context (e.g., show only @computer actions)
- Default view shows all contexts with context headings
- Each action shows its title, context, and associated project (if any)
- User can mark an action as complete
- User can delete an action
- User can edit an action's title
- Completed actions are removed from the active list (shown in project view if linked)
- Works without JavaScript; HTMX enhances

**Success Metric:** User can find and start working on a relevant action within 10 seconds.

### US-GTD-4: Manage Projects

**As a** GTD practitioner,
**I want to** see all my active projects and their next actions,
**So that** I can ensure every project is moving forward.

**Acceptance Criteria:**
- Projects list shows all active projects
- Each project shows its title and the count of next actions
- User can view a project to see all its linked next actions and completed actions
- Projects without a next action are flagged (stalled projects)
- User can add a project directly (without going through inbox)
- User can add a next action directly to a project
- User can complete a project (marks it as finished)
- User can delete a project
- Works without JavaScript; HTMX enhances

**Success Metric:** During weekly review, user can verify all projects have next actions
in under 2 minutes.

### US-GTD-5: Waiting For List

**As a** GTD practitioner,
**I want to** track items I'm waiting on from others,
**So that** I can follow up at the right time and nothing falls through the cracks.

**Acceptance Criteria:**
- Waiting For list shows all items the user is waiting on
- Each item shows: what is being waited on, who/what it's waiting on, date added
- User can add items directly to Waiting For
- User can mark a Waiting For item as received (moves to inbox or completes)
- User can convert a Waiting For item to a Next Action
- Works without JavaScript; HTMX enhances

**Success Metric:** User can see everything they're waiting on in one glance.

### US-GTD-6: Someday/Maybe List

**As a** GTD practitioner,
**I want to** park ideas and someday projects,
**So that** they don't clutter my action lists but aren't forgotten either.

**Acceptance Criteria:**
- Someday/Maybe list shows all parked ideas
- User can add items directly to Someday/Maybe
- User can activate an item (moves to inbox for clarification, or directly to Next Actions)
- User can delete items that are no longer interesting
- List is reviewed during the weekly review
- Works without JavaScript; HTMX enhances

**Success Metric:** Ideas have a home that is not the user's head or their active lists.

### US-GTD-7: Weekly Review

**As a** GTD practitioner,
**I want to** be guided through a weekly review process,
**So that** I can keep my system current and trustworthy.

**Acceptance Criteria:**
- Weekly Review is accessible from the main navigation
- Review follows three phases:
  1. **Get Clear**: Shows inbox count, prompts user to process to zero
  2. **Get Current**: Shows each active project; user confirms next action exists or adds one.
     Shows Waiting For items; user confirms still waiting or resolves. Shows Next Actions;
     user confirms still relevant or removes.
  3. **Get Creative**: Shows Someday/Maybe list for review. Prompts for any new ideas.
- Each phase can be completed independently
- Review progress is tracked (which phases are done)
- Works without JavaScript; HTMX enhances

**Success Metric:** A weekly review can be completed in under 30 minutes.

### US-GTD-8: Navigation and GTD Dashboard

**As a** GTD practitioner,
**I want to** navigate between Inbox, Next Actions, Projects, Waiting For, and
Someday/Maybe,
**So that** I can access any part of my GTD system quickly.

**Acceptance Criteria:**
- Main navigation includes: Inbox (with count), Next Actions, Projects, Waiting For,
  Someday/Maybe, Weekly Review
- Dashboard/home page shows a summary: inbox count, stalled projects count, next actions count
- Navigation is accessible (keyboard, screen reader) with current page indicated
- Works without JavaScript

**Success Metric:** Any list is reachable in one click from any page.

---

## 6. Success Criteria for V1

V1 of the GTD system is successful when:

1. A user can capture items to their inbox and process them to zero
2. Clarified items land in the correct list (Next Actions, Projects, Waiting For,
   Someday/Maybe, or trash)
3. Next Actions are filterable by context so users can work from context-appropriate lists
4. Every active project has at least one next action (the system flags stalled projects)
5. The Weekly Review guided flow helps users keep their system current
6. The entire system is accessible (WCAG 2.2 AA)
7. The system works without JavaScript (progressive enhancement)
8. A first-time GTD user can understand the system without external instructions
9. All existing auth functionality (register, login, logout) continues to work

**How we will know it is working:**
- A user can complete the full GTD loop (capture -> clarify -> organize -> review ->
  engage) without hesitation
- The inbox reaches zero during a review session
- The user trusts the system enough to stop keeping tasks in their head

---

## 7. UX Principles (Cross-Cutting)

These principles apply to every feature and every view. They are non-negotiable.

1. **Don't make me think about GTD**: The UI teaches the methodology through clear labels,
   helpful empty states, and gentle guidance. Users should not need to read the GTD book
   to use this app. Where GTD terms are used (Someday/Maybe, Waiting For), add subtitle
   copy that explains the concept in plain language.

2. **Progressive disclosure**: Show the simplest interface first. Reveal complexity only
   when the user needs it. The clarify form shows radio buttons; selecting a destination
   reveals only the fields for that destination. Navigation shows all lists; the user
   explores them as they build their system.

3. **Empty states are onboarding**: Every empty list explains its purpose and guides the
   user to the next action. "No next actions yet. Process your inbox to find things to
   do." Empty states are the first impression of each feature.

4. **Clarify must feel fast**: Processing inbox items is the hardest GTD habit to build.
   The clarify flow should feel like sorting mail -- a few clicks per item, minimal typing.
   If it feels like filling out a form, we've failed.

5. **Convention over configuration**: We provide the GTD structure with sensible defaults.
   5 default contexts, not a context builder. Structured review phases, not a customizable
   workflow. The app makes GTD decisions for the user where possible.

6. **Calm over urgent**: GTD's purpose is reducing anxiety. The design should feel calm,
   organized, and spacious. Inbox count badges are informational, not alarming. No red
   badges, no pulsing alerts, no "you have 47 unprocessed items!" warnings.

7. **The Trunk Test**: At any point in the app, the user knows: Where am I? What are my
   options? How do I get back? Clear navigation, descriptive page titles, obvious current
   page indicator.

---

## 8. Key Architectural Considerations

### Migration from Simple Todo to GTD

The existing `TodoItem` enum (Pending/Completed) must evolve. Key changes:

1. **New domain model**: The simple `TodoItem` becomes multiple types:
   - `InboxItem` -- raw capture, unclarified
   - `NextAction` -- actionable, with context and optional project link
   - `WaitingForItem` -- delegated, with who/what and date
   - `SomedayMaybeItem` -- parked idea
   - `Project` -- multi-step outcome with linked actions

2. **Database migration path**: We need to decide whether to:
   - Add new tables alongside the existing `todos` table and migrate data
   - Replace the `todos` table entirely with the new schema
   - Recommendation: Replace. The MVP has few users (if any in production). Clean break.

3. **Context as a domain concept**: Contexts are user-defined labels with a few defaults
   provided. They could be modeled as:
   - A separate `contexts` table with a foreign key from next actions
   - An enum with custom values
   - Recommendation: Separate table. Users need custom contexts.

4. **URL structure**: GTD lists suggest these routes:
   - `/inbox` -- Inbox capture and list
   - `/inbox/{id}/clarify` -- Process a single inbox item
   - `/next-actions` -- Next actions, filterable by context
   - `/projects` -- Projects list
   - `/projects/{id}` -- Single project view
   - `/waiting-for` -- Waiting For list
   - `/someday-maybe` -- Someday/Maybe list
   - `/review` -- Weekly Review flow

5. **Navigation**: The app needs a proper navigation structure. The current single-page
   todo list becomes a multi-page application with a persistent header navigation bar
   (team consensus: header nav, not sidebar -- simpler, more accessible, works with
   progressive enhancement).

6. **Shared types**: Several list types share behaviors (title, created_at, user ownership).
   The domain architect should consider whether to use traits, shared fields via structs,
   or keep them fully independent types.

### Data Model Sketch

```
users (existing)
  id: UUID PK
  email: TEXT
  password_hash: TEXT

contexts
  id: UUID PK
  user_id: UUID FK -> users
  name: TEXT
  position: INT (for ordering)

projects
  id: UUID PK
  user_id: UUID FK -> users
  title: TEXT
  status: TEXT (active, completed, dropped)
  created_at: TIMESTAMPTZ
  completed_at: TIMESTAMPTZ NULL

inbox_items
  id: UUID PK
  user_id: UUID FK -> users
  title: TEXT
  created_at: TIMESTAMPTZ

next_actions
  id: UUID PK
  user_id: UUID FK -> users
  project_id: UUID FK -> projects NULL
  context_id: UUID FK -> contexts
  title: TEXT
  created_at: TIMESTAMPTZ
  completed_at: TIMESTAMPTZ NULL

waiting_for_items
  id: UUID PK
  user_id: UUID FK -> users
  project_id: UUID FK -> projects NULL
  title: TEXT
  waiting_on: TEXT (who or what)
  created_at: TIMESTAMPTZ
  resolved_at: TIMESTAMPTZ NULL

someday_maybe_items
  id: UUID PK
  user_id: UUID FK -> users
  title: TEXT
  created_at: TIMESTAMPTZ
```

### Implementation Order (Walking Skeleton)

Recommended build order, following the same incremental approach as the MVP. The
clarify workflow (US-GTD-2) is split into sub-stories and interleaved with the lists
it routes to, so each clarify path ships immediately after its destination exists:

> **Current step**: 7 -- Clarify as Project

1. [x] **Navigation + Dashboard** -- Multi-page structure, header nav, route skeleton
2. [x] **Inbox Capture** -- The new entry point (replaces direct todo add)
3. [x] **Contexts** -- Domain type + DB + default contexts for new users
4. [x] **Next Actions with Contexts** -- Core working list
5. [x] **Clarify as Next Action + Trash** -- First clarify paths
6. [x] **Projects** -- Multi-step outcome tracking
7. [ ] **Clarify as Project** -- Inbox -> new project + first next action  <!-- NEXT -->
8. [ ] **Waiting For** -- Delegation tracking
9. [ ] **Clarify as Waiting For** -- Inbox -> waiting for
10. [ ] **Someday/Maybe** -- Idea parking
11. [ ] **Clarify as Someday/Maybe** -- Inbox -> someday/maybe
12. [ ] **Weekly Review** -- The glue that keeps it all trustworthy
13. [ ] **Data Migration + Cleanup** -- Migrate old todos, remove legacy code

Each step follows the Atomic Green Step pipeline. Each step delivers a usable increment.
When a step is completed (CI green + 9/9 consensus), the Driver marks it `[x]` and
moves the `<!-- NEXT -->` marker to the next unchecked step. The "Current step" summary
line at the top of this section is also updated.

---

## 9. Resolved Decisions (Team Consensus)

The following questions were discussed during team review and resolved by consensus:

1. **Default contexts**: @computer, @home, @errands, @phone, @anywhere (5 contexts).
   @office was dropped (Lea + Steve Schoger) -- too similar to @computer for many users.
   Users can add/edit/delete their own contexts.

2. **Completed items**: Option (a) -- completed next actions are removed from the active
   Next Actions list. They remain visible in the project detail view (for project-linked
   actions). This keeps working lists clean and focused.

3. **Existing todo data**: Pending todos migrate to Next Actions with a default context
   of @anywhere (Scott's recommendation -- preserves existing workflow rather than forcing
   users to re-clarify items they've already decided on). Completed todos migrate to
   completed next actions.

4. **Mobile-first**: Yes. Mobile-first layout, consistent with TEAM_AGREEMENTS.md.

5. **Quick capture**: Available on every authenticated page via a persistent input in the
   header. On the Inbox page, the header capture input and the page's own input are the
   same form (avoid duplicate inputs -- Lea's feedback).

6. **Navigation pattern**: Header navigation bar (not sidebar). Carson and Heydon agreed:
   a `<nav>` landmark in the header with links to each GTD view. Simpler, more accessible,
   works naturally with progressive enhancement. No JavaScript-dependent sidebar collapse.

7. **Clarify interaction pattern**: A single form on `/inbox/{id}/clarify` with radio
   buttons for destination choice. Selecting a destination reveals relevant fields via
   progressive disclosure. Standard HTML form, HTMX enhances. No wizard, no modals.
   (Carson's recommendation, Heydon approved.)

---

*This document has been reviewed and approved by all 9 team members.
Pending project owner approval for PROJECT.md scope changes.*
