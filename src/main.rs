/// Project: Generation of word digrams (2 grams) pt_PT in Rust
/// 
/// Author: João Nuno Carvalho
/// Date:   2022.01.15
/// 
/// Description: This small project is a efficient way of generating a digrams
///              file and frequency of words file for pt_PT (Portuguese) from the
///              European Parliament Proceedings Parallel Corpus 1996-2011 (see
///              reference below for details).
///              
///              The processing that I have made:
///               -The text file was divided into phrases and word space delimited.
///               -Each word was checked for valid chars in Portuguese with a reg_ex.
///               -Each word was checked if it was a valid Portuguese word, with the
///                HunSpell dictionary pt_PT from 2021.12.25 (see Project Natura,
///                below).
///               -Because the text is pre orthographic treaty, I tried to save the 
///                words that had one more muted 'c' or 'p' char and words that started
///                with an uppercase like "alemanha" vs "Alemanha" or opec vs OPEC.
///               -Then I have made some mapping between wrong written words in terms
///                of just one accent sign.
///              
///              This was possible because HunSpell has a suggest() function in
///              it's API that returns close lexical valid words. The previous
///              similarity process seemed to me a "safe" and simple process to do.   
/// 
///              Note: The suggest() function in HunSpell is rather slow because it
///                    has to try every single permutation for a predetermined
///                    distance. SymSpell algorithm is faster in this regard. 
///                    So I implemented a cache over the correct and incorrect words,
///                    to lower the number of calls made to the suggest() function
///                    from HunSpell. It worked the processing time went down from 3 H
///                    or 4 H to 20 minutes, on a single core.
/// 
/// References: 
///     1. Projecto Natura - Hunspell - dictionary pt_PT in Portuguese
///        https://natura.di.uminho.pt/wiki/doku.php?id=dicionarios:main
///
///     2. European Parliament Proceedings Parallel Corpus 1996-2011 - Portuguese
///        https://www.statmt.org/europarl/
///
///     3. SymSpell crate in Rust
///        https://github.com/reneklacan/symspell
/// 
///     4. Original SymSpell by Wolfgarbe in C#
///        It doesn't have a pt_PT word frequency dictionary and digrams.
///        https://github.com/wolfgarbe/SymSpell
///
/// License: I put my code on MIT Open Source License.
///          But the license of the digrams and the words of the text are in there
///          respective licenses.
///          In reference 2. is said about the big 320 MB pt_PT text "We are not
///          aware of any copyright restrictions of the material.". See the reference
///          European Parliament Proceedings Parallel Corpus 1996-2011 above.
/// 
/// 


mod strings_extender;

use hunspell_rs::Hunspell;
use strings_extender::{StringUtils, StringUtilsSlices};
use std::{fs, collections::HashMap};    // Readfile

extern crate unicode_normalization;

// use unicode_normalization::char::compose;
use unicode_normalization::UnicodeNormalization;

// use std::collections::HashMap;
use std::collections::BTreeMap;

use regex::Regex;

use crate::strings_extender::StringUtilsVecChars;

use std::time::{Instant, Duration};

fn main() {
    println!("**********************************************************");
    println!("**  Generation of word digrams (2 grams) pt_PT in Rust  **");
    println!("**********************************************************");

    // test_hunspell();

    // NOTE: Use only in the beginning to process small file from a big txt file.
    // let num_lines = 1_000;
    // big_380_mb_to_10_lines(num_lines);

    // Time the execution of the function.
    let bench_1 = || -> usize {
        // Inner closure, the actual bench test

        //***********
        // Small file
        read_all_words_freq_of_file_auto();
        // read_all_2_grams_words_of_file_auto();

        //*********
        // Big file (ex: 20 minutes of processing)
        // read_all_words_freq_of_file_auto_full_to_dev_shm();
        // read_all_2_grams_words_of_file_auto_full_to_dev_shm();

        0
    };    

    let (res_1, elapsed_1) = time_it(bench_1);
    println!("Benchmark: {} => time: {:.4} horas.",
            //decimal_mark2("0".to_string()), 
            //decimal_mark2((elapsed_1.as_secs_f64() / 3600.0).to_string())
            "0",
            elapsed_1.as_secs_f64() / 3600.0
        );
    println!("  res = {}", res_1);
}

fn get_hunspell() -> Hunspell {
    let path = "./hunspell-pt_PT-20211125/";
    let dic = "pt_PT";

    Hunspell::new(&(String::new() + path + dic + ".aff"),
                  &(String::new() + path + dic + ".dic"))
    }

fn test_hunspell() {
    let path = "./hunspell-pt_PT-20211125/";
    let dic = "pt_PT";

    let hs = Hunspell::new(&(String::new() + path + dic + ".aff"),
                                    &(String::new() + path + dic + ".dic"));

    // Verify if word exists.
    let word = "serviç";
    let flag_word_exists = hs.check(word);
    println!("\nThe word: \"{}\": {}", word, flag_word_exists); 

    // Get suggestions for word. 
    let suggest =  hs.suggest(word);
    println!("\nThe suggestions to the word: \"{}\":", word);
    for sugg_word in &suggest {
        println!("   {}", sugg_word);
    }

    // Analyse word. 
    let analyse =  hs.analyze(word);
    println!("\nThe analysis for the word: \"{}\":", word);
    for analyse_word in &analyse {
        println!("   {}", analyse_word);
    }

    // Note: This doesn't work, the binding possibly it's not finished.
    // Generate word. 
    let word_1 = "bebida";
    let word_2 = "pratos";
    let generate =  hs.generate(word_1,word_2);
    println!("\nDoesn't work. Generate the word between \"{}\" and \"{}\" :", word_1, word_2);
    for generate_word in &generate {
        println!("   {}", generate_word);
    }
}

//*********************
// Read the 380 MB file from disc and create a 10 lines small file.
fn big_380_mb_to_10_lines(num_lines: usize) {
    let in_path = r"../pt-en/";
    let in_filename = "europarl-v7.pt-en.pt";
    let in_final_path = in_path.to_string() + in_filename; 
    
    use std::env;
    
    let cur_dir = env::current_dir().unwrap();
    println!("\n current_dir: {}", cur_dir.to_string_lossy());
    println!("\n path: {}", &in_final_path);
    
    let input_string = fs::read_to_string(&in_final_path)
        .expect("Something went wrong reading the file");

    let output_string = input_string.lines().take(num_lines).fold("".to_string(),|acc, x_str| acc + &x_str.nfkc().collect::<String>() ); 

    let out_path = r"./data/";
    let out_filename =  "small_europarl-v7.pt-en.pt";
    let out_final_path = out_path.to_string() + out_filename; 

    let _res = fs::write(&out_final_path, &output_string);

    println!("\nFile Written: {}", &out_final_path);

    println!("\nWith text:\n{}", &output_string);
}

//******
// Read all unique words frequency of 10 lines small file. (Seconds)
fn read_all_words_freq_of_file_auto() {
    let in_path = r"./data/";
    let in_filename = "small_europarl-v7.pt-en.pt";
    let in_text_corpus_path = in_path.to_string() + in_filename; 
    
    let out_path = r"/dev/shm/";
    let out_filename =  "dic_corpus_unique_small.words";
    let out_dic_unique_words_path = out_path.to_string() + out_filename; 

    let flag_check_spell = true;

    read_all_words_freq_of_file(&in_text_corpus_path, &out_dic_unique_words_path, flag_check_spell); 
}

// Read all unique words frequency of 320 MB text big file. (20 minutes)
fn read_all_words_freq_of_file_auto_full_to_dev_shm() {
    let in_path = r"../pt-en/";
    let in_filename = "europarl-v7.pt-en.pt";
    let in_text_corpus_path = in_path.to_string() + in_filename; 
    
    let out_path = r"/dev/shm/";
    let out_filename =  "dic_corpus_unique.words";
    let out_dic_unique_words_path = out_path.to_string() + out_filename; 

    let flag_check_spell = true;

    read_all_words_freq_of_file(&in_text_corpus_path, &out_dic_unique_words_path, flag_check_spell); 
}

fn map_accents() -> HashMap<char, char> {
    let map_accents: HashMap<char, char> = HashMap::from([
                        ('ê','e'),
                        ('á','a'),
                        ('à','a'),
                        ('é','e'),
                        ('e','é'),
                        ('ã','a'),
                        ('a','á'),
                        ('â','a'),
                        ('õ','o'),
                        ('o','õ'),
                        ('í','i'),
                        ('i','í'),
                        ('ç','c'),
                        ('c','ç')
                    ]);
    map_accents
}

// This uses the EU Parliament sessions writings that are written in old
// Portuguese, prior to the new orthographic treaty to construct the
// 2 grams list of relations between words. But correcting in part the
// old to the new in the 'c' and 'p' that were removed.
// It also tests the world for upper case of the first word in case of
// a name, a country or a short form of writing (in this last case all
// letters are uppercase).
fn from_old_treaty_to_new_treaty_orthographic(word: &str, suggestion_vec: &Vec<String>, map_accents_p: &HashMap<char, char>) -> Option<String> {

    fn test_string(mod_word: &str, suggestion_vec: &Vec<String>) -> Option<String> {
        for sug in suggestion_vec {
            if *sug == mod_word {
                return Some(mod_word.to_string());
            }
        }
        None    
    }

    let mut tmp_string = String::with_capacity(50);

    // First letter upper case.
    // Ex: alemanha vs Alemanha
    let mut chars_vec  = word.get_vec_chars();
    let up_vec: Vec<_> = chars_vec[0].to_uppercase().collect();
    chars_vec[0] = up_vec[0];
    chars_vec.to_string_buf(&mut tmp_string);
    if let Some(found) = test_string(&tmp_string, suggestion_vec) {
        return Some(found);
    }

    // All letter upper case.
    // Ex: opec vs OPEC
    let all_letter_upper_case = word.to_uppercase();
    if let Some(found) = test_string(&all_letter_upper_case, suggestion_vec) {
        return Some(found);
    }

    // The suggestion has one more 'c' or one more 'p' then take the suggestion word.
    // Ex: acta    vs ata 
    //     adopção vs adoção.
    let mut chars_vec  = word.get_vec_chars();
    for i in 1..(chars_vec.len() - 1) {
        // Remove 'c' or 'p'
        let mut ch: Option<char> = None;
        if chars_vec[i] == 'c' || chars_vec[i] == 'p' {
            ch = Some(chars_vec.remove(i));
        } 
        chars_vec.to_string_buf(&mut tmp_string);
        if let Some(found) = test_string(&tmp_string, suggestion_vec) {
            return Some(found);
        }
        if let Some(ch_tmp) = ch {
            chars_vec.insert(i, ch_tmp);
        }
    }

    // Test every combination of distance one accents.
    let mut chars_vec  = word.get_vec_chars();
    for i in 0..chars_vec.len() {
        let new_char = map_accents_p.get(&chars_vec[i]);
        if let Some(new_char_tmp) = new_char {
            let old_char = chars_vec[i];
            chars_vec[i] = *new_char_tmp;
            chars_vec.to_string_buf(&mut tmp_string);
            if let Some(found) = test_string(&tmp_string, suggestion_vec) {
                return Some(found);
            }
            chars_vec[i] = old_char;
        }
    }

    None
}

fn make_string_from_BTreeHap(dic_word_freq: &BTreeMap<String, u64>) -> String {
    let mut dic_unique_freq_words_string = String::with_capacity(1_000_000);
    for (key, count) in dic_word_freq {
        dic_unique_freq_words_string.push_str(key);
        dic_unique_freq_words_string.push(' ');
        dic_unique_freq_words_string.push_str(&(*count.to_string()));
        dic_unique_freq_words_string.push('\n');
    }
    dic_unique_freq_words_string
}

fn read_all_words_freq_of_file(in_text_corpus_path: &str, out_dic_unique_words_path: &str, flag_check_spell: bool) {
    let hunspell = get_hunspell();
    
    let map_accents_tmp = map_accents();

    use std::env;
    
    let cur_dir = env::current_dir().unwrap();
    println!("\n current_dir: {}", cur_dir.to_string_lossy());
    println!("\n path: {}", &in_text_corpus_path);
    
    let input_string_corrected: String;
    {
        let input_string = fs::read_to_string(&in_text_corpus_path)
            .expect("Something went wrong reading the file");
        input_string_corrected = input_string.nfkc().collect::<String>();
    }

    let reg_ex_pt = LangRegEx::new(Lang::PT).reg_ex_comp_word_pattern;

    // HunSpell cache
    let mut hunspell_word_cache: HashMap<String, String> = HashMap::new(); 

    // Counts all words.
    let mut dic_unique_freq_words: BTreeMap<String, u64> = BTreeMap::new();

    let mut not_dic_unique_freq_words: BTreeMap<String, u64> = BTreeMap::new();

    let mut dic_not_check_unique_freq_words: BTreeMap<String, u64> = BTreeMap::new();

    for phrase in input_string_corrected.split('.') {
        for word in phrase.split_whitespace() {
            let captures = reg_ex_pt.captures(word);
            if captures.is_none() {
                let count = not_dic_unique_freq_words.entry(word.to_string()).or_insert(0);
                *count += 1;
                continue;
            }
            let captures = captures.unwrap();
            if let Some(word_only) = captures.get(0) {
                let mut lower_case_word = word_only.as_str().to_ascii_lowercase();

                let flag_word_exists = hunspell.check(&lower_case_word);
                if !flag_word_exists {
                    // See if the correct map of the word to the hunspell dictionary is on the cache, if it is use it.
                    let correct_word = hunspell_word_cache.get(&lower_case_word);
                    let res;
                    if correct_word.is_none() {
                        let suggestion_vec = hunspell.suggest(&lower_case_word);
                        res = from_old_treaty_to_new_treaty_orthographic(&lower_case_word, &suggestion_vec, &map_accents_tmp);
                        if res.is_some() {
                            // Substituted the word for a correct word.
                            let new_word = res.unwrap();
                            hunspell_word_cache.insert(lower_case_word.clone(), new_word.clone());
                            // Let's use it!
                            lower_case_word = new_word;   
                        } else {
                            // Didn't found a correct substitution, so puts the word in the not check_dic. 
                            hunspell_word_cache.insert(lower_case_word.clone(), "*".to_string() + &suggestion_vec.join(" "));
                            let count = dic_not_check_unique_freq_words.entry(lower_case_word + " -> " + &suggestion_vec.join(" ")).or_insert(0);
                            *count += 1;
                            continue;    
                        }
                    } else {
                        // Found the word in hunspell cache.
                        let inner_correct_word = correct_word.unwrap();
                        if inner_correct_word.starts_with("*") {
                            // The word is invalid, because it could find a correct word for it. 
                            continue;
                        }
                        // The word that was on hunspell cache is a correct word.
                        // Let's use it!
                        lower_case_word = inner_correct_word.clone();
                    }
                }
                let count = dic_unique_freq_words.entry(lower_case_word).or_insert(0);
                *count += 1;
            }
        }
    }

    let dic_unique_freq_words_string = make_string_from_BTreeHap(&dic_unique_freq_words);

    let not_dic_unique_freq_words_string = make_string_from_BTreeHap(&not_dic_unique_freq_words);

    let dic_not_check_unique_freq_words_string = make_string_from_BTreeHap(&dic_not_check_unique_freq_words);

    // Save to file.
    let _res = fs::write(&out_dic_unique_words_path, &dic_unique_freq_words_string);

    // println!("\nFile written: {}", &dic_unique_freq_words_string);

    // Save to file.
    let not_out_dic_unique_words_path = out_dic_unique_words_path.replace("dic", "not_dic");
    let _res = fs::write(&not_out_dic_unique_words_path, &not_dic_unique_freq_words_string);

    // Save to file.
    let not_out_dic_not_check_unique_words_path = out_dic_unique_words_path.replace("dic", "not_check_dic");
    let _res = fs::write(&not_out_dic_not_check_unique_words_path, &dic_not_check_unique_freq_words_string);

    // println!("\nhunspell_word_cache: \n {:?}", hunspell_word_cache);
    println!("\nhunspell_word_cache.len(): \n {}", hunspell_word_cache.len());

    // println!("\nWords not written: {}", &not_dic_unique_freq_words_string);

    println!("\n1. With text:\n{}", &out_dic_unique_words_path);
    println!("\n2. With text:\n{}", &not_out_dic_unique_words_path);
    println!("\n2. With text:\n{}", &not_out_dic_not_check_unique_words_path);
}

// Generate 2 grams for small file (seconds).
fn read_all_2_grams_words_of_file_auto() {
    let in_path = r"./data/";
    let in_filename = "small_europarl-v7.pt-en.pt";
    let in_text_corpus_path = in_path.to_string() + in_filename; 
    
    let out_path = r"/dev/shm/";
    let out_filename =  "2_grams_small.words";
    let out_2_grams_words_path = out_path.to_string() + out_filename; 

    let flag_check_words = true;

    read_all_2_grams_words_of_file(&in_text_corpus_path, &out_2_grams_words_path, flag_check_words);
}

// Generate 2 grams for big file 320 MB (20 minutes).
fn read_all_2_grams_words_of_file_auto_full_to_dev_shm() {
    let in_path = r"../pt-en/";
    let in_filename = "europarl-v7.pt-en.pt";
    let in_text_corpus_path = in_path.to_string() + in_filename; 
    
    let out_path = r"/dev/shm/";
    let out_filename =  "2_grams_big.words";
    let out_2_grams_words_path = out_path.to_string() + out_filename; 

    let flag_check_words = true;

    read_all_2_grams_words_of_file(&in_text_corpus_path, &out_2_grams_words_path, flag_check_words);
}

fn read_all_2_grams_words_of_file(in_text_corpus_path: &str, out_2_grams_words_path: &str, flag_check_hunspell: bool) {
    let hunspell = get_hunspell();

    let map_accents_tmp = map_accents();

    use std::env;
    
    let cur_dir = env::current_dir().unwrap();
    println!("\n current_dir: {}", cur_dir.to_string_lossy());
    println!("\n path: {}", &in_text_corpus_path);
    
    let input_string_corrected: String;
    {
        let input_string = fs::read_to_string(&in_text_corpus_path)
            .expect("Something went wrong reading the file");
        input_string_corrected = input_string.nfkc().collect::<String>();
    }

    let reg_ex_pt = LangRegEx::new(Lang::PT).reg_ex_comp_word_pattern;

    // HunSpell cache
    let mut hunspell_word_cache: HashMap<String, String> = HashMap::new(); 

    // Counts all words.
    let mut dic_2_grams_freq_words: BTreeMap<String, u64> = BTreeMap::new();

    let mut not_dic_2_grams_freq_words: BTreeMap<String, u64> = BTreeMap::new();

    let mut not_dic_2_grams_not_check_freq_words: BTreeMap<String, u64> = BTreeMap::new();

    for phrase in input_string_corrected.split('.') {
        let mut prev_word: Option<String> = None;
        for word in phrase.split_whitespace() {
            let captures = reg_ex_pt.captures(word);
            if captures.is_none() {
                let count = not_dic_2_grams_freq_words.entry(word.to_string()).or_insert(0);
                *count += 1;
                continue;
            }
            let captures = captures.unwrap();
            if let Some(word_only) = captures.get(0) {
                let mut lower_case_word = word_only.as_str().to_ascii_lowercase();

                /* Start */
                let flag_word_exists = hunspell.check(&lower_case_word);
                if !flag_word_exists {
                    // See if the correct map of the word to the hunspell dictionary is on the cache, if it is use it.
                    let correct_word = hunspell_word_cache.get(&lower_case_word);
                    let res;
                    if correct_word.is_none() {
                        let suggestion_vec = hunspell.suggest(&lower_case_word);
                        res = from_old_treaty_to_new_treaty_orthographic(&lower_case_word, &suggestion_vec, &map_accents_tmp);
                        if res.is_some() {
                            // Substituted the word for a correct word.
                            let new_word = res.unwrap();
                            hunspell_word_cache.insert(lower_case_word.clone(), new_word.clone());
                            // Let's use it!
                            lower_case_word = new_word;   
                        } else {
                            // Didn't found a correct substitution, so puts the word in the not check_dic.
                            hunspell_word_cache.insert(lower_case_word.clone(), "*".to_string() + &suggestion_vec.join(" "));
                            let count = not_dic_2_grams_not_check_freq_words.entry(lower_case_word + " -> " + &suggestion_vec.join(" ")).or_insert(0);
                            *count += 1;
                            prev_word = None;  // diff
                            continue;    
                        }
                    } else {
                        // Found the word in hunspell cache.
                        let inner_correct_word = correct_word.unwrap();
                        if inner_correct_word.starts_with("*") {
                            // The word is invalid, because it could find a correct word for it. 
                            prev_word = None; // diff
                            continue;
                        }
                        // The word that was on hunspell cache is a correct word.
                        // Let's use it!
                        lower_case_word = inner_correct_word.clone();
                    }
                }

                if prev_word.is_some() {
                    let word_2_grams = prev_word.unwrap() + " " +  &lower_case_word;
                    
                    let count = dic_2_grams_freq_words.entry(word_2_grams).or_insert(0);
                    *count += 1;
                }

                prev_word = Some(lower_case_word);

                /* End */
            }
        }
    }

    let dic_2_grams_freq_words_string = make_string_from_BTreeHap(&dic_2_grams_freq_words);

    let not_dic_2_grams_freq_words_string = make_string_from_BTreeHap(&not_dic_2_grams_freq_words);

    let not_dic_2_grams_not_check_freq_words_string = make_string_from_BTreeHap(&not_dic_2_grams_not_check_freq_words);

    // Save to file.
    let _res = fs::write(&out_2_grams_words_path, &dic_2_grams_freq_words_string);

    //println!("\nFile written: {}", &dic_unique_freq_words_string);

    // Save to file.
    
    let not_out_2_grams_words_path = out_2_grams_words_path.replace("2_grams", "not_2_grams");
    let _res = fs::write(&not_out_2_grams_words_path, &not_dic_2_grams_freq_words_string);

    // Save to file.
    let not_dic_2_grams_not_check_freq_words_path = out_2_grams_words_path.replace("2_grams", "not_2_grams_not_check_dic");
    let _res = fs::write(&not_dic_2_grams_not_check_freq_words_path, &not_dic_2_grams_not_check_freq_words_string);

    println!("\nWords not written: {}", &not_dic_2_grams_freq_words_string);

    println!("\n1. With text:\n{}", &out_2_grams_words_path);
    println!("\n2. With text:\n{}", &not_out_2_grams_words_path);

    println!("\n3. dic_2_grams_freq_words.len():\n  {}", dic_2_grams_freq_words.len());
}

enum Lang {
    PT,
    EN
}

struct LangRegEx {
    letters_class_for_word_pattern: String,
    reg_ex_str_word_pattern: String,
    reg_ex_comp_word_pattern: Regex
}

impl LangRegEx {
    fn new(lang: Lang) -> Self {
        match lang {
            Lang::PT => {
                    // PT regular expressions pre-compilation.

                    // Regular expression to detect words in the Portuguese language:
                    // [a-zãõàáéíóúâêôç]+([-][a-zãõàáéíóúâêôç]+)+|[a-zãõàáéíóúâêôç]+
                    let letter_for_word_pt = r#"[a-zãõàáéíóúâêôç]"#;
                    let reg_ex_pt = format!("(?i){0}+([-]{0}+)+|{0}+", letter_for_word_pt);
                    LangRegEx{ 
                        letters_class_for_word_pattern: letter_for_word_pt.to_string(), 
                        reg_ex_str_word_pattern: reg_ex_pt.to_string(), 
                        reg_ex_comp_word_pattern: Regex::new(&reg_ex_pt).unwrap(),
                    }              
                },

            Lang::EN => {
                    // EN regular expressions pre-compilation.

                    // Regular expression to detect words in the Portuguese language:
                    // [a-z]+['][a-z]+|[a-z]+
                    let letter_for_word_en = r#"[a-z]"#;
                    let reg_ex_en = format!("(?i){0}+[']{0}+|{0}+", letter_for_word_en);
                    LangRegEx{ 
                        letters_class_for_word_pattern: letter_for_word_en.to_string(), 
                        reg_ex_str_word_pattern: reg_ex_en.to_string(), 
                        reg_ex_comp_word_pattern: Regex::new(&reg_ex_en).unwrap(),
                    }              
                },
        }
    }

}

//********************
//********************
// Util functions
//

// Run function and return result with duration (seconds or nano seconds).
pub fn time_it<F, T>(f: F) -> (T, Duration)
        where F: FnOnce() -> T {
    
    let start = Instant::now();
    let res = f();
    let elapsed = start.elapsed();

    (res, elapsed)
}

fn decimal_mark2(s: String) -> String {
    let mut result = String::with_capacity(s.len() + ((s.len() - 1) / 3));
    let mut i = s.len();
    for c in s.chars() {
        result.push(c);
        i -= 1;
        if i > 0 && i % 3 == 0 {
            result.push('.');
        }
    }
    result
}
