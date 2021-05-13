use crate::application::Config;
use crate::colorscheme;
use colorscheme::ToForeground;
use tui::style::Color;

use super::utils::randorst::Randorst;
use fastrand::Rng as FastRng;
use std::fs::File;
use std::io::{BufRead, BufReader};
use tui::text::Span;

use rand::distributions::WeightedIndex;
use rand::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum Punctuation {
    // comma, doesnt warrant Capital letter
    Normal(char),
    // full stop, exclamation etc
    End(char),
    // brackets of all kind, and dquotes
    Paired(char, char),
    // gonna use it like an em dash so in between words "word - word"
    DashLike(char),
    Null,
}

#[allow(dead_code)]
struct PFreq {
    weighted_index: WeightedIndex<u16>,
    symbols: Vec<Punctuation>,
}

impl Default for PFreq {
    fn default() -> Self {
        let we = vec![
            (Punctuation::End('.'), 65),
            (Punctuation::End('?'), 6),
            (Punctuation::End('!'), 3),
            (Punctuation::Normal(','), 61),
            (Punctuation::Normal(';'), 3),
            (Punctuation::Normal(':'), 3),
            (Punctuation::Paired('<', '>'), 2),
            (Punctuation::Paired('(', ')'), 3),
            (Punctuation::Paired('{', '}'), 2),
            (Punctuation::Paired('[', ']'), 2),
            (Punctuation::Paired('"', '"'), 13),
            (Punctuation::Paired('\'', '\''), 10),
            (Punctuation::DashLike('-'), 10),
            (Punctuation::Null, 800),
        ];

        let mut weighted_index: Vec<u16> = Vec::with_capacity(we.len());
        let mut symbols: Vec<Punctuation> = Vec::with_capacity(we.len());
        for (p, w) in we.into_iter() {
            weighted_index.push(w);
            symbols.push(p);
        }

        let weighted_index = WeightedIndex::new(weighted_index).unwrap();
        Self {
            weighted_index,
            symbols,
        }
    }
}

#[allow(dead_code)]
impl PFreq {
    fn choose(&self, rng: &mut ThreadRng) -> Punctuation {
        self.symbols[self.weighted_index.sample(rng)]
    }
}

fn get_shuffled_words(config: &Config) -> Vec<String> {
    let file = File::open(&config.get_source()).expect("couldn't open file");
    let reader = BufReader::new(file);
    let mut line_iter = reader.lines();
    let mut container: Vec<String> = Vec::new();

    let mut prng = Randorst::gen(config.length, 0..config.freq_cut_off);
    let mut last = prng.next().unwrap();
    let out = line_iter.nth(last).unwrap().unwrap();
    container.push(out);
    let mut cached_word: usize = container.len() - 1;

    for (i, val) in prng.enumerate() {
        if val == last {
            container.push(container[cached_word].to_string());
            continue;
        }
        container.push(line_iter.nth(val - last - 1).unwrap().unwrap());
        cached_word = i + 1;
        last = val;
    }

    FastRng::new().shuffle(&mut container);
    container
}

fn add_space_with_blank(container: &mut Vec<Span>, wrong: Color, todo: Color) {
    container.push(Span::styled("", wrong.fg()));
    container.push(Span::styled(" ", todo.fg()));
}

fn punctuation_word_prep(
    container: &mut Vec<Span>,
    word: &str,
    punctuation: Punctuation,
    todo: Color,
) {
    for c in word.chars() {
        container.push(Span::styled(c.to_string(), todo.fg()));
    }
}

pub fn prep_test<'a>(
    config: &Config,
    limit: usize,
    wrong: Color,
    todo: Color,
) -> Vec<Vec<Span<'a>>> {
    let prep = get_shuffled_words(config);
    let mut test: Vec<Vec<Span>> = vec![];
    let mut tmp: Vec<Vec<Span>> = vec![vec![]];
    let mut count = 0;

    let p = PFreq::default();

    let flag = false;

    match flag {
        true => {
            for word in &prep {
                count += word.len() + 1;
                if count > limit {
                    test.append(&mut tmp);
                    count = word.len();
                    tmp.push(vec![]);
                }

                for c in word.chars() {
                    tmp[0].push(Span::styled(c.to_string(), todo.fg()));
                }

                add_space_with_blank(&mut tmp[0], wrong, todo);
            }
        }
        false => {
            let mut rng = thread_rng();
            let mut capitalize: bool = false;
            let mut begin: Option<char> = None;
            let mut end: Option<char> = None;

            for word in &prep {
                count += word.len() + 1;
                if count > limit {
                    test.append(&mut tmp);
                    count = word.len();
                    tmp.push(vec![]);
                }

                let punct = p.choose(&mut rng);
                match punct {
                    Punctuation::Null => {
                        begin = None;
                        end = None;
                    }

                    Punctuation::End(c) => {
                        capitalize = true;
                        begin = None;
                        end = Some(c);
                    }

                    Punctuation::Normal(c) => {
                        begin = None;
                        end = Some(c);
                    }

                    Punctuation::Paired(a, z) => {
                        begin = Some(a);
                        end = Some(z);
                    }

                    // TODO implement this bullshit
                    // i am kinda fed up with what this became
                    // need to think it through
                    Punctuation::DashLike(_) => {
                        begin = None;
                        end = None;
                    }
                }

                let mut iter_chars = word.chars();

                if let Some(c) = begin {
                    tmp[0].push(Span::styled(c.to_string(), todo.fg()));
                }

                if capitalize {
                    let upper = iter_chars
                        .next()
                        .expect("word should never be empty")
                        .to_uppercase();
                    for upper_char in upper {
                        tmp[0].push(Span::styled(upper_char.to_string(), todo.fg()));
                    }
                }
                capitalize = false;


                for c in iter_chars {
                    tmp[0].push(Span::styled(c.to_string(), todo.fg()));
                }

                if let Some(c) = end {
                    tmp[0].push(Span::styled(c.to_string(), todo.fg()));
                }

                add_space_with_blank(&mut tmp[0], wrong, todo);
            }
        }
    };

    // get rid of the space and blank at the end
    let last = tmp.len() - 1;
    tmp[last].pop();
    tmp[last].pop();

    // change order so the [0] element is the one with potentially fewer words
    // and with no space at the end
    // makes it more convienient to pop from the vec to get a new line
    test.append(&mut tmp);
    let last = test.len() - 1;
    test.swap(0, last);

    test
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::Config;
    use tui::style::Color;

    #[test]
    fn test_prep() {
        let mut cfg = Config::default();
        cfg.length = 200;
        let mut words = 1;
        let limit = 65;
        let mut char_count = 0;

        let result = prep_test(&cfg, limit, Color::Red, Color::Blue);
        for line in &result {
            for span in line {
                if span.content == " " {
                    words += 1;
                }
                if !span.content.is_empty() {
                    char_count += 1;
                }
            }
            // there can be space at the end and I dont care for it
            assert!(char_count <= limit + 1);
            char_count = 0;
        }

        assert_eq!(words, cfg.length);
    }
}
