use ffmpeg::format::chapter::Chapter;
use std::fmt;

crate struct ChapterInfo {
    crate title: String,
    crate start: f64,
    crate end: f64,
}

impl ChapterInfo {
    crate fn new(chapter: &Chapter<'_>) -> Self {
        let title = match chapter.metadata().get("title") {
            Some(title) => title.to_string(),
            None => chapter.id().to_string(),
        };
        let denominator = f64::from(chapter.time_base().denominator());
        let start = (chapter.start() as f64) / denominator;
        let end = (chapter.end() as f64) / denominator;

        ChapterInfo { title, start, end }
    }
}

impl fmt::Display for ChapterInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} - {}", self.title, self.start, self.end)
    }
}
