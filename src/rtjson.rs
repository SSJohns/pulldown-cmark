// SSJohns July 17th, 2017

//! RTJson renderer based on the HTML renderer

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write;

use parse::{Event, Tag};
use parse::Event::{Start, End, Text, Html, InlineHtml, SoftBreak, HardBreak, FootnoteReference};
use parse::Alignment;
use escape::{escape_html, escape_href};

enum TableState {
    Head,
    Body,
}

enum Element {
	text("text"),
	raw("raw"),
	link("link")
}

enum Format {
	bold = 1,
	italic = 2,
	underline = 4,
	strikethrough = 8,
	subscript = 16,
	superscript = 32,
	code = 64
}

struct FormatRange {
	format(Format),
	start(u8),
	length(u8)
}

struct FormatRangeArray {
	FormatRange(Vec<FormatRange>)
}

struct Text {
	e: 'text',
	t: String,
    f: FormatRangeArray
}

struct RawText {
	e: 'raw',
	t: String
}

struct UrlString {
	string: String
}

struct Link {
	e: Element,
	t: String,
	u: UrlString
}

enum Paragraph {
	text(String)
}

enum List {
	items(Vec)
}

enum TableCell {
	Cell(String)
}

enum Table {
	Head(Vec<String>)
	Columns(Vec<TableCell>)
}

struct Link {
    e: 'link',
    t: String,
    u: UrlString
}

struct UserLink {
	e: 'u/',
	t: String
}

struct SubredditLink {
	e: 'r/',
	t: String
}

struct PostLink {
	e: 'p/',
	t: String
}

struct CommentLink {
	e: 'c/',
	t: String
}

enum RedditLink {
	user(UserLink),
	subreddit(SubredditLink),
	postlink(PostLink),
	commentlink(CommentLink)
}

struct PostMediaId {
    p: String
}

struct Gallery {
	e: 'gallery',
    pId: PostMediaId,
    c: String,
    m: Vec<Media>
}

enum ListTypes {
    li(ListItem),
    list(List)
}

struct ListItem {
    e: 'li',
    c: Vec<TextNode>
}

struct List {
    e: 'list',
    o: bool,
    c: Vec<ListTypes>
}

enum CodeBlock {
	e: 'code',
    c: Vec<RawText>,
    l: String
}

struct Heading {
	e: 'h',
    l: u8,
    c: Vec<HeadingText>
}

struct LineBreak {
    e: 'br'
}

struct BlockQoute {
    e: 'blockquote',
    c: Vec<Paragraph>,
    a: TextNode
}

enum PlainText {
    text(Text),
    link(Link),
    rlink(RedditLink)
}

struct SpoilerText {
    e: 's',
    c: PlainText
}

enum TextNode {
    plain(PlainText),
    spoiler(SpoilerText)
}

enum HeadingText {
    raw(RawText),
    link(Link),
    rlink(RedditLink)
}

// Tables

enum ColumnAlignment {
    l('L'),
    r('R'),
    c('C')
}

struct TableCell {
    c: Vec<TextNode>
}

struct TableHeaderCell {
    a: ColumnAlignment,
    c: Vec<TextNode>
}

struct TableHeaderRow {
    r: Vec<TableHeaderCell>
}

struct TableRow {
    r: Vec<TableCell>
}

struct Table {
    e: 'table',
    h: TableHeaderRow,
    c: Vec<TableRow>
}

// Embeds

struct Embed {
    e: 'embed',
    u: UrlString,
    c: UrlString,
    x: u8,
    y: u8
}

enum DocumentNode {
	p(Paragraph),
	list(List),
	table(Table),
	MediaGallery(Media),
	CodeBlock(CodeBlock),
	Head(Heading),
	Image(Image),
	AnimatedImage(),
	Video(),
	Embed(),
	BlockQuote()
}

enum TableState {
    Head,
    Body,
}

struct Document {
	d: Vec<DocumentNode>
}

struct Ctx<'b, I> {
    iter: I,
    buf: &'b mut String,
    table_state: TableState,
    table_alignments: Vec<Alignment>,
    table_cell_index: usize,
    doc: Document,
}

impl<'a, 'b, I: Iterator<Item=Event<'a>>> Ctx<'b, I> {
    fn fresh_line(&mut self) {
        if !(self.buf.is_empty() || self.buf.ends_with('\n')) {
            self.buf.push('\n');
        }
    }

    pub fn run(&mut self) {
        let mut numbers = HashMap::new();
        while let Some(event) = self.iter.next() {
            match event {
                Start(tag) => self.start_tag(tag, &mut numbers),
                End(tag) => self.end_tag(tag),
                Text(text) => escape_html(self.buf, &text, false),
                Html(html) |
                InlineHtml(html) => self.buf.push_str(&html),
                SoftBreak => self.buf.push('\n'),
                HardBreak => self.buf.push_str("<br />\n"),
                FootnoteReference(name) => {
                    let len = numbers.len() + 1;
                    self.buf.push_str("<sup class=\"footnote-reference\"><a href=\"#");
                    escape_html(self.buf, &*name, false);
                    self.buf.push_str("\">");
                    let number = numbers.entry(name).or_insert(len);
                    self.buf.push_str(&*format!("{}", number));
                    self.buf.push_str("</a></sup>");
                },
            }
        }
    }

    fn start_tag(&mut self, tag: Tag<'a>, numbers: &mut HashMap<Cow<'a, str>, usize>) {
        match tag {
            Tag::Paragraph =>  {
                self.fresh_line();
                self.buf.push_str("<p>");
                let p = Paragraph {text: &text}
                self.doc.push(p)
            }
            Tag::Rule => {
                self.fresh_line();
                self.buf.push_str("<hr />\n")
            }
            Tag::Header(level) => {
                self.fresh_line();
                self.buf.push_str("<h");
                self.buf.push((b'0' + level as u8) as char);
                self.buf.push('>');
                let ht = HeadingText::text
                let h = Header { e: 'h', l: level as u8, c: vec![ht] }
                self.doc.push(h)
            }
            Tag::Table(alignments) => {
                self.table_alignments = alignments;
                self.buf.push_str("<table>");
                let t = Table {  }
            }
            Tag::TableHead => {
                self.table_state = TableState::Head;
                self.buf.push_str("<thead><tr>");
                let th = TableHeaderRow {  }
            }
            Tag::TableRow => {
                self.table_cell_index = 0;
                self.buf.push_str("<tr>");
            }
            Tag::TableCell => {
                match self.table_state {
                    TableState::Head => self.buf.push_str("<th"),
                    TableState::Body => self.buf.push_str("<td"),
                }
                match self.table_alignments.get(self.table_cell_index) {
                    Some(&Alignment::Left) => self.buf.push_str(" align=\"left\""),
                    Some(&Alignment::Center) => self.buf.push_str(" align=\"center\""),
                    Some(&Alignment::Right) => self.buf.push_str(" align=\"right\""),
                    _ => (),
                }
                self.buf.push_str(">");
            }
            Tag::BlockQuote => {
                self.fresh_line();
                self.buf.push_str("<blockquote>\n");
            }
            Tag::CodeBlock(info) => {
                self.fresh_line();
                let lang = info.split(' ').next().unwrap();
                if lang.is_empty() {
                    self.buf.push_str("<pre><code>");
                } else {
                    self.buf.push_str("<pre><code class=\"language-");
                    escape_html(self.buf, lang, false);
                    self.buf.push_str("\">");
                }
            }
            Tag::List(Some(1)) => {
                self.fresh_line();
                self.buf.push_str("<ol>\n");
            }
            Tag::List(Some(start)) => {
                self.fresh_line();
                let _ = write!(self.buf, "<ol start=\"{}\">\n", start);
            }
            Tag::List(None) => {
                self.fresh_line();
                self.buf.push_str("<ul>\n");
            }
            Tag::Item => {
                self.fresh_line();
                self.buf.push_str("<li>");
            }
            Tag::Emphasis => self.buf.push_str("<em>"),
            Tag::Strong => self.buf.push_str("<strong>"),
            Tag::Underline => self.buf.push_str("<u>"),
            Tag::Strikethrough => self.buf.push_str("<del>"),
            Tag::Code => self.buf.push_str("<code>"),
            Tag::Link(dest, title) => {
                self.buf.push_str("<a href=\"");
                escape_href(self.buf, &dest);
                if !title.is_empty() {
                    self.buf.push_str("\" title=\"");
                    escape_html(self.buf, &title, false);
                }
                self.buf.push_str("\">");
            }
            Tag::RedditLink(link_type, dest, trim_len) => {
                for _ in 0..trim_len {
                    self.buf.pop();
                }
                self.buf.push_str("<a href=\"");
                let redditlink = "/".to_owned() + &link_type + &dest;
                escape_href(self.buf, &redditlink);
                self.buf.push_str("\" />");
            }
            Tag::Image(dest, title) => {
                self.buf.push_str("<img src=\"");
                escape_href(self.buf, &dest);
                self.buf.push_str("\" alt=\"");
                self.raw_text(numbers);
                if !title.is_empty() {
                    self.buf.push_str("\" title=\"");
                    escape_html(self.buf, &title, false);
                }
                self.buf.push_str("\" />")
            }
            Tag::FootnoteDefinition(name) => {
                self.fresh_line();
                let len = numbers.len() + 1;
                self.buf.push_str("<div class=\"footnote-definition\" id=\"");
                escape_html(self.buf, &*name, false);
                self.buf.push_str("\"><sup class=\"footnote-definition-label\">");
                let number = numbers.entry(name).or_insert(len);
                self.buf.push_str(&*format!("{}", number));
                self.buf.push_str("</sup>");
            }
        }
    }

    fn end_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Paragraph => self.buf.push_str("</p>\n"),
            Tag::Rule => (),
            Tag::Header(level) => {
                self.buf.push_str("</h");
                self.buf.push((b'0' + level as u8) as char);
                self.buf.push_str(">\n");
            }
            Tag::Table(_) => {
                self.buf.push_str("</tbody></table>\n");
            }
            Tag::TableHead => {
                self.buf.push_str("</tr></thead><tbody>\n");
                self.table_state = TableState::Body;
            }
            Tag::TableRow => {
                self.buf.push_str("</tr>\n");
            }
            Tag::TableCell => {
                match self.table_state {
                    TableState::Head => self.buf.push_str("</th>"),
                    TableState::Body => self.buf.push_str("</td>"),
                }
                self.table_cell_index += 1;
            }
            Tag::BlockQuote => self.buf.push_str("</blockquote>\n"),
            Tag::CodeBlock(_) => self.buf.push_str("</code></pre>\n"),
            Tag::List(Some(_)) => self.buf.push_str("</ol>\n"),
            Tag::List(None) => self.buf.push_str("</ul>\n"),
            Tag::Item => self.buf.push_str("</li>\n"),
            Tag::Emphasis => self.buf.push_str("</em>"),
            Tag::Strong => self.buf.push_str("</strong>"),
            Tag::Underline => self.buf.push_str("</u>"),
            Tag::Strikethrough => self.buf.push_str("</del>"),
            Tag::Code => self.buf.push_str("</code>"),
            Tag::Link(_, _) => self.buf.push_str("</a>"),
            Tag::RedditLink(_, _, _) => self.buf.push_str("</a>"),
            Tag::Image(_, _) => (), // shouldn't happen, handled in start
            Tag::FootnoteDefinition(_) => self.buf.push_str("</div>\n"),
        }
    }

    // run raw text, consuming end tag
    fn raw_text<'c>(&mut self, numbers: &'c mut HashMap<Cow<'a, str>, usize>) {
        let mut nest = 0;
        while let Some(event) = self.iter.next() {
            match event {
                Start(_) => nest += 1,
                End(_) => {
                    if nest == 0 { break; }
                    nest -= 1;
                }
                Text(text) => escape_html(self.buf, &text, false),
                Html(_) => (),
                InlineHtml(html) => escape_html(self.buf, &html, false),
                SoftBreak | HardBreak => self.buf.push(' '),
                FootnoteReference(name) => {
                    let len = numbers.len() + 1;
                    let number = numbers.entry(name).or_insert(len);
                    self.buf.push_str(&*format!("[{}]", number));
                }
            }
        }
    }
}

pub fn push_html<'a, I: Iterator<Item=Event<'a>>>(buf: &mut String, iter: I) {
    let mut ctx = Ctx {
        iter: iter,
        buf: buf,
        table_state: TableState::Head,
        table_alignments: vec![],
        table_cell_index: 0,
    };
    ctx.run();
}
