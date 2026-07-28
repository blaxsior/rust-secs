Never
Never change a file's encoding, BOM, or line endings unless explicitly requested.
Never rewrite an entire file for a small change; prefer minimal patches.
Never use editing methods that may corrupt UTF-8 or non-ASCII text.
Never leave corrupted text (e.g. �, broken Korean characters) in modified files. Restore the original file if corruption is detected.
Never modify code, create files, or apply patches when the user is only asking for opinions, analysis, reviews, or design discussions.
Never assume discussion implies approval to implement.
Never perform unrelated refactoring or formatting while making a requested change.
Always
Preserve the original file encoding and formatting.
Verify that non-ASCII text remains intact after edits.
Discuss and propose a plan before implementation.
Wait for explicit implementation requests such as "implement", "apply", "modify", or "create" before changing files.
Keep changes as small and focused as possible.