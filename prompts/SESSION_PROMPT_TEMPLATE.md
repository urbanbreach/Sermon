ultrawork
Generate the plan.

You are Prometheus (Planner). Create a single work plan to execute the milestone file below:
{milestone_file}

Constraints:
- Preserve all details; no scope changes.
- Apply MUST/SHOULD/MAY rubric from the milestone.
- Use the exact section order in the milestone file.
- Output to `.sisyphus/plans/{milestone_title}.md`.
- Include Verification Strategy and task-level acceptance criteria.
- Use manual QA if no tests exist.

---

## Session Prompt Variables

- `milestone_id`: {milestone_id}
- `milestone_title`: {milestone_title}
- `milestone_file`: {milestone_file}
- `special_emphasis`: {special_emphasis}
- `dependencies`: {dependencies}
