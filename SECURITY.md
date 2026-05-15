# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in Awake, please **do not open a public GitHub issue**. Instead, report it privately using [GitHub's private vulnerability reporting](../../security/advisories/new) for this repository (Security tab → "Report a vulnerability").

Please include:

- A description of the vulnerability and its potential impact.
- Steps to reproduce it (a minimal repro is ideal).
- The affected platform(s) and Awake version.

We aim to acknowledge reports within 5 business days and to keep you updated as we investigate and address the issue. Please give us reasonable time to release a fix before any public disclosure.

## Scope

Awake is a local desktop application with no backend, no network services, and no user accounts. Realistic vulnerability classes include:

- Privilege escalation via the native power-management, input-simulation, or window-visibility integrations.
- Local configuration-file handling issues (e.g. path traversal, unsafe deserialization).
- Ways activity automation could be triggered or configured to do something the user didn't intend or consent to.

Out of scope: the third-party dependencies' own CVEs (report those upstream), and issues that require physical/root access to the machine already.

## Supported Versions

Only the latest released version is supported with security fixes.

## Our Commitments

- We never bypass OS security mechanisms or request more permissions than a given feature requires.
- Activity automation is always opt-in and always overridable by real user input.
- No user data ever leaves the local machine (see [docs/privacy.md](docs/privacy.md)).
