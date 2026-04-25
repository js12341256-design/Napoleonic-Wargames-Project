//! Strategic-map graph and pathfinding (PROMPT.md §16.3).
//!
//! Two distance metrics:
//!
//! - **Hop distance** — count of land edges, computed by BFS.
//!   Always defined when the destination is reachable.
//! - **Cost distance** — sum of integer `AreaAdjacency.cost` values,
//!   computed by Dijkstra.  Returns `None` if any edge on the
//!   shortest-cost candidate is `Maybe::Placeholder`.
//!
//! Iteration order is deterministic everywhere: neighbour lists are
//! `BTreeSet<AreaId>` and the Dijkstra frontier ties on `AreaId`
//! lexicographically.  See `docs/rules/movement.md`.

use std::collections::{BTreeMap, BTreeSet, BinaryHeap};

use gc1805_core_schema::ids::AreaId;
use gc1805_core_schema::scenario::Scenario;
use gc1805_core_schema::tables::Maybe;

/// Compiled view of a scenario's land-area topology.
#[derive(Debug, Clone)]
pub struct MapGraph {
    /// `from` → ordered set of neighbours.
    neighbours: BTreeMap<AreaId, BTreeSet<AreaId>>,
    /// `(from, to)` → cost.  Edges absent from the scenario do not
    /// appear here.  Cost is `None` when the scenario authored a
    /// placeholder.
    costs: BTreeMap<(AreaId, AreaId), Option<i32>>,
}

impl MapGraph {
    /// Build a graph from a scenario.  Symmetry was already verified at
    /// load time (`crates/core/src/validate.rs`); we still enumerate
    /// each scenario edge in both directions for safety.
    pub fn from_scenario(s: &Scenario) -> Self {
        let mut neighbours: BTreeMap<AreaId, BTreeSet<AreaId>> = BTreeMap::new();
        for area in s.areas.keys() {
            neighbours.insert(area.clone(), BTreeSet::new());
        }
        let mut costs: BTreeMap<(AreaId, AreaId), Option<i32>> = BTreeMap::new();
        for edge in &s.adjacency {
            neighbours
                .entry(edge.from.clone())
                .or_default()
                .insert(edge.to.clone());
            let resolved = match &edge.cost {
                Maybe::Value(v) => Some(*v),
                Maybe::Placeholder(_) => None,
            };
            costs.insert((edge.from.clone(), edge.to.clone()), resolved);
        }
        Self { neighbours, costs }
    }

    pub fn area_count(&self) -> usize {
        self.neighbours.len()
    }

    pub fn neighbours_of<'a>(&'a self, area: &AreaId) -> Option<&'a BTreeSet<AreaId>> {
        self.neighbours.get(area)
    }

    /// Are `a` and `b` directly connected (in either direction)?
    pub fn adjacent(&self, a: &AreaId, b: &AreaId) -> bool {
        self.neighbours
            .get(a)
            .map(|set| set.contains(b))
            .unwrap_or(false)
    }

    /// BFS shortest path in hops.  Returns `Some(path)` inclusive of
    /// both endpoints, or `None` if unreachable / unknown areas.
    pub fn shortest_path_hops(&self, from: &AreaId, to: &AreaId) -> Option<Vec<AreaId>> {
        if !self.neighbours.contains_key(from) || !self.neighbours.contains_key(to) {
            return None;
        }
        if from == to {
            return Some(vec![from.clone()]);
        }
        let mut prev: BTreeMap<AreaId, AreaId> = BTreeMap::new();
        let mut visited: BTreeSet<AreaId> = BTreeSet::new();
        // Use a deterministic Vec-based BFS frontier; we drain in
        // insertion order, neighbours are already in BTreeSet order.
        let mut frontier: Vec<AreaId> = vec![from.clone()];
        visited.insert(from.clone());
        while !frontier.is_empty() {
            let mut next: Vec<AreaId> = Vec::new();
            for u in &frontier {
                if let Some(nbrs) = self.neighbours.get(u) {
                    for v in nbrs {
                        if visited.insert(v.clone()) {
                            prev.insert(v.clone(), u.clone());
                            if v == to {
                                return Some(reconstruct(&prev, from, to));
                            }
                            next.push(v.clone());
                        }
                    }
                }
            }
            frontier = next;
        }
        None
    }

    /// Hop count, i.e. `path.len() - 1`.
    pub fn distance_hops(&self, from: &AreaId, to: &AreaId) -> Option<i32> {
        self.shortest_path_hops(from, to)
            .map(|p| (p.len() as i32) - 1)
    }

    /// Dijkstra by integer edge cost.
    ///
    /// Edges with placeholder costs are treated as impassable for
    /// cost-weighted pathfinding — they are not "broken," they simply
    /// have no authored cost yet.  The function returns `None` when
    /// the destination is unreachable through known-cost edges.
    ///
    /// When two paths have equal total cost the tiebreaker is
    /// lexicographic on the destination AreaId at every relaxation
    /// step (PROMPT.md §2.2).
    pub fn shortest_path_cost(&self, from: &AreaId, to: &AreaId) -> Option<(i32, Vec<AreaId>)> {
        if !self.neighbours.contains_key(from) || !self.neighbours.contains_key(to) {
            return None;
        }
        if from == to {
            return Some((0, vec![from.clone()]));
        }
        // Min-heap keyed on (cost, area) so equal-cost frontier picks
        // the lexicographically smallest area first.
        #[derive(PartialEq, Eq)]
        struct Frontier {
            neg_cost: i64, // negate for max-heap → min-heap
            area: AreaId,
        }
        impl Ord for Frontier {
            fn cmp(&self, o: &Self) -> std::cmp::Ordering {
                // higher (neg_cost, smaller area name) wins
                self.neg_cost
                    .cmp(&o.neg_cost)
                    .then_with(|| o.area.cmp(&self.area))
            }
        }
        impl PartialOrd for Frontier {
            fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(o))
            }
        }

        let mut dist: BTreeMap<AreaId, i32> = BTreeMap::new();
        let mut prev: BTreeMap<AreaId, AreaId> = BTreeMap::new();
        let mut heap: BinaryHeap<Frontier> = BinaryHeap::new();

        dist.insert(from.clone(), 0);
        heap.push(Frontier {
            neg_cost: 0,
            area: from.clone(),
        });

        while let Some(Frontier { neg_cost, area }) = heap.pop() {
            let cost_here = -neg_cost as i32;
            if let Some(d) = dist.get(&area) {
                if cost_here > *d {
                    continue;
                }
            }
            if &area == to {
                let path = reconstruct(&prev, from, to);
                return Some((cost_here, path));
            }
            let nbrs = match self.neighbours.get(&area) {
                Some(n) => n,
                None => continue,
            };
            for v in nbrs {
                let edge_cost = match self.costs.get(&(area.clone(), v.clone())).copied() {
                    Some(Some(c)) => c,
                    Some(None) => continue, // placeholder cost: skip
                    None => continue,
                };
                if edge_cost < 0 {
                    return None; // negative-cost edges are illegal
                }
                let next_cost = cost_here + edge_cost;
                let improve = match dist.get(v) {
                    None => true,
                    Some(existing) => next_cost < *existing,
                };
                if improve {
                    dist.insert(v.clone(), next_cost);
                    prev.insert(v.clone(), area.clone());
                    heap.push(Frontier {
                        neg_cost: -(next_cost as i64),
                        area: v.clone(),
                    });
                }
            }
        }
        None
    }
}

fn reconstruct(prev: &BTreeMap<AreaId, AreaId>, from: &AreaId, to: &AreaId) -> Vec<AreaId> {
    let mut out: Vec<AreaId> = vec![to.clone()];
    let mut cur = to.clone();
    while &cur != from {
        let p = prev
            .get(&cur)
            .expect("predecessor must exist on a reconstructed path");
        out.push(p.clone());
        cur = p.clone();
    }
    out.reverse();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use gc1805_core_schema::ids::AreaId;
    use gc1805_core_schema::scenario::{
        Area, AreaAdjacency, Features, GameDate, MovementRules, Owner, PowerSlot, Scenario,
        Terrain, SCHEMA_VERSION,
    };
    use gc1805_core_schema::tables::Maybe;
    use std::collections::BTreeMap;

    fn area(name: &str, owner: &str) -> Area {
        Area {
            display_name: name.into(),
            owner: Owner::Power(PowerSlot {
                power: gc1805_core_schema::ids::PowerId::from(owner),
            }),
            terrain: Terrain::Open,
            fort_level: 0,
            money_yield: Maybe::Value(0),
            manpower_yield: Maybe::Value(0),
            capital_of: None,
            port: false,
            map_x: 0,
            map_y: 0,
        }
    }

    fn edge(from: &str, to: &str, cost: Option<i32>) -> AreaAdjacency {
        AreaAdjacency {
            from: AreaId::from(from),
            to: AreaId::from(to),
            cost: match cost {
                Some(v) => Maybe::Value(v),
                None => Maybe::placeholder(),
            },
        }
    }

    /// 5-node line graph: A — B — C — D — E with unit costs.
    fn line() -> Scenario {
        let mut s = empty();
        for n in ["AREA_A", "AREA_B", "AREA_C", "AREA_D", "AREA_E"] {
            s.areas.insert(AreaId::from(n), area(n, "FRA"));
        }
        for (a, b) in [
            ("AREA_A", "AREA_B"),
            ("AREA_B", "AREA_C"),
            ("AREA_C", "AREA_D"),
            ("AREA_D", "AREA_E"),
        ] {
            s.adjacency.push(edge(a, b, Some(1)));
            s.adjacency.push(edge(b, a, Some(1)));
        }
        s
    }

    /// Diamond graph — A connected to B and C, both connected to D.
    /// Tests tiebreaking.
    fn diamond() -> Scenario {
        let mut s = empty();
        for n in ["AREA_A", "AREA_B", "AREA_C", "AREA_D"] {
            s.areas.insert(AreaId::from(n), area(n, "FRA"));
        }
        for (a, b, c) in [
            ("AREA_A", "AREA_B", 1),
            ("AREA_A", "AREA_C", 1),
            ("AREA_B", "AREA_D", 1),
            ("AREA_C", "AREA_D", 1),
        ] {
            s.adjacency.push(edge(a, b, Some(c)));
            s.adjacency.push(edge(b, a, Some(c)));
        }
        s
    }

    /// Same diamond but with one placeholder edge.
    fn diamond_with_placeholder_branch() -> Scenario {
        let mut s = empty();
        for n in ["AREA_A", "AREA_B", "AREA_C", "AREA_D"] {
            s.areas.insert(AreaId::from(n), area(n, "FRA"));
        }
        s.adjacency.push(edge("AREA_A", "AREA_B", Some(1)));
        s.adjacency.push(edge("AREA_B", "AREA_A", Some(1)));
        s.adjacency.push(edge("AREA_A", "AREA_C", None));
        s.adjacency.push(edge("AREA_C", "AREA_A", None));
        s.adjacency.push(edge("AREA_B", "AREA_D", Some(1)));
        s.adjacency.push(edge("AREA_D", "AREA_B", Some(1)));
        s.adjacency.push(edge("AREA_C", "AREA_D", Some(1)));
        s.adjacency.push(edge("AREA_D", "AREA_C", Some(1)));
        s
    }

    fn empty() -> Scenario {
        Scenario {
            schema_version: SCHEMA_VERSION,
            rules_version: 0,
            scenario_id: "fixture".into(),
            name: "fixture".into(),
            start: GameDate::new(1805, 4),
            end: GameDate::new(1805, 5),
            unplayable_in_release: true,
            features: Features::default(),
            movement_rules: MovementRules::default(),
            current_turn: 0,
            power_state: BTreeMap::new(),
            production_queue: Vec::new(),
            replacement_queue: Vec::new(),
            subsidy_queue: Vec::new(),
            powers: BTreeMap::new(),
            minors: BTreeMap::new(),
            leaders: BTreeMap::new(),
            areas: BTreeMap::new(),
            sea_zones: BTreeMap::new(),
            corps: BTreeMap::new(),
            fleets: BTreeMap::new(),
            diplomacy: BTreeMap::new(),
            adjacency: Vec::new(),
            coast_links: Vec::new(),
            sea_adjacency: Vec::new(),
        }
    }

    // ── adjacency lookups (5 cases) ─────────────────────────────────

    #[test]
    fn case_a01_adjacent_neighbours_in_line() {
        let s = line();
        let g = MapGraph::from_scenario(&s);
        assert!(g.adjacent(&AreaId::from("AREA_A"), &AreaId::from("AREA_B")));
        assert!(g.adjacent(&AreaId::from("AREA_B"), &AreaId::from("AREA_A")));
    }

    #[test]
    fn case_a02_non_neighbours_in_line() {
        let s = line();
        let g = MapGraph::from_scenario(&s);
        assert!(!g.adjacent(&AreaId::from("AREA_A"), &AreaId::from("AREA_C")));
    }

    #[test]
    fn case_a03_unknown_area_not_adjacent() {
        let s = line();
        let g = MapGraph::from_scenario(&s);
        assert!(!g.adjacent(&AreaId::from("AREA_X"), &AreaId::from("AREA_A")));
    }

    #[test]
    fn case_a04_diamond_neighbours_sorted() {
        let s = diamond();
        let g = MapGraph::from_scenario(&s);
        let nbrs = g.neighbours_of(&AreaId::from("AREA_A")).unwrap();
        let names: Vec<&str> = nbrs.iter().map(|a| a.as_str()).collect();
        assert_eq!(names, vec!["AREA_B", "AREA_C"]);
    }

    #[test]
    fn case_a05_area_count_matches_scenario() {
        let s = line();
        let g = MapGraph::from_scenario(&s);
        assert_eq!(g.area_count(), 5);
    }

    // ── hop distances (8 cases) ─────────────────────────────────────

    #[test]
    fn case_h01_zero_distance_to_self() {
        let s = line();
        let g = MapGraph::from_scenario(&s);
        assert_eq!(
            g.distance_hops(&AreaId::from("AREA_A"), &AreaId::from("AREA_A")),
            Some(0)
        );
    }

    #[test]
    fn case_h02_one_hop_in_line() {
        let s = line();
        let g = MapGraph::from_scenario(&s);
        assert_eq!(
            g.distance_hops(&AreaId::from("AREA_A"), &AreaId::from("AREA_B")),
            Some(1)
        );
    }

    #[test]
    fn case_h03_full_line() {
        let s = line();
        let g = MapGraph::from_scenario(&s);
        assert_eq!(
            g.distance_hops(&AreaId::from("AREA_A"), &AreaId::from("AREA_E")),
            Some(4)
        );
    }

    #[test]
    fn case_h04_unreachable_returns_none() {
        let mut s = line();
        s.areas
            .insert(AreaId::from("AREA_Z"), area("AREA_Z", "AUS"));
        // No edges to AREA_Z
        let g = MapGraph::from_scenario(&s);
        assert_eq!(
            g.distance_hops(&AreaId::from("AREA_A"), &AreaId::from("AREA_Z")),
            None
        );
    }

    #[test]
    fn case_h05_unknown_area_returns_none() {
        let s = line();
        let g = MapGraph::from_scenario(&s);
        assert_eq!(
            g.distance_hops(&AreaId::from("AREA_A"), &AreaId::from("AREA_X")),
            None
        );
    }

    #[test]
    fn case_h06_diamond_two_hops() {
        let s = diamond();
        let g = MapGraph::from_scenario(&s);
        assert_eq!(
            g.distance_hops(&AreaId::from("AREA_A"), &AreaId::from("AREA_D")),
            Some(2)
        );
    }

    #[test]
    fn case_h07_diamond_path_picks_lexicographic_first() {
        let s = diamond();
        let g = MapGraph::from_scenario(&s);
        let path = g
            .shortest_path_hops(&AreaId::from("AREA_A"), &AreaId::from("AREA_D"))
            .unwrap();
        assert_eq!(path[1].as_str(), "AREA_B");
    }

    #[test]
    fn case_h08_path_endpoints_correct() {
        let s = line();
        let g = MapGraph::from_scenario(&s);
        let path = g
            .shortest_path_hops(&AreaId::from("AREA_A"), &AreaId::from("AREA_E"))
            .unwrap();
        assert_eq!(path.first().unwrap().as_str(), "AREA_A");
        assert_eq!(path.last().unwrap().as_str(), "AREA_E");
    }

    // ── cost distances (5 cases) ────────────────────────────────────

    #[test]
    fn case_c01_unit_costs_match_hops_in_line() {
        let s = line();
        let g = MapGraph::from_scenario(&s);
        assert_eq!(
            g.shortest_path_cost(&AreaId::from("AREA_A"), &AreaId::from("AREA_E"))
                .map(|(c, _)| c),
            Some(4)
        );
    }

    #[test]
    fn case_c02_diamond_unit_cost_two() {
        let s = diamond();
        let g = MapGraph::from_scenario(&s);
        assert_eq!(
            g.shortest_path_cost(&AreaId::from("AREA_A"), &AreaId::from("AREA_D"))
                .map(|(c, _)| c),
            Some(2)
        );
    }

    #[test]
    fn case_c03_placeholder_branch_returns_none_when_used() {
        // The B-route is cost 2, the C-route via placeholder is also
        // length 2 but unknown; Dijkstra explores B first and returns
        // 2.  Placeholder is never *used*, so we still get a value.
        let s = diamond_with_placeholder_branch();
        let g = MapGraph::from_scenario(&s);
        let r = g.shortest_path_cost(&AreaId::from("AREA_A"), &AreaId::from("AREA_D"));
        assert_eq!(r.map(|(c, _)| c), Some(2));
    }

    #[test]
    fn case_c04_placeholder_required_returns_none() {
        // Two-node graph where the only edge is placeholder.
        let mut s = empty();
        s.areas.insert(AreaId::from("AREA_A"), area("a", "FRA"));
        s.areas.insert(AreaId::from("AREA_B"), area("b", "FRA"));
        s.adjacency.push(edge("AREA_A", "AREA_B", None));
        s.adjacency.push(edge("AREA_B", "AREA_A", None));
        let g = MapGraph::from_scenario(&s);
        let r = g.shortest_path_cost(&AreaId::from("AREA_A"), &AreaId::from("AREA_B"));
        assert_eq!(r, None);
    }

    #[test]
    fn case_c05_unequal_costs_pick_cheaper() {
        let mut s = empty();
        for n in ["AREA_A", "AREA_B", "AREA_C", "AREA_D"] {
            s.areas.insert(AreaId::from(n), area(n, "FRA"));
        }
        // A → B (cost 5) → D, vs A → C (cost 1) → D (cost 1)
        for (a, b, c) in [
            ("AREA_A", "AREA_B", 5),
            ("AREA_B", "AREA_D", 5),
            ("AREA_A", "AREA_C", 1),
            ("AREA_C", "AREA_D", 1),
        ] {
            s.adjacency.push(edge(a, b, Some(c)));
            s.adjacency.push(edge(b, a, Some(c)));
        }
        let g = MapGraph::from_scenario(&s);
        let (cost, path) = g
            .shortest_path_cost(&AreaId::from("AREA_A"), &AreaId::from("AREA_D"))
            .unwrap();
        assert_eq!(cost, 2);
        assert_eq!(path[1].as_str(), "AREA_C");
    }
}
