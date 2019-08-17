
# Local Search

A simple command line search engine for your local plain text files.

Work in progress! Most functionality has been implemented, but is likely to be
rough around the edges.

## Installation

Standard rust project. `cargo install` or git clone and `cargo build` should
work fine.


## Usage

1. Create a config file to configure what files you want indexed and where to
   store the indexes. (see configuration section below)
2. Create the index: `mkdir <my index location> && local-search index`
3. Search: `local-search search "foo bar"

See [tantivy
docs](https://docs.rs/tantivy/0.10.1/tantivy/query/struct.QueryParser.html) for
the supported query syntax.


## Configuration

See [example_config.toml](./example_config.toml) for an example. The software
searches for the configuration file in the following locations in order, and
uses the first match:

1. path passed to `-c` on the command line
2. `$XDG_CONFIG_HOME/local-search/config.toml`
3. `$HOME/.config/local-search/config.toml`

If none are found, the software exits with an error.


## Notes

- builds indexes and searches with [tantivy](https://github.com/tantivy-search/tantivy). This also means that other tantivy-based tools can be used on the indexes, like [tantivy-cli](https://github.com/tantivy-search/tantivy-cli).

## Why?

My note taking workflow involves a large number of plain text files loosely
inspired by
[Zettelkasten](https://zettelkasten.de/posts/zettelkasten-improves-thinking-writing/)
and [vimwiki](https://github.com/vimwiki/vimwiki).
Since I generally avoid categories and hierarchies, various types of searches
is the main method I use for finding a note. Currently there are two methods I
use for finding a note:

1. fuzzy text search on the note filenames (Fzf or [this
   script](https://github.com/swalladge/dotfiles/blob/master/bin/open-wiki-page))
2. grep through the files

The first method is perfect if I roughly know what what I'm looking for and
know there's a note with that in the title. The second is great if I'm looking
for a specific pattern or word. What's missing though is a general full text
content search.  Grepping doesn't work so well when you're looking for a set of
keywords. So that's where this tool comes in! Now I have a third method for
searching my notes that sits between a fuzzy title search and specific regex
searches.

There are other full text search engines around, but so far I haven't found
something that is small and simple. I wanted something that I could configure
and set up by nothing more than editing a short config file and running a
single command.



## License

Copyright © 2019 Samuel Walladge

Dual licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
