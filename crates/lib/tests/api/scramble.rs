use std::{
    collections::{HashMap, HashSet},
    hash::{BuildHasherDefault, DefaultHasher},
};

use rubiks_cube::*;

use crate::stats;

#[test]
fn random_scramble_on_3x3_is_uniform_and_all_are_reachable() {
    let mut corner_perm_buckets: HashMap<
        Corner,
        [usize; Corner::ALL.len()],
        BuildHasherDefault<DefaultHasher>,
    > = HashMap::from_iter(Corner::ALL.iter().map(|&c| (c, [0; Corner::ALL.len()])));
    let mut corner_orient_buckets: HashMap<Corner, [usize; 3], BuildHasherDefault<DefaultHasher>> =
        HashMap::from_iter(Corner::ALL.iter().map(|&c| (c, [0; 3])));

    let mut edge_perm_buckets: HashMap<
        Edge,
        [usize; Edge::ALL.len()],
        BuildHasherDefault<DefaultHasher>,
    > = HashMap::from_iter(Edge::ALL.iter().map(|&e| (e, [0; Edge::ALL.len()])));
    let mut edge_orient_buckets: HashMap<Edge, [usize; 2], BuildHasherDefault<DefaultHasher>> =
        HashMap::from_iter(Edge::ALL.iter().map(|&c| (c, [0; 2])));

    for seed in 0..100_000 {
        let random_cube = Cube3x3::scrambled_with_seed(seed);
        assert!(random_cube.is_reachable());
        for c in Corner::ALL {
            corner_perm_buckets.get_mut(&c).unwrap()[random_cube.corners().piece_at(c) as usize] +=
                1;
            corner_orient_buckets.get_mut(&c).unwrap()
                [random_cube.corners().orientation_at(c).value()] += 1;
        }
        for e in Edge::ALL {
            edge_perm_buckets.get_mut(&e).unwrap()[random_cube.edges().piece_at(e) as usize] += 1;
            edge_orient_buckets.get_mut(&e).unwrap()
                [random_cube.edges().orientation_at(e).value()] += 1;
        }
    }

    for v in corner_perm_buckets.values() {
        stats::assert_uniform(v, None);
    }
    for v in edge_perm_buckets.values() {
        stats::assert_uniform(v, None);
    }
    for v in corner_orient_buckets.values() {
        stats::assert_uniform(v, None);
    }
    for v in edge_orient_buckets.values() {
        stats::assert_uniform(v, None);
    }
}

#[test]
fn scramble_never_clashes_and_is_never_solved() {
    let mut scrambles = HashSet::new();
    for seed in 0..100_000 {
        let cube = Cube3x3::scrambled_with_seed(seed);
        assert!(!scrambles.contains(&cube));
        assert!(!cube.is_solved());
        scrambles.insert(cube);
    }
}
