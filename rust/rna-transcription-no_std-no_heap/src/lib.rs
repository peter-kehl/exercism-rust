//! no_std heapless (bare metal/embedded-friendly) implementation
#![no_std]

extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use core::fmt::{self, Debug, Formatter};
use core::str::Chars;

#[derive(Debug, PartialEq)]
pub struct Dna<'a>(&'a str);

pub enum Rna<'a> {
    GivenNucleotides(&'a str), // RNA nucleotides
    // Original DNA nucleotides, but *not* transformed.
    // Instead, it will generate RNA nucleotides on the fly by iterating when
    // the consumer calls `PartialEq::eq(...)` on `self`.
    DnaBased(&'a str),
}

const DNA_NUCLEOTIDES: &str = "GCTA";
const RNA_NUCLEOTIDES: &str = "CGAU";

/// Check that any characters in chars_to_be_checked are in allowed_chars.
/// On success return Ok(()). On error return Err with a 0-based index of the first incorrect
///  character.
fn check<'a>(chars_to_be_checked: &'a str, allowed_chars: &'a str) -> Result<(), usize> {
    for (i, c) in chars_to_be_checked.chars().enumerate() {
        if !allowed_chars.contains(c) {
            return Err(i);
        }
    }
    Ok(())
}

pub enum RnaIterator<'a> {
    GivenNucleotides(Chars<'a>),
    DnaBased(Chars<'a>),
}

impl<'a> Rna<'a> {
    pub fn iter(&self) -> RnaIterator<'a> {
        match *self {
            Rna::GivenNucleotides(rna) => RnaIterator::GivenNucleotides(rna.chars()),

            Rna::DnaBased(dna) => RnaIterator::DnaBased(dna.chars()),
        }
    }
    pub fn iter_dyn(&self) -> Box<dyn Iterator<Item = char>> {
        todo!()
        /*
        match *self {
            Rna::GivenNucleotides(rna) => {
                let rna = rna.to_owned();
                (move ||
                    Box::new(rna.chars())
                )()
            },

            Rna::DnaBased(dna) => {
                let dna = dna.to_owned();

                (move || {
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
                })()
            }
        }
        */
    }
    // TODO alternative where dna/rna &str are 'static
}

impl<'a> Iterator for RnaIterator<'a> {
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

impl<'a> Dna<'a> {
    /** On error return Err with a 0-based index of the first incorrect character. */
    pub fn new(dna: &'a str) -> Result<Self, usize> {
        match check(dna, DNA_NUCLEOTIDES) {
            Ok(()) => Ok(Self(dna)),
            Err(i) => Err(i),
        }
    }

    pub fn into_rna(self) -> Rna<'a> {
        match self {
            Dna(dna) => Rna::DnaBased(dna),
        }
    }
}

impl<'a> PartialEq for Rna<'a> {
    // TODO could we .eq without a custom iterator, just .map()?
    fn eq(&self, other: &Self) -> bool {
        //self.iter().eq(other.iter())
        self.iter().eq("abc".chars())
        //todo!()
    }
}

impl<'a> Debug for Rna<'a> {
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

impl<'a> Rna<'a> {
    /** On error return Err with a 0-based index of the first incorrect character. */
    pub fn new(rna: &'a str) -> Result<Self, usize> {
        match check(rna, RNA_NUCLEOTIDES) {
            Ok(()) => Ok(Self::GivenNucleotides(rna)),
            Err(i) => Err(i),
        }
    }
}
