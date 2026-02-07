# USAGE

## Reference patterns
### Fixture helper
```
use bijux_testkit::fixtures::load;
let bytes = load("fastq/trim/example.txt")?;
```

### Snapshot helper
```
use bijux_testkit::snapshots::assert_snapshot;
assert_snapshot!(value);
```

### Golden run builder
```
use bijux_testkit::golden::build_run;
let run = build_run("fastq.default.v1")?;
```
