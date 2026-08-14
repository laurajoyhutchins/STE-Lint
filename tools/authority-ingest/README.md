# Authority ingest

This directory is reserved for maintenance tooling that builds or verifies versioned STE runtime data from authorized source material.

It is deliberately outside the normal lint path. A released `ste` binary or package must contain the runtime language data it needs and must not fetch an ASD-STE100 PDF during ordinary use.

The first vertical slice does not implement ingestion. Before a populated source-derived dictionary is committed to this public repository, establish the redistribution basis and add lossless extraction and verification tests.
