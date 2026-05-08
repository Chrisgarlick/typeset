use docx_rs::*;

use crate::brand::engine::BrandedDocument;
use crate::models::document::DocumentNode;

pub fn render(branded: &BrandedDocument) -> anyhow::Result<Vec<u8>> {
    let mut docx = Docx::new();

    // Page size in twips
    let (w, h) = match branded.profile.page_size.as_str() {
        "A4" => (11906u32, 16838u32),
        "Letter" => (12240u32, 15840u32),
        "Legal" => (12240u32, 20160u32),
        _ => (11906u32, 16838u32),
    };

    let mt = mm_to_twips(branded.profile.margin_top) as i32;
    let mb = mm_to_twips(branded.profile.margin_bottom) as i32;
    let ml = mm_to_twips(branded.profile.margin_left) as i32;
    let mr = mm_to_twips(branded.profile.margin_right) as i32;

    // Cover page
    if branded.profile.cover_enabled {
        docx = add_cover_page(docx, branded);
    }

    // Content nodes
    for node in &branded.doc.nodes {
        docx = render_node(docx, node, branded);
    }

    // Page size and margins
    docx = docx
        .page_size(w, h)
        .page_margin(
            PageMargin::new()
                .top(mt)
                .bottom(mb)
                .left(ml)
                .right(mr),
        );

    let mut buf = Vec::new();
    docx.build().pack(&mut std::io::Cursor::new(&mut buf))?;
    Ok(buf)
}

fn render_node(docx: Docx, node: &DocumentNode, branded: &BrandedDocument) -> Docx {
    match node {
        DocumentNode::Heading { level, text, .. } => {
            let (size, colour_hex) = match level {
                1 => ((branded.profile.font_size_h1 * 2.0) as usize, &branded.profile.colour_primary),
                2 => ((branded.profile.font_size_h2 * 2.0) as usize, &branded.profile.colour_secondary),
                3 => ((branded.profile.font_size_h3 * 2.0) as usize, &branded.profile.colour_text),
                _ => (((branded.profile.font_size_base + 1.0) * 2.0) as usize, &branded.profile.colour_text),
            };
            let colour = colour_hex.trim_start_matches('#');
            let run = Run::new().add_text(text).size(size).bold().color(colour);
            let spacing_before = match level {
                1 => 240u32,
                2 => 200,
                3 => 160,
                _ => 120,
            };
            let para = Paragraph::new()
                .add_run(run)
                .line_spacing(LineSpacing::new().before(spacing_before).after(80));
            docx.add_paragraph(para)
        }

        DocumentNode::Paragraph { text } => {
            let colour = branded.profile.colour_text.trim_start_matches('#');
            let run = Run::new()
                .add_text(text)
                .size((branded.profile.font_size_base * 2.0) as usize)
                .color(colour);
            let ps = mm_to_twips(branded.profile.paragraph_spacing);
            let line = (branded.profile.line_height * 240.0) as i32;
            let para = Paragraph::new()
                .add_run(run)
                .line_spacing(LineSpacing::new().after(ps).line(line));
            docx.add_paragraph(para)
        }

        DocumentNode::BulletList { items } => {
            let mut d = docx;
            let colour = branded.profile.colour_text.trim_start_matches('#');
            for item in items {
                let run = Run::new()
                    .add_text(item)
                    .size((branded.profile.font_size_base * 2.0) as usize)
                    .color(colour);
                let para = Paragraph::new()
                    .add_run(run)
                    .numbering(NumberingId::new(1), IndentLevel::new(0));
                d = d.add_paragraph(para);
            }
            d
        }

        DocumentNode::OrderedList { items, .. } => {
            let mut d = docx;
            let colour = branded.profile.colour_text.trim_start_matches('#');
            for item in items {
                let run = Run::new()
                    .add_text(item)
                    .size((branded.profile.font_size_base * 2.0) as usize)
                    .color(colour);
                let para = Paragraph::new()
                    .add_run(run)
                    .numbering(NumberingId::new(2), IndentLevel::new(0));
                d = d.add_paragraph(para);
            }
            d
        }

        DocumentNode::Table { headers, rows } => {
            let header_bg = branded.profile.colour_table_header.trim_start_matches('#').to_string();
            let mut table_rows = vec![];

            // Header row
            let mut header_cells = vec![];
            for h in headers {
                let run = Run::new()
                    .add_text(h)
                    .size((branded.profile.font_size_base * 2.0) as usize)
                    .bold();
                let para = Paragraph::new().add_run(run);
                let cell = TableCell::new()
                    .add_paragraph(para)
                    .shading(Shading::new().fill(&header_bg));
                header_cells.push(cell);
            }
            table_rows.push(TableRow::new(header_cells));

            // Data rows
            let colour = branded.profile.colour_text.trim_start_matches('#');
            for row in rows {
                let mut cells = vec![];
                for cell_text in row {
                    let run = Run::new()
                        .add_text(cell_text)
                        .size((branded.profile.font_size_base * 2.0) as usize)
                        .color(colour);
                    let para = Paragraph::new().add_run(run);
                    cells.push(TableCell::new().add_paragraph(para));
                }
                table_rows.push(TableRow::new(cells));
            }

            docx.add_table(Table::new(table_rows))
        }

        DocumentNode::CodeBlock { code, .. } => {
            let run = Run::new()
                .add_text(code)
                .size(((branded.profile.font_size_base - 1.5) * 2.0) as usize)
                .fonts(RunFonts::new().ascii("Courier New"));
            let para = Paragraph::new().add_run(run);
            docx.add_paragraph(para)
        }

        DocumentNode::Blockquote { text } => {
            let accent = branded.profile.colour_accent.trim_start_matches('#');
            let run = Run::new()
                .add_text(text)
                .size((branded.profile.font_size_base * 2.0) as usize)
                .italic()
                .color(accent);
            let para = Paragraph::new()
                .add_run(run)
                .indent(Some(720), None, None, None);
            docx.add_paragraph(para)
        }

        DocumentNode::HorizontalRule => {
            docx.add_paragraph(Paragraph::new().add_run(Run::new()))
        }

        DocumentNode::PageBreak => {
            let run = Run::new().add_break(BreakType::Page);
            docx.add_paragraph(Paragraph::new().add_run(run))
        }

        DocumentNode::Image { .. } => docx, // deferred to v1.1
    }
}

fn add_cover_page(mut docx: Docx, branded: &BrandedDocument) -> Docx {
    let title = branded.doc.frontmatter.as_ref()
        .and_then(|f| f.title.clone())
        .unwrap_or_else(|| "Untitled".to_string());

    // Space before title
    for _ in 0..8 {
        docx = docx.add_paragraph(Paragraph::new().add_run(Run::new()));
    }

    // Title
    let colour = branded.profile.colour_primary.trim_start_matches('#');
    let title_run = Run::new()
        .add_text(&title)
        .size((branded.profile.font_size_h1 * 3.0) as usize)
        .bold()
        .color(colour);
    docx = docx.add_paragraph(Paragraph::new().add_run(title_run));

    // Subtitle
    if let Some(sub) = branded.doc.frontmatter.as_ref().and_then(|f| f.subtitle.clone()) {
        let sub_colour = branded.profile.colour_secondary.trim_start_matches('#');
        let sub_run = Run::new()
            .add_text(&sub)
            .size((branded.profile.font_size_h2 * 2.0) as usize)
            .color(sub_colour);
        docx = docx.add_paragraph(Paragraph::new().add_run(sub_run));
    }

    // Recipient
    if let Some(recip) = branded.doc.frontmatter.as_ref().and_then(|f| f.recipient.clone()) {
        docx = docx.add_paragraph(Paragraph::new());
        let run = Run::new()
            .add_text(&format!("Prepared for: {recip}"))
            .size((branded.profile.font_size_base * 2.0) as usize);
        docx = docx.add_paragraph(Paragraph::new().add_run(run));
    }

    // Date
    if let Some(date) = branded.doc.frontmatter.as_ref().and_then(|f| f.date.clone()) {
        let run = Run::new()
            .add_text(&date)
            .size((branded.profile.font_size_base * 2.0) as usize);
        docx = docx.add_paragraph(Paragraph::new().add_run(run));
    }

    // Page break
    docx = docx.add_paragraph(Paragraph::new().add_run(Run::new().add_break(BreakType::Page)));
    docx
}

fn mm_to_twips(mm: f32) -> u32 {
    (mm * 56.693) as u32
}
