---
description: Retrieve PR review comments, apply fixes, and push changes
---

# PR Review Handler

1. **Fetch Review Comments**
   - Use `mcp_pull_request_read` with `method="get_review_comments"` to get all comments for the current PR.

2. **Analyze and Fix**
   - For each unresolved comment:
     - Locate the file and line.
     - Apply fixes to the code.
     - Verify with `cargo check` and `cargo test`.

3. **Push Changes**
   - Validate strict adherence to coding standards.
   - `git add .`
   - `git commit -m "refactor: Address review comments"`
   - `git push`

4. **Respond**
   - Add a comment to the PR summarizing the changes.
