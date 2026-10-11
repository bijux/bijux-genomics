# Population ownership

Each capability separates admission, model, workflow, execution, observations and
publication. IBD and demography share output exclusion; demography invokes the
already locked IBD service without reacquiring the same lock.

Staged population publications preflight every replacement and retirement. An
ownership inventory records exact generated files, while unrelated files survive.
Legacy reports admit only the same stage/tool/schema and claim only paths generated
by the current service. Unclaimed legacy paths survive. File replacement is atomic;
a complete multi-file publication is not transactional on filesystem failure.

Native execution, fallback, synthetic and proxy observations retain their existing
classification. Producing a report is not proof of scientific interchangeability.
