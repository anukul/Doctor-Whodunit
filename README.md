# Mission: Doctor Whodunit

## The Challenge
Something in the vehicle just failed - whodunit?

Travel your system timeline, inject faults you have seen before (or expect in the future), and let the evidence tell the story.

In this challenge, teams build a Safety Evidence Factory around the Battery Thermal Guardian: an EV thermal-runaway early-warning service. Regulations require that occupants are warned minutes before a battery thermal event becomes dangerous. A stale, stuck, delayed, or implausible cell-temperature signal can silently disarm that warning chain.

Your job is to make that failure visible, diagnosable, and provable.

## Your Mission
- Build and run the Battery Thermal Guardian on an AutoSD-based runtime.
- Supervise services and recovery behavior with Eclipse Ankaios.
- Exchange heartbeat, fault, and mitigation events over Eclipse uProtocol.
- Expose diagnostic truth across layers through Eclipse OpenSOVD.
- Execute repeatable fault campaigns with openDuT.
- Produce traceable safety evidence that links:
  - hazard -> safety goal -> injected fault -> detection -> mitigation -> verdict
- Package your setup and evidence process as a reusable SDV Blueprint.

## Challenge Summary
The challenge focuses on continuous safety validation under realistic failure conditions.

Participants inject faults at multiple levels and verify whether the warning chain still meets safety intent:
- Communication-level faults: delayed, dropped, duplicated, or reordered messages.
- Signal-level faults: stuck or implausible VSS temperature values.
- Device-level faults: sensor dropout or data-source interruption.

The expected outcome is not just detection logic, but defensible evidence for every test run.

## Reference Architecture Scope
### In-Scope Runtime Layers
- Safety Application: Battery Thermal Guardian service logic.
- Communications: uProtocol event exchange.
- Middleware/Orchestration: Ankaios lifecycle and recovery control.
- OS Runtime: AutoSD/Linux container runtime.
- Diagnostics: OpenSOVD visibility across layers.

### Evidence and Test Harness
- Fault campaigns executed by openDuT.
- Scenario definitions for repeatable replay.
- Evidence collector that correlates telemetry, diagnostics, and verdicts.
- Exportable, machine-readable results suitable for audit and scoring.

## Key Technologies and Projects
- [openDuT](https://github.com/eclipse-opendut/opendut)
- [Eclipse uProtocol](https://uprotocol.org/)
- [Eclipse Ankaios](https://eclipse-ankaios.github.io/ankaios/)
- [Eclipse OpenSOVD](https://github.com/eclipse-opensovd)
- [AutoSD](https://sig.centos.org/automotive/autosd-10/)
- [S-CORE](https://eclipse.dev/score/)

## Prerequisites
- Basic Rust,C++ or Python
- Linux and container fundamentals
- Pub/sub messaging concepts

## Suggested Deliverables
- Working Guardian deployment with supervised runtime behavior.
- Fault injection campaign definitions and replay scripts.
- Detection and mitigation metrics (for example: detection latency, mitigation success rate, warning lead time).
- Evidence package per scenario with a clear pass/fail verdict.
- Reusable SDV Blueprint bundle (architecture, configs, scripts, evidence format).

## Evaluation Focus
- Safety correctness: does the system detect and mitigate in time?
- Traceability: can each verdict be traced through the full evidence chain?
- Repeatability: can fault campaigns be replayed consistently?
- Observability: are diagnostics complete and useful across layers?
- Reusability: can other teams adopt your blueprint with minimal effort?

## Getting Started in This Repository
- Use [Safety_Evidence_Hackathon.drawio](./Docs/Safety_Evidence_Hackathon.drawio) as the reference architecture diagram.

## Tagline
Every fault leaves evidence. Solve the case.

