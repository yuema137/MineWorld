# PR Design Doc Requirements

audit relevant code base to give a detailed plan. For each commit, you need to list the plan, the validation or test needed after implementation. Use [] to mark not finished item and use [x] to mark the finished item.

For each planned commit, provide a detailed implementation checklist.

For every commit, include:

1. **Goal**  
   - What exact problem this commit solves.  
   - Why it belongs in this commit rather than another one.

2. **Scope**  
   - Files, classes, functions, schemas, CLI arguments, and documentation expected to change.  
   - Important non-goals and behaviors that must remain unchanged.  
   - Dependencies on earlier commits.

3. **Implementation plan**  
   - List each concrete implementation step using `[ ]`.  
   - Use `[x]` only after the change is implemented and verified with recorded evidence.  
   - Keep the steps specific enough to track progress, but do not invent low-level implementation details before inspecting the relevant code.

4. **Validation plan**  
   - Unit tests required.  
   - Integration or pseudo tests required.  
   - Negative and invalid-input tests required.  
   - Backward-compatibility or default-parity tests required.  
   - Real-training Gate tests, if required, must be listed separately. Run them autonomously when authorized by the project-specific implementation contract and within its bounded validation budget; request operator approval before exceeding that authorization or budget.

5. **Acceptance criteria**  
   - State the exact observable conditions that must be true before the commit can be considered complete.  
   - Do not use vague criteria such as "works correctly" or "tests pass."  
   - For ordering behavior, validate the actual visited sample/file sequence, not only the configuration value.  
   - For the default `shuffle` path, prove that selection, random-seed behavior, visited sequence, and step count remain unchanged.

6. **Failure and edge cases**  
   - List the important failure cases this commit must handle.  
   - Include invalid `file_order`, missing files, duplicate files, scope mismatches, legacy configuration, resume behavior, and propagation failures where relevant.  
   - Explain whether each failure should stop execution, fall back safely, or produce a warning.

7. **Verification commands and evidence**  
   - List the intended test commands.  
   - Record test counts and wall time after execution.  
   - Record any test that could not be run and explain why; never claim it passed.  
   - Update the design document immediately after each implementation or test checkpoint.

8. **Commit boundary**  
   - Confirm that the commit is independently reviewable.  
   - Confirm that it does not include unrelated cleanup or future follow-up work.  
   - Before committing, inspect and record the exact diff summary, staged file list, tests, and any deviations from the approved design; update the live design document, then commit and continue autonomously without waiting for operator approval.

Additional rules:

- Inspect the relevant code before finalizing each commit plan. Do not guess file paths, interfaces, or behavior.  
- If code inspection reveals ambiguity, audit the relevant source and evidence first. Resolve bounded implementation details autonomously and record the decision; stop for operator input before changing frozen invariants or materially expanding the approved scope.  
- Planner exposure and production-default changes are outside the implementation commits and require separate evidence and operator approval.  
- Do not overdesign the future empirical comparison campaign now; include only the initial validation requirements needed to implement the feature safely.
