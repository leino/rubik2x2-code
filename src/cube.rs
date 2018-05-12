use std;

#[derive(Debug, Clone)]
pub struct Cube {
    pub transforms: [[[Transform; 2]; 2]; 2]
}

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub entries: [[i32; 3]; 3],
}


impl Transform {
    pub fn identity() -> Transform {
        let mut entries = [[0; 3]; 3];
        for i in 0..3 {
            entries[i][i] = 1;
        }
        Transform {entries: entries}
    }

    pub fn inverse(&self) -> Transform {
        // We know that the transform is orthogonal, so the inverse is just the transpose.
        let mut transposed_entries = [[0; 3]; 3];
        for row_idx in 0..3 {
            for column_idx in 0..3 {
                transposed_entries[row_idx][column_idx] = self.entries[column_idx][row_idx];
            }
        }
        Transform{entries: transposed_entries}
    }
    fn sequence(&self, next: Transform) -> Transform {
        let mut e = [[0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    e[i][j] += next.entries[i][k] * self.entries[k][j];
                }
            }
        }
        Transform{entries: e}
    }

    pub fn apply(&self, x: &[i32; 3]) -> [i32; 3] {
        assert!(x[0].abs() <= 1);
        assert!(x[1].abs() <= 1);
        assert!(x[2].abs() <= 1);        
        let mut y = [0; 3];
        for row_idx in 0..3 {
            for column_idx in 0..3 {
                y[row_idx] += self.entries[row_idx][column_idx] * x[column_idx];
            }
        }
        return y;
    }
}

fn index_position(index: &[i32; 3]) -> [i32; 3]
{
    [2*index[0] - 1, 2*index[1] - 1, 2*index[2] - 1]
}

fn position_index(position: &[i32; 3]) -> [i32; 3]
{
    assert!(in_cube(position));
    [(position[0] + 1)/2, (position[1] + 1)/2, (position[2] + 1)/2]
}

fn cubicle_face_normals(cubicle_position: &[i32; 3]) -> [[i32; 3]; 3]
{
    return
        [
            [cubicle_position[0], 0, 0],
            [0, cubicle_position[1], 0],
            [0, 0, cubicle_position[2]],
        ];
}

fn vector_direction_index(vector: &[i32; 3]) -> usize{
    for i in 0..3 {
        if vector[i] != 0
        {
            return i;
        }
    }
    panic!();
}

impl Cube {

    pub fn positions_orientations(&self) -> ([u8; 8], [u8; 8])
    {
        let mut ps = [0u8; 8];
        let mut os = [0u8; 8];        
        for position_idx in 0..8 {
            let cubicle_index = [(position_idx >> 0) & 1, (position_idx >> 1) & 1, (position_idx >> 2) & 1];
            let cubicle_position = index_position(&cubicle_index);
            let cubicle_face_normals = cubicle_face_normals(&cubicle_position);
            let transform = &self.transforms[cubicle_index[0] as usize]
                [cubicle_index[1] as usize][cubicle_index[2] as usize];
            let inverse_transform = transform.inverse();
            let solved_normals = [
                inverse_transform.apply(&cubicle_face_normals[0]),
                inverse_transform.apply(&cubicle_face_normals[1]),
                inverse_transform.apply(&cubicle_face_normals[2]),
            ];
            let solved_position = [
                solved_normals[0][0] + solved_normals[1][0] + solved_normals[2][0],
                solved_normals[0][1] + solved_normals[1][1] + solved_normals[2][1],
                solved_normals[0][2] + solved_normals[1][2] + solved_normals[2][2],
            ];
            let solved_position_index = position_index(&solved_position);
            let solved_position_idx =
                (solved_position_index[0] << 0) | (solved_position_index[1] << 1) | (solved_position_index[2] << 2);
            ps[position_idx as usize] = solved_position_idx as u8;
            os[position_idx as usize] =
                [[0, 2, 1], [0, 1, 2]]
                [(solved_position_idx.count_ones() & 1) as usize]
                [vector_direction_index(&solved_normals[0])];
        }
        return (ps, os);
    }
    
    pub fn transform(&self, position: [i32; 3]) -> &Transform {
        assert!(in_cube(&position));
        let (i, j, k)=
            ((1 + position[0]) >> 1, (1 + position[1]) >> 1, (1 + position[2]) >> 1);
        &self.transforms[i as usize][j as usize][k as usize]
    }
    fn sequence(&self, m: Move) -> Cube {
        let mut transformed_cube = (*self).clone();
        let move_transform = m.transform();
        let side = m.side();
        for start_i in 0..2 {
            for start_j in 0..2 {
                for start_k in 0..2 {
                    let start_position: [i32; 3] = [2*start_i - 1, 2*start_j - 1, 2*start_k - 1];
                    assert!(start_position[0].abs() == 1);
                    assert!(start_position[1].abs() == 1);
                    assert!(start_position[2].abs() == 1);
                    if inner_product(&normal(side), &start_position) > 0 {
                        let &start_transform = self.transform(start_position);
                        let end_position = move_transform.apply(&start_position);
                        assert!(end_position[0].abs() == 1);
                        assert!(end_position[1].abs() == 1);
                        assert!(end_position[2].abs() == 1);
                        let (end_i, end_j, end_k) =
                            ((end_position[0] + 1) >> 1,
                             (end_position[1] + 1) >> 1,
                             (end_position[2] + 1) >> 1);
                        transformed_cube.transforms[end_i as usize][end_j as usize][end_k as usize] =
                            start_transform.sequence(move_transform);
                    }
                }
            }
        }
        return transformed_cube;
    }

    pub fn sequence_moves(&self, moves: std::slice::Iter<Move>) -> Cube {
        let mut transformed_cube = (*self).clone();
        for m in moves {
            transformed_cube = transformed_cube.sequence(*m);
        }
        return transformed_cube;
    }
}

#[derive(Debug)]
#[derive(Copy)]
#[derive(Clone)]
#[derive(PartialEq)]
pub enum Side {
    L, R,
    D, U,
    B, F
}

impl Side {
    pub fn from(idx: i32) -> Side {
        use self::Side::*;
        match idx {
            0 => L,
            1 => R,
            2 => D,
            3 => U,
            4 => B,
            5 => F,
            _ => panic!(),
        }
    }
    pub fn deserialize(serialization: &str) -> Option<Side> {
        use self::Side::*;
        match serialization {
            "left" => Some(L),
            "right" => Some(R),
            "down" => Some(D),
            "up" => Some(U),
            "back" => Some(B),
            "front" => Some(F),
            _ => None,
        }
    }
    pub fn serialization(side: Side) -> &'static str {
        use self::Side::*;
        match side {
            L => "left",
            R => "right",
            D => "down",
            U => "up",
            B => "back",
            F => "front",
        }
    }
}


#[derive(Clone, Copy, Debug)]
pub enum Move {
    L1, L2, L3,
    R1, R2, R3,
    D1, D2, D3,
    U1, U2, U3,
    B1, B2, B3,
    F1, F2, F3,
}

impl Move {
    fn side(&self) -> Side {
        Side::from( ((*self) as i32) / 3 )
    }
    fn transform(&self) -> Transform {
        use self::Move::*;
        let e = 
            match *self {
                L1 => [[1,  0, 0,],
                       [0,  0, 1,],
                       [0, -1, 0,],],
                L2 => [[1,  0,  0,],
                       [0, -1,  0,],
                       [0,  0, -1,],],
                L3 => [[1, 0,  0,],
                       [0, 0, -1,],
                       [0, 1,  0,],],
                R1 => [[1, 0,  0,],
                       [0, 0, -1,],
                       [0, 1,  0,],],                
                R2 => [[1,  0,  0,],
                       [0, -1,  0,],
                       [0,  0, -1,],],
                R3 => [[1,  0, 0,],
                       [0,  0, 1,],
                       [0, -1, 0,],],
                D1 => [[0, 0, -1,],
                       [0, 1,  0,],
                       [1, 0,  0,],],
                D2 => [[-1, 0,  0,],
                       [ 0, 1,  0,],
                       [ 0, 0, -1,],],
                D3 => [[ 0, 0, 1,],
                       [ 0, 1, 0,],
                       [-1, 0, 0,],],
                U1 => [[ 0, 0, 1,],
                       [ 0, 1, 0,],
                       [-1, 0, 0,],],
                U2 => [[-1, 0,  0,],
                       [ 0, 1,  0,],
                       [ 0, 0, -1,],],
                U3 => [[0, 0, -1,],
                       [0, 1,  0,],
                       [1, 0,  0,],],
                B1 => [[ 0, 1, 0,],
                       [-1, 0, 0,],
                       [ 0, 0, 1,],],
                B2 => [[-1,  0, 0,],
                       [ 0, -1, 0,],
                       [ 0,  0, 1,],],
                B3 => [[0, -1, 0,],
                       [1,  0, 0,],
                       [0,  0, 1,],],
                F1 => [[0, -1, 0,],
                       [1,  0, 0,],
                       [0,  0, 1,],],                                
                F2 => [[-1,  0, 0,],
                       [ 0, -1, 0,],
                       [ 0,  0, 1,],],
                F3 => [[ 0, 1, 0,],
                       [-1, 0, 0,],
                       [ 0, 0, 1,],],
            };
        Transform{entries: e}
    }
}


#[derive(Debug)]
pub struct Macro {
    pub moves: Vec<Move>,
}

impl Macro {
    pub fn move_iter<'a>(&'a self) -> std::slice::Iter<'a, Move> {
        self.moves.iter()
    }
    pub fn identity() -> Macro {
        Macro {
            moves: vec![]
        }
    }
}

impl From<Vec<Move>> for Macro {
    fn from(v: Vec<Move>) -> Macro {
        Macro {moves: v}
    }
}

impl From<[Move; 4]> for Macro {
    fn from(v: [Move; 4]) -> Macro {
        Macro {moves: v.to_vec()}
    }
}

impl From<[Move; 13]> for Macro {
    fn from(v: [Move; 13]) -> Macro {
        Macro {moves: v.to_vec()}
    }
}

impl From<[Move; 11]> for Macro {
    fn from(v: [Move; 11]) -> Macro {
        Macro {moves: v.to_vec()}
    }
}

impl<'a> From<&'a [Move]> for Macro {
    fn from(s: &[Move]) -> Macro {
        Macro {moves: s.to_vec()}
    }
}

impl From<Move> for Macro {
    fn from(m: Move) -> Macro {
        Macro {moves: vec![m]}
    }
}

impl From<u8> for Move {
    fn from(i: u8) -> Move {
        use self::Move::*;
        match i {
            00 => L1, 01 => L2, 02 => L3,
            03 => R1, 04 => R2, 05 => R3,
            06 => D1, 07 => D2, 08 => D3,
            09 => U1, 10 => U2, 11 => U3,
            12 => B1, 13 => B2, 14 => B3,
            15 => F1, 16 => F2, 17 => F3,
            _ => panic!(),
        }
    }
}
impl From<i32> for Move {
    fn from(i: i32) -> Move {
        use self::Move::*;
        match i {
            00 => L1, 01 => L2, 02 => L3,
            03 => R1, 04 => R2, 05 => R3,
            06 => D1, 07 => D2, 08 => D3,
            09 => U1, 10 => U2, 11 => U3,
            12 => B1, 13 => B2, 14 => B3,
            15 => F1, 16 => F2, 17 => F3,
            _ => panic!(),
        }
    }
}

pub fn normal(side: Side) -> [i32; 3] {
    use self::Side::*;
    match side {
        L => [-1, 0, 0],
        R => [ 1, 0, 0],
        D => [0, -1, 0],
        U => [0,  1, 0],
        B => [0, 0, -1],
        F => [0, 0,  1],
    }
}

pub fn normal_side(normal: [i32; 3]) -> Side {
    use self::Side::*;
    assert!(is_face_normal(normal));
    if normal[0] == -1 {L}
    else if normal[0] ==  1 {R}
    else if normal[1] == -1 {D}
    else if normal[1] ==  1 {U}
    else if normal[2] == -1 {B}
    else if normal[2] ==  1 {F}
    else {panic!()}
}

fn in_cube(position: &[i32; 3]) -> bool {
    for coordinate_idx in 0..3 {
        if position[coordinate_idx].abs() > 1 {
            return false;
        }
    }
    return true;
}

fn inner_product(a: &[i32; 3], b: &[i32; 3]) -> i32 {
    let mut p = 0;
    for i in 0..3 {
        p += a[i]*b[i];
    }
    return p;
}

fn is_face_normal(normal: [i32; 3]) -> bool {
    let mut zero_count = 0;
    for coordinate_idx in 0..3 {
        let coordinate = normal[coordinate_idx];
        if coordinate == 0 {
            zero_count += 1;
        } else if coordinate.abs() > 1 {
            return false;
        }
    }
    return zero_count == 2;
}
