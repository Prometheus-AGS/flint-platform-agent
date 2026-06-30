# Goals

- Wire the composition root: build the four plane adapters and inject fpa_app::TaskRunner as Axum state
- Define the A2A administrative task catalog (project + application management operations) and dispatch tools/call + task submission through it
- Implement project management capabilities: enumerate, inspect, and administer fabric projects/entities via the forge port
- Implement application management capabilities: lifecycle operations across forge/fabric/gate planes
- Surface management operations as A2UI primitives bound to A2A task kinds
