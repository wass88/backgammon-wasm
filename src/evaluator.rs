use std::collections::HashMap;

use crate::board::{Action, Board, Dice, DiceRoll, Match, Player, State};

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
            State::Init => self.eval_init(&board),
            State::ToDouble => self.eval_to_double(&board),
            State::ToRoll => self.eval_to_roll(&board),
            State::Doubled => self.eval_doubled(&board),
            State::ToMove => self.eval_move(&board),
            State::End => self.eval_end(&board),
            State::MatchEnd => self.eval_end(board),
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
    fn eval_to_double(&mut self, board: &Board) -> Equities {
        let mut no_double = board.clone();
        no_double.act(&Action::NoDouble);
        let no_double_eq = self
            .eval(&no_double)
            .0
            .into_iter()
            .map(|(a, eq)| {
                if let Action::Roll(d) = a {
                    d.prob() * eq
                } else {
                    unreachable!()
                }
            })
            .sum();
        let mut eq = vec![(Action::NoDouble, no_double_eq)];
        if !board.can_double() {
            return Equities(eq);
        }
        let mut double = board.clone();
        double.act(&Action::Double);
        let double_eq = self.eval(&double).max(board.player.unwrap()).1;
        eq.push((Action::Double, double_eq));
        Equities(eq)
    }
    fn eval_to_roll(&mut self, board: &Board) -> Equities {
        let roll = DiceRoll::all()
            .into_iter()
            .map(|dice| {
                let act = Action::Roll(dice);
                let mut roll = board.clone();
                roll.act(&act);
                let eq = self.eval(&roll).max(roll.player.unwrap()).1;
                (act, eq)
            })
            .collect();
        Equities(roll)
    }
    fn eval_doubled(&mut self, board: &Board) -> Equities {
        println!("doubled {}", board);
        let mut pass = board.clone();
        pass.act(&Action::Pass);
        let pass_eq = self.eval(&pass).max(board.player.unwrap());
        println!("pass {}", pass);
        println!("pass_eq {:?}", pass_eq);

        let mut take = board.clone();
        take.act(&Action::Take);
        println!("take {}", take);
        let take_eq = self.eval(&take).max(board.player.unwrap());
        println!("take_eq {:?}", take_eq);
        Equities(vec![(Action::Pass, pass_eq.1), (Action::Take, take_eq.1)])
    }

    fn gen_tree(&self, board: &Board) -> Tree {
        match board.state() {
            crate::board::State::ToRoll => {}
            crate::board::State::End => {}
            crate::board::State::MatchEnd => {}
            _ => {}
        }
        todo!()
    }
}

struct Tree {
    root: Board,
    children: Vec<Equities>,
    prob: f64,
}

fn fetch_match_equities(game: &Match) -> f64 {
    if let Some(winner) = game.winner() {
        if winner == Player::White {
            1.
        } else {
            -1.
        }
    } else {
        assert!(game.length <= 5, "assert match length {} <= 5", game.length);
        // https://bkgm.com/articles/Kazaross/RockwellKazarossMET/index.html
        let pc = vec![0.50, 0.51, 0.68, 0.69, 0.81];
        let table = vec![
            vec![0.50, 0.68, 0.75, 0.81, 0.84],
            vec![0.32, 0.50, 0.57, 0.63, 0.66],
            vec![0.25, 0.43, 0.50, 0.56, 0.59],
            vec![0.19, 0.37, 0.44, 0.50, 0.53],
            vec![0.16, 0.34, 0.41, 0.47, 0.50],
        ];
        let (w, b) = game.score;
        let (aw, ab) = (game.length - w, game.length - b);
        if aw == 1 && !game.crawford {
            return pc[ab - 1];
        }
        if ab == 1 && !game.crawford {
            return 1. - pc[aw - 1];
        }
        table[aw - 1][ab - 1]
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn minimum_case() {
        let b = Board::from_xgid("XGID=-A----------------------a-:0:0:1::0:0:0:1:10");
        let mut e = OpenEvaluator::new();
        let eq = e.eval(&b);
        println!("{:?}", eq);
        assert_eq!(eq.0.len(), 2);
        let eps = 1e5;
        assert!((eq.0[0].1 - 1.).abs() < eps, "{}", eq.0[0].1);
        assert!((eq.0[1].1 - 1.).abs() < eps, "{}", eq.0[1].1);
    }
    #[test]
    fn small_case() {
        let b = Board::from_xgid("XGID=-----A-----------------a--:0:0:1::0:0:0:1:10");
        println!("{}", b);
        let mut e = OpenEvaluator::new();
        let eq = e.eval(&b);
        println!("{:?}", eq);
        assert_eq!(eq.0.len(), 2);
        let eps = 1e-5;
        assert!(eq.0[0].1 + eps < 1., "assert {} < 1", eq.0[0].1);
        assert!((eq.0[1].1 - 1.).abs() < eps, "{}", eq.0[1].1);
    }
    #[test]
    fn take_case() {
        let b = Board::from_xgid("XGID=--------A--------------a--:0:0:1::0:0:0:3:10");
        println!("{}", b);
        let mut e = OpenEvaluator::new();
        let eq = e.eval(&b);
        println!("{:?}", eq);
    }

    #[test]
    fn match_eq() {
        let p = fetch_match_equities(&Match {
            length: 5,
            score: (0, 0),
            crawford: false,
        });
        assert!((p - 0.5) < 1e-5, "{}", p);
        let p = fetch_match_equities(&Match {
            length: 5,
            score: (4, 1),
            crawford: true,
        });
        assert!((p - 0.81) < 1e-5, "{}", p);
        let p = fetch_match_equities(&Match {
            length: 5,
            score: (4, 1),
            crawford: false,
        });
        assert!((p - 0.69) < 1e-5, "{}", p);
    }
}
