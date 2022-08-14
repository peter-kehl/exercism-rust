#![feature(generic_associated_types)]
#![feature(associated_type_bounds)]
//#![allow(unused)] // @TODO remove after debugging.

//use rayon::prelude::*;

mod abstra;

macro_rules! extra_assert {
    ($cond:expr $(,)?) => {{ /* compiler built-in */ }};
    ($cond:expr, $($arg:tt)+) => {{ /* compiler built-in */ }};
}

use abstra::{set::HashedSet, Set};
use std::{collections::HashMap, fmt, ops::Deref};

/// Each side of the equation contains one or more operands, all added together.
#[derive(Debug)]
struct Equation {
    left: Vec<Operand>,
    right: Vec<Operand>,
}

impl Equation {
    // Work around to move out of an iterator
    fn new(mut eq_sides: impl Iterator<Item = Vec<Operand>>) -> Self {
        let left = eq_sides.next().unwrap();
        let right = eq_sides.next().unwrap();
        assert!(
            eq_sides.next().is_none(),
            "Only the two above items expected, but more present."
        );
        Self { left, right }
    }
}

/// A *reversed* operand (strings of letters). Reversed to make it easier to handle magnitude
/// by magnitude, since we start from the lowest magnitude.
#[derive(Debug)]
struct Operand(Vec<char>);

impl Deref for Operand {
    type Target = Vec<char>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// `&mut` fields are mutated in functions, but reverted back if the functions returns false.
/// `letters` = available letters (not assigned yet). Somewhat auxilliary - used for asserts.
/// `digits` = available digits (not assigned yet). Somewhat auxilliary - used for asserts.
/// `pairs = (so far) assigned letters & digits. On success this will be the solution.
#[derive(Debug)]
struct Assigned<'a, CHARSET: Set<char> + fmt::Debug, U8SET: Set<u8> + fmt::Debug> {
    eq: &'a Equation,
    letters: &'a mut CHARSET,
    digits: &'a mut U8SET,
    pairs: &'a mut HashMap<char, u8>,
}

/// Carried over from the lower magnitude.
#[derive(Debug, Clone)]
struct MagnitudeSubtotals {
    left: u16,
    right: u16,
}

/// Rather than brute-force, go magnitude by magnitude, starting from the lowest magnitude (the last digit of each operand). How:
/// magnitude by magnitude: letters -> digit
/// magnitude carried over to the higher magnitude
/// higher magnitude:
/// if the result digit (for that magnitude) has an already assigned letter, check that the letter matches the existing assigned digit. Otherwise pick any of the available digits.
/// recurse (into the higher magnitude)
///
/// param `magnitude` = which decimal magnitude to assign (0 for the last digit, 1 for the second last, 2 for the third last...)
/// return whether succeeded (then use `assigned.pairs` as mutated).
fn assign_for_magnitude<CHARSET: Set<char> + fmt::Debug, U8SET: Set<u8> + fmt::Debug>(
    assigned: &mut Assigned<CHARSET, U8SET>,
    empty_charset: &CHARSET,
    magnitude: usize,
    subtotals_carried_over: MagnitudeSubtotals,
) -> bool {
    let mut left_letters_for_this_magnitude =
        all_letters_per_magnitude::<CHARSET>(&assigned.eq.left, magnitude, &empty_charset);
    let mut right_letters_for_this_magnitude =
        all_letters_per_magnitude::<CHARSET>(&assigned.eq.right, magnitude, &empty_charset);
    pick_and_assign_letter/*::<CHARSET, U8SET>*/(
        assigned,
        &empty_charset,
        magnitude,
        subtotals_carried_over,
        &mut CurrentMagnitudeLetters {
            left: &mut left_letters_for_this_magnitude,
            right: &mut right_letters_for_this_magnitude,
        },
        true,
    )
}

/// Letters not assigned yet and present at the current magnitude.
#[derive(Debug)]
struct CurrentMagnitudeLetters<'a, CHARSET: Set<char> + fmt::Debug> {
    left: &'a mut CHARSET,
    right: &'a mut CHARSET,
}

/// Assign one (not yet assigned) letter, either on the left or right side of the equation
/// (wherever an unassigned letter is available) for given `magnitude`.
/// All `&mut` parameters are mutated, but reverted back if the function returns None.
/// param left_letters_for_this_magnitude = available left side lettes (unassigned yet) for this magnitude
/// param right_letters_for_this_magnitude = available right side lettes (unassigned yet) for this magnitude
fn pick_and_assign_letter<'m, 'l, CHARSET: Set<char> + fmt::Debug, U8SET: Set<u8> + fmt::Debug>(
    assigned: &mut Assigned<'m, CHARSET, U8SET>,
    empty_charset: &CHARSET,
    magnitude: usize,
    subtotals: MagnitudeSubtotals,
    mag_letters: &mut CurrentMagnitudeLetters<'l, CHARSET>,
    first_call_per_magnitude: bool,
) -> bool {
    // Pick only one letter (for the current magnitude). Don't loop - that is done by below recursion instead. If we can't find a solution starting with this letter, we can't find it starting with any letter (for the current magnitude).
    let letter: char;
    // The same letter may be in one, or both, left and right equation sides. Hence store that fact (so we can remove and later re-insert it at the appropriate equation side(s)).
    let (letter_was_on_left, letter_was_on_right): (bool, bool);
    if let Some(c) = mag_letters.left.iter().next() {
        letter = c;
        letter_was_on_left = true;
        letter_was_on_right = mag_letters.right.contains(&letter);
    } else if let Some(c) = mag_letters.right.iter().next() {
        letter = c;
        letter_was_on_left = mag_letters.left.contains(&letter);
        letter_was_on_right = true;
    } else {
        return if first_call_per_magnitude {
            subtotals.left == subtotals.right
        } else {
            let left =
                sum_per_magnitude(&assigned.eq.left, &assigned.pairs, magnitude) + subtotals.left;
            let right =
                sum_per_magnitude(&assigned.eq.right, &assigned.pairs, magnitude) + subtotals.right;
            left % 10 == right % 10
                && assign_for_magnitude::<CHARSET, U8SET>(
                        assigned,
                        &empty_charset,
                magnitude + 1,
                MagnitudeSubtotals {

                        left: left / 10,
                        right: right / 10,
                }
                    //     &magnitude_args.eq,
                    // &mut magnitude_args.letters,
                    // &mut magnitude_args.digits,
                    // &mut magnitude_args.assigned,
                    // magnitude_args.magnitude + 1,
                    // left / 10,
                    // right / 10,
                    )
        };
    }

    if letter_was_on_left {
        // To simplify the code we could call .remove() regardless of `letter_was_on_left`, but having the `if` shortcuts it.
        mag_letters.left.remove(&letter);
    }
    if letter_was_on_right {
        mag_letters.right.remove(&letter);
    }
    if assigned.letters.contains(&letter) {
        assigned.letters.remove(&letter);
        dbg!(&assigned.letters);

        let digit_found = pick_and_assign_digit::<CHARSET, U8SET>(
            assigned,
            &empty_charset,
            magnitude,
            subtotals,
            mag_letters,
            letter,
        );
        if digit_found {
            // overall success -> bubble up. The result is in `assigned`.
            return true;
        }

        assert!(!assigned.letters.contains(&letter)); // to make sure the deeper processing cleaned up
        assigned.letters.insert(letter);
        dbg!(&assigned.letters);
    } else {
        // letter has been assigned already (at a lower magnitude).
        let pick_another_letter_result = pick_and_assign_letter::<CHARSET, U8SET>(
            assigned,
            &empty_charset,
            magnitude,
            subtotals,
            mag_letters,
            false,
        );
        if pick_another_letter_result {
            return true;
        }
    }
    if letter_was_on_left {
        mag_letters.left.insert(letter);
    }
    if letter_was_on_right {
        mag_letters.right.insert(letter);
    }
    return false;
}

fn pick_and_assign_digit<'m, 'l, CHARSET: Set<char> + fmt::Debug, U8SET: Set<u8> + fmt::Debug>(
    assigned: &mut Assigned<'m, CHARSET, U8SET>,
    empty_charset: &CHARSET,
    magnitude: usize,
    subtotals: MagnitudeSubtotals,
    mag_letters: &mut CurrentMagnitudeLetters<'l, CHARSET>,
    letter: char,
) -> bool {
    // Clone digits into a vector, so we can iterate over it in a stable order. Needed, because as we recurse in & return out, we remove and re-insert each digit into digits. It could re-order it (if entropy changes?). (And we couldn't iterate over it while we borrow it as mutable anyway.)
    let digits_vec = assigned.digits.iter().collect::<Vec<u8>>();
    dbg!(&digits_vec);
    let digit_found = digits_vec.iter().find(|&&digit| {
        if digit == 0 {
            //If the digit (zero) is a leading letter of any operand (on any equation side),
            //skip this assignment. No need for comparing with <=, because that would be caught already at lower magnitude.
            // @TODO into vec![] or local array [] and any() over any() - instead of an or ||
            if [&assigned.eq.left, &assigned.eq.right]
                .iter()
                .any(|&eq_side| {
                    eq_side
                        .iter()
                        .any(|operand| operand_starts_with_letter(&operand, letter))
                })
            {
                return false;
            }
        }

        assigned.digits.remove(&digit);
        dbg!(&assigned.digits);
        let letter_already_assigned = assigned.pairs.insert(letter, digit);
        assert_eq!(letter_already_assigned, None);
        dbg!(&assigned.pairs);

        if pick_and_assign_letter::<CHARSET, U8SET>(
            assigned,
            &empty_charset,
            magnitude,
            subtotals.clone(),
            mag_letters,
            false,
        ) {
            return true;
        }

        let letter_was_still_present = assigned.pairs.remove(&letter);
        dbg!(&assigned.pairs);
        assert!(letter_was_still_present.is_some());
        let digit_was_not_present = assigned.digits.insert(digit);
        assert!(digit_was_not_present);
        dbg!(&assigned.digits);
        false
    });
    digit_found.is_some()
}

fn operand_starts_with_letter(operand: &Operand, letter: char) -> bool {
    operand.last().map(|l| *l == letter) == Some(true)
}

fn all_letters_per_magnitude<CHARSET: Set<char> + fmt::Debug>(
    eq_side: &Vec<Operand>,
    magnitude: usize,
    empty_charset: &CHARSET,
) -> CHARSET {
    let mut result = (*empty_charset).clone();
    result.insert_all(
        eq_side
            .iter()
            .filter_map(|operand| operand.get(magnitude).map(|&c| c)),
    );
    result
}

fn sum_per_magnitude(eq_side: &Vec<Operand>, pairs: &HashMap<char, u8>, magnitude: usize) -> u16 {
    eq_side.iter().fold(0u16, |total, operand| {
        total
            + operand
                .get(magnitude)
                .map_or(0, |&c| *pairs.get(&c).unwrap() as u16)
    })
}

impl TryFrom<&str> for Equation {
    type Error = ();
    fn try_from(from: &str) -> Result<Self, Self::Error> {
        let eq_sides = from.split("==").map(|side| {
            let operand_strs_iter = side.split('+');
            let operands = operand_strs_iter.map(|letters| {
                // reverse, so that we can index & check by magnitude easily
                let words = letters.trim().chars().rev().collect();
                Operand(words)
            });
            operands.collect::<Vec<_>>()
        });

        Ok(Equation::new(eq_sides.into_iter()))
    }
}

// The tests are chosen so that there exists either exactly one solution, or no solution.
// So once we find one, we can return it.
pub fn solve(input: &str) -> Option<HashMap<char, u8>> {
    let mut letters = ('A'..='Z').collect::<HashedSet<char>>();
    let mut digits = (0..=9).collect::<HashedSet<u8>>();
    let empty_charset = HashedSet::<char>::new();
    solve_with(input, &empty_charset, &mut letters, &mut digits)
}

fn solve_with<CHARSET: Set<char> + fmt::Debug, U8SET: Set<u8> + fmt::Debug>(
    input: &str,
    empty_charset: &CHARSET,
    letters: &mut CHARSET,
    digits: &mut U8SET, //mut assigned: &mut CHARSET
) -> Option<HashMap<char, u8>> {
    // parse -> letters, +, ==
    // left side, right side
    let mut assigned = HashMap::<char, u8>::new();
    let success = assign_for_magnitude(
        &mut Assigned {
            eq: &Equation::try_from(input).unwrap(),
            letters,
            digits,
            pairs: &mut assigned,
        },
        &empty_charset,
        0,
        MagnitudeSubtotals { left: 0, right: 0 },
    );
    if success {
        Some(assigned)
    } else {
        None
    }
}
