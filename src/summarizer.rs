use std::collections::{HashMap, HashSet};

use lingua::Language::{self, Chinese, English};
use lingua::{LanguageDetector, LanguageDetectorBuilder};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub enum SumError {
    NoLangFit,
}

#[derive(Default)]
pub struct Summarizer {
    lang: Option<Language>,
}

impl Summarizer {
    pub fn new(lang: Language) -> Self {
        Self {
            lang: Some(lang),
            ..Self::default()
        }
    }

    pub fn get_lang(&self) -> &Option<Language> {
        &self.lang
    }

    pub fn detect(&mut self, src: &str) -> Result<(), SumError> {
        let languages = vec![English, Chinese];
        let detector: LanguageDetector =
            LanguageDetectorBuilder::from_languages(&languages).build();
        let detected_lang: Option<Language> = detector.detect_language_of(src);

        match detected_lang {
            Some(lang) => {
                self.lang = Some(lang);
                Ok(())
            }
            None => Err(SumError::NoLangFit),
        }
    }

    pub fn tokenize<'a>(
        &self,
        src: &'a str,
        tokens: &mut HashMap<&'a str, usize>,
    ) -> Option<Vec<usize>> {
        let mut sequence = vec![];

        match &self.lang {
            Some(lang_type) => match lang_type {
                Language::English => {
                    let mut index = tokens.len() + 1;

                    src.unicode_words().for_each(|w| {
                        if let Some(&i) = tokens.get(w) {
                            sequence.push(i);
                        } else {
                            sequence.push(index);
                            tokens.insert(w, index);
                            index += 1;
                        }
                    });

                    Some(sequence)
                }
                Language::Chinese => {
                    let mut index = tokens.len() + 1;

                    src.unicode_words().for_each(|w| {
                        if let Some(&i) = tokens.get(w) {
                            sequence.push(i);
                        } else {
                            sequence.push(index);
                            tokens.insert(w, index);
                            index += 1;
                        }
                    });

                    Some(sequence)
                }
            },
            None => None,
        }
    }

    pub fn fit(&self, src: &str, n: usize) -> String {
        let mut tokens = HashMap::new();
        let sentences = src.unicode_sentences().collect::<Vec<&str>>();
        let sequences = sentences
            .iter()
            .map(|s| self.tokenize(s, &mut tokens).unwrap_or(vec![]))
            .collect::<Vec<Vec<usize>>>();

        let mut relevance_matrix = vec![];
        let mut tr_scores: Vec<(usize, f64)> = vec![];
        let mut sum = 0.0;
        let dump = 0.8;
        for i in 0..sequences.len() {
            let mut temp = vec![];
            for j in 0..sequences.len() {
                let relevance = if i == j {
                    0.0
                } else {
                    compute_relevance(&sequences[i], &sequences[j])
                };
                temp.push(relevance);
                sum += relevance;
            }

            // add reletion rating to matrix
            relevance_matrix.push(temp);
        }
        // print_matrix(&relevance_matrix);

        // calculate TextRank score for each sentence
        for (index, sen) in relevance_matrix.iter().enumerate() {
            tr_scores.push((
                index,
                (1.0 - dump) + (dump * sen.iter().map(|rating| rating / sum).sum::<f64>()),
            ));
        }

        // get n most important sentences
        let nth_max = max_n(&tr_scores, n);
        let summary = nth_max
            .iter()
            .map(|&i| sentences[i])
            .collect::<Vec<&str>>()
            .join("");

        println!("{}", summary);

        summary
    }
}

fn compute_relevance(s1: &Vec<usize>, s2: &Vec<usize>) -> f64 {
    let s1_set: HashSet<&usize> = HashSet::from_iter(s1.iter());
    let s2_set: HashSet<&usize> = HashSet::from_iter(s2.iter());

    let num_common = s1_set.intersection(&s2_set).count();
    let relevance = num_common as f64 / ((s1.len() as f64).log10() + (s2.len() as f64).log10());

    // println!(
    //     "s1: {:?}\ns2:{:?}\nRelevance: {}\nNumber of common token: {}",
    //     s1, s2, relevance, num_common
    // );
    relevance
}

/// Return the maximum n elements in the array. Array length must be larger than n.
fn max_n(arr: &Vec<(usize, f64)>, n: usize) -> Vec<usize> {
    let mut maxs = vec![];
    let mut copied = arr.clone();

    while maxs.len() < n && copied.len() > 0 {
        if let Some((index, v)) = copied
            .iter()
            .enumerate()
            .max_by(|(_, x), (_, y)| x.1.partial_cmp(&y.1).expect(&format!("{:?} {:?}", x, y)))
        {
            maxs.insert(0, v.0);
            copied.remove(index);
        }
    }

    maxs.sort();
    maxs
}

fn print_matrix<T: std::fmt::Display>(m: &Vec<Vec<T>>) {
    for i in m {
        print!("[");
        for (index, j) in i.iter().enumerate() {
            if index == i.len() - 1 {
                print!("{:.3}", j);
            } else {
                print!("{:.3}, ", j);
            }
        }
        println!("]");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_chinese() {
        let cns_text = "这是一段中文句子，测试一下能否识别。";
        let eng_text = "This is a short Engilsh sentence.";

        let mut summarizer1 = Summarizer::default();
        let mut summarizer2 = Summarizer::default();
        let cns_res = summarizer1.detect(cns_text);
        let eng_res = summarizer2.detect(eng_text);

        // test fit correctly
        assert!(cns_res.is_ok());
        assert!(eng_res.is_ok());

        // test correct language detected
        assert_eq!(summarizer1.get_lang(), &Some(Chinese));
        assert_eq!(summarizer2.get_lang(), &Some(English));
    }

    #[test]
    fn test_english_sequencing() {
        let text = "This is a simple sentence. This is another simple sentence.";
        let mut summarizer = Summarizer::default();
        summarizer.detect(text).expect("Cannot detect language");
        summarizer.fit(text, 5);
    }
}
