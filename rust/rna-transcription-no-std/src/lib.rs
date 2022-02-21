//#![no_std]

use core::fmt::{self, Debug, Formatter};
use core::str::Chars;

#[derive(Debug, PartialEq)]
pub struct Dna<'a>(&'a str);

pub enum Rna<'a> {
    GivenNucleotides(&'a str),
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
}

impl<'a> Iterator for RnaIterator<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            RnaIterator::DnaBased(chars) => {
                let dna = chars.next();
                match dna {
                    Some(dna) => match dna {
                        'G' => Some('C'),
                        'C' => Some('G'),
                        'T' => Some('A'),
                        'A' => Some('U'),
                        _ => panic!("Unrecognized DNA nucleotide {dna} (for a DNA-based RNA)."),
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
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
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
                // @TODO FromResidual: use operator ? within the closure:
                //#[allow(unused_must_use)]
                self.iter().for_each(|c| {
                    write!(f, "{c}");
                });
            }
        }
        write!(f, "}}")
    }
}

impl<'a> Rna<'a> {
    /** On error return Err with a 0-based index of the first incorrect character. */
    pub fn new(rna: &'a str) -> Result<Self, usize> {
        match check(rna, RNA_NUCLEOTIDES) {
            Ok(()) => Ok(Self::GivenNucleotides(rna)),
            Err(i) => Err(i),
        }
    }
}
