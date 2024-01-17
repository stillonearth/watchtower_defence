use std::collections::HashSet;

use bevy::prelude::*;

pub const BOARD_SIZE: usize = 19;

#[derive(Resource)]
pub struct GameLogic {
    log: Vec<(GamePhase, Turn)>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Side {
    Black,
    White,
}

#[derive(Component)]
pub struct Stone {
    pub i: usize,
    pub j: usize,
    pub side: Side,
}

#[derive(Component, Debug)]
pub struct Draught {
    pub i: usize,
    pub j: usize,
    pub n: i8,
    pub side: Side,
}

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GamePhase {
    #[default]
    Initialize,
    PlaceWatchtower,
    // can't trigger PlaceWatchtower after PlaceWatchtower so there's a workaround
    TriggerPlaceWatchtower,
    PlaceGoPiece,
    MoveDraught,
    GameOver,
}

#[derive(Debug)]
pub struct GameStats {
    n_moves: usize,
    white_territory: usize,
    black_territory: usize,
    white_draughts: usize,
    black_draughts: usize,
}

#[derive(Resource, Default, Clone, Copy, Debug)]
pub enum Turn {
    Black,
    #[default]
    White,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CheckersMoveType {
    Regular,
    DraughtTakeOver,
    TowerTakeOver,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GoMoveType {
    Regular,
    StoneRemoval,
    TowerTakeOver,
}

impl GameLogic {
    pub fn new() -> Self {
        GameLogic { log: vec![] }
    }

    pub fn stats(
        &self,
        black_draughts: Vec<(usize, usize)>,
        white_draughts: Vec<(usize, usize)>,
        white_stones: Vec<(usize, usize)>,
        black_stones: Vec<(usize, usize)>,
    ) -> GameStats {
        let n_moves = self.log.len() - 2;
        let (white_regions, _) = self.legal_go_moves(
            Turn::White,
            black_draughts.clone(),
            white_draughts.clone(),
            white_stones.clone(),
            black_stones.clone(),
            (0, 0),
            (0, 0),
        );

        let (black_regions, _) = self.legal_go_moves(
            Turn::Black,
            black_draughts.clone(),
            white_draughts.clone(),
            white_stones.clone(),
            black_stones.clone(),
            (0, 0),
            (0, 0),
        );

        let mut white_territory = 0;
        for set in white_regions.iter() {
            white_territory += set.len();
        }

        let mut black_territory = 0;
        for set in black_regions.iter() {
            black_territory += set.len();
        }

        GameStats {
            n_moves,
            white_territory,
            black_territory,
            white_draughts: white_draughts.len(),
            black_draughts: black_draughts.len(),
        }
    }

    pub fn log(&mut self, game_phase: GamePhase, turn: Turn) {
        self.log.push((game_phase, turn));
    }

    pub fn next_state(&self) -> (GamePhase, Turn) {
        let (game_phase, turn) = self.log.last().unwrap();
        match game_phase {
            GamePhase::PlaceWatchtower => match *turn {
                Turn::Black => (GamePhase::MoveDraught, Turn::White),
                Turn::White => (GamePhase::PlaceWatchtower, Turn::Black),
            },
            GamePhase::PlaceGoPiece => (
                GamePhase::MoveDraught,
                match turn {
                    Turn::Black => Turn::White,
                    Turn::White => Turn::Black,
                },
            ),
            GamePhase::MoveDraught => (GamePhase::PlaceGoPiece, *turn),
            _ => (GamePhase::PlaceWatchtower, *turn),
        }
    }

    pub fn expand_from(
        &self,
        start: (usize, usize),
        region: Vec<(usize, usize)>,
    ) -> Vec<(usize, usize)> {
        let mut visited_points: Vec<(usize, usize)> = vec![];
        let mut stack: Vec<(usize, usize)> = vec![start];

        let mut reached_top = false;
        let mut reached_bottom = false;
        let mut reached_left = false;
        let mut reached_right = false;

        let candidates: Vec<(i8, i8)> = vec![(-1, 0), (1, 0), (0, -1), (0, 1)];
        let mut region_ = region.clone();

        // this is a hack but i spent way too much time debugging this
        let mut hack_elements: Vec<(usize, usize)> = Vec::new();
        for i in 0..BOARD_SIZE {
            hack_elements.push((i, BOARD_SIZE - 1));
        }
        for i in 0..BOARD_SIZE {
            hack_elements.push((BOARD_SIZE - 1, i));
        }
        for e in hack_elements.clone() {
            region_.push(e);
        }
        // ---

        while let Some((i, j)) = stack.pop() {
            if visited_points.contains(&(i, j)) {
                continue;
            }

            region_.push((i, j));
            visited_points.push((i, j));

            for c in candidates.clone() {
                let candidate = (i as i8 + c.0, j as i8 + c.1);

                if candidate.0 < 0
                    || candidate.1 < 0
                    || candidate.0 > BOARD_SIZE as i8
                    || candidate.1 > BOARD_SIZE as i8
                {
                    continue;
                }

                let candidate = (candidate.0 as usize, candidate.1 as usize);

                if candidate.0 == 0 {
                    reached_left = true;
                }
                if candidate.0 == (BOARD_SIZE - 1) {
                    reached_right = true;
                }
                if candidate.1 == 0 {
                    reached_bottom = true;
                }
                if candidate.1 == (BOARD_SIZE - 1) {
                    reached_top = true;
                }

                if !region_.contains(&candidate) {
                    stack.push(candidate);
                }
            }
        }

        if !(reached_bottom && reached_left && reached_right && reached_top) {
            return region_
                .iter()
                .filter(|e| !hack_elements.clone().contains(e))
                .copied()
                .collect();
        }

        region
    }

    pub fn fill_region(&self, region: Vec<(usize, usize)>) -> (Vec<(usize, usize)>, bool) {
        let mut region = region.clone();
        let mut is_expanded = false;
        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                let expanded_region = self.expand_from((i, j), region.clone());

                if expanded_region.len() > region.len() {
                    region = expanded_region;
                    is_expanded = true;
                }
            }
        }

        (region, is_expanded)
    }

    pub fn find_region(
        &self,
        start: (usize, usize),
        our_stones: Vec<(usize, usize)>,
        visited: Vec<(usize, usize)>,
    ) -> (
        Option<Vec<(usize, usize)>>,
        Vec<(usize, usize)>,
        Vec<(usize, usize)>,
    ) {
        if !our_stones.contains(&start) {
            return (None, visited, vec![]);
        }

        let mut region: Vec<(usize, usize)> = vec![];
        let mut stack: Vec<(usize, usize)> = vec![start];
        let mut visited = visited.clone();

        while let Some((i, j)) = stack.pop() {
            if visited.contains(&(i, j)) {
                continue;
            }
            visited.push((i, j));
            region.push((i, j));

            // up
            let up = (i, j + 1);
            if up.1 < BOARD_SIZE && our_stones.contains(&up) && !region.contains(&up) {
                stack.push(up);
            }

            // down
            if j >= 1 {
                let down = (i, j - 1);
                if our_stones.contains(&down) && !region.contains(&down) {
                    stack.push(down);
                }
            }

            // left
            if i >= 1 {
                let left = (i - 1, j);
                if our_stones.contains(&left) && !region.contains(&left) {
                    stack.push(left);
                }
            }

            // right
            let right = (i + 1, j);
            if right.0 < BOARD_SIZE && our_stones.contains(&right) && !region.contains(&right) {
                stack.push(right);
            }

            // up-right
            if i >= 1 {
                let up_right = (i + 1, j + 1);
                if up_right.1 < BOARD_SIZE
                    && up_right.0 < BOARD_SIZE
                    && our_stones.contains(&up_right)
                    && !region.contains(&up_right)
                {
                    stack.push(up_right);
                }
            }
            // up-left
            if i >= 1 {
                let up_left = (i - 1, j + 1);
                if up_left.1 < BOARD_SIZE
                    && our_stones.contains(&up_left)
                    && !region.contains(&up_left)
                {
                    stack.push(up_left);
                }
            }
            // down-right
            if j >= 1 {
                let down_right = (i + 1, j - 1);
                if down_right.0 < BOARD_SIZE
                    && our_stones.contains(&down_right)
                    && !region.contains(&down_right)
                {
                    stack.push(down_right);
                }
            }
            // down-left
            if j >= 1 && i >= 1 {
                let down_left = (i - 1, j - 1);
                if our_stones.contains(&down_left) && !region.contains(&down_left) {
                    stack.push(down_left);
                }
            }
        }

        if region.len() < 3 {
            return (None, visited, vec![]);
        }

        // fill gaps within region

        let (filled_region, _is_expanded) = self.fill_region(region.clone());
        // if !is_expanded && region.len() > 4 {
        //     return (None, visited);
        // }

        // remove duplicates
        let filled_region: HashSet<(usize, usize)> = filled_region
            .into_iter()
            .collect::<Vec<(usize, usize)>>()
            .into_iter()
            .collect();
        // convert to vector
        let mut filled_region: Vec<(usize, usize)> = filled_region.into_iter().collect();

        let stone_annihilate_region = filled_region.clone();

        // clean-up
        let stone_exists = |set: Vec<(usize, usize)>, (i, j): (usize, usize)| -> bool {
            set.iter()
                .filter(|stone| stone.0 == i && stone.1 == j)
                .count()
                != 0
        };

        let mut clean_region: Vec<(usize, usize)> = Vec::new();

        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                let stone = (i, j);

                if !stone_exists(filled_region.clone(), stone) {
                    continue;
                }

                if i == 0 || i == (BOARD_SIZE - 1) {
                    clean_region.push(stone);
                    continue;
                }

                let next_stone = (i + 1, j);
                let prev_stone = (i - 1, j);

                let has_next_stone = stone_exists(filled_region.clone(), next_stone);
                let has_prev_stone = stone_exists(filled_region.clone(), prev_stone);
                if has_next_stone || has_prev_stone {
                    clean_region.push(stone);
                }
            }
        }

        filled_region = clean_region.clone();
        let mut clean_region: Vec<(usize, usize)> = Vec::new();

        for j in 0..BOARD_SIZE {
            for i in 0..BOARD_SIZE {
                let stone = (i, j);

                if !stone_exists(filled_region.clone(), stone) {
                    continue;
                }

                if i == 0 || i == (BOARD_SIZE - 1) {
                    clean_region.push(stone);
                    continue;
                }

                let next_stone = (i, j + 1);
                let prev_stone = (i, j - 1);

                let has_next_stone = stone_exists(filled_region.clone(), next_stone);
                let has_prev_stone = stone_exists(filled_region.clone(), prev_stone);
                if has_next_stone || has_prev_stone {
                    clean_region.push(stone);
                }
            }
        }

        let floor_region: Vec<(i32, i32)> = clean_region
            .iter()
            .map(|(i, j)| (*i as f32 + 0.5, *j as f32 + 0.5))
            .map(|(i, j)| (i.ceil() as i32, j.ceil() as i32))
            .collect();

        let ceil_region: Vec<(i32, i32)> = clean_region
            .iter()
            .map(|(i, j)| (*i as f32 + 0.5, *j as f32 + 0.5))
            .map(|(i, j)| (i.floor() as i32, j.floor() as i32))
            .collect();

        let mut clean_region: Vec<(usize, usize)> = Vec::new();

        let stone_exists = |set: Vec<(i32, i32)>, (i, j): (usize, usize)| -> bool {
            set.iter()
                .filter(|stone| stone.0 == i as i32 && stone.1 == j as i32)
                .count()
                != 0
        };

        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                let stone = (i, j);

                if stone_exists(floor_region.clone(), stone)
                    && stone_exists(ceil_region.clone(), stone)
                {
                    clean_region.push(stone);
                }
            }
        }

        let stone_exists = |set: Vec<(usize, usize)>, (i, j): (usize, usize)| -> bool {
            set.iter()
                .filter(|stone| stone.0 == i && stone.1 == j)
                .count()
                != 0
        };

        let mut super_clean_region: Vec<(usize, usize)> = Vec::new();

        for i in 1..BOARD_SIZE {
            for j in 1..BOARD_SIZE {
                let stone = (i, j);
                let previous_stone = (i - 1, j);

                if !stone_exists(clean_region.clone(), previous_stone)
                    && stone_exists(clean_region.clone(), stone)
                {
                    let stone_1 = (i - 1, j - 1);
                    let stone_2 = (i - 1, j);

                    if stone_exists(our_stones.clone(), stone_1)
                        && stone_exists(our_stones.clone(), stone_2)
                    {
                        super_clean_region.push(stone);
                    }
                } else if stone_exists(clean_region.clone(), stone) {
                    super_clean_region.push(stone);
                }
            }
        }

        let clean_region = super_clean_region.clone();
        let mut super_clean_region: Vec<(usize, usize)> = Vec::new();

        for i in (0..BOARD_SIZE).rev() {
            for j in 0..BOARD_SIZE {
                let stone = (i, j);
                let previous_stone = (i + 1, j);

                if !stone_exists(clean_region.clone(), previous_stone)
                    && stone_exists(clean_region.clone(), stone)
                {
                    let stone_1 = (i, j - 1);
                    let stone_2 = (i, j);

                    if stone_exists(our_stones.clone(), stone_1)
                        && stone_exists(our_stones.clone(), stone_2)
                    {
                        super_clean_region.push(stone);
                    }
                } else if stone_exists(clean_region.clone(), stone) {
                    super_clean_region.push(stone);
                }
            }
        }

        (Some(super_clean_region), visited, stone_annihilate_region)
    }

    pub fn legal_go_moves(
        &self,
        turn: Turn,
        black_draughts: Vec<(usize, usize)>,
        white_draughts: Vec<(usize, usize)>,
        white_stones: Vec<(usize, usize)>,
        black_stones: Vec<(usize, usize)>,
        white_tower: (usize, usize),
        black_tower: (usize, usize),
    ) -> (Vec<Vec<(usize, usize)>>, Vec<(usize, usize)>) {
        let (
            (_our_draughts, our_stones, _our_tower),
            (_enemy_draughts, _enemy_stones, _enemy_tower),
        ) = match turn {
            Turn::Black => (
                (black_draughts, black_stones, black_tower),
                (white_draughts, white_stones, white_tower),
            ),
            Turn::White => (
                (white_draughts, white_stones, white_tower),
                (black_draughts, black_stones, black_tower),
            ),
        };

        let mut stone_removal_coords: Vec<(usize, usize)> = Vec::new();
        let mut convexes = Vec::new();
        let mut visited: Vec<(usize, usize)> = Vec::new();
        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                let (region, visited_, stone_annihite_region) =
                    self.find_region((i, j), our_stones.clone(), visited.clone());

                if region.is_some() {
                    convexes.push(region.unwrap());
                    stone_removal_coords.extend(stone_annihite_region);
                    visited = visited_;
                }
            }
        }

        stone_removal_coords = stone_removal_coords
            .into_iter()
            .collect::<HashSet<(usize, usize)>>()
            .into_iter()
            .collect();

        (convexes, stone_removal_coords)
    }

    pub fn legal_draught_moves(
        &self,
        turn: Turn,
        draught: (usize, usize),
        black_draughts: Vec<(usize, usize)>,
        white_draughts: Vec<(usize, usize)>,
        white_stones: Vec<(usize, usize)>,
        black_stones: Vec<(usize, usize)>,
        white_tower: (usize, usize),
        black_tower: (usize, usize),
    ) -> (
        Vec<(usize, usize)>,
        Vec<CheckersMoveType>,
        Vec<(usize, usize)>,
        Vec<(usize, usize)>,
    ) {
        let (
            (our_draughts, _our_stones, _our_tower),
            (enemy_draughts, _enemy_stones, _enemy_tower),
        ) = match turn {
            Turn::Black => (
                (black_draughts.clone(), black_stones.clone(), black_tower),
                (white_draughts.clone(), white_stones.clone(), white_tower),
            ),
            Turn::White => (
                (white_draughts.clone(), white_stones.clone(), white_tower),
                (black_draughts.clone(), black_stones.clone(), black_tower),
            ),
        };

        let mut legal_moves: Vec<(usize, usize)> = Vec::new();
        let mut takeovers: Vec<(usize, usize)> = Vec::new();
        let mut legal_movetypes: Vec<CheckersMoveType> = Vec::new();
        let mut occupied_squares = our_draughts.clone();
        occupied_squares.extend(enemy_draughts.clone());
        // occupied_squares.extend(our_stones);
        // occupied_squares.extend(enemy_stones);

        let (opposite_stones, stone_removals) = self.legal_go_moves(
            match turn {
                Turn::Black => Turn::White,
                Turn::White => Turn::Black,
            },
            black_draughts.clone(),
            white_draughts.clone(),
            white_stones.clone(),
            black_stones.clone(),
            white_tower,
            black_tower,
        );

        let enemy_stones = match turn {
            Turn::Black => white_stones.clone(),
            Turn::White => black_stones.clone(),
        };

        let mut opposite_occupied_squares: Vec<(usize, usize)> = Vec::new();
        for region in opposite_stones.iter() {
            opposite_occupied_squares.extend(region);
        }
        // opposite_occupied_squares.extend(enemy_stones);
        // keep only unique elements
        opposite_occupied_squares = opposite_occupied_squares
            .into_iter()
            .collect::<HashSet<(usize, usize)>>()
            .into_iter()
            .collect();

        // move up
        let up = (draught.0, draught.1 + 1);
        if up.1 < BOARD_SIZE
            && !occupied_squares.contains(&up)
            && !opposite_occupied_squares.contains(&up)
        {
            legal_moves.push(up);
            takeovers.push((0, 0));
            legal_movetypes.push(CheckersMoveType::Regular);
        }

        // move down
        if draught.1 >= 1 {
            let down = (draught.0, draught.1 - 1);
            if !occupied_squares.contains(&down) && !opposite_occupied_squares.contains(&down) {
                legal_moves.push(down);
                takeovers.push((0, 0));
                legal_movetypes.push(CheckersMoveType::Regular);
            }
        }

        // move left
        if draught.0 >= 1 {
            let left = (draught.0 - 1, draught.1);
            if !occupied_squares.contains(&left) && !opposite_occupied_squares.contains(&left) {
                legal_moves.push(left);
                takeovers.push((0, 0));
                legal_movetypes.push(CheckersMoveType::Regular);
            }
        }

        // move right
        let right = (draught.0 + 1, draught.1);
        if right.0 < BOARD_SIZE
            && !occupied_squares.contains(&right)
            && !opposite_occupied_squares.contains(&right)
        {
            legal_moves.push(right);
            takeovers.push((0, 0));
            legal_movetypes.push(CheckersMoveType::Regular);
        }

        // move up left
        if draught.0 >= 1 {
            let up_left = (draught.0 - 1, draught.1 + 1);
            if up_left.1 < BOARD_SIZE
                && !occupied_squares.contains(&up_left)
                && !opposite_occupied_squares.contains(&up_left)
            {
                legal_moves.push(up_left);
                takeovers.push((0, 0));
                legal_movetypes.push(CheckersMoveType::Regular);
            }
        }

        // move up right
        let up_right = (draught.0 + 1, draught.1 + 1);
        if up_right.0 < BOARD_SIZE
            && up_right.1 < BOARD_SIZE
            && !occupied_squares.contains(&up_right)
            && !opposite_occupied_squares.contains(&up_right)
        {
            legal_moves.push(up_right);
            takeovers.push((0, 0));
            legal_movetypes.push(CheckersMoveType::Regular);
        }

        // move down left
        if draught.0 >= 1 && draught.1 >= 1 {
            let down_left = (draught.0 - 1, draught.1 - 1);
            if !occupied_squares.contains(&down_left)
                && !opposite_occupied_squares.contains(&down_left)
            {
                legal_moves.push(down_left);
                takeovers.push((0, 0));
                legal_movetypes.push(CheckersMoveType::Regular);
            }
        }

        // move down right
        if draught.1 >= 1 {
            let down_right = (draught.0 + 1, draught.1 - 1);
            if down_right.0 < BOARD_SIZE
                && !occupied_squares.contains(&down_right)
                && !opposite_occupied_squares.contains(&down_right)
            {
                legal_moves.push(down_right);
                takeovers.push((0, 0));
                legal_movetypes.push(CheckersMoveType::Regular);
            }
        }

        // now, takeovers
        // up
        let up = (draught.0, draught.1 + 2);
        let up_takeover = (draught.0, draught.1 + 1);
        if up.1 < BOARD_SIZE
            && enemy_draughts.contains(&up_takeover)
            && !occupied_squares.contains(&up)
            && !opposite_occupied_squares.contains(&up)
        {
            legal_moves.push(up);
            takeovers.push(up_takeover);
            legal_movetypes.push(CheckersMoveType::DraughtTakeOver);
        }

        // down
        if draught.1 >= 2 {
            let down = (draught.0, draught.1 - 2);
            let down_takeover = (draught.0, draught.1 - 1);
            if enemy_draughts.contains(&down_takeover)
                && !occupied_squares.contains(&down)
                && !opposite_occupied_squares.contains(&down)
            {
                legal_moves.push(down);
                takeovers.push(down_takeover);
                legal_movetypes.push(CheckersMoveType::DraughtTakeOver);
            }
        }

        // left
        if draught.0 >= 2 {
            let left = (draught.0 - 2, draught.1);
            let left_takeover = (draught.0 - 1, draught.1);
            if enemy_draughts.contains(&left_takeover)
                && !occupied_squares.contains(&left)
                && !opposite_occupied_squares.contains(&left)
            {
                {
                    legal_moves.push(left);
                    takeovers.push(left_takeover);
                    legal_movetypes.push(CheckersMoveType::DraughtTakeOver);
                }
            }
        }

        // right
        let right = (draught.0 + 2, draught.1);
        let right_takeover: (usize, usize) = (draught.0 + 1, draught.1);
        if right.0 < BOARD_SIZE
            && enemy_draughts.contains(&right_takeover)
            && !occupied_squares.contains(&right)
            && !opposite_occupied_squares.contains(&right)
        {
            legal_moves.push(right);
            takeovers.push(right_takeover);
            legal_movetypes.push(CheckersMoveType::DraughtTakeOver);
        }

        // up-right
        let up_right = (draught.0 + 2, draught.1 + 2);
        let up_right_takeover = (draught.0 + 1, draught.1 + 1);
        if up_right.0 < BOARD_SIZE
            && up_right.1 < BOARD_SIZE
            && enemy_draughts.contains(&up_right_takeover)
            && !occupied_squares.contains(&up_right)
            && !opposite_occupied_squares.contains(&up_right)
            && !opposite_occupied_squares.contains(&(up_right.0, up_right.1))
        {
            legal_moves.push(up_right);
            takeovers.push(up_right_takeover);
            legal_movetypes.push(CheckersMoveType::DraughtTakeOver);
        }

        // up-left
        if draught.0 >= 2 {
            let up_left = (draught.0 - 2, draught.1 + 2);
            let up_left_takeover = (draught.0 - 1, draught.1 + 1);
            if up_left.1 < BOARD_SIZE
                && enemy_draughts.contains(&up_left_takeover)
                && !occupied_squares.contains(&up_left)
                && !opposite_occupied_squares.contains(&up_left)
            {
                legal_moves.push(up_left);
                takeovers.push(up_left_takeover);
                legal_movetypes.push(CheckersMoveType::DraughtTakeOver);
            }
        }

        // down-right
        if draught.1 >= 2 {
            let down_right = (draught.0 + 2, draught.1 - 2);
            let down_right_takeover = (draught.0 + 1, draught.1 - 1);
            if down_right.0 < BOARD_SIZE
                && enemy_draughts.contains(&down_right_takeover)
                && !occupied_squares.contains(&down_right)
                && !opposite_occupied_squares.contains(&down_right)
            {
                legal_moves.push(down_right);
                takeovers.push(down_right_takeover);
                legal_movetypes.push(CheckersMoveType::DraughtTakeOver);
            }
        }

        // down-left
        if draught.0 >= 2 && draught.1 >= 2 {
            let down_left = (draught.0 - 2, draught.1 - 2);
            let down_left_takeover = (draught.0 - 1, draught.1 - 1);
            if enemy_draughts.contains(&down_left_takeover)
                && !occupied_squares.contains(&down_left)
                && !opposite_occupied_squares.contains(&down_left)
            {
                legal_moves.push(down_left);
                takeovers.push(down_left_takeover);
                legal_movetypes.push(CheckersMoveType::DraughtTakeOver);
            }
        }

        let is_two_above = enemy_stones
            .iter()
            .position(|(i, j)| {
                let (i, j) = (*i, *j);
                i == draught.0 && j == draught.1 - 1
            })
            .is_some()
            && enemy_stones
                .iter()
                .position(|(i, j)| {
                    let (i, j) = (*i, *j);
                    i == draught.0 - 1 && j == draught.1 - 1
                })
                .is_some();
        // remove lower-right
        if is_two_above {
            let index = legal_moves.iter().position(|(i, j)| {
                let (i, j) = (*i, *j);
                i == draught.0 + 1 && j == draught.1 - 1
            });
            if index.is_some() {
                legal_moves.remove(index.unwrap());
                takeovers.remove(index.unwrap());
                legal_movetypes.remove(index.unwrap());
            }

            let index = legal_moves.iter().position(|(i, j)| {
                let (i, j) = (*i, *j);
                i == draught.0 && j == draught.1 - 1
            });
            if index.is_some() {
                legal_moves.remove(index.unwrap());
                takeovers.remove(index.unwrap());
                legal_movetypes.remove(index.unwrap());
            }

            let index = legal_moves.iter().position(|(i, j)| {
                let (i, j) = (*i, *j);
                i == draught.0 - 1 && j == draught.1 - 1
            });
            if index.is_some() {
                legal_moves.remove(index.unwrap());
                takeovers.remove(index.unwrap());
                legal_movetypes.remove(index.unwrap());
            }
        }

        let is_two_below = enemy_stones
            .iter()
            .position(|(i, j)| {
                let (i, j) = (*i, *j);
                i == draught.0 && j == draught.1
            })
            .is_some()
            && enemy_stones
                .iter()
                .position(|(i, j)| {
                    let (i, j) = (*i, *j);
                    i == draught.0 - 1 && j == draught.1
                })
                .is_some();
        // remove lower-right
        if is_two_below {
            let index = legal_moves.iter().position(|(i, j)| {
                let (i, j) = (*i, *j);
                i == draught.0 - 1 && j == draught.1 + 1
            });
            if index.is_some() {
                legal_moves.remove(index.unwrap());
                takeovers.remove(index.unwrap());
                legal_movetypes.remove(index.unwrap());
            }

            let index = legal_moves.iter().position(|(i, j)| {
                let (i, j) = (*i, *j);
                i == draught.0 && j == draught.1 + 1
            });
            if index.is_some() {
                legal_moves.remove(index.unwrap());
                takeovers.remove(index.unwrap());
                legal_movetypes.remove(index.unwrap());
            }

            let index = legal_moves.iter().position(|(i, j)| {
                let (i, j) = (*i, *j);
                i == draught.0 + 1 && j == draught.1 + 1
            });
            if index.is_some() {
                legal_moves.remove(index.unwrap());
                takeovers.remove(index.unwrap());
                legal_movetypes.remove(index.unwrap());
            }
        }

        let is_two_left = enemy_stones
            .iter()
            .position(|(i, j)| {
                let (i, j) = (*i, *j);
                i == draught.0 - 1 && j == draught.1 - 1
            })
            .is_some()
            && enemy_stones
                .iter()
                .position(|(i, j)| {
                    let (i, j) = (*i, *j);
                    i == draught.0 - 1 && j == draught.1
                })
                .is_some();
        // remove lower-right
        if is_two_left {
            let index = legal_moves.iter().position(|(i, j)| {
                let (i, j) = (*i, *j);
                i == draught.0 - 1 && j == draught.1 - 1
            });
            if index.is_some() {
                legal_moves.remove(index.unwrap());
                takeovers.remove(index.unwrap());
                legal_movetypes.remove(index.unwrap());
            }

            let index = legal_moves.iter().position(|(i, j)| {
                let (i, j) = (*i, *j);
                i == draught.0 - 1 && j == draught.1
            });
            if index.is_some() {
                legal_moves.remove(index.unwrap());
                takeovers.remove(index.unwrap());
                legal_movetypes.remove(index.unwrap());
            }

            let index = legal_moves.iter().position(|(i, j)| {
                let (i, j) = (*i, *j);
                i == draught.0 - 1 && j == draught.1 + 1
            });
            if index.is_some() {
                legal_moves.remove(index.unwrap());
                takeovers.remove(index.unwrap());
                legal_movetypes.remove(index.unwrap());
            }
        }

        let is_two_right = enemy_stones
            .iter()
            .position(|(i, j)| {
                let (i, j) = (*i, *j);
                i == draught.0 && j == draught.1
            })
            .is_some()
            && enemy_stones
                .iter()
                .position(|(i, j)| {
                    let (i, j) = (*i, *j);
                    i == draught.0 && j == draught.1 - 1
                })
                .is_some();
        // remove lower-right
        if is_two_right {
            println!("is_two_right");

            let index = legal_moves.iter().position(|(i, j)| {
                let (i, j) = (*i, *j);
                i == draught.0 + 1 && j == draught.1 - 1
            });
            if index.is_some() {
                legal_moves.remove(index.unwrap());
                takeovers.remove(index.unwrap());
                legal_movetypes.remove(index.unwrap());
            }

            let index = legal_moves.iter().position(|(i, j)| {
                let (i, j) = (*i, *j);
                i == draught.0 + 1 && j == draught.1
            });
            if index.is_some() {
                legal_moves.remove(index.unwrap());
                takeovers.remove(index.unwrap());
                legal_movetypes.remove(index.unwrap());
            }

            let index = legal_moves.iter().position(|(i, j)| {
                let (i, j) = (*i, *j);
                i == draught.0 + 1 && j == draught.1 + 1
            });
            if index.is_some() {
                legal_moves.remove(index.unwrap());
                takeovers.remove(index.unwrap());
                legal_movetypes.remove(index.unwrap());
            }
        }

        let is_one_above_right = enemy_stones
            .iter()
            .position(|(i, j)| {
                let (i, j) = (*i, *j);
                i == draught.0 && (j + 1) == draught.1
            })
            .is_some();
        // remove lower-right
        if is_one_above_right {
            let index = legal_moves.iter().position(|(i, j)| {
                let (i, j) = (*i, *j);
                i - 1 == draught.0 && j + 1 == draught.1
            });
            if index.is_some() {
                legal_moves.remove(index.unwrap());
                takeovers.remove(index.unwrap());
                legal_movetypes.remove(index.unwrap());
            }
        }

        let is_one_above_left = enemy_stones
            .iter()
            .position(|(i, j)| {
                let (i, j) = (*i, *j);
                (i + 1) == draught.0 && (j + 1) == draught.1
            })
            .is_some();
        // remove lower-right
        if is_one_above_left {
            let index = legal_moves.iter().position(|(i, j)| {
                let (i, j) = (*i, *j);
                i == draught.0 - 1 && j == draught.1 - 1
            });
            if index.is_some() {
                legal_moves.remove(index.unwrap());
                takeovers.remove(index.unwrap());
                legal_movetypes.remove(index.unwrap());
            }
        }

        let is_one_below_left = enemy_stones
            .iter()
            .position(|(i, j)| {
                let (i, j) = (*i, *j);
                i == draught.0 - 1 && j == draught.1
            })
            .is_some();
        // remove lower-right
        if is_one_below_left {
            println!("is_one_below_left");

            let index = legal_moves.iter().position(|(i, j)| {
                let (i, j) = (*i, *j);
                i == draught.0 - 1 && j == draught.1 + 1
            });
            if index.is_some() {
                legal_moves.remove(index.unwrap());
                takeovers.remove(index.unwrap());
                legal_movetypes.remove(index.unwrap());
            }
        }

        let is_one_below_right = enemy_stones
            .iter()
            .position(|(i, j)| {
                let (i, j) = (*i, *j);
                i == draught.0 && j == draught.1
            })
            .is_some();
        // remove lower-right
        if is_one_below_right {
            println!("is_one_below_right");

            let index = legal_moves.iter().position(|(i, j)| {
                let (i, j) = (*i, *j);
                i == draught.0 + 1 && j == draught.1 + 1
            });
            if index.is_some() {
                legal_moves.remove(index.unwrap());
                takeovers.remove(index.unwrap());
                legal_movetypes.remove(index.unwrap());
            }
        }

        // let is_stone_on_draught_place = opposite_occupied_squares.iter().position(|(i, j)| {
        //     let (i, j) = (*i, *j);
        //     i == draught.0 && j == draught.1
        // });
        // if is_stone_on_draught_place.is_some() {
        //     let illegal_moves: Vec<(usize, usize)> = vec![(1, 1)];
        //     for im in illegal_moves.iter() {
        //         let index = legal_moves.iter().position(|(i, j)| {
        //             let (i, j) = (*i, *j);
        //             i == im.0 && j == im.1
        //         });
        //         if index.is_some() {
        //             legal_moves.remove(index.unwrap());
        //             takeovers.remove(index.unwrap());
        //             legal_movetypes.remove(index.unwrap());
        //         }
        //     }
        // }

        (legal_moves, legal_movetypes, takeovers, stone_removals)
    }
}
