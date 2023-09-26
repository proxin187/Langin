use lib_lexin::{Lexer, Section, Token};

pub fn lex(file: &str) -> Result<Vec<Token>, Box<dyn std::error::Error>> {
    let mut lexer = Lexer::new(
        &[
            "return",
            "let",
            "if",
            "else",
            "while",
            "include",
            "asm",

            // Types
            "int",
            "ptr",
        ],
        &[
            Section::new(
                "comment",
                "#",
                "#"
            ),
            Section::new(
                "string",
                "\"",
                "\""
            ),
        ],
        &[
            (',', "Comma"),
            (':', "Colon"),
            (';', "SemiColon"),
            ('{', "OpenBrace"),
            ('}', "CloseBrace"),
            ('(', "OpenParen"),
            (')', "CloseParen"),
            ('[', "OpenBracket"),
            (']', "CloseBracket"),

            ('-', "Minus"),
            ('+', "Plus"),
            ('*', "Asterisk"),
            ('/', "Slash"),
            ('!', "Bang"),
            ('&', "And"),
            ('=', "Equal"),

            ('>', "BThen"),
            ('<', "SThen"),
        ],
    );

    lexer.load_file(file)?;

    return lexer.tokenize();
}


