Always
- Discuss a plan before implementation and wait for explicit requests such as "implement", "apply", "modify", or "create".
- Keep changes minimal, focused, and unrelated refactoring-free.
- Preserve file encoding, BOM, line endings, formatting, and non-ASCII text.
- Prefer Rust `filename.rs` modules over directory `mod.rs` modules.

Never
- Modify code, create files, or apply patches for design discussion, analysis, opinions, or reviews only.
- Assume discussion implies approval to implement.
- Use editing methods that may corrupt UTF-8 or non-ASCII text.
- Leave corrupted text in modified files; restore the original file if corruption is detected.
- Introduce Rust `mod.rs` module files unless explicitly requested or required by the existing structure.
