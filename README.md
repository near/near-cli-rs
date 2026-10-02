# PR #688 terminal screenshots

Recorded on a Mac Mini from locally built near-cli-rs 0.30.1 at source 7ce8def46fe576854ba76ccfeb4740f93687f9d7.
Images are actual terminal-control PTY states; each matches its raw recording marker byte for byte.
01–04 use a loopback mock RPC. 05–06 are read-only wrap.testnet ft_metadata calls on public testnet.
The terminal combines stdout and stderr; independent capture verifies metadata on stderr and exact quiet output with empty stderr.
The quiet screenshot's following newline and exit label are emitted by the demo script.
