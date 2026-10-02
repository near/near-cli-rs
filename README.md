# PR #688 authentic terminal screenshots

Improved captures: actual interactive Zsh using the existing Starship prompt, installed JetBrainsMono Nerd Font Mono, and the user's Ghostty Catppuccin Mocha palette. These are real terminal-control PTY states, not Ghostty GUI screenshots. No prompts or CLI responses were fabricated. Each PNG is byte-identical to replaying its raw recording marker.

Source: 7ce8def46fe576854ba76ccfeb4740f93687f9d7; locally built near-cli-rs 0.30.1.
All calls are read-only wrap.testnet ft_metadata calls on public testnet. The terminal visually combines stdout and stderr; separate byte evidence confirms metadata on stderr, raw quiet stdout, and empty quiet stderr. Zsh's percent marker denotes a missing trailing newline and is not CLI output.

The original mock/plain captures are retained alongside the improved captures for provenance.
