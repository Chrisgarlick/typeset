use printpdf::*;
use printpdf::path::PaintMode;

use crate::brand::engine::{wrap_text, BrandedDocument};
use crate::models::document::DocumentNode;

fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color::Rgb(Rgb::new(r, g, b, None))
}

fn make_line(x1: f32, y1: f32, x2: f32, y2: f32) -> Line {
    Line {
        points: vec![
            (Point::new(Mm(x1), Mm(y1)), false),
            (Point::new(Mm(x2), Mm(y2)), false),
        ],
        is_closed: false,
    }
}

fn filled_rect(x1: f32, y1: f32, x2: f32, y2: f32) -> Rect {
    Rect::new(Mm(x1), Mm(y1), Mm(x2), Mm(y2)).with_mode(PaintMode::Fill)
}

pub fn render(branded: &BrandedDocument) -> anyhow::Result<Vec<u8>> {
    let title = branded
        .doc
        .frontmatter
        .as_ref()
        .and_then(|f| f.title.as_deref())
        .unwrap_or("Document");

    let (doc, page1, layer1) = PdfDocument::new(
        title,
        Mm(branded.page.width_mm),
        Mm(branded.page.height_mm),
        "Layer 1",
    );

    let heading_font = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
    let body_font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
    let mono_font = doc.add_builtin_font(BuiltinFont::Courier)?;

    let mut renderer = PdfPageRenderer {
        doc: &doc,
        branded,
        heading_font: &heading_font,
        body_font: &body_font,
        mono_font: &mono_font,
        current_page: page1,
        current_layer: layer1,
        cursor_y: branded.page.height_mm - branded.page.margin_top_mm,
        page_number: 1,
        total_pages: 1,
        all_pages: vec![(page1, layer1)],
        cover_rendered: false,
    };

    let has_title = branded
        .doc
        .frontmatter
        .as_ref()
        .and_then(|f| f.title.as_ref())
        .is_some();
    let render_cover = branded.profile.cover_enabled || has_title;

    if render_cover {
        renderer.render_cover()?;
        renderer.new_page();
        // new_page() already drew the header on page 2; no further header call needed.
    } else if branded.profile.header_enabled {
        renderer.render_header()?;
    }

    for node in &branded.doc.nodes {
        renderer.render_node(node)?;
    }

    if branded.profile.footer_enabled {
        renderer.apply_footers()?;
    }

    let bytes = doc.save_to_bytes()?;
    Ok(bytes)
}

struct PdfPageRenderer<'a> {
    doc: &'a PdfDocumentReference,
    branded: &'a BrandedDocument,
    heading_font: &'a IndirectFontRef,
    body_font: &'a IndirectFontRef,
    mono_font: &'a IndirectFontRef,
    current_page: PdfPageIndex,
    current_layer: PdfLayerIndex,
    cursor_y: f32,
    page_number: u32,
    total_pages: u32,
    /// Store all (page, layer) indices for footer pass
    all_pages: Vec<(PdfPageIndex, PdfLayerIndex)>,
    cover_rendered: bool,
}

impl<'a> PdfPageRenderer<'a> {
    fn layer(&self) -> PdfLayerReference {
        self.doc
            .get_page(self.current_page)
            .get_layer(self.current_layer)
    }

    fn new_page(&mut self) {
        let (page, layer) = self.doc.add_page(
            Mm(self.branded.page.width_mm),
            Mm(self.branded.page.height_mm),
            "Layer 1",
        );
        self.current_page = page;
        self.current_layer = layer;
        self.cursor_y = self.branded.page.height_mm - self.branded.page.margin_top_mm;
        self.page_number += 1;
        self.total_pages += 1;
        self.all_pages.push((page, layer));

        if self.branded.profile.header_enabled {
            let _ = self.render_header();
        }
    }

    fn check_page_break(&mut self, needed_height: f32) {
        let bottom = self.branded.page.margin_bottom_mm + 15.0;
        if self.cursor_y - needed_height < bottom {
            self.new_page();
        }
    }

    fn render_node(&mut self, node: &DocumentNode) -> anyhow::Result<()> {
        match node {
            DocumentNode::Heading { level, text, .. } => self.render_heading(*level, text),
            DocumentNode::Paragraph { text } => self.render_paragraph(text),
            DocumentNode::BulletList { items } => self.render_bullet_list(items),
            DocumentNode::OrderedList { items, start } => self.render_ordered_list(items, *start),
            DocumentNode::Table { headers, rows } => self.render_table(headers, rows),
            DocumentNode::CodeBlock { code, .. } => self.render_code_block(code),
            DocumentNode::Blockquote { text } => self.render_blockquote(text),
            DocumentNode::HorizontalRule => self.render_rule(),
            DocumentNode::PageBreak => {
                self.new_page();
                Ok(())
            }
            DocumentNode::Image { .. } => Ok(()), // deferred to v1.1
        }
    }

    fn render_heading(&mut self, level: u8, text: &str) -> anyhow::Result<()> {
        let (size, colour, spacing_before) = match level {
            1 => (self.branded.profile.font_size_h1, self.branded.colours.primary, 12.0),
            2 => (self.branded.profile.font_size_h2, self.branded.colours.secondary, 10.0),
            3 => (self.branded.profile.font_size_h3, self.branded.colours.text, 8.0),
            _ => (self.branded.profile.font_size_base + 1.0, self.branded.colours.text, 6.0),
        };

        let lh = pt_to_mm(size * self.branded.profile.line_height);
        let body_lh = pt_to_mm(self.branded.profile.font_size_base * self.branded.profile.line_height);
        self.check_page_break(lh + spacing_before + 4.0 + body_lh * 3.0);
        self.cursor_y -= spacing_before;

        let layer = self.layer();
        let [r, g, b] = colour;
        layer.set_fill_color(rgb(r, g, b));
        layer.use_text(text, size as f32, Mm(self.branded.page.margin_left_mm), Mm(self.cursor_y), self.heading_font);

        if level == 1 {
            self.cursor_y -= 2.0;
            let [ar, ag, ab] = self.branded.colours.accent;
            layer.set_outline_color(rgb(ar, ag, ab));
            layer.set_outline_thickness(0.5);
            layer.add_line(make_line(
                self.branded.page.margin_left_mm,
                self.cursor_y,
                self.branded.page.margin_left_mm + self.branded.page.content_width_mm,
                self.cursor_y,
            ));
        }

        self.cursor_y -= lh + 2.0;
        Ok(())
    }

    fn render_paragraph(&mut self, text: &str) -> anyhow::Result<()> {
        let size = self.branded.profile.font_size_base;
        let lh = pt_to_mm(size * self.branded.profile.line_height);
        let cpl = (self.branded.page.content_width_mm / (size * 0.35)) as usize;
        let lines = wrap_text(text, cpl);

        self.check_page_break((lh * 3.0).min(lines.len() as f32 * lh));

        let [r, g, b] = self.branded.colours.text;
        for line in &lines {
            self.check_page_break(lh);
            let layer = self.layer();
            layer.set_fill_color(rgb(r, g, b));
            layer.use_text(line, size as f32, Mm(self.branded.page.margin_left_mm), Mm(self.cursor_y), self.body_font);
            self.cursor_y -= lh;
        }
        self.cursor_y -= self.branded.profile.paragraph_spacing;
        Ok(())
    }

    fn render_bullet_list(&mut self, items: &[String]) -> anyhow::Result<()> {
        let size = self.branded.profile.font_size_base;
        let lh = pt_to_mm(size * self.branded.profile.line_height);
        let cpl = ((self.branded.page.content_width_mm - 8.0) / (size * 0.35)) as usize;
        let [r, g, b] = self.branded.colours.text;
        let left = self.branded.page.margin_left_mm;

        for item in items {
            let lines = wrap_text(item, cpl);
            for (i, line) in lines.iter().enumerate() {
                self.check_page_break(lh + 1.0);
                let layer = self.layer();
                if i == 0 {
                    let [ar, ag, ab] = self.branded.colours.accent;
                    layer.set_fill_color(rgb(ar, ag, ab));
                    layer.use_text("\u{2022}", size as f32, Mm(left + 2.0), Mm(self.cursor_y), self.body_font);
                }
                layer.set_fill_color(rgb(r, g, b));
                layer.use_text(line, size as f32, Mm(left + 8.0), Mm(self.cursor_y), self.body_font);
                self.cursor_y -= lh;
            }
            self.cursor_y -= 1.0;
        }
        self.cursor_y -= self.branded.profile.paragraph_spacing;
        Ok(())
    }

    fn render_ordered_list(&mut self, items: &[String], start: u64) -> anyhow::Result<()> {
        let size = self.branded.profile.font_size_base;
        let lh = pt_to_mm(size * self.branded.profile.line_height);
        let cpl = ((self.branded.page.content_width_mm - 10.0) / (size * 0.35)) as usize;
        let [r, g, b] = self.branded.colours.text;
        let left = self.branded.page.margin_left_mm;

        for (i, item) in items.iter().enumerate() {
            let lines = wrap_text(item, cpl);
            for (j, line) in lines.iter().enumerate() {
                self.check_page_break(lh + 1.0);
                let layer = self.layer();
                if j == 0 {
                    let label = format!("{}.", i as u64 + start);
                    let [sr, sg, sb] = self.branded.colours.secondary;
                    layer.set_fill_color(rgb(sr, sg, sb));
                    layer.use_text(&label, size as f32, Mm(left + 2.0), Mm(self.cursor_y), self.body_font);
                }
                layer.set_fill_color(rgb(r, g, b));
                layer.use_text(line, size as f32, Mm(left + 10.0), Mm(self.cursor_y), self.body_font);
                self.cursor_y -= lh;
            }
            self.cursor_y -= 1.0;
        }
        self.cursor_y -= self.branded.profile.paragraph_spacing;
        Ok(())
    }

    fn render_table(&mut self, headers: &[String], rows: &[Vec<String>]) -> anyhow::Result<()> {
        if headers.is_empty() {
            return Ok(());
        }
        let cw = self.branded.page.content_width_mm / headers.len() as f32;
        let rh = pt_to_mm(self.branded.profile.font_size_base * 2.2);
        let size = self.branded.profile.font_size_base;
        let left = self.branded.page.margin_left_mm;

        self.check_page_break((rh * 4.0).min((rows.len() + 1) as f32 * rh));

        // Header row
        {
            let layer = self.layer();
            let [hr, hg, hb] = self.branded.colours.table_header;
            layer.set_fill_color(rgb(hr, hg, hb));
            layer.add_rect(filled_rect(left, self.cursor_y - rh, left + self.branded.page.content_width_mm, self.cursor_y));

            let [r, g, b] = self.branded.colours.text;
            layer.set_fill_color(rgb(r, g, b));
            for (i, h) in headers.iter().enumerate() {
                layer.use_text(h, (size - 0.5) as f32, Mm(left + i as f32 * cw + 2.0), Mm(self.cursor_y - rh + 2.0), self.heading_font);
            }
        }
        self.cursor_y -= rh;

        // Data rows
        for row in rows {
            self.check_page_break(rh);
            let layer = self.layer();
            let [r, g, b] = self.branded.colours.text;
            layer.set_fill_color(rgb(r, g, b));

            for (i, cell) in row.iter().enumerate() {
                let max_chars = (cw / (size * 0.35)) as usize;
                let display = if cell.len() > max_chars && max_chars > 3 {
                    format!("{}...", &cell[..max_chars - 3])
                } else {
                    cell.clone()
                };
                layer.use_text(&display, (size - 0.5) as f32, Mm(left + i as f32 * cw + 2.0), Mm(self.cursor_y - rh + 2.0), self.body_font);
            }

            let [br, bg, bb] = self.branded.colours.table_border;
            layer.set_outline_color(rgb(br, bg, bb));
            layer.set_outline_thickness(0.2);
            layer.add_line(make_line(left, self.cursor_y - rh, left + self.branded.page.content_width_mm, self.cursor_y - rh));
            self.cursor_y -= rh;
        }
        self.cursor_y -= self.branded.profile.section_spacing;
        Ok(())
    }

    fn render_blockquote(&mut self, text: &str) -> anyhow::Result<()> {
        let size = self.branded.profile.font_size_base;
        let lh = pt_to_mm(size * self.branded.profile.line_height);
        let pad = 4.0_f32;
        let indent = 10.0_f32;
        let cpl = ((self.branded.page.content_width_mm - indent - 4.0) / (size * 0.35)) as usize;
        let lines = wrap_text(text, cpl);
        let bh = lines.len() as f32 * lh + pad * 2.0;
        let left = self.branded.page.margin_left_mm;

        self.check_page_break(bh + 4.0);
        let layer = self.layer();

        // Background
        let [cr, cg, cb] = self.branded.colours.callout_bg;
        layer.set_fill_color(rgb(cr, cg, cb));
        layer.add_rect(filled_rect(left, self.cursor_y - bh, left + self.branded.page.content_width_mm, self.cursor_y));

        // Accent bar
        let [ar, ag, ab] = self.branded.colours.accent;
        layer.set_fill_color(rgb(ar, ag, ab));
        layer.add_rect(filled_rect(left, self.cursor_y - bh, left + 2.5, self.cursor_y));

        // Text
        let [tr, tg, tb] = self.branded.colours.text;
        layer.set_fill_color(rgb(tr, tg, tb));
        let mut y = self.cursor_y - pad - lh;
        for line in &lines {
            layer.use_text(line, size as f32, Mm(left + indent), Mm(y), self.body_font);
            y -= lh;
        }

        self.cursor_y -= bh + self.branded.profile.paragraph_spacing;
        Ok(())
    }

    fn render_code_block(&mut self, code: &str) -> anyhow::Result<()> {
        let size = self.branded.profile.font_size_base - 1.5;
        let lh = pt_to_mm(size * 1.4);
        let pad = 4.0_f32;
        let code_lines: Vec<&str> = code.lines().collect();
        let left = self.branded.page.margin_left_mm;
        let right = left + self.branded.page.content_width_mm;
        let bottom_limit = self.branded.page.margin_bottom_mm + 15.0;
        let usable_per_page =
            self.branded.page.height_mm - self.branded.page.margin_top_mm - bottom_limit;

        // If the whole block fits on a fresh page but not the current one, move it.
        let full_height = code_lines.len() as f32 * lh + pad * 2.0;
        let available_now = self.cursor_y - bottom_limit;
        if full_height > available_now && full_height <= usable_per_page {
            self.new_page();
        }

        // Render segment-by-segment, painting a dark rectangle behind each segment.
        let mut idx = 0;
        while idx < code_lines.len() {
            let available = self.cursor_y - bottom_limit - pad * 2.0;
            let max_lines = ((available / lh).floor() as usize).max(1);
            let take = max_lines.min(code_lines.len() - idx);
            let segment_height = take as f32 * lh + pad * 2.0;

            {
                let layer = self.layer();
                layer.set_fill_color(rgb(0.12, 0.12, 0.14));
                layer.add_rect(filled_rect(
                    left,
                    self.cursor_y - segment_height,
                    right,
                    self.cursor_y,
                ));
            }

            let mut y = self.cursor_y - pad - lh;
            for line in &code_lines[idx..idx + take] {
                let layer = self.layer();
                layer.set_fill_color(rgb(0.85, 0.87, 0.89));
                layer.use_text(*line, size as f32, Mm(left + 4.0), Mm(y), self.mono_font);
                y -= lh;
            }

            self.cursor_y -= segment_height;
            idx += take;

            if idx < code_lines.len() {
                self.new_page();
            }
        }

        self.cursor_y -= self.branded.profile.paragraph_spacing;
        Ok(())
    }

    fn render_rule(&mut self) -> anyhow::Result<()> {
        self.check_page_break(8.0);
        self.cursor_y -= 4.0;
        let layer = self.layer();
        let [r, g, b] = self.branded.colours.table_border;
        layer.set_outline_color(rgb(r, g, b));
        layer.set_outline_thickness(0.3);
        layer.add_line(make_line(
            self.branded.page.margin_left_mm,
            self.cursor_y,
            self.branded.page.margin_left_mm + self.branded.page.content_width_mm,
            self.cursor_y,
        ));
        self.cursor_y -= 4.0;
        Ok(())
    }

    fn render_cover(&mut self) -> anyhow::Result<()> {
        let page_w = self.branded.page.width_mm;
        let page_h = self.branded.page.height_mm;
        let left = self.branded.page.margin_left_mm;

        let fm = self.branded.doc.frontmatter.as_ref();
        let title = fm.and_then(|f| f.title.clone()).unwrap_or_else(|| "Untitled".to_string());
        let subtitle = fm.and_then(|f| f.subtitle.clone());
        let recipient = fm.and_then(|f| f.recipient.clone());
        let date = fm.and_then(|f| f.date.clone());
        let author = fm.and_then(|f| f.author.clone());

        let [pr, pg, pb] = self.branded.colours.primary;
        let [ar, ag, ab] = self.branded.colours.accent;
        let [sr, sg, sb] = self.branded.colours.secondary;
        let [tr, tg, tb] = self.branded.colours.text;
        let [br, bg, bb] = self.branded.colours.table_border;

        let layer = self.layer();

        // Top accent band (full width, primary colour)
        let top_band = 22.0_f32;
        layer.set_fill_color(rgb(pr, pg, pb));
        layer.add_rect(filled_rect(0.0, page_h - top_band, page_w, page_h));

        // Bottom accent band (thinner)
        let bottom_band = 8.0_f32;
        layer.set_fill_color(rgb(pr, pg, pb));
        layer.add_rect(filled_rect(0.0, 0.0, page_w, bottom_band));

        // Title — large, ~55% from top
        let title_size = self.branded.profile.font_size_h1 * 2.0;
        let title_y = page_h * 0.58;
        layer.set_fill_color(rgb(pr, pg, pb));
        layer.use_text(&title, title_size as f32, Mm(left), Mm(title_y), self.heading_font);

        // Accent rule under title
        let rule_y = title_y - 6.0;
        layer.set_outline_color(rgb(ar, ag, ab));
        layer.set_outline_thickness(1.5);
        layer.add_line(make_line(left, rule_y, left + 80.0, rule_y));

        // Subtitle
        if let Some(sub) = subtitle {
            let sub_size = self.branded.profile.font_size_h2;
            layer.set_fill_color(rgb(sr, sg, sb));
            layer.use_text(&sub, sub_size as f32, Mm(left), Mm(rule_y - 12.0), self.body_font);
        }

        // Metadata pillar at the bottom: divider rule + label/value rows
        let meta_size = self.branded.profile.font_size_base;
        let label_size = (meta_size - 1.5).max(7.0);
        let value_step = pt_to_mm(meta_size * 1.6);
        let label_step = pt_to_mm(label_size * 1.5);

        let mut rows: Vec<(&str, String)> = Vec::new();
        if let Some(v) = recipient { rows.push(("PREPARED FOR", v)); }
        if let Some(v) = author { rows.push(("BY", v)); }
        if let Some(v) = date { rows.push(("DATE", v)); }

        if !rows.is_empty() {
            let block_height: f32 = rows
                .iter()
                .map(|_| label_step + value_step + 4.0)
                .sum();
            let meta_top = bottom_band + 30.0 + block_height;

            layer.set_outline_color(rgb(br, bg, bb));
            layer.set_outline_thickness(0.4);
            layer.add_line(make_line(left, meta_top, left + 60.0, meta_top));

            let mut y = meta_top - 8.0;
            for (label, value) in &rows {
                layer.set_fill_color(rgb(sr, sg, sb));
                layer.use_text(*label, label_size as f32, Mm(left), Mm(y), self.body_font);
                y -= label_step;
                layer.set_fill_color(rgb(tr, tg, tb));
                layer.use_text(value, meta_size as f32, Mm(left), Mm(y), self.heading_font);
                y -= value_step + 2.0;
            }
        }

        self.cover_rendered = true;
        Ok(())
    }

    fn render_header(&mut self) -> anyhow::Result<()> {
        let layer = self.layer();
        let header_y = self.branded.page.height_mm - self.branded.page.margin_top_mm + 5.0;
        let left = self.branded.page.margin_left_mm;

        if let Some(t) = self.branded.doc.frontmatter.as_ref().and_then(|f| f.title.clone()) {
            let [r, g, b] = self.branded.colours.secondary;
            layer.set_fill_color(rgb(r, g, b));
            layer.use_text(&t, 8.0, Mm(left), Mm(header_y), self.body_font);
        }

        if self.branded.profile.header_border {
            let by = self.branded.page.height_mm - self.branded.page.margin_top_mm + 2.0;
            let [r, g, b] = self.branded.colours.table_border;
            layer.set_outline_color(rgb(r, g, b));
            layer.set_outline_thickness(0.3);
            layer.add_line(make_line(left, by, left + self.branded.page.content_width_mm, by));
        }
        Ok(())
    }

    fn apply_footers(&mut self) -> anyhow::Result<()> {
        let total = self.all_pages.len() as u32;
        let start_idx = if self.cover_rendered { 1usize } else { 0 };
        let left = self.branded.page.margin_left_mm;
        let footer_y = self.branded.page.margin_bottom_mm - 5.0;

        for (idx, &(page_idx, layer_idx)) in self.all_pages.iter().enumerate() {
            if idx < start_idx {
                continue;
            }

            let page = self.doc.get_page(page_idx);
            let layer = page.get_layer(layer_idx);
            let pn = if self.branded.profile.cover_enabled { idx as u32 } else { idx as u32 + 1 };

            if self.branded.profile.footer_border {
                let by = self.branded.page.margin_bottom_mm;
                let [r, g, b] = self.branded.colours.table_border;
                layer.set_outline_color(rgb(r, g, b));
                layer.set_outline_thickness(0.3);
                layer.add_line(make_line(left, by, left + self.branded.page.content_width_mm, by));
            }

            let [r, g, b] = self.branded.colours.secondary;
            layer.set_fill_color(rgb(r, g, b));
            let txt = format!("Page {pn} of {total}");
            let tw = txt.len() as f32 * 2.0;
            let x = left + self.branded.page.content_width_mm - tw;
            layer.use_text(&txt, 7.0, Mm(x), Mm(footer_y), self.body_font);
        }
        Ok(())
    }
}

fn pt_to_mm(pt: f32) -> f32 {
    pt / 2.834_645_7
}
