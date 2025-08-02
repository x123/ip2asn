---
description: "propose a commit for the git-changes"
---

Propose a commit message for the @git-changes

- Do not use markdown formatting, just plaintext
- The lines *MUST* be 72 characters in length maximum

Here is an example of a good commit message:

```
refactor(cli): Remove 'AS' prefix from human-readable output

Removes the "AS" prefix from the ASN in the default human-readable
output format to match the requested format.

The output now is:
`15169 | 8.8.8.8 | 8.8.8.0/24 | GOOGLE | US`

- Updates the `println!` macro in `perform_lookup`.
- Updates all relevant integration tests to assert the new format.
```
