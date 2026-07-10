use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PaperData {
    pub title: String,
    pub author_line: String,
    pub authors: Vec<Author>,
    pub abstract_sections: AbstractSections,
    pub sections: Vec<Section>,
    pub references: Vec<String>,
    pub word_count: usize,
    pub page_count: usize,
    pub journal: String,
    pub doi: String,
    pub volume: String,
    pub issue: String,
    pub received: String,
    pub accepted: String,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Author {
    pub given_name: String,
    pub family_name: String,
    pub orcid: String,
    pub affiliation: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AbstractSections {
    pub background: String,
    pub methods: String,
    pub results: String,
    pub conclusions: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Section {
    pub heading: String,
    pub paragraphs: Vec<String>,
    pub blockquotes: Vec<String>,
}

static AUTHORS: &[(&str, &str, &str)] = &[
    ("A.", "Marlowe", "a.marlowe@literary-analytics.org"),
    ("J.", "Whitfield", "j.whitfield@cognitextlab.edu"),
    ("R.", "Nakamura", "r.nakamura@digitalhumanities.jp"),
    ("S.", "Okafor", "s.okafor@lingua-analytics.ng"),
    ("E.", "Vasquez", "e.vasquez@narratology-institute.org"),
    ("L.", "Chen", "l.chen@comp-lit-systems.cn"),
    ("K.", "Petrov", "k.petrov@textmining.ru"),
    ("M.", "O'Brien", "m.obrien@corpus-studies.ie"),
];

static AFFILIATIONS: &[&str] = &[
    "Institute for Computational Literary Analysis",
    "Centre for Digital Humanities and Narrative Studies",
    "Laboratory for Quantitative Text Analysis",
    "Department of Computational Philology",
    "Research Group for Digital Narratology",
];

static JOURNALS: &[&str] = &[
    "Journal of Computational Literary Studies",
    "Digital Scholarship in the Humanities",
    "Literary and Linguistic Computing",
    "Journal of Cultural Analytics",
    "Poetics: Journal of Empirical Research",
];

fn pick<'a, T: ?Sized>(items: &'a [&'a T], seed: usize) -> &'a T {
    items[seed % items.len()]
}

fn generate_orcid(seed: usize) -> String {
    format!(
        "{:04}-{:04}-{:04}-{:04}",
        (seed * 1234) % 10000,
        (seed * 5678) % 10000,
        (seed * 9012) % 10000,
        (seed * 3456) % 10000,
    )
}

fn generate_title(text: &str, seed: usize) -> String {
    let topics = [
        "Narrative Structure",
        "Discourse Analysis",
        "Stylistic Variation",
        "Narrative Progression",
        "Textual Coherence",
        "Corpus Stylistics",
        "Diegetic Frameworks",
        "Narrative Modalities",
    ];
    let focus = [
        "Contemporary Literary Prose",
        "Modern Narrative Fiction",
        "Twenty-First-Century Literature",
        "Experimental Narrative Forms",
    ];

    let t = topics[seed % topics.len()];
    let f = focus[(seed + 2) % focus.len()];

    let word_sample: String = text
        .split_whitespace()
        .skip(10)
        .take(5)
        .collect::<Vec<_>>()
        .join(" ");

    format!("\"{}\": {} of {}", word_sample, t, f)
    //    format!("{t}: {m} for {f}")
}

fn generate_authors(seed: usize) -> Vec<Author> {
    let count = 2 + (seed % 4);
    (0..count)
        .map(|i| {
            let idx = (seed + i) % AUTHORS.len();
            let (given, family, email) = AUTHORS[idx];
            Author {
                given_name: given.to_string(),
                family_name: family.to_string(),
                orcid: generate_orcid(seed + i),
                affiliation: pick(AFFILIATIONS, seed + i).to_string(),
                email: email.to_string(),
            }
        })
        .collect()
}

fn generate_abstract(text: &str, _word_count: usize) -> AbstractSections {
    let paras: Vec<&str> = text.split("\n\n").filter(|p| !p.trim().is_empty()).collect();
    let first = paras.first().unwrap_or(&text);
    let sentences: Vec<&str> = first
        .split(|c: char| c == '.' || c == '!' || c == '?')
        .filter(|s| !s.trim().is_empty())
        .collect();
    let sample = sentences.first().unwrap_or(first).trim();

    AbstractSections {
        background: format!(
            "The computational analysis of literary texts has emerged as a significant \
             area of research within the digital humanities, offering new methods for \
             understanding narrative structure, stylistic variation, and thematic \
             progression at scale. Despite these advances, the application of such \
             techniques to contemporary fiction remains comparatively underexplored."
        ),
        methods: format!(
            "This study employs a mixed-methods framework combining corpus linguistic \
             techniques with computational text analysis. The corpus comprises {n} \
             textual segments drawn from contemporary narrative prose, analyzed for \
             lexical distribution and syntactic complexity using automated parsing tools.",
            n = paras.len()
        ),
        results: format!(
            "Analysis revealed systematic patterns in the distribution of key linguistic \
             features across the narrative. {sample} These patterns were consistent \
             with established theories of narrative discourse while also revealing novel \
             structural characteristics not previously documented in the literature.",
            sample = {
                let s = sample.chars().take(150).collect::<String>();
                s
            }
        ),
        conclusions: format!(
            "Our findings demonstrate the efficacy of computational approaches for \
             literary analysis and identify new directions for research at the intersection \
             of quantitative methods and narrative theory. The methodology developed here \
             is broadly applicable to a wide range of literary texts."
        ),
    }
}

fn generate_sections(text: &str, word_count: usize, seed: usize) -> Vec<Section> {
    let raw_paras: Vec<&str> = text
        .split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .collect();

    let section_headings = [
        "Introduction",
        "Related Work",
        "Corpus and Methodology",
        "Results and Analysis",
        "Discussion",
        "Conclusion",
    ];

    let citation_authors = ["Marlowe et al.", "Whitfield & Nakamura", "Chen & Petrov", "Okafor et al.", "Vasquez & O'Brien"];
    let citation_years = ["2019", "2021", "2022", "2023", "2024"];
    let _citation_journals = pick(JOURNALS, seed + 3);
    let citations: Vec<String> = (0..6)
        .map(|i| {
            let ca = citation_authors[(seed + i) % citation_authors.len()];
            let cy = citation_years[(seed + i) % citation_years.len()];
            format!("{ca} ({cy})")
        })
        .collect();

    let intro_sample: String = raw_paras
        .first()
        .map(|p| {
            let s: String = p.chars().take(300).collect();
            s
        })
        .unwrap_or_default();

    let blockquote_candidates: Vec<&str> = raw_paras.iter().filter(|p| p.len() > 100).copied().collect();

    let mut sections = Vec::new();

    // Introduction
    {
        let paras = vec![
            format!(
                "Over the past decade, computational approaches to literary analysis have \
                 gained significant traction within the digital humanities. Scholars have \
                 employed a range of quantitative methods to examine stylistic variation, \
                 narrative structure, and thematic development across diverse corpora \
                 {}.\n\n\
                 The present study contributes to this growing body of work by applying \
                 computational text analysis to a previously unexamined work of contemporary \
                 fiction. Our approach integrates corpus linguistic techniques with \
                 established frameworks from narratology.",
                citations[0]
            ),
            format!(
                "As {} have demonstrated, the application of computational methods to \
                 literary texts can reveal patterns that are not readily apparent through \
                 traditional close reading alone. Building on these foundations, we seek \
                 to extend the methodological repertoire available to literary scholars.",
                citations[1]
            ),
        ];
        let mut blockquotes = Vec::new();
        if !intro_sample.is_empty() {
            blockquotes.push(format!(
                "The narrative opens: '{}'",
                intro_sample
            ));
        }

        sections.push(Section {
            heading: section_headings[0].into(),
            paragraphs: paras,
            blockquotes,
        });
    }

    // Related Work
    {
        let paras = vec![
            format!(
                "Previous research in computational literary studies has broadly followed \
                 two methodological trajectories. The first, exemplified by the work of \
                 {}, focuses on large-scale corpus analyses that identify stylistic \
                 patterns across extensive collections of texts. The second approach, \
                 represented by {}, employs close computational reading techniques \
                 that combine algorithmic analysis with interpretative frameworks drawn \
                 from literary theory.",
                citations[2], citations[3]
            ),
            format!(
                "More recently, {} have proposed integrated frameworks that synthesis \
                 these approaches, arguing for a 'scalable reading' methodology that \
                 moves beyond the traditional opposition between quantitative and \
                 qualitative methods.",
                citations[4]
            ),
        ];
        sections.push(Section {
            heading: section_headings[1].into(),
            paragraphs: paras,
            blockquotes: vec![],
        });
    }

    // Corpus and Methodology
    {
        let n_sentences = word_count / 15;
        let paras = vec![
            format!(
                "The corpus for this study consists of a single sustained work of \
                 contemporary narrative prose, comprising approximately {wc} words \
                 across {n} sentences. The text was digitized and processed using \
                 automated text extraction tools, then segmented into paragraphs and \
                 sentences using a rule-based sentence boundary detector.\n\n\
                 Each segment was analyzed using a pipeline that included part-of-speech \
                 tagging, dependency parsing, and lexical frequency analysis. Statistical \
                 analyses were performed using custom scripts.",
                wc = word_count,
                n = n_sentences
            ),
        ];
        sections.push(Section {
            heading: section_headings[2].into(),
            paragraphs: paras,
            blockquotes: vec![],
        });
    }

    // Results
    {
        let mut blockquotes = Vec::new();
        for c in blockquote_candidates.iter().take(2) {
            let s: String = c.chars().take(200).collect();
            blockquotes.push(format!(
                "A representative passage illustrates the characteristic stylistic \
                 features identified: '{}'",
                s
            ));
        }

        let adj = ["attributive", "qualitative", "evaluative"][seed % 3];
        let paras = vec![
            format!(
                "The analysis revealed several notable patterns in the linguistic \
                 structure of the text. Lexical frequency analysis indicated a marked \
                 preference for {adj}-type modifiers, consistent with the narrative's \
                 stylistic register. Sentence length variability was found to correlate \
                 with shifts in narrative pacing, a finding that aligns with the \
                 observations of {cite} regarding the relationship between syntax and \
                 narrative rhythm.",
                adj = adj,
                cite = citations[5],
            ),
        ];
        sections.push(Section {
            heading: section_headings[3].into(),
            paragraphs: paras,
            blockquotes,
        });
    }

    // Discussion
    {
        let paras = vec![
            format!(
                "The patterns identified through our computational analysis offer new \
                 insights into the stylistic and structural features of contemporary \
                 narrative prose. The correlation between syntactic complexity and \
                 narrative pacing, in particular, merits further investigation across \
                 a larger corpus of texts.\n\n\
                 Our findings also raise important methodological questions about the \
                 relationship between computational analysis and literary interpretation. \
                 While quantitative methods can identify patterns at scale, the \
                 interpretative significance of these patterns ultimately depends on \
                 close engagement with the text."
            ),
        ];
        sections.push(Section {
            heading: section_headings[4].into(),
            paragraphs: paras,
            blockquotes: vec![],
        });
    }

    // Conclusion
    {
        let paras = vec![
            format!(
                "This study has demonstrated the value of computational approaches for \
                 the analysis of contemporary literary fiction. By combining corpus \
                 linguistic techniques with narratological frameworks, we have identified \
                 stylistic and structural patterns that contribute to our understanding \
                 of narrative discourse.\n\n\
                 Future work should extend this analysis to a larger corpus and explore \
                 the application of more advanced natural language processing techniques, \
                 including transformer-based language models, to the analysis of literary \
                 style."
            ),
        ];
        sections.push(Section {
            heading: section_headings[5].into(),
            paragraphs: paras,
            blockquotes: vec![],
        });
    }

    sections
}

fn generate_references(_text: &str, seed: usize) -> Vec<String> {
    let journals = JOURNALS;

    vec![
        format!(
            "Marlowe, A. ({y}). Quantitative approaches to narrative structure. \
             {j}, 45(3), 234–256.",
            y = 2019 + (seed % 5),
            j = journals[0]
        ),
        format!(
            "Whitfield, J., & Nakamura, R. ({y}). Corpus stylistics and the analysis \
             of contemporary fiction. {j}, 12(2), 89–112.",
            y = 2020 + (seed % 4),
            j = journals[1]
        ),
        format!(
            "Chen, L., & Petrov, K. ({y}). Computational narratology: Methods and \
             applications. {j}, 8(4), 345–378.",
            y = 2021 + (seed % 3),
            j = journals[2]
        ),
        format!(
            "Okafor, S., Vasquez, E., & O'Brien, M. ({y}). Scalable reading: \
             Integrating computational and interpretative approaches to literature. \
             {j}, 33(1), 67–89.",
            y = 2022 + (seed % 2),
            j = journals[3]
        ),
        format!(
            "Marlowe, A., & Chen, L. ({y}). Lexical distribution and narrative \
             discourse: A corpus-based study. {j}, 29(4), 412–435.",
            y = 2023 + (seed % 1),
            j = journals[4]
        ),
        format!(
            "Nakamura, R. ({y}). Syntactic complexity and narrative rhythm in \
             contemporary prose. {j}, 15(2), 178–201.",
            y = 2020 + (seed % 4),
            j = journals[0]
        ),
        format!(
            "Vasquez, E., & Okafor, S. ({y}). Digital humanities and the future \
             of literary criticism. {j}, 41(1), 23–45.",
            y = 2021 + (seed % 3),
            j = journals[1]
        ),
    ]
}

pub fn transform_text(text: &str, title_override: Option<String>, author_override: Option<String>, page_count: usize) -> PaperData {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.len().hash(&mut hasher);
    text.hash(&mut hasher);
    let seed = hasher.finish() as usize;

    let word_count = text.split_whitespace().count();
    let title = title_override.unwrap_or_else(|| generate_title(text, seed));
    let authors = generate_authors(seed);

    let author_line = if let Some(override_name) = author_override {
        if authors.len() > 1 {
            format!("{}, et al.", override_name)
        } else {
            override_name
        }
    } else {
        let names: Vec<String> = authors
            .iter()
            .map(|a| format!("{} {}", a.given_name, a.family_name))
            .collect();
        if names.len() > 3 {
            format!("{}, et al.", names[0])
        } else {
            names.join(", ")
        }
    };

    PaperData {
        title,
        author_line,
        authors,
        abstract_sections: generate_abstract(text, word_count),
        sections: generate_sections(text, word_count, seed),
        references: generate_references(text, seed),
        journal: generate_journal(seed),
        doi: generate_doi(seed),
        volume: format!("{}", 1 + (seed % 45)),
        issue: format!("{}", 1 + (seed % 12)),
        received: generate_date(seed, -180, -60),
        accepted: generate_date(seed + 1, -60, -10),
        keywords: generate_keywords(text, seed),
        word_count,
        page_count,
    }
}

fn generate_journal(seed: usize) -> String {
    let journals = [
        "Journal of Computational Literary Studies",
        "Digital Scholarship in the Humanities",
        "Literary and Linguistic Computing",
        "Journal of Cultural Analytics",
        "Poetics: Journal of Empirical Research",
        "Computational Linguistics",
        "Narrative Inquiry",
        "Text and Talk",
    ];
    journals[seed % journals.len()].to_string()
}

fn generate_doi(seed: usize) -> String {
    format!(
        "10.10{}/jcls.{:04}.{:04}",
        2 + (seed % 8),
        (seed * 1234) % 10000,
        (seed * 5678) % 10000,
    )
}

fn generate_date(seed: usize, min_offset: i64, max_offset: i64) -> String {
    // Generate a date within the offset range (in days from today)
    let offset = min_offset + (seed as i64 % (max_offset - min_offset + 1));
    let months = [
        "January", "February", "March", "April", "May", "June",
        "July", "August", "September", "October", "November", "December",
    ];
    let month = months[(seed + offset as usize) % 12];
    let day = 1 + ((seed * 7 + offset as usize * 13) % 28);
    let year = 2024 + ((seed + offset as usize) % 3);
    format!("{} {}, {}", month, day, year)
}

fn generate_keywords(text: &str, seed: usize) -> Vec<String> {
    let pools = [
        ["computational literary analysis", "narrative theory", "corpus stylistics", "quantitative methods"],
        ["digital humanities", "text mining", "stylistic variation", "discourse analysis"],
        ["computational narratology", "literary computing", "distant reading", "stylometry"],
    ];
    let pool = pools[seed % pools.len()];
    pool.iter().take(4 + (seed % 2)).map(|s| s.to_string()).collect()
}

pub fn extract_blockquotes(text: &str, count: usize) -> Vec<String> {
    let paras: Vec<&str> = text
        .split("\n\n")
        .filter(|p| p.len() > 80)
        .collect();
    paras
        .iter()
        .take(count)
        .map(|p| {
            let s: String = p.chars().take(200).collect();
            format!("'{}'", s.trim())
        })
        .collect()
}

pub fn compute_readability_stats(text: &str) -> Vec<(String, String)> {
    let word_count = text.split_whitespace().count();
    let char_count = text.chars().count();
    let sentences: Vec<&str> = text
        .split(|c: char| c == '.' || c == '!' || c == '?')
        .filter(|s| !s.trim().is_empty())
        .collect();
    let sentence_count = sentences.len().max(1);
    let avg_words_per_sentence = word_count as f64 / sentence_count as f64;
    let avg_chars_per_word = char_count as f64 / word_count.max(1) as f64;

    let flesch = 206.835 - 1.015 * avg_words_per_sentence - 84.6 * avg_chars_per_word;

    fn comma_sep(n: usize) -> String {
        let s = n.to_string();
        let mut result = String::new();
        for (i, c) in s.chars().rev().enumerate() {
            if i > 0 && i % 3 == 0 {
                result.insert(0, ',');
            }
            result.insert(0, c);
        }
        result
    }

    vec![
        ("Words".into(), comma_sep(word_count)),
        ("Characters".into(), comma_sep(char_count)),
        ("Sentences".into(), comma_sep(sentence_count)),
        (
            "Avg words/sentence".into(),
            format!("{:.1}", avg_words_per_sentence),
        ),
        (
            "Avg chars/word".into(),
            format!("{:.1}", avg_chars_per_word),
        ),
        (
            "Flesch Readability".into(),
            format!("{:.1}", flesch),
        ),
    ]
}
