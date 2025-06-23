use std::{
    error::Error,
    fs::File,
    io::{Read, stdin},
    path::{Path, PathBuf},
};

use clap::Parser;
use std::collections::HashSet;

use ortalib::{JokerCard, Joker, Round, Card, PokerHand, Rank};

mod modifier;
mod joker;

use crate::joker::{apply_jokers, apply_scored_jokers, apply_held_jokers, apply_sock_and_buskin, JokerStatusContext};
use modifier::{apply_modifiers, card_to_explain_string, hand_display_name, identify_best_hand};

#[derive(Parser)]
struct Opts {
    file: PathBuf,

    #[arg(long)]
    explain: bool,
}

pub fn has_joker(jokers: &[JokerCard], target: Joker) -> bool {
    jokers.iter().any(|j| j.joker == target)
}

pub fn is_face_card(card: &Card, has_pareidolia: bool) -> bool {
    has_pareidolia || matches!(card.rank, Rank::Jack | Rank::Queen | Rank::King)
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::parse();
    let round = parse_round(&opts)?;
    let (hand, best_cards) = identify_best_hand(&round.cards_played);
    let main_cards = get_main_cards(hand, &best_cards);

    let (base_chips, base_mult) = hand.hand_value();
    let mut mult_mult = 1.0;
    let mut chips = base_chips;
    let mut mult = base_mult;
    let mut mult_add = 0.0;
    let mut mult_mult_joker = 1.0;
    let mut first_face = true;
    let mut face_boosted = false;
    let mut suit_tracker = HashSet::new();

    let mut status = JokerStatusContext {
        is_first_face: &mut first_face,
        face_boosted: &mut face_boosted,
        suit_tracker: &mut suit_tracker,
    };

    let has_pareidolia = has_joker(&round.jokers, Joker::Pareidolia);

    if opts.explain {
        println!("{} ({} x {})", hand_display_name(hand), base_chips, base_mult);
    }

    let has_splash = has_joker(&round.jokers, Joker::Splash);

    let scoring_cards = if has_splash {
        &round.cards_played
    } else {
        &main_cards
    };

    for card in scoring_cards {
        let value = card.rank.rank_value();
        chips += value;

        if opts.explain {
            println!(
                "{} +{} Chips ({} x ({} x {}))",
                card_to_explain_string(card, &round.cards_played),
                value,
                chips,
                mult,
                mult_mult
            );
        }

        apply_modifiers(
            card,
            false,
            &mut chips,
            &mut mult,
            &mut mult_mult,
            opts.explain,
            &round.cards_played,
        );

        apply_scored_jokers(
            &round.jokers,
            card,
            &mut chips,
            &mut mult,
            &mut mult_mult,
            opts.explain,
            &mut status,
        );

        apply_sock_and_buskin(
            &round.jokers,
            card,
            &mut chips,
            &mut mult,
            &mut mult_mult,
            opts.explain,
            has_pareidolia,
        );
    }

    for card in &main_cards {
        suit_tracker.insert(card.suit);
    }

    if suit_tracker.len() == 4 && round.jokers.iter().any(|j| matches!(j.joker, Joker::FlowerPot)) {
        mult_mult *= 3.0;
        if opts.explain {
            println!("Flower Pot x3 Mult (all suits)");
        }
    }

    for card in &round.cards_held_in_hand {
        apply_modifiers(
            card,
            true,
            &mut chips,
            &mut mult,
            &mut mult_mult,
            opts.explain,
            &round.cards_played,
        );
    }

    apply_jokers(
        &round.jokers,
        &round.cards_played,
        &mut chips,
        &mut mult_add,
        &mut mult_mult_joker,
        opts.explain,
    );

    apply_held_jokers(
        &round.jokers,
        &round.cards_held_in_hand,
        &mut mult_add,
        &mut mult_mult,
        opts.explain,
    );

    let final_score = (chips * (mult * mult_mult + mult_add) * mult_mult_joker).floor();
    println!("{}", final_score);

    Ok(())
}

fn parse_round(opts: &Opts) -> Result<Round, Box<dyn Error>> {
    let mut input = String::new();
    if opts.file == Path::new("-") {
        stdin().read_to_string(&mut input)?;
    } else {
        File::open(&opts.file)?.read_to_string(&mut input)?;
    }

    let round = serde_yaml::from_str(&input)?;
    Ok(round)
}

fn get_main_cards(hand: PokerHand, cards: &[Card]) -> Vec<Card> {
    use itertools::Itertools;

    // Group cards by rank
    let rank_groups = cards.iter().cloned().into_group_map_by(|c| c.rank);

    match hand {
        PokerHand::HighCard => cards.iter()
            .max_by_key(|c| c.rank as u8)
            .into_iter()
            .cloned()
            .collect(),

        PokerHand::Pair => rank_groups.into_iter()
            .find(|(_, g)| g.len() == 2)
            .map(|(_, g)| g)
            .unwrap_or_default(),

        PokerHand::TwoPair => cards.iter()
            .cloned()
            .into_group_map_by(|c| c.rank)
            .into_iter()
            .filter(|(_, g)| g.len() == 2)
            .sorted_by_key(|(r, _)| *r)
            .rev()
            .take(2)
            .flat_map(|(_, g)| g)
            .collect(),

        PokerHand::ThreeOfAKind => cards.iter()
            .cloned()
            .into_group_map_by(|c| c.rank)
            .into_iter()
            .find(|(_, g)| g.len() == 3)
            .map(|(_, g)| g)
            .unwrap_or_default(),

        PokerHand::FourOfAKind => cards.iter()
            .cloned()
            .into_group_map_by(|c| c.rank)
            .into_iter()
            .find(|(_, g)| g.len() == 4)
            .map(|(_, g)| g)
            .unwrap_or_default(),

        PokerHand::FiveOfAKind => cards.iter()
            .cloned()
            .into_group_map_by(|c| c.rank)
            .into_iter()
            .find(|(_, g)| g.len() == 5)
            .map(|(_, g)| g)
            .unwrap_or_default(),

        PokerHand::FullHouse => {
            let grouped = cards.iter().cloned().into_group_map_by(|c| c.rank);
            let three = grouped.iter()
                .find(|(_, g)| g.len() == 3)
                .map(|(_, g)| g.clone())
                .unwrap_or_default();
            let two = grouped.iter()
                .find(|(_, g)| g.len() == 2)
                .map(|(_, g)| g.clone())
                .unwrap_or_default();
            [three, two].concat()
        }

        PokerHand::Straight => find_straight(cards),
        PokerHand::Flush => find_flush(cards),
        PokerHand::StraightFlush => {
            let flush_cards = find_flush(cards);
            find_straight(&flush_cards)
        }

        PokerHand::FlushFive => find_flush(cards),

        PokerHand::FlushHouse => {
            let flush_cards = find_flush(cards);
            let grouped = flush_cards.iter().cloned().into_group_map_by(|c| c.rank);
            let three = grouped.iter()
                .find(|(_, g)| g.len() == 3)
                .map(|(_, g)| g.clone())
                .unwrap_or_default();
            let two = grouped.iter()
                .find(|(_, g)| g.len() == 2)
                .map(|(_, g)| g.clone())
                .unwrap_or_default();
            [three, two].concat()
        }
    }
}

fn find_flush(cards: &[Card]) -> Vec<Card> {
    use itertools::Itertools;

    cards.iter()
        .cloned()
        .into_group_map_by(|c| c.suit)
        .into_iter()
        .find(|(_, group)| group.len() >= 5)
        .map(|(_, group)| group.into_iter().take(5).collect())
        .unwrap_or_default()
}

fn find_straight(cards: &[Card]) -> Vec<Card> {
    use itertools::Itertools;
    use std::collections::HashSet;

    // Remove duplicates by rank
    let mut unique_cards = cards.iter().cloned()
        .unique_by(|c| c.rank)
        .collect::<Vec<_>>();

    // Sort by rank
    unique_cards.sort_by_key(|c| c.rank as u8);

    // Check for standard straights
    for window in unique_cards.windows(5).rev() {
        let ranks: Vec<u8> = window.iter().map(|c| c.rank as u8).collect();
        if ranks.windows(2).all(|w| w[1] == w[0] + 1) {
            return window.to_vec();
        }
    }

    // Special case: A-2-3-4-5
    let rank_set: HashSet<_> = unique_cards.iter().map(|c| c.rank).collect();
    let required = [Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Ace];

    if required.iter().all(|r| rank_set.contains(r)) {
        return unique_cards.iter()
            .filter(|c| required.contains(&c.rank))
            .cloned()
            .take(5)
            .collect();
    }

    vec![]
}