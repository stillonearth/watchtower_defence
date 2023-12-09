use std::collections::HashSet;
use std::ops::Index;
use std::time::Duration;

use crate::loading::{MaterialAssets, MeshAssets};
use crate::GameState;
use bevy::prelude::*;
use bevy::transform::commands;
use bevy_mod_picking::prelude::*;
use bevy_tweening::*;

pub struct WatchtowerPlugin;

#[derive(Component, Clone, Copy)]
pub struct Watchtower {
    pub i: usize,
    pub j: usize,
    pub side: Side,
}

#[derive(Component)]
struct BoardCamera;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Side {
    Black,
    White,
}

#[derive(Component)]
struct Square {
    pub i: usize,
    pub j: usize,
    pub side: Side,
}

#[derive(Component)]
struct Circle {
    pub i: usize,
    pub j: usize,
}

#[derive(Component)]
struct Stone {
    pub i: usize,
    pub j: usize,
    pub side: Side,
}

#[derive(Component, Debug)]
struct Draught {
    pub i: usize,
    pub j: usize,
    pub n: i8,
    pub side: Side,
}

const BOARD_SIZE: usize = 19;

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GamePhase {
    #[default]
    Initialize,
    PlaceWatchtower,
    // can't trigger PlaceWatchtower after PlaceWatchtower so there's a workaround
    TriggerPlaceWatchtower,
    PlaceGoPiece,
    MoveDraught,
}

#[derive(Resource, Default, Clone, Copy, Debug)]
enum Turn {
    Black,
    #[default]
    White,
}

#[derive(Resource)]
struct GameLogic {
    log: Vec<(GamePhase, Turn)>,
}

#[derive(Resource)]
struct SelectedDraught {
    n: Option<i8>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CheckersMoveType {
    Regular,
    DraughtTakeOver,
    TowerTakeOver,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GoMoveType {
    Regular,
    StoneRemoval,
    TowerTakeOver,
}

impl GameLogic {
    fn new() -> Self {
        GameLogic { log: vec![] }
    }

    fn log(&mut self, game_phase: GamePhase, turn: Turn) {
        self.log.push((game_phase, turn));
    }

    fn next_state(&self) -> (GamePhase, Turn) {
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

    fn expand_from(
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

    fn fill_region(&self, region: Vec<(usize, usize)>) -> (Vec<(usize, usize)>, bool) {
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

    fn find_region(
        &self,
        start: (usize, usize),
        our_stones: Vec<(usize, usize)>,
        visited: Vec<(usize, usize)>,
    ) -> (Option<Vec<(usize, usize)>>, Vec<(usize, usize)>) {
        if !our_stones.contains(&start) {
            return (None, visited);
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

        if region.len() < 2 {
            return (None, visited);
        }

        // fill gaps within region

        let (filled_region, is_expanded) = self.fill_region(region.clone());
        if !is_expanded {
            return (None, visited);
        }

        // remove duplicates
        let filled_region: HashSet<(usize, usize)> = filled_region
            .into_iter()
            .collect::<Vec<(usize, usize)>>()
            .into_iter()
            .collect();
        // convert to vector
        let filled_region: Vec<(usize, usize)> = filled_region.into_iter().collect();

        (Some(filled_region), visited)
    }

    fn legal_go_moves(
        &self,
        turn: Turn,
        black_draughts: Vec<(usize, usize)>,
        white_draughts: Vec<(usize, usize)>,
        white_stones: Vec<(usize, usize)>,
        black_stones: Vec<(usize, usize)>,
        white_tower: (usize, usize),
        black_tower: (usize, usize),
    ) -> Vec<Vec<(usize, usize)>> {
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

        let mut visited: Vec<(usize, usize)> = Vec::new();
        let mut convexes = Vec::new();
        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                let (region, visited_) =
                    self.find_region((i, j), our_stones.clone(), visited.clone());

                if region.is_some() {
                    convexes.push(region.unwrap());
                    visited = visited_;
                }
            }
        }

        convexes
    }

    fn legal_draught_moves(
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

        let opposite_stones = self.legal_go_moves(
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

        (legal_moves, legal_movetypes, takeovers)
    }
}

fn check_game_termination(
    // mut q_draughts: Query<(Entity, &mut Draught)>,
    // mut q_stones: Query<(Entity, &Stone)>,
    mut q_watchtowers: Query<(Entity, &Watchtower)>,
    // mut selected_draught: ResMut<SelectedDraught>,
    // mut q_nuke_draught_button: Query<(Entity, &mut Visibility, &ButtonNukeDraught)>,
    mut text_query: Query<(&mut Text, &mut Visibility, &NextMoveText)>,
    game_phase: Res<State<GamePhase>>,
) {
    let game_phase = game_phase.into_inner();
    let game_phase = game_phase.get();
    if (*game_phase == GamePhase::Initialize
        || *game_phase == GamePhase::PlaceWatchtower
        || *game_phase == GamePhase::TriggerPlaceWatchtower)
    {
        return;
    }

    let n_white_towers = q_watchtowers
        .iter()
        .filter(|(_, w)| w.side == Side::White)
        .count();

    let n_black_towers = q_watchtowers
        .iter()
        .filter(|(_, w)| w.side == Side::Black)
        .count();

    let mut gameover_text = "";

    if n_white_towers == 0 {
        gameover_text = "Black Won!"
    }

    if n_black_towers == 0 {
        gameover_text = "White Won!"
    }

    for (mut text, mut v, _tag) in text_query.iter_mut() {
        text.sections[0].value = gameover_text.into();
        *v = Visibility::Visible;
    }
}

#[allow(clippy::type_complexity)]
fn nuke_draught_button_system(
    mut commands: Commands,
    mut interaction_query: Query<
        (&ButtonNukeDraught, &Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut q_draughts: Query<(Entity, &mut Draught)>,
    mut q_stones: Query<(Entity, &Stone)>,
    mut q_watchtowers: Query<(Entity, &Watchtower)>,
    mut selected_draught: ResMut<SelectedDraught>,
    mut q_nuke_draught_button: Query<(Entity, &mut Visibility, &ButtonNukeDraught)>,
    turn: Res<Turn>,
) {
    let side = match *turn.into_inner() {
        Turn::Black => Side::Black,
        Turn::White => Side::White,
    };

    for (_, interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                let draught = q_draughts
                    .iter()
                    .find(|d| d.1.n == selected_draught.n.unwrap() && d.1.side == side)
                    .unwrap()
                    .1;

                let ccs_to_remove: Vec<(i32, i32)> = vec![
                    (draught.i as i32 - 1, draught.j as i32 - 1),
                    (draught.i as i32 - 1, draught.j as i32),
                    (draught.i as i32 - 1, draught.j as i32 + 1),
                    (draught.i as i32, draught.j as i32 - 1),
                    (draught.i as i32, draught.j as i32),
                    (draught.i as i32, draught.j as i32 + 1),
                    (draught.i as i32 + 1, draught.j as i32 - 1),
                    (draught.i as i32 + 1, draught.j as i32),
                    (draught.i as i32 + 1, draught.j as i32 + 1),
                ];

                let ccs_to_remove: Vec<_> = ccs_to_remove
                    .iter()
                    .filter(|(i, j)| *i >= 0 && *j >= 0)
                    .collect();

                for (e, d) in q_draughts.iter() {
                    for cc in ccs_to_remove.iter() {
                        if d.i as i32 == cc.0.clone() && d.j as i32 == cc.1.clone() {
                            commands.entity(e).despawn_recursive();
                        }
                    }
                }

                for (e, d) in q_stones.iter() {
                    for cc in ccs_to_remove.iter() {
                        if d.i as i32 == cc.0.clone() && d.j as i32 == cc.1.clone() {
                            commands.entity(e).despawn_recursive();
                        }
                    }
                }

                for (e, d) in q_watchtowers.iter() {
                    for cc in ccs_to_remove.iter() {
                        if d.i as i32 == cc.0.clone() && d.j as i32 == cc.1.clone() {
                            commands.entity(e).despawn_recursive();
                        }
                    }
                }

                selected_draught.n = None;

                for (_, mut v, _) in q_nuke_draught_button.iter_mut() {
                    *v = Visibility::Hidden;
                }

                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn init_game_over_text(mut commands: Commands) {
    let text = Text::from_section(
        "GAME OVER",
        TextStyle {
            font_size: 50.0,
            color: Color::WHITE,
            ..default()
        },
    )
    .with_alignment(TextAlignment::Left);

    // root node
    commands
        .spawn((NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.),
                bottom: Val::Percent(50.),
                ..Default::default()
            },
            ..Default::default()
        },))
        .with_children(|parent| {
            parent
                .spawn(TextBundle {
                    text,
                    ..Default::default()
                })
                .insert((NextMoveText, Visibility::Hidden));
        })
        .insert(Pickable::IGNORE);
}

impl Plugin for WatchtowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GamePhase>();
        app.init_resource::<Turn>();
        app.add_plugins(TweeningPlugin)
            .add_plugins(DefaultPickingPlugins)
            .add_systems(Startup, init_buttons)
            .add_systems(Update, nuke_draught_button_system)
            .add_systems(Update, init_game_over_text)
            .add_systems(Update, check_game_termination)
            .add_systems(OnEnter(GameState::Watchtower), (spawn_camera, spawn_board))
            .add_systems(OnEnter(GamePhase::PlaceWatchtower), spawn_watchtower)
            .add_systems(
                OnEnter(GamePhase::TriggerPlaceWatchtower),
                |mut game_phase: ResMut<NextState<GamePhase>>| {
                    game_phase.set(GamePhase::PlaceWatchtower);
                },
            )
            .add_systems(OnEnter(GamePhase::PlaceGoPiece), spawn_go_piece)
            .add_systems(OnEnter(GamePhase::MoveDraught), prepare_move_draught)
            .add_systems(
                Update,
                place_watchtower.run_if(in_state(GamePhase::PlaceWatchtower)),
            )
            .add_systems(
                Update,
                place_stone.run_if(in_state(GamePhase::PlaceGoPiece)),
            )
            .add_systems(
                Update,
                (select_draught, move_draught).run_if(in_state(GamePhase::MoveDraught)),
            )
            .add_event::<EventHoverSquare>()
            .add_event::<EventClickWatchtower>()
            .add_event::<EventClickSquare>()
            .add_event::<EventClickCircle>()
            .add_event::<EventClickDraught>()
            .insert_resource(GameLogic::new())
            .insert_resource(SelectedDraught { n: None });
    }
}

// ---
// UI

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.35);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Component)]
struct NextMoveText;

#[derive(Component)]
struct ButtonNukeDraught;

#[derive(Component)]
struct GameOverText;

fn init_buttons(mut commands: Commands) {
    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(170.0),
                    height: Val::Px(65.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                background_color: NORMAL_BUTTON.into(),
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            ButtonNukeDraught,
            // Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    text: Text::from_section(
                        "Nuke the Draught",
                        TextStyle {
                            font_size: 30.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..Default::default()
                        },
                    ),
                    ..Default::default()
                },
                // Visibility::Hidden,
            ));
        })
        .insert(Pickable::IGNORE);
}

// ---

fn select_draught(
    mut er_click_draught: EventReader<EventClickDraught>,
    q_draughts: Query<(Entity, &mut Draught)>,
    mut selected_draught: ResMut<SelectedDraught>,
    mut q_nuke_draught_button: Query<(Entity, &mut Visibility, &ButtonNukeDraught)>,
    turn: Res<Turn>,
) {
    let side = match *turn.into_inner() {
        Turn::Black => Side::Black,
        Turn::White => Side::White,
    };

    for click in er_click_draught.read() {
        let draught = q_draughts.get_component::<Draught>(click.0).unwrap();

        if draught.side != side {
            return;
        }

        selected_draught.n = Some(draught.n);

        for (_, mut v, _) in q_nuke_draught_button.iter_mut() {
            *v = Visibility::Visible;
            break;
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TransformPositionWithYJumpLens {
    pub start: Vec3,
    pub end: Vec3,
}

impl Lens<Transform> for TransformPositionWithYJumpLens {
    fn lerp(&mut self, target: &mut Transform, ratio: f32) {
        let mut value = self.start + (self.end - self.start) * ratio;
        if ratio < 0.5 {
            value.y = ratio * 2.0 + 0.1;
        } else {
            value.y = (1.0 - ratio) * 2.0 + 0.1;
        }
        target.translation = value;
    }
}

fn move_draught(
    mut commands: Commands,
    mut er_click_square: EventReader<EventClickSquare>,
    mut q_draughts: Query<(Entity, &mut Transform, &mut Draught)>,
    mut q_stones: Query<&Stone>,
    mut q_watchtowers: Query<&Watchtower>,
    q_squares: Query<(Entity, &mut Transform, &mut Square), Without<Draught>>,
    mut selected_draught: ResMut<SelectedDraught>,
    mut turn: ResMut<Turn>,
    mut game_logic: ResMut<GameLogic>,
    mut game_phase: ResMut<NextState<GamePhase>>,
    materials: Res<MaterialAssets>,
    meshes: Res<MeshAssets>,

    mut q_nuke_draught_button: Query<(Entity, &mut Visibility, &ButtonNukeDraught)>,
) {
    if selected_draught.n.is_none() {
        return;
    }

    let turn_ = *turn;
    let side = match turn_ {
        Turn::Black => Side::Black,
        Turn::White => Side::White,
    };

    let black_draughts = q_draughts
        .iter_mut()
        .filter(|(_, _, draught)| draught.side == Side::Black)
        .map(|(_, _, draught)| (draught.i, draught.j))
        .collect::<Vec<(usize, usize)>>();

    let white_draughts = q_draughts
        .iter_mut()
        .filter(|(_, _, draught)| draught.side == Side::White)
        .map(|(_, _, draught)| (draught.i, draught.j))
        .collect::<Vec<(usize, usize)>>();

    let black_stones = q_stones
        .iter_mut()
        .filter(|go_piece| go_piece.side == Side::Black)
        .map(|go_piece| (go_piece.i, go_piece.j))
        .collect::<Vec<(usize, usize)>>();

    let white_stones = q_stones
        .iter_mut()
        .filter(|go_piece| go_piece.side == Side::White)
        .map(|go_piece| (go_piece.i, go_piece.j))
        .collect::<Vec<(usize, usize)>>();

    let white_watchtower = q_watchtowers
        .iter_mut()
        .find(|watchtower| watchtower.side == Side::White)
        .unwrap();
    let white_watchtower = (white_watchtower.i, white_watchtower.j);

    let black_watchtower = q_watchtowers
        .iter_mut()
        .find(|watchtower| watchtower.side == Side::Black)
        .unwrap();
    let black_watchtower = (black_watchtower.i, black_watchtower.j);

    for click in er_click_square.read() {
        let square = q_squares.get_component::<Square>(click.0).unwrap();

        let draught = q_draughts
            .iter()
            .find(|d| d.2.n == selected_draught.n.unwrap() && d.2.side == side)
            .unwrap();

        let draught_position = draught.1.translation;
        let draught_entity = draught.0;
        let draught = draught.2;

        let (possible_moves, possible_movetypes, takeovers) = game_logic.legal_draught_moves(
            turn_,
            (draught.i, draught.j),
            black_draughts,
            white_draughts,
            white_stones,
            black_stones,
            white_watchtower,
            black_watchtower,
        );

        if !possible_moves.contains(&(square.i, square.j)) {
            println!("Illegal move");
            return;
        }

        let possible_move_index = possible_moves
            .iter()
            .position(|(i, j)| *i == square.i && *j == square.j)
            .unwrap();
        let move_type = possible_movetypes[possible_move_index];

        if move_type == CheckersMoveType::DraughtTakeOver {
            let takeover = takeovers[possible_move_index];
            let enemy_draught = q_draughts
                .iter()
                .find(|d| d.2.i == takeover.0 && d.2.j == takeover.1)
                .unwrap();

            let enemy_draught_entity = enemy_draught.0;
            let enemy_draught = enemy_draught.2;
            let n_draughts = q_draughts.iter().filter(|d| d.2.side == side).count();

            commands.entity(enemy_draught_entity).despawn_recursive();

            let transform =
                Transform::from_xyz(enemy_draught.i as f32, 0.0, enemy_draught.j as f32)
                    .with_scale(Vec3::splat(0.1));
            let draught = Draught {
                i: enemy_draught.i,
                j: enemy_draught.j,
                n: (n_draughts + 1) as i8,
                side,
            };

            commands.spawn((
                PbrBundle {
                    mesh: meshes.checkers_piece.clone(),
                    transform,
                    material: match side {
                        Side::Black => materials.black.clone(),
                        _ => materials.white.clone(),
                    },
                    ..default()
                },
                Name::new("Draught"),
                draught,
                On::<Pointer<Click>>::send_event::<EventClickDraught>(),
            ));
        }

        let square_position = q_squares
            .get_component::<Transform>(click.0)
            .unwrap()
            .translation;

        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_millis(1000),
            TransformPositionWithYJumpLens {
                start: draught_position,
                end: square_position,
            },
        );

        commands.entity(draught_entity).insert((
            Animator::new(tween),
            Draught {
                i: square.i,
                j: square.j,
                ..*draught
            },
        ));

        selected_draught.n = None;

        for (_, mut v, _) in q_nuke_draught_button.iter_mut() {
            *v = Visibility::Hidden;
        }

        game_logic.log(GamePhase::MoveDraught, *turn);
        let (next_phase, next_turn) = game_logic.next_state();
        *turn = next_turn;
        game_phase.set(next_phase);

        return;
    }
}

fn prepare_move_draught(
    mut commands: Commands,
    mut q_circles: Query<(Entity, &mut Visibility, &Circle)>,
    mut q_draughts: Query<(Entity, &mut Draught)>,
    turn: ResMut<Turn>,
) {
    let turn = *turn;
    let side = match turn {
        Turn::Black => Side::Black,
        Turn::White => Side::White,
    };

    for (entity, mut visibility, _) in q_circles.iter_mut() {
        *visibility = Visibility::Hidden;
        commands.entity(entity).remove::<PickableBundle>();
    }

    for (entity, draught) in q_draughts.iter_mut() {
        if draught.side != side {
            commands.entity(entity).remove::<PickableBundle>();
        } else {
            commands.entity(entity).insert(PickableBundle::default());
        }
    }
}

fn spawn_watchtower(
    mut commands: Commands,
    meshes: Res<MeshAssets>,
    materials: Res<MaterialAssets>,
    turn: ResMut<Turn>,
    mut q_circles: Query<(Entity, &mut Visibility, &Circle)>,
) {
    for (entity, mut visibility, _) in q_circles.iter_mut() {
        *visibility = Visibility::Hidden;
        commands.entity(entity).remove::<PickableBundle>();
    }

    let center = (10.0, 10.0);

    let turn_ = *turn;
    let side = match turn_ {
        Turn::Black => Side::Black,
        Turn::White => Side::White,
    };

    let material = match side {
        Side::Black => materials.transparent_black.clone(),
        _ => materials.transparent_white.clone(),
    };

    let transform = Transform::from_xyz(center.0, 0.9, center.1)
        .with_scale(Vec3::splat(0.1))
        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));
    commands.spawn((
        PbrBundle {
            mesh: meshes.watchtower.clone(),
            material: material.clone(),
            transform,
            ..default()
        },
        // PickableBundle::default(),
        Name::new("Watchtower"),
        Watchtower {
            i: center.0 as usize,
            j: center.1 as usize,
            side,
        },
        On::<Pointer<Click>>::send_event::<EventClickSquare>(),
    ));

    for i in 0..9 {
        if i == 4 {
            continue;
        }

        let piece_position = match i {
            0 => (center.0 - 1.0, center.1 - 1.0),
            1 => (center.0 - 1.0, center.1),
            2 => (center.0 - 1.0, center.1 + 1.0),

            3 => (center.0, center.1 - 1.0),
            4 => (center.0, center.1),
            5 => (center.0, center.1 + 1.0),

            6 => (center.0 + 1.0, center.1 - 1.0),
            7 => (center.0 + 1.0, center.1),
            8 => (center.0 + 1.0, center.1 + 1.0),

            _ => (0.0, 0.0),
        };

        let transform = Transform::from_xyz(piece_position.0, 0.0, piece_position.1)
            .with_scale(Vec3::splat(0.1));
        let draught = Draught {
            i: piece_position.0 as usize,
            j: piece_position.1 as usize,
            n: i,
            side,
        };

        commands.spawn((
            PbrBundle {
                mesh: meshes.checkers_piece.clone(),
                transform,
                material: material.clone(),
                ..default()
            },
            Name::new("Draught"),
            draught,
            On::<Pointer<Click>>::send_event::<EventClickDraught>(),
        ));
    }
}

#[derive(Event)]
struct EventHoverSquare(Entity);

impl From<ListenerInput<Pointer<Over>>> for EventHoverSquare {
    fn from(event: ListenerInput<Pointer<Over>>) -> Self {
        EventHoverSquare(event.target)
    }
}

#[derive(Event)]
struct EventClickSquare(Entity);

impl From<ListenerInput<Pointer<Click>>> for EventClickSquare {
    fn from(event: ListenerInput<Pointer<Click>>) -> Self {
        EventClickSquare(event.target)
    }
}

#[derive(Event)]
struct EventClickDraught(Entity);

impl From<ListenerInput<Pointer<Click>>> for EventClickDraught {
    fn from(event: ListenerInput<Pointer<Click>>) -> Self {
        EventClickDraught(event.target)
    }
}

#[derive(Event)]
struct EventClickCircle(Entity);

impl From<ListenerInput<Pointer<Click>>> for EventClickCircle {
    fn from(event: ListenerInput<Pointer<Click>>) -> Self {
        EventClickCircle(event.target)
    }
}

#[derive(Event)]
struct EventClickWatchtower(Entity);

impl From<ListenerInput<Pointer<Click>>> for EventClickWatchtower {
    fn from(event: ListenerInput<Pointer<Click>>) -> Self {
        EventClickWatchtower(event.target)
    }
}

fn spawn_go_piece(
    mut commands: Commands,
    q_squares: Query<(Entity, &Square)>,
    mut q_circles: Query<(Entity, &mut Visibility, &Circle)>,
) {
    for (entity, _) in q_squares.iter() {
        commands.entity(entity).remove::<PickableBundle>();
    }

    for (_, mut visibility, _) in q_circles.iter_mut() {
        *visibility = Visibility::Visible;
    }
}

#[derive(Component)]
struct DebugSquare;

fn place_stone(
    mut commands: Commands,
    mut er_click_circle: EventReader<EventClickCircle>,
    q_circles: Query<(Entity, &mut Visibility, &Circle)>,
    meshes: Res<MeshAssets>,
    materials: Res<MaterialAssets>,
    mut turn: ResMut<Turn>,
    mut game_phase: ResMut<NextState<GamePhase>>,
    mut game_logic: ResMut<GameLogic>,
    mut q_draughts: Query<(Entity, &mut Transform, &mut Draught)>,
    mut q_stones: Query<(Entity, &Stone)>,
    mut q_watchtowers: Query<(Entity, &Watchtower)>,
    mut q_debug_squares: Query<Entity, With<DebugSquare>>,
) {
    let black_draughts = q_draughts
        .iter_mut()
        .filter(|(_, _, draught)| draught.side == Side::Black)
        .map(|(_, _, draught)| (draught.i, draught.j))
        .collect::<Vec<(usize, usize)>>();

    let white_draughts = q_draughts
        .iter_mut()
        .filter(|(_, _, draught)| draught.side == Side::White)
        .map(|(_, _, draught)| (draught.i, draught.j))
        .collect::<Vec<(usize, usize)>>();

    let mut black_stones = q_stones
        .iter_mut()
        .filter(|(_, go_piece)| go_piece.side == Side::Black)
        .map(|go_piece| (go_piece.1.i, go_piece.1.j))
        .collect::<Vec<(usize, usize)>>();

    let mut white_stones = q_stones
        .iter_mut()
        .filter(|(_, go_piece)| go_piece.side == Side::White)
        .map(|go_piece| (go_piece.1.i, go_piece.1.j))
        .collect::<Vec<(usize, usize)>>();

    let white_watchtower = q_watchtowers
        .iter_mut()
        .find(|(_, watchtower)| watchtower.side == Side::White)
        .unwrap();
    let white_watchtower = (white_watchtower.1.i, white_watchtower.1.j);

    let black_watchtower = q_watchtowers
        .iter_mut()
        .find(|(_, watchtower)| watchtower.side == Side::Black)
        .unwrap();
    let black_watchtower = (black_watchtower.1.i, black_watchtower.1.j);

    let turn_ = *turn;
    let side = match turn_ {
        Turn::Black => Side::Black,
        Turn::White => Side::White,
    };

    for er_click in er_click_circle.read() {
        let circle = q_circles.get_component::<Circle>(er_click.0).unwrap();
        commands.spawn((
            PbrBundle {
                mesh: meshes.go_piece.clone(),
                transform: Transform::from_translation(Vec3::new(
                    circle.i as f32 + 0.5,
                    0.0002,
                    circle.j as f32 + 0.5,
                ))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                material: match side {
                    Side::Black => materials.blue.clone(),
                    _ => materials.yellow.clone(),
                },
                ..default()
            },
            Name::new("Stone"),
            Stone {
                i: circle.i,
                j: circle.j,
                side,
            },
        ));

        match side {
            Side::Black => black_stones.push((circle.i, circle.j)),
            Side::White => white_stones.push((circle.i, circle.j)),
        }

        let convex_sets = game_logic.legal_go_moves(
            turn_,
            black_draughts.clone(),
            white_draughts.clone(),
            white_stones.clone(),
            black_stones.clone(),
            white_watchtower,
            black_watchtower,
        );

        // despawn debug squares
        for entity in q_debug_squares.iter_mut() {
            commands.entity(entity).despawn_recursive();
        }

        // remove opposite stones
        let enemy_stones = match side {
            Side::Black => white_stones.clone(),
            Side::White => black_stones.clone(),
        };

        let enemy_draughts = match side {
            Side::Black => white_draughts.clone(),
            Side::White => black_draughts.clone(),
        };

        for convex in convex_sets.iter() {
            for (i, j) in convex.iter() {
                for enemy_stone in enemy_stones.iter() {
                    if !(enemy_stone.0 == *i && enemy_stone.1 == *j) {
                        continue;
                    }

                    let stone_entity = q_stones
                        .iter()
                        .find(|(_, stone)| (stone.i == enemy_stone.0 && stone.j == enemy_stone.1));
                    if stone_entity.is_some() {
                        let stone_entity = stone_entity.unwrap().0;
                        commands.entity(stone_entity).despawn_recursive();
                    }
                }

                // takeover pieces
                for enemy_draught in enemy_draughts.iter() {
                    if !(enemy_draught.0 == *i && enemy_draught.1 == *j) {
                        continue;
                    }

                    let enemy_draught = q_draughts
                        .iter()
                        .find(|d| d.2.i == enemy_draught.0 && d.2.j == enemy_draught.1)
                        .unwrap();

                    let enemy_draught_entity = enemy_draught.0;
                    let enemy_draught = enemy_draught.2;
                    let n_draughts = q_draughts.iter().filter(|d| d.2.side == side).count();

                    commands.entity(enemy_draught_entity).despawn_recursive();

                    let transform =
                        Transform::from_xyz(enemy_draught.i as f32, 0.0, enemy_draught.j as f32)
                            .with_scale(Vec3::splat(0.1));
                    let draught = Draught {
                        i: enemy_draught.i,
                        j: enemy_draught.j,
                        n: (n_draughts + 1) as i8,
                        side,
                    };

                    commands.spawn((
                        PbrBundle {
                            mesh: meshes.checkers_piece.clone(),
                            transform,
                            material: match side {
                                Side::Black => materials.black.clone(),
                                _ => materials.white.clone(),
                            },
                            ..default()
                        },
                        Name::new("Draught"),
                        draught,
                        On::<Pointer<Click>>::send_event::<EventClickDraught>(),
                    ));
                }

                // spawn debug square
                // commands.spawn((
                //     PbrBundle {
                //         mesh: meshes.square_plane.clone(),
                //         material: materials.red.clone(),
                //         transform: Transform::from_translation(Vec3::new(
                //             *i as f32 + 0.5,
                //             0.0005,
                //             *j as f32 + 0.5,
                //         )),
                //         ..default()
                //     },
                //     Name::new("DebugSquare"),
                //     DebugSquare,
                // ));
            }
        }

        game_logic.log(GamePhase::PlaceGoPiece, *turn);
        let (next_phase, next_turn) = game_logic.next_state();
        *turn = next_turn;
        game_phase.set(next_phase);
        return;
    }
}

fn spawn_board(
    mut commands: Commands,
    materials: Res<MaterialAssets>,
    meshes: Res<MeshAssets>,
    mut game_phase: ResMut<NextState<GamePhase>>,
    turn: Res<Turn>,
) {
    let side = match *turn.into_inner() {
        Turn::Black => Side::Black,
        Turn::White => Side::White,
    };

    // spawn checkerboard
    for i in 0..BOARD_SIZE {
        for j in 0..BOARD_SIZE {
            let n = i * BOARD_SIZE + j;
            let material = match n % 2 {
                0 => materials.black.clone(),
                _ => materials.white.clone(),
            };

            commands.spawn((
                PbrBundle {
                    mesh: meshes.square_plane.clone(),
                    material,
                    transform: Transform::from_translation(Vec3::new(i as f32, 0.0, j as f32)),
                    ..default()
                },
                Square { i, j, side },
                Name::new("Square"),
                PickableBundle::default(),
                On::<Pointer<Over>>::send_event::<EventHoverSquare>(),
                On::<Pointer<Click>>::send_event::<EventClickSquare>(),
            ));
        }
    }

    // spawn go pieces intersections
    for i in 0..BOARD_SIZE - 1 {
        for j in 0..BOARD_SIZE - 1 {
            commands.spawn((
                PbrBundle {
                    mesh: meshes.circle.clone(),
                    material: materials.red.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        i as f32 + 0.5,
                        0.0001,
                        j as f32 + 0.5,
                    ))
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                    visibility: Visibility::Hidden,
                    ..default()
                },
                Name::new("Circle"),
                Circle { i, j },
                PickableBundle::default(),
                On::<Pointer<Click>>::send_event::<EventClickCircle>(),
            ));
        }
    }

    game_phase.set(GamePhase::PlaceWatchtower);
}

fn spawn_camera(mut commands: Commands) {
    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 3000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(BOARD_SIZE as f32 / 2.0, 10.0, BOARD_SIZE as f32 / 2.0),
        ..Default::default()
    });

    let camera_transform = Transform::from_translation(Vec3::new(-5.0, 30.0, 20.0))
        .looking_at(Vec3::new(19.0 / 2.0, 0.0, 19.0 / 2.0), Vec3::Y);

    commands.spawn((
        Camera3dBundle {
            transform: camera_transform,
            ..default()
        },
        BoardCamera,
        Name::new("Camera"),
    ));
}

fn place_watchtower(
    mut commands: Commands,
    materials: Res<MaterialAssets>,
    mut er_hover_square: EventReader<EventHoverSquare>,
    mut er_click_square: EventReader<EventClickSquare>,
    _er_click_watchtower: EventReader<EventClickWatchtower>,
    q_squares: Query<Entity, &Square>,
    mut q_pieces: Query<(Entity, &mut Transform, &mut Draught)>,
    mut q_watchtower: Query<(Entity, &mut Transform, &mut Watchtower), Without<Draught>>,
    mut turn: ResMut<Turn>,
    mut game_phase: ResMut<NextState<GamePhase>>,
    mut game_logic: ResMut<GameLogic>,
    meshes: Res<MeshAssets>,
) {
    let turn_ = *turn;

    let side = match turn_ {
        Turn::Black => Side::Black,
        Turn::White => Side::White,
    };

    let mut stop = |center: (usize, usize)| -> Turn {
        let material = match side {
            Side::Black => materials.black.clone(),
            _ => materials.white.clone(),
        };

        for (entity, _, draught) in q_pieces.iter() {
            if draught.side != side {
                continue;
            }
            commands
                .entity(entity)
                .insert((PickableBundle::default(), material.clone()));
        }

        for (entity, _, watchtower) in q_watchtower.iter() {
            if watchtower.side != side {
                continue;
            }
            commands.entity(entity).insert(material.clone());
        }

        // place stones around watchtower
        let initial_stone_positions: Vec<(i32, i32)> = vec![
            (1, 1),
            (1, 0),
            (1, -1),
            (1, -2),
            (0, 1),
            (0, -2),
            (-1, 1),
            (-1, -2),
            (-2, 1),
            (-2, 0),
            (-2, -1),
            (-2, -2),
        ];

        for isp in initial_stone_positions {
            commands.spawn((
                PbrBundle {
                    mesh: meshes.go_piece.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        (center.0 as i32 + isp.0) as f32 + 0.5,
                        0.0002,
                        (center.1 as i32 + isp.1) as f32 + 0.5,
                    ))
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                    material: match side {
                        Side::Black => materials.blue.clone(),
                        _ => materials.yellow.clone(),
                    },
                    ..default()
                },
                Name::new("Stone"),
                Stone {
                    i: (center.0 as i32 + isp.0) as usize,
                    j: (center.1 as i32 + isp.1) as usize,
                    side,
                },
            ));
        }

        game_logic.log(GamePhase::PlaceWatchtower, turn_);
        let (next_phase, next_turn) = game_logic.next_state();
        if next_phase == GamePhase::PlaceWatchtower {
            game_phase.set(GamePhase::TriggerPlaceWatchtower);
        } else {
            game_phase.set(next_phase);
        }
        *turn = next_turn;

        next_turn
    };

    for click in er_click_square.read() {
        let watchtower = q_watchtower.get_component::<Watchtower>(click.0).unwrap();
        let center = (watchtower.i, watchtower.j);
        *turn = stop(center);
        return;
    }

    let opposite_side = match side {
        Side::Black => Side::White,
        _ => Side::Black,
    };

    let opposite_watchtower = q_watchtower
        .iter_mut()
        .find(|(_, _, watchtower)| watchtower.side == opposite_side);
    let mut opposite_watchtower_position = (-128.0, 128.0);
    if opposite_watchtower.is_some() {
        let opp_watchtower = opposite_watchtower.unwrap().2;
        opposite_watchtower_position = (opp_watchtower.i as f32, opp_watchtower.j as f32);
    }

    for hover in er_hover_square.read() {
        let square = q_squares.get_component::<Square>(hover.0).unwrap();
        let center = (square.i, square.j);

        for (_entity, mut transform, mut piece) in q_pieces.iter_mut() {
            if piece.side != side {
                continue;
            }

            // don't allow placing pieces on the edge
            if center.0 > 15 || center.1 > 15 || center.0 < 3 || center.1 < 3 {
                continue;
            }

            // don't allow placing pieces on the opposite watchtower
            let distance = ((center.0 as f32 - opposite_watchtower_position.0).powi(2)
                + (center.1 as f32 - opposite_watchtower_position.1).powi(2))
            .sqrt();

            if distance < 5.0 {
                continue;
            }

            match piece.n {
                0 => {
                    transform.translation.x = center.0 as f32 - 1.0;
                    transform.translation.z = center.1 as f32 - 1.0;
                    piece.i = center.0 - 1;
                    piece.j = center.1 - 1;
                }

                1 => {
                    transform.translation.x = center.0 as f32 - 1.0;
                    transform.translation.z = center.1 as f32;
                    piece.i = center.0 - 1;
                    piece.j = center.1;
                }

                2 => {
                    transform.translation.x = center.0 as f32 - 1.0;
                    transform.translation.z = center.1 as f32 + 1.0;
                    piece.i = center.0 - 1;
                    piece.j = center.1 + 1;
                }

                3 => {
                    transform.translation.x = center.0 as f32;
                    transform.translation.z = center.1 as f32 - 1.0;
                    piece.i = center.0;
                    piece.j = center.1 - 1;
                }

                4 => {
                    transform.translation.x = center.0 as f32;
                    transform.translation.z = center.1 as f32;
                    piece.i = center.0;
                    piece.j = center.1;
                }

                5 => {
                    transform.translation.x = center.0 as f32;
                    transform.translation.z = center.1 as f32 + 1.0;
                    piece.i = center.0;
                    piece.j = center.1 + 1;
                }

                6 => {
                    transform.translation.x = center.0 as f32 + 1.0;
                    transform.translation.z = center.1 as f32 - 1.0;
                    piece.i = center.0 + 1;
                    piece.j = center.1 - 1;
                }

                7 => {
                    transform.translation.x = center.0 as f32 + 1.0;
                    transform.translation.z = center.1 as f32;
                    piece.i = center.0 + 1;
                    piece.j = center.1;
                }

                8 => {
                    transform.translation.x = center.0 as f32 + 1.0;
                    transform.translation.z = center.1 as f32 + 1.0;
                    piece.i = center.0 + 1;
                    piece.j = center.1 + 1;
                }

                _ => {}
            }

            for (_entity, mut transform, mut watchtower) in q_watchtower.iter_mut() {
                if watchtower.side != side {
                    continue;
                }

                transform.translation.x = center.0 as f32;
                transform.translation.z = center.1 as f32;
                watchtower.i = center.0;
                watchtower.j = center.1;
            }
        }
    }
}
