//! no_std heapless (bare metal/embedded-friendly) implementation
#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use core::fmt::{self, Debug, Formatter};
use core::str::Chars;

#[derive(Debug, PartialEq)]
pub struct Dna(&'static str);

pub enum Rna {
    GivenNucleotides(&'static str), // RNA nucleotides
    // Original DNA nucleotides, but *not* transformed.
    // Instead, it will generate RNA nucleotides on the fly by iterating when
    // the consumer calls `PartialEq::eq(...)` on `self`.
    DnaBased(&'static str),
}

const DNA_NUCLEOTIDES: &str = "GCTA";
const RNA_NUCLEOTIDES: &str = "CGAU";

/// Check that any characters in chars_to_be_checked are in allowed_chars.
/// On success return Ok(()). On error return Err with a 0-based index of the first incorrect
///  character.
fn check(chars_to_be_checked: &'static str, allowed_chars: &'static str) -> Result<(), usize> {
    for (i, c) in chars_to_be_checked.chars().enumerate() {
        if !allowed_chars.contains(c) {
            return Err(i);
        }
    }
    Ok(())
}

pub enum RnaIterator {
    GivenNucleotides(Chars<'static>),
    DnaBased(Chars<'static>),
}

impl Rna {
    pub fn iter(&self) -> RnaIterator {
        match *self {
            Rna::GivenNucleotides(rna) => RnaIterator::GivenNucleotides(rna.chars()),

            Rna::DnaBased(dna) => RnaIterator::DnaBased(dna.chars()),
        }
    }
    pub fn iter_box_dyn(&self) -> Box<dyn Iterator<Item = char>> {
        match *self {
            Rna::GivenNucleotides(rna) => Box::new(rna.chars()),

            Rna::DnaBased(dna) => {
                Box::new(dna.chars().map(|nucl| {
                    // @TODO factor match {...} out to a separate function
                    match nucl {
                        'G' => 'C',
                        'C' => 'G',
                        'T' => 'A',
                        'A' => 'U',
                        _ => {
                            panic!("Unrecognized DNA nucleotide {nucl} (for a DNA-based RNA).")
                        }
                    }
                }))
            }
        }
    }
}

impl Iterator for RnaIterator {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            RnaIterator::DnaBased(chars) => {
                let dna = chars.next();
                match dna {
                    Some(nucl) => match nucl {
                        'G' => Some('C'),
                        'C' => Some('G'),
                        'T' => Some('A'),
                        'A' => Some('U'),
                        _ => panic!("Unrecognized DNA nucleotide {nucl} (for a DNA-based RNA)."),
                    },
                    None => None,
                }
            }
            RnaIterator::GivenNucleotides(chars) => chars.next(),
        }
    }
}

impl Dna {
    /** On error return Err with a 0-based index of the first incorrect character. */
    pub fn new(dna: &'static str) -> Result<Self, usize> {
        match check(dna, DNA_NUCLEOTIDES) {
            Ok(()) => Ok(Self(dna)),
            Err(i) => Err(i),
        }
    }

    pub fn into_rna(self) -> Rna {
        match self {
            Dna(dna) => Rna::DnaBased(dna),
        }
    }
}

impl PartialEq for Rna {
    // TODO could we .eq without a custom iterator, just .map()?
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl Debug for Rna {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "RNA {{")?;
        match self {
            Rna::GivenNucleotides(str) => {
                write!(f, "GivenNucleotides {{{str}}}")?;
            }
            Rna::DnaBased(str) => {
                write!(f, "DnaBased {{{str}}} which translates to ")?;
                let main_result: Result<(), fmt::Error> =
                    self.iter().fold(Ok(()), |prev_result, c| {
                        if prev_result.is_ok() {
                            write!(f, "{c}")
                        } else {
                            prev_result
                        }
                    });
                if main_result.is_err() {
                    return main_result;
                }
            }
        }
        write!(f, "}}")
    }
}

/*
#[cfg(test)]
pub mod test {
    use arrform::{arrform, ArrForm};

    #[test]
    #[allow(unused_must_use)]
    fn test_rna_given_nucleotides_debug() {
        super::Dna::new("GCTA").map(|dna| {
            let rna = dna.into_rna();
            let rna_af = arrform!(64, "{:?}", rna);
            assert_eq!(
                "RNA {DnaBased {GCTA} which translates to CGAU}",
                rna_af.as_str()
            );
        });
    }
}*/

impl Rna {
    /** On error return Err with a 0-based index of the first incorrect character. */
    pub fn new(rna: &'static str) -> Result<Self, usize> {
        match check(rna, RNA_NUCLEOTIDES) {
            Ok(()) => Ok(Self::GivenNucleotides(rna)),
            Err(i) => Err(i),
        }
    }
}
