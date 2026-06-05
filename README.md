# ternary-sheaf

**Local data, global truth. Sheaf theory and Čech cohomology over ternary spaces.**

A sheaf assigns data to every open set in a topological space, with the constraint that locally compatible data must agree on overlaps. Sheaf theory is the mathematical language of gluing: if you know what happens in each neighborhood, and the neighborhoods agree where they overlap, do you know what happens globally?

The answer is: not always. The obstruction is measured by sheaf cohomology. H⁰ counts the global sections (the data that glues successfully). H¹ counts the obstructions (the local data that *can't* be glued). In ternary sheaf theory, both are computable as Z₃-valued dimensions.

This crate implements: open sets and sections with ternary values, restriction maps between open sets, compatibility checking on overlaps, global section construction, the Čech nerve (a simplicial complex from an open cover), coboundary operators and cohomology groups (H⁰ and H¹), the sheaf Laplacian (spectral tool for analyzing sheaf structure), and flabbiness detection (can every local section be extended globally?).

## What's Inside

- **`OpenSet`** — sorted vertex sets with intersection/superset operations
- **`Section`** — ternary values {-1, 0, +1} assigned to vertices
- **`Sheaf`** — maps OpenSets to sections with restriction maps
- **`restriction()`** — restrict a section to a smaller open set
- **`compatible()`** — do two sections agree on their overlap?
- **`global_section()`** — find a global section from compatible local ones
- **`cech_complex()`** — build the Čech nerve from an open cover
- **`coboundary()`** — the δ operator on cochains
- **`cohomology()`** — compute H⁰ and H¹ dimensions over Z₃
- **`sheaf_laplacian()`** — discrete Laplacian on sheaf sections
- **`is_flabby()`** — can every section extend globally?

## Quick Example

```rust
use ternary_sheaf::*;

// Define an open cover
let cover = vec![
    OpenSet::new(vec![0, 1, 2]),
    OpenSet::new(vec![1, 2, 3]),
];

// Build Čech complex
let complex = cech_complex(&cover);
let (h0, h1) = cohomology(&complex);
println!("H⁰ = {}, H¹ = {}", h0, h1);
```

## The Deeper Truth

**Sheaf cohomology is the measure of how local truth fails to become global truth.** When H¹ = 0, every compatible local picture glues into a consistent global picture. When H¹ > 0, there are obstructions — local data that looks consistent but can't be assembled globally.

In ternary systems, this has a direct physical interpretation. Imagine a fleet of agents, each maintaining a local view of the world (a section over their "neighborhood"). If two adjacent agents agree on their overlap, they should be able to merge their views. H⁰ counts how many globally consistent worldviews exist. H¹ counts how many *seemingly consistent* local views actually fail to merge.

The sheaf Laplacian — a discrete analog of the continuous Laplacian — reveals the spectral structure of the sheaf. Its kernel corresponds to global sections. Its nonzero eigenvalues measure the "tension" in the sheaf: how hard the local data is pulling in different directions. In ternary, the eigenvalues are mod-3 values, giving a purely algebraic spectral theory.

**Use cases:**
- **Distributed consensus** — sheaf cohomology measures consensus achievability
- **Sensor fusion** — gluing local sensor readings into global state
- **Data integration** — merging heterogeneous data sources with overlap
- **Multi-agent world models** — detecting inconsistencies in distributed beliefs
- **Topological data analysis** — the Čech complex is the foundation of TDA

## See Also

- **ternary-topology** — topological spaces and simplicial complexes
- **ternary-entropy** — information-theoretic view of local/global structure
- **ternary-consensus** — consensus protocols and their sheaf-theoretic interpretation
- **ternary-crystal** — crystallography uses sheaves on periodic lattices
- **ternary-graph** — graph Laplacians are a special case of sheaf Laplacians
- **ternary-network** — network flows as sheaf sections

## Install

```bash
cargo add ternary-sheaf
```

## License

MIT
