# counter

An h5i-app kernel and its Lean proofs.

| Path | Contents |
|---|---|
| `kernel/src/lib.rs` | the application logic the proofs are about |
| `proofs/Theorems.lean` | the spec and its proofs |
| `proofs/generated/` | Lean extracted from `kernel/` by `h5i app extract`; never edit |
| `h5i-app.toml` | what `h5i app` extracts, gates and mutates |

```
h5i app doctor         # Charon, Aeneas and Lean at the pinned versions
h5i app prove          # extract, then build and gate the proofs
h5i app mutate         # the bugs listed in h5i-app.toml must break a proof
h5i app mutate --auto  # so should generated ones; a survivor is a gap in the spec
```

`h5i app check` rejects `sorry`, `native_decide` and `axiom`, and any theorem
in a hand-written module that depends on an axiom beyond `propext`,
`Classical.choice` and `Quot.sound`.

The tactics and lemmas for the proofs (`h5i_invert`, `h5i_arith`,
`h5i_search_any`, ...) are listed in H5iAppLib's README:
`crates/h5i-app-core/proofs/README.md` in the h5i repository.
