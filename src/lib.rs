use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Player {
    White,
    Black,
}
impl Player {
    fn opponent(&self) -> Player {
        match self {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Piece(isize);

#[derive(Debug, PartialEq, Eq, Clone)]
struct Pieces(Vec<Piece>);
impl Pieces {
    const BOARD_SIZE: usize = 26;
    const INNER_BOARD: usize = 6;
    const BAR: usize = 25;
    const GOAL: usize = 0;
    const BLACK_BAR: usize = 27;
    const BLACK_GOAL: usize = 26;
    const MAX_PIECES: usize = 15;

    fn new() -> Pieces {
        let p = vec![
            0_, -2, 0, 0, 0, 0, 5_, 0, 3, 0, 0, 0, -5_, 5, 0, 0, 0, -3, 0_, -5, 0, 0, 0, 0, 2_, 0,
            0, 0,
        ];
        Pieces(p.into_iter().map(Piece).collect::<Vec<_>>())
    }

    fn reverse(&self) -> Pieces {
        let mut p = vec![Piece(0); Pieces::BOARD_SIZE + 2];
        for i in 0..Pieces::BOARD_SIZE {
            p[i] = self.0[Pieces::BOARD_SIZE - i - 1];
        }
        p[Pieces::BAR] = self.0[Pieces::BLACK_BAR];
        p[Pieces::GOAL] = self.0[Pieces::BLACK_GOAL];
        p[Pieces::BLACK_BAR] = self.0[Pieces::BAR];
        p[Pieces::BLACK_GOAL] = self.0[Pieces::GOAL];

        Pieces(p)
    }
    fn reversed(&self, p: Player) -> Pieces {
        if p == Player::White {
            self.clone()
        } else {
            self.reverse()
        }
    }

    fn get(&self, i: usize) -> Option<(Player, usize)> {
        let p = self.0[i];
        if p.0 > 0 {
            Some((Player::White, p.0 as usize))
        } else if p.0 < 0 {
            Some((Player::Black, (-p.0) as usize))
        } else {
            None
        }
    }
    fn set(&mut self, i: usize, p: Player, c: usize) {
        if c == 0 {
            self.0[i] = Piece(0);
        } else if p == Player::White {
            self.0[i] = Piece(c as isize)
        } else {
            self.0[i] = Piece(-(c as isize))
        }
    }
    fn add(&mut self, i: usize, p: Player, d: isize) {
        if p == Player::White {
            self.0[i] = Piece(self.0[i].0 + d)
        } else {
            self.0[i] = Piece(self.0[i].0 - d)
        }
    }
    fn hittable(&self, to: usize, player: Player) -> bool {
        if let Some((p, c)) = self.get(to) {
            p != player && c == 1
        } else {
            false
        }
    }
    fn hit(&mut self, to: usize, p: Player) {
        assert!(self.hittable(to, p));
        self.set(to, p.opponent(), 0);
        self.add(Pieces::BLACK_BAR, p.opponent(), 1);
    }
    fn movable(&self, from: usize, to: usize, player: Player) -> bool {
        if let Some((p, _)) = self.get(from) {
            if p != player {
                return false;
            }
            if let Some((o, d)) = self.get(to) {
                if o == player {
                    true
                } else {
                    d == 1
                }
            } else {
                true
            }
        } else {
            false
        }
    }
    fn mov(&mut self, from: usize, to: usize, player: Player) {
        assert!(self.movable(from, to, player));
        if self.hittable(to, player) {
            self.hit(to, player);
        }
        self.add(from, player, -1);
        self.add(to, player, 1);
    }
    fn backman(&self, p: Player) -> usize {
        for i in (0..Pieces::BOARD_SIZE).rev() {
            if let Some((o, c)) = self.get(i) {
                if o == p {
                    return i;
                }
            }
        }
        panic!("no pieces")
    }
    fn listup(&self, dice: &[usize], p: Player) -> Vec<Move> {
        let pieces = self;
        if dice.len() == 0 {
            return vec![Move(vec![])];
        }
        let (d, dice) = dice.split_at(1);
        let mut d = d[0];
        let mut mov = vec![];
        for i in (0..Pieces::BOARD_SIZE).rev() {
            let backman = pieces.backman(p);
            // move from bar
            if backman == 0 && !pieces.movable(0, d, p) {
                continue;
            }
            // backman can be bearoff over rolled
            if i == backman && backman <= Pieces::INNER_BOARD && i < d {
                d = backman;
            }
            // bareoff is not allowed if backman dose not reached
            if backman > Pieces::INNER_BOARD && i == d {
                continue;
            }
            // too big move
            if i < d {
                continue;
            }
            if pieces.movable(i, i - d, p) {
                let mut np = pieces.clone();
                np.mov(i, i - d, p);
                for mut m in np.listup(dice, p) {
                    m.0.push((i, i - d, pieces.hittable(i - d, p)));
                    mov.push(m);
                }
            }
        }
        mov.iter_mut().for_each(|m| m.0.reverse());
        mov
    }
    fn goal(&self, p: Player) -> usize {
        let ps = self.reversed(p);
        if ps.backman(p) > 0 {
            return 0;
        }
        let ps = ps.reverse();
        if ps.get(Pieces::GOAL).is_some() {
            return 1;
        }
        let backman = ps.backman(p);
        if backman < 19 {
            2
        } else {
            3
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Dice(usize, usize);
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct DiceRoll(Option<Dice>);
impl DiceRoll {
    fn new() -> DiceRoll {
        DiceRoll(None)
    }
    fn roll(x: usize, y: usize) -> DiceRoll {
        DiceRoll(Some(Dice(x, y)))
    }
    fn init_player(&self) -> Option<Player> {
        match self.0 {
            None => None,
            Some(Dice(a, b)) => {
                if a > b {
                    Some(Player::White)
                } else {
                    Some(Player::Black)
                }
            }
        }
    }

    fn moves(&self) -> Vec<Vec<usize>> {
        let Dice(x, y) = self.0.unwrap();
        if x == y {
            vec![vec![x; 4]]
        } else {
            vec![vec![x, y], vec![y, x]]
        }
    }
    fn all() -> Vec<Dice> {
        let mut v = vec![];
        for x in 1..=6 {
            for y in x..=6 {
                v.push(Dice(x, y));
            }
        }
        v
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Cube {
    position: Option<Player>,
    value: usize,
    doubled: bool,
}
impl Cube {
    const CenterInit: Cube = Cube {
        position: None,
        value: 1,
        doubled: false,
    };
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Match {
    score: (usize, usize),
    length: usize,
}
impl Match {
    fn single() -> Match {
        Match {
            score: (0, 0),
            length: 1,
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
struct Board {
    pieces: Pieces,
    dice: DiceRoll,
    cube: Cube,
    player: Option<Player>,
    doubled: Option<Cube>,
    game: Match,
    result: Option<Result>,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Result {
    player: Player,
    score: usize,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum State {
    Init,
    ToRoll,
    Doubled,
    ToMove,
    End,
    MatchEnd,
}
#[derive(Debug, PartialEq, Eq, Clone)]
enum Action {
    InitRoll(Dice),
    Roll(Dice),
    Move(Move),
    Double,
    Pass,
    Take,
    Reset,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Move(Vec<(usize, usize, bool)>);
impl Move {
    const DANCE: Move = Move(vec![]);
    fn to_str(&self) -> String {
        let mut mov = self.0.clone();
        mov.sort_by(|(a, b, _), (c, d, _)| (-(*a as isize), b).cmp(&(-(*c as isize), d)));
        let mut s = String::new();
        let mut i = 0;
        while i < mov.len() {
            let (from, to, hit) = mov[i];
            if i != 0 {
                s.push_str(" ");
            }
            s.push_str(&format!("{}", from));
            let mut prev = to;
            let mut last_hit = hit;
            let mut j = i + 1;
            while j < mov.len() {
                let (from, to, hit) = mov[j];
                if prev != from {
                    j += 1;
                    continue;
                }
                mov.remove(j);
                if last_hit {
                    s.push_str(&format!("/{}", prev));
                    s.push_str("*");
                }
                prev = to;
                last_hit = hit;
            }
            s.push_str(&format!("/{}", prev));
            s.push_str(if last_hit { "*" } else { "" });
            i += 1;
        }
        s
    }
    fn uniq_moves(moves: &[Move]) -> Vec<Move> {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        for m in moves {
            let s = m.to_str();
            if let Some(v) = map.get(&s) {
                if *v > m {
                    map.insert(s, m);
                }
            } else {
                map.insert(s, m);
            }
        }
        let mut moves = vec![];
        for m in map.into_values() {
            moves.push(m.clone());
        }
        moves.sort();
        moves
    }
}
impl PartialOrd for Move {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let x = self.0.iter().map(|(a, b, _)| (-(*a as isize), b));
        let y = other.0.iter().map(|(a, b, _)| (-(*a as isize), b));
        Some(x.cmp(y))
    }
}
impl Ord for Move {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Board {
    fn new() -> Board {
        Board {
            pieces: Pieces::new(),
            dice: DiceRoll::new(),
            cube: Cube::CenterInit,
            player: None,
            doubled: None,
            result: None,
            game: Match::single(),
        }
    }
    fn act(&mut self, act: &Action) {
        match act {
            Action::InitRoll(d) => self.init_roll(*d),
            Action::Roll(d) => self.roll(*d),
            Action::Move(m) => self.act_move(m),
            Action::Double => self.double(),
            Action::Pass => self.pass(),
            Action::Take => self.take(),
            Action::Reset => self.reset(),
        }
    }
    fn actions(&self) -> Vec<Action> {
        match self.state() {
            State::Init => DiceRoll::all()
                .into_iter()
                .map(|d| Action::InitRoll(d))
                .collect(),
            State::ToMove => self.moves().into_iter().map(|m| Action::Move(m)).collect(),
            State::ToRoll => {
                let mut res: Vec<Action> = DiceRoll::all()
                    .into_iter()
                    .map(|d| Action::InitRoll(d))
                    .collect();
                res.push(Action::Double);
                res
            }
            State::Doubled => vec![Action::Pass, Action::Take],
            State::End => vec![Action::Reset],
            State::MatchEnd => vec![],
        }
    }
    fn init_roll(&mut self, dice: Dice) {
        self.dice = DiceRoll(Some(dice));
        self.player = self.dice.init_player();
    }
    fn roll(&mut self, dice: Dice) {
        self.dice = DiceRoll(Some(dice));
    }
    fn act_move(&mut self, mov: &Move) {
        let p = self.player.unwrap();
        let mut ps = self.pieces.reversed(p);
        for m in mov.0.iter() {
            ps.mov(m.0, m.1, p);
        }
        self.pieces = ps.reversed(p);
        self.dice = DiceRoll(None);
        self.player = Some(p.opponent());

        self.check_end()
    }
    fn moves(&self) -> Vec<Move> {
        let p = self.player.unwrap();
        let mut moves = vec![];
        for dice in self.dice.moves().iter_mut() {
            let pieces = self.pieces.reversed(p);
            let mut m = pieces.listup(dice, self.player.unwrap());
            moves.append(&mut m);
        }
        Move::uniq_moves(&moves)
    }

    fn double(&mut self) {
        todo!()
    }

    fn pass(&mut self) {
        self.game_end()
    }

    fn take(&mut self) {
        todo!()
    }

    fn reset(&mut self) {
        todo!()
    }

    fn check_end(&mut self) {
        let p = self.player.unwrap();
        let ps = self.pieces.reversed(p);
        let white = ps.goal(Player::White);
        let black = ps.goal(Player::Black);
        if white > 0 {
            self.result = Some(Result {
                player: Player::White,
                score: white * self.cube.value,
            });
            self.game_end();
        } else if black > 0 {
            self.result = Some(Result {
                player: Player::Black,
                score: black,
            });
            self.game_end();
        }
    }

    fn game_end(&mut self) {
        self.player = None;
    }

    fn state(&self) -> State {
        if self.result.is_some() {
            return State::End;
        }
        if self.dice.0.is_some() {
            return State::ToMove;
        }
        if self.cube.doubled {
            return State::Doubled;
        }
        if self.dice.0.is_none() {
            return State::ToRoll;
        }
        return State::Init;
    }
}
impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.player == Some(Player::White) {
            write!(f, "White to play... ↓")?;
        } else if self.player == Some(Player::Black) {
            write!(f, "Black to play... ↑")?;
        } else {
            write!(f, "No player to play... ")?;
        }
        if let Some(dice) = self.dice.0 {
            write!(f, "Dice: {:?} ", dice)?;
        }
        write!(f, "\nCube: {:?} ", self.cube)?;
        write!(f, "\n")?;
        write!(f, "Score: {:?} ", self.game.score)?;
        write!(f, "Length: {:?} \n", self.game.length)?;
        use std::fmt::Write;
        let mut board = String::new();
        fn print_piece(
            f: &mut String,
            pieces: &Pieces,
            count: usize,
            pos: usize,
        ) -> std::fmt::Result {
            if pos == 19 || pos == 6 {
                write!(f, "|")?;
            }
            match pieces.get(pos) {
                None => {
                    write!(f, "   ")?;
                }
                Some((p, c)) => {
                    if c <= count {
                        write!(f, "   ")?;
                    } else if c == 5 && count > 5 {
                        write!(f, " {} ", count)?;
                    } else {
                        if p == Player::White {
                            write!(f, " W ")?;
                        } else {
                            write!(f, " B ")?;
                        }
                    }
                }
            }
            Ok(())
        }
        if self.player == Some(Player::Black) {
            write!(board, " 12 11 10  9  8  7   6  5  4  3  2  1\n")?;
        } else {
            write!(board, " 13 14 15 16 17 18  19 20 21 22 23 24\n")?;
        }
        write!(board, "+-=--*--=--*--=--*-+-=--*--=--*--=--*-+\n")?;
        for c in 0..6 {
            write!(board, "|")?;
            for i in 13..25 {
                print_piece(&mut board, &self.pieces, c, i)?;
            }
            write!(board, "|")?;
            print_piece(&mut board, &self.pieces, c, Pieces::BLACK_GOAL)?;
            write!(board, "\n")?;
        }
        write!(board, "+------------------+------------------+\n")?;
        for c in (0..6).rev() {
            write!(board, "|")?;
            for i in (1..13).rev() {
                print_piece(&mut board, &self.pieces, c, i)?;
            }
            write!(board, "|")?;
            print_piece(&mut board, &self.pieces, c, Pieces::GOAL)?;
            write!(board, "\n")?;
        }
        write!(board, "+-*--=--*--=--*--=-+-*--=--*--=--*--=-+\n")?;
        if self.player == Some(Player::Black) {
            write!(board, " 13 14 15 16 17 18  19 20 21 22 23 24\n")?;
        } else {
            write!(board, " 12 11 10  9  8  7   6  5  4  3  2  1\n")?;
        }
        if let Some((_, c)) = self.pieces.get(Pieces::BAR) {
            if c > 0 {
                write!(board, "BAR W: {}  ", c)?;
            }
        }
        if let Some((_, c)) = self.pieces.get(Pieces::BLACK_BAR) {
            if c > 0 {
                write!(board, "BAR B: {}  ", c)?;
            }
        }
        write!(f, "{}", board)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn print_board() {
        let b = Board::new();
        println!("{}", b);
        assert_eq!(b.state(), State::Init);
    }

    #[test]
    fn print_moves() {
        let mut b = Board::new();
        b.init_roll(Dice(2, 1));
        println!("{}", b);
        let moves = b.moves();
        assert_eq!(moves.len(), 15);
        for m in moves {
            println!("{}", m.to_str());
        }
    }

    #[test]
    fn single_game() {
        let mut b = Board::new();
        b.init_roll(Dice(2, 1));
        println!("{}", b);

        let act = &b.actions()[0];
        println!("{:?}", act);
        b.act(&act);
        println!("{}", b);
        assert_eq!(b.state(), State::ToRoll);
        assert_eq!(b.player, Some(Player::Black));

        b.act(&Action::Roll(Dice(2, 1)));
        println!("{}", b);
        assert_eq!(b.state(), State::ToMove);
        assert_eq!(b.player, Some(Player::Black));

        let moves = b.moves();
        for m in moves {
            println!("{}", m.to_str());
        }
        let m = Move(vec![(6, 4, false), (4, 3, true)]);
        b.act(&Action::Move(m));
        println!("{}", b);
        assert_eq!(b.pieces.get(Pieces::BAR), Some((Player::White, 1)));
    }

    #[test]
    fn big_roll() {
        let mut b = Board::new();
        b.init_roll(Dice(5, 6));
        let act = &b.actions()[0];
        b.act(&act);
        let mut i = 0;
        while b.state() != State::End {
            b.act(&Action::Roll(Dice(5, 6)));
            println!("{}", b);
            let act = &b.actions()[0];
            b.act(&act);
            i += 1;
            if i > 39 {
                assert!(false, "infinite loop")
            }
        }
        print!("{}", b);
        assert_eq!(
            b.result,
            Some(Result {
                player: Player::Black,
                score: 1
            })
        );
    }

    #[test]
    fn move_ord() {
        assert!(
            Move(vec![(24, 22, false), (24, 23, false)])
                < Move(vec![(24, 23, false), (24, 22, false)])
        );
    }

    #[test]
    fn move_to_str() {
        assert_eq!(Move(vec![(6, 4, false), (4, 3, true)]).to_str(), "6/3*")
    }
}
