use ffmpeg::format::chapter::Chapter;
use std::fmt;

pub struct ChapterInfo {
    pub title: String,
    pub start: f64,
    pub end: f64,
}

impl ChapterInfo {
    pub fn new(chapter: &Chapter) -> Self {
        let title = match chapter.metadata().get("title") {
            Some(title) => title.to_string(),
            None => chapter.id().to_string(),
        };
        let denominator = chapter.time_base().denominator() as f64;
        let start = (chapter.start() as f64) / denominator;
        let end = (chapter.end() as f64) / denominator;

        ChapterInfo {
            title: title,
            start: start,
            end: end,
        }
    }
}

impl fmt::Display for ChapterInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {} - {}", self.title, self.start, self.end)
    }
}
