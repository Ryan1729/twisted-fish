use models::{Basket, Card, CpuId, Hand, HandId, NetPredicate, Predicate, Rank, Suit, DECK_SIZE, append_card_text};

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

type PredicateSet = std::collections::BTreeSet<Predicate>;

#[derive(Clone, Debug, Default)]
struct HandKnowledge {
    did_not_have: PredicateSet
}

#[derive(Clone, Debug)]
pub struct Memory {
    locations: [Location; DECK_SIZE as _],
    hand_knowledge: [HandKnowledge; HandId::ALL.len()],
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            locations: [Location::default(); models::DECK_SIZE as _],
            hand_knowledge: core::array::from_fn(|_| <_>::default()),
        }
    }
}

impl Memory {
    pub fn append_debug_info(&self, output: &mut Vec<u8>) {
        use std::io::Write;

        // Locations
        for card_i in 0..self.locations.len() {
            let mut no_location_line_this_time = false;

            match self.locations[card_i] {
                Location::Known(hand_id) => {
                    append_card_text(output, card_i as Card);

                    output.extend_from_slice(
                        b" is in "
                    );

                    output.extend_from_slice(
                        hand_id.text()
                    );
                },
                Location::KnownGone => {
                    append_card_text(output, card_i as Card);

                    output.extend_from_slice(
                        b" is in the discard pile or in a full basket."
                    );
                },
                Location::Incomplete(incomplete) => {
                    for (hand_i, evidence) in incomplete.iter().enumerate() {
                        let mut no_incomplete_line_this_time = false;

                        let hand_id = HandId::ALL[hand_i];

                        match evidence {
                            Evidence::Unknown => {
                                // Nothing to say here
                                no_incomplete_line_this_time = true;
                            },
                            Evidence::AskedForSimilar(ask_count) => {
                                output.extend_from_slice(
                                    hand_id.text()
                                );

                                output.extend_from_slice(
                                    b" asked for similar to "
                                );

                                append_card_text(output, card_i as Card);

                                let _ = write!(
                                    output,
                                    " {ask_count:?} time{}",
                                    if *ask_count == AskCount::One { "" } else { "s" }
                                );
                            },
                            Evidence::DidNotHave => {
                                output.extend_from_slice(
                                    hand_id.text()
                                );

                                output.extend_from_slice(
                                    b" did not have "
                                );

                                append_card_text(output, card_i as Card);
                            }
                        }

                        if no_incomplete_line_this_time {
                            no_location_line_this_time = true;
                        } else {
                            output.push(b'\n');
                        }
                    }
                }
            }

            if !no_location_line_this_time {
                output.push(b'\n');
            }
        }

        output.push(b'\n');

        // Hand Knowledge
        for hand_id in HandId::ALL {
            let label = match hand_id {
                HandId::Player => b"Player".as_slice(),
                HandId::Cpu1 => b"Cpu1".as_slice(),
                HandId::Cpu2 => b"Cpu2".as_slice(),
                HandId::Cpu3 => b"Cpu3".as_slice(),
            };

            output.extend_from_slice(label);
            output.push(b' ');

            match &self.hand_knowledge[hand_id as usize].did_not_have {
                predicates => {
                    output.extend_from_slice(b"none of: \n");

                    for predicate in predicates {
                        output.extend_from_slice(b"    ");
                        predicate.append_debug_info(output);
                        output.push(b'\n');
                    }
                },
            }

            output.push(b'\n');
        }

        output.push(b'\n');
    }

    fn prioritized_suits(
        rank: Rank,
        my_hand: &Hand,
    ) -> [Suit; Suit::ALL.len()] {
        let mut suits = Suit::ALL;

        // Put the suits we don't have first, so we do the right thing when we have 4 of them.
        suits.sort_by_key(|&suit| {
            for card in my_hand.iter() {
                match (models::get_rank(card), models::get_suit(card)) {
                    (Some(r), Some(s)) if r == rank && s == suit => return true, // sort later
                    _ => {},
                }
            }

            false // sort earlier
        });

        suits
    }

    fn question_for_known_card_with_rank(
        &self,
        rank: Rank,
        my_hand: &Hand,
        my_id: HandId
    ) -> Option<(Suit, HandId)> {
        for suit in Memory::prioritized_suits(rank, my_hand) {
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
        my_hand: &Hand,
        my_id: HandId
    ) -> Option<(Suit, HandId)> {
        let mut best = None;

        for suit in Memory::prioritized_suits(rank, my_hand) {
            let card = models::fish_card(rank, suit);

            let location = self.locations[card as usize];
            match location {
                Location::Incomplete(incomplete) => {
                    let my_index = my_id as _;
                    'evidence: for (hand_i, evidence) in incomplete.iter().enumerate() {
                        if hand_i == my_index { continue }

                        // Skip the ones that we saw they didn't have before
                        match &self.hand_knowledge[hand_i].did_not_have {
                            predicates => {
                                for predicate in predicates {
                                    if predicate.matches(card) {
                                        continue 'evidence;
                                    }
                                }
                            },
                        }

                        match (*evidence, best) {
                            (Evidence::AskedForSimilar(count), None) => {
                                best = Some((count, (suit, HandId::ALL[hand_i])));
                            },
                            (Evidence::AskedForSimilar(count), Some((prev_count, _))) => {
                                if count > prev_count {
                                    best = Some((count, (suit, HandId::ALL[hand_i])));
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

        if best.is_none() {
            for suit in Memory::prioritized_suits(rank, my_hand) {
                let card = models::fish_card(rank, suit);
    
                let location = self.locations[card as usize];
                match location {
                    Location::Incomplete(incomplete) => {
                        let my_index = my_id as _;
                        for (hand_i, evidence) in incomplete.iter().enumerate() {
                            if hand_i == my_index { continue }
    
                            // This time, check the ones that we saw they didn't have before

                            match (*evidence, best) {
                                (Evidence::AskedForSimilar(count), None) => {
                                    best = Some((count, (suit, HandId::ALL[hand_i])));
                                },
                                (Evidence::AskedForSimilar(count), Some((prev_count, _))) => {
                                    if count > prev_count {
                                        best = Some((count, (suit, HandId::ALL[hand_i])));
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

    pub fn did_not_have(&mut self, hand_id: HandId, predicate: Predicate) {
        let knowledge = &mut self.hand_knowledge[hand_id as usize];

        knowledge.did_not_have.insert(predicate);
    }

    /// Update memory based on the fact that the given player drew a card
    pub fn drew_card(&mut self, _hand_id: HandId) {
        // TODO We could take completed baskets into account, and the known locations, and then count cards.
        
        // This used to be here, but became obsolete
        //self.hand_knowledge[hand_id as usize] = HandKnowledge::CouldBeAnything;
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

    pub fn is_unknown(
        &self,
        rank: Rank,
        suit: Suit,
    ) -> bool {
        use Location::*;
        use Evidence::*;

        match self.locations[models::fish_card(rank, suit) as usize] {
            Incomplete(incomplete) => {
                incomplete.iter().all(|x| matches!(x, Unknown))
            },
            _ => false,
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
                    my_hand,
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
                    my_hand,
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

#[derive(Clone, Debug, Default)]
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

    /// A (different) player asked for something.
    pub fn asked_for(&mut self, hand_id: HandId, predicate: Predicate) {
        for cpu_id in CpuId::ALL {
            self.memory_mut(cpu_id).asked_for(hand_id, predicate);
        }
    }

    /// A (different) player drew a new card we haven't seen.
    pub fn drew_card(&mut self, hand_id: HandId) {
        for cpu_id in CpuId::ALL {
            self.memory_mut(cpu_id).drew_card(hand_id);
        }
    }

    /// A player successfully got a card when asking and this was revealed to all.
    pub fn found(&mut self, hand_id: HandId, card: Card) {
        for cpu_id in CpuId::ALL {
            self.memory_mut(cpu_id).known(hand_id, card);
        }
    }

    /// A player was asked for a card but didn't have it, and this was revealed to all.
    pub fn did_not_have(&mut self, hand_id: HandId, predicate: Predicate) {
        for cpu_id in CpuId::ALL {
            self.memory_mut(cpu_id).did_not_have(hand_id, predicate);
        }
    }

    /// Note the basket was removed
    pub fn basket_removed(&mut self, basket: Basket) {
        for cpu_id in CpuId::ALL {
            self.memory_mut(cpu_id).basket_removed(basket);
        }
    }
}
