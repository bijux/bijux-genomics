# PERFORMANCE_BUDGET

This document summarizes performance constraints enforced by tests.

- Report rendering should complete within the local unit test time budget.
- SQLite load queries should avoid N+1 patterns and use indexed lookups.
- Report JSON size should remain stable for fixture inputs (no unbounded growth).

If a test budget fails, adjust logic or explicitly document why the budget changes.
