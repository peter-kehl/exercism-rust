#![allow(unused)]
use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

/// Each side of the equation contains one or more operands, all added together.
struct Equation {
    left: Vec<Operand>,
    right: Vec<Operand>,
}

impl Equation {
    // Work around to move out of an iterator
    fn new(mut eq_sides: impl Iterator<Item = Vec<Operand>>) -> Self {
        let left = eq_sides.next().unwrap();
        let right = eq_sides.next().unwrap();
        assert!(eq_sides.next().is_none());
        Self { left, right }
    }
}

/// A *reversed* operand (strings of letters). Reversed to make it easier to handle magnitude
/// by magnitude, since we start from the lowest magnitude.
struct Operand(Vec<char>);

impl Deref for Operand {
    type Target = Vec<char>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Rather than brute-force, go magnitude by magnitude, starting from the lowest magnitude (the last digit of each operand). How:
/// magnitude by magnitude: letters -> digit
/// magnitude carried over to the higher magnitude
/// higher magnitude:
/// if the result digit (for that magnitude) has an already assigned letter, check that the letter matches the existing assigned digit. Otherwise pick any of the available digits.
/// recurse (into the higher magnitude)
///
/// All `&mut` parameters are mutated, but reverted back if the function returns None.
/// param `letters` = available letters (not assigned yet). Somewhat auxilliary - used for asserts.
/// param `digits` = available digits (not assigned yet). Somewhat auxilliary - used for asserts.
/// param assigned = (so far) assigned letters & digits. On success this will be the solution.
/// param `magnitude` = which decimal magnitude to assign (0 for the last digit, 1 for the second last, 2 for the third last...)
/// param carried_over_from_lower_magnitude - if the sum of the lower magnitude' digits (along with any lower-lower carry over) was over 10, this is the lower level result modulo 10.
/// return Some(()) on success (then use assigned as mutated); None if no solution exists.
fn assign_digits_for_magnitude(
    eq: &Equation,
    letters: &mut HashSet<char>,
    digits: &mut HashSet<u8>,
    assigned: &mut HashMap<char, u8>,
    magnitude: usize,
    left_magnitude_subtotal: u16,
    right_magnitude_subtotal: u16,
) -> bool {
    pick_and_assign_letter(
        eq,
        letters,
        digits,
        assigned,
        magnitude,
        &mut all_letters_per_magnitude(&eq.left, magnitude),
        &mut all_letters_per_magnitude(&eq.right, magnitude),
        left_magnitude_subtotal,
        right_magnitude_subtotal,
        true,
    )
}

/// Assign one (not yet assigned) letter, either on the left or right side of the equation
/// (wherever an unassigned letter is available) for given `magnitude`.
/// All `&mut` parameters are mutated, but reverted back if the function returns None.
/// param left_letters_for_this_magnitude = available left side lettes (unassigned yet) for this magnitude
/// param right_letters_for_this_magnitude = available right side lettes (unassigned yet) for this magnitude
/// param left_magnitude_subtotal carried over from the lower magnitude
/// param right_magnitude_subtotal carried over from the lower magnitude
/// For other parameters see `assign_digits_for_magnitude`
fn pick_and_assign_letter(
    eq: &Equation,
    mut letters: &mut HashSet<char>,
    mut digits: &mut HashSet<u8>,
    mut assigned: &mut HashMap<char, u8>,
    magnitude: usize,
    mut left_letters_for_this_magnitude: &mut HashSet<char>,
    mut right_letters_for_this_magnitude: &mut HashSet<char>,
    left_magnitude_carry_over: u16,
    right_magnitude_carry_over: u16,
    first_call_per_magnitude: bool,
) -> bool {
    // Pick only one letter (for the current magnitude). Don't loop - that is done by below recursion instead. If we can't find a solution starting with this letter, we can't find it starting with any letter (for the current magnitude).
    let letter: char;
    // The same letter may be in one, or both, left and right equation sides. Hence store that fact (so we can remove and later re-insert it at the appropriate equation side(s)).
    let (letter_was_on_left, letter_was_on_right): (bool, bool);
    if let Some(c) = left_letters_for_this_magnitude.iter().next().map(|c| *c) {
        letter = c;
        letter_was_on_left = true;
        letter_was_on_right = right_letters_for_this_magnitude.contains(&letter);
    } else if let Some(c) = right_letters_for_this_magnitude.iter().next().map(|c| *c) {
        letter = c;
        letter_was_on_left = left_letters_for_this_magnitude.contains(&letter);
        letter_was_on_right = true;
    } else {
        return if first_call_per_magnitude {
            left_magnitude_carry_over == right_magnitude_carry_over
        } else {
            let left =
                sum_per_magnitude(&eq.left, &assigned, magnitude) + left_magnitude_carry_over;
            let right =
                sum_per_magnitude(&eq.right, &assigned, magnitude) + right_magnitude_carry_over;
            left % 10 == right % 10
                && assign_digits_for_magnitude(
                    &eq,
                    &mut letters,
                    &mut digits,
                    &mut assigned,
                    magnitude + 1,
                    left / 10,
                    right / 10,
                )
        };
    }

    if letter_was_on_left {
        // To simplify the code we could call .remove() regardless of `letter_was_on_left`, but having the `if` shortcuts it.
        left_letters_for_this_magnitude.remove(&letter);
    }
    if letter_was_on_right {
        right_letters_for_this_magnitude.remove(&letter);
    }
    if letters.contains(&letter) {
        letters.remove(&letter);
        dbg!(&letters);

        let digit_found = pick_and_assign_digit(
            &eq,
            &mut letters,
            &mut digits,
            &mut assigned,
            magnitude,
            &mut left_letters_for_this_magnitude,
            &mut right_letters_for_this_magnitude,
            left_magnitude_carry_over,
            right_magnitude_carry_over,
            false,
            letter,
        );
        if digit_found {
            // overall success -> bubble up. The result is in `assigned`.
            return true;
        }

        assert!(!letters.contains(&letter)); // to make sure the deeper processing cleaned up
        letters.insert(letter);
        dbg!(&letters);
    } else {
        // letter has been assigned already (at a lower magnitude).
        let pick_another_letter_result = pick_and_assign_letter(
            &eq,
            &mut letters,
            &mut digits,
            &mut assigned,
            magnitude,
            &mut left_letters_for_this_magnitude,
            &mut right_letters_for_this_magnitude,
            left_magnitude_carry_over,
            right_magnitude_carry_over,
            false,
        );
        if pick_another_letter_result {
            return true;
        }
    }
    if letter_was_on_left {
        left_letters_for_this_magnitude.insert(letter);
    }
    if letter_was_on_right {
        right_letters_for_this_magnitude.insert(letter);
    }
    return false;
}

fn pick_and_assign_digit(
    eq: &Equation,
    mut letters: &mut HashSet<char>,
    mut digits: &mut HashSet<u8>,
    mut assigned: &mut HashMap<char, u8>,
    magnitude: usize,
    mut left_letters_for_this_magnitude: &mut HashSet<char>,
    mut right_letters_for_this_magnitude: &mut HashSet<char>,
    left_magnitude_carry_over: u16,
    right_magnitude_carry_over: u16,
    first_call_per_magnitude: bool,
    letter: char,
) -> bool {
    // Clone digits into a vector, so we can iterate over it in a stable order. Needed, because as we recurse in & return out, we remove and re-insert each digit into digits. It could re-order it (if entropy changes?). (And we couldn't iterate over it while we borrow it as mutable anyway.)
    let digits_vec = digits.iter().map(|&d| d).collect::<Vec<u8>>();
    dbg!(&digits_vec);
    let digit_found = digits_vec.iter().find(|&&digit| {
        if digit == 0 {
            //If the digit (zero) is a leading letter of any operand (on any equation side),
            //skip this assignment. No need for comparing with <=, because that would be caught already at lower magnitude.
            // @TODO into vec![] or local array [] and any() over any() - instead of an or ||
            if [&eq.left, &eq.right].iter().any(|&eq_side| {
                eq_side
                    .iter()
                    .any(|operand| operand_starts_with_letter(&operand, letter))
            }) {
                return false;
            }
        }

        digits.remove(&digit);
        dbg!(&digits);
        let letter_already_assigned = assigned.insert(letter, digit);
        assert_eq!(letter_already_assigned, None);
        dbg!(&assigned);

        if pick_and_assign_letter(
            &eq,
            &mut letters,
            &mut digits,
            &mut assigned,
            magnitude,
            &mut left_letters_for_this_magnitude,
            &mut right_letters_for_this_magnitude,
            left_magnitude_carry_over,
            right_magnitude_carry_over,
            false,
        ) {
            return true;
        }

        let letter_was_still_present = assigned.remove(&letter);
        dbg!(&assigned);
        assert!(letter_was_still_present.is_some());
        let digit_was_not_present = digits.insert(digit);
        assert!(digit_was_not_present);
        dbg!(&digits);
        false
    });
    digit_found.is_some()
}

fn operand_starts_with_letter(operand: &Operand, letter: char) -> bool {
    operand.last().map(|l| *l == letter) == Some(true)
}

// @TODO remove
fn remove_value<T: PartialEq>(set: &mut HashSet<T>, value: &T) {
    set.retain(|item| *item != *value);
}

mod test {
    use std::collections::HashSet;

    #[test]
    fn remove_value_1() {
        let mut set = [1, 2, 3].into_iter().collect::<HashSet<_>>();
        super::remove_value(&mut set, &1);
        assert!(set.len() == 2);
    }
    #[test]
    fn remove_value_b() {
        let mut set = ['A', 'B', 'C'].into_iter().collect::<HashSet<_>>();
        super::remove_value(&mut set, &'B');
        assert!(set.len() == 2);
    }

    #[test]
    fn parsed_str_chars_in_hashset() {
        let str_a = "A";
        //let mut set = str_ab.split(',').mapcollect::<Vec<_>>();
        let mut set = str_a.chars().collect::<HashSet<_>>();
        let contains = set.contains(&'A');
        assert!(contains);
    }
}

fn all_letters_per_magnitude(eq_side: &Vec<Operand>, magnitude: usize) -> HashSet<char> {
    eq_side
        .iter()
        .filter_map(|operand| operand.get(magnitude).map(|&c| c))
        .collect::<HashSet<_>>()
}

fn sum_per_magnitude(
    eq_side: &Vec<Operand>,
    assigned: &HashMap<char, u8>,
    magnitude: usize,
) -> u16 {
    eq_side.iter().fold(0u16, |total, operand| {
        total
            + operand
                .get(magnitude)
                .map_or(0, |&c| *assigned.get(&c).unwrap() as u16)
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
    // parse -> letters, +, ==
    // left side, right side
    let mut assigned = HashMap::<char, u8>::new();
    let success = assign_digits_for_magnitude(
        &Equation::try_from(input).unwrap(),
        &mut ('A'..='Z').collect::<HashSet<char>>(),
        &mut (0..=9).collect::<HashSet<u8>>(),
        &mut assigned,
        0,
        0,
        0,
    );
    if success {
        Some(assigned)
    } else {
        None
    }
}
