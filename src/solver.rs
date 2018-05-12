use permutations;
use cube;

pub fn solution(cube: &cube::Cube) -> Vec<cube::Move>
{
    let (initial_positions, initial_orientations) = cube.positions_orientations();
    
    let mp = positions::solve_macro(permutations::inverse(initial_positions));
    let mut orientations = initial_orientations.clone();
    for m in mp.move_iter() {
        orientations::apply(&mut orientations, m);
    }    
    let mo = orientations::solve_macro(orientations);

    let mut solution_moves: Vec<cube::Move> = Vec::new();
    solution_moves.extend(mp.move_iter());
    solution_moves.extend(mo.move_iter());

    return solution_moves;
}

pub fn test()
{
    {
        use solver::positions::*;
        test_swaps();
        test_solve([0, 1, 2, 3, 4, 5, 6, 7]);
        test_solve([0, 1, 2, 4, 3, 5, 6, 7]);
        test_solve([0, 7, 2, 4, 3, 5, 6, 1]);
        test_solve([6, 7, 5, 2, 3, 4, 0, 1]);
    }

    {
        use solver::orientations::*;
        test_apply();
        test_twists();
        test_solve([0, 1, 2, 0, 1, 2, 0, 0]);
        test_solve([1, 1, 1, 2, 2, 2, 1, 2]);
        test_solve([1, 1, 1, 0, 2, 2, 0, 2]);
        test_solve([0, 0, 0, 0, 0, 0, 0, 0]);
        test_solve([2, 0, 0, 0, 0, 0, 0, 1]);
        test_solve([2, 0, 0, 1, 1, 1, 0, 1]);
        test_solve([2, 2, 2, 2, 2, 2, 2, 1]);
    }    
}

mod positions {    
    use cube::Move;
    use cube::Macro;
    use permutations;
    
    // pos = pos * m
    pub fn apply(positions: &mut [u8; 8], m: &Move){
        let f = (*m as u8) / 3;
        let c = ((*m as u8) % 3) + 1;
        let d = f >> 1;
        let s = f & 1;
        for p in 0..8 {
            let mut k = positions[p as usize];
            if ((k >> d) & 1) == s {
                let idx = |i| (d + i + 1) % 3;
                for _ in 0 .. c {
                    k = (0 .. 2).fold((s << d)^(1 << idx(s^1)), |m, j| m^(((k >> idx(j)) & 1) << idx(j^1)));
                }
            }
            positions[p as usize] = k;
        }
    }

    // TODO: possible to save two moves here?
    fn swap_opposite_moves(i: u8, j: u8) -> [Move; 13]{
        assert!((i ^ j).count_ones() == 3);
        // Method of derivation: start with a sequence of moves that flips i=0 and j=7.
        // To generalize, just let T be a transform that inverts (flips) the X Y and Z coordinates such that
        // T(i) = 0.
        // If there is an even number of inversion, then that's all. If there is an odd number of inversions, then
        // also flip clockwise moves for counter-clockiwe moves. (Space has been turned inside out.)
        let f = (i.count_ones() & 1) as u8; // Number of inversions
        let m = |p, d, c: u8| Move::from(2u8*3u8*d + 3u8*((p >> d) & 1) + ((2u8 + f + (c << f)) % 3u8));
        [
            m(i, 0, 2), m(j, 2, 3), m(j, 0, 3), m(i, 1, 3),
            m(j, 0, 1), m(i, 1, 1), m(j, 2, 1), m(j, 0, 3),
            m(i, 1, 3), m(j, 0, 1), m(i, 1, 1), m(j, 2, 3),
            m(i, 0, 2),
        ]
    }

    fn swap_edge_moves(i: u8, j: u8) -> [Move; 11]{
        assert!((i ^ j).count_ones() == 1);
        let f = ((i & !(i^j)).count_ones() & 1) as u8; // Number of inversions
        let d = |k| (k + ((i^j) >> 1)) % 3;
        let flip = |k: u8| (((i & (!(i^j))) >> d(k)) & 1u8) as u8;
        let m = |k, s, c: u8| Move::from(2u8*3u8*d(k) + 3u8*(s^flip(k)) + ((2u8 + f + (c << f)) % 3u8));
        [m(2, 0, 3), m(0, 1, 3), m(1, 1, 3),
         m(0, 1, 1), m(1, 1, 1), m(2, 0, 1), m(0, 1, 3),
         m(1, 1, 3), m(0, 1, 1), m(1, 1, 1), m(2, 0, 3)]
    }

    fn swap_diag_moves(i: u8, j: u8) -> [Move; 13]{
        assert!((i ^ j).count_ones() == 2);
        let d = |k| (((i^j^7) >> 1) + 1 + k) % 3;
        let f = (i.count_ones() as u8) & 1u8;
        let flip = |k| ((i) >> d(k)) & 1;
        let m = |k, s, c: u8| Move::from(2u8*3u8*d(k) + 3u8*(s^flip(k)) + ((2u8 + f + (c << f)) % 3u8));
        [m(0, 1, 3), m(2, 0, 3), m(0, 1, 3), m(1, 1, 3),
         m(0, 1, 1), m(1, 1, 1), m(2, 0, 1), m(0, 1, 3),
         m(1, 1, 3), m(0, 1, 1), m(1, 1, 1), m(2, 0, 3),
         m(0, 1, 1)]
    }


    pub fn solve_macro(p: [u8; 8]) -> Macro {
        let mut m = Macro::identity();
        for (i, j) in permutations::TranspositionIterator::new(permutations::inverse(p)) {
            match (i^j).count_ones() {
                3 => {
                    m.moves.extend_from_slice( &swap_opposite_moves(i as u8, j as u8) );
                },
                2 => {
                    m.moves.extend_from_slice( &swap_diag_moves(i as u8, j as u8) );
                },
                1 => {
                    m.moves.extend_from_slice( &swap_edge_moves(i as u8, j as u8) );
                }
                _ => panic!(),
            }        
        }
        m
    }

    pub fn test_solve(p: [u8; 8]) {
        let mut q = p;
        for m in solve_macro(p).move_iter() {
            apply(&mut q, m);
        }
        assert!(q == permutations::IDENTITY);
    }

    pub fn test_swap(i: u8, j: u8, m: Macro) {
        let mut p = permutations::IDENTITY;
        for m in m.move_iter() {
            apply(&mut p, m);
        }
        for k in 0 .. 8u8 {
            if k == i {
                assert!(p[k as usize] == j);
            } else if k == j {
                assert!(p[k as usize] == i);
            } else {
                assert!(p[k as usize] == k);
            }
        }
    }
    
    
    pub fn test_swaps() {
        use solver::positions::*;
        for i in 0 .. 8 {
            for j in 0 .. 8 {
                if i != j {
                    match (i^j as u8).count_ones() {
                        3 => {
                            test_swap(i, j, Macro::from(swap_opposite_moves(i, j)));
                        },
                        2 => {
                            test_swap(i, j, Macro::from(swap_diag_moves(i, j)));
                        },
                        1 => {
                            test_swap(i, j, Macro::from(swap_edge_moves(i, j)));
                        }
                        _ => panic!(),
                    }
                }
            }
        }
    }
    
}

mod orientations {
    use solver::positions;
    use cube::Move;
    use cube::Macro;
    use permutations;
    
    pub fn identity() -> [u8; 8] {
        [0; 8]
    }

    pub fn apply(o: &mut [u8; 8], m: &Move) {

        let f = (*m as u8) / 3;
        let c = ((*m as u8) - 3*f) + 1;
        assert!(c >= 1);
        assert!(c <= 3);
        let changes = {
            match f {
                0 => [0, 0, 0, 0, 0, 0, 0, 0],
                1 => [0, 0, 0, 0, 0, 0, 0, 0],
                2 => [2, 1, 0, 0, 1, 2, 0, 0],
                3 => [0, 0, 1, 2, 0, 0, 2, 1],
                4 => [1, 2, 2, 1, 0, 0, 0, 0],
                5 => [0, 0, 0, 0, 2, 1, 1, 2],
                _ => panic!(),
            }
        };
        let mb = Move::from(f*3);
        for _ in 0..c {
            let mut p = permutations::IDENTITY;
            positions::apply(&mut p, &mb);
            permutations::apply(o, &p);
            for i in 0..8 {
                o[i] = (o[i] + changes[i]) % 3;
            }
        }
    }
    
    pub fn twist_edge_moves(i: u8, j: u8) -> [Move; 12] {
        assert!((i^j).count_ones() == 1);
        let d0 = (i^j) >> 1;
        let d = |k: u8| (d0 + 1u8 + k) % 3;
        let s = |k| (i >> d(k)) & 1;
        let f = (i.count_ones() as u8) & 1;
        let m = |k, c| Move::from(2*3*d(k^f) + 3*s(k^f) + c - 1);
        [m(0, 1), m(1, 3), m(0, 1), m(1, 3), m(0, 1), m(1, 3), m(0, 3), m(1, 1), m(0, 3), m(1, 1), m(0, 3), m(1, 1)]
    }

    pub fn twist_diag_moves(i: u8, j: u8) -> [Move; 14] {
        assert!((i^j).count_ones() == 2);
        let d0 = (i^j^7) >> 1;
        let d = |k| (k + d0) % 3;
        let f = (i.count_ones() as u8) & 1;        
        let fs = |k| ((if f == 1 {j} else {i}) >> d(k)) & 1;
        let m = |k: u8, s: u8, c: u8| Move::from(2*3*d(k) + 3*(s^fs(k)) + ((2u8 + f + (c << f)) % 3u8) );
        [m(2, 1, 1), m(0, 0, 1), m(1, 0, 3), m(0, 0, 1), m(1, 0, 3), m(0, 0, 1), m(1, 0, 3), m(0, 0, 3),
         m(1, 0, 1), m(0, 0, 3), m(1, 0, 1), m(0, 0, 3), m(1, 0, 1), m(2, 1, 3)]
    }

    pub fn twist_opposite_moves(i: u8, j: u8) -> [Move; 14] {
        assert!((i^j).count_ones() == 3);
        let f = (i.count_ones() & 1) as u8;
        let fd = |d| (i >> d) & 1;
        let a = |d: u8, f: u8| d^f^(f & (d>>1));
        let m = |d: u8, s: u8, c: u8| Move::from(2*3*(a(d, f)) + 3*(s^fd(a(d, f))) + c-1);
        [
            m(2, 1, 2),
            m(0, 0, 1), m(1, 0, 3), m(0, 0, 1), m(1, 0, 3), m(0, 0, 1), m(1, 0, 3),
            m(0, 0, 3), m(1, 0, 1), m(0, 0, 3), m(1, 0, 1), m(0, 0, 3), m(1, 0, 1),
            m(2, 1, 2)
        ]
    }
    
    pub fn test_apply() {
        for s in 0..6 {
            use cube::Move::*;
            let m = Move::from(3*s);
            let mut o = identity();
            apply(&mut o, &m);
            match m {
                L1 => assert!(o == [0; 8]),
                R1 => assert!(o == [0; 8]),
                D1 => assert!(o == [2, 1, 0, 0, 1, 2, 0, 0]),
                U1 => assert!(o == [0, 0, 1, 2, 0, 0, 2, 1]),
                B1 => assert!(o == [1, 2, 2, 1, 0, 0, 0, 0]),
                F1 => assert!(o == [0, 0, 0, 0, 2, 1, 1, 2]),
                _ => panic!(),
            }
            let mi = Move::from(3*s + 2);
            apply(&mut o, &mi);
            assert!(o == [0; 8]);
        }
    }
    
    pub fn test_twist(i: u8, j: u8) {
        let mut twist_macro = Macro::identity();
        use solver::orientations; 
        let distinct_side_count = (i^j).count_ones();
        match distinct_side_count {
            1 => {
                twist_macro.moves.extend_from_slice(&orientations::twist_edge_moves(i, j));
            },
            2 => {
                twist_macro.moves.extend_from_slice(&orientations::twist_diag_moves(i, j));
            },
            3 => {
                twist_macro.moves.extend_from_slice(&orientations::twist_opposite_moves(i, j));
            },
            _ => panic!(),
        }

        {
            let mut o = orientations::identity();
            let mut p = permutations::IDENTITY;
            for m in twist_macro.move_iter() {
                orientations::apply(&mut o, m);
                positions::apply(&mut p, m);
            }
            
            let mut correct: [u8; 8] = [0; 8];
            for k in 0..8 {
                if k == i {
                    correct[k as usize] = 1;
                } else if k == j {
                    correct[k as usize] = 2;
                }
            }

            assert!(o == correct);
            assert!(p == permutations::IDENTITY);
        }
    }
    
    pub fn test_twists() {
        for i in 0 .. 8 {
            for j in 0 .. 8 {
                if i != j {
                    test_twist(i, j);
                }
            }
        }
    }

    pub fn solve_macro(o: [u8; 8]) -> Macro {
        let mut m = Macro::identity();
        for k in 1..8u8 {
            use solver::orientations;
            let t = o[k as usize];
            if t != 0 {
                let (i, j) = if t == 1 {(0, k)} else {(k, 0)};
                match (i^j).count_ones() {
                    1 => {
                        m.moves.extend_from_slice(
                            &orientations::twist_edge_moves(i, j)
                        );
                    },
                    2 => {
                        m.moves.extend_from_slice(
                            &orientations::twist_diag_moves(i, j)
                        );
                    },
                    3 => {
                        m.moves.extend_from_slice(
                            &orientations::twist_opposite_moves(i, j)
                        );
                    },
                    _ => panic!(),
                }
            }
        }
        m
    }

    pub fn test_solve(o: [u8; 8]) {
        let mut s = o;
        let mac = solve_macro(o);
        for m in mac.move_iter() {
            apply(&mut s, m);
        }
        assert!(s == identity());
    }
}
