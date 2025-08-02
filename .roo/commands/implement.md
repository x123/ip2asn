---
description: "Implement a feature"
---

Review docs/plan.md
Implement *just* $ARGUMENTS , and do it step a a time and incrementally, running `just all` along the way to make sure things are working. review your work, propose a commit message, and wait for review.

Consider that the following questions will be asked upon review, so make sure they are sufficiently answered *before* you submit the work for review.

- Have we achieved everything we set out to in the Phase/Chunk? Are we missing anything to move on to the next Chunk?
- Are all the linter and tests passing (i.e., does `just all` work without error?)
- Do we need to add, remove, or update any other tests?
- Are they changes DRY, idiomatic, YAGNI, robust, and sustainable?
- Have you updated any documentation that might be relevant? (README.md, docs/spec.md, docs/plan.md, etc.)
- If any changes were made to settings, have you updated the default JSON configs, alerters.d configs, and/or helm charts?
- If we made a *major* architectural change or overcame significant hurdles, append a journal.md entry following the idioms and format of that file.
