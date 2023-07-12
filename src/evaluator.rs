use std::collections::HashMap;

use crate::board::{Action, Board, DiceRoll, Match, Player};

#[derive(Clone, Debug)]
struct Equities(Vec<(Action, f64)>);
impl Equities {
    fn max(&self, player: Player) -> (Action, f64) {
        self.0
            .iter()
            .max_by(|(_, a), (_, b)| {
                if player == Player::White {
                    a.partial_cmp(b).unwrap()
                } else {
                    b.partial_cmp(a).unwrap()
                }
            })
            .unwrap()
            .clone()
    }
}

trait Evaluator {
    fn eval(&mut self, board: &Board) -> Equities;
}

struct OpenEvaluator {
    tree: HashMap<Board, Equities>,
}

impl Evaluator for OpenEvaluator {
    fn eval(&mut self, board: &Board) -> Equities {
        if let Some(eq) = self.tree.get(&board) {
            return eq.clone();
        }
        let eq = match board.state() {
            crate::board::State::Init => self.eval_init(&board),
            crate::board::State::ToRoll => self.eval_double(&board),
            crate::board::State::Doubled => self.eval_doubled(&board),
            crate::board::State::ToMove => self.eval_move(&board),
            crate::board::State::End => self.eval_end(&board),
            crate::board::State::MatchEnd => self.eval_end(board),
        };
        self.tree.insert(board.to_owned(), eq.clone());
        eq
    }
}

impl OpenEvaluator {
    fn new() -> Self {
        Self {
            tree: HashMap::new(),
        }
    }
    fn eval_init(&mut self, board: &Board) -> Equities {
        Equities(vec![(Action::None, fetch_match_equities(&board.game))])
    }
    fn eval_end(&mut self, board: &Board) -> Equities {
        Equities(vec![(Action::Reset, fetch_match_equities(&board.game))])
    }
    fn eval_move(&mut self, board: &Board) -> Equities {
        let moves = board.actions();
        let mut equities = Vec::new();
        for mov in moves {
            let mut next = board.clone();
            next.act(&mov);
            equities.push((mov, self.eval(&next).max(board.player.unwrap()).1));
        }
        Equities(equities)
    }
    fn eval_double(&mut self, board: &Board) -> Equities {
        let roll = DiceRoll::all_with_prob()
            .into_iter()
            .map(|(dice, c)| {
                let act = Action::Roll(dice);
                let mut roll = board.clone();
                roll.act(&act);
                self.eval(&roll).max(roll.player.unwrap()).1 * (c as f64)
            })
            .sum();
        if !board.can_double() {
            Equities(vec![(Action::None, roll)])
        } else {
            let mut double = board.clone();
            double.act(&Action::Double);
            let double_eq = self.eval(&double).max(board.player.unwrap()).1;
            Equities(vec![(Action::None, roll), (Action::Double, double_eq)])
        }
    }
    fn eval_doubled(&mut self, board: &Board) -> Equities {
        let mut pass = board.clone();
        pass.act(&Action::Pass);
        let pass_eq = self.eval(&pass).max(board.player.unwrap());

        let mut take = board.clone();
        take.act(&Action::Take);
        let take_eq = self.eval(&take).max(board.player.unwrap());
        Equities(vec![(Action::Pass, pass_eq.1), (Action::Take, take_eq.1)])
    }
}

fn fetch_match_equities(game: &Match) -> f64 {
    if let Some(winner) = game.winner() {
        if winner == Player::White {
            1.
        } else {
            0.
        }
    } else {
        todo!("give match equiity table")
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn small_case() {
        let b = Board::from_xgid("XGID=-A----------------------a-:0:0:1::0:0:0:1:10");
        let mut e = OpenEvaluator::new();
        let eq = e.eval(&b);
        println!("{:?}", eq);
        assert_eq!(eq.0.len(), 2);
        let eps = 1e5;
        assert!((eq.0[0].1 - 1.).abs() < eps, "{}", eq.0[0].1);
        assert!((eq.0[1].1 - 1.).abs() < eps, "{}", eq.0[1].1);
    }
}
