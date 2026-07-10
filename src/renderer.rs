use crate::templates;
use crate::transformer::PaperData;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn render_tex(paper: &PaperData, output: &Path, template: &str) -> Result<PathBuf> {
    let mut tex = String::new();

    // Preamble
    tex.push_str(&templates::get_preamble(
        template,
        &paper.title,
        &paper.author_line,
    ));

    // Journal metadata (deep disguise)
    tex.push_str(&format!(
        "\\renewcommand\\thepage{{\\arabic{{page}}}}\n\
         \\fancyhead[LE,LO]{{{journal}}}\n\
         \\fancyhead[RE,RO]{{Vol.~{vol}, No.~{iss}}}\n\n",
        journal = paper.journal,
        vol = paper.volume,
        iss = paper.issue,
    ));

    // Manuscript metadata
    tex.push_str(&format!(
        "\\noindent{{\\small\\itshape {}}}\n\n\
         \\noindent{{\\small DOI: {}}}\n\n\
         \\noindent{{\\small Received: {}; Accepted: {}}}\n\n",
        paper.journal, paper.doi, paper.received, paper.accepted,
    ));

    // Author list with ORCIDs and affiliations
    for author in &paper.authors {
        tex.push_str(&format!(
            "\\noindent{{{} {}$^{{\\dagger}}$\\ \\href{{https://orcid.org/{}}}{{\\scriptsize\\textcolor{{gray}}{{ORCID: {}}}}}$^{{}}$}}\n",
            author.given_name, author.family_name, author.orcid, author.orcid,
        ));
    }
    tex.push_str("\\vspace{0.3cm}\n\n");

    // Affiliations
    for (i, author) in paper.authors.iter().enumerate() {
        tex.push_str(&format!(
            "$^{{\\dagger}}${}. {}\\\\\n",
            i + 1,
            author.affiliation,
        ));
    }
    tex.push_str(&format!(
        "{} \\texttt{{{}}}\n\n",
        if paper.authors.len() > 1 {
            "Correspondence: "
        } else {
            ""
        },
        paper.authors[0].email,
    ));

    // Keywords
    tex.push_str("\\noindent{\\textbf{Keywords:}} ");
    tex.push_str(&paper.keywords.join("; "));
    tex.push_str("\n\n");

    // Abstract
    tex.push_str("\\begin{abstract}\n");
    tex.push_str("\\textbf{Background:} ");
    tex.push_str(&paper.abstract_sections.background);
    tex.push_str("\\par\n\\textbf{Methods:} ");
    tex.push_str(&paper.abstract_sections.methods);
    tex.push_str("\\par\n\\textbf{Results:} ");
    tex.push_str(&paper.abstract_sections.results);
    tex.push_str("\\par\n\\textbf{Conclusions:} ");
    tex.push_str(&paper.abstract_sections.conclusions);
    tex.push_str("\n\\end{abstract}\n\n");

    // Sections
    for section in &paper.sections {
        tex.push_str(&format!("\\section{{{}}}\n", section.heading));
        for para in &section.paragraphs {
            tex.push_str(&format!("{}\n\n", para));
        }
        for bq in &section.blockquotes {
            tex.push_str(&format!("\\begin{{quote}}\n{}\\end{{quote}}\n\n", bq));
        }
    }

    // Acknowledgements
    tex.push_str("\\section*{Acknowledgements}\n");
    tex.push_str("The authors thank the participants of the Workshop on Computational Approaches to Literary Analysis for their valuable feedback. This research was partially supported by a grant from the Digital Humanities Research Council.\n\n");

    // Data availability
    tex.push_str("\\section*{Data Availability}\n");
    tex.push_str("The corpus used in this study is available from the authors upon reasonable request. Analysis scripts are hosted on GitHub (https://github.com/papercoat/papercoat).\n\n");

    // Competing interests
    tex.push_str("\\section*{Competing Interests}\n");
    tex.push_str("The authors declare no competing interests.\n\n");

    // References
    tex.push_str("\\begin{thebibliography}{99}\n");
    for (i, ref_text) in paper.references.iter().enumerate() {
        tex.push_str(&format!("\\bibitem{{ref{}}} {}\n", i + 1, ref_text));
    }
    tex.push_str("\\end{thebibliography}\n");

    // Postamble
    tex.push_str(templates::get_postamble());
    tex.push('\n');

    std::fs::write(output, tex)?;
    Ok(output.to_path_buf())
}

pub fn compile_latex(tex_path: &Path) -> Result<Option<PathBuf>> {
    let pdflatex = if cfg!(target_os = "windows") {
        "pdflatex.exe"
    } else {
        "pdflatex"
    };

    if !has_command(pdflatex) {
        return Ok(None);
    }

    let dir = tex_path.parent().unwrap_or(Path::new("."));
    let stem = tex_path.file_stem().unwrap().to_string_lossy();
    let pdf_path = dir.join(format!("{}.pdf", stem));

    for pass in 0..2 {
        let output = Command::new(pdflatex)
            .arg("-interaction=nonstopmode")
            .arg(format!("-output-directory={}", dir.display()))
            .arg(tex_path)
            .output()
            .map_err(|e| anyhow::anyhow!("pdflatex failed: {}", e))?;

        if !output.status.success() && pass == 1 {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("LaTeX compilation failed: {}", stderr);
        }
    }

    if pdf_path.exists() {
        // Inject metadata
        if let Err(e) = inject_metadata(&pdf_path, tex_path) {
            eprintln!("Warning: metadata injection failed: {}", e);
        }
        Ok(Some(pdf_path))
    } else {
        Ok(None)
    }
}

fn has_command(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn inject_metadata(pdf_path: &Path, tex_path: &Path) -> Result<()> {
    use lopdf::{Document, Object};

    let mut doc = Document::load(pdf_path)?;

    let title = tex_path.file_stem().unwrap().to_string_lossy().to_string();
    let info = lopdf::Dictionary::from_iter([
        (b"Title".to_vec(), Object::string_literal(&*title)),
        (b"Author".to_vec(), Object::string_literal("PaperCoat")),
        (
            b"Subject".to_vec(),
            Object::string_literal("Generated by PaperCoat - Computational Literary Analysis"),
        ),
    ]);

    doc.trailer.set(b"Info", Object::Dictionary(info));

    doc.save(pdf_path)?;
    Ok(())
}
