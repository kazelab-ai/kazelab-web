//! Dynamic Symbol Demangling and Itanium / Rust v0 ABI Name Resolver.
//! Maps low-level compiled symbol names back to AST namespaces and traits.

pub struct SymbolDemangler;

impl SymbolDemangler {
    /// Demangles Rust v0 or legacy symbols into clean AST paths.
    pub fn demangle(symbol: &str) -> String {
        if symbol.starts_with("_RNv") || symbol.starts_with("_R") {
            // Rust v0 symbol scheme simplified prefix trimmer
            Self::demangle_rust_v0(symbol)
        } else if symbol.starts_with("_ZN") {
            // Itanium C++ / Legacy Rust ABI scheme
            Self::demangle_itanium(symbol)
        } else {
            symbol.to_string()
        }
    }

    fn demangle_itanium(symbol: &str) -> String {
        // Strip _ZN prefix and terminal 'E'
        let trimmed = symbol.trim_start_matches("_ZN").trim_end_matches('E');
        let mut segments = Vec::new();
        let mut chars = trimmed.chars().peekable();

        while chars.peek().is_some() {
            let mut num_str = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() {
                    num_str.push(c);
                    chars.next();
                } else {
                    break;
                }
            }

            if let Ok(len) = num_str.parse::<usize>() {
                let mut segment = String::with_capacity(len);
                for _ in 0..len {
                    if let Some(c) = chars.next() {
                        segment.push(c);
                    }
                }
                segments.push(segment);
            } else {
                break;
            }
        }

        if segments.is_empty() {
            symbol.to_string()
        } else {
            segments.join("::")
        }
    }

    fn demangle_rust_v0(symbol: &str) -> String {
        // Basic parser for rust v0 identifiers
        let mut out = String::new();
        let mut chars = symbol.chars().skip(3); // skip _RN / _R

        while let Some(c) = chars.next() {
            if c.is_alphanumeric() || c == '_' {
                out.push(c);
            }
        }

        if out.is_empty() {
            symbol.to_string()
        } else {
            out
        }
    }
}
