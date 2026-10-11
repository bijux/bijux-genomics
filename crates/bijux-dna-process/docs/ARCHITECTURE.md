# Process ownership

`capture` configures isolated spawning and explicit limits. `readiness` drains Unix
pipes with one deadline and a combined stdout/stderr byte budget. `lifecycle` owns
process-group cancellation and direct child reaping. Domain interpretation, executable
selection, argv, environment and publication belong to consumers.

The public bounded capture API rejects non-Unix platforms before spawning. Runner's
existing non-Unix threaded adapter remains uncapped and is not proof of inherited-pipe
deadline protection. Processes escaping the owned group are outside this mechanism.

Runner and containers compile against this small leaf independently of each other.
Only anyhow, wait-timeout and the safe Unix rustix interface are normal dependencies.
This removes duplicate transport ownership without adding a runner dependency to
containers. Cargo graph isolation is distinct from a measured build speedup.
