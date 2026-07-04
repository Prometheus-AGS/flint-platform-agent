## ADDED Requirements

### Requirement: One compose stack builds and runs everything, all real

`smoke/compose.real.yml` SHALL build and run the real agent + real gate + real forge
(PG + migrate + `fdb-gateway`) + real fabric (gateway + its deps), with the agent wired
to every plane's real endpoint and exposed on `:8088`. No plane is stubbed.

#### Scenario: The full real stack comes up

- **WHEN** `smoke/run-real.sh` runs
- **THEN** every service becomes healthy and the agent boots wired to real gate/forge/fabric, serving `:8088`

### Requirement: The smoke drives the real stack end-to-end incl. a realtime event

An automated smoke SHALL drive the live agent against the real planes — authenticate,
project CRUD (agent store), `fabric.health` (real fabric), gate + forge reads (real) —
AND assert a **realtime event**: the agent subscribes and receives a `EventEnvelope`
change when an upstream change is driven.

#### Scenario: HTTP flow passes against real planes

- **WHEN** the smoke runs its HTTP hops
- **THEN** they pass against the real gate/forge/fabric (any wire drift found is fixed)

#### Scenario: Realtime change is received

- **WHEN** a change is driven (forge write / fabric dev trigger) while the agent is subscribed
- **THEN** the agent receives the corresponding `EventEnvelope` change event over the subscription

### Requirement: The run is one command and cleans up

`smoke/run-real.sh` SHALL bring the stack up, run the smoke, and tear it down (`down
-v`) on success and failure. The stub `run.sh` remains functional.

#### Scenario: One-command real run with teardown

- **WHEN** `smoke/run-real.sh` finishes (pass or fail)
- **THEN** the real stack is torn down with no leftover containers/volumes
