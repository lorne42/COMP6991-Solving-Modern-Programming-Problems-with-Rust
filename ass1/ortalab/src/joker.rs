use std::collections::{HashMap, HashSet};
use ortalib::{Chips, Mult, Card, Rank, Suit, JokerCard, Joker, Edition, Enhancement};

/// Compares two suits for equality, with special behavior if 'smeared' is true.
/// When smeared, red suits and black suits are considered equivalent.
pub fn smeared_eq_suit(a: Suit, b: Suit, smeared: bool) -> bool {
    if !smeared {
        return a == b;
    }
    use Suit::*;
    match (a, b) {
        (Hearts, Diamonds) | (Diamonds, Hearts) => true,
        (Spades, Clubs) | (Clubs, Spades) => true,
        _ => a == b,
    }
}

/// Applies global joker effects based on the hand structure.
pub fn apply_jokers(
    jokers: &[JokerCard],
    played_cards: &[Card],
    chips: &mut Chips,
    mult_add: &mut Mult,
    mult_mult: &mut Mult,
    explain: bool,
) {
    let mut rank_counts: HashMap<Rank, usize> = HashMap::new();
    for card in played_cards {
        *rank_counts.entry(card.rank).or_default() += 1;
    }
    let mut counts: Vec<usize> = rank_counts.values().copied().collect();
    counts.sort_by(|a, b| b.cmp(a));

    let contains_pair = counts.iter().filter(|&&c| c >= 2).count() >= 1;
    let contains_two_pair = counts.iter().filter(|&&c| c == 2).count() >= 2;
    let contains_three = counts.iter().any(|&c| c >= 3);

    let contains_straight = !crate::find_straight(played_cards).is_empty();
    let contains_flush = !crate::find_flush(played_cards).is_empty();

    // Used by Abstract Joker to scale its bonus.
    let count = jokers.len();

    for joker in jokers {
        let name = format!("{:?}", joker.joker);
        match joker.joker {
            // Handle different joker types with associated bonuses
            Joker::Joker => {
                *mult_add += 4.0;
                if explain {
                    println!("Joker +4 Mult");
                }
            }
            Joker::JollyJoker if contains_pair => {
                *mult_add += 8.0;
                if explain {
                    println!("Jolly Joker +8 Mult (Pair)");
                }
            }
            Joker::ZanyJoker if contains_three => {
                *mult_add += 12.0;
                if explain {
                    println!("Zany Joker +12 Mult (Three of a Kind)");
                }
            }
            Joker::MadJoker if contains_two_pair => {
                *mult_add += 10.0;
                if explain {
                    println!("Mad Joker +10 Mult (Two Pair)");
                }
            }
            Joker::CrazyJoker if contains_straight => {
                *mult_add += 12.0;
                if explain {
                    println!("Crazy Joker +12 Mult (Straight)");
                }
            }
            Joker::DrollJoker if contains_flush => {
                *mult_add += 10.0;
                if explain {
                    println!("Droll Joker +10 Mult (Flush)");
                }
            }
            Joker::SlyJoker if contains_pair => {
                *chips += 50.0;
                if explain {
                    println!("Sly Joker +50 Chips (Pair)");
                }
            }
            Joker::WilyJoker if contains_three => {
                *chips += 100.0;
                if explain {
                    println!("Wily Joker +100 Chips (Three of a Kind)");
                }
            }
            Joker::CleverJoker if contains_two_pair => {
                *chips += 80.0;
                if explain {
                    println!("Clever Joker +80 Chips (Two Pair)");
                }
            }
            Joker::DeviousJoker if contains_straight => {
                *chips += 100.0;
                if explain {
                    println!("Devious Joker +100 Chips (Straight)");
                }
            }
            Joker::CraftyJoker if contains_flush => {
                *chips += 80.0;
                if explain {
                    println!("Crafty Joker +80 Chips (Flush)");
                }
            }
            Joker::AbstractJoker => {
                let bonus = (3 * count) as f64;
                *mult_add += bonus;
                if explain {
                    println!("Abstract Joker +{} Mult ({} jokers)", bonus, count);
                }
            }
            _ => {}
        }

        // Apply edition-based enhancements
        if let Some(edition) = &joker.edition {
            match edition {
                Edition::Foil => {
                    *chips += 50.0;
                    if explain {
                        println!("{} Foil +50 Chips", name);
                    }
                }
                Edition::Holographic => {
                    *mult_add += 10.0;
                    if explain {
                        println!("{} Holographic +10 Mult", name);
                    }
                }
                Edition::Polychrome => {
                    *mult_mult *= 1.5;
                    if explain {
                        println!("{} Polychrome x1.5 Mult", name);
                    }
                }
            }
        }
    }
}

/// Context passed to apply_scored_jokers, used to track state across scoring calls.
pub struct JokerStatusContext<'a> {
    /// Whether this is the first face card encountered.
    pub is_first_face: &'a mut bool,
    /// Whether a face card has already triggered Photograph bonus.
    pub face_boosted: &'a mut bool,
    /// Set of suits seen so far, used for Flower Pot and smeared logic.
    pub suit_tracker: &'a mut HashSet<Suit>,
}


/// Applies per-card joker effects during scoring.
/// This function uses JokerStatusContext to track face cards and suit diversity.
pub fn apply_scored_jokers(
    jokers: &[JokerCard],
    card: &Card,
    chips: &mut Chips,
    mult_add: &mut Mult,
    mult_mult: &mut Mult,
    explain: bool,
    status: &mut JokerStatusContext,
) {
    // Determine card properties
    let is_face = crate::is_face_card(card, crate::has_joker(jokers, Joker::Pareidolia));
    let is_even = matches!(card.rank, Rank::Two | Rank::Four | Rank::Six | Rank::Eight | Rank::Ten);
    let is_odd = matches!(card.rank, Rank::Ace | Rank::Three | Rank::Five | Rank::Seven | Rank::Nine);

    // Check if Smeared Joker is active
    let has_smeared = jokers.iter().any(|j| j.joker == Joker::SmearedJoker);

    // Update suit tracker if suit is new (considering smear logic)
    if !status.suit_tracker.iter().any(|&s| smeared_eq_suit(s, card.suit, has_smeared)) {
        status.suit_tracker.insert(card.suit);
    }

    // Apply individual joker effects
    for joker in jokers {
        match joker.joker {
            Joker::LustyJoker if smeared_eq_suit(card.suit, Suit::Hearts, has_smeared) || is_wild(card) => {
                *mult_add += 3.0;
                if explain { println!("Lusty Joker +3 Mult (♥)"); }
            }
            Joker::GreedyJoker if  smeared_eq_suit(card.suit, Suit::Diamonds, has_smeared) || is_wild(card) => {
                *mult_add += 3.0;
                if explain { println!("Greedy Joker +3 Mult (♦)"); }
            }
            Joker::WrathfulJoker if smeared_eq_suit(card.suit, Suit::Spades, has_smeared) || is_wild(card) => {
                *mult_add += 3.0;
                if explain { println!("Wrathful Joker +3 Mult (♠)"); }
            }
            Joker::GluttonousJoker if smeared_eq_suit(card.suit, Suit::Clubs, has_smeared) || is_wild(card) => {
                *mult_add += 3.0;
                if explain { println!("Gluttonous Joker +3 Mult (♣)"); }
            }
            Joker::Fibonacci if matches!(card.rank, Rank::Ace | Rank::Two | Rank::Three | Rank::Five | Rank::Eight) => {
                *mult_add += 8.0;
                if explain { println!("Fibonacci +8 Mult"); }
            }
            Joker::ScaryFace if is_face => {
                *chips += 30.0;
                if explain { println!("Scary Face +30 Chips (face card)"); }
            }
            Joker::EvenSteven if is_even => {
                *mult_add += 4.0;
                if explain { println!("Even Steven +4 Mult"); }
            }
            Joker::OddTodd if is_odd => {
                *chips += 31.0;
                if explain { println!("Odd Todd +31 Chips"); }
            }
            Joker::SmileyFace if is_face => {
                *mult_add += 5.0;
                if explain { println!("Smiley Face +5 Mult"); }
            }
            Joker::Photograph if is_face && *status.is_first_face && !*status.face_boosted => {
                *mult_mult *= 2.0;
                *status.face_boosted = true;
                if explain { println!("Photograph x2 Mult (first face card)"); }
            }
            _ => {}
        }
    }

    // Mark first face card handled
    if is_face {
        *status.is_first_face = false;
    }
}

/// Applies joker effects for cards held in hand (not played).
pub fn apply_held_jokers(
    jokers: &[JokerCard],
    held_cards: &[Card],
    mult_add: &mut Mult,
    mult_mult: &mut Mult,
    explain: bool,
) {
    // Raised Fist: boost from lowest card in hand
    if let Some(min_card) = held_cards.iter().min_by_key(|c| c.rank as u8) {
        for joker in jokers {
            if let Joker::RaisedFist = joker.joker {
                let bonus = min_card.rank.rank_value() * 2.0;
                *mult_add += bonus;
                if explain {
                    println!("Raised Fist +{} Mult (lowest rank in hand)", bonus);
                }
            }
        }
    }

    // Baron: check for King in hand
    for card in held_cards {
        if matches!(card.rank, Rank::King) {
            for joker in jokers {
                if let Joker::Baron = joker.joker {
                    *mult_mult *= 1.5;
                    if explain {
                        println!("Baron x1.5 Mult (King in hand)");
                    }
                }
            }
        }
    }

    // Blackboard: all cards are black or Wild
    let all_black = held_cards.iter().all(|c| matches!(c.suit, Suit::Spades | Suit::Clubs) || is_wild(c));
    if all_black {
        for joker in jokers {
            if let Joker::Blackboard = joker.joker {
                *mult_mult *= 3.0;
                if explain {
                    println!("Blackboard x3 Mult (all black cards or empty)");
                }
            }
        }
    }
}

/// Checks whether a card has the Wild enhancement.
fn is_wild(card: &Card) -> bool {
    matches!(card.enhancement, Some(Enhancement::Wild))
}

/// Special handling for Sock and Buskin Joker when a face card is played.
pub fn apply_sock_and_buskin(
    jokers: &[JokerCard],
    card: &Card,
    chips: &mut Chips,
    mult: &mut Mult,
    mult_mult: &mut Mult,
    explain: bool,
    is_face: bool,
) {
    // Local scoring context just for this effect
    let mut is_first_face = false;
    let mut face_boosted = false;
    let mut suit_tracker = HashSet::new();

    let mut status = JokerStatusContext {
        is_first_face: &mut is_first_face,
        face_boosted: &mut face_boosted,
        suit_tracker: &mut suit_tracker,
    };

    if is_face && crate::has_joker(jokers, Joker::SockAndBuskin) {
        crate::apply_modifiers(card, false, chips, mult, mult_mult, explain, &[]);
        crate::apply_scored_jokers(
            jokers,
            card,
            chips,
            mult,
            mult_mult,
            explain,
            &mut status,
        );
    }
}
