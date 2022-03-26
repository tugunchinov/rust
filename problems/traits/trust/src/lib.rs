#![forbid(unsafe_code)]

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundOutcome {
    BothCooperated,
    LeftCheated,
    RightCheated,
    BothCheated,
}

#[derive(Clone, Copy)]
pub enum AgentOutcome {
    Cooperated,
    Cheated,
}

pub trait Agent {
    fn get_score(&self) -> i32;
    fn make_decision(&mut self) -> AgentOutcome;
    fn change_score(&mut self, delta: i32);
}

pub struct Game {
    left: Box<dyn Agent>,
    right: Box<dyn Agent>,
}

impl Game {
    pub fn new(left: Box<dyn Agent>, right: Box<dyn Agent>) -> Self {
        Self { left, right }
    }

    pub fn left_score(&self) -> i32 {
        self.left.get_score()
    }

    pub fn right_score(&self) -> i32 {
        self.right.get_score()
    }

    pub fn play_round(&mut self) -> RoundOutcome {
        let left_outcome = self.left.make_decision();
        let right_outcome = self.right.make_decision();

        match (left_outcome, right_outcome) {
            (AgentOutcome::Cooperated, AgentOutcome::Cooperated) => {
                self.left.change_score(2);
                self.right.change_score(2);
                RoundOutcome::BothCooperated
            }
            (AgentOutcome::Cooperated, AgentOutcome::Cheated) => {
                self.left.change_score(-1);
                self.right.change_score(3);
                RoundOutcome::RightCheated
            }
            (AgentOutcome::Cheated, AgentOutcome::Cooperated) => {
                self.left.change_score(3);
                self.right.change_score(-1);
                RoundOutcome::LeftCheated
            }
            (AgentOutcome::Cheated, AgentOutcome::Cheated) => {
                self.left.change_score(0);
                self.right.change_score(0);
                RoundOutcome::BothCheated
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct CheatingAgent {
    score: i32,
}

impl Agent for CheatingAgent {
    fn get_score(&self) -> i32 {
        self.score
    }

    fn make_decision(&mut self) -> AgentOutcome {
        AgentOutcome::Cheated
    }

    fn change_score(&mut self, delta: i32) {
        self.score += delta
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct CooperatingAgent {
    score: i32,
}

impl Agent for CooperatingAgent {
    fn get_score(&self) -> i32 {
        self.score
    }

    fn make_decision(&mut self) -> AgentOutcome {
        AgentOutcome::Cooperated
    }

    fn change_score(&mut self, delta: i32) {
        self.score += delta
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct GrudgerAgent {
    score: i32,
    betrayed: bool,
}

impl Agent for GrudgerAgent {
    fn get_score(&self) -> i32 {
        self.score
    }

    fn make_decision(&mut self) -> AgentOutcome {
        if self.betrayed {
            AgentOutcome::Cheated
        } else {
            AgentOutcome::Cooperated
        }
    }

    fn change_score(&mut self, delta: i32) {
        if delta < 0 {
            self.betrayed = true;
        }
        self.score += delta
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct CopycatAgent {
    score: i32,
    next_decision: AgentOutcome,
}

impl Default for CopycatAgent {
    fn default() -> Self {
        Self {
            score: 0,
            next_decision: AgentOutcome::Cooperated,
        }
    }
}

impl Agent for CopycatAgent {
    fn get_score(&self) -> i32 {
        self.score
    }

    fn make_decision(&mut self) -> AgentOutcome {
        self.next_decision
    }

    fn change_score(&mut self, delta: i32) {
        if delta > 0 {
            self.next_decision = AgentOutcome::Cooperated;
        } else {
            self.next_decision = AgentOutcome::Cheated;
        }

        self.score += delta
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct DetectiveAgent {
    score: i32,
    next_decision: AgentOutcome,
    turn: usize,
    betrayed: bool,
}

impl Default for DetectiveAgent {
    fn default() -> Self {
        Self {
            score: 0,
            next_decision: AgentOutcome::Cooperated,
            turn: 0,
            betrayed: false,
        }
    }
}

impl Agent for DetectiveAgent {
    fn get_score(&self) -> i32 {
        self.score
    }

    fn make_decision(&mut self) -> AgentOutcome {
        self.turn += 1;
        self.next_decision
    }

    fn change_score(&mut self, delta: i32) {
        match self.turn {
            1 => {
                self.next_decision = AgentOutcome::Cheated;

                if delta <= 0 {
                    self.betrayed = true;
                }
            }

            2 | 3 => {
                self.next_decision = AgentOutcome::Cooperated;

                if delta <= 0 {
                    self.betrayed = true;
                }
            }

            _ => {
                if self.betrayed {
                    if delta > 0 {
                        self.next_decision = AgentOutcome::Cooperated;
                    } else {
                        self.next_decision = AgentOutcome::Cheated;
                    }
                } else {
                    self.next_decision = AgentOutcome::Cheated;
                };
            }
        }

        self.score += delta
    }
}
