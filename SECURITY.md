# Security Policy

## Supported Versions

The latest release receives security fixes. Older releases do not.

## Reporting a Vulnerability

Report a vulnerability privately through GitHub: open the repository's Security
tab and choose "Report a vulnerability", or go straight to
<https://github.com/Akshith-AI/git-koi/security/advisories/new>. Do not open a
public issue for a vulnerability.

Include the command or workflow that triggers the problem and what an attacker
gains from it. The maintainer will review the report and coordinate a fix, with
credit to you unless you ask otherwise.

## Scope

`ltkoi` reads only public GitHub and LeetCode profile data, through the GitHub
CLI and HTTPS and the token the caller supplies, and writes only the files the
caller names. The Action runs inside the calling workflow with that workflow's
permissions. A report that shows it reading, writing, or sending anything beyond
that is in scope.
