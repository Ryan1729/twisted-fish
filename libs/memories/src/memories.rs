use models::{Basket, Card, CpuId, Hand, HandId, NetPredicate, Predicate, Rank, Suit, DECK_SIZE};

/// It seems intuitive that counting an amount of asks larger than the amount of
/// suits would not be needed, but I don't have an explicitly worked out reason for
/// that.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum AskCount {
    One,
    Two,
    Three,
    Four,
    FivePlus,
}

impl AskCount {
    fn saturating_inc(self) -> Self {
        use AskCount::*;
        match self {
            One => Two,
            Two => Three,
            Three => Four,
            Four | FivePlus => FivePlus,
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
enum Evidence {
    #[default]
    Unknown,
    AskedForSimilar(AskCount),
    // TODO Allow marking that a player was asked for something and didn't have it.
    // Will need to have a way to clear that as well, once they've drawn a card.
    // TODO? Is it worth tracking how many cards they drew since they didn't have it?
    #[allow(dead_code)]
    DidNotHave
}

// TODO? Avoid storing an extra one for this player's own hand id? Or does the
// simpler indexing logic end up being a win overall?
type Incomplete = [Evidence; HandId::COUNT as _];

#[derive(Copy, Clone, Debug)]
enum Location {
    Incomplete(Incomplete),
    Known(HandId),
    /// discard pile or in a full basket.
    KnownGone,
}

impl Default for Location {
    fn default() -> Self {
        Self::Incomplete(Incomplete::default())
    }
}

#[derive(Clone)]
pub struct Memory {
    locations: [Location; DECK_SIZE as _],
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            locations: [Location::default(); models::DECK_SIZE as _],
        }
    }
}

impl Memory {
    fn question_for_known_card_with_rank(
        &self,
        rank: Rank,
        my_id: HandId
    ) -> Option<(Suit, HandId)> {
        // TODO? randomize order of suits? Prioritize them somehow?
        for suit in Suit::ALL {
            let location = self.locations[models::fish_card(rank, suit) as usize];
            match location {
                Location::Known(id) if id != my_id => {
                    return Some((suit, id));
                },
                Location::Incomplete(_)
                | Location::Known(_)
                | Location::KnownGone => {},
            }
        }

        None
    }

    fn question_for_likely_card_with_rank(
        &self,
        rank: Rank,
        my_id: HandId
    ) -> Option<(Suit, HandId)> {
        let mut best = None;
        // TODO? randomize order of suits? Prioritize them somehow?
        for suit in Suit::ALL {
            let location = self.locations[models::fish_card(rank, suit) as usize];
            match location {
                Location::Incomplete(incomplete) => {
                    let my_index = my_id as _;
                    for (i, evidence) in incomplete.iter().enumerate() {
                        if i == my_index { continue }

                        match (*evidence, best) {
                            (Evidence::AskedForSimilar(count), None) => {
                                best = Some((count, (suit, HandId::ALL[i])));
                            },
                            (Evidence::AskedForSimilar(count), Some((prev_count, _))) => {
                                if count > prev_count {
                                    best = Some((count, (suit, HandId::ALL[i])));
                                }
                            }
                            (Evidence::Unknown | Evidence::DidNotHave, _) => {}
                        }

                    }
                },
                | Location::Known(_)
                | Location::KnownGone => {},
            }
        }

        best.map(|(_, out)| out)
    }

    fn asked_for(&mut self, hand_id: HandId, predicate: Predicate) {
        use Predicate::*;
        match predicate {
            RankSuit(rank, _asked_suit) => {
                // TODO? Should we treat the card that was asked for differently?
                // The thing is one can ask for a card that one has to trip people up.
                // Maybe another Location variant is needed for that case?
                for suit in Suit::ALL {
                    let loc = &mut self.locations[models::fish_card(rank, suit) as usize];
                    match *loc {
                        Location::Known(_)
                        | Location::KnownGone => {},
                        Location::Incomplete(mut incomplete) => {
                            let i = hand_id as usize;
                            incomplete[i] = match incomplete[i] {
                                Evidence::Unknown
                                | Evidence::DidNotHave => Evidence::AskedForSimilar(AskCount::One),
                                Evidence::AskedForSimilar(count) => {
                                    Evidence::AskedForSimilar(count.saturating_inc())
                                }
                            };
        
                            *loc = Location::Incomplete(incomplete);
                        },
                    }
                }
            },
            Net(net_predicate) => {
                match net_predicate {
                    NetPredicate::Rank(rank) => {
                        // TODO is the information gained here different than if the 
                        // predicate was not used?
                        for suit in Suit::ALL {
                            let loc = &mut self.locations[models::fish_card(rank, suit) as usize];
                            match *loc {
                                Location::Known(_)
                                | Location::KnownGone => {},
                                Location::Incomplete(mut incomplete) => {
                                    let i = hand_id as usize;
                                    incomplete[i] = match incomplete[i] {
                                        Evidence::Unknown
                                        | Evidence::DidNotHave => Evidence::AskedForSimilar(AskCount::One),
                                        Evidence::AskedForSimilar(count) => {
                                            Evidence::AskedForSimilar(count.saturating_inc())
                                        }
                                    };
                
                                    *loc = Location::Incomplete(incomplete);
                                },
                            }
                        }
                    }
                    NetPredicate::Suit(_suit) => {
                        // TODO Does this display any information worth recording?
                    }
                }
            },
        }
    }

    pub fn known(&mut self, hand_id: HandId, card: Card) {
        self.locations[card as usize] = Location::Known(hand_id);
    }

    fn basket_removed(&mut self, basket: Basket) {
        for card in basket {
            self.locations[card as usize] = Location::KnownGone;
        }
    }

    pub fn likely_to_fill_basket_soon(&self, target_id: HandId) -> Option<Rank> {
        // Do high scoring ranks first so we will return them when there are 
        // multiple options.
        for &rank in Rank::ALL.iter().rev() {
            if self.is_likely_to_fill_rank_soon(target_id, rank) {
                return Some(rank)
            }
        }

        None
    }

    pub fn is_likely_to_fill_rank_soon(
        &self,
        target_id: HandId,
        rank: Rank,
    ) -> bool {
        const KNOWN_SCORE: u32 = 3;
        let mut score = 0;
        for suit in Suit::ALL {
            use Location::*;
            use Evidence::*;

            match self.locations[models::fish_card(rank, suit) as usize] {
                Incomplete(incomplete) => match incomplete[target_id as usize] {
                    Unknown | DidNotHave => {},
                    AskedForSimilar(AskCount::One | AskCount::Two) => {
                        score += 1;    
                    },
                    AskedForSimilar(_) => {
                        score += 2;    
                    },
                },
                Known(id) if id == target_id => {
                    score += KNOWN_SCORE;
                },
                Known(_) => {},
                KnownGone => break,
            }   
        }

        // TODO? check this actually produces the behaviour we want?
        // How important is this actually?
        score >= KNOWN_SCORE * (Suit::ALL.len() - 2) as u32
    }

    pub fn informed_question(
        &self,
        my_hand: &Hand,
        my_hand_id: HandId
    ) -> Option<(Rank, Suit, HandId)> {
        // TODO? maybe prioritize questions which
        // are known to result in full baskets?
        // TODO Avoid asking for cards you just successfully got from
        // another player's hand, but still ask for cards you have
        // sometimes, to throw others off.
        for card in my_hand.iter() {
            if let Some(rank) = models::get_rank(card) {
                let question = self.question_for_known_card_with_rank(
                    rank,
                    my_hand_id,
                );

                if question.is_some() {
                    return question.map(|(suit, id)| (rank, suit, id))
                }
            }
        }

        for card in my_hand.iter() {
            if let Some(rank) = models::get_rank(card) {
                let question = self.question_for_likely_card_with_rank(
                    rank,
                    my_hand_id,
                );

                if question.is_some() {
                    return question.map(|(suit, id)| (rank, suit, id))
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod informed_question_returns_the_expected_result {
    use super::*;

    const R: Rank = models::Rank::Barnacle;
    //const R2: Rank = models::Rank::Crab;

    #[test]
    fn on_this_one_found_red_example() {
        let mut my_hand = Hand::default();
        my_hand.push(models::fish_card(R, Suit::Green));

        let my_id = HandId::Cpu1;
        let other_id = HandId::Cpu2;

        let mut mem = Memory::default();

        mem.known(other_id, models::fish_card(R, Suit::Red));

        assert_eq!(
            mem.informed_question(&my_hand, my_id),
            Some((R, Suit::Red, other_id))
        );
    }

    #[test]
    fn on_this_one_found_then_asked_example() {
        let mut my_hand = Hand::default();
        my_hand.push(models::fish_card(R, Suit::Green));

        let my_id = HandId::Cpu1;
        let other_id = HandId::Cpu2;
        let other_id_2 = HandId::Cpu3;

        let mut mem = Memory::default();

        mem.known(other_id, models::fish_card(R, Suit::Red));
        // This previously caused `other_id_2` to be asked instead of `other_id`!
        mem.asked_for(other_id_2, Predicate::RankSuit(R, Suit::Red));

        assert_eq!(
            mem.informed_question(&my_hand, my_id),
            Some((R, Suit::Red, other_id))
        );
    }
}

#[derive(Clone, Default)]
pub struct Memories {
    cpu1: Memory,
    cpu2: Memory,
    cpu3: Memory,
}

impl Memories {
    pub fn memory(&self, id: CpuId) -> &Memory {
        match id {
            CpuId::One => &self.cpu1,
            CpuId::Two => &self.cpu2,
            CpuId::Three => &self.cpu3,
        }
    }

    pub fn memory_mut(&mut self, id: CpuId) -> &mut Memory {
        match id {
            CpuId::One => &mut self.cpu1,
            CpuId::Two => &mut self.cpu2,
            CpuId::Three => &mut self.cpu3,
        }
    }

    pub fn asked_for(&mut self, hand_id: HandId, predicate: Predicate) {
        for cpu_id in CpuId::ALL {
            self.memory_mut(cpu_id).asked_for(hand_id, predicate);
        }
    }

    /// A player successfully got a card when asking and this was revealed to all.
    pub fn found(&mut self, hand_id: HandId, card: Card) {
        for cpu_id in CpuId::ALL {
            self.memory_mut(cpu_id).known(hand_id, card);
        }
    }

    pub fn basket_removed(&mut self, basket: Basket) {
        for cpu_id in CpuId::ALL {
            self.memory_mut(cpu_id).basket_removed(basket);
        }
    }
}
