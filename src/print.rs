use genpdf::{Document, Element, Alignment, Size};
use genpdf::elements::{Paragraph, Break, PaddedElement, TableLayout, StyledElement};
use genpdf::fonts::{FontData, FontFamily};
use genpdf::style::{self, Style};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::db::Db;

static FONT_CACHE: OnceLock<FontFamily<FontData>> = OnceLock::new();

fn get_fonts() -> Result<FontFamily<FontData>, Box<dyn std::error::Error>> {
    if let Some(f) = FONT_CACHE.get() {
        return Ok(f.clone());
    }
    let fonts = load_font_family()?;
    let _ = FONT_CACHE.set(fonts.clone());
    Ok(fonts)
}

fn load_font_family() -> Result<FontFamily<FontData>, Box<dyn std::error::Error>> {
    let font_dirs: &[&str] = &[
        "/usr/share/fonts/TTF",
        "/usr/share/fonts/truetype/dejavu",
        "/usr/share/fonts/dejavu",
        "/usr/share/fonts",
    ];

    let mut regular_path = None;
    let mut bold_path = None;

    for dir in font_dirs {
        let r = PathBuf::from(dir).join("DejaVuSans.ttf");
        let b = PathBuf::from(dir).join("DejaVuSans-Bold.ttf");
        if r.exists() && b.exists() {
            regular_path = Some(r);
            bold_path = Some(b);
            break;
        }
    }

    let regular_path = regular_path.ok_or("DejaVuSans.ttf not found in system font directories")?;
    let bold_path = bold_path.ok_or("DejaVuSans-Bold.ttf not found")?;

    let regular = FontData::load(&regular_path, None)?;
    let bold = FontData::load(&bold_path, None)?;

    Ok(FontFamily {
        regular: regular.clone(),
        bold: bold.clone(),
        italic: regular,
        bold_italic: bold,
    })
}

fn build_doc(db: &Db, ukrainian: bool, cp_id: i64) -> Result<Document, Box<dyn std::error::Error>> {
    let font_family = get_fonts()?;
    let mut doc = Document::new(font_family);

    let title = if ukrainian { "Тренувальний звіт" } else { "Training Report" };
    doc.set_title(title);
    doc.set_minimal_conformance();
    doc.set_paper_size(Size::new(210, 297)); // A4
    doc.set_font_size(10);

    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(15);
    doc.set_page_decorator(decorator);

    let bold = Style::from(style::Effect::Bold);
    let small = Style::new().with_font_size(9);
    let small_bold = Style::from(style::Effect::Bold).with_font_size(9);
    let title_style = Style::from(style::Effect::Bold).with_font_size(16);

    // Title
    doc.push(Paragraph::new(title).aligned(Alignment::Center).styled(title_style));
    let date_str = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    doc.push(Paragraph::new(&date_str).aligned(Alignment::Center));
    doc.push(Break::new(1.5));

    let all_programs = db.get_active_client_programs().unwrap_or_default();
    let programs: Vec<_> = all_programs
        .into_iter()
        .filter(|(cp, _, _, _)| cp.id == cp_id)
        .collect();

    if programs.is_empty() {
        let msg = if ukrainian { "Немає активних програм." } else { "No active programs." };
        doc.push(Paragraph::new(msg));
        return Ok(doc);
    }

    let t = Tr::new(ukrainian);

    for (idx, (cp, client_name, program_name, trainer_name)) in programs.iter().enumerate() {
        // Client + Program header
        doc.push(Paragraph::new(format!(
            "{}. {} — {}", idx + 1, client_name, program_name
        )).styled(bold));

        doc.push(Paragraph::new(format!(
            "{}: {}   |   {}: {}",
            t.trainer, trainer_name, t.start, cp.start_date
        )).styled(small));

        doc.push(Break::new(0.5));

        // Check-ins as simple text lines
        let checkins = db.get_checkins_for_client_program(cp.id).unwrap_or_default();
        let n_weeks = checkins.len().min(10);

        if !checkins.is_empty() {
            doc.push(Paragraph::new(t.checkins).styled(small_bold));

            for (i, ci) in checkins.iter().enumerate() {
                let mark = if ci.completed { "☑" } else { "☐" };
                let line = format!("    {} {}:  {}   {}", t.week, i + 1, ci.date, mark);
                doc.push(Paragraph::new(&line).styled(small));
            }

            doc.push(Break::new(0.3));
        }

        // Exercises table
        let exercises = db.get_exercise_completions(cp.id).unwrap_or_default();
        let week_done: HashMap<(i64, i64), bool> = db
            .get_exercise_week_completions(cp.id)
            .unwrap_or_default()
            .into_iter()
            .map(|(eid, cid, done)| ((eid, cid), done))
            .collect();
        if !exercises.is_empty() {
            doc.push(Paragraph::new(t.exercises).styled(small_bold));

            let has_weight = exercises.iter().any(|e| e.weight > 0);

            // name(5) + sets(1) + reps(1) + [weight(1)] + N week cols(1 each) + notes(3)
            let mut col_widths: Vec<usize> = vec![5, 1, 1];
            if has_weight { col_widths.push(1); }
            if n_weeks > 0 {
                for _ in 0..n_weeks { col_widths.push(1); }
            } else {
                col_widths.push(2); // fallback: single done column
            }
            col_widths.push(3);

            let mut table = TableLayout::new(col_widths);
            table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));

            // Header row
            let mut hr = table.row();
            hr.push_element(cell(t.exercise, small_bold));
            hr.push_element(cell_center(t.sets, small_bold));
            hr.push_element(cell_center(t.reps, small_bold));
            if has_weight { hr.push_element(cell_center(t.weight, small_bold)); }
            if n_weeks > 0 {
                for i in 0..n_weeks {
                    let label = format!("{}{}", t.week_short, i + 1);
                    hr.push_element(cell_center(&label, small_bold));
                }
            } else {
                hr.push_element(cell_center(t.done, small_bold));
            }
            hr.push_element(cell(t.notes, small_bold));
            hr.push().unwrap();

            // Exercise rows — one checkbox per week based on check-in completion
            for ex in &exercises {
                let mut row = table.row();
                row.push_element(cell(&ex.name, small));
                row.push_element(cell_center(&ex.sets.to_string(), small));
                row.push_element(cell_center(&ex.reps.to_string(), small));
                if has_weight {
                    let w = if ex.weight > 0 { format!("{}kg", ex.weight) } else { String::new() };
                    row.push_element(cell_center(&w, small));
                }
                if n_weeks > 0 {
                    for ci in &checkins[..n_weeks] {
                        let done = week_done.get(&(ex.exercise_id, ci.id)).copied().unwrap_or(false);
                        let mark = if done { "☑" } else { "☐" };
                        row.push_element(cell_center(mark, small));
                    }
                } else {
                    let mark = if ex.completed { "☑" } else { "☐" };
                    row.push_element(cell_center(mark, small));
                }
                row.push_element(cell(&ex.notes, small));
                row.push().unwrap();
            }

            doc.push(PaddedElement::new(table, genpdf::Margins::trbl(1, 0, 1, 4)));
        }

        // Separator between programs
        if idx < programs.len() - 1 {
            doc.push(Break::new(0.5));
            doc.push(Paragraph::new("────────────────────────────────────────").styled(small));
            doc.push(Break::new(0.8));
        }
    }

    Ok(doc)
}

fn cell(text: &str, style: Style) -> PaddedElement<StyledElement<Paragraph>> {
    PaddedElement::new(
        Paragraph::new(text).aligned(Alignment::Left).styled(style),
        genpdf::Margins::trbl(1, 2, 1, 2),
    )
}

fn cell_center(text: &str, style: Style) -> PaddedElement<StyledElement<Paragraph>> {
    PaddedElement::new(
        Paragraph::new(text).aligned(Alignment::Center).styled(style),
        genpdf::Margins::trbl(1, 2, 1, 2),
    )
}

/// Translated labels
struct Tr {
    trainer: &'static str,
    start: &'static str,
    checkins: &'static str,
    exercises: &'static str,
    week: &'static str,
    week_short: &'static str,
    exercise: &'static str,
    sets: &'static str,
    reps: &'static str,
    weight: &'static str,
    notes: &'static str,
    done: &'static str,
}

impl Tr {
    fn new(ua: bool) -> Self {
        if ua {
            Tr {
                trainer: "Тренер",
                start: "Початок",
                checkins: "Відвідування:",
                exercises: "Вправи:",
                week: "Тиж.",
                week_short: "Т",
                exercise: "Вправа",
                sets: "Підх.",
                reps: "Повт.",
                weight: "Вага",
                notes: "Нотатки",
                done: "✓",
            }
        } else {
            Tr {
                trainer: "Trainer",
                start: "Start",
                checkins: "Check-ins:",
                exercises: "Exercises:",
                week: "Wk",
                week_short: "W",
                exercise: "Exercise",
                sets: "Sets",
                reps: "Reps",
                weight: "Weight",
                notes: "Notes",
                done: "✓",
            }
        }
    }
}

pub fn save_pdf(db: &Db, ukrainian: bool, cp_id: i64) -> Result<(), Box<dyn std::error::Error>> {
    // Show dialog FIRST so UI responds immediately
    let default_name = format!(
        "training_report_{}.pdf",
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    );

    let path = rfd::FileDialog::new()
        .set_title(if ukrainian { "Зберегти PDF" } else { "Save PDF" })
        .set_file_name(&default_name)
        .add_filter("PDF", &["pdf"])
        .save_file();

    if let Some(path) = path {
        // Build and write only after user confirmed the path
        let doc = build_doc(db, ukrainian, cp_id)?;
        doc.render_to_file(&path)?;
    }

    Ok(())
}

pub fn print_pdf(db: &Db, ukrainian: bool, cp_id: i64) -> Result<(), Box<dyn std::error::Error>> {
    let doc = build_doc(db, ukrainian, cp_id)?;

    let tmp = std::env::temp_dir().join("slint_crm_print.pdf");
    doc.render_to_file(&tmp)?;

    let path = tmp.to_str().ok_or("invalid temp path")?.to_owned();

    // Use XDG Desktop Portal (org.freedesktop.portal.Print)
    // Shows the native system print dialog on GNOME and KDE
    let rt = tokio::runtime::Runtime::new()?;
    if let Err(e) = rt.block_on(portal_print(&path)) {
        eprintln!("Print portal failed ({e}), falling back to xdg-open");
        std::process::Command::new("xdg-open").arg(&path).spawn()?;
    }

    Ok(())
}

async fn portal_print(path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use ashpd::desktop::print::{PageSetup, PreparePrintOptions, PrintOptions, PrintProxy, Settings};

    let proxy = PrintProxy::new().await?;

    // Step 1: show the print dialog, get back chosen settings + token
    let response = proxy
        .prepare_print(
            None,
            "Training Report",
            Settings::default(),
            PageSetup::default(),
            PreparePrintOptions::default(),
        )
        .await?
        .response()?;

    // Step 2: send the file to CUPS using the token from the dialog
    let file = std::fs::File::open(path)?;
    proxy
        .print(
            None,
            "Training Report",
            &file,
            PrintOptions::default().set_token(response.token),
        )
        .await?;

    Ok(())
}
