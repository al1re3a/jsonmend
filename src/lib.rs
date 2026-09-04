#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairResult {
    pub output: String,
    pub changes: Vec<&'static str>,
}

pub fn repair(input: &str) -> RepairResult {
    let mut changes = Vec::new();
    let mut value = strip_fence(input, &mut changes);

    let next = strip_comments(&value);
    record_change(&value, &next, "removed comments", &mut changes);
    value = next;

    let next = normalize_single_quotes(&value);
    record_change(
        &value,
        &next,
        "converted single-quoted strings",
        &mut changes,
    );
    value = next;

    let next = quote_keys(&value);
    record_change(&value, &next, "quoted unquoted object keys", &mut changes);
    value = next;

    let next = close_containers(&value);
    record_change(
        &value,
        &next,
        "closed unterminated containers",
        &mut changes,
    );
    value = next;

    let next = remove_trailing_commas(&value);
    record_change(&value, &next, "removed trailing commas", &mut changes);

    RepairResult {
        output: next.trim().to_string(),
        changes,
    }
}

fn record_change(before: &str, after: &str, label: &'static str, changes: &mut Vec<&'static str>) {
    if before != after {
        changes.push(label);
    }
}

fn strip_fence(input: &str, changes: &mut Vec<&'static str>) -> String {
    let trimmed = input.trim();
    if !trimmed.starts_with("```") {
        return trimmed.to_string();
    }
    let Some(first_newline) = trimmed.find('\n') else {
        return trimmed.to_string();
    };
    if let Some(end) = trimmed.rfind("```") {
        if end > first_newline {
            changes.push("removed Markdown fence");
            return trimmed[first_newline + 1..end].trim().to_string();
        }
    }
    trimmed.to_string()
}

fn strip_comments(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut output = String::with_capacity(input.len());
    let mut i = 0;
    let mut quote = None;
    let mut escaped = false;
    while i < chars.len() {
        let c = chars[i];
        if let Some(q) = quote {
            output.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        if c == '"' || c == '\'' {
            quote = Some(c);
            output.push(c);
            i += 1;
        } else if c == '/' && chars.get(i + 1) == Some(&'/') {
            i += 2;
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
        } else if c == '/' && chars.get(i + 1) == Some(&'*') {
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i = (i + 2).min(chars.len());
        } else {
            output.push(c);
            i += 1;
        }
    }
    output
}

fn normalize_single_quotes(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut output = String::with_capacity(input.len());
    let mut i = 0;
    let mut in_double = false;
    let mut escaped = false;
    while i < chars.len() {
        let c = chars[i];
        if in_double {
            output.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_double = false;
            }
            i += 1;
        } else if c == '"' {
            in_double = true;
            output.push(c);
            i += 1;
        } else if c == '\'' {
            output.push('"');
            i += 1;
            while i < chars.len() {
                let current = chars[i];
                if current == '\\' && chars.get(i + 1) == Some(&'\'') {
                    output.push('\'');
                    i += 2;
                } else if current == '"' {
                    output.push_str("\\\"");
                    i += 1;
                } else if current == '\'' {
                    output.push('"');
                    i += 1;
                    break;
                } else {
                    output.push(current);
                    i += 1;
                }
            }
        } else {
            output.push(c);
            i += 1;
        }
    }
    output
}

fn quote_keys(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut output = String::with_capacity(input.len());
    let mut i = 0;
    let mut in_string = false;
    let mut escaped = false;
    let mut expect_key = false;
    while i < chars.len() {
        let c = chars[i];
        if in_string {
            output.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match c {
            '"' => {
                in_string = true;
                output.push(c);
                i += 1;
            }
            '{' | ',' => {
                expect_key = true;
                output.push(c);
                i += 1;
            }
            '}' => {
                expect_key = false;
                output.push(c);
                i += 1;
            }
            c if expect_key && (c.is_ascii_alphabetic() || c == '_') => {
                let start = i;
                i += 1;
                while i < chars.len()
                    && (chars[i].is_ascii_alphanumeric() || chars[i] == '_' || chars[i] == '-')
                {
                    i += 1;
                }
                let mut probe = i;
                while probe < chars.len() && chars[probe].is_whitespace() {
                    probe += 1;
                }
                if chars.get(probe) == Some(&':') {
                    output.push('"');
                    output.extend(chars[start..i].iter());
                    output.push('"');
                } else {
                    output.extend(chars[start..i].iter());
                }
                expect_key = false;
            }
            ':' => {
                expect_key = false;
                output.push(c);
                i += 1;
            }
            c if c.is_whitespace() => {
                output.push(c);
                i += 1;
            }
            _ => {
                expect_key = false;
                output.push(c);
                i += 1;
            }
        }
    }
    output
}

fn remove_trailing_commas(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut output = String::with_capacity(input.len());
    let mut in_string = false;
    let mut escaped = false;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if in_string {
            output.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            i += 1;
        } else if c == '"' {
            in_string = true;
            output.push(c);
            i += 1;
        } else if c == ',' {
            let mut probe = i + 1;
            while probe < chars.len() && chars[probe].is_whitespace() {
                probe += 1;
            }
            if matches!(chars.get(probe), Some('}') | Some(']')) {
                i += 1;
            } else {
                output.push(c);
                i += 1;
            }
        } else {
            output.push(c);
            i += 1;
        }
    }
    output
}

fn close_containers(input: &str) -> String {
    let mut stack = Vec::new();
    let mut in_string = false;
    let mut escaped = false;
    for c in input.chars() {
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
        } else if c == '"' {
            in_string = true;
        } else if c == '{' {
            stack.push('}');
        } else if c == '[' {
            stack.push(']');
        } else if matches!(c, '}' | ']') && stack.last() == Some(&c) {
            stack.pop();
        }
    }
    let mut output = input.to_string();
    while let Some(c) = stack.pop() {
        output.push(c);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_fences_and_trailing_commas() {
        let result = repair("```json\n{\"ok\": true,}\n```");
        assert_eq!(result.output, "{\"ok\": true}");
        assert_eq!(result.changes.len(), 2);
    }

    #[test]
    fn repairs_typical_model_output() {
        let result = repair("{name: 'Ada', // note\nitems: [1, 2,]");
        assert_eq!(result.output, "{\"name\": \"Ada\", \n\"items\": [1, 2]}");
    }

    #[test]
    fn preserves_comment_markers_inside_strings() {
        let result = repair(r#"{"url":"https://example.test/a/*b*/"}"#);
        assert_eq!(result.output, r#"{"url":"https://example.test/a/*b*/"}"#);
        assert!(result.changes.is_empty());
    }

    #[test]
    fn escapes_double_quotes_in_single_strings() {
        assert_eq!(
            repair("{'say': 'hello \"world\"'}").output,
            "{\"say\": \"hello \\\"world\\\"\"}"
        );
    }
}
