#[derive(Debug, PartialEq)]
pub struct Dna(String);

#[derive(Debug, PartialEq)]
pub struct Rna(String);

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

impl Dna {
    /** On error return Err with a 0-based index of the first incorrect character. */
    pub fn new(dna: &str) -> Result<Self, usize> {
        match check(dna, DNA_NUCLEOTIDES) {
            Ok(()) => Ok(Self(dna.to_owned())),
            Err(i) => Err(i),
        }
    }

    pub fn into_rna(self) -> Rna {
        match self {
            Dna(dna) => {
                let rna_chars = dna
                    .chars()
                    .map(|dna_char| match dna_char {
                        'G' => 'C',
                        'C' => 'G',
                        'T' => 'A',
                        'A' => 'U',
                        _ => panic!("Unrecognized nucleotide {dna_char}."),
                    })
                    .collect();
                Rna(rna_chars)
            }
        }
    }
}

impl<'a> Rna {
    /** On error return Err with a 0-based index of the first incorrect character. */
    pub fn new(rna: &str) -> Result<Self, usize> {
        match check(rna, RNA_NUCLEOTIDES) {
            Ok(()) => Ok(Self(rna.to_owned())),
            Err(i) => Err(i),
        }
    }
}
