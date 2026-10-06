# jev-sample
Rust based sample to demonstrate use of Jev model in FSI applications

[Development setup](docs/development.md)

## Project decisions

This sample explores synthetic auto-insurance claim-intake analysis and triage,
not coverage or payout decisions. A local Axum health API and integration tests
exist; domain behavior and provider integration have not started.

Run `cargo run --bin api --locked`, then
`curl -i http://127.0.0.1:3000/health`. See the development guide for the exact
contract, Rust explanations, tests, and prototype limitations.

- [Jev capabilities and limitations](docs/jev-capabilities.md)
- [Approved product scope](docs/product-scope.md)
- [Product discovery draft and clickable mockup](docs/product-spec.md)
- [Customer, employee and operator journey specifications](docs/persona-journeys.md)
- [Approved metrics and targets](docs/metrics.md)
- [Approved comparison design](docs/evaluation-design.md)
- [Approved application architecture](docs/adr/0007-single-package-api-and-evaluation.md)
- [Engineering standards](engineering-standards.md)
- [Harness working agreement](AGENTS.md)
- [Engineering learning loop](docs/engineering-maintenance.md)
- [Architecture decision records](docs/adr/README.md)
- [Milestone task list](docs/task-list.md)

Documents distinguish approved decisions from proposals. Work pauses for discussion
after each logical milestone.
