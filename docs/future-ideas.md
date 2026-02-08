# Future Ideas (v2+)

Ideas the team loves but that are **out of scope for MVP**. We capture them here so
they don't get lost -- and so we stop talking about them during the current iteration.

The answer to "Wouldn't it be cool if..." is **"Yes -- in v2."**

---

## Feature Ideas

| Idea | Who Raised It | Why It's Interesting | Why Not Now |
|------|---------------|---------------------|-------------|
| Multiple lists / categories | -- | Users may want to organize by context | MVP validates single-list usage first |
| Due dates and reminders | -- | Time-sensitive tasks need deadlines | Adds calendar complexity; validate core loop first |
| Sharing / collaboration | -- | Shared grocery lists, family tasks | Multi-user interaction is a different product |
| Tags or labels | -- | Filtering and organization | Validate that a flat list is sufficient first |
| Search and filter | -- | Finding items in long lists | MVP lists are expected to be short |
| Mobile app (native) | -- | Better mobile experience | Responsive web is sufficient for MVP |
| Social login (OAuth) | -- | Easier signup flow | Email/password is simpler to implement and test |
| Dark mode | -- | User preference, reduced eye strain | Can be added later with CSS custom property reassignment |
| Reorder via drag-and-drop | -- | Nice UX for prioritization | "Could have" in MVP; defer if it delays launch |
| Undo delete (toast-style) | Steve Krug | Better UX than confirm dialogs; "undo is better than confirm" | Requires temporary state management; inline confirm suffices for MVP |
| Undo/redo (general) | -- | Safety net for accidental actions | Adds state complexity; confirm dialogs suffice for MVP |
| Password reset (forgot password) | Steve Krug | Users expect it on every login page | Requires email infrastructure (SMTP, tokens, expiry); placeholder link for MVP |
| Recurring tasks | -- | Daily/weekly habits | Different mental model; validate one-off tasks first |
| Subtasks / checklists | -- | Breaking down complex tasks | Adds hierarchy; flat list validates the core need |

---

## How to Add Ideas

When someone on the team (or a user) suggests a feature beyond MVP scope:

1. Add it to the table above
2. Note who raised it and why it's interesting
3. Explain briefly why it's deferred
4. Move on -- don't design it, don't discuss implementation details
