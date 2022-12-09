use models::{Basket, CpuId, Hand, HandId, Suit, Rank, DECK_SIZE};

#[derive(Copy, Clone, Debug, Default)]
enum Location {
    #[default]
    Unknown,
    Likely(HandId),
    Known(HandId),
    /// discard pile or in a full basket.
    KnownGone,
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
                    dbg!(self.locations, rank, suit);
                    return Some((suit, id));
                },
                Location::Unknown
                | Location::Likely(_)
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
        // TODO? randomize order of suits? Prioritize them somehow?
        for suit in Suit::ALL {
            let location = self.locations[models::fish_card(rank, suit) as usize];
            match location {
                Location::Likely(id) if id != my_id => {
                    return Some((suit, id));
                },
                Location::Unknown
                | Location::Likely(_)
                | Location::Known(_)
                | Location::KnownGone => {},
            }
        }

        None
    }

    fn asked_for(&mut self, hand_id: HandId, rank: Rank, _asked_suit: Suit) {
        // TODO? Should we treat the card that was asked for differently?
        // The thing is one can ask for a card that one has to trip people up.
        // Maybe another Location variant is needed for that case?
        for suit in Suit::ALL {
            self.locations[models::fish_card(rank, suit) as usize] =
                Location::Likely(hand_id);
        }
    }

    fn found(&mut self, hand_id: HandId, rank: Rank, suit: Suit) {
        self.locations[models::fish_card(rank, suit) as usize] =
            Location::Known(hand_id);
    }

    fn fished_for(&mut self, hand_id: HandId, rank: Rank, suit: Suit) {
        self.locations[models::fish_card(rank, suit) as usize] =
            Location::Known(hand_id);
    }

    fn basket_removed(&mut self, basket: Basket) {
        for card in basket {
            self.locations[card as usize] = Location::KnownGone;
        }
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

    const R: Rank = models::ranks::BARNACLE;

    #[test]
    fn on_this_one_found_red_example() {
        let mut my_hand = Hand::default();
        my_hand.push(models::fish_card(R, Suit::Green));

        let my_id = HandId::Cpu1;
        let other_id = HandId::Cpu2;

        let mut mem = Memory::default();

        mem.found(other_id, R, Suit::Red);

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

    fn memory_mut(&mut self, id: CpuId) -> &mut Memory {
        match id {
            CpuId::One => &mut self.cpu1,
            CpuId::Two => &mut self.cpu2,
            CpuId::Three => &mut self.cpu3,
        }
    }

    pub fn asked_for(&mut self, hand_id: HandId, rank: Rank, asked_suit: Suit) {
        for cpu_id in CpuId::ALL {
            self.memory_mut(cpu_id).asked_for(hand_id, rank, asked_suit);
        }
    }

    pub fn found(&mut self, hand_id: HandId, rank: Rank, suit: Suit) {
        for cpu_id in CpuId::ALL {
            self.memory_mut(cpu_id).found(hand_id, rank, suit);
        }
    }

    pub fn fished_for(&mut self, hand_id: HandId, rank: Rank, suit: Suit) {
        for cpu_id in CpuId::ALL {
            self.memory_mut(cpu_id).fished_for(hand_id, rank, suit);
        }
    }

    pub fn basket_removed(&mut self, basket: Basket) {
        for cpu_id in CpuId::ALL {
            self.memory_mut(cpu_id).basket_removed(basket);
        }
    }
}
