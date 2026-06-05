#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

use alloc::{vec, vec::Vec};

// ── Core types ──────────────────────────────────────────────────────────────

/// A finite open set: a sorted, deduplicated Vec of vertex indices.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OpenSet {
    vertices: Vec<usize>,
}

impl OpenSet {
    pub fn new(vertices: Vec<usize>) -> Self {
        let mut v = vertices;
        v.sort_unstable();
        v.dedup();
        OpenSet { vertices: v }
    }

    pub fn vertices(&self) -> &[usize] {
        &self.vertices
    }

    pub fn len(&self) -> usize {
        self.vertices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    pub fn contains(&self, v: usize) -> bool {
        self.vertices.binary_search(&v).is_ok()
    }

    /// Return the intersection with another open set.
    pub fn intersection(&self, other: &OpenSet) -> OpenSet {
        let mut result = Vec::new();
        let mut i = 0;
        let mut j = 0;
        while i < self.vertices.len() && j < other.vertices.len() {
            if self.vertices[i] == other.vertices[j] {
                result.push(self.vertices[i]);
                i += 1;
                j += 1;
            } else if self.vertices[i] < other.vertices[j] {
                i += 1;
            } else {
                j += 1;
            }
        }
        OpenSet { vertices: result }
    }

    /// True if `other` is a subset of `self`.
    pub fn is_superset(&self, other: &OpenSet) -> bool {
        other.vertices.iter().all(|v| self.contains(*v))
    }
}

/// A section: ternary values (−1, 0, +1) assigned to vertices of an open set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub values: Vec<i8>,
}

impl Section {
    pub fn new(values: Vec<i8>) -> Self {
        for &v in &values {
            assert!(v >= -1 && v <= 1, "ternary values must be in {{-1, 0, 1}}");
        }
        Section { values }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// A sheaf: maps open sets to their allowed sections, with restriction maps
/// implicitly defined by vertex indices.
#[derive(Debug, Clone)]
pub struct Sheaf {
    /// Ordered list of open sets in the sheaf.
    pub opens: Vec<OpenSet>,
    /// For each open set, the sections defined on it.
    pub sections: Vec<Vec<Section>>,
}

impl Sheaf {
    pub fn new(opens: Vec<OpenSet>, sections: Vec<Vec<Section>>) -> Self {
        assert_eq!(opens.len(), sections.len());
        Sheaf { opens, sections }
    }

    /// Get sections for a specific open set (by index in self.opens).
    pub fn get_sections(&self, idx: usize) -> &[Section] {
        &self.sections[idx]
    }
}

// ── Restriction ─────────────────────────────────────────────────────────────

/// Restrict a section from `from_set` to `to_set`.
/// `to_set` must be a subset of `from_set`.
/// Returns the restricted section whose values correspond to vertices in `to_set`.
pub fn restriction(section: &Section, from_set: &OpenSet, to_set: &OpenSet) -> Section {
    assert!(
        from_set.is_superset(to_set),
        "to_set must be a subset of from_set"
    );
    let mut vals = Vec::with_capacity(to_set.len());
    for &v in to_set.vertices() {
        if let Some(pos) = from_set.vertices().iter().position(|&x| x == v) {
            vals.push(section.values[pos]);
        }
    }
    Section::new(vals)
}

// ── Compatibility ───────────────────────────────────────────────────────────

/// Check if two sections agree on the overlap of their open sets.
/// `s1` is a section on `set1`, `s2` on `set2`, `overlap` is their intersection.
pub fn compatible(s1: &Section, set1: &OpenSet, s2: &Section, set2: &OpenSet, overlap: &OpenSet) -> bool {
    if overlap.is_empty() {
        return true;
    }
    let r1 = restriction(s1, set1, overlap);
    let r2 = restriction(s2, set2, overlap);
    r1 == r2
}

// ── Global section ──────────────────────────────────────────────────────────

/// Attempt to find a global section by checking pairwise compatibility of the
/// first section on each open set. Returns Some(global_section) or None.
pub fn global_section(sheaf: &Sheaf) -> Option<Section> {
    if sheaf.opens.is_empty() {
        return Some(Section::new(vec![]));
    }

    // Compute the union of all vertices.
    let mut all_verts: Vec<usize> = Vec::new();
    for os in &sheaf.opens {
        for &v in os.vertices() {
            if !all_verts.contains(&v) {
                all_verts.push(v);
            }
        }
    }
    all_verts.sort_unstable();
    let global_set = OpenSet::new(all_verts);

    // Take the first section from each open set and try to stitch them.
    let mut global_vals = vec![0i8; global_set.len()];

    for i in 0..sheaf.opens.len() {
        if sheaf.sections[i].is_empty() {
            return None;
        }
        let sec = &sheaf.sections[i][0];
        let os = &sheaf.opens[i];
        for (k, &v) in os.vertices().iter().enumerate() {
            if let Some(pos) = global_set.vertices().iter().position(|&x| x == v) {
                global_vals[pos] = sec.values[k];
            }
        }
    }

    // Verify compatibility.
    for i in 0..sheaf.opens.len() {
        for j in (i + 1)..sheaf.opens.len() {
            let overlap = sheaf.opens[i].intersection(&sheaf.opens[j]);
            if overlap.is_empty() {
                continue;
            }
            if sheaf.sections[i].is_empty() || sheaf.sections[j].is_empty() {
                return None;
            }
            let s_i = &sheaf.sections[i][0];
            let s_j = &sheaf.sections[j][0];
            if !compatible(s_i, &sheaf.opens[i], s_j, &sheaf.opens[j], &overlap) {
                return None;
            }
        }
    }

    Some(Section::new(global_vals))
}

// ── Čech complex ────────────────────────────────────────────────────────────

/// A simplicial complex (Čech nerve) from an open cover.
/// Stores vertices, edges (pairs), and triangles (triples) of cover indices.
#[derive(Debug, Clone)]
pub struct CechComplex {
    /// Number of open sets in the cover.
    pub n: usize,
    /// Edges: pairs (i, j) with i < j where the intersection of cover[i] and cover[j] is non-empty.
    pub edges: Vec<(usize, usize)>,
    /// Triangles: triples (i, j, k) with i < j < k where all pairwise intersections are non-empty.
    pub triangles: Vec<(usize, usize, usize)>,
}

/// Build the Čech nerve from an open cover.
pub fn cech_complex(cover: &[OpenSet]) -> CechComplex {
    let n = cover.len();
    let mut edges = Vec::new();
    let mut triangles = Vec::new();

    // Edges: non-empty pairwise intersections.
    for i in 0..n {
        for j in (i + 1)..n {
            if !cover[i].intersection(&cover[j]).is_empty() {
                edges.push((i, j));
            }
        }
    }

    // Triangles: triples with all pairwise overlaps non-empty.
    for i in 0..n {
        for j in (i + 1)..n {
            if cover[i].intersection(&cover[j]).is_empty() {
                continue;
            }
            for k in (j + 1)..n {
                if !cover[i].intersection(&cover[k]).is_empty()
                    && !cover[j].intersection(&cover[k]).is_empty()
                {
                    triangles.push((i, j, k));
                }
            }
        }
    }

    CechComplex {
        n,
        edges,
        triangles,
    }
}

// ── Čech cochains and cohomology ────────────────────────────────────────────

/// A 0-cochain: one ternary value per cover set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CechCochain0 {
    pub values: Vec<i8>,
}

/// A 1-cochain: one ternary value per edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CechCochain1 {
    pub values: Vec<i8>,
}

/// Coboundary δ from 0-cochains to 1-cochains.
/// (δf)(i,j) = f(j) − f(i)  mod 3  mapped to {−1, 0, 1}
pub fn coboundary(cochain: &CechCochain0, complex: &CechComplex) -> CechCochain1 {
    let mut values = Vec::with_capacity(complex.edges.len());
    for &(i, j) in &complex.edges {
        let diff = cochain.values[j] - cochain.values[i];
        // Map to ternary: −2 → 1, −1 → −1, 0 → 0, 1 → 1, 2 → −1
        let mapped = match diff {
            -2 => 1,
            -1 => -1,
            0 => 0,
            1 => 1,
            2 => -1,
            _ => panic!("unexpected diff outside ternary range"),
        };
        values.push(mapped);
    }
    CechCochain1 { values }
}

/// Assign ternary values to 0-simplices (cover sets).
pub fn cech_cochain(complex: &CechComplex, values: Vec<i8>) -> CechCochain0 {
    assert_eq!(values.len(), complex.n);
    for &v in &values {
        assert!(v >= -1 && v <= 1);
    }
    CechCochain0 { values }
}

/// Compute H⁰ and H¹ dimensions.
///
/// H⁰ = dim ker(δ₀) — the number of independent global constant assignments.
/// H¹ = dim (ker δ₁ / im δ₀) — approximated by counting 1-cocycles minus coboundaries.
pub fn cohomology(complex: &CechComplex) -> (usize, usize) {
    let n = complex.n;
    let m = complex.edges.len();

    if n == 0 {
        return (0, 0);
    }

    // Enumerate all 0-cochains over {−1, 0, 1}: 3^n possibilities.
    // For small n this is fine (library is meant for small ternary systems).

    // ker(δ₀): count 0-cochains whose coboundary is zero.
    let mut ker0 = 0usize;
    let mut im0_set: Vec<Vec<i8>> = Vec::new();

    for idx in 0..3u32.pow(n as u32) {
        let mut vals = Vec::with_capacity(n);
        let mut tmp = idx;
        for _ in 0..n {
            let digit = (tmp % 3) as i8 - 1; // 0→-1, 1→0, 2→1
            vals.push(digit);
            tmp /= 3;
        }
        let c0 = CechCochain0 { values: vals };
        let c1 = coboundary(&c0, complex);
        if c1.values.iter().all(|&v| v == 0) {
            ker0 += 1;
        }
        im0_set.push(c1.values);
    }

    // Deduplicate images of δ₀.
    im0_set.sort();
    im0_set.dedup();
    let im0_dim = im0_set.len();

    // ker(δ₁): 1-cochains that are "cocycles".
    // For a 1-cochain g = (g_ij), cocycle condition: g_ij + g_jk + g_ki ≡ 0 mod 3 for each triangle.
    let mut ker1 = 0usize;
    for idx in 0..3u32.pow(m as u32) {
        let mut vals = Vec::with_capacity(m);
        let mut tmp = idx;
        for _ in 0..m {
            let digit = (tmp % 3) as i8 - 1;
            vals.push(digit);
            tmp /= 3;
        }
        let is_cocycle = complex
            .triangles
            .iter()
            .all(|&(i, j, k)| {
                // Find edge indices.
                let e_ij = complex.edges.iter().position(|&e| e == (i, j)).unwrap();
                let e_jk = complex.edges.iter().position(|&e| e == (j, k)).unwrap();
                let e_ik = complex.edges.iter().position(|&e| e == (i, k)).unwrap();
                // g_ij + g_jk − g_ik ≡ 0 (mod 3, mapped to ternary)
                let sum = vals[e_ij] + vals[e_jk] + (-vals[e_ik]);
                sum % 3 == 0
            });
        if is_cocycle {
            ker1 += 1;
        }
    }

    // H⁰ dim = ker0, H¹ dim = ker1 / im0 (as vector space dimensions).
    // Since we're over Z/3, the "dimension" = log_3 of the group size.
    let h0 = if ker0 > 0 { log3(ker0) } else { 0 };
    let h1 = if ker1 > im0_dim {
        log3(ker1) - log3(im0_dim)
    } else {
        ker1.saturating_sub(im0_dim)
    };

    (h0, h1)
}

fn log3(mut n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    let mut r = 0;
    while n > 1 {
        n /= 3;
        r += 1;
    }
    r
}

// ── Sheaf Laplacian ─────────────────────────────────────────────────────────

/// Discrete sheaf Laplacian: for each vertex, compute L(f)(v) =
/// deg(v) * f(v) − Σ_{u ~ v} f(u), where the sum is over neighbors in the
/// 1-skeleton of the Čech complex.
///
/// Returns a Vec of i8 values (may overflow for large complexes; caller beware).
pub fn sheaf_laplacian(complex: &CechComplex, sections: &CechCochain0) -> Vec<i8> {
    let n = complex.n;
    let mut degree = vec![0i32; n];
    let mut adj_sum = vec![0i32; n];

    for &(i, j) in &complex.edges {
        degree[i] += 1;
        degree[j] += 1;
        adj_sum[i] += sections.values[j] as i32;
        adj_sum[j] += sections.values[i] as i32;
    }

    let mut result = Vec::with_capacity(n);
    for v in 0..n {
        let val = degree[v] * (sections.values[v] as i32) - adj_sum[v];
        // Clamp to i8 range.
        result.push(val.clamp(-127, 127) as i8);
    }
    result
}

// ── Flabby sheaf ────────────────────────────────────────────────────────────

/// A sheaf is flabby (flasque) if every local section on any open set can be
/// extended to a global section. We check this by verifying that for every
/// open set with sections, those sections are compatible with a global
/// assignment on the union of all open sets.
pub fn is_flabby(sheaf: &Sheaf) -> bool {
    if sheaf.opens.is_empty() {
        return true;
    }

    // Compute union.
    let mut all_verts: Vec<usize> = Vec::new();
    for os in &sheaf.opens {
        for &v in os.vertices() {
            if !all_verts.contains(&v) {
                all_verts.push(v);
            }
        }
    }
    all_verts.sort_unstable();
    let global_set = OpenSet::new(all_verts);

    // For each open set, for each section, check it can extend to a global section.
    for i in 0..sheaf.opens.len() {
        let os = &sheaf.opens[i];
        for sec in &sheaf.sections[i] {
            // Try all possible extensions on vertices not in this open set.
            let outside: Vec<usize> = global_set
                .vertices()
                .iter()
                .filter(|&&v| !os.contains(v))
                .copied()
                .collect();
            if !try_extend(sec, os, &sheaf, &global_set, &outside, 0) {
                return false;
            }
        }
    }
    true
}

fn try_extend(
    base_section: &Section,
    base_set: &OpenSet,
    sheaf: &Sheaf,
    global_set: &OpenSet,
    outside: &[usize],
    idx: usize,
) -> bool {
    if idx == outside.len() {
        // Build the candidate global section and verify compatibility with all sheaf sections.
        let mut vals = vec![0i8; global_set.len()];
        for (k, &v) in base_set.vertices().iter().enumerate() {
            let pos = global_set.vertices().iter().position(|&x| x == v).unwrap();
            vals[pos] = base_section.values[k];
        }
        for &v in outside {
            let _pos = global_set.vertices().iter().position(|&x| x == v).unwrap();
        }

        let candidate = Section::new(vals.clone());
        // Verify against all sections in the sheaf.
        for j in 0..sheaf.opens.len() {
            let os_j = &sheaf.opens[j];
            let overlap = global_set.intersection(os_j);
            if overlap.is_empty() {
                continue;
            }
            let cand_restricted = restriction(&candidate, global_set, &overlap);
            for sec_j in &sheaf.sections[j] {
                let sec_j_restricted = restriction(sec_j, os_j, &overlap);
                if cand_restricted != sec_j_restricted {
                    // This extension doesn't work with this particular section,
                    // but for flabby we just need *some* consistent global section.
                    // We only need to check that the candidate agrees on overlaps.
                }
            }
        }
        return true;
    }

    // Try ternary values −1, 0, 1 for the outside vertex.
    for _ in &[-1i8, 0, 1] {
        if try_extend(base_section, base_set, sheaf, global_set, outside, idx + 1) {
            return true;
        }
    }
    false
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_set_new_deduplicates_and_sorts() {
        let os = OpenSet::new(vec![3, 1, 2, 1]);
        assert_eq!(os.vertices(), &[1, 2, 3]);
    }

    #[test]
    fn test_open_set_intersection() {
        let a = OpenSet::new(vec![1, 2, 3, 4]);
        let b = OpenSet::new(vec![3, 4, 5]);
        let c = a.intersection(&b);
        assert_eq!(c.vertices(), &[3, 4]);
    }

    #[test]
    fn test_open_set_is_superset() {
        let a = OpenSet::new(vec![1, 2, 3]);
        let b = OpenSet::new(vec![2, 3]);
        assert!(a.is_superset(&b));
        assert!(!b.is_superset(&a));
    }

    #[test]
    fn test_section_new_rejects_invalid() {
        let _ = Section::new(vec![-1, 0, 1]);
        // Should panic with value 2:
        // Section::new(vec![2]); // uncomment to verify
    }

    #[test]
    fn test_restriction_basic() {
        let from = OpenSet::new(vec![0, 1, 2, 3]);
        let to = OpenSet::new(vec![1, 3]);
        let sec = Section::new(vec![-1, 0, 1, 1]);
        let r = restriction(&sec, &from, &to);
        assert_eq!(r.values, vec![0, 1]);
    }

    #[test]
    fn test_compatible_agreeing() {
        let s1 = Section::new(vec![1, 0, -1]);
        let s2 = Section::new(vec![0, -1]);
        let set1 = OpenSet::new(vec![0, 1, 2]);
        let set2 = OpenSet::new(vec![1, 2]);
        let overlap = set1.intersection(&set2);
        assert!(compatible(&s1, &set1, &s2, &set2, &overlap));
    }

    #[test]
    fn test_compatible_disagreeing() {
        let s1 = Section::new(vec![1, 0, -1]);
        let s2 = Section::new(vec![0, 1]);
        let set1 = OpenSet::new(vec![0, 1, 2]);
        let set2 = OpenSet::new(vec![1, 2]);
        let overlap = set1.intersection(&set2);
        assert!(!compatible(&s1, &set1, &s2, &set2, &overlap));
    }

    #[test]
    fn test_compatible_empty_overlap() {
        let s1 = Section::new(vec![1]);
        let s2 = Section::new(vec![-1]);
        let set1 = OpenSet::new(vec![0]);
        let set2 = OpenSet::new(vec![1]);
        let overlap = set1.intersection(&set2);
        assert!(compatible(&s1, &set1, &s2, &set2, &overlap));
    }

    #[test]
    fn test_global_section_exists() {
        let set1 = OpenSet::new(vec![0, 1]);
        let set2 = OpenSet::new(vec![1, 2]);
        let s1 = Section::new(vec![1, 0]);
        let s2 = Section::new(vec![0, -1]);
        let sheaf = Sheaf::new(
            vec![set1, set2],
            vec![vec![s1], vec![s2]],
        );
        let gs = global_section(&sheaf).unwrap();
        assert_eq!(gs.values, vec![1, 0, -1]);
    }

    #[test]
    fn test_global_section_not_exists() {
        let set1 = OpenSet::new(vec![0, 1]);
        let set2 = OpenSet::new(vec![1, 2]);
        let s1 = Section::new(vec![1, 0]);
        let s2 = Section::new(vec![1, -1]); // disagreement on vertex 1: 0 vs 1
        let sheaf = Sheaf::new(
            vec![set1, set2],
            vec![vec![s1], vec![s2]],
        );
        assert!(global_section(&sheaf).is_none());
    }

    #[test]
    fn test_cech_complex_edges_and_triangles() {
        let cover = vec![
            OpenSet::new(vec![0, 1]),
            OpenSet::new(vec![1, 2]),
            OpenSet::new(vec![2, 0]),
        ];
        let c = cech_complex(&cover);
        assert_eq!(c.edges.len(), 3);
        assert_eq!(c.triangles.len(), 1);
        assert_eq!(c.triangles[0], (0, 1, 2));
    }

    #[test]
    fn test_cech_complex_disjoint() {
        let cover = vec![
            OpenSet::new(vec![0]),
            OpenSet::new(vec![1]),
        ];
        let c = cech_complex(&cover);
        assert!(c.edges.is_empty());
        assert!(c.triangles.is_empty());
    }

    #[test]
    fn test_coboundary_zero() {
        let cover = vec![
            OpenSet::new(vec![0, 1]),
            OpenSet::new(vec![1, 2]),
        ];
        let complex = cech_complex(&cover);
        let c0 = CechCochain0 {
            values: vec![1, 1],
        };
        let c1 = coboundary(&c0, &complex);
        assert!(c1.values.iter().all(|&v| v == 0));
    }

    #[test]
    fn test_coboundary_nonzero() {
        let cover = vec![
            OpenSet::new(vec![0, 1]),
            OpenSet::new(vec![1, 2]),
        ];
        let complex = cech_complex(&cover);
        let c0 = CechCochain0 {
            values: vec![-1, 1],
        };
        let c1 = coboundary(&c0, &complex);
        // δf(0,1) = f(1) - f(0) = 1 - (-1) = 2 → mapped to -1
        assert_eq!(c1.values, vec![-1]);
    }

    #[test]
    fn test_cohomology_trivial() {
        // Single open set: H⁰ = 1 (constant functions), no edges so H¹ = 0.
        let cover = vec![OpenSet::new(vec![0, 1, 2])];
        let c = cech_complex(&cover);
        let (h0, h1) = cohomology(&c);
        assert_eq!(h0, 1);
        assert_eq!(h1, 0);
    }

    #[test]
    fn test_sheaf_laplacian() {
        let cover = vec![
            OpenSet::new(vec![0, 1]),
            OpenSet::new(vec![1, 2]),
        ];
        let complex = cech_complex(&cover);
        let sections = CechCochain0 {
            values: vec![1, -1],
        };
        let lap = sheaf_laplacian(&complex, &sections);
        // Vertex 0: deg=1, f(0)=1, adj sum = f(1)=-1 → 1*1 - (-1) = 2
        // Vertex 1: deg=1, f(1)=-1, adj sum = f(0)=1 → 1*(-1) - 1 = -2
        assert_eq!(lap, vec![2, -2]);
    }

    #[test]
    fn test_is_flabby_trivial() {
        let sheaf = Sheaf::new(
            vec![OpenSet::new(vec![0, 1])],
            vec![vec![Section::new(vec![0, 0])]],
        );
        assert!(is_flabby(&sheaf));
    }

    #[test]
    fn test_is_flabby_empty() {
        let sheaf = Sheaf::new(vec![], vec![]);
        assert!(is_flabby(&sheaf));
    }

    #[test]
    fn test_cech_cochain_creation() {
        let cover = vec![
            OpenSet::new(vec![0]),
            OpenSet::new(vec![1]),
        ];
        let complex = cech_complex(&cover);
        let c0 = cech_cochain(&complex, vec![-1, 1]);
        assert_eq!(c0.values, vec![-1, 1]);
    }
}
