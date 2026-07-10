pub struct Template {
    pub name: &'static str,
    pub description: &'static str,
}

pub fn list() -> Vec<Template> {
    vec![
        Template {
            name: "default",
            description: "Two-column, Times Roman",
        },
        Template {
            name: "nature",
            description: "Single-column Nature style",
        },
        Template {
            name: "ieee",
            description: "IEEE Transactions style",
        },
    ]
}

pub fn validate(name: &str) -> bool {
    matches!(name, "default" | "nature" | "ieee")
}

pub fn get_preamble(template: &str, title: &str, author: &str) -> String {
    match template {
        "nature" => nature_preamble(title, author),
        "ieee" => ieee_preamble(title, author),
        _ => default_preamble(title, author),
    }
}

fn default_preamble(title: &str, author: &str) -> String {
    format!(
        r#"\documentclass[twocolumn,11pt]{{article}}
\usepackage{{geometry}}
\geometry{{margin=1in}}
\usepackage{{setspace}}
\usepackage{{fancyhdr}}
\usepackage{{hyperref}}
\usepackage{{titlesec}}
\usepackage{{caption}}
\usepackage{{natbib}}
\usepackage{{microtype}}

\pagestyle{{fancy}}
\fancyhead[LE,LO]{{{title}}}
\fancyhead[RE,RO]{{{author}}}
\fancyfoot[C]{{\thepage}}
\renewcommand{{\headrulewidth}}{{0.4pt}}

\title{{{title}}}
\author{{{author}}}
\date{{}}

\begin{{document}}
\maketitle
"#,
        title = title.replace("&", "\\&"),
        author = author.replace("&", "\\&"),
    )
}

fn nature_preamble(title: &str, author: &str) -> String {
    format!(
        r#"\documentclass[11pt]{{article}}
\usepackage{{geometry}}
\geometry{{margin=1in}}
\usepackage{{setspace}}
\onehalfspacing
\usepackage{{fancyhdr}}
\usepackage{{hyperref}}
\usepackage{{titlesec}}
\usepackage{{caption}}
\usepackage{{natbib}}
\usepackage{{microtype}}

\pagestyle{{fancy}}
\fancyhead[LE,LO]{{{title}}}
\fancyhead[RE,RO]{{{author}}}
\fancyfoot[C]{{\thepage}}
\renewcommand{{\headrulewidth}}{{0pt}}

\title{{{title}}}
\author{{{author}}}
\date{{}}

\begin{{document}}
\maketitle
"#,
        title = title.replace("&", "\\&"),
        author = author.replace("&", "\\&"),
    )
}

fn ieee_preamble(title: &str, author: &str) -> String {
    format!(
        r#"\documentclass[10pt,conference]{{IEEEtran}}
\usepackage{{hyperref}}
\usepackage{{cite}}
\usepackage{{amsmath}}
\usepackage{{microtype}}

\title{{{title}}}
\author{{{author}}}
\date{{}}

\begin{{document}}
\maketitle
"#,
        title = title.replace("&", "\\&"),
        author = author.replace("&", "\\&"),
    )
}

pub fn get_postamble() -> &'static str {
    r#"\end{document}"#
}
