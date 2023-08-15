#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Player {
    White,
    Black,
}
impl Player {
    pub fn opponent(&self) -> Player {
        match self {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Piece(isize);

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Pieces(Vec<Piece>);
impl Pieces {
    const BOARD_SIZE: usize = 26;
    const INNER_BOARD: usize = 6;
    const BAR: usize = 25;
    const GOAL: usize = 0;
    const BLACK_GOAL: usize = 26;
    const BLACK_BAR: usize = 27;
    const MAX_PIECES: usize = 15;

    fn empty() -> Pieces {
        Pieces(vec![Piece(0); Pieces::BOARD_SIZE + 2])
    }
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
        assert!(
            self.movable(from, to, player),
            "{:?}/{:?} {:?}",
            from,
            to,
            player
        );
        if self.hittable(to, player) {
            self.hit(to, player);
        }
        self.add(from, player, -1);
        self.add(to, player, 1);
    }
    fn backman(&self, p: Player) -> usize {
        for i in (0..=Pieces::BAR).rev() {
            if let Some((o, _)) = self.get(i) {
                if o == p {
                    return i;
                }
            }
        }
        panic!("no pieces")
    }
    fn listup(&self, dice: &[usize], p: Player) -> Vec<Move> {
        let pieces = self;
        if dice.len() == 0 || self.backman(p) == 0 {
            return vec![Move(vec![])];
        }
        let (d, dice) = dice.split_at(1);
        let mut d = d[0];
        let mut mov = vec![];
        for i in (1..=Pieces::BOARD_SIZE).rev() {
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
                    m.0.insert(0, (i, i - d, pieces.hittable(i - d, p)));
                    mov.push(m);
                }
            }
        }
        mov
    }
    fn goal(&self, p: Player) -> usize {
        let ps = self.reversed(p);
        if ps.backman(p) > 0 {
            return 0;
        }
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
pub struct Dice(pub usize, pub usize);
impl Dice {
    pub fn prob(&self) -> f64 {
        if self.0 == self.1 {
            1.0 / 36.0
        } else {
            2.0 / 36.0
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct DiceRoll(Option<Dice>);
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
    pub fn all() -> Vec<Dice> {
        let mut v = vec![];
        for x in 1..=6 {
            for y in x..=6 {
                v.push(Dice(x, y));
            }
        }
        v
    }
    pub fn all_with_prob() -> Vec<(Dice, f64)> {
        let mut v = vec![];
        for x in 1..=6 {
            for y in x..=6 {
                v.push((Dice(x, y), if x == y { 1. / 36. } else { 2. / 36. }));
            }
        }
        v
    }
    fn to_str(&self) -> String {
        match self.0 {
            None => "-".to_string(),
            Some(Dice(x, y)) => format!("{}{}", x, y),
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Cube {
    position: Option<Player>,
    level: usize,
    doubled: bool,
    max_level: usize,
}
impl Cube {
    const CENTER_INIT: Cube = Cube {
        position: None,
        level: 0,
        doubled: false,
        max_level: Cube::DEFAULT_MAX_LEVEL,
    };
    const DEFAULT_MAX_LEVEL: usize = 10;

    fn double(&self, p: Player) -> Cube {
        Cube {
            position: Some(p.opponent()),
            level: self.level,
            doubled: true,
            max_level: 10,
        }
    }
    fn reach_max(self) -> bool {
        self.level >= self.max_level
    }
    fn take(&self) -> Cube {
        let mut cube = self.clone();
        cube.doubled = false;
        cube.level += 1;
        cube
    }
    fn value(&self) -> usize {
        1 << self.level
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Match {
    pub score: (usize, usize),
    pub length: usize,
    pub crawford: bool,
}
impl Match {
    fn single() -> Match {
        Match {
            score: (0, 0),
            length: 1,
            crawford: false,
        }
    }
    fn with_length(length: usize) -> Match {
        Match {
            score: (0, 0),
            length,
            crawford: false,
        }
    }
    fn add_score(&mut self, player: Player, score: usize) {
        let not_reached = self.score.0 < self.length - 1 && self.score.1 < self.length - 1;
        if player == Player::White {
            self.score.0 += score;
        } else {
            self.score.1 += score;
        }
        if self.score.0 >= self.length {
            self.score.0 = self.length;
        }
        if self.score.1 >= self.length {
            self.score.1 = self.length;
        }
        if self.crawford {
            self.crawford = false;
        } else if not_reached && self.score.0 == self.length - 1 && self.score.1 == self.length - 1
        {
            self.crawford = true;
        }
    }
    pub fn winner(&self) -> Option<Player> {
        if self.score.0 >= self.length {
            Some(Player::White)
        } else if self.score.1 >= self.length {
            Some(Player::Black)
        } else {
            None
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Board {
    pub pieces: Pieces,
    pub dice: DiceRoll,
    pub cube: Cube,
    pub to_roll: bool,
    pub player: Option<Player>,
    pub game: Match,
    pub result: Option<Result>,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Result {
    player: Player,
    score: usize,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum State {
    Init,
    ToDouble,
    ToRoll,
    Doubled,
    ToMove,
    End,
    MatchEnd,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Action {
    InitRoll(Dice),
    Roll(Dice),
    Move(Move),
    NoDouble,
    Double,
    Pass,
    Take,
    Reset,
    None, // for tree search
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Move(Vec<(usize, usize, bool)>);
impl Move {
    const DANCE: Move = Move(vec![]);
    pub fn to_str(&self) -> String {
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
    fn filter_moves(moves: &[Move]) -> Vec<Move> {
        if let Some(max_moves) = moves.iter().map(|m| m.0.len()).max() {
            let use_all = moves.iter().filter(|m| m.0.len() == max_moves);
            if max_moves == 1 {
                let max_roll = use_all.clone().map(|m| m.0[0].0 - m.0[0].1).max().unwrap();
                use_all
                    .filter(|m| m.0[0].0 - m.0[0].1 == max_roll)
                    .map(|m| m.clone())
                    .collect()
            } else if max_moves == 0 {
                vec![Move::DANCE]
            } else {
                use_all.cloned().collect()
            }
        } else {
            vec![]
        }
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
    pub fn new() -> Board {
        Board {
            pieces: Pieces::new(),
            dice: DiceRoll::new(),
            cube: Cube::CENTER_INIT,
            to_roll: false,
            player: None,
            result: None,
            game: Match::single(),
        }
    }
    pub fn act(&mut self, act: &Action) {
        match act {
            Action::InitRoll(d) => self.init_roll(*d),
            Action::Roll(d) => self.roll(*d),
            Action::Move(m) => self.act_move(m),
            Action::NoDouble => self.no_double(),
            Action::Double => self.double(),
            Action::Pass => self.pass(),
            Action::Take => self.take(),
            Action::Reset => self.reset(),
            Action::None => unreachable!(),
        }
    }
    pub fn actions(&self) -> Vec<Action> {
        match self.state() {
            State::Init => DiceRoll::all()
                .into_iter()
                .map(|d| Action::InitRoll(d))
                .collect(),
            State::ToMove => self.moves().into_iter().map(|m| Action::Move(m)).collect(),
            State::ToDouble => {
                if self.can_double() {
                    vec![Action::Double, Action::NoDouble]
                } else {
                    vec![Action::NoDouble]
                }
            }
            State::ToRoll => DiceRoll::all()
                .into_iter()
                .map(|d| Action::InitRoll(d))
                .collect(),
            State::Doubled => vec![Action::Pass, Action::Take],
            State::End => vec![Action::Reset],
            State::MatchEnd => vec![],
        }
    }
    pub fn init_roll(&mut self, dice: Dice) {
        self.dice = DiceRoll(Some(dice));
        self.player = self.dice.init_player();
    }
    fn roll(&mut self, dice: Dice) {
        self.dice = DiceRoll(Some(dice));
        self.to_roll = false;
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
        Move::filter_moves(&Move::uniq_moves(&moves))
    }

    pub fn can_double(&self) -> bool {
        !self.game.crawford
            && !self.cube.reach_max()
            && (self.cube.position.is_none() || self.cube.position == self.player)
    }

    fn no_double(&mut self) {
        self.to_roll = true;
    }
    fn double(&mut self) {
        let p = self.player.unwrap();
        assert!(self.can_double());
        self.cube = self.cube.double(p);
        self.player = Some(p.opponent());
    }

    fn pass(&mut self) {
        let p = self.player.unwrap();
        assert!(self.cube.doubled);
        self.result = Some(Result {
            player: p.opponent(),
            score: self.cube.value(),
        });
        self.game_end()
    }

    fn take(&mut self) {
        assert!(self.cube.doubled);
        self.cube = self.cube.take();
        self.player = Some(self.player.unwrap().opponent());
        self.to_roll = true;
    }

    fn reset(&mut self) {
        todo!()
    }

    fn check_end(&mut self) {
        let p = self.player.unwrap();
        let white = self.pieces.goal(Player::White);
        let black = self.pieces.goal(Player::Black);
        if white > 0 {
            self.result = Some(Result {
                player: Player::White,
                score: white * self.cube.value(),
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
        let result = self.result.unwrap();
        self.game.add_score(result.player, result.score);
    }

    pub fn state(&self) -> State {
        if self.game.winner().is_some() {
            return State::MatchEnd;
        }
        if self.result.is_some() {
            return State::End;
        }
        if self.dice.0.is_some() {
            return State::ToMove;
        }
        if self.cube.doubled {
            return State::Doubled;
        }
        if self.to_roll {
            return State::ToRoll;
        }
        if self.dice.0.is_none() && self.player.is_some() {
            return State::ToDouble;
        }
        return State::Init;
    }

    pub fn xgid(&self) -> String {
        let mut s = "XGID=".to_string();
        // TOP = Black, BOTTOM = White
        let mut order = vec![];
        for i in 0..=Pieces::BOARD_SIZE {
            order.push(i);
        }
        order.push(Pieces::BLACK_BAR);
        for i in order {
            if let Some((p, c)) = self.pieces.get(i) {
                if p == Player::White {
                    s.push(('A' as usize + c - 1) as u8 as char);
                } else {
                    s.push(('a' as usize + c - 1) as u8 as char);
                }
            } else {
                s.push('-');
            }
        }

        let level = self.cube.level;
        let pos = match self.cube.position {
            Some(Player::White) => "1",
            Some(Player::Black) => "-1",
            None => "0",
        };
        let player = match self.player {
            Some(Player::White) => "1",
            Some(Player::Black) => "0",
            None => "",
        };
        let dice = match self.state() {
            State::ToMove => self.dice.to_str(),
            State::Doubled => "D".to_owned(),
            _ => "".to_owned(),
        };
        let white_score = self.game.score.0;
        let black_score = self.game.score.1;

        let crawford = if self.game.crawford { '1' } else { '0' };
        let length = self.game.length;
        let max_level = self.cube.max_level;

        // piece:cube:pos:player:dice:white_score:black_score:crawford:length:max_level
        s.push_str(&format!(
            ":{}:{}:{}:{}:{}:{}:{}:{}:{}",
            level, pos, player, dice, white_score, black_score, crawford, length, max_level
        ));

        s
    }
    pub fn from_xgid(id: &str) -> Board {
        let i = id.find("=").unwrap();
        let id = &id[i + 1..];
        let id: Vec<&str> = id.split(':').collect();

        let mut pieces = Pieces::empty();
        let mut white_goal = 15;
        let mut black_goal = 15;
        id[0].chars().enumerate().for_each(|(i, b)| {
            if b == '-' {
                return;
            }
            let (p, c) = if b.is_uppercase() {
                let c = b as u8 + 1 - 'A' as u8;
                white_goal -= c;
                (Player::White, c)
            } else {
                let c = b as u8 + 1 - 'a' as u8;
                black_goal -= c;
                (Player::Black, c)
            };
            pieces.set(i, p, c as usize);
        });
        pieces.set(Pieces::GOAL, Player::White, white_goal as usize);
        pieces.set(Pieces::BLACK_GOAL, Player::Black, black_goal as usize);

        let level: usize = id[1].parse().unwrap();
        let max_level: usize = id[9].parse().unwrap();
        let cube = Cube {
            level,
            max_level,
            position: match id[2] {
                "1" => Some(Player::White),
                "-1" => Some(Player::Black),
                _ => None,
            },
            doubled: id[4] == "D",
        };

        let player = match id[3] {
            "1" => Some(Player::White),
            "-1" => Some(Player::Black),
            _ => None,
        };

        let dice = match id[4] {
            "D" => DiceRoll::new(),
            "" => DiceRoll::new(),
            s => DiceRoll::roll(
                (s.chars().nth(0).unwrap() as u8 - '0' as u8) as usize,
                (s.chars().nth(1).unwrap() as u8 - '0' as u8) as usize,
            ),
        };
        let game = Match {
            score: (id[5].parse().unwrap(), id[6].parse().unwrap()),
            crawford: id[7] == "1",
            length: id[8].parse().unwrap(),
        };

        Board {
            pieces,
            cube,
            player,
            to_roll: cube.position == player.map(|p| p.opponent()),
            dice,
            game,
            result: None,
        }
    }
}
impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} ", self.state())?;
        if self.player == Some(Player::White) {
            write!(f, "White to play... ↓")?;
        } else if self.player == Some(Player::Black) {
            write!(f, "Black to play... ↑")?;
        } else {
            write!(f, "No player to play... ")?;
            if self.state() == State::End {
                write!(f, "{:?}", self.result.unwrap())?
            }
        }
        if let Some(dice) = self.dice.0 {
            write!(f, "Dice: {:?} ", dice)?;
        }
        write!(f, "\nCube: {:?} ", self.cube)?;
        if self.to_roll {
            write!(f, "To roll... ")?;
        }
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
                    } else if count == 5 && c > 5 {
                        write!(f, " {:<2}", c)?;
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
impl core::hash::Hash for Board {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.xgid().hash(state);
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

        assert_eq!(b.state(), State::ToDouble);
        assert_eq!(b.player, Some(Player::Black));
        b.act(&Action::NoDouble);

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
        while b.state() != State::MatchEnd {
            b.act(&Action::NoDouble);
            b.act(&Action::Roll(Dice(5, 6)));
            println!("{}", b);
            let act = &b.actions()[0];
            println!("{:?}", act);
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
    fn double_pass() {
        let mut b = Board::new();
        b.game.length = 3;
        b.init_roll(Dice(5, 6));
        let act = &b.actions()[0];
        b.act(&act);

        assert_eq!(b.player, Some(Player::White));
        b.act(&Action::Double);
        assert_eq!(b.state(), State::Doubled);
        assert_eq!(b.player, Some(Player::Black));
        b.act(&Action::Pass);
        assert_eq!(b.state(), State::End);
        assert_eq!(
            b.result,
            Some(Result {
                player: Player::White,
                score: 1
            })
        );
    }

    #[test]
    fn double_take() {
        let mut b = Board::new();
        b.game.length = 3;
        b.init_roll(Dice(5, 6));
        let act = &b.actions()[0];
        b.act(&act);

        assert_eq!(b.player, Some(Player::White));
        b.act(&Action::Double);

        assert_eq!(b.state(), State::Doubled);
        assert_eq!(b.player, Some(Player::Black));
        b.act(&Action::Take);

        assert_eq!(b.player, Some(Player::White));
        assert_eq!(b.state(), State::ToRoll);
        assert_eq!(
            b.cube,
            Cube {
                position: Some(Player::Black),
                level: 1,
                max_level: Cube::DEFAULT_MAX_LEVEL,
                doubled: false
            }
        )
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

    #[test]
    fn xgid_test() {
        let b = Board::new();
        assert_eq!(
            b.xgid(),
            "XGID=-b----E-C---eE---c-e----B---:0:0:::0:0:0:1:10"
        );

        let id = "XGID=-b----E-C---eE---c-e----B---:1:1:1:11:1:2:1:3:10";
        let b = Board::from_xgid(id);
        assert_eq!(b.xgid(), id);
    }
    #[test]
    fn moves() {
        let b = Board::from_xgid("XGID=-A----------------------a-:0:0:1:11:0:0:0:1:10");
        let moves = b.moves();
        println!("{:?}", moves);
        assert_eq!(moves, vec![Move(vec![(1, 0, false)])]);

        let b = Board::from_xgid("XGID=-a------------------A-----:0:0:1:11:0:0:0:1:10");
        print!("{}", b);
        let moves = b.moves();
        println!("{:?}", moves);
        assert_eq!(
            moves,
            vec![Move(vec![
                (20, 19, false),
                (19, 18, false),
                (18, 17, false),
                (17, 16, false),
            ])]
        );
    }
    #[test]
    fn reverse() {
        let ps = Pieces(
            vec![
                1, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 5, -6,
                -7,
            ]
            .into_iter()
            .map(Piece)
            .collect(),
        );
        let r = ps.reverse();
        assert_eq!(
            r.0,
            vec![
                -6, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 2, -7, 1,
                5
            ]
            .into_iter()
            .map(Piece)
            .collect::<Vec<_>>()
        )
    }
}
