Software Requirements Specification (SRS) for pbcat

1. Introduction

1.1 Purpose

pbcat is a command-line utility that recursively collects the contents of one or more files and copies the merged output to the system clipboard. It is designed for fast, minimal-friction workflows such as preparing code snippets for LLMs. File selection is performed entirely by the user and the shell (via explicit paths or glob expansion). pbcat focuses exclusively on file processing, filtering, and clipboard integration.

1.2 Scope

pbcat operates on:
•	Explicit file paths
•	Directories (recursively)
•	Shell-expanded glob patterns (e.g., *.swift, **/*.json)

The tool:
•	Concatenates file contents in the order arguments are provided
•	Optionally groups directory contents in deterministic order
•	Copies merged output to clipboard using OS-native utilities
•	Supports macOS and Linux (X11 and Wayland)

1.3 Definitions
•	Clipboard: The system pasteboard used for copy/paste
•	Gitignore rules: Patterns defined in .gitignore files
•	Shell expansion: Filename/glob expansion performed before execution

⸻

2. Overall Description

2.1 Product Perspective

pbcat behaves similarly to cat, but instead of printing to stdout, it copies the aggregated content to the system clipboard. It is a standalone tool that does not require external services.

2.2 Product Functions
•	Accept variable number of file or directory arguments
•	Recursively process directories
•	Resolve duplicates and ignore non-files
•	Apply ignore patterns
•	Concatenate contents with optional separators
•	Copy final result to clipboard

2.3 User Classes
•	Software developers
•	LLM-intensive users
•	Automation and scripting users

2.4 Operating Environment
•	macOS (pbcopy)
•	Linux Wayland (wl-copy)
•	Linux X11 (xclip/xsel)
•	Filesystem with POSIX-compatible semantics

⸻

3. Functional Requirements

FR-1: Input Handling
•	FR-1.1: Accept one or more command-line arguments
•	FR-1.2: Each argument may be a file or directory
•	FR-1.3: If an argument does not exist, report an error

FR-2: Directory Expansion
•	FR-2.1: Recursively walk through directories
•	FR-2.2: Apply .gitignore if enabled
•	FR-2.3: Skip known build/artifact directories (e.g., Xcode project internals, DerivedData)

FR-3: File Processing
•	FR-3.1: Read file contents as UTF-8
•	FR-3.2: Optionally insert separators before each file
•	FR-3.3: Preserve file ordering when multiple inputs overlap

FR-4: Clipboard Integration
•	FR-4.1: On macOS use pbcopy
•	FR-4.2: On Linux Wayland use wl-copy
•	FR-4.3: On Linux X11 use xclip or xsel
•	FR-4.4: If no clipboard utility is available, display guidance

FR-5: Output Summary
•	FR-5.1: Print number of files processed
•	FR-5.2: Print total bytes copied
•	FR-5.3: Do not print file contents to stdout unless requested

⸻

4. Non-Functional Requirements

NFR-1: Performance
•	Must handle thousands of files with low latency
•	Must load and merge files efficiently

NFR-2: Usability
•	CLI interface should be minimal and intuitive
•	Errors must be concise and informative

NFR-3: Portability
•	Single-file implementation preferred
•	Avoid external dependencies other than system clipboard utilities

NFR-4: Reliability
•	Must gracefully handle permission errors, unreadable files
•	Must not partially copy data on error

⸻

5. Constraints
   •	Shell handles glob expansion
   •	Clipboard access depends on platform utilities
   •	Tool does not attempt to interpret file syntax

⸻

6. Usage Examples

6.1 Single File

pbcat PeepApp.swift

Processes exactly one file.

6.2 Multiple Explicit Files

pbcat PeepApp.swift AlarmViewModel.swift ConfigurationView.swift

Order is preserved.

6.3 Shell Glob Expansion

pbcat *.swift
pbcat **/*.swift
pbcat Views/*.swift

Shell expands patterns before pbcat runs.

6.4 Directory Input

pbcat Views/
pbcat Peep/

Recursively collects all files in the directory.

6.5 Mixed Input

pbcat *.swift Views/ Specific.swift

Combines files from globs, directories, and explicit paths.

7. Future Enhancements (Not in Scope)
   •	Comment stripping
   •	Max-size per file
   •	Syntax highlighting
   •	Compression
   •	JSON/Markdown packaging

⸻
