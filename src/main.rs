/*
Enigma encode/decode

Enigma brute force
*/
use std::str;
//use std::ascii::AsciiExt;

/* Enigma rotors are called walzenlage (walze). They are constants. There are up to 8 walze available, but
  only 3 are used at a time. Lets start with 3 and work up from there.
  Each walze has two arrays. The walze encode and the inverse for the return from the reflector.
  TODO: Move to a different file? Do we need a module? what is the rust way?

  Note some walze are numbered 1->26 not A->Z. We can handle this conversion later.
*/
const _WALZE_1: [char; 26] = [
    // A->E, B->K, ...
    'E', 'K', 'M', 'F', 'L', 'G', 'D', 'Q', 'V', 'Z', 'N', 'T', 'O', 'W', 'Y', 'H', 'X', 'U', 'S',
    'P', 'A', 'I', 'B', 'R', 'C', 'J',
];
const _WALZE_1_INV: [char; 26] = [
    // Inverse walze_1 E->A, K->B, ...
    'U', 'W', 'Y', 'G', 'A', 'D', 'F', 'P', 'V', 'Z', 'B', 'E', 'C', 'K', 'M', 'T', 'H', 'X', 'S',
    'L', 'R', 'I', 'N', 'Q', 'O', 'J',
];

const _WALZE_2: [char; 26] = [
    'A', 'J', 'D', 'K', 'S', 'I', 'R', 'U', 'X', 'B', 'L', 'H', 'W', 'T', 'M', 'C', 'Q', 'G', 'Z',
    'N', 'P', 'Y', 'F', 'V', 'O', 'E',
];
const _WALZE_2_INV: [char; 26] = [
    'A', 'J', 'P', 'C', 'Z', 'W', 'R', 'L', 'F', 'B', 'D', 'K', 'O', 'T', 'Y', 'U', 'Q', 'G', 'E',
    'N', 'H', 'X', 'M', 'I', 'V', 'S',
];

const _WALZE_3: [char; 26] = [
    'B', 'D', 'F', 'H', 'J', 'L', 'C', 'P', 'R', 'T', 'X', 'V', 'Z', 'N', 'Y', 'E', 'I', 'W', 'G',
    'A', 'K', 'M', 'U', 'S', 'Q', 'O',
];
const _WALZE_3_INV: [char; 26] = [
    'T', 'A', 'G', 'B', 'P', 'C', 'S', 'D', 'Q', 'E', 'U', 'F', 'V', 'N', 'Z', 'H', 'Y', 'I', 'X',
    'J', 'W', 'L', 'R', 'K', 'O', 'M',
];

const _REFLECTOR_B: [char; 26] = [
    'Y', 'R', 'U', 'H', 'Q', 'S', 'L', 'D', 'P', 'X', 'N', 'G', 'O', 'K', 'M', 'I', 'E', 'B', 'F',
    'Z', 'C', 'W', 'V', 'J', 'A', 'T',
];

// Walze notch positions
const _WALZE_1_NOTCH: u8 = ('Q' as u8) - ASCII_A;
const _WALZE_2_NOTCH: u8 = ('E' as u8) - ASCII_A;

// ASCII 'A' = 65,
const ASCII_A: u8 = 65;

// Reflector (German: Umkehrwalze)
fn reflector(i: &u8, umkehr: &Vec<u8>) -> u8 {
    let w_pos: u8 = i % 26;
    umkehr[w_pos as usize]
}

/* Take a value and run it through the walze to get the next value */
fn encode_plugboard(in_l: &u8, plugboard_values: &Vec<[u8; 2]>) -> u8 {
    for plug in plugboard_values.iter() {
        if plug[0] == *in_l {
            return plug[1];
        } else if plug[1] == *in_l {
            return plug[0];
        }
    }

    // Return back the input if it didn't change. Maybe this should be an Option thingy in Rust.
    *in_l
}

/* Take a value and run it through the walze to get the next value */
fn encode_slot(in_l: &u8, ring_pos: &u8, walze: &Vec<u8>) -> u8 {
    // TODO: This requires some explanation...Got this from example code, but I don't know
    // why/how this works with the ring_pos it increments the rotor and gets the value, but then
    // has to subtract. This works and matches encoding by all the enigma demos.
    let w_pos: u8 = (in_l + ring_pos + 26) % 26;

    println!("w_pos {}", w_pos);
    // TODO: explain this!
    ((walze[w_pos as usize] + 26) - ring_pos) % 26 // cast to usize to get array offset
}

/* Enigma is symmetric, so  encryption and decryption use the same function

encrypt/decrypt takes a string slice (&str) and returns a String.  Since this is
symmetric, we don't need to grow/shrink the string, so we use a string slice.

We convert the slice to a byte array to make changing the characters easy. This requires that
the input be ascii alphabetical only. This should be passed in correctly, but we will check one
more time after we convert to the byte array.

TODO: Use serde for this stuff, somehow?
TODO: Make an enigma Method with machine settings in a struct and encode functions implemented, etc.
      Then implement concurrency for each machine for brute force cracking.
*/
fn encrypt_decrypt(input_text: &str, input_plugboard: &Vec<[char; 2]>) -> String {
    let mut enc_buffer: Vec<u8> = input_text.as_bytes().to_vec();
    // Convert from ASCII value to 0==A, 1==B...25==Z
    for i in enc_buffer.iter_mut() {
        *i -= ASCII_A;
    }

    /* Ringstellung is the ring setting of the walze. 1->A 2->B, etc */
    let mut ring_pos_s1: u8 = 0;
    let mut ring_pos_s2: u8 = 0;
    let mut ring_pos_s3: u8 = 0;

    //TODO - set arbitrary walze for each slot
    let walze_s1: Vec<u8> = _WALZE_1
        .iter()
        .map(|c| (*c as u8) - ASCII_A)
        .collect::<Vec<_>>();
    let walze_s2: Vec<u8> = _WALZE_2
        .iter()
        .map(|c| (*c as u8) - ASCII_A)
        .collect::<Vec<_>>();
    let walze_s3: Vec<u8> = _WALZE_3
        .iter()
        .map(|c| (*c as u8) - ASCII_A)
        .collect::<Vec<_>>();
    let umkehr: Vec<u8> = _REFLECTOR_B
        .iter()
        .map(|c| (*c as u8) - ASCII_A)
        .collect::<Vec<_>>();
    let walze_s1_inv: Vec<u8> = _WALZE_1_INV
        .iter()
        .map(|c| (*c as u8) - ASCII_A)
        .collect::<Vec<_>>();
    let walze_s2_inv: Vec<u8> = _WALZE_2_INV
        .iter()
        .map(|c| (*c as u8) - ASCII_A)
        .collect::<Vec<_>>();
    let walze_s3_inv: Vec<u8> = _WALZE_3_INV
        .iter()
        .map(|c| (*c as u8) - ASCII_A)
        .collect::<Vec<_>>();

    // Convert the ASCII to 0->25 values
    // Should this be array, vector, or slice? Build a hashmap? Might be overkill? Might be good for performance.
    let plugboard_values: Vec<[u8; 2]> = input_plugboard
        .iter()
        .enumerate()
        .map(|(_usize, c)| [(c[0] as u8) - ASCII_A, (c[1] as u8) - ASCII_A])
        .collect::<Vec<_>>();
    println!("{:?}", plugboard_values);

    // Encode each character in the buffer.
    // TODO - clean up the assumptions/hard-coded things noted below
    for i in enc_buffer.iter_mut() {
        // Increment the rotor for each character (key press), Ringstellung
        // Check the notch position to see if the middle and slow rotor need to increment.
        // TODO - This assumes the walze are I, II, III in slot fast, middle, slow slots
        ring_pos_s1 = (ring_pos_s1 + 1) % 26;
        // walze I in the fast slot, check notch position
        if ring_pos_s1 == _WALZE_1_NOTCH {
            ring_pos_s2 = (ring_pos_s2 + 1) % 26;
            // walze II in middle slot
            if ring_pos_s2 == _WALZE_2_NOTCH {
                ring_pos_s3 = (ring_pos_s3 + 1) % 26;
            }
        }
        println!("ring_pos: {} {} {}", ring_pos_s3, ring_pos_s2, ring_pos_s1);

        // Start with encoding through the plugboard, Steckerverbindungen (stecker)
        *i = encode_plugboard(*&i, &plugboard_values);

        // encode through each rotor, through the reflector and back
        *i = encode_slot(*&i, &ring_pos_s1, &walze_s1);
        println!("slot1 {}", i);

        *i = encode_slot(*&i, &ring_pos_s2, &walze_s2);
        println!("slot2 {}", i);

        *i = encode_slot(*&i, &ring_pos_s3, &walze_s3);
        println!("slot3 {}", i,);

        *i = reflector(*&i, &umkehr);
        println!("reflect {}", i);

        *i = encode_slot(*&i, &ring_pos_s3, &walze_s3_inv);
        println!("slot3 {}", i);

        *i = encode_slot(*&i, &ring_pos_s2, &walze_s2_inv);
        println!("slot2 {}", i);

        *i = encode_slot(*&i, &ring_pos_s1, &walze_s1_inv);
        println!("slot1 {}", i);

        // End with encoding through the plugboard, Steckerverbindungen (stecker)
        *i = encode_plugboard(*&i, &plugboard_values);
    }
    println!("{:?}", enc_buffer);

    // Convert back to ASCII value
    // a.iter_mut().for_each(|i| *i += 1); also would work.
    for i in enc_buffer.iter_mut() {
        *i += ASCII_A;
    }
    println!("{:?}", enc_buffer);

    let output = str::from_utf8(&enc_buffer).unwrap();
    output.to_string()
}

fn main() {
    /*     for (a, w) in ('A'..='Z').zip(WALZE_1.iter()) {
        println!("WALZE 1: {},{} ", a, w);
    }
    for (a, w) in ('A'..='Z').zip(WALZE_2.iter()) {
        println!("WALZE 2: {},{} ", a, w);
    }
    for (a, w) in ('A'..='Z').zip(WALZE_3.iter()) {
        println!("WALZE 3: {},{} ", a, w);
    } */

    // TODO: Get clear text from command line or from a file.
    // TODO: Check clear test string for chars that at not in the alphabet (no numbers or punctuation)

    let mut clear_text = String::from("ABCD");
    println!("clear_text: {}", clear_text);

    /* Steckerverbindungen (stecker) are the plugboard settings */
    // TODO: Check that there are no repeat chars. Letters can only be mapped once.
    let plugboard_setting: Vec<[char; 2]> =
        vec![['A', 'B'], ['C', 'D'], ['E', 'F'], ['M', 'N'], ['Q', 'R']];
    println!("{:?}", plugboard_setting);

    // Do some cleanup and error checking.
    // No spaces are allowed. Replace then with X (should we just remove them?)
    clear_text = clear_text.replace(" ", "X");
    println!("clear_text: {}", clear_text);

    // Check for any no alphabetic, no numbers, punctuation, os spaces
    assert!(clear_text.bytes().all(|c| c.is_ascii_alphabetic()));

    // Make all uppercase (there is only one case)
    clear_text = clear_text.to_uppercase();
    println!("clear_text: {}", clear_text);

    //encrypt the clear_text
    let encrypted_text: String = encrypt_decrypt(&clear_text, &plugboard_setting);
    println!("encrypted_text: {}", encrypted_text);

    // Decrypt what we just encrypted
    let decrypted_text: String = encrypt_decrypt(&encrypted_text, &plugboard_setting);
    println!("decrypted_text: {}", decrypted_text);
    assert!(clear_text == decrypted_text);
    println!("success");
}
