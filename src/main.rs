use arboard::Clipboard;
use clap::Parser;
use summarizer::Summarizer;

mod summarizer;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Original clipped content
    src: String,
    /// Number of sentences in summary
    #[clap(short, long, value_parser, default_value_t = 5)]
    n: usize,
}

fn main() {
    let args = Args::parse();
    let mut clipboard = Clipboard::new().unwrap();
    // let text = include_str!("test/test_chinese.txt");
    // let text = "This is another simple sentence. I like to eat sandwich as my breakfirst.";

    let mut summarizer = Summarizer::default();
    summarizer
        .detect(&args.src)
        .expect("Cannot detect language");
    let summary = summarizer.fit(&args.src, args.n);
    clipboard
        .set_text(summary)
        .expect("Failed to save to clipboard");
}
