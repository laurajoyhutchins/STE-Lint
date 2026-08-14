# Repair protocol

A destination sentence can be valid under implemented STE checks and still be an invalid rewrite. STE-Lint therefore validates the change as well as the destination.

## Agent loop

```text
source
  |
  v
ste lint
  |
  +-- deterministic fixes
  |
  `-- unresolved diagnostics
          |
          v
      repair proposal
          |
          v
   ste check-rewrite
          |
          v
       ste lint
```

The repair backend can be an LLM, a human, or another tool. It does not become the compliance authority.

## Acceptance model

The long-term acceptance contract is:

1. intended target diagnostics are resolved;
2. no new error diagnostics are introduced;
3. protected semantic invariants are preserved.

The first-slice `ste check-rewrite` implements only the deterministic semantic portion. It does not yet invoke the destination linter or prove that a named target diagnostic disappeared. Callers must run `ste lint` after the rewrite.

## Implemented semantic invariants

### Modality

The checker compares case-insensitive multisets of `may`, `can`, `could`, `must`, `should`, and `will`. A change emits `SEM-MODALITY-001`.

This is conservative. It can reject a rewrite that is semantically safe but expresses the same modality differently. Conservative rejection is preferable to silently strengthening or weakening the source.

### Negation

The checker compares occurrences of `not`, `no`, `never`, and `cannot`. A change emits `SEM-NEGATION-001`.

### Numeric literals

The checker compares the ordered sequence of signed integer and decimal literals. A change emits `SEM-QUANTITY-001`.

This protects literal quantities but does not yet understand unit conversions, written numbers, ranges, inequalities, or equivalent numeric expressions.

## Future checks

The design reserves checks for conditions and exceptions, actor identity, object identity, temporal ordering, causal claims, and literal machine tokens. These should be added conservatively with regression fixtures from observed bad repairs.
