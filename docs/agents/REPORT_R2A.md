STATUS: COMPLETE
FILES_MODIFIED:
  - crates/forge-core/src/event_bus.rs (complete rewrite: single broadcast → fan-out mpsc+broadcast)
  - crates/forge-core/src/lib.rs (3 tests updated to new constructor, 1 new test for persist channel)
  - crates/forge-app/src/main.rs (EventBus construction + BatchWriter wiring via mpsc)
  - crates/forge-api/src/routes/ws.rs (comment added: best-effort delivery note)
  - crates/forge-api/src/routes/agents.rs (3 emit calls → async .await)
  - crates/forge-api/src/routes/hooks.rs (4 emit calls → async .await)
  - crates/forge-api/src/routes/workflows.rs (3 emit calls → async .await)
  - crates/forge-api/src/middleware.rs (4 emit calls → async .await, 5 test constructors updated)
  - crates/forge-api/src/lib.rs (test helper constructor updated)
  - crates/forge-process/src/runner.rs (10 emit calls → emit_sync, 3 test constructors updated)
  - crates/forge-process/src/concurrent.rs (5 emit calls → emit_sync, 1 test constructor updated)
EMIT_CALLSITES_UPDATED: 14 async (.await), 15 sync (emit_sync)
TESTS_UPDATED: 13 (4 forge-core, 3 forge-process/runner, 1 forge-process/concurrent, 5 forge-api/middleware)
CARGO_CHECK: pass (zero warnings)
CARGO_TEST: pass (all green)
NOTES:
  - EventBus::new() now returns (EventBus, mpsc::Receiver<ForgeEvent>) tuple
  - persist_capacity=1024, broadcast_capacity=256 in production (main.rs)
  - BatchWriter wiring simplified: no more Lagged error handling (mpsc backpressures instead)
  - forge-process uses emit_sync() since ProcessRunner methods are synchronous
  - concurrent.rs also uses emit_sync() — calls were already best-effort (let _ =)
  - Added emit_event_received_by_persist_channel test to verify mpsc delivery
