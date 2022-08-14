//#![allow(unused)]

//use std::collections::HashSet;

pub fn reply(message: &str) -> &str {
    let message = message.trim();
    /*let chars = message.chars();

    //let mut all_capitals = ;
    let mut last_char = ' ';
    chars.for_each( |c| {
        /*if c.is_alphabetic() {
          }*/
        if c.is_alphabetic() {
            //all_capitals = false;
        }
        last_char = c;
    } );*/
    let non_whitespace = message.chars().filter(|c| !c.is_whitespace());
    let non_whitespace_vec: Vec<_> = non_whitespace.collect();
    //let non_whitespace_set: HashSet<_> = non_whitespace.collect();
    //let non_whitespace_vec = non_whitespace.collect::<Vec<_>>();
    //let non_whitespace_set = non_whitespace.collect::<HashSet<_>>();
    //#[allow(unused)]

    let any_are_lowercase = non_whitespace_vec.iter().any(|c| c.is_lowercase());
    let any_alphabetic = non_whitespace_vec.iter().any(|&c| c.is_alphabetic());
    let has_letters_and_all_uppercase = any_alphabetic && !any_are_lowercase;

    let ends_with_questionmark = non_whitespace_vec.last() == Some(&'?');

    if false {
        // Before making one big return:
        if has_letters_and_all_uppercase {
            if ends_with_questionmark {
                return "Calm down, I know what I'm doing!";
            } else {
                return "Whoa, chill out!";
            }
        } else {
            if ends_with_questionmark {
                return "Sure.";
            }
            //...
        }

        if non_whitespace_vec.is_empty() {
            return "Fine. Be that way!";
        }

        if has_letters_and_all_uppercase && non_whitespace_vec.last() != Some(&'?') {
            return "Whoa, chill out!";
        } else if has_letters_and_all_uppercase && non_whitespace_vec.last() == Some(&'?') {
            return "Calm down, I know what I'm doing!";
        } //else  if ...

        //non_whitespace_vec.iter().all(f)

        return "Whatever.";
    }

    /*if non_whitespace_vec.last() == Some(&'X') {
        return "Ends with X."
    };*/
    /* else {
        "Doesn't end with X."
    };*/

    return if has_letters_and_all_uppercase {
        if ends_with_questionmark {
            "Calm down, I know what I'm doing!"
        } else {
            "Whoa, chill out!"
        }
    } else {
        if ends_with_questionmark {
            "Sure."
        } else if non_whitespace_vec.is_empty() {
            "Fine. Be that way!"
        } else {
            if has_letters_and_all_uppercase && non_whitespace_vec.last() != Some(&'?') {
                "Whoa, chill out!"
            } else if has_letters_and_all_uppercase && non_whitespace_vec.last() == Some(&'?') {
                "Calm down, I know what I'm doing!"
            } else {
                "Whatever."
            }
        }
    };
}
