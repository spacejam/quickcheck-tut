const LEADER_PING_INTERVAL: u64 = 8;

pub enum ElectionMessage {
    RequestVotes { epoch: u64 },
    GrantVote { epoch: u64 },
    Ping { epoch: u64 },
}
use ElectionMessage::*;

pub enum PeerState<P> {
    Init { since: u64 },
    Candidate { since: u64, votes: Vec<P> },
    Leader { last_ping_tx: u64 },
    Follower { leader: P, last_ping_rx: u64 },
}
use PeerState::*;

pub trait Transport {
    type Peer;
    type Message;
    fn send(&mut self, to: &Self::Peer, msg: Self::Message);
}

impl Transport for Vec<(u8, ElectionMessage)> {
    type Peer = u8;
    type Message = ElectionMessage;
    fn send(&mut self, to: &Self::Peer, msg: Self::Message) {
        self.push((*to, msg));
    }
}

pub trait Clock {
    type Time;
    fn time(&self) -> Self::Time;
}

impl Clock for u64 {
    type Time = u64;
    fn time(&self) -> u64 {
        *self
    }
}


pub trait Reactor {
    type Peer;
    type Message;
    fn receive(&mut self, from: Self::Peer, msg: Self::Message);
    fn tick(&mut self);
}

pub struct ElectionPeer<T> {
    pub state: PeerState<T>,
    pub peers: Vec<T>,
    pub epoch: u64,
    pub clock: Box<Clock<Time = u64>>,
    pub transport: Box<Transport<Peer = T, Message = ElectionMessage>>,
}

impl<T> Reactor for ElectionPeer<T>
where
    T: PartialEq,
{
    type Peer = T;
    type Message = ElectionMessage;

    fn receive(&mut self, from: Self::Peer, msg: Self::Message) {
        match msg {
            RequestVotes { epoch } => {
                if epoch > self.epoch {
                    self.epoch = epoch;

                    self.transport.send(&from, GrantVote { epoch: epoch });

                    self.state = Follower {
                        leader: from,
                        last_ping_rx: self.clock.time(),
                    };
                }
            }
            GrantVote { epoch } => {
                if epoch != self.epoch {
                    return;
                }
                let become_leader = match self.state {
                    Candidate { ref mut votes, .. } => {
                        if !votes.contains(&from) {
                            votes.push(from);
                        }
                        if votes.len() > (self.peers.len() / 2) {
                            true
                        } else {
                            false
                        }
                    }
                    Init { .. } | Leader { .. } | Follower { .. } => false,
                };

                if become_leader {
                    let now = self.clock.time();

                    self.state = Leader { last_ping_tx: now };

                    for ref peer in &self.peers {
                        self.transport.send(&peer, Ping { epoch: self.epoch });
                    }
                }
            }
            Ping { epoch } => {
                if epoch >= self.epoch {
                    match self.state {
                        Candidate { ref votes, .. } => {
                            assert!(votes.len() <= self.peers.len() / 2);
                        }
                        Leader { .. } => {
                            assert!(epoch > self.epoch);
                        }
                        Init { .. } | Follower { .. } => {}
                    }

                    self.epoch = epoch;

                    self.state = Follower {
                        leader: from,
                        last_ping_rx: self.clock.time(),
                    };
                }
            }
        }
    }

    fn tick(&mut self) {
        let now = self.clock.time();

        match self.state {
            Leader { last_ping_tx } => {
                if last_ping_tx + LEADER_PING_INTERVAL < now {
                    for ref peer in &self.peers {
                        self.transport.send(&peer, Ping { epoch: self.epoch });
                    }
                }
            }
            Candidate { since, .. } |
            Follower { last_ping_rx: since, .. } |
            Init { since } => {
                if since + LEADER_PING_INTERVAL < now {
                    self.epoch += 1;
                    self.state = Candidate {
                        votes: vec![],
                        since: now,
                    };

                    for ref peer in &self.peers {
                        self.transport.send(
                            &peer,
                            RequestVotes { epoch: self.epoch },
                        );
                    }
                }
            }
        }
    }
}
