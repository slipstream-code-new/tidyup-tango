# Future Ideas (Post-V1 GTD)

Ideas the team loves but that are **out of scope for V1 GTD**. We capture them here so
they don't get lost -- and so we stop talking about them during the current iteration.

The answer to "Wouldn't it be cool if..." is **"Yes -- after V1."**

Note: Several items from the original MVP future list are now in scope for V1 GTD
(multiple lists/contexts, tags/labels as contexts). Items marked "NOW IN SCOPE" have
been moved to the V1 feature list.

---

## Feature Ideas

| Idea | Who Raised It | Why It's Interesting | Why Not Now |
|------|---------------|---------------------|-------------|
| ~~Multiple lists / categories~~ | -- | NOW IN SCOPE (V1 GTD: Inbox, Next Actions, Projects, Waiting For, Someday/Maybe) | -- |
| ~~Tags or labels~~ | -- | NOW IN SCOPE (V1 GTD: Contexts on next actions) | -- |
| Calendar / date-specific items | -- | GTD calendar for date-bound commitments | Requires date picker, time zones, recurring events |
| Due dates and reminders | -- | Time-sensitive tasks need deadlines | Calendar integration complexity |
| Sharing / collaboration | -- | Shared projects, delegation to real users | Multi-user interaction is a different product |
| Search and filter | -- | Finding items across GTD lists | Lists should be manageable at V1 scale |
| Mobile app (native) | -- | Better mobile experience | Responsive web is sufficient |
| Social login (OAuth) | -- | Easier signup flow | Email/password is simpler to implement and test |
| Dark mode | -- | User preference, reduced eye strain | CSS custom property swap; not a GTD feature |
| Reorder via drag-and-drop | -- | Nice UX for prioritization within lists | Not core to GTD workflow |
| Undo delete (toast-style) | Steve Krug | Better UX than confirm dialogs | Requires temporary state management |
| Undo/redo (general) | -- | Safety net for accidental actions | Adds state complexity |
| Password reset (forgot password) | Steve Krug | Users expect it on every login page | Requires email infrastructure (SMTP, tokens, expiry) |
| Recurring items | -- | Daily/weekly habits, recurring next actions | Adds scheduling complexity |
| Reference file storage | -- | Store non-actionable reference material | Needs file upload, search, storage |
| Horizons of Focus | -- | Goals, vision, purpose layers above projects | Higher-level planning; validate core workflow first |
| Natural language input | -- | "call Bob tomorrow" parsed automatically | NLP parsing complexity |
| Email-to-inbox capture | -- | Forward emails to inbox automatically | Requires email receiving infrastructure |
| Bulk operations | -- | Select multiple items, batch process | UI complexity; process one at a time first |
| Subtasks / checklists | -- | Breaking down actions into steps | GTD says actions should be atomic; projects handle multi-step |

---

## How to Add Ideas

When someone on the team (or a user) suggests a feature beyond MVP scope:

1. Add it to the table above
2. Note who raised it and why it's interesting
3. Explain briefly why it's deferred
4. Move on -- don't design it, don't discuss implementation details
