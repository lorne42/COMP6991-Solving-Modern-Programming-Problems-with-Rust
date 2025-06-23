// modifiers.rs
use ortalib::{Card, Chips, Enhancement, Edition, Suit, PokerHand, Rank, Mult};
use itertools::Itertools;

/// Identify the best hand type from a set of cards.
/// Supports special illegal types (FlushFive, FlushHouse) and wild card logic.
pub fn identify_hand(cards: &[Card]) -> PokerHand {
    use itertools::Itertools;
    use std::collections::HashMap;

    if cards.is_empty() {
        return PokerHand::HighCard;
    }
    let is_flush = cards.iter().all(|c| c.suit == cards[0].suit);
    let mut ranks: Vec<u8> = cards.iter().map(|c| (c.rank as u8) + 2).collect();
    ranks.sort_unstable();

    let is_straight = ranks.len() >= 5 && (
        ranks.windows(5).any(|w| w.windows(2).all(|x| x[1] == x[0] + 1)) ||
        ranks.ends_with(&[2, 3, 4, 5, 14])
    );

    let mut rank_counts: HashMap<_, usize> = HashMap::new();
    for card in cards {
        *rank_counts.entry(card.rank).or_insert(0) += 1;
    }
    let counts: Vec<usize> = rank_counts.values().copied().sorted().rev().collect();

    // Handle illegal types first
    if cards.len() == 5 && is_flush && counts == [5] {
        return PokerHand::FlushFive;
    }
    if cards.len() == 5 && is_flush && counts == [3, 2] {
        return PokerHand::FlushHouse;
    }
    if counts == [5] {
        return PokerHand::FiveOfAKind;
    }

    // Handle standard types by ranking
    if cards.len() == 5 && is_flush && is_straight {
        PokerHand::StraightFlush
    } else if counts == [4, 1] || counts == [4] {
        PokerHand::FourOfAKind
    } else if counts == [3, 2] {
        PokerHand::FullHouse
    } else if is_flush && cards.len() >= 5 {
        PokerHand::Flush
    } else if is_straight {
        PokerHand::Straight
    } else if counts == [3, 1, 1] || counts == [3, 1] || counts == [3] {
        PokerHand::ThreeOfAKind
    } else if counts == [2, 2, 1] || counts == [2, 2] {
        PokerHand::TwoPair
    } else if counts == [2, 1, 1, 1] || counts == [2, 1, 1] || counts == [2, 1] || counts == [2] {
        PokerHand::Pair
    } else {
        PokerHand::HighCard
    }
}

/// Applies per-card modifier effects from enhancement and edition fields.
pub fn apply_modifiers(
    card: &Card,
    is_held: bool,
    chips: &mut Chips,
    mult: &mut Mult,
    mult_mult: &mut Mult,
    explain: bool,
    original_cards: &[Card],
) {
    let card_str = card_to_explain_string(card, original_cards);

    // Apply enhancement effects
    if let Some(enhancement) = &card.enhancement {
        match enhancement {
            Enhancement::Bonus => if !is_held {
                *chips += 30.0;
                if explain {
                    println!("{} Bonus +30 Chips ({} x ({} x {}))", card_str, chips, mult, mult_mult);
                }
            },
            Enhancement::Mult => if !is_held {
                *mult += 4.0;
                if explain {
                    println!("{} Mult +4 Mult ({} x ({} x {}))", card_str, chips, mult, mult_mult);
                }
            },
            Enhancement::Glass => if !is_held {
                *mult_mult *= 2.0;
                if explain {
                    println!("{} Glass x2 Mult ({} x ({} x {}))", card_str, chips, mult, mult_mult);
                }
            },
            Enhancement::Steel if is_held => {
                *mult_mult *= 1.5;
                if explain {
                    println!("{} Steel x1.5 Mult ({} x ({} x {}))", card_str, chips, mult, mult_mult);
                }
            },
            Enhancement::Wild if !is_held && explain => {
                println!("{} Wild (no effect on scoring)", card_str);
            },
            _ => {}
        }
    }

    // Apply edition effects
    if let Some(edition) = &card.edition {
        match edition {
            Edition::Foil => if !is_held {
                *chips += 50.0;
                if explain {
                    println!("{} Foil +50 Chips ({} x ({} x {}))", card_str, chips, mult, mult_mult);
                }
            },
            Edition::Holographic => if !is_held {
                *mult += 10.0;
                if explain {
                    println!("{} Holographic +10 Mult ({} x ({} x {}))", card_str, chips, mult, mult_mult);
                }
            },
            Edition::Polychrome => if !is_held {
                *mult_mult *= 1.5;
                if explain {
                    println!("{} Polychrome x1.5 Mult ({} x ({} x {}))", card_str, chips, mult, mult_mult);
                }
            },
        }
    }
}

/// Generate all card sets by replacing Wild cards with all suit possibilities.
pub fn generate_wild_variants(cards: &[Card]) -> Vec<Vec<Card>> {
    let mut wilds = vec![];
    let mut fixed = vec![];

    for card in cards {
        if let Some(Enhancement::Wild) = card.enhancement {
            wilds.push(*card);
        } else {
            fixed.push(*card);
        }
    }

    if wilds.is_empty() {
        return vec![cards.to_vec()];
    }

    let suits = [Suit::Hearts, Suit::Spades, Suit::Diamonds, Suit::Clubs];

    let wild_variants: Vec<Vec<Card>> = wilds
        .iter()
        .map(|c| {
            suits.iter().map(move |suit| {
                let mut clone = *c;
                clone.suit = *suit;
                clone
            }).collect::<Vec<Card>>()
        })
        .collect();

    wild_variants
        .into_iter()
        .multi_cartesian_product()
        .map(|combo| {
            let mut full = fixed.clone();
            full.extend(combo);
            full
        })
        .collect()
}

/// Identify the best hand among all wild card substitutions.
pub fn identify_best_hand(cards: &[Card]) -> (PokerHand, Vec<Card>) {
    generate_wild_variants(cards)
        .into_iter()
        .map(|variant| (identify_hand(&variant), variant))
        .max_by_key(|(hand, _)| *hand as u8)
        .unwrap_or((PokerHand::HighCard, cards.to_vec()))
}

/// Convert a card into a string representation, appending "(Wild)" if applicable.
pub fn card_to_explain_string(card: &Card, original_cards: &[Card]) -> String {
    let rank_str = match card.rank {
        Rank::Two => "2",
        Rank::Three => "3",
        Rank::Four => "4",
        Rank::Five => "5",
        Rank::Six => "6",
        Rank::Seven => "7",
        Rank::Eight => "8",
        Rank::Nine => "9",
        Rank::Ten => "10",
        Rank::Jack => "J",
        Rank::Queen => "Q",
        Rank::King => "K",
        Rank::Ace => "A",
    };

    let suit_str = match card.suit {
        Suit::Hearts => "♥",
        Suit::Spades => "♠",
        Suit::Diamonds => "♦",
        Suit::Clubs => "♣",
    };

    let is_wild = original_cards.iter().any(|c|
        c.rank == card.rank &&
        c.suit == card.suit &&
        c.enhancement == Some(Enhancement::Wild)
    );

    if is_wild {
        format!("{}{} (Wild)", rank_str, suit_str)
    } else {
        format!("{}{}", rank_str, suit_str)
    }
}

/// Return a human-readable name for a PokerHand.
pub fn hand_display_name(hand: PokerHand) -> &'static str {
    match hand {
        PokerHand::HighCard => "High Card",
        PokerHand::Pair => "Pair",
        PokerHand::TwoPair => "Two Pair",
        PokerHand::ThreeOfAKind => "Three Of A Kind",
        PokerHand::Straight => "Straight",
        PokerHand::Flush => "Flush",
        PokerHand::FullHouse => "Full House",
        PokerHand::FourOfAKind => "Four Of A Kind",
        PokerHand::StraightFlush => "Straight Flush",
        PokerHand::FiveOfAKind => "Five Of A Kind",
        PokerHand::FlushHouse => "Flush House",
        PokerHand::FlushFive => "Flush Five",
    }
}