# ADR 0003: Separate engineering principles from tooling choices

Date: 2026-10-05
Status: Accepted

## Context

Frameworks and tools change. Naming them as permanent engineering practices
confuses current implementation preferences with enduring quality requirements.
The user requested constitution-like standards supported by established guidance.

## Decision

Keep architecture, design, coding, verification, and AI-development principles
in engineering-standards.md and focused practices documents. Reference established
Rust guidance rather than duplicate a full external checklist.

Record consequential tool/framework selections in ADRs. Keep provisional
candidates separate from accepted decisions. Put current versions and commands
in manifests, executable checks, and verified runbooks once they exist.

## Alternatives

One document mixing principles and command catalogs is convenient initially but
becomes stale and makes optional tool choices appear mandatory.
External links alone are insufficient: project-specific obligations still need
to be explicit and reviewable.

## Consequences

Tools may change while their underlying safeguards remain required. Update
affected ADRs and operating guidance together. Changes to principles require
deliberate rationale, not incidental dependency changes.

## Verification and follow-up

Rust principles now link to the Rust API Guidelines, Book, and Reference.
Previous framework/tool candidates moved to tooling-options.md and remain
provisional. No tooling has been installed or enforced.
